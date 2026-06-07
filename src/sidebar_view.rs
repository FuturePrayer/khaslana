use gpui::{
    ClickEvent, Context, IntoElement, MouseButton, MouseDownEvent, Window, div, prelude::*, px, rgb,
};
use khaslana::{BranchInfo, BranchKind, StashInfo, TagInfo};

use crate::{
    BranchContextMenu, COLOR_BLUE, COLOR_BLUE_DARK, COLOR_BLUE_SOFT, COLOR_BORDER,
    COLOR_BORDER_STRONG, COLOR_PANEL_BG, COLOR_ROW_SELECTED, COLOR_SURFACE, COLOR_TEXT,
    COLOR_TEXT_MUTED, RepositoryView, StashContextMenu, TagContextMenu, context_menu_item,
    menu_separator, nav_list, nav_row, placeholder_row, section_header_action,
};

const COLOR_ROW_SELECTED_BORDER: u32 = 0x9bbcff;

impl RepositoryView {
    pub(crate) fn render_sidebar(
        &self,
        _window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let snapshot = self.snapshot.as_ref();
        let branches = snapshot
            .map(|snapshot| snapshot.branches.clone())
            .unwrap_or_default();
        let local_rows = branches
            .iter()
            .filter(|branch| branch.kind == BranchKind::Local)
            .cloned()
            .map(|branch| self.branch_row(branch, cx).into_any_element())
            .collect::<Vec<_>>();
        let remote_branch_rows = branches
            .into_iter()
            .filter(|branch| branch.kind == BranchKind::Remote)
            .map(|branch| self.branch_row(branch, cx).into_any_element())
            .collect::<Vec<_>>();
        let remote_rows = snapshot
            .map(|snapshot| snapshot.remotes.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|remote| self.remote_row(remote, cx).into_any_element())
            .collect::<Vec<_>>();
        let tag_rows = snapshot
            .map(|snapshot| snapshot.tags.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|tag| self.tag_row(tag, cx).into_any_element())
            .collect::<Vec<_>>();
        let stash_rows = snapshot
            .map(|snapshot| snapshot.stashes.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|stash| self.stash_row(stash, cx).into_any_element())
            .collect::<Vec<_>>();

        let mut sidebar = div()
            .flex()
            .flex_none()
            .flex_col()
            .w(px(self.sidebar_width))
            .min_w(px(self.sidebar_width))
            .h_full()
            .border_r_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(COLOR_PANEL_BG))
            .child(
                self.render_nav_section(
                    "本地分支",
                    "local-branch-list",
                    local_rows,
                    None,
                    3.0,
                    Some(
                        self.button(
                            "新建",
                            self.repo_path.is_some() && !self.busy,
                            |this, _, _| this.open_create_branch_dialog(),
                            cx,
                        )
                        .into_any_element(),
                    ),
                ),
            )
            .child(self.render_nav_section(
                "远端",
                "remote-list",
                remote_rows,
                self.loading.remote().then_some("远端加载中..."),
                2.0,
                None,
            ))
            .child(self.render_nav_section(
                "远端分支",
                "remote-branch-list",
                remote_branch_rows,
                self.loading.remote().then_some("远端分支加载中..."),
                3.0,
                None,
            ));

        if !tag_rows.is_empty() {
            sidebar = sidebar
                .child(self.render_nav_section("标签", "tag-list", tag_rows, None, 2.0, None));
        }
        if !stash_rows.is_empty() {
            sidebar = sidebar.child(self.render_nav_section(
                "贮藏",
                "stash-list",
                stash_rows,
                None,
                2.0,
                None,
            ));
        }

