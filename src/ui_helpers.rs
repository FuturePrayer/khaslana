use gpui::{Context, IntoElement, SharedString, div, prelude::*, px, rgb};
use khaslana::{DiffLineKind, DiffScope};

use crate::{DiffHeaderTarget, RepositoryView};

pub(crate) const COLOR_APP_BG: u32 = 0xf7f9fc;
pub(crate) const COLOR_SURFACE: u32 = 0xffffff;
pub(crate) const COLOR_SURFACE_SOFT: u32 = 0xf3f6fb;
pub(crate) const COLOR_PANEL_BG: u32 = 0xffffff;
pub(crate) const COLOR_HEADER_BG: u32 = 0xf8fbff;
pub(crate) const COLOR_BORDER: u32 = 0xdbe5f2;
pub(crate) const COLOR_BORDER_STRONG: u32 = 0x3b82f6;
pub(crate) const COLOR_BLUE: u32 = 0x3b82f6;
pub(crate) const COLOR_BLUE_DARK: u32 = 0x1d4ed8;
pub(crate) const COLOR_BLUE_SOFT: u32 = 0xeaf2ff;
pub(crate) const COLOR_TEXT: u32 = 0x111827;
pub(crate) const COLOR_TEXT_MUTED: u32 = 0x4b5563;
pub(crate) const COLOR_TEXT_FAINT: u32 = 0x8a96a8;
pub(crate) const COLOR_ROW_SELECTED: u32 = 0xeaf2ff;
pub(crate) const COLOR_HASH_BG: u32 = 0xf1f5fb;

pub(crate) fn section_header(label: impl Into<SharedString>) -> impl IntoElement {
    div()
        .flex_none()
        .px_3()
        .py_2()
        .border_b_1()
        .border_color(rgb(COLOR_BORDER))
        .bg(rgb(COLOR_HEADER_BG))
        .text_size(px(12.0))
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(rgb(COLOR_TEXT))
        .child(label.into())
}

pub(crate) fn section_header_action(
    label: impl Into<SharedString>,
    action: Option<gpui::AnyElement>,
) -> impl IntoElement {
    let header = div()
        .flex_none()
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .px_3()
        .py_2()
        .border_b_1()
        .border_color(rgb(COLOR_BORDER))
        .bg(rgb(COLOR_HEADER_BG))
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(COLOR_TEXT))
                .child(label.into()),
        );
    if let Some(action) = action {
        header.child(action)
    } else {
        header
    }
}

pub(crate) fn nav_list(id: &'static str, rows: Vec<gpui::AnyElement>) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .min_w(px(0.0))
        .min_h(px(0.0))
        .p_2()
        .overflow_y_scroll()
        .children(rows)
}

pub(crate) fn nav_row(
    id: impl Into<SharedString>,
    selected: bool,
    emphasized: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id.into())
        .flex()
        .flex_none()
        .items_center()
        .justify_between()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_sm()
        .cursor_pointer()
        .bg(if selected {
            rgb(COLOR_ROW_SELECTED)
        } else if emphasized {
            rgb(COLOR_BLUE_SOFT)
        } else {
            rgb(COLOR_SURFACE)
        })
        .border_1()
        .border_color(if selected {
            rgb(0x9bbcff)
        } else if emphasized {
            rgb(COLOR_BORDER_STRONG)
        } else {
            rgb(COLOR_BORDER)
        })
}

pub(crate) fn placeholder_row(text: &'static str) -> impl IntoElement {
    div()
        .flex_none()
        .px_2()
        .py_2()
        .text_size(px(12.0))
        .text_color(rgb(COLOR_TEXT_FAINT))
        .child(text)
}

pub(crate) fn commit_time_label(seconds: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, 0)
        .map(|time| {
            time.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "时间未知".to_string())
}

pub(crate) fn author_avatar(author: &str) -> impl IntoElement {
    div()
        .flex_none()
        .size(px(20.0))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(author_avatar_color(author)))
        .text_color(rgb(COLOR_SURFACE))
        .text_size(px(11.0))
        .font_weight(gpui::FontWeight::BOLD)
        .child(author_avatar_initial(author))
}

