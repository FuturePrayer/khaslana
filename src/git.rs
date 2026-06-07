use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{
    AnnotatedCommit, BranchType, Cred, CredentialType, Delta, DiffFormat, DiffOptions, ErrorCode,
    FetchOptions, IndexAddOption, MergeAnalysis, MergeOptions, PushOptions, Reference,
    RemoteCallbacks, Repository, ResetType, Signature, Sort, StashApplyOptions, Status,
    StatusOptions,
};

use crate::credentials::{CredentialProvider, CredentialRequest, to_git_credential};
use crate::types::{
    BranchInfo, BranchKind, BranchName, ChangeState, CommitFileChange, CommitInfo, CommitMessage,
    CommitRefInfo, CommitRefKind, DiffLine, DiffLineKind, DiffScope, FileDiff, GitError,
    OperationEvent, RemoteName, RepoPath, RepositorySnapshot, ResetMode, Result, StashInfo,
    TagInfo, TagName, WorktreeChange,
};

const DIFF_CONTEXT_LINES: u32 = 3;

pub trait ProgressEmitter: Send + Sync {
    fn emit(&self, event: OperationEvent);
}

#[derive(Clone, Default)]
pub struct NoopProgress;

impl ProgressEmitter for NoopProgress {
    fn emit(&self, _event: OperationEvent) {}
}

#[derive(Clone)]
pub struct GitService {
    credential_provider: Arc<dyn CredentialProvider>,
    progress: Arc<dyn ProgressEmitter>,
}

impl GitService {
    pub fn new(
        credential_provider: Arc<dyn CredentialProvider>,
        progress: Arc<dyn ProgressEmitter>,
    ) -> Self {
        Self {
            credential_provider,
            progress,
        }
    }

    pub fn open(&self, path: &RepoPath) -> Result<RepositorySnapshot> {
        let mut repo = Repository::open(&path.0)?;
        self.snapshot(&mut repo)
    }

    pub fn open_fast(&self, path: &RepoPath) -> Result<RepositorySnapshot> {
        let repo = Repository::open(&path.0)?;
        self.fast_snapshot(&repo)
    }

    pub fn clone_repo(&self, url: &str, into: &RepoPath) -> Result<RepositorySnapshot> {
        self.progress
            .emit(OperationEvent::Started(format!("正在克隆 {url}")));

        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(self.remote_callbacks(None));

        let mut checkout = CheckoutBuilder::new();
        checkout.progress(|path, current, total| {
            if let Some(path) = path {
                tracing::debug!("checkout {current}/{total}: {}", path.display());
            }
        });

        let mut repo = RepoBuilder::new()
            .fetch_options(fetch_options)
            .with_checkout(checkout)
            .clone(url, &into.0)?;

        self.progress
            .emit(OperationEvent::Finished(format!("已克隆 {url}")));
        self.snapshot(&mut repo)
    }

    pub fn snapshot(&self, repo: &mut Repository) -> Result<RepositorySnapshot> {
        self.snapshot_details(repo)
    }

    pub fn fast_snapshot(&self, repo: &Repository) -> Result<RepositorySnapshot> {
        Ok(RepositorySnapshot {
            path: repo
                .workdir()
                .or_else(|| repo.path().parent())
                .unwrap_or_else(|| repo.path())
                .to_path_buf(),
            head: self.head_name(repo),
            branches: self.local_branches(repo)?,
            changes: Vec::new(),
            remotes: Vec::new(),
            tags: Vec::new(),
            stashes: Vec::new(),
            conflicts: Vec::new(),
        })
    }

    pub fn snapshot_details(&self, repo: &mut Repository) -> Result<RepositorySnapshot> {
        Ok(RepositorySnapshot {
            path: repo
                .workdir()
                .or_else(|| repo.path().parent())
                .unwrap_or_else(|| repo.path())
                .to_path_buf(),
            head: self.head_name(repo),
            branches: self.branches(repo)?,
            changes: self.status(repo)?,
            remotes: self.remotes(repo)?,
            tags: self.tags(repo)?,
            stashes: self.stashes(repo)?,
            conflicts: self.conflicts(repo)?,
        })
    }

    pub fn snapshot_metadata(&self, repo: &mut Repository) -> Result<RepositorySnapshot> {
        Ok(RepositorySnapshot {
            path: repo
                .workdir()
                .or_else(|| repo.path().parent())
                .unwrap_or_else(|| repo.path())
                .to_path_buf(),
            head: self.head_name(repo),
            branches: self.branches(repo)?,
            changes: Vec::new(),
            remotes: self.remotes(repo)?,
            tags: self.tags(repo)?,
            stashes: self.stashes(repo)?,
            conflicts: self.conflicts(repo)?,
        })
    }

    pub fn local_branches(&self, repo: &Repository) -> Result<Vec<BranchInfo>> {
        self.branches_by_type(repo, Some(BranchType::Local))
    }

    pub fn branches(&self, repo: &Repository) -> Result<Vec<BranchInfo>> {
        self.branches_by_type(repo, None)
    }

    fn branches_by_type(
        &self,
        repo: &Repository,
        branch_filter: Option<BranchType>,
    ) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();

        for branch in repo.branches(branch_filter)? {
            let (branch, branch_type) = branch?;
            let Some(name) = branch.name()? else {
                continue;
            };
            let upstream = if branch_type == BranchType::Local {
                branch
                    .upstream()
                    .ok()
                    .and_then(|upstream| upstream.name().ok().flatten().map(str::to_string))
            } else {
                None
            };
            branches.push(BranchInfo {
                name: name.to_string(),
                kind: match branch_type {
                    BranchType::Local => BranchKind::Local,
                    BranchType::Remote => BranchKind::Remote,
                },
                is_head: branch.is_head(),
                upstream,
            });
        }

