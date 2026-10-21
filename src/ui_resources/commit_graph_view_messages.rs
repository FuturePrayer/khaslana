//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const HIGHLIGHT_BRANCH: UiMessage = UiMessage::new("views.commit_graph_view.highlight_branch", "高亮：{branch} ▾", "Highlight: {branch} ▾");
pub const FIRST_COMMIT_TRACE_OID_LIMIT_COMMITS_HIGHLIGHTED: UiMessage = UiMessage::new("views.commit_graph_view.first_commit_trace_oid_limit_commits_highlighted", "已高亮前 {COMMIT_TRACE_OID_LIMIT} 个提交", "First {COMMIT_TRACE_OID_LIMIT} commits highlighted");
pub const COMMIT_DETAILS_SUMMARY: UiMessage = UiMessage::new("views.commit_graph_view.commit_details_summary", "提交详情 · {summary}", "Commit details · {summary}");
pub const AUTHOR_COMMIT: UiMessage = UiMessage::new("views.commit_graph_view.author_commit", "作者 {commit}", "Author {commit}");
pub const COMMITTED_TIME: UiMessage = UiMessage::new("views.commit_graph_view.committed_time", "提交时间 {time}", "Committed {time}");
pub const COMMITTER_COMMITTER: UiMessage = UiMessage::new("views.commit_graph_view.committer_committer", "提交者 {committer}", "Committer {committer}");
pub const COMMIT_GRAPH: UiMessage = UiMessage::new("views.commit_graph_view.commit_graph", "提交图谱", "Commit graph");
pub const BRANCH_TOPOLOGY_AND_LINEAGE_TRACKING: UiMessage = UiMessage::new("views.commit_graph_view.branch_topology_and_lineage_tracking", "分支拓扑与动向追踪", "Branch topology and lineage tracking");
pub const CLOSE: UiMessage = UiMessage::new("views.commit_graph_view.close", "关闭", "Close");
pub const BRANCH_HIGHLIGHT: UiMessage = UiMessage::new("views.commit_graph_view.branch_highlight", "分支高亮 ▾", "Branch highlight ▾");
pub const COMPUTING_BRANCH_LINEAGE: UiMessage = UiMessage::new("views.commit_graph_view.computing_branch_lineage", "正在计算分支谱系...", "Computing branch lineage…");
pub const SELECT_A_LOCAL_BRANCH_TO_TRACK_COMMITS_IN_ITS_LINEAGE: UiMessage = UiMessage::new("views.commit_graph_view.select_a_local_branch_to_track_commits_in_its_lineage", "选择要追踪动向的本地分支；谱系内提交保持高亮，其余淡化", "Select a local branch to track. Commits in its lineage stay highlighted; others are dimmed.");
pub const CURRENT_BRANCH: UiMessage = UiMessage::new("views.commit_graph_view.current_branch", "当前分支", "Current branch");
pub const ALL_BRANCHES: UiMessage = UiMessage::new("views.commit_graph_view.all_branches", "所有分支", "All branches");
pub const SELECT_A_BRANCH_HIGHLIGHT_FIRST: UiMessage = UiMessage::new("views.commit_graph_view.select_a_branch_highlight_first", "先选择分支高亮后可用", "Select a branch highlight first");
pub const ONLY_AHEAD_OF_HEAD: UiMessage = UiMessage::new("views.commit_graph_view.only_ahead_of_head", "仅领先 HEAD", "Only ahead of HEAD");
pub const DIM_MERGE_COMMITS: UiMessage = UiMessage::new("views.commit_graph_view.dim_merge_commits", "淡化合并提交", "Dim merge commits");
pub const COMPUTING_LINEAGE: UiMessage = UiMessage::new("views.commit_graph_view.computing_lineage", "谱系计算中...", "Computing lineage…");
pub const LOCAL_BRANCHES: UiMessage = UiMessage::new("views.commit_graph_view.local_branches", "本地分支", "Local branches");
pub const REMOTE_BRANCHES: UiMessage = UiMessage::new("views.commit_graph_view.remote_branches", "远端分支", "Remote branches");
pub const LOADING_REMOTE_BRANCHES: UiMessage = UiMessage::new("views.commit_graph_view.loading_remote_branches", "远端分支加载中...", "Loading remote branches…");
pub const NO_MATCHING_BRANCHES: UiMessage = UiMessage::new("views.commit_graph_view.no_matching_branches", "没有匹配的分支", "No matching branches");
pub const CLEAR_HIGHLIGHT: UiMessage = UiMessage::new("views.commit_graph_view.clear_highlight", "关闭高亮", "Clear highlight");
pub const LOADING_COMMIT_HISTORY: UiMessage = UiMessage::new("views.commit_graph_view.loading_commit_history", "提交记录加载中...", "Loading commit history…");
pub const OPEN_A_REPOSITORY_FIRST: UiMessage = UiMessage::new("views.commit_graph_view.open_a_repository_first", "请先打开一个仓库", "Open a repository first");
pub const NO_MATCHING_COMMITS: UiMessage = UiMessage::new("views.commit_graph_view.no_matching_commits", "没有匹配的提交", "No matching commits");
pub const LOADING: UiMessage = UiMessage::new("views.commit_graph_view.loading", "加载中...", "Loading…");
pub const LOAD_MORE: UiMessage = UiMessage::new("views.commit_graph_view.load_more", "加载更多", "Load more");
pub const COMMIT_DETAILS: UiMessage = UiMessage::new("views.commit_graph_view.commit_details", "提交详情", "Commit details");
pub const SELECT_A_LANE_ROW_TO_VIEW_FULL_COMMIT_DETAILS: UiMessage = UiMessage::new("views.commit_graph_view.select_a_lane_row_to_view_full_commit_details", "点击泳道行选中提交，查看完整信息", "Select a lane row to view full commit details");
pub const EXPAND: UiMessage = UiMessage::new("views.commit_graph_view.expand", "展开", "Expand");
pub const COLLAPSE: UiMessage = UiMessage::new("views.commit_graph_view.collapse", "收起", "Collapse");
pub const VIEW_IN_HISTORY: UiMessage = UiMessage::new("views.commit_graph_view.view_in_history", "在提交记录页查看", "View in History");
pub const COPY_SHA: UiMessage = UiMessage::new("views.commit_graph_view.copy_sha", "复制 SHA", "Copy SHA");
pub const COPY_MESSAGE: UiMessage = UiMessage::new("views.commit_graph_view.copy_message", "复制信息", "Copy message");
pub const COMMIT_MESSAGE_COPIED: UiMessage = UiMessage::new("views.commit_graph_view.commit_message_copied", "已复制提交信息", "Commit message copied");
pub const NO_REFERENCE_LABELS: UiMessage = UiMessage::new("views.commit_graph_view.no_reference_labels", "无引用标签", "No reference labels");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    HIGHLIGHT_BRANCH,
    FIRST_COMMIT_TRACE_OID_LIMIT_COMMITS_HIGHLIGHTED,
    COMMIT_DETAILS_SUMMARY,
    AUTHOR_COMMIT,
    COMMITTED_TIME,
    COMMITTER_COMMITTER,
    COMMIT_GRAPH,
    BRANCH_TOPOLOGY_AND_LINEAGE_TRACKING,
    CLOSE,
    BRANCH_HIGHLIGHT,
    COMPUTING_BRANCH_LINEAGE,
    SELECT_A_LOCAL_BRANCH_TO_TRACK_COMMITS_IN_ITS_LINEAGE,
    CURRENT_BRANCH,
    ALL_BRANCHES,
    SELECT_A_BRANCH_HIGHLIGHT_FIRST,
    ONLY_AHEAD_OF_HEAD,
    DIM_MERGE_COMMITS,
    COMPUTING_LINEAGE,
    LOCAL_BRANCHES,
    REMOTE_BRANCHES,
    LOADING_REMOTE_BRANCHES,
    NO_MATCHING_BRANCHES,
    CLEAR_HIGHLIGHT,
    LOADING_COMMIT_HISTORY,
    OPEN_A_REPOSITORY_FIRST,
    NO_MATCHING_COMMITS,
    LOADING,
    LOAD_MORE,
    COMMIT_DETAILS,
    SELECT_A_LANE_ROW_TO_VIEW_FULL_COMMIT_DETAILS,
    EXPAND,
    COLLAPSE,
    VIEW_IN_HISTORY,
    COPY_SHA,
    COPY_MESSAGE,
    COMMIT_MESSAGE_COPIED,
    NO_REFERENCE_LABELS,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
