//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const UP_PUSH_AHEAD_PULL_BEHIND: UiMessage = UiMessage::new("views.sidebar_view.up_push_ahead_pull_behind", "→{up} · 待推送 {ahead} · 待拉取 {behind}", "→{up} · Push {ahead} · Pull {behind}");
pub const AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT: UiMessage = UiMessage::new("views.sidebar_view.an_operation_is_in_progress_please_wait", "当前操作进行中，请稍候", "An operation is in progress. Please wait.");
pub const OPEN_A_REPOSITORY_FIRST: UiMessage = UiMessage::new("views.sidebar_view.open_a_repository_first", "请先打开仓库", "Open a repository first");
pub const NO_MATCHING_LOCAL_BRANCHES: UiMessage = UiMessage::new("views.sidebar_view.no_matching_local_branches", "没有匹配的本地分支", "No matching local branches");
pub const NO_MATCHING_REMOTE_BRANCHES: UiMessage = UiMessage::new("views.sidebar_view.no_matching_remote_branches", "没有匹配的远端分支", "No matching remote branches");
pub const LOADING_REMOTES: UiMessage = UiMessage::new("views.sidebar_view.loading_remotes", "远端加载中...", "Loading remotes…");
pub const LOADING_REMOTE_BRANCHES: UiMessage = UiMessage::new("views.sidebar_view.loading_remote_branches", "远端分支加载中...", "Loading remote branches…");
pub const LOCAL_BRANCHES: UiMessage = UiMessage::new("views.sidebar_view.local_branches", "本地分支", "Local branches");
pub const REMOTES: UiMessage = UiMessage::new("views.sidebar_view.remotes", "远端", "Remotes");
pub const REMOTE_BRANCHES: UiMessage = UiMessage::new("views.sidebar_view.remote_branches", "远端分支", "Remote branches");
pub const TAGS: UiMessage = UiMessage::new("views.sidebar_view.tags", "标签", "Tags");
pub const STASHES: UiMessage = UiMessage::new("views.sidebar_view.stashes", "贮藏", "Stashes");
pub const MANAGE: UiMessage = UiMessage::new("views.sidebar_view.manage", "管理", "Manage");
pub const REFRESH: UiMessage = UiMessage::new("views.sidebar_view.refresh", "刷新", "Refresh");
pub const COPY_REMOTE_NAME: UiMessage = UiMessage::new("views.sidebar_view.copy_remote_name", "复制远端名", "Copy remote name");
pub const RUN_WORKFLOW: UiMessage = UiMessage::new("views.sidebar_view.run_workflow", "运行工作流…", "Run workflow…");
pub const COPY_BRANCH_NAME: UiMessage = UiMessage::new("views.sidebar_view.copy_branch_name", "复制分支名", "Copy branch name");
pub const COPY_CHECKOUT_COMMAND: UiMessage = UiMessage::new("views.sidebar_view.copy_checkout_command", "复制 checkout 命令", "Copy checkout command");
pub const SWITCH_TO_THIS_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.switch_to_this_branch", "切换到此分支", "Switch to this branch");
pub const PULL_BRANCH_UPDATES: UiMessage = UiMessage::new("views.sidebar_view.pull_branch_updates", "拉取此分支更新", "Pull branch updates");
pub const MERGE_INTO_CURRENT_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.merge_into_current_branch", "合并到当前分支", "Merge into current branch");
pub const REBASE_ONTO_CURRENT_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.rebase_onto_current_branch", "变基到当前分支", "Rebase onto current branch");
pub const FETCH_LOCALLY_AND_CHECKOUT: UiMessage = UiMessage::new("views.sidebar_view.fetch_locally_and_checkout", "拉取到本地并切换", "Fetch locally and checkout");
pub const SET_CHANGE_UPSTREAM: UiMessage = UiMessage::new("views.sidebar_view.set_change_upstream", "设置/修改 upstream...", "Set/change upstream…");
pub const RENAME: UiMessage = UiMessage::new("views.sidebar_view.rename", "重命名...", "Rename…");
pub const BROWSE_THIS_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.browse_this_branch", "浏览此分支", "Browse this branch");
pub const COMPARE_WITH_CURRENT_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.compare_with_current_branch", "与当前分支比较", "Compare with current branch");
pub const DELETE_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.delete_branch", "删除分支", "Delete branch");
pub const DELETE_REMOTE_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.delete_remote_branch", "删除远端分支", "Delete remote branch");
pub const LOCAL: UiMessage = UiMessage::new("views.sidebar_view.local", "本地", "Local");
pub const BACK_TO_BRANCH_MENU: UiMessage = UiMessage::new("views.sidebar_view.back_to_branch_menu", "← 返回分支菜单", "← Back to branch menu");
pub const PULL_BRANCH: UiMessage = UiMessage::new("views.sidebar_view.pull_branch", "拉取分支", "Pull branch");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    UP_PUSH_AHEAD_PULL_BEHIND,
    AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT,
    OPEN_A_REPOSITORY_FIRST,
    NO_MATCHING_LOCAL_BRANCHES,
    NO_MATCHING_REMOTE_BRANCHES,
    LOADING_REMOTES,
    LOADING_REMOTE_BRANCHES,
    LOCAL_BRANCHES,
    REMOTES,
    REMOTE_BRANCHES,
    TAGS,
    STASHES,
    MANAGE,
    REFRESH,
    COPY_REMOTE_NAME,
    RUN_WORKFLOW,
    COPY_BRANCH_NAME,
    COPY_CHECKOUT_COMMAND,
    SWITCH_TO_THIS_BRANCH,
    PULL_BRANCH_UPDATES,
    MERGE_INTO_CURRENT_BRANCH,
    REBASE_ONTO_CURRENT_BRANCH,
    FETCH_LOCALLY_AND_CHECKOUT,
    SET_CHANGE_UPSTREAM,
    RENAME,
    BROWSE_THIS_BRANCH,
    COMPARE_WITH_CURRENT_BRANCH,
    DELETE_BRANCH,
    DELETE_REMOTE_BRANCH,
    LOCAL,
    BACK_TO_BRANCH_MENU,
    PULL_BRANCH,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
