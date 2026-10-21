//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const CURRENT_BRANCH_BRANCH: UiMessage = UiMessage::new("views.dialog_view.current_branch_branch", "当前分支：{branch}", "Current branch: {branch}");
pub const TARGET_COMMIT_OID_SUMMARY: UiMessage = UiMessage::new("views.dialog_view.target_commit_oid_summary", "目标提交：{oid} {summary}", "Target commit: {oid} {summary}");
pub const TAG_TAG: UiMessage = UiMessage::new("views.dialog_view.tag_tag", "标签：{tag}", "Tag: {tag}");
pub const REMOTE_TAG_REMOTE_TAG: UiMessage = UiMessage::new("views.dialog_view.remote_tag_remote_tag", "远端标签：{remote}/{tag}", "Remote tag: {remote}/{tag}");
pub const SELECTED_FILES_COUNT: UiMessage = UiMessage::new("views.dialog_view.selected_files_count", "选定文件（{count} 个）", "Selected files ({count})");
pub const ALL_STAGED_FILES_COUNT: UiMessage = UiMessage::new("views.dialog_view.all_staged_files_count", "暂存区全部文件（{count} 个）", "All staged files ({count})");
pub const ALL_UNSTAGED_FILES_COUNT: UiMessage = UiMessage::new("views.dialog_view.all_unstaged_files_count", "修改区全部文件（{count} 个）", "All unstaged files ({count})");
pub const UNRESOLVED_COUNT_BLOCKS_ARE_STILL_UNRESOLVED_MARK_THIS_FILE_AS: UiMessage = UiMessage::new("views.dialog_view.unresolved_count_blocks_are_still_unresolved_mark_this_file_as", "还有 {unresolved_count} 个代码块未处理，是否继续标记已解决？", "{unresolved_count} blocks are still unresolved. Mark this file as resolved anyway?");
pub const NEW_VERSION_V_UPDATE_VERSION_AVAILABLE: UiMessage = UiMessage::new("views.dialog_view.new_version_v_update_version_available", "发现新版本 v{update_version}", "New version v{update_version} available");
pub const RELEASED_UPDATE_PUBLISHED_AT: UiMessage = UiMessage::new("views.dialog_view.released_update_published_at", "发布于 {update_published_at}", "Released {update_published_at}");
pub const PACKAGE_SIZE_UPDATE_SIZE: UiMessage = UiMessage::new("views.dialog_view.package_size_update_size", "包大小 {update_size}", "Package size {update_size}");
pub const CURRENT_CURRENT_DB_LABEL: UiMessage = UiMessage::new("views.dialog_view.current_current_db_label", "当前：{current_db_label}", "Current: {current_db_label}");
pub const CURRENT_RELOCATION_DB_LABEL: UiMessage = UiMessage::new("views.dialog_view.current_relocation_db_label", "当前：{relocation_db_label}", "Current: {relocation_db_label}");
pub const NEW_VERSION_V_VERSION_AVAILABLE: UiMessage = UiMessage::new("views.dialog_view.new_version_v_version_available", "发现新版本 v{version}", "New version v{version} available");
pub const RELEASED_PUBLISHED_AT: UiMessage = UiMessage::new("views.dialog_view.released_published_at", "发布于 {published_at}", "Released {published_at}");
pub const PACKAGE_SIZE_SIZE_MB_MB: UiMessage = UiMessage::new("views.dialog_view.package_size_size_mb_mb", "包大小：{size_mb} MB", "Package size: {size_mb} MB");
pub const VERSION_V_VERSION_HAS_BEEN_DOWNLOADED_AND_VERIFIED_THE_APPLICATION: UiMessage = UiMessage::new("views.dialog_view.version_v_version_has_been_downloaded_and_verified_the_application", "版本 v{version} 已下载并校验通过，应用将重启以完成安装。", "Version v{version} has been downloaded and verified. The application will restart to install it.");
pub const THE_CURRENT_DIRECTORY_IS_NOT_WRITABLE_SO_VERSION_V_VERSION: UiMessage = UiMessage::new("views.dialog_view.the_current_directory_is_not_writable_so_version_v_version", "当前目录没有写入权限，无法自动安装新版本（v{version}）。请手动下载新版本：", "The current directory is not writable, so version v{version} cannot be installed automatically. Download it manually:");
pub const AUTOMATIC_RECORD: UiMessage = UiMessage::new("views.dialog_view.automatic_record", "自动：{record}", "Automatic: {record}");
pub const DELETE_REMOTE_NAME: UiMessage = UiMessage::new("views.dialog_view.delete_remote_name", "确认删除远端：{name}", "Delete remote {name}?");
pub const DELETE_REMOTE_BRANCH_FULL_NAME: UiMessage = UiMessage::new("views.dialog_view.delete_remote_branch_full_name", "确认删除远端分支：{full_name}", "Delete remote branch {full_name}?");
pub const PROVIDER_LABEL_IS_OPEN_IN_YOUR_BROWSER_VERIFY_THE_CODE: UiMessage = UiMessage::new("views.dialog_view.provider_label_is_open_in_your_browser_verify_the_code", "已在浏览器打开{provider_label}，请确认验证码后完成登录：", "{provider_label} is open in your browser. Verify the code and complete sign-in:");
pub const COMPLETE_PROVIDER_LABEL_SIGN_IN_IN_YOUR_BROWSER: UiMessage = UiMessage::new("views.dialog_view.complete_provider_label_sign_in_in_your_browser", "请在浏览器中完成{provider_label}登录...", "Complete {provider_label} sign-in in your browser…");
pub const CREDENTIAL_RECORD_LABEL: UiMessage = UiMessage::new("views.dialog_view.credential_record_label", "凭据：{record_label}", "Credential: {record_label}");
pub const DELETE_CREDENTIAL_LABEL: UiMessage = UiMessage::new("views.dialog_view.delete_credential_label", "确认删除凭据：{label}", "Delete credential {label}?");
pub const CLONE_REPOSITORY: UiMessage = UiMessage::new("views.dialog_view.clone_repository", "克隆仓库", "Clone repository");
pub const CLONE_SUBMODULES_RECURSIVELY: UiMessage = UiMessage::new("views.dialog_view.clone_submodules_recursively", "递归克隆子模块", "Clone submodules recursively");
pub const ENTER_A_REMOTE_REPOSITORY_URL_AND_PARENT_FOLDER_TO_SEE: UiMessage = UiMessage::new("views.dialog_view.enter_a_remote_repository_url_and_parent_folder_to_see", "填写远程仓库 URL 和父文件夹后显示最终代码路径", "Enter a remote repository URL and parent folder to see the final checkout path");
pub const SELECT_FOLDER: UiMessage = UiMessage::new("views.dialog_view.select_folder", "选择目录", "Select folder");
pub const CANCEL: UiMessage = UiMessage::new("views.dialog_view.cancel", "取消", "Cancel");
pub const CLONE: UiMessage = UiMessage::new("views.dialog_view.clone", "克隆", "Clone");
pub const CLOSE_KHASLANA: UiMessage = UiMessage::new("views.dialog_view.close_khaslana", "关闭 Khaslana", "Close Khaslana");
pub const EXIT_THE_APPLICATION_OR_KEEP_KHASLANA_RUNNING_IN_THE_SYSTEM: UiMessage = UiMessage::new("views.dialog_view.exit_the_application_or_keep_khaslana_running_in_the_system", "要直接退出应用，还是让 Khaslana 继续在系统托盘中运行？", "Exit the application, or keep Khaslana running in the system tray?");
pub const WHEN_MINIMIZED_TO_THE_TRAY_CLICK_THE_TRAY_ICON_TO: UiMessage = UiMessage::new("views.dialog_view.when_minimized_to_the_tray_click_the_tray_icon_to", "缩小到托盘后，可点击托盘图标恢复主窗口，或从托盘菜单退出。", "When minimized to the tray, click the tray icon to restore the main window, or exit from the tray menu.");
pub const MINIMIZE_TO_TRAY: UiMessage = UiMessage::new("views.dialog_view.minimize_to_tray", "缩小到托盘", "Minimize to tray");
pub const EXIT: UiMessage = UiMessage::new("views.dialog_view.exit", "直接退出", "Exit");
pub const MIGRATE_TO_PORTABLE_DIRECTORY: UiMessage = UiMessage::new("views.dialog_view.migrate_to_portable_directory", "迁移到便携目录", "Migrate to portable directory");
pub const APPLICATION_DATA_IS_CURRENTLY_IN_A_SYSTEM_DIRECTORY_ON_DRIVE: UiMessage = UiMessage::new("views.dialog_view.application_data_is_currently_in_a_system_directory_on_drive", "检测到应用数据当前存放在 C 盘系统目录，是否迁移到程序所在目录？", "Application data is currently in a system directory on drive C. Migrate it to the application directory?");
pub const KEEP_CURRENT_LOCATION: UiMessage = UiMessage::new("views.dialog_view.keep_current_location", "保持现状", "Keep current location");
pub const MIGRATE_AND_RESTART: UiMessage = UiMessage::new("views.dialog_view.migrate_and_restart", "迁移并重启", "Migrate and restart");
pub const UNKNOWN: UiMessage = UiMessage::new("views.dialog_view.unknown", "未知", "Unknown");
pub const MOVE_TO_A_SAFE_DIRECTORY: UiMessage = UiMessage::new("views.dialog_view.move_to_a_safe_directory", "移动到安全目录", "Move to a safe directory");
pub const MOVE_AND_RESTART: UiMessage = UiMessage::new("views.dialog_view.move_and_restart", "移动并重启", "Move and restart");
pub const CREATE_BRANCH: UiMessage = UiMessage::new("views.dialog_view.create_branch", "新建分支", "Create branch");
pub const BASE_BRANCH: UiMessage = UiMessage::new("views.dialog_view.base_branch", "基于分支", "Base branch");
pub const CURRENT_HEAD: UiMessage = UiMessage::new("views.dialog_view.current_head", "当前 HEAD", "Current HEAD");
pub const SELECT_BASE_BRANCH: UiMessage = UiMessage::new("views.dialog_view.select_base_branch", "选择基础分支", "Select base branch");
pub const SWITCH_TO_THE_NEW_BRANCH_AFTER_CREATION: UiMessage = UiMessage::new("views.dialog_view.switch_to_the_new_branch_after_creation", "创建成功后切换到新分支", "Switch to the new branch after creation");
pub const CREATE: UiMessage = UiMessage::new("views.dialog_view.create", "创建", "Create");
pub const RENAME_BRANCH: UiMessage = UiMessage::new("views.dialog_view.rename_branch", "重命名分支", "Rename branch");
pub const RENAME: UiMessage = UiMessage::new("views.dialog_view.rename", "重命名", "Rename");
pub const CONFIRM_BRANCH_RESET: UiMessage = UiMessage::new("views.dialog_view.confirm_branch_reset", "确认重置分支", "Confirm branch reset");
pub const A_HARD_RESET_MOVES_THE_CURRENT_BRANCH_AND_REMOVES_LATER: UiMessage = UiMessage::new("views.dialog_view.a_hard_reset_moves_the_current_branch_and_removes_later", "强制重置会移动当前分支，目标提交之后的已提交代码会从分支历史中移除。确认前请确保目标提交正确。", "A hard reset moves the current branch and removes later commits from its history. Verify the target commit before confirming.");
pub const CONFIRM_RESET: UiMessage = UiMessage::new("views.dialog_view.confirm_reset", "确认重置", "Confirm reset");
pub const CONFIRM_REVERT_COMMIT: UiMessage = UiMessage::new("views.dialog_view.confirm_revert_commit", "确认回滚提交", "Confirm revert commit");
pub const THIS_CREATES_A_NEW_COMMIT_THAT_REVERTS_THE_CHANGES_INTRODUCED: UiMessage = UiMessage::new("views.dialog_view.this_creates_a_new_commit_that_reverts_the_changes_introduced", "确认后会创建一个新的提交，用于撤销该提交引入的修改。", "This creates a new commit that reverts the changes introduced by this commit.");
pub const CONFIRM_REVERT: UiMessage = UiMessage::new("views.dialog_view.confirm_revert", "确认回滚", "Confirm revert");
pub const CONFIRM_REVERT_MERGE_COMMIT: UiMessage = UiMessage::new("views.dialog_view.confirm_revert_merge_commit", "确认撤销合并提交", "Confirm revert merge commit");
pub const THIS_CREATES_A_NEW_COMMIT_THAT_REVERTS_THE_CHANGES_INTRODUCED_2: UiMessage = UiMessage::new("views.dialog_view.this_creates_a_new_commit_that_reverts_the_changes_introduced_2", "确认后会创建一个新的提交，用于撤销这次合并相对主线引入的修改。", "This creates a new commit that reverts the changes introduced by this merge relative to the mainline.");
pub const THE_ORIGINAL_MERGE_COMMIT_IS_RETAINED_AND_HISTORY_IS_NOT: UiMessage = UiMessage::new("views.dialog_view.the_original_merge_commit_is_retained_and_history_is_not", "该操作不会删除原合并提交，也不会重写分支历史；若产生冲突，请在冲突解决中心处理后手动提交。", "The original merge commit is retained and history is not rewritten. Resolve any conflicts in the conflict workbench, then commit manually.");
pub const THIS_IS_EQUIVALENT_TO_GIT_REVERT_M_PRESERVING_THE_FIRST: UiMessage = UiMessage::new("views.dialog_view.this_is_equivalent_to_git_revert_m_preserving_the_first", "这等价于 git revert -m 1，表示保留合并提交的第一父提交一侧。后续再次合并同一分支时，Git 会认为这次合并的改动曾被主动撤销。", "This is equivalent to git revert -m 1, preserving the first-parent side of the merge. Git will remember these changes as explicitly reverted when the same branch is merged again.");
pub const CONFIRM_REVERT_MERGE: UiMessage = UiMessage::new("views.dialog_view.confirm_revert_merge", "确认撤销合并", "Confirm revert merge");
pub const CONFIRM_UNDO_TO_STAGING: UiMessage = UiMessage::new("views.dialog_view.confirm_undo_to_staging", "确认还原到暂存区", "Confirm undo to staging");
pub const THIS_REMOVES_THE_COMMIT_AND_KEEPS_ITS_CHANGES_IN_THE: UiMessage = UiMessage::new("views.dialog_view.this_removes_the_commit_and_keeps_its_changes_in_the", "确认后会撤销该提交记录，并把该提交引入的修改保留在暂存区。", "This removes the commit and keeps its changes in the staging area.");
pub const ONLY_THE_LATEST_UNPUSHED_NON_MERGE_COMMIT_ON_THE_CURRENT: UiMessage = UiMessage::new("views.dialog_view.only_the_latest_unpushed_non_merge_commit_on_the_current", "该操作只支持当前分支最新且尚未推送的普通提交。", "Only the latest unpushed, non-merge commit on the current branch is supported.");
pub const CONFIRM_UNDO: UiMessage = UiMessage::new("views.dialog_view.confirm_undo", "确认还原", "Confirm undo");
pub const AMEND_A_PUSHED_COMMIT: UiMessage = UiMessage::new("views.dialog_view.amend_a_pushed_commit", "修补已推送的提交", "Amend a pushed commit");
pub const THE_LATEST_COMMIT_HAS_ALREADY_BEEN_PUSHED_TO_THE_REMOTE: UiMessage = UiMessage::new("views.dialog_view.the_latest_commit_has_already_been_pushed_to_the_remote", "当前最新提交已推送到远端。", "The latest commit has already been pushed to the remote.");
pub const AMENDING_REWRITES_THIS_COMMIT_UPDATING_THE_REMOTE_THEN_REQUIRES_A: UiMessage = UiMessage::new("views.dialog_view.amending_rewrites_this_commit_updating_the_remote_then_requires_a", "修补会重写这条提交，之后必须用强制推送才能覆盖远端历史；当前版本暂不支持强推，其他协作者的本地历史会与远端分叉。", "Amending rewrites this commit. Updating the remote then requires a force push, which this version does not support. Other collaborators’ local histories will diverge from the remote.");
pub const AMEND_ONLY_COMMITS_THAT_HAVE_NOT_BEEN_PUSHED: UiMessage = UiMessage::new("views.dialog_view.amend_only_commits_that_have_not_been_pushed", "建议仅对尚未推送的提交使用修补。", "Amend only commits that have not been pushed.");
pub const AMEND_AND_PUSH_ANYWAY: UiMessage = UiMessage::new("views.dialog_view.amend_and_push_anyway", "仍要修补并推送", "Amend and push anyway");
pub const AMEND_ANYWAY: UiMessage = UiMessage::new("views.dialog_view.amend_anyway", "仍要修补", "Amend anyway");
pub const THIS_REPOSITORY_HAS_NO_REMOTE: UiMessage = UiMessage::new("views.dialog_view.this_repository_has_no_remote", "当前仓库没有远端", "This repository has no remote");
pub const TARGET_COMMIT_HEAD_LATEST_COMMIT_ON_THE_CURRENT_BRANCH: UiMessage = UiMessage::new("views.dialog_view.target_commit_head_latest_commit_on_the_current_branch", "目标提交：HEAD（当前分支最新提交）", "Target commit: HEAD (latest commit on the current branch)");
pub const CREATE_TAG: UiMessage = UiMessage::new("views.dialog_view.create_tag", "创建标签", "Create tag");
pub const CREATE_ANNOTATED_TAG_RECORDS_MESSAGE_AND_CREATOR_RECOMMENDED_FOR_RELEASES: UiMessage = UiMessage::new("views.dialog_view.create_annotated_tag_records_message_and_creator_recommended_for_releases", "创建附注标签（记录标签信息与创建者，发布推荐）", "Create annotated tag (records message and creator; recommended for releases)");
pub const SELECT_REMOTE: UiMessage = UiMessage::new("views.dialog_view.select_remote", "选择远端", "Select remote");
pub const PUSH_TAG: UiMessage = UiMessage::new("views.dialog_view.push_tag", "推送标签", "Push tag");
pub const PUSH: UiMessage = UiMessage::new("views.dialog_view.push", "推送", "Push");
pub const DELETE_TAG: UiMessage = UiMessage::new("views.dialog_view.delete_tag", "删除标签", "Delete tag");
pub const THIS_DELETES_THE_LOCAL_TAG_ONLY_REMOTE_TAGS_ARE_NOT: UiMessage = UiMessage::new("views.dialog_view.this_deletes_the_local_tag_only_remote_tags_are_not", "确认后删除本地标签，不影响远端标签。", "This deletes the local tag only. Remote tags are not affected.");
pub const CONFIRM_DELETION: UiMessage = UiMessage::new("views.dialog_view.confirm_deletion", "确认删除", "Confirm deletion");
pub const DELETE_REMOTE_TAG: UiMessage = UiMessage::new("views.dialog_view.delete_remote_tag", "删除远端标签", "Delete remote tag");
pub const THIS_DELETES_THE_TAG_FROM_THE_REMOTE_PUBLISHED_VERSION_REFERENCES: UiMessage = UiMessage::new("views.dialog_view.this_deletes_the_tag_from_the_remote_published_version_references", "确认后从远端删除该标签，已发布的版本引用将不可再用，删除后无法恢复。", "This deletes the tag from the remote. Published version references will be unavailable. This cannot be undone.");
pub const TARGET_FILE: UiMessage = UiMessage::new("views.dialog_view.target_file", "目标文件", "Target file");
pub const DISCARD_ALL_UNCOMMITTED_CHANGES_IN_THESE_FILES_BOTH_STAGED_AND: UiMessage = UiMessage::new("views.dialog_view.discard_all_uncommitted_changes_in_these_files_both_staged_and", "将丢弃这些文件全部未提交更改，包括暂存区和工作区。新增文件会被删除，删除文件会被恢复。此操作无法从 Khaslana 内撤销。", "Discard all uncommitted changes in these files, both staged and unstaged. Added files will be deleted and deleted files restored. This cannot be undone in Khaslana.");
pub const DISCARD_ONLY_UNSTAGED_CHANGES_IN_THESE_FILES_STAGED_CHANGES_ARE: UiMessage = UiMessage::new("views.dialog_view.discard_only_unstaged_changes_in_these_files_staged_changes_are", "将仅丢弃这些文件尚未暂存的更改，已暂存内容会保留。未跟踪新增文件会被删除。此操作无法从 Khaslana 内撤销。", "Discard only unstaged changes in these files; staged changes are kept. New untracked files will be deleted. This cannot be undone in Khaslana.");
pub const CONFIRM_DISCARD_CHANGES: UiMessage = UiMessage::new("views.dialog_view.confirm_discard_changes", "确认回滚更改", "Confirm discard changes");
pub const OVERWRITE_EXISTING_CONFLICT_RESOLUTION: UiMessage = UiMessage::new("views.dialog_view.overwrite_existing_conflict_resolution", "覆盖现有冲突处理？", "Overwrite existing conflict resolution?");
pub const THIS_DRAFT_ALREADY_CONTAINS_BLOCK_RESOLUTIONS_OR_MANUAL_EDITS_GENERATING: UiMessage = UiMessage::new("views.dialog_view.this_draft_already_contains_block_resolutions_or_manual_edits_generating", "该文件的草稿已有块处理或手工修改，生成 AI 合并建议会覆盖这些内容。", "This draft already contains block resolutions or manual edits. Generating AI merge suggestions will overwrite them.");
pub const ACCEPTED_BLOCKS_IGNORED_BLOCKS_AND_MANUAL_EDITS_WILL_BE_LOST: UiMessage = UiMessage::new("views.dialog_view.accepted_blocks_ignored_blocks_and_manual_edits_will_be_lost", "覆盖后已接受的块、忽略操作和手工编辑都会丢失，需要重新处理。", "Accepted blocks, ignored blocks, and manual edits will be lost and must be redone.");
pub const GO_BACK_AND_KEEP_CHANGES: UiMessage = UiMessage::new("views.dialog_view.go_back_and_keep_changes", "返回保留现状", "Go back and keep changes");
pub const OVERWRITE_AND_GENERATE: UiMessage = UiMessage::new("views.dialog_view.overwrite_and_generate", "覆盖并生成", "Overwrite and generate");
pub const CURRENT_CONFLICT_FILE: UiMessage = UiMessage::new("views.dialog_view.current_conflict_file", "当前冲突文件", "Current conflict file");
pub const UNRESOLVED_BLOCKS_REMAIN: UiMessage = UiMessage::new("views.dialog_view.unresolved_blocks_remain", "仍有未处理代码块", "Unresolved blocks remain");
pub const THE_CURRENT_RESULT_WILL_BE_WRITTEN_TO_THE_WORKING_TREE: UiMessage = UiMessage::new("views.dialog_view.the_current_result_will_be_written_to_the_working_tree", "继续后会直接把当前结果写入工作区并从索引中移除冲突标记。", "The current result will be written to the working tree and conflict markers removed from the index.");
pub const GO_BACK_TO_RESOLVING: UiMessage = UiMessage::new("views.dialog_view.go_back_to_resolving", "返回继续处理", "Go back to resolving");
pub const RESOLVE_ANYWAY: UiMessage = UiMessage::new("views.dialog_view.resolve_anyway", "继续解决", "Resolve anyway");
pub const NO_RELEASE_NOTES_FOR_THIS_VERSION: UiMessage = UiMessage::new("views.dialog_view.no_release_notes_for_this_version", "（此版本未附版本说明）", "(No release notes for this version)");
pub const NONE: UiMessage = UiMessage::new("views.dialog_view.none", "无", "None");
pub const UPDATES: UiMessage = UiMessage::new("views.dialog_view.updates", "更新", "Updates");
pub const AUTOMATIC_CHECKS_AT_STARTUP_AND_MANUAL_CHECKS_SKIPPED_VERSIONS_ONLY: UiMessage = UiMessage::new("views.dialog_view.automatic_checks_at_startup_and_manual_checks_skipped_versions_only", "启动时自动检查与手动检查；跳过版本只影响自动提示。", "Automatic checks at startup and manual checks; skipped versions only suppress automatic prompts.");
pub const CURRENT_VERSION: UiMessage = UiMessage::new("views.dialog_view.current_version", "当前版本", "Current version");
pub const CHECK_FOR_UPDATES_AUTOMATICALLY: UiMessage = UiMessage::new("views.dialog_view.check_for_updates_automatically", "自动检查更新", "Check for updates automatically");
pub const RECEIVE_BETA_UPDATES: UiMessage = UiMessage::new("views.dialog_view.receive_beta_updates", "接收测试版（Beta）更新", "Receive Beta updates");
pub const ALSO_CHECK_FOR_AND_INSTALL_BETA_RELEASES_THEY_MAY_BE: UiMessage = UiMessage::new("views.dialog_view.also_check_for_and_install_beta_releases_they_may_be", "开启后同时检测并安装测试版；测试版可能不稳定", "Also check for and install Beta releases; they may be unstable");
pub const SKIPPED_VERSION: UiMessage = UiMessage::new("views.dialog_view.skipped_version", "已跳过版本", "Skipped version");
pub const CHECK_NOW: UiMessage = UiMessage::new("views.dialog_view.check_now", "立即检查", "Check now");
pub const CLEAR_SKIPPED_VERSION: UiMessage = UiMessage::new("views.dialog_view.clear_skipped_version", "清除跳过", "Clear skipped version");
pub const CHANNELS_AND_MIGRATION: UiMessage = UiMessage::new("views.dialog_view.channels_and_migration", "渠道与迁移", "Channels and migration");
pub const PENDING_UPDATES_DATA_DIRECTORY_MIGRATION_AND_APPLICATION_RELOCATION: UiMessage = UiMessage::new("views.dialog_view.pending_updates_data_directory_migration_and_application_relocation", "待安装的新版本，以及数据目录与程序位置的迁移入口。", "Pending updates, data directory migration, and application relocation.");
pub const UPDATE_NOW: UiMessage = UiMessage::new("views.dialog_view.update_now", "立即更新", "Update now");
pub const PREPARING_DOWNLOAD: UiMessage = UiMessage::new("views.dialog_view.preparing_download", "准备下载...", "Preparing download…");
pub const DATA_DIRECTORY: UiMessage = UiMessage::new("views.dialog_view.data_directory", "数据目录", "Data directory");
pub const MOVE_DATA_FROM_THE_SYSTEM_DIRECTORY_ON_DRIVE_C_TO: UiMessage = UiMessage::new("views.dialog_view.move_data_from_the_system_directory_on_drive_c_to", "可将数据从 C 盘系统目录迁移到程序所在目录，便于整体备份并减少 C 盘占用。", "Move data from the system directory on drive C to the application directory for easier backups and less drive C usage.");
pub const APPLICATION_LOCATION: UiMessage = UiMessage::new("views.dialog_view.application_location", "程序位置", "Application location");
pub const SKIP_THIS_VERSION: UiMessage = UiMessage::new("views.dialog_view.skip_this_version", "跳过此版本", "Skip this version");
pub const LATER: UiMessage = UiMessage::new("views.dialog_view.later", "稍后", "Later");
pub const UPDATE_READY: UiMessage = UiMessage::new("views.dialog_view.update_ready", "更新准备就绪", "Update ready");
pub const THE_APPLICATION_EXITS_AND_RESTARTS_DURING_INSTALLATION_SAVE_YOUR_WORK: UiMessage = UiMessage::new("views.dialog_view.the_application_exits_and_restarts_during_installation_save_your_work", "安装过程中应用会自动退出并重启，请确保没有未保存的工作。", "The application exits and restarts during installation. Save your work first.");
pub const RESTART_NOW: UiMessage = UiMessage::new("views.dialog_view.restart_now", "立即重启", "Restart now");
pub const THE_STAGING_DIRECTORY_IS_MISSING: UiMessage = UiMessage::new("views.dialog_view.the_staging_directory_is_missing", "staging 目录丢失", "The staging directory is missing");
pub const CANNOT_UPDATE_AUTOMATICALLY: UiMessage = UiMessage::new("views.dialog_view.cannot_update_automatically", "无法自动更新", "Cannot update automatically");
pub const OPEN_CNB_DOWNLOADS: UiMessage = UiMessage::new("views.dialog_view.open_cnb_downloads", "打开 CNB 下载页", "Open CNB downloads");
pub const OPEN_GITHUB_RELEASE: UiMessage = UiMessage::new("views.dialog_view.open_github_release", "打开 GitHub Release", "Open GitHub Release");
pub const CLOSE: UiMessage = UiMessage::new("views.dialog_view.close", "关闭", "Close");
pub const NO_REMOTES_YET_CLICK_ADD_REMOTE_TO_CREATE_ONE: UiMessage = UiMessage::new("views.dialog_view.no_remotes_yet_click_add_remote_to_create_one", "暂无远端。可以点击“新增远端”添加。", "No remotes yet. Click “Add remote” to create one.");
pub const MANAGE_REMOTES: UiMessage = UiMessage::new("views.dialog_view.manage_remotes", "远端管理", "Manage remotes");
pub const ADD_REMOTE: UiMessage = UiMessage::new("views.dialog_view.add_remote", "新增远端", "Add remote");
pub const THE_REMOTE_ADDRESS_IS_USED_FOR_BOTH_FETCH_AND_PUSH: UiMessage = UiMessage::new("views.dialog_view.the_remote_address_is_used_for_both_fetch_and_push", "远端地址会同时作为 fetch 和 push URL；凭据只从已保存凭据中选择。", "The remote address is used for both fetch and push. Credentials can only be selected from saved records.");
pub const NAME: UiMessage = UiMessage::new("views.dialog_view.name", "名称", "Name");
pub const ADDRESS: UiMessage = UiMessage::new("views.dialog_view.address", "地址", "Address");
pub const CREDENTIALS: UiMessage = UiMessage::new("views.dialog_view.credentials", "凭据", "Credentials");
pub const ACTIONS: UiMessage = UiMessage::new("views.dialog_view.actions", "操作", "Actions");
pub const NO_CREDENTIALS: UiMessage = UiMessage::new("views.dialog_view.no_credentials", "无凭据", "No credentials");
pub const CREDENTIAL_RECORD_MISSING: UiMessage = UiMessage::new("views.dialog_view.credential_record_missing", "凭据不存在", "Credential record missing");
pub const AUTOMATIC_MATCH: UiMessage = UiMessage::new("views.dialog_view.automatic_match", "自动匹配", "Automatic match");
pub const EDIT: UiMessage = UiMessage::new("views.dialog_view.edit", "编辑", "Edit");
pub const DELETE: UiMessage = UiMessage::new("views.dialog_view.delete", "删除", "Delete");
pub const EDIT_REMOTE: UiMessage = UiMessage::new("views.dialog_view.edit_remote", "编辑远端", "Edit remote");
pub const BIND_CREDENTIALS: UiMessage = UiMessage::new("views.dialog_view.bind_credentials", "绑定凭据", "Bind credentials");
pub const SAVE: UiMessage = UiMessage::new("views.dialog_view.save", "保存", "Save");
pub const AUTOMATICALLY_MATCH_SAVED_CREDENTIALS: UiMessage = UiMessage::new("views.dialog_view.automatically_match_saved_credentials", "自动匹配保存凭据", "Automatically match saved credentials");
pub const NOT_A_MATCH: UiMessage = UiMessage::new("views.dialog_view.not_a_match", "（不匹配）", "(Not a match)");
pub const DELETE_REMOTE: UiMessage = UiMessage::new("views.dialog_view.delete_remote", "删除远端", "Delete remote");
pub const THIS_ONLY_REMOVES_THE_REMOTE_CONFIGURATION_FROM_THIS_REPOSITORY_SAVED: UiMessage = UiMessage::new("views.dialog_view.this_only_removes_the_remote_configuration_from_this_repository_saved", "这只会删除当前仓库的远端配置，不会删除任何已保存凭据。", "This only removes the remote configuration from this repository. Saved credentials are not deleted.");
pub const DELETE_REMOTE_BRANCH: UiMessage = UiMessage::new("views.dialog_view.delete_remote_branch", "删除远端分支", "Delete remote branch");
pub const THIS_DELETES_THE_BRANCH_ON_THE_REMOTE_AND_REFRESHES_LOCAL: UiMessage = UiMessage::new("views.dialog_view.this_deletes_the_branch_on_the_remote_and_refreshes_local", "这会删除远端仓库上的分支，并刷新本地远端分支列表；不会删除同名本地分支。", "This deletes the branch on the remote and refreshes local remote-tracking branches. Local branches with the same name are not deleted.");
pub const QUICK_SIGN_IN: UiMessage = UiMessage::new("views.dialog_view.quick_sign_in", "快速登录", "Quick sign-in");
pub const GITEE_SIGN_IN_REQUIRES_A_MAINTAINER_DEPLOYED_TOKEN_EXCHANGE_SERVICE: UiMessage = UiMessage::new("views.dialog_view.gitee_sign_in_requires_a_maintainer_deployed_token_exchange_service", "Gitee 登录需由维护者部署令牌交换服务（见 AGENTS.md）；GitHub 可直接使用。", "Gitee sign-in requires a maintainer-deployed token exchange service (see AGENTS.md). GitHub works directly.");
pub const UNKNOWN_RECORD: UiMessage = UiMessage::new("views.dialog_view.unknown_record", "未知记录", "Unknown record");
pub const TEST_CREDENTIAL_CONNECTION: UiMessage = UiMessage::new("views.dialog_view.test_credential_connection", "测试凭据连接", "Test credential connection");
pub const THIS_ADDRESS_IS_USED_FOR_A_GIT_CONNECTION_TO_VERIFY: UiMessage = UiMessage::new("views.dialog_view.this_address_is_used_for_a_git_connection_to_verify", "将使用此地址发起一次 Git 连接验证凭据。建议填写真实仓库地址；                         裸站点地址（如 https://gitee.com）可能因服务器不发起认证而无法验证凭据。", "This address is used for a Git connection to verify the credentials. Use a real repository URL; a bare host URL such as https://gitee.com may not request authentication and cannot verify credentials.");
pub const START_TEST: UiMessage = UiMessage::new("views.dialog_view.start_test", "开始测试", "Start test");
pub const ADD_CREDENTIALS: UiMessage = UiMessage::new("views.dialog_view.add_credentials", "添加凭据", "Add credentials");
pub const TYPE: UiMessage = UiMessage::new("views.dialog_view.type", "类型", "Type");
pub const PREFER_SSH_AGENT_WHEN_USING_A_PRIVATE_KEY_FILE_ONLY: UiMessage = UiMessage::new("views.dialog_view.prefer_ssh_agent_when_using_a_private_key_file_only", "推荐优先使用 SSH Agent；使用私钥文件时，应用只保存路径，密码短语仍存入系统 Keyring。", "Prefer SSH Agent. When using a private key file, only its path is saved; the passphrase stays in the system Keyring.");
pub const USE_SSH_AGENT_DO_NOT_SAVE_A_PRIVATE_KEY_PATH: UiMessage = UiMessage::new("views.dialog_view.use_ssh_agent_do_not_save_a_private_key_path", "使用 SSH Agent（不保存私钥路径）", "Use SSH Agent (do not save a private key path)");
pub const SELECT_PRIVATE_KEY_FILE: UiMessage = UiMessage::new("views.dialog_view.select_private_key_file", "选择私钥文件", "Select private key file");
pub const REUSE_SCOPE: UiMessage = UiMessage::new("views.dialog_view.reuse_scope", "复用范围", "Reuse scope");
pub const THIS_REMOTE_ONLY: UiMessage = UiMessage::new("views.dialog_view.this_remote_only", "仅此远端", "This remote only");
pub const SAME_HOST: UiMessage = UiMessage::new("views.dialog_view.same_host", "同站点", "Same host");
pub const SECRETS_ARE_STORED_ONLY_IN_THE_SYSTEM_CREDENTIAL_MANAGER_PASSWORDS: UiMessage = UiMessage::new("views.dialog_view.secrets_are_stored_only_in_the_system_credential_manager_passwords", "密文仅保存在系统凭据管理器；这里不显示、不复制密码、PAT 或 SSH 密码短语。", "Secrets are stored only in the system credential manager. Passwords, PATs, and SSH passphrases are never displayed or copied here.");
pub const REFRESH: UiMessage = UiMessage::new("views.dialog_view.refresh", "刷新", "Refresh");
pub const CREDENTIAL_LIST_REFRESHED: UiMessage = UiMessage::new("views.dialog_view.credential_list_refreshed", "凭据列表已刷新", "Credential list refreshed");
pub const NO_SAVED_CREDENTIALS_SAVE_CREDENTIALS_DURING_A_REMOTE_OPERATION_TO: UiMessage = UiMessage::new("views.dialog_view.no_saved_credentials_save_credentials_during_a_remote_operation_to", "暂无已保存凭据。远程操作时勾选保存后会出现在这里。", "No saved credentials. Save credentials during a remote operation to list them here.");
pub const CREDENTIAL_DETAILS: UiMessage = UiMessage::new("views.dialog_view.credential_details", "凭据详情", "Credential details");
pub const THIS_CREDENTIAL_RECORD_DOES_NOT_EXIST_IT_MAY_HAVE_BEEN: UiMessage = UiMessage::new("views.dialog_view.this_credential_record_does_not_exist_it_may_have_been", "凭据记录不存在，可能已经被删除。", "This credential record does not exist. It may have been deleted.");
pub const HOST_REMOTE: UiMessage = UiMessage::new("views.dialog_view.host_remote", "站点 / 远端", "Host / remote");
pub const USERNAME: UiMessage = UiMessage::new("views.dialog_view.username", "用户名", "Username");
pub const SSH_KEY_PATH: UiMessage = UiMessage::new("views.dialog_view.ssh_key_path", "SSH Key 路径", "SSH key path");
pub const CREATED: UiMessage = UiMessage::new("views.dialog_view.created", "创建时间", "Created");
pub const UPDATED: UiMessage = UiMessage::new("views.dialog_view.updated", "更新时间", "Updated");
pub const LAST_USED: UiMessage = UiMessage::new("views.dialog_view.last_used", "最后使用时间", "Last used");
pub const PASSWORDS_PATS_AND_SSH_PASSPHRASES_ARE_NEVER_DISPLAYED_HERE: UiMessage = UiMessage::new("views.dialog_view.passwords_pats_and_ssh_passphrases_are_never_displayed_here", "密码、PAT 和 SSH 密码短语不会在这里显示。", "Passwords, PATs, and SSH passphrases are never displayed here.");
pub const SCOPE: UiMessage = UiMessage::new("views.dialog_view.scope", "范围", "Scope");
pub const TEST: UiMessage = UiMessage::new("views.dialog_view.test", "测试", "Test");
pub const DELETE_CREDENTIALS: UiMessage = UiMessage::new("views.dialog_view.delete_credentials", "删除凭据", "Delete credentials");
pub const THIS_REMOVES_BOTH_THE_NON_SENSITIVE_INDEX_RECORD_AND_THE: UiMessage = UiMessage::new("views.dialog_view.this_removes_both_the_non_sensitive_index_record_and_the", "删除会同时移除非敏感索引和系统凭据管理器中的密文。", "This removes both the non-sensitive index record and the secret from the system credential manager.");