        branches.sort_by(|a, b| {
            let kind = match (&a.kind, &b.kind) {
                (BranchKind::Local, BranchKind::Remote) => std::cmp::Ordering::Less,
                (BranchKind::Remote, BranchKind::Local) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };
            kind.then_with(|| a.name.cmp(&b.name))
        });
        Ok(branches)
    }

    pub fn status(&self, repo: &Repository) -> Result<Vec<WorktreeChange>> {
        self.status_full(repo)
    }

    pub fn status_fast(&self, repo: &Repository) -> Result<Vec<WorktreeChange>> {
        self.status_with_options(repo, false, false)
    }

    pub fn status_full(&self, repo: &Repository) -> Result<Vec<WorktreeChange>> {
        self.status_with_options(repo, true, true)
    }

    fn status_with_options(
        &self,
        repo: &Repository,
        include_untracked: bool,
        recurse_untracked_dirs: bool,
    ) -> Result<Vec<WorktreeChange>> {
        let mut options = StatusOptions::new();
        options
            .include_untracked(include_untracked)
            .recurse_untracked_dirs(recurse_untracked_dirs)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = repo.statuses(Some(&mut options))?;
        let mut changes = BTreeMap::<String, WorktreeChange>::new();

        for entry in statuses.iter() {
            let path = entry.path()?.to_string();
            let status = entry.status();
            let change = changes
                .entry(path.clone())
                .or_insert_with(|| WorktreeChange {
                    path,
                    staged: None,
                    unstaged: None,
                });

            if let Some(state) = staged_state(status) {
                change.staged = Some(state);
            }
            if let Some(state) = unstaged_state(status) {
                change.unstaged = Some(state);
            }
        }

        Ok(changes.into_values().collect())
    }

    pub fn remotes(&self, repo: &Repository) -> Result<Vec<String>> {
        let remotes = repo.remotes()?;
        let mut names = remotes.iter().try_fold(Vec::new(), |mut names, name| {
            if let Some(name) = name? {
                names.push(name.to_string());
            }
            Ok::<_, git2::Error>(names)
        })?;
        names.sort();
        Ok(names)
    }

    pub fn tags(&self, repo: &Repository) -> Result<Vec<TagInfo>> {
        let tags = repo.tag_names(None)?;
        let mut tags = tags
            .iter()
            .flatten()
            .flatten()
            .map(|name| TagInfo {
                name: name.to_string(),
            })
            .collect::<Vec<_>>();
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tags)
    }

    pub fn stashes(&self, repo: &mut Repository) -> Result<Vec<StashInfo>> {
        let mut stashes = Vec::new();
        repo.stash_foreach(|index, message, oid| {
            stashes.push(StashInfo {
                index,
                message: message.to_string(),
                oid: oid.to_string(),
            });
            true
        })?;
        Ok(stashes)
    }

    pub fn fetch(&self, repo: &mut Repository, remote: &RemoteName) -> Result<RepositorySnapshot> {
        self.progress
            .emit(OperationEvent::Started(format!("正在获取 {}", remote.0)));
        let mut remote_handle = repo.find_remote(&remote.0)?;
        let mut options = FetchOptions::new();
        options.remote_callbacks(self.remote_callbacks(Some(repo)));
        remote_handle.fetch(&[] as &[&str], Some(&mut options), Some("khaslana fetch"))?;
        drop(remote_handle);
        drop(options);
        self.progress
            .emit(OperationEvent::Finished(format!("已获取 {}", remote.0)));
        self.snapshot(repo)
    }

    pub fn pull(&self, repo: &mut Repository, remote: &RemoteName) -> Result<RepositorySnapshot> {
        self.progress
            .emit(OperationEvent::Started(format!("正在拉取 {}", remote.0)));
        self.fetch(repo, remote)?;

        let head = repo.head()?;
        let branch = head.shorthand().map_err(GitError::from)?.to_string();
        drop(head);

        let remote_ref = self.remote_ref_for_branch(repo, remote, &branch)?;
        let annotated = repo.reference_to_annotated_commit(&remote_ref)?;
        self.merge_annotated(repo, &annotated, &format!("{}/{}", remote.0, branch))?;
        drop(annotated);
        drop(remote_ref);

        self.progress
            .emit(OperationEvent::Finished(format!("已拉取 {}", remote.0)));
        self.snapshot(repo)
    }

    pub fn push(&self, repo: &mut Repository, remote: &RemoteName) -> Result<RepositorySnapshot> {
        let head = repo.head()?;
        let branch = head.shorthand().map_err(GitError::from)?.to_string();
        drop(head);

        self.progress
            .emit(OperationEvent::Started(format!("正在推送 {branch}")));
        let mut remote_handle = repo.find_remote(&remote.0)?;
        let mut options = PushOptions::new();
        options.remote_callbacks(self.remote_callbacks(Some(repo)));
        let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
        remote_handle.push(&[refspec.as_str()], Some(&mut options))?;
        drop(remote_handle);
        drop(options);

        if let Ok(mut local) = repo.find_branch(&branch, BranchType::Local) {
            let upstream = format!("{}/{}", remote.0, branch);
            let _ = local.set_upstream(Some(&upstream));
        }

        self.progress
            .emit(OperationEvent::Finished(format!("已推送 {branch}")));
        self.snapshot(repo)
    }

    pub fn merge_branch(
        &self,
        repo: &mut Repository,
        branch: &BranchName,
    ) -> Result<RepositorySnapshot> {
        self.progress
            .emit(OperationEvent::Started(format!("正在合并 {}", branch.0)));
        let reference = self.find_branch_reference(repo, &branch.0)?;
        let annotated = repo.reference_to_annotated_commit(&reference)?;
        self.merge_annotated(repo, &annotated, &branch.0)?;
        drop(annotated);
        drop(reference);
        self.progress
            .emit(OperationEvent::Finished(format!("已合并 {}", branch.0)));
        self.snapshot(repo)
    }

    pub fn checkout_branch(
        &self,
        repo: &mut Repository,
        branch: &BranchName,
    ) -> Result<RepositorySnapshot> {
        let branch_handle = repo.find_branch(&branch.0, BranchType::Local)?;
        let reference = branch_handle.get();
        let target = reference
            .target()
            .ok_or_else(|| GitError::Message(format!("分支 {} 没有目标提交", branch.0)))?;
        let object = repo.find_object(target, None)?;

        let mut checkout = CheckoutBuilder::new();
        checkout.safe();
        repo.checkout_tree(&object, Some(&mut checkout))?;
        let refname = reference.name().map_err(GitError::from)?;
        repo.set_head(refname)?;
        drop(object);
        drop(branch_handle);
        self.snapshot(repo)
    }

    pub fn checkout_remote_branch(
        &self,
        repo: &mut Repository,
        remote_branch: &BranchName,
    ) -> Result<RepositorySnapshot> {
        let (remote, local_name) = remote_branch_name_parts(&remote_branch.0)?;
        validate_branch_name(local_name)?;

        let remote_branch_handle = repo.find_branch(&remote_branch.0, BranchType::Remote)?;
        let reference = remote_branch_handle.get();
        let target = reference.target().ok_or_else(|| {
            GitError::Message(format!("远端分支 {} 没有目标提交", remote_branch.0))
        })?;
        let commit = repo.find_commit(target)?;
        let upstream = format!("{remote}/{local_name}");

        if let Ok(mut local) = repo.find_branch(local_name, BranchType::Local) {
            local.set_upstream(Some(&upstream))?;
        } else {
            let mut local = repo.branch(local_name, &commit, false)?;
            local.set_upstream(Some(&upstream))?;
        }

        drop(commit);
        drop(remote_branch_handle);
        self.checkout_branch(repo, &BranchName::new(local_name))
            .map_err(|err| match err {
                GitError::Git(git_err) => GitError::Message(format!(
                    "无法切换到本地分支 {local_name}：{}",
                    git_err.message()
                )),
                other => other,
            })
    }

    pub fn checkout_tag(&self, repo: &mut Repository, tag: &TagName) -> Result<RepositorySnapshot> {
        let object = repo.revparse_single(&format!("refs/tags/{}", tag.0))?;
        let commit = object.peel_to_commit()?;
        let mut checkout = CheckoutBuilder::new();
        checkout.safe();
        repo.checkout_tree(commit.as_object(), Some(&mut checkout))?;
        repo.set_head_detached(commit.id())?;
        drop(commit);
        drop(object);
        self.snapshot(repo)
    }

    pub fn create_branch(
        &self,
        repo: &mut Repository,
        branch: &BranchName,
    ) -> Result<RepositorySnapshot> {
        validate_branch_name(&branch.0)?;
        let head = repo.head()?.peel_to_commit()?;
        repo.branch(&branch.0, &head, false)?;
        drop(head);
        self.snapshot(repo)
    }

    pub fn delete_branch(
        &self,
        repo: &mut Repository,
        branch: &BranchName,
    ) -> Result<RepositorySnapshot> {
        let mut branch_handle = repo.find_branch(&branch.0, BranchType::Local)?;
        branch_handle.delete()?;
        drop(branch_handle);
        self.snapshot(repo)
    }

    pub fn rename_branch(
        &self,
        repo: &mut Repository,
        old: &BranchName,
        new: &BranchName,
    ) -> Result<RepositorySnapshot> {
        validate_branch_name(&new.0)?;
        let mut branch = repo.find_branch(&old.0, BranchType::Local)?;
        branch.rename(&new.0, false)?;
        drop(branch);
        self.snapshot(repo)
    }

    pub fn stage_path(&self, repo: &mut Repository, path: &Path) -> Result<RepositorySnapshot> {
        self.stage_paths(repo, [path])
    }

    pub fn stage_paths<'a, I>(&self, repo: &mut Repository, paths: I) -> Result<RepositorySnapshot>
    where
        I: IntoIterator<Item = &'a Path>,
    {
        let mut index = repo.index()?;
        for path in paths {
            if path == Path::new(".") {
                index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
            } else if repo
                .status_file(path)
                .map(|status| status.contains(Status::WT_DELETED))
                .unwrap_or(false)
            {
                index.remove_path(path)?;
            } else {
                index.add_path(path)?;
            }
        }
        index.write()?;
        self.snapshot(repo)
    }

    pub fn unstage_path(&self, repo: &mut Repository, path: &Path) -> Result<RepositorySnapshot> {
        self.unstage_paths(repo, [path])
    }

    pub fn unstage_paths<'a, I>(
        &self,
        repo: &mut Repository,
        paths: I,
    ) -> Result<RepositorySnapshot>
    where
        I: IntoIterator<Item = &'a Path>,
    {
        let object = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
        let paths = paths.into_iter().collect::<Vec<_>>();
        repo.reset_default(object.as_ref().map(|tree| tree.as_object()), paths)?;
        drop(object);
        self.snapshot(repo)
    }

    pub fn commit(
        &self,
        repo: &mut Repository,
        message: &CommitMessage,
    ) -> Result<RepositorySnapshot> {
        let message = message.0.trim();
        if message.is_empty() {
            return Err(GitError::EmptyCommitMessage);
        }

        let mut index = repo.index()?;
        if index.has_conflicts() {
            return Err(GitError::Conflicts(self.conflicts(repo)?));
        }
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = signature(repo)?;
        let parent_commits = parents(repo)?;
        let parent_refs = parent_commits.iter().collect::<Vec<_>>();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )?;

        repo.cleanup_state()?;
        drop(tree);
        drop(parent_commits);
        self.snapshot(repo)
    }

    pub fn apply_stash(&self, repo: &mut Repository, index: usize) -> Result<RepositorySnapshot> {
        self.progress.emit(OperationEvent::Started(format!(
            "正在应用贮藏 stash@{{{index}}}"
        )));
        let mut options = StashApplyOptions::new();
        repo.stash_apply(index, Some(&mut options))?;
        self.progress.emit(OperationEvent::Finished(format!(
            "已应用贮藏 stash@{{{index}}}"
        )));
        self.snapshot(repo)
    }

    pub fn pop_stash(&self, repo: &mut Repository, index: usize) -> Result<RepositorySnapshot> {
        self.progress.emit(OperationEvent::Started(format!(
            "正在弹出贮藏 stash@{{{index}}}"
        )));
        let mut options = StashApplyOptions::new();
        repo.stash_pop(index, Some(&mut options))?;
        self.progress.emit(OperationEvent::Finished(format!(
            "已弹出贮藏 stash@{{{index}}}"
        )));
        self.snapshot(repo)
    }

    pub fn diff_for_path(
        &self,
        repo: &Repository,
        path: &Path,
        scope: DiffScope,
    ) -> Result<FileDiff> {
        let mut options = DiffOptions::new();
        options
            .pathspec(path)
            .include_untracked(true)
            .context_lines(DIFF_CONTEXT_LINES);

        let diff = match scope {
            DiffScope::Staged => {
                let head_tree = repo.head().ok().and_then(|head| head.peel_to_tree().ok());
                repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut options))?
            }
            DiffScope::Unstaged => repo.diff_index_to_workdir(None, Some(&mut options))?,
        };

        self.file_diff_from_diff(diff, path_to_git(path), scope)
    }

    pub fn commit_history(
        &self,
        repo: &Repository,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<CommitInfo>> {
        self.commit_graph(repo, offset, limit)
    }

    pub fn reset_to_commit(
        &self,
        repo: &mut Repository,
        commit_oid: &str,
        mode: ResetMode,
    ) -> Result<RepositorySnapshot> {
        if repo.head_detached()? {
            return Err(GitError::Git(git2::Error::from_str(
                "当前处于 detached HEAD，不能重置分支",
            )));
        }
        let commit = self.find_commit_by_oid(repo, commit_oid)?;
        let reset_type = match mode {
            ResetMode::Soft => ResetType::Soft,
            ResetMode::Mixed => ResetType::Mixed,
            ResetMode::Hard => ResetType::Hard,
        };
        self.progress
            .emit(OperationEvent::Started("正在重置分支".into()));
        repo.reset(commit.as_object(), reset_type, None)?;
        drop(commit);
        self.progress
            .emit(OperationEvent::Finished("分支已重置".into()));
        self.snapshot(repo)
    }

    pub fn revert_commit(
        &self,
        repo: &mut Repository,
        commit_oid: &str,
    ) -> Result<RepositorySnapshot> {
        if !self.status_full(repo)?.is_empty() || !self.conflicts(repo)?.is_empty() {
            return Err(GitError::Git(git2::Error::from_str(
                "回滚提交前需要先提交、暂存或丢弃当前工作区修改",
            )));
        }
        let revert_commit = self.find_commit_by_oid(repo, commit_oid)?;
        if revert_commit.parent_count() > 1 {
            return Err(GitError::Git(git2::Error::from_str("暂不支持回滚合并提交")));
        }
        let head_commit = repo.head()?.peel_to_commit()?;
        self.progress
            .emit(OperationEvent::Started("正在回滚提交".into()));
        let mut index = repo.revert_commit(&revert_commit, &head_commit, 0, None)?;
        if index.has_conflicts() {
            index.write()?;
            repo.checkout_index(Some(&mut index), None)?;
            return Err(GitError::Git(git2::Error::from_str(
                "回滚提交产生冲突，请解决冲突后再提交",
            )));
        }

        let tree_oid = index.write_tree_to(repo)?;
        let tree = repo.find_tree(tree_oid)?;
        let signature = signature(repo)?;
        let summary = revert_commit.summary().ok().flatten().unwrap_or("commit");
        let message = format!(
            "Revert \"{summary}\"\n\nThis reverts commit {}.",
            revert_commit.id()
        );
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&head_commit],
        )?;
        let mut checkout = CheckoutBuilder::new();
        checkout.force();
        repo.checkout_head(Some(&mut checkout))?;
        drop(tree);
        drop(head_commit);
        drop(revert_commit);
        self.progress
            .emit(OperationEvent::Finished("回滚提交完成".into()));
        self.snapshot(repo)
    }

    pub fn commit_graph(
        &self,
        repo: &Repository,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<CommitInfo>> {
        let (starts, refs_by_oid) = self.commit_graph_refs(repo)?;
        let mut walk = repo.revwalk()?;
        walk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;

        if starts.is_empty() {
            if let Err(err) = walk.push_head() {
                if is_empty_head_error(&err) {
                    return Ok(Vec::new());
                }
                return Err(err.into());
            }
        } else {
            for oid in starts {
                walk.push(oid)?;
            }
        }

        let mut commits = Vec::new();
        for oid in walk.skip(offset).take(limit) {
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            let author = commit.author();
            let author_name = author.name().unwrap_or("未知作者").to_string();
            let oid_string = oid.to_string();
            let parents = commit
                .parent_ids()
                .map(|parent| parent.to_string())
                .collect::<Vec<_>>();
            commits.push(CommitInfo {
                oid: oid_string.clone(),
                short_oid: oid_string.chars().take(8).collect(),
                summary: commit
                    .summary()
                    .ok()
                    .flatten()
                    .unwrap_or("(无提交信息)")
                    .to_string(),
                author: author_name,
                time: commit.time().seconds(),
                parents,
                refs: refs_by_oid.get(&oid_string).cloned().unwrap_or_default(),
            });
        }
        Ok(commits)
    }

    fn commit_graph_refs(
        &self,
        repo: &Repository,
    ) -> Result<(Vec<git2::Oid>, BTreeMap<String, Vec<CommitRefInfo>>)> {
        let mut starts = Vec::<git2::Oid>::new();
        let mut refs_by_oid = BTreeMap::<String, Vec<CommitRefInfo>>::new();

        let branches = match repo.branches(None) {
            Ok(branches) => Some(branches),
            Err(err) if is_empty_head_error(&err) => None,
            Err(err) => return Err(err.into()),
        };
        if let Some(branches) = branches {
            for branch in branches {
                let (branch, branch_type) = branch?;
                let Some(name) = branch.name()? else {
                    continue;
                };
                if branch_type == BranchType::Remote && name.ends_with("/HEAD") {
                    continue;
                }
                let Some(target) = branch.get().target() else {
                    continue;
                };
                if repo.find_commit(target).is_err() {
                    continue;
                }
                starts.push(target);
                refs_by_oid
                    .entry(target.to_string())
                    .or_default()
                    .push(CommitRefInfo {
                        name: name.to_string(),
                        kind: match branch_type {
                            BranchType::Local => CommitRefKind::LocalBranch,
                            BranchType::Remote => CommitRefKind::RemoteBranch,
                        },
                    });
            }
        }

        for name in repo.tag_names(None)?.iter().flatten().flatten() {
            let Ok(reference) = repo.find_reference(&format!("refs/tags/{name}")) else {
                continue;
            };
            let Ok(object) = reference.peel(git2::ObjectType::Commit) else {
                continue;
            };
            let Ok(commit) = object.into_commit() else {
                continue;
            };
            let oid = commit.id();
            refs_by_oid
                .entry(oid.to_string())
                .or_default()
                .push(CommitRefInfo {
                    name: name.to_string(),
                    kind: CommitRefKind::Tag,
                });
        }

        if let Ok(head) = repo.head()
            && let Ok(commit) = head.peel_to_commit()
        {
            let oid = commit.id();
            starts.push(oid);
            refs_by_oid
                .entry(oid.to_string())
                .or_default()
                .push(CommitRefInfo {
                    name: "HEAD".to_string(),
                    kind: CommitRefKind::Head,
                });
        }

        starts.sort();
        starts.dedup();
        for refs in refs_by_oid.values_mut() {
            refs.sort_by(|a, b| {
                ref_kind_order(&a.kind)
                    .cmp(&ref_kind_order(&b.kind))
                    .then_with(|| a.name.cmp(&b.name))
            });
            refs.dedup_by(|a, b| a.kind == b.kind && a.name == b.name);
        }

        Ok((starts, refs_by_oid))
    }

    pub fn commit_files(
        &self,
        repo: &Repository,
        commit_oid: &str,
    ) -> Result<Vec<CommitFileChange>> {
        let commit = self.find_commit_by_oid(repo, commit_oid)?;
        let diff = self.commit_diff(repo, &commit, None)?;
        let mut files = Vec::new();
        for delta in diff.deltas() {
            let Some(path) = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(path_to_git)
            else {
                continue;
            };
            let old_path = delta.old_file().path().map(path_to_git);
            files.push(CommitFileChange {
                path,
                old_path,
                status: change_state_from_delta(delta.status()),
            });
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }

    pub fn commit_file_diff(
        &self,
        repo: &Repository,
        commit_oid: &str,
        path: &Path,
    ) -> Result<FileDiff> {
        let commit = self.find_commit_by_oid(repo, commit_oid)?;
        let diff = self.commit_diff(repo, &commit, Some(path))?;
        self.file_diff_from_diff(diff, path_to_git(path), DiffScope::Staged)
    }

    fn find_commit_by_oid<'repo>(
        &self,
        repo: &'repo Repository,
        commit_oid: &str,
    ) -> Result<git2::Commit<'repo>> {
        let oid = git2::Oid::from_str(commit_oid)
            .map_err(|err| GitError::Message(format!("提交 ID 无效：{}", err.message())))?;
        Ok(repo.find_commit(oid)?)
    }

    fn commit_diff<'repo>(
        &self,
        repo: &'repo Repository,
        commit: &git2::Commit<'repo>,
        path: Option<&Path>,
    ) -> Result<git2::Diff<'repo>> {
        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };
        let mut options = DiffOptions::new();
        options.context_lines(DIFF_CONTEXT_LINES);
        if let Some(path) = path {
            options.pathspec(path);
        }
        Ok(repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut options))?)
    }

    fn file_diff_from_diff(
        &self,
        diff: git2::Diff<'_>,
        path: String,
        scope: DiffScope,
    ) -> Result<FileDiff> {
        let mut lines = Vec::new();
        let mut is_binary = false;
        for delta in diff.deltas() {
            if delta.flags().contains(git2::DiffFlags::BINARY) {
                is_binary = true;
            }
        }

        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            let kind = match line.origin() {
                '+' => DiffLineKind::Added,
                '-' => DiffLineKind::Removed,
                'F' | 'H' => DiffLineKind::Header,
                _ => DiffLineKind::Context,
            };
            let content = String::from_utf8_lossy(line.content())
                .trim_end_matches(['\r', '\n'])
                .to_string();
            lines.push(DiffLine {
                kind,
                old_lineno: line.old_lineno(),
                new_lineno: line.new_lineno(),
                content,
            });
            true
        })?;

        Ok(FileDiff {
            path,
            scope,
            is_binary,
            lines,
        })
    }

    fn head_name(&self, repo: &Repository) -> Option<String> {
        repo.head()
            .ok()
            .and_then(|head| head.shorthand().ok().map(str::to_string))
    }

    fn remote_callbacks<'a>(&'a self, repo: Option<&'a Repository>) -> RemoteCallbacks<'a> {
        let provider = self.credential_provider.clone();
        let progress = self.progress.clone();
        let config = repo.and_then(|repo| repo.config().ok());

        let mut callbacks = RemoteCallbacks::new();
        callbacks.transfer_progress(move |stats| {
            progress.emit(OperationEvent::Progress(format!(
                "已接收 {}/{} 个对象",
                stats.received_objects(),
                stats.total_objects()
            )));
            true
        });

        let provider_for_credentials = provider;
        callbacks.credentials(move |url, username_from_url, allowed_types| {
            credential_for_remote(
                config.as_ref(),
                provider_for_credentials.as_ref(),
                url,
                username_from_url,
                allowed_types,
            )
        });
        callbacks
    }

    fn remote_ref_for_branch<'repo>(
        &self,
        repo: &'repo Repository,
        remote: &RemoteName,
        branch: &str,
    ) -> Result<Reference<'repo>> {
        if let Ok(local) = repo.find_branch(branch, BranchType::Local)
            && let Ok(upstream) = local.upstream()
        {
            return Ok(upstream.into_reference());
        }

        repo.find_reference(&format!("refs/remotes/{}/{}", remote.0, branch))
            .map_err(GitError::from)
    }

    fn find_branch_reference<'repo>(
        &self,
        repo: &'repo Repository,
        name: &str,
    ) -> Result<Reference<'repo>> {
        if let Ok(branch) = repo.find_branch(name, BranchType::Local) {
            return Ok(branch.into_reference());
        }
        if let Ok(branch) = repo.find_branch(name, BranchType::Remote) {
            return Ok(branch.into_reference());
        }
        repo.find_reference(name).map_err(GitError::from)
    }

    fn merge_annotated(
        &self,
        repo: &Repository,
        annotated: &AnnotatedCommit<'_>,
        label: &str,
    ) -> Result<()> {
        let (analysis, _preference) = repo.merge_analysis(&[annotated])?;

        if analysis.contains(MergeAnalysis::ANALYSIS_UP_TO_DATE) {
            return Ok(());
        }

        if analysis.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) {
            fast_forward(repo, annotated)?;
            return Ok(());
        }

        if analysis.contains(MergeAnalysis::ANALYSIS_NORMAL) {
            let head_commit = repo.head()?.peel_to_commit()?;
            let other_commit = repo.find_commit(annotated.id())?;
            let mut merge_options = MergeOptions::new();
            let mut index =
                repo.merge_commits(&head_commit, &other_commit, Some(&mut merge_options))?;

            if index.has_conflicts() {
                repo.checkout_index(
                    Some(&mut index),
                    Some(CheckoutBuilder::new().allow_conflicts(true)),
                )?;
                repo.cleanup_state()?;
                return Err(GitError::Conflicts(self.conflicts(repo)?));
            }

            let tree_id = index.write_tree_to(repo)?;
            let tree = repo.find_tree(tree_id)?;
            let signature = signature(repo)?;

            let mut repo_index = repo.index()?;
            repo_index.read_tree(&tree)?;
            repo_index.write()?;
            let mut checkout = CheckoutBuilder::new();
            checkout.safe();
            repo.checkout_index(Some(&mut repo_index), Some(&mut checkout))?;

            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &format!("Merge branch '{label}'"),
                &tree,
                &[&head_commit, &other_commit],
            )?;
            repo.cleanup_state()?;
            return Ok(());
        }

        Err(GitError::Message(format!(
            "无法合并 {label}：不支持的合并分析结果"
        )))
    }

    fn conflicts(&self, repo: &Repository) -> Result<Vec<String>> {
        let mut conflicts = Vec::new();
        let index = repo.index()?;
        if !index.has_conflicts() {
            return Ok(conflicts);
        }

        let conflicts_iter = index.conflicts()?;
        for conflict in conflicts_iter {
            let conflict = conflict?;
            if let Some(path) = conflict
                .our
                .as_ref()
                .or(conflict.their.as_ref())
                .or(conflict.ancestor.as_ref())
                .and_then(|entry| std::str::from_utf8(&entry.path).ok())
            {
                conflicts.push(path.to_string());
            }
        }
        conflicts.sort();
        conflicts.dedup();
        Ok(conflicts)
    }
}

