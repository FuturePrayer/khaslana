#![cfg_attr(windows, windows_subsystem = "windows")]

mod history_view;
mod sidebar_view;
mod ui_helpers;

use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use async_channel::{Receiver, Sender};
use directories::ProjectDirs;
use git2::Repository;
use gpui::{
    App, Application, Bounds, ClipboardItem, Context, CursorStyle, FocusHandle, Focusable,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, SharedString,
    TitlebarOptions, WeakEntity, Window, WindowBounds, WindowOptions, canvas, div, prelude::*, px,
    rgb, rgba, size,
};
use khaslana::{
    BranchKind, BranchName, CommitFileChange, CommitInfo, CommitMessage, CredentialProvider,
    CredentialRecord, CredentialRequest, CredentialScope, CredentialStore, DiffLineKind, DiffScope,
    FileDiff, GitCredential, GitService, KeyringCredentialStore, OperationEvent, ProgressEmitter,
    RemoteName, RepoPath, RepositorySnapshot, ResetMode, TagName, credential_display_target,
    credential_key_filename, credential_kind_label, credential_record_label,
    credential_scope_label, test_credential_connection,
};
use serde::{Deserialize, Serialize};
use ui_helpers::*;

const DEFAULT_SIDEBAR_WIDTH: f32 = 330.0;
const DEFAULT_CHANGES_WIDTH: f32 = 330.0;
const MIN_COLUMN_WIDTH: f32 = 240.0;
const MAX_COLUMN_WIDTH: f32 = 640.0;
const CHANGE_ROW_HEIGHT: f32 = 36.0;
const DEFAULT_HISTORY_TOP_HEIGHT: f32 = 430.0;
const MIN_HISTORY_TOP_HEIGHT: f32 = 180.0;
const MAX_HISTORY_TOP_HEIGHT: f32 = 760.0;
const DEFAULT_HISTORY_FILES_WIDTH: f32 = 520.0;
const MIN_HISTORY_FILES_WIDTH: f32 = 260.0;
const MAX_HISTORY_FILES_WIDTH: f32 = 720.0;
const HISTORY_PAGE_SIZE: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldId {
    CloneUrl,
    ClonePath,
    BranchName,
    BranchRename,
    CommitMessage,
    CredentialUsername,
    CredentialSecret,
    CredentialKeyPath,
    CredentialPassphrase,
}

#[derive(Clone, Debug)]
struct TextFieldState {
    focus: FocusHandle,
    value: String,
    placeholder: SharedString,
    secret: bool,
}

impl TextFieldState {
    fn new(cx: &mut Context<RepositoryView>, placeholder: impl Into<SharedString>) -> Self {
        Self {
            focus: cx.focus_handle().tab_stop(true),
            value: String::new(),
            placeholder: placeholder.into(),
            secret: false,
        }
    }

    fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    fn display(&self) -> String {
        if self.value.is_empty() {
            self.placeholder.to_string()
        } else if self.secret {
            "*".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }
}

#[derive(Clone, Debug)]
struct PendingCredential {
    tab_id: Option<RepoTabId>,
    request: CredentialRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DialogState {
    CloneRepo,
    CreateBranch,
    RenameBranch {
        branch: String,
    },
    ConfirmReset {
        oid: String,
        summary: String,
        mode: ResetMode,
    },
    ConfirmRevert {
        oid: String,
        summary: String,
    },
    CredentialManager,
    ConfirmDeleteCredential {
        record_id: String,
        label: String,
    },
}

#[derive(Clone, Debug)]
pub(crate) struct BranchContextMenu {
    pub(crate) branch: String,
    pub(crate) kind: BranchKind,
    pub(crate) is_head: bool,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct TagContextMenu {
    pub(crate) tag: String,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct StashContextMenu {
    pub(crate) index: usize,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct CommitContextMenu {
    pub(crate) oid: String,
    pub(crate) short_oid: String,
    pub(crate) summary: String,
    pub(crate) parent_count: usize,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChangeSelection {
    staged: BTreeSet<String>,
    unstaged: BTreeSet<String>,
    staged_anchor: Option<String>,
    unstaged_anchor: Option<String>,
}

impl ChangeSelection {
    fn clear(&mut self) {
        self.staged.clear();
        self.unstaged.clear();
        self.staged_anchor = None;
        self.unstaged_anchor = None;
    }

    fn selected(&self, scope: &DiffScope) -> &BTreeSet<String> {
        match scope {
            DiffScope::Staged => &self.staged,
            DiffScope::Unstaged => &self.unstaged,
        }
    }

    fn selected_mut(&mut self, scope: &DiffScope) -> &mut BTreeSet<String> {
        match scope {
            DiffScope::Staged => &mut self.staged,
            DiffScope::Unstaged => &mut self.unstaged,
        }
    }

    fn anchor(&self, scope: &DiffScope) -> Option<&String> {
        match scope {
            DiffScope::Staged => self.staged_anchor.as_ref(),
            DiffScope::Unstaged => self.unstaged_anchor.as_ref(),
        }
    }

    fn set_anchor(&mut self, scope: &DiffScope, path: String) {
        match scope {
            DiffScope::Staged => self.staged_anchor = Some(path),
            DiffScope::Unstaged => self.unstaged_anchor = Some(path),
        }
    }
}

impl Default for ChangeSelection {
    fn default() -> Self {
        Self {
            staged: BTreeSet::new(),
            unstaged: BTreeSet::new(),
            staged_anchor: None,
            unstaged_anchor: None,
        }
    }
}

#[derive(Clone, Debug)]
struct ChangeContextMenu {
    scope: DiffScope,
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RepoTabId(u64);

#[derive(Clone, Debug)]
struct RepoTabState {
    pub(crate) id: RepoTabId,
    pub(crate) repo_path: Option<PathBuf>,
    pub(crate) snapshot: Option<RepositorySnapshot>,
    pub(crate) selected_branch: Option<String>,
    pub(crate) selected_remote: Option<String>,
    pub(crate) change_selection: ChangeSelection,
    pub(crate) diff: Option<FileDiff>,
    pub(crate) diff_headers_expanded: bool,
    pub(crate) main_mode: MainMode,
    pub(crate) history_commits: Vec<CommitInfo>,
    pub(crate) history_has_more: bool,
    pub(crate) history_selected_commit: Option<String>,
    pub(crate) history_files: Vec<CommitFileChange>,
    pub(crate) history_selected_file: Option<String>,
    pub(crate) history_diff: Option<FileDiff>,
    pub(crate) history_diff_headers_expanded: bool,
    pub(crate) history_loading: HistoryLoading,
    pub(crate) busy: bool,
    pub(crate) loading: RepositoryLoading,
    pub(crate) repository_load_id: u64,
    pub(crate) status: String,
    pub(crate) last_error: Option<String>,
}

impl RepoTabState {
    fn new(id: RepoTabId, repo_path: Option<PathBuf>) -> Self {
        Self {
            id,
            repo_path,
            snapshot: None,
            selected_branch: None,
            selected_remote: None,
            change_selection: ChangeSelection::default(),
            diff: None,
            diff_headers_expanded: false,
            main_mode: MainMode::Worktree,
            history_commits: Vec::new(),
            history_has_more: false,
            history_selected_commit: None,
            history_files: Vec::new(),
            history_selected_file: None,
            history_diff: None,
            history_diff_headers_expanded: false,
            history_loading: HistoryLoading::default(),
            busy: false,
            loading: RepositoryLoading::default(),
            repository_load_id: 0,
            status: "就绪".to_string(),
            last_error: None,
        }
    }

    fn display_name(&self) -> String {
        self.repo_path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "未命名仓库".to_string())
    }

    fn path_key(&self) -> Option<String> {
        self.repo_path
            .as_ref()
            .map(|path| normalize_repo_path(path))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct SessionState {
    repo_paths: Vec<PathBuf>,
    active_repo_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug)]
struct ResizeState {
    start_x: f32,
    start_y: f32,
    start_width: f32,
    start_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ResizeTarget {
    Sidebar,
    Changes,
    HistoryFiles,
    HistoryTop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainMode {
    Worktree,
    History,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiffHeaderTarget {
    Worktree,
    History,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HistoryLoading {
    commits: bool,
    files: bool,
    diff: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RepositoryLoading {
    metadata: bool,
    status_fast: bool,
    status_full: bool,
}

impl RepositoryLoading {
    pub(crate) fn remote(self) -> bool {
        self.metadata
    }

    fn unstaged(self) -> bool {
        self.status_fast || self.status_full
    }

    fn staged(self) -> bool {
        self.status_fast
    }
}

#[derive(Clone, Debug)]
enum UiEvent {
    OperationStarted {
        tab_id: Option<RepoTabId>,
        message: String,
    },
    OperationProgress {
        tab_id: Option<RepoTabId>,
        message: String,
    },
    RepositoryFastLoaded {
        tab_id: RepoTabId,
        message: String,
        snapshot: RepositorySnapshot,
        load_id: u64,
    },
    RepositoryMetadataLoaded {
        tab_id: RepoTabId,
        message: String,
        snapshot: RepositorySnapshot,
        load_id: u64,
    },
    RepositoryStatusFastLoaded {
        tab_id: RepoTabId,
        message: String,
        changes: Vec<khaslana::WorktreeChange>,
        load_id: u64,
    },
    RepositoryStatusFullLoaded {
        tab_id: RepoTabId,
        message: String,
        changes: Vec<khaslana::WorktreeChange>,
        load_id: u64,
    },
    RepositoryLoadStageFailed {
        tab_id: RepoTabId,
        error: String,
        load_id: u64,
    },
    OperationFinished {
        tab_id: Option<RepoTabId>,
        message: String,
        snapshot: Option<RepositorySnapshot>,
        diff: Option<FileDiff>,
    },
    HistoryCommitsLoaded {
        tab_id: RepoTabId,
        commits: Vec<CommitInfo>,
        append: bool,
        has_more: bool,
        load_id: u64,
    },
    HistoryFilesLoaded {
        tab_id: RepoTabId,
        commit_oid: String,
        files: Vec<CommitFileChange>,
        load_id: u64,
    },
    HistoryDiffLoaded {
        tab_id: RepoTabId,
        commit_oid: String,
        path: String,
        diff: FileDiff,
        load_id: u64,
    },
    HistoryLoadFailed {
        tab_id: RepoTabId,
        error: String,
        load_id: u64,
    },
    OperationFailed {
        tab_id: Option<RepoTabId>,
        error: String,
    },
    CredentialRecordsLoaded {
        records: Vec<CredentialRecord>,
        message: String,
    },
    CredentialRequested {
        tab_id: Option<RepoTabId>,
        request: CredentialRequest,
    },
}

#[derive(Clone)]
struct TabProgress {
    tx: Sender<UiEvent>,
    tab_id: RepoTabId,
}

impl ProgressEmitter for TabProgress {
    fn emit(&self, event: OperationEvent) {
        let event = match event {
            OperationEvent::Started(message) => UiEvent::OperationStarted {
                tab_id: Some(self.tab_id),
                message,
            },
            OperationEvent::Progress(message) => UiEvent::OperationProgress {
                tab_id: Some(self.tab_id),
                message,
            },
            OperationEvent::Finished(message) => UiEvent::OperationProgress {
                tab_id: Some(self.tab_id),
                message,
            },
        };
        send_ui_event(&self.tx, event);
    }
}

#[derive(Clone)]
struct TabCredentialProvider {
    store: Arc<dyn khaslana::CredentialStore>,
    tx: Sender<UiEvent>,
    supplied: Arc<Mutex<Option<GitCredential>>>,
    rejected_record_ids: Arc<Mutex<Vec<String>>>,
    last_stored_attempt: Arc<Mutex<Option<(String, String)>>>,
    tab_id: RepoTabId,
}

impl TabCredentialProvider {
    fn new(
        store: Arc<dyn khaslana::CredentialStore>,
        tx: Sender<UiEvent>,
        supplied: Arc<Mutex<Option<GitCredential>>>,
        tab_id: RepoTabId,
    ) -> Self {
        Self {
            store,
            tx,
            supplied,
            rejected_record_ids: Arc::new(Mutex::new(Vec::new())),
            last_stored_attempt: Arc::new(Mutex::new(None)),
            tab_id,
        }
    }
}

impl CredentialProvider for TabCredentialProvider {
    fn credential_for(
        &self,
        request: CredentialRequest,
    ) -> khaslana::Result<Option<GitCredential>> {
        if let Ok(mut last) = self.last_stored_attempt.lock()
            && let Some((url, record_id)) = last.clone()
            && url == request.url
        {
            if let Ok(mut rejected) = self.rejected_record_ids.lock()
                && !rejected.contains(&record_id)
            {
                rejected.push(record_id.clone());
            }
            if let Err(err) = self.store.delete_record(&record_id) {
                tracing::warn!("rejected credential delete skipped: {err}");
            }
            *last = None;
        }

        let rejected_record_ids = self
            .rejected_record_ids
            .lock()
            .map(|rejected| rejected.clone())
            .unwrap_or_default();
        match self.store.get_stored(&request, &rejected_record_ids) {
            Ok(Some(stored)) => {
                if let Ok(mut last) = self.last_stored_attempt.lock() {
                    *last = Some((request.url.clone(), stored.record.id.clone()));
                }
                return Ok(Some(stored.credential));
            }
            Ok(None) => {}
            Err(err) => tracing::warn!("keyring read skipped: {err}"),
        }

        let supplied = self
            .supplied
            .lock()
            .map_err(|_| khaslana::GitError::Credential("凭据输入状态异常".into()))?
            .take();

        if let Some(credential) = supplied {
            if credential.should_save() {
                match self.store.save_record(&request, &credential) {
                    Ok(record) => {
                        if let Ok(mut last) = self.last_stored_attempt.lock() {
                            *last = Some((request.url.clone(), record.id));
                        }
                    }
                    Err(err) => {
                        tracing::warn!("keyring save skipped: {err}");
                        if let Ok(mut last) = self.last_stored_attempt.lock() {
                            *last = None;
                        }
                    }
                }
            } else if let Ok(mut last) = self.last_stored_attempt.lock() {
                *last = None;
            }
            return Ok(Some(credential));
        }

        send_ui_event(
            &self.tx,
            UiEvent::CredentialRequested {
                tab_id: Some(self.tab_id),
                request,
            },
        );
        Ok(None)
    }
}

fn send_ui_event(tx: &Sender<UiEvent>, event: UiEvent) {
    let _ = tx.try_send(event);
}

pub(crate) struct RepositoryView {
    tx: Sender<UiEvent>,
    rx: Receiver<UiEvent>,
    supplied_credential: Arc<Mutex<Option<GitCredential>>>,
    credential_store: Arc<KeyringCredentialStore>,
    credential_records: Vec<CredentialRecord>,
    tabs: Vec<RepoTabState>,
    active_tab: Option<RepoTabId>,
    next_tab_id: u64,
    fallback_tab: RepoTabState,
    restoring_session: bool,
    pub(crate) sidebar_width: f32,
    pub(crate) changes_width: f32,
    pub(crate) history_top_height: f32,
    pub(crate) history_files_width: f32,
    resizing_sidebar_width: Option<ResizeState>,
    resizing_changes_width: Option<ResizeState>,
    resizing_history_files_width: Option<ResizeState>,
    resizing_history_top_height: Option<ResizeState>,
    pending_credential: Option<PendingCredential>,
    pending_credentials: VecDeque<PendingCredential>,
    pub(crate) active_dialog: Option<DialogState>,
    pub(crate) branch_context_menu: Option<BranchContextMenu>,
    change_context_menu: Option<ChangeContextMenu>,
    pub(crate) tag_context_menu: Option<TagContextMenu>,
    pub(crate) stash_context_menu: Option<StashContextMenu>,
    pub(crate) commit_context_menu: Option<CommitContextMenu>,
    save_credential: bool,
    credential_scope: CredentialScope,
    clone_url: TextFieldState,
    clone_path: TextFieldState,
    branch_name: TextFieldState,
    branch_rename: TextFieldState,
    commit_message: TextFieldState,
    credential_username: TextFieldState,
    credential_secret: TextFieldState,
    credential_key_path: TextFieldState,
    credential_passphrase: TextFieldState,
}

impl RepositoryView {
    fn new(cx: &mut Context<Self>) -> Self {
        let (tx, rx) = async_channel::unbounded();
        let supplied_credential = Arc::new(Mutex::new(None));
        let credential_store = Arc::new(KeyringCredentialStore::new());
        Self::spawn_event_pump(rx.clone(), cx);

        Self {
            tx,
            rx,
            supplied_credential,
            credential_store,
            credential_records: Vec::new(),
            tabs: Vec::new(),
            active_tab: None,
            next_tab_id: 1,
            fallback_tab: RepoTabState::new(RepoTabId(0), None),
            restoring_session: false,
            sidebar_width: DEFAULT_SIDEBAR_WIDTH,
            changes_width: DEFAULT_CHANGES_WIDTH,
            history_top_height: DEFAULT_HISTORY_TOP_HEIGHT,
            history_files_width: DEFAULT_HISTORY_FILES_WIDTH,
            resizing_sidebar_width: None,
            resizing_changes_width: None,
            resizing_history_files_width: None,
            resizing_history_top_height: None,
            pending_credential: None,
            pending_credentials: VecDeque::new(),
            active_dialog: None,
            branch_context_menu: None,
            change_context_menu: None,
            tag_context_menu: None,
            stash_context_menu: None,
            commit_context_menu: None,
            save_credential: false,
            credential_scope: CredentialScope::RemoteUrl,
            clone_url: TextFieldState::new(cx, "远程仓库 URL"),
            clone_path: TextFieldState::new(cx, "克隆目标文件夹"),
            branch_name: TextFieldState::new(cx, "新分支名称"),
            branch_rename: TextFieldState::new(cx, "重命名为"),
            commit_message: TextFieldState::new(cx, "提交信息"),
            credential_username: TextFieldState::new(cx, "用户名"),
            credential_secret: TextFieldState::new(cx, "密码或 PAT").secret(),
            credential_key_path: TextFieldState::new(cx, "SSH 私钥路径"),
            credential_passphrase: TextFieldState::new(cx, "SSH 密码短语").secret(),
        }
    }

    fn new_with_session(cx: &mut Context<Self>) -> Self {
        let mut view = Self::new(cx);
        view.restore_session();
        view
    }

    fn active_tab_id(&self) -> Option<RepoTabId> {
        self.active_tab
    }

    fn active_tab(&self) -> Option<&RepoTabState> {
        let id = self.active_tab?;
        self.tabs.iter().find(|tab| tab.id == id)
    }

    fn tab_mut(&mut self, tab_id: RepoTabId) -> Option<&mut RepoTabState> {
        self.tabs.iter_mut().find(|tab| tab.id == tab_id)
    }

    fn tab(&self, tab_id: RepoTabId) -> Option<&RepoTabState> {
        self.tabs.iter().find(|tab| tab.id == tab_id)
    }

    fn ensure_tab_for_path(&mut self, path: PathBuf) -> RepoTabId {
        let key = normalize_repo_path(&path);
        if let Some(tab) = self
            .tabs
            .iter()
            .find(|tab| tab.path_key().as_deref() == Some(key.as_str()))
        {
            self.active_tab = Some(tab.id);
            self.save_session();
            return tab.id;
        }

        let id = RepoTabId(self.next_tab_id);
        self.next_tab_id = self.next_tab_id.wrapping_add(1).max(1);
        self.tabs.push(RepoTabState::new(id, Some(path)));
        self.active_tab = Some(id);
        self.save_session();
        id
    }

    fn activate_tab(&mut self, tab_id: RepoTabId) {
        if self.active_tab == Some(tab_id) || self.tab(tab_id).is_none() {
            return;
        }
        self.close_popups();
        self.active_tab = Some(tab_id);
        self.ensure_history_loaded();
        self.save_session();
    }

    fn close_tab(&mut self, tab_id: RepoTabId) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == tab_id) else {
            return;
        };
        self.close_popups();
        self.tabs.remove(index);
        self.pending_credentials
            .retain(|pending| pending.tab_id != Some(tab_id));
        if self
            .pending_credential
            .as_ref()
            .and_then(|pending| pending.tab_id)
            == Some(tab_id)
        {
            self.show_next_credential_request();
        }
        if self.active_tab == Some(tab_id) {
            self.active_tab = self
                .tabs
                .get(index)
                .or_else(|| index.checked_sub(1).and_then(|prev| self.tabs.get(prev)))
                .map(|tab| tab.id);
        }
        self.save_session();
    }

    fn session_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "Khaslana").map(|dirs| dirs.config_dir().join("session.json"))
    }

    fn load_session_state() -> Option<SessionState> {
        let path = Self::session_path()?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_session(&self) {
        if self.restoring_session {
            return;
        }
        let Some(path) = Self::session_path() else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if let Err(err) = fs::create_dir_all(parent) {
            tracing::warn!("session directory create skipped: {err}");
            return;
        }

        let repo_paths = dedupe_repo_paths(
            self.tabs
                .iter()
                .filter_map(|tab| tab.repo_path.clone())
                .collect::<Vec<_>>(),
        );
        let active_repo_path = self
            .active_tab()
            .and_then(|tab| tab.repo_path.as_ref())
            .cloned();
        let state = SessionState {
            repo_paths,
            active_repo_path,
        };
        match serde_json::to_string_pretty(&state) {
            Ok(content) => {
                if let Err(err) = fs::write(path, content) {
                    tracing::warn!("session write skipped: {err}");
                }
            }
            Err(err) => tracing::warn!("session encode skipped: {err}"),
        }
    }

    fn restore_session(&mut self) {
        let Some(session) = Self::load_session_state() else {
            return;
        };
        self.restoring_session = true;
        let mut restored = Vec::new();
        let mut failed = 0usize;
        let mut seen = BTreeSet::new();

        for path in session.repo_paths {
            let key = normalize_repo_path(&path);
            if !seen.insert(key) {
                continue;
            }
            if !path.exists() || Repository::open(&path).is_err() {
                failed += 1;
                continue;
            }
            restored.push(path);
        }

        if restored.is_empty() {
            if failed > 0 {
                self.fallback_tab.last_error = Some(format!("{failed} 个上次打开的仓库无法恢复"));
                self.fallback_tab.status = "会话恢复失败".to_string();
            }
            self.restoring_session = false;
            self.save_session();
            return;
        }

        let active_key = session
            .active_repo_path
            .as_ref()
            .map(|path| normalize_repo_path(path));
        let mut active = None;
        for path in restored {
            let id = self.ensure_tab_for_path(path.clone());
            if active_key.as_deref() == Some(normalize_repo_path(&path).as_str()) {
                active = Some(id);
            }
        }
        if let Some(active) = active.or(self.active_tab) {
            self.active_tab = Some(active);
        }
        if failed > 0 {
            self.fallback_tab.last_error = Some(format!("{failed} 个上次打开的仓库无法恢复"));
        }

        let tabs = self.tabs.iter().map(|tab| tab.id).collect::<Vec<_>>();
        for tab_id in tabs {
            if let Some(path) = self.tab(tab_id).and_then(|tab| tab.repo_path.clone()) {
                self.load_repository_for_tab(tab_id, path, "正在恢复仓库", "仓库已恢复");
            }
        }
        self.restoring_session = false;
        self.save_session();
    }

    fn active_tab_state(&self) -> &RepoTabState {
        self.active_tab().unwrap_or_else(|| &self.fallback_tab)
    }

    fn active_tab_state_mut(&mut self) -> &mut RepoTabState {
        let id = self.active_tab;
        if let Some(id) = id
            && let Some(index) = self.tabs.iter().position(|tab| tab.id == id)
        {
            return &mut self.tabs[index];
        }
        &mut self.fallback_tab
    }

    fn service_for_tab(&self, tab_id: RepoTabId) -> GitService {
        GitService::new(
            Arc::new(TabCredentialProvider::new(
                self.credential_store.clone(),
                self.tx.clone(),
                self.supplied_credential.clone(),
                tab_id,
            )),
            Arc::new(TabProgress {
                tx: self.tx.clone(),
                tab_id,
            }),
        )
    }

    fn with_tab_context<R>(
        &mut self,
        tab_id: RepoTabId,
        f: impl FnOnce(&mut Self) -> R,
    ) -> Option<R> {
        self.tab(tab_id)?;
        let previous = self.active_tab;
        self.active_tab = Some(tab_id);
        let result = f(self);
        self.active_tab = previous
            .filter(|id| self.tab(*id).is_some())
            .or_else(|| self.tabs.first().map(|tab| tab.id));
        Some(result)
    }

    fn apply_status_event(&mut self, tab_id: Option<RepoTabId>, f: impl FnOnce(&mut Self)) {
        if let Some(tab_id) = tab_id {
            let _ = self.with_tab_context(tab_id, f);
        } else {
            f(self);
        }
    }

    fn enqueue_credential_request(&mut self, pending: PendingCredential) {
        if pending
            .tab_id
            .is_some_and(|tab_id| self.tab(tab_id).is_none())
        {
            return;
        }
        if self.pending_credential.is_none() {
            self.pending_credential = Some(pending);
        } else {
            self.pending_credentials.push_back(pending);
        }
    }

    fn show_next_credential_request(&mut self) {
        self.pending_credential = None;
        while let Some(pending) = self.pending_credentials.pop_front() {
            if pending
                .tab_id
                .is_none_or(|tab_id| self.tab(tab_id).is_some())
            {
                self.pending_credential = Some(pending);
                break;
            }
        }
    }

    fn spawn_event_pump(rx: Receiver<UiEvent>, cx: &mut Context<Self>) {
        cx.spawn(async move |weak: WeakEntity<RepositoryView>, cx| {
            while let Ok(event) = rx.recv().await {
                if weak
                    .update(cx, |this, cx| {
                        this.handle_ui_event(event, cx);
                        this.drain_pending_events(cx);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
                let _ = cx.refresh();
            }
        })
        .detach();
    }

    fn drain_pending_events(&mut self, cx: &mut Context<Self>) {
        while let Ok(event) = self.rx.try_recv() {
            self.handle_ui_event(event, cx);
        }
    }

    fn handle_ui_event(&mut self, event: UiEvent, cx: &mut Context<Self>) {
        match event {
            UiEvent::OperationStarted { tab_id, message } => {
                self.apply_status_event(tab_id, |this| {
                    this.busy = true;
                    this.status = message;
                    this.last_error = None;
                });
            }
            UiEvent::OperationProgress { tab_id, message } => {
                self.apply_status_event(tab_id, |this| {
                    this.status = message;
                });
            }
            UiEvent::RepositoryFastLoaded {
                tab_id,
                message,
                snapshot,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.busy = false;
                        this.loading = RepositoryLoading {
                            metadata: true,
                            status_fast: true,
                            status_full: true,
                        };
                        this.status = message;
                        this.last_error = None;
                        this.diff = None;
                        this.clear_history();
                        this.change_selection.clear();
                        this.repo_path = Some(snapshot.path.clone());
                        this.sync_selected_remote(&snapshot);
                        this.snapshot = Some(snapshot);
                        this.reload_history_if_active();
                    }
                });
            }
            UiEvent::RepositoryMetadataLoaded {
                tab_id,
                message,
                snapshot,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.busy = false;
                        this.loading.metadata = false;
                        this.status = message;
                        this.merge_metadata_snapshot(snapshot);
                    }
                });
            }
            UiEvent::RepositoryStatusFastLoaded {
                tab_id,
                message,
                changes,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.loading.status_fast = false;
                        this.status = message;
                        this.replace_changes(changes);
                    }
                });
            }
            UiEvent::RepositoryStatusFullLoaded {
                tab_id,
                message,
                changes,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.loading.status_full = false;
                        this.status = message;
                        this.replace_changes(changes);
                    }
                });
            }
            UiEvent::RepositoryLoadStageFailed {
                tab_id,
                error,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.loading = RepositoryLoading::default();
                        this.status = "仓库已打开，后台加载失败".to_string();
                        this.last_error = Some(error);
                    }
                });
            }
            UiEvent::OperationFinished {
                tab_id,
                message,
                snapshot,
                diff,
            } => {
                self.apply_status_event(tab_id, |this| {
                    this.busy = false;
                    this.loading = RepositoryLoading::default();
                    this.status = message;
                    if let Some(snapshot) = snapshot {
                        this.repo_path = (!snapshot.path.as_os_str().is_empty())
                            .then(|| snapshot.path.clone())
                            .or_else(|| this.repo_path.clone());
                        this.sync_selected_remote(&snapshot);
                        this.snapshot = Some(snapshot);
                        this.prune_change_selection();
                        this.clear_history();
                        this.reload_history_if_active();
                    }
                    if let Some(diff) = diff {
                        this.diff = Some(diff);
                        this.diff_headers_expanded = false;
                    }
                });
            }
            UiEvent::CredentialRecordsLoaded { records, message } => {
                self.busy = false;
                self.credential_records = records;
                self.status = message;
                self.last_error = None;
            }
            UiEvent::HistoryCommitsLoaded {
                tab_id,
                commits,
                append,
                has_more,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.history_loading.commits = false;
                        this.history_has_more = has_more;
                        if append {
                            this.history_commits.extend(commits);
                        } else {
                            this.history_commits = commits;
                            this.history_selected_commit = None;
                            this.history_files.clear();
                            this.history_selected_file = None;
                            this.history_diff = None;
                        }

                        if this.history_selected_commit.is_none() {
                            if let Some(commit) = this.history_commits.first() {
                                this.select_history_commit(commit.oid.clone());
                            } else {
                                this.status = "当前分支暂无提交记录".to_string();
                            }
                        } else {
                            this.status = "提交记录已加载".to_string();
                        }
                    }
                });
            }
            UiEvent::HistoryFilesLoaded {
                tab_id,
                commit_oid,
                files,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id
                        && this.history_selected_commit.as_deref() == Some(commit_oid.as_str())
                    {
                        this.history_loading.files = false;
                        this.history_files = files;
                        this.history_selected_file = None;
                        this.history_diff = None;
                        this.history_diff_headers_expanded = false;

                        if let Some(file) = this.history_files.first() {
                            this.select_history_file(file.path.clone());
                        } else {
                            this.status = "该提交没有文件变更".to_string();
                        }
                    }
                });
            }
            UiEvent::HistoryDiffLoaded {
                tab_id,
                commit_oid,
                path,
                diff,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id
                        && this.history_selected_commit.as_deref() == Some(commit_oid.as_str())
                        && this.history_selected_file.as_deref() == Some(path.as_str())
                    {
                        this.history_loading.diff = false;
                        this.history_diff = Some(diff);
                        this.history_diff_headers_expanded = false;
                        this.status = "提交差异已加载".to_string();
                    }
                });
            }
            UiEvent::HistoryLoadFailed {
                tab_id,
                error,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.history_loading = HistoryLoading::default();
                        this.status = "提交记录加载失败".to_string();
                        this.last_error = Some(error);
                    }
                });
            }
            UiEvent::OperationFailed { tab_id, error } => {
                self.apply_status_event(tab_id, |this| {
                    this.busy = false;
                    this.loading = RepositoryLoading::default();
                    this.status = "操作失败".to_string();
                    this.last_error = Some(error);
                });
            }
            UiEvent::CredentialRequested { tab_id, request } => {
                if tab_id.is_some_and(|tab_id| self.tab(tab_id).is_none()) {
                    return;
                }
                self.apply_status_event(tab_id, |this| {
                    this.busy = false;
                    this.status = "需要凭据".to_string();
                });
                self.save_credential = false;
                self.credential_scope = CredentialScope::RemoteUrl;
                self.credential_username.value = request
                    .username_from_url
                    .clone()
                    .unwrap_or_else(|| "git".to_string());
                self.credential_secret.value.clear();
                self.credential_key_path.value.clear();
                self.credential_passphrase.value.clear();
                self.enqueue_credential_request(PendingCredential { tab_id, request });
            }
        }
        cx.notify();
    }

    fn merge_metadata_snapshot(&mut self, snapshot: RepositorySnapshot) {
        let mut merged = self.snapshot.take().unwrap_or_default();
        merged.path = snapshot.path;
        merged.head = snapshot.head;
        merged.branches = snapshot.branches;
        merged.remotes = snapshot.remotes;
        merged.tags = snapshot.tags;
        merged.stashes = snapshot.stashes;
        merged.conflicts = snapshot.conflicts;
        self.repo_path = Some(merged.path.clone());
        self.sync_selected_remote(&merged);
        self.snapshot = Some(merged);
    }

    fn replace_changes(&mut self, changes: Vec<khaslana::WorktreeChange>) {
        if let Some(snapshot) = self.snapshot.as_mut() {
            snapshot.changes = changes;
        }
        self.prune_change_selection();
    }

    fn prune_change_selection(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            self.change_selection.clear();
            return;
        };
        let staged = snapshot
            .changes
            .iter()
            .filter(|change| change.staged.is_some())
            .map(|change| change.path.clone())
            .collect::<BTreeSet<_>>();
        let unstaged = snapshot
            .changes
            .iter()
            .filter(|change| change.unstaged.is_some())
            .map(|change| change.path.clone())
            .collect::<BTreeSet<_>>();
        self.change_selection
            .staged
            .retain(|path| staged.contains(path));
        self.change_selection
            .unstaged
            .retain(|path| unstaged.contains(path));
        if self
            .change_selection
            .staged_anchor
            .as_ref()
            .is_some_and(|path| !staged.contains(path))
        {
            self.change_selection.staged_anchor = None;
        }
        if self
            .change_selection
            .unstaged_anchor
            .as_ref()
            .is_some_and(|path| !unstaged.contains(path))
        {
            self.change_selection.unstaged_anchor = None;
        }
    }

    fn handle_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(field) = self.focused_field(window, cx) {
            self.handle_field_key(field, event, window, cx);
        }
    }

    fn handle_field_key(
        &mut self,
        field: FieldId,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        match key {
            "backspace" => {
                self.field_mut(field).value.pop();
                cx.notify();
            }
            "delete" => {
                self.field_mut(field).value.clear();
                cx.notify();
            }
            "enter" => {
                if matches!(field, FieldId::CommitMessage) {
                    self.commit();
                } else if matches!(field, FieldId::CloneUrl | FieldId::ClonePath) {
                    if self.active_dialog == Some(DialogState::CloneRepo) {
                        self.clone_repo();
                    }
                } else if matches!(field, FieldId::BranchName) {
                    if self.active_dialog == Some(DialogState::CreateBranch) {
                        self.create_branch();
                    }
                } else if matches!(field, FieldId::BranchRename) {
                    if let Some(DialogState::RenameBranch { branch }) = self.active_dialog.clone() {
                        self.rename_branch(branch);
                    }
                } else if matches!(
                    field,
                    FieldId::CredentialSecret | FieldId::CredentialPassphrase
                ) {
                    self.use_credentials();
                }
            }
            _ => {
                if (event.keystroke.modifiers.control || event.keystroke.modifiers.platform)
                    && key.eq_ignore_ascii_case("v")
                {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        self.push_field_text(field, &text);
                        cx.notify();
                    }
                } else if !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.platform
                    && let Some(text) = event.keystroke.key_char.as_ref()
                {
                    self.push_field_text(field, text);
                    cx.notify();
                }
            }
        }
    }

    fn push_field_text(&mut self, field: FieldId, text: &str) {
        let field_state = self.field_mut(field);
        if field != FieldId::CommitMessage {
            field_state.value.push_str(&text.replace(['\r', '\n'], ""));
        } else {
            field_state.value.push_str(text);
        }
    }

    fn focused_field(&self, window: &Window, _cx: &App) -> Option<FieldId> {
        [
            (FieldId::CloneUrl, &self.clone_url),
            (FieldId::ClonePath, &self.clone_path),
            (FieldId::BranchName, &self.branch_name),
            (FieldId::BranchRename, &self.branch_rename),
            (FieldId::CommitMessage, &self.commit_message),
            (FieldId::CredentialUsername, &self.credential_username),
            (FieldId::CredentialSecret, &self.credential_secret),
            (FieldId::CredentialKeyPath, &self.credential_key_path),
            (FieldId::CredentialPassphrase, &self.credential_passphrase),
        ]
        .into_iter()
        .find_map(|(id, field)| field.focus.is_focused(window).then_some(id))
    }

    fn field(&self, id: FieldId) -> &TextFieldState {
        match id {
            FieldId::CloneUrl => &self.clone_url,
            FieldId::ClonePath => &self.clone_path,
            FieldId::BranchName => &self.branch_name,
            FieldId::BranchRename => &self.branch_rename,
            FieldId::CommitMessage => &self.commit_message,
            FieldId::CredentialUsername => &self.credential_username,
            FieldId::CredentialSecret => &self.credential_secret,
            FieldId::CredentialKeyPath => &self.credential_key_path,
            FieldId::CredentialPassphrase => &self.credential_passphrase,
        }
    }

    fn field_mut(&mut self, id: FieldId) -> &mut TextFieldState {
        match id {
            FieldId::CloneUrl => &mut self.clone_url,
            FieldId::ClonePath => &mut self.clone_path,
            FieldId::BranchName => &mut self.branch_name,
            FieldId::BranchRename => &mut self.branch_rename,
            FieldId::CommitMessage => &mut self.commit_message,
            FieldId::CredentialUsername => &mut self.credential_username,
            FieldId::CredentialSecret => &mut self.credential_secret,
            FieldId::CredentialKeyPath => &mut self.credential_key_path,
            FieldId::CredentialPassphrase => &mut self.credential_passphrase,
        }
    }

    fn browse_open(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.open_repo(path);
        }
    }

    fn browse_clone_target(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.clone_path.value = path.display().to_string();
        }
    }

    fn open_clone_dialog(&mut self, window: &mut Window) {
        self.close_popups();
        self.active_dialog = Some(DialogState::CloneRepo);
        self.last_error = None;
        window.focus(&self.clone_url.focus);
    }

    pub(crate) fn open_create_branch_dialog(&mut self) {
        if self.repo_path.is_none() {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        }
        self.close_popups();
        self.branch_name.value.clear();
        self.active_dialog = Some(DialogState::CreateBranch);
        self.last_error = None;
    }

    pub(crate) fn open_rename_branch_dialog(&mut self, branch: String) {
        self.close_popups();
        self.branch_rename.value = branch.clone();
        self.active_dialog = Some(DialogState::RenameBranch { branch });
        self.last_error = None;
    }

    pub(crate) fn close_popups(&mut self) {
        self.active_dialog = None;
        self.branch_context_menu = None;
        self.change_context_menu = None;
        self.tag_context_menu = None;
        self.stash_context_menu = None;
        self.commit_context_menu = None;
    }

    fn close_dialog(&mut self) {
        self.active_dialog = None;
    }

    fn open_credential_manager(&mut self) {
        self.close_popups();
        self.active_dialog = Some(DialogState::CredentialManager);
        self.reload_credential_records("凭据列表已加载");
    }

    fn reload_credential_records(&mut self, message: &'static str) {
        match self.credential_store.list_records() {
            Ok(records) => {
                self.credential_records = records;
                self.status = message.to_string();
                self.last_error = None;
            }
            Err(err) => {
                self.last_error = Some(err.to_string());
            }
        }
    }

    fn open_delete_credential_confirm(&mut self, record_id: String, label: String) {
        self.active_dialog = Some(DialogState::ConfirmDeleteCredential { record_id, label });
        self.last_error = None;
    }

    fn delete_credential_record(&mut self, record_id: String) {
        match self.credential_store.delete_record(&record_id) {
            Ok(()) => {
                self.active_dialog = Some(DialogState::CredentialManager);
                self.reload_credential_records("凭据已删除");
            }
            Err(err) => {
                self.active_dialog = Some(DialogState::CredentialManager);
                self.last_error = Some(err.to_string());
            }
        }
    }

    fn test_credential_record(&mut self, record_id: String) {
        if self.busy {
            self.last_error = Some("已有操作正在运行".into());
            return;
        }
        let Some(record) = self
            .credential_records
            .iter()
            .find(|record| record.id == record_id)
            .cloned()
        else {
            self.last_error = Some("凭据记录不存在".into());
            return;
        };
        self.busy = true;
        self.status = "正在测试凭据连接".to_string();
        self.last_error = None;
        let store: Arc<dyn CredentialStore> = self.credential_store.clone();
        let tx = self.tx.clone();
        thread::spawn(
            move || match test_credential_connection(store.as_ref(), &record) {
                Ok(()) => {
                    let records = store.list_records().unwrap_or_default();
                    send_ui_event(
                        &tx,
                        UiEvent::CredentialRecordsLoaded {
                            records,
                            message: "凭据测试通过".to_string(),
                        },
                    );
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::OperationFailed {
                            tab_id: None,
                            error: err.to_string(),
                        },
                    );
                }
            },
        );
    }

    fn open_repo(&mut self, path: PathBuf) {
        let tab_id = self.ensure_tab_for_path(path.clone());
        self.load_repository_for_tab(tab_id, path, "正在打开仓库", "仓库已打开");
    }

    fn clone_repo(&mut self) {
        let url = self.clone_url.value.trim().to_string();
        let path_text = self.clone_path.value.trim().to_string();
        if url.is_empty() || path_text.is_empty() {
            self.last_error = Some("需要填写远程仓库 URL 和目标文件夹".into());
            return;
        }
        let path = PathBuf::from(path_text.clone());
        let key = normalize_repo_path(&path);
        if let Some(tab) = self
            .tabs
            .iter()
            .find(|tab| tab.path_key().as_deref() == Some(key.as_str()))
        {
            self.active_tab = Some(tab.id);
            self.last_error = Some("该仓库已经打开".into());
            self.save_session();
            return;
        }

        let tab_id = self.ensure_tab_for_path(path.clone());
        let service = self.service_for_tab(tab_id);
        self.spawn_operation_for_tab(Some(tab_id), "正在克隆仓库", move || {
            service
                .clone_repo(&url, &RepoPath::new(path_text))
                .map(|snapshot| UiEvent::OperationFinished {
                    tab_id: Some(tab_id),
                    message: "克隆完成".to_string(),
                    snapshot: Some(snapshot),
                    diff: None,
                })
        });
    }

    fn refresh(&mut self) {
        let Some(tab_id) = self.active_tab_id() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let Some(path) = self.repo_path.clone() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        self.load_repository_for_tab(tab_id, path, "正在刷新仓库", "已刷新");
    }

    fn load_repository_for_tab(
        &mut self,
        tab_id: RepoTabId,
        path: PathBuf,
        started: &'static str,
        finished: &'static str,
    ) {
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        let load_id = {
            let Some(tab) = self.tab_mut(tab_id) else {
                return;
            };
            let load_id = tab.repository_load_id.wrapping_add(1);
            tab.repository_load_id = load_id;
            tab.repo_path = Some(path.clone());
            tab.busy = true;
            tab.loading = RepositoryLoading::default();
            tab.status = started.to_string();
            tab.last_error = None;
            load_id
        };
        self.close_popups();
        self.save_session();
        send_ui_event(
            &tx,
            UiEvent::OperationStarted {
                tab_id: Some(tab_id),
                message: started.to_string(),
            },
        );
        thread::spawn(move || {
            let repo_path = RepoPath::new(path);
            let fast = match service.open_fast(&repo_path) {
                Ok(snapshot) => snapshot,
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::OperationFailed {
                            tab_id: Some(tab_id),
                            error: err.to_string(),
                        },
                    );
                    return;
                }
            };
            send_ui_event(
                &tx,
                UiEvent::RepositoryFastLoaded {
                    tab_id,
                    message: "本地分支已加载，正在加载仓库详情".to_string(),
                    snapshot: fast,
                    load_id,
                },
            );

            let mut repo = match Repository::open(&repo_path.0) {
                Ok(repo) => repo,
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryLoadStageFailed {
                            tab_id,
                            error: err.to_string(),
                            load_id,
                        },
                    );
                    return;
                }
            };

            match service.snapshot_metadata(&mut repo) {
                Ok(snapshot) => {
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryMetadataLoaded {
                            tab_id,
                            message: "仓库信息已加载".to_string(),
                            snapshot,
                            load_id,
                        },
                    );
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryLoadStageFailed {
                            tab_id,
                            error: err.to_string(),
                            load_id,
                        },
                    );
                    return;
                }
            }

            match service.status_fast(&repo) {
                Ok(changes) => {
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryStatusFastLoaded {
                            tab_id,
                            message: "快速变更已加载，正在补全未跟踪文件".to_string(),
                            changes,
                            load_id,
                        },
                    );
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryLoadStageFailed {
                            tab_id,
                            error: err.to_string(),
                            load_id,
                        },
                    );
                    return;
                }
            }

            match service.status_full(&repo) {
                Ok(changes) => {
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryStatusFullLoaded {
                            tab_id,
                            message: finished.to_string(),
                            changes,
                            load_id,
                        },
                    );
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryLoadStageFailed {
                            tab_id,
                            error: err.to_string(),
                            load_id,
                        },
                    );
                }
            }
        });
    }

    fn with_repo<F>(&mut self, label: &'static str, f: F)
    where
        F: FnOnce(GitService, &mut Repository) -> khaslana::Result<RepositorySnapshot>
            + Send
            + 'static,
    {
        let Some(tab_id) = self.active_tab_id() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let Some(path) = self.repo_path.clone() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let service = self.service_for_tab(tab_id);
        self.spawn_operation_for_tab(Some(tab_id), label, move || {
            let mut repo = Repository::open(path)?;
            f(service, &mut repo).map(|snapshot| UiEvent::OperationFinished {
                tab_id: Some(tab_id),
                message: label.to_string(),
                snapshot: Some(snapshot),
                diff: None,
            })
        });
    }

    pub(crate) fn current_remote(&self) -> Option<String> {
        let snapshot = self.snapshot.as_ref()?;
        self.selected_remote
            .as_ref()
            .filter(|remote| snapshot.remotes.contains(*remote))
            .cloned()
            .or_else(|| {
                snapshot
                    .remotes
                    .iter()
                    .find(|remote| remote.as_str() == "origin")
                    .cloned()
            })
            .or_else(|| snapshot.remotes.first().cloned())
    }

    fn sync_selected_remote(&mut self, snapshot: &RepositorySnapshot) {
        if snapshot.remotes.is_empty() {
            self.selected_remote = None;
            return;
        }

        if self
            .selected_remote
            .as_ref()
            .is_some_and(|remote| snapshot.remotes.contains(remote))
        {
            return;
        }

        self.selected_remote = snapshot
            .remotes
            .iter()
            .find(|remote| remote.as_str() == "origin")
            .cloned()
            .or_else(|| snapshot.remotes.first().cloned());
    }

    fn fetch(&mut self) {
        let Some(remote) = self.current_remote() else {
            self.last_error = Some("当前仓库没有远端".into());
            return;
        };
        self.with_repo("拉取远程引用完成", move |service, repo| {
            service.fetch(repo, &RemoteName::new(remote))
        });
    }

    fn pull(&mut self) {
        let Some(remote) = self.current_remote() else {
            self.last_error = Some("当前仓库没有远端".into());
            return;
        };
        self.with_repo("拉取完成", move |service, repo| {
            service.pull(repo, &RemoteName::new(remote))
        });
    }

    fn push(&mut self) {
        let Some(remote) = self.current_remote() else {
            self.last_error = Some("当前仓库没有远端".into());
            return;
        };
        self.with_repo("推送完成", move |service, repo| {
            service.push(repo, &RemoteName::new(remote))
        });
    }

    pub(crate) fn checkout(&mut self, name: String) {
        self.with_repo("切换分支完成", move |service, repo| {
            service.checkout_branch(repo, &BranchName::new(name))
        });
    }

    fn create_branch(&mut self) {
        let name = self.branch_name.value.trim().to_string();
        if name.is_empty() {
            self.last_error = Some("需要填写分支名称".into());
            return;
        }
        self.with_repo("分支已创建", move |service, repo| {
            service.create_branch(repo, &BranchName::new(name))
        });
    }

    fn rename_branch(&mut self, old: String) {
        let new = self.branch_rename.value.trim().to_string();
        if new.is_empty() {
            self.last_error = Some("需要填写新的分支名称".into());
            return;
        }
        self.with_repo("分支已重命名", move |service, repo| {
            service.rename_branch(repo, &BranchName::new(old), &BranchName::new(new))
        });
    }

    pub(crate) fn delete_branch(&mut self, name: String) {
        self.with_repo("分支已删除", move |service, repo| {
            service.delete_branch(repo, &BranchName::new(name))
        });
    }

    pub(crate) fn merge_branch(&mut self, name: String) {
        self.with_repo("合并完成", move |service, repo| {
            service.merge_branch(repo, &BranchName::new(name))
        });
    }

    pub(crate) fn checkout_remote_branch(&mut self, name: String) {
        self.with_repo("远端分支已拉取到本地", move |service, repo| {
            service.checkout_remote_branch(repo, &BranchName::new(name))
        });
    }

    pub(crate) fn checkout_tag(&mut self, name: String) {
        self.with_repo("检出标签完成", move |service, repo| {
            service.checkout_tag(repo, &TagName::new(name))
        });
    }

    pub(crate) fn apply_stash(&mut self, index: usize) {
        self.with_repo("应用贮藏完成", move |service, repo| {
            service.apply_stash(repo, index)
        });
    }

    pub(crate) fn pop_stash(&mut self, index: usize) {
        self.with_repo("弹出贮藏完成", move |service, repo| {
            service.pop_stash(repo, index)
        });
    }

    pub(crate) fn open_reset_confirm_dialog(
        &mut self,
        oid: String,
        summary: String,
        mode: ResetMode,
    ) {
        self.close_popups();
        self.active_dialog = Some(DialogState::ConfirmReset { oid, summary, mode });
        self.last_error = None;
    }

    pub(crate) fn open_revert_confirm_dialog(&mut self, oid: String, summary: String) {
        self.close_popups();
        self.active_dialog = Some(DialogState::ConfirmRevert { oid, summary });
        self.last_error = None;
    }

    fn reset_to_commit(&mut self, oid: String, mode: ResetMode) {
        self.with_repo("分支已重置", move |service, repo| {
            service.reset_to_commit(repo, &oid, mode)
        });
    }

    fn revert_commit(&mut self, oid: String) {
        self.with_repo("回滚提交完成", move |service, repo| {
            service.revert_commit(repo, &oid)
        });
    }

    pub(crate) fn copy_commit_sha(&mut self, oid: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(oid));
        self.commit_context_menu = None;
        self.status = "已复制提交 SHA".into();
        self.last_error = None;
    }

    fn change_paths(&self, scope: DiffScope) -> Vec<String> {
        self.snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .changes
                    .iter()
                    .filter(|change| match scope {
                        DiffScope::Staged => change.staged.is_some(),
                        DiffScope::Unstaged => change.unstaged.is_some(),
                    })
                    .map(|change| change.path.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn selected_change_paths(&self, scope: DiffScope) -> Vec<String> {
        self.change_selection
            .selected(&scope)
            .iter()
            .cloned()
            .collect()
    }

    fn is_change_selected(&self, scope: &DiffScope, path: &str) -> bool {
        self.change_selection.selected(scope).contains(path)
    }

    pub(crate) fn has_local_branch_for_remote(&self, remote_branch: &str) -> bool {
        let Some((_, local_name)) = remote_branch.split_once('/') else {
            return false;
        };
        self.snapshot.as_ref().is_some_and(|snapshot| {
            snapshot.branches.iter().any(|branch| {
                branch.kind == BranchKind::Local && branch.name.as_str() == local_name
            })
        })
    }

    fn clear_opposite_change_selection(&mut self, scope: &DiffScope) {
        match scope {
            DiffScope::Staged => {
                self.change_selection.unstaged.clear();
                self.change_selection.unstaged_anchor = None;
            }
            DiffScope::Unstaged => {
                self.change_selection.staged.clear();
                self.change_selection.staged_anchor = None;
            }
        }
    }

    fn clear_change_anchor_if_empty(&mut self, scope: &DiffScope) {
        if !self.change_selection.selected(scope).is_empty() {
            return;
        }
        match scope {
            DiffScope::Staged => self.change_selection.staged_anchor = None,
            DiffScope::Unstaged => self.change_selection.unstaged_anchor = None,
        }
    }

    fn select_change_from_mouse(&mut self, path: String, scope: DiffScope, event: &MouseDownEvent) {
        self.clear_opposite_change_selection(&scope);
        let multi = event.modifiers.control || event.modifiers.platform;
        if event.modifiers.shift {
            self.select_change_range(path.clone(), scope.clone());
        } else if multi {
            let selected = self.change_selection.selected_mut(&scope);
            if selected.contains(&path) {
                selected.remove(&path);
                self.clear_change_anchor_if_empty(&scope);
            } else {
                selected.insert(path.clone());
                self.change_selection.set_anchor(&scope, path.clone());
                self.load_diff(path.clone(), scope.clone());
            }
        } else if self.is_change_selected(&scope, &path) {
            self.change_selection.selected_mut(&scope).remove(&path);
            self.clear_change_anchor_if_empty(&scope);
        } else {
            self.change_selection.selected_mut(&scope).clear();
            self.change_selection
                .selected_mut(&scope)
                .insert(path.clone());
            self.change_selection.set_anchor(&scope, path.clone());
            self.load_diff(path, scope);
        }
    }

    fn select_change_range(&mut self, path: String, scope: DiffScope) {
        self.clear_opposite_change_selection(&scope);
        let paths = self.change_paths(scope.clone());
        let Some(current_index) = paths.iter().position(|candidate| candidate == &path) else {
            return;
        };
        let Some(anchor) = self.change_selection.anchor(&scope).cloned() else {
            self.change_selection.selected_mut(&scope).clear();
            self.change_selection
                .selected_mut(&scope)
                .insert(path.clone());
            self.change_selection.set_anchor(&scope, path.clone());
            self.load_diff(path, scope);
            return;
        };
        let Some(anchor_index) = paths.iter().position(|candidate| candidate == &anchor) else {
            self.change_selection.selected_mut(&scope).clear();
            self.change_selection
                .selected_mut(&scope)
                .insert(path.clone());
            self.change_selection.set_anchor(&scope, path.clone());
            self.load_diff(path, scope);
            return;
        };
        let (start, end) = if anchor_index <= current_index {
            (anchor_index, current_index)
        } else {
            (current_index, anchor_index)
        };
        let selected = self.change_selection.selected_mut(&scope);
        selected.clear();
        selected.extend(paths[start..=end].iter().cloned());
        self.load_diff(path, scope);
    }

    fn ensure_change_context_selection(&mut self, path: String, scope: DiffScope) {
        self.clear_opposite_change_selection(&scope);
        if !self.is_change_selected(&scope, &path) {
            self.change_selection.selected_mut(&scope).clear();
            self.change_selection
                .selected_mut(&scope)
                .insert(path.clone());
            self.change_selection.set_anchor(&scope, path.clone());
            self.load_diff(path, scope);
        }
    }

    fn open_change_context_menu(&mut self, path: String, scope: DiffScope, event: &MouseDownEvent) {
        self.ensure_change_context_selection(path, scope.clone());
        self.branch_context_menu = None;
        self.tag_context_menu = None;
        self.stash_context_menu = None;
        self.commit_context_menu = None;
        self.active_dialog = None;
        self.change_context_menu = Some(ChangeContextMenu {
            scope,
            x: event.position.x.into(),
            y: event.position.y.into(),
        });
    }

    fn mouse_down_inside_context_menu(&self, event: &MouseDownEvent) -> bool {
        let x: f32 = event.position.x.into();
        let y: f32 = event.position.y.into();
        self.branch_context_menu
            .as_ref()
            .is_some_and(|menu| point_in_menu(x, y, menu.x, menu.y, 190.0, 230.0))
            || self
                .change_context_menu
                .as_ref()
                .is_some_and(|menu| point_in_menu(x, y, menu.x, menu.y, 210.0, 170.0))
            || self
                .tag_context_menu
                .as_ref()
                .is_some_and(|menu| point_in_menu(x, y, menu.x, menu.y, 170.0, 80.0))
            || self
                .stash_context_menu
                .as_ref()
                .is_some_and(|menu| point_in_menu(x, y, menu.x, menu.y, 170.0, 110.0))
            || self
                .commit_context_menu
                .as_ref()
                .is_some_and(|menu| point_in_menu(x, y, menu.x, menu.y, 230.0, 230.0))
    }

    pub(crate) fn open_commit_context_menu(
        &mut self,
        oid: String,
        short_oid: String,
        summary: String,
        parent_count: usize,
        event: &MouseDownEvent,
    ) {
        self.select_history_commit(oid.clone());
        self.branch_context_menu = None;
        self.change_context_menu = None;
        self.tag_context_menu = None;
        self.stash_context_menu = None;
        self.active_dialog = None;
        self.commit_context_menu = Some(CommitContextMenu {
            oid,
            short_oid,
            summary,
            parent_count,
            x: event.position.x.into(),
            y: event.position.y.into(),
        });
    }

    fn start_resize_column(&mut self, target: ResizeTarget, event: &MouseDownEvent) {
        self.close_popups();
        let state = ResizeState {
            start_x: event.position.x.into(),
            start_y: event.position.y.into(),
            start_width: self.column_width(target),
            start_height: self.row_height(target),
        };
        match target {
            ResizeTarget::Sidebar => self.resizing_sidebar_width = Some(state),
            ResizeTarget::Changes => self.resizing_changes_width = Some(state),
            ResizeTarget::HistoryFiles => self.resizing_history_files_width = Some(state),
            ResizeTarget::HistoryTop => self.resizing_history_top_height = Some(state),
        }
    }

    fn update_resize_column(&mut self, target: ResizeTarget, event: &MouseMoveEvent) {
        let Some(resize) = self.resize_state(target) else {
            return;
        };
        let current_x: f32 = event.position.x.into();
        let delta = current_x - resize.start_x;
        match target {
            ResizeTarget::HistoryTop => {
                let current_y: f32 = event.position.y.into();
                let delta = current_y - resize.start_y;
                let height = (resize.start_height + delta)
                    .clamp(MIN_HISTORY_TOP_HEIGHT, MAX_HISTORY_TOP_HEIGHT);
                self.set_row_height(target, height);
            }
            ResizeTarget::HistoryFiles => {
                let width = (resize.start_width + delta)
                    .clamp(MIN_HISTORY_FILES_WIDTH, MAX_HISTORY_FILES_WIDTH);
                self.set_column_width(target, width);
            }
            ResizeTarget::Sidebar | ResizeTarget::Changes => {
                let width = (resize.start_width + delta).clamp(MIN_COLUMN_WIDTH, MAX_COLUMN_WIDTH);
                self.set_column_width(target, width);
            }
        }
    }

    fn finish_resize_column(&mut self, target: ResizeTarget) {
        match target {
            ResizeTarget::Sidebar => self.resizing_sidebar_width = None,
            ResizeTarget::Changes => self.resizing_changes_width = None,
            ResizeTarget::HistoryFiles => self.resizing_history_files_width = None,
            ResizeTarget::HistoryTop => self.resizing_history_top_height = None,
        }
    }

    fn reset_resize_target(&mut self, target: ResizeTarget) {
        self.finish_resize_column(target);
        match target {
            ResizeTarget::Sidebar => self.sidebar_width = DEFAULT_SIDEBAR_WIDTH,
            ResizeTarget::Changes => self.changes_width = DEFAULT_CHANGES_WIDTH,
            ResizeTarget::HistoryFiles => self.history_files_width = DEFAULT_HISTORY_FILES_WIDTH,
            ResizeTarget::HistoryTop => self.history_top_height = DEFAULT_HISTORY_TOP_HEIGHT,
        }
    }

    fn column_width(&self, target: ResizeTarget) -> f32 {
        match target {
            ResizeTarget::Sidebar => self.sidebar_width,
            ResizeTarget::Changes => self.changes_width,
            ResizeTarget::HistoryFiles => self.history_files_width,
            ResizeTarget::HistoryTop => 0.0,
        }
    }

    fn set_column_width(&mut self, target: ResizeTarget, width: f32) {
        match target {
            ResizeTarget::Sidebar => self.sidebar_width = width,
            ResizeTarget::Changes => self.changes_width = width,
            ResizeTarget::HistoryFiles => self.history_files_width = width,
            ResizeTarget::HistoryTop => {}
        }
    }

    fn row_height(&self, target: ResizeTarget) -> f32 {
        match target {
            ResizeTarget::HistoryTop => self.history_top_height,
            ResizeTarget::Sidebar | ResizeTarget::Changes | ResizeTarget::HistoryFiles => 0.0,
        }
    }

    fn set_row_height(&mut self, target: ResizeTarget, height: f32) {
        match target {
            ResizeTarget::HistoryTop => self.history_top_height = height,
            ResizeTarget::Sidebar | ResizeTarget::Changes | ResizeTarget::HistoryFiles => {}
        }
    }

    fn resize_state(&self, target: ResizeTarget) -> Option<ResizeState> {
        match target {
            ResizeTarget::Sidebar => self.resizing_sidebar_width,
            ResizeTarget::Changes => self.resizing_changes_width,
            ResizeTarget::HistoryFiles => self.resizing_history_files_width,
            ResizeTarget::HistoryTop => self.resizing_history_top_height,
        }
    }

    fn toggle_diff_headers(&mut self) {
        self.diff_headers_expanded = !self.diff_headers_expanded;
    }

    fn toggle_history_diff_headers(&mut self) {
        self.history_diff_headers_expanded = !self.history_diff_headers_expanded;
    }

    fn set_main_mode(&mut self, mode: MainMode) {
        self.main_mode = mode;
        self.close_popups();
        self.ensure_history_loaded();
    }

    fn clear_history(&mut self) {
        self.history_commits.clear();
        self.history_has_more = false;
        self.history_selected_commit = None;
        self.history_files.clear();
        self.history_selected_file = None;
        self.history_diff = None;
        self.history_diff_headers_expanded = false;
        self.history_loading = HistoryLoading::default();
    }

    fn reload_history_if_active(&mut self) {
        if self.main_mode == MainMode::History {
            self.load_history_page(false);
        }
    }

    fn ensure_history_loaded(&mut self) {
        if self.main_mode == MainMode::History
            && self.repo_path.is_some()
            && self.history_commits.is_empty()
            && !self.history_loading.commits
        {
            self.load_history_page(false);
        }
    }

    fn load_history_page(&mut self, append: bool) {
        let Some(tab_id) = self.active_tab_id() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        if self.history_loading.commits {
            return;
        }

        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        let offset = if append {
            self.history_commits.len()
        } else {
            0
        };
        let load_id = self.repository_load_id;
        self.history_loading.commits = true;
        self.status = if append {
            "正在加载更多提交记录".to_string()
        } else {
            "正在加载提交记录".to_string()
        };
        self.last_error = None;

        thread::spawn(move || {
            let result = (|| -> khaslana::Result<UiEvent> {
                let repo = Repository::open(repo_path)?;
                let mut commits = service.commit_history(&repo, offset, HISTORY_PAGE_SIZE + 1)?;
                let has_more = commits.len() > HISTORY_PAGE_SIZE;
                commits.truncate(HISTORY_PAGE_SIZE);
                Ok(UiEvent::HistoryCommitsLoaded {
                    tab_id,
                    commits,
                    append,
                    has_more,
                    load_id,
                })
            })();

            match result {
                Ok(event) => {
                    send_ui_event(&tx, event);
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::HistoryLoadFailed {
                            tab_id,
                            error: err.to_string(),
                            load_id,
                        },
                    );
                }
            }
        });
    }

    pub(crate) fn load_more_history(&mut self) {
        if !self.history_has_more {
            return;
        }
        self.load_history_page(true);
    }

    pub(crate) fn select_history_commit(&mut self, oid: String) {
        if self.history_selected_commit.as_deref() == Some(oid.as_str())
            && !self.history_files.is_empty()
        {
            return;
        }

        self.history_selected_commit = Some(oid.clone());
        self.history_files.clear();
        self.history_selected_file = None;
        self.history_diff = None;
        self.history_diff_headers_expanded = false;
        self.history_loading.files = true;
        self.history_loading.diff = false;
        self.status = "正在加载提交文件".to_string();

        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        let load_id = self.repository_load_id;

        thread::spawn(move || {
            let result = (|| -> khaslana::Result<UiEvent> {
                let repo = Repository::open(repo_path)?;
                let files = service.commit_files(&repo, &oid)?;
                Ok(UiEvent::HistoryFilesLoaded {
                    tab_id,
                    commit_oid: oid,
                    files,
                    load_id,
                })
            })();

            match result {
                Ok(event) => {
                    send_ui_event(&tx, event);
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::HistoryLoadFailed {
                            tab_id,
                            error: err.to_string(),
                            load_id,
                        },
                    );
                }
            }
        });
    }

    pub(crate) fn select_history_file(&mut self, path: String) {
        let Some(commit_oid) = self.history_selected_commit.clone() else {
            return;
        };
        if self.history_selected_file.as_deref() == Some(path.as_str())
            && self.history_diff.is_some()
        {
            return;
        }

        self.history_selected_file = Some(path.clone());
        self.history_diff = None;
        self.history_diff_headers_expanded = false;
        self.history_loading.diff = true;
        self.status = "正在加载提交差异".to_string();

        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        let load_id = self.repository_load_id;

        thread::spawn(move || {
            let result = (|| -> khaslana::Result<UiEvent> {
                let repo = Repository::open(repo_path)?;
                let diff = service.commit_file_diff(&repo, &commit_oid, Path::new(&path))?;
                Ok(UiEvent::HistoryDiffLoaded {
                    tab_id,
                    commit_oid,
                    path,
                    diff,
                    load_id,
                })
            })();

            match result {
                Ok(event) => {
                    send_ui_event(&tx, event);
                }
                Err(err) => {
                    send_ui_event(
                        &tx,
                        UiEvent::HistoryLoadFailed {
                            tab_id,
                            error: err.to_string(),
                            load_id,
                        },
                    );
                }
            }
        });
    }

    fn stage_selected(&mut self) {
        let paths = self.selected_change_paths(DiffScope::Unstaged);
        if paths.is_empty() {
            self.last_error = Some("请先在修改区选择文件".into());
            return;
        }
        self.stage_paths(paths, "已暂存选定文件");
    }

    fn stage_all(&mut self) {
        let paths = self.change_paths(DiffScope::Unstaged);
        if paths.is_empty() {
            self.last_error = Some("修改区没有可暂存文件".into());
            return;
        }
        self.stage_paths(paths, "已暂存所有文件");
    }

    fn stage_paths(&mut self, paths: Vec<String>, label: &'static str) {
        self.with_repo(label, move |service, repo| {
            let path_bufs = paths.into_iter().map(PathBuf::from).collect::<Vec<_>>();
            service.stage_paths(repo, path_bufs.iter().map(|path| path.as_path()))
        });
    }

    fn unstage_selected(&mut self) {
        let paths = self.selected_change_paths(DiffScope::Staged);
        if paths.is_empty() {
            self.last_error = Some("请先在暂存区选择文件".into());
            return;
        }
        self.unstage_paths(paths, "已取消暂存选定文件");
    }

    fn unstage_all(&mut self) {
        let paths = self.change_paths(DiffScope::Staged);
        if paths.is_empty() {
            self.last_error = Some("暂存区没有可取消暂存文件".into());
            return;
        }
        self.unstage_paths(paths, "已取消暂存所有文件");
    }

    fn unstage_paths(&mut self, paths: Vec<String>, label: &'static str) {
        self.with_repo(label, move |service, repo| {
            let path_bufs = paths.into_iter().map(PathBuf::from).collect::<Vec<_>>();
            service.unstage_paths(repo, path_bufs.iter().map(|path| path.as_path()))
        });
    }

    fn commit(&mut self) {
        let message = self.commit_message.value.trim().to_string();
        if message.is_empty() {
            self.last_error = Some("需要填写提交信息".into());
            return;
        }
        self.commit_message.value.clear();
        self.with_repo("提交完成", move |service, repo| {
            service.commit(repo, &CommitMessage::new(message))
        });
    }

    fn load_diff(&mut self, path: String, scope: DiffScope) {
        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        let service = self.service_for_tab(tab_id);
        self.spawn_operation_for_tab(Some(tab_id), "正在加载差异", move || {
            let repo = Repository::open(repo_path)?;
            service
                .diff_for_path(&repo, Path::new(&path), scope)
                .map(|diff| UiEvent::OperationFinished {
                    tab_id: Some(tab_id),
                    message: "差异已加载".to_string(),
                    snapshot: None,
                    diff: Some(diff),
                })
        });
    }

    fn use_credentials(&mut self) {
        let Some(pending) = self.pending_credential.clone() else {
            return;
        };

        let username = self
            .credential_username
            .value
            .trim()
            .to_string()
            .if_empty_then(|| {
                pending
                    .request
                    .username_from_url
                    .clone()
                    .unwrap_or_else(|| "git".into())
            });
        let secret = self.credential_secret.value.clone();
        let key_path = self.credential_key_path.value.trim().to_string();
        let passphrase = self.credential_passphrase.value.clone();

        let credential = if !key_path.is_empty() || pending.request.url.starts_with("ssh") {
            GitCredential::SshPassphrase {
                username,
                private_key_path: (!key_path.is_empty()).then_some(key_path),
                passphrase: (!passphrase.is_empty()).then_some(passphrase),
                save_to_keyring: self.save_credential,
                scope: self.credential_scope,
            }
        } else {
            GitCredential::UserPass {
                username,
                secret,
                save_to_keyring: self.save_credential,
                scope: self.credential_scope,
            }
        };

        let supplied_ok = {
            match self.supplied_credential.lock() {
                Ok(mut supplied) => {
                    *supplied = Some(credential);
                    true
                }
                Err(_) => false,
            }
        };
        if !supplied_ok {
            self.last_error = Some("凭据输入状态异常".into());
            return;
        }
        self.show_next_credential_request();
        self.apply_status_event(pending.tab_id, |this| {
            this.status = "凭据已准备好，请重试远程操作".into();
            this.last_error = None;
        });
    }

    fn spawn_operation_for_tab<F>(&mut self, tab_id: Option<RepoTabId>, started: &'static str, f: F)
    where
        F: FnOnce() -> khaslana::Result<UiEvent> + Send + 'static,
    {
        if let Some(tab_id) = tab_id
            && self.tab(tab_id).is_none()
        {
            return;
        }
        let busy = tab_id
            .and_then(|id| self.tab(id).map(|tab| tab.busy))
            .unwrap_or(self.busy);
        if busy {
            self.apply_status_event(tab_id, |this| {
                this.last_error = Some("已有操作正在运行".into());
            });
            return;
        }
        self.close_popups();
        self.apply_status_event(tab_id, |this| {
            this.repository_load_id = this.repository_load_id.wrapping_add(1);
            this.loading = RepositoryLoading::default();
            this.busy = true;
            this.status = started.to_string();
            this.last_error = None;
        });
        let tx = self.tx.clone();
        send_ui_event(
            &tx,
            UiEvent::OperationStarted {
                tab_id,
                message: started.to_string(),
            },
        );
        thread::spawn(move || match f() {
            Ok(event) => {
                send_ui_event(&tx, event);
            }
            Err(err) => {
                send_ui_event(
                    &tx,
                    UiEvent::OperationFailed {
                        tab_id,
                        error: err.to_string(),
                    },
                );
            }
        });
    }

    fn button(
        &self,
        label: &'static str,
        enabled: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let enabled_color = if enabled {
            COLOR_SURFACE
        } else {
            COLOR_SURFACE_SOFT
        };
        let text_color = if enabled {
            COLOR_TEXT
        } else {
            COLOR_TEXT_FAINT
        };
        div()
            .id(label)
            .flex_none()
            .px_2()
            .py_1()
            .border_1()
            .border_color(rgb(COLOR_BORDER))
            .rounded_sm()
            .bg(rgb(enabled_color))
            .text_color(rgb(text_color))
            .text_size(px(12.0))
            .cursor_pointer()
            .when(enabled, |this| {
                this.hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
                    .active(|this| this.opacity(0.82))
            })
            .on_click(cx.listener(move |this, _event, window, cx| {
                if enabled {
                    on_click(this, window, cx);
                    cx.notify();
                }
            }))
            .child(label)
    }

    fn mode_button(
        &self,
        label: &'static str,
        mode: MainMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.main_mode == mode;
        div()
            .id(format!("mode-{label}"))
            .flex_none()
            .px_2()
            .py_1()
            .border_1()
            .border_color(if selected {
                rgb(COLOR_BORDER_STRONG)
            } else {
                rgb(COLOR_BORDER)
            })
            .rounded_sm()
            .bg(if selected {
                rgb(COLOR_BLUE)
            } else {
                rgb(COLOR_SURFACE)
            })
            .text_color(if selected {
                rgb(COLOR_SURFACE)
            } else {
                rgb(COLOR_TEXT_MUTED)
            })
            .text_size(px(12.0))
            .cursor_pointer()
            .hover(|this| {
                this.bg(rgb(COLOR_BLUE_SOFT))
                    .text_color(rgb(COLOR_BLUE_DARK))
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.set_main_mode(mode);
                cx.notify();
            }))
            .child(label)
    }

    fn credential_scope_button(
        &self,
        label: &'static str,
        scope: CredentialScope,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.credential_scope == scope;
        let enabled = self.save_credential;
        div()
            .id(format!("credential-scope-{label}"))
            .flex_none()
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(if selected {
                rgb(COLOR_BORDER_STRONG)
            } else {
                rgb(COLOR_BORDER)
            })
            .bg(if selected {
                rgb(COLOR_BLUE_SOFT)
            } else {
                rgb(COLOR_SURFACE)
            })
            .text_size(px(12.0))
            .text_color(if selected {
                rgb(COLOR_BLUE_DARK)
            } else {
                rgb(COLOR_TEXT_MUTED)
            })
            .cursor_pointer()
            .when(enabled, |this| {
                this.hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                if enabled {
                    this.credential_scope = scope;
                    cx.notify();
                }
            }))
            .child(label)
    }

    fn input(
        &self,
        id: FieldId,
        compact: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let field = self.field(id);
        let focused = field.focus.is_focused(window);
        let empty = field.value.is_empty();
        div()
            .id(format!("field-{id:?}"))
            .track_focus(&field.focus)
            .on_mouse_down(MouseButton::Left, {
                let focus = field.focus.clone();
                move |_event, window, cx| {
                    window.focus(&focus);
                    cx.stop_propagation();
                }
            })
            .on_key_down(cx.listener(move |this, event, window, cx| {
                this.handle_field_key(id, event, window, cx);
                cx.stop_propagation();
            }))
            .px_2()
            .py_1()
            .min_h(if compact { px(26.0) } else { px(32.0) })
            .w_full()
            .rounded_sm()
            .border_1()
            .border_color(if focused {
                rgb(COLOR_BORDER_STRONG)
            } else {
                rgb(COLOR_BORDER)
            })
            .bg(if focused {
                rgb(COLOR_BLUE_SOFT)
            } else {
                rgb(COLOR_SURFACE)
            })
            .text_size(px(12.0))
            .text_color(if empty {
                rgb(COLOR_TEXT_FAINT)
            } else {
                rgb(COLOR_TEXT)
            })
            .child(field.display())
    }

    fn render_toolbar(&self, _window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let repo_open = self.repo_path.is_some();
        let remote_open = !self.loading.remote() && self.current_remote().is_some();
        div()
            .id("repo-tab-bar")
            .flex()
            .items_center()
            .justify_between()
            .gap_3()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(COLOR_PANEL_BG))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.button("打开仓库", !self.busy, |this, _, _| this.browse_open(), cx))
                    .child(self.button(
                        "克隆仓库",
                        !self.busy,
                        |this, window, _| this.open_clone_dialog(window),
                        cx,
                    ))
                    .child(self.button(
                        "刷新",
                        repo_open && !self.busy,
                        |this, _, _| this.refresh(),
                        cx,
                    ))
                    .child(self.button(
                        "获取",
                        repo_open && remote_open && !self.busy,
                        |this, _, _| this.fetch(),
                        cx,
                    ))
                    .child(self.button(
                        "拉取",
                        repo_open && remote_open && !self.busy,
                        |this, _, _| this.pull(),
                        cx,
                    ))
                    .child(self.button(
                        "推送",
                        repo_open && remote_open && !self.busy,
                        |this, _, _| this.push(),
                        cx,
                    ))
                    .child(self.button(
                        "凭据管理",
                        !self.busy,
                        |this, _, _| this.open_credential_manager(),
                        cx,
                    ))
                    .child(
                        div()
                            .ml_2()
                            .text_size(px(12.0))
                            .text_color(rgb(COLOR_BLUE_DARK))
                            .child(
                                self.repo_path
                                    .as_ref()
                                    .map(|path| path.display().to_string())
                                    .unwrap_or_else(|| "未打开仓库".into()),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_1()
                    .child(self.mode_button("工作区", MainMode::Worktree, cx))
                    .child(self.mode_button("提交记录", MainMode::History, cx)),
            )
    }

    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.tabs.is_empty() {
            return div().into_any_element();
        }

        div()
            .id("repo-tab-bar-scroll")
            .flex()
            .items_center()
            .gap_1()
            .min_w(px(0.0))
            .w_full()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(COLOR_HEADER_BG))
            .overflow_x_scroll()
            .children(
                self.tabs
                    .iter()
                    .map(|tab| self.render_repo_tab(tab, cx).into_any_element())
                    .collect::<Vec<_>>(),
            )
            .into_any_element()
    }

    fn render_repo_tab(&self, tab: &RepoTabState, cx: &mut Context<Self>) -> impl IntoElement {
        let id = tab.id;
        let selected = self.active_tab == Some(id);
        let title = tab.display_name();
        let status_dot = if tab.busy || tab.loading != RepositoryLoading::default() {
            "..."
        } else {
            ""
        };

        div()
            .id(format!("repo-tab-{}", id.0))
            .flex()
            .flex_none()
            .items_center()
            .gap_2()
            .w(px(214.0))
            .min_w(px(120.0))
            .max_w(px(280.0))
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(if selected {
                rgb(COLOR_BORDER_STRONG)
            } else {
                rgb(COLOR_BORDER)
            })
            .bg(if selected {
                rgb(COLOR_SURFACE)
            } else {
                rgb(COLOR_SURFACE_SOFT)
            })
            .cursor_pointer()
            .hover(|this| {
                if selected {
                    this.bg(rgb(COLOR_SURFACE))
                } else {
                    this.bg(rgb(COLOR_BLUE_SOFT))
                }
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.activate_tab(id);
                cx.notify();
            }))
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(if selected {
                        rgb(COLOR_TEXT)
                    } else {
                        rgb(COLOR_TEXT_MUTED)
                    })
                    .truncate()
                    .child(format!("{title}{status_dot}")),
            )
            .child(
                div()
                    .id(format!("repo-tab-close-{}", id.0))
                    .flex_none()
                    .px_1()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_FAINT))
                    .cursor_pointer()
                    .hover(|this| this.text_color(rgb(0xa03a3a)))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.close_tab(id);
                        cx.stop_propagation();
                        cx.notify();
                    }))
                    .child("x"),
            )
    }

    fn render_tag_context_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(menu) = self.tag_context_menu.clone() else {
            return div().into_any_element();
        };

        div()
            .absolute()
            .left(px(menu.x))
            .top(px(menu.y))
            .w(px(170.0))
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .text_size(px(12.0))
            .child(context_menu_item(
                "检出标签",
                !self.busy,
                {
                    let tag = menu.tag.clone();
                    move |this| this.checkout_tag(tag.clone())
                },
                cx,
            ))
            .into_any_element()
    }

    fn render_stash_context_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(menu) = self.stash_context_menu.clone() else {
            return div().into_any_element();
        };

        div()
            .absolute()
            .left(px(menu.x))
            .top(px(menu.y))
            .w(px(170.0))
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .text_size(px(12.0))
            .child(context_menu_item(
                "应用贮藏",
                !self.busy,
                {
                    let index = menu.index;
                    move |this| this.apply_stash(index)
                },
                cx,
            ))
            .child(context_menu_item(
                "弹出贮藏",
                !self.busy,
                {
                    let index = menu.index;
                    move |this| this.pop_stash(index)
                },
                cx,
            ))
            .into_any_element()
    }

    fn render_change_context_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(menu) = self.change_context_menu.clone() else {
            return div().into_any_element();
        };
        let selected_count = self.change_selection.selected(&menu.scope).len();
        let all_count = self.change_paths(menu.scope.clone()).len();

        let mut menu_el = div()
            .absolute()
            .left(px(menu.x))
            .top(px(menu.y))
            .w(px(210.0))
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .text_size(px(12.0));

        menu_el = match menu.scope {
            DiffScope::Staged => menu_el
                .child(context_menu_item(
                    "取消暂存选定文件",
                    selected_count > 0 && !self.busy,
                    |this| this.unstage_selected(),
                    cx,
                ))
                .child(context_menu_item(
                    "取消暂存所有文件",
                    all_count > 0 && !self.busy,
                    |this| this.unstage_all(),
                    cx,
                )),
            DiffScope::Unstaged => menu_el
                .child(context_menu_item(
                    "暂存选定文件",
                    selected_count > 0 && !self.busy,
                    |this| this.stage_selected(),
                    cx,
                ))
                .child(context_menu_item(
                    "暂存所有文件",
                    all_count > 0 && !self.busy,
                    |this| this.stage_all(),
                    cx,
                )),
        };

        menu_el.into_any_element()
    }

    fn render_commit_context_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(menu) = self.commit_context_menu.clone() else {
            return div().into_any_element();
        };
        let can_revert = !self.busy && menu.parent_count <= 1;

        div()
            .absolute()
            .left(px(menu.x))
            .top(px(menu.y))
            .w(px(230.0))
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .text_size(px(12.0))
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .px_3()
                    .py_1()
                    .text_size(px(11.0))
                    .text_color(rgb(COLOR_TEXT_FAINT))
                    .child(format!("提交 {}", menu.short_oid)),
            )
            .child(menu_separator())
            .child(context_menu_item(
                "软重置分支到此次提交",
                !self.busy,
                {
                    let oid = menu.oid.clone();
                    let summary = menu.summary.clone();
                    move |this| {
                        this.open_reset_confirm_dialog(
                            oid.clone(),
                            summary.clone(),
                            ResetMode::Soft,
                        )
                    }
                },
                cx,
            ))
            .child(context_menu_item(
                "混合重置分支到此次提交",
                !self.busy,
                {
                    let oid = menu.oid.clone();
                    let summary = menu.summary.clone();
                    move |this| {
                        this.open_reset_confirm_dialog(
                            oid.clone(),
                            summary.clone(),
                            ResetMode::Mixed,
                        )
                    }
                },
                cx,
            ))
            .child(context_menu_item(
                "强制重置分支到此次提交",
                !self.busy,
                {
                    let oid = menu.oid.clone();
                    let summary = menu.summary.clone();
                    move |this| {
                        this.open_reset_confirm_dialog(
                            oid.clone(),
                            summary.clone(),
                            ResetMode::Hard,
                        )
                    }
                },
                cx,
            ))
            .child(menu_separator())
            .child(context_menu_item(
                if menu.parent_count > 1 {
                    "回滚提交（合并提交暂不支持）"
                } else {
                    "回滚提交"
                },
                can_revert,
                {
                    let oid = menu.oid.clone();
                    let summary = menu.summary.clone();
                    move |this| this.open_revert_confirm_dialog(oid.clone(), summary.clone())
                },
                cx,
            ))
            .child(self.commit_copy_sha_menu_item(menu.oid.clone(), cx))
            .into_any_element()
    }

    fn commit_copy_sha_menu_item(&self, oid: String, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("context-menu-copy-commit-sha")
            .px_3()
            .py_1()
            .text_color(rgb(COLOR_TEXT))
            .bg(rgb(COLOR_SURFACE))
            .cursor_pointer()
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                cx.stop_propagation();
                this.copy_commit_sha(oid.clone(), cx);
                cx.notify();
            }))
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .on_mouse_down(MouseButton::Right, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .child("复制 SHA 到剪贴板")
    }

    fn render_changes(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let changes = self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.changes.clone())
            .unwrap_or_default();
        let unstaged_rows = changes
            .iter()
            .filter(|change| change.unstaged.is_some())
            .cloned()
            .map(|change| {
                self.change_row(change, DiffScope::Unstaged, cx)
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        let staged_rows = changes
            .into_iter()
            .filter(|change| change.staged.is_some())
            .map(|change| {
                self.change_row(change, DiffScope::Staged, cx)
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        let has_staged_selection = !self.change_selection.staged.is_empty();
        let has_unstaged_selection = !self.change_selection.unstaged.is_empty();
        let has_staged = !staged_rows.is_empty();
        let has_unstaged = !unstaged_rows.is_empty();

        div()
            .flex()
            .flex_none()
            .flex_col()
            .w(px(self.changes_width))
            .min_w(px(self.changes_width))
            .h_full()
            .bg(rgb(COLOR_PANEL_BG))
            .child(self.render_change_section(
                "暂存区",
                "staged-change-list",
                "暂存区加载中...",
                self.loading.staged(),
                staged_rows,
                vec![
                        self.button(
                            "取消暂存选定文件",
                            has_staged_selection && !self.busy,
                            |this, _, _| this.unstage_selected(),
                            cx,
                        )
                        .into_any_element(),
                        self.button(
                            "取消暂存所有文件",
                            has_staged && !self.busy,
                            |this, _, _| this.unstage_all(),
                            cx,
                        )
                        .into_any_element(),
                    ],
            ))
            .child(div().flex_none().h(px(1.0)).bg(rgb(COLOR_BORDER)))
            .child(self.render_change_section(
                "修改区",
                "unstaged-change-list",
                "修改区加载中...",
                self.loading.unstaged(),
                unstaged_rows,
                vec![
                        self.button(
                            "暂存选定文件",
                            has_unstaged_selection && !self.busy,
                            |this, _, _| this.stage_selected(),
                            cx,
                        )
                        .into_any_element(),
                        self.button(
                            "暂存所有文件",
                            has_unstaged && !self.busy,
                            |this, _, _| this.stage_all(),
                            cx,
                        )
                        .into_any_element(),
                    ],
            ))
    }

    fn render_change_section(
        &self,
        title: &'static str,
        id: &'static str,
        loading_text: &'static str,
        loading: bool,
        rows: Vec<gpui::AnyElement>,
        actions: Vec<gpui::AnyElement>,
    ) -> impl IntoElement {
        let rows = if rows.is_empty() && loading {
            vec![placeholder_row(loading_text).into_any_element()]
        } else {
            rows
        };

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h(px(0.0))
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(rgb(COLOR_BORDER))
                    .bg(rgb(COLOR_HEADER_BG))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(COLOR_TEXT))
                            .child(title),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .justify_end()
                            .gap_1()
                            .children(actions),
                    ),
            )
            .child(
                div()
                    .id(id)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .p_2()
                    .overflow_scroll()
                    .children(rows),
            )
    }

    pub(crate) fn render_column_splitter(
        &self,
        target: ResizeTarget,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let entity = cx.entity();
        let active = self.resize_state(target).is_some();
        let horizontal = target == ResizeTarget::HistoryTop;

        div()
            .flex_none()
            .relative()
            .map(|this| {
                if horizontal {
                    this.h(px(8.0)).w_full()
                } else {
                    this.w(px(8.0)).h_full()
                }
            })
            .cursor(if horizontal {
                CursorStyle::ResizeRow
            } else {
                CursorStyle::ResizeColumn
            })
            .bg(if active {
                rgb(COLOR_BLUE_SOFT)
            } else {
                rgb(COLOR_PANEL_BG)
            })
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(move |this, _event: &MouseUpEvent, _window, cx| {
                    if this.resize_state(target).is_some() {
                        this.finish_resize_column(target);
                        cx.notify();
                    }
                }),
            )
            .child(if horizontal {
                div()
                    .absolute()
                    .left(px(0.0))
                    .right(px(0.0))
                    .top(px(3.0))
                    .h(px(1.0))
                    .bg(if active {
                        rgb(COLOR_BLUE)
                    } else {
                        rgb(COLOR_BORDER)
                    })
                    .into_any_element()
            } else {
                div()
                    .absolute()
                    .left(px(3.0))
                    .top(px(0.0))
                    .bottom(px(0.0))
                    .w(px(1.0))
                    .bg(if active {
                        rgb(COLOR_BLUE)
                    } else {
                        rgb(COLOR_BORDER)
                    })
                    .into_any_element()
            })
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        window.on_mouse_event({
                            let entity = entity.clone();
                            move |event: &MouseDownEvent, _, _, cx| {
                                if !bounds.contains(&event.position) {
                                    return;
                                }
                                entity.update(cx, |this, cx| {
                                    if event.click_count >= 2 {
                                        this.reset_resize_target(target);
                                    } else {
                                        this.start_resize_column(target, event);
                                    }
                                    cx.notify();
                                });
                            }
                        });
                        window.on_mouse_event({
                            let entity = entity.clone();
                            move |event: &MouseMoveEvent, _, _, cx| {
                                let resizing = entity.read(cx).resize_state(target).is_some();
                                if !resizing || !event.dragging() {
                                    return;
                                }
                                entity.update(cx, |this, cx| {
                                    this.update_resize_column(target, event);
                                    cx.notify();
                                });
                            }
                        });
                        window.on_mouse_event(move |_: &MouseUpEvent, _, _, cx| {
                            let resizing = entity.read(cx).resize_state(target).is_some();
                            if !resizing {
                                return;
                            }
                            entity.update(cx, |this, cx| {
                                this.finish_resize_column(target);
                                cx.notify();
                            });
                        });
                    },
                )
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .right(px(0.0))
                .bottom(px(0.0)),
            )
    }

    fn change_row(
        &self,
        change: khaslana::WorktreeChange,
        scope: DiffScope,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let path = change.path.clone();
        let selected = self.is_change_selected(&scope, &change.path);
        let state = match scope {
            DiffScope::Staged => change.staged.as_ref(),
            DiffScope::Unstaged => change.unstaged.as_ref(),
        }
        .map(|state| state.label())
        .unwrap_or(" ");

        div()
            .id(format!("change-{}-{}", diff_scope_id(&scope), change.path))
            .flex()
            .flex_none()
            .items_center()
            .gap_1()
            .h(px(CHANGE_ROW_HEIGHT))
            .px_2()
            .py_1()
            .overflow_hidden()
            .rounded_sm()
            .cursor_pointer()
            .bg(if selected {
                rgb(COLOR_ROW_SELECTED)
            } else {
                rgb(COLOR_SURFACE)
            })
            .border_1()
            .border_color(if selected {
                rgb(COLOR_BORDER_STRONG)
            } else {
                rgb(COLOR_BORDER)
            })
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener({
                    let path = path.clone();
                    let scope = scope.clone();
                    move |this, event: &MouseDownEvent, _window, cx| {
                        this.select_change_from_mouse(path.clone(), scope.clone(), event);
                        this.change_context_menu = None;
                        cx.notify();
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    this.open_change_context_menu(path.clone(), scope.clone(), event);
                    cx.notify();
                }),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(24.0))
                    .text_size(px(11.0))
                    .font_family("monospace")
                    .text_color(rgb(COLOR_BLUE))
                    .child(state),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .truncate()
                    .child(change.path),
            )
    }

    fn render_diff_and_commit(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .bg(rgb(COLOR_PANEL_BG))
            .child(self.render_diff(cx))
            .child(self.render_commit_box(window, cx))
    }

    fn render_diff(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self
            .diff
            .as_ref()
            .map(|diff| format!("差异：{} ({})", diff.path, diff_scope_label(&diff.scope)))
            .unwrap_or_else(|| "差异".to_string());
        let diff_rows = self.render_diff_rows(cx);

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(260.0))
            .child(section_header(title))
            .child(
                div()
                    .id("diff-scroll")
                    .flex()
                    .flex_col()
                    .gap_0()
                    .min_w(px(0.0))
                    .p_2()
                    .overflow_scroll()
                    .font_family("Consolas, monospace")
                    .text_size(px(12.0))
                    .bg(rgb(COLOR_PANEL_BG))
                    .children(diff_rows),
            )
    }

    fn render_diff_rows(&self, cx: &mut Context<Self>) -> Vec<gpui::AnyElement> {
        let Some(diff) = self.diff.as_ref() else {
            return vec![
                diff_line(
                    DiffLineKind::Context,
                    None,
                    None,
                    "请选择一个变更文件查看差异".to_string(),
                )
                .into_any_element(),
            ];
        };

        self.render_file_diff_rows(
            diff,
            self.diff_headers_expanded,
            DiffHeaderTarget::Worktree,
            cx,
        )
    }

    pub(crate) fn render_file_diff_rows(
        &self,
        diff: &FileDiff,
        headers_expanded: bool,
        header_target: DiffHeaderTarget,
        cx: &mut Context<Self>,
    ) -> Vec<gpui::AnyElement> {
        let header_count = diff
            .lines
            .iter()
            .take_while(|line| line.kind == DiffLineKind::Header)
            .count();
        let mut rows = Vec::new();

        if header_count > 0 {
            let summary = if headers_expanded {
                "Diff 元信息（点击折叠）"
            } else {
                "Diff 元信息（点击展开）"
            };
            rows.push(diff_header_toggle(summary, header_target, cx).into_any_element());
            if headers_expanded {
                rows.extend(
                    diff.lines.iter().take(header_count).cloned().map(|line| {
                        diff_line(line.kind, None, None, line.content).into_any_element()
                    }),
                );
            }
        }

        rows.extend(diff.lines.iter().skip(header_count).cloned().map(|line| {
            diff_line(line.kind, line.old_lineno, line.new_lineno, line.content).into_any_element()
        }));
        if rows.is_empty() {
            rows.push(
                diff_line(
                    DiffLineKind::Context,
                    None,
                    None,
                    if diff.is_binary {
                        "二进制文件仅显示元信息".to_string()
                    } else {
                        "没有可显示的文本差异".to_string()
                    },
                )
                .into_any_element(),
            );
        }
        rows
    }

    fn render_commit_box(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .border_t_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(COLOR_PANEL_BG))
            .child(self.input(FieldId::CommitMessage, false, window, cx))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(COLOR_TEXT_MUTED))
                            .child(self.status.clone()),
                    )
                    .child(self.button(
                        "提交",
                        self.repo_path.is_some() && !self.busy,
                        |this, _, _| this.commit(),
                        cx,
                    )),
            )
    }

    fn render_status(&self) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_1()
            .border_t_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(COLOR_HEADER_BG))
            .text_size(px(12.0))
            .child(
                div()
                    .text_color(if self.busy {
                        rgb(COLOR_BLUE_DARK)
                    } else {
                        rgb(COLOR_TEXT_MUTED)
                    })
                    .child(if self.busy {
                        format!("{}...", self.status)
                    } else {
                        self.status.clone()
                    }),
            )
            .when_some(self.last_error.clone(), |this, error| {
                this.child(
                    div()
                        .text_color(rgb(0xa03a3a))
                        .child(format!("错误：{error}")),
                )
            })
    }

    fn render_credentials(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(pending) = self.pending_credential.as_ref() else {
            return div().into_any_element();
        };

        div()
            .absolute()
            .top(px(70.0))
            .right(px(18.0))
            .w(px(420.0))
            .p_3()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .gap_2()
            .cursor(CursorStyle::Arrow)
            .occlude()
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(COLOR_TEXT))
                    .child("需要凭据"),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(format!("远端：{}", pending.request.url)),
            )
            .child(self.input(FieldId::CredentialUsername, true, window, cx))
            .child(self.input(FieldId::CredentialSecret, true, window, cx))
            .child(self.input(FieldId::CredentialKeyPath, true, window, cx))
            .child(self.input(FieldId::CredentialPassphrase, true, window, cx))
            .child(
                div()
                    .id("save-credential")
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.save_credential = !this.save_credential;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .size(px(14.0))
                            .border_1()
                            .border_color(rgb(COLOR_BORDER_STRONG))
                            .bg(if self.save_credential {
                                rgb(COLOR_BLUE)
                            } else {
                                rgb(COLOR_SURFACE)
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(COLOR_TEXT))
                            .child("保存到系统凭据管理器"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .when(!self.save_credential, |this| this.opacity(0.55))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(COLOR_TEXT_MUTED))
                            .child("复用范围"),
                    )
                    .child(self.credential_scope_button("仅此远端", CredentialScope::RemoteUrl, cx))
                    .child(self.credential_scope_button("同站点", CredentialScope::Host, cx)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .justify_end()
                    .child(self.button(
                        "使用凭据",
                        !self.busy,
                        |this, _, _| this.use_credentials(),
                        cx,
                    ))
                    .child(self.button(
                        "取消",
                        !self.busy,
                        |this, _, _| {
                            this.show_next_credential_request();
                        },
                        cx,
                    )),
            )
            .into_any_element()
    }

    fn render_dialogs(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(dialog) = self.active_dialog.clone() else {
            return div().into_any_element();
        };

        let content = match dialog {
            DialogState::CloneRepo => self.render_clone_dialog(window, cx).into_any_element(),
            DialogState::CreateBranch => self
                .render_create_branch_dialog(window, cx)
                .into_any_element(),
            DialogState::RenameBranch { branch } => self
                .render_rename_branch_dialog(branch, window, cx)
                .into_any_element(),
            DialogState::ConfirmReset { oid, summary, mode } => self
                .render_confirm_reset_dialog(oid, summary, mode, cx)
                .into_any_element(),
            DialogState::ConfirmRevert { oid, summary } => self
                .render_confirm_revert_dialog(oid, summary, cx)
                .into_any_element(),
            DialogState::CredentialManager => {
                self.render_credential_manager_dialog(cx).into_any_element()
            }
            DialogState::ConfirmDeleteCredential { record_id, label } => self
                .render_confirm_delete_credential_dialog(record_id, label, cx)
                .into_any_element(),
        };

        div()
            .absolute()
            .top(px(0.0))
            .left(px(0.0))
            .right(px(0.0))
            .bottom(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x10182055))
            .cursor(CursorStyle::Arrow)
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.close_dialog();
                    cx.notify();
                }),
            )
            .child(content)
            .into_any_element()
    }

    fn render_clone_dialog(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.dialog_panel("克隆仓库", cx)
            .child(self.input(FieldId::CloneUrl, false, window, cx))
            .child(self.input(FieldId::ClonePath, false, window, cx))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .gap_2()
                    .child(self.button(
                        "选择目录",
                        !self.busy,
                        |this, _, _| this.browse_clone_target(),
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.button(
                                "取消",
                                !self.busy,
                                |this, _, _| this.close_dialog(),
                                cx,
                            ))
                            .child(self.button(
                                "克隆",
                                !self.busy,
                                |this, _, _| this.clone_repo(),
                                cx,
                            )),
                    ),
            )
    }

    fn render_create_branch_dialog(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.dialog_panel("新建分支", cx)
            .child(self.input(FieldId::BranchName, false, window, cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(self.button("取消", !self.busy, |this, _, _| this.close_dialog(), cx))
                    .child(self.button(
                        "创建",
                        self.repo_path.is_some() && !self.busy,
                        |this, _, _| this.create_branch(),
                        cx,
                    )),
            )
    }

    fn render_rename_branch_dialog(
        &self,
        branch: String,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.dialog_panel("重命名分支", cx)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(format!("当前分支：{branch}")),
            )
            .child(self.input(FieldId::BranchRename, false, window, cx))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(self.button("取消", !self.busy, |this, _, _| this.close_dialog(), cx))
                    .child(self.button(
                        "重命名",
                        !self.busy,
                        {
                            let branch = branch.clone();
                            move |this, _, _| this.rename_branch(branch.clone())
                        },
                        cx,
                    )),
            )
    }

    fn render_confirm_reset_dialog(
        &self,
        oid: String,
        summary: String,
        mode: ResetMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mode_label = reset_mode_label(mode);
        let mode_help = reset_mode_help(mode);
        self.dialog_panel("确认重置分支", cx)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .child(format!("目标提交：{} {}", short_oid(&oid), summary)),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(format!("将当前分支重置到该提交。{mode_label}：{mode_help}")),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(self.button("取消", !self.busy, |this, _, _| this.close_dialog(), cx))
                    .child(self.button(
                        "确认重置",
                        !self.busy,
                        {
                            let oid = oid.clone();
                            move |this, _, _| this.reset_to_commit(oid.clone(), mode)
                        },
                        cx,
                    )),
            )
    }

    fn render_confirm_revert_dialog(
        &self,
        oid: String,
        summary: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.dialog_panel("确认回滚提交", cx)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .child(format!("目标提交：{} {}", short_oid(&oid), summary)),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child("确认后会创建一个新的提交，用于撤销该提交引入的修改。"),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(self.button("取消", !self.busy, |this, _, _| this.close_dialog(), cx))
                    .child(self.button(
                        "确认回滚",
                        !self.busy,
                        {
                            let oid = oid.clone();
                            move |this, _, _| this.revert_commit(oid.clone())
                        },
                        cx,
                    )),
            )
    }

    fn render_credential_manager_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = if self.credential_records.is_empty() {
            vec![
                placeholder_row("暂无已保存凭据。远程操作时勾选保存后会出现在这里。")
                    .into_any_element(),
            ]
        } else {
            self.credential_records
                .iter()
                .cloned()
                .map(|record| self.credential_record_row(record, cx).into_any_element())
                .collect::<Vec<_>>()
        };

        div()
            .id("dialog-凭据管理")
            .w(px(780.0))
            .max_h(px(620.0))
            .p_4()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .gap_3()
            .cursor(CursorStyle::Arrow)
            .occlude()
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(COLOR_TEXT))
                            .child("凭据管理"),
                    )
                    .child(self.button(
                        "刷新",
                        !self.busy,
                        |this, _, _| this.reload_credential_records("凭据列表已刷新"),
                        cx,
                    )),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(
                        "密文仅保存在系统凭据管理器；这里不显示、不复制密码、PAT 或 SSH 密码短语。",
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_h(px(0.0))
                    .max_h(px(440.0))
                    .border_1()
                    .border_color(rgb(COLOR_BORDER))
                    .rounded_sm()
                    .child(self.credential_manager_header())
                    .child(
                        div()
                            .id("credential-record-list")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap_0()
                            .min_w(px(0.0))
                            .min_h(px(0.0))
                            .overflow_y_scroll()
                            .children(rows),
                    ),
            )
            .child(div().flex().justify_end().child(self.button(
                "关闭",
                !self.busy,
                |this, _, _| this.close_dialog(),
                cx,
            )))
    }

    fn credential_manager_header(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap_2()
            .px_2()
            .py_2()
            .border_b_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(COLOR_HEADER_BG))
            .text_size(px(11.0))
            .font_weight(gpui::FontWeight::BOLD)
            .text_color(rgb(COLOR_TEXT_MUTED))
            .child(div().flex_none().w(px(112.0)).child("类型"))
            .child(div().flex_none().w(px(72.0)).child("范围"))
            .child(div().flex_1().min_w(px(0.0)).child("站点 / 远端"))
            .child(div().flex_none().w(px(92.0)).child("用户名"))
            .child(div().flex_none().w(px(92.0)).child("SSH Key"))
            .child(div().flex_none().w(px(132.0)).child("更新时间"))
            .child(div().flex_none().w(px(106.0)).child("操作"))
    }

    fn credential_record_row(
        &self,
        record: CredentialRecord,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let record_id = record.id.clone();
        let delete_id = record.id.clone();
        let label = credential_record_label(&record);
        let target = credential_display_target(&record);
        let key_file = credential_key_filename(&record);
        div()
            .id(format!("credential-record-{}", record.id))
            .flex()
            .flex_none()
            .items_center()
            .gap_2()
            .px_2()
            .py_2()
            .border_b_1()
            .border_color(rgb(COLOR_BORDER))
            .text_size(px(12.0))
            .bg(rgb(COLOR_SURFACE))
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .child(
                div()
                    .flex_none()
                    .w(px(112.0))
                    .text_color(rgb(COLOR_TEXT))
                    .truncate()
                    .child(credential_kind_label(record.kind)),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(72.0))
                    .text_color(rgb(COLOR_BLUE_DARK))
                    .truncate()
                    .child(credential_scope_label(record.scope)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_color(rgb(COLOR_TEXT))
                    .truncate()
                    .child(target),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(92.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .truncate()
                    .child(record.username),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(92.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .truncate()
                    .child(key_file),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(132.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .truncate()
                    .child(timestamp_label(record.updated_at)),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(106.0))
                    .flex()
                    .gap_1()
                    .child(self.button(
                        "测试",
                        !self.busy,
                        move |this, _, _| this.test_credential_record(record_id.clone()),
                        cx,
                    ))
                    .child(self.button(
                        "删除",
                        !self.busy,
                        move |this, _, _| {
                            this.open_delete_credential_confirm(delete_id.clone(), label.clone())
                        },
                        cx,
                    )),
            )
    }

    fn render_confirm_delete_credential_dialog(
        &self,
        record_id: String,
        label: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.dialog_panel("删除凭据", cx)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .child(format!("确认删除凭据：{label}")),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child("删除会同时移除非敏感索引和系统凭据管理器中的密文。"),
            )
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap_2()
                    .child(self.button(
                        "取消",
                        !self.busy,
                        |this, _, _| {
                            this.active_dialog = Some(DialogState::CredentialManager);
                        },
                        cx,
                    ))
                    .child(self.button(
                        "确认删除",
                        !self.busy,
                        move |this, _, _| this.delete_credential_record(record_id.clone()),
                        cx,
                    )),
            )
    }

    fn dialog_panel(
        &self,
        title: &'static str,
        _cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        div()
            .id(format!("dialog-{title}"))
            .w(px(480.0))
            .p_4()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .gap_3()
            .cursor(CursorStyle::Arrow)
            .occlude()
            .on_mouse_down(MouseButton::Left, |_event, _window, cx| {
                cx.stop_propagation();
            })
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(COLOR_TEXT))
                    .child(title),
            )
    }
}

