//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const MERGE_CONFLICTS_OCCURRED_USE_IDEA_FROM_THE_WORKING_TREE_OR: UiMessage = UiMessage::new("views.conflicts.mod.merge_conflicts_occurred_use_idea_from_the_working_tree_or", "合并产生冲突，请在工作区使用 IDEA 或进入“冲突处理”解决（{count} 个文件）", "Merge conflicts occurred. Use IDEA from the working tree or open Conflicts to resolve them ({count} files).");
pub const OPERATION_CONFLICTS: UiMessage = UiMessage::new("views.conflicts.mod.operation_conflicts", "操作产生冲突，请在左侧“冲突”区域解决（{count} 个文件）", "The operation caused conflicts. Resolve them in Conflicts on the left ({count} files).");
pub const CONFLICTS_FILES_AWAITING_RESOLUTION: UiMessage = UiMessage::new("views.conflicts.mod.conflicts_files_awaiting_resolution", "{conflicts} 个文件等待解决", "{conflicts} files awaiting resolution");
pub const UNRESOLVED_UNRESOLVED: UiMessage = UiMessage::new("views.conflicts.mod.unresolved_unresolved", "{unresolved} 个待处理", "{unresolved} unresolved");
pub const READ_FAILED_E_SELECT_THIS_FILE_AGAIN_TO_RETRY: UiMessage = UiMessage::new("views.conflicts.mod.read_failed_e_select_this_file_again_to_retry", "读取失败：{e}。请重新选择该文件重试。", "Read failed: {e}. Select this file again to retry.");
pub const COUNT_CONFLICT_FILES_REMAIN: UiMessage = UiMessage::new("views.conflicts.mod.count_conflict_files_remain", "存在 {count} 个冲突文件", "{count} conflict files remain");
pub const CONFLICTS: UiMessage = UiMessage::new("views.conflicts.mod.conflicts", "冲突", "Conflicts");
pub const CONFLICT_FILES: UiMessage = UiMessage::new("views.conflicts.mod.conflict_files", "冲突文件", "Conflict files");
pub const REBASE_IN_PROGRESS: UiMessage = UiMessage::new("views.conflicts.mod.rebase_in_progress", "变基进行中", "Rebase in progress");
pub const MERGE_IN_PROGRESS: UiMessage = UiMessage::new("views.conflicts.mod.merge_in_progress", "合并进行中", "Merge in progress");
pub const RESOLVING_CONFLICTS: UiMessage = UiMessage::new("views.conflicts.mod.resolving_conflicts", "冲突处理中", "Resolving conflicts");
pub const APPLIES_TO_THE_CURRENT_MERGE_ACROSS_THE_ENTIRE_REPOSITORY: UiMessage = UiMessage::new("views.conflicts.mod.applies_to_the_current_merge_across_the_entire_repository", "作用于整个仓库的当前合并", "Applies to the current merge across the entire repository");
pub const ABORT_MERGE: UiMessage = UiMessage::new("views.conflicts.mod.abort_merge", "中止合并", "Abort merge");
pub const TEXT: UiMessage = UiMessage::new("views.conflicts.mod.text", "文本", "Text");
pub const BINARY: UiMessage = UiMessage::new("views.conflicts.mod.binary", "二进制", "Binary");
pub const FALLBACK: UiMessage = UiMessage::new("views.conflicts.mod.fallback", "回退", "Fallback");
pub const WORKING_TREE: UiMessage = UiMessage::new("views.conflicts.mod.working_tree", "工作区", "Working tree");
pub const READ_FAILED: UiMessage = UiMessage::new("views.conflicts.mod.read_failed", "读取失败", "Read failed");
pub const LOADING: UiMessage = UiMessage::new("views.conflicts.mod.loading", "加载中", "Loading");
pub const BINARY_CONFLICT: UiMessage = UiMessage::new("views.conflicts.mod.binary_conflict", "二进制冲突", "Binary conflict");
pub const MODIFY_DELETE_OR_OTHER_CONFLICT: UiMessage = UiMessage::new("views.conflicts.mod.modify_delete_or_other_conflict", "修改 / 删除等冲突", "Modify/delete or other conflict");
pub const WORKING_TREE_CONFLICT_ONLY: UiMessage = UiMessage::new("views.conflicts.mod.working_tree_conflict_only", "仅工作区冲突", "Working tree conflict only");
pub const DRAFT_MODIFIED: UiMessage = UiMessage::new("views.conflicts.mod.draft_modified", "草稿已修改", "Draft modified");
pub const APPLIED: UiMessage = UiMessage::new("views.conflicts.mod.applied", "已应用", "Applied");
pub const SELECT_A_CONFLICT_FILE: UiMessage = UiMessage::new("views.conflicts.mod.select_a_conflict_file", "请选择一个冲突文件", "Select a conflict file");
pub const READING_THREE_WAY_VERSIONS: UiMessage = UiMessage::new("views.conflicts.mod.reading_three_way_versions", "正在读取三方版本…", "Reading three-way versions…");
pub const VISUAL_TEXT_EDITING_IS_NOT_SUPPORTED_FOR_THIS_CONFLICT: UiMessage = UiMessage::new("views.conflicts.mod.visual_text_editing_is_not_supported_for_this_conflict", "该冲突暂不支持可视化文本编辑", "Visual text editing is not supported for this conflict");
pub const OPEN_CONTAINING_FOLDER: UiMessage = UiMessage::new("views.conflicts.mod.open_containing_folder", "打开文件所在目录", "Open containing folder");
pub const CURRENT_VERSION: UiMessage = UiMessage::new("views.conflicts.mod.current_version", "当前版本", "Current version");
pub const INCOMING_VERSION: UiMessage = UiMessage::new("views.conflicts.mod.incoming_version", "传入版本", "Incoming version");
pub const MARK_RESOLVED: UiMessage = UiMessage::new("views.conflicts.mod.mark_resolved", "标记解决", "Mark resolved");
pub const SELECT_A_CONFLICT_FILE_FIRST: UiMessage = UiMessage::new("views.conflicts.mod.select_a_conflict_file_first", "请先选择一个冲突文件", "Select a conflict file first");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    MERGE_CONFLICTS_OCCURRED_USE_IDEA_FROM_THE_WORKING_TREE_OR,
    OPERATION_CONFLICTS,
    CONFLICTS_FILES_AWAITING_RESOLUTION,
    UNRESOLVED_UNRESOLVED,
    READ_FAILED_E_SELECT_THIS_FILE_AGAIN_TO_RETRY,
    COUNT_CONFLICT_FILES_REMAIN,
    CONFLICTS,
    CONFLICT_FILES,
    REBASE_IN_PROGRESS,
    MERGE_IN_PROGRESS,
    RESOLVING_CONFLICTS,
    APPLIES_TO_THE_CURRENT_MERGE_ACROSS_THE_ENTIRE_REPOSITORY,
    ABORT_MERGE,
    TEXT,
    BINARY,
    FALLBACK,
    WORKING_TREE,
    READ_FAILED,
    LOADING,
    BINARY_CONFLICT,
    MODIFY_DELETE_OR_OTHER_CONFLICT,
    WORKING_TREE_CONFLICT_ONLY,
    DRAFT_MODIFIED,
    APPLIED,
    SELECT_A_CONFLICT_FILE,
    READING_THREE_WAY_VERSIONS,
    VISUAL_TEXT_EDITING_IS_NOT_SUPPORTED_FOR_THIS_CONFLICT,
    OPEN_CONTAINING_FOLDER,
    CURRENT_VERSION,
    INCOMING_VERSION,
    MARK_RESOLVED,
    SELECT_A_CONFLICT_FILE_FIRST,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
