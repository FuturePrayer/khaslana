//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const MERGE_IN_PROGRESS_CONFLICT_COUNT_CONFLICTS_TO_RESOLVE: UiMessage = UiMessage::new("views.merge_view.merge_in_progress_conflict_count_conflicts_to_resolve", "合并进行中 · {conflict_count} 个冲突待解决", "Merge in progress · {conflict_count} conflicts to resolve");
pub const A_MERGE_IS_IN_PROGRESS_CANNOT_ACTION_COMPLETE_OR_ABORT: UiMessage = UiMessage::new("views.merge_view.a_merge_is_in_progress_cannot_action_complete_or_abort", "合并正在进行，不能{action}；请先完成合并或中止合并", "A merge is in progress. Cannot {action}; complete or abort the merge first.");
pub const COMPLETE_MERGE: UiMessage = UiMessage::new("views.merge_view.complete_merge", "完成合并", "Complete merge");
pub const COMMIT: UiMessage = UiMessage::new("views.merge_view.commit", "提交", "Commit");
pub const ENTER_A_MERGE_COMMIT_MESSAGE: UiMessage = UiMessage::new("views.merge_view.enter_a_merge_commit_message", "需要填写合并提交信息", "Enter a merge commit message");
pub const UNRESOLVED_CONFLICT_FILES_REMAIN_CANNOT_COMPLETE_THE_MERGE: UiMessage = UiMessage::new("views.merge_view.unresolved_conflict_files_remain_cannot_complete_the_merge", "仍有冲突文件未解决，不能完成合并", "Unresolved conflict files remain; cannot complete the merge");
pub const RESOLVE_CONFLICTS_WITH_IDEA: UiMessage = UiMessage::new("views.merge_view.resolve_conflicts_with_idea", "使用 IDEA 解决冲突", "Resolve conflicts with IDEA");
pub const ABORT_MERGE: UiMessage = UiMessage::new("views.merge_view.abort_merge", "中止合并", "Abort merge");
pub const ABORT_THE_CURRENT_MERGE: UiMessage = UiMessage::new("views.merge_view.abort_the_current_merge", "确定要中止当前合并吗？", "Abort the current merge?");
pub const ALL_CHANGES_FROM_THIS_MERGE_INCLUDING_COMPLETED_CONFLICT_RESOLUTIONS_WILL: UiMessage = UiMessage::new("views.merge_view.all_changes_from_this_merge_including_completed_conflict_resolutions_will", "本次合并产生的内容以及已经完成的冲突解决结果都会被丢弃。", "All changes from this merge, including completed conflict resolutions, will be discarded.");
pub const CANCEL: UiMessage = UiMessage::new("views.merge_view.cancel", "取消", "Cancel");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    MERGE_IN_PROGRESS_CONFLICT_COUNT_CONFLICTS_TO_RESOLVE,
    A_MERGE_IS_IN_PROGRESS_CANNOT_ACTION_COMPLETE_OR_ABORT,
    COMPLETE_MERGE,
    COMMIT,
    ENTER_A_MERGE_COMMIT_MESSAGE,
    UNRESOLVED_CONFLICT_FILES_REMAIN_CANNOT_COMPLETE_THE_MERGE,
    RESOLVE_CONFLICTS_WITH_IDEA,
    ABORT_MERGE,
    ABORT_THE_CURRENT_MERGE,
    ALL_CHANGES_FROM_THIS_MERGE_INCLUDING_COMPLETED_CONFLICT_RESOLUTIONS_WILL,
    CANCEL,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
