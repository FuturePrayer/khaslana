//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const SHORTCUT_DEFAULT_KEYSTROKE_IS_ALREADY_USED_BY_DESCRIBE_CANNOT_RESTORE: UiMessage = UiMessage::new("views.shortcuts_view.shortcut_default_keystroke_is_already_used_by_describe_cannot_restore", "快捷键 {default_keystroke} 已被「{describe}」占用，无法恢复默认", "Shortcut {default_keystroke} is already used by “{describe}”; cannot restore the default");
pub const FINISH_RECORDING_THE_SHORTCUT_FIRST: UiMessage = UiMessage::new("views.shortcuts_view.finish_recording_the_shortcut_first", "请先结束快捷键录制", "Finish recording the shortcut first");
pub const APPLICATION_SHORTCUTS: UiMessage = UiMessage::new("views.shortcuts_view.application_shortcuts", "应用快捷键", "Application shortcuts");
pub const DESCRIPTION: UiMessage = UiMessage::new("views.shortcuts_view.description", "说明", "Description");
pub const CLICK_REBIND_THEN_PRESS_A_KEY_COMBINATION_ESC_CANCELS_RECORDING: UiMessage = UiMessage::new("views.shortcuts_view.click_rebind_then_press_a_key_combination_esc_cancels_recording", "点击「重新绑定」后按下组合键录入；按 Esc 取消录制。点「恢复默认」复位单条快捷键。", "Click “Rebind”, then press a key combination. Esc cancels recording. “Restore default” resets a single shortcut.");
pub const PRESS_A_KEY_COMBINATION: UiMessage = UiMessage::new("views.shortcuts_view.press_a_key_combination", "按下组合键…", "Press a key combination…");
pub const CANCEL: UiMessage = UiMessage::new("views.shortcuts_view.cancel", "取消", "Cancel");
pub const REBIND: UiMessage = UiMessage::new("views.shortcuts_view.rebind", "重新绑定", "Rebind");
pub const RESTORE_DEFAULT: UiMessage = UiMessage::new("views.shortcuts_view.restore_default", "恢复默认", "Restore default");
pub const WORKFLOW_SHORTCUTS: UiMessage = UiMessage::new("views.shortcuts_view.workflow_shortcuts", "工作流快捷键", "Workflow shortcuts");
pub const RIGHT_CLICK_A_TEMPLATE_ON_THE_WORKFLOWS_PAGE_AND_CHOOSE: UiMessage = UiMessage::new("views.shortcuts_view.right_click_a_template_on_the_workflows_page_and_choose", "在工作流页右键模板行选择「绑定快捷键...」录制；此处可查看与清除。", "Right-click a template on the Workflows page and choose “Bind shortcut…” to record one. View and clear bindings here.");
pub const NO_WORKFLOW_SHORTCUTS: UiMessage = UiMessage::new("views.shortcuts_view.no_workflow_shortcuts", "暂无工作流快捷键", "No workflow shortcuts");
pub const BACKGROUND: UiMessage = UiMessage::new("views.shortcuts_view.background", "后台", "Background");
pub const CLEAR: UiMessage = UiMessage::new("views.shortcuts_view.clear", "清除", "Clear");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    SHORTCUT_DEFAULT_KEYSTROKE_IS_ALREADY_USED_BY_DESCRIBE_CANNOT_RESTORE,
    FINISH_RECORDING_THE_SHORTCUT_FIRST,
    APPLICATION_SHORTCUTS,
    DESCRIPTION,
    CLICK_REBIND_THEN_PRESS_A_KEY_COMBINATION_ESC_CANCELS_RECORDING,
    PRESS_A_KEY_COMBINATION,
    CANCEL,
    REBIND,
    RESTORE_DEFAULT,
    WORKFLOW_SHORTCUTS,
    RIGHT_CLICK_A_TEMPLATE_ON_THE_WORKFLOWS_PAGE_AND_CHOOSE,
    NO_WORKFLOW_SHORTCUTS,
    BACKGROUND,
    CLEAR,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
