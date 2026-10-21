//! V2 文档式工作流视图。沿用模态生命周期与字段路由，在页面内承接焦点隔离。
#[path = "document_resources.rs"]
mod deep_messages;

use super::*;
use gpui_kit::base::FocusTrapElement;
use crate::ui::components::{floating_panel, settings_command_button, SettingsButtonTone};
use super::visual::{badge, icon, icon_button, menu_button, step_icon, measure_menu_field, fit_menu};
use gpui_kit::assets::IconName;
use gpui_kit::component::{button::{Button, ButtonVariants}, menu::{DropdownMenu, PopupMenuItem}};

pub(super) fn column() -> gpui::Div {
    div().flex().flex_col().min_w_0().gap_3()
}

pub(super) fn card() -> gpui::Div {
    column().p_4().border_1().border_color(rgb(ui_theme::BORDER_MUTED))
        .rounded(px(ui_theme::RADIUS_MD)).bg(rgb(ui_theme::WB_PANEL))
}

pub(super) fn text(value: impl Into<gpui::SharedString>) -> gpui::Div {
    div().text_size(px(14.0)).text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(value.into())
}

pub(super) fn hint(value: impl Into<gpui::SharedString>) -> gpui::Div {
    div().text_size(px(13.0)).line_height(px(20.0)).text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(value.into())
}

pub(super) fn variable_rail_visible(width: f32) -> bool { width >= 1100.0 }

pub(super) fn filter_document_actions(query: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    let query = query.trim().to_lowercase();
    [("skill.run", "AI Skill", khaslana::ui_i18n::text(deep_messages::RUN_A_TASK_USING_AN_INSTALLED_SKILL)),
        ("js.run", "JavaScript", khaslana::ui_i18n::text(deep_messages::TRANSFORM_INPUTS_COMBINE_VARIABLES_AND_CALL_TOOLS_AS_NEEDED))]
        .into_iter().filter(|(uses, label, description)| query.is_empty()
            || format!("{uses} {label} {description}").to_lowercase().contains(&query)).collect()
}

pub(super) fn next_step_id(steps: &[WorkflowEditorStepData]) -> String {
    (1..).map(|index| format!("step-{index}"))
        .find(|id| steps.iter().all(|step| step.invoke.value(WorkflowStepSlot::StepId) != id))
        .expect("步骤数有限")
}

pub(super) fn v2_copy_data(data: &WorkflowEditorData) -> WorkflowEditorData {
    let mut copy = data.clone();
    copy.version = 2;
    copy.editing_path = None;
    copy.file_name = format!("{}-v2", data.file_name.trim());
    copy.error = None;
    copy
}

pub(super) fn duplicate_step_data(steps: &[WorkflowEditorStepData], index: usize) -> Option<WorkflowEditorStepData> {
    let mut step = steps.get(index)?.clone();
    if step.kind == WorkflowStepKind::Invoke {
        step.invoke.set(WorkflowStepSlot::StepId, next_step_id(steps));
        step.invoke.set(WorkflowStepSlot::SaveAs, String::new());
    } else if step.kind == WorkflowStepKind::FilterBranches {
        step.output.clear();
    }
    Some(step)
}

pub(super) fn edit_tool_selection(source: &str, server: &str, tool: &str, selected: bool) -> Result<String, String> {
    let mut value: serde_json::Value = if source.trim().is_empty() { serde_json::json!([]) }
        else { json5::from_str(source).map_err(|error| khaslana::ui_i18n::encode(deep_messages::INVALID_TOOL_LIST, &[("error", &(error).to_string())]))? };
    let array = value.as_array_mut().ok_or_else(|| khaslana::ui_i18n::encode(deep_messages::ALLOWED_TOOLS_MUST_BE_AN_ARRAY, &[]))?;
    array.retain(|item| !(item.get("server").and_then(serde_json::Value::as_str) == Some(server)
        && item.get("tool").and_then(serde_json::Value::as_str) == Some(tool)));
    if selected { array.push(serde_json::json!({"server":server,"tool":tool})); }
    serde_json::to_string_pretty(&value).map_err(|error| error.to_string())
}

impl RepositoryView {
    pub(crate) fn submit_workflow_document_field(&mut self, field: FieldId, cx: &mut Context<Self>) -> bool {
        let FieldId::WorkflowEditor(id) = field else { return false; };
        if !self.workflow_document_editor_visible() { return false; }
        if self.workflow_editor.as_ref().is_some_and(|editor| editor.ai_loading) { return true; }
        match id {
            WorkflowEditorFieldId::AiDescription => self.generate_workflow_template_with_ai(cx),
            WorkflowEditorFieldId::PickerSearch => {
                let query = self.workflow_editor.as_ref().map(|editor| editor.picker_search_field.value.to_lowercase()).unwrap_or_default();
                if let Some((uses, _, _)) = filter_document_actions(&query).first() {
                    self.workflow_document_add_action(uses);
                } else if let Some(kind) = filter_workflow_step_kinds(&query).first() {
                    self.workflow_editor_add_step_of_kind(*kind);
                }
            }
            _ => self.save_workflow_editor(cx),
        }
        cx.notify();
        true
    }

