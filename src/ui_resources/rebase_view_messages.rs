//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const START_REBASE: UiMessage = UiMessage::new("views.rebase_view.start_rebase", "开始变基", "Start rebase");
pub const REBASE_IN_PROGRESS_CONFLICTS_TO_RESOLVE: UiMessage = UiMessage::new("views.rebase_view.rebase_in_progress_conflicts_to_resolve", "变基进行中 · 存在冲突待解决", "Rebase in progress · Conflicts to resolve");
pub const REBASE_IN_PROGRESS: UiMessage = UiMessage::new("views.rebase_view.rebase_in_progress", "变基进行中", "Rebase in progress");
pub const CONTINUE_REBASE: UiMessage = UiMessage::new("views.rebase_view.continue_rebase", "继续变基", "Continue rebase");
pub const SKIP_THIS_COMMIT: UiMessage = UiMessage::new("views.rebase_view.skip_this_commit", "跳过此提交", "Skip this commit");
pub const ABORT: UiMessage = UiMessage::new("views.rebase_view.abort", "中止", "Abort");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    START_REBASE,
    REBASE_IN_PROGRESS_CONFLICTS_TO_RESOLVE,
    REBASE_IN_PROGRESS,
    CONTINUE_REBASE,
    SKIP_THIS_COMMIT,
    ABORT,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