        sidebar
    }

    fn render_nav_section(
        &self,
        title: &'static str,
        id: &'static str,
        rows: Vec<gpui::AnyElement>,
        placeholder: Option<&'static str>,
        weight: f32,
        action: Option<gpui::AnyElement>,
    ) -> impl IntoElement {
        let rows = if rows.is_empty() {
            placeholder
                .map(|text| placeholder_row(text).into_any_element())
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            rows
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(96.0))
            .border_t_1()
            .border_color(rgb(COLOR_BORDER))
            .map(|this| {
                let mut this = this;
                this.style().flex_grow = Some(weight);
                this
            })
            .child(section_header_action(title, action))
            .child(nav_list(id, rows))
    }

    fn remote_row(&self, remote: String, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.current_remote().as_deref() == Some(remote.as_str());
        let name = remote.clone();

        nav_row(format!("remote-{remote}"), false, selected)
            .hover(move |this| {
                if selected {
                    this.bg(rgb(COLOR_BLUE_SOFT))
                } else {
                    this.bg(rgb(0xf5f8ff))
                }
            })
            .child(
                div()
                    .flex_none()
                    .w(px(3.0))
                    .h(px(18.0))
                    .rounded_sm()
                    .bg(if selected {
                        rgb(COLOR_BLUE)
                    } else {
                        rgb(COLOR_BORDER)
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(if selected {
                        rgb(COLOR_BLUE_DARK)
                    } else {
                        rgb(COLOR_TEXT)
                    })
                    .truncate()
                    .child(remote),
            )
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.selected_remote = Some(name.clone());
                this.close_popups();
                cx.notify();
            }))
    }

    fn tag_row(&self, tag: TagInfo, cx: &mut Context<Self>) -> impl IntoElement {
        let name = tag.name.clone();
        let right_click_name = tag.name.clone();

        nav_row(format!("tag-{}", tag.name), false, false)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(name),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    this.branch_context_menu = None;
                    this.change_context_menu = None;
                    this.stash_context_menu = None;
                    this.commit_context_menu = None;
                    this.active_dialog = None;
                    this.tag_context_menu = Some(TagContextMenu {
                        tag: right_click_name.clone(),
                        x: event.position.x.into(),
                        y: event.position.y.into(),
                    });
                    cx.notify();
                }),
            )
    }

    fn stash_row(&self, stash: StashInfo, cx: &mut Context<Self>) -> impl IntoElement {
        let index = stash.index;
        let label = format!("stash@{{{}}} {}", stash.index, stash.message);

        nav_row(format!("stash-{}", stash.index), false, false)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .overflow_hidden()
                    .child(label),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    this.branch_context_menu = None;
                    this.change_context_menu = None;
                    this.tag_context_menu = None;
                    this.commit_context_menu = None;
                    this.active_dialog = None;
                    this.stash_context_menu = Some(StashContextMenu {
                        index,
                        x: event.position.x.into(),
                        y: event.position.y.into(),
                    });
                    cx.notify();
                }),
            )
    }

    fn branch_row(&self, branch: BranchInfo, cx: &mut Context<Self>) -> impl IntoElement {
        let is_local = branch.kind == BranchKind::Local;
        let is_current = branch.is_head;
        let name = branch.name.clone();
        let click_name = branch.name.clone();
        let click_kind = branch.kind.clone();
        let click_is_head = branch.is_head;
        let right_click_name = branch.name.clone();
        let right_click_kind = branch.kind.clone();
        let right_click_is_head = branch.is_head;
        let marker = if branch.is_head { "* " } else { "" };
        let selected = self.selected_branch.as_deref() == Some(&branch.name);
        let label = match branch.kind {
            BranchKind::Local => format!("{marker}{}", branch.name),
            BranchKind::Remote => format!("  {}", branch.name),
        };
        let row_bg = if is_current {
            COLOR_BLUE_SOFT
        } else if selected {
            COLOR_ROW_SELECTED
        } else {
            COLOR_SURFACE
        };
        let row_border = if is_current {
            COLOR_BORDER_STRONG
        } else if selected {
            COLOR_ROW_SELECTED_BORDER
        } else {
            COLOR_BORDER
        };
        let marker_bg = if is_current {
            COLOR_BLUE
        } else if selected {
            COLOR_ROW_SELECTED_BORDER
        } else {
            COLOR_SURFACE
        };

        div()
            .id(format!("branch-{}", branch.name))
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_2()
            .py_1()
            .rounded_sm()
            .bg(rgb(row_bg))
            .border_1()
            .border_color(rgb(row_border))
            .hover(move |this| {
                if is_current {
                    this.bg(rgb(COLOR_BLUE_SOFT))
                } else {
                    this.bg(rgb(0xf5f8ff))
                }
            })
            .child(
                div()
                    .flex_none()
                    .w(px(3.0))
                    .h(px(18.0))
                    .rounded_sm()
                    .bg(rgb(marker_bg)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(if is_current {
                        rgb(COLOR_BLUE_DARK)
                    } else if is_local {
                        rgb(COLOR_TEXT)
                    } else {
                        rgb(COLOR_TEXT_MUTED)
                    })
                    .truncate()
                    .child(label),
            )
            .cursor_pointer()
            .on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
                this.selected_branch = Some(name.clone());
                this.branch_context_menu = None;
                this.change_context_menu = None;
                this.commit_context_menu = None;
                if event.standard_click() && event.click_count() >= 2 && !this.busy {
                    match click_kind {
                        BranchKind::Local if !click_is_head => this.checkout(click_name.clone()),
                        BranchKind::Remote if !this.has_local_branch_for_remote(&click_name) => {
                            this.checkout_remote_branch(click_name.clone())
                        }
                        _ => {}
                    }
                }
                cx.notify();
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    this.selected_branch = Some(right_click_name.clone());
                    this.active_dialog = None;
                    this.branch_context_menu = Some(BranchContextMenu {
                        branch: right_click_name.clone(),
                        kind: right_click_kind.clone(),
                        is_head: right_click_is_head,
                        x: event.position.x.into(),
                        y: event.position.y.into(),
                    });
                    this.tag_context_menu = None;
                    this.stash_context_menu = None;
                    this.change_context_menu = None;
                    this.commit_context_menu = None;
                    cx.notify();
                }),
            )
    }

    pub(crate) fn render_branch_context_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(menu) = self.branch_context_menu.clone() else {
            return div().into_any_element();
        };
        let is_local = menu.kind == BranchKind::Local;

        div()
            .absolute()
            .left(px(menu.x))
            .top(px(menu.y))
            .w(px(190.0))
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .text_size(px(12.0))
            .child(context_menu_item(
                "切换到此分支",
                is_local && !menu.is_head && !self.busy,
                {
                    let branch = menu.branch.clone();
                    move |this| this.checkout(branch.clone())
                },
                cx,
            ))
            .child(context_menu_item(
                "合并到当前分支",
                !menu.is_head && !self.busy,
                {
                    let branch = menu.branch.clone();
                    move |this| this.merge_branch(branch.clone())
                },
                cx,
            ))
            .child(context_menu_item(
                "拉取到本地并切换",
                !is_local && !self.busy,
                {
                    let branch = menu.branch.clone();
                    move |this| this.checkout_remote_branch(branch.clone())
                },
                cx,
            ))
            .child(menu_separator())
            .child(context_menu_item(
                "重命名...",
                is_local && !self.busy,
                {
                    let branch = menu.branch.clone();
                    move |this| this.open_rename_branch_dialog(branch.clone())
                },
                cx,
            ))
            .child(context_menu_item(
                "删除分支",
                is_local && !menu.is_head && !self.busy,
                {
                    let branch = menu.branch.clone();
                    move |this| this.delete_branch(branch.clone())
                },
                cx,
            ))
            .into_any_element()
    }
}