    pub(crate) fn workflow_editor_copy_as_v2(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = self.workflow_editor.as_mut() else { return; };
        if editor.ai_loading { return; }
        editor.sync_from_fields();
        let data = v2_copy_data(&editor.data);
        let editor = WorkflowEditorState::from_data(data, cx);
        self.show_workflow_editor(editor, cx);
    }

    pub(crate) fn workflow_document_editor_visible(&self) -> bool {
        self.workflow_editor.as_ref().is_some_and(|editor| editor.data.version == 2)
            && self.main_mode == crate::MainMode::Workflow
    }

    pub(crate) fn workflow_document_picker_focus(&self) -> Option<gpui::FocusHandle> {
        self.workflow_editor.as_ref().filter(|editor| self.workflow_document_editor_visible() && editor.step_picker_open)
            .map(|editor| editor.document_picker_focus.clone())
    }

    pub(crate) fn workflow_document_picker_search_focus(&self) -> Option<gpui::FocusHandle> {
        self.workflow_editor.as_ref().filter(|editor| self.workflow_document_editor_visible() && editor.step_picker_open)
            .map(|editor| editor.picker_search_field.focus.clone())
    }

    pub(crate) fn workflow_document_picker_blocks_mouse(&self, position: gpui::Point<gpui::Pixels>) -> bool {
        self.workflow_document_picker_focus().is_some() && self.active_dialog.is_none()
            && self.top_overlay_kind() == crate::TopOverlayKind::Dialog
            && self.workflow_editor.as_ref().is_some_and(|editor| editor.document_bounds.get().is_none_or(|bounds| !bounds.contains(&position)))
    }

    fn workflow_document_add_action(&mut self, uses: &'static str) {
        if let Some(editor) = self.workflow_editor.as_mut() {
            editor.sync_from_fields();
            let mut step = WorkflowEditorStepData::new(WorkflowStepKind::Invoke);
            step.invoke = v2::InvokeEditorData::new_action(uses, &next_step_id(&editor.data.steps));
            editor.data.steps.push(step);
            editor.data.selected_step = editor.data.steps.len() - 1;
            editor.step_picker_open = false;
            editor.ensure_field_capacity();
        }
    }

    fn workflow_document_duplicate(&mut self, index: usize) {
        if let Some(editor) = self.workflow_editor.as_mut() {
            editor.sync_from_fields();
            // 副本不能复用输出名，否则后续步骤可能读到不同动作的结果。
            let Some(step) = duplicate_step_data(&editor.data.steps, index) else { return; };
            editor.data.steps.insert(index + 1, step);
            editor.step_fields.insert(index + 1, WorkflowEditorStepState { fields: Default::default() });
            editor.data.selected_step = index + 1;
            editor.document_advanced_step = None;
        }
    }

    pub(super) fn workflow_document_set_slot(&mut self, index: usize, slot: WorkflowStepSlot, value: String) {
        if let Some(editor) = self.workflow_editor.as_mut() {
            if let Some(step) = editor.data.steps.get_mut(index) { step.set_slot_value(slot, value.clone()); }
            if let Some(field) = editor.step_fields.get_mut(index).and_then(|step| step.fields.get_mut(&slot)) {
                field.set_value(value);
            }
        }
    }

    fn workflow_document_check(&mut self) {
        let Some(editor) = self.workflow_editor.as_mut() else { return; };
        editor.sync_from_fields();
        let result = build_workflow_definition(&editor.data).and_then(|definition| {
            let serialized = json5::to_string(&definition).map_err(|error| error.to_string())?;
            parse_workflow_json5(&serialized).map_err(|error| error.to_string())?;
            serde_json::to_string_pretty(&definition).map_err(|error| error.to_string())
        });
        editor.data.error = result.as_ref().err().cloned();
        editor.document_preview_open = result.is_ok();
    }

    pub(super) fn document_command(&self, id: impl Into<gpui::ElementId>, label: impl Into<gpui::SharedString>,
        primary: bool, enabled: bool, cx: &Context<Self>) -> gpui_kit::component::button::Button {
        settings_command_button(id, label, if primary { SettingsButtonTone::Primary } else { SettingsButtonTone::Secondary }, enabled, cx)
            .min_w(px(0.0))
    }

    pub(super) fn document_field(&self, id: WorkflowEditorFieldId, label: &str, window: &Window,
        cx: &mut Context<Self>) -> gpui::Div {
        let multiline = self.is_field_multiline(FieldId::WorkflowEditor(id));
        div().flex().items_start().min_w_0().gap_3()
            .child(text(label.to_string()).w(px(108.0)).flex_none().pt(px(8.0)))
            .child(div().flex_1().min_w_0().child(self.document_input(id, window, cx)))
            .when(multiline, |row| row.items_start())
    }

