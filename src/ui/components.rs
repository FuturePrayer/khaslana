use std::time::{Duration, Instant};

use gpui::{
    App, Context, CursorStyle, Div, IntoElement, Render, Stateful, Window, div, prelude::*, px,
    rgb, rgba,
};

use crate::{RepositoryView, ui::theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AppToastKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub(crate) struct FeedbackMessage {
    pub(crate) id: u64,
    pub(crate) kind: AppToastKind,
    pub(crate) title: &'static str,
    pub(crate) message: String,
    pub(crate) expires_at: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ButtonTone {
    Neutral,
    Primary,
    Danger,
}

#[derive(Clone, Copy, Debug)]
struct ButtonPalette {
    bg: u32,
    hover_bg: u32,
    fg: u32,
    border: u32,
}

struct TextTooltip {
    text: gpui::SharedString,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputFrameSize {
    Compact,
    Regular,
    Multiline,
}

impl AppToastKind {
    fn label(self) -> &'static str {
        match self {
            AppToastKind::Info => "提示",
            AppToastKind::Success => "完成",
            AppToastKind::Warning => "注意",
            AppToastKind::Error => "失败",
        }
    }

    fn palette(self) -> (u32, u32, u32) {
        match self {
            AppToastKind::Info => (
                theme::FEEDBACK_INFO_BG,
                theme::FEEDBACK_INFO_BORDER,
                theme::FEEDBACK_INFO_TEXT,
            ),
            AppToastKind::Success => (
                theme::FEEDBACK_SUCCESS_BG,
                theme::FEEDBACK_SUCCESS_BORDER,
                theme::FEEDBACK_SUCCESS_TEXT,
            ),
            AppToastKind::Warning => (
                theme::FEEDBACK_WARNING_BG,
                theme::FEEDBACK_WARNING_BORDER,
                theme::FEEDBACK_WARNING_TEXT,
            ),
            AppToastKind::Error => (
                theme::FEEDBACK_ERROR_BG,
                theme::FEEDBACK_ERROR_BORDER,
                theme::FEEDBACK_ERROR_TEXT,
            ),
        }
    }

    pub(crate) fn is_important(self) -> bool {
        matches!(self, AppToastKind::Warning | AppToastKind::Error)
    }
}

impl FeedbackMessage {
    pub(crate) fn new(id: u64, kind: AppToastKind, message: String) -> Self {
        let ttl = if kind.is_important() {
            Duration::from_secs(7)
        } else {
            Duration::from_secs(4)
        };
        Self {
            id,
            kind,
            title: kind.label(),
            message,
            expires_at: Instant::now() + ttl,
        }
    }

    pub(crate) fn is_expired(&self, now: Instant) -> bool {
        now >= self.expires_at
    }
}

impl Render for TextTooltip {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .max_w(px(280.0))
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(theme::TOOLTIP_BORDER))
            .bg(rgb(theme::TOOLTIP_BG))
            .text_color(rgb(theme::SURFACE))
            .text_size(px(12.0))
            .line_height(px(18.0))
            .shadow_lg()
            .child(self.text.clone())
    }
}

fn tooltip_text(text: impl Into<gpui::SharedString>, cx: &mut App) -> gpui::AnyView {
    let text = text.into();
    cx.new(move |_| TextTooltip { text }).into()
}

fn app_button_palette(tone: ButtonTone, enabled: bool) -> ButtonPalette {
    if !enabled {
        return ButtonPalette {
            bg: theme::SURFACE_MUTED,
            hover_bg: theme::SURFACE_MUTED,
            fg: theme::TEXT_FAINT,
            border: theme::BORDER,
        };
    }

    match tone {
        ButtonTone::Neutral => ButtonPalette {
            bg: theme::SURFACE,
            hover_bg: theme::ACCENT_SOFT,
            fg: theme::TEXT,
            border: theme::BORDER,
        },
        ButtonTone::Primary => ButtonPalette {
            bg: theme::ACCENT,
            hover_bg: theme::ACCENT_STRONG,
            fg: theme::SURFACE,
            border: theme::ACCENT_STRONG,
        },
        ButtonTone::Danger => ButtonPalette {
            bg: theme::DANGER,
            hover_bg: theme::DANGER_STRONG,
            fg: theme::SURFACE,
            border: theme::DANGER_BORDER,
        },
    }
}

