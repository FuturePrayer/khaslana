//! 恢复只新增一个引用；对象读取和写后核验都不依赖历史图谱或工作区快照。

use crate::types::domain_ui_resources;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use git2::{ErrorCode, ObjectType, Oid, Repository};

use super::{GitService, validate_branch_name};
use crate::types::{
    CreateRecoveryBranchOutcome, CreateRecoveryBranchRequest, GitError, RecoveryCommitDetails,
    RecoveryObjectState, RecoveryRepoIdentity, Result,
};

const FULL_OID_HEX_LEN: usize = 40;
const HEAD_SYMBOLIC_DEPTH_LIMIT: usize = 32;

impl GitService {
    pub fn recovery_repo_identity(&self, repo: &Repository) -> Result<RecoveryRepoIdentity> {
        if repo.is_bare() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::LOCAL_RECOVERY_DOES_NOT_SUPPORT_BARE_REPOSITORIES, &[])));
        }
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::LOCAL_RECOVERY_DOES_NOT_SUPPORT_REPOSITORIES_WITHOUT_A_WORKING_TREE, &[])))?;
        Ok(RecoveryRepoIdentity {
            worktree_path: fs::canonicalize(workdir)?,
            git_dir: fs::canonicalize(repo.path())?,
            common_dir: fs::canonicalize(repo.commondir())?,
        })
    }

    pub fn recovery_commit_details(
        &self,
        repo: &Repository,
        oid: &str,
    ) -> Result<RecoveryCommitDetails> {
        let oid = parse_full_recovery_oid(oid)?;
        let mut details = RecoveryCommitDetails {
            oid: oid.to_string(),
            state: RecoveryObjectState::Zero,
            commit: None,
        };
        if oid.is_zero() {
            return Ok(details);
        }

        // 只按完整 OID 读取对象，不能把不可达或未加载进图谱的提交误判为缺失。
        match repo.find_object(oid, None) {
            Ok(object) if object.kind() != Some(ObjectType::Commit) => {
                details.state = RecoveryObjectState::NonCommit;
                return Ok(details);
            }
            Ok(_) => {}
            Err(error) => {
                details.state = recovery_object_error_state(error);
                return Ok(details);
            }
        }
        match self.collect_commit_infos(repo, [Ok(oid)], &BTreeMap::new()) {
            Ok(mut commits) => {
                details.commit = commits.pop();
                details.state = if details.commit.is_some() {
                    RecoveryObjectState::Available
                } else {
                    RecoveryObjectState::Unreadable(crate::ui_i18n::text(domain_ui_resources::COMMIT_DETAILS_ARE_EMPTY).into())
                };
            }
            Err(GitError::Git(error)) => {
                details.state = recovery_object_error_state(error);
            }
            Err(error) => details.state = RecoveryObjectState::Unreadable(error.to_string()),
        }
        Ok(details)
    }

    pub fn validate_recovery_branch_name(&self, name: &str) -> Result<()> {
        validate_branch_name(name)
    }

    pub fn create_recovery_branch(
        &self,
        repo: &Repository,
        request: &CreateRecoveryBranchRequest,
    ) -> Result<CreateRecoveryBranchOutcome> {
        self.create_recovery_branch_with_verifier(repo, request, read_recovery_reference)
    }

    fn create_recovery_branch_with_verifier<F>(
        &self,
        repo: &Repository,
        request: &CreateRecoveryBranchRequest,
        read_back: F,
    ) -> Result<CreateRecoveryBranchOutcome>
    where
        F: FnOnce(&Repository, &str) -> std::result::Result<Option<Oid>, git2::Error>,
    {
        let identity = self.recovery_repo_identity(repo)?;
        let write_lock = recovery_write_lock(&identity.common_dir);
        let _guard = write_lock
            .lock()
            .map_err(|_| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_RECOVERY_WRITE_LOCK_IS_INVALID_REOPEN_THE_REPOSITORY_AND_RETRY, &[])))?;

        // 等待同仓库其它恢复写入后重新检查，不能信任弹窗打开时的身份和对象状态。
        let identity = self.recovery_repo_identity(repo)?;
        if identity != request.identity {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_REPOSITORY_IDENTITY_HAS_CHANGED_SINCE_THE_RECOVERY_REQUEST_SELECT_THE_REPOSITORY, &[])));
        }
        let oid = parse_full_recovery_oid(&request.target_oid)?;
        if oid.is_zero() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::A_ZERO_OID_IS_NOT_A_RECOVERABLE_COMMIT_OBJECT, &[])));
        }
        let object = repo
            .find_object(oid, None)
            .map_err(|error| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CANNOT_READ_THE_RECOVERY_TARGET_COMMIT, &[("error", &format!("{}", error))])))?;
        if object.kind() != Some(ObjectType::Commit) {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_RECOVERY_TARGET_IS_NOT_A_COMMIT_OBJECT, &[])));
        }
        self.validate_recovery_branch_name(&request.branch_name)?;
        let fullname = format!("refs/heads/{}", request.branch_name);
        guard_recovery_heads(repo, &identity, &fullname)?;

        // 零期望 OID 必须交给 ref 后端在锁内检查；普通 branch(false) 的锁前检查有竞态。
        // 已存在的 loose/packed/symbolic ref、目录冲突和锁文件均由后端拒绝，绝不覆盖。
        repo.reference_matching(
            &fullname,
            oid,
            false,
            Oid::ZERO_SHA1,
            "本地恢复：从指定提交创建分支",
        )?;

        // 到此引用已经创建成功。核验失败只能报告警告，不能让调用方误以为没有创建，
        // 也不能删除或回滚可能已由另一个进程更新的引用。
        let verification =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read_back(repo, &fullname)));
        let verification_warning = match verification {
            Ok(Ok(Some(actual))) if actual == oid => None,
            Ok(Ok(Some(actual))) => Some(crate::ui_i18n::format(domain_ui_resources::THE_BRANCH_WAS_CREATED_BUT_ITS_TARGET_IS_INSTEAD_OF_THE_REQUESTED_CHECK_THE_REFERENCE, &[("actual", &format!("{}", actual)), ("oid", &format!("{}", oid))])),
            Ok(Ok(None)) => Some(crate::ui_i18n::text(domain_ui_resources::THE_BRANCH_WAS_CREATED_BUT_THE_REFERENCE_READ_BACK_IS_NOT_DIRECT_CHECK_THE_REFERENCE).into()),
            Ok(Err(error)) => Some(crate::ui_i18n::format(domain_ui_resources::THE_BRANCH_WAS_CREATED_BUT_READ_BACK_VERIFICATION_FAILED, &[("error", &format!("{}", error))])),
            Err(_) => Some(crate::ui_i18n::text(domain_ui_resources::THE_BRANCH_WAS_CREATED_BUT_READ_BACK_VERIFICATION_WAS_INTERRUPTED_CHECK_THE_REFERENCE).into()),
        };
        Ok(CreateRecoveryBranchOutcome {
            branch_name: request.branch_name.clone(),
            target_oid: oid.to_string(),
            verification_warning,
        })
    }
}

