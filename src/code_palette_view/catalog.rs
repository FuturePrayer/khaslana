//! 搜索目录只持有导航目标；确认后仍调用业务入口，不直接执行危险 Git 操作。

#[path = "../ui_resources/code_palette_view_catalog_messages.rs"]
mod messages;

use crate::{MainMode, RepoTabId, SettingsCategory, ai_extensions_view::AiSettingsTab};
use khaslana::{BranchKind, RepositorySnapshot};
use std::path::PathBuf;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum SearchScope {
    #[default]
    CurrentRepository,
    AllRepositories,
}

impl SearchScope {
    pub fn includes_repository(self, id: RepoTabId, active: Option<RepoTabId>) -> bool {
        self == Self::AllRepositories || active == Some(id)
    }

    pub fn scans_saved_repositories(self, query: &str) -> bool {
        self == Self::AllRepositories && !query.trim().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SearchTarget {
    Mode(RepoTabId, MainMode),
    Feature(RepoTabId, &'static str, MainMode),
    Settings(SettingsCategory),
    AiSettings(AiSettingsTab),
    Repository(RepoTabId),
    SavedRepository(PathBuf),
    SavedBranch(PathBuf, String, BranchKind),
    Branch(RepoTabId, String, BranchKind),
    Tag(RepoTabId, String),
    Remote(RepoTabId, String),
    Stash(RepoTabId, String),
    Workflow(RepoTabId, PathBuf),
    OpenRepository,
    CloneRepository,
    CreateBranch(RepoTabId),
    CreateTag(RepoTabId),
    CreateStash(RepoTabId),
    Submodules(RepoTabId),
    Remotes(RepoTabId),
    Pull(RepoTabId),
    Push(RepoTabId),
    WorkflowEditor(RepoTabId),
    ReviewHistory(RepoTabId),
}

#[derive(Clone, Debug)]
pub(super) struct SearchEntry {
    pub title: String,
    pub subtitle: String,
    pub group: &'static str,
    keywords: &'static str,
    pub target: SearchTarget,
    pub disabled_reason: Option<&'static str>,
}

impl SearchEntry {
    pub fn new(
        title: impl Into<String>,
        subtitle: impl Into<String>,
        group: &'static str,
        keywords: &'static str,
        target: SearchTarget,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            group,
            keywords,
            target,
            disabled_reason: None,
        }
    }

    fn disabled(mut self, reason: Option<&'static str>) -> Self {
        self.disabled_reason = reason;
        self
    }
}

pub(super) fn function_entries(
    tab_id: Option<RepoTabId>,
    busy: bool,
    snapshot: Option<&RepositorySnapshot>,
    ai_enabled: bool,
) -> Vec<SearchEntry> {
    let id = tab_id.unwrap_or(RepoTabId(0));
    let repo_reason = if tab_id.is_none() {
        Some(khaslana::ui_i18n::text(messages::OPEN_A_REPOSITORY_FIRST))
    } else {
        None
    };
    let operation_reason = repo_reason.or(busy.then_some(khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT)));
    let remote_reason = operation_reason
        .or_else(|| {
            snapshot
                .filter(|snapshot| !snapshot.remotes.is_empty())
                .is_none()
                .then_some(khaslana::ui_i18n::text(messages::THIS_REPOSITORY_HAS_NO_AVAILABLE_REMOTE))
        })
        .or_else(|| {
            snapshot
                .is_some_and(|snapshot| snapshot.merge_in_progress)
                .then_some(khaslana::ui_i18n::text(messages::COMPLETE_OR_ABORT_THE_MERGE_FIRST))
        });
    let mut entries = vec![
        SearchEntry::new(
            khaslana::ui_i18n::text(messages::OPEN_REPOSITORY),
            khaslana::ui_i18n::text(messages::CHOOSE_A_LOCAL_GIT_REPOSITORY_FOLDER),
            khaslana::ui_i18n::text(messages::COMMANDS),
            khaslana::ui_i18n::text(messages::OPEN_PROJECT_OPEN_REPOSITORY_REPO),
            SearchTarget::OpenRepository,
        )
        .disabled(busy.then_some(khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT))),
        SearchEntry::new(
            khaslana::ui_i18n::text(messages::CLONE_REPOSITORY),
            khaslana::ui_i18n::text(messages::ENTER_A_REMOTE_ADDRESS_AND_LOCAL_DIRECTORY),
            khaslana::ui_i18n::text(messages::COMMANDS),
            khaslana::ui_i18n::text(messages::DOWNLOAD_PROJECT_CLONE),
            SearchTarget::CloneRepository,
        )
        .disabled(busy.then_some(khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT))),
    ];
    for (title, description, keywords, mode) in [
        (
            khaslana::ui_i18n::text(messages::WORKING_TREE),
            khaslana::ui_i18n::text(messages::VIEW_CHANGES_STAGE_FILES_AND_COMMIT),
            khaslana::ui_i18n::text(messages::CHANGES_COMMIT_STAGE_UNSTAGE_AMEND_COMMIT_STAGE_UNSTAGE_DIFF_REFRESH),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::HISTORY),
            khaslana::ui_i18n::text(messages::VIEW_HISTORY_DIFFERENCES_AND_COMMIT_OPERATIONS),
            khaslana::ui_i18n::text(messages::HISTORY_LOG_UNDO_RESET_REVERT_CHERRY_PICK_RESET_REVERT_LOG),
            MainMode::History,
        ),
        (
            khaslana::ui_i18n::text(messages::COMMIT_GRAPH),
            khaslana::ui_i18n::text(messages::VIEW_BRANCH_LINEAGE_AND_COMMIT_RELATIONSHIPS),
            khaslana::ui_i18n::text(messages::GRAPH_BRANCHES_GRAPH),
            MainMode::CommitGraph,
        ),
        (
            khaslana::ui_i18n::text(messages::WORKFLOWS),
            khaslana::ui_i18n::text(messages::CHOOSE_TEMPLATES_CONFIGURE_PARAMETERS_AND_VIEW_EXECUTION_LOGS),
            khaslana::ui_i18n::text(messages::AUTOMATION_TEMPLATES_WORKFLOW),
            MainMode::Workflow,
        ),
        (
            khaslana::ui_i18n::text(messages::CONFLICTS),
            khaslana::ui_i18n::text(messages::VIEW_CONFLICTS_AND_ACCEPT_BLOCKS_OR_MERGE_WITH_AI),
            khaslana::ui_i18n::text(messages::MERGE_RESOLVE_MERGE_CONFLICT_AI),
            MainMode::Conflict,
        ),
        (
            khaslana::ui_i18n::text(messages::STASH_LIST),
            khaslana::ui_i18n::text(messages::VIEW_SAVED_STASHES_AND_FILE_DIFFERENCES),
            khaslana::ui_i18n::text(messages::STASH_STASH),
            MainMode::Stash,
        ),
        (
            khaslana::ui_i18n::text(messages::BROWSE_FILES),
            khaslana::ui_i18n::text(messages::BROWSE_CURRENT_BRANCH_FILES_AND_COMPARE_DIFFERENCES),
            khaslana::ui_i18n::text(messages::FILES_VIEW_BRANCH_COMPARISON_COMPARE_BROWSE),
            MainMode::Browse,
        ),
        (
            khaslana::ui_i18n::text(messages::ATTRIBUTION),
            khaslana::ui_i18n::text(messages::VIEW_THE_ORIGINATING_COMMIT_FOR_EACH_FILE_LINE),
            khaslana::ui_i18n::text(messages::BLAME_AUTHOR_ATTRIBUTION),
            MainMode::Blame,
        ),
    ] {
        let reason = if mode == MainMode::Conflict {
            repo_reason.or_else(|| {
                snapshot
                    .is_none_or(|snapshot| snapshot.conflicts.is_empty())
                    .then_some(khaslana::ui_i18n::text(messages::THIS_REPOSITORY_HAS_NO_CONFLICTS))
            })
        } else {
            repo_reason
        };
        entries.push(
            SearchEntry::new(
                title,
                description,
                khaslana::ui_i18n::text(messages::COMMANDS),
                keywords,
                SearchTarget::Mode(id, mode),
            )
            .disabled(reason),
        );
    }
    // 需要先选择文件、分支或提交的操作，定位到它们所在页面，不代替用户选对象。
    for (title, description, keywords, mode) in [
        (
            khaslana::ui_i18n::text(messages::COMMIT_CHANGES),
            khaslana::ui_i18n::text(messages::WORKING_TREE_STAGE_CHANGES_AND_ENTER_A_COMMIT_MESSAGE_BELOW),
            khaslana::ui_i18n::text(messages::COMMIT_COMMIT_STAGE_STAGE),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::AI_COMMIT_MESSAGE),
            khaslana::ui_i18n::text(messages::WORKING_TREE_AI_GENERATION_BUTTON_IN_THE_COMMIT_AREA),
            khaslana::ui_i18n::text(messages::COMMIT_MESSAGE_COMMIT_MESSAGE),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::AI_CODE_REVIEW),
            khaslana::ui_i18n::text(messages::WORKING_TREE_SELECT_A_DIFF_AND_OPEN_AI_REVIEW),
            khaslana::ui_i18n::text(messages::REVIEW_REVIEW),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::REFRESH_REPOSITORY),
            khaslana::ui_i18n::text(messages::TITLE_BAR_REFRESH_BUTTON),
            khaslana::ui_i18n::text(messages::REFRESH_REFRESH),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::FETCH_REMOTE_UPDATES),
            khaslana::ui_i18n::text(messages::TITLE_BAR_FETCH_BUTTON),
            khaslana::ui_i18n::text(messages::FETCH_FETCH),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::SWITCH_BRANCH),
            khaslana::ui_i18n::text(messages::WORKING_TREE_SELECT_A_CHECKOUT_TARGET_IN_THE_BRANCH_LIST),
            khaslana::ui_i18n::text(messages::CHECKOUT_CHECKOUT),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::MERGE_BRANCH),
            khaslana::ui_i18n::text(messages::WORKING_TREE_MERGE_ACTION_IN_THE_BRANCH_CONTEXT_MENU),
            "merge",
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::REBASE),
            khaslana::ui_i18n::text(messages::WORKING_TREE_REBASE_ACTION_IN_THE_BRANCH_CONTEXT_MENU),
            "rebase",
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::RENAME_BRANCH),
            khaslana::ui_i18n::text(messages::WORKING_TREE_RENAME_ACTION_IN_THE_BRANCH_CONTEXT_MENU),
            "rename",
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::DELETE_BRANCH),
            khaslana::ui_i18n::text(messages::WORKING_TREE_DELETE_CONFIRMATION_IN_THE_BRANCH_CONTEXT_MENU),
            "delete branch",
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::COMPARE_BRANCHES),
            khaslana::ui_i18n::text(messages::WORKING_TREE_COMPARE_ACTION_IN_THE_BRANCH_CONTEXT_MENU),
            "compare",
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::SET_UPSTREAM_BRANCH),
            khaslana::ui_i18n::text(messages::WORKING_TREE_UPSTREAM_SETTING_IN_THE_BRANCH_CONTEXT_MENU),
            khaslana::ui_i18n::text(messages::UPSTREAM_TRACKING),
            MainMode::Worktree,
        ),
        (
            khaslana::ui_i18n::text(messages::CHERRY_PICK_COMMIT),
            khaslana::ui_i18n::text(messages::HISTORY_SELECT_A_COMMIT_AND_CHERRY_PICK_FROM_ITS_CONTEXT),
            "cherry-pick",
            MainMode::History,
        ),
        (
            khaslana::ui_i18n::text(messages::RESET_TO_COMMIT),
            khaslana::ui_i18n::text(messages::HISTORY_SELECT_A_COMMIT_AND_RESET_FROM_ITS_CONTEXT_MENU),
            "reset",
            MainMode::History,
        ),
        (
            khaslana::ui_i18n::text(messages::REVERT_COMMIT),
            khaslana::ui_i18n::text(messages::HISTORY_SELECT_A_COMMIT_AND_REVERT_FROM_ITS_CONTEXT_MENU),
            khaslana::ui_i18n::text(messages::REVERT_REVERT),
            MainMode::History,
        ),
        (
            khaslana::ui_i18n::text(messages::UNDO_COMMIT),
            khaslana::ui_i18n::text(messages::HISTORY_SELECT_A_COMMIT_AND_UNDO_FROM_ITS_CONTEXT_MENU),
            "uncommit",
            MainMode::History,
        ),
        (
            khaslana::ui_i18n::text(messages::FILE_HISTORY),
            khaslana::ui_i18n::text(messages::WORKING_TREE_HISTORY_ACTION_IN_THE_FILE_CONTEXT_MENU),
            khaslana::ui_i18n::text(messages::HISTORY_HISTORY),
            MainMode::Worktree,
        ),
    ] {
        entries.push(
            SearchEntry::new(
                title,
                description,
                khaslana::ui_i18n::text(messages::COMMANDS),
                keywords,
                SearchTarget::Feature(id, title, mode),
            )
            .disabled(repo_reason),
        );
    }
    entries.push(SearchEntry::new(
        khaslana::ui_i18n::text(messages::SETTINGS_CENTER),
        khaslana::ui_i18n::text(messages::OPEN_APPLICATION_SETTINGS_AND_PREFERENCES),
        khaslana::ui_i18n::text(messages::SETTINGS),
        "settings preferences",
        SearchTarget::Settings(SettingsCategory::Credentials),
    ));
    for category in crate::settings_center::SETTINGS_CATEGORY_ORDER {
        let meta = crate::settings_center::settings_pane_meta(category);
        let keywords = match category {
            SettingsCategory::Credentials => khaslana::ui_i18n::text(messages::SETTINGS_KEYS_TOKENS_ACCOUNTS_SSH_PASSWORDS_CREDENTIALS_TOKEN),
            SettingsCategory::Proxy => khaslana::ui_i18n::text(messages::SETTINGS_PROXY_SOCKS_HTTP_HTTPS_PROXY),
            SettingsCategory::Ai => khaslana::ui_i18n::text(messages::SETTINGS_MODELS_PROVIDERS_APIS_SERVICES_PROVIDER_API_AI),
            SettingsCategory::ExternalMerge => khaslana::ui_i18n::text(messages::SETTINGS_EXTERNAL_MERGE_INTELLIJ_IDEA_MERGE_TOOL),
            SettingsCategory::CodeIndex => khaslana::ui_i18n::text(messages::SETTINGS_CODE_INDEX_KNOWLEDGE_GRAPH_REBUILD_CODE_INDEX),
            SettingsCategory::Theme => khaslana::ui_i18n::text(messages::SETTINGS_THEME_DARK_LIGHT_COLORS_FONTS_THEME_APPEARANCE),
            SettingsCategory::Update => khaslana::ui_i18n::text(messages::SETTINGS_VERSION_UPDATE_CHECKS_DATA_DIRECTORY_MIGRATION_UPDATE),
            SettingsCategory::Shortcuts => khaslana::ui_i18n::text(messages::SETTINGS_KEYS_KEYBOARD_SHORTCUTS_KEYBOARD_SHORTCUT),
            SettingsCategory::About => khaslana::ui_i18n::text(messages::SETTINGS_ABOUT_VERSION_INFORMATION_ABOUT),
        };
        entries.push(SearchEntry::new(
            meta.title,
            meta.description,
            khaslana::ui_i18n::text(messages::SETTINGS),
            keywords,
            SearchTarget::Settings(category),
        ));
    }
    for (title, description, keywords, tab) in [
        (
            khaslana::ui_i18n::text(messages::MANAGE_SKILLS),
            khaslana::ui_i18n::text(messages::AI_SETTINGS_IMPORT_VIEW_AND_MANAGE_SKILLS),
            khaslana::ui_i18n::text(messages::SKILLS_EXTENSIONS_SKILLS),
            AiSettingsTab::Skill,
        ),
        (
            khaslana::ui_i18n::text(messages::MCP_SERVICES),
            khaslana::ui_i18n::text(messages::AI_SETTINGS_CONFIGURE_SERVERS_AND_TOOL_PERMISSIONS),
            khaslana::ui_i18n::text(messages::TOOLS_EXTENSIONS_MCP_SERVER),
            AiSettingsTab::Mcp,
        ),
        (
            khaslana::ui_i18n::text(messages::BROWSER_RUNTIME),
            khaslana::ui_i18n::text(messages::AI_SETTINGS_CONFIGURE_THE_WORKFLOW_BROWSER_RUNTIME),
            "browser runtime playwright",
            AiSettingsTab::Runtime,
        ),
    ] {
        entries.push(
            SearchEntry::new(
                title,
                description,
                khaslana::ui_i18n::text(messages::SETTINGS),
                keywords,
                SearchTarget::AiSettings(tab),
            )
            .disabled((!ai_enabled).then_some(khaslana::ui_i18n::text(messages::ENABLE_AI_IN_AI_SETTINGS_FIRST))),
        );
    }
    for (title, description, keywords, target, reason) in [
        (
            khaslana::ui_i18n::text(messages::CREATE_BRANCH),
            khaslana::ui_i18n::text(messages::OPEN_THE_NEW_BRANCH_FORM),
            khaslana::ui_i18n::text(messages::CREATE_BRANCH_2),
            SearchTarget::CreateBranch(id),
            operation_reason,
        ),
        (
            khaslana::ui_i18n::text(messages::CREATE_TAG),
            khaslana::ui_i18n::text(messages::CREATE_A_TAG_FOR_THE_CURRENT_HEAD),
            khaslana::ui_i18n::text(messages::CREATE_TAG_2),
            SearchTarget::CreateTag(id),
            operation_reason,
        ),
        (
            khaslana::ui_i18n::text(messages::CREATE_STASH),
            khaslana::ui_i18n::text(messages::OPEN_THE_STASH_FORM_TO_SAVE_CURRENT_CHANGES),
            khaslana::ui_i18n::text(messages::STASH_STASH),
            SearchTarget::CreateStash(id),
            operation_reason.or_else(|| {
                snapshot
                    .is_some_and(|snapshot| snapshot.merge_in_progress)
                    .then_some(khaslana::ui_i18n::text(messages::COMPLETE_OR_ABORT_THE_MERGE_FIRST))
            }),
        ),
        (
            khaslana::ui_i18n::text(messages::SUBMODULES),
            khaslana::ui_i18n::text(messages::MANAGE_INITIALIZE_AND_UPDATE_SUBMODULES),
            "submodule",
            SearchTarget::Submodules(id),
            operation_reason,
        ),
        (
            khaslana::ui_i18n::text(messages::MANAGE_REMOTES),
            khaslana::ui_i18n::text(messages::ADD_AND_EDIT_REMOTES_AND_CONFIGURE_CREDENTIALS),
            khaslana::ui_i18n::text(messages::REMOTE_REMOTE_ADDRESS_FETCH_FETCH),
            SearchTarget::Remotes(id),
            operation_reason,
        ),
        (
            khaslana::ui_i18n::text(messages::PULL),
            khaslana::ui_i18n::text(messages::SELECT_A_REMOTE_AND_BRANCH_TO_PULL),
            khaslana::ui_i18n::text(messages::PULL_DOWNLOAD),
            SearchTarget::Pull(id),
            remote_reason,
        ),
        (
            khaslana::ui_i18n::text(messages::PUSH),
            khaslana::ui_i18n::text(messages::SELECT_A_REMOTE_AND_BRANCH_TO_PUSH),
            khaslana::ui_i18n::text(messages::PUSH_UPLOAD),
            SearchTarget::Push(id),
            remote_reason,
        ),
        (
            khaslana::ui_i18n::text(messages::WORKFLOW_EDITOR),
            khaslana::ui_i18n::text(messages::CREATE_OR_EDIT_A_WORKFLOW_TEMPLATE),
            khaslana::ui_i18n::text(messages::TEMPLATE_CREATE_MODIFY_EDIT_WORKFLOW_EDITOR),
            SearchTarget::WorkflowEditor(id),
            repo_reason,
        ),
        (
            khaslana::ui_i18n::text(messages::AI_REVIEW_HISTORY),
            khaslana::ui_i18n::text(messages::VIEW_SAVED_CODE_REVIEW_REPORTS),
            khaslana::ui_i18n::text(messages::REVIEW_REVIEW),
            SearchTarget::ReviewHistory(id),
            repo_reason,
        ),
    ] {
        entries
            .push(SearchEntry::new(title, description, khaslana::ui_i18n::text(messages::COMMANDS), keywords, target).disabled(reason));
    }
    entries
}

