//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const BEFORE_OPERATION: UiMessage = UiMessage::new("views.recovery_view.state.before_operation", "操作前", "Before operation");
pub const AFTER_OPERATION: UiMessage = UiMessage::new("views.recovery_view.state.after_operation", "操作后", "After operation");
pub const SEARCH_OPERATIONS_REFERENCES_ACTORS_OR_OIDS: UiMessage = UiMessage::new("views.recovery_view.state.search_operations_references_actors_or_oids", "搜索操作、引用、操作者或 OID", "Search operations, references, actors, or OIDs");
pub const RECOVERY_BRANCH_NAME: UiMessage = UiMessage::new("views.recovery_view.state.recovery_branch_name", "恢复分支名称", "Recovery branch name");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    BEFORE_OPERATION,
    AFTER_OPERATION,
    SEARCH_OPERATIONS_REFERENCES_ACTORS_OR_OIDS,
    RECOVERY_BRANCH_NAME,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