fn parse_full_recovery_oid(value: &str) -> Result<Oid> {
    // 不 trim、不 revparse，也不接受前缀；日志中的完整对象身份才是恢复目标。
    if value.len() != FULL_OID_HEX_LEN || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_RECOVERY_TARGET_MUST_BE_A_FULL_40_DIGIT_HEXADECIMAL_OID, &[])));
    }
    Ok(Oid::from_str(value)?)
}

fn recovery_object_error_state(error: git2::Error) -> RecoveryObjectState {
    if error.code() == ErrorCode::NotFound {
        RecoveryObjectState::Missing
    } else {
        RecoveryObjectState::Unreadable(error.to_string())
    }
}

fn read_recovery_reference(
    repo: &Repository,
    fullname: &str,
) -> std::result::Result<Option<Oid>, git2::Error> {
    repo.find_reference(fullname)
        .map(|reference| reference.target())
}

fn recovery_write_lock(common_dir: &Path) -> Arc<Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<()>>>>> = OnceLock::new();
    let mut locks = LOCKS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    locks.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = locks.get(common_dir).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(Mutex::new(()));
    locks.insert(common_dir.to_path_buf(), Arc::downgrade(&lock));
    lock
}

fn guard_recovery_heads(
    repo: &Repository,
    identity: &RecoveryRepoIdentity,
    fullname: &str,
) -> Result<()> {
    guard_recovery_head_chain(repo, fullname)?;
    let mut checked = HashSet::from([identity.git_dir.clone()]);
    if checked.insert(identity.common_dir.clone()) {
        guard_other_recovery_head(&identity.common_dir, fullname)?;
    }

    // 直接枚举公共目录里的登记信息，不能漏掉工作目录暂时不可访问的已知工作树。
    // 无法打开登记的仓库时拒绝写入，避免静默跳过其 unborn HEAD。
    let worktrees = match fs::read_dir(identity.common_dir.join("worktrees")) {
        Ok(worktrees) => worktrees,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    for entry in worktrees {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let git_dir = fs::canonicalize(entry.path())?;
        if checked.insert(git_dir.clone()) {
            guard_other_recovery_head(&git_dir, fullname)?;
        }
    }
    Ok(())
}

fn guard_other_recovery_head(git_dir: &Path, fullname: &str) -> Result<()> {
    let other = Repository::open(git_dir).map_err(|error| {
        GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CANNOT_INSPECT_HEAD_OF_A_KNOWN_WORKING_TREE, &[("value0", &format!("{}", git_dir.display())), ("error", &format!("{}", error))]))
    })?;
    guard_recovery_head_chain(&other, fullname)
}

fn guard_recovery_head_chain(repo: &Repository, fullname: &str) -> Result<()> {
    let mut name = "HEAD".to_string();
    let mut visited = HashSet::new();
    for _ in 0..HEAD_SYMBOLIC_DEPTH_LIMIT {
        if name == fullname {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::HEAD_OF_THIS_OR_ANOTHER_WORKING_TREE_POINTS_TO_THIS_BRANCH_RECOVERY_WAS_REFUSED_BECAU, &[])));
        }
        if !visited.insert(name.clone()) {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_HEAD_SYMBOLIC_REFERENCE_CHAIN_CONTAINS_A_CYCLE_CANNOT_SAFELY_CREATE_THE_RECOVERY, &[])));
        }
        let reference = match repo.find_reference(&name) {
            Ok(reference) => reference,
            Err(error) if error.code() == ErrorCode::NotFound && name != "HEAD" => {
                return Ok(());
            }
            Err(error) => {
                return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CANNOT_INSPECT_WORKING_TREE_HEAD, &[("error", &format!("{}", error))])));
            }
        };
        match reference.symbolic_target()? {
            Some(target) => name = target.to_string(),
            None => return Ok(()),
        }
    }
    Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_HEAD_SYMBOLIC_REFERENCE_CHAIN_IS_TOO_LONG_CANNOT_SAFELY_CREATE_THE_RECOVERY_BRANC, &[])))
}

#[cfg(test)]
#[path = "../tests/git/recovery.rs"]
mod tests;
