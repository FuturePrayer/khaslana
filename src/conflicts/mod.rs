use std::path::Path;

use gpui::{Context, IntoElement, MouseButton, MouseDownEvent, Window, div, prelude::*, px, rgb};
use khaslana::{
    ConflictBlockResolution, ConflictFileKind, ConflictFileView, ConflictResolutionSide,
    RepositorySnapshot,
};

use crate::{
    FieldId, MainMode, RepositoryView,
    ui::{components::app_panel, theme as ui_theme},
};

pub(crate) fn conflict_status_message(label: &str, count: usize) -> String {
    let operation = label.strip_suffix("完成").unwrap_or("操作");
    format!("{operation}产生冲突，请在左侧“冲突”区域解决（{count} 个文件）")
}

fn conflict_paths(snapshot: Option<&RepositorySnapshot>) -> Vec<String> {
    snapshot
        .map(|snapshot| snapshot.conflicts.clone())
        .unwrap_or_default()
}

impl RepositoryView {
    pub(crate) fn render_conflict_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let conflicts = conflict_paths(self.snapshot.as_ref());
        let conflict_rows = conflicts
            .iter()
            .cloned()
            .map(|path| self.conflict_row(path, cx).into_any_element())
            .collect::<Vec<_>>();
        let has_conflicts = !conflict_rows.is_empty();

