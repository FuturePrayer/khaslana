//! 领域自有提示与错误的稳定资源；业务身份不使用译文，参数保持原始数据。
use crate::ui_i18n::UiMessage;

pub const GIT_ERROR: UiMessage = UiMessage::new("domain.git_error", "Git 错误：{detail}", "Git error: {detail}");
pub const IO_ERROR: UiMessage = UiMessage::new("domain.io_error", "I/O 错误：{detail}", "I/O error: {detail}");
pub const CREDENTIAL_ERROR: UiMessage = UiMessage::new("domain.credential_error", "凭据错误：{detail}", "Credential error: {detail}");
pub const INVALID_BRANCH_NAME: UiMessage = UiMessage::new("domain.invalid_branch_name", "分支名称无效：{detail}", "Invalid branch name: {detail}");
pub const EMPTY_COMMIT_MESSAGE: UiMessage = UiMessage::new("domain.empty_commit_message", "提交信息不能为空", "Commit message cannot be empty");
pub const EMPTY_COMMIT: UiMessage = UiMessage::new("domain.empty_commit", "没有暂存的变更，不能创建空提交", "No staged changes; cannot create an empty commit");
pub const NO_REPOSITORY: UiMessage = UiMessage::new("domain.no_repository", "尚未打开仓库", "No repository is open");
pub const OPERATION_CONFLICTS: UiMessage = UiMessage::new("domain.operation_conflicts", "操作产生冲突：{detail}", "The operation caused conflicts: {detail}");
pub const WORKFLOW_CANCELLED: UiMessage = UiMessage::new("domain.workflow_cancelled", "工作流已取消", "Workflow cancelled");
pub const FULL_FILE_TOO_LARGE: UiMessage = UiMessage::new("domain.full_file_too_large", "文件过大，无法显示全文视图", "The file is too large to display in full");
pub const NON_FAST_FORWARD_PUSH: UiMessage = UiMessage::new("domain.non_fast_forward_push", "推送被拒绝（non-fast-forward）：远端分支有新提交，请先拉取并解决后再推送", "Push rejected (non-fast-forward): the remote branch has new commits; pull and resolve them before pushing");

