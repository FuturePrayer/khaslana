//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const WORKING_TREE: UiMessage = UiMessage::new("views.chrome_view.working_tree", "工作区", "Working tree");
pub const CONFLICTS: UiMessage = UiMessage::new("views.chrome_view.conflicts", "冲突处理", "Conflicts");
pub const HISTORY: UiMessage = UiMessage::new("views.chrome_view.history", "提交记录", "History");
pub const WORKFLOWS: UiMessage = UiMessage::new("views.chrome_view.workflows", "工作流", "Workflows");
pub const STASHES: UiMessage = UiMessage::new("views.chrome_view.stashes", "贮藏", "Stashes");
pub const BROWSE_BRANCH: UiMessage = UiMessage::new("views.chrome_view.browse_branch", "分支浏览", "Browse branch");
pub const ATTRIBUTION: UiMessage = UiMessage::new("views.chrome_view.attribution", "追溯", "Attribution");
pub const COMMIT_GRAPH: UiMessage = UiMessage::new("views.chrome_view.commit_graph", "提交图谱", "Commit graph");
pub const AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT: UiMessage = UiMessage::new("views.chrome_view.an_operation_is_in_progress_please_wait", "当前操作进行中，请稍候", "An operation is in progress. Please wait.");
pub const OPEN_A_REPOSITORY_FIRST: UiMessage = UiMessage::new("views.chrome_view.open_a_repository_first", "请先打开仓库", "Open a repository first");
pub const THIS_REPOSITORY_HAS_NO_AVAILABLE_REMOTE: UiMessage = UiMessage::new("views.chrome_view.this_repository_has_no_available_remote", "当前仓库没有可用远端", "This repository has no available remote");
pub const A_MERGE_IS_IN_PROGRESS_COMPLETE_OR_ABORT_IT_FIRST: UiMessage = UiMessage::new("views.chrome_view.a_merge_is_in_progress_complete_or_abort_it_first", "合并进行中，完成或中止合并后再操作", "A merge is in progress. Complete or abort it first.");
pub const REFRESH: UiMessage = UiMessage::new("views.chrome_view.refresh", "刷新", "Refresh");
pub const FETCH: UiMessage = UiMessage::new("views.chrome_view.fetch", "获取", "Fetch");
pub const PULL: UiMessage = UiMessage::new("views.chrome_view.pull", "拉取", "Pull");
pub const PUSH: UiMessage = UiMessage::new("views.chrome_view.push", "推送", "Push");
pub const SUBMODULES: UiMessage = UiMessage::new("views.chrome_view.submodules", "子模块", "Submodules");
pub const GLOBAL_SEARCH_BRANCHES_COMMANDS_SETTINGS_AND_REPOSITORIES: UiMessage = UiMessage::new("views.chrome_view.global_search_branches_commands_settings_and_repositories", "全局搜索：分支、功能、设置和仓库", "Global search: branches, commands, settings, and repositories");
pub const GLOBAL_SEARCH_BRANCHES_COMMANDS_SETTINGS_AND_REPOSITORIES_CTRL_P: UiMessage = UiMessage::new("views.chrome_view.global_search_branches_commands_settings_and_repositories_ctrl_p", "全局搜索：分支、功能、设置和仓库（Ctrl+P）", "Global search: branches, commands, settings, and repositories (Ctrl+P)");
pub const SEARCH: UiMessage = UiMessage::new("views.chrome_view.search", "搜索", "Search");
pub const SEARCH_BRANCHES_OR_COMMANDS: UiMessage = UiMessage::new("views.chrome_view.search_branches_or_commands", "搜索分支或功能", "Search branches or commands");
pub const COLLAPSE_CONTEXT_NAVIGATION: UiMessage = UiMessage::new("views.chrome_view.collapse_context_navigation", "收起上下文导航", "Collapse context navigation");
pub const EXPAND_CONTEXT_NAVIGATION: UiMessage = UiMessage::new("views.chrome_view.expand_context_navigation", "展开上下文导航", "Expand context navigation");
pub const SETTINGS: UiMessage = UiMessage::new("views.chrome_view.settings", "设置", "Settings");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    WORKING_TREE,
    CONFLICTS,
    HISTORY,
    WORKFLOWS,
    STASHES,
    BROWSE_BRANCH,
    ATTRIBUTION,
    COMMIT_GRAPH,
    AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT,
    OPEN_A_REPOSITORY_FIRST,
    THIS_REPOSITORY_HAS_NO_AVAILABLE_REMOTE,
    A_MERGE_IS_IN_PROGRESS_COMPLETE_OR_ABORT_IT_FIRST,
    REFRESH,
    FETCH,
    PULL,
    PUSH,
    SUBMODULES,
    GLOBAL_SEARCH_BRANCHES_COMMANDS_SETTINGS_AND_REPOSITORIES,
    GLOBAL_SEARCH_BRANCHES_COMMANDS_SETTINGS_AND_REPOSITORIES_CTRL_P,
    SEARCH,
    SEARCH_BRANCHES_OR_COMMANDS,
    COLLAPSE_CONTEXT_NAVIGATION,
    EXPAND_CONTEXT_NAVIGATION,
    SETTINGS,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
