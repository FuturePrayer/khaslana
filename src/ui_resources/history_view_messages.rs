//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const COMMIT_DETAILS_SUMMARY: UiMessage = UiMessage::new("views.history_view.commit_details_summary", "提交详情 · {summary}", "Commit details · {summary}");
pub const AUTHOR_COMMIT: UiMessage = UiMessage::new("views.history_view.author_commit", "作者 {commit}", "Author {commit}");
pub const COMMITTED_TIME: UiMessage = UiMessage::new("views.history_view.committed_time", "提交时间 {time}", "Committed {time}");
pub const COMMITTER_COMMITTER: UiMessage = UiMessage::new("views.history_view.committer_committer", "提交者 {committer}", "Committer {committer}");
pub const FILES_PATH: UiMessage = UiMessage::new("views.history_view.files_path", "文件：{path}", "Files: {path}");
pub const PARENT_COMMIT_PARENTS: UiMessage = UiMessage::new("views.history_view.parent_commit_parents", "父提交 {parents}", "Parent commit {parents}");
pub const PARENT_COMMITS_PARENTS_PARENTS_VALUE_MERGE_COMMIT: UiMessage = UiMessage::new("views.history_view.parent_commits_parents_parents_value_merge_commit", "父提交 {parents} / {parents_value}（合并提交）", "Parent commits {parents} / {parents_value} (merge commit)");
pub const COUNT_PARENT_COMMITS_OCTOPUS_MERGE: UiMessage = UiMessage::new("views.history_view.count_parent_commits_octopus_merge", "父提交 {count} 个（章鱼合并）", "{count} parent commits (octopus merge)");
pub const HIDDEN_REFERENCES_REFS_N_N: UiMessage = UiMessage::new("views.history_view.hidden_references_refs_n_n", "隐藏引用（{refs} 个）：\n{n}", "Hidden references ({refs}):\n{n}");
pub const NO_REPOSITORY_OPEN: UiMessage = UiMessage::new("views.history_view.no_repository_open", "还没有打开的仓库", "No repository open");
pub const OPEN_A_LOCAL_REPOSITORY_OR_CLONE_A_REMOTE_REPOSITORY_TO: UiMessage = UiMessage::new("views.history_view.open_a_local_repository_or_clone_a_remote_repository_to", "从本地磁盘打开现有仓库，或克隆一个远端仓库开始工作", "Open a local repository or clone a remote repository to get started");
pub const OPEN_REPOSITORY: UiMessage = UiMessage::new("views.history_view.open_repository", "打开仓库…", "Open repository…");
pub const CLONE_REPOSITORY: UiMessage = UiMessage::new("views.history_view.clone_repository", "克隆仓库…", "Clone repository…");
pub const COMMIT_GRAPH: UiMessage = UiMessage::new("views.history_view.commit_graph", "提交图谱", "Commit graph");
pub const BRANCH_TOPOLOGY_AND_LINEAGE_TRACKING: UiMessage = UiMessage::new("views.history_view.branch_topology_and_lineage_tracking", "分支拓扑与动向追踪", "Branch topology and lineage tracking");
pub const VIEW_COMMIT_GRAPH_FULL_SCREEN: UiMessage = UiMessage::new("views.history_view.view_commit_graph_full_screen", "全屏查看提交图谱", "View commit graph full screen");
pub const COMMIT_DETAILS: UiMessage = UiMessage::new("views.history_view.commit_details", "提交详情", "Commit details");
pub const SELECT_A_COMMIT_TO_INSPECT_ITS_DETAILS_FILES_AND_DIFFERENCES: UiMessage = UiMessage::new("views.history_view.select_a_commit_to_inspect_its_details_files_and_differences", "选择一个提交以检查详情、文件与差异", "Select a commit to inspect its details, files, and differences");
pub const EXPAND: UiMessage = UiMessage::new("views.history_view.expand", "展开", "Expand");
pub const COLLAPSE: UiMessage = UiMessage::new("views.history_view.collapse", "收起", "Collapse");
pub const COPY_SHA: UiMessage = UiMessage::new("views.history_view.copy_sha", "复制 SHA", "Copy SHA");
pub const COPY_MESSAGE: UiMessage = UiMessage::new("views.history_view.copy_message", "复制信息", "Copy message");
pub const COMMIT_MESSAGE_COPIED: UiMessage = UiMessage::new("views.history_view.commit_message_copied", "已复制提交信息", "Commit message copied");
pub const LOADING_COMMIT_FILES: UiMessage = UiMessage::new("views.history_view.loading_commit_files", "提交文件加载中...", "Loading commit files…");
pub const THIS_COMMIT_HAS_NO_FILE_CHANGES: UiMessage = UiMessage::new("views.history_view.this_commit_has_no_file_changes", "该提交没有文件变更", "This commit has no file changes");
pub const SELECT_A_COMMIT: UiMessage = UiMessage::new("views.history_view.select_a_commit", "请选择一个提交", "Select a commit");
pub const COMMIT_FILES: UiMessage = UiMessage::new("views.history_view.commit_files", "提交文件", "Commit files");
pub const COMMIT_DIFF: UiMessage = UiMessage::new("views.history_view.commit_diff", "提交差异", "Commit diff");
pub const LOADING_COMMIT_DIFFERENCES: UiMessage = UiMessage::new("views.history_view.loading_commit_differences", "提交差异加载中...", "Loading commit differences…");
pub const SELECT_A_COMMIT_FILE_TO_VIEW_ITS_DIFFERENCES: UiMessage = UiMessage::new("views.history_view.select_a_commit_file_to_view_its_differences", "请选择一个提交文件查看差异", "Select a commit file to view its differences");
pub const UNPUSHED: UiMessage = UiMessage::new("views.history_view.unpushed", "未推送", "Unpushed");
pub const ROOT_COMMIT_NO_PARENTS: UiMessage = UiMessage::new("views.history_view.root_commit_no_parents", "根提交（无父提交）", "Root commit (no parents)");
pub const LOCAL_BRANCHES: UiMessage = UiMessage::new("views.history_view.local_branches", "本地分支", "Local branches");
pub const REMOTE_BRANCHES: UiMessage = UiMessage::new("views.history_view.remote_branches", "远端分支", "Remote branches");
pub const TAGS: UiMessage = UiMessage::new("views.history_view.tags", "标签", "Tags");

