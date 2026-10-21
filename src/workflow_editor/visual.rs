//! V2 文档控件。图标与弹出菜单复用 Kit；字段继续使用同一输入宿主。
#[path = "visual_resources.rs"]
mod deep_messages;

use super::*;
use super::document::{card, column, hint, text, edit_tool_selection};
use gpui_kit::assets::IconName;
use gpui_kit::base::ElementExt;
use gpui_kit::component::{Icon, Sizable, Disableable, button::{Button, ButtonVariants}, collapsible::Collapsible, menu::{DropdownMenu, PopupMenu, PopupMenuItem}};

type MenuWidth = std::rc::Rc<std::cell::Cell<Option<gpui::Pixels>>>;

pub(super) fn measure_menu_field(field: gpui::Div, width: MenuWidth) -> gpui::Div {
    field.on_prepaint(move |bounds, _, _| width.set(Some(bounds.size.width)))
}

pub(super) fn fit_menu(menu: PopupMenu, width: &MenuWidth, window: &Window) -> PopupMenu {
    // Kit 的 min_w / max_w 都作用于菜单内容盒；同时设置才能覆盖默认最大宽度。
    let available = (window.viewport_size().width - px(16.0)).max(px(1.0));
    let width = width.get().unwrap_or(px(200.0)).min(available).max(px(1.0));
    menu.min_w(width).max_w(width)
}

pub(super) fn icon(name: IconName, color: u32) -> Icon {
    Icon::new(name).with_size(px(16.0)).text_color(rgb(color))
}

pub(super) fn badge(label: impl Into<gpui::SharedString>) -> gpui::Div {
    div().flex().items_center().flex_none().px_2().py_1().rounded(px(ui_theme::RADIUS_XS))
        .bg(rgb(ui_theme::PRIMARY_SUBTLE)).text_color(rgb(ui_theme::PRIMARY))
        .text_size(px(12.0)).child(label.into())
}

pub(super) fn icon_button(id: impl Into<gpui::ElementId>, label: &str, name: IconName, enabled: bool) -> Button {
    Button::new(id).ghost().compact().icon(icon(name, ui_theme::CONTENT_SECONDARY))
        .tooltip(label.to_string()).accessibility_label(label.to_string()).disabled(!enabled)
        .w(px(28.0)).h(px(28.0)).min_w(px(0.0)).rounded(px(ui_theme::RADIUS_XS))
}

pub(super) fn menu_button(id: impl Into<gpui::ElementId>, label: impl Into<gpui::SharedString>, enabled: bool) -> Button {
    // 下拉需要原生 label 测量文字，宽度由外层字段容器分配，不能在弹层包装内 flex_1。
    Button::new(id).small().label(label).dropdown_caret(true).disabled(!enabled)
        .w_full().h(px(36.0)).min_w(px(0.0)).px_3()
        .bg(rgb(ui_theme::WB_INPUT_SURFACE)).text_color(rgb(ui_theme::CONTENT_PRIMARY))
        .border_1().border_color(rgb(ui_theme::BORDER_MUTED)).rounded(px(ui_theme::RADIUS_SM))
}

pub(super) fn step_icon(step: &WorkflowEditorStepData) -> (IconName, &'static str) {
    if step.kind == WorkflowStepKind::Invoke {
        match step.invoke.value(WorkflowStepSlot::Uses) {
            "skill.run" => (IconName::Sparkles, "Skill"),
            "js.run" => (IconName::Braces, "JavaScript"),
            "mcp.call" => (IconName::Plug, "MCP"),
            _ => (IconName::Workflow, khaslana::ui_i18n::text(deep_messages::ACTION)),
        }
    } else {
        (match step.kind {
            WorkflowStepKind::Merge => IconName::GitMerge,
            WorkflowStepKind::Fetch | WorkflowStepKind::Pull => IconName::Download,
            WorkflowStepKind::Push => IconName::Upload,
            WorkflowStepKind::DeleteBranches => IconName::Trash,
            WorkflowStepKind::EnsureClean | WorkflowStepKind::GuardRemoteBranch | WorkflowStepKind::AssertBranch => IconName::ShieldCheck,
            _ => IconName::GitBranch,
        }, "Git")
    }
}

