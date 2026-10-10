use std::{ops::Range, path::Path, sync::Arc};

pub(crate) mod editor;
mod workbench;

use crate::ui::theme::rgb;
use gpui::{
    Context, IntoElement, ListHorizontalSizingBehavior, ListSizingBehavior, MouseButton,
    MouseDownEvent, Window, div, prelude::*,
    px, uniform_list,
};
use khaslana::{
    ConflictBlock, ConflictBlockStatus, ConflictFileKind,
    ConflictFileView, ConflictResolutionSide, RepositorySnapshot,
};

use crate::{
    MainMode, RepositoryView,
    ui::{
        components::{PanelHeaderSurface, app_panel, panel_section_header},
        theme as ui_theme,
    },
    ui_helpers::{ScrollbarMode, scrollable_uniform_frame},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConflictDocumentPane {
    Ours,
    Result,
    Theirs,
}

/// 冲突工作台的平面层级：画布、文件 rail、标题栏与代码内容各自使用稳定语义层。
/// 集中在纯函数中，避免视图重新引入旧 WB_PANEL/STATE_HOVER 作为通用容器颜色。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConflictWorkbenchSurfaces {
    canvas: u32,
    rail: u32,
    header: u32,
    content: u32,
    border: u32,
}

const fn conflict_workbench_surfaces() -> ConflictWorkbenchSurfaces {
    ConflictWorkbenchSurfaces {
        canvas: ui_theme::SURFACE_CANVAS,
        rail: ui_theme::RAIL_SURFACE,
        header: ui_theme::TITLEBAR_SURFACE,
        content: ui_theme::SURFACE_BASE,
        border: ui_theme::BORDER_MUTED,
    }
}

pub(crate) fn conflict_status_message(label: &str, count: usize) -> String {
    if label.starts_with("合并") {
        return format!("合并产生冲突，请在工作区使用 IDEA 或进入“冲突处理”解决（{count} 个文件）");
    }
    let operation = label.strip_suffix("完成").unwrap_or("操作");
    format!("{operation}产生冲突，请在左侧“冲突”区域解决（{count} 个文件）")
}

fn conflict_paths(snapshot: Option<&RepositorySnapshot>) -> Vec<String> {
    snapshot
        .map(|snapshot| snapshot.conflicts.clone())
        .unwrap_or_default()
}

#[cfg(test)]
fn conflict_document_line_owners(
    content: &str,
    pane: ConflictDocumentPane,
    view: &ConflictFileView,
    line_count: usize,
) -> Vec<Option<usize>> {
    let mut owners = vec![None; line_count.max(1)];
    for (index, block) in view.blocks.iter().enumerate() {
        let (start, end) = conflict_document_byte_range(block, pane);
        let range = conflict_byte_range_to_lines(content, start, end);
        for line_index in range {
            if let Some(owner) = owners.get_mut(line_index) {
                *owner = Some(index);
            }
        }
    }
    owners
}

fn conflict_document_byte_range(
    block: &ConflictBlock,
    pane: ConflictDocumentPane,
) -> (usize, usize) {
    match pane {
        ConflictDocumentPane::Ours => (block.ours_start, block.ours_end),
        ConflictDocumentPane::Result => (block.start, block.end),
        ConflictDocumentPane::Theirs => (block.theirs_start, block.theirs_end),
    }
}