pub(super) fn append_repository_entries(
    entries: &mut Vec<SearchEntry>,
    id: RepoTabId,
    repo_name: &str,
    snapshot: &RepositorySnapshot,
) {
    for branch in &snapshot.branches {
        let group = if branch.kind == BranchKind::Local {
            khaslana::ui_i18n::text(messages::LOCAL_BRANCHES)
        } else {
            khaslana::ui_i18n::text(messages::REMOTE_BRANCHES)
        };
        entries.push(SearchEntry::new(
            &branch.name,
            khaslana::ui_i18n::format(if branch.is_head { messages::HIGHLIGHT_CURRENT_BRANCH } else { messages::HIGHLIGHT_BRANCH }, &[("repo_name", repo_name)]),
            group,
            khaslana::ui_i18n::text(messages::BRANCH_BRANCH),
            SearchTarget::Branch(id, branch.name.clone(), branch.kind.clone()),
        ));
    }
    for tag in &snapshot.tags {
        entries.push(SearchEntry::new(
            &tag.name,
            khaslana::ui_i18n::format(messages::REPO_NAME_BROWSE_TAGGED_FILES, &[("repo_name", &(repo_name).to_string())]),
            khaslana::ui_i18n::text(messages::TAGS),
            "tag",
            SearchTarget::Tag(id, tag.name.clone()),
        ));
    }
    for remote in &snapshot.remotes {
        entries.push(SearchEntry::new(
            &remote.name,
            format!("{repo_name} · {}", remote.url),
            khaslana::ui_i18n::text(messages::REMOTES),
            "remote",
            SearchTarget::Remote(id, remote.name.clone()),
        ));
    }
    for stash in &snapshot.stashes {
        entries.push(SearchEntry::new(
            format!("stash@{{{}}} {}", stash.index, stash.message),
            khaslana::ui_i18n::format(messages::REPO_NAME_VIEW_STASH_DIFFERENCES, &[("repo_name", &(repo_name).to_string())]),
            khaslana::ui_i18n::text(messages::STASHES),
            khaslana::ui_i18n::text(messages::STASH_STASH_2),
            SearchTarget::Stash(id, stash.oid.clone()),
        ));
    }
}

