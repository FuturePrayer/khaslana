//! 本地恢复领域对象：日志证据与引用创建结果分开，绝不把界面序号当作恢复目标。

use std::path::PathBuf;

use super::CommitInfo;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RecoveryRepoIdentity {
    pub worktree_path: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ReflogScope {
    #[default]
    Head,
    CurrentBranch,
    LocalBranch(String),
    All,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogSources {
    pub identity: RecoveryRepoIdentity,
    pub current_branch: Option<String>,
    pub branches: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogEntry {
    /// 只在同一不可变快照内标识源记录，不能代替完整 OID。
    pub id: String,
    pub ref_name: String,
    pub old_oid: String,
    pub new_oid: String,
    pub actor: String,
    pub actor_email: String,
    pub timestamp: i64,
    pub timezone_offset: i32,
    pub message: String,
    pub lossy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogSnapshot {
    pub identity: RecoveryRepoIdentity,
    pub entries: Vec<ReflogEntry>,
    pub diagnostics: Vec<String>,
    pub sources_read: usize,
    pub sources_total: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReflogQuery {
    pub search: String,
    pub since: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflogPage {
    pub entries: Vec<ReflogEntry>,
    pub total: usize,
    pub next_offset: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryObjectState {
    Available,
    Zero,
    Missing,
    NonCommit,
    Unreadable(String),
}

impl RecoveryObjectState {
    pub fn label(&self) -> &str {
        match self {
            Self::Available => "提交对象可用",
            Self::Zero => "此侧没有提交",
            Self::Missing => "当前本地提交对象不可用",
            Self::NonCommit => "该对象不是提交",
            Self::Unreadable(_) => "提交对象读取失败",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryCommitDetails {
    pub oid: String,
    pub state: RecoveryObjectState,
    pub commit: Option<CommitInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateRecoveryBranchRequest {
    pub identity: RecoveryRepoIdentity,
    pub target_oid: String,
    pub branch_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateRecoveryBranchOutcome {
    pub branch_name: String,
    pub target_oid: String,
    /// 原子创建已经成功，后续验证异常不能被报告为“未创建”。
    pub verification_warning: Option<String>,
}