fn author_avatar_color(author: &str) -> u32 {
    const PALETTE: [u32; 10] = [
        0x6366f1, 0x3b82f6, 0x06b6d4, 0x14b8a6, 0x22c55e, 0x84cc16, 0xf59e0b, 0xf97316, 0xef4444,
        0xa855f7,
    ];
    let mut hash = 0u32;
    for byte in author.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }
    PALETTE[(hash as usize) % PALETTE.len()]
}

fn author_avatar_initial(author: &str) -> String {
    author
        .trim()
        .chars()
        .find(|ch| !ch.is_whitespace())
        .map(|ch| ch.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}

pub(crate) fn diff_scope_label(scope: &DiffScope) -> &'static str {
    match scope {
        DiffScope::Staged => "已暂存",
        DiffScope::Unstaged => "未暂存",
    }
}

pub(crate) fn diff_scope_id(scope: &DiffScope) -> &'static str {
    match scope {
        DiffScope::Staged => "staged",
        DiffScope::Unstaged => "unstaged",
    }
}

pub(crate) fn context_menu_item(
    label: &'static str,
    enabled: bool,
    on_click: impl Fn(&mut RepositoryView) + 'static,
    cx: &mut Context<RepositoryView>,
) -> impl IntoElement {
    div()
        .id(format!("context-menu-{label}"))
        .px_3()
        .py_1()
        .text_color(if enabled {
            rgb(COLOR_TEXT)
        } else {
            rgb(COLOR_TEXT_FAINT)
        })
        .bg(rgb(COLOR_SURFACE))
        .cursor_pointer()
        .when(enabled, |this| {
            this.hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
        })
        .on_click(cx.listener(move |this, _event, _window, cx| {
            cx.stop_propagation();
            if enabled {
                on_click(this);
                cx.notify();
            }
        }))
        .child(label)
}

pub(crate) fn menu_separator() -> impl IntoElement {
    div().h(px(1.0)).mx_1().my_1().bg(rgb(COLOR_BORDER))
}

pub(crate) fn diff_header_toggle(
    label: &'static str,
    target: DiffHeaderTarget,
    cx: &mut Context<RepositoryView>,
) -> impl IntoElement {
    div()
        .id(match target {
            DiffHeaderTarget::Worktree => "diff-header-toggle",
            DiffHeaderTarget::History => "history-diff-header-toggle",
        })
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .py(px(2.0))
        .bg(rgb(COLOR_HEADER_BG))
        .text_color(rgb(COLOR_TEXT_MUTED))
        .cursor_pointer()
        .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
        .on_click(cx.listener(move |this, _event, _window, cx| {
            match target {
                DiffHeaderTarget::Worktree => this.toggle_diff_headers(),
                DiffHeaderTarget::History => this.toggle_history_diff_headers(),
            }
            cx.notify();
        }))
        .child(
            div()
                .flex_none()
                .w(px(92.0))
                .text_align(gpui::TextAlign::Right)
                .text_color(rgb(COLOR_TEXT_FAINT))
                .child(""),
        )
        .child(label)
}

pub(crate) fn diff_line(
    kind: DiffLineKind,
    old_lineno: Option<u32>,
    new_lineno: Option<u32>,
    content: String,
) -> impl IntoElement {
    let (bg, fg) = match kind {
        DiffLineKind::Added => (0xe9fff2, 0x168447),
        DiffLineKind::Removed => (0xffeeee, 0xc43a3a),
        DiffLineKind::Header => (0xeef3fb, 0x4b5563),
        DiffLineKind::Context => (COLOR_PANEL_BG, COLOR_TEXT),
    };
    let old_lineno = old_lineno.map(|line| line.to_string()).unwrap_or_default();
    let new_lineno = new_lineno.map(|line| line.to_string()).unwrap_or_default();

    div()
        .flex()
        .items_start()
        .py(px(1.0))
        .bg(rgb(bg))
        .text_color(rgb(fg))
        .child(diff_lineno(old_lineno))
        .child(diff_lineno(new_lineno))
        .child(div().flex_none().px_2().whitespace_nowrap().child(content))
}

fn diff_lineno(line: String) -> impl IntoElement {
    div()
        .flex_none()
        .w(px(46.0))
        .px_1()
        .text_align(gpui::TextAlign::Right)
        .text_color(rgb(COLOR_TEXT_FAINT))
        .bg(rgb(COLOR_HEADER_BG))
        .child(line)
}