pub const CLONING: UiMessage = UiMessage::new("domain.cloning", "正在克隆 {url}", "Cloning {url}");
pub const CLONED: UiMessage = UiMessage::new("domain.cloned", "已克隆 {url}", "Cloned {url}");
pub const REMOTE_NAME_ALREADY_EXISTS: UiMessage = UiMessage::new("domain.remote_name_already_exists", "远端名称已存在：{value0}", "Remote name already exists: {value0}");
pub const FETCHING: UiMessage = UiMessage::new("domain.fetching", "正在获取 {value0}", "Fetching {value0}");
pub const FETCHED: UiMessage = UiMessage::new("domain.fetched", "已获取 {value0}", "Fetched {value0}");
pub const REFRESHING: UiMessage = UiMessage::new("domain.refreshing", "正在刷新 {value0}", "Refreshing {value0}");
pub const REFRESHED: UiMessage = UiMessage::new("domain.refreshed", "已刷新 {value0}", "Refreshed {value0}");
pub const PULLING: UiMessage = UiMessage::new("domain.pulling", "正在拉取 {value0}", "Pulling {value0}");
pub const PULLED: UiMessage = UiMessage::new("domain.pulled", "已拉取 {value0}", "Pulled {value0}");
pub const PULLING_2: UiMessage = UiMessage::new("domain.pulling_2", "正在拉取 {value0}/{value1}", "Pulling {value0}/{value1}");
pub const PULLED_2: UiMessage = UiMessage::new("domain.pulled_2", "已拉取 {value0}/{value1}", "Pulled {value0}/{value1}");
pub const PULLING_LOCAL_BRANCH: UiMessage = UiMessage::new("domain.pulling_local_branch", "正在拉取本地分支 {value0}（{remote}/{remote_branch}）", "Pulling local branch {value0} ({remote}/{remote_branch})");
pub const LOCAL_BRANCH_IS_UP_TO_DATE: UiMessage = UiMessage::new("domain.local_branch_is_up_to_date", "本地分支 {value0} 已是最新", "Local branch {value0} is up to date");
pub const BRANCH_HAS_DIVERGED_FROM_SWITCH_TO_THAT_BRANCH_BEFORE_PULLING: UiMessage = UiMessage::new("domain.branch_has_diverged_from_switch_to_that_branch_before_pulling", "分支 {value0} 与 {remote}/{remote_branch} 已分叉，请先切换到该分支再拉取", "Branch {value0} has diverged from {remote}/{remote_branch}; switch to that branch before pulling");
pub const PULLED_LOCAL_BRANCH: UiMessage = UiMessage::new("domain.pulled_local_branch", "已拉取本地分支 {value0}", "Pulled local branch {value0}");
pub const LOCAL_BRANCH_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.local_branch_does_not_exist", "本地分支不存在：{value0}", "Local branch does not exist: {value0}");
pub const PUSHING_TO: UiMessage = UiMessage::new("domain.pushing_to", "正在推送 {value0} 到 {value1}/{value2}", "Pushing {value0} to {value1}/{value2}");
pub const PUSHED_TO: UiMessage = UiMessage::new("domain.pushed_to", "已推送 {value0} 到 {value1}/{value2}", "Pushed {value0} to {value1}/{value2}");
pub const REMOTE_BRANCH_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.remote_branch_does_not_exist", "远端分支不存在：{upstream}", "Remote branch does not exist: {upstream}");
pub const DELETING_REMOTE_BRANCH: UiMessage = UiMessage::new("domain.deleting_remote_branch", "正在删除远端分支 {upstream}", "Deleting remote branch {upstream}");
pub const DELETED_REMOTE_BRANCH: UiMessage = UiMessage::new("domain.deleted_remote_branch", "已删除远端分支 {upstream}", "Deleted remote branch {upstream}");
pub const BRANCH_HAS_NO_TARGET_COMMIT: UiMessage = UiMessage::new("domain.branch_has_no_target_commit", "分支 {value0} 没有目标提交", "Branch {value0} has no target commit");
pub const REMOTE_BRANCH_HAS_NO_TARGET_COMMIT: UiMessage = UiMessage::new("domain.remote_branch_has_no_target_commit", "远端分支 {value0} 没有目标提交", "Remote branch {value0} has no target commit");
pub const CANNOT_SWITCH_TO_LOCAL_BRANCH: UiMessage = UiMessage::new("domain.cannot_switch_to_local_branch", "无法切换到本地分支 {local_name}：{value0}", "Cannot switch to local branch {local_name}: {value0}");
pub const TAG_ALREADY_EXISTS: UiMessage = UiMessage::new("domain.tag_already_exists", "标签已存在：{value0}", "Tag already exists: {value0}");
pub const TAG_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.tag_does_not_exist", "标签不存在：{value0}", "Tag does not exist: {value0}");
pub const LOCAL_TAG_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.local_tag_does_not_exist", "本地标签不存在：{value0}", "Local tag does not exist: {value0}");
pub const PUSHING_TAG_TO: UiMessage = UiMessage::new("domain.pushing_tag_to", "正在推送标签 {value0} 到 {value1}", "Pushing tag {value0} to {value1}");
pub const DELETING_REMOTE_TAG: UiMessage = UiMessage::new("domain.deleting_remote_tag", "正在删除远端标签 {value0}/{value1}", "Deleting remote tag {value0}/{value1}");
pub const BRANCH_NAME_ALREADY_EXISTS: UiMessage = UiMessage::new("domain.branch_name_already_exists", "分支名称已存在：{value0}", "Branch name already exists: {value0}");
pub const CHANGES_FROM_COMMIT_ARE_ALREADY_IN_THE_CURRENT_BRANCH_CHERRY_PICK_RESULT_IS_EMPTY: UiMessage = UiMessage::new("domain.changes_from_commit_are_already_in_the_current_branch_cherry_pick_result_is_empty", "提交 {short_oid} 的改动已在当前分支中，拣选结果为空", "Changes from commit {short_oid} are already in the current branch; cherry-pick result is empty");
pub const INVALID_COMMIT_ID: UiMessage = UiMessage::new("domain.invalid_commit_id", "提交 ID 无效：{value0}", "Invalid commit ID: {value0}");
pub const RECEIVED_OBJECTS: UiMessage = UiMessage::new("domain.received_objects", "已接收 {value0}/{value1} 个对象", "Received {value0}/{value1} objects");
pub const REMOTE_REJECTED_PUSH_TO: UiMessage = UiMessage::new("domain.remote_rejected_push_to", "远端拒绝推送 {name}：{msg}", "Remote rejected push to {name}: {msg}");
pub const REMOTE_BRANCH_DOES_NOT_EXIST_2: UiMessage = UiMessage::new("domain.remote_branch_does_not_exist_2", "远端分支不存在：{value0}/{value1}", "Remote branch does not exist: {value0}/{value1}");
pub const THIS_FILE_HAS_CONFLICTS_RESOLVE_THEM_BEFORE_ATTEMPTING_TO: UiMessage = UiMessage::new("domain.this_file_has_conflicts_resolve_them_before_attempting_to", "该文件存在冲突，请先解决冲突后再{action}", "This file has conflicts; resolve them before attempting to {action}");
pub const INVALID_REMOTE_NAME: UiMessage = UiMessage::new("domain.invalid_remote_name", "远端名称无效：{name}", "Invalid remote name: {name}");
pub const INVALID_TAG_NAME: UiMessage = UiMessage::new("domain.invalid_tag_name", "标签名称无效：{name}", "Invalid tag name: {name}");
pub const AUTHENTICATION_IS_REQUIRED_TO_ACCESS: UiMessage = UiMessage::new("domain.authentication_is_required_to_access", "访问 {url} 需要身份验证", "Authentication is required to access {url}");
pub const INVALID_FILE_PATH: UiMessage = UiMessage::new("domain.invalid_file_path", "文件路径无效，{action}", "Invalid file path; {action}");
pub const THE_UPSTREAM_REMOTE_NAME_IS_NOT_VALID_UTF_8: UiMessage = UiMessage::new("domain.the_upstream_remote_name_is_not_valid_utf_8", "upstream 远端名称不是有效 UTF-8", "The upstream remote name is not valid UTF-8");
pub const THE_UPSTREAM_BRANCH_NAME_IS_NOT_VALID_UTF_8: UiMessage = UiMessage::new("domain.the_upstream_branch_name_is_not_valid_utf_8", "upstream 分支名称不是有效 UTF-8", "The upstream branch name is not valid UTF-8");
pub const CANNOT_IDENTIFY_THE_REMOTE_BRANCH_ASSOCIATED_WITH_THE_LOCAL_BRANCH: UiMessage = UiMessage::new("domain.cannot_identify_the_remote_branch_associated_with_the_local_branch", "无法识别本地分支关联的远程分支", "Cannot identify the remote branch associated with the local branch");
pub const THE_LOCAL_BRANCH_HAS_NO_COMMIT_TO_UPDATE: UiMessage = UiMessage::new("domain.the_local_branch_has_no_commit_to_update", "本地分支没有可更新的提交", "The local branch has no commit to update");
pub const THE_REMOTE_BRANCH_HAS_NO_COMMIT_TO_PULL: UiMessage = UiMessage::new("domain.the_remote_branch_has_no_commit_to_pull", "远程分支没有可拉取的提交", "The remote branch has no commit to pull");
pub const CREATING_TAG: UiMessage = UiMessage::new("domain.creating_tag", "正在创建标签", "Creating tag");
pub const TAG_CREATED: UiMessage = UiMessage::new("domain.tag_created", "标签已创建", "Tag created");
pub const DELETING_TAG: UiMessage = UiMessage::new("domain.deleting_tag", "正在删除标签", "Deleting tag");
pub const TAG_DELETED: UiMessage = UiMessage::new("domain.tag_deleted", "标签已删除", "Tag deleted");
pub const TAG_PUSHED: UiMessage = UiMessage::new("domain.tag_pushed", "标签已推送", "Tag pushed");
pub const REMOTE_TAG_DELETED: UiMessage = UiMessage::new("domain.remote_tag_deleted", "远端标签已删除", "Remote tag deleted");
pub const DISCARDING_UNSTAGED_CHANGES: UiMessage = UiMessage::new("domain.discarding_unstaged_changes", "正在回滚未暂存更改", "Discarding unstaged changes");
pub const UNSTAGED_CHANGES_DISCARDED: UiMessage = UiMessage::new("domain.unstaged_changes_discarded", "已回滚未暂存更改", "Unstaged changes discarded");
pub const DISCARDING_ALL_FILE_CHANGES: UiMessage = UiMessage::new("domain.discarding_all_file_changes", "正在回滚文件全部更改", "Discarding all file changes");
pub const THIS_REPOSITORY_HAS_NO_HEAD_YET_CANNOT_DISCARD_CHANGES_TO_THIS_FILE: UiMessage = UiMessage::new("domain.this_repository_has_no_head_yet_cannot_discard_changes_to_this_file", "当前仓库还没有 HEAD，不能回滚该文件更改", "This repository has no HEAD yet; cannot discard changes to this file");
pub const ALL_FILE_CHANGES_DISCARDED: UiMessage = UiMessage::new("domain.all_file_changes_discarded", "已回滚文件全部更改", "All file changes discarded");
pub const A_REBASE_IS_IN_PROGRESS_CONTINUE_SKIP_OR_ABORT_FROM_THE_REBASE_STATUS_BAR_INSTEAD_OF: UiMessage = UiMessage::new("domain.a_rebase_is_in_progress_continue_skip_or_abort_from_the_rebase_status_bar_instead_of", "变基进行中，请通过变基状态条继续/跳过/中止，不能直接提交", "A rebase is in progress; continue, skip or abort from the rebase status bar instead of committing directly");
pub const A_MERGE_IS_IN_PROGRESS_COMPLETE_OR_ABORT_IT_BEFORE_AMENDING_A_COMMIT: UiMessage = UiMessage::new("domain.a_merge_is_in_progress_complete_or_abort_it_before_amending_a_commit", "合并进行中，请先完成或中止合并，不能修补提交", "A merge is in progress; complete or abort it before amending a commit");
pub const A_REBASE_IS_IN_PROGRESS_CONTINUE_SKIP_OR_ABORT_FROM_THE_REBASE_STATUS_BAR_BEFORE_AMEN: UiMessage = UiMessage::new("domain.a_rebase_is_in_progress_continue_skip_or_abort_from_the_rebase_status_bar_before_amen", "变基进行中，请通过变基状态条继续/跳过/中止，不能修补提交", "A rebase is in progress; continue, skip or abort from the rebase status bar before amending a commit");
pub const HEAD_IS_DETACHED_SWITCH_TO_A_BRANCH_BEFORE_AMENDING_A_COMMIT: UiMessage = UiMessage::new("domain.head_is_detached_switch_to_a_branch_before_amending_a_commit", "当前处于 detached HEAD，不能修补提交，请先切回分支", "HEAD is detached; switch to a branch before amending a commit");
pub const HEAD_DOES_NOT_POINT_TO_A_LOCAL_BRANCH_CANNOT_AMEND_A_COMMIT: UiMessage = UiMessage::new("domain.head_does_not_point_to_a_local_branch_cannot_amend_a_commit", "当前 HEAD 未指向本地分支，不能修补提交", "HEAD does not point to a local branch; cannot amend a commit");
pub const AMENDING_COMMIT: UiMessage = UiMessage::new("domain.amending_commit", "正在修补提交", "Amending commit");
pub const COMMIT_AMENDED: UiMessage = UiMessage::new("domain.commit_amended", "修补提交完成", "Commit amended");
pub const HEAD_IS_DETACHED_CANNOT_RESET_A_BRANCH: UiMessage = UiMessage::new("domain.head_is_detached_cannot_reset_a_branch", "当前处于 detached HEAD，不能重置分支", "HEAD is detached; cannot reset a branch");
pub const RESETTING_BRANCH: UiMessage = UiMessage::new("domain.resetting_branch", "正在重置分支", "Resetting branch");
pub const BRANCH_RESET: UiMessage = UiMessage::new("domain.branch_reset", "分支已重置", "Branch reset");
pub const HEAD_IS_DETACHED_CANNOT_RESTORE_A_COMMIT_TO_THE_INDEX: UiMessage = UiMessage::new("domain.head_is_detached_cannot_restore_a_commit_to_the_index", "当前处于 detached HEAD，不能还原提交到暂存区", "HEAD is detached; cannot restore a commit to the index");
pub const HEAD_DOES_NOT_POINT_TO_A_LOCAL_BRANCH_CANNOT_RESTORE_A_COMMIT_TO_THE_INDEX: UiMessage = UiMessage::new("domain.head_does_not_point_to_a_local_branch_cannot_restore_a_commit_to_the_index", "当前 HEAD 未指向本地分支，不能还原提交到暂存区", "HEAD does not point to a local branch; cannot restore a commit to the index");
pub const THE_CURRENT_BRANCH_HAS_NO_COMMIT_TO_RESTORE: UiMessage = UiMessage::new("domain.the_current_branch_has_no_commit_to_restore", "当前分支没有可还原的提交", "The current branch has no commit to restore");
pub const ONLY_THE_CURRENT_LATEST_COMMIT_CAN_BE_RESTORED_TO_THE_INDEX: UiMessage = UiMessage::new("domain.only_the_current_latest_commit_can_be_restored_to_the_index", "只能将当前最新提交还原到暂存区", "Only the current latest commit can be restored to the index");
pub const RESTORING_THE_INITIAL_COMMIT_TO_THE_INDEX_IS_NOT_SUPPORTED: UiMessage = UiMessage::new("domain.restoring_the_initial_commit_to_the_index_is_not_supported", "初始提交暂不支持还原到暂存区", "Restoring the initial commit to the index is not supported");
pub const RESTORING_A_MERGE_COMMIT_TO_THE_INDEX_IS_NOT_SUPPORTED: UiMessage = UiMessage::new("domain.restoring_a_merge_commit_to_the_index_is_not_supported", "合并提交暂不支持还原到暂存区", "Restoring a merge commit to the index is not supported");
pub const RESTORING_COMMIT_TO_THE_INDEX: UiMessage = UiMessage::new("domain.restoring_commit_to_the_index", "正在还原提交到暂存区", "Restoring commit to the index");
pub const COMMIT_RESTORED_TO_THE_INDEX: UiMessage = UiMessage::new("domain.commit_restored_to_the_index", "提交已还原到暂存区", "Commit restored to the index");
pub const COMMIT_STASH_OR_DISCARD_WORKING_TREE_CHANGES_BEFORE_REVERTING_A_COMMIT: UiMessage = UiMessage::new("domain.commit_stash_or_discard_working_tree_changes_before_reverting_a_commit", "回滚提交前需要先提交、暂存或丢弃当前工作区修改", "Commit, stash or discard working tree changes before reverting a commit");
pub const REVERTING_MERGE_COMMITS_IS_NOT_SUPPORTED: UiMessage = UiMessage::new("domain.reverting_merge_commits_is_not_supported", "暂不支持回滚合并提交", "Reverting merge commits is not supported");
pub const REVERTING_COMMIT: UiMessage = UiMessage::new("domain.reverting_commit", "正在回滚提交", "Reverting commit");
pub const COMMIT_REVERTED: UiMessage = UiMessage::new("domain.commit_reverted", "回滚提交完成", "Commit reverted");
pub const COMMIT_STASH_OR_DISCARD_WORKING_TREE_CHANGES_BEFORE_REVERTING_A_MERGE: UiMessage = UiMessage::new("domain.commit_stash_or_discard_working_tree_changes_before_reverting_a_merge", "撤销合并前需要先提交、暂存或丢弃当前工作区修改", "Commit, stash or discard working tree changes before reverting a merge");
pub const THIS_IS_NOT_A_MERGE_COMMIT_USE_REGULAR_COMMIT_REVERT: UiMessage = UiMessage::new("domain.this_is_not_a_merge_commit_use_regular_commit_revert", "该提交不是合并提交，请使用普通回滚提交", "This is not a merge commit; use regular commit revert");
pub const REVERTING_MERGE_COMMIT: UiMessage = UiMessage::new("domain.reverting_merge_commit", "正在撤销合并提交", "Reverting merge commit");
pub const MERGE_REVERTED: UiMessage = UiMessage::new("domain.merge_reverted", "撤销合并完成", "Merge reverted");
pub const COMMIT_STASH_OR_DISCARD_WORKING_TREE_CHANGES_BEFORE_CHERRY_PICKING: UiMessage = UiMessage::new("domain.commit_stash_or_discard_working_tree_changes_before_cherry_picking", "拣选提交前需要先提交、暂存或丢弃当前工作区修改", "Commit, stash or discard working tree changes before cherry-picking");
pub const CHERRY_PICKING_MERGE_COMMITS_IS_NOT_SUPPORTED: UiMessage = UiMessage::new("domain.cherry_picking_merge_commits_is_not_supported", "暂不支持拣选合并提交", "Cherry-picking merge commits is not supported");
pub const CHERRY_PICKING_COMMIT: UiMessage = UiMessage::new("domain.cherry_picking_commit", "正在拣选提交", "Cherry-picking commit");
pub const CHERRY_PICK_COMPLETED: UiMessage = UiMessage::new("domain.cherry_pick_completed", "拣选提交完成", "Cherry-pick completed");
pub const UNKNOWN_AUTHOR: UiMessage = UiMessage::new("domain.unknown_author", "未知作者", "Unknown author");
pub const UNKNOWN_COMMITTER: UiMessage = UiMessage::new("domain.unknown_committer", "未知提交者", "Unknown committer");
pub const NO_COMMIT_MESSAGE: UiMessage = UiMessage::new("domain.no_commit_message", "(无提交信息)", "(No commit message)");
pub const DISCARD_CHANGES: UiMessage = UiMessage::new("domain.discard_changes", "回滚更改", "discard changes");
pub const REMOTE_URL_CANNOT_BE_EMPTY: UiMessage = UiMessage::new("domain.remote_url_cannot_be_empty", "远端地址不能为空", "Remote URL cannot be empty");
pub const THE_COMMIT_AUTHOR_NAME_IS_MISSING: UiMessage = UiMessage::new("domain.the_commit_author_name_is_missing", "提交作者名字缺失", "The commit author name is missing");
pub const THE_COMMIT_AUTHOR_EMAIL_IS_MISSING: UiMessage = UiMessage::new("domain.the_commit_author_email_is_missing", "提交作者邮箱缺失", "The commit author email is missing");
pub const CANNOT_DISCARD_CHANGES: UiMessage = UiMessage::new("domain.cannot_discard_changes", "不能回滚更改", "cannot discard changes");
pub const A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_DISCARD_FILE_CHANGES: UiMessage = UiMessage::new("domain.a_bare_repository_has_no_working_tree_cannot_discard_file_changes", "裸仓库没有工作区，不能回滚文件更改", "A bare repository has no working tree; cannot discard file changes");
pub const THIS_FILE_HAS_NOT_BEEN_COMMITTED_NO_TRACE_INFORMATION_IS_AVAILABLE: UiMessage = UiMessage::new("domain.this_file_has_not_been_committed_no_trace_information_is_available", "该文件尚未提交，暂无追溯信息", "This file has not been committed; no trace information is available");
pub const THIS_PATH_IS_NOT_A_FILE: UiMessage = UiMessage::new("domain.this_path_is_not_a_file", "该路径不是文件", "This path is not a file");
pub const BINARY_FILES_DO_NOT_SUPPORT_TRACE_INFORMATION: UiMessage = UiMessage::new("domain.binary_files_do_not_support_trace_information", "二进制文件不支持追溯", "Binary files do not support trace information");
pub const CANNOT_RESOLVE_REFERENCE: UiMessage = UiMessage::new("domain.cannot_resolve_reference", "无法解析引用 {name}: {err}", "Cannot resolve reference {name}: {err}");
pub const IS_NOT_A_COMMIT: UiMessage = UiMessage::new("domain.is_not_a_commit", "{name} 不是提交", "{name} is not a commit");
pub const TAG_DOES_NOT_EXIST_2: UiMessage = UiMessage::new("domain.tag_does_not_exist_2", "标签 {name} 不存在", "Tag {name} does not exist");
pub const TAG_DOES_NOT_POINT_TO_A_COMMIT: UiMessage = UiMessage::new("domain.tag_does_not_point_to_a_commit", "标签 {name} 不指向提交", "Tag {name} does not point to a commit");
pub const PATH_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.path_does_not_exist", "路径不存在: {value0}", "Path does not exist: {value0}");
pub const FILE_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.file_does_not_exist", "文件不存在: {value0}", "File does not exist: {value0}");
pub const THIS_PATH_IS_NOT_A_DIRECTORY: UiMessage = UiMessage::new("domain.this_path_is_not_a_directory", "该路径不是目录", "This path is not a directory");
pub const BRANCH: UiMessage = UiMessage::new("domain.branch", "分支 {name}", "Branch {name}");
pub const REMOTE_BRANCH: UiMessage = UiMessage::new("domain.remote_branch", "远端分支 {name}", "Remote branch {name}");
pub const TAG: UiMessage = UiMessage::new("domain.tag", "标签 {name}", "Tag {name}");
pub const UNCOMMITTED_CHANGES_WERE_STASHED_BUT_SWITCHING_TO_STILL_FAILED_CHANGES_REMAIN_IN_THE: UiMessage = UiMessage::new("domain.uncommitted_changes_were_stashed_but_switching_to_still_failed_changes_remain_in_the", "已暂存未提交修改，但切换到{target}仍失败：{err}；修改已保留在贮藏中，可手动弹出或删除", "Uncommitted changes were stashed, but switching to {target} still failed: {err}; changes remain in the stash and can be popped or deleted manually");
pub const SWITCHED_TO_UNCOMMITTED_CHANGES_ARE_IN_THE_FIRST_STASH_ENTRY_APPLY_OR_POP_IT_FROM_THE: UiMessage = UiMessage::new("domain.switched_to_uncommitted_changes_are_in_the_first_stash_entry_apply_or_pop_it_from_the", "已切换到{target}，未提交修改已保留在贮藏列表第一条，需要时可在左侧「贮藏」区域应用或弹出", "Switched to {target}; uncommitted changes are in the first stash entry. Apply or pop it from the Stashes section when needed");
pub const SWITCHED_TO_BUT_RESTORING_CHANGES_FAILED_ORIGINAL_CHANGES_REMAIN_IN_THE_FIRST_STASH_E: UiMessage = UiMessage::new("domain.switched_to_but_restoring_changes_failed_original_changes_remain_in_the_first_stash_e", "已切换到{target}，但恢复修改失败：{err}；原修改保留在贮藏列表第一条，请检查工作区后手动应用", "Switched to {target}, but restoring changes failed: {err}; original changes remain in the first stash entry. Check the working tree and apply them manually");
pub const SWITCHED_TO_BUT_RESTORING_CHANGES_CAUSED_WORKING_TREE_CONFLICTS_THE_STASH_WAS_RETAINE: UiMessage = UiMessage::new("domain.switched_to_but_restoring_changes_caused_working_tree_conflicts_the_stash_was_retaine", "已切换到{target}，但恢复修改时产生工作区冲突；贮藏已保留。请在「冲突」区域查看文件，并用外部编辑器处理冲突标记", "Switched to {target}, but restoring changes caused working tree conflicts; the stash was retained. Review the files in Conflicts and resolve conflict markers in an external editor");
pub const SWITCHED_TO_BUT_RESTORING_UNCOMMITTED_CHANGES_CAUSED_CONFLICTS_ORIGINAL_CHANGES_REMAI: UiMessage = UiMessage::new("domain.switched_to_but_restoring_uncommitted_changes_caused_conflicts_original_changes_remai", "已切换到{target}，但未提交修改恢复时存在冲突；原修改已保留在贮藏列表第一条，请在冲突区域解决", "Switched to {target}, but restoring uncommitted changes caused conflicts; original changes remain in the first stash entry. Resolve them in Conflicts");
pub const THE_CONFLICT_FILE_PATH_IS_NOT_VALID_UTF_8_AUTOMATIC_RESOLUTION_IS_UNAVAILABLE: UiMessage = UiMessage::new("domain.the_conflict_file_path_is_not_valid_utf_8_automatic_resolution_is_unavailable", "冲突文件路径不是有效 UTF-8，无法自动落定", "The conflict file path is not valid UTF-8; automatic resolution is unavailable");
pub const BRANCH_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.branch_does_not_exist", "分支不存在：{branch}", "Branch does not exist: {branch}");
pub const THIS_FILE_HAS_NO_CONFLICTS: UiMessage = UiMessage::new("domain.this_file_has_no_conflicts", "该文件不存在冲突：{git_path}", "This file has no conflicts: {git_path}");
pub const RESOLVING_CONFLICT_USING_THE: UiMessage = UiMessage::new("domain.resolving_conflict_using_the", "正在使用{label}解决冲突", "Resolving conflict using the {label}");
pub const CONFLICT_RESOLVED_USING_THE: UiMessage = UiMessage::new("domain.conflict_resolved_using_the", "已使用{label}解决冲突", "Conflict resolved using the {label}");
pub const THIS_FILE_HAS_NO_CONFLICTS_2: UiMessage = UiMessage::new("domain.this_file_has_no_conflicts_2", "该文件不存在冲突：{value0}", "This file has no conflicts: {value0}");
pub const CANNOT_READ_CONFLICT_DETAILS: UiMessage = UiMessage::new("domain.cannot_read_conflict_details", "不能读取冲突详情", "cannot read conflict details");
pub const A_BARE_REPOSITORY_HAS_NO_WORKING_TREE: UiMessage = UiMessage::new("domain.a_bare_repository_has_no_working_tree", "裸仓库没有工作区", "A bare repository has no working tree");
pub const RESTORING_THE_STASH_WROTE_CONFLICT_MARKERS_TO_THE_WORKING_TREE_BUT_THE_GIT_INDEX_HAS: UiMessage = UiMessage::new("domain.restoring_the_stash_wrote_conflict_markers_to_the_working_tree_but_the_git_index_has", "贮藏恢复在工作区写入了冲突标记，但 Git 索引没有冲突阶段。请在外部编辑器中打开此文件，处理 <<<<<<<、=======、>>>>>>> 标记后刷新仓库；原修改仍保留在贮藏中。", "Restoring the stash wrote conflict markers to the working tree, but the Git index has no conflict stages. Open this file in an external editor, resolve the <<<<<<<, ======= and >>>>>>> markers, then refresh the repository; original changes remain in the stash.");
pub const THREE_WAY_TEXT_IS_MISSING_FOR_THIS_CONFLICT_USE_THE_QUICK_RESOLUTION_BUTTONS: UiMessage = UiMessage::new("domain.three_way_text_is_missing_for_this_conflict_use_the_quick_resolution_buttons", "该冲突缺少三方文本内容，请使用快捷解决按钮", "Three-way text is missing for this conflict; use the quick resolution buttons");
pub const THE_TEXT_MERGE_EDITOR_CANNOT_OPEN_THIS_CONFLICT_FILE: UiMessage = UiMessage::new("domain.the_text_merge_editor_cannot_open_this_conflict_file", "该冲突文件不能使用文本合并编辑器", "The text merge editor cannot open this conflict file");
pub const THIS_CONFLICT_FILE_IS_NOT_UTF_8_TEXT_THE_VISUAL_EDITOR_IS_UNAVAILABLE: UiMessage = UiMessage::new("domain.this_conflict_file_is_not_utf_8_text_the_visual_editor_is_unavailable", "该冲突文件不是 UTF-8 文本，暂不能使用可视化编辑器", "This conflict file is not UTF-8 text; the visual editor is unavailable");
pub const THE_CONFLICT_STAGE_IS_NO_LONGER_VALID: UiMessage = UiMessage::new("domain.the_conflict_stage_is_no_longer_valid", "冲突阶段已失效", "The conflict stage is no longer valid");
pub const THE_CONFLICT_VERSION_IS_NOT_UTF_8_TEXT: UiMessage = UiMessage::new("domain.the_conflict_version_is_not_utf_8_text", "冲突版本不是 UTF-8 文本", "The conflict version is not UTF-8 text");
pub const CANNOT_GENERATE_AI_MERGE_SUGGESTIONS: UiMessage = UiMessage::new("domain.cannot_generate_ai_merge_suggestions", "不能生成 AI 合并建议", "cannot generate AI merge suggestions");
pub const THREE_WAY_TEXT_IS_MISSING_FOR_THIS_CONFLICT_AI_MERGE_SUGGESTIONS_ARE_UNAVAILABLE: UiMessage = UiMessage::new("domain.three_way_text_is_missing_for_this_conflict_ai_merge_suggestions_are_unavailable", "该冲突缺少三方文本内容，不支持 AI 合并建议", "Three-way text is missing for this conflict; AI merge suggestions are unavailable");
pub const BINARY_CONFLICT_FILES_DO_NOT_SUPPORT_AI_MERGE_SUGGESTIONS_USE_THE_QUICK_RESOLUTION_BU: UiMessage = UiMessage::new("domain.binary_conflict_files_do_not_support_ai_merge_suggestions_use_the_quick_resolution_bu", "二进制冲突文件不支持 AI 合并建议，请使用快捷解决按钮", "Binary conflict files do not support AI merge suggestions; use the quick resolution buttons");
pub const THIS_CONFLICT_FILE_IS_NOT_UTF_8_TEXT_AI_MERGE_SUGGESTIONS_ARE_UNAVAILABLE: UiMessage = UiMessage::new("domain.this_conflict_file_is_not_utf_8_text_ai_merge_suggestions_are_unavailable", "该冲突文件不是 UTF-8 文本，暂不支持 AI 合并建议", "This conflict file is not UTF-8 text; AI merge suggestions are unavailable");
pub const CANNOT_APPLY_THE_CONFLICT_DRAFT: UiMessage = UiMessage::new("domain.cannot_apply_the_conflict_draft", "不能应用冲突草稿", "cannot apply the conflict draft");
pub const APPLYING_CONFLICT_DRAFT: UiMessage = UiMessage::new("domain.applying_conflict_draft", "正在应用冲突草稿", "Applying conflict draft");
pub const CONFLICT_DRAFT_APPLIED_TO_THE_WORKING_TREE: UiMessage = UiMessage::new("domain.conflict_draft_applied_to_the_working_tree", "冲突草稿已应用到工作区", "Conflict draft applied to the working tree");
pub const CANNOT_APPLY_AND_RESOLVE_THE_CONFLICT: UiMessage = UiMessage::new("domain.cannot_apply_and_resolve_the_conflict", "不能应用并解决冲突", "cannot apply and resolve the conflict");
pub const APPLYING_RESULT_AND_MARKING_CONFLICT_AS_RESOLVED: UiMessage = UiMessage::new("domain.applying_result_and_marking_conflict_as_resolved", "正在应用结果并标记冲突解决", "Applying result and marking conflict as resolved");
pub const CONFLICT_RESULT_APPLIED_AND_MARKED_AS_RESOLVED: UiMessage = UiMessage::new("domain.conflict_result_applied_and_marked_as_resolved", "冲突结果已应用并标记解决", "Conflict result applied and marked as resolved");
pub const CANNOT_RESOLVE_CONFLICTS_USING_INTELLIJ_IDEA: UiMessage = UiMessage::new("domain.cannot_resolve_conflicts_using_intellij_idea", "不能使用 IntelliJ IDEA 解决冲突", "cannot resolve conflicts using IntelliJ IDEA");
pub const WAITING_FOR_INTELLIJ_IDEA_TO_FINISH_MERGING: UiMessage = UiMessage::new("domain.waiting_for_intellij_idea_to_finish_merging", "正在等待 IntelliJ IDEA 合并完成", "Waiting for IntelliJ IDEA to finish merging");
pub const INTELLIJ_IDEA_MERGE_RESULT_APPLIED: UiMessage = UiMessage::new("domain.intellij_idea_merge_result_applied", "IntelliJ IDEA 合并结果已应用", "IntelliJ IDEA merge result applied");
pub const CANNOT_RESOLVE_THE_CONFLICT: UiMessage = UiMessage::new("domain.cannot_resolve_the_conflict", "不能解决冲突", "cannot resolve the conflict");
pub const CURRENT_VERSION: UiMessage = UiMessage::new("domain.current_version", "当前版本", "current version");
pub const INCOMING_VERSION: UiMessage = UiMessage::new("domain.incoming_version", "传入版本", "incoming version");
pub const CANNOT_MARK_THE_CONFLICT_AS_RESOLVED: UiMessage = UiMessage::new("domain.cannot_mark_the_conflict_as_resolved", "不能标记冲突已解决", "cannot mark the conflict as resolved");
pub const MARKING_CONFLICT_AS_RESOLVED: UiMessage = UiMessage::new("domain.marking_conflict_as_resolved", "正在标记冲突已解决", "Marking conflict as resolved");
pub const CONFLICT_MARKED_AS_RESOLVED: UiMessage = UiMessage::new("domain.conflict_marked_as_resolved", "冲突已标记为解决", "Conflict marked as resolved");
pub const A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_MARK_CONFLICTS_AS_RESOLVED: UiMessage = UiMessage::new("domain.a_bare_repository_has_no_working_tree_cannot_mark_conflicts_as_resolved", "裸仓库没有工作区，不能标记冲突已解决", "A bare repository has no working tree; cannot mark conflicts as resolved");
pub const THE_CONFLICT_PATH_IS_A_DIRECTORY_IT_CANNOT_BE_MARKED_AS_RESOLVED: UiMessage = UiMessage::new("domain.the_conflict_path_is_a_directory_it_cannot_be_marked_as_resolved", "冲突路径是文件夹，不能标记为已解决", "The conflict path is a directory; it cannot be marked as resolved");
pub const A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_WRITE_CONFLICT_RESULTS: UiMessage = UiMessage::new("domain.a_bare_repository_has_no_working_tree_cannot_write_conflict_results", "裸仓库没有工作区，不能写入冲突结果", "A bare repository has no working tree; cannot write conflict results");
pub const CONFLICT_TEXT_IS_MISSING_THE_BASE_SEPARATOR: UiMessage = UiMessage::new("domain.conflict_text_is_missing_the_base_separator", "冲突文本缺少 BASE 分隔标记", "Conflict text is missing the BASE separator");
pub const CONFLICT_TEXT_IS_MISSING_THE_MIDDLE_SEPARATOR: UiMessage = UiMessage::new("domain.conflict_text_is_missing_the_middle_separator", "冲突文本缺少中间分隔标记", "Conflict text is missing the middle separator");
pub const CONFLICT_TEXT_IS_MISSING_THE_THEIRS_END_MARKER: UiMessage = UiMessage::new("domain.conflict_text_is_missing_the_theirs_end_marker", "冲突文本缺少 THEIRS 结束标记", "Conflict text is missing the THEIRS end marker");
pub const MERGING: UiMessage = UiMessage::new("domain.merging", "正在合并 {value0}", "Merging {value0}");
pub const MERGE_RESULT_FOR_WRITTEN_TO_THE_INDEX: UiMessage = UiMessage::new("domain.merge_result_for_written_to_the_index", "{value0} 的合并结果已写入暂存区", "Merge result for {value0} written to the index");
pub const MERGED: UiMessage = UiMessage::new("domain.merged", "已合并 {value0}", "Merged {value0}");
pub const CANNOT_MERGE_UNSUPPORTED_MERGE_ANALYSIS_RESULT: UiMessage = UiMessage::new("domain.cannot_merge_unsupported_merge_analysis_result", "无法合并 {label}：不支持的合并分析结果", "Cannot merge {label}: unsupported merge analysis result");
pub const NO_MERGE_IS_CURRENTLY_IN_PROGRESS: UiMessage = UiMessage::new("domain.no_merge_is_currently_in_progress", "当前没有正在进行的合并", "No merge is currently in progress");
pub const COMPLETING_MERGE: UiMessage = UiMessage::new("domain.completing_merge", "正在完成合并", "Completing merge");
pub const MERGE_COMMIT_CREATED: UiMessage = UiMessage::new("domain.merge_commit_created", "合并提交已创建", "Merge commit created");
pub const ABORTING_MERGE: UiMessage = UiMessage::new("domain.aborting_merge", "正在中止合并", "Aborting merge");
pub const MERGE_ABORTED: UiMessage = UiMessage::new("domain.merge_aborted", "合并已中止", "Merge aborted");
pub const ANOTHER_GIT_OPERATION_IS_IN_PROGRESS_COMPLETE_OR_ABORT_IT_BEFORE_MERGING: UiMessage = UiMessage::new("domain.another_git_operation_is_in_progress_complete_or_abort_it_before_merging", "仓库正在执行其他 Git 操作，请先完成或中止后再合并", "Another Git operation is in progress; complete or abort it before merging");
pub const THERE_ARE_UNCOMMITTED_CHANGES_COMMIT_OR_STASH_THEM_BEFORE_MERGING: UiMessage = UiMessage::new("domain.there_are_uncommitted_changes_commit_or_stash_them_before_merging", "当前存在未提交改动，请先提交或贮藏后再合并", "There are uncommitted changes; commit or stash them before merging");
pub const MERGE_STATE_IS_MISSING_MERGE_HEAD_CANNOT_COMPLETE_THE_MERGE: UiMessage = UiMessage::new("domain.merge_state_is_missing_merge_head_cannot_complete_the_merge", "合并状态缺少 MERGE_HEAD，无法完成合并", "Merge state is missing MERGE_HEAD; cannot complete the merge");
pub const SLIDE: UiMessage = UiMessage::new("domain.slide", "── 幻灯片 {num} ──", "── Slide {num} ──");
pub const THIS_FILE_WAS_ADDED_DELETED_OR_RENAMED_AS_A_WHOLE_CANNOT_PART_OF_IT_USE_THE_WHOLE_FIL: UiMessage = UiMessage::new("domain.this_file_was_added_deleted_or_renamed_as_a_whole_cannot_part_of_it_use_the_whole_fil", "该文件为整文件新增/删除/重命名，暂不支持部分{action}，请使用整文件按钮", "This file was added, deleted or renamed as a whole; cannot {action} part of it. Use the whole-file button");
pub const CANNOT_PART_OF_A_BINARY_FILE_USE_THE_WHOLE_FILE_BUTTON: UiMessage = UiMessage::new("domain.cannot_part_of_a_binary_file_use_the_whole_file_button", "二进制文件不支持部分{action}，请使用整文件按钮", "Cannot {action} part of a binary file; use the whole-file button");
pub const NO_CHANGES_IN_THIS_FILE_CAN_BE_PARTIALLY_PROCESSED_WITH_IT_MAY_BE_UNTRACKED_OR_HAVE_C: UiMessage = UiMessage::new("domain.no_changes_in_this_file_can_be_partially_processed_with_it_may_be_untracked_or_have_c", "该文件没有可部分{action}的改动（可能是未跟踪文件或内容已变化），请刷新后使用整文件按钮", "No changes in this file can be partially processed with {action} (it may be untracked or have changed); refresh and use the whole-file button");
pub const FAILED_TO_BUILD_THE_PARTIAL_PATCH: UiMessage = UiMessage::new("domain.failed_to_build_the_partial_patch", "构造部分{action}补丁失败：{err}", "Failed to build the partial {action} patch: {err}");
pub const CANNOT_APPLY_SELECTED_CHANGES_TO_THE_INDEX_CONTENT_MAY_HAVE_CHANGED_REFRESH_AND_RETRY: UiMessage = UiMessage::new("domain.cannot_apply_selected_changes_to_the_index_content_may_have_changed_refresh_and_retry", "所选改动无法应用到暂存区（内容可能已变化，请刷新后重试）：{err}", "Cannot apply selected changes to the index (content may have changed; refresh and retry): {err}");
pub const THIS_HUNK_CONTAINS_A_NO_NEWLINE_AT_END_OF_FILE_MARKER_STAGE_THE_WHOLE_HUNK_INSTEAD_OF: UiMessage = UiMessage::new("domain.this_hunk_contains_a_no_newline_at_end_of_file_marker_stage_the_whole_hunk_instead_of", "该改动块包含无尾换行文件的标记，暂不支持按行部分暂存，可整块操作", "This hunk contains a no-newline-at-end-of-file marker; stage the whole hunk instead of individual lines");
pub const NO_CHANGED_LINES_SELECTED: UiMessage = UiMessage::new("domain.no_changed_lines_selected", "没有选中的改动行", "No changed lines selected");
pub const STAGE: UiMessage = UiMessage::new("domain.stage", "暂存", "stage");
pub const STAGING_SELECTED_CHANGES: UiMessage = UiMessage::new("domain.staging_selected_changes", "正在暂存选中改动", "Staging selected changes");
pub const SELECTED_CHANGES_STAGED: UiMessage = UiMessage::new("domain.selected_changes_staged", "已暂存选中改动", "Selected changes staged");
pub const UNSTAGE: UiMessage = UiMessage::new("domain.unstage", "取消暂存", "unstage");
pub const UNSTAGING_SELECTED_CHANGES: UiMessage = UiMessage::new("domain.unstaging_selected_changes", "正在取消暂存选中改动", "Unstaging selected changes");
pub const THERE_IS_NO_COMMIT_HISTORY_PARTIAL_UNSTAGING_IS_UNAVAILABLE_UNSTAGE_THE_WHOLE_FILE_IN: UiMessage = UiMessage::new("domain.there_is_no_commit_history_partial_unstaging_is_unavailable_unstage_the_whole_file_in", "暂无提交历史，不能部分取消暂存，请使用整文件取消暂存", "There is no commit history; partial unstaging is unavailable. Unstage the whole file instead");
pub const SELECTED_CHANGES_UNSTAGED: UiMessage = UiMessage::new("domain.selected_changes_unstaged", "已取消暂存选中改动", "Selected changes unstaged");
pub const THE_SELECTED_CHANGES_ALREADY_MATCH_THE_CURRENT_CONTENT_NOTHING_TO_APPLY: UiMessage = UiMessage::new("domain.the_selected_changes_already_match_the_current_content_nothing_to_apply", "所选改动与当前内容已一致，没有需要应用的变更", "The selected changes already match the current content; nothing to apply");
pub const REBASING: UiMessage = UiMessage::new("domain.rebasing", "正在变基 {value0}", "Rebasing {value0}");
pub const REBASE_PAUSED_WHILE_REPLAYING_COMMIT_RESOLVE_CONFLICTS_BEFORE_CONTINUING: UiMessage = UiMessage::new("domain.rebase_paused_while_replaying_commit_resolve_conflicts_before_continuing", "变基暂停：正在重放提交 {current}/{total}，请解决冲突后继续", "Rebase paused while replaying commit {current}/{total}; resolve conflicts before continuing");
pub const REBASED: UiMessage = UiMessage::new("domain.rebased", "已变基 {value0}", "Rebased {value0}");
pub const PULLING_WITH_REBASE: UiMessage = UiMessage::new("domain.pulling_with_rebase", "正在变基拉取 {value0}/{value1}", "Pulling {value0}/{value1} with rebase");
pub const PULLED_WITH_REBASE: UiMessage = UiMessage::new("domain.pulled_with_rebase", "已变基拉取 {value0}/{value1}", "Pulled {value0}/{value1} with rebase");
pub const CONTINUING_REBASE: UiMessage = UiMessage::new("domain.continuing_rebase", "正在继续变基", "Continuing rebase");
pub const UNRESOLVED_CONFLICTS_REMAIN_RESOLVE_ALL_CONFLICTS_BEFORE_CONTINUING_THE_REBASE: UiMessage = UiMessage::new("domain.unresolved_conflicts_remain_resolve_all_conflicts_before_continuing_the_rebase", "仍有冲突未解决，请先解决所有冲突再继续变基", "Unresolved conflicts remain; resolve all conflicts before continuing the rebase");
pub const REBASE_COMPLETED: UiMessage = UiMessage::new("domain.rebase_completed", "变基已完成", "Rebase completed");
pub const SKIPPING_REBASE_COMMIT: UiMessage = UiMessage::new("domain.skipping_rebase_commit", "正在跳过变基提交", "Skipping rebase commit");
pub const ABORTING_REBASE: UiMessage = UiMessage::new("domain.aborting_rebase", "正在中止变基", "Aborting rebase");
pub const REBASE_ABORTED: UiMessage = UiMessage::new("domain.rebase_aborted", "变基已中止", "Rebase aborted");
pub const CANNOT_READ_THE_RECOVERY_TARGET_COMMIT: UiMessage = UiMessage::new("domain.cannot_read_the_recovery_target_commit", "无法读取恢复目标提交：{error}", "Cannot read the recovery target commit: {error}");
pub const THE_BRANCH_WAS_CREATED_BUT_ITS_TARGET_IS_INSTEAD_OF_THE_REQUESTED_CHECK_THE_REFERENCE: UiMessage = UiMessage::new("domain.the_branch_was_created_but_its_target_is_instead_of_the_requested_check_the_reference", "分支已创建，但读回目标为 {actual}，与请求的 {oid} 不一致，请检查引用", "The branch was created, but its target is {actual} instead of the requested {oid}; check the reference");
pub const THE_BRANCH_WAS_CREATED_BUT_READ_BACK_VERIFICATION_FAILED: UiMessage = UiMessage::new("domain.the_branch_was_created_but_read_back_verification_failed", "分支已创建，但无法读回验证：{error}", "The branch was created, but read-back verification failed: {error}");
pub const CANNOT_INSPECT_HEAD_OF_A_KNOWN_WORKING_TREE: UiMessage = UiMessage::new("domain.cannot_inspect_head_of_a_known_working_tree", "无法检查已知工作树的 HEAD（{value0}）：{error}", "Cannot inspect HEAD of a known working tree ({value0}): {error}");
pub const CANNOT_INSPECT_WORKING_TREE_HEAD: UiMessage = UiMessage::new("domain.cannot_inspect_working_tree_head", "无法检查工作树 HEAD：{error}", "Cannot inspect working tree HEAD: {error}");
pub const LOCAL_RECOVERY_DOES_NOT_SUPPORT_BARE_REPOSITORIES: UiMessage = UiMessage::new("domain.local_recovery_does_not_support_bare_repositories", "本地恢复暂不支持裸仓库", "Local recovery does not support bare repositories");
pub const LOCAL_RECOVERY_DOES_NOT_SUPPORT_REPOSITORIES_WITHOUT_A_WORKING_TREE: UiMessage = UiMessage::new("domain.local_recovery_does_not_support_repositories_without_a_working_tree", "本地恢复暂不支持没有工作树的仓库", "Local recovery does not support repositories without a working tree");
pub const COMMIT_DETAILS_ARE_EMPTY: UiMessage = UiMessage::new("domain.commit_details_are_empty", "提交详情为空", "Commit details are empty");
pub const THE_RECOVERY_WRITE_LOCK_IS_INVALID_REOPEN_THE_REPOSITORY_AND_RETRY: UiMessage = UiMessage::new("domain.the_recovery_write_lock_is_invalid_reopen_the_repository_and_retry", "恢复写入锁异常，请重新打开仓库后重试", "The recovery write lock is invalid; reopen the repository and retry");
pub const THE_REPOSITORY_IDENTITY_HAS_CHANGED_SINCE_THE_RECOVERY_REQUEST_SELECT_THE_REPOSITORY: UiMessage = UiMessage::new("domain.the_repository_identity_has_changed_since_the_recovery_request_select_the_repository", "恢复请求的仓库身份已变化，请重新选择仓库与提交", "The repository identity has changed since the recovery request; select the repository and commit again");
pub const A_ZERO_OID_IS_NOT_A_RECOVERABLE_COMMIT_OBJECT: UiMessage = UiMessage::new("domain.a_zero_oid_is_not_a_recoverable_commit_object", "零 OID 不是可恢复的提交对象", "A zero OID is not a recoverable commit object");
pub const THE_RECOVERY_TARGET_IS_NOT_A_COMMIT_OBJECT: UiMessage = UiMessage::new("domain.the_recovery_target_is_not_a_commit_object", "恢复目标不是提交对象", "The recovery target is not a commit object");
pub const THE_BRANCH_WAS_CREATED_BUT_THE_REFERENCE_READ_BACK_IS_NOT_DIRECT_CHECK_THE_REFERENCE: UiMessage = UiMessage::new("domain.the_branch_was_created_but_the_reference_read_back_is_not_direct_check_the_reference", "分支已创建，但读回的引用不是直接引用，请检查引用", "The branch was created, but the reference read back is not direct; check the reference");
pub const THE_BRANCH_WAS_CREATED_BUT_READ_BACK_VERIFICATION_WAS_INTERRUPTED_CHECK_THE_REFERENCE: UiMessage = UiMessage::new("domain.the_branch_was_created_but_read_back_verification_was_interrupted_check_the_reference", "分支已创建，但读回验证意外中断，请检查引用", "The branch was created, but read-back verification was interrupted; check the reference");
pub const THE_RECOVERY_TARGET_MUST_BE_A_FULL_40_DIGIT_HEXADECIMAL_OID: UiMessage = UiMessage::new("domain.the_recovery_target_must_be_a_full_40_digit_hexadecimal_oid", "恢复目标必须是完整的 40 位十六进制 OID", "The recovery target must be a full 40-digit hexadecimal OID");
pub const HEAD_OF_THIS_OR_ANOTHER_WORKING_TREE_POINTS_TO_THIS_BRANCH_RECOVERY_WAS_REFUSED_BECAU: UiMessage = UiMessage::new("domain.head_of_this_or_another_working_tree_points_to_this_branch_recovery_was_refused_becau", "该分支正由当前或其他工作树的 HEAD 指向，创建会改变 HEAD 解析，已拒绝恢复", "HEAD of this or another working tree points to this branch; recovery was refused because creating it would change HEAD resolution");
pub const THE_HEAD_SYMBOLIC_REFERENCE_CHAIN_CONTAINS_A_CYCLE_CANNOT_SAFELY_CREATE_THE_RECOVERY: UiMessage = UiMessage::new("domain.the_head_symbolic_reference_chain_contains_a_cycle_cannot_safely_create_the_recovery", "HEAD 符号引用存在循环，无法安全创建恢复分支", "The HEAD symbolic reference chain contains a cycle; cannot safely create the recovery branch");
pub const THE_HEAD_SYMBOLIC_REFERENCE_CHAIN_IS_TOO_LONG_CANNOT_SAFELY_CREATE_THE_RECOVERY_BRANC: UiMessage = UiMessage::new("domain.the_head_symbolic_reference_chain_is_too_long_cannot_safely_create_the_recovery_branc", "HEAD 符号引用链过长，无法安全创建恢复分支", "The HEAD symbolic reference chain is too long; cannot safely create the recovery branch");
pub const ADDITIONAL_REFLOG_DIAGNOSTICS_WERE_OMITTED: UiMessage = UiMessage::new("domain.additional_reflog_diagnostics_were_omitted", "另有 {value0} 条日志诊断未逐条展示", "{value0} additional reflog diagnostics were omitted");
pub const INCOMPLETE_REFLOG_COVERAGE_READ_SOURCES_MISSING_OR_UNREADABLE_SOURCES_ARE_NOT_INCLUDE: UiMessage = UiMessage::new("domain.incomplete_reflog_coverage_read_sources_missing_or_unreadable_sources_are_not_include", "日志覆盖不完整：成功读取 {sources_read}/{sources_total} 个来源；缺失或不可读的来源未计入结果", "Incomplete reflog coverage: read {sources_read}/{sources_total} sources; missing or unreadable sources are not included");
pub const UNSUPPORTED_REFERENCE_BACKEND_LOCAL_REFLOG_CURRENTLY_SUPPORTS_ONLY_THE_FILES_BACKEND: UiMessage = UiMessage::new("domain.unsupported_reference_backend_local_reflog_currently_supports_only_the_files_backend", "不支持引用后端 {backend}：本地 Reflog 目前仅支持 files 引用后端", "Unsupported reference backend {backend}: local reflog currently supports only the files backend");
pub const CANNOT_DETERMINE_THE_REFERENCE_BACKEND_LOCAL_REFLOG_WAS_NOT_READ: UiMessage = UiMessage::new("domain.cannot_determine_the_reference_backend_local_reflog_was_not_read", "无法确认引用后端，未读取本地 Reflog：{error}", "Cannot determine the reference backend; local reflog was not read: {error}");
pub const CANNOT_READ_THE_HEAD_SYMBOLIC_REFERENCE_NAME: UiMessage = UiMessage::new("domain.cannot_read_the_head_symbolic_reference_name", "HEAD 的符号引用名不可读：{error}", "Cannot read the HEAD symbolic reference name: {error}");
pub const CANNOT_READ_THE_LOCAL_BRANCH_REFERENCE_NAME: UiMessage = UiMessage::new("domain.cannot_read_the_local_branch_reference_name", "本地分支引用名不可读：{error}", "Cannot read the local branch reference name: {error}");
pub const NOT_A_LOCAL_BRANCH_REFERENCE: UiMessage = UiMessage::new("domain.not_a_local_branch_reference", "不是本地分支引用：{full_name}", "Not a local branch reference: {full_name}");
pub const CANNOT_READ_LOCAL_BRANCH: UiMessage = UiMessage::new("domain.cannot_read_local_branch", "无法读取本地分支 {name}：{error}", "Cannot read local branch {name}: {error}");
pub const REFLOG_PATH_IS_OUTSIDE_THE_REPOSITORY_DIRECTORY_READING_REFUSED: UiMessage = UiMessage::new("domain.reflog_path_is_outside_the_repository_directory_reading_refused", "日志路径超出仓库目录，拒绝读取：{value0}", "Reflog path is outside the repository directory; reading refused: {value0}");
pub const REFLOG_IS_MISSING_THIS_SOURCE_IS_NOT_COVERED_THE_READ_ONLY_SERVICE_DID_NOT_CREATE_ANY: UiMessage = UiMessage::new("domain.reflog_is_missing_this_source_is_not_covered_the_read_only_service_did_not_create_any", "{value0}：日志不存在，该来源未覆盖；只读服务未创建文件", "{value0}: reflog is missing; this source is not covered. The read-only service did not create any files");
pub const REFLOG_IS_UNREADABLE_THIS_SOURCE_IS_NOT_COVERED: UiMessage = UiMessage::new("domain.reflog_is_unreadable_this_source_is_not_covered", "{value0}：日志不可读，该来源未覆盖：{reason}", "{value0}: reflog is unreadable; this source is not covered: {reason}");
pub const REFLOG_IS_EMPTY: UiMessage = UiMessage::new("domain.reflog_is_empty", "{value0}：日志为空", "{value0}: reflog is empty");
pub const LINE_EXCEEDS_THE_256_KIB_LINE_LIMIT_SKIPPED: UiMessage = UiMessage::new("domain.line_exceeds_the_256_kib_line_limit_skipped", "{ref_name} 第 {line_number} 行超过单行 256 KiB 上限，已跳过", "{ref_name} line {line_number} exceeds the 256 KiB line limit; skipped");
pub const LINE_CONTAINS_NON_UTF_8_TEXT_DISPLAYED_WITH_REPLACEMENT_CHARACTERS: UiMessage = UiMessage::new("domain.line_contains_non_utf_8_text_displayed_with_replacement_characters", "{ref_name} 第 {line_number} 行含非 UTF-8 文本，已使用替换字符显示", "{ref_name} line {line_number} contains non-UTF-8 text; displayed with replacement characters");
pub const LINE_HAS_AN_INVALID_FORMAT_SKIPPED: UiMessage = UiMessage::new("domain.line_has_an_invalid_format_skipped", "{ref_name} 第 {line_number} 行格式无效，已跳过：{reason}", "{ref_name} line {line_number} has an invalid format; skipped: {reason}");
pub const LINE_IS_INCOMPLETE_MISSING_LINE_ENDING_SKIPPED: UiMessage = UiMessage::new("domain.line_is_incomplete_missing_line_ending_skipped", "{ref_name} 第 {line_number} 行不完整（缺少行结束符），已跳过", "{ref_name} line {line_number} is incomplete (missing line ending); skipped");
pub const THE_REFLOG_KEPT_CHANGING_DURING_READING_CANNOT_CREATE_A_CONSISTENT_SNAPSHOT_RETRY: UiMessage = UiMessage::new("domain.the_reflog_kept_changing_during_reading_cannot_create_a_consistent_snapshot_retry", "读取期间 Reflog 日志持续变化，无法生成一致快照，请重试", "The reflog kept changing during reading; cannot create a consistent snapshot. Retry");
pub const REFLOG_READING_CANCELLED: UiMessage = UiMessage::new("domain.reflog_reading_cancelled", "Reflog 读取已取消", "Reflog reading cancelled");
pub const HEAD_HAS_NO_SYMBOLIC_REFERENCE_TARGET: UiMessage = UiMessage::new("domain.head_has_no_symbolic_reference_target", "HEAD 缺少符号引用目标", "HEAD has no symbolic reference target");
pub const HEAD_DOES_NOT_POINT_TO_A_LOCAL_BRANCH_THERE_IS_NO_CURRENT_BRANCH_REFLOG_SOURCE: UiMessage = UiMessage::new("domain.head_does_not_point_to_a_local_branch_there_is_no_current_branch_reflog_source", "当前 HEAD 未指向本地分支，没有当前分支日志来源", "HEAD does not point to a local branch; there is no current-branch reflog source");
pub const THE_REFLOG_PATH_IS_NOT_A_REGULAR_FILE: UiMessage = UiMessage::new("domain.the_reflog_path_is_not_a_regular_file", "日志路径不是普通文件", "The reflog path is not a regular file");
pub const RAW_REFLOG_DATA_EXCEEDS_64_MIB_NO_PARTIAL_SNAPSHOT_WAS_GENERATED: UiMessage = UiMessage::new("domain.raw_reflog_data_exceeds_64_mib_no_partial_snapshot_was_generated", "Reflog 原始日志累计超过 64 MiB，未生成部分快照", "Raw reflog data exceeds 64 MiB; no partial snapshot was generated");
pub const THE_REFLOG_EXCEEDS_200000_VALID_ENTRIES_NO_PARTIAL_SNAPSHOT_WAS_GENERATED: UiMessage = UiMessage::new("domain.the_reflog_exceeds_200000_valid_entries_no_partial_snapshot_was_generated", "Reflog 有效条目累计超过 200000 条，未生成部分快照", "The reflog exceeds 200000 valid entries; no partial snapshot was generated");
pub const MISSING_FULL_OIDS_FOR_BOTH_SIDES: UiMessage = UiMessage::new("domain.missing_full_oids_for_both_sides", "缺少完整的两侧 OID", "Missing full OIDs for both sides");
pub const MISSING_TIME_ZONE: UiMessage = UiMessage::new("domain.missing_time_zone", "缺少时区", "Missing time zone");
pub const MISSING_TIMESTAMP: UiMessage = UiMessage::new("domain.missing_timestamp", "缺少时间戳", "Missing timestamp");
pub const MISSING_ACTOR: UiMessage = UiMessage::new("domain.missing_actor", "缺少操作者", "Missing actor");
pub const TIMESTAMP_IS_INVALID_OR_OUT_OF_RANGE: UiMessage = UiMessage::new("domain.timestamp_is_invalid_or_out_of_range", "时间戳无效或超出范围", "Timestamp is invalid or out of range");
pub const INVALID_ACTOR_EMAIL_FORMAT: UiMessage = UiMessage::new("domain.invalid_actor_email_format", "操作者邮箱格式无效", "Invalid actor email format");
pub const INVALID_SEPARATOR_BETWEEN_ACTOR_NAME_AND_EMAIL: UiMessage = UiMessage::new("domain.invalid_separator_between_actor_name_and_email", "操作者姓名与邮箱分隔无效", "Invalid separator between actor name and email");
pub const OID_MUST_BE_A_FULL_40_DIGIT_HEXADECIMAL_VALUE: UiMessage = UiMessage::new("domain.oid_must_be_a_full_40_digit_hexadecimal_value", "OID 必须是完整的 40 位十六进制值", "OID must be a full 40-digit hexadecimal value");
pub const OID_CONTAINS_NON_ASCII_BYTES: UiMessage = UiMessage::new("domain.oid_contains_non_ascii_bytes", "OID 含非 ASCII 字节", "OID contains non-ASCII bytes");
pub const INVALID_OID: UiMessage = UiMessage::new("domain.invalid_oid", "OID 无效", "Invalid OID");
pub const TIME_ZONE_MUST_BE_HHMM: UiMessage = UiMessage::new("domain.time_zone_must_be_hhmm", "时区必须为 +/-HHMM", "Time zone must be +/-HHMM");
pub const TIME_ZONE_IS_OUT_OF_RANGE: UiMessage = UiMessage::new("domain.time_zone_is_out_of_range", "时区超出有效范围", "Time zone is out of range");
pub const DIRECTORY_DOES_NOT_EXIST_CHECK_THE_SPELLING_OF_PATH_PREFIX: UiMessage = UiMessage::new("domain.directory_does_not_exist_check_the_spelling_of_path_prefix", "目录不存在：{prefix}（请检查 path_prefix 拼写）", "Directory does not exist: {prefix} (check the spelling of path_prefix)");
pub const DIRECTORY_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.directory_does_not_exist", "目录不存在：{prefix}", "Directory does not exist: {prefix}");
pub const IS_A_FILE_NOT_A_DIRECTORY_REMOVE_THE_FILE_NAME_AND_KEEP_ONLY_THE_DIRECTORY_PREFIX: UiMessage = UiMessage::new("domain.is_a_file_not_a_directory_remove_the_file_name_and_keep_only_the_directory_prefix", "{prefix} 是文件不是目录，请去掉文件名只保留目录前缀", "{prefix} is a file, not a directory; remove the file name and keep only the directory prefix");
pub const INVALID_REGULAR_EXPRESSION: UiMessage = UiMessage::new("domain.invalid_regular_expression", "正则表达式无效：{err}", "Invalid regular expression: {err}");
pub const SEARCH_TEXT_CANNOT_BE_EMPTY: UiMessage = UiMessage::new("domain.search_text_cannot_be_empty", "搜索内容不能为空", "Search text cannot be empty");
pub const DELETING_STASH: UiMessage = UiMessage::new("domain.deleting_stash", "正在删除贮藏 {stash_ref}", "Deleting {stash_ref}");
pub const DELETED_STASH: UiMessage = UiMessage::new("domain.deleted_stash", "已删除贮藏 {stash_ref}", "Deleted {stash_ref}");
pub const APPLYING_STASH: UiMessage = UiMessage::new("domain.applying_stash", "正在应用贮藏 {stash_ref}", "Applying {stash_ref}");
pub const APPLIED_STASH: UiMessage = UiMessage::new("domain.applied_stash", "已应用贮藏 {stash_ref}", "Applied {stash_ref}");
pub const POPPING_STASH: UiMessage = UiMessage::new("domain.popping_stash", "正在弹出贮藏 {stash_ref}", "Popping {stash_ref}");
pub const POPPED_STASH: UiMessage = UiMessage::new("domain.popped_stash", "已弹出贮藏 {stash_ref}", "Popped {stash_ref}");
pub const STASH_DOES_NOT_EXIST_STASH: UiMessage = UiMessage::new("domain.stash_does_not_exist_stash", "贮藏不存在：{stash_ref}", "Stash does not exist: {stash_ref}");
pub const CANNOT_CREATE_A_STASH_WHILE_CONFLICTS_EXIST: UiMessage = UiMessage::new("domain.cannot_create_a_stash_while_conflicts_exist", "存在冲突，无法创建贮藏", "Cannot create a stash while conflicts exist");
pub const ONLY_UNTRACKED_FILES_EXIST_SELECT_INCLUDE_UNTRACKED_FILES_TO_STASH_THEM: UiMessage = UiMessage::new("domain.only_untracked_files_exist_select_include_untracked_files_to_stash_them", "当前只有未跟踪文件；如需贮藏请勾选“包含未跟踪文件”", "Only untracked files exist; select Include untracked files to stash them");
pub const THERE_ARE_NO_CHANGES_TO_STASH: UiMessage = UiMessage::new("domain.there_are_no_changes_to_stash", "当前没有可贮藏的更改", "There are no changes to stash");
pub const STASHING_CURRENT_CHANGES: UiMessage = UiMessage::new("domain.stashing_current_changes", "正在贮藏当前修改", "Stashing current changes");
pub const CURRENT_CHANGES_STASHED: UiMessage = UiMessage::new("domain.current_changes_stashed", "已贮藏当前修改", "Current changes stashed");
pub const UPDATING_SUBMODULE_TO_THE_LATEST_REMOTE_REVISION: UiMessage = UiMessage::new("domain.updating_submodule_to_the_latest_remote_revision", "正在更新子模块 {name} 到远端最新", "Updating submodule {name} to the latest remote revision");
pub const SUBMODULE_UPDATED_TO_THE_LATEST_REMOTE_REVISION: UiMessage = UiMessage::new("domain.submodule_updated_to_the_latest_remote_revision", "子模块 {name} 已更新到远端最新", "Submodule {name} updated to the latest remote revision");
pub const SYNCHRONIZING_SUBMODULE: UiMessage = UiMessage::new("domain.synchronizing_submodule", "正在同步子模块 {name}", "Synchronizing submodule {name}");
pub const SYNCHRONIZING_THE_RECORDED_REVISION_OF_SUBMODULE: UiMessage = UiMessage::new("domain.synchronizing_the_recorded_revision_of_submodule", "正在同步子模块 {name} 的记录版本", "Synchronizing the recorded revision of submodule {name}");
pub const CANNOT_RESOLVE_CURRENT_HEAD_OF_SUBMODULE: UiMessage = UiMessage::new("domain.cannot_resolve_current_head_of_submodule", "子模块 {name} 当前 HEAD 无法解析", "Cannot resolve current HEAD of submodule {name}");
pub const REMOTE_BRANCH_NOT_FOUND_IN_SUBMODULE: UiMessage = UiMessage::new("domain.remote_branch_not_found_in_submodule", "子模块 {name} 未找到远端分支 {remote_ref}：{err}", "Remote branch {remote_ref} not found in submodule {name}: {err}");
pub const THE_REMOTE_BRANCH_OF_SUBMODULE_IS_NOT_A_DIRECT_REFERENCE: UiMessage = UiMessage::new("domain.the_remote_branch_of_submodule_is_not_a_direct_reference", "子模块 {name} 的远端分支不是直接引用", "The remote branch of submodule {name} is not a direct reference");
pub const CANNOT_RESOLVE_CURRENT_HEAD_OF_SUBMODULE_2: UiMessage = UiMessage::new("domain.cannot_resolve_current_head_of_submodule_2", "子模块 {value0} 当前 HEAD 无法解析", "Cannot resolve current HEAD of submodule {value0}");
pub const SUBMODULE_CANNOT_FAST_FORWARD_TO_THE_LATEST_REMOTE_REVISION_RESOLVE_DIVERGENCE_OR_LOC: UiMessage = UiMessage::new("domain.submodule_cannot_fast_forward_to_the_latest_remote_revision_resolve_divergence_or_loc", "子模块 {value0} 不能快进到远端最新，请先手动处理分叉或本地提交", "Submodule {value0} cannot fast-forward to the latest remote revision; resolve divergence or local commits manually first");
pub const SUBMODULE_HAS_LOCAL_CHANGES_COMMIT_STASH_OR_DISCARD_THEM_IN_THE_SUBMODULE_BEFORE_UPDA: UiMessage = UiMessage::new("domain.submodule_has_local_changes_commit_stash_or_discard_them_in_the_submodule_before_upda", "子模块 {value0} 有本地改动，请先在子模块中提交、贮藏或清理后再更新", "Submodule {value0} has local changes; commit, stash or discard them in the submodule before updating");
pub const SUBMODULE_HAS_BRANCH_BUT_THE_PARENT_REPOSITORY_IS_NOT_ON_A_LOCAL_BRANCH: UiMessage = UiMessage::new("domain.submodule_has_branch_but_the_parent_repository_is_not_on_a_local_branch", "子模块 {submodule_name} 配置 branch = .，但父仓库当前不是本地分支", "Submodule {submodule_name} has branch = ., but the parent repository is not on a local branch");
pub const SUBMODULE_HAS_NO_TRACKING_BRANCH_AND_ITS_REMOTE_DEFAULT_BRANCH_CANNOT_BE_DETERMINED: UiMessage = UiMessage::new("domain.submodule_has_no_tracking_branch_and_its_remote_default_branch_cannot_be_determined", "子模块 {submodule_name} 未配置跟踪分支，且无法确定远端默认分支", "Submodule {submodule_name} has no tracking branch and its remote default branch cannot be determined");
pub const CANNOT_DETERMINE_THE_REMOTE_FOR_SUBMODULE_KEEP_ORIGIN_OR_CONFIGURE_ONLY_ONE_REMOTE: UiMessage = UiMessage::new("domain.cannot_determine_the_remote_for_submodule_keep_origin_or_configure_only_one_remote", "子模块 {submodule_name} 无法确定远端，请保留 origin 或只配置一个远端", "Cannot determine the remote for submodule {submodule_name}; keep origin or configure only one remote");
pub const SYNCHRONIZING_RECORDED_SUBMODULE_REVISIONS: UiMessage = UiMessage::new("domain.synchronizing_recorded_submodule_revisions", "正在同步子模块记录版本", "Synchronizing recorded submodule revisions");
pub const RECORDED_SUBMODULE_REVISIONS_SYNCHRONIZED: UiMessage = UiMessage::new("domain.recorded_submodule_revisions_synchronized", "子模块已同步记录版本", "Recorded submodule revisions synchronized");
pub const UPDATING_SUBMODULES_TO_THE_LATEST_REMOTE_REVISIONS: UiMessage = UiMessage::new("domain.updating_submodules_to_the_latest_remote_revisions", "正在更新子模块到远端最新", "Updating submodules to the latest remote revisions");
pub const SUBMODULES_UPDATED_TO_THE_LATEST_REMOTE_REVISIONS: UiMessage = UiMessage::new("domain.submodules_updated_to_the_latest_remote_revisions", "子模块已更新到远端最新", "Submodules updated to the latest remote revisions");
pub const SUBMODULE_IS_NOT_INITIALIZED: UiMessage = UiMessage::new("domain.submodule_is_not_initialized", "子模块未初始化", "Submodule is not initialized");
pub const SUBMODULE_IS_NOT_CHECKED_OUT: UiMessage = UiMessage::new("domain.submodule_is_not_checked_out", "子模块未检出", "Submodule is not checked out");
pub const STASH_CREATED_BUT_CLEANING_THE_WORKING_TREE_FAILED_RELEASE_LOCKED_FILES_AND_RETRY_OR: UiMessage = UiMessage::new("domain.stash_created_but_cleaning_the_working_tree_failed_release_locked_files_and_retry_or", "贮藏已创建（{short_oid}），但清理工作区失败：{err}；可先处理占用后重试或手动丢弃工作区改动", "Stash created ({short_oid}), but cleaning the working tree failed: {err}; release locked files and retry, or discard working tree changes manually");
pub const CANNOT_LOCATE_THE_CONFLICT_BLOCK_IN_THE_ORIGINAL_VERSION: UiMessage = UiMessage::new("domain.cannot_locate_the_conflict_block_in_the_original_version", "无法定位原始版本中的冲突块", "Cannot locate the conflict block in the original version");
pub const THE_ORIGINAL_VERSION_DOES_NOT_MATCH_THE_CONFLICT_BLOCK_REFRESH_THE_REPOSITORY: UiMessage = UiMessage::new("domain.the_original_version_does_not_match_the_conflict_block_refresh_the_repository", "原始版本与冲突块内容不匹配，请刷新仓库", "The original version does not match the conflict block; refresh the repository");
pub const FAILED_TO_CREATE_THE_FIXED_DATA_DIRECTORY: UiMessage = UiMessage::new("domain.failed_to_create_the_fixed_data_directory", "创建固定数据目录失败：{err}", "Failed to create the fixed data directory: {err}");
pub const FAILED_TO_WRITE_THE_RELOCATION_MARKER: UiMessage = UiMessage::new("domain.failed_to_write_the_relocation_marker", "写入搬迁标记失败：{err}", "Failed to write the relocation marker: {err}");
pub const FAILED_TO_START_THE_NEW_INSTANCE: UiMessage = UiMessage::new("domain.failed_to_start_the_new_instance", "启动新实例失败：{err}", "Failed to start the new instance: {err}");
pub const FAILED_TO_PARSE_AI_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_parse_ai_settings", "AI 配置解析失败：{err}", "Failed to parse AI settings: {err}");
pub const FAILED_TO_SERIALIZE_AI_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_serialize_ai_settings", "AI 配置序列化失败：{err}", "Failed to serialize AI settings: {err}");
pub const FAILED_TO_PARSE_EXTERNAL_MERGE_TOOL_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_parse_external_merge_tool_settings", "外部合并工具配置解析失败：{err}", "Failed to parse external merge tool settings: {err}");
pub const FAILED_TO_SERIALIZE_EXTERNAL_MERGE_TOOL_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_serialize_external_merge_tool_settings", "外部合并工具配置序列化失败：{err}", "Failed to serialize external merge tool settings: {err}");
pub const FAILED_TO_PARSE_SHORTCUT_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_parse_shortcut_settings", "快捷键配置解析失败：{err}", "Failed to parse shortcut settings: {err}");
pub const FAILED_TO_SERIALIZE_SHORTCUT_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_serialize_shortcut_settings", "快捷键配置序列化失败：{err}", "Failed to serialize shortcut settings: {err}");
pub const FAILED_TO_PARSE_WORKFLOW_SHORTCUT_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_parse_workflow_shortcut_settings", "工作流快捷键配置解析失败：{err}", "Failed to parse workflow shortcut settings: {err}");
pub const FAILED_TO_SERIALIZE_WORKFLOW_SHORTCUT_SETTINGS: UiMessage = UiMessage::new("domain.failed_to_serialize_workflow_shortcut_settings", "工作流快捷键配置序列化失败：{err}", "Failed to serialize workflow shortcut settings: {err}");
pub const FAILED_TO_PARSE_LAYOUT_PREFERENCES: UiMessage = UiMessage::new("domain.failed_to_parse_layout_preferences", "布局偏好解析失败：{err}", "Failed to parse layout preferences: {err}");
pub const FAILED_TO_SERIALIZE_LAYOUT_PREFERENCES: UiMessage = UiMessage::new("domain.failed_to_serialize_layout_preferences", "布局偏好序列化失败：{err}", "Failed to serialize layout preferences: {err}");
pub const LOCAL_SETTINGS_DATABASE_ERROR: UiMessage = UiMessage::new("domain.local_settings_database_error", "本地配置数据库错误：{err}", "Local settings database error: {err}");
pub const UNKNOWN_PROXY_MODE: UiMessage = UiMessage::new("domain.unknown_proxy_mode", "未知代理模式：{value}", "Unknown proxy mode: {value}");
pub const UNKNOWN_THEME_MODE: UiMessage = UiMessage::new("domain.unknown_theme_mode", "未知主题模式：{value}", "Unknown theme mode: {value}");
pub const UNKNOWN_DIFF_ENCODING_PREFERENCE: UiMessage = UiMessage::new("domain.unknown_diff_encoding_preference", "未知 diff 编码偏好：{value}", "Unknown diff encoding preference: {value}");
pub const UNKNOWN_REMOTE_CREDENTIAL_POLICY: UiMessage = UiMessage::new("domain.unknown_remote_credential_policy", "未知远端凭据策略：{kind}", "Unknown remote credential policy: {kind}");
pub const UNKNOWN_CREDENTIAL_SCOPE: UiMessage = UiMessage::new("domain.unknown_credential_scope", "未知凭据作用域：{value}", "Unknown credential scope: {value}");
pub const UNKNOWN_CREDENTIAL_TYPE: UiMessage = UiMessage::new("domain.unknown_credential_type", "未知凭据类型：{value}", "Unknown credential type: {value}");
pub const CANNOT_LOCATE_THE_LOCAL_SETTINGS_DATABASE_DIRECTORY: UiMessage = UiMessage::new("domain.cannot_locate_the_local_settings_database_directory", "无法定位本地配置数据库目录", "Cannot locate the local settings database directory");
pub const THE_LOCAL_SETTINGS_DATABASE_IS_IN_AN_INVALID_STATE: UiMessage = UiMessage::new("domain.the_local_settings_database_is_in_an_invalid_state", "本地配置数据库状态异常", "The local settings database is in an invalid state");
pub const CANNOT_LOCATE_THE_LEGACY_DATA_DIRECTORY: UiMessage = UiMessage::new("domain.cannot_locate_the_legacy_data_directory", "无法定位旧数据目录", "Cannot locate the legacy data directory");
pub const CANNOT_LOCATE_THE_PORTABLE_DATA_DIRECTORY: UiMessage = UiMessage::new("domain.cannot_locate_the_portable_data_directory", "无法定位便携数据目录", "Cannot locate the portable data directory");
pub const THE_LEGACY_DATABASE_FILE_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.the_legacy_database_file_does_not_exist", "旧数据库文件不存在", "The legacy database file does not exist");
pub const CANNOT_LOCATE_THE_FIXED_DATA_DIRECTORY: UiMessage = UiMessage::new("domain.cannot_locate_the_fixed_data_directory", "无法定位固定数据目录", "Cannot locate the fixed data directory");
pub const THE_RELOCATION_MARKER_IS_EMPTY: UiMessage = UiMessage::new("domain.the_relocation_marker_is_empty", "搬迁标记内容为空", "The relocation marker is empty");
pub const THE_REMOTE_CREDENTIAL_BINDING_IS_MISSING_A_RECORD_ID: UiMessage = UiMessage::new("domain.the_remote_credential_binding_is_missing_a_record_id", "远端凭据绑定缺少记录 ID", "The remote credential binding is missing a record ID");
pub const THE_SYSTEM_CREDENTIAL_MANAGER_IS_UNAVAILABLE: UiMessage = UiMessage::new("domain.the_system_credential_manager_is_unavailable", "系统凭据管理器不可用：{err}", "The system credential manager is unavailable: {err}");
pub const FAILED_TO_CREATE_A_SYSTEM_CREDENTIAL_ENTRY: UiMessage = UiMessage::new("domain.failed_to_create_a_system_credential_entry", "系统凭据条目创建失败：{err}", "Failed to create a system credential entry: {err}");
pub const FAILED_TO_READ_SYSTEM_CREDENTIALS: UiMessage = UiMessage::new("domain.failed_to_read_system_credentials", "系统凭据读取失败：{err}", "Failed to read system credentials: {err}");
pub const FAILED_TO_WRITE_SYSTEM_CREDENTIALS: UiMessage = UiMessage::new("domain.failed_to_write_system_credentials", "系统凭据写入失败：{err}", "Failed to write system credentials: {err}");
pub const FAILED_TO_DELETE_SYSTEM_CREDENTIALS: UiMessage = UiMessage::new("domain.failed_to_delete_system_credentials", "系统凭据删除失败：{err}", "Failed to delete system credentials: {err}");
pub const FAILED_TO_UPDATE_SYSTEM_CREDENTIALS: UiMessage = UiMessage::new("domain.failed_to_update_system_credentials", "系统凭据更新失败：{err}", "Failed to update system credentials: {err}");
pub const BUILT_IN_GIT_TEST_FAILED_THE_SYSTEM_GIT_OPENSSH_CHECK_SUCCEEDED_BUT_IN_APP_PUSH_AND_P: UiMessage = UiMessage::new("domain.built_in_git_test_failed_the_system_git_openssh_check_succeeded_but_in_app_push_and_p", "内置 Git 测试失败：{libgit2_error}；系统 Git/OpenSSH 复核成功，但应用内推送和拉取仍无法使用此连接", "Built-in Git test failed: {libgit2_error}; the system Git/OpenSSH check succeeded, but in-app push and pull still cannot use this connection");
pub const LIBGIT2_TEST_FAILED_SYSTEM_GIT_OPENSSH_CHECK_FAILED: UiMessage = UiMessage::new("domain.libgit2_test_failed_system_git_openssh_check_failed", "libgit2 测试失败：{libgit2_error}；系统 Git/OpenSSH 复核失败：{system_error}", "libgit2 test failed: {libgit2_error}; system Git/OpenSSH check failed: {system_error}");
pub const CANNOT_START_SYSTEM_GIT: UiMessage = UiMessage::new("domain.cannot_start_system_git", "无法启动系统 Git：{error}", "Cannot start system Git: {error}");
pub const SYSTEM_GIT_EXITED_WITH_STATUS: UiMessage = UiMessage::new("domain.system_git_exited_with_status", "系统 Git 返回状态 {value0}", "System Git exited with status {value0}");
pub const FAILED_TO_CREATE_A_GITEE_REFRESH_ENTRY: UiMessage = UiMessage::new("domain.failed_to_create_a_gitee_refresh_entry", "Gitee 续期条目创建失败：{err}", "Failed to create a Gitee refresh entry: {err}");
pub const FAILED_TO_SERIALIZE_GITEE_REFRESH_DATA: UiMessage = UiMessage::new("domain.failed_to_serialize_gitee_refresh_data", "Gitee 续期材料序列化失败：{err}", "Failed to serialize Gitee refresh data: {err}");
pub const FAILED_TO_SAVE_GITEE_REFRESH_DATA: UiMessage = UiMessage::new("domain.failed_to_save_gitee_refresh_data", "Gitee 续期材料保存失败：{err}", "Failed to save Gitee refresh data: {err}");
pub const THE_CREDENTIAL_CACHE_IS_IN_AN_INVALID_STATE: UiMessage = UiMessage::new("domain.the_credential_cache_is_in_an_invalid_state", "凭据缓存状态异常", "The credential cache is in an invalid state");
pub const CANNOT_PARSE_THE_REMOTE_URL_CREDENTIALS_CANNOT_BE_SAVED: UiMessage = UiMessage::new("domain.cannot_parse_the_remote_url_credentials_cannot_be_saved", "无法解析远端地址，不能保存凭据", "Cannot parse the remote URL; credentials cannot be saved");
pub const THE_CREDENTIAL_RECORD_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.the_credential_record_does_not_exist", "凭据记录不存在", "The credential record does not exist");
pub const CANNOT_PARSE_THE_REMOTE_URL_CREDENTIALS_CANNOT_BE_BOUND: UiMessage = UiMessage::new("domain.cannot_parse_the_remote_url_credentials_cannot_be_bound", "无法解析远端地址，不能绑定凭据", "Cannot parse the remote URL; credentials cannot be bound");
pub const THE_SYSTEM_CREDENTIAL_MANAGER_INITIALIZATION_STATE_IS_INVALID: UiMessage = UiMessage::new("domain.the_system_credential_manager_initialization_state_is_invalid", "系统凭据管理器初始化状态异常", "The system credential manager initialization state is invalid");
pub const THE_CREDENTIAL_INDEX_LOCK_IS_IN_AN_INVALID_STATE: UiMessage = UiMessage::new("domain.the_credential_index_lock_is_in_an_invalid_state", "凭据索引锁状态异常", "The credential index lock is in an invalid state");
pub const THE_ENCRYPTED_CREDENTIAL_WAS_NOT_FOUND_IN_THE_SYSTEM_CREDENTIAL_MANAGER: UiMessage = UiMessage::new("domain.the_encrypted_credential_was_not_found_in_the_system_credential_manager", "系统凭据管理器中未找到该凭据密文", "The encrypted credential was not found in the system credential manager");
pub const SSH_HTTPS_CREDENTIALS_WERE_ALREADY_TRIED_THE_REMOTE_REJECTED_AUTHENTICATION: UiMessage = UiMessage::new("domain.ssh_https_credentials_were_already_tried_the_remote_rejected_authentication", "SSH/HTTPS 凭据已尝试，远端拒绝认证", "SSH/HTTPS credentials were already tried; the remote rejected authentication");
pub const THE_SELECTED_RECORD_IS_NOT_AN_SSH_CREDENTIAL: UiMessage = UiMessage::new("domain.the_selected_record_is_not_an_ssh_credential", "所选记录不是 SSH 凭据", "The selected record is not an SSH credential");
pub const THE_REMOTE_DOES_NOT_ACCEPT_SSH_KEY_CREDENTIALS: UiMessage = UiMessage::new("domain.the_remote_does_not_accept_ssh_key_credentials", "远端不接受 SSH 密钥凭据", "The remote does not accept SSH key credentials");
pub const THIS_REMOTE_ONLY: UiMessage = UiMessage::new("domain.this_remote_only", "仅此远端", "This remote only");
pub const SAME_HOST: UiMessage = UiMessage::new("domain.same_host", "同站点", "Same host");
pub const HTTPS_USERNAME_PAT: UiMessage = UiMessage::new("domain.https_username_pat", "HTTPS 用户名/PAT", "HTTPS username/PAT");
pub const SSH_PRIVATE_KEY: UiMessage = UiMessage::new("domain.ssh_private_key", "SSH 私钥", "SSH private key");
pub const USING_CUSTOM_PROXY: UiMessage = UiMessage::new("domain.using_custom_proxy", "使用自定义代理 {url}", "Using custom proxy {url}");
pub const NO_PROXY: UiMessage = UiMessage::new("domain.no_proxy", "不使用代理", "No proxy");
pub const USING_A_PROXY_FROM_GIT_CONFIGURATION_OR_ENVIRONMENT_VARIABLES: UiMessage = UiMessage::new("domain.using_a_proxy_from_git_configuration_or_environment_variables", "使用 Git 配置或环境变量代理", "Using a proxy from Git configuration or environment variables");
pub const NO_MATCHING_CUSTOM_PROXY_CONNECTING_DIRECTLY: UiMessage = UiMessage::new("domain.no_matching_custom_proxy_connecting_directly", "未匹配到自定义代理，直连", "No matching custom proxy; connecting directly");
pub const PROXY_URLS_CANNOT_CONTAIN_CONTROL_CHARACTERS_OR_WHITESPACE: UiMessage = UiMessage::new("domain.proxy_urls_cannot_contain_control_characters_or_whitespace", "代理地址不能包含控制字符或空白", "Proxy URLs cannot contain control characters or whitespace");
pub const UNSUPPORTED_PROXY_URL_SCHEME_USE_HTTP_HTTPS_SOCKS5_OR_SOCKS5H: UiMessage = UiMessage::new("domain.unsupported_proxy_url_scheme_use_http_https_socks5_or_socks5h", "代理地址协议不支持，请使用 http://、https://、socks5:// 或 socks5h://", "Unsupported proxy URL scheme; use http://, https://, socks5:// or socks5h://");
pub const UNSUPPORTED_UPDATE_MANIFEST_FORMAT_SCHEMA_ONLY_SCHEMA_1_IS_SUPPORTED: UiMessage = UiMessage::new("domain.unsupported_update_manifest_format_schema_only_schema_1_is_supported", "不支持的更新清单格式（schema={value0}），当前仅支持 schema=1", "Unsupported update manifest format (schema={value0}); only schema=1 is supported");
pub const INVALID_VERSION_IN_THE_UPDATE_MANIFEST: UiMessage = UiMessage::new("domain.invalid_version_in_the_update_manifest", "更新清单版本号格式错误：{err}", "Invalid version in the update manifest: {err}");
pub const INVALID_SKIPPED_VERSION: UiMessage = UiMessage::new("domain.invalid_skipped_version", "已跳过版本号格式错误：{err}", "Invalid skipped version: {err}");
pub const FAILED_TO_EXTRACT_THE_UPDATE_PACKAGE: UiMessage = UiMessage::new("domain.failed_to_extract_the_update_package", "更新包解压失败：{err}", "Failed to extract the update package: {err}");
pub const FAILED_TO_READ_AN_UPDATE_PACKAGE_ENTRY: UiMessage = UiMessage::new("domain.failed_to_read_an_update_package_entry", "更新包读取条目失败：{err}", "Failed to read an update package entry: {err}");
pub const FAILED_TO_PARSE_THE_UPDATE_MANIFEST: UiMessage = UiMessage::new("domain.failed_to_parse_the_update_manifest", "更新清单解析失败（{source}）：{err}", "Failed to parse the update manifest ({source}): {err}");
pub const FAILED_TO_DOWNLOAD_THE_UPDATE_MANIFEST: UiMessage = UiMessage::new("domain.failed_to_download_the_update_manifest", "更新清单下载失败（{source}）：{err}", "Failed to download the update manifest ({source}): {err}");
pub const INVALID_PROXY_CONFIGURATION_UPDATE_REQUEST_CANCELLED: UiMessage = UiMessage::new("domain.invalid_proxy_configuration_update_request_cancelled", "代理配置无效，已取消更新请求：{err}", "Invalid proxy configuration; update request cancelled: {err}");
pub const FAILED_TO_DOWNLOAD_THE_UPDATE_PACKAGE: UiMessage = UiMessage::new("domain.failed_to_download_the_update_package", "下载更新包失败（{url}）：{err}", "Failed to download the update package ({url}): {err}");
pub const INVALID_UPDATE_PACKAGE_SIZE_BYTES_LIMIT_DOWNLOAD_CANCELLED: UiMessage = UiMessage::new("domain.invalid_update_package_size_bytes_limit_download_cancelled", "更新包体积异常（{content_length} 字节，上限 {MAX_DOWNLOAD_BYTES}），已取消下载", "Invalid update package size ({content_length} bytes; limit {MAX_DOWNLOAD_BYTES}); download cancelled");
pub const UPDATE_PACKAGE_DOWNLOAD_EXCEEDED_THE_SIZE_LIMIT_BYTES_CANCELLED: UiMessage = UiMessage::new("domain.update_package_download_exceeded_the_size_limit_bytes_cancelled", "更新包下载超出体积上限（{MAX_DOWNLOAD_BYTES} 字节），已取消", "Update package download exceeded the size limit ({MAX_DOWNLOAD_BYTES} bytes); cancelled");
pub const STABLE: UiMessage = UiMessage::new("domain.stable", "正式版", "Stable");
pub const BETA: UiMessage = UiMessage::new("domain.beta", "测试版", "Beta");
pub const AUTOMATIC_UPDATES_ARE_NOT_SUPPORTED_ON_THIS_PLATFORM_MISSING_WINDOWS_X86_64_DOWNLOAD: UiMessage = UiMessage::new("domain.automatic_updates_are_not_supported_on_this_platform_missing_windows_x86_64_download", "当前平台不支持自动更新（缺少 windows-x86_64 下载项）", "Automatic updates are not supported on this platform (missing windows-x86_64 download)");
pub const FAILED_TO_DOWNLOAD_THE_UPDATE_PACKAGE_NO_AVAILABLE_SOURCES: UiMessage = UiMessage::new("domain.failed_to_download_the_update_package_no_available_sources", "下载更新包失败：无可用源", "Failed to download the update package: no available sources");
pub const THE_UPDATE_PACKAGE_IS_MISSING_KHASLANA_EXE: UiMessage = UiMessage::new("domain.the_update_package_is_missing_khaslana_exe", "更新包缺少 khaslana.exe", "The update package is missing khaslana.exe");
pub const THE_UPDATE_PACKAGE_IS_MISSING_KHASLANA_UPDATER_EXE: UiMessage = UiMessage::new("domain.the_update_package_is_missing_khaslana_updater_exe", "更新包缺少 khaslana_updater.exe", "The update package is missing khaslana_updater.exe");
pub const CANNOT_FETCH_THE_UPDATE_MANIFEST: UiMessage = UiMessage::new("domain.cannot_fetch_the_update_manifest", "无法获取更新清单", "Cannot fetch the update manifest");
pub const UNKNOWN_REASON: UiMessage = UiMessage::new("domain.unknown_reason", "未知原因", "Unknown reason");
pub const NOT_INITIALIZED: UiMessage = UiMessage::new("domain.not_initialized", "未初始化", "Not initialized");
pub const NOT_CHECKED_OUT: UiMessage = UiMessage::new("domain.not_checked_out", "未检出", "Not checked out");
pub const MODIFIED: UiMessage = UiMessage::new("domain.modified", "有改动", "Modified");
pub const HAS_UNTRACKED_FILES: UiMessage = UiMessage::new("domain.has_untracked_files", "有未跟踪文件", "Has untracked files");
pub const UPDATE_NEEDED: UiMessage = UiMessage::new("domain.update_needed", "需更新", "Update needed");
pub const UP_TO_DATE: UiMessage = UiMessage::new("domain.up_to_date", "已同步", "Up to date");
pub const AUTO: UiMessage = UiMessage::new("domain.auto", "自动", "Auto");
pub const CURRENT_BRANCH: UiMessage = UiMessage::new("domain.current_branch", "当前分支", "Current branch");
pub const ALL_BRANCHES: UiMessage = UiMessage::new("domain.all_branches", "所有分支", "All branches");

