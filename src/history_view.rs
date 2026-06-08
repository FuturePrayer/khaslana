use std::collections::BTreeSet;

use gpui::{Context, IntoElement, PathBuilder, div, point, prelude::*, px, rgb};
use khaslana::{CommitFileChange, CommitInfo, CommitRefInfo, CommitRefKind};

use crate::{
    CHANGE_ROW_HEIGHT, COLOR_BLUE, COLOR_BLUE_DARK, COLOR_BLUE_SOFT, COLOR_BORDER,
    COLOR_BORDER_STRONG, COLOR_HASH_BG, COLOR_PANEL_BG, COLOR_ROW_SELECTED, COLOR_SURFACE,
    COLOR_TEXT, COLOR_TEXT_FAINT, COLOR_TEXT_MUTED, DiffHeaderTarget, EncodingMenuTarget,
    RepositoryView, ResizeTarget, author_avatar, commit_time_label, nav_list, placeholder_row,
    section_header,
};

const HISTORY_GRAPH_WIDTH: f32 = 96.0;
const HISTORY_GRAPH_ROW_HEIGHT: f32 = 42.0;
const GRAPH_LANE_START: f32 = 12.0;
const GRAPH_LANE_SPACING: f32 = 14.0;
const GRAPH_COLORS: [u32; 8] = [
    0xf97316, 0x14b8a6, 0x3b82f6, 0xeab308, 0xef4444, 0x8b5cf6, 0x22c55e, 0xec4899,
];

#[derive(Clone, Debug, Default)]
struct CommitGraphRow {
    lane: usize,
    lanes: Vec<usize>,
    connectors: Vec<usize>,
}

