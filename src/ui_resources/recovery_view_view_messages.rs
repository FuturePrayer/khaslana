//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const FAILED_TO_READ_COMMIT_OBJECT_ERROR: UiMessage = UiMessage::new("views.recovery_view.view.failed_to_read_commit_object_error", "提交对象读取失败：{error}", "Failed to read commit object: {error}");
pub const BRANCH_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.branch_branch", "分支：{branch}", "Branch: {branch}");
pub const CURRENT_BRANCH_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.current_branch_branch", "当前分支：{branch}", "Current branch: {branch}");
pub const LAST_DAYS_DAYS: UiMessage = UiMessage::new("views.recovery_view.view.last_days_days", "近 {days} 天", "Last {days} days");
pub const VIEW_LOCAL_RECOVERY_BRANCH_NAME: UiMessage = UiMessage::new("views.recovery_view.view.view_local_recovery_branch_name", "查看本地恢复分支 {name}", "View local recovery branch {name}");
pub const LOCAL_BRANCH_CREATED_NAME: UiMessage = UiMessage::new("views.recovery_view.view.local_branch_created_name", "已创建本地分支：{name}", "Local branch created: {name}");
pub const DIAGNOSTIC_DIAGNOSTICS_MORE: UiMessage = UiMessage::new("views.recovery_view.view.diagnostic_diagnostics_more", "{diagnostic}（另有 {diagnostics} 项）", "{diagnostic} ({diagnostics} more)");
pub const SOURCE_NAME: UiMessage = UiMessage::new("views.recovery_view.view.source_name", "来源：{name}", "Source: {name}");
pub const VIEW_RECOVERY_BRANCH_NAME: UiMessage = UiMessage::new("views.recovery_view.view.view_recovery_branch_name", "查看恢复分支 {name}", "View recovery branch {name}");
pub const TOTAL_RECORDS: UiMessage = UiMessage::new("views.recovery_view.view.total_records", "{total} 条", "{total} records");
pub const SHOWING_ENTRIES_TOTAL_RECORDS: UiMessage = UiMessage::new("views.recovery_view.view.showing_entries_total_records", "已显示 {entries} / {total} 条", "Showing {entries} / {total} records");
pub const BEFORE_OLD_STATE_AFTER_NEW_STATE: UiMessage = UiMessage::new("views.recovery_view.view.before_old_state_after_new_state", "· 前：{old_state} / 后：{new_state}", "· Before: {old_state} / After: {new_state}");
pub const ENCODING_LABEL: UiMessage = UiMessage::new("views.recovery_view.view.encoding_label", "编码：{label}", "Encoding: {label}");
pub const MERGE_COMMIT_PREVIEW_RELATIVE_TO_FIRST_PARENT_PARENTS: UiMessage = UiMessage::new("views.recovery_view.view.merge_commit_preview_relative_to_first_parent_parents", "合并提交：以第一父提交 {parents} 为基准预览。", "Merge commit: preview relative to first parent {parents}.");
pub const COMMIT_FILES_FILES: UiMessage = UiMessage::new("views.recovery_view.view.commit_files_files", "提交文件 · {files}", "Commit files · {files}");
pub const NO_COMMIT: UiMessage = UiMessage::new("views.recovery_view.view.no_commit", "无提交", "No commit");
pub const AVAILABLE: UiMessage = UiMessage::new("views.recovery_view.view.available", "可用", "Available");
pub const UNAVAILABLE_LOCALLY: UiMessage = UiMessage::new("views.recovery_view.view.unavailable_locally", "本地不可用", "Unavailable locally");
pub const NOT_A_COMMIT_OBJECT: UiMessage = UiMessage::new("views.recovery_view.view.not_a_commit_object", "非提交对象", "Not a commit object");
pub const READ_FAILED: UiMessage = UiMessage::new("views.recovery_view.view.read_failed", "读取失败", "Read failed");
pub const NOT_CHECKED: UiMessage = UiMessage::new("views.recovery_view.view.not_checked", "待检查", "Not checked");
pub const THE_COMMIT_OBJECT_IS_AVAILABLE_VIEW_IT_OR_CREATE_A: UiMessage = UiMessage::new("views.recovery_view.view.the_commit_object_is_available_view_it_or_create_a", "提交对象可用，可以查看提交或从此提交创建本地分支。", "The commit object is available. View it or create a local branch from it.");
pub const THERE_IS_NO_COMMIT_ON_THIS_SIDE_A_ZERO_OID: UiMessage = UiMessage::new("views.recovery_view.view.there_is_no_commit_on_this_side_a_zero_oid", "此侧没有提交。零 OID 表示引用尚未创建或已经删除，无法作为分支目标。", "There is no commit on this side. A zero OID means the reference was not created yet or has been deleted, so it cannot be a branch target.");
pub const THE_COMMIT_OBJECT_IS_UNAVAILABLE_LOCALLY_IT_MAY_HAVE_BEEN: UiMessage = UiMessage::new("views.recovery_view.view.the_commit_object_is_unavailable_locally_it_may_have_been", "当前本地提交对象不可用，可能已被清理或尚未存在于本地。可以复制完整 OID 继续查找。", "The commit object is unavailable locally. It may have been pruned or may not exist locally yet. Copy the full OID to investigate.");
pub const THIS_OID_DOES_NOT_POINT_TO_A_COMMIT_AND_CANNOT: UiMessage = UiMessage::new("views.recovery_view.view.this_oid_does_not_point_to_a_commit_and_cannot", "该 OID 指向的对象不是提交，不能从此对象创建恢复分支。", "This OID does not point to a commit and cannot be used to create a recovery branch.");
pub const LOCAL_COMMIT_AVAILABILITY_HAS_NOT_BEEN_CHECKED_WAIT_FOR_VERIFICATION: UiMessage = UiMessage::new("views.recovery_view.view.local_commit_availability_has_not_been_checked_wait_for_verification", "尚未确认本地提交对象是否可用，确认完成后才能查看提交或创建分支。", "Local commit availability has not been checked. Wait for verification before viewing the commit or creating a branch.");
pub const COMMIT_HISTORY: UiMessage = UiMessage::new("views.recovery_view.view.commit_history", "提交历史", "Commit history");
pub const RECOVERY_RECORDS: UiMessage = UiMessage::new("views.recovery_view.view.recovery_records", "恢复记录", "Recovery records");
pub const VIEW_THE_CURRENT_WORKTREE_S_LOCAL_REFERENCE_LOGS: UiMessage = UiMessage::new("views.recovery_view.view.view_the_current_worktree_s_local_reference_logs", "查看当前工作区的本地引用日志", "View the current worktree’s local reference logs");
pub const NO_REPOSITORY_OPEN: UiMessage = UiMessage::new("views.recovery_view.view.no_repository_open", "还没有打开的仓库", "No repository open");
pub const OPEN_A_LOCAL_REPOSITORY_TO_VIEW_RECOVERY_RECORDS_FOR_THE: UiMessage = UiMessage::new("views.recovery_view.view.open_a_local_repository_to_view_recovery_records_for_the", "打开一个本地仓库后，可查看当前工作区的恢复记录", "Open a local repository to view recovery records for the current worktree");
pub const OPEN_REPOSITORY: UiMessage = UiMessage::new("views.recovery_view.view.open_repository", "打开仓库…", "Open repository…");
pub const LOCATE_COMMITS_USING_LOCAL_REFERENCE_LOGS: UiMessage = UiMessage::new("views.recovery_view.view.locate_commits_using_local_reference_logs", "从本地引用日志定位提交", "Locate commits using local reference logs");
pub const RECORDS: UiMessage = UiMessage::new("views.recovery_view.view.records", "记录", "Records");
pub const DETAILS: UiMessage = UiMessage::new("views.recovery_view.view.details", "详情", "Details");
pub const VIEW_COMMITS_BEFORE_AND_AFTER_THE_SELECTED_OPERATION: UiMessage = UiMessage::new("views.recovery_view.view.view_commits_before_and_after_the_selected_operation", "查看选中记录的操作前后提交", "View commits before and after the selected operation");
pub const SELECT_A_RECOVERY_RECORD_FIRST: UiMessage = UiMessage::new("views.recovery_view.view.select_a_recovery_record_first", "请先选择一条恢复记录", "Select a recovery record first");
pub const RECOVERY_RECORDS_ARE_NOT_PERMANENT_BACKUPS_OBJECTS_MAY_BE_PRUNED: UiMessage = UiMessage::new("views.recovery_view.view.recovery_records_are_not_permanent_backups_objects_may_be_pruned", "恢复记录不是永久备份；对象可能被清理，未提交修改通常不在记录中。", "Recovery records are not permanent backups. Objects may be pruned, and uncommitted changes are usually not recorded.");
pub const SPECIFIC_LOCAL_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.specific_local_branch", "指定本地分支", "Specific local branch");
pub const NO_LOCAL_BRANCHES_ARE_AVAILABLE_IN_THE_CURRENT_WORKTREE: UiMessage = UiMessage::new("views.recovery_view.view.no_local_branches_are_available_in_the_current_worktree", "当前工作区没有可选的本地分支", "No local branches are available in the current worktree");
pub const SOURCE: UiMessage = UiMessage::new("views.recovery_view.view.source", "来源", "Source");
pub const CURRENT_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.current_branch", "当前分支", "Current branch");
pub const ALL: UiMessage = UiMessage::new("views.recovery_view.view.all", "全部", "All");
pub const HEAD_IS_DETACHED_THERE_IS_NO_CURRENT_BRANCH_LOG: UiMessage = UiMessage::new("views.recovery_view.view.head_is_detached_there_is_no_current_branch_log", "当前处于分离 HEAD，没有当前分支日志", "HEAD is detached; there is no current branch log");
pub const HEAD_AND_ALL_LOCAL_BRANCH_LOGS: UiMessage = UiMessage::new("views.recovery_view.view.head_and_all_local_branch_logs", "HEAD 与所有本地分支日志", "HEAD and all local branch logs");
pub const CURRENT_WORKTREE_HEAD_LOG: UiMessage = UiMessage::new("views.recovery_view.view.current_worktree_head_log", "当前工作区的 HEAD 日志", "Current worktree HEAD log");
pub const ALL_TIME: UiMessage = UiMessage::new("views.recovery_view.view.all_time", "全部时间", "All time");
pub const FILTER_BY_WHEN_THE_RECOVERY_OPERATION_OCCURRED: UiMessage = UiMessage::new("views.recovery_view.view.filter_by_when_the_recovery_operation_occurred", "按恢复操作发生的时间筛选", "Filter by when the recovery operation occurred");
pub const LAST_DAYS: UiMessage = UiMessage::new("views.recovery_view.view.last_days", "近 7 天", "Last 7 days");
pub const LAST_DAYS_2: UiMessage = UiMessage::new("views.recovery_view.view.last_days_2", "近 30 天", "Last 30 days");
pub const LAST_DAYS_3: UiMessage = UiMessage::new("views.recovery_view.view.last_days_3", "近 90 天", "Last 90 days");
pub const SEARCH: UiMessage = UiMessage::new("views.recovery_view.view.search", "搜索", "Search");
pub const SEARCH_LOADED_LOCAL_LOGS: UiMessage = UiMessage::new("views.recovery_view.view.search_loaded_local_logs", "在已读取的本地日志中搜索", "Search loaded local logs");
pub const READING: UiMessage = UiMessage::new("views.recovery_view.view.reading", "读取中…", "Reading…");
pub const REFRESH: UiMessage = UiMessage::new("views.recovery_view.view.refresh", "刷新", "Refresh");
pub const READING_LOCAL_RECOVERY_RECORDS: UiMessage = UiMessage::new("views.recovery_view.view.reading_local_recovery_records", "正在读取本地恢复记录", "Reading local recovery records");
pub const RELOAD_LOCAL_RECOVERY_RECORDS: UiMessage = UiMessage::new("views.recovery_view.view.reload_local_recovery_records", "重新读取本地恢复记录", "Reload local recovery records");
pub const CANCEL_READ: UiMessage = UiMessage::new("views.recovery_view.view.cancel_read", "取消读取", "Cancel read");
pub const VIEW_RECOVERY_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.view_recovery_branch", "查看恢复分支", "View recovery branch");
pub const SOURCE_HEAD: UiMessage = UiMessage::new("views.recovery_view.view.source_head", "来源：HEAD", "Source: HEAD");
pub const SOURCE_CURRENT_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.source_current_branch", "来源：当前分支", "Source: current branch");
pub const SOURCE_ALL: UiMessage = UiMessage::new("views.recovery_view.view.source_all", "来源：全部", "Source: all");
pub const ALL_LOCAL_SOURCES: UiMessage = UiMessage::new("views.recovery_view.view.all_local_sources", "全部本地来源", "All local sources");
pub const CHOOSE_HEAD_THE_CURRENT_BRANCH_OR_ANOTHER_LOCAL_SOURCE: UiMessage = UiMessage::new("views.recovery_view.view.choose_head_the_current_branch_or_another_local_source", "选择 HEAD、当前分支或其他本地来源", "Choose HEAD, the current branch, or another local source");
pub const FILTER_RECOVERY_OPERATION_TIME: UiMessage = UiMessage::new("views.recovery_view.view.filter_recovery_operation_time", "筛选恢复操作的时间", "Filter recovery operation time");
pub const ACTIONS: UiMessage = UiMessage::new("views.recovery_view.view.actions", "操作", "Actions");
pub const SEARCH_REFRESH_AND_VIEW_CREATED_BRANCHES: UiMessage = UiMessage::new("views.recovery_view.view.search_refresh_and_view_created_branches", "搜索、刷新和查看已创建分支", "Search, refresh, and view created branches");
pub const FILTERING: UiMessage = UiMessage::new("views.recovery_view.view.filtering", "筛选中…", "Filtering…");
pub const LOADING: UiMessage = UiMessage::new("views.recovery_view.view.loading", "加载中…", "Loading…");
pub const LOAD_MORE: UiMessage = UiMessage::new("views.recovery_view.view.load_more", "加载更多", "Load more");
pub const LOAD_UP_TO_RECORDS_AT_A_TIME: UiMessage = UiMessage::new("views.recovery_view.view.load_up_to_records_at_a_time", "每次加载最多 100 条记录", "Load up to 100 records at a time");
pub const READING_LOCAL_RECOVERY_RECORDS_2: UiMessage = UiMessage::new("views.recovery_view.view.reading_local_recovery_records_2", "正在读取本地恢复记录…", "Reading local recovery records…");
pub const FILTERING_RECOVERY_RECORDS: UiMessage = UiMessage::new("views.recovery_view.view.filtering_recovery_records", "正在筛选恢复记录…", "Filtering recovery records…");
pub const READ_INCOMPLETE_REFRESH_TO_TRY_AGAIN: UiMessage = UiMessage::new("views.recovery_view.view.read_incomplete_refresh_to_try_again", "读取未完成，请刷新重试", "Read incomplete. Refresh to try again.");
pub const NO_RECOVERY_RECORDS_MATCH_THE_CURRENT_SOURCE_AND_FILTERS: UiMessage = UiMessage::new("views.recovery_view.view.no_recovery_records_match_the_current_source_and_filters", "当前来源与筛选条件下没有恢复记录", "No recovery records match the current source and filters");
pub const NO_OPERATION_DESCRIPTION: UiMessage = UiMessage::new("views.recovery_view.view.no_operation_description", "（无操作说明）", "(No operation description)");
pub const COMMIT_PREVIEW: UiMessage = UiMessage::new("views.recovery_view.view.commit_preview", "提交预览", "Commit preview");
pub const RECOVERY_DETAILS: UiMessage = UiMessage::new("views.recovery_view.view.recovery_details", "恢复详情", "Recovery details");
pub const BACK_TO_DETAILS: UiMessage = UiMessage::new("views.recovery_view.view.back_to_details", "返回详情", "Back to details");
pub const SELECT_A_RECOVERY_RECORD: UiMessage = UiMessage::new("views.recovery_view.view.select_a_recovery_record", "选择一条恢复记录", "Select a recovery record");
pub const COMPARE_FULL_OIDS_BEFORE_AND_AFTER_THE_OPERATION_VERIFY_THE: UiMessage = UiMessage::new("views.recovery_view.view.compare_full_oids_before_and_after_the_operation_verify_the", "对比操作前后的完整 OID，确认目标提交后创建本地分支", "Compare full OIDs before and after the operation, verify the target commit, then create a local branch");
pub const READING_TARGET_COMMIT_PLEASE_WAIT: UiMessage = UiMessage::new("views.recovery_view.view.reading_target_commit_please_wait", "正在读取目标提交，请稍候", "Reading target commit. Please wait.");
pub const AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT: UiMessage = UiMessage::new("views.recovery_view.view.an_operation_is_in_progress_please_wait", "当前操作进行中，请稍候", "An operation is in progress. Please wait.");
pub const COPY_OID: UiMessage = UiMessage::new("views.recovery_view.view.copy_oid", "复制 OID", "Copy OID");
pub const VIEW_COMMIT: UiMessage = UiMessage::new("views.recovery_view.view.view_commit", "查看提交", "View commit");
pub const CREATE_BRANCH_FROM_THIS_COMMIT: UiMessage = UiMessage::new("views.recovery_view.view.create_branch_from_this_commit", "从此提交创建分支", "Create branch from this commit");
pub const CREATE_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.create_branch", "创建分支", "Create branch");
pub const BEFORE_OPERATION_FULL_TARGET_OID: UiMessage = UiMessage::new("views.recovery_view.view.before_operation_full_target_oid", "操作前 · 目标完整 OID", "Before operation · Full target OID");
pub const AFTER_OPERATION_FULL_TARGET_OID: UiMessage = UiMessage::new("views.recovery_view.view.after_operation_full_target_oid", "操作后 · 目标完整 OID", "After operation · Full target OID");
pub const READING_TARGET_COMMIT: UiMessage = UiMessage::new("views.recovery_view.view.reading_target_commit", "正在读取目标提交…", "Reading target commit…");
pub const AUTHOR: UiMessage = UiMessage::new("views.recovery_view.view.author", "作者", "Author");
pub const COMMIT_TIME: UiMessage = UiMessage::new("views.recovery_view.view.commit_time", "提交时间", "Commit time");
pub const PARENT_COMMITS: UiMessage = UiMessage::new("views.recovery_view.view.parent_commits", "父提交", "Parent commits");
pub const COMMITTER: UiMessage = UiMessage::new("views.recovery_view.view.committer", "提交者", "Committer");
pub const FULL_COMMIT_MESSAGE: UiMessage = UiMessage::new("views.recovery_view.view.full_commit_message", "完整提交说明", "Full commit message");
pub const THIS_IS_A_MERGE_COMMIT_FILES_AND_DIFFERENCES_ARE_SHOWN: UiMessage = UiMessage::new("views.recovery_view.view.this_is_a_merge_commit_files_and_differences_are_shown", "这是合并提交，文件列表与差异以第一父提交为基准。", "This is a merge commit. Files and differences are shown relative to its first parent.");
pub const LOG_EVIDENCE: UiMessage = UiMessage::new("views.recovery_view.view.log_evidence", "日志证据", "Log evidence");
pub const OPERATION_DESCRIPTION: UiMessage = UiMessage::new("views.recovery_view.view.operation_description", "操作说明", "Operation description");
pub const SOURCE_REFERENCE: UiMessage = UiMessage::new("views.recovery_view.view.source_reference", "来源引用", "Source reference");
pub const OPERATION_TIME: UiMessage = UiMessage::new("views.recovery_view.view.operation_time", "操作时间", "Operation time");
pub const ACTOR: UiMessage = UiMessage::new("views.recovery_view.view.actor", "操作者", "Actor");
pub const FULL_OID_BEFORE_OPERATION: UiMessage = UiMessage::new("views.recovery_view.view.full_oid_before_operation", "操作前完整 OID", "Full OID before operation");
pub const FULL_OID_AFTER_OPERATION: UiMessage = UiMessage::new("views.recovery_view.view.full_oid_after_operation", "操作后完整 OID", "Full OID after operation");
pub const THE_ORIGINAL_LOG_CONTAINS_INVALID_UTF_AND_IS_DISPLAYED_WITH: UiMessage = UiMessage::new("views.recovery_view.view.the_original_log_contains_invalid_utf_and_is_displayed_with", "原始日志包含无法按 UTF-8 解码的内容，展示时已替换。恢复目标仍使用完整 OID。", "The original log contains invalid UTF-8 and is displayed with replacement characters. Recovery still uses the full OID.");
pub const CHOOSE_DIFF_DECODING_FOR_THE_CURRENT_REPOSITORY: UiMessage = UiMessage::new("views.recovery_view.view.choose_diff_decoding_for_the_current_repository", "选择当前仓库的差异解码方式", "Choose diff decoding for the current repository");
pub const DIFF: UiMessage = UiMessage::new("views.recovery_view.view.diff", "差异", "Diff");
pub const FULL_FILE: UiMessage = UiMessage::new("views.recovery_view.view.full_file", "全文", "Full file");
pub const SHOW_DIFFERENCES_AND_SURROUNDING_CONTEXT: UiMessage = UiMessage::new("views.recovery_view.view.show_differences_and_surrounding_context", "显示差异与相邻上下文", "Show differences and surrounding context");
pub const SHOW_FULL_FILE_CONTEXT: UiMessage = UiMessage::new("views.recovery_view.view.show_full_file_context", "显示完整文件上下文", "Show full file context");
pub const COMMIT_DIFF: UiMessage = UiMessage::new("views.recovery_view.view.commit_diff", "提交差异", "Commit diff");
pub const LOADING_COMMIT_DIFFERENCES: UiMessage = UiMessage::new("views.recovery_view.view.loading_commit_differences", "提交差异加载中…", "Loading commit differences…");
pub const SELECT_A_COMMIT_FILE_TO_VIEW_ITS_DIFFERENCES: UiMessage = UiMessage::new("views.recovery_view.view.select_a_commit_file_to_view_its_differences", "选择一个提交文件查看差异", "Select a commit file to view its differences");
pub const LOADING_COMMIT_FILES: UiMessage = UiMessage::new("views.recovery_view.view.loading_commit_files", "提交文件加载中…", "Loading commit files…");
pub const FAILED_TO_READ_FILES: UiMessage = UiMessage::new("views.recovery_view.view.failed_to_read_files", "文件读取失败", "Failed to read files");
pub const THIS_COMMIT_HAS_NO_FILE_CHANGES: UiMessage = UiMessage::new("views.recovery_view.view.this_commit_has_no_file_changes", "该提交没有文件变更", "This commit has no file changes");
pub const WORKING_TREE: UiMessage = UiMessage::new("views.recovery_view.view.working_tree", "工作区", "Working tree");
pub const FULL_TARGET_OID: UiMessage = UiMessage::new("views.recovery_view.view.full_target_oid", "目标完整 OID", "Full target OID");
pub const SOURCE_RECORD: UiMessage = UiMessage::new("views.recovery_view.view.source_record", "来源记录", "Source record");
pub const RECOVERY_TARGET: UiMessage = UiMessage::new("views.recovery_view.view.recovery_target", "恢复目标", "Recovery target");
pub const LOCAL_BRANCH_NAME: UiMessage = UiMessage::new("views.recovery_view.view.local_branch_name", "本地分支名称", "Local branch name");
pub const ONLY_CREATES_A_LOCAL_BRANCH_POINTING_TO_THIS_COMMIT_THE: UiMessage = UiMessage::new("views.recovery_view.view.only_creates_a_local_branch_pointing_to_this_commit_the", "只创建指向此提交的本地分支，不切换当前分支，未提交修改不受影响。", "Only creates a local branch pointing to this commit. The current branch and uncommitted changes are not affected.");
pub const CREATE_RECOVERY_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.create_recovery_branch", "创建恢复分支", "Create recovery branch");
pub const CANCEL: UiMessage = UiMessage::new("views.recovery_view.view.cancel", "取消", "Cancel");
pub const CREATING: UiMessage = UiMessage::new("views.recovery_view.view.creating", "创建中…", "Creating…");
pub const CREATE_LOCAL_BRANCH: UiMessage = UiMessage::new("views.recovery_view.view.create_local_branch", "创建本地分支", "Create local branch");

