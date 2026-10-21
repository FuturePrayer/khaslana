//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const ENCODING_LABEL: UiMessage = UiMessage::new("views.browse_view.encoding_label", "编码：{label}", "Encoding: {label}");
pub const FULL_TARGET_BRANCH_CONTENT: UiMessage = UiMessage::new("views.browse_view.full_target_branch_content", "目标分支全文", "Full target branch content");
pub const FILE_CONTENT: UiMessage = UiMessage::new("views.browse_view.file_content", "文件内容", "File content");
pub const DIFFERENCES_FROM_CURRENT_BRANCH: UiMessage = UiMessage::new("views.browse_view.differences_from_current_branch", "与当前分支差异", "Differences from current branch");
pub const LOADING: UiMessage = UiMessage::new("views.browse_view.loading", "加载中...", "Loading…");
pub const FILE_TREE: UiMessage = UiMessage::new("views.browse_view.file_tree", "文件树", "File tree");
pub const RESOLVING_REFERENCE: UiMessage = UiMessage::new("views.browse_view.resolving_reference", "正在解析引用...", "Resolving reference…");
pub const LOADING_FILE_TREE: UiMessage = UiMessage::new("views.browse_view.loading_file_tree", "正在加载文件树...", "Loading file tree…");
pub const THE_REPOSITORY_IS_EMPTY: UiMessage = UiMessage::new("views.browse_view.the_repository_is_empty", "仓库为空", "The repository is empty");
pub const BROWSE_BRANCH: UiMessage = UiMessage::new("views.browse_view.browse_branch", "分支浏览", "Browse branch");
pub const READ_ONLY_FILE_TREE_OF_THE_TARGET_REFERENCE: UiMessage = UiMessage::new("views.browse_view.read_only_file_tree_of_the_target_reference", "目标引用的只读文件树", "Read-only file tree of the target reference");
pub const CLOSE: UiMessage = UiMessage::new("views.browse_view.close", "关闭", "Close");
pub const NO_FILE_SELECTED: UiMessage = UiMessage::new("views.browse_view.no_file_selected", "未选择文件", "No file selected");
pub const CONTENT_CANVAS: UiMessage = UiMessage::new("views.browse_view.content_canvas", "内容画布", "Content canvas");
pub const READ_ONLY_CONTENT_OR_DIFFERENCES_FOR_THE_SELECTED_FILE: UiMessage = UiMessage::new("views.browse_view.read_only_content_or_differences_for_the_selected_file", "所选文件的只读内容或差异", "Read-only content or differences for the selected file");
pub const CONTENT: UiMessage = UiMessage::new("views.browse_view.content", "内容", "Content");
pub const DIFF: UiMessage = UiMessage::new("views.browse_view.diff", "差异", "Diff");
pub const LOADING_FILE_CONTENT: UiMessage = UiMessage::new("views.browse_view.loading_file_content", "正在加载文件内容...", "Loading file content…");
pub const THIS_FILE_DOES_NOT_EXIST_IN_THE_TARGET_BRANCH_SWITCH: UiMessage = UiMessage::new("views.browse_view.this_file_does_not_exist_in_the_target_branch_switch", "目标分支中不存在该文件，请切换到差异视图查看删除内容", "This file does not exist in the target branch. Switch to the diff view to see deleted content.");
pub const SELECT_A_FILE_TO_VIEW_ITS_CONTENT: UiMessage = UiMessage::new("views.browse_view.select_a_file_to_view_its_content", "请选择一个文件查看内容", "Select a file to view its content");
pub const CANNOT_PREVIEW_BINARY_FILES: UiMessage = UiMessage::new("views.browse_view.cannot_preview_binary_files", "无法预览二进制文件", "Cannot preview binary files");
pub const SWITCH_TO_THE_DIFF_VIEW_TO_SEE_FILE_CHANGE_INFORMATION: UiMessage = UiMessage::new("views.browse_view.switch_to_the_diff_view_to_see_file_change_information", "请切换到差异视图查看文件变更信息", "Switch to the diff view to see file change information");
pub const LOADING_FILE_DIFFERENCES: UiMessage = UiMessage::new("views.browse_view.loading_file_differences", "文件差异加载中...", "Loading file differences…");
pub const SELECT_A_FILE_TO_COMPARE_WITH_THE_CURRENT_BRANCH: UiMessage = UiMessage::new("views.browse_view.select_a_file_to_compare_with_the_current_branch", "请选择一个文件查看与当前分支的差异", "Select a file to compare with the current branch");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    ENCODING_LABEL,
    FULL_TARGET_BRANCH_CONTENT,
    FILE_CONTENT,
    DIFFERENCES_FROM_CURRENT_BRANCH,
    LOADING,
    FILE_TREE,
    RESOLVING_REFERENCE,
    LOADING_FILE_TREE,
    THE_REPOSITORY_IS_EMPTY,
    BROWSE_BRANCH,
    READ_ONLY_FILE_TREE_OF_THE_TARGET_REFERENCE,
    CLOSE,
    NO_FILE_SELECTED,
    CONTENT_CANVAS,
    READ_ONLY_CONTENT_OR_DIFFERENCES_FOR_THE_SELECTED_FILE,
    CONTENT,
    DIFF,
    LOADING_FILE_CONTENT,
    THIS_FILE_DOES_NOT_EXIST_IN_THE_TARGET_BRANCH_SWITCH,
    SELECT_A_FILE_TO_VIEW_ITS_CONTENT,
    CANNOT_PREVIEW_BINARY_FILES,
    SWITCH_TO_THE_DIFF_VIEW_TO_SEE_FILE_CHANGE_INFORMATION,
    LOADING_FILE_DIFFERENCES,
    SELECT_A_FILE_TO_COMPARE_WITH_THE_CURRENT_BRANCH,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
