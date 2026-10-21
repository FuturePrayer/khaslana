use git2::Repository;
#[path = "remote_branch_guard_resources.rs"]
mod deep_messages;

use serde::{Deserialize, Serialize};

use crate::{GitError, Result};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RemoteBranchGuardAction {
    Fail,
    Continue,
}

impl RemoteBranchGuardAction {
    pub(super) fn should_fail(self) -> bool {
        matches!(self, Self::Fail)
    }
}

pub(super) fn default_on_exists() -> RemoteBranchGuardAction {
    RemoteBranchGuardAction::Fail
}

pub(super) fn default_on_missing() -> RemoteBranchGuardAction {
    RemoteBranchGuardAction::Continue
}

pub(super) fn default_guard_fetch() -> bool {
    true
}

pub(super) fn validate_remote_branch_name(remote: &str, branch: &str) -> Result<()> {
    let branch = branch.trim();
    if branch.is_empty() {
        return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::REMOTE_BRANCH_NAME_CANNOT_BE_EMPTY, &[])));
    }
    if branch
        .strip_prefix(remote)
        .is_some_and(|rest| rest.starts_with('/'))
    {
        return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::DO_NOT_PREFIX_THE_BRANCH_NAME_WITH, &[("remote", &(remote).to_string()), ("branch", &(branch).to_string())])));
    }
    if branch.starts_with('/') || branch.ends_with('/') || branch.contains("//") {
        return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::INVALID_REMOTE_BRANCH_NAME, &[("branch", &(branch).to_string())])));
    }
    Ok(())
}

pub(super) fn remote_branch_exists(repo: &Repository, remote: &str, branch: &str) -> Result<bool> {
    validate_remote_branch_name(remote, branch)?;
    let refname = format!("refs/remotes/{remote}/{branch}");
    match repo.find_reference(&refname) {
        Ok(_) => Ok(true),
        Err(err) if err.code() == git2::ErrorCode::NotFound => Ok(false),
        Err(err) => Err(GitError::Git(err)),
    }
}

pub(super) fn guard_remote_branch(
    repo: &Repository,
    remote: &str,
    branch: &str,
    on_exists: RemoteBranchGuardAction,
    on_missing: RemoteBranchGuardAction,
) -> Result<()> {
    let exists = remote_branch_exists(repo, remote, branch)?;
    if exists && on_exists.should_fail() {
        return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::REMOTE_BRANCH_ALREADY_EXISTS, &[("remote", &(remote).to_string()), ("branch", &(branch).to_string())])));
    }
    if !exists && on_missing.should_fail() {
        return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::REMOTE_BRANCH_DOES_NOT_EXIST, &[("remote", &(remote).to_string()), ("branch", &(branch).to_string())])));
    }
    Ok(())
}

pub(super) fn guard_summary(
    remote: &str,
    branch: &str,
    fetch: bool,
    on_exists: RemoteBranchGuardAction,
    on_missing: RemoteBranchGuardAction,
) -> String {
    let refresh = if fetch {
        crate::ui_i18n::encode(deep_messages::AFTER_FETCHING, &[])
    } else {
        crate::ui_i18n::encode(deep_messages::USING_LOCAL_REFERENCES, &[])
    };
    let exists = match on_exists {
        RemoteBranchGuardAction::Fail => crate::ui_i18n::encode(deep_messages::STOP_IF_PRESENT, &[]),
        RemoteBranchGuardAction::Continue => crate::ui_i18n::encode(deep_messages::CONTINUE_IF_PRESENT, &[]),
    };
    let missing = match on_missing {
        RemoteBranchGuardAction::Fail => crate::ui_i18n::encode(deep_messages::STOP_IF_MISSING, &[]),
        RemoteBranchGuardAction::Continue => crate::ui_i18n::encode(deep_messages::CONTINUE_IF_MISSING, &[]),
    };
    crate::ui_i18n::encode_with_messages(deep_messages::CHECK_REMOTE_BRANCH,
        &[("remote", remote), ("branch", branch)],
        &[("refresh", &refresh), ("exists", &exists), ("missing", &missing)])
}

#[cfg(test)]
#[path = "../tests/workflow/remote_branch_guard.rs"]
mod tests;
