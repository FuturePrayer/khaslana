//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const COUNT_RESULTS_READING_PENDING_REPOSITORIES: UiMessage = UiMessage::new("views.code_palette_view.repositories.count_results_reading_pending_repositories", "{count} 个结果 · 正在读取 {pending} 个仓库", "{count} results · Reading {pending} repositories");
pub const COUNT_RESULTS_FAILED_REPOSITORIES_UNAVAILABLE: UiMessage = UiMessage::new("views.code_palette_view.repositories.count_results_failed_repositories_unavailable", "{count} 个结果 · {failed} 个仓库不可用", "{count} results · {failed} repositories unavailable");
pub const COUNT_RESULTS: UiMessage = UiMessage::new("views.code_palette_view.repositories.count_results", "{count} 个结果", "{count} results");
pub const NAME_PATH_OPEN_REPOSITORY_AND_LOCATE_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.repositories.name_path_open_repository_and_locate_branch", "{name} · {path} · 打开仓库并定位分支", "{name} · {path} · Open repository and locate branch");
pub const FAILED_TO_READ_BRANCHES_PANIC: UiMessage = UiMessage::new("views.code_palette_view.repositories.failed_to_read_branches_panic", "读取分支失败：{panic}", "Failed to read branches: {panic}");
pub const BRANCH_NO_LONGER_EXISTS_NAME: UiMessage = UiMessage::new("views.code_palette_view.repositories.branch_no_longer_exists_name", "分支已不存在：{name}", "Branch no longer exists: {name}");
pub const REPOSITORIES: UiMessage = UiMessage::new("views.code_palette_view.repositories.repositories", "仓库", "Repositories");
pub const PROJECT_REPOSITORY_REPO: UiMessage = UiMessage::new("views.code_palette_view.repositories.project_repository_repo", "项目 repository repo", "Project repository repo");
pub const REPOSITORY_DIRECTORY_UNAVAILABLE_CANNOT_READ_BRANCHES: UiMessage = UiMessage::new("views.code_palette_view.repositories.repository_directory_unavailable_cannot_read_branches", "仓库目录不可用，无法读取分支", "Repository directory unavailable; cannot read branches");
pub const LOCAL_BRANCHES: UiMessage = UiMessage::new("views.code_palette_view.repositories.local_branches", "本地分支", "Local branches");
pub const REMOTE_BRANCHES: UiMessage = UiMessage::new("views.code_palette_view.repositories.remote_branches", "远端分支", "Remote branches");
pub const BRANCH_BRANCH: UiMessage = UiMessage::new("views.code_palette_view.repositories.branch_branch", "branch 分支", "branch branch");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    COUNT_RESULTS_READING_PENDING_REPOSITORIES,
    COUNT_RESULTS_FAILED_REPOSITORIES_UNAVAILABLE,
    COUNT_RESULTS,
    NAME_PATH_OPEN_REPOSITORY_AND_LOCATE_BRANCH,
    FAILED_TO_READ_BRANCHES_PANIC,
    BRANCH_NO_LONGER_EXISTS_NAME,
    REPOSITORIES,
    PROJECT_REPOSITORY_REPO,
    REPOSITORY_DIRECTORY_UNAVAILABLE_CANNOT_READ_BRANCHES,
    LOCAL_BRANCHES,
    REMOTE_BRANCHES,
    BRANCH_BRANCH,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