fn validate_branch_name(name: &str) -> Result<()> {
    if name.trim().is_empty()
        || name.contains('\\')
        || name.starts_with('-')
        || !git2::Branch::name_is_valid(name)?
    {
        return Err(GitError::InvalidBranchName(name.to_string()));
    }
    Ok(())
}

fn remote_branch_name_parts(name: &str) -> Result<(&str, &str)> {
    let Some((remote, branch)) = name.split_once('/') else {
        return Err(GitError::InvalidBranchName(name.to_string()));
    };
    if remote.trim().is_empty() || branch.trim().is_empty() {
        return Err(GitError::InvalidBranchName(name.to_string()));
    }
    Ok((remote, branch))
}

fn staged_state(status: Status) -> Option<ChangeState> {
    if status.contains(Status::CONFLICTED) {
        Some(ChangeState::Conflicted)
    } else if status.contains(Status::INDEX_RENAMED) {
        Some(ChangeState::Renamed)
    } else if status.contains(Status::INDEX_TYPECHANGE) {
        Some(ChangeState::Typechange)
    } else if status.contains(Status::INDEX_NEW) {
        Some(ChangeState::Added)
    } else if status.contains(Status::INDEX_MODIFIED) {
        Some(ChangeState::Modified)
    } else if status.contains(Status::INDEX_DELETED) {
        Some(ChangeState::Deleted)
    } else {
        None
    }
}

