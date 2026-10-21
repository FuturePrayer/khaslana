#[path = "domain_ui_resources.rs"]
pub mod domain_ui_resources;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};


mod blame;
mod browse;
mod conflicts;
mod recovery;

pub use recovery::{
    CreateRecoveryBranchOutcome, CreateRecoveryBranchRequest, RecoveryCommitDetails,
    RecoveryObjectState, RecoveryRepoIdentity, ReflogEntry, ReflogPage, ReflogQuery,
    ReflogScope, ReflogSnapshot, ReflogSources,
};

pub use blame::{BlameCommitInfo, BlameHunkInfo, BlameView};

pub use conflicts::{
    ConflictBlock, ConflictBlockResolution, ConflictBlockStatus, ConflictDraftStatus,
    ConflictFileKind, ConflictFileView, ConflictResolutionSide,
};

pub use browse::{
    BrowseCompareFile, BrowseEntry, BrowseEntryKind, BrowseFileContent, BrowseListMode,
    BrowseTarget,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RepoPath(pub PathBuf);

impl RepoPath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BranchName(pub String);

impl BranchName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RemoteName(pub String);

impl RemoteName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl Default for RemoteName {
    fn default() -> Self {
        Self("origin".to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagName(pub String);

impl TagName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommitMessage(pub String);

impl CommitMessage {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchKind {
    Local,
    Remote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchInfo {
    pub name: String,
    pub kind: BranchKind,
    pub is_head: bool,
    pub upstream: Option<String>,
    /// 相对 upstream 需要推送的提交数；未关联 upstream 时为 None。
    pub ahead: Option<usize>,
    /// 相对 upstream 需要拉取的提交数；未关联 upstream 时为 None。
    pub behind: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagInfo {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    pub credential_record_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CloneOptions {
    pub recursive_submodules: bool,
}

impl Default for CloneOptions {
    fn default() -> Self {
        Self {
            recursive_submodules: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmoduleInfo {
    pub name: String,
    pub path: PathBuf,
    pub url: Option<String>,
    pub branch: Option<String>,
    pub head_id: Option<String>,
    pub index_id: Option<String>,
    pub workdir_id: Option<String>,
    pub status: SubmoduleState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmoduleRemoteSyncStatus {
    Unknown,
    Checking,
    UpToDate,
    Behind(usize),
    Ahead(usize),
    Diverged { ahead: usize, behind: usize },
    Unavailable(String),
}

impl SubmoduleRemoteSyncStatus {
    pub fn from_ahead_behind(ahead: usize, behind: usize) -> Self {
        match (ahead, behind) {
            (0, 0) => Self::UpToDate,
            (0, behind) => Self::Behind(behind),
            (ahead, 0) => Self::Ahead(ahead),
            (ahead, behind) => Self::Diverged { ahead, behind },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubmoduleState {
    pub initialized: bool,
    pub checked_out: bool,
    pub head_matches_index: bool,
    pub workdir_modified: bool,
    pub workdir_untracked: bool,
}

impl SubmoduleState {
    pub fn is_ready(&self) -> bool {
        self.initialized
            && self.checked_out
            && self.head_matches_index
            && !self.workdir_modified
            && !self.workdir_untracked
    }

    pub fn label(&self) -> &'static str {
        if !self.initialized {
            crate::ui_i18n::text(domain_ui_resources::NOT_INITIALIZED)
        } else if !self.checked_out {
            crate::ui_i18n::text(domain_ui_resources::NOT_CHECKED_OUT)
        } else if self.workdir_modified {
            crate::ui_i18n::text(domain_ui_resources::MODIFIED)
        } else if self.workdir_untracked {
            crate::ui_i18n::text(domain_ui_resources::HAS_UNTRACKED_FILES)
        } else if !self.head_matches_index {
            crate::ui_i18n::text(domain_ui_resources::UPDATE_NEEDED)
        } else {
            crate::ui_i18n::text(domain_ui_resources::UP_TO_DATE)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StashInfo {
    pub index: usize,
    pub message: String,
    pub oid: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StashFileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub status: ChangeState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeState {
    Added,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Conflicted,
    Untracked,
}

impl ChangeState {
    pub fn label(&self) -> &'static str {
        match self {
            ChangeState::Added => "A",
            ChangeState::Modified => "M",
            ChangeState::Deleted => "D",
            ChangeState::Renamed => "R",
            ChangeState::Typechange => "T",
            ChangeState::Conflicted => "!",
            ChangeState::Untracked => "?",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeChange {
    pub path: String,
    pub staged: Option<ChangeState>,
    pub unstaged: Option<ChangeState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DiffScope {
    Staged,
    Unstaged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
    Header,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiffEncodingChoice {
    #[default]
    Auto,
    Utf8,
    Gb18030,
    Big5,
}

impl DiffEncodingChoice {
    pub fn label(self) -> &'static str {
        match self {
            DiffEncodingChoice::Auto => crate::ui_i18n::text(domain_ui_resources::AUTO),
            DiffEncodingChoice::Utf8 => "UTF-8",
            DiffEncodingChoice::Gb18030 => "GB18030/GBK",
            DiffEncodingChoice::Big5 => "Big5",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiffEncodingInfo {
    pub requested: DiffEncodingChoice,
    pub resolved: DiffEncodingChoice,
    pub lossy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
    /// 所属 hunk 序号：文件头行为 0，首个 `@@` hunk 头之后为 1 并递增。
    /// 供按块暂存等 hunk 级交互定位行分组。
    pub hunk_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDiff {
    pub path: String,
    pub scope: DiffScope,
    pub is_binary: bool,
    /// 未跟踪文件（git 状态 ？）：内容整体展示但不按「新增」绿色高亮，
    /// 渲染侧白底显示（SourceTree 式），行 kind 仍为 Added。
    pub untracked: bool,
    /// 旧侧文件大小（字节）；新增文件（Added delta）为 None。
    pub old_size: Option<u64>,
    /// 新侧文件大小（字节）；删除文件（Deleted delta）为 None。
    pub new_size: Option<u64>,
    pub encoding: DiffEncodingInfo,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitInfo {
    pub oid: String,
    pub short_oid: String,
    /// 仅首行摘要（列表行展示）。
    pub summary: String,
    /// 完整提交信息（含多行 body，供提交详情区展示）。
    pub message: String,
    pub author: String,
    pub author_email: Option<String>,
    /// 提交者（rebase/cherry-pick 后可能与作者不同，此时详情区单独展示）。
    pub committer: String,
    pub committer_email: Option<String>,
    pub time: i64,
    pub parents: Vec<String>,
    pub refs: Vec<CommitRefInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitRefInfo {
    pub name: String,
    pub kind: CommitRefKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BranchSyncStatus {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub unpushed_oids: Vec<String>,
    pub unpushed_oids_truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommitRefKind {
    LocalBranch,
    RemoteBranch,
    Tag,
    Head,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistoryScope {
    #[default]
    CurrentBranch,
    AllRefs,
}

impl HistoryScope {
    pub fn label(self) -> &'static str {
        match self {
            Self::CurrentBranch => crate::ui_i18n::text(domain_ui_resources::CURRENT_BRANCH),
            Self::AllRefs => crate::ui_i18n::text(domain_ui_resources::ALL_BRANCHES),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommitFileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub status: ChangeState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RepositorySnapshot {
    pub path: PathBuf,
    pub head: Option<String>,
    pub branches: Vec<BranchInfo>,
    pub changes: Vec<WorktreeChange>,
    pub remotes: Vec<RemoteInfo>,
    pub tags: Vec<TagInfo>,
    pub stashes: Vec<StashInfo>,
    pub conflicts: Vec<String>,
    /// 是否有普通合并正在进行（由 libgit2 的 RepositoryState::Merge 判定）。
    pub merge_in_progress: bool,
    /// 合并提交的默认信息；合并冲突解决后允许用户在提交框中修改。
    pub merge_message: Option<String>,
    /// 是否有变基正在进行（检测 .git/rebase-merge 或 rebase-apply 目录）。
    pub rebase_in_progress: bool,
}

/// 变基操作结果：要么全部完成，要么暂停在冲突处等待用户解决。
#[derive(Clone, Debug)]
pub enum RebaseOutcome {
    /// 变基全部完成，无冲突。
    Completed(RepositorySnapshot),
    /// 变基暂停：当前提交重放产生冲突，需要用户解决后继续。
    Conflicts {
        snapshot: RepositorySnapshot,
        /// 当前正在重放的提交序号（1-based）。
        current: usize,
        /// 待重放的提交总数。
        total: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationEvent {
    Started(String),
    Progress(String),
    Finished(String),
}

#[derive(Debug)]
pub enum GitError {
    Git(git2::Error),
    Io(std::io::Error),
    Credential(String),
    CredentialRequired { url: String },
    InvalidBranchName(String),
    EmptyCommitMessage,
    EmptyCommit,
    NoRepository,
    Conflicts(Vec<String>),
    WorkflowCancelled,
    Message(String),
    Localized(crate::ui_i18n::LocalizedMessage),
    LocalizedCredential(crate::ui_i18n::LocalizedMessage),
}

impl GitError {
    /// 错误创建后保留稳定身份，显示时才读取语言；不从译文反推业务分支。
    pub fn message_key(&self) -> Option<&'static str> {
        use domain_ui_resources as r;
        Some(match self {
            Self::Localized(message) | Self::LocalizedCredential(message) => message.message.key,
            Self::EmptyCommitMessage => r::EMPTY_COMMIT_MESSAGE.key,
            Self::EmptyCommit => r::EMPTY_COMMIT.key,
            Self::NoRepository => r::NO_REPOSITORY.key,
            Self::WorkflowCancelled => r::WORKFLOW_CANCELLED.key,
            Self::CredentialRequired { .. } => r::AUTHENTICATION_IS_REQUIRED_TO_ACCESS.key,
            Self::InvalidBranchName(_) => r::INVALID_BRANCH_NAME.key,
            Self::Conflicts(_) => r::OPERATION_CONFLICTS.key,
            Self::Git(_) | Self::Io(_) | Self::Credential(_) | Self::Message(_) => return None,
        })
    }

    pub fn is_message(&self, key: &str) -> bool { self.message_key() == Some(key) }

    /// 显式语言入口供无全局状态竞态的测试、导出和长驻错误展示复用。
    pub fn display_in(&self, language: crate::language::Language) -> String {
        use crate::ui_i18n::{format_in, text_in};
        use domain_ui_resources as r;
        let wrap = |message, detail: String| format_in(language, message, &[("detail", &detail)]);
        let localized = |message: &crate::ui_i18n::LocalizedMessage| {
            let args: Vec<_> = message.args.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect();
            format_in(language, message.message, &args)
        };
        match self {
            Self::Git(error) => wrap(r::GIT_ERROR, crate::ui_i18n::resolve_in(language, &error.to_string())),
            Self::Io(error) => wrap(r::IO_ERROR, error.to_string()),
            Self::Credential(detail) => wrap(r::CREDENTIAL_ERROR, detail.clone()),
            Self::LocalizedCredential(message) => wrap(r::CREDENTIAL_ERROR, localized(message)),
            Self::CredentialRequired { url } => format_in(language, r::AUTHENTICATION_IS_REQUIRED_TO_ACCESS, &[("url", url)]),
            Self::InvalidBranchName(name) => wrap(r::INVALID_BRANCH_NAME, name.clone()),
            Self::EmptyCommitMessage => text_in(language, r::EMPTY_COMMIT_MESSAGE).into(),
            Self::EmptyCommit => text_in(language, r::EMPTY_COMMIT).into(),
            Self::NoRepository => text_in(language, r::NO_REPOSITORY).into(),
            Self::Conflicts(paths) => wrap(r::OPERATION_CONFLICTS, format!("{paths:?}")),
            Self::WorkflowCancelled => text_in(language, r::WORKFLOW_CANCELLED).into(),
            Self::Message(message) => message.clone(),
            Self::Localized(message) => localized(message),
        }
    }
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_in(crate::ui_i18n::language()))
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self { Self::Git(error) => Some(error), Self::Io(error) => Some(error), _ => None }
    }
}
impl From<git2::Error> for GitError { fn from(error: git2::Error) -> Self { Self::Git(error) } }
impl From<std::io::Error> for GitError { fn from(error: std::io::Error) -> Self { Self::Io(error) } }

pub type Result<T> = std::result::Result<T, GitError>;