    pub(crate) fn render_workflow_document_editor(&self, window: &Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let Some(editor) = self.workflow_editor.as_ref() else { return div().into_any_element(); };
        let wide = variable_rail_visible(f32::from(window.viewport_size().width));
        let branch = self.snapshot.as_ref().and_then(|snapshot| snapshot.branches.iter().find(|branch| branch.is_head))
            .map(|branch| branch.name.clone()).unwrap_or_else(|| khaslana::ui_i18n::text(deep_messages::NO_REPOSITORY_OPEN).into());
        let scroll_handle = self.scroll_handle("workflow-v2-document");
        let mut document = column().flex_1().min_w_0();
        if editor.document_metadata_open {
            document = document.child(card().child(text(khaslana::ui_i18n::text(deep_messages::TEMPLATE_SETTINGS)))
                .child(column()
                .child(self.document_field(WorkflowEditorFieldId::Name, khaslana::ui_i18n::text(deep_messages::TEMPLATE_NAME), window, cx).flex_1())
                .child(self.document_field(WorkflowEditorFieldId::FileName, khaslana::ui_i18n::text(deep_messages::FILE_NAME), window, cx)))
                .child(self.toggle_row("workflow-v2-clean", khaslana::ui_i18n::text(deep_messages::REQUIRE_A_CLEAN_WORKING_TREE_BEFORE_RUNNING), editor.data.require_clean_worktree,
                    |this, _, _| { if let Some(editor) = this.workflow_editor.as_mut() { editor.data.require_clean_worktree = !editor.data.require_clean_worktree; } }, cx)));
        }
        if editor.document_ai_open {
            document = document.child(card().child(text(khaslana::ui_i18n::text(deep_messages::DESCRIBE_YOUR_WORKFLOW)))
                .child(self.document_input(WorkflowEditorFieldId::AiDescription, window, cx))
                .child(hint(khaslana::ui_i18n::text(deep_messages::REVIEW_EACH_AI_GENERATED_ITEM_BEFORE_SAVING_THE_TEMPLATE)))
                .child(self.document_command("workflow-v2-ai-generate", if editor.ai_loading { khaslana::ui_i18n::text(deep_messages::GENERATING) } else { khaslana::ui_i18n::text(deep_messages::GENERATE_TEMPLATE) }, true, !editor.ai_loading && self.ai_settings.is_usable(), cx)
                    .tooltip(if self.ai_settings.is_usable() { khaslana::ui_i18n::text(deep_messages::GENERATE_OR_UPDATE_STEPS_BASED_ON_THE_CURRENT_DRAFT) } else { khaslana::ui_i18n::text(deep_messages::CONFIGURE_AND_ENABLE_AI_IN_SETTINGS_FIRST) })
                    .on_click(cx.listener(|this, _, _, cx| this.generate_workflow_template_with_ai(cx)))));
        }
        let mut inputs = card().child(div().flex().flex_wrap().items_center().justify_between().gap_2()
            .child(div().flex().flex_wrap().items_center().gap_4().child(text(khaslana::ui_i18n::text(deep_messages::WORKFLOW_INPUTS)).font_weight(gpui::FontWeight::SEMIBOLD))
                .child(hint(khaslana::ui_i18n::text(deep_messages::PROVIDE_THESE_VALUES_AT_RUN_TIME_FOR_LATER_STEPS_TO_REFERENCE))))
            .child(self.document_command("workflow-v2-input-add", khaslana::ui_i18n::text(deep_messages::ADD_INPUT), false, true, cx)
                .icon(icon(IconName::Plus, ui_theme::CONTENT_SECONDARY))
                .on_click(cx.listener(|this, _, _, cx| { this.workflow_editor_add_input_row(); cx.notify(); }))));
        if editor.data.inputs.is_empty() { inputs = inputs.child(hint(khaslana::ui_i18n::text(deep_messages::ADD_PARAMETERS_TO_FILL_IN_BEFORE_RUNNING_REFERENCE_THEM_AS_IN_STEPS))); }
        for (index, row) in editor.data.inputs.iter().enumerate() {
            inputs = inputs.child(column().gap_3().pt_3()
                .when(index > 0, |group| group.border_t_1().border_color(rgb(ui_theme::BORDER_MUTED)))
                .child(div().flex().flex_wrap().items_start().min_w_0().gap_3()
                    .child(column().flex_1().min_w(px(160.0)).gap_1().child(hint(khaslana::ui_i18n::text(deep_messages::VARIABLE_NAME)))
                        .child(self.document_input(WorkflowEditorFieldId::InputPart { index, part: WorkflowInputPart::Key }, window, cx)))
                    .child(column().w(px(140.0)).flex_none().gap_1().child(hint(khaslana::ui_i18n::text(deep_messages::INPUT_TYPE)))
                        .child(self.render_workflow_editor_input_type(index)))
                    .child(div().flex().items_center().gap_2().h(px(36.0)).mt(px(24.0)).flex_none()
                        .child(self.toggle_switch(format!("workflow-v2-input-required-{index}"), row.required, false,
                            move |this, next, _, _| { if let Some(row) = this.workflow_editor.as_mut().and_then(|editor| editor.data.inputs.get_mut(index)) { row.required = next; } }, cx))
                        .child(hint(khaslana::ui_i18n::text(deep_messages::REQUIRED))))
                    .child(div().mt(px(28.0)).flex_none().child(icon_button(format!("workflow-v2-input-remove-{index}"), khaslana::ui_i18n::text(deep_messages::DELETE_INPUT), IconName::Trash, true)
                        .icon(icon(IconName::Trash, ui_theme::DESTRUCTIVE))
                        .on_click(cx.listener(move |this, _, _, cx| { this.workflow_editor_remove_input_row(index); cx.notify(); })))))
                .child(column().gap_1().child(hint(khaslana::ui_i18n::text(deep_messages::DEFAULT_VALUE_OPTIONAL)))
                    .child(self.document_input(WorkflowEditorFieldId::InputPart { index, part: WorkflowInputPart::Default }, window, cx))
                    .when(row.kind == WorkflowInputKind::Select, |field| field.child(hint(khaslana::ui_i18n::text(deep_messages::ENTER_THE_ACTUAL_OPTION_VALUE_LEAVE_BLANK_TO_KEEP_IT_UNSELECTED_BEFORE))))
                    .when(row.kind == WorkflowInputKind::Date, |field| field.child(hint(khaslana::ui_i18n::text(deep_messages::USE_YYYY_MM_DD_OR)))))
                .child(self.render_workflow_editor_options(index, window, cx))
                .when(!editor.document_details_open && !row.description.is_empty(), |group| group.child(hint(row.description.clone())))
                .when(editor.document_details_open, |group| group
                    .child(self.document_field(WorkflowEditorFieldId::InputPart { index, part: WorkflowInputPart::Label }, khaslana::ui_i18n::text(deep_messages::RUN_TIME_DISPLAY_NAME), window, cx))
                    .child(self.document_field(WorkflowEditorFieldId::InputPart { index, part: WorkflowInputPart::Description }, khaslana::ui_i18n::text(deep_messages::DESCRIPTION), window, cx))));
        }
        document = document.child(inputs)
            .child(self.document_command("workflow-v2-input-details", if editor.document_details_open { khaslana::ui_i18n::text(deep_messages::COLLAPSE_VARIABLE_SETTINGS) } else { khaslana::ui_i18n::text(deep_messages::INPUT_DESCRIPTIONS_AND_CUSTOM_VARIABLES) }, false, true, cx)
                .on_click(cx.listener(|this, _, _, cx| { if let Some(editor) = this.workflow_editor.as_mut() { editor.document_details_open = !editor.document_details_open; } cx.notify(); })));
        if editor.document_details_open {
            let mut vars = card().child(text(khaslana::ui_i18n::text(deep_messages::CUSTOM_VARIABLES)));
            for index in 0..editor.data.vars.len() { vars = vars.child(self.render_workflow_var_card(index, window, cx)); }
            document = document.child(vars.child(self.document_command("workflow-v2-var-add", khaslana::ui_i18n::text(deep_messages::ADD_VARIABLE), false, true, cx)
                .on_click(cx.listener(|this, _, _, cx| { this.workflow_editor_add_var_row(); cx.notify(); }))));
        }
        {
            for index in 0..editor.data.steps.len() { document = document.child(self.render_document_step(index, window, cx)); }
            if editor.data.steps.is_empty() {
                document = document.child(card().child(text(khaslana::ui_i18n::text(deep_messages::START_WITH_A_STEP)))
                    .child(hint(khaslana::ui_i18n::text(deep_messages::COMBINE_GIT_ACTIONS_SKILLS_AND_JAVASCRIPT_IN_ORDER_STEPS_CAN_CALL_TOOLS)))
                    .children(WORKFLOW_EDITOR_PRESETS.iter().map(|preset| {
                        let preset = *preset;
                        self.document_command(format!("workflow-v2-preset-{}", preset.title()), preset.title(), false, true, cx)
                            .on_click(cx.listener(move |this, _, _, cx| { this.apply_workflow_editor_preset(preset, cx); cx.notify(); }))
                    }).collect::<Vec<_>>()));
            }
            document = document.child(self.document_command("workflow-v2-step-add", khaslana::ui_i18n::text(deep_messages::ADD_STEP), false, true, cx)
                .icon(icon(IconName::Plus, ui_theme::PRIMARY)).w_full().h(px(40.0))
                .text_color(rgb(ui_theme::PRIMARY)).border_color(rgb(ui_theme::WORKFLOW_OUTLINE)).border_dashed()
                .on_click(cx.listener(|this, _, _, cx| { this.workflow_editor_open_step_picker(); cx.notify(); })));
        }
        document = document.child(div().flex().items_center().gap_2().child(icon(IconName::Info, ui_theme::WORKFLOW_META))
            .child(hint(khaslana::ui_i18n::text(deep_messages::INPUTS_AND_TOOL_SCOPE_ARE_CHECKED_BEFORE_RUNNING_CUSTOM_MCP_REQUIRES_AUTHORIZATION))));
        if editor.document_preview_open {
            if let Ok(definition) = build_workflow_definition(&editor.data) {
                document = document.child(card().child(text(khaslana::ui_i18n::text(deep_messages::TEMPLATE_VALIDATION_PASSED)))
                    .child(hint(khaslana::ui_i18n::text(deep_messages::THE_SAVED_CONTENT_IS_SHOWN_BELOW_AN_EXECUTION_PREVIEW_IS_GENERATED_ON)))
                    .child(text(serde_json::to_string_pretty(&definition).unwrap_or_default())
                        .font_family("Consolas").text_size(px(13.0)).line_height(px(20.0))));
            }
        }
        let mut columns = div().flex().min_w_0().gap_4().child(document);
        if wide { columns = columns.child(self.render_document_variables(cx).w(px(250.0)).flex_none()); }
        let document_bounds = editor.document_bounds.clone();
        floating_panel().id("workflow-v2-editor").relative().flex().flex_col().flex_1().min_w_0().min_h(px(0.0))
            .child(gpui::canvas(|_, _, _| (), move |bounds, _, _, _| document_bounds.set(Some(bounds)))
                .absolute().top_0().left_0().right_0().bottom_0())
            .child(column().flex_none().p_4().gap_2()
                .child(hint(khaslana::ui_i18n::text(deep_messages::WORKFLOW_EDIT_WORKFLOW)))
                .child(div().flex().flex_wrap().items_center().gap_2()
                    .child(div().flex().items_center().gap_3().flex_1().min_w_0()
                        .child(text(if editor.name_field.value.trim().is_empty() { khaslana::ui_i18n::text(deep_messages::NEW_WORKFLOW).to_string() } else { editor.name_field.value.clone() })
                            .text_size(px(if wide { 26.0 } else { ui_theme::TYPE_PAGE_TITLE })).font_weight(gpui::FontWeight::BOLD).truncate()).child(badge("V2")))
                    .child(div().flex().items_center().gap_1().p_1().rounded(px(ui_theme::RADIUS_SM)).bg(rgb(ui_theme::WORKFLOW_SURFACE))
                        .child(settings_command_button("workflow-v2-edit-mode", khaslana::ui_i18n::text(deep_messages::EDIT), SettingsButtonTone::SelectedTab, true, cx)
                            .min_w(px(0.0)).icon(icon(IconName::Pencil, ui_theme::PRIMARY)).tooltip(khaslana::ui_i18n::text(deep_messages::CURRENTLY_EDITING_THE_WORKFLOW)))
                        .child(settings_command_button("workflow-v2-run-mode", khaslana::ui_i18n::text(deep_messages::RUN), SettingsButtonTone::Quiet, !editor.ai_loading, cx)
                            .min_w(px(0.0)).icon(icon(IconName::Play, ui_theme::CONTENT_SECONDARY))
                            .tooltip(khaslana::ui_i18n::text(deep_messages::SAVE_THE_DRAFT_AND_OPEN_THE_RUN_PAGE_CUSTOM_MCP_REQUIRES_AUTHORIZATION))
                            .on_click(cx.listener(|this, _, _, cx| this.save_workflow_editor(cx)))))
                    .child(self.document_command("workflow-v2-save", khaslana::ui_i18n::text(deep_messages::SAVE_WORKFLOW), true, !editor.ai_loading, cx).icon(icon(IconName::Save, ui_theme::PRIMARY_FOREGROUND))
                        .on_click(cx.listener(|this, _, _, cx| this.save_workflow_editor(cx))))
                    .child(self.document_command("workflow-v2-check", khaslana::ui_i18n::text(deep_messages::VALIDATE_TEMPLATE), false, !editor.ai_loading, cx).icon(icon(IconName::FileCheck, ui_theme::CONTENT_PRIMARY))
                        .on_click(cx.listener(|this, _, _, cx| { this.workflow_document_check(); cx.notify(); })))
                    .child(self.document_command("workflow-v2-ai", khaslana::ui_i18n::text(deep_messages::GENERATE_WITH_AI), false, true, cx).icon(icon(IconName::Sparkles, ui_theme::PRIMARY)).on_click(cx.listener(|this, _, _, cx| {
                        if let Some(editor) = this.workflow_editor.as_mut() { editor.document_ai_open = !editor.document_ai_open; } cx.notify(); })))
                    .child(icon_button("workflow-v2-settings", khaslana::ui_i18n::text(deep_messages::TEMPLATE_NAME_AND_RUN_SETTINGS), IconName::Settings2, !editor.ai_loading).on_click(cx.listener(|this, _, _, cx| {
                        if let Some(editor) = this.workflow_editor.as_mut() { editor.document_metadata_open = !editor.document_metadata_open; } cx.notify(); })))
                    .child(icon_button("workflow-v2-close", khaslana::ui_i18n::text(deep_messages::CLOSE_EDITOR), IconName::X, true).on_click(cx.listener(|this, _, _, cx| { this.close_workflow_editor(); cx.notify(); }))))
                .child(div().flex().items_center().gap_2().child(icon(IconName::GitBranch, ui_theme::PRIMARY))
                    .child(hint(khaslana::ui_i18n::text(deep_messages::CURRENT_BRANCH))).child(text(branch))))
            .when_some(editor.data.error.clone(), |panel, error| panel.child(div().mx_4().mb_3().p_3()
                .rounded(px(ui_theme::RADIUS_SM)).bg(rgb(ui_theme::FEEDBACK_ERROR_BG))
                .child(text(error).text_color(rgb(ui_theme::FEEDBACK_ERROR_TEXT)))))
            .child(scrollable_frame_when("workflow-v2-document", ScrollbarMode::Vertical,
                div().id("workflow-v2-document").flex().flex_col().flex_1().min_h(px(0.0)).p_4()
                    .bg(rgb(ui_theme::WORKFLOW_SURFACE)).overflow_y_scroll().track_scroll(&scroll_handle).child(columns)
                    .when(!wide, |content| content.child(self.render_document_variables(cx).mt_3())).into_any_element(),
                scroll_handle, true, cx))
            .when(editor.step_picker_open, |panel| {
                let picker_scroll = self.scroll_handle("workflow-v2-picker-scroll");
                let width = (f32::from(window.viewport_size().width) - 320.0).clamp(280.0, 560.0);
                let height = (f32::from(window.viewport_size().height) - 220.0).clamp(180.0, 600.0);
                panel.child(div().absolute().inset_0().flex().items_center().justify_center()
                    .bg(crate::ui::theme::rgba(ui_theme::DIALOG_OVERLAY))
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(floating_panel().w(px(width)).max_h(px(height)).flex().flex_col().min_h(px(0.0))
                        .focus_trap("workflow-v2-picker-trap", &editor.document_picker_focus)
                        .child(scrollable_frame_when("workflow-v2-picker-scroll", ScrollbarMode::Vertical,
                            div().id("workflow-v2-picker-scroll").flex().flex_col().min_h(px(0.0)).max_h(px(height))
                                .overflow_y_scroll().track_scroll(&picker_scroll).child(self.render_document_step_picker(window, cx)).into_any_element(),
                            picker_scroll, true, cx))))
            })
            .into_any_element()
    }

