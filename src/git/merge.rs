//! 普通合并会话：开始、完成和中止。

use crate::types::domain_ui_resources;
use git2::build::CheckoutBuilder;
use git2::{AnnotatedCommit, MergeAnalysis, MergeOptions, Repository, RepositoryState, ResetType};

use super::worktree_compat::{
    checkout_tree_preserving_locked_directories, merge_preserving_locked_directories,
    reset_preserving_locked_directories,
};
use super::{GitService, signature};
use crate::types::{
    BranchName, CommitMessage, GitError, OperationEvent, RepositorySnapshot, Result,
};

impl GitService {
    pub fn merge_branch(
        &self,
        repo: &mut Repository,
        branch: &BranchName,
    ) -> Result<RepositorySnapshot> {
        self.progress
            .emit(OperationEvent::Started(crate::ui_i18n::format(domain_ui_resources::MERGING, &[("value0", &format!("{}", branch.0))])));
        let reference = self.find_branch_reference(repo, &branch.0)?;
        let annotated = repo.reference_to_annotated_commit(&reference)?;
        self.merge_annotated(repo, &annotated, &branch.0)?;
        drop(annotated);
        drop(reference);
        // 无冲突的正常非快进合并：自动提交（双父提交），不保留 merge 会话。
        self.complete_clean_merge(repo, &branch.0)?;
        let message = if merge_in_progress(repo) {
            crate::ui_i18n::format(domain_ui_resources::MERGE_RESULT_FOR_WRITTEN_TO_THE_INDEX, &[("value0", &format!("{}", branch.0))])
        } else {
            crate::ui_i18n::format(domain_ui_resources::MERGED, &[("value0", &format!("{}", branch.0))])
        };
        self.progress.emit(OperationEvent::Finished(message));
        self.snapshot_after_operation(repo)
    }

    /// 完成无冲突的非快进合并：处于合并态（MERGE_HEAD 存在，即合并结果已
    /// 暂存等待提交）时，收集 MERGE_HEAD 父提交并创建双父提交，成功后清理
    /// 合并状态；不在合并态（快进/已最新）则原样返回。
    ///
    /// pull 系列与显式合并共用：干净合并不留待确认会话，避免用户把
    /// “完成合并/中止合并”界面误认为冲突处理模式。提交失败时状态保留，
    /// 可通过“完成合并”重试（`cleanup_state` 仅提交成功后执行）。
    pub(super) fn complete_clean_merge(
        &self,
        repo: &mut Repository,
        merge_label: &str,
    ) -> Result<()> {
        if !merge_in_progress(repo) {
            return Ok(());
        }
        let message =
            merge_message(repo).unwrap_or_else(|| format!("Merge branch '{merge_label}'"));
        let mut merge_head_ids = Vec::new();
        repo.mergehead_foreach(|oid| {
            merge_head_ids.push(*oid);
            true
        })?;
        self.commit_merge(repo, &CommitMessage::new(message), &merge_head_ids)
    }