pub const RELOCATION_DETAILS: UiMessage = UiMessage::new("views.dialog_view.relocation_details", "程序当前位于可能被清理的目录（临时/聊天软件接收/下载目录），建议把程序与数据移动到独立的安全目录。", "The application is in a folder that may be cleaned (temporary, chat downloads, or Downloads). Move the program and data to a dedicated safe folder.");

pub const DOWNLOAD_LOCATION_WARNING: UiMessage = UiMessage::new("views.dialog_view.download_location_warning", "检测到程序当前位于下载文件夹。下载文件夹可能被系统「存储感知」或清理工具定期清空，届时程序与其中的数据都会丢失。", "The application is in Downloads. Storage Sense or cleanup tools may empty this folder and delete the program and its data.");

pub const VOLATILE_LOCATION_WARNING: UiMessage = UiMessage::new("views.dialog_view.volatile_location_warning", "检测到程序当前位于可能被自动清理的目录（临时目录或聊天软件的接收文件目录）。这类目录会被系统或聊天软件的清理功能定期清空，届时程序与其中的数据都会丢失。", "The application is in a temporary or chat download folder that may be automatically cleaned. Cleanup may delete the program and its data.");

pub const PORTABLE_MIGRATION_DETAILS: UiMessage = UiMessage::new("views.dialog_view.portable_migration_details", "迁移后，数据库、更新缓存和工作流模板将统一保存在可执行文件同级的 data/ 目录，便于整体备份并减少 C 盘占用。点击「迁移并重启」后应用将关闭，并在下次启动时自动完成数据搬运。若选择「保持现状」，后续可在「设置」-「更新设置」中手动执行迁移。", "After migration, the database, update cache, and workflow templates will be stored in data/ beside the executable for easier backups and less drive C usage. “Migrate and restart” closes the application; data is moved automatically on next startup. You can migrate later under Settings → Updates.");

