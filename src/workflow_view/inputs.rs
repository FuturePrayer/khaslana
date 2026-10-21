//! 表单控件是字符串真值的投影；事件必须匹配仓库及字段 uid。
#[path = "inputs_resources.rs"]
mod deep_messages;

use chrono::NaiveDate;
use gpui::{Context, Entity, Focusable, IntoElement, SharedString, Window, prelude::*};
use gpui_kit::component::{
    Disableable,
    calendar::Date,
    date_picker::{DatePicker, DatePickerEvent, DatePickerState},
    select::{Select, SelectEvent, SelectItem, SelectState},
};
use khaslana::WorkflowInputKind;

use super::*;

#[derive(Clone, Debug)]
pub(crate) struct WorkflowChoice { pub label: String, pub value: String }

impl SelectItem for WorkflowChoice {
    type Value = String;
    fn title(&self) -> SharedString { self.label.clone().into() }
    fn value(&self) -> &String { &self.value }
}

#[derive(Clone, Debug)]
pub(super) enum WorkflowInputControl {
    Select(Entity<SelectState<Vec<WorkflowChoice>>>),
    Date(Entity<DatePickerState>),
}

impl RepositoryView {
    pub(crate) fn ensure_workflow_input_controls(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab_id) = self.active_tab_id() else { return };
        for index in 0..self.workflow_state.inputs.len() {
            let input = &self.workflow_state.inputs[index];
            let uid = input.field.uid;
            let key = input.key.clone();
            let value = input.field.value.clone();
            let kind = input.definition.kind;
            if input.control.is_none() {
                let control = match kind {
                    WorkflowInputKind::Select => {
                        let choices = input.definition.options.iter().map(|option| WorkflowChoice {
                            label: option.label.clone(), value: option.value.clone(),
                        }).collect::<Vec<_>>();
                        let state = cx.new(|cx| SelectState::new(choices, None, window, cx));
                        cx.subscribe_in(&state, window, move |this, _, event: &SelectEvent<Vec<WorkflowChoice>>, _, cx| {
                            let SelectEvent::Confirm(value) = event;
                            this.change_workflow_control(tab_id, &key, uid, value.clone().unwrap_or_default(), cx);
                        }).detach();
                        WorkflowInputControl::Select(state)
                    }
                    WorkflowInputKind::Date => {
                        let state = cx.new(|cx| DatePickerState::new(window, cx).date_format("%Y-%m-%d"));
                        cx.subscribe_in(&state, window, move |this, _, event: &DatePickerEvent, _, cx| {
                            let DatePickerEvent::Change(date) = event;
                            let value = match date { Date::Single(Some(date)) => date.format("%Y-%m-%d").to_string(), _ => String::new() };
                            this.change_workflow_control(tab_id, &key, uid, value, cx);
                        }).detach();
                        WorkflowInputControl::Date(state)
                    }
                    _ => continue,
                };
                self.workflow_state.inputs[index].control = Some(control);
            }
            let control = self.workflow_state.inputs[index].control.clone().expect("选择控件已初始化");
            let focus = match control {
                WorkflowInputControl::Select(state) => {
                    if state.read(cx).selected_value().map(String::as_str).unwrap_or("") != value {
                        state.update(cx, |state, cx| {
                            if value.is_empty() { state.set_selected_index(None, window, cx); }
                            else { state.set_selected_value(&value, window, cx); }
                        });
                    }
                    state.read(cx).focus_handle(cx)
                }
                WorkflowInputControl::Date(state) => {
                    let date = NaiveDate::parse_from_str(&value, "%Y-%m-%d").ok();
                    if state.read(cx).date() != Date::Single(date) {
                        state.update(cx, |state, cx| state.set_date(Date::Single(date), window, cx));
                    }
                    state.read(cx).focus_handle(cx)
                }
            };
            self.workflow_state.inputs[index].field.focus = focus;
        }
    }

    fn change_workflow_control(&mut self, tab_id: crate::RepoTabId, key: &str, uid: u64, value: String, cx: &mut Context<Self>) {
        if self.active_tab_id() != Some(tab_id) || self.busy { return; }
        let Some(input) = self.workflow_state.inputs.iter_mut().find(|input| input.key == key && input.field.uid == uid) else { return };
        if input.field.value == value { return; }
        input.field.set_value(value);
        self.workflow_input_changed();
        cx.notify();
    }

    pub(super) fn render_workflow_input_control(&self, index: usize, window: &Window, cx: &mut Context<Self>) -> gpui::AnyElement {
        let input = &self.workflow_state.inputs[index];
        match &input.control {
            Some(WorkflowInputControl::Select(state)) => Select::new(state).w_full()
                .placeholder(khaslana::ui_i18n::text(deep_messages::SELECT_AN_OPTION)).accessibility_label(input.label.clone()).cleanable(true).disabled(self.busy).into_any_element(),
            Some(WorkflowInputControl::Date(state)) => DatePicker::new(state).w_full()
                .placeholder(khaslana::ui_i18n::text(deep_messages::SELECT_A_DATE)).cleanable(true).disabled(self.busy).into_any_element(),
            _ => self.input(FieldId::WorkflowInput(index), false, window, cx).into_any_element(),
        }
    }
}