impl RepositoryView {
    pub(crate) fn render_history_view(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .bg(rgb(COLOR_PANEL_BG))
            .child(self.render_commit_history(cx))
            .child(self.render_column_splitter(ResizeTarget::HistoryTop, cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .child(self.render_commit_files(cx))
                    .child(self.render_column_splitter(ResizeTarget::HistoryFiles, cx))
                    .child(self.render_history_diff(cx)),
            )
    }

    fn render_commit_history(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let graph_rows = commit_graph_rows(&self.history_commits);
        let mut rows = Vec::new();
        if self.history_commits.is_empty() {
            rows.push(
                placeholder_row(if self.history_loading.commits {
                    "提交记录加载中..."
                } else if self.repo_path.is_some() {
                    "暂无提交记录"
                } else {
                    "请先打开一个仓库"
                })
                .into_any_element(),
            );
        } else {
            rows.extend(
                self.history_commits
                    .iter()
                    .cloned()
                    .zip(graph_rows)
                    .map(|(commit, graph)| self.commit_row(commit, graph, cx).into_any_element()),
            );
            if self.history_has_more {
                rows.push(
                    div()
                        .flex_none()
                        .py_1()
                        .child(self.button(
                            if self.history_loading.commits {
                                "加载中..."
                            } else {
                                "加载更多"
                            },
                            !self.history_loading.commits,
                            |this, _, _| this.load_more_history(),
                            cx,
                        ))
                        .into_any_element(),
                );
            }
        }

        div()
            .flex()
            .flex_col()
            .flex_none()
            .min_w(px(0.0))
            .h(px(self.history_top_height))
            .min_h(px(180.0))
            .w_full()
            .child(section_header("提交记录（所有分支）"))
            .child(nav_list(self, "commit-history-list", rows, cx))
    }

    fn commit_row(
        &self,
        commit: CommitInfo,
        graph: CommitGraphRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.history_selected_commit.as_deref() == Some(commit.oid.as_str());
        let oid = commit.oid.clone();
        let right_click_oid = commit.oid.clone();
        let right_click_short_oid = commit.short_oid.clone();
        let right_click_summary = commit.summary.clone();
        let right_click_parent_count = commit.parents.len();
        let author = commit.author.clone();
        let time = commit_time_label(commit.time);
        let ref_labels = commit_ref_labels(&commit.refs);
        let hidden_ref_count = commit.refs.len().saturating_sub(3);

        div()
            .id(format!("commit-{}", commit.short_oid))
            .flex()
            .flex_none()
            .items_center()
            .gap_2()
            .pr_2()
            .py_1()
            .h(px(HISTORY_GRAPH_ROW_HEIGHT))
            .rounded_sm()
            .cursor_pointer()
            .bg(if selected {
                rgb(COLOR_ROW_SELECTED)
            } else {
                rgb(COLOR_SURFACE)
            })
            .border_1()
            .border_color(if selected {
                rgb(COLOR_BORDER_STRONG)
            } else {
                rgb(COLOR_BORDER)
            })
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.select_history_commit(oid.clone());
                cx.notify();
            }))
            .on_mouse_down(
                gpui::MouseButton::Right,
                cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                    this.open_commit_context_menu(
                        right_click_oid.clone(),
                        right_click_short_oid.clone(),
                        right_click_summary.clone(),
                        right_click_parent_count,
                        event,
                        _window,
                    );
                    cx.notify();
                }),
            )
            .child(render_commit_graph_cell(graph))
            .child(
                div()
                    .flex_none()
                    .w(px(72.0))
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(rgb(COLOR_HASH_BG))
                    .font_family("Consolas, monospace")
                    .text_size(px(11.0))
                    .text_color(rgb(COLOR_BLUE_DARK))
                    .text_align(gpui::TextAlign::Center)
                    .child(commit.short_oid),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .flex_1()
                    .min_w(px(0.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(COLOR_TEXT))
                            .truncate()
                            .child(commit.summary),
                    )
                    .children(ref_labels)
                    .when(hidden_ref_count > 0, |this| {
                        this.child(commit_ref_overflow_label(hidden_ref_count))
                    }),
            )
            .child(author_avatar(&author))
            .child(
                div()
                    .flex_none()
                    .w(px(118.0))
                    .text_size(px(11.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .truncate()
                    .child(author),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(142.0))
                    .text_size(px(11.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(time),
            )
            .child(
                div()
                    .flex_none()
                    .text_size(px(16.0))
                    .text_color(rgb(COLOR_TEXT_FAINT))
                    .child(">"),
            )
    }

    fn render_commit_files(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = if self.history_files.is_empty() {
            vec![
                placeholder_row(if self.history_loading.files {
                    "提交文件加载中..."
                } else if self.history_selected_commit.is_some() {
                    "该提交没有文件变更"
                } else {
                    "请选择一个提交"
                })
                .into_any_element(),
            ]
        } else {
            self.history_files
                .iter()
                .cloned()
                .map(|file| self.commit_file_row(file, cx).into_any_element())
                .collect()
        };

        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(self.history_files_width))
            .min_w(px(self.history_files_width))
            .min_h(px(0.0))
            .h_full()
            .child(section_header("提交文件"))
            .child(nav_list(self, "commit-file-list", rows, cx))
    }

    fn commit_file_row(&self, file: CommitFileChange, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.history_selected_file.as_deref() == Some(file.path.as_str());
        let path = file.path.clone();
        let path_label = file
            .old_path
            .as_ref()
            .filter(|old_path| old_path.as_str() != file.path.as_str())
            .map(|old_path| format!("{old_path} -> {}", file.path))
            .unwrap_or_else(|| file.path.clone());
        let state = file.status.label();

        div()
            .id(format!("commit-file-{}", file.path))
            .flex()
            .flex_none()
            .items_center()
            .gap_1()
            .h(px(CHANGE_ROW_HEIGHT))
            .px_2()
            .py_1()
            .rounded_sm()
            .cursor_pointer()
            .overflow_hidden()
            .bg(if selected {
                rgb(COLOR_ROW_SELECTED)
            } else {
                rgb(COLOR_SURFACE)
            })
            .border_1()
            .border_color(if selected {
                rgb(COLOR_BORDER_STRONG)
            } else {
                rgb(COLOR_BORDER)
            })
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.select_history_file(path.clone());
                cx.notify();
            }))
            .child(
                div()
                    .flex_none()
                    .w(px(24.0))
                    .text_size(px(11.0))
                    .font_family("Consolas, monospace")
                    .text_color(rgb(COLOR_BLUE))
                    .child(state),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .truncate()
                    .child(path_label),
            )
    }

    fn render_history_diff(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self
            .history_selected_file
            .as_ref()
            .map(|path| format!("提交差异：{path}"))
            .unwrap_or_else(|| "提交差异".to_string());
        let empty_message = if self.history_loading.diff {
            "提交差异加载中..."
        } else {
            "请选择一个提交文件查看差异"
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .relative()
            .min_w(px(0.0))
            .h_full()
            .child(self.diff_section_header(title, EncodingMenuTarget::History, cx))
            .child(self.render_virtual_diff(
                "history-diff-scroll",
                self.history_diff.clone(),
                self.history_diff_headers_expanded,
                DiffHeaderTarget::History,
                empty_message.to_string(),
                cx,
            ))
            .child(self.render_encoding_dropdown(EncodingMenuTarget::History, cx))
    }
}