pub(crate) fn section_title(title: &'static str) -> impl IntoElement {
    div()
        .flex_none()
        .px_2()
        .py_2()
        .border_b_1()
        .border_color(rgb(theme::BORDER))
        .bg(rgb(theme::HEADER_BG))
        .text_size(px(11.0))
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(rgb(theme::TEXT_MUTED))
        .child(title)
}

pub(crate) fn app_panel() -> Div {
    div()
        .rounded_sm()
        .border_1()
        .border_color(rgb(theme::BORDER))
        .bg(rgb(theme::PANEL_BG))
}

pub(crate) fn dialog_overlay() -> Div {
    div()
        .absolute()
        .top(px(0.0))
        .left(px(0.0))
        .right(px(0.0))
        .bottom(px(0.0))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(theme::DIALOG_OVERLAY))
        .cursor(CursorStyle::Arrow)
        .occlude()
}

pub(crate) fn dialog_panel(title: &'static str) -> Stateful<Div> {
    div()
        .id(format!("dialog-{title}"))
        .w(px(480.0))
        .p_4()
        .rounded_sm()
        .border_1()
        .border_color(rgb(theme::BORDER))
        .bg(rgb(theme::SURFACE))
        .shadow_lg()
        .flex()
        .flex_col()
        .gap_3()
        .cursor(CursorStyle::Arrow)
        .occlude()
        .on_mouse_down(gpui::MouseButton::Left, |_event, _window, cx| {
            cx.stop_propagation();
        })
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .pb_1()
                .border_b_1()
                .border_color(rgb(theme::BORDER_MUTED))
                .child(
                    div()
                        .min_w(px(0.0))
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(theme::TEXT))
                        .truncate()
                        .child(title),
                ),
        )
}

pub(crate) fn dialog_actions() -> Div {
    div()
        .flex()
        .items_center()
        .justify_end()
        .gap_2()
        .pt_1()
        .border_t_1()
        .border_color(rgb(theme::BORDER_MUTED))
}

pub(crate) fn danger_callout(message: impl Into<gpui::SharedString>) -> impl IntoElement {
    div()
        .px_3()
        .py_2()
        .rounded_sm()
        .border_1()
        .border_color(rgb(theme::DANGER_BORDER_SOFT))
        .bg(rgb(theme::DANGER_SOFT))
        .text_size(px(12.0))
        .line_height(px(18.0))
        .text_color(rgb(theme::DANGER_TEXT))
        .child(message.into())
}

pub(crate) fn input_frame(id: String, focused: bool, size: InputFrameSize) -> Stateful<Div> {
    let height = match size {
        InputFrameSize::Compact => px(28.0),
        InputFrameSize::Regular => px(34.0),
        InputFrameSize::Multiline => px(92.0),
    };
    div()
        .id(id)
        .relative()
        .w_full()
        .min_h(height)
        .rounded_sm()
        .border_1()
        .border_color(if focused {
            rgb(theme::INPUT_BORDER_FOCUSED)
        } else {
            rgb(theme::INPUT_BORDER)
        })
        .bg(if focused {
            rgb(theme::INPUT_BG_FOCUSED)
        } else {
            rgb(theme::INPUT_BG)
        })
        .text_size(px(12.0))
        .line_height(px(18.0))
        .cursor(CursorStyle::IBeam)
        .when(!focused, |this| {
            this.hover(|this| this.bg(rgb(theme::SURFACE_HOVER)))
        })
        .when(focused, |this| {
            this.shadow_sm()
                .border_color(rgb(theme::INPUT_BORDER_FOCUSED))
        })
}

