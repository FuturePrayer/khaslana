//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const CODE_MERGE_FILENAME: UiMessage = UiMessage::new("views.conflicts.workbench.code_merge_filename", "代码合并 / {filename}", "Code merge / {filename}");
pub const CONFLICT_SELECTED_COUNT: UiMessage = UiMessage::new("views.conflicts.workbench.conflict_selected_count", "冲突 {selected} / {count}", "Conflict {selected} / {count}");
pub const RESOLVED_HANDLED_BLOCK_COUNT_BLOCKS_UNRESOLVED_UNRESOLVED_BLOCK_COUNT: UiMessage = UiMessage::new("views.conflicts.workbench.resolved_handled_block_count_blocks_unresolved_unresolved_block_count", "已处理 {handled_block_count} / {blocks} · 未处理 {unresolved_block_count}", "Resolved {handled_block_count} / {blocks} · Unresolved {unresolved_block_count}");
pub const RESOLVE_WITH_IDEA: UiMessage = UiMessage::new("views.conflicts.workbench.resolve_with_idea", "用 IDEA 解决", "Resolve with IDEA");
pub const COMPLETE_MERGE: UiMessage = UiMessage::new("views.conflicts.workbench.complete_merge", "完成合并", "Complete merge");
pub const AN_OPERATION_IS_RUNNING: UiMessage = UiMessage::new("views.conflicts.workbench.an_operation_is_running", "当前操作运行中", "An operation is running");
pub const RESOLVE_ALL_CONFLICT_FILES_FIRST: UiMessage = UiMessage::new("views.conflicts.workbench.resolve_all_conflict_files_first", "先解决全部冲突文件", "Resolve all conflict files first");
pub const ENTER_A_MERGE_COMMIT_MESSAGE_IN_THE_WORKING_TREE_FIRST: UiMessage = UiMessage::new("views.conflicts.workbench.enter_a_merge_commit_message_in_the_working_tree_first", "请先在工作区填写合并提交说明", "Enter a merge commit message in the working tree first");
pub const ACCEPT_BOTH_CURRENT_FIRST: UiMessage = UiMessage::new("views.conflicts.workbench.accept_both_current_first", "接受两边：当前在前", "Accept both: current first");
pub const ACCEPT_BOTH_INCOMING_FIRST: UiMessage = UiMessage::new("views.conflicts.workbench.accept_both_incoming_first", "接受两边：传入在前", "Accept both: incoming first");
pub const SHOW_HIDE_COMMON_ANCESTOR: UiMessage = UiMessage::new("views.conflicts.workbench.show_hide_common_ancestor", "显示 / 隐藏共同祖先", "Show / hide common ancestor");
pub const IGNORE_BLOCK_AND_KEEP_CURRENT_RESULT: UiMessage = UiMessage::new("views.conflicts.workbench.ignore_block_and_keep_current_result", "忽略该块，保留现有结果", "Ignore block and keep current result");
pub const APPLY_TO_WORKING_TREE: UiMessage = UiMessage::new("views.conflicts.workbench.apply_to_working_tree", "应用到工作区", "Apply to working tree");
pub const RESOLVE_WITH_INTELLIJ_IDEA: UiMessage = UiMessage::new("views.conflicts.workbench.resolve_with_intellij_idea", "用 IntelliJ IDEA 解决", "Resolve with IntelliJ IDEA");
pub const UNDO_RESULT_EDIT: UiMessage = UiMessage::new("views.conflicts.workbench.undo_result_edit", "撤销结果编辑", "Undo result edit");
pub const REDO_RESULT_EDIT: UiMessage = UiMessage::new("views.conflicts.workbench.redo_result_edit", "重做结果编辑", "Redo result edit");
pub const COPY_UNAPPLIED_AI_SUGGESTION: UiMessage = UiMessage::new("views.conflicts.workbench.copy_unapplied_ai_suggestion", "复制未回填的 AI 建议", "Copy unapplied AI suggestion");
pub const AI_SUGGESTION_COPIED_REVIEW_IT_BEFORE_PASTING_INTO_THE_MERGE: UiMessage = UiMessage::new("views.conflicts.workbench.ai_suggestion_copied_review_it_before_pasting_into_the_merge", "已复制 AI 建议，请检查后粘贴到合并结果", "AI suggestion copied. Review it before pasting into the merge result.");
pub const ACCEPT_CURRENT_FOR_ENTIRE_FILE: UiMessage = UiMessage::new("views.conflicts.workbench.accept_current_for_entire_file", "整文件接受当前", "Accept current for entire file");
pub const ACCEPT_INCOMING_FOR_ENTIRE_FILE: UiMessage = UiMessage::new("views.conflicts.workbench.accept_incoming_for_entire_file", "整文件接受传入", "Accept incoming for entire file");
pub const ABORT_MERGE: UiMessage = UiMessage::new("views.conflicts.workbench.abort_merge", "中止合并", "Abort merge");
pub const CONTINUE_REBASE: UiMessage = UiMessage::new("views.conflicts.workbench.continue_rebase", "继续变基", "Continue rebase");
pub const SKIP_CURRENT_REBASE_COMMIT: UiMessage = UiMessage::new("views.conflicts.workbench.skip_current_rebase_commit", "跳过当前变基提交", "Skip current rebase commit");
pub const ABORT_REBASE: UiMessage = UiMessage::new("views.conflicts.workbench.abort_rebase", "中止变基", "Abort rebase");
pub const PREVIOUS_CONFLICT_BLOCK: UiMessage = UiMessage::new("views.conflicts.workbench.previous_conflict_block", "上一个冲突块", "Previous conflict block");
pub const NEXT_CONFLICT_BLOCK: UiMessage = UiMessage::new("views.conflicts.workbench.next_conflict_block", "下一个冲突块", "Next conflict block");
pub const FILES: UiMessage = UiMessage::new("views.conflicts.workbench.files", "文件", "Files");
pub const SINGLE_DOCUMENT_CANDIDATE_BLOCKS: UiMessage = UiMessage::new("views.conflicts.workbench.single_document_candidate_blocks", "单文档 · 候选块", "Single document · Candidate blocks");
pub const ACCEPT_BOTH: UiMessage = UiMessage::new("views.conflicts.workbench.accept_both", "接受两边", "Accept both");
pub const COMMON_ANCESTOR: UiMessage = UiMessage::new("views.conflicts.workbench.common_ancestor", "共同祖先", "Common ancestor");
pub const IGNORE_BLOCK: UiMessage = UiMessage::new("views.conflicts.workbench.ignore_block", "忽略该块", "Ignore block");
pub const UNDO_RESULT_EDIT_CTRL_Z: UiMessage = UiMessage::new("views.conflicts.workbench.undo_result_edit_ctrl_z", "撤销结果编辑 · Ctrl+Z", "Undo result edit · Ctrl+Z");
pub const REDO_RESULT_EDIT_CTRL_Y: UiMessage = UiMessage::new("views.conflicts.workbench.redo_result_edit_ctrl_y", "重做结果编辑 · Ctrl+Y", "Redo result edit · Ctrl+Y");
pub const AI_GENERATING: UiMessage = UiMessage::new("views.conflicts.workbench.ai_generating", "AI 生成中…", "AI generating…");
pub const AI_NOT_CONFIGURED: UiMessage = UiMessage::new("views.conflicts.workbench.ai_not_configured", "AI 未配置", "AI not configured");
pub const AI_MERGE_SUGGESTION: UiMessage = UiMessage::new("views.conflicts.workbench.ai_merge_suggestion", "AI 合并建议", "AI merge suggestion");
pub const MORE: UiMessage = UiMessage::new("views.conflicts.workbench.more", "更多", "More");
pub const MERGE_RESULT: UiMessage = UiMessage::new("views.conflicts.workbench.merge_result", "合并结果", "Merge result");
pub const EDITABLE: UiMessage = UiMessage::new("views.conflicts.workbench.editable", "可编辑", "Editable");
pub const UTF_READ_ONLY_CANDIDATES_ONLY_RESULT_CONTENT_IS_SAVED: UiMessage = UiMessage::new("views.conflicts.workbench.utf_read_only_candidates_only_result_content_is_saved", "UTF-8 · 候选只读 · 实际保存结果内容", "UTF-8 · Read-only candidates · Only result content is saved");
pub const COMMON_ANCESTOR_CURRENT_BLOCK_READ_ONLY: UiMessage = UiMessage::new("views.conflicts.workbench.common_ancestor_current_block_read_only", "共同祖先 · 当前块 · 只读", "Common ancestor · Current block · Read-only");
pub const RESULT_IS_ALWAYS_EDITABLE_CANDIDATES_ARE_NEVER_SAVED_CTRL_Z: UiMessage = UiMessage::new("views.conflicts.workbench.result_is_always_editable_candidates_are_never_saved_ctrl_z", "结果始终可编辑 · 候选不会混入保存内容 · Ctrl+Z 撤销", "Result is always editable · Candidates are never saved · Ctrl+Z Undo");
pub const APPLY_AND_MARK_RESOLVED: UiMessage = UiMessage::new("views.conflicts.workbench.apply_and_mark_resolved", "应用并标记已解决", "Apply and mark resolved");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    CODE_MERGE_FILENAME,
    CONFLICT_SELECTED_COUNT,
    RESOLVED_HANDLED_BLOCK_COUNT_BLOCKS_UNRESOLVED_UNRESOLVED_BLOCK_COUNT,
    RESOLVE_WITH_IDEA,
    COMPLETE_MERGE,
    AN_OPERATION_IS_RUNNING,
    RESOLVE_ALL_CONFLICT_FILES_FIRST,
    ENTER_A_MERGE_COMMIT_MESSAGE_IN_THE_WORKING_TREE_FIRST,
    ACCEPT_BOTH_CURRENT_FIRST,
    ACCEPT_BOTH_INCOMING_FIRST,
    SHOW_HIDE_COMMON_ANCESTOR,
    IGNORE_BLOCK_AND_KEEP_CURRENT_RESULT,
    APPLY_TO_WORKING_TREE,
    RESOLVE_WITH_INTELLIJ_IDEA,
    UNDO_RESULT_EDIT,
    REDO_RESULT_EDIT,
    COPY_UNAPPLIED_AI_SUGGESTION,
    AI_SUGGESTION_COPIED_REVIEW_IT_BEFORE_PASTING_INTO_THE_MERGE,
    ACCEPT_CURRENT_FOR_ENTIRE_FILE,
    ACCEPT_INCOMING_FOR_ENTIRE_FILE,
    ABORT_MERGE,
    CONTINUE_REBASE,
    SKIP_CURRENT_REBASE_COMMIT,
    ABORT_REBASE,
    PREVIOUS_CONFLICT_BLOCK,
    NEXT_CONFLICT_BLOCK,
    FILES,
    SINGLE_DOCUMENT_CANDIDATE_BLOCKS,
    ACCEPT_BOTH,
    COMMON_ANCESTOR,
    IGNORE_BLOCK,
    UNDO_RESULT_EDIT_CTRL_Z,
    REDO_RESULT_EDIT_CTRL_Y,
    AI_GENERATING,
    AI_NOT_CONFIGURED,
    AI_MERGE_SUGGESTION,
    MORE,
    MERGE_RESULT,
    EDITABLE,
    UTF_READ_ONLY_CANDIDATES_ONLY_RESULT_CONTENT_IS_SAVED,
    COMMON_ANCESTOR_CURRENT_BLOCK_READ_ONLY,
    RESULT_IS_ALWAYS_EDITABLE_CANDIDATES_ARE_NEVER_SAVED_CTRL_Z,
    APPLY_AND_MARK_RESOLVED,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
