use gpui::{
    ClickEvent, Context, IntoElement, MouseButton, MouseDownEvent, Window, div, prelude::*, px, rgb,
};
use khaslana::{BranchInfo, BranchKind, RemoteInfo, StashInfo, TagInfo};

use crate::{
    BRANCH_MENU_HEIGHT, BRANCH_MENU_WIDTH, BranchContextMenu, NAV_ROW_HEIGHT, RepositoryView,
    STASH_MENU_HEIGHT, STASH_MENU_WIDTH, SidebarSection, StashContextMenu, TAG_MENU_HEIGHT,
    TAG_MENU_WIDTH, TagContextMenu, clamped_menu_position, context_menu_item, menu_separator,
    nav_list, nav_row, placeholder_row, ui::theme as ui_theme,
};

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
            .border_color(rgb(ui_theme::BORDER))
            .bg(rgb(ui_theme::PANEL_BG))
            .child(
                self.render_nav_section(
                    "本地分支",
                    "local-branch-list",
                    SidebarSection::LocalBranches,
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
                    cx,
                ),
            )
            .child(
                self.render_nav_section(
                    "远端",
                    "remote-list",
                    SidebarSection::Remotes,
                    remote_rows,
                    self.loading.remote().then_some("远端加载中..."),
                    2.0,
                    Some(
                        self.button(
                            "管理",
                            self.repo_path.is_some() && !self.busy,
                            |this, _, _| this.open_remote_manager(),
                            cx,
                        )
                        .into_any_element(),
                    ),
                    cx,
                ),
            )
            .child(self.render_nav_section(
                "远端分支",
                "remote-branch-list",
                SidebarSection::RemoteBranches,
                remote_branch_rows,
                self.loading.remote().then_some("远端分支加载中..."),
                3.0,
                None,
                cx,
            ));

        if !tag_rows.is_empty() {
            sidebar = sidebar.child(self.render_nav_section(
                "标签",
                "tag-list",
                SidebarSection::Tags,
                tag_rows,
                None,
                2.0,
                None,
                cx,
            ));
        }
        if !stash_rows.is_empty() {
            sidebar = sidebar.child(self.render_nav_section(
                "贮藏",
                "stash-list",
                SidebarSection::Stashes,
                stash_rows,
                None,
                2.0,
                None,
                cx,
            ));
        }

        sidebar
    }

    fn render_nav_section(
        &self,
        title: &'static str,
        id: &'static str,
        section: SidebarSection,
        rows: Vec<gpui::AnyElement>,
        placeholder: Option<&'static str>,
        weight: f32,
        action: Option<gpui::AnyElement>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let expanded = self.sidebar_sections.is_expanded(section);
        let header = self.nav_section_header(title, section, expanded, action, cx);
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
            .when(expanded, |this| this.flex_1().min_h(px(96.0)))
            .when(!expanded, |this| this.flex_none())
            .border_t_1()
            .border_color(rgb(ui_theme::BORDER))
            .when(expanded, |this| {
                let mut this = this;
                this.style().flex_grow = Some(weight);
                this
            })
            .child(header)
            .when(expanded, |this| this.child(nav_list(self, id, rows, cx)))
    }

    fn nav_section_header(
        &self,
        title: &'static str,
        section: SidebarSection,
        expanded: bool,
        action: Option<gpui::AnyElement>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let toggle_label = if expanded { "∧" } else { "∨" };
        let toggle = div()
            .id(format!("sidebar-section-toggle-{title}"))
            .flex_none()
            .size(px(24.0))
            .rounded_sm()
            .flex()
            .items_center()
            .justify_center()
            .border_1()
            .border_color(rgb(ui_theme::BORDER))
            .bg(rgb(ui_theme::SURFACE))
            .text_size(px(13.0))
            .text_color(rgb(ui_theme::TEXT_MUTED))
            .cursor_pointer()
            .hover(|this| this.bg(rgb(ui_theme::ROW_HOVER)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.toggle_sidebar_section(section);
                cx.notify();
            }))
            .child(toggle_label)
            .into_any_element();

        let actions = div()
            .flex_none()
            .flex()
            .items_center()
            .gap_1()
            .when_some(action, |this, action| this.child(action))
            .child(toggle);

        div()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(rgb(ui_theme::BORDER))
            .bg(rgb(ui_theme::HEADER_BG))
            .child(
                div()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(ui_theme::TEXT))
                    .truncate()
                    .child(title),
            )
            .child(actions)
    }

    fn remote_row(&self, remote: RemoteInfo, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.current_remote().as_deref() == Some(remote.name.as_str());
        let name = remote.name.clone();

        nav_row(format!("remote-{}", remote.name), false, selected)
            .hover(move |this| {
                if selected {
                    this.bg(rgb(ui_theme::ACCENT_SOFT))
                } else {
                    this.bg(rgb(ui_theme::ROW_HOVER))
                }
            })
            .child(
                div()
                    .flex_none()
                    .w(px(3.0))
                    .h(px(18.0))
                    .rounded_sm()
                    .bg(if selected {
                        rgb(ui_theme::ACCENT)
                    } else {
                        rgb(ui_theme::BORDER)
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(if selected {
                        rgb(ui_theme::ACCENT_STRONG)
                    } else {
                        rgb(ui_theme::TEXT)
                    })
                    .truncate()
                    .child(remote.name),
            )
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.selected_remote = Some(name.clone());
                this.close_popups();
                if let Some((tab_id, path, remote, load_id, request_id)) =
                    this.prepare_branch_sync_status_request()
                {
                    this.load_branch_sync_status_for_tab(tab_id, path, remote, load_id, request_id);
                }
                cx.notify();
            }))
    }

    fn tag_row(&self, tag: TagInfo, cx: &mut Context<Self>) -> impl IntoElement {
        let name = tag.name.clone();
        let right_click_name = tag.name.clone();

        nav_row(format!("tag-{}", tag.name), false, false)
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(rgb(ui_theme::TEXT_MUTED))
                    .truncate()
                    .child(name),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    this.branch_context_menu = None;
                    this.change_context_menu = None;
                    this.stash_context_menu = None;
                    this.commit_context_menu = None;
                    this.encoding_menu_target = None;
                    this.active_dialog = None;
                    let (x, y) =
                        clamped_menu_position(event, window, TAG_MENU_WIDTH, TAG_MENU_HEIGHT);
                    this.tag_context_menu = Some(TagContextMenu {
                        tag: right_click_name.clone(),
                        x,
                        y,
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
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(rgb(ui_theme::TEXT_MUTED))
                    .truncate()
                    .child(label),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    this.branch_context_menu = None;
                    this.change_context_menu = None;
                    this.tag_context_menu = None;
                    this.commit_context_menu = None;
                    this.encoding_menu_target = None;
                    this.active_dialog = None;
                    let (x, y) =
                        clamped_menu_position(event, window, STASH_MENU_WIDTH, STASH_MENU_HEIGHT);
                    this.stash_context_menu = Some(StashContextMenu { index, x, y });
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
            ui_theme::ACCENT_SOFT
        } else if selected {
            ui_theme::ROW_SELECTED
        } else {
            ui_theme::SURFACE
        };
        let row_border = if is_current {
            ui_theme::BORDER_STRONG
        } else if selected {
            ui_theme::ROW_SELECTED_BORDER
        } else {
            ui_theme::BORDER
        };
        let marker_bg = if is_current {
            ui_theme::ACCENT
        } else if selected {
            ui_theme::ROW_SELECTED_BORDER
        } else {
            ui_theme::SURFACE
        };

        div()
            .id(format!("branch-{}", branch.name))
            .flex()
            .h(px(NAV_ROW_HEIGHT))
            .min_h(px(NAV_ROW_HEIGHT))
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
                    this.bg(rgb(ui_theme::ACCENT_SOFT))
                } else {
                    this.bg(rgb(ui_theme::ROW_HOVER))
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
                        rgb(ui_theme::ACCENT_STRONG)
                    } else if is_local {
                        rgb(ui_theme::TEXT)
                    } else {
                        rgb(ui_theme::TEXT_MUTED)
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
                this.encoding_menu_target = None;
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
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    this.selected_branch = Some(right_click_name.clone());
                    this.active_dialog = None;
                    let (x, y) =
                        clamped_menu_position(event, window, BRANCH_MENU_WIDTH, BRANCH_MENU_HEIGHT);
                    this.branch_context_menu = Some(BranchContextMenu {
                        branch: right_click_name.clone(),
                        kind: right_click_kind.clone(),
                        is_head: right_click_is_head,
                        x,
                        y,
                    });
                    this.tag_context_menu = None;
                    this.stash_context_menu = None;
                    this.change_context_menu = None;
                    this.commit_context_menu = None;
                    this.encoding_menu_target = None;
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
            .w(px(BRANCH_MENU_WIDTH))
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(ui_theme::BORDER_STRONG))
            .bg(rgb(ui_theme::SURFACE))
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
