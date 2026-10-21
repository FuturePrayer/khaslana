//! 工作流本模块的稳定可见消息。
use crate::ui_i18n::UiMessage;
pub const WORKFLOW_DROPDOWN_VARIABLE_REQUIRES_AT_LEAST_ONE_OPTION: UiMessage = UiMessage::new("workflow.inputs.workflow_dropdown_variable_requires_at_least_one_option", "工作流下拉变量「{name}」至少需要一个选项", "Workflow dropdown variable \"{name}\" requires at least one option");
pub const WORKFLOW_DROPDOWN_VARIABLE_HAS_AN_EMPTY_OPTION_LABEL_OR_VALUE_OR_DUPLICATE: UiMessage = UiMessage::new("workflow.inputs.workflow_dropdown_variable_has_an_empty_option_label_or_value_or_duplicate", "工作流下拉变量「{name}」的选项名称或值为空，或值重复", "Workflow dropdown variable \"{name}\" has an empty option label or value, or duplicate values");
pub const ONLY_SINGLE_SELECT_WORKFLOW_VARIABLE_CAN_DEFINE_OPTIONS: UiMessage = UiMessage::new("workflow.inputs.only_single_select_workflow_variable_can_define_options", "工作流变量「{name}」只有单选下拉可以配置 options", "Only single-select workflow variable \"{name}\" can define options");
pub const ENTER_A_VALUE_FOR_WORKFLOW_VARIABLE: UiMessage = UiMessage::new("workflow.inputs.enter_a_value_for_workflow_variable", "请填写工作流变量：{label}", "Enter a value for workflow variable: {label}");
pub const SELECT_A_VALID_OPTION_FOR_WORKFLOW_VARIABLE: UiMessage = UiMessage::new("workflow.inputs.select_a_valid_option_for_workflow_variable", "请选择工作流变量「{label}」的有效选项", "Select a valid option for workflow variable \"{label}\"");
pub const WORKFLOW_VARIABLE_MUST_BE_A_VALID_YYYY_MM_DD_DATE: UiMessage = UiMessage::new("workflow.inputs.workflow_variable_must_be_a_valid_yyyy_mm_dd_date", "工作流变量「{label}」必须是有效的 YYYY-MM-DD 日期", "Workflow variable \"{label}\" must be a valid YYYY-MM-DD date");
pub const SINGLE_LINE_TEXT: UiMessage = UiMessage::new("workflow.inputs.single_line_text", "单行文本", "Single-line text");
pub const MULTILINE_TEXT: UiMessage = UiMessage::new("workflow.inputs.multiline_text", "多行文本", "Multiline text");
pub const SINGLE_SELECT_DROPDOWN: UiMessage = UiMessage::new("workflow.inputs.single_select_dropdown", "单选下拉", "Single-select dropdown");
pub const DATE: UiMessage = UiMessage::new("workflow.inputs.date", "日期", "Date");
