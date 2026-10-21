//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const NEW_FILE_NEW_SIZE: UiMessage = UiMessage::new("views.ui_helpers.new_file_new_size", "新增文件 · {new_size}", "New file · {new_size}");
pub const DELETED_ORIGINAL_SIZE_OLD_SIZE: UiMessage = UiMessage::new("views.ui_helpers.deleted_original_size_old_size", "已删除 · 原大小 {old_size}", "Deleted · Original size {old_size}");
pub const SIZE_NEW_SIZE: UiMessage = UiMessage::new("views.ui_helpers.size_new_size", "大小 {new_size}", "Size {new_size}");
pub const BINARY_FILE: UiMessage = UiMessage::new("views.ui_helpers.binary_file", "二进制文件", "Binary file");
pub const CANNOT_DISPLAY_DIFFERENCES_AS_TEXT: UiMessage = UiMessage::new("views.ui_helpers.cannot_display_differences_as_text", "无法以文本形式展示差异", "Cannot display differences as text");
pub const UNKNOWN_TIME: UiMessage = UiMessage::new("views.ui_helpers.unknown_time", "时间未知", "Unknown time");
pub const STAGED: UiMessage = UiMessage::new("views.ui_helpers.staged", "已暂存", "Staged");
pub const UNSTAGED: UiMessage = UiMessage::new("views.ui_helpers.unstaged", "未暂存", "Unstaged");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    NEW_FILE_NEW_SIZE,
    DELETED_ORIGINAL_SIZE_OLD_SIZE,
    SIZE_NEW_SIZE,
    BINARY_FILE,
    CANNOT_DISPLAY_DIFFERENCES_AS_TEXT,
    UNKNOWN_TIME,
    STAGED,
    UNSTAGED,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
