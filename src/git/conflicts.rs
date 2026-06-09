use std::fs;
use std::path::Path;

use git2::build::CheckoutBuilder;
use git2::{ErrorCode, Repository};

use super::{GitService, ensure_worktree_relative_path, path_to_git, remove_worktree_path};
use crate::{ConflictResolutionSide, GitError, OperationEvent, RepositorySnapshot, Result};

impl GitService {
    pub fn resolve_conflict_with_side(
        &self,
        repo: &mut Repository,
        path: &Path,
        side: ConflictResolutionSide,
    ) -> Result<RepositorySnapshot> {
        ensure_worktree_relative_path(path, "不能解决冲突")?;
        let label = match side {
            ConflictResolutionSide::Ours => "当前版本",
            ConflictResolutionSide::Theirs => "传入版本",
        };
        self.progress
            .emit(OperationEvent::Started(format!("正在使用{label}解决冲突")));

        let mut index = repo.index()?;
        let conflict = conflict_for_path(&index, path)?;
        let selected_entry = match side {
            ConflictResolutionSide::Ours => conflict.our.as_ref(),
            ConflictResolutionSide::Theirs => conflict.their.as_ref(),
        };

        if selected_entry.is_some() {
            let mut checkout = CheckoutBuilder::new();
            checkout
                .force()
                .path(path)
                .disable_pathspec_match(true)
                .update_index(false);
            match side {
                ConflictResolutionSide::Ours => {
                    checkout.use_ours(true);
                }
                ConflictResolutionSide::Theirs => {
                    checkout.use_theirs(true);
                }
            }
            repo.checkout_index(Some(&mut index), Some(&mut checkout))?;
            drop(index);
            let snapshot = self.mark_conflict_resolved_inner(repo, path)?;
            self.progress
                .emit(OperationEvent::Finished(format!("已使用{label}解决冲突")));
            return Ok(snapshot);
        }

        remove_worktree_path(repo, path)?;
        index.conflict_remove(path)?;
        let _ = index.remove_path(path);
        index.write()?;
        drop(index);
        self.progress
            .emit(OperationEvent::Finished(format!("已使用{label}解决冲突")));
        self.snapshot_after_operation(repo)
    }

    pub fn mark_conflict_resolved(
        &self,
        repo: &mut Repository,
        path: &Path,
    ) -> Result<RepositorySnapshot> {
        ensure_worktree_relative_path(path, "不能标记冲突已解决")?;
        self.progress
            .emit(OperationEvent::Started("正在标记冲突已解决".into()));
        let snapshot = self.mark_conflict_resolved_inner(repo, path)?;
        self.progress
            .emit(OperationEvent::Finished("冲突已标记为解决".into()));
        Ok(snapshot)
    }

    fn mark_conflict_resolved_inner(
        &self,
        repo: &mut Repository,
        path: &Path,
    ) -> Result<RepositorySnapshot> {
        let mut index = repo.index()?;
        conflict_for_path(&index, path)?;

        let workdir = repo
            .workdir()
            .ok_or_else(|| GitError::Message("裸仓库没有工作区，不能标记冲突已解决".into()))?;
        let full_path = workdir.join(path);
        match fs::symlink_metadata(&full_path) {
            Ok(metadata) => {
                if metadata.is_dir() && !metadata.file_type().is_symlink() {
                    return Err(GitError::Message(
                        "冲突路径是文件夹，不能标记为已解决".into(),
                    ));
                }
                index.conflict_remove(path)?;
                index.add_path(path)?;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                index.conflict_remove(path)?;
                let _ = index.remove_path(path);
            }
            Err(err) => return Err(GitError::Io(err)),
        }
        index.write()?;
        drop(index);
        self.snapshot_after_operation(repo)
    }
}

