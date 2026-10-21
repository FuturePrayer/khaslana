//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const FAILED_TO_LOAD_SUBMODULE_LIST_ERROR: UiMessage = UiMessage::new("views.submodule_view.failed_to_load_submodule_list_error", "子模块列表加载失败：{error}", "Failed to load submodule list: {error}");
pub const BEHIND_BEHIND: UiMessage = UiMessage::new("views.submodule_view.behind_behind", "落后 {behind}", "Behind {behind}");
pub const AHEAD_AHEAD: UiMessage = UiMessage::new("views.submodule_view.ahead_ahead", "超前 {ahead}", "Ahead {ahead}");
pub const DIVERGED_AHEAD_BEHIND: UiMessage = UiMessage::new("views.submodule_view.diverged_ahead_behind", "分叉 {ahead}/{behind}", "Diverged {ahead}/{behind}");
pub const OPEN_A_REPOSITORY_FIRST: UiMessage = UiMessage::new("views.submodule_view.open_a_repository_first", "请先打开一个仓库", "Open a repository first");
pub const SUBMODULE_OPENED: UiMessage = UiMessage::new("views.submodule_view.submodule_opened", "子模块已打开", "Submodule opened");
pub const LOADING_SUBMODULE_LIST: UiMessage = UiMessage::new("views.submodule_view.loading_submodule_list", "正在加载子模块列表", "Loading submodule list");
pub const CHECKING_SUBMODULE_REMOTE_STATUS: UiMessage = UiMessage::new("views.submodule_view.checking_submodule_remote_status", "正在检查子模块远端状态", "Checking submodule remote status");
pub const SYNC_RECORDED_SUBMODULE_REVISIONS: UiMessage = UiMessage::new("views.submodule_view.sync_recorded_submodule_revisions", "同步子模块记录版本", "Sync recorded submodule revisions");
pub const UPDATE_SUBMODULES: UiMessage = UiMessage::new("views.submodule_view.update_submodules", "更新子模块", "Update submodules");
pub const LOADING_SUBMODULE_LIST_2: UiMessage = UiMessage::new("views.submodule_view.loading_submodule_list_2", "正在加载子模块列表...", "Loading submodule list…");
pub const THIS_REPOSITORY_HAS_NO_SUBMODULES: UiMessage = UiMessage::new("views.submodule_view.this_repository_has_no_submodules", "当前仓库没有子模块", "This repository has no submodules");
pub const SUBMODULES: UiMessage = UiMessage::new("views.submodule_view.submodules", "子模块", "Submodules");
pub const REFRESH: UiMessage = UiMessage::new("views.submodule_view.refresh", "刷新", "Refresh");
pub const SYNC_RECORDED_REVISIONS: UiMessage = UiMessage::new("views.submodule_view.sync_recorded_revisions", "同步记录版本", "Sync recorded revisions");
pub const UPDATE_TO_REMOTE_LATEST: UiMessage = UiMessage::new("views.submodule_view.update_to_remote_latest", "更新到远端最新", "Update to remote latest");
pub const THE_LIST_IS_READ_ONLY_WHEN_OPENING_THIS_DIALOG_AHEAD: UiMessage = UiMessage::new("views.submodule_view.the_list_is_read_only_when_opening_this_dialog_ahead", "列表仅在打开弹窗时读取；正在后台检查子模块相对远端分支的超前/落后状态。", "The list is read only when opening this dialog. Ahead/behind status relative to remote branches is being checked in the background.");
pub const THE_LIST_IS_READ_ONLY_WHEN_OPENING_THIS_DIALOG_SYNC: UiMessage = UiMessage::new("views.submodule_view.the_list_is_read_only_when_opening_this_dialog_sync", "列表仅在打开弹窗时读取；同步记录版本会检出父仓库记录的提交，更新到远端最新会修改父仓库子模块指针，需要后续提交该变更。", "The list is read only when opening this dialog. Sync checks out the revision recorded by the parent repository. Updating to remote latest changes the parent repository’s submodule pointer, which must then be committed.");
pub const CLOSE: UiMessage = UiMessage::new("views.submodule_view.close", "关闭", "Close");
pub const PATH: UiMessage = UiMessage::new("views.submodule_view.path", "路径", "Path");
pub const STATUS: UiMessage = UiMessage::new("views.submodule_view.status", "状态", "Status");
pub const TARGET: UiMessage = UiMessage::new("views.submodule_view.target", "目标", "Target");
pub const CURRENT: UiMessage = UiMessage::new("views.submodule_view.current", "当前", "Current");
pub const ACTIONS: UiMessage = UiMessage::new("views.submodule_view.actions", "操作", "Actions");
pub const URL_NOT_CONFIGURED: UiMessage = UiMessage::new("views.submodule_view.url_not_configured", "未配置 URL", "URL not configured");
pub const UPDATE_LATEST: UiMessage = UiMessage::new("views.submodule_view.update_latest", "更新最新", "Update latest");
pub const UP_TO_DATE_WITH_REMOTE: UiMessage = UiMessage::new("views.submodule_view.up_to_date_with_remote", "远端同步", "Up to date with remote");
pub const REMOTE_UNKNOWN: UiMessage = UiMessage::new("views.submodule_view.remote_unknown", "远端未知", "Remote unknown");
pub const NOT_INITIALIZED: UiMessage = UiMessage::new("views.submodule_view.not_initialized", "未初始化", "Not initialized");
pub const NOT_CHECKED_OUT: UiMessage = UiMessage::new("views.submodule_view.not_checked_out", "未检出", "Not checked out");
pub const MODIFIED: UiMessage = UiMessage::new("views.submodule_view.modified", "有改动", "Modified");
pub const UNTRACKED_FILES: UiMessage = UiMessage::new("views.submodule_view.untracked_files", "有未跟踪文件", "Untracked files");




#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    FAILED_TO_LOAD_SUBMODULE_LIST_ERROR,
    BEHIND_BEHIND,
    AHEAD_AHEAD,
    DIVERGED_AHEAD_BEHIND,
    OPEN_A_REPOSITORY_FIRST,
    SUBMODULE_OPENED,
    LOADING_SUBMODULE_LIST,
    CHECKING_SUBMODULE_REMOTE_STATUS,
    SYNC_RECORDED_SUBMODULE_REVISIONS,
    UPDATE_SUBMODULES,
    LOADING_SUBMODULE_LIST_2,
    THIS_REPOSITORY_HAS_NO_SUBMODULES,
    SUBMODULES,
    REFRESH,
    SYNC_RECORDED_REVISIONS,
    UPDATE_TO_REMOTE_LATEST,
    THE_LIST_IS_READ_ONLY_WHEN_OPENING_THIS_DIALOG_AHEAD,
    THE_LIST_IS_READ_ONLY_WHEN_OPENING_THIS_DIALOG_SYNC,
    CLOSE,
    PATH,
    STATUS,
    TARGET,
    CURRENT,
    ACTIONS,
    URL_NOT_CONFIGURED,
    UPDATE_LATEST,
    UP_TO_DATE_WITH_REMOTE,
    REMOTE_UNKNOWN,
    NOT_INITIALIZED,
    NOT_CHECKED_OUT,
    MODIFIED,
    UNTRACKED_FILES,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