impl Deref for RepositoryView {
    type Target = RepoTabState;

    fn deref(&self) -> &Self::Target {
        self.active_tab_state()
    }
}

impl DerefMut for RepositoryView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.active_tab_state_mut()
    }
}

impl Render for RepositoryView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.drain_pending_events(cx);

        div()
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .bg(rgb(COLOR_APP_BG))
            .text_color(rgb(COLOR_TEXT))
            .on_key_down(cx.listener(Self::handle_key))
            .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                if this.mouse_down_inside_context_menu(event) {
                    return;
                }
                if this.branch_context_menu.is_some()
                    || this.change_context_menu.is_some()
                    || this.tag_context_menu.is_some()
                    || this.stash_context_menu.is_some()
                    || this.commit_context_menu.is_some()
                {
                    this.branch_context_menu = None;
                    this.change_context_menu = None;
                    this.tag_context_menu = None;
                    this.stash_context_menu = None;
                    this.commit_context_menu = None;
                    cx.notify();
                }
            }))
            .child(self.render_toolbar(window, cx))
            .child(self.render_tab_bar(cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .child(self.render_sidebar(window, cx))
                    .child(self.render_column_splitter(ResizeTarget::Sidebar, cx))
                    .child(match self.main_mode {
                        MainMode::Worktree => div()
                            .flex()
                            .flex_1()
                            .min_w(px(0.0))
                            .min_h(px(0.0))
                            .child(self.render_changes(cx))
                            .child(self.render_column_splitter(ResizeTarget::Changes, cx))
                            .child(self.render_diff_and_commit(window, cx))
                            .into_any_element(),
                        MainMode::History => self.render_history_view(cx).into_any_element(),
                    }),
            )
            .child(self.render_status())
            .child(self.render_branch_context_menu(cx))
            .child(self.render_change_context_menu(cx))
            .child(self.render_commit_context_menu(cx))
            .child(self.render_tag_context_menu(cx))
            .child(self.render_stash_context_menu(cx))
            .child(self.render_dialogs(window, cx))
            .child(self.render_credentials(window, cx))
    }
}