fn commit_graph_rows(commits: &[CommitInfo]) -> Vec<CommitGraphRow> {
    let loaded_oids = commits
        .iter()
        .map(|commit| commit.oid.as_str())
        .collect::<BTreeSet<_>>();
    let mut lanes = Vec::<Option<String>>::new();
    let mut rows = Vec::with_capacity(commits.len());

    for commit in commits {
        let lane = lanes
            .iter()
            .position(|oid| oid.as_deref() == Some(commit.oid.as_str()))
            .unwrap_or_else(|| {
                if let Some(index) = lanes.iter().position(Option::is_none) {
                    lanes[index] = Some(commit.oid.clone());
                    index
                } else {
                    lanes.push(Some(commit.oid.clone()));
                    lanes.len() - 1
                }
            });
        let lanes_before = active_lane_indices(&lanes, lane);
        let mut connectors = Vec::new();

        if let Some(first_parent) = commit.parents.first() {
            lanes[lane] = Some(first_parent.clone());
            connectors.push(lane);
        } else {
            lanes[lane] = None;
        }

        for parent in commit.parents.iter().skip(1) {
            let parent_lane = lanes
                .iter()
                .position(|oid| oid.as_deref() == Some(parent.as_str()))
                .unwrap_or_else(|| {
                    if let Some(index) = lanes.iter().position(Option::is_none) {
                        lanes[index] = Some(parent.clone());
                        index
                    } else {
                        lanes.push(Some(parent.clone()));
                        lanes.len() - 1
                    }
                });
            connectors.push(parent_lane);
        }

        for lane in lanes.iter_mut() {
            if let Some(oid) = lane.as_ref()
                && !loaded_oids.contains(oid.as_str())
            {
                *lane = None;
            }
        }

        connectors.sort_unstable();
        connectors.dedup();
        rows.push(CommitGraphRow {
            lane,
            lanes: lanes_before,
            connectors,
        });
    }

    rows
}

fn active_lane_indices(lanes: &[Option<String>], current_lane: usize) -> Vec<usize> {
    let mut indices = lanes
        .iter()
        .enumerate()
        .filter_map(|(index, oid)| oid.as_ref().map(|_| index))
        .collect::<Vec<_>>();
    if !indices.contains(&current_lane) {
        indices.push(current_lane);
    }
    indices.sort_unstable();
    indices.dedup();
    indices
}

fn render_commit_graph_cell(graph: CommitGraphRow) -> impl IntoElement {
    let max_lane = graph
        .lanes
        .iter()
        .chain(graph.connectors.iter())
        .copied()
        .max()
        .unwrap_or(graph.lane)
        .min(5);

    div()
        .relative()
        .flex_none()
        .w(px(HISTORY_GRAPH_WIDTH))
        .h_full()
        .overflow_hidden()
        .child(
            gpui::canvas(
                |_, _, _| graph,
                |bounds, graph, window, _cx| {
                    let top_y = bounds.origin.y;
                    let bottom_y = bounds.origin.y + bounds.size.height;
                    let center_y = bounds.origin.y + px(HISTORY_GRAPH_ROW_HEIGHT / 2.0);
                    let current_lane = graph.lane.min(5);
                    let current_x = bounds.origin.x + px(graph_x(current_lane));

                    for lane in graph.lanes.iter().copied().filter(|lane| *lane <= 5) {
                        let x = bounds.origin.x + px(graph_x(lane));
                        paint_graph_line(window, x, top_y, x, bottom_y, graph_color(lane));
                    }

                    for target in graph.connectors.iter().copied().filter(|lane| *lane <= 5) {
                        let target_x = bounds.origin.x + px(graph_x(target));
                        paint_graph_line(
                            window,
                            current_x,
                            center_y,
                            target_x,
                            bottom_y,
                            graph_color(target),
                        );
                    }

                    paint_graph_dot(window, current_x, center_y, graph_color(current_lane));
                },
            )
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .right(px(0.0))
            .bottom(px(0.0)),
        )
        .when(max_lane >= 5, |this| {
            this.child(
                div()
                    .absolute()
                    .right(px(4.0))
                    .top(px(15.0))
                    .text_size(px(10.0))
                    .font_family("Consolas, monospace")
                    .text_color(rgb(COLOR_TEXT_FAINT))
                    .child("..."),
            )
        })
}

