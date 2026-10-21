//! 恢复工作台只组织日志证据、目标提交和只读预览；Git 操作统一交给 actions。

#[path = "../ui_resources/recovery_view_view_messages.rs"]
mod messages;

use gpui::{
    AnyElement, Context, Div, IntoElement, KeyDownEvent, ListSizingBehavior, ScrollStrategy,
    Window, div, prelude::*, px, uniform_list,
};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use khaslana::{CommitFileChange, RecoveryObjectState, ReflogEntry, ReflogScope};

use super::state::RecoverySide;
use crate::{
    CHANGE_ROW_HEIGHT, DiffHeaderTarget, EncodingMenuTarget, FieldId, RepositoryView,
    ScrollbarMode, TopOverlayKind, change_state_badge, commit_time_label, scrollable_frame_when,
    scrollable_uniform_frame,
    ui::{
        components::{
            EmptyState, PlaceholderAlign, dialog_actions, dialog_panel_size, floating_panel,
            inline_error_bubble, list_row_surface, page_header, panel_empty_row,
            panel_section_header, tooltip_text, workbench_command_button,
        },
        theme::{self, rgb},
    },
};

const RECOVERY_ROW_HEIGHT: f32 = 56.0;
const RECOVERY_PANE_GAP: f32 = 12.0;
const RECOVERY_FILES_WIDTH: f32 = 180.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct RecoveryLayout {
    pub(super) dual_pane: bool,
    pub(super) list_width: f32,
    pub(super) detail_width: f32,
    pub(super) files_side_by_side: bool,
    pub(super) compact: bool,
}

/// 参数是主画布的逻辑像素宽度，临时切换详情不会写回应用的导航布局偏好。
pub(super) fn recovery_layout(canvas_width: f32) -> RecoveryLayout {
    let width = if canvas_width.is_finite() {
        canvas_width.max(0.0)
    } else {
        0.0
    };
    let dual_pane = width >= 1040.0;
    let list_width = if dual_pane {
        (width * 0.4).clamp(320.0, 440.0)
    } else {
        width
    };
    let detail_width = if dual_pane {
        width - list_width - RECOVERY_PANE_GAP
    } else {
        width
    };
    RecoveryLayout {
        dual_pane,
        list_width,
        detail_width,
        files_side_by_side: detail_width >= 560.0,
        compact: false,
    }
}

pub(super) fn recovery_is_compact(window_height: f32) -> bool {
    !window_height.is_finite() || window_height < 680.0
}

/// 导航停靠列和收起窄条都有一个壳层间隙；覆盖式导航不额外挤占画布。
pub(super) fn recovery_canvas_width(viewport_width: f32, navigator_width: f32) -> f32 {
    if !viewport_width.is_finite() || !navigator_width.is_finite() {
        return 0.0;
    }
    (viewport_width - crate::chrome_view::SHELL_PADDING * 3.0 - navigator_width.max(0.0)).max(0.0)
}

fn short_oid(oid: &str) -> String {
    if !oid.is_empty() && oid.bytes().all(|byte| byte == b'0') {
        return khaslana::ui_i18n::text(messages::NO_COMMIT).into();
    }
    oid.chars().take(8).collect()
}

fn object_status(state: Option<&RecoveryObjectState>) -> &'static str {
    match state {
        Some(RecoveryObjectState::Available) => khaslana::ui_i18n::text(messages::AVAILABLE),
        Some(RecoveryObjectState::Zero) => khaslana::ui_i18n::text(messages::NO_COMMIT),
        Some(RecoveryObjectState::Missing) => khaslana::ui_i18n::text(messages::UNAVAILABLE_LOCALLY),
        Some(RecoveryObjectState::NonCommit) => khaslana::ui_i18n::text(messages::NOT_A_COMMIT_OBJECT),
        Some(RecoveryObjectState::Unreadable(_)) => khaslana::ui_i18n::text(messages::READ_FAILED),
        None => khaslana::ui_i18n::text(messages::NOT_CHECKED),
    }
}

fn object_explanation(state: Option<&RecoveryObjectState>) -> String {
    match state {
        Some(RecoveryObjectState::Available) => {
            khaslana::ui_i18n::text(messages::THE_COMMIT_OBJECT_IS_AVAILABLE_VIEW_IT_OR_CREATE_A).into()
        }
        Some(RecoveryObjectState::Zero) => {
            khaslana::ui_i18n::text(messages::THERE_IS_NO_COMMIT_ON_THIS_SIDE_A_ZERO_OID).into()
        }
        Some(RecoveryObjectState::Missing) => {
            khaslana::ui_i18n::text(messages::THE_COMMIT_OBJECT_IS_UNAVAILABLE_LOCALLY_IT_MAY_HAVE_BEEN)
                .into()
        }
        Some(RecoveryObjectState::NonCommit) => {
            khaslana::ui_i18n::text(messages::THIS_OID_DOES_NOT_POINT_TO_A_COMMIT_AND_CANNOT).into()
        }
        Some(RecoveryObjectState::Unreadable(error)) => khaslana::ui_i18n::format(messages::FAILED_TO_READ_COMMIT_OBJECT_ERROR, &[("error", &khaslana::ui_i18n::resolve(error))]),
        None => khaslana::ui_i18n::text(messages::LOCAL_COMMIT_AVAILABILITY_HAS_NOT_BEEN_CHECKED_WAIT_FOR_VERIFICATION).into(),
    }
}

fn meta(label: &'static str, value: String, monospace: bool) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_none()
        .min_w(px(0.0))
        .gap_1()
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::CONTENT_SECONDARY))
                .child(label),
        )
        .child(
            div()
                .min_w(px(0.0))
                .text_size(px(12.0))
                .line_height(px(18.0))
                .text_color(rgb(theme::CONTENT_PRIMARY))
                .when(monospace, |this| this.font_family("Consolas"))
                .child(value),
        )
}