pub const DIFF_TITLE: UiMessage = UiMessage::new("views.history_view.diff_title", "提交差异：{path}", "Commit diff: {path}");

pub const FULL_FILE_TITLE: UiMessage = UiMessage::new("views.history_view.full_file_title", "提交全文：{path}", "Commit full file: {path}");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    COMMIT_DETAILS_SUMMARY,
    AUTHOR_COMMIT,
    COMMITTED_TIME,
    COMMITTER_COMMITTER,
    FILES_PATH,
    PARENT_COMMIT_PARENTS,
    PARENT_COMMITS_PARENTS_PARENTS_VALUE_MERGE_COMMIT,
    COUNT_PARENT_COMMITS_OCTOPUS_MERGE,
    HIDDEN_REFERENCES_REFS_N_N,
    NO_REPOSITORY_OPEN,
    OPEN_A_LOCAL_REPOSITORY_OR_CLONE_A_REMOTE_REPOSITORY_TO,
    OPEN_REPOSITORY,
    CLONE_REPOSITORY,
    COMMIT_GRAPH,
    BRANCH_TOPOLOGY_AND_LINEAGE_TRACKING,
    VIEW_COMMIT_GRAPH_FULL_SCREEN,
    COMMIT_DETAILS,
    SELECT_A_COMMIT_TO_INSPECT_ITS_DETAILS_FILES_AND_DIFFERENCES,
    EXPAND,
    COLLAPSE,
    COPY_SHA,
    COPY_MESSAGE,
    COMMIT_MESSAGE_COPIED,
    LOADING_COMMIT_FILES,
    THIS_COMMIT_HAS_NO_FILE_CHANGES,
    SELECT_A_COMMIT,
    COMMIT_FILES,
    COMMIT_DIFF,
    LOADING_COMMIT_DIFFERENCES,
    SELECT_A_COMMIT_FILE_TO_VIEW_ITS_DIFFERENCES,
    UNPUSHED,
    ROOT_COMMIT_NO_PARENTS,
    LOCAL_BRANCHES,
    REMOTE_BRANCHES,
    TAGS,
    DIFF_TITLE,
    FULL_FILE_TITLE,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
