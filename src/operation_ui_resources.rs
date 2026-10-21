//! Git 操作的完成状态身份；刷新策略只读取 key，不读取译文。
use khaslana::ui_i18n::UiMessage;
pub const FETCHED: UiMessage = UiMessage::new("operation.fetched", "拉取远程引用完成", "Remote references fetched");
pub const PULLED: UiMessage = UiMessage::new("operation.pulled", "拉取完成", "Pull completed");
pub const PUSHED: UiMessage = UiMessage::new("operation.pushed", "推送完成", "Push completed");
pub const COMMITTED: UiMessage = UiMessage::new("operation.committed", "提交完成", "Commit completed");
pub const AMENDED: UiMessage = UiMessage::new("operation.amended", "修补提交完成", "Commit amended");
pub const CHERRY_PICKED: UiMessage = UiMessage::new("operation.cherry_picked", "拣选提交完成", "Cherry-pick completed");
pub const BRANCH_CREATED: UiMessage = UiMessage::new("operation.branch_created", "分支已创建", "Branch created");
pub const BRANCH_RENAMED: UiMessage = UiMessage::new("operation.branch_renamed", "分支已重命名", "Branch renamed");
pub const BRANCH_DELETED: UiMessage = UiMessage::new("operation.branch_deleted", "分支已删除", "Branch deleted");
pub const REMOTE_BRANCH_DELETED: UiMessage = UiMessage::new("operation.remote_branch_deleted", "远端分支已删除", "Remote branch deleted");
pub const REMOTE_UPDATED: UiMessage = UiMessage::new("operation.remote_updated", "远端已更新", "Remote updated");
pub const REMOTE_ADDED: UiMessage = UiMessage::new("operation.remote_added", "远端已新增", "Remote added");
pub const REMOTE_DELETED: UiMessage = UiMessage::new("operation.remote_deleted", "远端已删除", "Remote deleted");
pub const REMOTE_REFRESHED: UiMessage = UiMessage::new("operation.remote_refreshed", "远端已刷新", "Remotes refreshed");
pub const UPSTREAM_SET: UiMessage = UiMessage::new("operation.upstream_set", "upstream 已设置", "Upstream set");
pub const TAG_CREATED: UiMessage = UiMessage::new("operation.tag_created", "标签已创建", "Tag created");
pub const TAG_DELETED: UiMessage = UiMessage::new("operation.tag_deleted", "标签已删除", "Tag deleted");
pub const TAG_PUSHED: UiMessage = UiMessage::new("operation.tag_pushed", "标签已推送", "Tag pushed");
pub const REMOTE_TAG_DELETED: UiMessage = UiMessage::new("operation.remote_tag_deleted", "远端标签已删除", "Remote tag deleted");
pub const STASH_APPLIED: UiMessage = UiMessage::new("operation.stash_applied", "应用贮藏完成", "Stash applied");
pub const STASH_POPPED: UiMessage = UiMessage::new("operation.stash_popped", "弹出贮藏完成", "Stash popped");
pub const STASH_CREATED: UiMessage = UiMessage::new("operation.stash_created", "当前改动已贮藏", "Current changes stashed");
pub const STASH_DELETED: UiMessage = UiMessage::new("operation.stash_deleted", "贮藏已删除", "Stash deleted");
pub const REBASE_COMPLETED: UiMessage = UiMessage::new("operation.rebase_completed", "变基完成", "Rebase completed");
pub const REBASE_ABORTED: UiMessage = UiMessage::new("operation.rebase_aborted", "变基已中止", "Rebase aborted");
pub const REBASE_PULLED: UiMessage = UiMessage::new("operation.rebase_pulled", "变基拉取完成", "Pull with rebase completed");
pub const MERGE_COMPLETED: UiMessage = UiMessage::new("operation.merge_completed", "合并已完成", "Merge completed");
pub const MERGE_ABORTED: UiMessage = UiMessage::new("operation.merge_aborted", "合并已中止", "Merge aborted");
pub const MERGE_OPERATION_COMPLETED: UiMessage = UiMessage::new("operation.merge_operation_completed", "合并操作已完成", "Branch merge completed");
pub const CONFLICT_RESOLVED: UiMessage = UiMessage::new("operation.conflict_resolved", "冲突已标记为解决", "Conflict marked as resolved");
pub const SUBMODULE_SYNCED: UiMessage = UiMessage::new("operation.submodule_synced", "子模块已同步记录版本", "Submodule synchronized to recorded revision");
pub const SUBMODULE_UPDATED: UiMessage = UiMessage::new("operation.submodule_updated", "子模块 {name} 已更新到远端最新", "Submodule {name} updated to latest remote revision");
pub const SUBMODULES_UPDATED: UiMessage = UiMessage::new("operation.submodules_updated", "子模块已更新到远端最新", "Submodules updated to latest remote revision");
pub const STAGED: UiMessage = UiMessage::new("operation.staged", "暂存", "Stage");
pub const UNSTAGED: UiMessage = UiMessage::new("operation.unstaged", "取消暂存", "Unstage");
pub const STAGED_FILES: UiMessage = UiMessage::new("operation.staged_files", "已暂存选定文件", "Selected files staged");
pub const UNSTAGED_FILES: UiMessage = UiMessage::new("operation.unstaged_files", "已取消暂存选定文件", "Selected files unstaged");
pub const STAGED_ALL: UiMessage = UiMessage::new("operation.staged_all", "已暂存所有文件", "All files staged");
pub const UNSTAGED_ALL: UiMessage = UiMessage::new("operation.unstaged_all", "已取消暂存所有文件", "All files unstaged");
pub const STAGED_LINES: UiMessage = UiMessage::new("operation.staged_lines", "已暂存选中改动", "Selected changes staged");
pub const UNSTAGED_LINES: UiMessage = UiMessage::new("operation.unstaged_lines", "已取消暂存选中改动", "Selected changes unstaged");
pub const BRANCH_RESET: UiMessage = UiMessage::new("operation.branch_reset", "分支已重置", "Branch reset");
pub const COMMIT_REVERTED: UiMessage = UiMessage::new("operation.commit_reverted", "回滚提交完成", "Commit reverted");
pub const MERGE_REVERTED: UiMessage = UiMessage::new("operation.merge_reverted", "撤销合并完成", "Merge reverted");
pub const COMMIT_UNDONE: UiMessage = UiMessage::new("operation.commit_undone", "提交已还原到暂存区", "Commit restored to staging area");
pub const BRANCH_CHECKED_OUT: UiMessage = UiMessage::new("operation.branch_checked_out", "切换分支完成", "Branch checked out");
pub const TAG_CHECKED_OUT: UiMessage = UiMessage::new("operation.tag_checked_out", "检出标签完成", "Tag checked out");
pub const REMOTE_BRANCH_PULLED: UiMessage = UiMessage::new("operation.remote_branch_pulled", "远端分支已拉取到本地", "Remote branch pulled locally");
pub const BRANCH_PULLED: UiMessage = UiMessage::new("operation.branch_pulled", "分支拉取完成", "Branch pull completed");
pub const COMMITTED_PUSHED: UiMessage = UiMessage::new("operation.committed_pushed", "提交并推送完成", "Commit and push completed");
pub const AMENDED_PUSHED: UiMessage = UiMessage::new("operation.amended_pushed", "修补提交并推送完成", "Amend and push completed");
pub const CLONED: UiMessage = UiMessage::new("operation.cloned", "克隆完成", "Clone completed");
pub const REFRESHED: UiMessage = UiMessage::new("operation.refreshed", "已刷新", "Refreshed");
pub const DIFF_LOADED: UiMessage = UiMessage::new("operation.diff_loaded", "差异已加载", "Diff loaded");
pub const EXTERNAL_MERGE_APPLIED: UiMessage = UiMessage::new("operation.external_merge_applied", "IntelliJ IDEA 合并结果已应用", "IntelliJ IDEA merge result applied");
pub const REFRESH_STARTED: UiMessage = UiMessage::new("operation.refreshed.started", "正在刷新仓库", "Refreshing repository");