impl Focusable for RepositoryView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.clone_url.focus.clone()
    }
}

trait EmptyStringExt {
    fn if_empty_then(self, f: impl FnOnce() -> String) -> String;
}

impl EmptyStringExt for String {
    fn if_empty_then(self, f: impl FnOnce() -> String) -> String {
        if self.is_empty() { f() } else { self }
    }
}

fn normalize_repo_path(path: &Path) -> String {
    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_lowercase()
}

fn short_oid(oid: &str) -> &str {
    oid.get(..8).unwrap_or(oid)
}

fn reset_mode_label(mode: ResetMode) -> &'static str {
    match mode {
        ResetMode::Soft => "软重置",
        ResetMode::Mixed => "混合重置",
        ResetMode::Hard => "强制重置",
    }
}

fn reset_mode_help(mode: ResetMode) -> &'static str {
    match mode {
        ResetMode::Soft => "保留暂存区和工作区修改",
        ResetMode::Mixed => "重置暂存区，保留工作区修改",
        ResetMode::Hard => "重置暂存区和工作区，丢弃未提交修改",
    }
}

fn timestamp_label(seconds: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, 0)
        .map(|time| {
            time.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

fn point_in_menu(x: f32, y: f32, menu_x: f32, menu_y: f32, width: f32, height: f32) -> bool {
    x >= menu_x && x <= menu_x + width && y >= menu_y && y <= menu_y + height
}

fn dedupe_repo_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = BTreeSet::new();
    paths
        .into_iter()
        .filter(|path| seen.insert(normalize_repo_path(path)))
        .collect()
}

#[cfg(test)]
mod app_tests {
    use super::*;

    #[test]
    fn session_json_round_trips_multiple_repositories() {
        let state = SessionState {
            repo_paths: vec![PathBuf::from("C:/work/a"), PathBuf::from("C:/work/b")],
            active_repo_path: Some(PathBuf::from("C:/work/b")),
        };

        let json = serde_json::to_string(&state).expect("encode session");
        let decoded: SessionState = serde_json::from_str(&json).expect("decode session");

        assert_eq!(decoded.repo_paths, state.repo_paths);
        assert_eq!(decoded.active_repo_path, state.active_repo_path);
    }

    #[test]
    fn session_paths_are_deduped_in_original_order() {
        let paths = dedupe_repo_paths(vec![
            PathBuf::from("C:/work/a"),
            PathBuf::from("C:/work/b"),
            PathBuf::from("C:/work/a"),
        ]);

        assert_eq!(
            paths,
            vec![PathBuf::from("C:/work/a"), PathBuf::from("C:/work/b")]
        );
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1280.0), px(820.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Khaslana".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(RepositoryView::new_with_session);
                window.focus(&view.read(cx).focus_handle(cx));
                view
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