pub const COMMIT_OBJECT_AVAILABLE: UiMessage = UiMessage::new("domain.commit_object_available", "提交对象可用", "Commit object available");

pub const NO_COMMIT_ON_THIS_SIDE: UiMessage = UiMessage::new("domain.no_commit_on_this_side", "此侧没有提交", "No commit on this side");

pub const LOCAL_COMMIT_OBJECT_UNAVAILABLE: UiMessage = UiMessage::new("domain.local_commit_object_unavailable", "当前本地提交对象不可用", "Local commit object unavailable");

pub const THIS_OBJECT_IS_NOT_A_COMMIT: UiMessage = UiMessage::new("domain.this_object_is_not_a_commit", "该对象不是提交", "This object is not a commit");

pub const FAILED_TO_READ_COMMIT_OBJECT: UiMessage = UiMessage::new("domain.failed_to_read_commit_object", "提交对象读取失败", "Failed to read commit object");

/// 领域资源注册表用于验证身份唯一及双语参数一致，不用于原文查表。
pub const ALL: &[UiMessage] = &[
    COVERAGE_NON_UTF8_PATH,
    COVERAGE_FILTERED_SNAPSHOT_ENTRY,
    TOO_MANY_STRINGS_IN_CALL_RECORDS,
    CALL_RECORDS_ARE_TOO_LARGE,
    INCOMPATIBLE_CALL_RECORD_FORMAT,
    TRUNCATED_CALL_RECORDS,
    CORRUPT_CALL_RECORD_DICTIONARY,
    TRUNCATED_CALL_RECORD_STRING,
    CORRUPT_CALL_RECORD_ENCODING,
    CORRUPT_CALL_RECORD_DICTIONARY_REFERENCE,
    CORRUPT_CALL_RECORD_IMPORT_TABLE,
    CORRUPT_CALL_RECORD_LIST,
    CALL_RECORDS_CONTAIN_TRAILING_DATA,
    FAILED_TO_START_THE_COVERAGE_QUERY,
    FAILED_TO_READ_COVERAGE_INFORMATION,
    FAILED_TO_FINISH_THE_COVERAGE_QUERY,
    UNRESOLVED_CALLS_INCLUDE_CALLS_WITH_NO_SAME_LANGUAGE_REPOSITORY_DEFINITION_POSSIBLY_E,
    PROVIDE_AT_LEAST_PATHS_OR_SCOPES,
    COVERAGE_PATHS_MUST_BE_RELATIVE_PATHS_WITHIN_THE_REPOSITORY,
    THE_CODE_INDEX_HAS_NOT_BEEN_BUILT,
    METADATA_MATCHES_ONLY_CONFIRMS_MATCHING_TIMESTAMPS_AND_SIZES_UNRESOLVED_NO_CANDIDATE,
    CANNOT_SCAN_THE_REPOSITORY_DIRECTORY,
    REPOSITORY_FILE_COUNT_EXCEEDS_THE_INDEXING_LIMIT_OF_INDEXING_STOPPED,
    FAILED_TO_SCAN_THE_REPOSITORY_THE_PREVIOUS_INDEX_WAS_RETAINED,
    FAILED_TO_INITIALIZE_THE_PARSER,
    REPOSITORY_PATH_UNAVAILABLE,
    IS_NOT_A_GIT_REPOSITORY,
    REPOSITORY_WORKING_TREE_UNAVAILABLE,
    FAILED_TO_READ_THE_REPOSITORY_PATH,
    THE_REPO_ARGUMENT_IS_NEITHER_AN_EXISTING_REPOSITORY_PATH_NOR_A_KNOWN_INDEX_DATABASE,
    THE_REPO_ARGUMENT_WAS_OMITTED_BUT_INDEXED_REPOSITORIES_EXIST_LOCALLY,
    INDEXING_COMPLETE_FILES_SYMBOLS_EDGES,
    UNKNOWN_TOOL,
    UNKNOWN_TOOL_2,
    MISSING_REQUIRED_ARGUMENT,
    FAILED_TO_SERIALIZE_DETAILS,
    THERE_ARE_DEFINITIONS_WITH_THIS_NAME_QUERY_AN_EXACT_QUALIFIED_NAME,
    FAILED_TO_SERIALIZE_RESULTS,
    THERE_ARE_CALLABLE_DEFINITIONS_WITH_THIS_NAME_QUERY_AN_EXACT_QUALIFIED_NAME,
    SERIALIZATION_FAILED,
    MUST_BE_AN_ARRAY_OF_STRINGS,
    UNKNOWN_MODE_EXPECTED_INCREMENTAL_OR_FULL,
    INVALID_TYPE_FOR_EXPECTED,
    IS_NOT_ONE_OF_THE_ALLOWED_VALUES,
    MUST_NOT_BE_LESS_THAN,
    MUST_NOT_EXCEED,
    MISSING_REQUIRED_ARGUMENT_2,
    IS_MISSING_REQUIRED_QUERY_CRITERIA_SEE_TOOLS_LIST,
    THE_DATA_DIRECTORY_IS_UNAVAILABLE_CANNOT_LOCATE_THE_INDEX_DATABASE,
    A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_BUILD_A_WORKING_TREE_CODE_INDEX,
    THE_INDEX_DATABASE_IS_UNAVAILABLE_OR_ITS_VERSION_IS_INCOMPATIBLE,
    PROVIDE_AN_ABSOLUTE_REPOSITORY_PATH_OR_CALL_LIST_PROJECTS_TO_SEE_AVAILABLE_REPOSITORI,
    NO_REPOSITORIES_HAVE_BEEN_INDEXED_YET,
    CALL_REFRESH_INDEX_WITH_REPO_SET_TO_AN_ABSOLUTE_REPOSITORY_PATH_TO_BUILD_THE_INDEX,
    PROVIDE_REPO_AS_AN_ABSOLUTE_REPOSITORY_PATH_OR_ONE_OF_THE_REPO_KEYS_ABOVE_CALL_LIST_P,
    INDEX_UNCHANGED,
    INDEXING_CANCELLED_THE_PREVIOUS_INDEX_WAS_RETAINED,
    SEE_TOOLS_LIST_FOR_AVAILABLE_TOOLS,
    THE_INDEX_WAS_REFRESHED_THE_PREVIOUS_PAGINATION_GENERATION_IS_NO_LONGER_VALID,
    REMOVE_GENERATION_AND_RESTART_THE_QUERY_AT_OFFSET_0,
    THE_INDEX_CHANGED_DURING_THE_QUERY,
    RESTART_THE_QUERY_AT_OFFSET_0,
    THE_INDEX_IS_BEING_BUILT_OR_REFRESHED_IN_THE_BACKGROUND_RETRY_LATER_AND_USE_INDEX_STA,
    SEE_THIS_TOOL_S_INPUTSCHEMA_IN_TOOLS_LIST,
    QUERY_OR_NAME_PATTERN_QN_PATTERN_FILE_PATTERN_LABEL,
    OFFSET_EXCEEDS_THE_TOTAL_RESULT_COUNT_RESTART_THE_QUERY_AT_OFFSET_0,
    NO_MATCHES_SHORTEN_QUERY_OR_USE_SEARCH_GRAPH_WITH_A_NAME_PATTERN_REGEX_OR_FILE_PATTER,
    CANNOT_READ_THE_SOURCE_FILE_MAKE_SURE_REPO_IS_AN_EXISTING_ABSOLUTE_REPOSITORY_PATH_RE,
    USE_SEARCH_SYMBOLS_FIRST_TO_FIND_THE_CORRECT_NAME,
    TOTAL_COUNTS_DISTINCT_NODES_WITHIN_THE_REQUESTED_DEPTH_NOT_CALLS_FOR_HOP_1_CALL_SITES,
    THIS_INDEX_DATABASE_HAS_NO_REPOSITORY_PATH_METADATA_CANNOT_READ_WORKING_TREE_STATUS,
    SET_REPO_TO_AN_ABSOLUTE_REPOSITORY_PATH_OR_CALL_REFRESH_INDEX_WITH_THAT_PATH_TO_RESTO,
    NO_CHANGED_FILES_DETECTED_BETWEEN_THE_WORKING_TREE_AND_THE_BASELINE,
    THE_INDEX_NEEDS_REBUILDING_FOR_AN_UPGRADE_WAIT_FOR_THE_BACKGROUND_REFRESH_OR_CALL_REF,
    THE_INDEX_USES_TREE_SITTER_AND_HEURISTIC_CALL_RESOLUTION_NORMAL_STATISTICS_DO_NOT_MEA,
    THE_INDEX_IS_EMPTY_CALL_REFRESH_INDEX_TO_BUILD_A_FULL_INDEX_PROVIDE_REPO_IN_MULTI_REP,
    NO_REPOSITORIES_HAVE_BEEN_INDEXED_YET_CALL_REFRESH_INDEX_WITH_REPO_SET_TO_AN_ABSOLUTE,
    FOR_OTHER_TOOLS_SELECT_THE_REPOSITORY_WITH_REPO_AN_ABSOLUTE_PATH_OR_A_REPO_KEY_SHOWN,
    THIS_INDEX_DATABASE_HAS_NO_REPOSITORY_PATH_METADATA_CANNOT_LOCATE_THE_REPOSITORY_ROOT,
    SET_REPO_TO_AN_ABSOLUTE_REPOSITORY_PATH_AND_REFRESH_AGAIN,
    REFRESH_WAS_NOT_SCHEDULED_A_TASK_FOR_THIS_REPOSITORY_IS_STILL_ACTIVE_OR_THE_SCHEDULER,
    USE_INDEX_STATUS_TO_WAIT_FOR_THE_CURRENT_TASK_TO_FINISH_THEN_CALL_REFRESH_INDEX_AGAIN,
    INDEXING_IS_STILL_RUNNING_IN_THE_BACKGROUND_USE_INDEX_STATUS_TO_CHECK_COMPLETION_OR_F,
    STATISTICS_ARE_STILL_EMPTY_AFTER_INDEXING_COMPLETED,
    PARAMS_MUST_BE_AN_OBJECT,
    PROJECTS,
    SYMBOL_SEARCH,
    SYMBOL_DETAILS,
    CALL_TRACING,
    ARCHITECTURE_OVERVIEW,
    CHANGE_IMPACT_ANALYSIS,
    INDEX_STATUS,
    REFRESH_INDEX,
    STRUCTURE_AND_KEYWORD_SEARCH,
    DEFINITION_SOURCE,
    INDEX_COVERAGE_CHECK,
    MCP_FRAME_OR_HEADER_EXCEEDS_THE_BYTE_LIMIT,
    MCP_HEADER_IS_NOT_UTF_8,
    INVALID_CONTENT_LENGTH,
    MCP_BODY_EXCEEDS_THE_BYTE_LIMIT,
    MCP_HEADER_EXCEEDS_THE_BYTE_LIMIT,
    INVALID_MCP_HEADER_FORMAT,
    DUPLICATE_CONTENT_LENGTH,
    PARSE_ERROR_MESSAGE_MUST_BE_UTF_8,
    INTERNAL_ERROR_TOOL_CALL_FAILED,
    FAILED_TO_OPEN_THE_REPOSITORY,
    FAILED_TO_READ_WORKING_TREE_STATUS,
    CANNOT_RESOLVE_BASELINE_BRANCH,
    CANNOT_RESOLVE_HEAD,
    FAILED_TO_COMPUTE_MERGE_BASE,
    FAILED_TO_READ_THE_BASELINE_TREE,
    FAILED_TO_READ_THE_HEAD_TREE,
    FAILED_TO_GENERATE_THE_BASELINE_DIFF,
    INVALID_NAME_REGULAR_EXPRESSION,
    FAILED_TO_START_THE_SEARCH_SNAPSHOT,
    FAILED_TO_FINISH_THE_SEARCH_SNAPSHOT,
    FULL_TEXT_SEARCH_FAILED,
    FAILED_TO_READ_SEARCH_RESULTS,
    FAILED_TO_PREPARE_THE_FULL_TEXT_INDEX,
    FAILED_TO_WRITE_THE_FULL_TEXT_INDEX,
    FAILED_TO_READ_MIGRATION_NODES,
    FAILED_TO_PREPARE_FULL_TEXT_INDEX_MIGRATION,
    FAILED_TO_MIGRATE_THE_FULL_TEXT_INDEX,
    FAILED_TO_UPDATE_THE_FULL_TEXT_INDEX,
    FAILED_TO_READ_OLD_SEARCH_NODES,
    FAILED_TO_READ_OLD_SEARCH_CONTENT,
    FAILED_TO_DELETE_THE_OLD_FULL_TEXT_INDEX,
    THE_CODE_INDEX_DOES_NOT_EXIST_OR_HAS_NOT_BEEN_BUILT_CALL_REFRESH_INDEX_FIRST,
    INVALID_REVIEW_COMMIT,
    FAILED_TO_CREATE_THE_REVIEW_INDEX_DIRECTORY,
    THE_REVIEW_INDEX_EXCEEDS_THE_LIMIT_OF_FILES,
    THE_REVIEW_INDEX_FOR_THIS_COMMIT_IS_BEING_BUILT_RETRY_LATER,
    FAILED_TO_START_THE_INDEX_QUERY,
    FAILED_TO_FINISH_THE_INDEX_QUERY,
    FAILED_TO_PREPARE_THE_NODE_QUERY,
    FAILED_TO_QUERY_NODES,
    FAILED_TO_READ_NODES,
    FAILED_TO_READ_SOURCE_FILE_METADATA,
    FAILED_TO_PREPARE_THE_STATISTICS_QUERY,
    FAILED_TO_READ_STATISTICS,
    FAILED_TO_READ_DIRECTORY_STATISTICS,
    FAILED_TO_READ_CALL_HOTSPOTS,
    LEGACY_CALL_RELATIONSHIPS_MUST_BE_REBUILT_AND_CANNOT_SERVE_AS_CURRENT_EVIDENCE_CALL_R,
    ROOT_DIRECTORY,
    FAILED_TO_CREATE_THE_INDEX_DIRECTORY,
    FAILED_TO_RESET_THE_INDEX_DATABASE,
    FAILED_TO_OPEN_THE_INDEX_DATABASE,
    FAILED_TO_MIGRATE_THE_CALL_RECORD_TABLE,
    FAILED_TO_INITIALIZE_THE_INDEX_SCHEMA,
    FAILED_TO_WRITE_INDEX_METADATA,
    FAILED_TO_START_INDEX_MIGRATION,
    FAILED_TO_UPDATE_THE_INDEX_VERSION,
    FAILED_TO_COMMIT_INDEX_MIGRATION,
    FAILED_TO_READ_THE_INDEXED_BRANCH,
    FAILED_TO_READ_INDEX_RETRY_INFORMATION,
    FAILED_TO_READ_DISCOVERY_COVERAGE,
    CORRUPT_DISCOVERY_COVERAGE_INFORMATION,
    FAILED_TO_START_THE_INDEX_TRANSACTION,
    FAILED_TO_CLEAR_AUXILIARY_DATABASE_INDEXES,
    FAILED_TO_CLEAR_THE_OLD_GRAPH,
    FAILED_TO_CLEAR_THE_FULL_TEXT_INDEX,
    FAILED_TO_BATCH_WRITE_NODES,
    FAILED_TO_BATCH_WRITE_RELATIONSHIPS,
    FAILED_TO_REBUILD_AUXILIARY_DATABASE_INDEXES,
    FAILED_TO_CLEAR_CALL_RECORDS,
    FAILED_TO_PREPARE_FILE_HASH_WRITES,
    FAILED_TO_WRITE_FILE_HASHES,
    FAILED_TO_COMMIT_THE_INDEX_TRANSACTION,
    FAILED_TO_LOCATE_AFFECTED_CALLS,
    FAILED_TO_READ_CALL_FILES,
    FAILED_TO_READ_CALL_RECORDS,
    FAILED_TO_READ_CALL_DATA,
    FAILED_TO_START_THE_INCREMENTAL_TRANSACTION,
    FAILED_TO_DELETE_OLD_FILE_NODES,
    FAILED_TO_DELETE_OLD_FILE_METADATA,
    FAILED_TO_DELETE_OLD_CALL_RECORDS,
    FAILED_TO_READ_NODE_IDS,
    FAILED_TO_UPDATE_NODE_COVERAGE,
    FAILED_TO_WRITE_INCREMENTAL_NODES,
    FAILED_TO_CLEAR_AFFECTED_CALLS,
    FAILED_TO_PREPARE_INCREMENTAL_RELATIONSHIPS,
    FAILED_TO_WRITE_INCREMENTAL_RELATIONSHIPS,
    FAILED_TO_READ_OLD_STRUCTURAL_NODES,
    FAILED_TO_DELETE_STALE_STRUCTURAL_NODES,
    FAILED_TO_PREPARE_INCREMENTAL_FILE_METADATA,
    FAILED_TO_WRITE_INCREMENTAL_FILE_METADATA,
    FAILED_TO_COMMIT_THE_INCREMENTAL_TRANSACTION,
    FAILED_TO_READ_NODES_2,
    FAILED_TO_QUERY_NODES_2,
    FAILED_TO_READ_EDGES,
    FAILED_TO_QUERY_EDGES,
    FAILED_TO_READ_EDGES_2,
    FAILED_TO_READ_FILE_HASHES,
    FAILED_TO_QUERY_FILE_HASHES,
    FAILED_TO_READ_FILE_HASHES_2,
    FAILED_TO_START_THE_INDEX_STATISTICS_SNAPSHOT,
    FAILED_TO_FINISH_THE_INDEX_STATISTICS_SNAPSHOT,
    FAILED_TO_READ_THE_INDEX_GENERATION,
    FAILED_TO_COMPUTE_INDEX_STATISTICS,
    FAILED_TO_CLEAR_OLD_METADATA,
    FAILED_TO_GENERATE_AN_INDEX_GENERATION,
    FAILED_TO_PREPARE_CALL_RECORDS,
    FAILED_TO_WRITE_CALL_RECORDS,
    FAILED_TO_CLEAR_CALL_NAMES,
    FAILED_TO_WRITE_CALL_NAMES,
    FAILED_TO_CREATE_THE_INDEX_DATABASE,

    COVERAGE_LEGACY_MISSING,
    COVERAGE_FILE_NOT_RECORDED,
    COVERAGE_PRUNED_DIRECTORY,
    INTELLIJ_COMMAND_NOT_FOUND,
    OAUTH_LOGIN_SUCCESS,
    OAUTH_CALLBACK_SUCCESS_BODY,
    INDEX_PROGRESS_PARSE,
    INDEX_PROGRESS_RESOLVE,
    INDEX_PROGRESS_WRITE,
    COVERAGE_PARSE_SIZE_LIMIT,
    COVERAGE_BINARY_FILE,
    COVERAGE_NO_SYNTAX_TREE,
    COVERAGE_UNSUPPORTED_LANGUAGE,
    COVERAGE_FILTERED_FILE,
    COVERAGE_FILTERED_DIRECTORY,

    SSH_AGENT_IDENTITIES_LOADED,
    CANNOT_LOCATE_THE_CURRENT_USER_S_HOME_DIRECTORY,
    THE_SSH_PRIVATE_KEY_FILE_DOES_NOT_EXIST_OR_IS_NOT_A_REGULAR_FILE,
    THE_SELECTED_FILE_IS_NOT_A_RECOGNIZED_OPENSSH_PEM_PRIVATE_KEY_DO_NOT_SELECT_A_PUB_PUB,
    DETECTING_LOCAL_SSH_IDENTITIES,
    DETECT_LOCAL_SSH_IDENTITIES,
    LOCAL_SSH_IDENTITIES,
    FIND_PRIVATE_KEYS_IN_SSH_IDENTITYFILE_ENTRIES_IN_SSH_CONFIG_AND_IDENTITIES_LOADED_IN,
    AGENT_SELECTED,
    USE_AGENT,
    SELECTED,
    USE_IDENTITY,
    NO_USABLE_IDENTITIES_FOUND_USE_SELECT_PRIVATE_KEY_FILE_BELOW_TO_SPECIFY_ONE_MANUALLY,
    THE_FORM_BELOW_HAS_BEEN_FILLED_IN_CONFIRM_THE_REMOTE_AND_USERNAME_THEN_CLICK_SAVE_TO,
    INVALID_PROXY_CONFIGURATION_OAUTH_REQUEST_CANCELLED,
    FAILED_TO_REQUEST_A_DEVICE_CODE,
    FAILED_TO_PARSE_THE_DEVICE_CODE_RESPONSE,
    GITHUB_RETURNED_AN_ERROR,
    FAILED_TO_FETCH_USER_INFORMATION,
    FAILED_TO_PARSE_USER_INFORMATION,
    CANNOT_LISTEN_ON_LOCAL_CALLBACK_PORT_IT_MAY_BE_IN_USE,
    FAILED_TO_MAKE_THE_CALLBACK_LISTENER_NON_BLOCKING,
    GITEE_DENIED_AUTHORIZATION,
    CALLBACK_LISTENER_ERROR,
    REFRESH_REQUEST_FAILED_BROKER,
    FAILED_TO_PARSE_THE_REFRESH_RESPONSE,
    TOKEN_EXCHANGE_FAILED_BROKER,
    FAILED_TO_PARSE_THE_TOKEN_RESPONSE,
    FAILED_TO_FETCH_GITEE_USER_INFORMATION,
    FAILED_TO_PARSE_GITEE_USER_INFORMATION,
    THE_DEVICE_CODE_HAS_EXPIRED_SIGN_IN_AGAIN,
    AUTHORIZATION_DENIED,
    CANCELLED,
    GITHUB_DID_NOT_RETURN_A_LOGIN_NAME,
    TIMED_OUT_WAITING_FOR_BROWSER_AUTHORIZATION,
    AUTHORIZATION_STATE_VALIDATION_FAILED_ABORTED_TO_PREVENT_CSRF,
    THE_CALLBACK_DID_NOT_CONTAIN_AN_AUTHORIZATION_CODE,
    THE_BROKER_DID_NOT_RETURN_A_NEW_TOKEN,
    THE_BROKER_DID_NOT_RETURN_A_TOKEN,
    GITEE_DID_NOT_RETURN_A_LOGIN_NAME,
    INTELLIJ_IDEA_COMMAND_DOES_NOT_EXIST,
    THIS_FILE_HAS_NO_CONFLICTS_3,
    CANNOT_START_THE_EXTERNAL_MERGE_TOOL,
    INTELLIJ_IDEA_MERGE_TOOL_EXITED_UNSUCCESSFULLY,
    THE_EXTERNAL_MERGE_TOOL_IS_DISABLED,
    INTELLIJ_IDEA_MERGE_DID_NOT_PRODUCE_A_RESULT_FILE,
    THIS_CONFLICT_IS_MISSING_BASE_OURS_THEIRS_INTELLIJ_IDEA_THREE_WAY_MERGE_IS_UNAVAILABL,
    FAILED_TO_CREATE_THE_TRAY_MENU,
    FAILED_TO_LOAD_THE_TRAY_ICON,
    FAILED_TO_CREATE_THE_SYSTEM_TRAY_ICON,
    FAILED_TO_READ_THE_MAIN_WINDOW_HANDLE,
    SHOW_MAIN_WINDOW,
    QUIT_KHASLANA,
    THE_CURRENT_WINDOW_IS_NOT_A_WIN32_WINDOW_CANNOT_MINIMIZE_TO_THE_SYSTEM_TRAY,
    SCANNING_FILES,
    PARSING_SYMBOLS,
    RESOLVING_CALLS,
    WRITING_INDEX_DATABASE,
    FAILED_TO_OPEN_THE_INDEX_WRITE_LOCK,
    FAILED_TO_ACQUIRE_THE_INDEX_WRITE_LOCK,
    THIS_REPOSITORY_IS_BEING_INDEXED_BY_ANOTHER_PROCESS_RETRY_LATER,
    INDEXING_TASK_CANCELLED,
    INDEXING_TASK_TERMINATED_UNEXPECTEDLY,
    DISCOVERED_FILES,
    THIS_REPOSITORY_IS_BEING_INDEXED_WAIT_FOR_THE_CURRENT_TASK_TO_FINISH,
    THE_INDEX_PARSER_THREAD_FAILED_THE_PREVIOUS_INDEX_WAS_RETAINED,

    COMMIT_OBJECT_AVAILABLE,
    NO_COMMIT_ON_THIS_SIDE,
    LOCAL_COMMIT_OBJECT_UNAVAILABLE,
    THIS_OBJECT_IS_NOT_A_COMMIT,
    FAILED_TO_READ_COMMIT_OBJECT,

    GIT_ERROR,
    IO_ERROR,
    CREDENTIAL_ERROR,
    INVALID_BRANCH_NAME,
    EMPTY_COMMIT_MESSAGE,
    EMPTY_COMMIT,
    NO_REPOSITORY,
    OPERATION_CONFLICTS,
    WORKFLOW_CANCELLED,
    FULL_FILE_TOO_LARGE,
    NON_FAST_FORWARD_PUSH,
    CLONING,
    CLONED,
    REMOTE_NAME_ALREADY_EXISTS,
    FETCHING,
    FETCHED,
    REFRESHING,
    REFRESHED,
    PULLING,
    PULLED,
    PULLING_2,
    PULLED_2,
    PULLING_LOCAL_BRANCH,
    LOCAL_BRANCH_IS_UP_TO_DATE,
    BRANCH_HAS_DIVERGED_FROM_SWITCH_TO_THAT_BRANCH_BEFORE_PULLING,
    PULLED_LOCAL_BRANCH,
    LOCAL_BRANCH_DOES_NOT_EXIST,
    PUSHING_TO,
    PUSHED_TO,
    REMOTE_BRANCH_DOES_NOT_EXIST,
    DELETING_REMOTE_BRANCH,
    DELETED_REMOTE_BRANCH,
    BRANCH_HAS_NO_TARGET_COMMIT,
    REMOTE_BRANCH_HAS_NO_TARGET_COMMIT,
    CANNOT_SWITCH_TO_LOCAL_BRANCH,
    TAG_ALREADY_EXISTS,
    TAG_DOES_NOT_EXIST,
    LOCAL_TAG_DOES_NOT_EXIST,
    PUSHING_TAG_TO,
    DELETING_REMOTE_TAG,
    BRANCH_NAME_ALREADY_EXISTS,
    CHANGES_FROM_COMMIT_ARE_ALREADY_IN_THE_CURRENT_BRANCH_CHERRY_PICK_RESULT_IS_EMPTY,
    INVALID_COMMIT_ID,
    RECEIVED_OBJECTS,
    REMOTE_REJECTED_PUSH_TO,
    REMOTE_BRANCH_DOES_NOT_EXIST_2,
    THIS_FILE_HAS_CONFLICTS_RESOLVE_THEM_BEFORE_ATTEMPTING_TO,
    INVALID_REMOTE_NAME,
    INVALID_TAG_NAME,
    AUTHENTICATION_IS_REQUIRED_TO_ACCESS,
    INVALID_FILE_PATH,
    THE_UPSTREAM_REMOTE_NAME_IS_NOT_VALID_UTF_8,
    THE_UPSTREAM_BRANCH_NAME_IS_NOT_VALID_UTF_8,
    CANNOT_IDENTIFY_THE_REMOTE_BRANCH_ASSOCIATED_WITH_THE_LOCAL_BRANCH,
    THE_LOCAL_BRANCH_HAS_NO_COMMIT_TO_UPDATE,
    THE_REMOTE_BRANCH_HAS_NO_COMMIT_TO_PULL,
    CREATING_TAG,
    TAG_CREATED,
    DELETING_TAG,
    TAG_DELETED,
    TAG_PUSHED,
    REMOTE_TAG_DELETED,
    DISCARDING_UNSTAGED_CHANGES,
    UNSTAGED_CHANGES_DISCARDED,
    DISCARDING_ALL_FILE_CHANGES,
    THIS_REPOSITORY_HAS_NO_HEAD_YET_CANNOT_DISCARD_CHANGES_TO_THIS_FILE,
    ALL_FILE_CHANGES_DISCARDED,
    A_REBASE_IS_IN_PROGRESS_CONTINUE_SKIP_OR_ABORT_FROM_THE_REBASE_STATUS_BAR_INSTEAD_OF,
    A_MERGE_IS_IN_PROGRESS_COMPLETE_OR_ABORT_IT_BEFORE_AMENDING_A_COMMIT,
    A_REBASE_IS_IN_PROGRESS_CONTINUE_SKIP_OR_ABORT_FROM_THE_REBASE_STATUS_BAR_BEFORE_AMEN,
    HEAD_IS_DETACHED_SWITCH_TO_A_BRANCH_BEFORE_AMENDING_A_COMMIT,
    HEAD_DOES_NOT_POINT_TO_A_LOCAL_BRANCH_CANNOT_AMEND_A_COMMIT,
    AMENDING_COMMIT,
    COMMIT_AMENDED,
    HEAD_IS_DETACHED_CANNOT_RESET_A_BRANCH,
    RESETTING_BRANCH,
    BRANCH_RESET,
    HEAD_IS_DETACHED_CANNOT_RESTORE_A_COMMIT_TO_THE_INDEX,
    HEAD_DOES_NOT_POINT_TO_A_LOCAL_BRANCH_CANNOT_RESTORE_A_COMMIT_TO_THE_INDEX,
    THE_CURRENT_BRANCH_HAS_NO_COMMIT_TO_RESTORE,
    ONLY_THE_CURRENT_LATEST_COMMIT_CAN_BE_RESTORED_TO_THE_INDEX,
    RESTORING_THE_INITIAL_COMMIT_TO_THE_INDEX_IS_NOT_SUPPORTED,
    RESTORING_A_MERGE_COMMIT_TO_THE_INDEX_IS_NOT_SUPPORTED,
    RESTORING_COMMIT_TO_THE_INDEX,
    COMMIT_RESTORED_TO_THE_INDEX,
    COMMIT_STASH_OR_DISCARD_WORKING_TREE_CHANGES_BEFORE_REVERTING_A_COMMIT,
    REVERTING_MERGE_COMMITS_IS_NOT_SUPPORTED,
    REVERTING_COMMIT,
    COMMIT_REVERTED,
    COMMIT_STASH_OR_DISCARD_WORKING_TREE_CHANGES_BEFORE_REVERTING_A_MERGE,
    THIS_IS_NOT_A_MERGE_COMMIT_USE_REGULAR_COMMIT_REVERT,
    REVERTING_MERGE_COMMIT,
    MERGE_REVERTED,
    COMMIT_STASH_OR_DISCARD_WORKING_TREE_CHANGES_BEFORE_CHERRY_PICKING,
    CHERRY_PICKING_MERGE_COMMITS_IS_NOT_SUPPORTED,
    CHERRY_PICKING_COMMIT,
    CHERRY_PICK_COMPLETED,
    UNKNOWN_AUTHOR,
    UNKNOWN_COMMITTER,
    NO_COMMIT_MESSAGE,
    DISCARD_CHANGES,
    REMOTE_URL_CANNOT_BE_EMPTY,
    THE_COMMIT_AUTHOR_NAME_IS_MISSING,
    THE_COMMIT_AUTHOR_EMAIL_IS_MISSING,
    CANNOT_DISCARD_CHANGES,
    A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_DISCARD_FILE_CHANGES,
    THIS_FILE_HAS_NOT_BEEN_COMMITTED_NO_TRACE_INFORMATION_IS_AVAILABLE,
    THIS_PATH_IS_NOT_A_FILE,
    BINARY_FILES_DO_NOT_SUPPORT_TRACE_INFORMATION,
    CANNOT_RESOLVE_REFERENCE,
    IS_NOT_A_COMMIT,
    TAG_DOES_NOT_EXIST_2,
    TAG_DOES_NOT_POINT_TO_A_COMMIT,
    PATH_DOES_NOT_EXIST,
    FILE_DOES_NOT_EXIST,
    THIS_PATH_IS_NOT_A_DIRECTORY,
    BRANCH,
    REMOTE_BRANCH,
    TAG,
    UNCOMMITTED_CHANGES_WERE_STASHED_BUT_SWITCHING_TO_STILL_FAILED_CHANGES_REMAIN_IN_THE,
    SWITCHED_TO_UNCOMMITTED_CHANGES_ARE_IN_THE_FIRST_STASH_ENTRY_APPLY_OR_POP_IT_FROM_THE,
    SWITCHED_TO_BUT_RESTORING_CHANGES_FAILED_ORIGINAL_CHANGES_REMAIN_IN_THE_FIRST_STASH_E,
    SWITCHED_TO_BUT_RESTORING_CHANGES_CAUSED_WORKING_TREE_CONFLICTS_THE_STASH_WAS_RETAINE,
    SWITCHED_TO_BUT_RESTORING_UNCOMMITTED_CHANGES_CAUSED_CONFLICTS_ORIGINAL_CHANGES_REMAI,
    THE_CONFLICT_FILE_PATH_IS_NOT_VALID_UTF_8_AUTOMATIC_RESOLUTION_IS_UNAVAILABLE,
    BRANCH_DOES_NOT_EXIST,
    THIS_FILE_HAS_NO_CONFLICTS,
    RESOLVING_CONFLICT_USING_THE,
    CONFLICT_RESOLVED_USING_THE,
    THIS_FILE_HAS_NO_CONFLICTS_2,
    CANNOT_READ_CONFLICT_DETAILS,
    A_BARE_REPOSITORY_HAS_NO_WORKING_TREE,
    RESTORING_THE_STASH_WROTE_CONFLICT_MARKERS_TO_THE_WORKING_TREE_BUT_THE_GIT_INDEX_HAS,
    THREE_WAY_TEXT_IS_MISSING_FOR_THIS_CONFLICT_USE_THE_QUICK_RESOLUTION_BUTTONS,
    THE_TEXT_MERGE_EDITOR_CANNOT_OPEN_THIS_CONFLICT_FILE,
    THIS_CONFLICT_FILE_IS_NOT_UTF_8_TEXT_THE_VISUAL_EDITOR_IS_UNAVAILABLE,
    THE_CONFLICT_STAGE_IS_NO_LONGER_VALID,
    THE_CONFLICT_VERSION_IS_NOT_UTF_8_TEXT,
    CANNOT_GENERATE_AI_MERGE_SUGGESTIONS,
    THREE_WAY_TEXT_IS_MISSING_FOR_THIS_CONFLICT_AI_MERGE_SUGGESTIONS_ARE_UNAVAILABLE,
    BINARY_CONFLICT_FILES_DO_NOT_SUPPORT_AI_MERGE_SUGGESTIONS_USE_THE_QUICK_RESOLUTION_BU,
    THIS_CONFLICT_FILE_IS_NOT_UTF_8_TEXT_AI_MERGE_SUGGESTIONS_ARE_UNAVAILABLE,
    CANNOT_APPLY_THE_CONFLICT_DRAFT,
    APPLYING_CONFLICT_DRAFT,
    CONFLICT_DRAFT_APPLIED_TO_THE_WORKING_TREE,
    CANNOT_APPLY_AND_RESOLVE_THE_CONFLICT,
    APPLYING_RESULT_AND_MARKING_CONFLICT_AS_RESOLVED,
    CONFLICT_RESULT_APPLIED_AND_MARKED_AS_RESOLVED,
    CANNOT_RESOLVE_CONFLICTS_USING_INTELLIJ_IDEA,
    WAITING_FOR_INTELLIJ_IDEA_TO_FINISH_MERGING,
    INTELLIJ_IDEA_MERGE_RESULT_APPLIED,
    CANNOT_RESOLVE_THE_CONFLICT,
    CURRENT_VERSION,
    INCOMING_VERSION,
    CANNOT_MARK_THE_CONFLICT_AS_RESOLVED,
    MARKING_CONFLICT_AS_RESOLVED,
    CONFLICT_MARKED_AS_RESOLVED,
    A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_MARK_CONFLICTS_AS_RESOLVED,
    THE_CONFLICT_PATH_IS_A_DIRECTORY_IT_CANNOT_BE_MARKED_AS_RESOLVED,
    A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_WRITE_CONFLICT_RESULTS,
    CONFLICT_TEXT_IS_MISSING_THE_BASE_SEPARATOR,
    CONFLICT_TEXT_IS_MISSING_THE_MIDDLE_SEPARATOR,
    CONFLICT_TEXT_IS_MISSING_THE_THEIRS_END_MARKER,
    MERGING,
    MERGE_RESULT_FOR_WRITTEN_TO_THE_INDEX,
    MERGED,
    CANNOT_MERGE_UNSUPPORTED_MERGE_ANALYSIS_RESULT,
    NO_MERGE_IS_CURRENTLY_IN_PROGRESS,
    COMPLETING_MERGE,
    MERGE_COMMIT_CREATED,
    ABORTING_MERGE,
    MERGE_ABORTED,
    ANOTHER_GIT_OPERATION_IS_IN_PROGRESS_COMPLETE_OR_ABORT_IT_BEFORE_MERGING,
    THERE_ARE_UNCOMMITTED_CHANGES_COMMIT_OR_STASH_THEM_BEFORE_MERGING,
    MERGE_STATE_IS_MISSING_MERGE_HEAD_CANNOT_COMPLETE_THE_MERGE,
    SLIDE,
    THIS_FILE_WAS_ADDED_DELETED_OR_RENAMED_AS_A_WHOLE_CANNOT_PART_OF_IT_USE_THE_WHOLE_FIL,
    CANNOT_PART_OF_A_BINARY_FILE_USE_THE_WHOLE_FILE_BUTTON,
    NO_CHANGES_IN_THIS_FILE_CAN_BE_PARTIALLY_PROCESSED_WITH_IT_MAY_BE_UNTRACKED_OR_HAVE_C,
    FAILED_TO_BUILD_THE_PARTIAL_PATCH,
    CANNOT_APPLY_SELECTED_CHANGES_TO_THE_INDEX_CONTENT_MAY_HAVE_CHANGED_REFRESH_AND_RETRY,
    THIS_HUNK_CONTAINS_A_NO_NEWLINE_AT_END_OF_FILE_MARKER_STAGE_THE_WHOLE_HUNK_INSTEAD_OF,
    NO_CHANGED_LINES_SELECTED,
    STAGE,
    STAGING_SELECTED_CHANGES,
    SELECTED_CHANGES_STAGED,
    UNSTAGE,
    UNSTAGING_SELECTED_CHANGES,
    THERE_IS_NO_COMMIT_HISTORY_PARTIAL_UNSTAGING_IS_UNAVAILABLE_UNSTAGE_THE_WHOLE_FILE_IN,
    SELECTED_CHANGES_UNSTAGED,
    THE_SELECTED_CHANGES_ALREADY_MATCH_THE_CURRENT_CONTENT_NOTHING_TO_APPLY,
    REBASING,
    REBASE_PAUSED_WHILE_REPLAYING_COMMIT_RESOLVE_CONFLICTS_BEFORE_CONTINUING,
    REBASED,
    PULLING_WITH_REBASE,
    PULLED_WITH_REBASE,
    CONTINUING_REBASE,
    UNRESOLVED_CONFLICTS_REMAIN_RESOLVE_ALL_CONFLICTS_BEFORE_CONTINUING_THE_REBASE,
    REBASE_COMPLETED,
    SKIPPING_REBASE_COMMIT,
    ABORTING_REBASE,
    REBASE_ABORTED,
    CANNOT_READ_THE_RECOVERY_TARGET_COMMIT,
    THE_BRANCH_WAS_CREATED_BUT_ITS_TARGET_IS_INSTEAD_OF_THE_REQUESTED_CHECK_THE_REFERENCE,
    THE_BRANCH_WAS_CREATED_BUT_READ_BACK_VERIFICATION_FAILED,
    CANNOT_INSPECT_HEAD_OF_A_KNOWN_WORKING_TREE,
    CANNOT_INSPECT_WORKING_TREE_HEAD,
    LOCAL_RECOVERY_DOES_NOT_SUPPORT_BARE_REPOSITORIES,
    LOCAL_RECOVERY_DOES_NOT_SUPPORT_REPOSITORIES_WITHOUT_A_WORKING_TREE,
    COMMIT_DETAILS_ARE_EMPTY,
    THE_RECOVERY_WRITE_LOCK_IS_INVALID_REOPEN_THE_REPOSITORY_AND_RETRY,
    THE_REPOSITORY_IDENTITY_HAS_CHANGED_SINCE_THE_RECOVERY_REQUEST_SELECT_THE_REPOSITORY,
    A_ZERO_OID_IS_NOT_A_RECOVERABLE_COMMIT_OBJECT,
    THE_RECOVERY_TARGET_IS_NOT_A_COMMIT_OBJECT,
    THE_BRANCH_WAS_CREATED_BUT_THE_REFERENCE_READ_BACK_IS_NOT_DIRECT_CHECK_THE_REFERENCE,
    THE_BRANCH_WAS_CREATED_BUT_READ_BACK_VERIFICATION_WAS_INTERRUPTED_CHECK_THE_REFERENCE,
    THE_RECOVERY_TARGET_MUST_BE_A_FULL_40_DIGIT_HEXADECIMAL_OID,
    HEAD_OF_THIS_OR_ANOTHER_WORKING_TREE_POINTS_TO_THIS_BRANCH_RECOVERY_WAS_REFUSED_BECAU,
    THE_HEAD_SYMBOLIC_REFERENCE_CHAIN_CONTAINS_A_CYCLE_CANNOT_SAFELY_CREATE_THE_RECOVERY,
    THE_HEAD_SYMBOLIC_REFERENCE_CHAIN_IS_TOO_LONG_CANNOT_SAFELY_CREATE_THE_RECOVERY_BRANC,
    ADDITIONAL_REFLOG_DIAGNOSTICS_WERE_OMITTED,
    INCOMPLETE_REFLOG_COVERAGE_READ_SOURCES_MISSING_OR_UNREADABLE_SOURCES_ARE_NOT_INCLUDE,
    UNSUPPORTED_REFERENCE_BACKEND_LOCAL_REFLOG_CURRENTLY_SUPPORTS_ONLY_THE_FILES_BACKEND,
    CANNOT_DETERMINE_THE_REFERENCE_BACKEND_LOCAL_REFLOG_WAS_NOT_READ,
    CANNOT_READ_THE_HEAD_SYMBOLIC_REFERENCE_NAME,
    CANNOT_READ_THE_LOCAL_BRANCH_REFERENCE_NAME,
    NOT_A_LOCAL_BRANCH_REFERENCE,
    CANNOT_READ_LOCAL_BRANCH,
    REFLOG_PATH_IS_OUTSIDE_THE_REPOSITORY_DIRECTORY_READING_REFUSED,
    REFLOG_IS_MISSING_THIS_SOURCE_IS_NOT_COVERED_THE_READ_ONLY_SERVICE_DID_NOT_CREATE_ANY,
    REFLOG_IS_UNREADABLE_THIS_SOURCE_IS_NOT_COVERED,
    REFLOG_IS_EMPTY,
    LINE_EXCEEDS_THE_256_KIB_LINE_LIMIT_SKIPPED,
    LINE_CONTAINS_NON_UTF_8_TEXT_DISPLAYED_WITH_REPLACEMENT_CHARACTERS,
    LINE_HAS_AN_INVALID_FORMAT_SKIPPED,
    LINE_IS_INCOMPLETE_MISSING_LINE_ENDING_SKIPPED,
    THE_REFLOG_KEPT_CHANGING_DURING_READING_CANNOT_CREATE_A_CONSISTENT_SNAPSHOT_RETRY,
    REFLOG_READING_CANCELLED,
    HEAD_HAS_NO_SYMBOLIC_REFERENCE_TARGET,
    HEAD_DOES_NOT_POINT_TO_A_LOCAL_BRANCH_THERE_IS_NO_CURRENT_BRANCH_REFLOG_SOURCE,
    THE_REFLOG_PATH_IS_NOT_A_REGULAR_FILE,
    RAW_REFLOG_DATA_EXCEEDS_64_MIB_NO_PARTIAL_SNAPSHOT_WAS_GENERATED,
    THE_REFLOG_EXCEEDS_200000_VALID_ENTRIES_NO_PARTIAL_SNAPSHOT_WAS_GENERATED,
    MISSING_FULL_OIDS_FOR_BOTH_SIDES,
    MISSING_TIME_ZONE,
    MISSING_TIMESTAMP,
    MISSING_ACTOR,
    TIMESTAMP_IS_INVALID_OR_OUT_OF_RANGE,
    INVALID_ACTOR_EMAIL_FORMAT,
    INVALID_SEPARATOR_BETWEEN_ACTOR_NAME_AND_EMAIL,
    OID_MUST_BE_A_FULL_40_DIGIT_HEXADECIMAL_VALUE,
    OID_CONTAINS_NON_ASCII_BYTES,
    INVALID_OID,
    TIME_ZONE_MUST_BE_HHMM,
    TIME_ZONE_IS_OUT_OF_RANGE,
    DIRECTORY_DOES_NOT_EXIST_CHECK_THE_SPELLING_OF_PATH_PREFIX,
    DIRECTORY_DOES_NOT_EXIST,
    IS_A_FILE_NOT_A_DIRECTORY_REMOVE_THE_FILE_NAME_AND_KEEP_ONLY_THE_DIRECTORY_PREFIX,
    INVALID_REGULAR_EXPRESSION,
    SEARCH_TEXT_CANNOT_BE_EMPTY,
    DELETING_STASH,
    DELETED_STASH,
    APPLYING_STASH,
    APPLIED_STASH,
    POPPING_STASH,
    POPPED_STASH,
    STASH_DOES_NOT_EXIST_STASH,
    CANNOT_CREATE_A_STASH_WHILE_CONFLICTS_EXIST,
    ONLY_UNTRACKED_FILES_EXIST_SELECT_INCLUDE_UNTRACKED_FILES_TO_STASH_THEM,
    THERE_ARE_NO_CHANGES_TO_STASH,
    STASHING_CURRENT_CHANGES,
    CURRENT_CHANGES_STASHED,
    UPDATING_SUBMODULE_TO_THE_LATEST_REMOTE_REVISION,
    SUBMODULE_UPDATED_TO_THE_LATEST_REMOTE_REVISION,
    SYNCHRONIZING_SUBMODULE,
    SYNCHRONIZING_THE_RECORDED_REVISION_OF_SUBMODULE,
    CANNOT_RESOLVE_CURRENT_HEAD_OF_SUBMODULE,
    REMOTE_BRANCH_NOT_FOUND_IN_SUBMODULE,
    THE_REMOTE_BRANCH_OF_SUBMODULE_IS_NOT_A_DIRECT_REFERENCE,
    CANNOT_RESOLVE_CURRENT_HEAD_OF_SUBMODULE_2,
    SUBMODULE_CANNOT_FAST_FORWARD_TO_THE_LATEST_REMOTE_REVISION_RESOLVE_DIVERGENCE_OR_LOC,
    SUBMODULE_HAS_LOCAL_CHANGES_COMMIT_STASH_OR_DISCARD_THEM_IN_THE_SUBMODULE_BEFORE_UPDA,
    SUBMODULE_HAS_BRANCH_BUT_THE_PARENT_REPOSITORY_IS_NOT_ON_A_LOCAL_BRANCH,
    SUBMODULE_HAS_NO_TRACKING_BRANCH_AND_ITS_REMOTE_DEFAULT_BRANCH_CANNOT_BE_DETERMINED,
    CANNOT_DETERMINE_THE_REMOTE_FOR_SUBMODULE_KEEP_ORIGIN_OR_CONFIGURE_ONLY_ONE_REMOTE,
    SYNCHRONIZING_RECORDED_SUBMODULE_REVISIONS,
    RECORDED_SUBMODULE_REVISIONS_SYNCHRONIZED,
    UPDATING_SUBMODULES_TO_THE_LATEST_REMOTE_REVISIONS,
    SUBMODULES_UPDATED_TO_THE_LATEST_REMOTE_REVISIONS,
    SUBMODULE_IS_NOT_INITIALIZED,
    SUBMODULE_IS_NOT_CHECKED_OUT,
    STASH_CREATED_BUT_CLEANING_THE_WORKING_TREE_FAILED_RELEASE_LOCKED_FILES_AND_RETRY_OR,
    CANNOT_LOCATE_THE_CONFLICT_BLOCK_IN_THE_ORIGINAL_VERSION,
    THE_ORIGINAL_VERSION_DOES_NOT_MATCH_THE_CONFLICT_BLOCK_REFRESH_THE_REPOSITORY,
    FAILED_TO_CREATE_THE_FIXED_DATA_DIRECTORY,
    FAILED_TO_WRITE_THE_RELOCATION_MARKER,
    FAILED_TO_START_THE_NEW_INSTANCE,
    FAILED_TO_PARSE_AI_SETTINGS,
    FAILED_TO_SERIALIZE_AI_SETTINGS,
    FAILED_TO_PARSE_EXTERNAL_MERGE_TOOL_SETTINGS,
    FAILED_TO_SERIALIZE_EXTERNAL_MERGE_TOOL_SETTINGS,
    FAILED_TO_PARSE_SHORTCUT_SETTINGS,
    FAILED_TO_SERIALIZE_SHORTCUT_SETTINGS,
    FAILED_TO_PARSE_WORKFLOW_SHORTCUT_SETTINGS,
    FAILED_TO_SERIALIZE_WORKFLOW_SHORTCUT_SETTINGS,
    FAILED_TO_PARSE_LAYOUT_PREFERENCES,
    FAILED_TO_SERIALIZE_LAYOUT_PREFERENCES,
    LOCAL_SETTINGS_DATABASE_ERROR,
    UNKNOWN_PROXY_MODE,
    UNKNOWN_THEME_MODE,
    UNKNOWN_DIFF_ENCODING_PREFERENCE,
    UNKNOWN_REMOTE_CREDENTIAL_POLICY,
    UNKNOWN_CREDENTIAL_SCOPE,
    UNKNOWN_CREDENTIAL_TYPE,
    CANNOT_LOCATE_THE_LOCAL_SETTINGS_DATABASE_DIRECTORY,
    THE_LOCAL_SETTINGS_DATABASE_IS_IN_AN_INVALID_STATE,
    CANNOT_LOCATE_THE_LEGACY_DATA_DIRECTORY,
    CANNOT_LOCATE_THE_PORTABLE_DATA_DIRECTORY,
    THE_LEGACY_DATABASE_FILE_DOES_NOT_EXIST,
    CANNOT_LOCATE_THE_FIXED_DATA_DIRECTORY,
    THE_RELOCATION_MARKER_IS_EMPTY,
    THE_REMOTE_CREDENTIAL_BINDING_IS_MISSING_A_RECORD_ID,
    THE_SYSTEM_CREDENTIAL_MANAGER_IS_UNAVAILABLE,
    FAILED_TO_CREATE_A_SYSTEM_CREDENTIAL_ENTRY,
    FAILED_TO_READ_SYSTEM_CREDENTIALS,
    FAILED_TO_WRITE_SYSTEM_CREDENTIALS,
    FAILED_TO_DELETE_SYSTEM_CREDENTIALS,
    FAILED_TO_UPDATE_SYSTEM_CREDENTIALS,
    BUILT_IN_GIT_TEST_FAILED_THE_SYSTEM_GIT_OPENSSH_CHECK_SUCCEEDED_BUT_IN_APP_PUSH_AND_P,
    LIBGIT2_TEST_FAILED_SYSTEM_GIT_OPENSSH_CHECK_FAILED,
    CANNOT_START_SYSTEM_GIT,
    SYSTEM_GIT_EXITED_WITH_STATUS,
    FAILED_TO_CREATE_A_GITEE_REFRESH_ENTRY,
    FAILED_TO_SERIALIZE_GITEE_REFRESH_DATA,
    FAILED_TO_SAVE_GITEE_REFRESH_DATA,
    THE_CREDENTIAL_CACHE_IS_IN_AN_INVALID_STATE,
    CANNOT_PARSE_THE_REMOTE_URL_CREDENTIALS_CANNOT_BE_SAVED,
    THE_CREDENTIAL_RECORD_DOES_NOT_EXIST,
    CANNOT_PARSE_THE_REMOTE_URL_CREDENTIALS_CANNOT_BE_BOUND,
    THE_SYSTEM_CREDENTIAL_MANAGER_INITIALIZATION_STATE_IS_INVALID,
    THE_CREDENTIAL_INDEX_LOCK_IS_IN_AN_INVALID_STATE,
    THE_ENCRYPTED_CREDENTIAL_WAS_NOT_FOUND_IN_THE_SYSTEM_CREDENTIAL_MANAGER,
    SSH_HTTPS_CREDENTIALS_WERE_ALREADY_TRIED_THE_REMOTE_REJECTED_AUTHENTICATION,
    THE_SELECTED_RECORD_IS_NOT_AN_SSH_CREDENTIAL,
    THE_REMOTE_DOES_NOT_ACCEPT_SSH_KEY_CREDENTIALS,
    THIS_REMOTE_ONLY,
    SAME_HOST,
    HTTPS_USERNAME_PAT,
    SSH_PRIVATE_KEY,
    USING_CUSTOM_PROXY,
    NO_PROXY,
    USING_A_PROXY_FROM_GIT_CONFIGURATION_OR_ENVIRONMENT_VARIABLES,
    NO_MATCHING_CUSTOM_PROXY_CONNECTING_DIRECTLY,
    PROXY_URLS_CANNOT_CONTAIN_CONTROL_CHARACTERS_OR_WHITESPACE,
    UNSUPPORTED_PROXY_URL_SCHEME_USE_HTTP_HTTPS_SOCKS5_OR_SOCKS5H,
    UNSUPPORTED_UPDATE_MANIFEST_FORMAT_SCHEMA_ONLY_SCHEMA_1_IS_SUPPORTED,
    INVALID_VERSION_IN_THE_UPDATE_MANIFEST,
    INVALID_SKIPPED_VERSION,
    FAILED_TO_EXTRACT_THE_UPDATE_PACKAGE,
    FAILED_TO_READ_AN_UPDATE_PACKAGE_ENTRY,
    FAILED_TO_PARSE_THE_UPDATE_MANIFEST,
    FAILED_TO_DOWNLOAD_THE_UPDATE_MANIFEST,
    INVALID_PROXY_CONFIGURATION_UPDATE_REQUEST_CANCELLED,
    FAILED_TO_DOWNLOAD_THE_UPDATE_PACKAGE,
    INVALID_UPDATE_PACKAGE_SIZE_BYTES_LIMIT_DOWNLOAD_CANCELLED,
    UPDATE_PACKAGE_DOWNLOAD_EXCEEDED_THE_SIZE_LIMIT_BYTES_CANCELLED,
    STABLE,
    BETA,
    AUTOMATIC_UPDATES_ARE_NOT_SUPPORTED_ON_THIS_PLATFORM_MISSING_WINDOWS_X86_64_DOWNLOAD,
    FAILED_TO_DOWNLOAD_THE_UPDATE_PACKAGE_NO_AVAILABLE_SOURCES,
    THE_UPDATE_PACKAGE_IS_MISSING_KHASLANA_EXE,
    THE_UPDATE_PACKAGE_IS_MISSING_KHASLANA_UPDATER_EXE,
    CANNOT_FETCH_THE_UPDATE_MANIFEST,
    UNKNOWN_REASON,
    NOT_INITIALIZED,
    NOT_CHECKED_OUT,
    MODIFIED,
    HAS_UNTRACKED_FILES,
    UPDATE_NEEDED,
    UP_TO_DATE,
    AUTO,
    CURRENT_BRANCH,
    ALL_BRANCHES,
];