pub(crate) fn segmented_button(id: String, selected: bool, enabled: bool) -> Stateful<Div> {
    div()
        .id(id)
        .flex_none()
        .min_h(px(28.0))
        .px_2()
        .py_1()
        .rounded_sm()
        .border_1()
        .border_color(if selected {
            rgb(theme::FOCUS_RING)
        } else {
            rgb(theme::BORDER)
        })
        .bg(if selected {
            rgb(theme::SEGMENT_SELECTED_BG)
        } else {
            rgb(theme::SEGMENT_BG)
        })
        .text_size(px(12.0))
        .text_color(if selected {
            rgb(theme::SEGMENT_SELECTED_TEXT)
        } else if enabled {
            rgb(theme::TEXT_MUTED)
        } else {
            rgb(theme::TEXT_FAINT)
        })
        .font_weight(if selected {
            gpui::FontWeight::BOLD
        } else {
            gpui::FontWeight::NORMAL
        })
        .when(enabled, |this| this.cursor_pointer())
        .when(!enabled, |this| this.cursor_not_allowed().opacity(0.68))
        .when(enabled, |this| {
            this.hover(|this| this.bg(rgb(theme::ROW_HOVER)))
        })
}

pub(crate) fn toggle_box(checked: bool) -> impl IntoElement {
    div()
        .size(px(14.0))
        .rounded_sm()
        .border_1()
        .border_color(if checked {
            rgb(theme::ACCENT)
        } else {
            rgb(theme::BORDER)
        })
        .bg(if checked {
            rgb(theme::ACCENT)
        } else {
            rgb(theme::SURFACE)
        })
        .child(
            div()
                .w_full()
                .h_full()
                .when(checked, |this| this.child("✓"))
                .items_center()
                .justify_center()
                .text_color(rgb(theme::SURFACE))
                .text_size(px(10.0)),
        )
}

pub(crate) fn list_row_surface(id: String, selected: bool) -> Stateful<Div> {
    div()
        .id(id)
        .rounded_sm()
        .border_1()
        .border_color(if selected {
            rgb(theme::ROW_SELECTED_BORDER)
        } else {
            rgb(theme::BORDER)
        })
        .bg(if selected {
            rgb(theme::ROW_SELECTED)
        } else {
            rgb(theme::SURFACE)
        })
        .hover(|this| this.bg(rgb(theme::ROW_HOVER)))
}

pub(crate) fn status_pill(label: &'static str, active: bool) -> impl IntoElement {
    div()
        .flex_none()
        .min_h(px(24.0))
        .px_2()
        .py_1()
        .rounded_full()
        .border_1()
        .border_color(if active {
            rgb(theme::FOCUS_RING)
        } else {
            rgb(theme::BORDER)
        })
        .bg(if active {
            rgb(theme::ACCENT_SOFT)
        } else {
            rgb(theme::SURFACE)
        })
        .text_color(if active {
            rgb(theme::ACCENT_STRONG)
        } else {
            rgb(theme::TEXT_MUTED)
        })
        .font_weight(gpui::FontWeight::BOLD)
        .child(label)
}

pub(crate) fn feedback_stack(important: bool) -> Div {
    div()
        .absolute()
        .bottom(px(54.0))
        .when(important, |this| this.right(px(18.0)))
        .when(!important, |this| this.left(px(18.0)))
        .w(px(340.0))
        .flex()
        .flex_col()
        .gap_2()
}

