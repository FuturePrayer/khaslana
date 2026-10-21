//! 工作流本模块的稳定可见消息。
use khaslana::ui_i18n::UiMessage;
pub const DELETE_OPTION: UiMessage = UiMessage::new("workflow_editor.inputs.delete_option", "删除第 {value0} 个选项", "Delete option {value0}");
pub const INPUT_CONTROL_TYPE: UiMessage = UiMessage::new("workflow_editor.inputs.input_control_type", "输入控件类型", "Input control type");
pub const DROPDOWN_OPTIONS: UiMessage = UiMessage::new("workflow_editor.inputs.dropdown_options", "下拉选项", "Dropdown options");
pub const DISPLAYED_IN_THIS_ORDER_THE_WORKFLOW_RECEIVES_THE_ACTUAL_VALUE: UiMessage = UiMessage::new("workflow_editor.inputs.displayed_in_this_order_the_workflow_receives_the_actual_value", "按此顺序展示，工作流接收实际值", "Displayed in this order; the workflow receives the actual value");
pub const NO_OPTIONS_ADDED_CONFIGURE_AT_LEAST_ONE_DISPLAY_NAME_AND_ACTUAL_VALUE: UiMessage = UiMessage::new("workflow_editor.inputs.no_options_added_configure_at_least_one_display_name_and_actual_value", "尚未添加选项，请至少配置一个显示名称和实际值。", "No options added. Configure at least one display name and actual value.");
pub const DISPLAY_NAME: UiMessage = UiMessage::new("workflow_editor.inputs.display_name", "显示名称", "Display name");
pub const ACTUAL_VALUE: UiMessage = UiMessage::new("workflow_editor.inputs.actual_value", "实际值", "Actual value");
pub const ADD_OPTION: UiMessage = UiMessage::new("workflow_editor.inputs.add_option", "添加选项", "Add option");
