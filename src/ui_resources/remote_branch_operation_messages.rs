//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const PULL: UiMessage = UiMessage::new("views.remote_branch_operation.pull", "拉取", "Pull");
pub const PUSH: UiMessage = UiMessage::new("views.remote_branch_operation.push", "推送", "Push");
pub const SET_UPSTREAM: UiMessage = UiMessage::new("views.remote_branch_operation.set_upstream", "设置 upstream", "Set upstream");
pub const SETTINGS: UiMessage = UiMessage::new("views.remote_branch_operation.settings", "设置", "Settings");
pub const PULL_FETCHES_THE_SELECTED_REMOTE_THEN_MERGES_THE_REMOTE_BRANCH: UiMessage = UiMessage::new("views.remote_branch_operation.pull_fetches_the_selected_remote_then_merges_the_remote_branch", "拉取会先获取所选远端，然后把远端分支合并到当前本地分支。远端分支必须已存在。", "Pull fetches the selected remote, then merges the remote branch into the current local branch. The remote branch must already exist.");
pub const PUSH_SENDS_THE_CURRENT_LOCAL_BRANCH_TO_THE_SPECIFIED_REMOTE: UiMessage = UiMessage::new("views.remote_branch_operation.push_sends_the_current_local_branch_to_the_specified_remote", "推送会把当前本地分支推送到填写的远程分支；远端分支不存在时会自动创建，并设置为上游分支。", "Push sends the current local branch to the specified remote branch, creating it if needed, and sets it as upstream.");
pub const UPSTREAM_IS_USED_TO_CALCULATE_PULL_AND_PUSH_STATUS_RELATIVE: UiMessage = UiMessage::new("views.remote_branch_operation.upstream_is_used_to_calculate_pull_and_push_status_relative", "upstream 用来计算本地分支相对远端分支的拉取和推送状态；只能选择已存在的远端分支。", "Upstream is used to calculate pull and push status relative to a remote branch. Select an existing remote branch.");
pub const NOT_ON_A_LOCAL_BRANCH_CANNOT_PULL_OR_PUSH: UiMessage = UiMessage::new("views.remote_branch_operation.not_on_a_local_branch_cannot_pull_or_push", "当前不是本地分支，无法拉取或推送", "Not on a local branch; cannot pull or push");
pub const THIS_REPOSITORY_HAS_NO_REMOTE: UiMessage = UiMessage::new("views.remote_branch_operation.this_repository_has_no_remote", "当前仓库没有远端", "This repository has no remote");
pub const NO_LOCAL_BRANCH: UiMessage = UiMessage::new("views.remote_branch_operation.no_local_branch", "无本地分支", "No local branch");
pub const REMOTE_BRANCH_DOES_NOT_EXIST_REFRESH_OR_CHOOSE_AN_EXISTING: UiMessage = UiMessage::new("views.remote_branch_operation.remote_branch_does_not_exist_refresh_or_choose_an_existing", "远端分支不存在，请点击刷新或选择已有分支", "Remote branch does not exist. Refresh or choose an existing branch.");
pub const CURRENT_LOCAL_BRANCH: UiMessage = UiMessage::new("views.remote_branch_operation.current_local_branch", "当前本地分支", "Current local branch");
pub const REFRESH: UiMessage = UiMessage::new("views.remote_branch_operation.refresh", "刷新", "Refresh");
pub const REMOTE_BRANCH: UiMessage = UiMessage::new("views.remote_branch_operation.remote_branch", "远程分支", "Remote branch");
pub const REBASE_INSTEAD_OF_MERGING: UiMessage = UiMessage::new("views.remote_branch_operation.rebase_instead_of_merging", "用变基代替合并", "Rebase instead of merging");
pub const CANCEL: UiMessage = UiMessage::new("views.remote_branch_operation.cancel", "取消", "Cancel");
pub const SELECT_REMOTE: UiMessage = UiMessage::new("views.remote_branch_operation.select_remote", "选择远端", "Select remote");
pub const REMOTES: UiMessage = UiMessage::new("views.remote_branch_operation.remotes", "远端", "Remotes");
pub const NO_REMOTE_BRANCHES_REFRESH_TO_FETCH_THEM: UiMessage = UiMessage::new("views.remote_branch_operation.no_remote_branches_refresh_to_fetch_them", "暂无远端分支，请点击刷新获取", "No remote branches. Refresh to fetch them.");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    PULL,
    PUSH,
    SET_UPSTREAM,
    SETTINGS,
    PULL_FETCHES_THE_SELECTED_REMOTE_THEN_MERGES_THE_REMOTE_BRANCH,
    PUSH_SENDS_THE_CURRENT_LOCAL_BRANCH_TO_THE_SPECIFIED_REMOTE,
    UPSTREAM_IS_USED_TO_CALCULATE_PULL_AND_PUSH_STATUS_RELATIVE,
    NOT_ON_A_LOCAL_BRANCH_CANNOT_PULL_OR_PUSH,
    THIS_REPOSITORY_HAS_NO_REMOTE,
    NO_LOCAL_BRANCH,
    REMOTE_BRANCH_DOES_NOT_EXIST_REFRESH_OR_CHOOSE_AN_EXISTING,
    CURRENT_LOCAL_BRANCH,
    REFRESH,
    REMOTE_BRANCH,
    REBASE_INSTEAD_OF_MERGING,
    CANCEL,
    SELECT_REMOTE,
    REMOTES,
    NO_REMOTE_BRANCHES_REFRESH_TO_FETCH_THEM,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
