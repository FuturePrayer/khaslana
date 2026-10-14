//! 两种编辑器共用输入类型和选项编辑，避免保存时丢失类型信息。
use gpui::{Context, IntoElement, Window, div, prelude::*};
use gpui_kit::component::select::{Select, SelectEvent, SelectState};

use super::*;
use super::document::{column, hint, text};
use super::visual::{icon, icon_button};
use gpui_kit::assets::IconName;

const KINDS: [WorkflowInputKind; 4] = [WorkflowInputKind::Text, WorkflowInputKind::Textarea, WorkflowInputKind::Select, WorkflowInputKind::Date];

impl RepositoryView {
    pub(crate) fn workflow_editor_input_kind(&self, index: usize) -> Option<WorkflowInputKind> {
        self.workflow_editor.as_ref()?.data.inputs.get(index).map(|input| input.kind)
    }
    pub(crate) fn ensure_workflow_editor_input_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = self.workflow_editor.as_mut() else { return };
        for index in 0..editor.data.inputs.len() {
            let Some(uid) = editor.input_fields[index].key_field.as_ref().map(|field| field.uid) else { continue };
            if editor.input_fields[index].kind_control.is_none() {
                let state = cx.new(|cx| SelectState::new(KINDS.iter().map(|kind| kind.label().to_string()).collect::<Vec<_>>(), None, window, cx));
                cx.subscribe_in(&state, window, move |this, _, event: &SelectEvent<Vec<String>>, _, cx| {
                    let Some(editor) = this.workflow_editor.as_mut() else { return };
                    let Some(index) = editor.input_fields.iter().position(|row| row.key_field.as_ref().is_some_and(|field| field.uid == uid)) else { return };
                    let SelectEvent::Confirm(Some(value)) = event else { return };
                    if let Some(kind) = KINDS.iter().find(|kind| kind.label() == value) {
                        editor.data.inputs[index].kind = *kind;
                        cx.notify();
                    }
                }).detach();
                editor.input_fields[index].kind_control = Some(state);
            }
            let state = editor.input_fields[index].kind_control.as_ref().expect("输入类型选择器已初始化");
            let value = editor.data.inputs[index].kind.label().to_string();
            if state.read(cx).selected_value() != Some(&value) {
                state.update(cx, |state, cx| state.set_selected_value(&value, window, cx));
            }
        }
    }

    pub(super) fn render_workflow_editor_input_type(&self, index: usize) -> gpui::AnyElement {
        self.workflow_editor.as_ref().and_then(|editor| editor.input_fields.get(index))
            .and_then(|row| row.kind_control.as_ref()).map(|state| Select::new(state)
                .accessibility_label("输入控件类型").w_full().h(px(36.0)).text_size(px(14.0))
                .bg(rgb(ui_theme::WB_INPUT_SURFACE)).border_color(rgb(ui_theme::BORDER_MUTED))
                .rounded(px(ui_theme::RADIUS_SM)).into_any_element())
            .unwrap_or_else(|| div().into_any_element())
    }

    pub(super) fn render_workflow_editor_options(&self, index: usize, window: &Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let Some(editor) = self.workflow_editor.as_ref() else { return div().into_any_element() };
        let Some(row) = editor.data.inputs.get(index).filter(|row| row.kind == WorkflowInputKind::Select) else { return div().into_any_element() };
        let Some(uid) = editor.input_fields[index].key_field.as_ref().map(|field| field.uid) else { return div().into_any_element() };
        let mut content = column().gap_2()
            .child(div().flex().flex_wrap().items_center().gap_2()
                .child(text("下拉选项").font_weight(gpui::FontWeight::MEDIUM))
                .child(hint("按此顺序展示，工作流接收实际值")));
        if row.options.is_empty() { content = content.child(hint("尚未添加选项，请至少配置一个显示名称和实际值。")); }
        for option in 0..row.options.len() {
            let Some(option_uid) = editor.input_fields[index].option_fields.get(option).map(|fields| fields.0.uid) else { continue };
            content = content.child(div().flex().flex_wrap().items_start().gap_2()
                .child(column().flex_1().min_w(px(140.0)).gap_1().child(hint("显示名称"))
                    .child(self.document_input(WorkflowEditorFieldId::InputPart { index, part: WorkflowInputPart::OptionLabel(option) }, window, cx)))
                .child(column().flex_1().min_w(px(140.0)).gap_1().child(hint("实际值"))
                    .child(self.document_input(WorkflowEditorFieldId::InputPart { index, part: WorkflowInputPart::OptionValue(option) }, window, cx)))
                .child(div().pt(px(24.0)).flex_none().child(icon_button(format!("workflow-option-remove-{uid}-{option_uid}"), &format!("删除第 {} 个选项", option + 1), IconName::Trash, true)
                    .icon(icon(IconName::Trash, ui_theme::DESTRUCTIVE))
                    .on_click(cx.listener(move |this, _, _, cx| { this.edit_workflow_option(uid, Some(option_uid)); cx.notify(); })))));
        }
        content.child(div().flex().child(self.document_command(format!("workflow-option-add-{uid}"), "添加选项", false, true, cx)
            .icon(icon(IconName::Plus, ui_theme::CONTENT_SECONDARY))
            .on_click(cx.listener(move |this, _, _, cx| { this.edit_workflow_option(uid, None); cx.notify(); })))).into_any_element()
    }

    fn edit_workflow_option(&mut self, uid: u64, remove: Option<u64>) {
        let Some(editor) = self.workflow_editor.as_mut() else { return };
        editor.sync_from_fields();
        let Some(index) = editor.input_fields.iter().position(|row| row.key_field.as_ref().is_some_and(|field| field.uid == uid)) else { return };
        if let Some(option_uid) = remove {
            if let Some(option) = editor.input_fields[index].option_fields.iter().position(|fields| fields.0.uid == option_uid) {
                editor.data.inputs[index].options.remove(option);
                editor.input_fields[index].option_fields.remove(option);
            }
        } else {
            editor.data.inputs[index].options.push(WorkflowInputOption { label: String::new(), value: String::new() });
        }
    }
}