/// 名称精确匹配优先于前缀、子串和说明/别名；多词查询需要全部命中。
/// 不按固定数量截断结果，滚动与大仓库列表由 Kit 虚拟化处理。
pub(super) fn filter_entries(entries: Vec<SearchEntry>, query: &str) -> Vec<SearchEntry> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return entries
            .into_iter()
            .filter(|entry| {
                matches!(entry.target, SearchTarget::Settings(_) | SearchTarget::AiSettings(_))
            })
            .collect();
    }
    let tokens: Vec<_> = query.split_whitespace().collect();
    let mut matches: Vec<_> = entries
        .into_iter()
        .filter_map(|entry| {
            let title = entry.title.to_lowercase();
            let extra =
                format!("{} {} {}", entry.subtitle, entry.group, entry.keywords).to_lowercase();
            let mut score = 0;
            for token in &tokens {
                score += if title == *token {
                    0
                } else if title.starts_with(token) {
                    1
                } else if title.contains(token) {
                    2
                } else if extra.contains(token) {
                    3
                } else {
                    return None;
                };
            }
            Some((score, entry))
        })
        .collect();
    matches.sort_by_key(|(score, _)| *score);
    matches.into_iter().map(|(_, entry)| entry).collect()
}

pub(super) fn next_selection(
    entries: &[SearchEntry],
    selected: Option<usize>,
    forward: bool,
) -> Option<usize> {
    let enabled: Vec<_> = entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| entry.disabled_reason.is_none().then_some(index))
        .collect();
    let position =
        selected.and_then(|selected| enabled.iter().position(|index| *index == selected));
    match position {
        Some(position) => enabled
            .get(if forward {
                (position + 1).min(enabled.len() - 1)
            } else {
                position.saturating_sub(1)
            })
            .copied(),
        None => {
            if forward {
                enabled.first().copied()
            } else {
                enabled.last().copied()
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/code_palette.rs"]
mod tests;