pub(crate) fn feedback_bubble(feedback: &FeedbackMessage) -> impl IntoElement {
    let (soft_bg, border, text) = feedback.kind.palette();
    let dot = match feedback.kind {
        AppToastKind::Info => "i",
        AppToastKind::Success => "✓",
        AppToastKind::Warning => "!",
        AppToastKind::Error => "×",
    };

    div()
        .id(format!("feedback-{}", feedback.id))
        .w_full()
        .px_3()
        .py_2()
        .rounded_sm()
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(theme::FEEDBACK_BG))
        .shadow_lg()
        .flex()
        .gap_2()
        .child(
            div()
                .flex_none()
                .size(px(22.0))
                .rounded_full()
                .items_center()
                .justify_center()
                .bg(rgb(soft_bg))
                .text_color(rgb(text))
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::BOLD)
                .child(dot),
        )
        .child(
            div()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(rgb(text))
                        .child(feedback.title),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(18.0))
                        .text_color(rgb(theme::TEXT))
                        .child(feedback.message.clone()),
                ),
        )
}

pub(crate) fn inline_error_bubble(message: impl Into<gpui::SharedString>) -> impl IntoElement {
    div()
        .flex_none()
        .max_w(px(460.0))
        .px_2()
        .py_1()
        .rounded_full()
        .border_1()
        .border_color(rgb(theme::FEEDBACK_ERROR_BORDER))
        .bg(rgb(theme::FEEDBACK_ERROR_BG))
        .text_color(rgb(theme::FEEDBACK_ERROR_TEXT))
        .truncate()
        .child(message.into())
}

pub(crate) fn bottom_progress_bar(phase: u64) -> impl IntoElement {
    let offset = ((phase % 7) as f32 - 2.0) * 72.0;
    div()
        .absolute()
        .left(px(0.0))
        .right(px(0.0))
        .bottom(px(0.0))
        .h(px(3.0))
        .overflow_hidden()
        .bg(rgb(theme::PROGRESS_TRACK))
        .child(
            div()
                .absolute()
                .top(px(0.0))
                .bottom(px(0.0))
                .left(px(offset))
                .w(px(260.0))
                .rounded_full()
                .bg(rgb(theme::PROGRESS_FILL)),
        )
}

pub(crate) fn operation_loading_bar(message: impl Into<gpui::SharedString>) -> impl IntoElement {
    div()
        .absolute()
        .left(px(16.0))
        .right(px(16.0))
        .bottom(px(46.0))
        .h(px(34.0))
        .px_3()
        .rounded_sm()
        .border_1()
        .border_color(rgb(theme::FEEDBACK_INFO_BORDER))
        .bg(rgb(theme::FEEDBACK_BG))
        .shadow_lg()
        .flex()
        .items_center()
        .gap_2()
        .text_size(px(12.0))
        .text_color(rgb(theme::ACCENT_STRONG))
        .child(
            div()
                .size(px(8.0))
                .rounded_full()
                .bg(rgb(theme::PROGRESS_FILL)),
        )
        .child(div().min_w(px(0.0)).truncate().child(message.into()))
}

fn format_badge_count(count: usize) -> String {
    if count > 99 {
        "99+".to_string()
    } else {
        count.to_string()
    }
}

impl RepositoryView {
    pub(crate) fn notify_toast(
        &mut self,
        kind: AppToastKind,
        message: impl Into<gpui::SharedString>,
        cx: &mut Context<Self>,
    ) {
        let message = message.into().to_string();
        if message.trim().is_empty() {
            return;
        }
        self.next_feedback_id = self.next_feedback_id.wrapping_add(1).max(1);
        self.feedbacks
            .push_back(FeedbackMessage::new(self.next_feedback_id, kind, message));
        while self.feedbacks.len() > 5 {
            self.feedbacks.pop_front();
        }
        cx.notify();
    }

    pub(crate) fn notify_success(
        &mut self,
        message: impl Into<gpui::SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.notify_toast(AppToastKind::Success, message, cx);
    }

    pub(crate) fn notify_warning(
        &mut self,
        message: impl Into<gpui::SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.notify_toast(AppToastKind::Warning, message, cx);
    }

    pub(crate) fn notify_error(
        &mut self,
        message: impl Into<gpui::SharedString>,
        cx: &mut Context<Self>,
    ) {
        self.notify_toast(AppToastKind::Error, message, cx);
    }