fn conflict_for_path(index: &git2::Index, path: &Path) -> Result<git2::IndexConflict> {
    index.conflict_get(path).map_err(|err| {
        if err.code() == ErrorCode::NotFound {
            GitError::Message(format!("该文件不存在冲突：{}", path_to_git(path)))
        } else {
            GitError::Git(err)
        }
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use git2::{IndexAddOption, Oid, Repository, RepositoryInitOptions};
    use tempfile::TempDir;

    use super::*;
    use crate::credentials::PromptCredentialProvider;
    use crate::{BranchName, CommitMessage, GitError};

    fn service() -> GitService {
        GitService::new(
            Arc::new(PromptCredentialProvider::memory_only(|_| Ok(None))),
            Arc::new(super::super::NoopProgress),
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

    fn assert_file_text(root: &Path, path: &str, expected: &str) {
        let actual = fs::read_to_string(root.join(path)).unwrap();
        assert_eq!(actual.replace("\r\n", "\n"), expected);
    }

    fn commit_all(repo: &Repository, message: &str) -> Oid {
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = super::super::signature(repo).unwrap();
        let parents = super::super::parents(repo).unwrap();
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

    fn create_text_conflict() -> (TempDir, Repository, GitService) {
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
        (dir, repo, service)
    }

    fn create_modify_delete_conflict() -> (TempDir, Repository, GitService) {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "same.txt", "base\n");
        commit_all(&repo, "initial");

        service
            .create_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        service
            .checkout_branch(&mut repo, &BranchName::new("feature"))
            .unwrap();
        fs::remove_file(dir.path().join("same.txt")).unwrap();
        {
            let mut index = repo.index().unwrap();
            index.remove_path(Path::new("same.txt")).unwrap();
            index.write().unwrap();
        }
        commit_all(&repo, "feature deletes");

        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();
        write_file(dir.path(), "same.txt", "main\n");
        commit_all(&repo, "main modifies");

        let err = service
            .merge_branch(&mut repo, &BranchName::new("feature"))
            .unwrap_err();
        assert!(matches!(err, GitError::Conflicts(paths) if paths == vec!["same.txt"]));
        (dir, repo, service)
    }

    #[test]
    fn resolve_conflict_with_ours_keeps_current_branch_version() {
        let (dir, mut repo, service) = create_text_conflict();

        let snapshot = service
            .resolve_conflict_with_side(
                &mut repo,
                Path::new("same.txt"),
                ConflictResolutionSide::Ours,
            )
            .unwrap();

        assert!(snapshot.conflicts.is_empty());
        assert_file_text(dir.path(), "same.txt", "main\n");
        service
            .commit(&mut repo, &CommitMessage::new("resolve with ours"))
            .unwrap();
        assert!(service.status_full(&repo).unwrap().is_empty());
    }

    #[test]
    fn resolve_conflict_with_theirs_keeps_incoming_branch_version() {
        let (dir, mut repo, service) = create_text_conflict();

        let snapshot = service
            .resolve_conflict_with_side(
                &mut repo,
                Path::new("same.txt"),
                ConflictResolutionSide::Theirs,
            )
            .unwrap();

        assert!(snapshot.conflicts.is_empty());
        assert_file_text(dir.path(), "same.txt", "feature\n");
        service
            .commit(&mut repo, &CommitMessage::new("resolve with theirs"))
            .unwrap();
        assert!(service.status_full(&repo).unwrap().is_empty());
    }

    #[test]
    fn mark_conflict_resolved_accepts_manual_resolution() {
        let (dir, mut repo, service) = create_text_conflict();
        write_file(dir.path(), "same.txt", "manual\n");

        let snapshot = service
            .mark_conflict_resolved(&mut repo, Path::new("same.txt"))
            .unwrap();

        assert!(snapshot.conflicts.is_empty());
        assert_file_text(dir.path(), "same.txt", "manual\n");
        service
            .commit(&mut repo, &CommitMessage::new("manual resolution"))
            .unwrap();
        assert!(service.status_full(&repo).unwrap().is_empty());
    }

    #[test]
    fn resolve_modify_delete_conflict_can_keep_or_delete_file() {
        let (dir, mut repo, service) = create_modify_delete_conflict();
        service
            .resolve_conflict_with_side(
                &mut repo,
                Path::new("same.txt"),
                ConflictResolutionSide::Ours,
            )
            .unwrap();
        assert_file_text(dir.path(), "same.txt", "main\n");
        assert!(service.conflicts(&repo).unwrap().is_empty());

        let (dir, mut repo, service) = create_modify_delete_conflict();
        service
            .resolve_conflict_with_side(
                &mut repo,
                Path::new("same.txt"),
                ConflictResolutionSide::Theirs,
            )
            .unwrap();
        assert!(!dir.path().join("same.txt").exists());
        assert!(service.conflicts(&repo).unwrap().is_empty());
    }

    #[test]
    fn resolve_conflict_rejects_non_conflicted_path() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "same.txt", "base\n");
        commit_all(&repo, "initial");

        let error = service
            .resolve_conflict_with_side(
                &mut repo,
                Path::new("same.txt"),
                ConflictResolutionSide::Ours,
            )
            .unwrap_err()
            .to_string();

        assert!(error.contains("不存在冲突"));
    }
}