        div().when(has_conflicts, |this| {
            this.child(self.render_conflict_summary(conflicts.len()))
                .child(self.render_change_section(
                    "冲突",
                    "conflict-list",
                    "",
                    false,
                    conflict_rows,
                    true,
                    Vec::new(),
                    cx,
                ))
                .child(div().flex_none().h(px(1.0)).bg(rgb(ui_theme::BORDER)))
        })
    }

    pub(crate) fn render_conflict_workbench(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        app_panel()
            .flex()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .child(self.render_conflict_file_rail(cx))
            .child(self.render_column_splitter(crate::ResizeTarget::Changes, cx))
            .child(self.render_conflict_detail(window, cx))
    }

    fn render_conflict_file_rail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let conflicts = conflict_paths(self.snapshot.as_ref());
        let selected_path = self.conflict_workbench.selected_path.as_deref();
        let file_rows = conflicts
            .iter()
            .cloned()
            .map(|path| {
                self.conflict_file_row(path.clone(), selected_path == Some(path.as_str()), cx)
                    .into_any_element()
            })
            .collect::<Vec<_>>();

        app_panel()
            .flex()
            .flex_none()
            .flex_col()
            .w(px(self.changes_width))
            .min_w(px(self.changes_width))
            .h_full()
            .child(
                div()
                    .flex_none()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(ui_theme::BORDER))
                    .bg(rgb(ui_theme::WARNING_SOFT))
                    .text_size(px(12.0))
                    .text_color(rgb(ui_theme::WARNING_TEXT))
                    .child(format!("存在 {} 个冲突文件", conflicts.len())),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .overflow_hidden()
                    .children(file_rows),
            )
    }

    fn conflict_file_row(
        &self,
        path: String,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let view = self.conflict_workbench.files.get(&path);
        let badge = match view.map(|view| view.kind) {
            Some(ConflictFileKind::Text) => "文本",
            Some(ConflictFileKind::Binary) => "二进制",
            Some(ConflictFileKind::Unsupported) => "回退",
            None => "加载中",
        };
        let unresolved = view
            .map(ConflictFileView::unresolved_block_count)
            .unwrap_or_default();
        let dirty = view
            .map(|view| view.draft_status)
            .is_some_and(|status| matches!(status, khaslana::ConflictDraftStatus::Dirty));
        let applied = view
            .map(|view| view.draft_status)
            .is_some_and(|status| matches!(status, khaslana::ConflictDraftStatus::Applied));
        let path_for_select = path.clone();

        crate::ui::components::list_row_surface(format!("conflict-workbench-{path}"), selected)
            .flex()
            .flex_none()
            .flex_col()
            .gap_1()
            .px_2()
            .py_2()
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _event: &MouseDownEvent, window, cx| {
                    window.focus(&this.conflict_editor.focus);
                    this.main_mode = MainMode::Conflict;
                    this.select_conflict_file(path_for_select.clone());
                    cx.notify();
                }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .min_w(px(0.0))
                            .text_size(px(12.0))
                            .text_color(rgb(ui_theme::TEXT))
                            .truncate()
                            .child(path),
                    )
                    .child(
                        div()
                            .flex_none()
                            .px_1()
                            .py(px(2.0))
                            .rounded_sm()
                            .bg(rgb(ui_theme::ACCENT_SOFT))
                            .text_size(px(10.0))
                            .text_color(rgb(ui_theme::ACCENT_STRONG))
                            .child(badge),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(10.0))
                    .text_color(rgb(ui_theme::TEXT_MUTED))
                    .child(format!("未处理 {unresolved}"))
                    .when(dirty, |this| this.child("草稿已修改"))
                    .when(applied, |this| this.child("已应用")),
            )
    }

    fn render_conflict_detail(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_path = self.conflict_workbench.selected_path.clone();
        let selected_view = selected_path
            .as_ref()
            .and_then(|path| self.conflict_workbench.files.get(path));
        let title = selected_path
            .clone()
            .unwrap_or_else(|| "请选择一个冲突文件".to_string());

        app_panel()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .child(self.render_conflict_header(title, selected_view, cx))
            .child(match selected_view {
                Some(view) if view.kind == ConflictFileKind::Text => self
                    .render_text_conflict_detail(window, view, cx)
                    .into_any_element(),
                Some(view) => self
                    .render_fallback_conflict_detail(view, cx)
                    .into_any_element(),
                None => div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(ui_theme::TEXT_MUTED))
                    .child("请选择一个冲突文件")
                    .into_any_element(),
            })
    }

    fn render_conflict_header(
        &self,
        title: String,
        view: Option<&ConflictFileView>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let block_count = view.map(|view| view.blocks.len()).unwrap_or_default();
        let selected_block = self
            .conflict_workbench
            .selected_block
            .min(block_count.saturating_sub(1));
        let progress = if block_count == 0 {
            "无文本冲突块".to_string()
        } else {
            format!(
                "块 {}/{}，未处理 {}",
                selected_block + 1,
                block_count,
                view.map(ConflictFileView::unresolved_block_count)
                    .unwrap_or_default()
            )
        };

        div()
            .flex_none()
            .flex()
            .flex_col()
            .gap_2()
            .px_3()
            .py_3()
            .border_b_1()
            .border_color(rgb(ui_theme::BORDER_MUTED))
            .bg(rgb(ui_theme::HEADER_BG))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .min_w(px(0.0))
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(ui_theme::ACCENT_STRONG))
                            .truncate()
                            .child(title),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(ui_theme::TEXT_MUTED))
                            .child(progress),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .flex_wrap()
                    .child(self.button(
                        "上一个块",
                        block_count > 0,
                        |this, _, _| this.step_conflict_block(-1),
                        cx,
                    ))
                    .child(self.button(
                        "下一个块",
                        block_count > 0,
                        |this, _, _| this.step_conflict_block(1),
                        cx,
                    ))
                    .child(self.button(
                        if self.conflict_workbench.show_base {
                            "隐藏 Base"
                        } else {
                            "显示 Base"
                        },
                        block_count > 0,
                        |this, _, _| {
                            this.conflict_workbench.show_base = !this.conflict_workbench.show_base
                        },
                        cx,
                    ))
                    .child(self.button(
                        "应用到工作区",
                        view.is_some_and(|view| view.kind == ConflictFileKind::Text) && !self.busy,
                        |this, _, _| this.apply_selected_conflict_draft(false),
                        cx,
                    ))
                    .child(self.primary_button(
                        "应用并标记已解决",
                        view.is_some_and(|view| view.kind == ConflictFileKind::Text) && !self.busy,
                        |this, _, _| this.apply_selected_conflict_draft(true),
                        cx,
                    )),
            )
    }

    fn render_text_conflict_detail(
        &self,
        window: &Window,
        view: &ConflictFileView,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let block = view.blocks.get(
            self.conflict_workbench
                .selected_block
                .min(view.blocks.len().saturating_sub(1)),
        );
        let warning = (view.has_manual_blocks() || view.unresolved_block_count() > 0).then(|| {
            "仍有未显式接受或手工改写的冲突块；你仍然可以直接应用并标记解决。".to_string()
        });

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .when_some(warning, |this, warning| {
                this.child(
                    div()
                        .mx_3()
                        .mt_3()
                        .px_3()
                        .py_2()
                        .rounded_sm()
                        .bg(rgb(ui_theme::WARNING_SOFT))
                        .text_size(px(11.0))
                        .text_color(rgb(ui_theme::WARNING_TEXT))
                        .child(warning),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .gap_2()
                    .p_3()
                    .child(self.render_conflict_side_pane(
                        "当前版本",
                        block.map(|block| block.ours.as_str()).unwrap_or(""),
                        vec![
                            self.button(
                                "接受当前",
                                !self.busy,
                                |this, _, _| {
                                    this.apply_selected_conflict_resolution(
                                        ConflictBlockResolution::Ours,
                                    )
                                },
                                cx,
                            )
                            .into_any_element(),
                            self.button(
                                "接受两边（当前在前）",
                                !self.busy,
                                |this, _, _| {
                                    this.apply_selected_conflict_resolution(
                                        ConflictBlockResolution::BothOursFirst,
                                    )
                                },
                                cx,
                            )
                            .into_any_element(),
                        ],
                    ))
                    .child(
                        app_panel()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w(px(0.0))
                            .h_full()
                            .child(
                                div()
                                    .flex_none()
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgb(ui_theme::BORDER_MUTED))
                                    .bg(rgb(ui_theme::HEADER_BG))
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(ui_theme::ACCENT_STRONG))
                                    .child("结果区"),
                            )
                            .child(self.input(FieldId::ConflictEditor, false, window, cx)),
                    )
                    .child(self.render_conflict_side_pane(
                        "传入版本",
                        block.map(|block| block.theirs.as_str()).unwrap_or(""),
                        vec![
                            self.button(
                                "接受传入",
                                !self.busy,
                                |this, _, _| {
                                    this.apply_selected_conflict_resolution(
                                        ConflictBlockResolution::Theirs,
                                    )
                                },
                                cx,
                            )
                            .into_any_element(),
                            self.button(
                                "接受两边（传入在前）",
                                !self.busy,
                                |this, _, _| {
                                    this.apply_selected_conflict_resolution(
                                        ConflictBlockResolution::BothTheirsFirst,
                                    )
                                },
                                cx,
                            )
                            .into_any_element(),
                        ],
                    )),
            )
            .when(
                self.conflict_workbench.show_base
                    && block.is_some_and(|block| block.base.as_ref().is_some()),
                |this| {
                    this.child(
                        app_panel()
                            .flex_none()
                            .mx_3()
                            .mb_3()
                            .child(
                                div()
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgb(ui_theme::BORDER_MUTED))
                                    .bg(rgb(ui_theme::HEADER_BG))
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(ui_theme::TEXT))
                                    .child("Base"),
                            )
                            .child(self.render_conflict_text_lines(
                                block.and_then(|block| block.base.as_deref()).unwrap_or(""),
                            )),
                    )
                },
            )
    }

    fn render_fallback_conflict_detail(
        &self,
        view: &ConflictFileView,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let Some(path) = self.conflict_workbench.selected_path.clone() else {
            return div().into_any_element();
        };
        let path_for_ours = path.clone();
        let path_for_theirs = path.clone();
        let path_for_mark = path.clone();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .gap_3()
            .p_4()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_sm()
                    .bg(rgb(ui_theme::WARNING_SOFT))
                    .text_size(px(12.0))
                    .text_color(rgb(ui_theme::WARNING_TEXT))
                    .child(
                        view.fallback_reason
                            .clone()
                            .unwrap_or_else(|| "该冲突暂不支持可视化文本编辑".into()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .child(self.button(
                        "当前版本",
                        !self.busy,
                        move |this, _, _| {
                            this.resolve_conflict_with_side(
                                path_for_ours.clone(),
                                ConflictResolutionSide::Ours,
                            )
                        },
                        cx,
                    ))
                    .child(self.button(
                        "传入版本",
                        !self.busy,
                        move |this, _, _| {
                            this.resolve_conflict_with_side(
                                path_for_theirs.clone(),
                                ConflictResolutionSide::Theirs,
                            )
                        },
                        cx,
                    ))
                    .child(self.primary_button(
                        "标记解决",
                        !self.busy,
                        move |this, _, _| this.mark_conflict_resolved(path_for_mark.clone()),
                        cx,
                    )),
            )
            .into_any_element()
    }

    fn render_conflict_side_pane(
        &self,
        title: &'static str,
        content: &str,
        actions: Vec<gpui::AnyElement>,
    ) -> impl IntoElement {
        app_panel()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(280.0))
            .min_w(px(240.0))
            .h_full()
            .child(
                div()
                    .flex_none()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(ui_theme::BORDER_MUTED))
                    .bg(rgb(ui_theme::HEADER_BG))
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(ui_theme::TEXT))
                    .child(title),
            )
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(ui_theme::BORDER_MUTED))
                    .children(actions),
            )
            .child(self.render_conflict_text_lines(content))
    }

    fn render_conflict_text_lines(&self, content: &str) -> impl IntoElement {
        let lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content
                .split('\n')
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .overflow_hidden()
            .p_3()
            .font_family("Consolas, monospace")
            .text_size(px(12.0))
            .bg(rgb(ui_theme::SURFACE))
            .children(lines.into_iter().map(|line| {
                div()
                    .min_h(px(18.0))
                    .text_color(rgb(ui_theme::TEXT))
                    .child(if line.is_empty() {
                        " ".to_string()
                    } else {
                        line
                    })
            }))
    }

    fn resolve_conflict_with_side(&mut self, path: String, side: ConflictResolutionSide) {
        self.diff = None;
        self.diff_headers_expanded = false;
        self.reset_uniform_scroll("diff-scroll");
        let label = match side {
            ConflictResolutionSide::Ours => "已使用当前版本解决冲突",
            ConflictResolutionSide::Theirs => "已使用传入版本解决冲突",
        };
        self.with_repo(label, move |service, repo| {
            service.resolve_conflict_with_side(repo, Path::new(&path), side)
        });
    }

    fn mark_conflict_resolved(&mut self, path: String) {
        self.diff = None;
        self.diff_headers_expanded = false;
        self.reset_uniform_scroll("diff-scroll");
        self.with_repo("冲突已标记为解决", move |service, repo| {
            service.mark_conflict_resolved(repo, Path::new(&path))
        });
    }

    fn render_conflict_summary(&self, count: usize) -> impl IntoElement {
        div()
            .flex_none()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(rgb(ui_theme::BORDER))
            .bg(rgb(ui_theme::WARNING_SOFT))
            .text_size(px(12.0))
            .text_color(rgb(ui_theme::WARNING_TEXT))
            .child(format!("存在 {count} 个冲突文件"))
    }

    fn conflict_row(&self, path: String, cx: &mut Context<Self>) -> impl IntoElement {
        let path_for_switch = path.clone();
        let path_for_ours = path.clone();
        let path_for_theirs = path.clone();
        let path_for_mark = path.clone();

        div()
            .id(format!("conflict-{path}"))
            .flex()
            .flex_none()
            .flex_col()
            .gap_1()
            .px_2()
            .py_2()
            .rounded_sm()
            .border_1()
            .border_color(rgb(ui_theme::WARNING))
            .bg(rgb(ui_theme::WARNING_SOFT))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w(px(0.0))
                    .cursor_pointer()
                    .hover(|this| this.bg(rgb(ui_theme::WARNING_HOVER)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event: &MouseDownEvent, window, cx| {
                            window.focus(&this.conflict_editor.focus);
                            this.main_mode = MainMode::Conflict;
                            this.select_conflict_file(path_for_switch.clone());
                            this.change_context_menu = None;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex_none()
                            .w(px(24.0))
                            .text_size(px(11.0))
                            .font_family("monospace")
                            .text_color(rgb(ui_theme::WARNING_ACCENT_TEXT))
                            .child("!"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_size(px(12.0))
                            .text_color(rgb(ui_theme::TEXT))
                            .truncate()
                            .child(path),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1()
                    .child(self.button(
                        "当前版本",
                        !self.busy,
                        move |this, _, _| {
                            this.resolve_conflict_with_side(
                                path_for_ours.clone(),
                                ConflictResolutionSide::Ours,
                            )
                        },
                        cx,
                    ))
                    .child(self.button(
                        "传入版本",
                        !self.busy,
                        move |this, _, _| {
                            this.resolve_conflict_with_side(
                                path_for_theirs.clone(),
                                ConflictResolutionSide::Theirs,
                            )
                        },
                        cx,
                    ))
                    .child(self.button(
                        "标记解决",
                        !self.busy,
                        move |this, _, _| this.mark_conflict_resolved(path_for_mark.clone()),
                        cx,
                    )),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_paths_follow_snapshot_conflict_order() {
        let mut snapshot = RepositorySnapshot::default();
        snapshot.conflicts = vec!["a.txt".to_string(), "dir/b.txt".to_string()];

        assert_eq!(
            conflict_paths(Some(&snapshot)),
            vec!["a.txt".to_string(), "dir/b.txt".to_string()]
        );
        assert!(conflict_paths(None).is_empty());
    }

    #[test]
    fn conflict_status_message_names_operation_and_resolution_area() {
        assert_eq!(
            conflict_status_message("合并完成", 2),
            "合并产生冲突，请在左侧“冲突”区域解决（2 个文件）"
        );
        assert_eq!(
            conflict_status_message("正在同步", 1),
            "操作产生冲突，请在左侧“冲突”区域解决（1 个文件）"
        );
    }
}
