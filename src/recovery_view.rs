//! 历史恢复工作台：日志读取、提交预览和明确确认后的引用创建各自维护任务身份。

mod actions;
mod state;
mod view;

pub(crate) use state::{RecoveryEvent, RecoveryState};

#[cfg(test)]
#[path = "tests/recovery_view.rs"]
mod tests;