pub const MOVE_PROGRAM_AND_DATA: UiMessage = UiMessage::new("views.dialog_view.move_program_and_data", "数据目前也存放在该目录中，强烈建议立即移动。点击「移动并重启」后应用将关闭，程序与数据会被搬到 {target_label} 并从新位置重新启动。若选择「保持现状」，之后可在「设置」-「更新设置」中手动执行移动。", "Your data is also stored here; moving immediately is strongly recommended. “Move and restart” closes the application, moves the program and data to {target_label}, and restarts it. You can also move later under Settings → Updates.");

pub const MOVE_PROGRAM_ONLY: UiMessage = UiMessage::new("views.dialog_view.move_program_only", "你的数据已保存在安全位置，仅程序本体存在丢失风险。点击「移动并重启」后应用将关闭，程序会被搬到 {target_label} 并从新位置重新启动。若选择「保持现状」，之后可在「设置」-「更新设置」中手动执行移动。", "Your data is stored safely elsewhere; only the program is at risk. “Move and restart” closes the application, moves the program to {target_label}, and restarts it. You can also move later under Settings → Updates.");

pub const RESET_SOFT_DESCRIPTION: UiMessage = UiMessage::new("views.dialog_view.reset_soft_description", "将当前分支重置到该提交。软重置：保留暂存区和工作区修改", "Reset the current branch to this commit. Soft reset keeps staged and working tree changes.");

