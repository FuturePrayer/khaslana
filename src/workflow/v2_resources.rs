//! 工作流本模块的稳定可见消息。
use crate::ui_i18n::UiMessage;
pub const INVALID_WORKFLOW_ACTION_NAME: UiMessage = UiMessage::new("workflow.v2.invalid_workflow_action_name", "工作流动作名称无效：{name}", "Invalid workflow action name: {name}");
pub const WORKFLOW_ACTION_REGISTERED_MORE_THAN_ONCE: UiMessage = UiMessage::new("workflow.v2.workflow_action_registered_more_than_once", "工作流动作重复注册：{name}", "Workflow action registered more than once: {name}");
pub const WORKFLOW_ACTION_IS_NOT_REGISTERED: UiMessage = UiMessage::new("workflow.v2.workflow_action_is_not_registered", "未注册工作流动作：{name}", "Workflow action is not registered: {name}");
pub const THE_WITH_ARGUMENT_OF_WORKFLOW_ACTION_MUST_BE_AN_OBJECT: UiMessage = UiMessage::new("workflow.v2.the_with_argument_of_workflow_action_must_be_an_object", "工作流动作 {uses} 的 with 必须是对象", "The with argument of workflow action {uses} must be an object");
pub const INVALID_ARGUMENTS_FOR_WORKFLOW_ACTION: UiMessage = UiMessage::new("workflow.v2.invalid_arguments_for_workflow_action", "工作流动作 {uses} 的参数无效：{err}", "Invalid arguments for workflow action {uses}: {err}");
