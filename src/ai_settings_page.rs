//! AI 设置页：作为同一个 Kit Settings 页面中的自定义无边框分组渲染。
//!
//! 分类导航、搜索与拖拽宽度全部由设置中心管理，不再另建 AI 侧栏。
//! AI 使用 Normal 分组变体，避免 GroupBox 内层的固定边框和内距。
//! 组内主体有确定高度，固定页头、页签与保存栏，仅当前页签内容滚动。
//! 有界外层必须是 flex 容器，让 scrollable_frame_when 的 flex_1 真正生效；
//! 滚动视口裁切内容，避免长文案画到保存栏上。

use khaslana::ui_i18n::{text, format as message, encode, resolve};
#[path = "ai_settings_ui_resources.rs"]
mod messages;

use gpui::{Context, Div, FontWeight, IntoElement, Window, div, prelude::*, px};
use gpui_kit::assets::IconName;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::{
    Icon, Size, Sizable, h_flex, v_flex,
};
pub(crate) mod state;

#[cfg(test)]
#[path = "tests/ai_settings_page.rs"]
mod tests;

use crate::{DialogState, FieldId, RepositoryView, ScrollbarMode,
    ai_extensions_view::AiSettingsTab, dialog_panel_size, scrollable_frame_when,
    ui::{components::{SettingsButtonTone, settings_command_button},
        theme::{self as ui_theme, rgb}}};

/// AI 页内容区的滚动 id。与其它页面的滚动容器同名空间隔离。
const AI_SETTINGS_PAGE_SCROLL_ID: &str = "ai-settings-page-scroll";

/// 开关与运行环境使用无底色图标。
fn card_icon(icon: IconName) -> impl IntoElement {
    div().flex_none().size(px(16.0))
        .flex().items_center().justify_center()
        .child(Icon::new(icon).with_size(Size::Size(px(16.0))))
}

/// 紧凑列表使用 24px 图标，避免方形色底抢占层级。
fn list_row_icon(icon: IconName) -> impl IntoElement {
    div().flex_none().size(px(24.0))
        .flex().items_center().justify_center()
        .child(Icon::new(icon).with_size(Size::Size(px(24.0)))
            .text_color(rgb(ui_theme::CONTENT_SECONDARY)))
}

/// 列表行右侧的「更多」下拉菜单：弱强调的文字按钮，点击展开 PopupMenu。
///
/// 行内操作（查看 / 编辑 / 移除）全部收进这里，按设计稿不再在行右侧平铺按钮。
/// `dropdown_menu` 的构建闭包是 `Fn`（每次展开都重跑），而 `PopupMenuItem`
/// 不实现 `Clone`，因此由调用方传入一个可重复调用的「给菜单追加条目」闭包，
/// 每次展开现场重建条目——条目里的点击回调本来就是各自 clone 出来的。
fn row_menu<F>(id: impl Into<gpui::ElementId>, build: F, cx: &gpui::App) -> impl IntoElement
where
    F: Fn(PopupMenu) -> PopupMenu + 'static,
{
    settings_command_button(id, text(messages::MORE), SettingsButtonTone::Secondary, true, cx)
        .h(px(28.0)).min_w(px(64.0)).px(px(10.0)).accessibility_label(text(messages::MORE_ACTIONS))
        .dropdown_menu(move |menu, _window, _cx| build(menu))
}

fn heading(text: impl Into<gpui::SharedString>) -> Div {
    div().text_size(px(ui_theme::TYPE_TITLE)).font_weight(FontWeight::SEMIBOLD)
        .line_height(px(20.0))
        .text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(text.into())
}

fn body(text: impl Into<gpui::SharedString>) -> Div {
    div().text_size(px(ui_theme::TYPE_BODY))
        .text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(text.into())
}

fn caption(text: impl Into<gpui::SharedString>) -> Div {
    div().text_size(px(ui_theme::TYPE_META)).line_height(px(16.0))
        .text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(text.into())
}

fn error_text(text: impl Into<gpui::SharedString>) -> Div {
    div().text_size(px(ui_theme::TYPE_META)).line_height(px(16.0))
        .text_color(rgb(ui_theme::DESTRUCTIVE)).child(text.into())
}