#[cfg(test)]
#[path = "tests/domain_ui_resources.rs"]
mod tests;

pub const SSH_AGENT_IDENTITIES_LOADED: UiMessage = UiMessage::new("domain.ssh_agent_identities_loaded", "SSH Agent · 已加载 {count} 个身份", "SSH Agent · {count} identities loaded");

pub const CANNOT_LOCATE_THE_CURRENT_USER_S_HOME_DIRECTORY: UiMessage = UiMessage::new("domain.cannot_locate_the_current_user_s_home_directory", "无法定位当前用户主目录", "Cannot locate the current user's home directory");

pub const THE_SSH_PRIVATE_KEY_FILE_DOES_NOT_EXIST_OR_IS_NOT_A_REGULAR_FILE: UiMessage = UiMessage::new("domain.the_ssh_private_key_file_does_not_exist_or_is_not_a_regular_file", "SSH 私钥文件不存在或不是普通文件", "The SSH private key file does not exist or is not a regular file");

pub const THE_SELECTED_FILE_IS_NOT_A_RECOGNIZED_OPENSSH_PEM_PRIVATE_KEY_DO_NOT_SELECT_A_PUB_PUB: UiMessage = UiMessage::new("domain.the_selected_file_is_not_a_recognized_openssh_pem_private_key_do_not_select_a_pub_pub", "所选文件不是可识别的 OpenSSH/PEM 私钥，请勿选择 .pub 公钥文件", "The selected file is not a recognized OpenSSH/PEM private key; do not select a .pub public key file");