fn unstaged_state(status: Status) -> Option<ChangeState> {
    if status.contains(Status::CONFLICTED) {
        Some(ChangeState::Conflicted)
    } else if status.contains(Status::WT_RENAMED) {
        Some(ChangeState::Renamed)
    } else if status.contains(Status::WT_TYPECHANGE) {
        Some(ChangeState::Typechange)
    } else if status.contains(Status::WT_NEW) {
        Some(ChangeState::Untracked)
    } else if status.contains(Status::WT_MODIFIED) {
        Some(ChangeState::Modified)
    } else if status.contains(Status::WT_DELETED) {
        Some(ChangeState::Deleted)
    } else {
        None
    }
}

fn change_state_from_delta(delta: Delta) -> ChangeState {
    match delta {
        Delta::Added => ChangeState::Added,
        Delta::Deleted => ChangeState::Deleted,
        Delta::Renamed => ChangeState::Renamed,
        Delta::Typechange => ChangeState::Typechange,
        Delta::Conflicted => ChangeState::Conflicted,
        _ => ChangeState::Modified,
    }
}

fn ref_kind_order(kind: &CommitRefKind) -> u8 {
    match kind {
        CommitRefKind::Head => 0,
        CommitRefKind::LocalBranch => 1,
        CommitRefKind::RemoteBranch => 2,
        CommitRefKind::Tag => 3,
    }
}

