//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const WAIT_FOR_THE_CURRENT_OPERATION_TO_FINISH_OTHER_OPERATIONS_ARE: UiMessage = UiMessage::new("views.operation_blocker_view.wait_for_the_current_operation_to_finish_other_operations_are", "请等待当前操作完成，期间不能进行其它操作。", "Wait for the current operation to finish. Other operations are unavailable in the meantime.");
pub const STOP_REMAINING_STEPS: UiMessage = UiMessage::new("views.operation_blocker_view.stop_remaining_steps", "停止后续步骤", "Stop remaining steps");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    WAIT_FOR_THE_CURRENT_OPERATION_TO_FINISH_OTHER_OPERATIONS_ARE,
    STOP_REMAINING_STEPS,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