pub const DETECTING_LOCAL_SSH_IDENTITIES: UiMessage = UiMessage::new("domain.detecting_local_ssh_identities", "正在检测本机 SSH...", "Detecting local SSH identities...");

pub const DETECT_LOCAL_SSH_IDENTITIES: UiMessage = UiMessage::new("domain.detect_local_ssh_identities", "检测本机 SSH", "Detect local SSH identities");

pub const LOCAL_SSH_IDENTITIES: UiMessage = UiMessage::new("domain.local_ssh_identities", "本机 SSH 身份", "Local SSH identities");

pub const FIND_PRIVATE_KEYS_IN_SSH_IDENTITYFILE_ENTRIES_IN_SSH_CONFIG_AND_IDENTITIES_LOADED_IN: UiMessage = UiMessage::new("domain.find_private_keys_in_ssh_identityfile_entries_in_ssh_config_and_identities_loaded_in", "检测 ~/.ssh 私钥、SSH config 的 IdentityFile 和 Agent 已加载身份", "Find private keys in ~/.ssh, IdentityFile entries in SSH config, and identities loaded in the agent");

pub const AGENT_SELECTED: UiMessage = UiMessage::new("domain.agent_selected", "已选择 Agent", "Agent selected");

pub const USE_AGENT: UiMessage = UiMessage::new("domain.use_agent", "一键使用 Agent", "Use agent");

