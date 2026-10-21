#[path = "../ui_resources/conflicts_mod_messages.rs"]
mod messages;

use std::{ops::Range, path::Path, sync::Arc};

pub(crate) mod editor;
mod document;
mod native_editor;
mod workbench;

use crate::ui::theme::rgb;
use gpui::{
    Context, IntoElement, ListHorizontalSizingBehavior, ListSizingBehavior, MouseButton,
    MouseDownEvent, Window, div, prelude::*,
    px, uniform_list,
};
use khaslana::{
    ConflictFileKind,
    ConflictFileView, ConflictResolutionSide, RepositorySnapshot,
};

use crate::{
    MainMode, RepositoryView,
    ui::{
        components::{PanelHeaderSurface, panel_section_header},
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

pub(crate) fn conflict_status_message(operation_key: &str, count: usize) -> String {
    // 操作身份来自核心事件链；语言和完成文案不参与冲突路由。
    let message = if operation_key == crate::operation_ui_resources::MERGE_OPERATION_COMPLETED.key
        || operation_key == crate::operation_ui_resources::MERGE_COMPLETED.key
    {
        messages::MERGE_CONFLICTS_OCCURRED_USE_IDEA_FROM_THE_WORKING_TREE_OR
    } else {
        messages::OPERATION_CONFLICTS
    };
    khaslana::ui_i18n::encode(message, &[("count", &count.to_string())])
}

fn conflict_paths(snapshot: Option<&RepositorySnapshot>) -> Vec<String> {
    snapshot
        .map(|snapshot| snapshot.conflicts.clone())
        .unwrap_or_default()
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
                    khaslana::ui_i18n::text(messages::CONFLICTS),
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

        crate::ui::components::floating_panel()
            .flex()
            .flex_none()
            .flex_col()
            .w(px(224.0))
            .min_w(px(224.0))
            .h_full()
            .bg(rgb(ui_theme::SURFACE_BASE))
            .child(
                div()
                    .flex_none()
                    .px_4()
                    .h(px(56.0))
                    .flex().items_center().justify_between()
                    .text_size(px(ui_theme::TYPE_BODY))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                    .child(khaslana::ui_i18n::text(messages::CONFLICT_FILES))
                    .child(div().text_size(px(12.0)).text_color(rgb(ui_theme::FEEDBACK_WARNING_TEXT)).child(conflicts.len().to_string())),
            )
            .child(div().flex_none().mx_3().mb_2().p_3().rounded(px(8.0)).bg(rgb(ui_theme::CONFLICT_GUTTER_BG))
                .child(div().text_size(px(12.0)).text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(if self.snapshot.as_ref().is_some_and(|s| s.rebase_in_progress) {khaslana::ui_i18n::text(messages::REBASE_IN_PROGRESS)} else if self.merge_in_progress() {khaslana::ui_i18n::text(messages::MERGE_IN_PROGRESS)} else {khaslana::ui_i18n::text(messages::RESOLVING_CONFLICTS)}))
                .child(div().mt_1().text_size(px(11.0)).text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(khaslana::ui_i18n::format(messages::CONFLICTS_FILES_AWAITING_RESOLUTION, &[("conflicts", &(conflicts.len()).to_string())]))))
            .child(scrollable_uniform_frame("conflict-files-scroll", ScrollbarMode::Vertical,
                list.into_any_element(), handle, true, cx))
            .when(self.merge_in_progress(), |rail| rail.child(
                div().flex_none().flex().flex_col().gap_2().p_3()
                    .child(div().text_size(px(11.0)).text_color(rgb(ui_theme::CONTENT_TERTIARY))
                        .child(khaslana::ui_i18n::text(messages::APPLIES_TO_THE_CURRENT_MERGE_ACROSS_THE_ENTIRE_REPOSITORY)))
                    .child(self.danger_button(khaslana::ui_i18n::text(messages::ABORT_MERGE), !self.busy,
                        |this, _, cx| { this.open_abort_merge_confirm_dialog(); cx.notify(); }, cx)),
            ))
    }

    fn conflict_file_row(
        &self,
        path: String,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let view = self.conflict_workbench.files.get(&path);
        let badge = match view.map(|view| view.kind) {
            Some(ConflictFileKind::Text) => khaslana::ui_i18n::text(messages::TEXT),
            Some(ConflictFileKind::Binary) => khaslana::ui_i18n::text(messages::BINARY),
            Some(ConflictFileKind::Unsupported) => khaslana::ui_i18n::text(messages::FALLBACK),
            Some(ConflictFileKind::WorktreeOnly) => khaslana::ui_i18n::text(messages::WORKING_TREE),
            None if self.conflict_workbench.load_errors.contains_key(&path) => khaslana::ui_i18n::text(messages::READ_FAILED),
            None => khaslana::ui_i18n::text(messages::LOADING),
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
        let tooltip_path = path.clone();
        let filename = path.rsplit('/').next().unwrap_or(&path).to_owned();
        let directory = path.rsplit_once('/').map(|(directory, _)| directory.to_owned()).unwrap_or_default();
        let status = match view.map(|view| view.kind) {
            Some(ConflictFileKind::Text) => khaslana::ui_i18n::format(messages::UNRESOLVED_UNRESOLVED, &[("unresolved", &(unresolved).to_string())]),
            Some(ConflictFileKind::Binary) => khaslana::ui_i18n::text(messages::BINARY_CONFLICT).into(),
            Some(ConflictFileKind::Unsupported) => khaslana::ui_i18n::text(messages::MODIFY_DELETE_OR_OTHER_CONFLICT).into(),
            Some(ConflictFileKind::WorktreeOnly) => khaslana::ui_i18n::text(messages::WORKING_TREE_CONFLICT_ONLY).into(),
            None => badge.into(),
        };

        div().h(px(86.0)).px_2().py_1().child(crate::ui::components::list_row_surface(format!("conflict-workbench-{path}"), selected)
            .flex()
            .flex_none()
            .flex_col()
            .gap_1()
            .rounded(px(8.0)).border_l_0().h(px(78.0))
            .px_3()
            .py_2()
            .cursor_pointer()
            .tooltip(move |_, cx| crate::ui::components::tooltip_text(tooltip_path.clone(), cx))
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
                    .flex_none().h(px(20.0))
                    .items_center()
                    .gap_2()
                    .child(gpui_kit::component::Icon::new(gpui_kit::assets::IconName::FileCode).size(px(14.0)).text_color(rgb(if selected {ui_theme::PRIMARY} else {ui_theme::CONTENT_SECONDARY})))
                    .child(
                        div()
                            .flex_1().min_w(px(0.0))
                            .text_size(px(ui_theme::TYPE_BODY))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(if selected {ui_theme::PRIMARY} else {ui_theme::CONTENT_PRIMARY}))
                            .truncate()
                            .child(filename),
                    ),
            )
            .child(div().flex_none().h(px(14.0)).line_height(px(14.0)).truncate().text_size(px(10.0)).text_color(rgb(ui_theme::CONTENT_TERTIARY)).child(directory))
            .child(
                div()
                    .flex()
                    .flex_none().h(px(16.0)).line_height(px(16.0))
                    .items_center()
                    .gap_2()
                    .text_size(px(ui_theme::TYPE_META))
                    .text_color(rgb(ui_theme::FEEDBACK_WARNING_TEXT))
                    .child(status)
                    .when(dirty, |this| this.child(khaslana::ui_i18n::text(messages::DRAFT_MODIFIED)))
                    .when(applied, |this| this.child(khaslana::ui_i18n::text(messages::APPLIED))),
            ))
    }

    fn render_conflict_detail(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let surfaces = conflict_workbench_surfaces();
        let selected_path = self.conflict_workbench.selected_path.clone();
        let selected_view = selected_path
            .as_ref()
            .and_then(|path| self.conflict_workbench.files.get(path));
        let title = selected_path
            .clone()
            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::SELECT_A_CONFLICT_FILE).to_string());

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .bg(rgb(surfaces.canvas))
            .rounded(px(ui_theme::RADIUS_PANEL))
            .child(self.render_conflict_header(title, selected_view, window, cx))
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
                        self.conflict_workbench.load_errors.get(path).map(|e| khaslana::ui_i18n::format(messages::READ_FAILED_E_SELECT_THIS_FILE_AGAIN_TO_RETRY, &[("e", &(e).to_string())]))
                            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::READING_THREE_WAY_VERSIONS).into()))
                        .unwrap_or_else(|| khaslana::ui_i18n::text(messages::SELECT_A_CONFLICT_FILE).into()))
                    .into_any_element(),
            })
    }

    fn render_conflict_header(&self, title: String, view: Option<&ConflictFileView>, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_approved_conflict_header(title, view, window, cx)
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
                            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::VISUAL_TEXT_EDITING_IS_NOT_SUPPORTED_FOR_THIS_CONFLICT).into()),
                    ),
            )
            .when(view.kind == ConflictFileKind::WorktreeOnly, |this| {
                this.child(self.button(
                    khaslana::ui_i18n::text(messages::OPEN_CONTAINING_FOLDER),
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
                        khaslana::ui_i18n::text(messages::CURRENT_VERSION),
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
                        khaslana::ui_i18n::text(messages::INCOMING_VERSION),
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
                        khaslana::ui_i18n::text(messages::MARK_RESOLVED),
                        !self.busy,
                        move |this, _, _| this.mark_conflict_resolved(path_for_mark.clone()),
                        cx,
                    )),
            ))
            .into_any_element()
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

    fn resolve_conflict_with_side(&mut self, path: String, side: ConflictResolutionSide) {
        self.diff = None;
        self.diff_headers_expanded = false;
        self.reset_uniform_scroll("diff-scroll");
        let label = match side {
            ConflictResolutionSide::Ours => crate::operation_ui_resources::CONFLICT_RESOLVED,
            ConflictResolutionSide::Theirs => crate::operation_ui_resources::CONFLICT_RESOLVED,
        };
        self.with_repo(label, move |service, repo| {
            service.resolve_conflict_with_side(repo, Path::new(&path), side)
        });
    }

    fn mark_conflict_resolved(&mut self, path: String) {
        self.diff = None;
        self.diff_headers_expanded = false;
        self.reset_uniform_scroll("diff-scroll");
        self.with_repo(crate::operation_ui_resources::CONFLICT_RESOLVED, move |service, repo| {
            service.mark_conflict_resolved(repo, Path::new(&path))
        });
    }

    pub(crate) fn resolve_selected_conflict_with_intellij_idea(&mut self) {
        let Some(path) = self.conflict_workbench.selected_path.clone() else {
            self.last_error = Some(khaslana::ui_i18n::encode(messages::SELECT_A_CONFLICT_FILE_FIRST, &[]));
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
        panel_section_header(khaslana::ui_i18n::format(messages::COUNT_CONFLICT_FILES_REMAIN, &[("count", &(count).to_string())]))
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
                        khaslana::ui_i18n::text(messages::CURRENT_VERSION),
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
                        khaslana::ui_i18n::text(messages::INCOMING_VERSION),
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
                        khaslana::ui_i18n::text(messages::MARK_RESOLVED),
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
