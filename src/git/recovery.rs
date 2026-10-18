//! 恢复只新增一个引用；对象读取和写后核验都不依赖历史图谱或工作区快照。

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
            return Err(GitError::Message("本地恢复暂不支持裸仓库".into()));
        }
        let workdir = repo
            .workdir()
            .ok_or_else(|| GitError::Message("本地恢复暂不支持没有工作树的仓库".into()))?;
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
                    RecoveryObjectState::Unreadable("提交详情为空".into())
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
            .map_err(|_| GitError::Message("恢复写入锁异常，请重新打开仓库后重试".into()))?;

        // 等待同仓库其它恢复写入后重新检查，不能信任弹窗打开时的身份和对象状态。
        let identity = self.recovery_repo_identity(repo)?;
        if identity != request.identity {
            return Err(GitError::Message(
                "恢复请求的仓库身份已变化，请重新选择仓库与提交".into(),
            ));
        }
        let oid = parse_full_recovery_oid(&request.target_oid)?;
        if oid.is_zero() {
            return Err(GitError::Message("零 OID 不是可恢复的提交对象".into()));
        }
        let object = repo
            .find_object(oid, None)
            .map_err(|error| GitError::Message(format!("无法读取恢复目标提交：{error}")))?;
        if object.kind() != Some(ObjectType::Commit) {
            return Err(GitError::Message("恢复目标不是提交对象".into()));
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
            Ok(Ok(Some(actual))) => Some(format!(
                "分支已创建，但读回目标为 {actual}，与请求的 {oid} 不一致，请检查引用"
            )),
            Ok(Ok(None)) => Some("分支已创建，但读回的引用不是直接引用，请检查引用".into()),
            Ok(Err(error)) => Some(format!("分支已创建，但无法读回验证：{error}")),
            Err(_) => Some("分支已创建，但读回验证意外中断，请检查引用".into()),
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
        return Err(GitError::Message(
            "恢复目标必须是完整的 40 位十六进制 OID".into(),
        ));
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
        GitError::Message(format!(
            "无法检查已知工作树的 HEAD（{}）：{error}",
            git_dir.display()
        ))
    })?;
    guard_recovery_head_chain(&other, fullname)
}

fn guard_recovery_head_chain(repo: &Repository, fullname: &str) -> Result<()> {
    let mut name = "HEAD".to_string();
    let mut visited = HashSet::new();
    for _ in 0..HEAD_SYMBOLIC_DEPTH_LIMIT {
        if name == fullname {
            return Err(GitError::Message(
                "该分支正由当前或其他工作树的 HEAD 指向，创建会改变 HEAD 解析，已拒绝恢复".into(),
            ));
        }
        if !visited.insert(name.clone()) {
            return Err(GitError::Message(
                "HEAD 符号引用存在循环，无法安全创建恢复分支".into(),
            ));
        }
        let reference = match repo.find_reference(&name) {
            Ok(reference) => reference,
            Err(error) if error.code() == ErrorCode::NotFound && name != "HEAD" => {
                return Ok(());
            }
            Err(error) => {
                return Err(GitError::Message(format!("无法检查工作树 HEAD：{error}")));
            }
        };
        match reference.symbolic_target()? {
            Some(target) => name = target.to_string(),
            None => return Ok(()),
        }
    }
    Err(GitError::Message(
        "HEAD 符号引用链过长，无法安全创建恢复分支".into(),
    ))
}

#[cfg(test)]
#[path = "../tests/git/recovery.rs"]
mod tests;