fn status(label: &str, available: bool) -> gpui::Div {
    badge(label.to_string()).gap_1()
        .bg(rgb(if available { ui_theme::WORKFLOW_SUCCESS_BG } else { ui_theme::FEEDBACK_WARNING_BG }))
        .text_color(rgb(if available { ui_theme::WORKFLOW_SUCCESS_TEXT } else { ui_theme::FEEDBACK_WARNING_TEXT }))
        .child(icon(if available { IconName::Check } else { IconName::Info },
            if available { ui_theme::WORKFLOW_SUCCESS_TEXT } else { ui_theme::FEEDBACK_WARNING_TEXT }))
}

impl RepositoryView {
    pub(super) fn document_menu_width(&self, key: String) -> MenuWidth {
        // 打开菜单会重新渲染，测量值需跨帧保留，不能在 builder 内重建为零宽。
        self.workflow_editor.as_ref().expect("编辑器状态缺失").document_menu_widths
            .borrow_mut().entry(key).or_default().clone()
    }

    pub(super) fn document_input(&self, id: WorkflowEditorFieldId, _window: &Window, _cx: &mut Context<Self>) -> gpui::AnyElement {
        let field_id = FieldId::WorkflowEditor(id);
        let blocked = self.active_operation_blocker_message().is_some() && !self.operation_blocker_allows_text_field(field_id);
        let code = matches!(id, WorkflowEditorFieldId::StepParam { slot: WorkflowStepSlot::Script | WorkflowStepSlot::JsInput | WorkflowStepSlot::Arguments | WorkflowStepSlot::ToolArguments | WorkflowStepSlot::Tools, .. });
        let height = match id {
            WorkflowEditorFieldId::StepParam { slot: WorkflowStepSlot::Script, .. } => 200.0,
            WorkflowEditorFieldId::StepParam { slot: WorkflowStepSlot::Task, .. } => 88.0,
            WorkflowEditorFieldId::InputPart { part: WorkflowInputPart::Default, .. } => 96.0,
            _ => 128.0,
        };
        self.kit_field(field_id).map(|field| field.render_workflow_document(height, code, blocked))
            .unwrap_or_else(|| div().into_any_element())
    }

    pub(super) fn document_options(&self, index: usize, slot: WorkflowStepSlot, options: Vec<String>, width: MenuWidth, cx: &Context<Self>) -> gpui::AnyElement {
        let entity = cx.entity();
        let current = self.workflow_editor.as_ref().and_then(|editor| editor.data.steps.get(index))
            .map(|step| step.invoke.value(slot).to_string()).unwrap_or_default();
        div().w(px(28.0)).h(px(36.0)).flex_none().child(
            icon_button(format!("workflow-v2-options-{index}-{slot:?}"), if options.is_empty() { khaslana::ui_i18n::text(deep_messages::NO_OPTIONS_AVAILABLE_CONFIGURE_THEM_IN_SETTINGS_FIRST) } else { khaslana::ui_i18n::text(deep_messages::SELECT_A_CONFIGURED_ITEM) }, IconName::ChevronDown, !options.is_empty())
            .dropdown_menu_with_anchor(gpui::Anchor::TopRight, move |mut menu, window, _| {
                menu = fit_menu(menu, &width, window).scrollable(true).max_h(px(240.0));
                for value in &options {
                    let value = value.clone();
                    menu = menu.item(PopupMenuItem::new(value.clone()).checked(value == current).on_click(window.listener_for(&entity, move |this, _, _, cx| {
                        this.workflow_document_set_slot(index, slot, value.clone()); cx.notify();
                    })));
                }
                menu
            })).into_any_element()
    }

