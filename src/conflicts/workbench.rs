use gpui::{ Context, Focusable, IntoElement, Window, div, prelude::*, px};
use std::rc::Rc;
use gpui_kit::base::{h_resizable, v_resizable, resizable_panel};

use gpui_kit::component::{Disableable as _, Sizable as _, button::Button,
    menu::{DropdownMenu as _, PopupMenuItem}};
use khaslana::{ConflictBlockResolution, ConflictFileKind, ConflictFileView, ConflictResolutionSide};

use super::{ConflictDocumentPane, conflict_byte_range_to_lines, conflict_document_byte_range};
use crate::{RepositoryView, ui::{components::floating_panel, theme}};
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
        floating_panel().flex().flex_col().flex_1().min_w(px(0.0)).min_h(px(0.0)).h_full()
            .child(div().flex().flex_1().min_h(px(0.0)).min_w(px(0.0))
                .when(rail, |this| this.child(div().w(px(224.0)).flex_none().h_full()
                    .child(self.render_conflict_file_rail(cx))))
                .child(div().flex().flex_col().flex_1().min_w(px(0.0)).min_h(px(0.0))
                    .child(self.render_conflict_detail(window, cx))))
            .into_any_element()
    }

    pub(super) fn render_approved_conflict_header(&self, title: String, view: Option<&ConflictFileView>, cx: &mut Context<Self>) -> gpui::AnyElement {
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
        let entity = cx.entity().downgrade();
        let sync = !self.conflict_workbench.independent_scroll;
        div().flex_none().flex().flex_col().bg(rgb(theme::SURFACE_BASE))
            .child(div().flex().items_center().gap_2().px_3().h(px(58.0))
                .child(div().flex_1().min_w(px(0.0)).flex().flex_col().gap_1()
                    .child(div().truncate().text_size(px(17.0)).font_weight(gpui::FontWeight::SEMIBOLD).text_color(rgb(theme::CONTENT_PRIMARY)).child("代码合并"))
                    .child(div().truncate().text_size(px(11.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child(title)))
                .child(div().text_size(px(12.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child(format!("块 {} / {count}", if count==0 {0} else {selected+1})))
                .child(Button::new("conflict-prev").label("↑").small().accessibility_label("上一个冲突块").tooltip("上一个冲突块").disabled(count==0)
                    .on_click(cx.listener(|this,_,_,cx| { this.step_conflict_block(-1); cx.notify(); })))
                .child(Button::new("conflict-next").label("↓").small().accessibility_label("下一个冲突块").tooltip("下一个冲突块").disabled(count==0)
                    .on_click(cx.listener(|this,_,_,cx| { this.step_conflict_block(1); cx.notify(); }))))
            .child(div().flex().items_center().gap_1().px_3().pb_2()
                .child(Button::new("conflict-files-menu").label("文件").small().dropdown_menu(move |mut menu,_,_| {
                    for item in &file_items { menu=menu.item(item()); } menu
                }))
                .child(Button::new("conflict-scroll-link").label(if sync {"同步滚动"} else {"独立滚动"}).small().disabled(!text)
                    .on_click(move |_,_,cx| { let _=entity.update(cx, |this,cx| {this.conflict_workbench.independent_scroll=!this.conflict_workbench.independent_scroll;this.conflict_pane_scroll_sync.borrow_mut().take();cx.notify();}); }))
                .child(Button::new("conflict-both-menu").label("接受两边").small().disabled(!enabled)
                    .dropdown_menu(move |menu,_,_| menu.item(both[0]()).item(both[1]())))
                .child(div().flex_1())
                .child(self.button(if self.ai_conflict_loading {"AI 生成中…"} else if !self.ai_settings.is_usable() {"AI 未配置"} else {"AI 合并建议"}, text && self.ai_conflict_merge_button_enabled(), |this,_,_| this.generate_ai_conflict_merge(),cx))
                .child(Button::new("conflict-more-menu").label("更多").small().dropdown_menu(move |mut menu,_,_| {
                    for item in &more {menu=menu.item(item());} menu
                })))
            .into_any_element()
    }

    pub(super) fn can_finish_conflict_merge(&self) -> bool {
        crate::merge_view::merge_can_finish(self.merge_in_progress(), self.snapshot.as_ref().map_or(0,|s|s.conflicts.len()), self.busy, &self.commit_message.value)
    }

    pub(super) fn dispatch_conflict_history(&mut self, redo: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(host)=self.selected_conflict_editor() else {return};
        window.focus(&host.states[1].read(cx).focus_handle(cx), cx);
        if redo {window.dispatch_action(Box::new(gpui_kit::base::input::Redo),cx)}
        else {window.dispatch_action(Box::new(gpui_kit::base::input::Undo),cx)}
    }

    pub(super) fn render_approved_text_conflict(&self, view: &ConflictFileView, cx: &mut Context<Self>) -> gpui::AnyElement {
        let selected=self.conflict_workbench.selected_block.min(view.blocks.len().saturating_sub(1));
        let columns=div().relative().flex().flex_1().min_w(px(0.0)).min_h(px(0.0))
            .child(h_resizable("conflict-three-way")
                .child(resizable_panel().size_range(px(120.0)..gpui::Pixels::MAX).child(
                    div().flex().size_full().min_w(px(0.0)).min_h(px(0.0))
                        .child(self.render_conflict_document_pane("当前版本","conflict-ours-scroll",crate::CONFLICT_OURS_SCROLL_HANDLE_ID,&view.ours_text,ConflictDocumentPane::Ours,view,selected,Vec::new(),cx))
                        .child(self.render_conflict_action_gutter(ConflictDocumentPane::Ours,view,cx))))
                .child(resizable_panel().size_range(px(120.0)..gpui::Pixels::MAX)
                    .child(self.render_conflict_document_pane("合并结果","conflict-result-scroll",crate::CONFLICT_RESULT_SCROLL_HANDLE_ID,&view.draft,ConflictDocumentPane::Result,view,selected,Vec::new(),cx)))
                .child(resizable_panel().size_range(px(120.0)..gpui::Pixels::MAX).child(
                    div().flex().size_full().min_w(px(0.0)).min_h(px(0.0))
                        .child(self.render_conflict_action_gutter(ConflictDocumentPane::Theirs,view,cx))
                        .child(self.render_conflict_document_pane("传入版本","conflict-theirs-scroll",crate::CONFLICT_THEIRS_SCROLL_HANDLE_ID,&view.theirs_text,ConflictDocumentPane::Theirs,view,selected,Vec::new(),cx)))))
            .child(self.render_conflict_connectors(view,selected));
        let document=if self.conflict_workbench.show_base {
            let base=div().size_full().min_h(px(0.0)).flex().flex_col().border_t_1().border_color(rgb(theme::BORDER_MUTED))
                .child(div().px_3().py_1().text_size(px(12.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child("共同祖先 · 当前块 · 只读"))
                .child(self.render_conflict_plain_text("conflict-base-scroll",view.blocks.get(selected).and_then(|b|b.base.as_deref()).unwrap_or(""),cx));
            v_resizable("conflict-base-height")
                .child(resizable_panel().size_range(px(100.0)..gpui::Pixels::MAX).child(columns))
                .child(resizable_panel().size(px(120.0)).size_range(px(64.0)..px(300.0)).child(base)).into_any_element()
        } else { columns.into_any_element() };
        let content=div().flex().flex_col().flex_1().min_w(px(0.0)).min_h(px(0.0)).child(document);
        content.child(div().flex_none().flex().items_center().gap_2().px_3().py_2().border_t_1().border_color(rgb(theme::BORDER_MUTED))
            .child(div().flex_1().min_w(px(0.0)).flex().flex_col().gap_1()
                .child(div().truncate().text_size(px(12.0)).text_color(rgb(theme::CONTENT_PRIMARY)).child(format!("已处理 {} / {} · 未处理 {}",view.handled_block_count(),view.blocks.len(),view.unresolved_block_count())))
                .child(div().truncate().text_size(px(10.0)).text_color(rgb(theme::CONTENT_SECONDARY)).child("结果可编辑 · Ctrl+Z 撤销 · 未处理块仍需确认")))
            .child(self.button("应用到工作区",!self.busy,|this,_,_|this.apply_selected_conflict_draft(false),cx))
            .child(self.primary_button("应用并标记已解决",!self.busy,|this,_,_|this.apply_selected_conflict_draft(true),cx)))
            .into_any_element()
    }

    fn render_conflict_action_gutter(&self, pane: ConflictDocumentPane, view: &ConflictFileView, cx: &mut Context<Self>) -> gpui::AnyElement {
        let index=if pane==ConflictDocumentPane::Ours {0} else {2};
        let host=self.selected_conflict_editor();
        let state=host.map(|h|h.states[index].read(cx));
        let origin=state.and_then(super::editor::editor_row_origin);
        let bounds=host.and_then(|h|*h.viewports[index].borrow());
        let mut gutter=div().relative().w(px(26.0)).flex_none().min_h(px(0.0)).overflow_hidden().bg(rgb(theme::SURFACE_CANVAS));
        if let (Some((origin,height)),Some(bounds))=(origin,bounds) {
            let text=if index==0 {&view.ours_text} else {&view.theirs_text};
            for (block_index,block) in view.blocks.iter().enumerate() {
                let (start,end)=conflict_document_byte_range(block,pane);
                let range=conflict_byte_range_to_lines(text,start,end);
                let y=origin+height*range.start as f32-f32::from(bounds.top());
                let bottom=origin+height*range.end as f32-f32::from(bounds.top());
                if bottom<0.0||y>f32::from(bounds.size.height){continue}
                let title=if index==0 {"接受当前版本的此块"} else {"接受传入版本的此块"};
                let path = view.path.clone();
                let session = self.conflict_workbench.session_id;
                let tab = self.active_tab_id();
                gutter=gutter.child(Button::new(format!("accept-{}-{}-{}-{block_index}",self.conflict_workbench.session_id,view.path,index)).label(if index==0 {"→"} else {"←"})
                    .accessibility_label(title).tooltip(title).xsmall().disabled(self.busy).absolute().top(px(y.max(0.0)+49.0)).left(px(1.0)).w(px(24.0))
                    .on_click(cx.listener(move |this,_,_,cx| {
                        if this.busy || this.active_tab_id() != tab || this.conflict_workbench.session_id != session || this.conflict_workbench.selected_path.as_ref() != Some(&path) { return }
                        this.select_conflict_block(block_index);this.apply_selected_conflict_resolution(if index==0 {ConflictBlockResolution::Ours} else {ConflictBlockResolution::Theirs});cx.notify(); })));
            }
        }
        gutter.into_any_element()
    }
}