    pub(crate) fn notify_completion(&mut self, message: &str, cx: &mut Context<Self>) {
        if message.contains("失败") || message.contains("冲突") {
            self.notify_warning(message.to_string(), cx);
        } else {
            self.notify_success(message.to_string(), cx);
        }
    }

    pub(crate) fn should_toast_completion(message: &str) -> bool {
        message.contains("完成")
            || message.contains("失败")
            || message.contains("冲突")
            || message.contains("已复制")
            || message.contains("已添加")
            || message.contains("已更新")
            || message.contains("已新增")
            || message.contains("已删除")
            || message.contains("已刷新")
            || message.contains("已提交")
            || message.contains("工作流")
    }

    pub(crate) fn button(
        &self,
        label: &'static str,
        enabled: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.app_button(label, None, ButtonTone::Neutral, enabled, on_click, cx)
    }

    pub(crate) fn button_with_badge(
        &self,
        label: &'static str,
        badge: Option<usize>,
        enabled: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.app_button(label, badge, ButtonTone::Neutral, enabled, on_click, cx)
    }

    pub(crate) fn primary_button(
        &self,
        label: &'static str,
        enabled: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.app_button(label, None, ButtonTone::Primary, enabled, on_click, cx)
    }

    pub(crate) fn danger_button(
        &self,
        label: &'static str,
        enabled: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.app_button(label, None, ButtonTone::Danger, enabled, on_click, cx)
    }

    fn app_button(
        &self,
        label: &'static str,
        badge: Option<usize>,
        tone: ButtonTone,
        enabled: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let palette = app_button_palette(tone, enabled);
        let disabled_reason = self.disabled_reason(enabled, "当前状态不可用");
        let enabled_color = if enabled {
            palette.bg
        } else {
            theme::SURFACE_MUTED
        };
        let text_color = if enabled {
            palette.fg
        } else {
            theme::TEXT_FAINT
        };
        div()
            .id(label)
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .flex_none()
            .min_h(px(28.0))
            .px_2()
            .py_1()
            .border_1()
            .border_color(rgb(palette.border))
            .rounded_sm()
            .bg(rgb(enabled_color))
            .text_color(rgb(text_color))
            .text_size(px(12.0))
            .when(enabled, |this| this.cursor_pointer())
            .when(!enabled, |this| this.cursor_not_allowed().opacity(0.78))
            .when(enabled, |this| {
                this.hover(move |this| this.bg(rgb(palette.hover_bg)))
                    .active(|this| this.opacity(0.82))
            })
            .when_some(disabled_reason, |this, tooltip| {
                this.tooltip(move |_window, cx| tooltip_text(tooltip, cx))
            })
            .on_click(cx.listener(move |this, _event, window, cx| {
                if enabled {
                    let previous_status = this.status.clone();
                    let previous_busy = this.busy;
                    let previous_feedback_count = this.feedbacks.len();
                    on_click(this, window, cx);
                    if this.feedbacks.len() == previous_feedback_count {
                        if let Some(error) = this.last_error.clone() {
                            this.notify_error(error, cx);
                        } else if !previous_busy
                            && !this.busy
                            && this.status != previous_status
                            && Self::should_toast_completion(&this.status)
                        {
                            this.notify_success(this.status.clone(), cx);
                        }
                    }
                    cx.notify();
                }
            }))
            .child(label)
            .when_some(badge.filter(|count| *count > 0), |this, count| {
                this.child(
                    div()
                        .absolute()
                        .top(px(-7.0))
                        .right(px(-7.0))
                        .min_w(px(16.0))
                        .h(px(16.0))
                        .px_1()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .border_1()
                        .border_color(rgb(theme::BADGE_BORDER))
                        .bg(rgb(theme::BADGE_BG))
                        .text_color(rgb(theme::SURFACE))
                        .text_size(px(10.0))
                        .line_height(px(14.0))
                        .child(format_badge_count(count)),
                )
            })
    }
}