fn conflict_byte_range_to_lines(content: &str, start: usize, end: usize) -> std::ops::Range<usize> {
    let start_line = content[..start.min(content.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let mut end_line = content[..end.min(content.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    if end_line == start_line {
        end_line += 1;
    }
    start_line..end_line
}

// ── 三栏连线（IDEA 式采用指示）──────────────────────────────

/// 块区域在某栏内的内容坐标 y 段（窗口坐标，含滚动 offset；
/// offset.y 向下滚动为负）。
#[cfg(test)]
fn uniform_list_line_count(content: &str) -> usize {
    content.bytes().filter(|b| *b == b'\n').count() + 1
}

#[cfg(test)]
fn conflict_block_y_range(
    viewport_top: f32,
    offset_y: f32,
    row_height: f32,
    lines: &Range<usize>,
) -> (f32, f32) {
    let top = viewport_top + offset_y + row_height * lines.start as f32;
    (top, top + row_height * (lines.end - lines.start) as f32)
}

/// 把块区域 y 段裁剪到视口可见范围并返回中点锚点 y；
/// 整段滚出视口（不可见）返回 `None`。
#[cfg(test)]
fn conflict_connector_anchor_y(
    viewport_top: f32,
    viewport_bottom: f32,
    block_top: f32,
    block_bottom: f32,
) -> Option<f32> {
    let visible_top = block_top.max(viewport_top);
    let visible_bottom = block_bottom.min(viewport_bottom);
    (visible_bottom > visible_top).then_some((visible_top + visible_bottom) / 2.0)
}

/// 同步滚动的源栏判定：与上帧比较，恰好一栏 offset 变化超过阈值时
/// 返回该栏（用户滚轮/拖动滚动条只改一栏）；多栏同时变化（程序化
/// 三栏联动 scrollToItem）或全部未变返回 `None`，不做同步。
fn conflict_scroll_sync_source(current: [f32; 3], prev: [f32; 3]) -> Option<usize> {
    let mut source = None;
    for index in 0..3 {
        if (current[index] - prev[index]).abs() > 0.5 {
            if source.is_some() {
                return None;
            }
            source = Some(index);
        }
    }
    source
}

#[cfg(test)]
#[derive(Clone, Debug)]
struct ConflictDocumentLineModel {
    content: Arc<str>,
    ranges: Arc<[Range<usize>]>,
    owners: Arc<[Option<usize>]>,
}

#[derive(Clone, Debug)]
struct ConflictPlainLineModel {
    content: Arc<str>,
    ranges: Arc<[Range<usize>]>,
}

impl ConflictPlainLineModel {
    fn new(content: &str) -> Self {
        Self {
            content: Arc::from(content),
            ranges: Arc::from(conflict_document_line_ranges(content)),
        }
    }

    fn line_count(&self) -> usize {
        self.ranges.len().max(1)
    }

    fn line_text(&self, index: usize) -> &str {
        let Some(range) = self.ranges.get(index) else {
            return "";
        };
        &self.content[range.clone()]
    }
}

#[cfg(test)]
impl ConflictDocumentLineModel {
    fn new(content: &str, pane: ConflictDocumentPane, view: &ConflictFileView) -> Self {
        let ranges = conflict_document_line_ranges(content);
        let owners = conflict_document_line_owners(content, pane, view, ranges.len());
        Self {
            content: Arc::from(content),
            ranges: Arc::from(ranges),
            owners: Arc::from(owners),
        }
    }

    fn line_count(&self) -> usize {
        self.ranges.len().max(1)
    }

    fn line_text(&self, index: usize) -> &str {
        let Some(range) = self.ranges.get(index) else {
            return "";
        };
        &self.content[range.clone()]
    }

    fn owner_at(&self, index: usize) -> Option<usize> {
        self.owners.get(index).copied().flatten()
    }
}

fn conflict_document_line_ranges(content: &str) -> Vec<Range<usize>> {
    if content.is_empty() {
        return vec![0..0];
    }
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, ch) in content.char_indices() {
        if ch == '\n' {
            ranges.push(start..index);
            start = index + ch.len_utf8();
        }
    }
    ranges.push(start..content.len());
    ranges
}

#[cfg(test)]
fn conflict_line_colors(
    pane: ConflictDocumentPane,
    block: &ConflictBlock,
    active: bool,
) -> (u32, u32) {
    match block.status {
        ConflictBlockStatus::Ignored => {
            if active {
                (ui_theme::STATE_SELECTION, ui_theme::CONTENT_PRIMARY)
            } else {
                (ui_theme::SURFACE_BASE, ui_theme::CONTENT_SECONDARY)
            }
        }
        ConflictBlockStatus::Resolved(_) => match pane {
            ConflictDocumentPane::Ours | ConflictDocumentPane::Theirs => {
                if active {
                    (ui_theme::FEEDBACK_ERROR_BG, ui_theme::FEEDBACK_ERROR_TEXT)
                } else {
                    (ui_theme::SURFACE_BASE, ui_theme::CONTENT_PRIMARY)
                }
            }
            ConflictDocumentPane::Result => {
                if active {
                    (ui_theme::STATE_SELECTION, ui_theme::CONTENT_PRIMARY)
                } else {
                    (ui_theme::SURFACE_BASE, ui_theme::CONTENT_PRIMARY)
                }
            }
        },
        // AI 合并块：选中时结果区用绿色高亮（区别于未处理的黄色），
        // 直观表达「这段已经合并完成」。
        ConflictBlockStatus::Merged => match pane {
            ConflictDocumentPane::Ours | ConflictDocumentPane::Theirs => {
                if active {
                    (ui_theme::FEEDBACK_ERROR_BG, ui_theme::FEEDBACK_ERROR_TEXT)
                } else {
                    (ui_theme::SURFACE_BASE, ui_theme::CONTENT_PRIMARY)
                }
            }
            ConflictDocumentPane::Result => {
                if active {
                    (
                        ui_theme::FEEDBACK_SUCCESS_BG,
                        ui_theme::FEEDBACK_SUCCESS_TEXT,
                    )
                } else {
                    (ui_theme::SURFACE_BASE, ui_theme::CONTENT_PRIMARY)
                }
            }
        },
        ConflictBlockStatus::Unresolved => match pane {
            ConflictDocumentPane::Ours | ConflictDocumentPane::Theirs => {
                if active {
                    (ui_theme::FEEDBACK_ERROR_BG, ui_theme::FEEDBACK_ERROR_TEXT)
                } else {
                    (ui_theme::FEEDBACK_WARNING_BG, ui_theme::CONTENT_PRIMARY)
                }
            }
            ConflictDocumentPane::Result => {
                if active {
                    (
                        ui_theme::FEEDBACK_WARNING_BG,
                        ui_theme::FEEDBACK_WARNING_TEXT,
                    )
                } else {
                    (ui_theme::SURFACE_BASE, ui_theme::CONTENT_PRIMARY)
                }
            }
        },
    }
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
        let conflict_count = conflicts.len();

        div().when(has_conflicts, |this| {
            this.child(self.render_conflict_summary(conflicts.len()))
                .child(self.render_change_section(
                    "冲突",
                    "conflict-list",
                    "",
                    false,
                    conflict_rows,
                    true,
                    conflict_count,
                    false,
                    Vec::new(),
                    cx,
                ))
                .child(div().flex_none().h(px(1.0)).bg(rgb(ui_theme::BORDER_MUTED)))
        })
    }

    pub(crate) fn render_conflict_workbench(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_approved_conflict_workbench(window, cx)
    }

    fn render_conflict_file_rail(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let surfaces = conflict_workbench_surfaces();
        let conflicts = conflict_paths(self.snapshot.as_ref());
        let selected_path = self.conflict_workbench.selected_path.as_deref();
        let row_count = conflicts.len();
        let selected_path = selected_path.map(str::to_owned);
        let paths = conflicts.clone();
        let handle = self.uniform_scroll_handle("conflict-files-scroll");
        let list = uniform_list("conflict-files", row_count,
            cx.processor(move |this, range: Range<usize>, _, cx| {
                range.filter_map(|index| paths.get(index)).map(|path|
                    this.conflict_file_row(path.clone(), selected_path.as_deref() == Some(path.as_str()), cx).into_any_element()
                ).collect::<Vec<_>>()
            }))
            .track_scroll(&handle).with_sizing_behavior(ListSizingBehavior::Auto)
            .flex_1().min_w(px(0.0)).min_h(px(0.0));

        app_panel()
            .flex()
            .flex_none()
            .flex_col()
            .w(px(224.0))
            .min_w(px(224.0))
            .h_full()
            .bg(rgb(surfaces.rail))
            .child(
                div()
                    .flex_none()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(surfaces.border))
                    .bg(rgb(surfaces.rail))
                    .text_size(px(ui_theme::TYPE_BODY))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                    .child(format!("存在 {} 个冲突文件", conflicts.len())),
            )
            .child(scrollable_uniform_frame("conflict-files-scroll", ScrollbarMode::Vertical,
                list.into_any_element(), handle, true, cx))
            .child(div().flex_none().p_2().text_size(px(11.0)).text_color(rgb(ui_theme::CONTENT_SECONDARY))
                .child(if self.snapshot.as_ref().is_some_and(|s| s.rebase_in_progress) { "变基进行中 · 更多菜单可继续或中止" } else if self.merge_in_progress() { "合并进行中 · 解决文件后继续" } else { "冲突处理中 · 结果需标记已解决" }))
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
            Some(ConflictFileKind::WorktreeOnly) => "工作区",
            None if self.conflict_workbench.load_errors.contains_key(&path) => "读取失败",
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
                    window.focus(&this.conflict_editor.focus, cx);
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
                            .text_size(px(ui_theme::TYPE_BODY))
                            .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                            .truncate()
                            .child(path),
                    )
                    .child(
                        div()
                            .flex_none()
                            .px_1()
                            .py(px(2.0))
                            .rounded(px(ui_theme::RADIUS_XS))
                            .bg(rgb(ui_theme::SURFACE_SUNKEN))
                            .text_size(px(ui_theme::TYPE_META))
                            .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                            .child(badge),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(ui_theme::TYPE_META))
                    .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                    .child(format!("未处理 {unresolved}"))
                    .when(dirty, |this| this.child("草稿已修改"))
                    .when(applied, |this| this.child("已应用")),
            )
    }

    fn render_conflict_detail(&self, _window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surfaces = conflict_workbench_surfaces();
        let selected_path = self.conflict_workbench.selected_path.clone();
        let selected_view = selected_path
            .as_ref()
            .and_then(|path| self.conflict_workbench.files.get(path));
        let title = selected_path
            .clone()
            .unwrap_or_else(|| "请选择一个冲突文件".to_string());

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .bg(rgb(surfaces.canvas))
            .child(self.render_conflict_header(title, selected_view, cx))
            .child(match selected_view {
                Some(view) if view.kind == ConflictFileKind::Text => self
                    .render_text_conflict_detail(view, cx)
                    .into_any_element(),
                Some(view) => self
                    .render_fallback_conflict_detail(view, cx)
                    .into_any_element(),
                None => div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                    .child(selected_path.as_ref().map(|path|
                        self.conflict_workbench.load_errors.get(path).map(|e| format!("读取失败：{e}。请重新选择该文件重试。"))
                            .unwrap_or_else(|| "正在读取三方版本…".into()))
                        .unwrap_or_else(|| "请选择一个冲突文件".into()))
                    .into_any_element(),
            })
    }

    fn render_conflict_header(&self, title: String, view: Option<&ConflictFileView>, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_approved_conflict_header(title, view, cx)
    }

    fn render_text_conflict_detail(&self, view: &ConflictFileView, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_approved_text_conflict(view, cx)
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
        let path_for_directory = path.clone();

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
                    .border_1()
                    .border_color(rgb(ui_theme::BORDER_MUTED))
                    .bg(rgb(ui_theme::SURFACE_SUNKEN))
                    .text_size(px(ui_theme::TYPE_BODY))
                    .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                    .child(
                        view.fallback_reason
                            .clone()
                            .unwrap_or_else(|| "该冲突暂不支持可视化文本编辑".into()),
                    ),
            )
            .when(view.kind == ConflictFileKind::WorktreeOnly, |this| {
                this.child(self.button(
                    "打开文件所在目录",
                    !self.busy,
                    move |this, _, cx| {
                        this.open_file_parent_directory(path_for_directory.clone(), cx)
                    },
                    cx,
                ))
            })
            .when(view.kind != ConflictFileKind::WorktreeOnly, |this| this.child(
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
            ))
            .into_any_element()
    }

    /// 三栏连线 overlay：从「当前版本/传入版本」两侧的冲突块区域画
    /// S 形曲线指向结果区对应块区域，指示采用后内容落点（IDEA 风格）。
    /// 纯绘制 canvas，不注册鼠标事件、不拦截交互；作为三栏行容器的
    /// 最后一个子元素绘制在最上层，每帧按各栏滚动 offset 重绘。
    fn render_conflict_connectors(&self, view: &ConflictFileView, selected_block: usize) -> impl IntoElement {
        self.render_editor_connectors(view, selected_block)
    }

    fn render_conflict_document_pane(
        &self,
        title: &'static str,
        scroll_id: &'static str,
        handle_id: &'static str,
        content: &str,
        pane: ConflictDocumentPane,
        view: &ConflictFileView,
        selected_block: usize,
        _actions: Vec<gpui::AnyElement>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().flex().flex_col().flex_1().min_w(px(0.0)).min_h(px(0.0)).h_full()
            .child(div().flex_none().h(px(49.0)).flex().flex_col().gap_1().px_3().py_1()
                .border_b_1().border_color(rgb(ui_theme::BORDER_MUTED))
                .bg(rgb(if pane == ConflictDocumentPane::Result { ui_theme::STATE_HOVER } else { ui_theme::SURFACE_BASE }))
                .child(div().flex().items_center().justify_between()
                    .child(div().text_size(px(13.0)).font_weight(gpui::FontWeight::SEMIBOLD).text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(title))
                    .child(div().text_size(px(10.0)).text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(if pane == ConflictDocumentPane::Result {"可编辑"} else {"只读"})))
                .child(div().flex().justify_between().items_center()
                    .child(div().truncate().text_size(px(10.0)).text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(if pane != ConflictDocumentPane::Result {"UTF-8 · 原始版本"} else if view.draft_status == khaslana::ConflictDraftStatus::Applied {"UTF-8 · 已应用到工作区"} else {"UTF-8 · 结果尚未应用"}))
                    .when_some(view.blocks.get(selected_block), |this,block| this.child(self.conflict_block_status_badge(block.status,block.has_manual_edits)))))
            .child(self.render_conflict_document_text(scroll_id,handle_id,content,pane,view,selected_block,cx))
    }

    /// 取某冲突文件某分栏的语法高亮（带主题变体守卫；未计算/已失效返回 None，
    /// 回退整行块状态前景色）。
    fn conflict_syntax_spans(
        &self,
        pane: ConflictDocumentPane,
        view: &ConflictFileView,
    ) -> Option<Arc<khaslana::syntax::SyntaxSpans>> {
        let entry = self.conflict_workbench.syntax.get(&view.path)?;
        let spans = match pane {
            ConflictDocumentPane::Ours => entry.ours.as_ref(),
            ConflictDocumentPane::Result => entry.draft.as_ref(),
            ConflictDocumentPane::Theirs => entry.theirs.as_ref(),
        };
        spans
            .filter(|spans| spans.dark == ui_theme::active_variant().is_dark())
            .cloned()
    }

    fn render_conflict_document_text(
        &self,
        _scroll_id: &'static str,
        _handle_id: &'static str,
        _content: &str,
        pane: ConflictDocumentPane,
        view: &ConflictFileView,
        _selected_block: usize,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.render_merge_editor(pane, view, cx)
    }

    fn render_conflict_plain_text(
        &self,
        scroll_id: &'static str,
        content: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let handle = self.uniform_scroll_handle(scroll_id);
        let list_handle = handle.clone();
        let model = Arc::new(ConflictPlainLineModel::new(content));
        let row_count = model.line_count();
        let model_for_list = model.clone();
        let content = div()
            .id(scroll_id)
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .overflow_y_scroll()
            .p_3()
            .font_family("Consolas")
            .text_size(px(ui_theme::TYPE_BODY))
            .bg(rgb(ui_theme::SURFACE_BASE))
            .child(
                uniform_list(
                    scroll_id,
                    row_count,
                    cx.processor(move |_this, range: Range<usize>, _window, _cx| {
                        range
                            .map(|line_index| {
                                let line = model_for_list.line_text(line_index);
                                div()
                                    .min_h(px(18.0))
                                    .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                                    .child(if line.is_empty() {
                                        " ".to_string()
                                    } else {
                                        line.to_string()
                                    })
                                    .into_any_element()
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .track_scroll(&list_handle)
                .with_sizing_behavior(ListSizingBehavior::Auto)
                .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(0.0)),
            )
            .into_any_element();
        scrollable_uniform_frame(
            scroll_id,
            ScrollbarMode::Vertical,
            content,
            handle,
            true,
            cx,
        )
    }

    fn conflict_block_status_badge(
        &self,
        status: ConflictBlockStatus,
        has_manual_edits: bool,
    ) -> impl IntoElement {
        let (label, bg, fg) = match status {
            ConflictBlockStatus::Ignored => {
                ("已忽略", ui_theme::STATE_HOVER, ui_theme::CONTENT_SECONDARY)
            }
            ConflictBlockStatus::Resolved(_) => {
                ("已处理", ui_theme::STATE_HOVER, ui_theme::PRIMARY)
            }
            ConflictBlockStatus::Merged => (
                "已合并",
                ui_theme::FEEDBACK_SUCCESS_BG,
                ui_theme::FEEDBACK_SUCCESS_TEXT,
            ),
            ConflictBlockStatus::Unresolved if has_manual_edits => (
                "手工修改",
                ui_theme::FEEDBACK_WARNING_BG,
                ui_theme::FEEDBACK_WARNING_TEXT,
            ),
            ConflictBlockStatus::Unresolved => (
                "未处理",
                ui_theme::FEEDBACK_WARNING_BG,
                ui_theme::FEEDBACK_WARNING_TEXT,
            ),
        };
        div()
            .flex_none()
            .px_2()
            .py(px(2.0))
            .rounded_sm()
            .bg(rgb(bg))
            .text_size(px(10.0))
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(rgb(fg))
            .child(label)
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

    pub(crate) fn resolve_selected_conflict_with_intellij_idea(&mut self) {
        let Some(path) = self.conflict_workbench.selected_path.clone() else {
            self.last_error = Some("请先选择一个冲突文件".into());
            return;
        };
        self.resolve_conflict_with_intellij_idea_path(path);
    }

    fn resolve_conflict_with_intellij_idea_path(&mut self, path: String) -> bool {
        self.request_external_merge_for_path(path)
    }

    pub(crate) fn maybe_auto_open_external_merge_for_selected_conflict(&mut self) {
        if self.busy
            || !self.external_merge_settings.enabled
            || !self.external_merge_settings.auto_open_intellij
        {
            return;
        }
        let Some(path) = self.conflict_workbench.selected_path.clone() else {
            return;
        };
        if !self.conflict_workbench.files.contains_key(&path)
            || self
                .conflict_workbench
                .external_merge_auto_opened
                .contains(&path)
        {
            return;
        }
        if self.resolve_conflict_with_intellij_idea_path(path.clone()) {
            self.conflict_workbench
                .mark_external_merge_auto_opened(path);
        }
    }

    fn render_conflict_summary(&self, count: usize) -> impl IntoElement {
        // 与工作台内其他面板头统一走分组标题（分组底色带替代贯穿分割线）。
        panel_section_header(format!("存在 {count} 个冲突文件"))
            .surface(PanelHeaderSurface::Grouped)
            .build()
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
            .rounded(px(ui_theme::RADIUS_XS))
            .border_1()
            .border_color(rgb(ui_theme::BORDER_MUTED))
            .bg(rgb(ui_theme::SURFACE_BASE))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w(px(0.0))
                    .cursor_pointer()
                    .hover(|this| this.bg(rgb(ui_theme::STATE_HOVER)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event: &MouseDownEvent, window, cx| {
                            window.focus(&this.conflict_editor.focus, cx);
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
                            .text_size(px(ui_theme::TYPE_META))
                            .font_family("Consolas")
                            .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                            .child("!"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .text_size(px(ui_theme::TYPE_BODY))
                            .text_color(rgb(ui_theme::CONTENT_PRIMARY))
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
#[path = "../tests/conflicts.rs"]
mod tests;