pub const SELECTED: UiMessage = UiMessage::new("domain.selected", "已选择", "Selected");

pub const USE_IDENTITY: UiMessage = UiMessage::new("domain.use_identity", "一键使用", "Use identity");

pub const NO_USABLE_IDENTITIES_FOUND_USE_SELECT_PRIVATE_KEY_FILE_BELOW_TO_SPECIFY_ONE_MANUALLY: UiMessage = UiMessage::new("domain.no_usable_identities_found_use_select_private_key_file_below_to_specify_one_manually", "未发现可用身份，可使用下方“选择私钥文件”手动指定。", "No usable identities found. Use Select private key file below to specify one manually.");

pub const THE_FORM_BELOW_HAS_BEEN_FILLED_IN_CONFIRM_THE_REMOTE_AND_USERNAME_THEN_CLICK_SAVE_TO: UiMessage = UiMessage::new("domain.the_form_below_has_been_filled_in_confirm_the_remote_and_username_then_click_save_to", "已填入下方表单；请确认适用远端和用户名，点击“保存”后生效。", "The form below has been filled in. Confirm the remote and username, then click Save to apply.");

pub const INVALID_PROXY_CONFIGURATION_OAUTH_REQUEST_CANCELLED: UiMessage = UiMessage::new("domain.invalid_proxy_configuration_oauth_request_cancelled", "代理配置无效，已取消 OAuth 请求：{err}", "Invalid proxy configuration; OAuth request cancelled: {err}");

pub const FAILED_TO_REQUEST_A_DEVICE_CODE: UiMessage = UiMessage::new("domain.failed_to_request_a_device_code", "请求设备码失败：{e}", "Failed to request a device code: {e}");

pub const FAILED_TO_PARSE_THE_DEVICE_CODE_RESPONSE: UiMessage = UiMessage::new("domain.failed_to_parse_the_device_code_response", "解析设备码响应失败：{e}", "Failed to parse the device code response: {e}");

pub const GITHUB_RETURNED_AN_ERROR: UiMessage = UiMessage::new("domain.github_returned_an_error", "GitHub 返回错误：{other}", "GitHub returned an error: {other}");

pub const FAILED_TO_FETCH_USER_INFORMATION: UiMessage = UiMessage::new("domain.failed_to_fetch_user_information", "获取用户信息失败：{e}", "Failed to fetch user information: {e}");

pub const FAILED_TO_PARSE_USER_INFORMATION: UiMessage = UiMessage::new("domain.failed_to_parse_user_information", "解析用户信息失败：{e}", "Failed to parse user information: {e}");

pub const CANNOT_LISTEN_ON_LOCAL_CALLBACK_PORT_IT_MAY_BE_IN_USE: UiMessage = UiMessage::new("domain.cannot_listen_on_local_callback_port_it_may_be_in_use", "无法监听本地回调端口 {GITEE_CALLBACK_PORT}：{e}（可能被占用）", "Cannot listen on local callback port {GITEE_CALLBACK_PORT}: {e} (it may be in use)");

pub const FAILED_TO_MAKE_THE_CALLBACK_LISTENER_NON_BLOCKING: UiMessage = UiMessage::new("domain.failed_to_make_the_callback_listener_non_blocking", "设置回调监听非阻塞失败：{e}", "Failed to make the callback listener non-blocking: {e}");

pub const GITEE_DENIED_AUTHORIZATION: UiMessage = UiMessage::new("domain.gitee_denied_authorization", "Gitee 拒绝授权：{err}", "Gitee denied authorization: {err}");

pub const CALLBACK_LISTENER_ERROR: UiMessage = UiMessage::new("domain.callback_listener_error", "回调监听错误：{e}", "Callback listener error: {e}");

pub const REFRESH_REQUEST_FAILED_BROKER: UiMessage = UiMessage::new("domain.refresh_request_failed_broker", "刷新请求失败（broker）：{e}", "Refresh request failed (broker): {e}");

pub const FAILED_TO_PARSE_THE_REFRESH_RESPONSE: UiMessage = UiMessage::new("domain.failed_to_parse_the_refresh_response", "解析刷新响应失败：{e}", "Failed to parse the refresh response: {e}");

pub const TOKEN_EXCHANGE_FAILED_BROKER: UiMessage = UiMessage::new("domain.token_exchange_failed_broker", "令牌交换失败（broker）：{e}", "Token exchange failed (broker): {e}");

pub const FAILED_TO_PARSE_THE_TOKEN_RESPONSE: UiMessage = UiMessage::new("domain.failed_to_parse_the_token_response", "解析令牌响应失败：{e}", "Failed to parse the token response: {e}");

pub const FAILED_TO_FETCH_GITEE_USER_INFORMATION: UiMessage = UiMessage::new("domain.failed_to_fetch_gitee_user_information", "获取 Gitee 用户信息失败：{e}", "Failed to fetch Gitee user information: {e}");

pub const FAILED_TO_PARSE_GITEE_USER_INFORMATION: UiMessage = UiMessage::new("domain.failed_to_parse_gitee_user_information", "解析 Gitee 用户信息失败：{e}", "Failed to parse Gitee user information: {e}");

pub const THE_DEVICE_CODE_HAS_EXPIRED_SIGN_IN_AGAIN: UiMessage = UiMessage::new("domain.the_device_code_has_expired_sign_in_again", "设备码已过期，请重新登录", "The device code has expired; sign in again");

pub const AUTHORIZATION_DENIED: UiMessage = UiMessage::new("domain.authorization_denied", "已拒绝授权", "Authorization denied");

pub const CANCELLED: UiMessage = UiMessage::new("domain.cancelled", "已取消", "Cancelled");

pub const GITHUB_DID_NOT_RETURN_A_LOGIN_NAME: UiMessage = UiMessage::new("domain.github_did_not_return_a_login_name", "GitHub 未返回登录名", "GitHub did not return a login name");

pub const TIMED_OUT_WAITING_FOR_BROWSER_AUTHORIZATION: UiMessage = UiMessage::new("domain.timed_out_waiting_for_browser_authorization", "等待浏览器授权超时", "Timed out waiting for browser authorization");

pub const AUTHORIZATION_STATE_VALIDATION_FAILED_ABORTED_TO_PREVENT_CSRF: UiMessage = UiMessage::new("domain.authorization_state_validation_failed_aborted_to_prevent_csrf", "授权 state 校验失败，已放弃以防 CSRF", "Authorization state validation failed; aborted to prevent CSRF");

pub const THE_CALLBACK_DID_NOT_CONTAIN_AN_AUTHORIZATION_CODE: UiMessage = UiMessage::new("domain.the_callback_did_not_contain_an_authorization_code", "回调未携带授权码", "The callback did not contain an authorization code");

pub const THE_BROKER_DID_NOT_RETURN_A_NEW_TOKEN: UiMessage = UiMessage::new("domain.the_broker_did_not_return_a_new_token", "broker 未返回新令牌", "The broker did not return a new token");

pub const THE_BROKER_DID_NOT_RETURN_A_TOKEN: UiMessage = UiMessage::new("domain.the_broker_did_not_return_a_token", "broker 未返回令牌", "The broker did not return a token");

pub const GITEE_DID_NOT_RETURN_A_LOGIN_NAME: UiMessage = UiMessage::new("domain.gitee_did_not_return_a_login_name", "Gitee 未返回登录名", "Gitee did not return a login name");

pub const INTELLIJ_IDEA_COMMAND_DOES_NOT_EXIST: UiMessage = UiMessage::new("domain.intellij_idea_command_does_not_exist", "IntelliJ IDEA 命令不存在：{value0}", "IntelliJ IDEA command does not exist: {value0}");

pub const THIS_FILE_HAS_NO_CONFLICTS_3: UiMessage = UiMessage::new("domain.this_file_has_no_conflicts_3", "该文件不存在冲突：{value0}", "This file has no conflicts: {value0}");

pub const CANNOT_START_THE_EXTERNAL_MERGE_TOOL: UiMessage = UiMessage::new("domain.cannot_start_the_external_merge_tool", "无法启动外部合并工具：{err}", "Cannot start the external merge tool: {err}");

pub const INTELLIJ_IDEA_MERGE_TOOL_EXITED_UNSUCCESSFULLY: UiMessage = UiMessage::new("domain.intellij_idea_merge_tool_exited_unsuccessfully", "IntelliJ IDEA 合并工具退出失败：{status}", "IntelliJ IDEA merge tool exited unsuccessfully: {status}");

pub const THE_EXTERNAL_MERGE_TOOL_IS_DISABLED: UiMessage = UiMessage::new("domain.the_external_merge_tool_is_disabled", "外部合并工具未启用", "The external merge tool is disabled");

pub const INTELLIJ_IDEA_MERGE_DID_NOT_PRODUCE_A_RESULT_FILE: UiMessage = UiMessage::new("domain.intellij_idea_merge_did_not_produce_a_result_file", "IntelliJ IDEA 合并未生成结果文件", "IntelliJ IDEA merge did not produce a result file");

pub const THIS_CONFLICT_IS_MISSING_BASE_OURS_THEIRS_INTELLIJ_IDEA_THREE_WAY_MERGE_IS_UNAVAILABL: UiMessage = UiMessage::new("domain.this_conflict_is_missing_base_ours_theirs_intellij_idea_three_way_merge_is_unavailabl", "该冲突缺少 BASE/OURS/THEIRS，暂不能用 IntelliJ IDEA 三方合并", "This conflict is missing BASE/OURS/THEIRS; IntelliJ IDEA three-way merge is unavailable");

pub const FAILED_TO_CREATE_THE_TRAY_MENU: UiMessage = UiMessage::new("domain.failed_to_create_the_tray_menu", "创建托盘菜单失败：{error}", "Failed to create the tray menu: {error}");

pub const FAILED_TO_LOAD_THE_TRAY_ICON: UiMessage = UiMessage::new("domain.failed_to_load_the_tray_icon", "读取托盘图标失败：{error}", "Failed to load the tray icon: {error}");

pub const FAILED_TO_CREATE_THE_SYSTEM_TRAY_ICON: UiMessage = UiMessage::new("domain.failed_to_create_the_system_tray_icon", "创建系统托盘失败：{error}", "Failed to create the system tray icon: {error}");

pub const FAILED_TO_READ_THE_MAIN_WINDOW_HANDLE: UiMessage = UiMessage::new("domain.failed_to_read_the_main_window_handle", "读取主窗口句柄失败：{error}", "Failed to read the main window handle: {error}");

pub const SHOW_MAIN_WINDOW: UiMessage = UiMessage::new("domain.show_main_window", "显示主窗口", "Show main window");

pub const QUIT_KHASLANA: UiMessage = UiMessage::new("domain.quit_khaslana", "退出 Khaslana", "Quit Khaslana");

pub const THE_CURRENT_WINDOW_IS_NOT_A_WIN32_WINDOW_CANNOT_MINIMIZE_TO_THE_SYSTEM_TRAY: UiMessage = UiMessage::new("domain.the_current_window_is_not_a_win32_window_cannot_minimize_to_the_system_tray", "当前窗口不是 Win32 窗口，无法缩小到系统托盘", "The current window is not a Win32 window; cannot minimize to the system tray");

pub const SCANNING_FILES: UiMessage = UiMessage::new("domain.scanning_files", "扫描文件", "Scanning files");

pub const PARSING_SYMBOLS: UiMessage = UiMessage::new("domain.parsing_symbols", "解析符号", "Parsing symbols");

pub const RESOLVING_CALLS: UiMessage = UiMessage::new("domain.resolving_calls", "解析调用关系", "Resolving calls");

pub const WRITING_INDEX_DATABASE: UiMessage = UiMessage::new("domain.writing_index_database", "写入索引库", "Writing index database");

pub const FAILED_TO_OPEN_THE_INDEX_WRITE_LOCK: UiMessage = UiMessage::new("domain.failed_to_open_the_index_write_lock", "打开索引写锁失败：{error}", "Failed to open the index write lock: {error}");

pub const FAILED_TO_ACQUIRE_THE_INDEX_WRITE_LOCK: UiMessage = UiMessage::new("domain.failed_to_acquire_the_index_write_lock", "获取索引写锁失败：{error}", "Failed to acquire the index write lock: {error}");

pub const THIS_REPOSITORY_IS_BEING_INDEXED_BY_ANOTHER_PROCESS_RETRY_LATER: UiMessage = UiMessage::new("domain.this_repository_is_being_indexed_by_another_process_retry_later", "该仓库正在被其他进程索引，请稍后重试", "This repository is being indexed by another process; retry later");

pub const INDEXING_TASK_CANCELLED: UiMessage = UiMessage::new("domain.indexing_task_cancelled", "索引任务已取消", "Indexing task cancelled");

pub const INDEXING_TASK_TERMINATED_UNEXPECTEDLY: UiMessage = UiMessage::new("domain.indexing_task_terminated_unexpectedly", "索引任务异常退出", "Indexing task terminated unexpectedly");

pub const DISCOVERED_FILES: UiMessage = UiMessage::new("domain.discovered_files", "发现 {total} 个文件", "Discovered {total} files");

pub const THIS_REPOSITORY_IS_BEING_INDEXED_WAIT_FOR_THE_CURRENT_TASK_TO_FINISH: UiMessage = UiMessage::new("domain.this_repository_is_being_indexed_wait_for_the_current_task_to_finish", "该仓库正在索引，请等待当前任务完成", "This repository is being indexed; wait for the current task to finish");

pub const THE_INDEX_PARSER_THREAD_FAILED_THE_PREVIOUS_INDEX_WAS_RETAINED: UiMessage = UiMessage::new("domain.the_index_parser_thread_failed_the_previous_index_was_retained", "索引解析线程异常，已保留旧索引", "The index parser thread failed; the previous index was retained");

pub const INTELLIJ_COMMAND_NOT_FOUND: UiMessage = UiMessage::new("domain.intellij_command_not_found", "未找到 IntelliJ IDEA 命令，请确认 idea64 或 idea 已加入 PATH", "IntelliJ IDEA command not found; make sure idea64 or idea is on PATH");