    pub(super) fn document_parameter(&self, index: usize, slot: WorkflowStepSlot, window: &Window, cx: &mut Context<Self>) -> gpui::Div {
        let editor = self.workflow_editor.as_ref().expect("编辑器状态缺失");
        let step = &editor.data.steps[index];
        let value = step.invoke.value(slot);
        let options = match slot {
            WorkflowStepSlot::Skill => self.ai_extensions.skills.iter().map(|(name, _)| name.clone()).collect(),
            WorkflowStepSlot::Server => std::iter::once("browser.edge".to_string()).chain(self.ai_extensions.mcp.servers.keys().cloned()).collect(),
            WorkflowStepSlot::Tool => self.document_server_tools(step.invoke.value(WorkflowStepSlot::Server)),
            _ => Vec::new(),
        };
        let selector = matches!(slot, WorkflowStepSlot::Skill | WorkflowStepSlot::Server | WorkflowStepSlot::Tool);
        let roomy = f32::from(window.viewport_size().width) >= 1380.0;
        let label = match slot { WorkflowStepSlot::Skill => khaslana::ui_i18n::text(deep_messages::SELECT_SKILL), WorkflowStepSlot::Task => khaslana::ui_i18n::text(deep_messages::TASK_DESCRIPTION), WorkflowStepSlot::SaveAs => khaslana::ui_i18n::text(deep_messages::OUTPUT_VARIABLE), _ => slot.label() };
        let width = self.document_menu_width(format!("options-{index}-{slot:?}"));
        let mut field = div().flex().min_w_0().flex_1().gap_1().items_center()
            .child(div().flex_1().min_w_0().when(slot == WorkflowStepSlot::Task, |input| input.w_full())
                .child(self.document_input(WorkflowEditorFieldId::StepParam { step: index, slot }, window, cx)))
            .when(selector, |row| row.child(self.document_options(index, slot, options, width.clone(), cx)));
        if roomy && (selector || slot == WorkflowStepSlot::SaveAs) {
            field = field.flex_none().w(px(if slot == WorkflowStepSlot::SaveAs { 400.0 } else { 308.0 }));
        }
        if selector { field = measure_menu_field(field, width); }
        if slot == WorkflowStepSlot::Task { field = field.flex_col().items_start().w_full().child(hint(khaslana::ui_i18n::text(deep_messages::DESCRIBE_THE_TASK_INPUT_VARIABLE_REFERENCES_ARE_SUPPORTED))); }
        let available = match slot {
            WorkflowStepSlot::Skill => self.ai_extensions.skills.iter().any(|(name, _)| name == value),
            WorkflowStepSlot::Server => value == "browser.edge" || self.ai_extensions.mcp.servers.contains_key(value),
            _ => false,
        };
        let mut row = div().flex().items_start().min_w_0().gap_3()
            .child(text(label.to_string()).w(px(108.0)).flex_none().pt(px(8.0))).child(field);
        if matches!(slot, WorkflowStepSlot::Skill | WorkflowStepSlot::Server) {
            row = row.child(status(if available { if slot == WorkflowStepSlot::Skill { khaslana::ui_i18n::text(deep_messages::INSTALLED) } else { khaslana::ui_i18n::text(deep_messages::CONFIGURED) } } else { khaslana::ui_i18n::text(deep_messages::NOT_FOUND) }, available));
        }
        let help = match slot {
            WorkflowStepSlot::Skill => khaslana::ui_i18n::text(deep_messages::RUN_A_TASK_USING_AN_INSTALLED_SKILL),
            WorkflowStepSlot::Server => khaslana::ui_i18n::text(deep_messages::CALL_TOOLS_THROUGH_A_CONFIGURED_SERVER),
            WorkflowStepSlot::SaveAs => khaslana::ui_i18n::text(deep_messages::SAVE_THE_RESULT_FOR_LATER_STEPS_TO_REFERENCE_AS),
            _ => "",
        };
        if !help.is_empty() && roomy { row = row.child(hint(help).flex_1().min_w_0().pt_2()); }
        else if slot == WorkflowStepSlot::SaveAs { return column().gap_1().child(row).child(hint(help).pl(px(120.0))); }
        row
    }

    fn document_server_tools(&self, name: &str) -> Vec<String> {
        if name == khaslana::workflow::browser_runtime::SERVER_ID {
            khaslana::workflow::extensions::builtin_browser_server().tools.keys().cloned().collect()
        } else { self.ai_extensions.mcp.servers.get(name).map(v2::configured_server_tools).unwrap_or_default() }
    }

