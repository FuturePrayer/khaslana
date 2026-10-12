use gpui::{ Context, Focusable, IntoElement, Window, div, prelude::*, px};
use std::rc::Rc;
use gpui_kit::base::{v_resizable, resizable_panel};

use gpui_kit::component::{Icon,
    menu::{DropdownMenu as _, PopupMenuItem}};
use gpui_kit::assets::IconName;
use khaslana::{ConflictBlockResolution, ConflictFileKind, ConflictFileView, ConflictResolutionSide};

use crate::{RepositoryView, ui::{components::{floating_panel, settings_command_button, SettingsButtonTone, workbench_command_button}, theme}};
use crate::ui::theme::rgb;

pub(super) fn conflict_rail_visible(width: f32) -> bool {
    width >= 1100.0
}

fn files_are_resolved(snapshot: Option<&khaslana::RepositorySnapshot>) -> bool {
    snapshot.is_some_and(|s| s.conflicts.is_empty())
}

#[cfg(test)]
#[path = "../tests/conflict_workbench.rs"]
mod tests;

impl RepositoryView {
    pub(super) fn render_conflict_file_actions(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let external = !self.busy && self.conflict_workbench.selected_path.as_ref()
            .and_then(|path| self.conflict_workbench.files.get(path))
            .is_some_and(|view| view.kind != ConflictFileKind::WorktreeOnly);
        div().flex().flex_wrap().items_center().justify_end().gap_2()
            .child(settings_command_button("conflict-file-idea", "用 IDEA 解决", SettingsButtonTone::Primary, external, cx)
                .min_w(px(0.0)).px(px(10.0)).rounded(px(6.0))
                .on_click(cx.listener(|this,_,_,cx| {this.resolve_selected_conflict_with_intellij_idea();cx.notify();})))
            .when(self.merge_in_progress(), |row| row
                .child(workbench_command_button("conflict-file-finish", "完成合并", false, self.can_finish_conflict_merge(), cx)
                    .tooltip(if self.busy {"当前操作运行中"} else if !files_are_resolved(self.snapshot.as_ref()) {"先解决全部冲突文件"} else if self.commit_message.value.trim().is_empty() {"请先在工作区填写合并提交说明"} else {"完成合并"})
                    .on_click(cx.listener(|this,_,_,cx| {this.finish_merge();cx.notify();}))))
            .into_any_element()
    }

    /// 当前文件与仓库身份随菜单一起捕获，菜单打开后切仓库不能执行旧命令。
    fn conflict_menu_item(
        &self, label: &str, enabled: bool,
        action: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &Context<Self>,
    ) -> Rc<dyn Fn() -> PopupMenuItem> {
        let entity = cx.entity().downgrade();
        let tab = self.active_tab_id();
        let session = self.conflict_workbench.session_id;
        let path = self.conflict_workbench.selected_path.clone();
        let label = label.to_owned();
        let action = Rc::new(action);
        Rc::new(move || {
            let entity = entity.clone();
            let path = path.clone();
            let action = action.clone();
            PopupMenuItem::new(label.clone()).disabled(!enabled).on_click(move |_, window, cx| {
            let _ = entity.update(cx, |this, cx| {
                if this.active_tab_id() != tab || this.conflict_workbench.session_id != session || this.conflict_workbench.selected_path != path || this.busy { return }
                action(this, window, cx);
                cx.notify();
            });
        })
        })
    }

    pub(super) fn render_approved_conflict_workbench(&self, window: &Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let rail = conflict_rail_visible(f32::from(window.viewport_size().width));
        div().flex().flex_1().min_w(px(0.0)).min_h(px(0.0)).h_full()
            .child(div().flex().gap_3().flex_1().min_h(px(0.0)).min_w(px(0.0))
                .when(rail, |this| this.child(div().w(px(224.0)).flex_none().h_full()
                    .child(self.render_conflict_file_rail(cx))))
                .child(floating_panel().flex().flex_col().flex_1().min_w(px(0.0)).min_h(px(0.0))
                    .child(self.render_conflict_detail(window, cx))))
            .into_any_element()
    }