pub const OAUTH_LOGIN_SUCCESS: UiMessage = UiMessage::new("domain.oauth_login_success", "登录成功", "Sign-in successful");

pub const OAUTH_CALLBACK_SUCCESS_BODY: UiMessage = UiMessage::new("domain.oauth_callback_success_body", "登录成功，可以关闭此页面返回 Khaslana。", "Sign-in successful. You can close this page and return to Khaslana.");

pub const INDEX_PROGRESS_PARSE: UiMessage = UiMessage::new("domain.index_progress_parse", "解析符号 {done}/{total}", "Parsing symbols {done}/{total}");

pub const INDEX_PROGRESS_RESOLVE: UiMessage = UiMessage::new("domain.index_progress_resolve", "解析调用关系 {done}/{total}", "Resolving calls {done}/{total}");

pub const INDEX_PROGRESS_WRITE: UiMessage = UiMessage::new("domain.index_progress_write", "写入索引库 {done}/{total}", "Writing index database {done}/{total}");

pub const COVERAGE_PARSE_SIZE_LIMIT: UiMessage = UiMessage::new("domain.coverage_parse_size_limit", "超过单文件解析上限", "File exceeds the parsing size limit");

pub const COVERAGE_BINARY_FILE: UiMessage = UiMessage::new("domain.coverage_binary_file", "二进制文件", "Binary file");

pub const COVERAGE_NO_SYNTAX_TREE: UiMessage = UiMessage::new("domain.coverage_no_syntax_tree", "解析器未返回语法树", "The parser did not return a syntax tree");

pub const COVERAGE_UNSUPPORTED_LANGUAGE: UiMessage = UiMessage::new("domain.coverage_unsupported_language", "当前语言不支持符号解析", "Symbol parsing is not supported for this language");

pub const COVERAGE_FILTERED_FILE: UiMessage = UiMessage::new("domain.coverage_filtered_file", "文件名或生成产物过滤", "Filtered file name or generated artifact");

pub const COVERAGE_FILTERED_DIRECTORY: UiMessage = UiMessage::new("domain.coverage_filtered_directory", "目录过滤规则", "Directory filter rule");

pub const COVERAGE_PRUNED_DIRECTORY: UiMessage = UiMessage::new("domain.coverage_pruned_directory", "目录剪枝或独立子仓库", "Pruned directory or separate nested repository");

pub const COVERAGE_LEGACY_MISSING: UiMessage = UiMessage::new("domain.coverage_legacy_missing", "旧索引没有覆盖记录，请刷新索引", "This legacy index has no coverage records; refresh the index");
pub const COVERAGE_FILE_NOT_RECORDED: UiMessage = UiMessage::new("domain.coverage_file_not_recorded", "未记录该文件；可能被忽略规则排除或尚未索引，需直接核对源码", "This file is not recorded; it may be ignored or not yet indexed. Check the source directly");

pub const TOO_MANY_STRINGS_IN_CALL_RECORDS: UiMessage = UiMessage::new("domain.too_many_strings_in_call_records", "调用记录字符串过多", "Too many strings in call records");

pub const CALL_RECORDS_ARE_TOO_LARGE: UiMessage = UiMessage::new("domain.call_records_are_too_large", "调用记录过大", "Call records are too large");

pub const INCOMPATIBLE_CALL_RECORD_FORMAT: UiMessage = UiMessage::new("domain.incompatible_call_record_format", "调用记录格式不兼容", "Incompatible call record format");

pub const TRUNCATED_CALL_RECORDS: UiMessage = UiMessage::new("domain.truncated_call_records", "调用记录截断", "Truncated call records");

pub const CORRUPT_CALL_RECORD_DICTIONARY: UiMessage = UiMessage::new("domain.corrupt_call_record_dictionary", "调用记录字典损坏", "Corrupt call record dictionary");

pub const TRUNCATED_CALL_RECORD_STRING: UiMessage = UiMessage::new("domain.truncated_call_record_string", "调用记录字符串截断", "Truncated call record string");

pub const CORRUPT_CALL_RECORD_ENCODING: UiMessage = UiMessage::new("domain.corrupt_call_record_encoding", "调用记录编码损坏", "Corrupt call record encoding");

pub const CORRUPT_CALL_RECORD_DICTIONARY_REFERENCE: UiMessage = UiMessage::new("domain.corrupt_call_record_dictionary_reference", "调用记录字典引用损坏", "Corrupt call record dictionary reference");

pub const CORRUPT_CALL_RECORD_IMPORT_TABLE: UiMessage = UiMessage::new("domain.corrupt_call_record_import_table", "调用记录导入表损坏", "Corrupt call record import table");

pub const CORRUPT_CALL_RECORD_LIST: UiMessage = UiMessage::new("domain.corrupt_call_record_list", "调用记录列表损坏", "Corrupt call record list");

pub const CALL_RECORDS_CONTAIN_TRAILING_DATA: UiMessage = UiMessage::new("domain.call_records_contain_trailing_data", "调用记录存在多余数据", "Call records contain trailing data");

pub const FAILED_TO_START_THE_COVERAGE_QUERY: UiMessage = UiMessage::new("domain.failed_to_start_the_coverage_query", "开启覆盖查询失败：{e}", "Failed to start the coverage query: {e}");

pub const FAILED_TO_READ_COVERAGE_INFORMATION: UiMessage = UiMessage::new("domain.failed_to_read_coverage_information", "读取覆盖信息失败：{e}", "Failed to read coverage information: {e}");

pub const FAILED_TO_FINISH_THE_COVERAGE_QUERY: UiMessage = UiMessage::new("domain.failed_to_finish_the_coverage_query", "结束覆盖查询失败：{e}", "Failed to finish the coverage query: {e}");

pub const UNRESOLVED_CALLS_INCLUDE_CALLS_WITH_NO_SAME_LANGUAGE_REPOSITORY_DEFINITION_POSSIBLY_E: UiMessage = UiMessage::new("domain.unresolved_calls_include_calls_with_no_same_language_repository_definition_possibly_e", "未解析调用分为无同语言仓库定义（可能为外部库）和有候选但无法安全确定目标；旧库可能没有分类，均不代表语法解析失败率。统计不是完整性证明。", "Unresolved calls include calls with no same-language repository definition (possibly external libraries) and calls with candidates but no safely determined target. Legacy indexes may lack this classification. Neither measures syntax parsing failures. Statistics do not prove completeness.");

pub const PROVIDE_AT_LEAST_PATHS_OR_SCOPES: UiMessage = UiMessage::new("domain.provide_at_least_paths_or_scopes", "至少提供 paths 或 scopes", "Provide at least paths or scopes");

pub const COVERAGE_PATHS_MUST_BE_RELATIVE_PATHS_WITHIN_THE_REPOSITORY: UiMessage = UiMessage::new("domain.coverage_paths_must_be_relative_paths_within_the_repository", "覆盖检查路径必须是仓库内的相对路径", "Coverage paths must be relative paths within the repository");

pub const THE_CODE_INDEX_HAS_NOT_BEEN_BUILT: UiMessage = UiMessage::new("domain.the_code_index_has_not_been_built", "代码索引尚未建立", "The code index has not been built");

pub const METADATA_MATCHES_ONLY_CONFIRMS_MATCHING_TIMESTAMPS_AND_SIZES_UNRESOLVED_NO_CANDIDATE: UiMessage = UiMessage::new("domain.metadata_matches_only_confirms_matching_timestamps_and_sizes_unresolved_no_candidate", "metadata_matches 只证明时间和大小一致。unresolved_no_candidate 为无同语言仓库定义，unresolved_with_candidates 为有候选但目标不确定；旧库可能缺分类，未解析调用不等于语法解析失败。缺口与过期需回读源码。", "metadata_matches only confirms matching timestamps and sizes. unresolved_no_candidate means no same-language repository definition; unresolved_with_candidates means candidates exist but the target is uncertain. Legacy indexes may lack this classification. Unresolved calls are not syntax parsing failures. Read source for gaps or stale coverage.");

pub const CANNOT_SCAN_THE_REPOSITORY_DIRECTORY: UiMessage = UiMessage::new("domain.cannot_scan_the_repository_directory", "无法扫描仓库目录：{error}", "Cannot scan the repository directory: {error}");

pub const REPOSITORY_FILE_COUNT_EXCEEDS_THE_INDEXING_LIMIT_OF_INDEXING_STOPPED: UiMessage = UiMessage::new("domain.repository_file_count_exceeds_the_indexing_limit_of_indexing_stopped", "仓库文件数超过索引上限 {value0} 个，已终止索引", "Repository file count exceeds the indexing limit of {value0}; indexing stopped");

pub const FAILED_TO_SCAN_THE_REPOSITORY_THE_PREVIOUS_INDEX_WAS_RETAINED: UiMessage = UiMessage::new("domain.failed_to_scan_the_repository_the_previous_index_was_retained", "扫描仓库失败，已保留原有索引：{value0}", "Failed to scan the repository; the previous index was retained: {value0}");

pub const FAILED_TO_INITIALIZE_THE_PARSER: UiMessage = UiMessage::new("domain.failed_to_initialize_the_parser", "初始化 {value0} 解析器失败：{e}", "Failed to initialize the {value0} parser: {e}");

pub const REPOSITORY_PATH_UNAVAILABLE: UiMessage = UiMessage::new("domain.repository_path_unavailable", "仓库路径不可用 {value0}: {e}", "Repository path unavailable {value0}: {e}");

pub const IS_NOT_A_GIT_REPOSITORY: UiMessage = UiMessage::new("domain.is_not_a_git_repository", "{value0} 不是 Git 仓库：{e}", "{value0} is not a Git repository: {e}");

pub const REPOSITORY_WORKING_TREE_UNAVAILABLE: UiMessage = UiMessage::new("domain.repository_working_tree_unavailable", "仓库工作区不可用：{e}", "Repository working tree unavailable: {e}");

pub const FAILED_TO_READ_THE_REPOSITORY_PATH: UiMessage = UiMessage::new("domain.failed_to_read_the_repository_path", "读取仓库路径失败：{error}", "Failed to read the repository path: {error}");

pub const THE_REPO_ARGUMENT_IS_NEITHER_AN_EXISTING_REPOSITORY_PATH_NOR_A_KNOWN_INDEX_DATABASE: UiMessage = UiMessage::new("domain.the_repo_argument_is_neither_an_existing_repository_path_nor_a_known_index_database", "repo 参数 {spec} 既不是存在的仓库路径，也没有对应的索引库", "The repo argument {spec} is neither an existing repository path nor a known index database");

pub const THE_REPO_ARGUMENT_WAS_OMITTED_BUT_INDEXED_REPOSITORIES_EXIST_LOCALLY: UiMessage = UiMessage::new("domain.the_repo_argument_was_omitted_but_indexed_repositories_exist_locally", "未指定 repo 参数且本机已有 {value0} 个已索引仓库", "The repo argument was omitted, but {value0} indexed repositories exist locally");

pub const INDEXING_COMPLETE_FILES_SYMBOLS_EDGES: UiMessage = UiMessage::new("domain.indexing_complete_files_symbols_edges", "索引完成：{value0} 文件 / {value1} 符号 / {value2} 边", "Indexing complete: {value0} files / {value1} symbols / {value2} edges");

pub const UNKNOWN_TOOL: UiMessage = UiMessage::new("domain.unknown_tool", "未知工具 {name}", "Unknown tool {name}");

pub const UNKNOWN_TOOL_2: UiMessage = UiMessage::new("domain.unknown_tool_2", "未知工具 {other}", "Unknown tool {other}");

pub const MISSING_REQUIRED_ARGUMENT: UiMessage = UiMessage::new("domain.missing_required_argument", "缺少必填参数 {key}", "Missing required argument {key}");

pub const FAILED_TO_SERIALIZE_DETAILS: UiMessage = UiMessage::new("domain.failed_to_serialize_details", "序列化详情失败：{e}", "Failed to serialize details: {e}");

pub const THERE_ARE_DEFINITIONS_WITH_THIS_NAME_QUERY_AN_EXACT_QUALIFIED_NAME: UiMessage = UiMessage::new("domain.there_are_definitions_with_this_name_query_an_exact_qualified_name", "有 {value0} 个同名定义，请用 qualified_name 精确查询", "There are {value0} definitions with this name; query an exact qualified_name");

pub const FAILED_TO_SERIALIZE_RESULTS: UiMessage = UiMessage::new("domain.failed_to_serialize_results", "序列化结果失败：{e}", "Failed to serialize results: {e}");

pub const THERE_ARE_CALLABLE_DEFINITIONS_WITH_THIS_NAME_QUERY_AN_EXACT_QUALIFIED_NAME: UiMessage = UiMessage::new("domain.there_are_callable_definitions_with_this_name_query_an_exact_qualified_name", "有 {value0} 个同名可调用定义，请用 qualified_name 精确查询", "There are {value0} callable definitions with this name; query an exact qualified_name");

pub const SERIALIZATION_FAILED: UiMessage = UiMessage::new("domain.serialization_failed", "序列化失败：{e}", "Serialization failed: {e}");

pub const MUST_BE_AN_ARRAY_OF_STRINGS: UiMessage = UiMessage::new("domain.must_be_an_array_of_strings", "{key} 必须是字符串数组", "{key} must be an array of strings");

pub const UNKNOWN_MODE_EXPECTED_INCREMENTAL_OR_FULL: UiMessage = UiMessage::new("domain.unknown_mode_expected_incremental_or_full", "未知 mode {other}，可选 incremental | full", "Unknown mode {other}; expected incremental or full");

pub const INVALID_TYPE_FOR_EXPECTED: UiMessage = UiMessage::new("domain.invalid_type_for_expected", "{path} 类型无效，应为 {value0}", "Invalid type for {path}; expected {value0}");

pub const IS_NOT_ONE_OF_THE_ALLOWED_VALUES: UiMessage = UiMessage::new("domain.is_not_one_of_the_allowed_values", "{path} 不在允许的取值中", "{path} is not one of the allowed values");

pub const MUST_NOT_BE_LESS_THAN: UiMessage = UiMessage::new("domain.must_not_be_less_than", "{path} 不得小于 {minimum}", "{path} must not be less than {minimum}");

pub const MUST_NOT_EXCEED: UiMessage = UiMessage::new("domain.must_not_exceed", "{path} 不得大于 {maximum}", "{path} must not exceed {maximum}");

pub const MISSING_REQUIRED_ARGUMENT_2: UiMessage = UiMessage::new("domain.missing_required_argument_2", "缺少必填参数 {path}.{key}", "Missing required argument {path}.{key}");

pub const IS_MISSING_REQUIRED_QUERY_CRITERIA_SEE_TOOLS_LIST: UiMessage = UiMessage::new("domain.is_missing_required_query_criteria_see_tools_list", "{path} 缺少必需的查询条件，请参考 tools/list", "{path} is missing required query criteria; see tools/list");

pub const THE_DATA_DIRECTORY_IS_UNAVAILABLE_CANNOT_LOCATE_THE_INDEX_DATABASE: UiMessage = UiMessage::new("domain.the_data_directory_is_unavailable_cannot_locate_the_index_database", "数据目录不可用，无法定位索引库", "The data directory is unavailable; cannot locate the index database");

pub const A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_BUILD_A_WORKING_TREE_CODE_INDEX: UiMessage = UiMessage::new("domain.a_bare_repository_has_no_working_tree_cannot_build_a_working_tree_code_index", "裸仓库没有工作区，不能建立工作区代码索引", "A bare repository has no working tree; cannot build a working tree code index");

pub const THE_INDEX_DATABASE_IS_UNAVAILABLE_OR_ITS_VERSION_IS_INCOMPATIBLE: UiMessage = UiMessage::new("domain.the_index_database_is_unavailable_or_its_version_is_incompatible", "索引库不可用或版本不兼容", "The index database is unavailable or its version is incompatible");

pub const PROVIDE_AN_ABSOLUTE_REPOSITORY_PATH_OR_CALL_LIST_PROJECTS_TO_SEE_AVAILABLE_REPOSITORI: UiMessage = UiMessage::new("domain.provide_an_absolute_repository_path_or_call_list_projects_to_see_available_repositori", "传仓库绝对路径，或先调 list_projects 查看可用仓库", "Provide an absolute repository path, or call list_projects to see available repositories");

pub const NO_REPOSITORIES_HAVE_BEEN_INDEXED_YET: UiMessage = UiMessage::new("domain.no_repositories_have_been_indexed_yet", "还没有任何已索引仓库", "No repositories have been indexed yet");

pub const CALL_REFRESH_INDEX_WITH_REPO_SET_TO_AN_ABSOLUTE_REPOSITORY_PATH_TO_BUILD_THE_INDEX: UiMessage = UiMessage::new("domain.call_refresh_index_with_repo_set_to_an_absolute_repository_path_to_build_the_index", "调用 refresh_index 并传 repo 参数（仓库绝对路径）建立索引", "Call refresh_index with repo set to an absolute repository path to build the index");

pub const PROVIDE_REPO_AS_AN_ABSOLUTE_REPOSITORY_PATH_OR_ONE_OF_THE_REPO_KEYS_ABOVE_CALL_LIST_P: UiMessage = UiMessage::new("domain.provide_repo_as_an_absolute_repository_path_or_one_of_the_repo_keys_above_call_list_p", "传 repo 参数（仓库绝对路径，或上方任一条目的 repo 键）；也可先调 list_projects 查看", "Provide repo as an absolute repository path or one of the repo keys above; call list_projects to see available repositories");

pub const INDEX_UNCHANGED: UiMessage = UiMessage::new("domain.index_unchanged", "索引无变化", "Index unchanged");

pub const INDEXING_CANCELLED_THE_PREVIOUS_INDEX_WAS_RETAINED: UiMessage = UiMessage::new("domain.indexing_cancelled_the_previous_index_was_retained", "索引已取消，保留原有索引", "Indexing cancelled; the previous index was retained");

pub const SEE_TOOLS_LIST_FOR_AVAILABLE_TOOLS: UiMessage = UiMessage::new("domain.see_tools_list_for_available_tools", "可用工具见 tools/list", "See tools/list for available tools");

pub const THE_INDEX_WAS_REFRESHED_THE_PREVIOUS_PAGINATION_GENERATION_IS_NO_LONGER_VALID: UiMessage = UiMessage::new("domain.the_index_was_refreshed_the_previous_pagination_generation_is_no_longer_valid", "索引已刷新，旧分页代际失效", "The index was refreshed; the previous pagination generation is no longer valid");

pub const REMOVE_GENERATION_AND_RESTART_THE_QUERY_AT_OFFSET_0: UiMessage = UiMessage::new("domain.remove_generation_and_restart_the_query_at_offset_0", "移除 generation 并从 offset=0 重新查询", "Remove generation and restart the query at offset=0");

pub const THE_INDEX_CHANGED_DURING_THE_QUERY: UiMessage = UiMessage::new("domain.the_index_changed_during_the_query", "查询期间索引已更新", "The index changed during the query");

pub const RESTART_THE_QUERY_AT_OFFSET_0: UiMessage = UiMessage::new("domain.restart_the_query_at_offset_0", "从 offset=0 重新查询", "Restart the query at offset=0");

pub const THE_INDEX_IS_BEING_BUILT_OR_REFRESHED_IN_THE_BACKGROUND_RETRY_LATER_AND_USE_INDEX_STA: UiMessage = UiMessage::new("domain.the_index_is_being_built_or_refreshed_in_the_background_retry_later_and_use_index_sta", "索引正在后台建立/刷新，稍候重试；可用 index_status 观察", "The index is being built or refreshed in the background; retry later and use index_status to monitor it");

pub const SEE_THIS_TOOL_S_INPUTSCHEMA_IN_TOOLS_LIST: UiMessage = UiMessage::new("domain.see_this_tool_s_inputschema_in_tools_list", "参考 tools/list 中该工具的 inputSchema", "See this tool's inputSchema in tools/list");

pub const QUERY_OR_NAME_PATTERN_QN_PATTERN_FILE_PATTERN_LABEL: UiMessage = UiMessage::new("domain.query_or_name_pattern_qn_pattern_file_pattern_label", "query 或 name_pattern / qn_pattern / file_pattern / label", "query or name_pattern / qn_pattern / file_pattern / label");

pub const OFFSET_EXCEEDS_THE_TOTAL_RESULT_COUNT_RESTART_THE_QUERY_AT_OFFSET_0: UiMessage = UiMessage::new("domain.offset_exceeds_the_total_result_count_restart_the_query_at_offset_0", "offset 已超过结果总数，从 offset=0 重新查询", "offset exceeds the total result count; restart the query at offset=0");

pub const NO_MATCHES_SHORTEN_QUERY_OR_USE_SEARCH_GRAPH_WITH_A_NAME_PATTERN_REGEX_OR_FILE_PATTER: UiMessage = UiMessage::new("domain.no_matches_shorten_query_or_use_search_graph_with_a_name_pattern_regex_or_file_patter", "无命中。缩短 query 或用 search_graph 的 name_pattern 正则、file_pattern 路径 glob 定位；怀疑过期时调用 refresh_index。索引是尽力解析，查不到不能证明代码不存在。", "No matches. Shorten query or use search_graph with a name_pattern regex or file_pattern path glob. Call refresh_index if the index may be stale. Indexing is best effort; no matches do not prove that code is absent.");

pub const CANNOT_READ_THE_SOURCE_FILE_MAKE_SURE_REPO_IS_AN_EXISTING_ABSOLUTE_REPOSITORY_PATH_RE: UiMessage = UiMessage::new("domain.cannot_read_the_source_file_make_sure_repo_is_an_existing_absolute_repository_path_re", "当前无法读取源码文件。确认 repo 为存在的仓库绝对路径；文件已移动或索引过期时先刷新。", "Cannot read the source file. Make sure repo is an existing absolute repository path; refresh first if files moved or the index is stale.");

pub const USE_SEARCH_SYMBOLS_FIRST_TO_FIND_THE_CORRECT_NAME: UiMessage = UiMessage::new("domain.use_search_symbols_first_to_find_the_correct_name", "先用 search_symbols 模糊检索确认名称", "Use search_symbols first to find the correct name");

pub const TOTAL_COUNTS_DISTINCT_NODES_WITHIN_THE_REQUESTED_DEPTH_NOT_CALLS_FOR_HOP_1_CALL_SITES: UiMessage = UiMessage::new("domain.total_counts_distinct_nodes_within_the_requested_depth_not_calls_for_hop_1_call_sites", "total 统计指定深度内去重的节点，不是调用次数；hop=1 的 call_sites 给出直接边的调用次数和最多 100 个行号。启发式调用图仍可能缺少动态调用或类型信息。", "total counts distinct nodes within the requested depth, not calls. For hop=1, call_sites provides direct-edge call counts and up to 100 line numbers. The heuristic call graph may still lack dynamic calls or type information.");

pub const THIS_INDEX_DATABASE_HAS_NO_REPOSITORY_PATH_METADATA_CANNOT_READ_WORKING_TREE_STATUS: UiMessage = UiMessage::new("domain.this_index_database_has_no_repository_path_metadata_cannot_read_working_tree_status", "该索引库缺少仓库路径元数据，无法读取工作区状态", "This index database has no repository path metadata; cannot read working tree status");

pub const SET_REPO_TO_AN_ABSOLUTE_REPOSITORY_PATH_OR_CALL_REFRESH_INDEX_WITH_THAT_PATH_TO_RESTO: UiMessage = UiMessage::new("domain.set_repo_to_an_absolute_repository_path_or_call_refresh_index_with_that_path_to_resto", "改传 repo 参数为仓库绝对路径；或先 refresh_index（同样传绝对路径）补写元数据", "Set repo to an absolute repository path, or call refresh_index with that path to restore metadata");

pub const NO_CHANGED_FILES_DETECTED_BETWEEN_THE_WORKING_TREE_AND_THE_BASELINE: UiMessage = UiMessage::new("domain.no_changed_files_detected_between_the_working_tree_and_the_baseline", "工作区与基线之间没有检测到变更文件", "No changed files detected between the working tree and the baseline");

pub const THE_INDEX_NEEDS_REBUILDING_FOR_AN_UPGRADE_WAIT_FOR_THE_BACKGROUND_REFRESH_OR_CALL_REF: UiMessage = UiMessage::new("domain.the_index_needs_rebuilding_for_an_upgrade_wait_for_the_background_refresh_or_call_ref", "索引需升级重建；等待后台刷新结束，或调用 refresh_index，再确认 needs_rebuild=false", "The index needs rebuilding for an upgrade; wait for the background refresh or call refresh_index, then confirm needs_rebuild=false");

pub const THE_INDEX_USES_TREE_SITTER_AND_HEURISTIC_CALL_RESOLUTION_NORMAL_STATISTICS_DO_NOT_MEA: UiMessage = UiMessage::new("domain.the_index_uses_tree_sitter_and_heuristic_call_resolution_normal_statistics_do_not_mea", "索引基于 tree-sitter 和启发式调用解析；统计正常不等于所有定义和关系均已覆盖。", "The index uses tree-sitter and heuristic call resolution; normal statistics do not mean all definitions and relationships are covered.");

pub const THE_INDEX_IS_EMPTY_CALL_REFRESH_INDEX_TO_BUILD_A_FULL_INDEX_PROVIDE_REPO_IN_MULTI_REP: UiMessage = UiMessage::new("domain.the_index_is_empty_call_refresh_index_to_build_a_full_index_provide_repo_in_multi_rep", "索引为空。调用 refresh_index 建立全量索引（多仓库模式需传 repo 参数）", "The index is empty. Call refresh_index to build a full index (provide repo in multi-repository mode)");

pub const NO_REPOSITORIES_HAVE_BEEN_INDEXED_YET_CALL_REFRESH_INDEX_WITH_REPO_SET_TO_AN_ABSOLUTE: UiMessage = UiMessage::new("domain.no_repositories_have_been_indexed_yet_call_refresh_index_with_repo_set_to_an_absolute", "还没有任何已索引仓库：调用 refresh_index 并传 repo 参数（仓库绝对路径）建立索引", "No repositories have been indexed yet: call refresh_index with repo set to an absolute repository path");

pub const FOR_OTHER_TOOLS_SELECT_THE_REPOSITORY_WITH_REPO_AN_ABSOLUTE_PATH_OR_A_REPO_KEY_SHOWN: UiMessage = UiMessage::new("domain.for_other_tools_select_the_repository_with_repo_an_absolute_path_or_a_repo_key_shown", "其余工具传 repo 参数（仓库绝对路径或这里的 repo 键）选择目标仓库；本机仅一个仓库时可不传", "For other tools, select the repository with repo (an absolute path or a repo key shown here); omit it only when there is one local repository");

pub const THIS_INDEX_DATABASE_HAS_NO_REPOSITORY_PATH_METADATA_CANNOT_LOCATE_THE_REPOSITORY_ROOT: UiMessage = UiMessage::new("domain.this_index_database_has_no_repository_path_metadata_cannot_locate_the_repository_root", "该索引库缺少仓库路径元数据，无法定位仓库根目录", "This index database has no repository path metadata; cannot locate the repository root");

pub const SET_REPO_TO_AN_ABSOLUTE_REPOSITORY_PATH_AND_REFRESH_AGAIN: UiMessage = UiMessage::new("domain.set_repo_to_an_absolute_repository_path_and_refresh_again", "改传 repo 参数为仓库绝对路径再刷新", "Set repo to an absolute repository path and refresh again");

pub const REFRESH_WAS_NOT_SCHEDULED_A_TASK_FOR_THIS_REPOSITORY_IS_STILL_ACTIVE_OR_THE_SCHEDULER: UiMessage = UiMessage::new("domain.refresh_was_not_scheduled_a_task_for_this_repository_is_still_active_or_the_scheduler", "刷新未安排：同仓库任务仍在运行或调度器已关闭", "Refresh was not scheduled: a task for this repository is still active or the scheduler is stopped");

pub const USE_INDEX_STATUS_TO_WAIT_FOR_THE_CURRENT_TASK_TO_FINISH_THEN_CALL_REFRESH_INDEX_AGAIN: UiMessage = UiMessage::new("domain.use_index_status_to_wait_for_the_current_task_to_finish_then_call_refresh_index_again", "先用 index_status 等待当前任务结束，再重新调用 refresh_index", "Use index_status to wait for the current task to finish, then call refresh_index again");