fn status_text(label: &'static str, positive: bool) -> Div {
    let (bg, border, fg) = if positive {
        (ui_theme::FEEDBACK_SUCCESS_BG, ui_theme::FEEDBACK_SUCCESS_BORDER, ui_theme::FEEDBACK_SUCCESS_TEXT)
    } else {
        (ui_theme::FEEDBACK_WARNING_BG, ui_theme::FEEDBACK_WARNING_BORDER, ui_theme::FEEDBACK_WARNING_TEXT)
    };
    h_flex().flex_none().h(px(24.0)).px_2().gap_1().items_center()
        .rounded(px(ui_theme::RADIUS_XS)).border_1()
        .border_color(rgb(border).opacity(0.28)).bg(rgb(bg).opacity(0.20))
        .text_color(rgb(fg)).text_size(px(ui_theme::TYPE_META)).line_height(px(14.0))
        .child(Icon::new(if positive { IconName::Check } else { IconName::Info })
            .with_size(Size::Size(px(12.0)))).child(label)
}

fn card() -> Div {
    v_flex().w_full().min_w(px(0.0)).gap_3().p_3()
        .rounded(px(ui_theme::RADIUS_SM))
        .bg(rgb(ui_theme::WB_NAV))
}

fn form_row(label: &'static str, value: impl IntoElement, compact: bool) -> Div {
    div().flex().w_full().min_w(px(0.0)).gap_3()
        .when(compact, |this| this.flex_col())
        .when(!compact, |this| this.flex_wrap().items_start())
        .child(body(label).flex_none().w(px(122.0)).pt(px(8.0)))
        .child(div().min_w(px(0.0))
            .when(compact, |this| this.w_full())
            .when(!compact, |this| this.flex_1().flex_basis(px(240.0)))
            .child(value))
}