fn is_empty_head_error(err: &git2::Error) -> bool {
    err.code() == ErrorCode::UnbornBranch
        || err.code() == ErrorCode::NotFound
        || err.message().contains("reference 'refs/heads/")
}

fn signature(repo: &Repository) -> Result<Signature<'static>> {
    repo.signature()
        .or_else(|_| Signature::now("Khaslana", "khaslana@example.invalid"))
        .map_err(GitError::from)
}

fn parents(repo: &Repository) -> Result<Vec<git2::Commit<'_>>> {
    if let Ok(head) = repo.head() {
        if let Ok(commit) = head.peel_to_commit() {
            return Ok(vec![commit]);
        }
    }
    Ok(Vec::new())
}

fn fast_forward(repo: &Repository, annotated: &AnnotatedCommit<'_>) -> Result<()> {
    let refname = repo.head()?.name().map_err(GitError::from)?.to_string();
    let mut reference = repo.find_reference(&refname)?;
    reference.set_target(annotated.id(), "khaslana fast-forward")?;
    repo.set_head(&refname)?;
    let mut checkout = CheckoutBuilder::new();
    checkout.safe();
    repo.checkout_head(Some(&mut checkout))?;
    Ok(())
}

fn credential_for_remote(
    config: Option<&git2::Config>,
    provider: &dyn CredentialProvider,
    url: &str,
    username_from_url: Option<&str>,
    allowed_types: CredentialType,
) -> std::result::Result<Cred, git2::Error> {
    if let Some(config) = config
        && let Ok(cred) = Cred::credential_helper(config, url, username_from_url)
    {
        return Ok(cred);
    }

    if allowed_types.contains(CredentialType::SSH_KEY) {
        let username = username_from_url.unwrap_or("git");
        if let Ok(cred) = Cred::ssh_key_from_agent(username) {
            return Ok(cred);
        }
    }

    if allowed_types.contains(CredentialType::DEFAULT)
        && let Ok(cred) = Cred::default()
    {
        return Ok(cred);
    }

    let request = CredentialRequest {
        url: url.to_string(),
        username_from_url: username_from_url.map(str::to_string),
        allowed_types,
    };
    match provider.credential_for(request.clone()) {
        Ok(Some(credential)) => to_git_credential(&request, credential),
        Ok(None) => Err(git2::Error::from_str(&format!("访问 {url} 需要身份验证"))),
        Err(err) => Err(git2::Error::from_str(&err.to_string())),
    }
}

