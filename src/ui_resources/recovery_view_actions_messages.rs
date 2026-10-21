//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const RECOVERY_TASK_FAILED_UNEXPECTEDLY_PAYLOAD: UiMessage = UiMessage::new("views.recovery_view.actions.recovery_task_failed_unexpectedly_payload", "恢复任务异常：{payload}", "Recovery task failed unexpectedly: {payload}");
pub const FAILED_TO_READ_RECOVERY_RECORDS_ERROR_PREVIOUS_AVAILABLE_RESULTS_RETAINED: UiMessage = UiMessage::new("views.recovery_view.actions.failed_to_read_recovery_records_error_previous_available_results_retained", "读取恢复记录失败：{error}。保留上次可用结果。", "Failed to read recovery records: {error}. Previous available results retained.");
pub const RECOVERY_BRANCH_CREATED_BUT_REFRESHING_BRANCHES_FAILED_ERROR: UiMessage = UiMessage::new("views.recovery_view.actions.recovery_branch_created_but_refreshing_branches_failed_error", "恢复分支已创建，分支列表刷新失败：{error}", "Recovery branch created, but refreshing branches failed: {error}");
pub const FAILED_TO_CREATE_RECOVERY_BRANCH_ERROR: UiMessage = UiMessage::new("views.recovery_view.actions.failed_to_create_recovery_branch_error", "创建恢复分支失败：{error}", "Failed to create recovery branch: {error}");
pub const REPOSITORY_OR_WORKTREE_IDENTITY_CHANGED_REFRESH_RECOVERY_RECORDS: UiMessage = UiMessage::new("views.recovery_view.actions.repository_or_worktree_identity_changed_refresh_recovery_records", "仓库或工作树身份已变化，请刷新恢复记录", "Repository or worktree identity changed. Refresh recovery records.");
pub const REPOSITORY_IDENTITY_CHANGED_WHILE_READING_TRY_AGAIN: UiMessage = UiMessage::new("views.recovery_view.actions.repository_identity_changed_while_reading_try_again", "读取期间仓库身份变化，请重试", "Repository identity changed while reading. Try again.");
pub const READ_CANCELLED_PREVIOUS_AVAILABLE_RESULTS_RETAINED: UiMessage = UiMessage::new("views.recovery_view.actions.read_cancelled_previous_available_results_retained", "读取已取消，保留上次可用结果", "Read cancelled; previous available results retained");
pub const REPOSITORY_CLOSED: UiMessage = UiMessage::new("views.recovery_view.actions.repository_closed", "仓库已关闭", "Repository closed");
pub const THIS_BRANCH_NAME_ALREADY_EXISTS_CHOOSE_ANOTHER_NAME: UiMessage = UiMessage::new("views.recovery_view.actions.this_branch_name_already_exists_choose_another_name", "分支名称已存在，请使用其他名称", "This branch name already exists. Choose another name.");
pub const CREATING_RECOVERY_BRANCH: UiMessage = UiMessage::new("views.recovery_view.actions.creating_recovery_branch", "正在创建恢复分支", "Creating recovery branch");
pub const CREATION_CANCELLED_NO_REFERENCES_WERE_WRITTEN: UiMessage = UiMessage::new("views.recovery_view.actions.creation_cancelled_no_references_were_written", "已取消创建，未执行引用写入", "Creation cancelled; no references were written");
pub const RECOVERY_BRANCH_NOT_CREATED: UiMessage = UiMessage::new("views.recovery_view.actions.recovery_branch_not_created", "恢复分支未创建", "Recovery branch not created");

pub const BRANCH_REFRESH_WARNING: UiMessage = UiMessage::new("views.recovery.actions.branch_refresh_warning", "恢复分支已创建，但分支列表刷新失败：{error}", "Recovery branch created, but refreshing the branch list failed: {error}");

pub const RECOVERY_BRANCH_CREATED: UiMessage = UiMessage::new("views.recovery.actions.recovery_branch_created", "恢复分支 {branch} 已创建，当前分支未切换", "Recovery branch {branch} created; current branch unchanged");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    RECOVERY_TASK_FAILED_UNEXPECTEDLY_PAYLOAD,
    FAILED_TO_READ_RECOVERY_RECORDS_ERROR_PREVIOUS_AVAILABLE_RESULTS_RETAINED,
    RECOVERY_BRANCH_CREATED_BUT_REFRESHING_BRANCHES_FAILED_ERROR,
    FAILED_TO_CREATE_RECOVERY_BRANCH_ERROR,
    REPOSITORY_OR_WORKTREE_IDENTITY_CHANGED_REFRESH_RECOVERY_RECORDS,
    REPOSITORY_IDENTITY_CHANGED_WHILE_READING_TRY_AGAIN,
    READ_CANCELLED_PREVIOUS_AVAILABLE_RESULTS_RETAINED,
    REPOSITORY_CLOSED,
    THIS_BRANCH_NAME_ALREADY_EXISTS_CHOOSE_ANOTHER_NAME,
    CREATING_RECOVERY_BRANCH,
    CREATION_CANCELLED_NO_REFERENCES_WERE_WRITTEN,
    RECOVERY_BRANCH_NOT_CREATED,
    BRANCH_REFRESH_WARNING,
    RECOVERY_BRANCH_CREATED,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