impl RepositoryView {
    pub(crate) fn ensure_ai_api_type_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        use gpui_kit::component::select::{SelectEvent, SelectState};
        if self.ai_api_type_select.is_none() {
            let state = cx.new(|cx| SelectState::new(state::API_TYPES.iter()
                .map(|kind| kind.label().to_string()).collect::<Vec<_>>(), None, window, cx));
            cx.subscribe_in(&state, window, |this, _, event: &SelectEvent<Vec<String>>, _, cx| {
                let SelectEvent::Confirm(Some(value)) = event else { return };
                if let Some(kind) = state::API_TYPES.iter().find(|kind| kind.label() == value) {
                    this.ai_api_type_form = *kind;
                    this.ai_extensions.save_error = None;
                    cx.notify();
                }
            }).detach();
            self.ai_api_type_select = Some(state);
        }
        let state = self.ai_api_type_select.as_ref().expect("协议选择器已初始化");
        let value = self.ai_api_type_form.label().to_string();
        if state.read(cx).selected_value() != Some(&value) {
            state.update(cx, |state, cx| state.set_selected_value(&value, window, cx));
        }
    }

    /// AI 是同一个 Kit Settings 的页面；这里只渲染右侧内容。
    pub(crate) fn render_ai_settings_page(&self, window: &Window,
        cx: &mut Context<Self>) -> impl IntoElement {
        let (panel_width, panel_height) = dialog_panel_size(window, 900.0, 640.0);
        let compact = f32::from(panel_width) - 250.0 < 460.0;
        // 共同顶栏 52px、底部内距 12px、Kit 列表单组上下各 16px。
        // 给组内主体确定高度，内部滚动不会撑开虚拟列表并挤走保存栏。
        let body_height = (f32::from(panel_height) - 96.0).max(0.0);
        div().w_full().h(px(body_height)).min_h(px(0.0))
            .child(self.render_ai_settings_body(window, compact, cx))
    }

    /// 页头、页签和保存栏固定；只有页签主体滚动。
    fn render_ai_settings_body(&self, window: &Window, compact: bool,
        cx: &mut Context<Self>) -> impl IntoElement {
        let handle = self.scroll_handle(AI_SETTINGS_PAGE_SCROLL_ID);
        let content = div().id(AI_SETTINGS_PAGE_SCROLL_ID).size_full()
            .overflow_y_scroll().track_scroll(&handle)
            .child(v_flex().w_full().min_w(px(0.0)).px(px(8.0)).py_2()
                .child(match (self.ai_settings.enabled, self.ai_extensions.tab) {
                    (false, _) => self.render_ai_locked_content(cx).into_any_element(),
                    (true, AiSettingsTab::Connection) =>
                        self.render_ai_connection_content(window, compact, cx).into_any_element(),
                    (true, AiSettingsTab::Skill) => self.render_ai_skill_content(cx).into_any_element(),
                    (true, AiSettingsTab::Mcp) => self.render_ai_mcp_content(cx).into_any_element(),
                    (true, AiSettingsTab::Runtime) => self.render_ai_runtime_content(cx).into_any_element(),
                }));
        v_flex().size_full().min_w(px(0.0)).min_h(px(0.0))
            .child(v_flex().flex_none().w_full().gap_3().px(px(8.0))
                .child(v_flex().gap_1()
                    .child(div().text_size(px(20.0)).font_weight(FontWeight::BOLD)
                        .text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(text(messages::TITLE)))
                    .child(caption(text(messages::DESCRIPTION))))
                .child(self.render_ai_enable_card(cx))
                .child(self.render_ai_page_tabs(cx)))
            // 有界外层直接包滚动 helper，内容 div 不设置 flex_1 / min_h。
            .child(v_flex().flex_1().min_h(px(0.0)).w_full().overflow_hidden()
                .child(scrollable_frame_when(AI_SETTINGS_PAGE_SCROLL_ID, ScrollbarMode::Vertical,
                    content.into_any_element(), handle, true, cx)))
            .child(self.render_ai_save_bar(cx))
    }

    fn render_ai_save_bar(&self, cx: &mut Context<Self>) -> Div {
        let view = cx.entity();
        let changed = state::form_is_dirty(&self.ai_form_settings(), &self.ai_settings);
        let status = if let Some(error) = &self.ai_extensions.save_error {
            error_text(resolve(error))
        } else if changed {
            caption(text(messages::DIRTY))
                .text_color(rgb(ui_theme::FEEDBACK_WARNING_TEXT))
        } else {
            caption(text(messages::SAVED))
        };
        h_flex().flex_none().w_full().min_h(px(64.0)).px(px(8.0)).py_3().gap_3()
            .border_t_1().border_color(rgb(ui_theme::BORDER_MUTED)).bg(rgb(ui_theme::WB_PANEL))
            .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                .child(status)
                .child(caption(text(messages::SAVE_SCOPE))))
            .child(settings_command_button("ai-save-settings", text(messages::SAVE), SettingsButtonTone::Primary, changed, cx)
                .w(px(96.0))
                .tooltip(if changed { text(messages::SAVE_HINT) } else { text(messages::UNCHANGED) })
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| {
                        if this.save_ai_provider_settings_from_form() {
                            this.notify_settings_save(encode(messages::SAVE_SUCCESS, &[]), cx);
                        }
                        cx.notify();
                    });
                }))
    }

    fn render_ai_enable_card(&self, cx: &mut Context<Self>) -> Div {
        card().flex_row().items_center().justify_between().gap_3().min_h(px(48.0)).py_2()
            .child(card_icon(IconName::Sparkles))
            .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                .child(body(text(messages::ENABLE)).font_weight(FontWeight::SEMIBOLD))
                .child(caption(text(messages::ENABLE_DESCRIPTION))))
            .child(self.toggle_switch("ai-enabled", self.ai_enabled_form, false,
                |this, enabled, _, _| this.set_ai_enabled_form(enabled), cx))
    }

    fn render_ai_page_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        // Kit 0.6.4 的 Tab 会覆盖调用方的底色、圆角与边框，无法表现稿中的
        // 淡色选中块。这里复用 Kit Button，补齐页签方向键与焦点切换。
        h_flex().id("ai-settings-tabs").w_full().flex_wrap().gap_1()
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if !this.ai_settings.enabled { return; }
                let current = this.ai_extensions.tab.index();
                let index = match event.keystroke.key.as_str() {
                    "left" => (current + 3) % 4,
                    "right" => (current + 1) % 4,
                    "home" => 0,
                    "end" => 3,
                    _ => return,
                };
                this.select_ai_settings_tab(AiSettingsTab::from_index(index));
                let focus = window.use_keyed_state(format!("ai-settings-tab-{index}"), cx,
                    |_, cx| cx.focus_handle()).read(cx).clone();
                focus.focus(window, cx);
                cx.stop_propagation();
                cx.notify();
            }))
            .children([text(messages::CONNECTION), "Skill", "MCP", text(messages::RUNTIME)].into_iter().enumerate()
                .map(|(index, label)| {
                    let selected = self.ai_extensions.tab.index() == index;
                    let view = view.clone();
                    settings_command_button(format!("ai-settings-tab-{index}"), label,
                        if selected { SettingsButtonTone::SelectedTab } else { SettingsButtonTone::Quiet },
                        self.ai_settings.enabled, cx)
                        .accessibility_label(message(messages::TAB_LABEL, &[("label", label)])).toggled(selected)
                        .h(px(36.0)).min_w(px(0.0))
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.select_ai_settings_tab(AiSettingsTab::from_index(index));
                                cx.notify();
                            });
                        })
                }))
    }

    fn render_ai_locked_content(&self, cx: &mut Context<Self>) -> Div {
        let view = cx.entity();
        card().items_center().gap_3().py_8()
            .child(card_icon(IconName::Lock))
            .child(heading(text(messages::LOCKED)))
            .child(caption(text(messages::LOCKED_DESCRIPTION)))
            .child(caption(text(messages::LOCKED_HINT)))
            .child(settings_command_button("ai-enable-from-locked", text(messages::TURN_ON), SettingsButtonTone::Primary, true, cx).on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| {
                        this.set_ai_enabled_form(true);
                        cx.notify();
                    });
                }))
    }

    fn render_ai_connection_content(&self, window: &Window, compact: bool,
        cx: &mut Context<Self>) -> Div {
        let field = |id, helper: Div, cx: &mut Context<Self>| {
            v_flex().w_full().min_w(px(0.0)).gap_1()
                .child(self.input(id, false, window, cx)).child(helper)
        };
        let settings = self.ai_form_settings();
        let test_state = &self.ai_extensions.connection_test;
        let outcome = settings.as_ref().ok().map(|settings| test_state.outcome_for(settings))
            .unwrap_or(&state::ConnectionTestOutcome::Idle);
        let feedback = match outcome {
            state::ConnectionTestOutcome::Idle if test_state.is_running() => caption(text(messages::TEST_OLD)),
            state::ConnectionTestOutcome::Idle => caption(text(messages::TEST_IDLE)),
            state::ConnectionTestOutcome::Running => caption(text(messages::TEST_RUNNING)),
            state::ConnectionTestOutcome::Passed(message) => caption(resolve(message))
                .text_color(rgb(ui_theme::FEEDBACK_SUCCESS_TEXT)),
            state::ConnectionTestOutcome::Failed(error) => error_text(resolve(error)),
        };
        let blocked = self.busy || self.global_busy_tab.is_some();
        let protocol = self.ai_api_type_select.as_ref().map(|state| {
            gpui_kit::component::select::Select::new(state)
                .accessibility_label(text(messages::API_TYPE)).w_full().h(px(34.0))
                .text_size(px(ui_theme::TYPE_BODY))
                .bg(rgb(ui_theme::WB_INPUT_SURFACE)).border_color(rgb(ui_theme::BORDER_MUTED))
                .rounded(px(ui_theme::RADIUS_SM)).into_any_element()
        }).unwrap_or_else(|| div().into_any_element());
        let view = cx.entity();
        v_flex().w_full().min_w(px(0.0)).gap_3()
            .child(form_row(text(messages::API_TYPE), protocol, compact))
            .when(self.ai_api_type_form == khaslana::AiApiType::AnthropicMessages, |this|
                this.child(caption(text(messages::CLAUDE_HINT))))
            .child(form_row(text(messages::BASE_URL), field(FieldId::AiBaseUrl,
                caption(message(messages::BASE_URL_HINT, &[("endpoint", self.ai_api_type_form.endpoint_path())])), cx), compact))
            .child(form_row(text(messages::API_KEY), field(FieldId::AiApiKey,
                caption(text(messages::KEY_WARNING))
                    .text_color(rgb(ui_theme::FEEDBACK_WARNING_TEXT)), cx), compact))
            .child(form_row(text(messages::MODEL), field(FieldId::AiModel,
                caption(text(messages::MODEL_HINT)), cx), compact))
            .child(settings_command_button("ai-advanced-settings", text(messages::ADVANCED),
                SettingsButtonTone::Quiet, true, cx)
                .toggled(self.ai_advanced_expanded)
                .on_click(cx.listener(|this, _, window, cx| {
                    if this.ai_advanced_expanded && [FieldId::AiTemperature,
                        FieldId::AiMaxTokens, FieldId::AiRequestTimeout, FieldId::AiReasoningEffort]
                        .iter().any(|id| this.field(*id).focus.is_focused(window)) {
                        // 隐藏输入前归还焦点，避免 Tab/Enter 仍落到已卸载字段。
                        let focus = window.use_keyed_state("ai-advanced-settings", cx,
                            |_, cx| cx.focus_handle()).read(cx).clone();
                        focus.focus(window, cx);
                    }
                    this.ai_advanced_expanded = !this.ai_advanced_expanded;
                    cx.notify();
                })))
            .when(self.ai_advanced_expanded, |this| this.child(v_flex().w_full().gap_3()
                .child(form_row(text(messages::TEMPERATURE), field(FieldId::AiTemperature, caption(if self.ai_api_type_form == khaslana::AiApiType::AnthropicMessages { text(messages::TEMPERATURE_ONE) } else { text(messages::TEMPERATURE_TWO) }), cx), compact))
                .child(form_row(text(messages::MAX_OUTPUT), field(FieldId::AiMaxTokens, caption(text(messages::MAX_OUTPUT_HINT)), cx), compact))
                .child(form_row(text(messages::TIMEOUT), field(FieldId::AiRequestTimeout, caption(text(messages::TIMEOUT_HINT)), cx), compact))
                .child(form_row(text(messages::REASONING), field(FieldId::AiReasoningEffort, caption(text(messages::REASONING_HINT)), cx), compact))))
            .when_some(settings.as_ref().err(), |this, error| this.child(error_text(resolve(error))))
            .child(card().flex_row().items_center().gap_3().min_h(px(56.0)).py_2()
                .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                    .child(feedback)
                    .child(caption(if blocked && !test_state.is_running() {
                        text(messages::TEST_BLOCKED)
                    } else {
                        text(messages::TEST_HINT)
                    })))
                .child(settings_command_button("ai-test-connection",
                    if test_state.is_running() { text(messages::TESTING) } else { text(messages::TEST) },
                    SettingsButtonTone::Secondary, !blocked && !test_state.is_running(), cx)
                    .on_click(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.test_ai_connection();
                            cx.notify();
                        });
                    })))
    }

    fn render_ai_skill_content(&self, cx: &mut Context<Self>) -> Div {
        let view = cx.entity();
        let count = self.ai_extensions.skills.len();
        // 分区头：标题 + 数量摘要 + 主动作按钮。
        let mut content = v_flex().w_full().min_w(px(0.0)).gap_0()
            .child(h_flex().w_full().items_center().justify_between().gap_3().pb_4()
                .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                    .child(heading(text(messages::SKILLS)))
                    .child(caption(message(messages::SKILL_COUNT, &[("count", &count.to_string())]))))
                .child(h_flex().flex_none().gap_2()
                    .child(settings_command_button("ai-skill-open-folder", text(messages::OPEN_FOLDER), SettingsButtonTone::Quiet, !self.ai_extensions.loading, cx)
                        .on_click({ let view = view.clone(); move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.open_ai_skill_folder(cx);
                                cx.notify();
                            });
                        }}))
                    .child(settings_command_button("ai-skill-import", text(messages::IMPORT_SKILL), SettingsButtonTone::Primary, !self.ai_extensions.loading, cx)
                        .on_click({ let view = view.clone(); move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.open_ai_skill_import();
                                cx.notify();
                            });
                        }}))));
        if let Some(error) = &self.ai_extensions.error {
            content = content.child(error_text(resolve(error)));
        }
        if self.ai_extensions.loading {
            content = content.child(card().child(caption(text(messages::LOADING_SKILLS))));
        } else if self.ai_extensions.skills.is_empty() {
            content = content.child(card()
                .flex_row().items_center().gap_3()
                .child(list_row_icon(IconName::Package))
                .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                    .child(heading(text(messages::NO_SKILLS)))
                    .child(caption(text(messages::INSTALL_SKILL_HINT)))));
        } else {
            content = content.child(div().w_full().border_t_1()
                .border_color(rgb(ui_theme::BORDER_MUTED)));
            for (index, (name, description)) in self.ai_extensions.skills.iter().enumerate() {
                let details_name = name.clone();
                let remove_name = name.clone();
                let menu_view = view.clone();
                let build_menu = move |menu: PopupMenu| {
                    let details = details_name.clone();
                    let remove = remove_name.clone();
                    let view = menu_view.clone();
                    menu.item(PopupMenuItem::new(text(messages::DETAILS)).on_click(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.open_ai_skill_details(details.clone());
                            cx.notify();
                        });
                    }))
                    .item({
                        let view = menu_view.clone();
                        PopupMenuItem::new(text(messages::REMOVE)).on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.active_dialog = Some(DialogState::AiSkillRemove { name: remove.clone() });
                                cx.notify();
                            });
                        })
                    })
                };
                content = content.child(h_flex().w_full().items_center().gap_3()
                    .min_h(px(72.0)).py_3()
                    .when(index > 0, |this| this.border_t_1().border_color(rgb(ui_theme::BORDER_MUTED)))
                    .child(list_row_icon(IconName::FileText))
                    .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                        .child(body(name.clone()).font_weight(FontWeight::SEMIBOLD).truncate())
                        .child(caption(description.clone()).truncate()))
                    .child(status_text(text(messages::INSTALLED), true))
                    .child(row_menu(format!("ai-skill-menu-{name}"), build_menu, cx)));
            }
        }
        content.child(div().w_full().border_t_1().border_color(rgb(ui_theme::BORDER_MUTED)).pt_3()
            .child(caption(text(messages::IMPORT_SKILL_HINT)))
            .child(caption(text(messages::SKILL_IMMEDIATE))))
    }

    fn render_ai_mcp_content(&self, cx: &mut Context<Self>) -> Div {
        let view = cx.entity();
        let ready = self.ai_extensions.runtime.as_ref().is_some_and(|info| info.mcp_ready);
        let custom_count = self.ai_extensions.mcp.servers.len();
        // 分区头：内置与自定义服务合并进同一列表，数量摘要区分两类。
        let mut content = v_flex().w_full().min_w(px(0.0)).gap_0()
            .child(h_flex().w_full().items_center().justify_between().gap_3().pb_4()
                .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                    .child(heading(text(messages::MCP)))
                    .child(caption(message(messages::MCP_COUNT, &[("count", &custom_count.to_string())]))))
                .child(settings_command_button("ai-mcp-add", text(messages::ADD_SERVER), SettingsButtonTone::Primary, !self.ai_extensions.loading, cx)
                    .on_click({ let view = view.clone(); move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.open_ai_mcp_form(None);
                            cx.notify();
                        });
                    }})));
        if let Some(error) = &self.ai_extensions.error {
            content = content.child(error_text(resolve(error)));
        }
        if self.ai_extensions.loading {
            content = content.child(card().child(caption(text(messages::LOADING_MCP))));
        } else {
            // 内置 browser.edge 与自定义服务同列表展示，靠描述前缀和图标区分。
            content = content.child(div().w_full().border_t_1()
                .border_color(rgb(ui_theme::BORDER_MUTED))
                .min_h(px(72.0)).py(px(ui_theme::SPACE_3)).flex().items_center()
                .child(h_flex().w_full().items_center().justify_between().gap_3()
                    .child(h_flex().flex_1().min_w(px(0.0)).items_center().gap_3()
                        .child(list_row_icon(IconName::Globe))
                        .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                            .child(div().text_size(px(ui_theme::TYPE_BODY))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                                .child("browser.edge"))
                            .child(caption(text(messages::BUILTIN_BROWSER)).truncate())))
                    .child(h_flex().flex_none().items_center().gap_2()
                        .child(status_text(if ready { text(messages::READY) } else { text(messages::NOT_INSTALLED) }, ready))
                        .child(row_menu("ai-mcp-menu-builtin", {
                            let view = view.clone();
                            move |menu: PopupMenu| {
                                let view = view.clone();
                                menu.item(PopupMenuItem::new(text(messages::DETAILS)).on_click(
                                    move |_, _, cx| {
                                        view.update(cx, |this, cx| {
                                            this.active_dialog =
                                                Some(DialogState::AiMcpBuiltinDetails);
                                            cx.notify();
                                        });
                                    },
                                ))
                            }
                        }, cx)))));
            for (name, server) in &self.ai_extensions.mcp.servers {
                let toggle_name = name.clone();
                let edit_name = name.clone();
                let remove_name = name.clone();
                let menu_view = view.clone();
                let build_menu = move |menu: PopupMenu| {
                    let edit = edit_name.clone();
                    let remove = remove_name.clone();
                    let view = menu_view.clone();
                    menu.item(PopupMenuItem::new(text(messages::EDIT)).on_click(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            this.open_ai_mcp_form(Some(edit.clone()));
                            cx.notify();
                        });
                    }))
                    .item({
                        let view = menu_view.clone();
                        PopupMenuItem::new(text(messages::REMOVE)).on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                if !this.ai_extensions.action_busy {
                                    this.ai_extensions.error = None;
                                    this.active_dialog = Some(DialogState::AiMcpRemove {
                                        name: remove.clone(),
                                    });
                                    cx.notify();
                                }
                            });
                        })
                    })
                };
                content = content.child(
                    h_flex().w_full().items_center().justify_between().gap_3()
                        .min_h(px(72.0)).py(px(ui_theme::SPACE_3))
                        .border_t_1().border_color(rgb(ui_theme::BORDER_MUTED))
                        .child(h_flex().flex_1().min_w(px(0.0)).items_center().gap_3()
                            .child(list_row_icon(IconName::SquareTerminal))
                            .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                                .child(div().text_size(px(ui_theme::TYPE_BODY))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                                    .truncate().child(name.clone()))
                                .child(caption(format!("{} · {}", server.command,
                                    if server.auto_discover { text(messages::DISCOVER).into() }
                                    else { message(messages::ALLOWLIST, &[("count", &server.tools.len().to_string())]) }
                                )).truncate())))
                        .child(h_flex().flex_none().items_center().gap_2()
                            .child(status_text(if server.enabled { text(messages::CONFIGURED) } else { text(messages::DISABLED) }, server.enabled))
                            .child(self.toggle_switch(format!("ai-mcp-enabled-{name}"), server.enabled,
                                self.ai_extensions.action_busy, move |this, checked, _, _| {
                                    this.set_ai_mcp_server_enabled(toggle_name.clone(), checked);
                                }, cx))
                            .child(row_menu(
                                format!("ai-mcp-menu-{name}"),
                                build_menu, cx,
                            ))),
                );
            }
            if self.ai_extensions.mcp.servers.is_empty() {
                content = content.child(card()
                    .mt(px(ui_theme::SPACE_3))
                    .flex_row().items_center().gap_3()
                    .child(list_row_icon(IconName::Plug))
                    .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                        .child(heading(text(messages::NO_MCP)))
                        .child(caption(text(messages::MCP_SAVE_HINT)))));
            }
        }
        content = content.child(div().w_full().border_t_1()
            .border_color(rgb(ui_theme::BORDER_MUTED))
            .pt_3()
            .child(caption(text(messages::MCP_MORE_HINT)))
            .child(caption(text(messages::MCP_PERMISSIONS))));
        content
    }

    fn render_ai_runtime_content(&self, cx: &mut Context<Self>) -> Div {
        let view = cx.entity();
        let runtime = self.ai_extensions.runtime.as_ref();
        let node_status = runtime.and_then(|info| info.node_version.as_ref()
            .zip(info.node_source)).map(|(version, source)| format!("v{version} · {source}"))
            .unwrap_or_else(|| if runtime.is_some_and(|info| info.system_checked) {
                text(messages::NODE_MISSING).into()
            } else { text(messages::NODE_UNCHECKED).into() });
        let ready = runtime.is_some_and(|info| info.mcp_ready);
        let node_found = runtime.is_some_and(|info| info.node_version.is_some());
        let downloading = self.browser_runtime_downloading;
        let mut content = v_flex().w_full().min_w(px(0.0)).gap_4()
            .child(v_flex().gap_1()
                .child(heading(text(messages::BROWSER_RUNTIME)))
                .child(caption(text(messages::RUNTIME_DESCRIPTION))))
            .child(card()
                .child(h_flex().w_full().items_center().justify_between().gap_3()
                    .child(h_flex().flex_1().min_w(px(0.0)).items_center().gap_3()
                        .child(card_icon(IconName::Cpu))
                        .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
                            .child(heading(text(messages::NODE_RUNTIME)))
                            .child(caption(node_status))))
                    .child(status_text(if node_found { text(messages::READY) }
                        else if runtime.is_some_and(|info| info.system_checked) { text(messages::NOT_FOUND) }
                        else { text(messages::UNCHECKED) },
                        node_found)))
                // 来源行：设计稿把「来源」标签与取值、重新检测并排放在卡片底部。
                .child(h_flex().w_full().items_center().justify_between().gap_3()
                    .child(h_flex().flex_1().min_w(px(0.0)).items_center().gap_3()
                        .child(body(text(messages::SOURCE)))
                        .child(caption(
                            // 来源取值在 `browser_runtime` 里已是中文文案；
                            // 未检测到时按设计稿显示回退方向。
                            runtime.and_then(|info| info.node_source)
                                .map(|source| source.to_string())
                                .unwrap_or_else(|| text(messages::SOURCE_FALLBACK).to_string()),
                        )))
                    .child(settings_command_button("ai-node-recheck", text(messages::RECHECK), SettingsButtonTone::Secondary, !self.ai_extensions.loading && !downloading, cx)
                        .on_click({ let view = view.clone(); move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.inspect_ai_runtime();
                                cx.notify();
                            });
                        }})))
                .child(caption(text(messages::RUNTIME_MISSING_HINT)))
                .when_some(runtime.and_then(|info| info.node_path.as_ref()), |this, path| {
                    this.child(caption(path.display().to_string()))
                }))
            .child(card()
                .child(h_flex().w_full().justify_between().gap_3()
                    .child(heading(text(messages::BROWSER_COMPONENT)))
                    .child(status_text(if ready { text(messages::READY) } else { text(messages::NOT_INSTALLED) }, ready)))
                .child(caption(if ready { text(messages::RUNTIME_READY) } else {
                    text(messages::RUNTIME_INSTALL_HINT)
                }))
                .child(h_flex().w_full().justify_end()
                    .child(settings_command_button("ai-runtime-download",
                        if downloading { text(messages::DOWNLOADING) } else { text(messages::DOWNLOAD) },
                        SettingsButtonTone::Primary, !downloading && !ready, cx)
                        .tooltip(if ready { text(messages::COMPONENT_READY) } else if downloading { text(messages::COMPONENT_DOWNLOADING) } else { text(messages::COMPONENT_DOWNLOAD) })
                        .on_click({ let view = view.clone(); move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.start_ai_runtime_download();
                                cx.notify();
                            });
                        }}))));
        if let Some(notice) = &self.browser_runtime_notice {
            content = content.child(caption(resolve(notice)));
        }
        if let Some(error) = &self.ai_extensions.error {
            content = content.child(error_text(resolve(error)));
        }
        content.child(caption(text(messages::RETRY_HINT)))
    }
}