pub const FULL_FILE_TITLE: UiMessage = UiMessage::new("views.recovery_view_view.full_file_title", "提交全文：{path}", "Commit full file: {path}");

pub const DIFF_TITLE: UiMessage = UiMessage::new("views.recovery_view_view.diff_title", "提交差异：{path}", "Commit diff: {path}");

pub const RECORD_TOOLTIP: UiMessage = UiMessage::new("views.recovery.record.record_tooltip", "{message}\n{timestamp} · {reference}\n操作前：{before} · {before_state}\n操作后：{after} · {after_state}", "{message}\n{timestamp} · {reference}\nBefore: {before} · {before_state}\nAfter: {after} · {after_state}");

pub const RECORD_TOOLTIP_LOSSY: UiMessage = UiMessage::new("views.recovery.record.record_tooltip_lossy", "{message}\n{timestamp} · {reference}\n操作前：{before} · {before_state}\n操作后：{after} · {after_state}\n原始日志包含无法按 UTF-8 解码的内容，展示时已替换。", "{message}\n{timestamp} · {reference}\nBefore: {before} · {before_state}\nAfter: {after} · {after_state}\nThe original log contains invalid UTF-8 and is displayed with replacement characters.");

pub const COPY_BEFORE_OID: UiMessage = UiMessage::new("views.recovery.copy.copy_before_oid", "复制操作前完整 OID：{oid}", "Copy full OID before operation: {oid}");