    fn document_select_tool(&mut self, index: usize, server: &str, tool: &str, selected: bool) {
        let source = self.workflow_editor.as_ref().and_then(|editor| editor.data.steps.get(index))
            .map(|step| step.invoke.value(WorkflowStepSlot::Tools).to_string()).unwrap_or_default();
        match edit_tool_selection(&source, server, tool, selected) {
            Ok(value) => self.workflow_document_set_slot(index, WorkflowStepSlot::Tools, value),
            Err(error) => { if let Some(editor) = self.workflow_editor.as_mut() { editor.data.error = Some(error); } }
        }
    }

    pub(super) fn document_tools(&self, index: usize, cx: &mut Context<Self>) -> gpui::Div {
        let editor = self.workflow_editor.as_ref().expect("编辑器状态缺失");
        let step = &editor.data.steps[index];
        let selected: serde_json::Value = json5::from_str(step.invoke.value(WorkflowStepSlot::Tools)).unwrap_or_default();
        let selected = selected.as_array().cloned().unwrap_or_default();
        let key = step.invoke.value(WorkflowStepSlot::StepId).to_string();
        let expanded = editor.document_tools_open.get(&key).copied().unwrap_or(!selected.is_empty());
        let enabled = !editor.ai_loading;
        let label = if selected.is_empty() { khaslana::ui_i18n::text(deep_messages::CALLABLE_TOOLS_OPTIONAL).to_string() }
            else { khaslana::ui_i18n::format(deep_messages::CALLABLE_TOOLS_SELECTED, &[("value0", &(selected.len()).to_string())]) };
        let toggle = Button::new(format!("workflow-v2-tools-toggle-{key}")).ghost().small().label(label)
            .icon(icon(if expanded { IconName::ChevronDown } else { IconName::ChevronRight }, ui_theme::WORKFLOW_META))
            .disabled(!enabled).accessibility_label(if expanded { khaslana::ui_i18n::text(deep_messages::COLLAPSE_CALLABLE_TOOLS) } else { khaslana::ui_i18n::text(deep_messages::EXPAND_CALLABLE_TOOLS_OPTIONAL) })
            .on_click(cx.listener(move |this, _, _, cx| {
                if let Some(editor) = this.workflow_editor.as_mut() { editor.document_tools_open.insert(key.clone(), !expanded); }
                cx.notify();
            }));
        let servers = std::iter::once("browser.edge".to_string())
            .chain(self.ai_extensions.mcp.servers.keys().filter(|name| name.as_str() != "browser.edge").cloned())
            .map(|name| { let tools = self.document_server_tools(&name); (name, tools) })
            .filter(|(_, tools)| !tools.is_empty()).collect::<Vec<_>>();
        let mut chips = div().flex().flex_wrap().items_center().min_w_0().flex_1().gap_1();
        for (position, item) in selected.iter().enumerate() {
            let Some(tool) = item["tool"].as_str() else { continue; };
            let Some(server) = item["server"].as_str() else { continue; };
            let tool = tool.to_string(); let server = server.to_string();
            chips = chips.child(badge(format!("{server} / {tool}")).py(px(2.0)).gap_1()
                .child(icon_button(format!("workflow-v2-tool-remove-{index}-{position}"), &khaslana::ui_i18n::format(deep_messages::REMOVE, &[("server", &(server).to_string()), ("tool", &(tool).to_string())]), IconName::X, enabled)
                    .w(px(18.0)).h(px(18.0)).on_click(cx.listener(move |this, _, _, cx| { this.document_select_tool(index, &server, &tool, false); cx.notify(); }))));
        }
        if selected.is_empty() { chips = chips.child(hint(khaslana::ui_i18n::text(deep_messages::NO_TOOLS_ADDED_THIS_STEP_CAN_RUN_DIRECTLY))); }
        let entity = cx.entity();
        let tool_width = self.document_menu_width(format!("tools-{index}"));
        let tool_measure = tool_width.clone();
        let menu_button = icon_button(format!("workflow-v2-tool-picker-{index}"), if servers.is_empty() { khaslana::ui_i18n::text(deep_messages::NO_TOOLS_AVAILABLE_CONFIGURE_THEM_IN_AI_SETTINGS_FIRST) } else { khaslana::ui_i18n::text(deep_messages::ADD_A_CALLABLE_TOOL) }, IconName::Plus, enabled && !servers.is_empty())
            .w(px(36.0)).h(px(36.0))
            .dropdown_menu_with_anchor(gpui::Anchor::TopRight, move |mut menu, window, _| {
                menu = fit_menu(menu, &tool_width, window).scrollable(true).max_h(px(260.0));
                for (server, tools) in &servers {
                    menu = menu.label(server.clone());
                    for tool in tools {
                        let checked = selected.iter().any(|item| item["server"] == *server && item["tool"] == *tool);
                        let tool = tool.clone(); let server = server.clone();
                        menu = menu.item(PopupMenuItem::new(tool.clone()).checked(checked).on_click(window.listener_for(&entity, move |this, _, _, cx| {
                            this.document_select_tool(index, &server, &tool, !checked); cx.notify();
                        })));
                    }
                } menu
            });
        let content = column().gap_2()
            .child(hint(khaslana::ui_i18n::text(deep_messages::ADD_TOOLS_ONLY_WHEN_THE_STEP_NEEDS_EXTERNAL_CAPABILITIES_AI_INVOKES_THEM)))
            .child(div().flex().items_start().gap_3().child(text(khaslana::ui_i18n::text(deep_messages::ALLOWED_CALLS)).w(px(108.0)).pt_2().flex_none())
                .child(measure_menu_field(div().relative().flex().flex_1().min_w_0().items_center().gap_1()
                    .bg(rgb(ui_theme::WB_INPUT_SURFACE)).rounded(px(ui_theme::RADIUS_SM))
                    // 边框独立绘制，箭头触发器的右边界与整个字段相同。
                    .child(div().absolute().inset_0().border_1().border_color(rgb(ui_theme::BORDER_MUTED)).rounded(px(ui_theme::RADIUS_SM)))
                    .child(chips.pl_1().py_1()).child(div().w(px(36.0)).h(px(36.0)).flex_none().child(menu_button)), tool_measure)));
        column().child(Collapsible::new().open(expanded).gap_2().child(toggle).content(content))
    }

