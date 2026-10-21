//! 工作流本模块的稳定可见消息。
use crate::ui_i18n::UiMessage;
pub const DO_NOT_PREFIX_THE_BRANCH_NAME_WITH: UiMessage = UiMessage::new("workflow.remote_branch_guard.do_not_prefix_the_branch_name_with", "远端分支名不要带远端名前缀，请填写不含 {remote}/ 的分支名：{branch}", "Do not prefix the branch name with {remote}/: {branch}");
pub const INVALID_REMOTE_BRANCH_NAME: UiMessage = UiMessage::new("workflow.remote_branch_guard.invalid_remote_branch_name", "远端分支名无效：{branch}", "Invalid remote branch name: {branch}");
pub const REMOTE_BRANCH_ALREADY_EXISTS: UiMessage = UiMessage::new("workflow.remote_branch_guard.remote_branch_already_exists", "远端分支已存在：{remote}/{branch}", "Remote branch already exists: {remote}/{branch}");
pub const REMOTE_BRANCH_DOES_NOT_EXIST: UiMessage = UiMessage::new("workflow.remote_branch_guard.remote_branch_does_not_exist", "远端分支不存在：{remote}/{branch}", "Remote branch does not exist: {remote}/{branch}");
pub const CHECK_REMOTE_BRANCH: UiMessage = UiMessage::new("workflow.remote_branch_guard.check_remote_branch", "检查远端分支 {remote}/{branch}（{refresh}，{exists}，{missing}）", "Check remote branch {remote}/{branch} ({refresh}, {exists}, {missing})");
pub const REMOTE_BRANCH_NAME_CANNOT_BE_EMPTY: UiMessage = UiMessage::new("workflow.remote_branch_guard.remote_branch_name_cannot_be_empty", "远端分支名不能为空", "Remote branch name cannot be empty");
pub const AFTER_FETCHING: UiMessage = UiMessage::new("workflow.remote_branch_guard.after_fetching", "刷新后", "after fetching");
pub const USING_LOCAL_REFERENCES: UiMessage = UiMessage::new("workflow.remote_branch_guard.using_local_references", "基于本地引用", "using local references");
pub const STOP_IF_PRESENT: UiMessage = UiMessage::new("workflow.remote_branch_guard.stop_if_present", "存在则停止", "stop if present");
pub const CONTINUE_IF_PRESENT: UiMessage = UiMessage::new("workflow.remote_branch_guard.continue_if_present", "存在则继续", "continue if present");
pub const STOP_IF_MISSING: UiMessage = UiMessage::new("workflow.remote_branch_guard.stop_if_missing", "不存在则停止", "stop if missing");
pub const CONTINUE_IF_MISSING: UiMessage = UiMessage::new("workflow.remote_branch_guard.continue_if_missing", "不存在则继续", "continue if missing");