pub const COPY_AFTER_OID: UiMessage = UiMessage::new("views.recovery.copy.copy_after_oid", "复制操作后完整 OID：{oid}", "Copy full OID after operation: {oid}");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    FAILED_TO_READ_COMMIT_OBJECT_ERROR,
    BRANCH_BRANCH,
    CURRENT_BRANCH_BRANCH,
    LAST_DAYS_DAYS,
    VIEW_LOCAL_RECOVERY_BRANCH_NAME,
    LOCAL_BRANCH_CREATED_NAME,
    DIAGNOSTIC_DIAGNOSTICS_MORE,
    SOURCE_NAME,
    VIEW_RECOVERY_BRANCH_NAME,
    TOTAL_RECORDS,
    SHOWING_ENTRIES_TOTAL_RECORDS,
    BEFORE_OLD_STATE_AFTER_NEW_STATE,
    ENCODING_LABEL,
    MERGE_COMMIT_PREVIEW_RELATIVE_TO_FIRST_PARENT_PARENTS,
    COMMIT_FILES_FILES,
    NO_COMMIT,
    AVAILABLE,
    UNAVAILABLE_LOCALLY,
    NOT_A_COMMIT_OBJECT,
    READ_FAILED,
    NOT_CHECKED,
    THE_COMMIT_OBJECT_IS_AVAILABLE_VIEW_IT_OR_CREATE_A,
    THERE_IS_NO_COMMIT_ON_THIS_SIDE_A_ZERO_OID,
    THE_COMMIT_OBJECT_IS_UNAVAILABLE_LOCALLY_IT_MAY_HAVE_BEEN,
    THIS_OID_DOES_NOT_POINT_TO_A_COMMIT_AND_CANNOT,
    LOCAL_COMMIT_AVAILABILITY_HAS_NOT_BEEN_CHECKED_WAIT_FOR_VERIFICATION,
    COMMIT_HISTORY,
    RECOVERY_RECORDS,
    VIEW_THE_CURRENT_WORKTREE_S_LOCAL_REFERENCE_LOGS,
    NO_REPOSITORY_OPEN,
    OPEN_A_LOCAL_REPOSITORY_TO_VIEW_RECOVERY_RECORDS_FOR_THE,
    OPEN_REPOSITORY,
    LOCATE_COMMITS_USING_LOCAL_REFERENCE_LOGS,
    RECORDS,
    DETAILS,
    VIEW_COMMITS_BEFORE_AND_AFTER_THE_SELECTED_OPERATION,
    SELECT_A_RECOVERY_RECORD_FIRST,
    RECOVERY_RECORDS_ARE_NOT_PERMANENT_BACKUPS_OBJECTS_MAY_BE_PRUNED,
    SPECIFIC_LOCAL_BRANCH,
    NO_LOCAL_BRANCHES_ARE_AVAILABLE_IN_THE_CURRENT_WORKTREE,
    SOURCE,
    CURRENT_BRANCH,
    ALL,
    HEAD_IS_DETACHED_THERE_IS_NO_CURRENT_BRANCH_LOG,
    HEAD_AND_ALL_LOCAL_BRANCH_LOGS,
    CURRENT_WORKTREE_HEAD_LOG,
    ALL_TIME,
    FILTER_BY_WHEN_THE_RECOVERY_OPERATION_OCCURRED,
    LAST_DAYS,
    LAST_DAYS_2,
    LAST_DAYS_3,
    SEARCH,
    SEARCH_LOADED_LOCAL_LOGS,
    READING,
    REFRESH,
    READING_LOCAL_RECOVERY_RECORDS,
    RELOAD_LOCAL_RECOVERY_RECORDS,
    CANCEL_READ,
    VIEW_RECOVERY_BRANCH,
    SOURCE_HEAD,
    SOURCE_CURRENT_BRANCH,
    SOURCE_ALL,
    ALL_LOCAL_SOURCES,
    CHOOSE_HEAD_THE_CURRENT_BRANCH_OR_ANOTHER_LOCAL_SOURCE,
    FILTER_RECOVERY_OPERATION_TIME,
    ACTIONS,
    SEARCH_REFRESH_AND_VIEW_CREATED_BRANCHES,
    FILTERING,
    LOADING,
    LOAD_MORE,
    LOAD_UP_TO_RECORDS_AT_A_TIME,
    READING_LOCAL_RECOVERY_RECORDS_2,
    FILTERING_RECOVERY_RECORDS,
    READ_INCOMPLETE_REFRESH_TO_TRY_AGAIN,
    NO_RECOVERY_RECORDS_MATCH_THE_CURRENT_SOURCE_AND_FILTERS,
    NO_OPERATION_DESCRIPTION,
    COMMIT_PREVIEW,
    RECOVERY_DETAILS,
    BACK_TO_DETAILS,
    SELECT_A_RECOVERY_RECORD,
    COMPARE_FULL_OIDS_BEFORE_AND_AFTER_THE_OPERATION_VERIFY_THE,
    READING_TARGET_COMMIT_PLEASE_WAIT,
    AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT,
    COPY_OID,
    VIEW_COMMIT,
    CREATE_BRANCH_FROM_THIS_COMMIT,
    CREATE_BRANCH,
    BEFORE_OPERATION_FULL_TARGET_OID,
    AFTER_OPERATION_FULL_TARGET_OID,
    READING_TARGET_COMMIT,
    AUTHOR,
    COMMIT_TIME,
    PARENT_COMMITS,
    COMMITTER,
    FULL_COMMIT_MESSAGE,
    THIS_IS_A_MERGE_COMMIT_FILES_AND_DIFFERENCES_ARE_SHOWN,
    LOG_EVIDENCE,
    OPERATION_DESCRIPTION,
    SOURCE_REFERENCE,
    OPERATION_TIME,
    ACTOR,
    FULL_OID_BEFORE_OPERATION,
    FULL_OID_AFTER_OPERATION,
    THE_ORIGINAL_LOG_CONTAINS_INVALID_UTF_AND_IS_DISPLAYED_WITH,
    CHOOSE_DIFF_DECODING_FOR_THE_CURRENT_REPOSITORY,
    DIFF,
    FULL_FILE,
    SHOW_DIFFERENCES_AND_SURROUNDING_CONTEXT,
    SHOW_FULL_FILE_CONTEXT,
    COMMIT_DIFF,
    LOADING_COMMIT_DIFFERENCES,
    SELECT_A_COMMIT_FILE_TO_VIEW_ITS_DIFFERENCES,
    LOADING_COMMIT_FILES,
    FAILED_TO_READ_FILES,
    THIS_COMMIT_HAS_NO_FILE_CHANGES,
    WORKING_TREE,
    FULL_TARGET_OID,
    SOURCE_RECORD,
    RECOVERY_TARGET,
    LOCAL_BRANCH_NAME,
    ONLY_CREATES_A_LOCAL_BRANCH_POINTING_TO_THIS_COMMIT_THE,
    CREATE_RECOVERY_BRANCH,
    CANCEL,
    CREATING,
    CREATE_LOCAL_BRANCH,
    FULL_FILE_TITLE,
    DIFF_TITLE,
    RECORD_TOOLTIP,
    RECORD_TOOLTIP_LOSSY,
    COPY_BEFORE_OID,
    COPY_AFTER_OID,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