    pub(super) fn document_git_parameters(&self, index: usize, window: &Window, cx: &mut Context<Self>) -> gpui::Div {
        let step = &self.workflow_editor.as_ref().expect("编辑器状态缺失").data.steps[index];
        let mut body = column();
        for slot in step.kind.slots() { body = body.child(self.document_field(WorkflowEditorFieldId::StepParam { step: index, slot: *slot }, slot.label(), window, cx)); }
        let flags = match step.kind {
            WorkflowStepKind::CreateBranch => vec![(WorkflowStepFlag::CreateCheckout, khaslana::ui_i18n::text(deep_messages::SWITCH_TO_THE_NEW_BRANCH_AFTER_CREATION), step.checkout)],
            WorkflowStepKind::Push => vec![(WorkflowStepFlag::PushSetUpstream, khaslana::ui_i18n::text(deep_messages::SET_UPSTREAM_TRACKING_WHEN_PUSHING), step.set_upstream)],
            WorkflowStepKind::GuardRemoteBranch => vec![(WorkflowStepFlag::GuardFetch, khaslana::ui_i18n::text(deep_messages::FETCH_REMOTE_REFERENCES_BEFORE_CHECKING), step.guard_fetch)],
            WorkflowStepKind::FilterBranches => vec![(WorkflowStepFlag::FilterSkipCurrent, khaslana::ui_i18n::text(deep_messages::EXCLUDE_CURRENT_BRANCH), step.filter_skip_current)],
            WorkflowStepKind::DeleteBranches => vec![(WorkflowStepFlag::DeleteDryRun, khaslana::ui_i18n::text(deep_messages::DRY_RUN_LIST_BRANCHES_WITHOUT_DELETING), step.delete_dry_run), (WorkflowStepFlag::DeleteSkipCurrent, khaslana::ui_i18n::text(deep_messages::SKIP_CURRENT_BRANCH), step.delete_skip_current)],
            _ => Vec::new(),
        };
        for (flag, label, current) in flags {
            body = body.child(div().flex().items_center().gap_3().child(div().w(px(108.0)).flex_none())
                .child(self.toggle_switch(format!("workflow-v2-flag-{index}-{flag:?}"), current, false, move |this, next, _, _| {
                    if let Some(step) = this.workflow_editor.as_mut().and_then(|editor| editor.data.steps.get_mut(index)) {
                        match flag { WorkflowStepFlag::CreateCheckout => step.checkout = next, WorkflowStepFlag::PushSetUpstream => step.set_upstream = next,
                            WorkflowStepFlag::GuardFetch => step.guard_fetch = next, WorkflowStepFlag::FilterSkipCurrent => step.filter_skip_current = next,
                            WorkflowStepFlag::DeleteDryRun => step.delete_dry_run = next, WorkflowStepFlag::DeleteSkipCurrent => step.delete_skip_current = next }
                    }
                }, cx)).child(hint(label)));
        }
        if step.kind == WorkflowStepKind::GuardRemoteBranch {
            for (exists, current) in [(true, step.on_exists), (false, step.on_missing)] {
                let entity = cx.entity();
                let width = self.document_menu_width(format!("guard-{index}-{exists}"));
                let measured = width.clone();
                body = body.child(div().flex().items_center().gap_3().child(text(if exists { khaslana::ui_i18n::text(deep_messages::REMOTE_BRANCH_EXISTS) } else { khaslana::ui_i18n::text(deep_messages::REMOTE_BRANCH_IS_MISSING) }).w(px(108.0)))
                    .child(measure_menu_field(div().flex_1().min_w_0().h(px(36.0)).child(
                        menu_button(format!("workflow-v2-guard-{index}-{exists}"), if current == RemoteBranchGuardAction::Fail { khaslana::ui_i18n::text(deep_messages::STOP_WORKFLOW) } else { khaslana::ui_i18n::text(deep_messages::CONTINUE) }, true)
                        .dropdown_menu(move |mut menu, window, _| {
                            menu = fit_menu(menu, &width, window);
                            for (value, label) in [(RemoteBranchGuardAction::Fail, khaslana::ui_i18n::text(deep_messages::STOP_WORKFLOW)), (RemoteBranchGuardAction::Continue, khaslana::ui_i18n::text(deep_messages::CONTINUE))] {
                                menu = menu.item(PopupMenuItem::new(label).checked(value == current).on_click(window.listener_for(&entity, move |this, _, _, cx| {
                                    this.workflow_editor_set_guard_action(index, exists.then_some(value), (!exists).then_some(value)); cx.notify();
                                })));
                            } menu
                        })), measured)));
            }
        }
        if step.kind == WorkflowStepKind::EnsureClean { body = body.child(hint(step.kind.description())); }
        body
    }