    pub(super) fn merge_annotated(
        &self,
        repo: &Repository,
        annotated: &AnnotatedCommit<'_>,
        label: &str,
    ) -> Result<()> {
        self.ensure_clean_for_merge(repo)?;
        let (analysis, _preference) = repo.merge_analysis(&[annotated])?;

        if analysis.contains(MergeAnalysis::ANALYSIS_UP_TO_DATE) {
            return Ok(());
        }
        if analysis.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) {
            fast_forward(repo, annotated)?;
            return Ok(());
        }
        if !analysis.contains(MergeAnalysis::ANALYSIS_NORMAL) {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::CANNOT_MERGE_UNSUPPORTED_MERGE_ANALYSIS_RESULT, &[("label", &format!("{}", label))])));
        }

        let mut merge_options = MergeOptions::new();
        let mut checkout = CheckoutBuilder::new();
        checkout
            .safe()
            .allow_conflicts(true)
            .conflict_style_merge(true);
        merge_preserving_locked_directories(
            repo,
            &[annotated],
            Some(&mut merge_options),
            &mut checkout,
        )?;

        if repo.index()?.has_conflicts() {
            // 保留 MERGE_HEAD、MERGE_MSG 和冲突 index，等待用户解决并完成合并。
            return Err(GitError::Conflicts(self.conflicts(repo)?));
        }

        // 无冲突的正常合并：由 merge_branch 自动提交（需要 &mut Repository 调用 mergehead_foreach）。
        Ok(())
    }

    pub fn finish_merge(
        &self,
        repo: &mut Repository,
        message: &CommitMessage,
    ) -> Result<RepositorySnapshot> {
        if !merge_in_progress(repo) {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::NO_MERGE_IS_CURRENTLY_IN_PROGRESS, &[])));
        }
        self.progress
            .emit(OperationEvent::Started(crate::ui_i18n::text(domain_ui_resources::COMPLETING_MERGE).into()));
        let mut merge_head_ids = Vec::new();
        repo.mergehead_foreach(|oid| {
            merge_head_ids.push(*oid);
            true
        })?;
        self.commit_merge(repo, message, &merge_head_ids)?;
        self.progress
            .emit(OperationEvent::Finished(crate::ui_i18n::text(domain_ui_resources::MERGE_COMMIT_CREATED).into()));
        self.snapshot_after_operation(repo)
    }

    pub fn abort_merge(&self, repo: &mut Repository) -> Result<RepositorySnapshot> {
        if !merge_in_progress(repo) {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::NO_MERGE_IS_CURRENTLY_IN_PROGRESS, &[])));
        }

        self.progress
            .emit(OperationEvent::Started(crate::ui_i18n::text(domain_ui_resources::ABORTING_MERGE).into()));
        let head = repo.head()?.peel_to_commit()?;
        reset_preserving_locked_directories(repo, head.as_object(), ResetType::Hard)?;
        drop(head);
        repo.cleanup_state()?;
        self.progress
            .emit(OperationEvent::Finished(crate::ui_i18n::text(domain_ui_resources::MERGE_ABORTED).into()));
        self.snapshot_after_operation(repo)
    }

    fn ensure_clean_for_merge(&self, repo: &Repository) -> Result<()> {
        if repo.state() != RepositoryState::Clean {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::ANOTHER_GIT_OPERATION_IS_IN_PROGRESS_COMPLETE_OR_ABORT_IT_BEFORE_MERGING, &[])));
        }
        if !self.status_full(repo)?.is_empty() || !self.conflicts(repo)?.is_empty() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THERE_ARE_UNCOMMITTED_CHANGES_COMMIT_OR_STASH_THEM_BEFORE_MERGING, &[])));
        }
        Ok(())
    }

    fn commit_merge(
        &self,
        repo: &Repository,
        message: &CommitMessage,
        merge_head_ids: &[git2::Oid],
    ) -> Result<()> {
        if !merge_in_progress(repo) {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::NO_MERGE_IS_CURRENTLY_IN_PROGRESS, &[])));
        }

        let message = message.0.trim();
        if message.is_empty() {
            return Err(GitError::EmptyCommitMessage);
        }

        let mut index = repo.index()?;
        if index.has_conflicts() {
            return Err(GitError::Conflicts(self.conflicts(repo)?));
        }
        let tree_id = index.write_tree()?;
        if merge_head_ids.is_empty() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::MERGE_STATE_IS_MISSING_MERGE_HEAD_CANNOT_COMPLETE_THE_MERGE, &[])));
        }
        let tree = repo.find_tree(tree_id)?;
        let head = repo.head()?.peel_to_commit()?;
        let merge_heads = merge_head_ids
            .iter()
            .map(|oid| repo.find_commit(*oid))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut parents = Vec::with_capacity(merge_heads.len() + 1);
        parents.push(&head);
        parents.extend(merge_heads.iter());
        let signature = signature(repo)?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;
        // 只有提交成功后才清理状态，失败时用户仍可修改信息后重试或中止。
        repo.cleanup_state()?;
        Ok(())
    }
}

pub(super) fn merge_in_progress(repo: &Repository) -> bool {
    repo.state() == RepositoryState::Merge
}

pub(super) fn merge_message(repo: &Repository) -> Option<String> {
    merge_in_progress(repo)
        .then(|| repo.message().ok())
        .flatten()
        .map(|message| message.trim().to_string())
        .filter(|message| !message.is_empty())
}

fn fast_forward(repo: &Repository, annotated: &AnnotatedCommit<'_>) -> Result<()> {
    let refname = repo.head()?.name().map_err(GitError::from)?.to_string();
    let target = repo.find_object(annotated.id(), None)?;
    let mut checkout = CheckoutBuilder::new();
    checkout.safe();
    checkout_tree_preserving_locked_directories(repo, &target, &mut checkout)?;

    let mut reference = repo.find_reference(&refname)?;
    reference.set_target(annotated.id(), "khaslana fast-forward")?;
    repo.set_head(&refname)?;
    Ok(())
}
