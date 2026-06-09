use std::path::Path;

use gpui::{Context, IntoElement, MouseButton, MouseDownEvent, div, prelude::*, px, rgb};
use khaslana::{ConflictResolutionSide, DiffScope, RepositorySnapshot};

use crate::{COLOR_BORDER, COLOR_TEXT, RepositoryView};

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
                .child(div().flex_none().h(px(1.0)).bg(rgb(COLOR_BORDER)))
        })
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
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(0xfffbeb))
            .text_size(px(12.0))
            .text_color(rgb(0x92400e))
            .child(format!("存在 {count} 个冲突文件"))
    }

    fn conflict_row(&self, path: String, cx: &mut Context<Self>) -> impl IntoElement {
        let path_for_load = path.clone();
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
            .border_color(rgb(0xf59e0b))
            .bg(rgb(0xfffbeb))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w(px(0.0))
                    .cursor_pointer()
                    .hover(|this| this.bg(rgb(0xfef3c7)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event: &MouseDownEvent, _window, cx| {
                            this.load_diff(path_for_load.clone(), DiffScope::Unstaged);
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
                            .text_color(rgb(0xb45309))
                            .child("!"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_size(px(12.0))
                            .text_color(rgb(COLOR_TEXT))
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