    pub(super) fn document_variable_card(&self, id: String, name: String, description: String, default: Option<String>, cx: &Context<Self>) -> gpui::Div {
        let copied = name.clone();
        card().border_0().gap_1().bg(rgb(ui_theme::WORKFLOW_SURFACE))
            .child(div().flex().items_center().gap_1().child(text(name.clone()).font_family("Consolas").text_size(px(13.0))
                .font_weight(gpui::FontWeight::SEMIBOLD).flex_1().min_w_0())
                .child(icon_button(format!("workflow-v2-variable-copy-{id}"), &khaslana::ui_i18n::format(deep_messages::COPY, &[("name", &(name).to_string())]), IconName::Copy, true)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(copied.clone()));
                        this.status = khaslana::ui_i18n::encode(deep_messages::COPIED_VARIABLE_REFERENCE, &[("copied", &(copied).to_string())]);
                        this.notify_success(this.status.clone(), cx);
                    }))))
            .child(badge(if default.is_some() { khaslana::ui_i18n::text(deep_messages::STRING) } else { khaslana::ui_i18n::text(deep_messages::VARIABLE) }))
            .when(!description.is_empty(), |card| card.child(hint(description)))
            .when_some(default, |card, value| card.child(hint(khaslana::ui_i18n::format(deep_messages::DEFAULT_VALUE, &[("value", &(value).to_string())]))))
    }
}
