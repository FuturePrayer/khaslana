//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const COUNT_RESULTS: UiMessage = UiMessage::new("views.code_palette_view.count_results", "{count} 个结果", "{count} results");
pub const REPOSITORIES: UiMessage = UiMessage::new("views.code_palette_view.repositories", "仓库", "Repositories");
pub const PROJECT_REPOSITORY_REPO: UiMessage = UiMessage::new("views.code_palette_view.project_repository_repo", "项目 repository repo", "Project repository repo");
pub const WORKFLOWS: UiMessage = UiMessage::new("views.code_palette_view.workflows", "工作流", "Workflows");
pub const TEMPLATE_WORKFLOW_TEMPLATE: UiMessage = UiMessage::new("views.code_palette_view.template_workflow_template", "模板 workflow template", "Template workflow template");
pub const AN_OPERATION_IS_IN_PROGRESS_CANNOT_SWITCH_REPOSITORIES: UiMessage = UiMessage::new("views.code_palette_view.an_operation_is_in_progress_cannot_switch_repositories", "当前操作进行中，暂不可切换仓库", "An operation is in progress. Cannot switch repositories.");
pub const AN_OPERATION_IS_IN_PROGRESS_CANNOT_OPEN_ANOTHER_REPOSITORY: UiMessage = UiMessage::new("views.code_palette_view.an_operation_is_in_progress_cannot_open_another_repository", "当前操作进行中，暂不可打开其他仓库", "An operation is in progress. Cannot open another repository.");
pub const CURRENT_HEAD: UiMessage = UiMessage::new("views.code_palette_view.current_head", "当前 HEAD", "Current HEAD");
pub const NO_MATCHING_RESULTS_TRY_A_BRANCH_NAME_COMMAND_NAME_OR: UiMessage = UiMessage::new("views.code_palette_view.no_matching_results_try_a_branch_name_command_name_or", "没有匹配结果，请尝试分支名称、功能名称或其他关键词。", "No matching results. Try a branch name, command name, or another keyword.");
pub const GLOBAL_SEARCH: UiMessage = UiMessage::new("views.code_palette_view.global_search", "全局搜索", "Global search");
pub const SEARCH_BRANCHES_COMMANDS_SETTINGS_REPOSITORIES_AND_WORKFLOWS: UiMessage = UiMessage::new("views.code_palette_view.search_branches_commands_settings_repositories_and_workflows", "搜索分支、功能、设置、仓库和工作流", "Search branches, commands, settings, repositories, and workflows");
pub const CLOSE_GLOBAL_SEARCH: UiMessage = UiMessage::new("views.code_palette_view.close_global_search", "关闭全局搜索", "Close global search");
pub const SEARCH_ALL_REPOSITORIES: UiMessage = UiMessage::new("views.code_palette_view.search_all_repositories", "搜索全部仓库", "Search all repositories");
pub const SEARCH_CURRENT_REPOSITORY_ONLY: UiMessage = UiMessage::new("views.code_palette_view.search_current_repository_only", "仅搜索当前仓库", "Search current repository only");
pub const SELECT_ENTER_GO_TO_ESC_CLOSE: UiMessage = UiMessage::new("views.code_palette_view.select_enter_go_to_esc_close", "↑↓ 选择 · Enter 定位 · Esc 关闭", "↑↓ Select · Enter Go to · Esc Close");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    COUNT_RESULTS,
    REPOSITORIES,
    PROJECT_REPOSITORY_REPO,
    WORKFLOWS,
    TEMPLATE_WORKFLOW_TEMPLATE,
    AN_OPERATION_IS_IN_PROGRESS_CANNOT_SWITCH_REPOSITORIES,
    AN_OPERATION_IS_IN_PROGRESS_CANNOT_OPEN_ANOTHER_REPOSITORY,
    CURRENT_HEAD,
    NO_MATCHING_RESULTS_TRY_A_BRANCH_NAME_COMMAND_NAME_OR,
    GLOBAL_SEARCH,
    SEARCH_BRANCHES_COMMANDS_SETTINGS_REPOSITORIES_AND_WORKFLOWS,
    CLOSE_GLOBAL_SEARCH,
    SEARCH_ALL_REPOSITORIES,
    SEARCH_CURRENT_REPOSITORY_ONLY,
    SELECT_ENTER_GO_TO_ESC_CLOSE,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
