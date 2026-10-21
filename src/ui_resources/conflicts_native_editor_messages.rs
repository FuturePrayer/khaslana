//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const CONFLICT_BLOCK_STATUS: UiMessage = UiMessage::new("views.conflicts.native_editor.conflict_block_status", "冲突 {block} · {status}", "Conflict {block} · {status}");
pub const MANUALLY_ADJUSTED_CONFIRMATION_NEEDED: UiMessage = UiMessage::new("views.conflicts.native_editor.manually_adjusted_confirmation_needed", "已手动微调 · 待确认", "Manually adjusted · Confirmation needed");
pub const PENDING_NOT_ACCEPTED: UiMessage = UiMessage::new("views.conflicts.native_editor.pending_not_accepted", "待处理 · 未采纳", "Pending · Not accepted");
pub const RESOLVED: UiMessage = UiMessage::new("views.conflicts.native_editor.resolved", "已处理", "Resolved");
pub const KEEP_CURRENT: UiMessage = UiMessage::new("views.conflicts.native_editor.keep_current", "保留当前", "Keep current");
pub const KEEP_INCOMING: UiMessage = UiMessage::new("views.conflicts.native_editor.keep_incoming", "保留传入", "Keep incoming");
pub const KEEP_BOTH: UiMessage = UiMessage::new("views.conflicts.native_editor.keep_both", "全部保留", "Keep both");
pub const CURRENT_FIRST_INCOMING_SECOND: UiMessage = UiMessage::new("views.conflicts.native_editor.current_first_incoming_second", "当前在前，传入在后", "Current first, incoming second");
pub const INCOMING_FIRST_CURRENT_SECOND: UiMessage = UiMessage::new("views.conflicts.native_editor.incoming_first_current_second", "传入在前，当前在后", "Incoming first, current second");
pub const RESET_TO_BASE: UiMessage = UiMessage::new("views.conflicts.native_editor.reset_to_base", "重置祖先", "Reset to base");
pub const RESTORE_THIS_BLOCK_S_COMMON_ANCESTOR_AND_RESOLVE_AGAIN: UiMessage = UiMessage::new("views.conflicts.native_editor.restore_this_block_s_common_ancestor_and_resolve_again", "恢复此块共同祖先，重新处理", "Restore this block’s common ancestor and resolve again");
pub const THIS_BLOCK_HAS_NO_COMMON_ANCESTOR: UiMessage = UiMessage::new("views.conflicts.native_editor.this_block_has_no_common_ancestor", "此块没有共同祖先", "This block has no common ancestor");
pub const CONFIRM_RESULT: UiMessage = UiMessage::new("views.conflicts.native_editor.confirm_result", "确认结果", "Confirm result");
pub const EXPAND_CANDIDATES_CHOOSE_AGAIN: UiMessage = UiMessage::new("views.conflicts.native_editor.expand_candidates_choose_again", "展开候选 · 重新选择", "Expand candidates · Choose again");
pub const ACCEPT_THIS_LINE_INTO_THE_CORRESPONDING_RESULT_LINE_THE_ENTIRE: UiMessage = UiMessage::new("views.conflicts.native_editor.accept_this_line_into_the_corresponding_result_line_the_entire", "采纳此行到对应结果行；仍需确认整个块", "Accept this line into the corresponding result line; the entire block still needs confirmation");
pub const CURRENT_VERSION_READ_ONLY_CANDIDATE: UiMessage = UiMessage::new("views.conflicts.native_editor.current_version_read_only_candidate", "当前版本 · 只读候选", "Current version · Read-only candidate");
pub const INCOMING_VERSION_READ_ONLY_CANDIDATE: UiMessage = UiMessage::new("views.conflicts.native_editor.incoming_version_read_only_candidate", "传入版本 · 只读候选", "Incoming version · Read-only candidate");
pub const CURRENT_RESULT_DIRECTLY_EDITABLE: UiMessage = UiMessage::new("views.conflicts.native_editor.current_result_directly_editable", "当前结果 · 可直接编辑", "Current result · Directly editable");

pub const ACCEPT_CURRENT_LINE: UiMessage = UiMessage::new("views.conflicts.native_editor.accept_current_line", "采纳当前第 {line} 行", "Accept current line {line}");

pub const ACCEPT_INCOMING_LINE: UiMessage = UiMessage::new("views.conflicts.native_editor.accept_incoming_line", "采纳传入第 {line} 行", "Accept incoming line {line}");

pub const EDITOR_ACCESSIBILITY: UiMessage = UiMessage::new("views.conflicts.native_editor.editor_accessibility", "合并结果，可编辑；当前与传入候选只读", "Merge result, editable; current and incoming candidates are read-only");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    CONFLICT_BLOCK_STATUS,
    MANUALLY_ADJUSTED_CONFIRMATION_NEEDED,
    PENDING_NOT_ACCEPTED,
    RESOLVED,
    KEEP_CURRENT,
    KEEP_INCOMING,
    KEEP_BOTH,
    CURRENT_FIRST_INCOMING_SECOND,
    INCOMING_FIRST_CURRENT_SECOND,
    RESET_TO_BASE,
    RESTORE_THIS_BLOCK_S_COMMON_ANCESTOR_AND_RESOLVE_AGAIN,
    THIS_BLOCK_HAS_NO_COMMON_ANCESTOR,
    CONFIRM_RESULT,
    EXPAND_CANDIDATES_CHOOSE_AGAIN,
    ACCEPT_THIS_LINE_INTO_THE_CORRESPONDING_RESULT_LINE_THE_ENTIRE,
    CURRENT_VERSION_READ_ONLY_CANDIDATE,
    INCOMING_VERSION_READ_ONLY_CANDIDATE,
    CURRENT_RESULT_DIRECTLY_EDITABLE,
    ACCEPT_CURRENT_LINE,
    ACCEPT_INCOMING_LINE,
    EDITOR_ACCESSIBILITY,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