impl RepositoryView {
    pub(crate) fn render_history_section_tabs(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .gap_2()
            .pb_2()
            .child(
                workbench_command_button(
                    "history-section-commits",
                    khaslana::ui_i18n::text(messages::COMMIT_HISTORY),
                    !self.recovery.show_recovery,
                    true,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| this.set_history_recovery(false, cx))),
            )
            .child(
                workbench_command_button(
                    "history-section-recovery",
                    khaslana::ui_i18n::text(messages::RECOVERY_RECORDS),
                    self.recovery.show_recovery,
                    true,
                    cx,
                )
                .tooltip(khaslana::ui_i18n::text(messages::VIEW_THE_CURRENT_WORKTREE_S_LOCAL_REFERENCE_LOGS))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.set_history_recovery(true, cx);
                    window.focus(&this.recovery.list_focus, cx);
                })),
            )
            .into_any_element()
    }

    pub(crate) fn render_recovery_view(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let navigator_width = if self.context_navigator_presentation(window)
            == crate::chrome_view::ContextNavigatorPresentation::Docked
        {
            self.sidebar_width
        } else {
            theme::NAVIGATOR_COLLAPSED_WIDTH
        };
        let mut layout = recovery_layout(recovery_canvas_width(
            window.viewport_size().width.into(),
            navigator_width,
        ));
        layout.compact = recovery_is_compact(window.viewport_size().height.into());
        let mut root = div()
            .id("recovery-workbench")
            .key_context("RecoveryWorkbench")
            .track_focus(&self.recovery.list_focus)
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .gap(px(if layout.compact { 8.0 } else { 12.0 }))
            .on_key_down(cx.listener(Self::recovery_panel_key));
        if self.repo_path.is_none() {
            return root
                .child(
                    floating_panel()
                        .flex()
                        .flex_1()
                        .min_w(px(0.0))
                        .min_h(px(0.0))
                        .child(
                            EmptyState::new(khaslana::ui_i18n::text(messages::NO_REPOSITORY_OPEN))
                                .detail(khaslana::ui_i18n::text(messages::OPEN_A_LOCAL_REPOSITORY_TO_VIEW_RECOVERY_RECORDS_FOR_THE))
                                .fill()
                                .action(
                                    self.button(
                                        khaslana::ui_i18n::text(messages::OPEN_REPOSITORY),
                                        true,
                                        |this, _, _| this.browse_open(),
                                        cx,
                                    )
                                    .into_any_element(),
                                )
                                .build(),
                        ),
                )
                .into_any_element();
        }
        root = root.child(self.render_history_section_tabs(cx)).child(
            floating_panel()
                .flex()
                .flex_col()
                .flex_none()
                .min_w(px(0.0))
                .when(!layout.compact, |this| {
                    this.child(page_header(khaslana::ui_i18n::text(messages::RECOVERY_RECORDS), Some(khaslana::ui_i18n::text(messages::LOCATE_COMMITS_USING_LOCAL_REFERENCE_LOGS))))
                })
                .child(self.render_recovery_toolbar(window, cx)),
        );
        if !layout.dual_pane {
            root = root.child(
                div()
                    .flex()
                    .flex_none()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        workbench_command_button(
                            "recovery-narrow-records",
                            khaslana::ui_i18n::text(messages::RECORDS),
                            !self.recovery.narrow_details,
                            true,
                            cx,
                        )
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.recovery.narrow_details = false;
                            window.focus(&this.recovery.list_focus, cx);
                            cx.notify();
                        })),
                    )
                    .child(
                        workbench_command_button(
                            "recovery-narrow-details",
                            khaslana::ui_i18n::text(messages::DETAILS),
                            self.recovery.narrow_details,
                            self.recovery.selected_entry().is_some(),
                            cx,
                        )
                        .tooltip(if self.recovery.selected_entry().is_some() {
                            khaslana::ui_i18n::text(messages::VIEW_COMMITS_BEFORE_AND_AFTER_THE_SELECTED_OPERATION)
                        } else {
                            khaslana::ui_i18n::text(messages::SELECT_A_RECOVERY_RECORD_FIRST)
                        })
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.recovery.narrow_details = true;
                            window.focus(&this.recovery.list_focus, cx);
                            cx.notify();
                        })),
                    ),
            );
        }
        root.child(
            div()
                .flex()
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(0.0))
                .gap(px(RECOVERY_PANE_GAP))
                .when(layout.dual_pane || !self.recovery.narrow_details, |this| {
                    this.child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w(px(0.0))
                            .min_h(px(0.0))
                            .when(layout.dual_pane, |this| {
                                this.flex_none().w(px(layout.list_width))
                            })
                            .when(!layout.dual_pane, |this| this.flex_1())
                            .child(self.render_recovery_records(cx)),
                    )
                })
                .when(layout.dual_pane || self.recovery.narrow_details, |this| {
                    this.child(self.render_recovery_details(layout, cx))
                }),
        )
        .child(
            div()
                .flex_none()
                .text_size(px(11.0))
                .line_height(px(17.0))
                .text_color(rgb(theme::CONTENT_SECONDARY))
                .child(khaslana::ui_i18n::text(messages::RECOVERY_RECORDS_ARE_NOT_PERMANENT_BACKUPS_OBJECTS_MAY_BE_PRUNED)),
        )
        .into_any_element()
    }

    fn recovery_panel_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // 只处理面板自身焦点，不借冒泡事件接管 Kit 输入、按钮或模态键盘。
        if !self.recovery.list_focus.is_focused(window)
            || self.top_overlay_kind() != TopOverlayKind::None
        {
            return;
        }
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.platform || modifiers.alt || modifiers.shift {
            return;
        }
        match event.keystroke.key.as_str() {
            "up" | "down" if !self.recovery.entries.is_empty() => {
                let selected = self
                    .recovery
                    .entries
                    .iter()
                    .position(|entry| Some(&entry.id) == self.recovery.selected.as_ref());
                let index = match (selected, event.keystroke.key.as_str()) {
                    (Some(index), "up") => index.saturating_sub(1),
                    (Some(index), _) => (index + 1).min(self.recovery.entries.len() - 1),
                    (None, "up") => self.recovery.entries.len() - 1,
                    (None, _) => 0,
                };
                let narrow_details = self.recovery.narrow_details;
                self.select_recovery_entry(index, cx);
                self.recovery.narrow_details = narrow_details;
                self.uniform_scroll_handle("recovery-record-list")
                    .scroll_to_item(index, ScrollStrategy::Center);
            }
            "enter" if !self.recovery.entries.is_empty() => {
                if self.recovery.selected_entry().is_none() {
                    self.select_recovery_entry(0, cx);
                }
                self.recovery.narrow_details = true;
            }
            "escape" if self.recovery.narrow_details => self.recovery.narrow_details = false,
            _ => return,
        }
        cx.stop_propagation();
        cx.notify();
    }

    fn render_recovery_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        if recovery_is_compact(window.viewport_size().height.into()) {
            return self.render_recovery_compact_toolbar(window, cx);
        }
        let enabled = !self.busy && self.recovery.operation.is_none() && !self.recovery.loading;
        let branch_names = self
            .recovery
            .sources
            .as_ref()
            .map(|sources| sources.branches.clone())
            .unwrap_or_default();
        let current_branch = self
            .recovery
            .sources
            .as_ref()
            .and_then(|sources| sources.current_branch.as_deref());
        let branch_label = match &self.recovery.scope {
            ReflogScope::LocalBranch(branch) => khaslana::ui_i18n::format(messages::BRANCH_BRANCH, &[("branch", &(branch).to_string())]),
            _ => khaslana::ui_i18n::text(messages::SPECIFIC_LOCAL_BRANCH).into(),
        };
        let branch_tooltip = if branch_names.is_empty() {
            khaslana::ui_i18n::text(messages::NO_LOCAL_BRANCHES_ARE_AVAILABLE_IN_THE_CURRENT_WORKTREE).into()
        } else {
            branch_label.clone()
        };
        let scope = self.recovery.scope.clone();
        let entity = cx.entity();
        let tab_id = self.active_tab_id();
        let repo_path = self.repo_path.clone();
        let branch_menu = workbench_command_button(
            "recovery-source-branch",
            branch_label,
            matches!(scope, ReflogScope::LocalBranch(_)),
            enabled && !branch_names.is_empty(),
            cx,
        )
        .max_w(px(230.0))
        .dropdown_caret(true)
        .tooltip(branch_tooltip)
        .dropdown_menu(move |mut menu, window, _| {
            menu = menu.scrollable(true).max_h(px(280.0)).max_w(px(360.0));
            for name in &branch_names {
                let name = name.clone();
                let target = ReflogScope::LocalBranch(name.clone());
                let checked = target == scope;
                let repo_path = repo_path.clone();
                menu = menu.item(PopupMenuItem::new(name).checked(checked).on_click(
                    window.listener_for(&entity, move |this, _, _, cx| {
                        if this.active_tab_id() == tab_id && this.repo_path == repo_path {
                            this.set_recovery_scope(target.clone(), cx);
                        }
                    }),
                ));
            }
            menu
        });
        let mut scopes = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .min_w(px(0.0))
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(rgb(theme::CONTENT_SECONDARY))
                    .child(khaslana::ui_i18n::text(messages::SOURCE)),
            );
        for (id, label, scope) in [
            ("recovery-source-head", "HEAD", ReflogScope::Head),
            (
                "recovery-source-current",
                khaslana::ui_i18n::text(messages::CURRENT_BRANCH),
                ReflogScope::CurrentBranch,
            ),
            ("recovery-source-all", khaslana::ui_i18n::text(messages::ALL), ReflogScope::All),
        ] {
            let available =
                enabled && (scope != ReflogScope::CurrentBranch || current_branch.is_some());
            let tooltip = if scope == ReflogScope::CurrentBranch {
                current_branch
                    .map(|branch| khaslana::ui_i18n::format(messages::CURRENT_BRANCH_BRANCH, &[("branch", &(branch).to_string())]))
                    .unwrap_or_else(|| khaslana::ui_i18n::text(messages::HEAD_IS_DETACHED_THERE_IS_NO_CURRENT_BRANCH_LOG).into())
            } else if scope == ReflogScope::All {
                khaslana::ui_i18n::text(messages::HEAD_AND_ALL_LOCAL_BRANCH_LOGS).into()
            } else {
                khaslana::ui_i18n::text(messages::CURRENT_WORKTREE_HEAD_LOG).into()
            };
            scopes = scopes.child(
                workbench_command_button(id, label, self.recovery.scope == scope, available, cx)
                    .tooltip(tooltip)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_recovery_scope(scope.clone(), cx)
                    })),
            );
        }
        scopes = scopes.child(branch_menu);
        let entity = cx.entity();
        let selected_days = self.recovery.days;
        let tab_id = self.active_tab_id();
        let repo_path = self.repo_path.clone();
        let days_label = selected_days
            .map(|days| khaslana::ui_i18n::format(messages::LAST_DAYS_DAYS, &[("days", &(days).to_string())]))
            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::ALL_TIME).into());
        let days_menu = workbench_command_button("recovery-days", days_label, false, enabled, cx)
            .dropdown_caret(true)
            .tooltip(khaslana::ui_i18n::text(messages::FILTER_BY_WHEN_THE_RECOVERY_OPERATION_OCCURRED))
            .dropdown_menu(move |mut menu, window, _| {
                for (label, days) in [
                    (khaslana::ui_i18n::text(messages::LAST_DAYS), Some(7)),
                    (khaslana::ui_i18n::text(messages::LAST_DAYS_2), Some(30)),
                    (khaslana::ui_i18n::text(messages::LAST_DAYS_3), Some(90)),
                    (khaslana::ui_i18n::text(messages::ALL_TIME), None),
                ] {
                    let repo_path = repo_path.clone();
                    menu = menu.item(
                        PopupMenuItem::new(label)
                            .checked(selected_days == days)
                            .on_click(window.listener_for(&entity, move |this, _, _, cx| {
                                if this.active_tab_id() == tab_id && this.repo_path == repo_path {
                                    this.set_recovery_days(days, cx);
                                }
                            })),
                    );
                }
                menu
            });
        let mut toolbar = div()
            .flex()
            .flex_col()
            .flex_none()
            .min_w(px(0.0))
            .px_3()
            .py_2()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .min_w(px(0.0))
                    .child(scopes)
                    .child(days_menu),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .min_w(px(0.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(180.0))
                            .max_w(px(420.0))
                            .child(self.input(FieldId::ReflogSearch, true, window, cx)),
                    )
                    .child(
                        workbench_command_button(
                            "recovery-search",
                            khaslana::ui_i18n::text(messages::SEARCH),
                            false,
                            enabled && !self.recovery.querying,
                            cx,
                        )
                        .tooltip(khaslana::ui_i18n::text(messages::SEARCH_LOADED_LOCAL_LOGS))
                        .on_click(
                            cx.listener(|this, _, _, cx| this.refresh_recovery_query(cx, false)),
                        ),
                    )
                    .child(
                        workbench_command_button(
                            "recovery-refresh",
                            if self.recovery.loading {
                                khaslana::ui_i18n::text(messages::READING)
                            } else {
                                khaslana::ui_i18n::text(messages::REFRESH)
                            },
                            false,
                            enabled,
                            cx,
                        )
                        .tooltip(if self.recovery.loading {
                            khaslana::ui_i18n::text(messages::READING_LOCAL_RECOVERY_RECORDS)
                        } else {
                            khaslana::ui_i18n::text(messages::RELOAD_LOCAL_RECOVERY_RECORDS)
                        })
                        .on_click(cx.listener(|this, _, _, cx| this.load_recovery(cx))),
                    )
                    .when(self.recovery.loading, |this| {
                        this.child(
                            workbench_command_button(
                                "recovery-cancel-read",
                                khaslana::ui_i18n::text(messages::CANCEL_READ),
                                false,
                                true,
                                cx,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.cancel_recovery_read();
                                cx.notify();
                            })),
                        )
                    }),
            );
        if let Some(name) = self.recovery.last_created.clone() {
            let tooltip = khaslana::ui_i18n::format(messages::VIEW_LOCAL_RECOVERY_BRANCH_NAME, &[("name", &(name).to_string())]);
            toolbar = toolbar.child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(theme::FEEDBACK_SUCCESS_TEXT))
                            .child(khaslana::ui_i18n::format(messages::LOCAL_BRANCH_CREATED_NAME, &[("name", &(name).to_string())])),
                    )
                    .child(
                        workbench_command_button(
                            "recovery-browse-created",
                            khaslana::ui_i18n::text(messages::VIEW_RECOVERY_BRANCH),
                            false,
                            !self.busy,
                            cx,
                        )
                        .tooltip(tooltip)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.open_browse_branch(name.clone(), khaslana::BranchKind::Local);
                            cx.notify();
                        })),
                    ),
            );
        }
        if let Some(error) = &self.recovery.error {
            let tooltip = khaslana::ui_i18n::resolve(&error);
            toolbar = toolbar.child(
                div()
                    .id("recovery-read-error")
                    .min_w(px(0.0))
                    .tooltip(move |_, cx| tooltip_text(tooltip.clone(), cx))
                    .child(inline_error_bubble(khaslana::ui_i18n::resolve(error))),
            );
        }
        if let Some(snapshot) = &self.recovery.snapshot
            && let Some(diagnostic) = snapshot.diagnostics.first()
        {
            let text = if snapshot.diagnostics.len() == 1 {
                diagnostic.clone()
            } else {
                khaslana::ui_i18n::format(messages::DIAGNOSTIC_DIAGNOSTICS_MORE, &[("diagnostic", &(diagnostic).to_string()), ("diagnostics", &(snapshot.diagnostics.len() - 1).to_string())])
            };
            let tooltip = snapshot.diagnostics.join("\n");
            toolbar = toolbar.child(
                div()
                    .id("recovery-read-diagnostics")
                    .min_w(px(0.0))
                    .text_size(px(11.0))
                    .text_color(rgb(theme::FEEDBACK_WARNING_TEXT))
                    .truncate()
                    .tooltip(move |_, cx| tooltip_text(tooltip.clone(), cx))
                    .child(text),
            );
        }
        toolbar.into_any_element()
    }

    fn render_recovery_compact_toolbar(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let enabled = !self.busy && !self.recovery.loading && !self.recovery.has_operation();
        let scope = self.recovery.scope.clone();
        let label = match &scope {
            ReflogScope::Head => khaslana::ui_i18n::text(messages::SOURCE_HEAD).into(),
            ReflogScope::CurrentBranch => khaslana::ui_i18n::text(messages::SOURCE_CURRENT_BRANCH).into(),
            ReflogScope::All => khaslana::ui_i18n::text(messages::SOURCE_ALL).into(),
            ReflogScope::LocalBranch(name) => khaslana::ui_i18n::format(messages::SOURCE_NAME, &[("name", &(name).to_string())]),
        };
        let mut choices = vec![
            ("HEAD".to_string(), ReflogScope::Head, true),
            (
                khaslana::ui_i18n::text(messages::CURRENT_BRANCH).into(),
                ReflogScope::CurrentBranch,
                self.recovery
                    .sources
                    .as_ref()
                    .and_then(|sources| sources.current_branch.as_ref())
                    .is_some(),
            ),
            (khaslana::ui_i18n::text(messages::ALL_LOCAL_SOURCES).into(), ReflogScope::All, true),
        ];
        if let Some(sources) = &self.recovery.sources {
            choices.extend(
                sources
                    .branches
                    .iter()
                    .map(|name| (name.clone(), ReflogScope::LocalBranch(name.clone()), true)),
            );
        }
        let entity = cx.entity();
        let tab_id = self.active_tab_id();
        let repo_path = self.repo_path.clone();
        let source = workbench_command_button("recovery-compact-source", label, false, enabled, cx)
            .max_w(px(180.0))
            .dropdown_caret(true)
            .tooltip(khaslana::ui_i18n::text(messages::CHOOSE_HEAD_THE_CURRENT_BRANCH_OR_ANOTHER_LOCAL_SOURCE))
            .dropdown_menu(move |mut menu, window, _| {
                menu = menu.scrollable(true).max_h(px(280.0)).max_w(px(360.0));
                for (label, target, available) in &choices {
                    let target = target.clone();
                    let checked = target == scope;
                    let repo_path = repo_path.clone();
                    menu = menu.item(
                        PopupMenuItem::new(label.clone())
                            .checked(checked)
                            .disabled(!available)
                            .on_click(window.listener_for(&entity, move |this, _, _, cx| {
                                if this.active_tab_id() == tab_id && this.repo_path == repo_path {
                                    this.set_recovery_scope(target.clone(), cx);
                                }
                            })),
                    );
                }
                menu
            });
        let entity = cx.entity();
        let repo_path = self.repo_path.clone();
        let selected_days = self.recovery.days;
        let days_label = selected_days
            .map(|days| khaslana::ui_i18n::format(messages::LAST_DAYS_DAYS, &[("days", &(days).to_string())]))
            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::ALL_TIME).into());
        let days =
            workbench_command_button("recovery-compact-days", days_label, false, enabled, cx)
                .dropdown_caret(true)
                .tooltip(khaslana::ui_i18n::text(messages::FILTER_RECOVERY_OPERATION_TIME))
                .dropdown_menu(move |mut menu, window, _| {
                    for (label, days) in [
                        (khaslana::ui_i18n::text(messages::LAST_DAYS), Some(7)),
                        (khaslana::ui_i18n::text(messages::LAST_DAYS_2), Some(30)),
                        (khaslana::ui_i18n::text(messages::LAST_DAYS_3), Some(90)),
                        (khaslana::ui_i18n::text(messages::ALL_TIME), None),
                    ] {
                        let repo_path = repo_path.clone();
                        menu = menu.item(
                            PopupMenuItem::new(label)
                                .checked(selected_days == days)
                                .on_click(window.listener_for(&entity, move |this, _, _, cx| {
                                    if this.active_tab_id() == tab_id && this.repo_path == repo_path
                                    {
                                        this.set_recovery_days(days, cx);
                                    }
                                })),
                        );
                    }
                    menu
                });
        let entity = cx.entity();
        let repo_path = self.repo_path.clone();
        let created = self.recovery.last_created.clone();
        let loading = self.recovery.loading;
        let actions = workbench_command_button("recovery-compact-actions", khaslana::ui_i18n::text(messages::ACTIONS), false, true, cx)
            .dropdown_caret(true)
            .tooltip(khaslana::ui_i18n::text(messages::SEARCH_REFRESH_AND_VIEW_CREATED_BRANCHES))
            .dropdown_menu(move |mut menu, window, _| {
                for (label, action, available) in [
                    (khaslana::ui_i18n::text(messages::SEARCH), 0, enabled),
                    (khaslana::ui_i18n::text(messages::REFRESH), 1, enabled),
                    (khaslana::ui_i18n::text(messages::CANCEL_READ), 2, loading),
                ] {
                    let repo_path = repo_path.clone();
                    menu = menu.item(PopupMenuItem::new(label).disabled(!available).on_click(
                        window.listener_for(&entity, move |this, _, _, cx| {
                            if this.active_tab_id() != tab_id || this.repo_path != repo_path {
                                return;
                            }
                            match action {
                                0 => this.refresh_recovery_query(cx, false),
                                1 => this.load_recovery(cx),
                                _ => this.cancel_recovery_read(),
                            }
                            cx.notify();
                        }),
                    ));
                }
                if let Some(name) = created.clone() {
                    let repo_path = repo_path.clone();
                    menu = menu.item(PopupMenuItem::new(khaslana::ui_i18n::format(messages::VIEW_RECOVERY_BRANCH_NAME, &[("name", &(name).to_string())])).on_click(
                        window.listener_for(&entity, move |this, _, _, cx| {
                            if this.active_tab_id() == tab_id
                                && this.repo_path == repo_path
                                && !this.busy
                            {
                                this.open_browse_branch(name.clone(), khaslana::BranchKind::Local);
                                cx.notify();
                            }
                        }),
                    ));
                }
                menu
            });
        let mut toolbar = div()
            .flex()
            .flex_col()
            .flex_none()
            .min_w(px(0.0))
            .px_3()
            .py_2()
            .gap_1()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .min_w(px(0.0))
                    .child(source)
                    .child(days)
                    .child(div().flex_1().min_w(px(160.0)).child(self.input(
                        FieldId::ReflogSearch,
                        true,
                        window,
                        cx,
                    )))
                    .child(actions),
            );
        if let Some(error) = &self.recovery.error {
            let tooltip = khaslana::ui_i18n::resolve(&error);
            toolbar = toolbar.child(
                div()
                    .id("recovery-compact-error")
                    .min_w(px(0.0))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .tooltip(move |_, cx| tooltip_text(tooltip.clone(), cx))
                    .child(inline_error_bubble(khaslana::ui_i18n::resolve(error))),
            );
        } else if let Some(snapshot) = &self.recovery.snapshot
            && let Some(diagnostic) = snapshot.diagnostics.first()
        {
            let tooltip = snapshot.diagnostics.join("\n");
            toolbar = toolbar.child(
                div()
                    .id("recovery-compact-diagnostic")
                    .min_w(px(0.0))
                    .text_size(px(11.0))
                    .text_color(rgb(theme::FEEDBACK_WARNING_TEXT))
                    .truncate()
                    .tooltip(move |_, cx| tooltip_text(tooltip.clone(), cx))
                    .child(diagnostic.clone()),
            );
        }
        toolbar.into_any_element()
    }

    fn render_recovery_records(&self, cx: &mut Context<Self>) -> AnyElement {
        let handle = self.uniform_scroll_handle("recovery-record-list");
        let list_handle = handle.clone();
        let content_present = !self.recovery.entries.is_empty();
        let content = div()
            .id("recovery-record-list")
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .py_1()
            .child(
                uniform_list(
                    "recovery-record-list",
                    self.recovery.entries.len().max(1),
                    cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                        range
                            .map(|index| this.recovery_record_row(index, cx))
                            .collect::<Vec<_>>()
                    }),
                )
                .track_scroll(&list_handle)
                .with_sizing_behavior(ListSizingBehavior::Auto)
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(0.0)),
            )
            .into_any_element();
        floating_panel()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .child(
                panel_section_header(khaslana::ui_i18n::text(messages::RECOVERY_RECORDS))
                    .action(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(theme::CONTENT_SECONDARY))
                            .child(if self.recovery.querying {
                                khaslana::ui_i18n::text(messages::FILTERING).into()
                            } else {
                                khaslana::ui_i18n::format(messages::TOTAL_RECORDS, &[("total", &(self.recovery.total).to_string())])
                            })
                            .into_any_element(),
                    )
                    .top_rounded()
                    .build(),
            )
            .child(scrollable_uniform_frame(
                "recovery-record-list",
                ScrollbarMode::Vertical,
                content,
                handle,
                content_present,
                cx,
            ))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(theme::CONTENT_SECONDARY))
                            .child(khaslana::ui_i18n::format(messages::SHOWING_ENTRIES_TOTAL_RECORDS, &[("entries", &(self.recovery.entries.len()).to_string()), ("total", &(self.recovery.total).to_string())])),
                    )
                    .when(self.recovery.next_offset.is_some(), |this| {
                        this.child(
                            workbench_command_button(
                                "recovery-load-more",
                                if self.recovery.querying {
                                    khaslana::ui_i18n::text(messages::LOADING)
                                } else {
                                    khaslana::ui_i18n::text(messages::LOAD_MORE)
                                },
                                false,
                                !self.recovery.loading
                                    && !self.recovery.querying
                                    && self.recovery.search_changed.is_none(),
                                cx,
                            )
                            .tooltip(khaslana::ui_i18n::text(messages::LOAD_UP_TO_RECORDS_AT_A_TIME))
                            .on_click(
                                cx.listener(|this, _, _, cx| this.refresh_recovery_query(cx, true)),
                            ),
                        )
                    }),
            )
            .into_any_element()
    }

    fn recovery_record_row(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(entry) = self.recovery.entries.get(index) else {
            let text = if self.recovery.loading {
                khaslana::ui_i18n::text(messages::READING_LOCAL_RECOVERY_RECORDS_2)
            } else if self.recovery.querying {
                khaslana::ui_i18n::text(messages::FILTERING_RECOVERY_RECORDS)
            } else if self.recovery.error.is_some() {
                khaslana::ui_i18n::text(messages::READ_INCOMPLETE_REFRESH_TO_TRY_AGAIN)
            } else {
                khaslana::ui_i18n::text(messages::NO_RECOVERY_RECORDS_MATCH_THE_CURRENT_SOURCE_AND_FILTERS)
            };
            return panel_empty_row(text, RECOVERY_ROW_HEIGHT, PlaceholderAlign::Start)
                .into_any_element();
        };
        let selected = self.recovery.selected.as_deref() == Some(entry.id.as_str());
        let old_state = self.recovery.availability.get(&entry.old_oid);
        let new_state = self.recovery.availability.get(&entry.new_oid);
        let tooltip = khaslana::ui_i18n::format(
            if entry.lossy { messages::RECORD_TOOLTIP_LOSSY } else { messages::RECORD_TOOLTIP },
            &[("message", &entry.message), ("timestamp", &commit_time_label(entry.timestamp)),
              ("reference", &entry.ref_name), ("before", &entry.old_oid), ("before_state", object_status(old_state)),
              ("after", &entry.new_oid), ("after_state", object_status(new_state))],
        );
        let id = entry.id.clone();
        list_row_surface(format!("recovery-record-{}", entry.id), selected)
            .flex()
            .flex_col()
            .flex_none()
            .w_full()
            .min_w(px(0.0))
            .h(px(RECOVERY_ROW_HEIGHT))
            .justify_center()
            .px_3()
            .gap(px(1.0))
            .overflow_hidden()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, window, cx| {
                if let Some(index) = this
                    .recovery
                    .entries
                    .iter()
                    .position(|entry| entry.id == id)
                {
                    let narrow_details = this.recovery.narrow_details;
                    this.select_recovery_entry(index, cx);
                    this.recovery.narrow_details = narrow_details;
                    window.focus(&this.recovery.list_focus, cx);
                    cx.notify();
                }
            }))
            .tooltip(move |_, cx| tooltip_text(tooltip.clone(), cx))
            .child(
                div()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(theme::CONTENT_PRIMARY))
                    .child(if entry.message.is_empty() {
                        khaslana::ui_i18n::text(messages::NO_OPERATION_DESCRIPTION).into()
                    } else {
                        entry.message.clone()
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_size(px(10.0))
                    .line_height(px(13.0))
                    .text_color(rgb(theme::CONTENT_SECONDARY))
                    .child(div().flex_none().child(commit_time_label(entry.timestamp)))
                    .child(
                        div()
                            .min_w(px(0.0))
                            .overflow_hidden()
                            .child(entry.ref_name.clone()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_size(px(10.0))
                    .line_height(px(13.0))
                    .text_color(rgb(theme::CONTENT_SECONDARY))
                    .child(
                        div()
                            .font_family("Consolas")
                            .child(short_oid(&entry.old_oid)),
                    )
                    .child(div().child("→"))
                    .child(
                        div()
                            .font_family("Consolas")
                            .child(short_oid(&entry.new_oid)),
                    )
                    .child(
                        div()
                            .min_w(px(0.0))
                            .overflow_hidden()
                            .text_color(rgb(
                                if matches!(old_state, Some(RecoveryObjectState::Available))
                                    && matches!(new_state, Some(RecoveryObjectState::Available))
                                {
                                    theme::CONTENT_SECONDARY
                                } else {
                                    theme::FEEDBACK_WARNING_TEXT
                                },
                            ))
                            .child(khaslana::ui_i18n::format(messages::BEFORE_OLD_STATE_AFTER_NEW_STATE, &[("old_state", &(object_status(old_state)).to_string()), ("new_state", &(object_status(new_state)).to_string())])),
                    ),
            )
            .into_any_element()
    }

    fn render_recovery_details(
        &self,
        layout: RecoveryLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let preview_open = self.recovery.preview.open;
        if layout.compact && preview_open {
            return self.render_recovery_compact_preview(layout, cx);
        }
        let mut header = panel_section_header(if preview_open {
            khaslana::ui_i18n::text(messages::COMMIT_PREVIEW)
        } else {
            khaslana::ui_i18n::text(messages::RECOVERY_DETAILS)
        })
        .top_rounded();
        if preview_open {
            header = header.action(
                workbench_command_button("recovery-preview-close", khaslana::ui_i18n::text(messages::BACK_TO_DETAILS), false, true, cx)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.recovery.preview.open = false;
                        cx.notify();
                    }))
                    .into_any_element(),
            );
        }
        let mut panel = floating_panel()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .child(header.build());
        let Some(entry) = self.recovery.selected_entry() else {
            return panel
                .child(
                    EmptyState::new(khaslana::ui_i18n::text(messages::SELECT_A_RECOVERY_RECORD))
                        .detail(khaslana::ui_i18n::text(messages::COMPARE_FULL_OIDS_BEFORE_AND_AFTER_THE_OPERATION_VERIFY_THE))
                        .fill()
                        .build(),
                )
                .into_any_element();
        };
        let oid = self.recovery.side.oid(entry).to_string();
        let details = self
            .recovery
            .details
            .as_ref()
            .filter(|details| details.oid == oid);
        let available = !self.recovery.details_loading
            && details.is_some_and(|details| {
                details.state == RecoveryObjectState::Available && details.commit.is_some()
            });
        let operation_busy = self.busy || self.recovery.operation.is_some();
        let reason = if self.recovery.details_loading {
            khaslana::ui_i18n::text(messages::READING_TARGET_COMMIT_PLEASE_WAIT).into()
        } else {
            object_explanation(details.map(|details| &details.state))
        };
        let mut sides = div()
            .flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .gap_2()
            .px_3()
            .py_2();
        for side in [RecoverySide::Before, RecoverySide::After] {
            let target_oid = side.oid(entry);
            let state = self.recovery.availability.get(target_oid);
            let tooltip = format!(
                "{}：{}\n{}",
                side.label(),
                target_oid,
                object_explanation(state)
            );
            sides = sides.child(
                workbench_command_button(
                    if side == RecoverySide::Before {
                        "recovery-side-before"
                    } else {
                        "recovery-side-after"
                    },
                    format!("{} · {}", side.label(), short_oid(target_oid)),
                    self.recovery.side == side,
                    !operation_busy,
                    cx,
                )
                .tooltip(tooltip)
                .on_click(cx.listener(move |this, _, _, cx| this.select_recovery_side(side, cx))),
            );
        }
        panel = panel.child(sides);
        let copy_oid = oid.clone();
        let create_reason = if operation_busy {
            khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT).into()
        } else {
            reason.clone()
        };
        panel =
            panel.child(
                div()
                    .flex()
                    .flex_none()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .pb_2()
                    .child(
                        workbench_command_button(
                            "recovery-copy-oid",
                            khaslana::ui_i18n::text(messages::COPY_OID),
                            false,
                            !oid.is_empty(),
                            cx,
                        )
                        .tooltip(khaslana::ui_i18n::format(match self.recovery.side { RecoverySide::Before => messages::COPY_BEFORE_OID, RecoverySide::After => messages::COPY_AFTER_OID }, &[("oid", &oid)]))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.copy_commit_sha(copy_oid.clone(), cx)
                        })),
                    )
                    .child(
                        workbench_command_button(
                            "recovery-show-commit",
                            khaslana::ui_i18n::text(messages::VIEW_COMMIT),
                            false,
                            available && !operation_busy,
                            cx,
                        )
                        .tooltip(if operation_busy {
                            khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT).into()
                        } else {
                            reason
                        })
                        .on_click(cx.listener(|this, _, _, cx| this.show_recovery_commit(cx))),
                    )
                    .child(
                        div()
                            .id("recovery-create-branch-command")
                            .tooltip(move |_, cx| tooltip_text(create_reason.clone(), cx))
                            .child(self.primary_button(
                                khaslana::ui_i18n::text(messages::CREATE_BRANCH_FROM_THIS_COMMIT),
                                available && !operation_busy,
                                |this, _, cx| this.open_recovery_branch_dialog(cx),
                                cx,
                            )),
                    ),
            );
        panel
            .child(if preview_open {
                self.render_recovery_preview(layout, cx)
            } else {
                self.render_recovery_details_content(entry, cx)
            })
            .into_any_element()
    }

    fn render_recovery_compact_preview(
        &self,
        layout: RecoveryLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(entry) = self.recovery.selected_entry() else {
            return div().into_any_element();
        };
        let enabled = !self.busy && !self.recovery.has_operation();
        let oid = self.recovery.side.oid(entry).to_string();
        let available = !self.recovery.details_loading
            && self.recovery.details.as_ref().is_some_and(|details| {
                details.oid == oid
                    && details.state == RecoveryObjectState::Available
                    && details.commit.is_some()
            });
        let mut controls = div()
            .flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .child(
                workbench_command_button("recovery-compact-back", khaslana::ui_i18n::text(messages::BACK_TO_DETAILS), false, true, cx)
                    .h(px(24.0))
                    .min_h(px(24.0))
                    .py_0()
                    .px_2()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.recovery.preview.open = false;
                        cx.notify();
                    })),
            );
        for side in [RecoverySide::Before, RecoverySide::After] {
            controls = controls.child(
                workbench_command_button(
                    if side == RecoverySide::Before {
                        "recovery-compact-before"
                    } else {
                        "recovery-compact-after"
                    },
                    side.label(),
                    self.recovery.side == side,
                    enabled,
                    cx,
                )
                .h(px(24.0))
                .min_h(px(24.0))
                .py_0()
                .px_2()
                .tooltip(format!("{}：{}", side.label(), side.oid(entry)))
                .on_click(cx.listener(move |this, _, _, cx| this.select_recovery_side(side, cx))),
            );
        }
        let copy_oid = oid.clone();
        controls = controls
            .child(
                workbench_command_button("recovery-compact-copy", khaslana::ui_i18n::text(messages::COPY_OID), false, true, cx)
                    .h(px(24.0))
                    .min_h(px(24.0))
                    .py_0()
                    .px_2()
                    .tooltip(oid)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_commit_sha(copy_oid.clone(), cx)
                    })),
            )
            .child(
                crate::ui::components::settings_command_button(
                    "recovery-compact-create",
                    khaslana::ui_i18n::text(messages::CREATE_BRANCH),
                    crate::ui::components::SettingsButtonTone::Primary,
                    enabled && available,
                    cx,
                )
                .on_click(cx.listener(|this, _, _, cx| this.open_recovery_branch_dialog(cx)))
                .h(px(24.0))
                .min_h(px(24.0))
                .py_0()
                .px_2(),
            );
        floating_panel()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .child(panel_section_header(khaslana::ui_i18n::text(messages::COMMIT_PREVIEW)).top_rounded().build())
            .child(controls)
            .child(self.render_recovery_preview(layout, cx))
            .into_any_element()
    }

    fn render_recovery_details_content(
        &self,
        entry: &ReflogEntry,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let oid = self.recovery.side.oid(entry);
        let details = self
            .recovery
            .details
            .as_ref()
            .filter(|details| details.oid == oid);
        let state = details.map(|details| &details.state);
        let available =
            !self.recovery.details_loading && matches!(state, Some(RecoveryObjectState::Available));
        let handle = self.scroll_handle("recovery-details-scroll");
        let mut content = div()
            .id("recovery-details-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .overflow_y_scroll()
            .track_scroll(&handle)
            .px_3()
            .py_2()
            .gap_3()
            .child(meta(
                if self.recovery.side == RecoverySide::Before {
                    khaslana::ui_i18n::text(messages::BEFORE_OPERATION_FULL_TARGET_OID)
                } else {
                    khaslana::ui_i18n::text(messages::AFTER_OPERATION_FULL_TARGET_OID)
                },
                oid.to_string(),
                true,
            ))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(18.0))
                    .text_color(rgb(if available {
                        theme::FEEDBACK_SUCCESS_TEXT
                    } else if self.recovery.details_loading || state.is_none() {
                        theme::CONTENT_SECONDARY
                    } else {
                        theme::FEEDBACK_WARNING_TEXT
                    }))
                    .child(if self.recovery.details_loading {
                        khaslana::ui_i18n::text(messages::READING_TARGET_COMMIT).into()
                    } else {
                        object_explanation(state)
                    }),
            );
        if let Some(commit) = details
            .and_then(|details| details.commit.as_ref())
            .filter(|_| available)
        {
            content = content
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(theme::CONTENT_PRIMARY))
                        .child(commit.summary.clone()),
                )
                .child(meta(
                    khaslana::ui_i18n::text(messages::AUTHOR),
                    crate::history_view::author_label(commit),
                    false,
                ))
                .child(meta(khaslana::ui_i18n::text(messages::COMMIT_TIME), commit_time_label(commit.time), false))
                .child(meta(
                    khaslana::ui_i18n::text(messages::PARENT_COMMITS),
                    crate::history_view::parents_note(&commit.parents),
                    false,
                ));
            if let Some(committer) = crate::history_view::committer_note(commit) {
                content = content.child(meta(khaslana::ui_i18n::text(messages::COMMITTER), committer, false));
            }
            if commit.message.trim() != commit.summary {
                content = content.child(meta(
                    khaslana::ui_i18n::text(messages::FULL_COMMIT_MESSAGE),
                    commit.message.trim().to_string(),
                    false,
                ));
            }
            if commit.parents.len() > 1 {
                content = content.child(
                    div()
                        .text_size(px(11.0))
                        .line_height(px(17.0))
                        .text_color(rgb(theme::CONTENT_SECONDARY))
                        .child(khaslana::ui_i18n::text(messages::THIS_IS_A_MERGE_COMMIT_FILES_AND_DIFFERENCES_ARE_SHOWN)),
                );
            }
        }
        content = content
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(theme::CONTENT_PRIMARY))
                    .child(khaslana::ui_i18n::text(messages::LOG_EVIDENCE)),
            )
            .child(meta(
                khaslana::ui_i18n::text(messages::OPERATION_DESCRIPTION),
                if entry.message.is_empty() {
                    khaslana::ui_i18n::text(messages::NO_OPERATION_DESCRIPTION).into()
                } else {
                    entry.message.clone()
                },
                false,
            ))
            .child(meta(khaslana::ui_i18n::text(messages::SOURCE_REFERENCE), entry.ref_name.clone(), true))
            .child(meta(khaslana::ui_i18n::text(messages::OPERATION_TIME), commit_time_label(entry.timestamp), false))
            .child(meta(
                khaslana::ui_i18n::text(messages::ACTOR),
                if entry.actor_email.is_empty() {
                    entry.actor.clone()
                } else {
                    format!("{} <{}>", entry.actor, entry.actor_email)
                },
                false,
            ))
            .child(meta(khaslana::ui_i18n::text(messages::FULL_OID_BEFORE_OPERATION), entry.old_oid.clone(), true))
            .child(meta(khaslana::ui_i18n::text(messages::FULL_OID_AFTER_OPERATION), entry.new_oid.clone(), true));
        if entry.lossy {
            content = content.child(div().text_size(px(11.0)).line_height(px(17.0)).text_color(rgb(theme::FEEDBACK_WARNING_TEXT)).child(khaslana::ui_i18n::text(messages::THE_ORIGINAL_LOG_CONTAINS_INVALID_UTF_AND_IS_DISPLAYED_WITH)));
        }
        scrollable_frame_when(
            "recovery-details-scroll",
            ScrollbarMode::Vertical,
            content.into_any_element(),
            handle,
            true,
            cx,
        )
        .into_any_element()
    }

    pub(crate) fn render_recovery_diff_tools(&self, cx: &mut Context<Self>) -> AnyElement {
        if self
            .recovery
            .preview
            .diff
            .as_ref()
            .is_some_and(|diff| diff.is_binary)
        {
            return div().into_any_element();
        }
        let enabled = !self.busy && self.recovery.preview.open;
        let current = self.current_diff_encoding_choice();
        let label = self
            .recovery
            .preview
            .diff
            .as_deref()
            .map(crate::diff_encoding_label)
            .unwrap_or_else(|| khaslana::ui_i18n::format(messages::ENCODING_LABEL, &[("label", &(current.label()).to_string())]));
        let entity = cx.entity();
        let tab_id = self.active_tab_id();
        let repo_path = self.repo_path.clone();
        let encoding = workbench_command_button("recovery-encoding-kit", label, false, enabled, cx)
            .h(px(24.0))
            .min_h(px(24.0))
            .py_0()
            .px_2()
            .dropdown_caret(true)
            .tooltip(khaslana::ui_i18n::text(messages::CHOOSE_DIFF_DECODING_FOR_THE_CURRENT_REPOSITORY))
            .dropdown_menu(move |mut menu, window, _| {
                for choice in [
                    crate::DiffEncodingChoice::Auto,
                    crate::DiffEncodingChoice::Utf8,
                    crate::DiffEncodingChoice::Gb18030,
                    crate::DiffEncodingChoice::Big5,
                ] {
                    let repo_path = repo_path.clone();
                    menu = menu.item(
                        PopupMenuItem::new(choice.label())
                            .checked(choice == current)
                            .on_click(window.listener_for(&entity, move |this, _, _, cx| {
                                if this.active_tab_id() == tab_id && this.repo_path == repo_path {
                                    this.choose_diff_encoding(choice);
                                    cx.notify();
                                }
                            })),
                    );
                }
                menu
            });
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                workbench_command_button(
                    "recovery-full-file-kit",
                    if self.full_file_view {
                        khaslana::ui_i18n::text(messages::DIFF)
                    } else {
                        khaslana::ui_i18n::text(messages::FULL_FILE)
                    },
                    self.full_file_view,
                    enabled,
                    cx,
                )
                .h(px(24.0))
                .min_h(px(24.0))
                .py_0()
                .px_2()
                .tooltip(if self.full_file_view {
                    khaslana::ui_i18n::text(messages::SHOW_DIFFERENCES_AND_SURROUNDING_CONTEXT)
                } else {
                    khaslana::ui_i18n::text(messages::SHOW_FULL_FILE_CONTEXT)
                })
                .on_click(cx.listener(|this, _, _, cx| this.toggle_full_file_view(cx))),
            )
            .child(encoding)
            .into_any_element()
    }

    fn render_recovery_preview(
        &self,
        layout: RecoveryLayout,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let title = self
            .recovery
            .preview
            .selected_file
            .as_ref()
            .map(|path| {
                khaslana::ui_i18n::format(if self.full_file_view { messages::FULL_FILE_TITLE } else { messages::DIFF_TITLE }, &[("path", path)])
            })
            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::COMMIT_DIFF).into());
        let empty = if self.recovery.preview.loading_diff {
            khaslana::ui_i18n::text(messages::LOADING_COMMIT_DIFFERENCES).into()
        } else {
            self.recovery
                .preview
                .error
                .as_ref().map(|error| khaslana::ui_i18n::resolve(error))
                .unwrap_or_else(|| khaslana::ui_i18n::text(messages::SELECT_A_COMMIT_FILE_TO_VIEW_ITS_DIFFERENCES).into())
        };
        let diff = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .relative()
            .child(self.diff_section_header(title, EncodingMenuTarget::Recovery, cx))
            .child(self.render_virtual_diff(
                "recovery-diff-scroll",
                self.recovery.preview.diff.clone(),
                self.recovery.preview.headers_expanded,
                DiffHeaderTarget::Recovery,
                empty,
                cx,
            ));
        let mut preview = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0));
        if let Some(commit) = self
            .recovery
            .details
            .as_ref()
            .filter(|details| Some(details.oid.as_str()) == self.recovery.selected_oid())
            .and_then(|details| details.commit.as_ref())
            .filter(|commit| commit.parents.len() > 1)
        {
            preview = preview.child(
                div()
                    .flex_none()
                    .px_3()
                    .py_1()
                    .text_size(px(11.0))
                    .line_height(px(17.0))
                    .text_color(rgb(theme::CONTENT_SECONDARY))
                    .child(khaslana::ui_i18n::format(messages::MERGE_COMMIT_PREVIEW_RELATIVE_TO_FIRST_PARENT_PARENTS, &[("parents", &(short_oid(&commit.parents[0])).to_string())])),
            );
        }
        preview
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .when(!layout.files_side_by_side, |this| this.flex_col())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w(px(0.0))
                            .min_h(px(0.0))
                            .flex_none()
                            .when(layout.files_side_by_side, |this| {
                                this.w(px(RECOVERY_FILES_WIDTH.min(layout.detail_width * 0.32)))
                                    .border_r_1()
                                    .border_color(rgb(theme::BORDER_MUTED))
                            })
                            .when(!layout.files_side_by_side, |this| {
                                this.h(px(96.0))
                                    .border_b_1()
                                    .border_color(rgb(theme::BORDER_MUTED))
                            })
                            .child(self.render_recovery_files(cx)),
                    )
                    .child(diff),
            )
            .into_any_element()
    }

    fn render_recovery_files(&self, cx: &mut Context<Self>) -> AnyElement {
        let handle = self.uniform_scroll_handle("recovery-file-list");
        let list_handle = handle.clone();
        let content_present = !self.recovery.preview.files.is_empty();
        let content = div()
            .id("recovery-file-list")
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .py_1()
            .child(
                uniform_list(
                    "recovery-file-list",
                    self.recovery.preview.files.len().max(1),
                    cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                        range
                            .map(|index| this.recovery_file_row(index, cx))
                            .collect::<Vec<_>>()
                    }),
                )
                .track_scroll(&list_handle)
                .with_sizing_behavior(ListSizingBehavior::Auto)
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(0.0)),
            )
            .into_any_element();
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .child(
                panel_section_header(khaslana::ui_i18n::format(messages::COMMIT_FILES_FILES, &[("files", &(self.recovery.preview.files.len()).to_string())]))
                    .build(),
            )
            .child(scrollable_uniform_frame(
                "recovery-file-list",
                ScrollbarMode::Vertical,
                content,
                handle,
                content_present,
                cx,
            ))
            .into_any_element()
    }

    fn recovery_file_row(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(file) = self.recovery.preview.files.get(index) else {
            return panel_empty_row(
                if self.recovery.preview.loading_files {
                    khaslana::ui_i18n::text(messages::LOADING_COMMIT_FILES)
                } else if self.recovery.preview.error.is_some() {
                    khaslana::ui_i18n::text(messages::FAILED_TO_READ_FILES)
                } else {
                    khaslana::ui_i18n::text(messages::THIS_COMMIT_HAS_NO_FILE_CHANGES)
                },
                CHANGE_ROW_HEIGHT,
                PlaceholderAlign::Start,
            )
            .into_any_element();
        };
        self.recovery_commit_file_row(file, cx)
    }

    fn recovery_commit_file_row(
        &self,
        file: &CommitFileChange,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let selected = self.recovery.preview.selected_file.as_deref() == Some(file.path.as_str());
        let label = file
            .old_path
            .as_ref()
            .filter(|old| *old != &file.path)
            .map(|old| format!("{old} → {}", file.path))
            .unwrap_or_else(|| file.path.clone());
        let path = file.path.clone();
        let tooltip = label.clone();
        list_row_surface(format!("recovery-file-{}", file.path), selected)
            .flex()
            .flex_none()
            .items_center()
            .gap_1()
            .w_full()
            .min_w(px(0.0))
            .h(px(CHANGE_ROW_HEIGHT))
            .px_3()
            .py_1()
            .overflow_hidden()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_recovery_file(path.clone());
                cx.notify();
            }))
            .tooltip(move |_, cx| tooltip_text(tooltip.clone(), cx))
            .child(change_state_badge(Some(&file.status)))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_size(px(12.0))
                    .text_color(rgb(theme::CONTENT_PRIMARY))
                    .child(label),
            )
            .into_any_element()
    }

    pub(crate) fn render_recovery_branch_dialog(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(dialog) = self.recovery.dialog.as_ref() else {
            return div().into_any_element();
        };
        let busy = self.busy || self.recovery.operation.is_some();
        let validation_error = self.recovery_branch_error();
        let error = dialog.error.clone().or_else(|| validation_error.clone());
        let can_submit = !busy
            && !self.recovery.branch_name.value.trim().is_empty()
            && validation_error.is_none();
        let (width, height) = dialog_panel_size(window, 560.0, 490.0);
        let handle = self.scroll_handle("recovery-branch-dialog-scroll");
        let mut body = div()
            .id("recovery-branch-dialog-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .overflow_y_scroll()
            .track_scroll(&handle)
            .gap_3()
            .pr_1()
            .child(meta(
                khaslana::ui_i18n::text(messages::WORKING_TREE),
                dialog.identity.worktree_path.display().to_string(),
                false,
            ))
            .child(meta(khaslana::ui_i18n::text(messages::FULL_TARGET_OID), dialog.oid.clone(), true))
            .child(meta(khaslana::ui_i18n::text(messages::SOURCE_RECORD), dialog.source.clone(), true))
            .child(meta(khaslana::ui_i18n::text(messages::RECOVERY_TARGET), dialog.side.label().to_string(), false))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(theme::CONTENT_PRIMARY))
                            .child(khaslana::ui_i18n::text(messages::LOCAL_BRANCH_NAME)),
                    )
                    .child(self.input(FieldId::RecoveryBranchName, false, window, cx)),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(18.0))
                    .text_color(rgb(theme::CONTENT_SECONDARY))
                    .child(khaslana::ui_i18n::text(messages::ONLY_CREATES_A_LOCAL_BRANCH_POINTING_TO_THIS_COMMIT_THE)),
            );
        if let Some(error) = error {
            let tooltip = khaslana::ui_i18n::resolve(&error);
            body = body.child(
                div()
                    .id("recovery-branch-error")
                    .min_w(px(0.0))
                    .tooltip(move |_, cx| tooltip_text(tooltip.clone(), cx))
                    .child(inline_error_bubble(khaslana::ui_i18n::resolve(&error))),
            );
        }
        self.dialog_panel(khaslana::ui_i18n::text(messages::CREATE_RECOVERY_BRANCH), cx)
            .w(width)
            .h(height)
            .min_w(px(0.0))
            .min_h(px(0.0))
            .child(scrollable_frame_when(
                "recovery-branch-dialog-scroll",
                ScrollbarMode::Vertical,
                body.into_any_element(),
                handle,
                true,
                cx,
            ))
            .child(
                dialog_actions()
                    .flex_none()
                    .flex_wrap()
                    .child(self.button(khaslana::ui_i18n::text(messages::CANCEL), !busy, |this, _, _| this.close_dialog(), cx))
                    .child(self.primary_button(
                        if busy {
                            khaslana::ui_i18n::text(messages::CREATING)
                        } else {
                            khaslana::ui_i18n::text(messages::CREATE_LOCAL_BRANCH)
                        },
                        can_submit,
                        |this, _, cx| this.submit_recovery_branch(cx),
                        cx,
                    )),
            )
            .into_any_element()
    }
}
