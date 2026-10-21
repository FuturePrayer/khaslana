//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const COMPARING_TARGET_DISPLAY: UiMessage = UiMessage::new("views.browse_compare_view.comparing_target_display", "比较：{target_display}", "Comparing: {target_display}");
pub const CHANGED_FILES_FILE_COUNT: UiMessage = UiMessage::new("views.browse_compare_view.changed_files_file_count", "差异文件 · {file_count}", "Changed files · {file_count}");
pub const LOADING: UiMessage = UiMessage::new("views.browse_compare_view.loading", "加载中...", "Loading…");
pub const CHANGED_FILES: UiMessage = UiMessage::new("views.browse_compare_view.changed_files", "差异文件", "Changed files");
pub const RESOLVING_REFERENCE: UiMessage = UiMessage::new("views.browse_compare_view.resolving_reference", "正在解析引用...", "Resolving reference…");
pub const LOADING_BRANCH_DIFFERENCES: UiMessage = UiMessage::new("views.browse_compare_view.loading_branch_differences", "正在加载分支差异...", "Loading branch differences…");
pub const THIS_BRANCH_HAS_NO_DIFFERENCES_FROM_THE_CURRENT_BRANCH: UiMessage = UiMessage::new("views.browse_compare_view.this_branch_has_no_differences_from_the_current_branch", "该分支与当前分支没有差异", "This branch has no differences from the current branch");
pub const COMPARE_BRANCHES: UiMessage = UiMessage::new("views.browse_compare_view.compare_branches", "分支比较", "Compare branches");
pub const CHANGES_IN_THE_TARGET_BRANCH_AHEAD_OF_THE_CURRENT_BRANCH: UiMessage = UiMessage::new("views.browse_compare_view.changes_in_the_target_branch_ahead_of_the_current_branch", "目标分支领先当前分支的变更", "Changes in the target branch ahead of the current branch");
pub const CLOSE: UiMessage = UiMessage::new("views.browse_compare_view.close", "关闭", "Close");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    COMPARING_TARGET_DISPLAY,
    CHANGED_FILES_FILE_COUNT,
    LOADING,
    CHANGED_FILES,
    RESOLVING_REFERENCE,
    LOADING_BRANCH_DIFFERENCES,
    THIS_BRANCH_HAS_NO_DIFFERENCES_FROM_THE_CURRENT_BRANCH,
    COMPARE_BRANCHES,
    CHANGES_IN_THE_TARGET_BRANCH_AHEAD_OF_THE_CURRENT_BRANCH,
    CLOSE,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