    fn render_document_step(&self, index: usize, window: &Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let editor = self.workflow_editor.as_ref().expect("编辑器状态缺失");
        let step = &editor.data.steps[index];
        let expanded = editor.data.selected_step == index;
        let title = if step.kind == WorkflowStepKind::Invoke { step.invoke.title() } else { step.kind.display_name() };
        let (glyph, kind) = step_icon(step);
        let identity = if step.kind == WorkflowStepKind::Invoke { format!("id: {}", step.invoke.value(WorkflowStepSlot::StepId)) } else { step.kind.op_name().to_string() };
        let enabled = !editor.ai_loading;
        let mut header = div().flex().items_center().min_w_0().gap_2().p_3()
            .when(expanded, |header| header.bg(rgb(ui_theme::WORKFLOW_SURFACE)))
            .child(Button::new(format!("workflow-v2-expand-{index}")).ghost().compact().flex_1().min_w(px(0.0)).h(px(32.0)).px_0().py_0()
                .accessibility_label(khaslana::ui_i18n::format(if expanded { deep_messages::COLLAPSE_STEP_NUMBER } else { deep_messages::EXPAND_STEP_NUMBER }, &[("index", &(index + 1).to_string())]))
                .child(div().flex().items_center().justify_start().w_full().min_w_0().gap_3()
                    .child(div().flex().items_center().justify_center().w(px(32.0)).h(px(32.0)).flex_none().rounded_full()
                        .bg(rgb(ui_theme::PRIMARY_SUBTLE)).text_color(rgb(ui_theme::PRIMARY)).child(format!("{:02}", index + 1)))
                    .child(icon(glyph, if kind == "Skill" { ui_theme::PRIMARY } else { ui_theme::CONTENT_SECONDARY }))
                    .child(div().flex().flex_wrap().items_center().min_w_0().gap_2()
                        .child(text(title.to_string()).font_weight(gpui::FontWeight::SEMIBOLD)).child(hint(identity)).child(badge(kind))))
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(editor) = this.workflow_editor.as_mut() { editor.data.selected_step = if expanded { usize::MAX } else { index }; } cx.notify();
                })));
        if expanded {
            for (action, label, glyph, available) in [("up", khaslana::ui_i18n::text(deep_messages::MOVE_UP), IconName::ArrowUp, index > 0), ("down", khaslana::ui_i18n::text(deep_messages::MOVE_DOWN), IconName::ArrowDown, index + 1 < editor.data.steps.len()), ("copy", khaslana::ui_i18n::text(deep_messages::COPY), IconName::Copy, true), ("delete", khaslana::ui_i18n::text(deep_messages::DELETE), IconName::Trash, true)] {
                header = header.child(icon_button(format!("workflow-v2-{action}-{index}"), if available { label } else if action == "up" { khaslana::ui_i18n::text(deep_messages::ALREADY_THE_FIRST_STEP) } else { khaslana::ui_i18n::text(deep_messages::ALREADY_THE_LAST_STEP) }, glyph, enabled && available)
                    .when(action == "delete", |button| button.icon(icon(glyph, ui_theme::DESTRUCTIVE)))
                    .on_click(cx.listener(move |this, _, _, cx| { this.document_step_action(index, action); cx.notify(); })));
            }
            if step.kind == WorkflowStepKind::Invoke {
                header = header.child(icon_button(format!("workflow-v2-advanced-{index}"), khaslana::ui_i18n::text(deep_messages::ADVANCED_SETTINGS), IconName::Settings2, enabled)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(editor) = this.workflow_editor.as_mut() { editor.document_advanced_step = if editor.document_advanced_step == Some(index) { None } else { Some(index) }; } cx.notify();
                    })));
            }
        } else {
            let entity = cx.entity();
            let count = editor.data.steps.len();
            header = header.child(icon_button(format!("workflow-v2-more-{index}"), khaslana::ui_i18n::text(deep_messages::STEP_ACTIONS), IconName::Ellipsis, enabled)
                .dropdown_menu(move |mut menu, window, _| {
                    for (action, label, available) in [("up", khaslana::ui_i18n::text(deep_messages::MOVE_UP), index > 0), ("down", khaslana::ui_i18n::text(deep_messages::MOVE_DOWN), index + 1 < count), ("copy", khaslana::ui_i18n::text(deep_messages::COPY), true), ("delete", khaslana::ui_i18n::text(deep_messages::DELETE), true)] {
                        if available { menu = menu.item(PopupMenuItem::new(label).on_click(window.listener_for(&entity, move |this, _, _, cx| { this.document_step_action(index, action); cx.notify(); }))); }
                    } menu
                }));
        }
        header = header.child(icon_button(format!("workflow-v2-chevron-{index}"), if expanded { khaslana::ui_i18n::text(deep_messages::COLLAPSE_STEP) } else { khaslana::ui_i18n::text(deep_messages::EXPAND_STEP) }, if expanded { IconName::ChevronUp } else { IconName::ChevronDown }, true)
            .on_click(cx.listener(move |this, _, _, cx| { if let Some(editor) = this.workflow_editor.as_mut() { editor.data.selected_step = if expanded { usize::MAX } else { index }; } cx.notify(); })));
        let mut panel = card().p_0().gap_0().border_color(rgb(if expanded { ui_theme::INPUT_BORDER_FOCUSED } else { ui_theme::BORDER_MUTED })).child(header);
        if !expanded {
            let summary = if step.kind == WorkflowStepKind::Invoke {
                let value = match step.invoke.value(WorkflowStepSlot::Uses) { "skill.run" => step.invoke.value(WorkflowStepSlot::Task), "mcp.call" => step.invoke.value(WorkflowStepSlot::Tool), _ => step.invoke.value(WorkflowStepSlot::Uses) };
                value.lines().next().unwrap_or_default().to_string()
            } else {
                step.kind.slots().iter().filter_map(|slot| { let value = step.slot_value(*slot); (!value.is_empty()).then(|| format!("{}：{value}", slot.label())) }).collect::<Vec<_>>().join(" · ")
            };
            return panel.when(!summary.trim().is_empty(), |panel| panel.child(hint(summary).pl(px(76.0)).pr_3().pb_3())).into_any_element();
        }
        let mut body = column().p_3();
        if step.kind != WorkflowStepKind::Invoke {
            body = body.child(self.document_git_parameters(index, window, cx));
            let entity = cx.entity();
            let current_kind = step.kind;
            let width = self.document_menu_width(format!("kind-{index}"));
            let measured = width.clone();
            body = body.child(div().flex().items_center().gap_3()
                .child(text(khaslana::ui_i18n::text(deep_messages::ACTION_TYPE)).w(px(108.0)).flex_none())
                .child(measure_menu_field(div().flex_1().min_w_0().h(px(36.0)).child(
                    menu_button(format!("workflow-v2-kind-{index}"), step.kind.display_name(), enabled).accessibility_label(khaslana::ui_i18n::text(deep_messages::CHANGE_GIT_ACTION_TYPE))
                .dropdown_menu(move |mut menu, window, _| {
                    menu = fit_menu(menu, &width, window).scrollable(true).max_h(px(240.0));
                    for kind in WorkflowStepKind::all() { menu = menu.item(PopupMenuItem::new(kind.display_name()).checked(kind == current_kind).on_click(window.listener_for(&entity, move |this, _, _, cx| { this.workflow_editor_set_step_kind(index, kind); cx.notify(); }))); } menu
                })), measured)));
            return panel.child(body).into_any_element();
        }
        for (slot, _, _) in step.invoke.parameters() {
            if slot != WorkflowStepSlot::Tools { body = body.child(self.document_parameter(index, slot, window, cx)); }
        }
        body = body.child(self.document_parameter(index, WorkflowStepSlot::SaveAs, window, cx))
            .when_some(self.ai_extensions.error.clone(), |body, error| body.child(hint(error)));
        if step.invoke.parameters().iter().any(|(slot, _, _)| *slot == WorkflowStepSlot::Tools) {
            body = body.child(self.document_tools(index, cx));
        }
        if step.invoke.value(WorkflowStepSlot::Uses) == "mcp.call" {
            body = body.child(hint(khaslana::ui_i18n::text(deep_messages::THIS_IS_A_COMPATIBILITY_STEP_FROM_A_LEGACY_TEMPLATE_FOR_NEW_WORKFLOWS)));
        }
        if editor.document_advanced_step == Some(index) || step.invoke.parameters().is_empty() {
            for slot in [WorkflowStepSlot::StepId, WorkflowStepSlot::Uses, WorkflowStepSlot::Arguments] { body = body.child(self.document_field(WorkflowEditorFieldId::StepParam { step: index, slot }, slot.label(), window, cx)); }
            if step.invoke.parameters().iter().any(|(slot, _, _)| *slot == WorkflowStepSlot::Tools) {
                body = body.child(self.document_field(WorkflowEditorFieldId::StepParam { step: index, slot: WorkflowStepSlot::Tools }, khaslana::ui_i18n::text(deep_messages::FULL_TOOL_ARGUMENTS), window, cx));
            }
            body = body.child(hint(khaslana::ui_i18n::text(deep_messages::ADVANCED_ARGUMENTS_PRESERVE_EXTRA_SETTINGS_SUCH_AS_BROWSERGUARD_FORM_FIELDS_TAKE_PRECEDENCE)));
        }
        panel = panel.child(body);
        panel.into_any_element()
    }

    fn document_step_action(&mut self, index: usize, action: &str) {
        match action { "up" => self.workflow_editor_move_step(index, true), "down" => self.workflow_editor_move_step(index, false), "copy" => self.workflow_document_duplicate(index), _ => self.workflow_editor_remove_step(index) }
    }

    fn render_document_step_picker(&self, window: &Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let editor = self.workflow_editor.as_ref().expect("编辑器状态缺失");
        let query = editor.picker_search_field.value.trim().to_lowercase();
        let mut picker = card().child(div().flex().justify_between().child(text(khaslana::ui_i18n::text(deep_messages::ADD_STEP)).font_weight(gpui::FontWeight::SEMIBOLD))
            .child(icon_button("workflow-v2-picker-close", khaslana::ui_i18n::text(deep_messages::BACK_TO_STEPS), IconName::X, true).on_click(cx.listener(|this, _, _, cx| { this.workflow_editor_close_step_picker(); cx.notify(); }))))
            .child(hint(khaslana::ui_i18n::text(deep_messages::CHOOSE_THE_NEXT_ACTION_SKILL_OR_JAVASCRIPT_CALLS_MCP_TOOLS_AS_NEEDED)))
            .child(self.document_input(WorkflowEditorFieldId::PickerSearch, window, cx));
        for (uses, label, description) in filter_document_actions(&query) {
            picker = picker.child(self.document_command(format!("workflow-v2-picker-{uses}"), label, false, true, cx)
                .h_auto().py_3().justify_start().icon(icon(if uses == "js.run" { IconName::Braces } else { IconName::Sparkles }, ui_theme::PRIMARY))
                .child(hint(description))
                .on_click(cx.listener(move |this, _, _, cx| { this.workflow_document_add_action(uses); cx.notify(); })));
        }
        picker = picker.child(text(khaslana::ui_i18n::text(deep_messages::GIT_ACTIONS)));
        for kind in filter_workflow_step_kinds(&query) {
            picker = picker.child(self.document_command(format!("workflow-v2-picker-{}", kind.op_name()), kind.display_name(), false, true, cx)
                .h_auto().py_3().justify_start().icon(icon(IconName::GitBranch, ui_theme::WORKFLOW_META)).child(hint(kind.description()))
                .on_click(cx.listener(move |this, _, _, cx| { this.workflow_editor_add_step_of_kind(kind); cx.notify(); })));
        }
        picker.into_any_element()
    }

    fn render_document_variables(&self, cx: &Context<Self>) -> gpui::Div {
        let editor = self.workflow_editor.as_ref().expect("编辑器状态缺失");
        let mut rail = card().child(text(khaslana::ui_i18n::text(deep_messages::AVAILABLE_VARIABLES)).font_weight(gpui::FontWeight::SEMIBOLD))
            .child(hint(khaslana::ui_i18n::text(deep_messages::USE_IN_FIELDS_OR_TASK_DESCRIPTIONS_OUTPUTS_ARE_RESOLVED_IN_STEP_ORDER)));
        for (index, row) in editor.data.inputs.iter().enumerate() {
            if !row.key.trim().is_empty() { rail = rail.child(self.document_variable_card(format!("input-{index}"), format!("${{{}}}", row.key), row.description.clone(), Some(row.default_value.clone()), cx)); }
        }
        for (index, row) in editor.data.vars.iter().enumerate() { rail = rail.child(self.document_variable_card(format!("custom-{index}"), format!("${{{}}}", row.key), khaslana::ui_i18n::text(deep_messages::CUSTOM_VARIABLES).into(), None, cx)); }
        for (index, step) in editor.data.steps.iter().enumerate() {
            if index >= editor.data.selected_step { break; }
            let output = if step.kind == WorkflowStepKind::Invoke { step.invoke.value(WorkflowStepSlot::SaveAs) } else if step.kind == WorkflowStepKind::FilterBranches { &step.output } else { "" };
            if !output.is_empty() { rail = rail.child(self.document_variable_card(format!("output-{index}-{output}"), format!("${{out.{output}}}"), khaslana::ui_i18n::format(deep_messages::OUTPUT_OF_STEP, &[("value0", &(index + 1).to_string())]), None, cx)); }
        }
        rail.child(self.document_variable_card("run-source-branch".into(), "${run.sourceBranch}".into(), khaslana::ui_i18n::text(deep_messages::SOURCE_BRANCH_NAME_AT_RUN_TIME).into(), None, cx))
            .child(self.document_variable_card("git-current-branch".into(), "${git.currentBranch}".into(), khaslana::ui_i18n::text(deep_messages::BRANCH_AT_THE_CURRENT_STEP).into(), None, cx))
    }
}