pub const INDEXING_IS_STILL_RUNNING_IN_THE_BACKGROUND_USE_INDEX_STATUS_TO_CHECK_COMPLETION_OR_F: UiMessage = UiMessage::new("domain.indexing_is_still_running_in_the_background_use_index_status_to_check_completion_or_f", "索引仍在后台处理，请用 index_status 查询完成或失败状态；无需重复 refresh_index", "Indexing is still running in the background; use index_status to check completion or failure. Do not repeat refresh_index");

pub const STATISTICS_ARE_STILL_EMPTY_AFTER_INDEXING_COMPLETED: UiMessage = UiMessage::new("domain.statistics_are_still_empty_after_indexing_completed", "索引完成后统计仍为空", "Statistics are still empty after indexing completed");

pub const PARAMS_MUST_BE_AN_OBJECT: UiMessage = UiMessage::new("domain.params_must_be_an_object", "params 必须是对象", "params must be an object");

pub const PROJECTS: UiMessage = UiMessage::new("domain.projects", "项目清单", "Projects");

pub const SYMBOL_SEARCH: UiMessage = UiMessage::new("domain.symbol_search", "符号搜索", "Symbol search");

pub const SYMBOL_DETAILS: UiMessage = UiMessage::new("domain.symbol_details", "符号详情", "Symbol details");

pub const CALL_TRACING: UiMessage = UiMessage::new("domain.call_tracing", "调用链追踪", "Call tracing");

pub const ARCHITECTURE_OVERVIEW: UiMessage = UiMessage::new("domain.architecture_overview", "架构概览", "Architecture overview");

pub const CHANGE_IMPACT_ANALYSIS: UiMessage = UiMessage::new("domain.change_impact_analysis", "变更影响分析", "Change impact analysis");

pub const INDEX_STATUS: UiMessage = UiMessage::new("domain.index_status", "索引状态", "Index status");

pub const REFRESH_INDEX: UiMessage = UiMessage::new("domain.refresh_index", "刷新索引", "Refresh index");

pub const STRUCTURE_AND_KEYWORD_SEARCH: UiMessage = UiMessage::new("domain.structure_and_keyword_search", "结构与关键词搜索", "Structure and keyword search");

pub const DEFINITION_SOURCE: UiMessage = UiMessage::new("domain.definition_source", "定义源码", "Definition source");

pub const INDEX_COVERAGE_CHECK: UiMessage = UiMessage::new("domain.index_coverage_check", "索引覆盖检查", "Index coverage check");

pub const MCP_FRAME_OR_HEADER_EXCEEDS_THE_BYTE_LIMIT: UiMessage = UiMessage::new("domain.mcp_frame_or_header_exceeds_the_byte_limit", "MCP 帧或头部超过字节上限", "MCP frame or header exceeds the byte limit");

pub const MCP_HEADER_IS_NOT_UTF_8: UiMessage = UiMessage::new("domain.mcp_header_is_not_utf_8", "MCP 头部不是 UTF-8", "MCP header is not UTF-8");

pub const INVALID_CONTENT_LENGTH: UiMessage = UiMessage::new("domain.invalid_content_length", "Content-Length 无效", "Invalid Content-Length");

pub const MCP_BODY_EXCEEDS_THE_BYTE_LIMIT: UiMessage = UiMessage::new("domain.mcp_body_exceeds_the_byte_limit", "MCP 正文超过字节上限", "MCP body exceeds the byte limit");

pub const MCP_HEADER_EXCEEDS_THE_BYTE_LIMIT: UiMessage = UiMessage::new("domain.mcp_header_exceeds_the_byte_limit", "MCP 头部超过字节上限", "MCP header exceeds the byte limit");

pub const INVALID_MCP_HEADER_FORMAT: UiMessage = UiMessage::new("domain.invalid_mcp_header_format", "MCP 头部格式无效", "Invalid MCP header format");

pub const DUPLICATE_CONTENT_LENGTH: UiMessage = UiMessage::new("domain.duplicate_content_length", "重复的 Content-Length", "Duplicate Content-Length");

pub const PARSE_ERROR_MESSAGE_MUST_BE_UTF_8: UiMessage = UiMessage::new("domain.parse_error_message_must_be_utf_8", "Parse error: 消息必须是 UTF-8", "Parse error: message must be UTF-8");

pub const INTERNAL_ERROR_TOOL_CALL_FAILED: UiMessage = UiMessage::new("domain.internal_error_tool_call_failed", "Internal error: 工具调用异常", "Internal error: tool call failed");

pub const FAILED_TO_OPEN_THE_REPOSITORY: UiMessage = UiMessage::new("domain.failed_to_open_the_repository", "打开仓库失败：{e}", "Failed to open the repository: {e}");

pub const FAILED_TO_READ_WORKING_TREE_STATUS: UiMessage = UiMessage::new("domain.failed_to_read_working_tree_status", "读取工作区状态失败：{e}", "Failed to read working tree status: {e}");

pub const CANNOT_RESOLVE_BASELINE_BRANCH: UiMessage = UiMessage::new("domain.cannot_resolve_baseline_branch", "无法解析基线分支 {base}：{e}", "Cannot resolve baseline branch {base}: {e}");

pub const CANNOT_RESOLVE_HEAD: UiMessage = UiMessage::new("domain.cannot_resolve_head", "无法解析 HEAD：{e}", "Cannot resolve HEAD: {e}");

pub const FAILED_TO_COMPUTE_MERGE_BASE: UiMessage = UiMessage::new("domain.failed_to_compute_merge_base", "计算 merge-base 失败：{e}", "Failed to compute merge-base: {e}");

pub const FAILED_TO_READ_THE_BASELINE_TREE: UiMessage = UiMessage::new("domain.failed_to_read_the_baseline_tree", "读取基线树失败：{e}", "Failed to read the baseline tree: {e}");

pub const FAILED_TO_READ_THE_HEAD_TREE: UiMessage = UiMessage::new("domain.failed_to_read_the_head_tree", "读取 HEAD 树失败：{e}", "Failed to read the HEAD tree: {e}");

pub const FAILED_TO_GENERATE_THE_BASELINE_DIFF: UiMessage = UiMessage::new("domain.failed_to_generate_the_baseline_diff", "生成基线差异失败：{e}", "Failed to generate the baseline diff: {e}");

pub const INVALID_NAME_REGULAR_EXPRESSION: UiMessage = UiMessage::new("domain.invalid_name_regular_expression", "名称正则表达式无效：{e}", "Invalid name regular expression: {e}");

pub const FAILED_TO_START_THE_SEARCH_SNAPSHOT: UiMessage = UiMessage::new("domain.failed_to_start_the_search_snapshot", "开启搜索快照失败：{e}", "Failed to start the search snapshot: {e}");

pub const FAILED_TO_FINISH_THE_SEARCH_SNAPSHOT: UiMessage = UiMessage::new("domain.failed_to_finish_the_search_snapshot", "结束搜索快照失败：{e}", "Failed to finish the search snapshot: {e}");

pub const FULL_TEXT_SEARCH_FAILED: UiMessage = UiMessage::new("domain.full_text_search_failed", "全文检索失败：{e}", "Full-text search failed: {e}");

pub const FAILED_TO_READ_SEARCH_RESULTS: UiMessage = UiMessage::new("domain.failed_to_read_search_results", "读取搜索结果失败：{e}", "Failed to read search results: {e}");

pub const FAILED_TO_PREPARE_THE_FULL_TEXT_INDEX: UiMessage = UiMessage::new("domain.failed_to_prepare_the_full_text_index", "准备全文索引失败：{e}", "Failed to prepare the full-text index: {e}");

pub const FAILED_TO_WRITE_THE_FULL_TEXT_INDEX: UiMessage = UiMessage::new("domain.failed_to_write_the_full_text_index", "写入全文索引失败：{e}", "Failed to write the full-text index: {e}");

pub const FAILED_TO_READ_MIGRATION_NODES: UiMessage = UiMessage::new("domain.failed_to_read_migration_nodes", "读取迁移节点失败：{e}", "Failed to read migration nodes: {e}");

pub const FAILED_TO_PREPARE_FULL_TEXT_INDEX_MIGRATION: UiMessage = UiMessage::new("domain.failed_to_prepare_full_text_index_migration", "准备全文索引迁移失败：{e}", "Failed to prepare full-text index migration: {e}");

pub const FAILED_TO_MIGRATE_THE_FULL_TEXT_INDEX: UiMessage = UiMessage::new("domain.failed_to_migrate_the_full_text_index", "迁移全文索引失败：{e}", "Failed to migrate the full-text index: {e}");

pub const FAILED_TO_UPDATE_THE_FULL_TEXT_INDEX: UiMessage = UiMessage::new("domain.failed_to_update_the_full_text_index", "更新全文索引失败：{error}", "Failed to update the full-text index: {error}");

pub const FAILED_TO_READ_OLD_SEARCH_NODES: UiMessage = UiMessage::new("domain.failed_to_read_old_search_nodes", "读取旧搜索节点失败：{error}", "Failed to read old search nodes: {error}");

pub const FAILED_TO_READ_OLD_SEARCH_CONTENT: UiMessage = UiMessage::new("domain.failed_to_read_old_search_content", "读取旧搜索内容失败：{error}", "Failed to read old search content: {error}");

pub const FAILED_TO_DELETE_THE_OLD_FULL_TEXT_INDEX: UiMessage = UiMessage::new("domain.failed_to_delete_the_old_full_text_index", "删除旧全文索引失败：{error}", "Failed to delete the old full-text index: {error}");

pub const THE_CODE_INDEX_DOES_NOT_EXIST_OR_HAS_NOT_BEEN_BUILT_CALL_REFRESH_INDEX_FIRST: UiMessage = UiMessage::new("domain.the_code_index_does_not_exist_or_has_not_been_built_call_refresh_index_first", "代码索引不存在或尚未建立，请先调用 refresh_index", "The code index does not exist or has not been built; call refresh_index first");

pub const INVALID_REVIEW_COMMIT: UiMessage = UiMessage::new("domain.invalid_review_commit", "评审提交无效：{error}", "Invalid review commit: {error}");

pub const FAILED_TO_CREATE_THE_REVIEW_INDEX_DIRECTORY: UiMessage = UiMessage::new("domain.failed_to_create_the_review_index_directory", "创建评审索引目录失败：{error}", "Failed to create the review index directory: {error}");

pub const THE_REVIEW_INDEX_EXCEEDS_THE_LIMIT_OF_FILES: UiMessage = UiMessage::new("domain.the_review_index_exceeds_the_limit_of_files", "评审索引文件数超过 {MAX_INDEX_FILES} 个", "The review index exceeds the limit of {MAX_INDEX_FILES} files");

pub const THE_REVIEW_INDEX_FOR_THIS_COMMIT_IS_BEING_BUILT_RETRY_LATER: UiMessage = UiMessage::new("domain.the_review_index_for_this_commit_is_being_built_retry_later", "该提交的评审索引正在构建，请稍后重试", "The review index for this commit is being built; retry later");

pub const FAILED_TO_START_THE_INDEX_QUERY: UiMessage = UiMessage::new("domain.failed_to_start_the_index_query", "开启索引查询失败：{error}", "Failed to start the index query: {error}");

pub const FAILED_TO_FINISH_THE_INDEX_QUERY: UiMessage = UiMessage::new("domain.failed_to_finish_the_index_query", "结束索引查询失败：{error}", "Failed to finish the index query: {error}");

pub const FAILED_TO_PREPARE_THE_NODE_QUERY: UiMessage = UiMessage::new("domain.failed_to_prepare_the_node_query", "准备节点查询失败：{error}", "Failed to prepare the node query: {error}");

pub const FAILED_TO_QUERY_NODES: UiMessage = UiMessage::new("domain.failed_to_query_nodes", "查询节点失败：{error}", "Failed to query nodes: {error}");

pub const FAILED_TO_READ_NODES: UiMessage = UiMessage::new("domain.failed_to_read_nodes", "读取节点失败：{error}", "Failed to read nodes: {error}");

pub const FAILED_TO_READ_SOURCE_FILE_METADATA: UiMessage = UiMessage::new("domain.failed_to_read_source_file_metadata", "读取源文件元数据失败：{error}", "Failed to read source file metadata: {error}");

pub const FAILED_TO_PREPARE_THE_STATISTICS_QUERY: UiMessage = UiMessage::new("domain.failed_to_prepare_the_statistics_query", "准备统计查询失败：{error}", "Failed to prepare the statistics query: {error}");

pub const FAILED_TO_READ_STATISTICS: UiMessage = UiMessage::new("domain.failed_to_read_statistics", "读取统计失败：{error}", "Failed to read statistics: {error}");

pub const FAILED_TO_READ_DIRECTORY_STATISTICS: UiMessage = UiMessage::new("domain.failed_to_read_directory_statistics", "读取目录统计失败：{error}", "Failed to read directory statistics: {error}");

pub const FAILED_TO_READ_CALL_HOTSPOTS: UiMessage = UiMessage::new("domain.failed_to_read_call_hotspots", "读取调用热点失败：{error}", "Failed to read call hotspots: {error}");

pub const LEGACY_CALL_RELATIONSHIPS_MUST_BE_REBUILT_AND_CANNOT_SERVE_AS_CURRENT_EVIDENCE_CALL_R: UiMessage = UiMessage::new("domain.legacy_call_relationships_must_be_rebuilt_and_cannot_serve_as_current_evidence_call_r", "旧索引调用关系需重建，不能作为当前证据；请调用 refresh_index，并等待 index_status.needs_rebuild=false", "Legacy call relationships must be rebuilt and cannot serve as current evidence; call refresh_index and wait for index_status.needs_rebuild=false");

pub const ROOT_DIRECTORY: UiMessage = UiMessage::new("domain.root_directory", "(根目录)", "(Root directory)");

pub const FAILED_TO_CREATE_THE_INDEX_DIRECTORY: UiMessage = UiMessage::new("domain.failed_to_create_the_index_directory", "创建索引目录失败：{e}", "Failed to create the index directory: {e}");

pub const FAILED_TO_RESET_THE_INDEX_DATABASE: UiMessage = UiMessage::new("domain.failed_to_reset_the_index_database", "重置索引库失败：{e}", "Failed to reset the index database: {e}");

pub const FAILED_TO_OPEN_THE_INDEX_DATABASE: UiMessage = UiMessage::new("domain.failed_to_open_the_index_database", "打开索引库失败：{e}", "Failed to open the index database: {e}");

pub const FAILED_TO_MIGRATE_THE_CALL_RECORD_TABLE: UiMessage = UiMessage::new("domain.failed_to_migrate_the_call_record_table", "迁移调用记录表失败：{error}", "Failed to migrate the call record table: {error}");

pub const FAILED_TO_INITIALIZE_THE_INDEX_SCHEMA: UiMessage = UiMessage::new("domain.failed_to_initialize_the_index_schema", "初始化索引 schema 失败：{e}", "Failed to initialize the index schema: {e}");

pub const FAILED_TO_WRITE_INDEX_METADATA: UiMessage = UiMessage::new("domain.failed_to_write_index_metadata", "写入索引元信息失败：{e}", "Failed to write index metadata: {e}");

pub const FAILED_TO_START_INDEX_MIGRATION: UiMessage = UiMessage::new("domain.failed_to_start_index_migration", "开启索引迁移失败：{e}", "Failed to start index migration: {e}");

pub const FAILED_TO_UPDATE_THE_INDEX_VERSION: UiMessage = UiMessage::new("domain.failed_to_update_the_index_version", "更新索引版本失败：{e}", "Failed to update the index version: {e}");

pub const FAILED_TO_COMMIT_INDEX_MIGRATION: UiMessage = UiMessage::new("domain.failed_to_commit_index_migration", "提交索引迁移失败：{e}", "Failed to commit index migration: {e}");

pub const FAILED_TO_READ_THE_INDEXED_BRANCH: UiMessage = UiMessage::new("domain.failed_to_read_the_indexed_branch", "读取索引分支失败：{error}", "Failed to read the indexed branch: {error}");

pub const FAILED_TO_READ_INDEX_RETRY_INFORMATION: UiMessage = UiMessage::new("domain.failed_to_read_index_retry_information", "读取索引重试信息失败：{e}", "Failed to read index retry information: {e}");

pub const FAILED_TO_READ_DISCOVERY_COVERAGE: UiMessage = UiMessage::new("domain.failed_to_read_discovery_coverage", "读取发现覆盖信息失败：{error}", "Failed to read discovery coverage: {error}");

pub const CORRUPT_DISCOVERY_COVERAGE_INFORMATION: UiMessage = UiMessage::new("domain.corrupt_discovery_coverage_information", "发现覆盖信息损坏：{error}", "Corrupt discovery coverage information: {error}");

pub const FAILED_TO_START_THE_INDEX_TRANSACTION: UiMessage = UiMessage::new("domain.failed_to_start_the_index_transaction", "开启索引事务失败：{e}", "Failed to start the index transaction: {e}");

pub const FAILED_TO_CLEAR_AUXILIARY_DATABASE_INDEXES: UiMessage = UiMessage::new("domain.failed_to_clear_auxiliary_database_indexes", "清理索引辅助索引失败：{e}", "Failed to clear auxiliary database indexes: {e}");

pub const FAILED_TO_CLEAR_THE_OLD_GRAPH: UiMessage = UiMessage::new("domain.failed_to_clear_the_old_graph", "清空旧图失败：{e}", "Failed to clear the old graph: {e}");

pub const FAILED_TO_CLEAR_THE_FULL_TEXT_INDEX: UiMessage = UiMessage::new("domain.failed_to_clear_the_full_text_index", "清空全文索引失败：{e}", "Failed to clear the full-text index: {e}");

pub const FAILED_TO_BATCH_WRITE_NODES: UiMessage = UiMessage::new("domain.failed_to_batch_write_nodes", "批量写入节点失败：{error}", "Failed to batch-write nodes: {error}");

pub const FAILED_TO_BATCH_WRITE_RELATIONSHIPS: UiMessage = UiMessage::new("domain.failed_to_batch_write_relationships", "批量写入关系失败：{error}", "Failed to batch-write relationships: {error}");

pub const FAILED_TO_REBUILD_AUXILIARY_DATABASE_INDEXES: UiMessage = UiMessage::new("domain.failed_to_rebuild_auxiliary_database_indexes", "重建辅助索引失败：{e}", "Failed to rebuild auxiliary database indexes: {e}");

pub const FAILED_TO_CLEAR_CALL_RECORDS: UiMessage = UiMessage::new("domain.failed_to_clear_call_records", "清理调用记录失败：{error}", "Failed to clear call records: {error}");

pub const FAILED_TO_PREPARE_FILE_HASH_WRITES: UiMessage = UiMessage::new("domain.failed_to_prepare_file_hash_writes", "准备文件哈希写入失败：{e}", "Failed to prepare file hash writes: {e}");

pub const FAILED_TO_WRITE_FILE_HASHES: UiMessage = UiMessage::new("domain.failed_to_write_file_hashes", "写入文件哈希失败：{e}", "Failed to write file hashes: {e}");

pub const FAILED_TO_COMMIT_THE_INDEX_TRANSACTION: UiMessage = UiMessage::new("domain.failed_to_commit_the_index_transaction", "提交索引事务失败：{e}", "Failed to commit the index transaction: {e}");

pub const FAILED_TO_LOCATE_AFFECTED_CALLS: UiMessage = UiMessage::new("domain.failed_to_locate_affected_calls", "定位受影响调用失败：{error}", "Failed to locate affected calls: {error}");

pub const FAILED_TO_READ_CALL_FILES: UiMessage = UiMessage::new("domain.failed_to_read_call_files", "读取调用文件失败：{error}", "Failed to read call files: {error}");

pub const FAILED_TO_READ_CALL_RECORDS: UiMessage = UiMessage::new("domain.failed_to_read_call_records", "读取调用记录失败：{error}", "Failed to read call records: {error}");

pub const FAILED_TO_READ_CALL_DATA: UiMessage = UiMessage::new("domain.failed_to_read_call_data", "读取调用数据失败：{error}", "Failed to read call data: {error}");

pub const FAILED_TO_START_THE_INCREMENTAL_TRANSACTION: UiMessage = UiMessage::new("domain.failed_to_start_the_incremental_transaction", "开启增量事务失败：{error}", "Failed to start the incremental transaction: {error}");

pub const FAILED_TO_DELETE_OLD_FILE_NODES: UiMessage = UiMessage::new("domain.failed_to_delete_old_file_nodes", "删除旧文件节点失败：{error}", "Failed to delete old file nodes: {error}");

pub const FAILED_TO_DELETE_OLD_FILE_METADATA: UiMessage = UiMessage::new("domain.failed_to_delete_old_file_metadata", "删除旧文件元数据失败：{error}", "Failed to delete old file metadata: {error}");

pub const FAILED_TO_DELETE_OLD_CALL_RECORDS: UiMessage = UiMessage::new("domain.failed_to_delete_old_call_records", "删除旧调用记录失败：{error}", "Failed to delete old call records: {error}");

pub const FAILED_TO_READ_NODE_IDS: UiMessage = UiMessage::new("domain.failed_to_read_node_ids", "读取节点编号失败：{error}", "Failed to read node IDs: {error}");

pub const FAILED_TO_UPDATE_NODE_COVERAGE: UiMessage = UiMessage::new("domain.failed_to_update_node_coverage", "更新节点覆盖失败：{error}", "Failed to update node coverage: {error}");

pub const FAILED_TO_WRITE_INCREMENTAL_NODES: UiMessage = UiMessage::new("domain.failed_to_write_incremental_nodes", "增量写入节点失败：{error}", "Failed to write incremental nodes: {error}");

pub const FAILED_TO_CLEAR_AFFECTED_CALLS: UiMessage = UiMessage::new("domain.failed_to_clear_affected_calls", "清理受影响调用失败：{error}", "Failed to clear affected calls: {error}");

pub const FAILED_TO_PREPARE_INCREMENTAL_RELATIONSHIPS: UiMessage = UiMessage::new("domain.failed_to_prepare_incremental_relationships", "准备增量关系失败：{error}", "Failed to prepare incremental relationships: {error}");

pub const FAILED_TO_WRITE_INCREMENTAL_RELATIONSHIPS: UiMessage = UiMessage::new("domain.failed_to_write_incremental_relationships", "增量写入关系失败：{error}", "Failed to write incremental relationships: {error}");

pub const FAILED_TO_READ_OLD_STRUCTURAL_NODES: UiMessage = UiMessage::new("domain.failed_to_read_old_structural_nodes", "读取旧结构节点失败：{error}", "Failed to read old structural nodes: {error}");

pub const FAILED_TO_DELETE_STALE_STRUCTURAL_NODES: UiMessage = UiMessage::new("domain.failed_to_delete_stale_structural_nodes", "删除过期结构节点失败：{error}", "Failed to delete stale structural nodes: {error}");

pub const FAILED_TO_PREPARE_INCREMENTAL_FILE_METADATA: UiMessage = UiMessage::new("domain.failed_to_prepare_incremental_file_metadata", "准备增量文件元数据失败：{error}", "Failed to prepare incremental file metadata: {error}");

pub const FAILED_TO_WRITE_INCREMENTAL_FILE_METADATA: UiMessage = UiMessage::new("domain.failed_to_write_incremental_file_metadata", "写入增量文件元数据失败：{error}", "Failed to write incremental file metadata: {error}");

pub const FAILED_TO_COMMIT_THE_INCREMENTAL_TRANSACTION: UiMessage = UiMessage::new("domain.failed_to_commit_the_incremental_transaction", "提交增量事务失败：{error}", "Failed to commit the incremental transaction: {error}");

pub const FAILED_TO_READ_NODES_2: UiMessage = UiMessage::new("domain.failed_to_read_nodes_2", "读取节点失败：{e}", "Failed to read nodes: {e}");

pub const FAILED_TO_QUERY_NODES_2: UiMessage = UiMessage::new("domain.failed_to_query_nodes_2", "查询节点失败：{e}", "Failed to query nodes: {e}");

pub const FAILED_TO_READ_EDGES: UiMessage = UiMessage::new("domain.failed_to_read_edges", "读取边失败：{e}", "Failed to read edges: {e}");

pub const FAILED_TO_QUERY_EDGES: UiMessage = UiMessage::new("domain.failed_to_query_edges", "查询边失败：{e}", "Failed to query edges: {e}");

pub const FAILED_TO_READ_EDGES_2: UiMessage = UiMessage::new("domain.failed_to_read_edges_2", "读取边失败：{error}", "Failed to read edges: {error}");

pub const FAILED_TO_READ_FILE_HASHES: UiMessage = UiMessage::new("domain.failed_to_read_file_hashes", "读取文件哈希失败：{e}", "Failed to read file hashes: {e}");

pub const FAILED_TO_QUERY_FILE_HASHES: UiMessage = UiMessage::new("domain.failed_to_query_file_hashes", "查询文件哈希失败：{e}", "Failed to query file hashes: {e}");

pub const FAILED_TO_READ_FILE_HASHES_2: UiMessage = UiMessage::new("domain.failed_to_read_file_hashes_2", "读取文件哈希失败：{error}", "Failed to read file hashes: {error}");

pub const FAILED_TO_START_THE_INDEX_STATISTICS_SNAPSHOT: UiMessage = UiMessage::new("domain.failed_to_start_the_index_statistics_snapshot", "开启索引统计快照失败：{error}", "Failed to start the index statistics snapshot: {error}");

pub const FAILED_TO_FINISH_THE_INDEX_STATISTICS_SNAPSHOT: UiMessage = UiMessage::new("domain.failed_to_finish_the_index_statistics_snapshot", "结束索引统计快照失败：{error}", "Failed to finish the index statistics snapshot: {error}");

pub const FAILED_TO_READ_THE_INDEX_GENERATION: UiMessage = UiMessage::new("domain.failed_to_read_the_index_generation", "读取索引代际失败：{error}", "Failed to read the index generation: {error}");

pub const FAILED_TO_COMPUTE_INDEX_STATISTICS: UiMessage = UiMessage::new("domain.failed_to_compute_index_statistics", "统计索引失败：{error}", "Failed to compute index statistics: {error}");

pub const FAILED_TO_CLEAR_OLD_METADATA: UiMessage = UiMessage::new("domain.failed_to_clear_old_metadata", "清理旧元信息失败：{e}", "Failed to clear old metadata: {e}");

pub const FAILED_TO_GENERATE_AN_INDEX_GENERATION: UiMessage = UiMessage::new("domain.failed_to_generate_an_index_generation", "生成索引代际失败：{e}", "Failed to generate an index generation: {e}");

pub const FAILED_TO_PREPARE_CALL_RECORDS: UiMessage = UiMessage::new("domain.failed_to_prepare_call_records", "准备调用记录失败：{error}", "Failed to prepare call records: {error}");

pub const FAILED_TO_WRITE_CALL_RECORDS: UiMessage = UiMessage::new("domain.failed_to_write_call_records", "写入调用记录失败：{error}", "Failed to write call records: {error}");

pub const FAILED_TO_CLEAR_CALL_NAMES: UiMessage = UiMessage::new("domain.failed_to_clear_call_names", "清理调用名字失败：{error}", "Failed to clear call names: {error}");

pub const FAILED_TO_WRITE_CALL_NAMES: UiMessage = UiMessage::new("domain.failed_to_write_call_names", "写入调用名字失败：{error}", "Failed to write call names: {error}");

pub const FAILED_TO_CREATE_THE_INDEX_DATABASE: UiMessage = UiMessage::new("domain.failed_to_create_the_index_database", "索引库创建失败", "Failed to create the index database");

pub const COVERAGE_NON_UTF8_PATH: UiMessage = UiMessage::new("domain.coverage_non_utf8_path", "非 UTF-8 路径", "Non-UTF-8 path");
pub const COVERAGE_FILTERED_SNAPSHOT_ENTRY: UiMessage = UiMessage::new("domain.coverage_filtered_snapshot_entry", "目录或产物过滤、符号链接、子模块", "Filtered directory or artifact, symbolic link, or submodule");