fn path_to_git(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use git2::{BranchType, Oid, RepositoryInitOptions};
    use tempfile::TempDir;

    use super::*;
    use crate::credentials::PromptCredentialProvider;

    fn service() -> GitService {
        GitService::new(
            Arc::new(PromptCredentialProvider::memory_only(|_| Ok(None))),
            Arc::new(NoopProgress),
        )
    }

    fn init_repo() -> (TempDir, Repository, GitService) {
        let dir = TempDir::new().unwrap();
        let mut options = RepositoryInitOptions::new();
        options.initial_head("main");
        let repo = Repository::init_opts(dir.path(), &options).unwrap();
        configure_user(&repo);
        (dir, repo, service())
    }

    fn configure_user(repo: &Repository) {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config
            .set_str("user.email", "test@example.invalid")
            .unwrap();
    }

    fn write_file(root: &Path, path: &str, body: &str) {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, body).unwrap();
    }

    fn commit_all(repo: &Repository, message: &str) -> Oid {
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = signature(repo).unwrap();
        let parents = parents(repo).unwrap();
        let parent_refs = parents.iter().collect::<Vec<_>>();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )
        .unwrap()
    }

    fn path_url(path: &Path) -> String {
        let normalized = path.display().to_string().replace('\\', "/");
        if cfg!(windows) {
            format!("file:///{normalized}")
        } else {
            format!("file://{normalized}")
        }
    }

    fn clone_repo_with_remote_feature()
    -> (TempDir, TempDir, std::path::PathBuf, Repository, GitService) {
        let remote_dir = TempDir::new().unwrap();
        let mut bare_opts = RepositoryInitOptions::new();
        bare_opts.bare(true).initial_head("main");
        Repository::init_opts(remote_dir.path(), &bare_opts).unwrap();

        let (seed_dir, mut seed_repo, service) = init_repo();
        write_file(seed_dir.path(), "README.md", "seed\n");
        commit_all(&seed_repo, "seed");
        seed_repo
            .remote("origin", &path_url(remote_dir.path()))
            .unwrap();
        service
            .push(&mut seed_repo, &RemoteName::new("origin"))
            .unwrap();
        service
            .create_branch(&mut seed_repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut seed_repo, &BranchName::new("feature"))
            .unwrap();
        write_file(seed_dir.path(), "feature.txt", "feature\n");
        commit_all(&seed_repo, "feature");
        service
            .push(&mut seed_repo, &RemoteName::new("origin"))
            .unwrap();

        let clone_dir = TempDir::new().unwrap();
        let clone_path = clone_dir.path().join("clone");
        service
            .clone_repo(&path_url(remote_dir.path()), &RepoPath::new(&clone_path))
            .unwrap();
        let clone_repo = Repository::open(&clone_path).unwrap();
        configure_user(&clone_repo);
        (remote_dir, clone_dir, clone_path, clone_repo, service)
    }

    #[test]
    fn branch_create_rename_delete() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "README.md", "hello");
        commit_all(&repo, "initial");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        assert!(repo.find_branch("feature", BranchType::Local).is_ok());

        service
            .rename_branch(
                &mut repo,
                &BranchName::new("feature"),
                &BranchName::new("topic"),
            )
            .unwrap();
        assert!(repo.find_branch("feature", BranchType::Local).is_err());
        assert!(repo.find_branch("topic", BranchType::Local).is_ok());

        service
            .delete_branch(&mut repo, &BranchName::new("topic"))
            .unwrap();
        assert!(repo.find_branch("topic", BranchType::Local).is_err());
    }

    #[test]
    fn stage_unstage_and_commit() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "src/lib.rs", "pub fn value() -> i32 { 1 }\n");

        service
            .stage_path(&mut repo, Path::new("src/lib.rs"))
            .unwrap();
        let changes = service.status(&repo).unwrap();
        assert_eq!(changes[0].staged, Some(ChangeState::Added));

        service
            .unstage_path(&mut repo, Path::new("src/lib.rs"))
            .unwrap();
        let changes = service.status(&repo).unwrap();
        assert_eq!(changes[0].unstaged, Some(ChangeState::Untracked));

        service
            .stage_path(&mut repo, Path::new("src/lib.rs"))
            .unwrap();
        write_file(dir.path(), "src/lib.rs", "pub fn value() -> i32 { 2 }\n");
        let changes = service.status(&repo).unwrap();
        let change = changes
            .iter()
            .find(|change| change.path == "src/lib.rs")
            .unwrap();
        assert_eq!(change.staged, Some(ChangeState::Added));
        assert_eq!(change.unstaged, Some(ChangeState::Modified));
        service
            .stage_path(&mut repo, Path::new("src/lib.rs"))
            .unwrap();
        service
            .commit(&mut repo, &CommitMessage::new("add library"))
            .unwrap();
        assert!(service.status(&repo).unwrap().is_empty());
    }

    #[test]
    fn batch_stage_and_unstage_paths() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "one.txt", "one\n");
        write_file(dir.path(), "two.txt", "two\n");

        let paths = [Path::new("one.txt"), Path::new("two.txt")];
        service.stage_paths(&mut repo, paths).unwrap();
        let changes = service.status(&repo).unwrap();
        assert!(changes.iter().any(|change| {
            change.path == "one.txt" && change.staged == Some(ChangeState::Added)
        }));
        assert!(changes.iter().any(|change| {
            change.path == "two.txt" && change.staged == Some(ChangeState::Added)
        }));

        service.unstage_paths(&mut repo, paths).unwrap();
        let changes = service.status(&repo).unwrap();
        assert!(changes.iter().any(|change| {
            change.path == "one.txt" && change.unstaged == Some(ChangeState::Untracked)
        }));
        assert!(changes.iter().any(|change| {
            change.path == "two.txt" && change.unstaged == Some(ChangeState::Untracked)
        }));
    }

    #[test]
    fn batch_stage_handles_deleted_files() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "keep.txt", "keep\n");
        write_file(dir.path(), "remove.txt", "remove\n");
        commit_all(&repo, "initial");

        fs::remove_file(dir.path().join("remove.txt")).unwrap();
        write_file(dir.path(), "keep.txt", "changed\n");

        let paths = [Path::new("keep.txt"), Path::new("remove.txt")];
        service.stage_paths(&mut repo, paths).unwrap();
        let changes = service.status(&repo).unwrap();
        assert!(changes.iter().any(|change| {
            change.path == "keep.txt" && change.staged == Some(ChangeState::Modified)
        }));
        assert!(changes.iter().any(|change| {
            change.path == "remove.txt" && change.staged == Some(ChangeState::Deleted)
        }));
    }

    #[test]
    fn status_fast_skips_untracked_but_keeps_tracked_and_staged_changes() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "tracked.txt", "one\n");
        write_file(dir.path(), "staged.txt", "one\n");
        commit_all(&repo, "initial");

        write_file(dir.path(), "tracked.txt", "one\ntwo\n");
        write_file(dir.path(), "staged.txt", "one\ntwo\n");
        service
            .stage_path(&mut repo, Path::new("staged.txt"))
            .unwrap();
        write_file(dir.path(), "untracked.txt", "new\n");

        let fast = service.status_fast(&repo).unwrap();
        assert!(fast.iter().any(|change| {
            change.path == "tracked.txt" && change.unstaged == Some(ChangeState::Modified)
        }));
        assert!(fast.iter().any(|change| {
            change.path == "staged.txt" && change.staged == Some(ChangeState::Modified)
        }));
        assert!(!fast.iter().any(|change| change.path == "untracked.txt"));

        let full = service.status_full(&repo).unwrap();
        assert!(full.iter().any(|change| {
            change.path == "untracked.txt" && change.unstaged == Some(ChangeState::Untracked)
        }));
    }

    #[test]
    fn metadata_snapshot_excludes_status_changes() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "tracked.txt", "one\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "tracked.txt", "one\ntwo\n");
        write_file(dir.path(), "untracked.txt", "new\n");

        let metadata = service.snapshot_metadata(&mut repo).unwrap();
        assert_eq!(metadata.head.as_deref(), Some("main"));
        assert!(metadata.branches.iter().any(|branch| branch.name == "main"));
        assert!(metadata.changes.is_empty());

        let full = service.snapshot_details(&mut repo).unwrap();
        assert!(!full.changes.is_empty());
    }

    #[test]
    fn merge_success() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "base.txt", "base\n");
        commit_all(&repo, "initial");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        write_file(dir.path(), "feature.txt", "feature\n");
        commit_all(&repo, "feature");

        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();
        write_file(dir.path(), "main.txt", "main\n");
        commit_all(&repo, "main");

        service
            .merge_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        assert!(dir.path().join("feature.txt").exists());
        assert!(service.conflicts(&repo).unwrap().is_empty());
    }

    #[test]
    fn merge_conflict_detection() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "same.txt", "base\n");
        commit_all(&repo, "initial");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        write_file(dir.path(), "same.txt", "feature\n");
        commit_all(&repo, "feature");

        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();
        write_file(dir.path(), "same.txt", "main\n");
        commit_all(&repo, "main");

        let err = service
            .merge_branch(&mut repo, &BranchName::new("feature"))
            .unwrap_err();
        assert!(matches!(err, GitError::Conflicts(paths) if paths == vec!["same.txt"]));
    }

    #[test]
    fn clone_fetch_push_against_local_bare_remote() {
        let remote_dir = TempDir::new().unwrap();
        let mut bare_opts = RepositoryInitOptions::new();
        bare_opts.bare(true).initial_head("main");
        Repository::init_opts(remote_dir.path(), &bare_opts).unwrap();

        let (seed_dir, mut seed_repo, service) = init_repo();
        write_file(seed_dir.path(), "README.md", "seed\n");
        commit_all(&seed_repo, "seed");
        seed_repo
            .remote("origin", &path_url(remote_dir.path()))
            .unwrap();
        service
            .push(&mut seed_repo, &RemoteName::new("origin"))
            .unwrap();

        let clone_dir = TempDir::new().unwrap();
        let clone_path = clone_dir.path().join("clone");
        let snapshot = service
            .clone_repo(&path_url(remote_dir.path()), &RepoPath::new(&clone_path))
            .unwrap();
        assert_eq!(snapshot.head.as_deref(), Some("main"));

        let mut clone_repo = Repository::open(&clone_path).unwrap();
        configure_user(&clone_repo);
        write_file(&clone_path, "clone.txt", "clone\n");
        commit_all(&clone_repo, "clone");
        service
            .push(&mut clone_repo, &RemoteName::new("origin"))
            .unwrap();

        let other_dir = TempDir::new().unwrap();
        let other_path = other_dir.path().join("other");
        service
            .clone_repo(&path_url(remote_dir.path()), &RepoPath::new(&other_path))
            .unwrap();
        assert!(other_path.join("clone.txt").exists());
    }

    #[test]
    fn open_fast_lists_only_local_branches() {
        let (_remote_dir, _clone_dir, clone_path, mut repo, service) =
            clone_repo_with_remote_feature();
        service
            .fetch(&mut repo, &RemoteName::new("origin"))
            .unwrap();

        let fast = service.open_fast(&RepoPath::new(&clone_path)).unwrap();
        assert!(
            fast.branches
                .iter()
                .all(|branch| branch.kind == BranchKind::Local)
        );
        assert!(fast.branches.iter().any(|branch| branch.name == "main"));
        assert!(fast.remotes.is_empty());
        assert!(fast.changes.is_empty());
        assert!(fast.tags.is_empty());
        assert!(fast.stashes.is_empty());

        let details = service.snapshot_details(&mut repo).unwrap();
        assert!(details.remotes.iter().any(|remote| remote == "origin"));
        assert!(details.branches.iter().any(|branch| {
            branch.kind == BranchKind::Remote && branch.name == "origin/feature"
        }));
    }

    #[test]
    fn checkout_remote_branch_creates_tracks_and_switches() {
        let (_remote_dir, _clone_dir, _clone_path, mut repo, service) =
            clone_repo_with_remote_feature();
        service
            .fetch(&mut repo, &RemoteName::new("origin"))
            .unwrap();

        let snapshot = service
            .checkout_remote_branch(&mut repo, &BranchName::new("origin/feature"))
            .unwrap();

        assert_eq!(snapshot.head.as_deref(), Some("feature"));
        assert!(repo.find_branch("feature", BranchType::Local).is_ok());
        let branch = repo.find_branch("feature", BranchType::Local).unwrap();
        let upstream = branch.upstream().unwrap();
        assert_eq!(upstream.name().unwrap(), Some("origin/feature"));
        assert!(repo.workdir().unwrap().join("feature.txt").exists());
    }

    #[test]
    fn checkout_remote_branch_reuses_existing_local_branch() {
        let (_remote_dir, _clone_dir, _clone_path, mut repo, service) =
            clone_repo_with_remote_feature();
        service
            .fetch(&mut repo, &RemoteName::new("origin"))
            .unwrap();
        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();

        let snapshot = service
            .checkout_remote_branch(&mut repo, &BranchName::new("origin/feature"))
            .unwrap();

        assert_eq!(snapshot.head.as_deref(), Some("feature"));
        let branch = repo.find_branch("feature", BranchType::Local).unwrap();
        let upstream = branch.upstream().unwrap();
        assert_eq!(upstream.name().unwrap(), Some("origin/feature"));
    }

    #[test]
    fn credential_provider_is_called_when_required() {
        struct CountingProvider(Arc<std::sync::atomic::AtomicUsize>);

        impl CredentialProvider for CountingProvider {
            fn credential_for(
                &self,
                _request: CredentialRequest,
            ) -> Result<Option<crate::credentials::GitCredential>> {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(None)
            }
        }

        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let service = GitService::new(
            Arc::new(CountingProvider(count.clone())),
            Arc::new(NoopProgress),
        );
        let result = credential_for_remote(
            None,
            service.credential_provider.as_ref(),
            "https://example.invalid/repo.git",
            None,
            CredentialType::USER_PASS_PLAINTEXT,
        );
        assert!(result.is_err());
        assert_eq!(count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn diff_for_staged_file() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "file.txt", "one\ntwo\n");
        service
            .stage_path(&mut repo, Path::new("file.txt"))
            .unwrap();

        let diff = service
            .diff_for_path(&repo, Path::new("file.txt"), DiffScope::Staged)
            .unwrap();
        let added = diff
            .lines
            .iter()
            .find(|line| line.kind == DiffLineKind::Added && line.content.contains("two"))
            .unwrap();
        assert_eq!(added.old_lineno, None);
        assert_eq!(added.new_lineno, Some(2));
    }

    #[test]
    fn diff_uses_three_context_lines() {
        let (dir, mut repo, service) = init_repo();
        let original = (1..=12)
            .map(|line| format!("line {line}\n"))
            .collect::<String>();
        write_file(dir.path(), "file.txt", &original);
        commit_all(&repo, "initial");

        let modified = (1..=12)
            .map(|line| {
                if line == 8 {
                    "line 8 changed\n".to_string()
                } else {
                    format!("line {line}\n")
                }
            })
            .collect::<String>();
        write_file(dir.path(), "file.txt", &modified);
        service
            .stage_path(&mut repo, Path::new("file.txt"))
            .unwrap();

        let diff = service
            .diff_for_path(&repo, Path::new("file.txt"), DiffScope::Staged)
            .unwrap();
        let body = diff
            .lines
            .iter()
            .filter(|line| line.kind != DiffLineKind::Header)
            .map(|line| line.content.as_str())
            .collect::<Vec<_>>();

        assert!(body.iter().any(|line| line.contains("line 5")));
        assert!(body.iter().any(|line| line.contains("line 11")));
        assert!(!body.iter().any(|line| line.contains("line 4")));
        assert!(!body.iter().any(|line| line.contains("line 12")));
    }

    #[test]
    fn commit_history_pages_and_commit_diff() {
        let (dir, repo, service) = init_repo();
        write_file(dir.path(), "root.txt", "root\n");
        let root_oid = commit_all(&repo, "root commit");
        write_file(dir.path(), "file.txt", "one\n");
        commit_all(&repo, "add file");
        write_file(dir.path(), "file.txt", "one\ntwo\n");
        commit_all(&repo, "modify file");

        let first_page = service.commit_history(&repo, 0, 2).unwrap();
        assert_eq!(first_page.len(), 2);
        assert_eq!(first_page[0].summary, "modify file");
        assert_eq!(first_page[1].summary, "add file");

        let second_page = service.commit_history(&repo, 2, 2).unwrap();
        assert_eq!(second_page.len(), 1);
        assert_eq!(second_page[0].summary, "root commit");
        assert_ne!(first_page[1].oid, second_page[0].oid);

        let files = service.commit_files(&repo, &first_page[0].oid).unwrap();
        assert!(
            files
                .iter()
                .any(|file| { file.path == "file.txt" && file.status == ChangeState::Modified })
        );
        let diff = service
            .commit_file_diff(&repo, &first_page[0].oid, Path::new("file.txt"))
            .unwrap();
        assert!(diff.lines.iter().any(|line| {
            line.kind == DiffLineKind::Added
                && line.content.contains("two")
                && line.new_lineno == Some(2)
        }));

        let root_files = service.commit_files(&repo, &root_oid.to_string()).unwrap();
        assert!(
            root_files
                .iter()
                .any(|file| { file.path == "root.txt" && file.status == ChangeState::Added })
        );
        let root_diff = service
            .commit_file_diff(&repo, &root_oid.to_string(), Path::new("root.txt"))
            .unwrap();
        assert!(
            root_diff
                .lines
                .iter()
                .any(|line| line.kind == DiffLineKind::Added && line.content.contains("root"))
        );
    }

    #[test]
    fn commit_graph_lists_all_branch_reachable_commits_and_refs() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "root.txt", "root\n");
        let root_oid = commit_all(&repo, "root commit");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        write_file(dir.path(), "feature.txt", "feature\n");
        let feature_oid = commit_all(&repo, "feature commit");

        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();
        write_file(dir.path(), "main.txt", "main\n");
        let main_oid = commit_all(&repo, "main commit");

        repo.reference("refs/remotes/origin/feature", feature_oid, true, "test")
            .unwrap();
        let feature_commit = repo.find_commit(feature_oid).unwrap();
        repo.tag_lightweight("v-feature", feature_commit.as_object(), false)
            .unwrap();
        drop(feature_commit);

        let commits = service.commit_graph(&repo, 0, 20).unwrap();
        let summaries = commits
            .iter()
            .map(|commit| commit.summary.as_str())
            .collect::<Vec<_>>();
        assert!(summaries.contains(&"main commit"));
        assert!(summaries.contains(&"feature commit"));
        assert!(summaries.contains(&"root commit"));

        let feature = commits
            .iter()
            .find(|commit| commit.oid == feature_oid.to_string())
            .unwrap();
        assert!(feature.parents.contains(&root_oid.to_string()));
        assert!(feature.refs.iter().any(|reference| {
            reference.kind == CommitRefKind::LocalBranch && reference.name == "feature"
        }));
        assert!(feature.refs.iter().any(|reference| {
            reference.kind == CommitRefKind::RemoteBranch && reference.name == "origin/feature"
        }));
        assert!(feature.refs.iter().any(|reference| {
            reference.kind == CommitRefKind::Tag && reference.name == "v-feature"
        }));

        let main = commits
            .iter()
            .find(|commit| commit.oid == main_oid.to_string())
            .unwrap();
        assert!(
            main.refs
                .iter()
                .any(|reference| reference.kind == CommitRefKind::Head)
        );
    }

    #[test]
    fn commit_graph_records_merge_parents() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "base.txt", "base\n");
        commit_all(&repo, "initial");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        write_file(dir.path(), "feature.txt", "feature\n");
        let feature_oid = commit_all(&repo, "feature");

        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();
        write_file(dir.path(), "main.txt", "main\n");
        let main_oid = commit_all(&repo, "main");

        service
            .merge_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        let merge_oid = repo.head().unwrap().target().unwrap();

        let commits = service.commit_graph(&repo, 0, 20).unwrap();
        let merge = commits
            .iter()
            .find(|commit| commit.oid == merge_oid.to_string())
            .unwrap();
        assert_eq!(merge.parents.len(), 2);
        assert!(merge.parents.contains(&main_oid.to_string()));
        assert!(merge.parents.contains(&feature_oid.to_string()));
    }

    #[test]
    fn commit_graph_paginates_without_duplicates() {
        let (dir, repo, service) = init_repo();
        for index in 0..5 {
            write_file(dir.path(), "file.txt", &format!("{index}\n"));
            commit_all(&repo, &format!("commit {index}"));
        }

        let first_page = service.commit_graph(&repo, 0, 3).unwrap();
        let second_page = service.commit_graph(&repo, 3, 3).unwrap();
        assert_eq!(first_page.len(), 3);
        assert_eq!(second_page.len(), 2);
        let first_oids = first_page
            .iter()
            .map(|commit| commit.oid.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for commit in second_page {
            assert!(!first_oids.contains(commit.oid.as_str()));
        }
    }

    #[test]
    fn commit_graph_empty_repo_returns_empty() {
        let (_dir, repo, service) = init_repo();
        let commits = service.commit_graph(&repo, 0, 20).unwrap();
        assert!(commits.is_empty());
    }

    #[test]
    fn reset_to_commit_soft_keeps_changes_staged() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        let first_oid = commit_all(&repo, "one");
        write_file(dir.path(), "file.txt", "two\n");
        commit_all(&repo, "two");

        service
            .reset_to_commit(&mut repo, &first_oid.to_string(), ResetMode::Soft)
            .unwrap();

        assert_eq!(repo.head().unwrap().target(), Some(first_oid));
        let changes = service.status_full(&repo).unwrap();
        let file = changes
            .iter()
            .find(|change| change.path == "file.txt")
            .unwrap();
        assert_eq!(file.staged, Some(ChangeState::Modified));
        assert_eq!(file.unstaged, None);
    }

    #[test]
    fn reset_to_commit_mixed_keeps_changes_unstaged() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        let first_oid = commit_all(&repo, "one");
        write_file(dir.path(), "file.txt", "two\n");
        commit_all(&repo, "two");

        service
            .reset_to_commit(&mut repo, &first_oid.to_string(), ResetMode::Mixed)
            .unwrap();

        assert_eq!(repo.head().unwrap().target(), Some(first_oid));
        let changes = service.status_full(&repo).unwrap();
        let file = changes
            .iter()
            .find(|change| change.path == "file.txt")
            .unwrap();
        assert_eq!(file.staged, None);
        assert_eq!(file.unstaged, Some(ChangeState::Modified));
    }

    #[test]
    fn reset_to_commit_hard_updates_head_and_worktree() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        let first_oid = commit_all(&repo, "one");
        write_file(dir.path(), "file.txt", "two\n");
        commit_all(&repo, "two");

        service
            .reset_to_commit(&mut repo, &first_oid.to_string(), ResetMode::Hard)
            .unwrap();

        assert_eq!(repo.head().unwrap().target(), Some(first_oid));
        assert_eq!(
            fs::read_to_string(dir.path().join("file.txt"))
                .unwrap()
                .replace("\r\n", "\n"),
            "one\n"
        );
        assert!(service.status_full(&repo).unwrap().is_empty());
    }

    #[test]
    fn reset_to_commit_rejects_detached_head() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        let first_oid = commit_all(&repo, "one");
        repo.set_head_detached(first_oid).unwrap();

        let error = service
            .reset_to_commit(&mut repo, &first_oid.to_string(), ResetMode::Mixed)
            .unwrap_err()
            .to_string();
        assert!(error.contains("detached HEAD"));
    }

    #[test]
    fn revert_commit_creates_new_commit_and_restores_content() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        commit_all(&repo, "one");
        write_file(dir.path(), "file.txt", "two\n");
        let second_oid = commit_all(&repo, "two");

        service
            .revert_commit(&mut repo, &second_oid.to_string())
            .unwrap();

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        assert_ne!(head.id(), second_oid);
        assert!(head.summary().unwrap().unwrap().contains("Revert"));
        assert_eq!(
            fs::read_to_string(dir.path().join("file.txt"))
                .unwrap()
                .replace("\r\n", "\n"),
            "one\n"
        );
        assert!(service.status_full(&repo).unwrap().is_empty());
    }

    #[test]
    fn revert_commit_rejects_dirty_worktree() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        commit_all(&repo, "one");
        write_file(dir.path(), "file.txt", "two\n");
        let second_oid = commit_all(&repo, "two");
        write_file(dir.path(), "scratch.txt", "dirty\n");

        let error = service
            .revert_commit(&mut repo, &second_oid.to_string())
            .unwrap_err()
            .to_string();
        assert!(error.contains("工作区修改"));
    }

    #[test]
    fn revert_commit_rejects_merge_commit() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "base.txt", "base\n");
        commit_all(&repo, "initial");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        write_file(dir.path(), "feature.txt", "feature\n");
        commit_all(&repo, "feature");

        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();
        write_file(dir.path(), "main.txt", "main\n");
        commit_all(&repo, "main");

        service
            .merge_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        let merge_oid = repo.head().unwrap().target().unwrap();

        let error = service
            .revert_commit(&mut repo, &merge_oid.to_string())
            .unwrap_err()
            .to_string();
        assert!(error.contains("合并提交"));
    }

    #[test]
    fn merge_commit_files_use_first_parent_diff() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "base.txt", "base\n");
        commit_all(&repo, "initial");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        write_file(dir.path(), "feature.txt", "feature\n");
        commit_all(&repo, "feature");

        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();
        write_file(dir.path(), "main.txt", "main\n");
        commit_all(&repo, "main");

        let snapshot = service
            .merge_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        assert_eq!(snapshot.head.as_deref(), Some("main"));

        let merge_oid = repo.head().unwrap().target().unwrap().to_string();
        let commit = repo
            .find_commit(git2::Oid::from_str(&merge_oid).unwrap())
            .unwrap();
        assert_eq!(commit.parent_count(), 2);

        let files = service.commit_files(&repo, &merge_oid).unwrap();
        assert!(
            files
                .iter()
                .any(|file| { file.path == "feature.txt" && file.status == ChangeState::Added })
        );
        assert!(!files.iter().any(|file| file.path == "main.txt"));
    }

    #[test]
    fn snapshot_lists_tags_and_stashes() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        commit_all(&repo, "initial");

        let head = repo.head().unwrap().peel_to_commit().unwrap();
        repo.tag_lightweight("v1.0.0", head.as_object(), false)
            .unwrap();
        drop(head);

        write_file(dir.path(), "scratch.txt", "stash me\n");
        let signature = signature(&repo).unwrap();
        repo.stash_save(
            &signature,
            "work in progress",
            Some(git2::StashFlags::INCLUDE_UNTRACKED),
        )
        .unwrap();

        let snapshot = service.snapshot(&mut repo).unwrap();
        assert!(snapshot.tags.iter().any(|tag| tag.name == "v1.0.0"));
        assert_eq!(snapshot.stashes.len(), 1);
        assert_eq!(snapshot.stashes[0].index, 0);
        assert!(snapshot.stashes[0].message.contains("work in progress"));
    }

    #[test]
    fn checkout_tag_detaches_head_and_updates_worktree() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "one\n");
        commit_all(&repo, "one");
        let first = repo.head().unwrap().peel_to_commit().unwrap();
        repo.tag_lightweight("v1", first.as_object(), false)
            .unwrap();
        drop(first);

        write_file(dir.path(), "file.txt", "two\n");
        commit_all(&repo, "two");

        service
            .checkout_tag(&mut repo, &TagName::new("v1"))
            .unwrap();

        assert!(repo.head_detached().unwrap());
        assert_eq!(
            fs::read_to_string(dir.path().join("file.txt"))
                .unwrap()
                .replace("\r\n", "\n"),
            "one\n"
        );
    }

    #[test]
    fn stash_apply_keeps_entry_and_pop_removes_it() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "file.txt", "base\n");
        commit_all(&repo, "initial");

        write_file(dir.path(), "file.txt", "applied\n");
        let sig = signature(&repo).unwrap();
        repo.stash_save(&sig, "change file", None).unwrap();
        assert_eq!(service.stashes(&mut repo).unwrap().len(), 1);

        service.apply_stash(&mut repo, 0).unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("file.txt"))
                .unwrap()
                .replace("\r\n", "\n"),
            "applied\n"
        );
        assert_eq!(service.stashes(&mut repo).unwrap().len(), 1);

        let (pop_dir, mut pop_repo, pop_service) = init_repo();
        write_file(pop_dir.path(), "file.txt", "base\n");
        commit_all(&pop_repo, "initial");
        write_file(pop_dir.path(), "file.txt", "popped\n");
        let sig = signature(&pop_repo).unwrap();
        pop_repo.stash_save(&sig, "pop file", None).unwrap();

        pop_service.pop_stash(&mut pop_repo, 0).unwrap();
        assert_eq!(
            fs::read_to_string(pop_dir.path().join("file.txt"))
                .unwrap()
                .replace("\r\n", "\n"),
            "popped\n"
        );
        assert!(pop_service.stashes(&mut pop_repo).unwrap().is_empty());
    }
}
