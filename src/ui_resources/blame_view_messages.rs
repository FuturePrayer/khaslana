//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const ENCODING_ENCODING_LABEL: UiMessage = UiMessage::new("views.blame_view.encoding_encoding_label", "编码：{encoding_label}", "Encoding: {encoding_label}");
pub const NO_FILE_SELECTED: UiMessage = UiMessage::new("views.blame_view.no_file_selected", "未选择文件", "No file selected");
pub const FILE_ATTRIBUTION: UiMessage = UiMessage::new("views.blame_view.file_attribution", "文件追溯", "File attribution");
pub const COMMIT_OWNERSHIP_AND_WORKING_TREE_CHANGES: UiMessage = UiMessage::new("views.blame_view.commit_ownership_and_working_tree_changes", "提交归属与工作区变更", "Commit ownership and working tree changes");
pub const LOADING: UiMessage = UiMessage::new("views.blame_view.loading", "加载中...", "Loading…");
pub const CLOSE: UiMessage = UiMessage::new("views.blame_view.close", "关闭", "Close");
pub const LOADING_FILE_ATTRIBUTION: UiMessage = UiMessage::new("views.blame_view.loading_file_attribution", "正在加载文件追溯...", "Loading file attribution…");
pub const SELECT_A_FILE_TO_VIEW_ITS_ATTRIBUTION: UiMessage = UiMessage::new("views.blame_view.select_a_file_to_view_its_attribution", "请选择一个文件查看追溯", "Select a file to view its attribution");
pub const UNCOMMITTED: UiMessage = UiMessage::new("views.blame_view.uncommitted", "未提交", "Uncommitted");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    ENCODING_ENCODING_LABEL,
    NO_FILE_SELECTED,
    FILE_ATTRIBUTION,
    COMMIT_OWNERSHIP_AND_WORKING_TREE_CHANGES,
    LOADING,
    CLOSE,
    LOADING_FILE_ATTRIBUTION,
    SELECT_A_FILE_TO_VIEW_ITS_ATTRIBUTION,
    UNCOMMITTED,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
