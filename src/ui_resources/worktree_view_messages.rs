//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const COMMIT_TO_TRUNCATED: UiMessage = UiMessage::new("views.worktree_view.commit_to_truncated", "提交到 {truncated}…", "Commit to {truncated}…");
pub const COMMIT_TO_TRUNCATED_2: UiMessage = UiMessage::new("views.worktree_view.commit_to_truncated_2", "提交到 {truncated}", "Commit to {truncated}");
pub const CONFLICT_COUNT_CONFLICT_FILES_REMAIN_RESOLVE_THEM_IN_THE_CONFLICT: UiMessage = UiMessage::new("views.worktree_view.conflict_count_conflict_files_remain_resolve_them_in_the_conflict", "存在 {conflict_count} 个冲突文件，请在冲突工作台中处理", "{conflict_count} conflict files remain. Resolve them in the conflict workbench.");
pub const COMMIT_TO_CURRENT_BRANCH: UiMessage = UiMessage::new("views.worktree_view.commit_to_current_branch", "提交到当前分支", "Commit to current branch");
pub const LOADING_UNSTAGED_CHANGES: UiMessage = UiMessage::new("views.worktree_view.loading_unstaged_changes", "修改区加载中...", "Loading unstaged changes…");
pub const ALL_UNSTAGED_CHANGES_HAVE_BEEN_STAGED: UiMessage = UiMessage::new("views.worktree_view.all_unstaged_changes_have_been_staged", "未暂存变更已全部暂存", "All unstaged changes have been staged");
pub const WORKING_TREE_CLEAN_NO_CHANGES_TO_COMMIT: UiMessage = UiMessage::new("views.worktree_view.working_tree_clean_no_changes_to_commit", "工作区干净，没有待提交的改动", "Working tree clean; no changes to commit");
pub const NO_REPOSITORY_OPEN: UiMessage = UiMessage::new("views.worktree_view.no_repository_open", "还没有打开的仓库", "No repository open");
pub const OPEN_A_LOCAL_REPOSITORY_OR_CLONE_A_REMOTE_REPOSITORY_TO: UiMessage = UiMessage::new("views.worktree_view.open_a_local_repository_or_clone_a_remote_repository_to", "从本地磁盘打开现有仓库，或克隆一个远端仓库开始工作", "Open a local repository or clone a remote repository to get started");
pub const OPEN_REPOSITORY: UiMessage = UiMessage::new("views.worktree_view.open_repository", "打开仓库…", "Open repository…");
pub const CLONE_REPOSITORY: UiMessage = UiMessage::new("views.worktree_view.clone_repository", "克隆仓库…", "Clone repository…");
pub const STAGED_CHANGES: UiMessage = UiMessage::new("views.worktree_view.staged_changes", "已暂存变更", "Staged changes");
pub const LOADING_STAGED_CHANGES: UiMessage = UiMessage::new("views.worktree_view.loading_staged_changes", "暂存区加载中...", "Loading staged changes…");
pub const UNSTAGE_ALL: UiMessage = UiMessage::new("views.worktree_view.unstage_all", "取消暂存全部", "Unstage all");
pub const UNSTAGED_CHANGES: UiMessage = UiMessage::new("views.worktree_view.unstaged_changes", "未暂存变更", "Unstaged changes");
pub const STAGE_ALL: UiMessage = UiMessage::new("views.worktree_view.stage_all", "暂存全部", "Stage all");
pub const DISCARD_ALL: UiMessage = UiMessage::new("views.worktree_view.discard_all", "丢弃全部", "Discard all");
pub const NO_STAGED_CHANGES: UiMessage = UiMessage::new("views.worktree_view.no_staged_changes", "暂无已暂存变更", "No staged changes");
pub const UNSTAGE_THIS_FILE: UiMessage = UiMessage::new("views.worktree_view.unstage_this_file", "取消暂存此文件", "Unstage this file");
pub const STAGE_THIS_FILE: UiMessage = UiMessage::new("views.worktree_view.stage_this_file", "暂存此文件", "Stage this file");
pub const DIFF: UiMessage = UiMessage::new("views.worktree_view.diff", "差异", "Diff");
pub const SELECT_A_CHANGED_FILE_TO_VIEW_ITS_DIFFERENCES: UiMessage = UiMessage::new("views.worktree_view.select_a_changed_file_to_view_its_differences", "请选择一个变更文件查看差异", "Select a changed file to view its differences");
pub const AMEND_COMMIT: UiMessage = UiMessage::new("views.worktree_view.amend_commit", "修补提交", "Amend commit");
pub const AMEND_AND_PUSH: UiMessage = UiMessage::new("views.worktree_view.amend_and_push", "修补提交并推送", "Amend and push");
pub const COMMIT_AND_PUSH: UiMessage = UiMessage::new("views.worktree_view.commit_and_push", "提交并推送", "Commit and push");
pub const ABORT_MERGE: UiMessage = UiMessage::new("views.worktree_view.abort_merge", "中止合并", "Abort merge");
pub const COMMIT_MESSAGE: UiMessage = UiMessage::new("views.worktree_view.commit_message", "提交信息", "Commit message");
pub const AMEND_LAST_COMMIT: UiMessage = UiMessage::new("views.worktree_view.amend_last_commit", "修补上次提交", "Amend last commit");

pub const DIFF_TITLE: UiMessage = UiMessage::new("views.worktree_view.diff_title", "差异：{path} ({scope})", "Diff: {path} ({scope})");

pub const FULL_FILE_TITLE: UiMessage = UiMessage::new("views.worktree_view.full_file_title", "全文：{path} ({scope})", "Full file: {path} ({scope})");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    COMMIT_TO_TRUNCATED,
    COMMIT_TO_TRUNCATED_2,
    CONFLICT_COUNT_CONFLICT_FILES_REMAIN_RESOLVE_THEM_IN_THE_CONFLICT,
    COMMIT_TO_CURRENT_BRANCH,
    LOADING_UNSTAGED_CHANGES,
    ALL_UNSTAGED_CHANGES_HAVE_BEEN_STAGED,
    WORKING_TREE_CLEAN_NO_CHANGES_TO_COMMIT,
    NO_REPOSITORY_OPEN,
    OPEN_A_LOCAL_REPOSITORY_OR_CLONE_A_REMOTE_REPOSITORY_TO,
    OPEN_REPOSITORY,
    CLONE_REPOSITORY,
    STAGED_CHANGES,
    LOADING_STAGED_CHANGES,
    UNSTAGE_ALL,
    UNSTAGED_CHANGES,
    STAGE_ALL,
    DISCARD_ALL,
    NO_STAGED_CHANGES,
    UNSTAGE_THIS_FILE,
    STAGE_THIS_FILE,
    DIFF,
    SELECT_A_CHANGED_FILE_TO_VIEW_ITS_DIFFERENCES,
    AMEND_COMMIT,
    AMEND_AND_PUSH,
    COMMIT_AND_PUSH,
    ABORT_MERGE,
    COMMIT_MESSAGE,
    AMEND_LAST_COMMIT,
    DIFF_TITLE,
    FULL_FILE_TITLE,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