#[cfg(test)]
#[test]
fn resources_have_unique_keys_and_matching_parameters() {
    khaslana::ui_i18n::validate(&[FETCHED, PULLED, PUSHED, COMMITTED, AMENDED, CHERRY_PICKED, BRANCH_CREATED, BRANCH_RENAMED, BRANCH_DELETED, REMOTE_BRANCH_DELETED, REMOTE_UPDATED, REMOTE_ADDED, REMOTE_DELETED, REMOTE_REFRESHED, UPSTREAM_SET, TAG_CREATED, TAG_DELETED, TAG_PUSHED, REMOTE_TAG_DELETED, STASH_APPLIED, STASH_POPPED, STASH_CREATED, STASH_DELETED, REBASE_COMPLETED, REBASE_ABORTED, REBASE_PULLED, MERGE_COMPLETED, MERGE_ABORTED, MERGE_OPERATION_COMPLETED, CONFLICT_RESOLVED, SUBMODULE_SYNCED, SUBMODULE_UPDATED, SUBMODULES_UPDATED, STAGED, UNSTAGED, STAGED_FILES, UNSTAGED_FILES, STAGED_ALL, UNSTAGED_ALL, STAGED_LINES, UNSTAGED_LINES, BRANCH_RESET, COMMIT_REVERTED, MERGE_REVERTED, COMMIT_UNDONE, BRANCH_CHECKED_OUT, TAG_CHECKED_OUT, REMOTE_BRANCH_PULLED, BRANCH_PULLED, COMMITTED_PUSHED, AMENDED_PUSHED, CLONED, REFRESHED, DIFF_LOADED, EXTERNAL_MERGE_APPLIED, REFRESH_STARTED]).unwrap();
}