    pub(super) fn render_approved_conflict_header(&self, title: String, view: Option<&ConflictFileView>, window: &Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let count = view.map_or(0, |v| v.blocks.len());
        let selected = self.conflict_workbench.selected_block.min(count.saturating_sub(1));
        let text = view.is_some_and(|v| v.kind == ConflictFileKind::Text);
        let enabled = text && !self.busy;
        let files = self.snapshot.as_ref().map(|v| v.conflicts.clone()).unwrap_or_default();
        let file_items: Vec<_> = files.into_iter().map(|path| {
            let name = path.clone();
            self.conflict_menu_item(&name, true, move |this,_,_| this.select_conflict_file(path.clone()), cx)
        }).collect();
        let both = [
            self.conflict_menu_item("接受两边：当前在前", enabled, |this,_,_| this.apply_selected_conflict_resolution(ConflictBlockResolution::BothOursFirst), cx),
            self.conflict_menu_item("接受两边：传入在前", enabled, |this,_,_| this.apply_selected_conflict_resolution(ConflictBlockResolution::BothTheirsFirst), cx),
        ];
        let mut more = vec![
            self.conflict_menu_item("显示 / 隐藏共同祖先", text, |this,_,_| this.conflict_workbench.show_base = !this.conflict_workbench.show_base, cx),
            self.conflict_menu_item("忽略该块，保留现有结果", enabled, |this,_,_| this.ignore_selected_conflict_block(), cx),
            self.conflict_menu_item("应用到工作区", enabled, |this,_,_| this.apply_selected_conflict_draft(false), cx),
            self.conflict_menu_item("用 IntelliJ IDEA 解决", view.is_some_and(|v| v.kind != ConflictFileKind::WorktreeOnly) && !self.busy,
                |this,_,_| this.resolve_selected_conflict_with_intellij_idea(), cx),
            self.conflict_menu_item("撤销结果编辑", enabled, |this,window,cx| this.dispatch_conflict_history(false,window,cx), cx),
            self.conflict_menu_item("重做结果编辑", enabled, |this,window,cx| this.dispatch_conflict_history(true,window,cx), cx),
        ];
        if let Some(path) = self.conflict_workbench.selected_path.clone()
            && self.conflict_workbench.ai_suggestions.contains_key(&path) {
            more.push(self.conflict_menu_item("复制未回填的 AI 建议", true, move |this,_,cx| {
                if let Some(text) = this.conflict_workbench.ai_suggestions.get(&path) {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text.clone()));
                    this.status = "已复制 AI 建议，请检查后粘贴到合并结果".into();
                }
            }, cx));
        }
        if let Some(path) = self.conflict_workbench.selected_path.clone().filter(|_| view.is_some_and(|v| v.kind != ConflictFileKind::WorktreeOnly)) {
            for (label,side) in [("整文件接受当前",ConflictResolutionSide::Ours),("整文件接受传入",ConflictResolutionSide::Theirs)] {
                let path = path.clone();
                more.push(self.conflict_menu_item(label, !self.busy, move |this,_,_| this.resolve_conflict_with_side(path.clone(), side), cx));
            }
        }
        if self.merge_in_progress() {
            more.push(self.conflict_menu_item("完成合并", self.can_finish_conflict_merge(), |this,_,_| this.finish_merge(), cx));
            more.push(self.conflict_menu_item("中止合并", !self.busy, |this,_,_| this.open_abort_merge_confirm_dialog(), cx));
        }
        if self.snapshot.as_ref().is_some_and(|s| s.rebase_in_progress) {
            more.push(self.conflict_menu_item("继续变基", !self.busy && files_are_resolved(self.snapshot.as_ref()), |this,_,_| this.rebase_continue(), cx));
            more.push(self.conflict_menu_item("跳过当前变基提交", !self.busy, |this,_,_| this.rebase_skip(), cx));
            more.push(self.conflict_menu_item("中止变基", !self.busy, |this,_,_| this.rebase_abort(), cx));
        }
        let filename = title.rsplit('/').next().unwrap_or(&title).to_owned();
        let wide = f32::from(window.viewport_size().width) >= 1400.0;
        div().flex_none().flex().flex_col().bg(rgb(theme::SURFACE_BASE)).rounded_t(px(theme::RADIUS_PANEL))
            .child(div().flex().items_center().gap_3().px_4().h(px(64.0))
                .child(Icon::new(IconName::FileCode).size(px(20.0)).text_color(rgb(theme::PRIMARY)))
                .child(div().flex_1().min_w(px(0.0)).flex().flex_col().gap_1()
                    .child(div().truncate().text_size(px(17.0)).font_weight(gpui::FontWeight::SEMIBOLD).text_color(rgb(theme::CONTENT_PRIMARY)).child(format!("代码合并 / {filename}")))
                    .child(div().truncate().text_size(px(11.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child(title)))
                .child(div().px_2().py_1().rounded(px(6.0)).bg(rgb(theme::CONFLICT_EDITOR_BG)).text_size(px(12.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child(format!("冲突 {} / {count}", if count==0 {0} else {selected+1})))
                .child(workbench_command_button("conflict-prev", "", false, count!=0, cx).icon(Icon::new(IconName::ArrowUp).size(px(14.0))).w(px(32.0)).px_0().accessibility_label("上一个冲突块").tooltip("上一个冲突块")
                    .on_click(cx.listener(|this,_,_,cx| { this.step_conflict_block(-1); cx.notify(); })))
                .child(workbench_command_button("conflict-next", "", false, count!=0, cx).icon(Icon::new(IconName::ArrowDown).size(px(14.0))).w(px(32.0)).px_0().accessibility_label("下一个冲突块").tooltip("下一个冲突块")
                    .on_click(cx.listener(|this,_,_,cx| { this.step_conflict_block(1); cx.notify(); }))))
            .child(div().flex().flex_wrap().items_center().gap_2().px_4().py_2().min_h(px(48.0)).bg(rgb(theme::CONFLICT_EDITOR_BG))
                .child(workbench_command_button("conflict-files-menu", "文件", false, true, cx).icon(Icon::new(IconName::PanelLeft).size(px(14.0))).dropdown_menu(move |mut menu,_,_| {
                    for item in &file_items { menu=menu.item(item()); } menu
                }))
                .child(div().px_2().text_size(px(12.0)).text_color(rgb(theme::PRIMARY)).child("单文档 · 候选块"))
                .child(workbench_command_button("conflict-both-menu", "接受两边", false, enabled, cx).dropdown_caret(true)
                    .dropdown_menu(move |menu,_,_| menu.item(both[0]()).item(both[1]())))
                .when(wide, |row| row
                    .child(workbench_command_button("conflict-base", "共同祖先", self.conflict_workbench.show_base, text, cx).icon(Icon::new(IconName::GitBranch).size(px(14.0)))
                        .on_click(cx.listener(|this,_,_,cx| {this.conflict_workbench.show_base=!this.conflict_workbench.show_base;cx.notify();})))
                    .child(workbench_command_button("conflict-ignore", "忽略该块", false, enabled, cx)
                        .on_click(cx.listener(|this,_,_,cx| {this.ignore_selected_conflict_block();cx.notify();})))
                    .child(workbench_command_button("conflict-undo", "", false, enabled, cx).icon(Icon::new(IconName::Undo2).size(px(14.0))).w(px(32.0)).px_0().accessibility_label("撤销结果编辑").tooltip("撤销结果编辑 · Ctrl+Z")
                    .on_click(cx.listener(|this,_,window,cx| this.dispatch_conflict_history(false, window, cx))))
                .child(workbench_command_button("conflict-redo", "", false, enabled, cx).icon(Icon::new(IconName::Redo2).size(px(14.0))).w(px(32.0)).px_0().accessibility_label("重做结果编辑").tooltip("重做结果编辑 · Ctrl+Y")
                    .on_click(cx.listener(|this,_,window,cx| this.dispatch_conflict_history(true, window, cx)))))
                .child(div().flex_1())
                .child(workbench_command_button("conflict-ai", if self.ai_conflict_loading {"AI 生成中…"} else if !self.ai_settings.is_usable() {"AI 未配置"} else {"AI 合并建议"}, false, text && self.ai_conflict_merge_button_enabled(), cx).icon(Icon::new(IconName::Sparkles).size(px(14.0)))
                    .on_click(cx.listener(|this,_,_,cx| {this.generate_ai_conflict_merge();cx.notify();})))
                .child(self.render_conflict_file_actions(cx))
                .child(workbench_command_button("conflict-more-menu", "更多", false, true, cx).icon(Icon::new(IconName::Ellipsis).size(px(14.0))).dropdown_menu(move |mut menu,_,_| {
                    for item in &more {menu=menu.item(item());} menu
                })))
            .into_any_element()
    }

    pub(super) fn can_finish_conflict_merge(&self) -> bool {
        crate::merge_view::merge_can_finish(self.merge_in_progress(), self.snapshot.as_ref().map_or(0,|s|s.conflicts.len()), self.busy, &self.commit_message.value)
    }

    pub(super) fn dispatch_conflict_history(&mut self, redo: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(host)=self.selected_conflict_editor() else {return};
        let state=host.state.clone();
        window.focus(&state.read(cx).focus_handle(cx), cx);
        state.update(cx, |editor,cx| editor.history(redo,cx));
    }

    pub(super) fn render_approved_text_conflict(&self, view: &ConflictFileView, cx: &mut Context<Self>) -> gpui::AnyElement {
        let selected=self.conflict_workbench.selected_block.min(view.blocks.len().saturating_sub(1));
        let columns=div().flex().flex_col().flex_1().min_w(px(0.0)).min_h(px(0.0))
            .child(div().h(px(52.0)).flex_none().px_4().flex().flex_col().justify_center().gap_1().bg(rgb(theme::PRIMARY_SUBTLE))
                .child(div().flex().justify_between().text_size(px(13.0)).text_color(rgb(theme::CONTENT_PRIMARY)).child("合并结果").child(div().text_size(px(11.0)).text_color(rgb(theme::PRIMARY)).child("可编辑")))
                .child(div().text_size(px(11.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child("UTF-8 · 候选只读 · 实际保存结果内容")))
            .child(self.render_single_conflict_editor());
        let document=if self.conflict_workbench.show_base {
            let base=div().size_full().min_h(px(0.0)).flex().flex_col().border_t_1().border_color(rgb(theme::BORDER_MUTED))
                .child(div().px_3().py_1().text_size(px(12.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child("共同祖先 · 当前块 · 只读"))
                .child(self.render_conflict_plain_text("conflict-base-scroll",view.blocks.get(selected).and_then(|b|b.base.as_deref()).unwrap_or(""),cx));
            v_resizable("conflict-base-height")
                .child(resizable_panel().size_range(px(100.0)..gpui::Pixels::MAX).child(columns))
                .child(resizable_panel().size(px(120.0)).size_range(px(64.0)..px(300.0)).child(base)).into_any_element()
        } else { columns.into_any_element() };
        let content=div().flex().flex_col().flex_1().min_w(px(0.0)).min_h(px(0.0)).child(document);
        content.child(div().flex_none().flex().items_center().gap_3().px_4().py_3().border_t_1().border_color(rgb(theme::BORDER_MUTED)).bg(rgb(theme::SURFACE_BASE)).rounded_b(px(theme::RADIUS_PANEL))
            .child(div().flex_1().min_w(px(0.0)).flex().flex_col().gap_1()
                .child(div().truncate().text_size(px(12.0)).text_color(rgb(theme::CONTENT_PRIMARY)).child(format!("已处理 {} / {} · 未处理 {}",view.handled_block_count(),view.blocks.len(),view.unresolved_block_count())))
                .child(div().truncate().text_size(px(10.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child("结果始终可编辑 · 候选不会混入保存内容 · Ctrl+Z 撤销")))
            .child(self.button("应用到工作区",!self.busy,|this,_,_|this.apply_selected_conflict_draft(false),cx))
            .child(self.primary_button("应用并标记已解决",!self.busy,|this,_,_|this.apply_selected_conflict_draft(true),cx)))
            .into_any_element()
    }

}