fn graph_x(lane: usize) -> f32 {
    GRAPH_LANE_START + GRAPH_LANE_SPACING * lane as f32
}

fn graph_color(lane: usize) -> u32 {
    GRAPH_COLORS[lane % GRAPH_COLORS.len()]
}

fn paint_graph_line(
    window: &mut gpui::Window,
    x1: gpui::Pixels,
    y1: gpui::Pixels,
    x2: gpui::Pixels,
    y2: gpui::Pixels,
    color: u32,
) {
    let mut builder = PathBuilder::stroke(px(2.0));
    builder.move_to(point(x1, y1));
    builder.line_to(point(x2, y2));
    if let Ok(path) = builder.build() {
        window.paint_path(path, rgb(color));
    }
}

fn paint_graph_dot(window: &mut gpui::Window, x: gpui::Pixels, y: gpui::Pixels, color: u32) {
    let outer = px(5.0);
    let inner = px(4.0);
    paint_graph_circle(window, x, y, outer, COLOR_PANEL_BG);
    paint_graph_circle(window, x, y, inner, color);
}

fn paint_graph_circle(
    window: &mut gpui::Window,
    x: gpui::Pixels,
    y: gpui::Pixels,
    radius: gpui::Pixels,
    color: u32,
) {
    let mut builder = PathBuilder::fill();
    builder.move_to(point(x - radius, y));
    builder.arc_to(
        point(radius, radius),
        px(0.0),
        false,
        true,
        point(x + radius, y),
    );
    builder.arc_to(
        point(radius, radius),
        px(0.0),
        false,
        true,
        point(x - radius, y),
    );
    builder.close();
    if let Ok(path) = builder.build() {
        window.paint_path(path, rgb(color));
    }
}

fn commit_ref_labels(refs: &[CommitRefInfo]) -> Vec<gpui::AnyElement> {
    refs.iter()
        .take(3)
        .cloned()
        .map(|reference| commit_ref_label(reference).into_any_element())
        .collect()
}

fn commit_ref_label(reference: CommitRefInfo) -> impl IntoElement {
    let (bg, border, fg, label) = match reference.kind {
        CommitRefKind::LocalBranch => (0xe1fbf4, 0x14b8a6, 0x0f766e, reference.name),
        CommitRefKind::RemoteBranch => (0xffeadf, 0xff6a3d, 0xc2410c, reference.name),
        CommitRefKind::Tag => (0xfff3bf, 0xf0b429, 0x9a6700, reference.name),
        CommitRefKind::Head => (0x2f2a24, 0x2f2a24, 0xffffff, reference.name),
    };

    div()
        .flex_none()
        .max_w(px(120.0))
        .px_1()
        .py(px(1.0))
        .rounded_sm()
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(bg))
        .text_size(px(10.0))
        .text_color(rgb(fg))
        .truncate()
        .child(label)
}

fn commit_ref_overflow_label(count: usize) -> impl IntoElement {
    div()
        .flex_none()
        .px_1()
        .py(px(1.0))
        .rounded_sm()
        .border_1()
        .border_color(rgb(COLOR_BORDER))
        .bg(rgb(COLOR_HASH_BG))
        .text_size(px(10.0))
        .text_color(rgb(COLOR_TEXT_MUTED))
        .child(format!("+{count}"))
}