pub const RESET_MIXED_DESCRIPTION: UiMessage = UiMessage::new("views.dialog_view.reset_mixed_description", "将当前分支重置到该提交。混合重置：重置暂存区，保留工作区修改", "Reset the current branch to this commit. Mixed reset clears the staging area and keeps working tree changes.");

pub const RESET_HARD_DESCRIPTION: UiMessage = UiMessage::new("views.dialog_view.reset_hard_description", "将当前分支重置到该提交。强制重置：重置暂存区和工作区，丢弃未提交修改", "Reset the current branch to this commit. Hard reset resets staging and the working tree, discarding uncommitted changes.");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    CURRENT_BRANCH_BRANCH,
    TARGET_COMMIT_OID_SUMMARY,
    TAG_TAG,
    REMOTE_TAG_REMOTE_TAG,
    SELECTED_FILES_COUNT,
    ALL_STAGED_FILES_COUNT,
    ALL_UNSTAGED_FILES_COUNT,
    UNRESOLVED_COUNT_BLOCKS_ARE_STILL_UNRESOLVED_MARK_THIS_FILE_AS,
    NEW_VERSION_V_UPDATE_VERSION_AVAILABLE,
    RELEASED_UPDATE_PUBLISHED_AT,
    PACKAGE_SIZE_UPDATE_SIZE,
    CURRENT_CURRENT_DB_LABEL,
    CURRENT_RELOCATION_DB_LABEL,
    NEW_VERSION_V_VERSION_AVAILABLE,
    RELEASED_PUBLISHED_AT,
    PACKAGE_SIZE_SIZE_MB_MB,
    VERSION_V_VERSION_HAS_BEEN_DOWNLOADED_AND_VERIFIED_THE_APPLICATION,
    THE_CURRENT_DIRECTORY_IS_NOT_WRITABLE_SO_VERSION_V_VERSION,
    AUTOMATIC_RECORD,
    DELETE_REMOTE_NAME,
    DELETE_REMOTE_BRANCH_FULL_NAME,
    PROVIDER_LABEL_IS_OPEN_IN_YOUR_BROWSER_VERIFY_THE_CODE,
    COMPLETE_PROVIDER_LABEL_SIGN_IN_IN_YOUR_BROWSER,
    CREDENTIAL_RECORD_LABEL,
    DELETE_CREDENTIAL_LABEL,
    CLONE_REPOSITORY,
    CLONE_SUBMODULES_RECURSIVELY,
    ENTER_A_REMOTE_REPOSITORY_URL_AND_PARENT_FOLDER_TO_SEE,
    SELECT_FOLDER,
    CANCEL,
    CLONE,
    CLOSE_KHASLANA,
    EXIT_THE_APPLICATION_OR_KEEP_KHASLANA_RUNNING_IN_THE_SYSTEM,
    WHEN_MINIMIZED_TO_THE_TRAY_CLICK_THE_TRAY_ICON_TO,
    MINIMIZE_TO_TRAY,
    EXIT,
    MIGRATE_TO_PORTABLE_DIRECTORY,
    APPLICATION_DATA_IS_CURRENTLY_IN_A_SYSTEM_DIRECTORY_ON_DRIVE,
    KEEP_CURRENT_LOCATION,
    MIGRATE_AND_RESTART,
    UNKNOWN,
    MOVE_TO_A_SAFE_DIRECTORY,
    MOVE_AND_RESTART,
    CREATE_BRANCH,
    BASE_BRANCH,
    CURRENT_HEAD,
    SELECT_BASE_BRANCH,
    SWITCH_TO_THE_NEW_BRANCH_AFTER_CREATION,
    CREATE,
    RENAME_BRANCH,
    RENAME,
    CONFIRM_BRANCH_RESET,
    A_HARD_RESET_MOVES_THE_CURRENT_BRANCH_AND_REMOVES_LATER,
    CONFIRM_RESET,
    CONFIRM_REVERT_COMMIT,
    THIS_CREATES_A_NEW_COMMIT_THAT_REVERTS_THE_CHANGES_INTRODUCED,
    CONFIRM_REVERT,
    CONFIRM_REVERT_MERGE_COMMIT,
    THIS_CREATES_A_NEW_COMMIT_THAT_REVERTS_THE_CHANGES_INTRODUCED_2,
    THE_ORIGINAL_MERGE_COMMIT_IS_RETAINED_AND_HISTORY_IS_NOT,
    THIS_IS_EQUIVALENT_TO_GIT_REVERT_M_PRESERVING_THE_FIRST,
    CONFIRM_REVERT_MERGE,
    CONFIRM_UNDO_TO_STAGING,
    THIS_REMOVES_THE_COMMIT_AND_KEEPS_ITS_CHANGES_IN_THE,
    ONLY_THE_LATEST_UNPUSHED_NON_MERGE_COMMIT_ON_THE_CURRENT,
    CONFIRM_UNDO,
    AMEND_A_PUSHED_COMMIT,
    THE_LATEST_COMMIT_HAS_ALREADY_BEEN_PUSHED_TO_THE_REMOTE,
    AMENDING_REWRITES_THIS_COMMIT_UPDATING_THE_REMOTE_THEN_REQUIRES_A,
    AMEND_ONLY_COMMITS_THAT_HAVE_NOT_BEEN_PUSHED,
    AMEND_AND_PUSH_ANYWAY,
    AMEND_ANYWAY,
    THIS_REPOSITORY_HAS_NO_REMOTE,
    TARGET_COMMIT_HEAD_LATEST_COMMIT_ON_THE_CURRENT_BRANCH,
    CREATE_TAG,
    CREATE_ANNOTATED_TAG_RECORDS_MESSAGE_AND_CREATOR_RECOMMENDED_FOR_RELEASES,
    SELECT_REMOTE,
    PUSH_TAG,
    PUSH,
    DELETE_TAG,
    THIS_DELETES_THE_LOCAL_TAG_ONLY_REMOTE_TAGS_ARE_NOT,
    CONFIRM_DELETION,
    DELETE_REMOTE_TAG,
    THIS_DELETES_THE_TAG_FROM_THE_REMOTE_PUBLISHED_VERSION_REFERENCES,
    TARGET_FILE,
    DISCARD_ALL_UNCOMMITTED_CHANGES_IN_THESE_FILES_BOTH_STAGED_AND,
    DISCARD_ONLY_UNSTAGED_CHANGES_IN_THESE_FILES_STAGED_CHANGES_ARE,
    CONFIRM_DISCARD_CHANGES,
    OVERWRITE_EXISTING_CONFLICT_RESOLUTION,
    THIS_DRAFT_ALREADY_CONTAINS_BLOCK_RESOLUTIONS_OR_MANUAL_EDITS_GENERATING,
    ACCEPTED_BLOCKS_IGNORED_BLOCKS_AND_MANUAL_EDITS_WILL_BE_LOST,
    GO_BACK_AND_KEEP_CHANGES,
    OVERWRITE_AND_GENERATE,
    CURRENT_CONFLICT_FILE,
    UNRESOLVED_BLOCKS_REMAIN,
    THE_CURRENT_RESULT_WILL_BE_WRITTEN_TO_THE_WORKING_TREE,
    GO_BACK_TO_RESOLVING,
    RESOLVE_ANYWAY,
    NO_RELEASE_NOTES_FOR_THIS_VERSION,
    NONE,
    UPDATES,
    AUTOMATIC_CHECKS_AT_STARTUP_AND_MANUAL_CHECKS_SKIPPED_VERSIONS_ONLY,
    CURRENT_VERSION,
    CHECK_FOR_UPDATES_AUTOMATICALLY,
    RECEIVE_BETA_UPDATES,
    ALSO_CHECK_FOR_AND_INSTALL_BETA_RELEASES_THEY_MAY_BE,
    SKIPPED_VERSION,
    CHECK_NOW,
    CLEAR_SKIPPED_VERSION,
    CHANNELS_AND_MIGRATION,
    PENDING_UPDATES_DATA_DIRECTORY_MIGRATION_AND_APPLICATION_RELOCATION,
    UPDATE_NOW,
    PREPARING_DOWNLOAD,
    DATA_DIRECTORY,
    MOVE_DATA_FROM_THE_SYSTEM_DIRECTORY_ON_DRIVE_C_TO,
    APPLICATION_LOCATION,
    SKIP_THIS_VERSION,
    LATER,
    UPDATE_READY,
    THE_APPLICATION_EXITS_AND_RESTARTS_DURING_INSTALLATION_SAVE_YOUR_WORK,
    RESTART_NOW,
    THE_STAGING_DIRECTORY_IS_MISSING,
    CANNOT_UPDATE_AUTOMATICALLY,
    OPEN_CNB_DOWNLOADS,
    OPEN_GITHUB_RELEASE,
    CLOSE,
    NO_REMOTES_YET_CLICK_ADD_REMOTE_TO_CREATE_ONE,
    MANAGE_REMOTES,
    ADD_REMOTE,
    THE_REMOTE_ADDRESS_IS_USED_FOR_BOTH_FETCH_AND_PUSH,
    NAME,
    ADDRESS,
    CREDENTIALS,
    ACTIONS,
    NO_CREDENTIALS,
    CREDENTIAL_RECORD_MISSING,
    AUTOMATIC_MATCH,
    EDIT,
    DELETE,
    EDIT_REMOTE,
    BIND_CREDENTIALS,
    SAVE,
    AUTOMATICALLY_MATCH_SAVED_CREDENTIALS,
    NOT_A_MATCH,
    DELETE_REMOTE,
    THIS_ONLY_REMOVES_THE_REMOTE_CONFIGURATION_FROM_THIS_REPOSITORY_SAVED,
    DELETE_REMOTE_BRANCH,
    THIS_DELETES_THE_BRANCH_ON_THE_REMOTE_AND_REFRESHES_LOCAL,
    QUICK_SIGN_IN,
    GITEE_SIGN_IN_REQUIRES_A_MAINTAINER_DEPLOYED_TOKEN_EXCHANGE_SERVICE,
    UNKNOWN_RECORD,
    TEST_CREDENTIAL_CONNECTION,
    THIS_ADDRESS_IS_USED_FOR_A_GIT_CONNECTION_TO_VERIFY,
    START_TEST,
    ADD_CREDENTIALS,
    TYPE,
    PREFER_SSH_AGENT_WHEN_USING_A_PRIVATE_KEY_FILE_ONLY,
    USE_SSH_AGENT_DO_NOT_SAVE_A_PRIVATE_KEY_PATH,
    SELECT_PRIVATE_KEY_FILE,
    REUSE_SCOPE,
    THIS_REMOTE_ONLY,
    SAME_HOST,
    SECRETS_ARE_STORED_ONLY_IN_THE_SYSTEM_CREDENTIAL_MANAGER_PASSWORDS,
    REFRESH,
    CREDENTIAL_LIST_REFRESHED,
    NO_SAVED_CREDENTIALS_SAVE_CREDENTIALS_DURING_A_REMOTE_OPERATION_TO,
    CREDENTIAL_DETAILS,
    THIS_CREDENTIAL_RECORD_DOES_NOT_EXIST_IT_MAY_HAVE_BEEN,
    HOST_REMOTE,
    USERNAME,
    SSH_KEY_PATH,
    CREATED,
    UPDATED,
    LAST_USED,
    PASSWORDS_PATS_AND_SSH_PASSPHRASES_ARE_NEVER_DISPLAYED_HERE,
    SCOPE,
    TEST,
    DELETE_CREDENTIALS,
    THIS_REMOVES_BOTH_THE_NON_SENSITIVE_INDEX_RECORD_AND_THE,
    RELOCATION_DETAILS,
    DOWNLOAD_LOCATION_WARNING,
    VOLATILE_LOCATION_WARNING,
    PORTABLE_MIGRATION_DETAILS,
    MOVE_PROGRAM_AND_DATA,
    MOVE_PROGRAM_ONLY,
    RESET_SOFT_DESCRIPTION,
    RESET_MIXED_DESCRIPTION,
    RESET_HARD_DESCRIPTION,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
