#![cfg_attr(windows, windows_subsystem = "windows")]

mod history_view;
mod sidebar_view;
mod ui_helpers;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Instant;

use async_channel::{Receiver, Sender};
use directories::ProjectDirs;
use git2::Repository;
use gpui::{
    App, Application, Bounds, ClipboardItem, Context, CursorStyle, FocusHandle, Focusable,
    KeyDownEvent, ListHorizontalSizingBehavior, ListSizingBehavior, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ScrollHandle, SharedString, TitlebarOptions,
    UniformListScrollHandle, WeakEntity, Window, WindowBounds, WindowOptions, canvas, div, point,
    prelude::*, px, rgb, rgba, size, uniform_list,
};
use khaslana::{
    BranchKind, BranchName, CommitFileChange, CommitInfo, CommitMessage, MemoryCredentialStore,
    CredentialProvider,
    CredentialRecord, CredentialRequest, CredentialScope, CredentialStore, DiffEncodingChoice,
    DiffLineKind, DiffScope, FileDiff, GitCredential, GitService, HistoryScope,
    KeyringCredentialStore, OperationEvent, ProgressEmitter, RemoteCredentialPolicy, RemoteInfo,
    RemoteName, RepoPath, RepositorySnapshot, ResetMode, TagName, credential_display_target,
    credential_key_filename, credential_kind_label, credential_record_is_compatible_with_url,
    credential_record_label, credential_record_matches_remote_url, credential_scope_label,
    test_credential_connection,
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
pub(crate) const BRANCH_MENU_WIDTH: f32 = 190.0;
pub(crate) const BRANCH_MENU_HEIGHT: f32 = 230.0;
const CHANGE_MENU_WIDTH: f32 = 210.0;
const CHANGE_MENU_HEIGHT: f32 = 255.0;
pub(crate) const TAG_MENU_WIDTH: f32 = 170.0;
pub(crate) const TAG_MENU_HEIGHT: f32 = 80.0;
pub(crate) const STASH_MENU_WIDTH: f32 = 170.0;
pub(crate) const STASH_MENU_HEIGHT: f32 = 110.0;
const COMMIT_MENU_WIDTH: f32 = 230.0;
const COMMIT_MENU_HEIGHT: f32 = 230.0;
const ENCODING_MENU_WIDTH: f32 = 170.0;
const MENU_VIEWPORT_MARGIN: f32 = 8.0;
const MAX_CONCURRENT_REPO_LOADS: usize = 2;
const LARGE_DIFF_CACHE_LINE_LIMIT: usize = 20_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldId {
    CloneUrl,
    ClonePath,
    BranchName,
    BranchRename,
    RemoteName,
    RemoteUrl,
    CommitMessage,
    CredentialUsername,
    CredentialSecret,
    CredentialKeyPath,
    CredentialPassphrase,
    CredentialRemoteUrl,
}

#[derive(Clone, Debug)]
struct TextEditState {
    value: String,
    secret: bool,
    caret: usize,
    selection_anchor: Option<usize>,
}

impl TextEditState {
    fn new() -> Self {
        Self {
            value: String::new(),
            secret: false,
            caret: 0,
            selection_anchor: None,
        }
    }

    #[cfg(test)]
    fn for_test(value: &str, secret: bool) -> Self {
        Self {
            value: value.to_string(),
            secret,
            caret: value.len(),
            selection_anchor: None,
        }
    }

    fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.caret = self.value.len();
        self.selection_anchor = None;
    }

    fn clear(&mut self) {
        self.value.clear();
        self.caret = 0;
        self.selection_anchor = None;
    }

    fn display_text(&self) -> String {
        if self.secret {
            "*".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }

    fn display_byte_for_value_byte(&self, value_byte: usize) -> usize {
        if self.secret {
            self.value[..value_byte].chars().count()
        } else {
            value_byte
        }
    }

    fn selected_range(&self) -> Option<std::ops::Range<usize>> {
        let anchor = self.selection_anchor?;
        if anchor == self.caret {
            None
        } else if anchor < self.caret {
            Some(anchor..self.caret)
        } else {
            Some(self.caret..anchor)
        }
    }

    fn selected_text(&self) -> Option<String> {
        self.selected_range()
            .map(|range| self.value[range].to_string())
    }

    fn copyable_selected_text(&self) -> Option<String> {
        (!self.secret).then(|| self.selected_text()).flatten()
    }

    fn select_all(&mut self) {
        self.caret = self.value.len();
        self.selection_anchor = Some(0);
    }

    fn delete_selection(&mut self) -> bool {
        let Some(range) = self.selected_range() else {
            return false;
        };
        let start = range.start;
        self.value.replace_range(range, "");
        self.caret = start;
        self.selection_anchor = None;
        true
    }

    fn insert_text(&mut self, text: &str, multiline: bool) {
        self.delete_selection();
        let text = if multiline {
            text.to_string()
        } else {
            text.replace(['\r', '\n'], "")
        };
        self.value.insert_str(self.caret, &text);
        self.caret += text.len();
        self.selection_anchor = None;
    }

    fn delete_backward(&mut self) {
        if self.delete_selection() || self.caret == 0 {
            return;
        }
        let previous = self.value[..self.caret]
            .char_indices()
            .last()
            .map(|(index, _)| index)
            .unwrap_or(0);
        self.value.replace_range(previous..self.caret, "");
        self.caret = previous;
    }

    fn delete_forward(&mut self) {
        if self.delete_selection() || self.caret >= self.value.len() {
            return;
        }
        let next = self.value[self.caret..]
            .char_indices()
            .nth(1)
            .map(|(index, _)| self.caret + index)
            .unwrap_or(self.value.len());
        self.value.replace_range(self.caret..next, "");
    }

    fn move_caret_to(&mut self, position: usize, extend_selection: bool) {
        let position = clamp_to_char_boundary(&self.value, position);
        if extend_selection {
            if self.selection_anchor.is_none() {
                self.selection_anchor = Some(self.caret);
            }
        } else {
            self.selection_anchor = None;
        }
        self.caret = position;
    }

    fn move_left(&mut self, extend_selection: bool) {
        if !extend_selection && let Some(range) = self.selected_range() {
            self.move_caret_to(range.start, false);
            return;
        }
        let previous = self.value[..self.caret]
            .char_indices()
            .last()
            .map(|(index, _)| index)
            .unwrap_or(0);
        self.move_caret_to(previous, extend_selection);
    }

    fn move_right(&mut self, extend_selection: bool) {
        if !extend_selection && let Some(range) = self.selected_range() {
            self.move_caret_to(range.end, false);
            return;
        }
        let next = self.value[self.caret..]
            .char_indices()
            .nth(1)
            .map(|(index, _)| self.caret + index)
            .unwrap_or(self.value.len());
        self.move_caret_to(next, extend_selection);
    }

    fn byte_for_approx_x(&self, x: f32) -> usize {
        let mut width = 0.0;
        let mut previous = 0;
        for (index, ch) in self.value.char_indices() {
            let char_width = approx_input_char_width(ch);
            if x < width + char_width / 2.0 {
                return index;
            }
            width += char_width;
            previous = index + ch.len_utf8();
        }
        previous
    }
}

#[derive(Clone, Debug)]
struct TextFieldState {
    focus: FocusHandle,
    placeholder: SharedString,
    edit: TextEditState,
}

impl TextFieldState {
    fn new(cx: &mut Context<RepositoryView>, placeholder: impl Into<SharedString>) -> Self {
        Self {
            focus: cx.focus_handle().tab_stop(true),
            placeholder: placeholder.into(),
            edit: TextEditState::new(),
        }
    }

    fn secret(mut self) -> Self {
        self.edit.secret = true;
        self
    }
}

impl Deref for TextFieldState {
    type Target = TextEditState;

    fn deref(&self) -> &Self::Target {
        &self.edit
    }
}

impl DerefMut for TextFieldState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.edit
    }
}

fn clamp_to_char_boundary(value: &str, mut position: usize) -> usize {
    position = position.min(value.len());
    while position > 0 && !value.is_char_boundary(position) {
        position -= 1;
    }
    position
}

fn approx_input_char_width(ch: char) -> f32 {
    if ch == '\t' {
        28.0
    } else if ch.is_ascii() {
        7.0
    } else if ch_width_is_wide(ch) {
        14.0
    } else {
        8.5
    }
}

fn ch_width_is_wide(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11FF
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE6F
            | 0xFF00..=0xFFEF
    )
}

#[derive(Clone, Debug)]
struct PendingCredential {
    tab_id: Option<RepoTabId>,
    request: CredentialRequest,
    response_tx: Arc<Mutex<Option<mpsc::Sender<khaslana::Result<Option<GitCredential>>>>>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CredentialFormMode {
    Https,
    Ssh,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct RemoteCredentialBindings {
    #[serde(default)]
    remotes: Vec<RemoteCredentialBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
struct RemoteCredentialBinding {
    repo_path: String,
    remote_name: String,
    remote_url: String,
    policy: RemoteCredentialPolicy,
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
    ConfirmDiscardChange {
        scope: DiffScope,
        target: DiscardTarget,
        paths: Vec<String>,
    },
    CredentialManager,
    CredentialForm {
        editing: Option<String>,
    },
    RemoteManager,
    RemoteForm {
        editing: Option<String>,
    },
    ConfirmDeleteRemote {
        name: String,
    },
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

#[derive(Clone, Debug, Default)]
struct ChangeListIndexes {
    staged: Vec<usize>,
    unstaged: Vec<usize>,
}

impl ChangeListIndexes {
    fn rebuild(changes: &[khaslana::WorktreeChange]) -> Self {
        let mut indexes = Self::default();
        for (index, change) in changes.iter().enumerate() {
            if change.staged.is_some() {
                indexes.staged.push(index);
            }
            if change.unstaged.is_some() {
                indexes.unstaged.push(index);
            }
        }
        indexes
    }
}

#[derive(Clone, Debug)]
struct ChangeContextMenu {
    path: String,
    scope: DiffScope,
    x: f32,
    y: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DiscardTarget {
    Single,
    Selected,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EncodingMenuTarget {
    Worktree,
    History,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiffRenderRow {
    HeaderToggle,
    DiffLine(usize),
    Empty,
}

fn discard_paths_preview(paths: &[String]) -> String {
    let mut preview = paths.iter().take(5).cloned().collect::<Vec<_>>().join("\n");
    if paths.len() > 5 {
        if !preview.is_empty() {
            preview.push('\n');
        }
        preview.push_str(&format!("... 以及另外 {} 个文件", paths.len() - 5));
    }
    preview
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DiffRenderModel {
    row_count: usize,
    header_count: usize,
    headers_expanded: bool,
    empty: bool,
}

impl DiffRenderModel {
    fn row_at(&self, row_index: usize) -> DiffRenderRow {
        if self.empty {
            return DiffRenderRow::Empty;
        }
        if self.header_count > 0 {
            if row_index == 0 {
                return DiffRenderRow::HeaderToggle;
            }
            if self.headers_expanded && row_index <= self.header_count {
                return DiffRenderRow::DiffLine(row_index - 1);
            }
        }
        let body_start = if self.header_count > 0 { 1 } else { 0 };
        let header_offset = if self.headers_expanded {
            self.header_count
        } else {
            0
        };
        DiffRenderRow::DiffLine(self.header_count + row_index - body_start - header_offset)
    }
}

fn diff_render_model_for(diff: Option<&FileDiff>, headers_expanded: bool) -> DiffRenderModel {
    let Some(diff) = diff else {
        return DiffRenderModel {
            row_count: 1,
            header_count: 0,
            headers_expanded,
            empty: true,
        };
    };
    let header_count = diff
        .lines
        .iter()
        .take_while(|line| line.kind == DiffLineKind::Header)
        .count();
    let mut row_count = diff.lines.len().saturating_sub(header_count);
    if header_count > 0 {
        row_count += 1;
        if headers_expanded {
            row_count += header_count;
        }
    }
    DiffRenderModel {
        row_count: row_count.max(1),
        header_count,
        headers_expanded,
        empty: row_count == 0,
    }
}

#[cfg(test)]
fn diff_render_rows_for(diff: Option<&FileDiff>, headers_expanded: bool) -> Vec<DiffRenderRow> {
    let model = diff_render_model_for(diff, headers_expanded);
    (0..model.row_count)
        .map(|index| model.row_at(index))
        .collect()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct DiffEncodingPreferences {
    repositories: BTreeMap<String, DiffEncodingChoice>,
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
    pub(crate) change_indexes: ChangeListIndexes,
    pub(crate) diff: Option<Arc<FileDiff>>,
    pub(crate) diff_headers_expanded: bool,
    pub(crate) main_mode: MainMode,
    pub(crate) history_commits: Vec<CommitInfo>,
    pub(crate) history_has_more: bool,
    pub(crate) history_selected_commit: Option<String>,
    pub(crate) history_files: Vec<CommitFileChange>,
    pub(crate) history_selected_file: Option<String>,
    pub(crate) history_diff: Option<Arc<FileDiff>>,
    pub(crate) history_diff_headers_expanded: bool,
    pub(crate) history_loading: HistoryLoading,
    pub(crate) history_scope: HistoryScope,
    pub(crate) history_graph_rows: Vec<history_view::CommitGraphRow>,
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
            change_indexes: ChangeListIndexes::default(),
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
            history_scope: HistoryScope::default(),
            history_graph_rows: Vec::new(),
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

    fn release_large_diff_caches(&mut self) {
        if self
            .diff
            .as_ref()
            .is_some_and(|diff| diff.lines.len() > LARGE_DIFF_CACHE_LINE_LIMIT)
        {
            self.diff = None;
            self.diff_headers_expanded = false;
        }
        if self
            .history_diff
            .as_ref()
            .is_some_and(|diff| diff.lines.len() > LARGE_DIFF_CACHE_LINE_LIMIT)
        {
            self.history_diff = None;
            self.history_diff_headers_expanded = false;
        }
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

#[derive(Clone, Debug)]
struct RepositoryLoadRequest {
    tab_id: RepoTabId,
    path: PathBuf,
    started: &'static str,
    finished: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LoadPriority {
    Background,
    User,
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
    RepositoryLoadFinished {
        tab_id: RepoTabId,
        load_id: u64,
    },
    OperationFinished {
        tab_id: Option<RepoTabId>,
        message: String,
        snapshot: Option<RepositorySnapshot>,
        diff: Option<FileDiff>,
    },
    DiscardChangeFinished {
        tab_id: RepoTabId,
        message: String,
        snapshot: RepositorySnapshot,
        changes: Vec<khaslana::WorktreeChange>,
        load_id: u64,
    },
    HistoryCommitsLoaded {
        tab_id: RepoTabId,
        commits: Vec<CommitInfo>,
        append: bool,
        has_more: bool,
        scope: HistoryScope,
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
        response_tx: Arc<Mutex<Option<mpsc::Sender<khaslana::Result<Option<GitCredential>>>>>>,
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
    remote_bindings: Arc<Mutex<RemoteCredentialBindings>>,
    tx: Sender<UiEvent>,
    rejected_record_ids: Arc<Mutex<Vec<String>>>,
    last_stored_attempt: Arc<Mutex<Option<(String, String)>>>,
    tab_id: RepoTabId,
}

impl TabCredentialProvider {
    fn new(
        store: Arc<dyn khaslana::CredentialStore>,
        remote_bindings: Arc<Mutex<RemoteCredentialBindings>>,
        tx: Sender<UiEvent>,
        tab_id: RepoTabId,
    ) -> Self {
        Self {
            store,
            remote_bindings,
            tx,
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

        let binding_policy = remote_binding_for_request(&self.remote_bindings, &request);
        let stored = match binding_policy {
            RemoteCredentialPolicy::NoCredential => Ok(None),
            RemoteCredentialPolicy::Record(record_id) => {
                if rejected_record_ids.contains(&record_id) {
                    Ok(None)
                } else {
                    match self.store.credential_for_record(&record_id) {
                        Ok(Some(credential)) => {
                            let touched = self.store.touch_record(&record_id)?;
                            let Some(record) = touched else {
                                return Ok(None);
                            };
                            if !credential_record_matches_remote_url(&record, &request.url) {
                                return Ok(None);
                            }
                            Ok(Some(khaslana::credentials::StoredCredential {
                                record,
                                credential,
                            }))
                        }
                        Ok(None) => Ok(None),
                        Err(err) => Err(err),
                    }
                }
            }
            RemoteCredentialPolicy::AutoMatch => {
                self.store.get_stored(&request, &rejected_record_ids)
            }
        };
        match stored {
            Ok(Some(stored)) => {
                if let Ok(mut last) = self.last_stored_attempt.lock() {
                    *last = Some((request.url.clone(), stored.record.id.clone()));
                }
                return Ok(Some(stored.credential));
            }
            Ok(None) => {}
            Err(err) => tracing::warn!("keyring read skipped: {err}"),
        }

        let (response_tx, response_rx) = mpsc::channel();
        let response_tx = Arc::new(Mutex::new(Some(response_tx)));
        send_ui_event(
            &self.tx,
            UiEvent::CredentialRequested {
                tab_id: Some(self.tab_id),
                request: request.clone(),
                response_tx: response_tx.clone(),
            },
        );
        let credential = response_rx
            .recv()
            .map_err(|_| khaslana::GitError::Credential("凭据输入已取消".into()))??;

        if let Some(credential) = credential {
            if credential.should_save() {
                match self.store.save_record(&request, &credential) {
                    Ok(record) => {
                        if let Ok(mut last) = self.last_stored_attempt.lock() {
                            *last = Some((request.url.clone(), record.id.clone()));
                        }
                        set_remote_binding_for_request(
                            &self.remote_bindings,
                            &request,
                            RemoteCredentialPolicy::Record(record.id),
                        );
                        if let Ok(bindings) = self.remote_bindings.lock() {
                            save_remote_credential_bindings_to_disk(&bindings);
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

        Ok(None)
    }
}

fn remote_binding_key(repo_path: &Path, remote_name: &str) -> (String, String) {
    (normalize_repo_path(repo_path), remote_name.to_string())
}

fn remote_binding_for_request(
    bindings: &Arc<Mutex<RemoteCredentialBindings>>,
    request: &CredentialRequest,
) -> RemoteCredentialPolicy {
    let (Some(repo_path), Some(remote_name)) = (&request.repo_path, request.remote_name.as_ref())
    else {
        return RemoteCredentialPolicy::AutoMatch;
    };
    let (repo_key, remote_key) = remote_binding_key(repo_path, remote_name);
    bindings
        .lock()
        .ok()
        .and_then(|bindings| {
            bindings
                .remotes
                .iter()
                .find(|binding| {
                    binding.repo_path == repo_key
                        && binding.remote_name == remote_key
                        && binding.remote_url == request.url
                })
                .map(|binding| binding.policy.clone())
        })
        .unwrap_or(RemoteCredentialPolicy::AutoMatch)
}

fn set_remote_binding_for_request(
    bindings: &Arc<Mutex<RemoteCredentialBindings>>,
    request: &CredentialRequest,
    policy: RemoteCredentialPolicy,
) {
    let (Some(repo_path), Some(remote_name)) = (&request.repo_path, request.remote_name.as_ref())
    else {
        return;
    };
    let (repo_key, remote_key) = remote_binding_key(repo_path, remote_name);
    let Ok(mut bindings) = bindings.lock() else {
        return;
    };
    if let Some(binding) = bindings
        .remotes
        .iter_mut()
        .find(|binding| binding.repo_path == repo_key && binding.remote_name == remote_key)
    {
        binding.remote_url = request.url.clone();
        binding.policy = policy;
    } else {
        bindings.remotes.push(RemoteCredentialBinding {
            repo_path: repo_key,
            remote_name: remote_key,
            remote_url: request.url.clone(),
            policy,
        });
    }
}

fn save_remote_credential_bindings_to_disk(bindings: &RemoteCredentialBindings) {
    let Some(path) = RepositoryView::remote_credential_bindings_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if let Err(err) = fs::create_dir_all(parent) {
        tracing::warn!("remote credential bindings directory create skipped: {err}");
        return;
    }
    match serde_json::to_string_pretty(bindings) {
        Ok(content) => {
            if let Err(err) = fs::write(path, content) {
                tracing::warn!("remote credential bindings write skipped: {err}");
            }
        }
        Err(err) => tracing::warn!("remote credential bindings encode skipped: {err}"),
    }
}

fn send_credential_response(
    pending: &PendingCredential,
    response: khaslana::Result<Option<GitCredential>>,
) -> bool {
    let Ok(mut response_tx) = pending.response_tx.lock() else {
        return false;
    };
    let Some(response_tx) = response_tx.take() else {
        return false;
    };
    response_tx.send(response).is_ok()
}

fn credential_form_mode_for_request(request: &CredentialRequest) -> CredentialFormMode {
    let lower = request.url.to_ascii_lowercase();
    if lower.starts_with("ssh://")
        || lower.starts_with("git@")
        || (!lower.starts_with("http://")
            && !lower.starts_with("https://")
            && request
                .allowed_types
                .contains(git2::CredentialType::SSH_KEY))
    {
        CredentialFormMode::Ssh
    } else {
        CredentialFormMode::Https
    }
}

fn send_ui_event(tx: &Sender<UiEvent>, event: UiEvent) {
    let _ = tx.try_send(event);
}

fn perf_log(stage: &'static str, started: Instant, details: impl AsRef<str>) {
    if std::env::var_os("KHASLANA_PERF_LOG").is_some() {
        tracing::info!(
            target: "khaslana::perf",
            stage,
            elapsed_ms = started.elapsed().as_millis(),
            "{}",
            details.as_ref()
        );
    }
}

pub(crate) struct RepositoryView {
    tx: Sender<UiEvent>,
    rx: Receiver<UiEvent>,
    credential_store: Arc<KeyringCredentialStore>,
    remote_credential_bindings: Arc<Mutex<RemoteCredentialBindings>>,
    credential_records: Vec<CredentialRecord>,
    diff_encoding_preferences: DiffEncodingPreferences,
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
    scroll_handles: RefCell<HashMap<String, ScrollHandle>>,
    uniform_scroll_handles: RefCell<HashMap<String, UniformListScrollHandle>>,
    pub(crate) scrollbar_drag: Option<ScrollbarDragState>,
    pending_credential: Option<PendingCredential>,
    pending_credentials: VecDeque<PendingCredential>,
    repository_load_queue: VecDeque<RepositoryLoadRequest>,
    active_repository_loads: usize,
    pub(crate) active_dialog: Option<DialogState>,
    pub(crate) branch_context_menu: Option<BranchContextMenu>,
    change_context_menu: Option<ChangeContextMenu>,
    pub(crate) tag_context_menu: Option<TagContextMenu>,
    pub(crate) stash_context_menu: Option<StashContextMenu>,
    pub(crate) commit_context_menu: Option<CommitContextMenu>,
    pub(crate) encoding_menu_target: Option<EncodingMenuTarget>,
    encoding_menu_closed_by_capture: Option<EncodingMenuTarget>,
    save_credential: bool,
    credential_scope: CredentialScope,
    credential_form_mode: CredentialFormMode,
    credential_use_ssh_agent: bool,
    clone_url: TextFieldState,
    clone_path: TextFieldState,
    branch_name: TextFieldState,
    branch_rename: TextFieldState,
    commit_message: TextFieldState,
    credential_username: TextFieldState,
    credential_secret: TextFieldState,
    credential_key_path: TextFieldState,
    credential_passphrase: TextFieldState,
    credential_remote_url: TextFieldState,
    remote_name: TextFieldState,
    remote_url: TextFieldState,
    remote_credential_policy: RemoteCredentialPolicy,
}

impl RepositoryView {
    fn new(cx: &mut Context<Self>) -> Self {
        let (tx, rx) = async_channel::unbounded();
        let credential_store = Arc::new(KeyringCredentialStore::new());
        let remote_credential_bindings =
            Arc::new(Mutex::new(Self::load_remote_credential_bindings()));
        Self::spawn_event_pump(rx.clone(), cx);

        Self {
            tx,
            rx,
            credential_store,
            remote_credential_bindings,
            credential_records: Vec::new(),
            diff_encoding_preferences: Self::load_diff_encoding_preferences(),
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
            scroll_handles: RefCell::new(HashMap::new()),
            uniform_scroll_handles: RefCell::new(HashMap::new()),
            scrollbar_drag: None,
            pending_credential: None,
            pending_credentials: VecDeque::new(),
            repository_load_queue: VecDeque::new(),
            active_repository_loads: 0,
            active_dialog: None,
            branch_context_menu: None,
            change_context_menu: None,
            tag_context_menu: None,
            stash_context_menu: None,
            commit_context_menu: None,
            encoding_menu_target: None,
            encoding_menu_closed_by_capture: None,
            save_credential: false,
            credential_scope: CredentialScope::RemoteUrl,
            credential_form_mode: CredentialFormMode::Https,
            credential_use_ssh_agent: false,
            clone_url: TextFieldState::new(cx, "远程仓库 URL"),
            clone_path: TextFieldState::new(cx, "克隆到父文件夹"),
            branch_name: TextFieldState::new(cx, "新分支名称"),
            branch_rename: TextFieldState::new(cx, "重命名为"),
            commit_message: TextFieldState::new(cx, "提交信息"),
            credential_username: TextFieldState::new(cx, "用户名"),
            credential_secret: TextFieldState::new(cx, "密码或 PAT").secret(),
            credential_key_path: TextFieldState::new(cx, "SSH 私钥路径"),
            credential_passphrase: TextFieldState::new(cx, "SSH 密码短语").secret(),
            credential_remote_url: TextFieldState::new(cx, "适用远端 URL"),
            remote_name: TextFieldState::new(cx, "远端名称"),
            remote_url: TextFieldState::new(cx, "远端地址"),
            remote_credential_policy: RemoteCredentialPolicy::AutoMatch,
        }
    }

    fn new_with_session(cx: &mut Context<Self>) -> Self {
        let mut view = Self::new(cx);
        view.restore_session();
        view
    }

    pub(crate) fn scroll_handle(&self, id: &'static str) -> ScrollHandle {
        let scoped_id = self
            .active_tab
            .map(|tab_id| format!("tab-{}:{id}", tab_id.0))
            .unwrap_or_else(|| format!("global:{id}"));
        self.scroll_handles
            .borrow_mut()
            .entry(scoped_id)
            .or_default()
            .clone()
    }

    fn scroll_local_branch_to_current(&self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let Some(index) = snapshot
            .branches
            .iter()
            .filter(|branch| branch.kind == BranchKind::Local)
            .position(|branch| branch.is_head)
        else {
            return;
        };

        let handle = self.scroll_handle("local-branch-list");
        let bounds = handle.bounds();
        let max_offset = f32::from(handle.max_offset().height).max(0.0);
        if max_offset <= 1.0 || f32::from(bounds.size.height) <= 1.0 {
            handle.scroll_to_top_of_item(index);
            return;
        }

        let item_top = handle
            .bounds_for_item(index)
            .map(|item_bounds| f32::from(item_bounds.top() - bounds.top()))
            .unwrap_or_else(|| {
                let list_padding = 8.0;
                let row_gap = 4.0;
                list_padding + index as f32 * (NAV_ROW_HEIGHT + row_gap)
            });
        let row_height = handle
            .bounds_for_item(index)
            .map(|item_bounds| f32::from(item_bounds.size.height))
            .unwrap_or(NAV_ROW_HEIGHT);
        let viewport_height = f32::from(bounds.size.height);
        let target_scroll =
            (item_top - viewport_height / 2.0 + row_height / 2.0).clamp(0.0, max_offset);
        handle.set_offset(point(px(0.0), px(-target_scroll)));
    }

    pub(crate) fn uniform_scroll_handle(&self, id: &'static str) -> UniformListScrollHandle {
        let scoped_id = self
            .active_tab
            .map(|tab_id| format!("tab-{}:{id}", tab_id.0))
            .unwrap_or_else(|| format!("global:{id}"));
        self.uniform_scroll_handles
            .borrow_mut()
            .entry(scoped_id)
            .or_insert_with(UniformListScrollHandle::new)
            .clone()
    }

    fn reset_uniform_scroll(&self, id: &'static str) {
        let handle = self.uniform_scroll_handle(id);
        handle
            .0
            .borrow_mut()
            .base_handle
            .set_offset(point(px(0.0), px(0.0)));
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
        if let Some(active) = self.active_tab
            && let Some(tab) = self.tab_mut(active)
        {
            tab.release_large_diff_caches();
        }
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
        let mut retained = VecDeque::new();
        while let Some(pending) = self.pending_credentials.pop_front() {
            if pending.tab_id == Some(tab_id) {
                send_credential_response(&pending, Ok(None));
            } else {
                retained.push_back(pending);
            }
        }
        self.pending_credentials = retained;
        self.repository_load_queue
            .retain(|request| request.tab_id != tab_id);
        if self
            .pending_credential
            .as_ref()
            .and_then(|pending| pending.tab_id)
            == Some(tab_id)
        {
            if let Some(pending) = self.pending_credential.as_ref() {
                send_credential_response(pending, Ok(None));
            }
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

    fn diff_encoding_preferences_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "Khaslana")
            .map(|dirs| dirs.config_dir().join("diff-encodings.json"))
    }

    fn remote_credential_bindings_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "Khaslana")
            .map(|dirs| dirs.config_dir().join("remote-credentials.json"))
    }

    fn load_session_state() -> Option<SessionState> {
        let path = Self::session_path()?;
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn load_diff_encoding_preferences() -> DiffEncodingPreferences {
        let Some(path) = Self::diff_encoding_preferences_path() else {
            return DiffEncodingPreferences::default();
        };
        let Ok(content) = fs::read_to_string(path) else {
            return DiffEncodingPreferences::default();
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    fn load_remote_credential_bindings() -> RemoteCredentialBindings {
        let Some(path) = Self::remote_credential_bindings_path() else {
            return RemoteCredentialBindings::default();
        };
        let Ok(content) = fs::read_to_string(path) else {
            return RemoteCredentialBindings::default();
        };
        serde_json::from_str(&content).unwrap_or_default()
    }

    fn save_diff_encoding_preferences(&self) {
        let Some(path) = Self::diff_encoding_preferences_path() else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if let Err(err) = fs::create_dir_all(parent) {
            tracing::warn!("diff encoding preferences directory create skipped: {err}");
            return;
        }
        match serde_json::to_string_pretty(&self.diff_encoding_preferences) {
            Ok(content) => {
                if let Err(err) = fs::write(path, content) {
                    tracing::warn!("diff encoding preferences write skipped: {err}");
                }
            }
            Err(err) => tracing::warn!("diff encoding preferences encode skipped: {err}"),
        }
    }

    fn save_remote_credential_bindings(&self) {
        let Ok(bindings) = self.remote_credential_bindings.lock() else {
            tracing::warn!("remote credential bindings state read skipped");
            return;
        };
        save_remote_credential_bindings_to_disk(&bindings);
    }

    fn diff_encoding_choice_for_path(&self, path: &Path) -> DiffEncodingChoice {
        self.diff_encoding_preferences
            .repositories
            .get(&normalize_repo_path(path))
            .copied()
            .unwrap_or_default()
    }

    fn current_diff_encoding_choice(&self) -> DiffEncodingChoice {
        self.repo_path
            .as_ref()
            .map(|path| self.diff_encoding_choice_for_path(path))
            .unwrap_or_default()
    }

    fn set_current_diff_encoding(&mut self, encoding: DiffEncodingChoice) {
        let Some(repo_path) = self.repo_path.clone() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let key = normalize_repo_path(&repo_path);
        if encoding == DiffEncodingChoice::Auto {
            self.diff_encoding_preferences.repositories.remove(&key);
        } else {
            self.diff_encoding_preferences
                .repositories
                .insert(key, encoding);
        }
        self.save_diff_encoding_preferences();
        self.status = format!("差异编码已切换为 {}", encoding.label());
        self.reload_visible_diffs_after_encoding_change();
    }

    fn reload_visible_diffs_after_encoding_change(&mut self) {
        if let Some(diff) = self.diff.clone() {
            self.load_diff(diff.path.clone(), diff.scope.clone());
        }
        if self.main_mode == MainMode::History
            && let Some(path) = self.history_selected_file.clone()
        {
            self.select_history_file_with_reload(path, true);
        }
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
                self.queue_repository_load(
                    tab_id,
                    path,
                    "正在恢复仓库",
                    "仓库已恢复",
                    LoadPriority::Background,
                );
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
                self.remote_credential_bindings.clone(),
                self.tx.clone(),
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
                self.prepare_current_credential_prompt();
                break;
            }
        }
    }

    fn prepare_current_credential_prompt(&mut self) {
        let Some(pending) = self.pending_credential.as_ref() else {
            return;
        };
        let request = pending.request.clone();
        self.save_credential = true;
        self.credential_scope = CredentialScope::RemoteUrl;
        self.credential_form_mode = credential_form_mode_for_request(&request);
        self.credential_use_ssh_agent = false;
        self.credential_username.set_value(
            request
                .username_from_url
                .clone()
                .unwrap_or_else(|| "git".to_string()),
        );
        self.credential_secret.clear();
        self.credential_key_path.clear();
        self.credential_passphrase.clear();
        self.credential_remote_url.set_value(request.url);
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
                        this.change_indexes = ChangeListIndexes::rebuild(&snapshot.changes);
                        this.snapshot = Some(snapshot);
                        this.scroll_local_branch_to_current();
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
                        this.scroll_local_branch_to_current();
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
            UiEvent::RepositoryLoadFinished { tab_id, load_id } => {
                self.active_repository_loads = self.active_repository_loads.saturating_sub(1);
                if self
                    .tab(tab_id)
                    .is_some_and(|tab| tab.repository_load_id == load_id)
                {
                    self.apply_status_event(Some(tab_id), |this| {
                        this.busy = false;
                    });
                }
                self.start_queued_repository_loads();
            }
            UiEvent::OperationFinished {
                tab_id,
                message,
                snapshot,
                diff,
            } => {
                let mut full_status_request = None;
                self.apply_status_event(tab_id, |this| {
                    this.busy = false;
                    this.loading = RepositoryLoading::default();
                    this.status = message;
                    if let Some(snapshot) = snapshot {
                        this.repo_path = (!snapshot.path.as_os_str().is_empty())
                            .then(|| snapshot.path.clone())
                            .or_else(|| this.repo_path.clone());
                        this.sync_selected_remote(&snapshot);
                        this.change_indexes = ChangeListIndexes::rebuild(&snapshot.changes);
                        this.snapshot = Some(snapshot);
                        this.prune_change_selection();
                        this.clear_history();
                        this.scroll_local_branch_to_current();
                        this.reload_history_if_active();
                        if let Some(tab_id) = tab_id {
                            full_status_request = this
                                .repo_path
                                .clone()
                                .map(|path| (tab_id, path, this.repository_load_id));
                            this.loading.status_full = true;
                        }
                    }
                    if let Some(diff) = diff {
                        this.diff = Some(Arc::new(diff));
                        this.diff_headers_expanded = false;
                        this.reset_uniform_scroll("diff-scroll");
                    }
                });
                if let Some((tab_id, path, load_id)) = full_status_request {
                    self.load_full_status_for_tab(tab_id, path, load_id, "变更已补全".to_string());
                }
            }
            UiEvent::DiscardChangeFinished {
                tab_id,
                message,
                snapshot,
                changes,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id {
                        this.busy = false;
                        this.loading = RepositoryLoading::default();
                        this.status = message;
                        this.last_error = None;
                        this.repo_path = Some(snapshot.path.clone());
                        this.sync_selected_remote(&snapshot);
                        this.change_indexes = ChangeListIndexes::rebuild(&snapshot.changes);
                        this.snapshot = Some(snapshot);
                        this.replace_changes(changes);
                        this.diff = None;
                        this.diff_headers_expanded = false;
                        this.reset_uniform_scroll("diff-scroll");
                        this.clear_history();
                        this.scroll_local_branch_to_current();
                        this.reload_history_if_active();
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
                scope,
                load_id,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if load_id == this.repository_load_id && scope == this.history_scope {
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
                        this.history_graph_rows =
                            history_view::commit_graph_rows(&this.history_commits);

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
                        this.history_diff = Some(Arc::new(diff));
                        this.history_diff_headers_expanded = false;
                        this.reset_uniform_scroll("history-diff-scroll");
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
            UiEvent::CredentialRequested {
                tab_id,
                request,
                response_tx,
            } => {
                if tab_id.is_some_and(|tab_id| self.tab(tab_id).is_none()) {
                    if let Ok(mut response_tx) = response_tx.lock()
                        && let Some(response_tx) = response_tx.take()
                    {
                        let _ = response_tx.send(Ok(None));
                    }
                    return;
                }
                self.apply_status_event(tab_id, |this| {
                    this.status = "需要凭据".to_string();
                });
                self.enqueue_credential_request(PendingCredential {
                    tab_id,
                    request,
                    response_tx,
                });
                self.prepare_current_credential_prompt();
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
        self.change_indexes = ChangeListIndexes::rebuild(&merged.changes);
        self.snapshot = Some(merged);
    }

    fn replace_changes(&mut self, changes: Vec<khaslana::WorktreeChange>) {
        if let Some(snapshot) = self.snapshot.as_mut() {
            snapshot.changes = changes;
            self.change_indexes = ChangeListIndexes::rebuild(&snapshot.changes);
        } else {
            self.change_indexes = ChangeListIndexes::default();
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
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        match key {
            "left" | "arrowleft" => {
                self.field_mut(field).move_left(shift);
                cx.notify();
            }
            "right" | "arrowright" => {
                self.field_mut(field).move_right(shift);
                cx.notify();
            }
            "home" => {
                self.field_mut(field).move_caret_to(0, shift);
                cx.notify();
            }
            "end" => {
                let end = self.field(field).value.len();
                self.field_mut(field).move_caret_to(end, shift);
                cx.notify();
            }
            "backspace" => {
                self.field_mut(field).delete_backward();
                cx.notify();
            }
            "delete" => {
                self.field_mut(field).delete_forward();
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
                } else if matches!(field, FieldId::RemoteName | FieldId::RemoteUrl) {
                    if let Some(DialogState::RemoteForm { editing }) = self.active_dialog.clone() {
                        self.save_remote(editing);
                    }
                } else if matches!(
                    field,
                    FieldId::CredentialSecret
                        | FieldId::CredentialPassphrase
                        | FieldId::CredentialUsername
                        | FieldId::CredentialKeyPath
                        | FieldId::CredentialRemoteUrl
                ) {
                    if matches!(self.active_dialog, Some(DialogState::CredentialForm { .. })) {
                        self.save_credential_form();
                    } else {
                        self.use_credentials();
                    }
                }
            }
            _ => {
                if control {
                    if key.eq_ignore_ascii_case("a") {
                        self.field_mut(field).select_all();
                        cx.notify();
                    } else if key.eq_ignore_ascii_case("c") {
                        if let Some(text) = self.field(field).copyable_selected_text() {
                            cx.write_to_clipboard(ClipboardItem::new_string(text));
                        }
                    } else if key.eq_ignore_ascii_case("x") {
                        if let Some(text) = self.field(field).copyable_selected_text() {
                            cx.write_to_clipboard(ClipboardItem::new_string(text));
                            self.field_mut(field).delete_selection();
                            cx.notify();
                        }
                    } else if key.eq_ignore_ascii_case("v")
                        && let Some(text) = cx.read_from_clipboard().and_then(|item| item.text())
                    {
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
        let multiline = field == FieldId::CommitMessage;
        self.field_mut(field).insert_text(text, multiline);
    }

    fn focused_field(&self, window: &Window, _cx: &App) -> Option<FieldId> {
        [
            (FieldId::CloneUrl, &self.clone_url),
            (FieldId::ClonePath, &self.clone_path),
            (FieldId::BranchName, &self.branch_name),
            (FieldId::BranchRename, &self.branch_rename),
            (FieldId::RemoteName, &self.remote_name),
            (FieldId::RemoteUrl, &self.remote_url),
            (FieldId::CommitMessage, &self.commit_message),
            (FieldId::CredentialUsername, &self.credential_username),
            (FieldId::CredentialSecret, &self.credential_secret),
            (FieldId::CredentialKeyPath, &self.credential_key_path),
            (FieldId::CredentialPassphrase, &self.credential_passphrase),
            (FieldId::CredentialRemoteUrl, &self.credential_remote_url),
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
            FieldId::RemoteName => &self.remote_name,
            FieldId::RemoteUrl => &self.remote_url,
            FieldId::CommitMessage => &self.commit_message,
            FieldId::CredentialUsername => &self.credential_username,
            FieldId::CredentialSecret => &self.credential_secret,
            FieldId::CredentialKeyPath => &self.credential_key_path,
            FieldId::CredentialPassphrase => &self.credential_passphrase,
            FieldId::CredentialRemoteUrl => &self.credential_remote_url,
        }
    }

    fn field_mut(&mut self, id: FieldId) -> &mut TextFieldState {
        match id {
            FieldId::CloneUrl => &mut self.clone_url,
            FieldId::ClonePath => &mut self.clone_path,
            FieldId::BranchName => &mut self.branch_name,
            FieldId::BranchRename => &mut self.branch_rename,
            FieldId::RemoteName => &mut self.remote_name,
            FieldId::RemoteUrl => &mut self.remote_url,
            FieldId::CommitMessage => &mut self.commit_message,
            FieldId::CredentialUsername => &mut self.credential_username,
            FieldId::CredentialSecret => &mut self.credential_secret,
            FieldId::CredentialKeyPath => &mut self.credential_key_path,
            FieldId::CredentialPassphrase => &mut self.credential_passphrase,
            FieldId::CredentialRemoteUrl => &mut self.credential_remote_url,
        }
    }

    fn browse_open(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.open_repo(path);
        }
    }

    fn browse_clone_target(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.clone_path.set_value(path.display().to_string());
        }
    }

    fn open_clone_dialog(&mut self, window: &mut Window) {
        self.close_popups();
        self.clone_url.clear();
        self.clone_path.clear();
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
        self.branch_name.clear();
        self.active_dialog = Some(DialogState::CreateBranch);
        self.last_error = None;
    }

    pub(crate) fn open_rename_branch_dialog(&mut self, branch: String) {
        self.close_popups();
        self.branch_rename.set_value(branch.clone());
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
        self.encoding_menu_target = None;
        self.encoding_menu_closed_by_capture = None;
    }

    fn close_dialog(&mut self) {
        self.active_dialog = None;
    }

    fn open_credential_manager(&mut self) {
        self.close_popups();
        self.active_dialog = Some(DialogState::CredentialManager);
        self.reload_credential_records("凭据列表已加载");
    }

    fn open_credential_form(&mut self) {
        self.credential_form_mode = CredentialFormMode::Https;
        self.credential_scope = CredentialScope::RemoteUrl;
        self.credential_use_ssh_agent = false;
        self.credential_remote_url.clear();
        self.credential_username.clear();
        self.credential_secret.clear();
        self.credential_key_path.clear();
        self.credential_passphrase.clear();
        self.active_dialog = Some(DialogState::CredentialForm { editing: None });
        self.last_error = None;
    }

    fn save_credential_form(&mut self) {
        let url = self.credential_remote_url.value.trim().to_string();
        if url.is_empty() {
            self.last_error = Some("需要填写适用远端 URL".into());
            return;
        }
        let inferred_mode = credential_form_mode_for_request(&CredentialRequest {
            url: url.clone(),
            username_from_url: None,
            allowed_types: git2::CredentialType::USER_PASS_PLAINTEXT
                | git2::CredentialType::SSH_KEY,
            repo_path: None,
            remote_name: None,
        });
        if inferred_mode != self.credential_form_mode {
            self.last_error = Some("凭据类型与远端 URL 协议不匹配".into());
            return;
        }
        let username = self
            .credential_username
            .value
            .trim()
            .to_string()
            .if_empty_then(|| "git".into());
        let credential = match self.credential_form_mode {
            CredentialFormMode::Https => {
                if self.credential_secret.value.is_empty() {
                    self.last_error = Some("需要填写密码或 PAT".into());
                    return;
                }
                GitCredential::UserPass {
                    username,
                    secret: self.credential_secret.value.clone(),
                    save_to_keyring: true,
                    scope: self.credential_scope,
                }
            }
            CredentialFormMode::Ssh => {
                let key_path = self.credential_key_path.value.trim().to_string();
                if !self.credential_use_ssh_agent && key_path.is_empty() {
                    self.last_error = Some("需要填写 SSH 私钥路径或选择使用 SSH agent".into());
                    return;
                }
                GitCredential::SshPassphrase {
                    username,
                    private_key_path: (!self.credential_use_ssh_agent).then_some(key_path),
                    passphrase: (!self.credential_passphrase.value.is_empty())
                        .then(|| self.credential_passphrase.value.clone()),
                    save_to_keyring: true,
                    scope: self.credential_scope,
                }
            }
        };
        let request = CredentialRequest {
            url,
            username_from_url: Some(credential.username().to_string()),
            allowed_types: match self.credential_form_mode {
                CredentialFormMode::Https => git2::CredentialType::USER_PASS_PLAINTEXT,
                CredentialFormMode::Ssh => git2::CredentialType::SSH_KEY,
            },
            repo_path: None,
            remote_name: None,
        };
        match self.credential_store.save_record(&request, &credential) {
            Ok(_) => {
                self.active_dialog = Some(DialogState::CredentialManager);
                self.reload_credential_records("凭据已添加");
            }
            Err(err) => {
                self.last_error = Some(err.to_string());
            }
        }
    }

    pub(crate) fn open_remote_manager(&mut self) {
        if self.repo_path.is_none() {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        }
        self.close_popups();
        self.active_dialog = Some(DialogState::RemoteManager);
        self.reload_credential_records("远端管理已打开");
    }

    fn open_remote_form(&mut self, editing: Option<String>) {
        let remote = match editing.as_ref() {
            Some(name) => {
                let Some(snapshot) = self.snapshot.as_ref() else {
                    self.last_error = Some("请先打开一个仓库".into());
                    return;
                };
                let Some(remote) = snapshot.remotes.iter().find(|remote| remote.name == *name)
                else {
                    self.last_error = Some("远端不存在".into());
                    return;
                };
                Some(remote.clone())
            }
            None => None,
        };

        self.remote_credential_policy = RemoteCredentialPolicy::AutoMatch;
        if let Some(remote) = remote {
            self.remote_name.set_value(remote.name.clone());
            self.remote_url.set_value(remote.url.clone());
            if let Some(repo_path) = self.repo_path.as_ref() {
                self.remote_credential_policy =
                    self.remote_credential_policy_for_remote(repo_path, &remote.name, &remote.url);
            }
        } else {
            self.remote_name.clear();
            self.remote_url.clear();
        }
        self.active_dialog = Some(DialogState::RemoteForm { editing });
        self.last_error = None;
    }

    fn open_delete_remote_confirm(&mut self, name: String) {
        self.active_dialog = Some(DialogState::ConfirmDeleteRemote { name });
        self.last_error = None;
    }

    fn save_remote(&mut self, editing: Option<String>) {
        let name = self.remote_name.value.trim().to_string();
        let url = self.remote_url.value.trim().to_string();
        if name.is_empty() {
            self.last_error = Some("需要填写远端名称".into());
            return;
        }
        if url.is_empty() {
            self.last_error = Some("需要填写远端地址".into());
            return;
        }
        if name.contains(char::is_whitespace) || name.contains('\\') || name.starts_with('-') {
            self.last_error = Some(format!("远端名称无效：{name}"));
            return;
        }

        let existing_remotes = self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.remotes.clone())
            .unwrap_or_default();
        if editing.as_deref() != Some(name.as_str())
            && existing_remotes.iter().any(|remote| remote.name == name)
        {
            self.last_error = Some(format!("远端名称已存在：{name}"));
            return;
        }

        let selected_record =
            if let RemoteCredentialPolicy::Record(id) = &self.remote_credential_policy {
                let Some(record) = self
                    .credential_records
                    .iter()
                    .find(|record| record.id == *id)
                    .cloned()
                else {
                    self.last_error = Some("所选凭据记录不存在".into());
                    return;
                };
                Some(record)
            } else {
                None
            };
        if let Some(record) = selected_record.as_ref() {
            let compatible = match record.scope {
                CredentialScope::RemoteUrl => {
                    credential_record_is_compatible_with_url(record, &url)
                }
                CredentialScope::Host => credential_record_matches_remote_url(record, &url),
            };
            if !compatible {
                self.last_error = Some("所选凭据与远端地址协议或站点不匹配".into());
                return;
            }
            if record.scope == CredentialScope::RemoteUrl
                && let Err(err) = self
                    .credential_store
                    .update_record_remote_url(&record.id, &url)
            {
                self.last_error = Some(err.to_string());
                return;
            }
            if record.scope == CredentialScope::Host
                && let Err(err) = self.credential_store.touch_record(&record.id)
            {
                self.last_error = Some(err.to_string());
                return;
            }
            self.reload_credential_records("远端凭据绑定已更新");
        }

        if let Some(repo_path) = self.repo_path.as_ref() {
            let request = CredentialRequest {
                url: url.clone(),
                username_from_url: None,
                allowed_types: git2::CredentialType::USER_PASS_PLAINTEXT
                    | git2::CredentialType::SSH_KEY,
                repo_path: Some(repo_path.clone()),
                remote_name: Some(name.clone()),
            };
            set_remote_binding_for_request(
                &self.remote_credential_bindings,
                &request,
                self.remote_credential_policy.clone(),
            );
            self.save_remote_credential_bindings();
        }

        let old_selected = self.selected_remote.clone();
        if let Some(old_name) = editing.as_ref() {
            if old_selected.as_deref() == Some(old_name.as_str()) {
                self.selected_remote = Some(name.clone());
            }
        }
        let new_name = name.clone();
        match editing {
            Some(old_name) => {
                self.with_repo("远端已更新", move |service, repo| {
                    service.update_remote(
                        repo,
                        &RemoteName::new(old_name),
                        &RemoteName::new(new_name),
                        &url,
                    )
                });
            }
            None => {
                self.selected_remote = Some(name.clone());
                self.with_repo("远端已新增", move |service, repo| {
                    service.add_remote(repo, &RemoteName::new(name), &url)
                });
            }
        }
    }

    fn delete_remote(&mut self, name: String) {
        if self.selected_remote.as_deref() == Some(name.as_str()) {
            self.selected_remote = None;
        }
        self.with_repo("远端已删除", move |service, repo| {
            service.delete_remote(repo, &RemoteName::new(name))
        });
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

    fn matching_credential_for_remote_url(&self, url: &str) -> Option<&CredentialRecord> {
        self.credential_records
            .iter()
            .filter(|record| credential_record_matches_remote_url(record, url))
            .max_by(|a, b| {
                let a_scope = match a.scope {
                    CredentialScope::RemoteUrl => 1,
                    CredentialScope::Host => 0,
                };
                let b_scope = match b.scope {
                    CredentialScope::RemoteUrl => 1,
                    CredentialScope::Host => 0,
                };
                a_scope
                    .cmp(&b_scope)
                    .then_with(|| a.last_used.unwrap_or(0).cmp(&b.last_used.unwrap_or(0)))
                    .then_with(|| a.updated_at.cmp(&b.updated_at))
            })
    }

    fn remote_credential_policy_for_remote(
        &self,
        repo_path: &Path,
        remote_name: &str,
        remote_url: &str,
    ) -> RemoteCredentialPolicy {
        let (repo_key, remote_key) = remote_binding_key(repo_path, remote_name);
        self.remote_credential_bindings
            .lock()
            .ok()
            .and_then(|bindings| {
                bindings
                    .remotes
                    .iter()
                    .find(|binding| {
                        binding.repo_path == repo_key
                            && binding.remote_name == remote_key
                            && binding.remote_url == remote_url
                    })
                    .map(|binding| binding.policy.clone())
            })
            .unwrap_or(RemoteCredentialPolicy::AutoMatch)
    }

    fn open_repo(&mut self, path: PathBuf) {
        let tab_id = self.ensure_tab_for_path(path.clone());
        self.queue_repository_load(
            tab_id,
            path,
            "正在打开仓库",
            "仓库已打开",
            LoadPriority::User,
        );
    }

    fn clone_repo(&mut self) {
        let url = self.clone_url.value.trim().to_string();
        let path_text = self.clone_path.value.trim().to_string();
        if url.is_empty() || path_text.is_empty() {
            self.last_error = Some("需要填写远程仓库 URL 和克隆到父文件夹".into());
            return;
        }
        if infer_clone_directory_name(&url).is_none() {
            self.last_error = Some("无法从远程仓库 URL 推导仓库文件夹名".into());
            return;
        };
        let Some(path) = infer_clone_target_path(&url, &path_text) else {
            self.last_error = Some("需要填写远程仓库 URL 和克隆到父文件夹".into());
            return;
        };
        if path.exists() {
            self.last_error = Some("目标仓库文件夹已存在".into());
            return;
        }
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
                .clone_repo(&url, &RepoPath::new(path))
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
        self.queue_repository_load(tab_id, path, "正在刷新仓库", "已刷新", LoadPriority::User);
    }

    fn queue_repository_load(
        &mut self,
        tab_id: RepoTabId,
        path: PathBuf,
        started: &'static str,
        finished: &'static str,
        priority: LoadPriority,
    ) {
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
        self.repository_load_queue
            .retain(|request| request.tab_id != tab_id);
        let request = RepositoryLoadRequest {
            tab_id,
            path,
            started,
            finished,
        };
        if priority == LoadPriority::User {
            self.repository_load_queue.push_front(request);
        } else {
            self.repository_load_queue.push_back(request);
        }
        self.start_queued_repository_loads();
        if let Some(tab) = self.tab(tab_id)
            && tab.repository_load_id == load_id
            && self
                .repository_load_queue
                .iter()
                .any(|request| request.tab_id == tab_id)
        {
            self.apply_status_event(Some(tab_id), |this| {
                this.status = "等待加载仓库".to_string();
            });
        }
    }

    fn start_queued_repository_loads(&mut self) {
        while self.active_repository_loads < MAX_CONCURRENT_REPO_LOADS {
            let Some(request) = self.repository_load_queue.pop_front() else {
                break;
            };
            if self.tab(request.tab_id).is_none() {
                continue;
            }
            self.active_repository_loads += 1;
            self.spawn_repository_load(request);
        }
    }

    fn spawn_repository_load(&mut self, request: RepositoryLoadRequest) {
        let tab_id = request.tab_id;
        let path = request.path;
        let started = request.started;
        let finished = request.finished;
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        let load_id = self
            .tab(tab_id)
            .map(|tab| tab.repository_load_id)
            .unwrap_or_default();
        let load_started = Instant::now();
        send_ui_event(
            &tx,
            UiEvent::OperationStarted {
                tab_id: Some(tab_id),
                message: started.to_string(),
            },
        );
        thread::spawn(move || {
            let stage_started = Instant::now();
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
                    send_ui_event(&tx, UiEvent::RepositoryLoadFinished { tab_id, load_id });
                    return;
                }
            };
            perf_log(
                "repo.open_fast",
                stage_started,
                format!("tab={} branches={}", tab_id.0, fast.branches.len()),
            );
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
                    send_ui_event(&tx, UiEvent::RepositoryLoadFinished { tab_id, load_id });
                    return;
                }
            };

            let stage_started = Instant::now();
            match service.snapshot_metadata(&mut repo) {
                Ok(snapshot) => {
                    perf_log(
                        "repo.metadata",
                        stage_started,
                        format!(
                            "tab={} branches={} remotes={} tags={} stashes={} conflicts={}",
                            tab_id.0,
                            snapshot.branches.len(),
                            snapshot.remotes.len(),
                            snapshot.tags.len(),
                            snapshot.stashes.len(),
                            snapshot.conflicts.len()
                        ),
                    );
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
                    send_ui_event(&tx, UiEvent::RepositoryLoadFinished { tab_id, load_id });
                    return;
                }
            }

            let stage_started = Instant::now();
            match service.status_fast(&repo) {
                Ok(changes) => {
                    perf_log(
                        "repo.status_fast",
                        stage_started,
                        format!("tab={} changes={}", tab_id.0, changes.len()),
                    );
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
                    send_ui_event(&tx, UiEvent::RepositoryLoadFinished { tab_id, load_id });
                    return;
                }
            }

            let stage_started = Instant::now();
            match service.status_full(&repo) {
                Ok(changes) => {
                    perf_log(
                        "repo.status_full",
                        stage_started,
                        format!("tab={} changes={}", tab_id.0, changes.len()),
                    );
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
            perf_log(
                "repo.load_total",
                load_started,
                format!("tab={} load_id={}", tab_id.0, load_id),
            );
            send_ui_event(&tx, UiEvent::RepositoryLoadFinished { tab_id, load_id });
        });
    }

    fn load_full_status_for_tab(
        &self,
        tab_id: RepoTabId,
        path: PathBuf,
        load_id: u64,
        message: String,
    ) {
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        thread::spawn(move || {
            let started = Instant::now();
            let result = (|| -> khaslana::Result<Vec<khaslana::WorktreeChange>> {
                let repo = Repository::open(path)?;
                service.status_full(&repo)
            })();
            match result {
                Ok(changes) => {
                    perf_log(
                        "repo.status_full.operation",
                        started,
                        format!("tab={} changes={}", tab_id.0, changes.len()),
                    );
                    send_ui_event(
                        &tx,
                        UiEvent::RepositoryStatusFullLoaded {
                            tab_id,
                            message,
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
            .filter(|remote| snapshot.remotes.iter().any(|info| info.name == **remote))
            .cloned()
            .or_else(|| {
                snapshot
                    .remotes
                    .iter()
                    .find(|remote| remote.name.as_str() == "origin")
                    .map(|remote| remote.name.clone())
            })
            .or_else(|| snapshot.remotes.first().map(|remote| remote.name.clone()))
    }

    fn sync_selected_remote(&mut self, snapshot: &RepositorySnapshot) {
        if snapshot.remotes.is_empty() {
            self.selected_remote = None;
            return;
        }

        if self
            .selected_remote
            .as_ref()
            .is_some_and(|remote| snapshot.remotes.iter().any(|info| info.name == *remote))
        {
            return;
        }

        self.selected_remote = snapshot
            .remotes
            .iter()
            .find(|remote| remote.name.as_str() == "origin")
            .map(|remote| remote.name.clone())
            .or_else(|| snapshot.remotes.first().map(|remote| remote.name.clone()));
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

    fn open_discard_change_confirm_dialog(
        &mut self,
        paths: Vec<String>,
        scope: DiffScope,
        target: DiscardTarget,
    ) {
        if paths.is_empty() {
            self.last_error = Some("没有可回滚的文件".into());
            self.change_context_menu = None;
            return;
        }
        self.close_popups();
        match target {
            DiscardTarget::Single => {
                if let Some(path) = paths.first() {
                    self.select_only_change(path.clone(), scope.clone(), false);
                }
            }
            DiscardTarget::Selected | DiscardTarget::All => {
                self.clear_opposite_change_selection(&scope);
            }
        }
        self.active_dialog = Some(DialogState::ConfirmDiscardChange {
            scope,
            target,
            paths,
        });
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

    fn discard_change(&mut self, paths: Vec<String>, scope: DiffScope, target: DiscardTarget) {
        let message = match scope {
            DiffScope::Staged => match target {
                DiscardTarget::Single => "已回滚文件全部更改",
                DiscardTarget::Selected => "已回滚选定文件全部更改",
                DiscardTarget::All => "已回滚暂存区全部更改",
            },
            DiffScope::Unstaged => match target {
                DiscardTarget::Single => "已回滚未暂存更改",
                DiscardTarget::Selected => "已回滚选定未暂存更改",
                DiscardTarget::All => "已回滚修改区全部更改",
            },
        };
        let Some(tab_id) = self.active_tab_id() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            self.last_error = Some("请先打开一个仓库".into());
            return;
        };
        self.close_dialog();
        self.change_context_menu = None;
        let path_set = paths.iter().cloned().collect::<BTreeSet<_>>();
        self.change_selection
            .selected_mut(&scope)
            .retain(|path| !path_set.contains(path));
        self.clear_change_anchor_if_empty(&scope);
        self.diff = None;
        self.diff_headers_expanded = false;
        self.reset_uniform_scroll("diff-scroll");

        let service = self.service_for_tab(tab_id);
        let load_id = {
            let Some(tab) = self.tab_mut(tab_id) else {
                return;
            };
            tab.repository_load_id = tab.repository_load_id.wrapping_add(1);
            tab.repository_load_id
        };
        self.spawn_operation_without_load_bump(
            Some(tab_id),
            "正在回滚文件更改",
            move || {
                let mut repo = Repository::open(repo_path)?;
                let paths = paths.iter().map(PathBuf::from).collect::<Vec<_>>();
                let path_refs = paths.iter().map(PathBuf::as_path).collect::<Vec<_>>();
                let snapshot = match scope {
                    DiffScope::Staged => service.discard_all_paths(&mut repo, path_refs)?,
                    DiffScope::Unstaged => service.discard_unstaged_paths(&mut repo, path_refs)?,
                };
                let changes = service.status_full(&repo)?;
                Ok(UiEvent::DiscardChangeFinished {
                    tab_id,
                    message: message.to_string(),
                    snapshot,
                    changes,
                    load_id,
                })
            },
        );
    }

    fn spawn_operation_without_load_bump<F>(
        &mut self,
        tab_id: Option<RepoTabId>,
        started: &'static str,
        f: F,
    ) where
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

    pub(crate) fn copy_commit_sha(&mut self, oid: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(oid));
        self.commit_context_menu = None;
        self.status = "已复制提交 SHA".into();
        self.last_error = None;
    }

    fn toggle_encoding_menu(&mut self, target: EncodingMenuTarget) {
        if self.encoding_menu_closed_by_capture == Some(target) {
            self.encoding_menu_closed_by_capture = None;
            self.encoding_menu_target = None;
            return;
        }
        self.encoding_menu_closed_by_capture = None;
        self.branch_context_menu = None;
        self.change_context_menu = None;
        self.tag_context_menu = None;
        self.stash_context_menu = None;
        self.commit_context_menu = None;
        self.active_dialog = None;
        self.encoding_menu_target = if self.encoding_menu_target == Some(target) {
            None
        } else {
            Some(target)
        };
    }

    fn choose_diff_encoding(&mut self, encoding: DiffEncodingChoice) {
        self.encoding_menu_target = None;
        self.encoding_menu_closed_by_capture = None;
        self.set_current_diff_encoding(encoding);
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

    fn select_only_change(&mut self, path: String, scope: DiffScope, load_diff: bool) {
        self.change_selection.clear();
        self.change_selection
            .selected_mut(&scope)
            .insert(path.clone());
        self.change_selection.set_anchor(&scope, path.clone());
        if load_diff {
            self.load_diff(path, scope);
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

    fn open_change_context_menu(
        &mut self,
        path: String,
        scope: DiffScope,
        event: &MouseDownEvent,
        window: &Window,
    ) {
        self.ensure_change_context_selection(path.clone(), scope.clone());
        self.branch_context_menu = None;
        self.tag_context_menu = None;
        self.stash_context_menu = None;
        self.commit_context_menu = None;
        self.encoding_menu_target = None;
        self.active_dialog = None;
        let (x, y) = clamped_menu_position(event, window, CHANGE_MENU_WIDTH, CHANGE_MENU_HEIGHT);
        self.change_context_menu = Some(ChangeContextMenu { path, scope, x, y });
    }

    fn mouse_down_inside_context_menu(&self, event: &MouseDownEvent) -> bool {
        let x: f32 = event.position.x.into();
        let y: f32 = event.position.y.into();
        self.branch_context_menu.as_ref().is_some_and(|menu| {
            point_in_menu(x, y, menu.x, menu.y, BRANCH_MENU_WIDTH, BRANCH_MENU_HEIGHT)
        }) || self.change_context_menu.as_ref().is_some_and(|menu| {
            point_in_menu(x, y, menu.x, menu.y, CHANGE_MENU_WIDTH, CHANGE_MENU_HEIGHT)
        }) || self.tag_context_menu.as_ref().is_some_and(|menu| {
            point_in_menu(x, y, menu.x, menu.y, TAG_MENU_WIDTH, TAG_MENU_HEIGHT)
        }) || self.stash_context_menu.as_ref().is_some_and(|menu| {
            point_in_menu(x, y, menu.x, menu.y, STASH_MENU_WIDTH, STASH_MENU_HEIGHT)
        }) || self.commit_context_menu.as_ref().is_some_and(|menu| {
            point_in_menu(x, y, menu.x, menu.y, COMMIT_MENU_WIDTH, COMMIT_MENU_HEIGHT)
        })
    }

    pub(crate) fn open_commit_context_menu(
        &mut self,
        oid: String,
        short_oid: String,
        summary: String,
        parent_count: usize,
        event: &MouseDownEvent,
        window: &Window,
    ) {
        self.select_history_commit(oid.clone());
        self.branch_context_menu = None;
        self.change_context_menu = None;
        self.tag_context_menu = None;
        self.stash_context_menu = None;
        self.encoding_menu_target = None;
        self.active_dialog = None;
        let (x, y) = clamped_menu_position(event, window, COMMIT_MENU_WIDTH, COMMIT_MENU_HEIGHT);
        self.commit_context_menu = Some(CommitContextMenu {
            oid,
            short_oid,
            summary,
            parent_count,
            x,
            y,
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
        self.reset_uniform_scroll("diff-scroll");
    }

    fn toggle_history_diff_headers(&mut self) {
        self.history_diff_headers_expanded = !self.history_diff_headers_expanded;
        self.reset_uniform_scroll("history-diff-scroll");
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
        self.history_graph_rows.clear();
    }

    pub(crate) fn set_history_scope(&mut self, scope: HistoryScope) {
        if self.history_scope == scope {
            return;
        }
        self.history_scope = scope;
        self.clear_history();
        self.status = format!("提交记录范围已切换为{}", scope.label());
        self.load_history_page(false);
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
        let scope = self.history_scope;
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
            let started = Instant::now();
            let result = (|| -> khaslana::Result<UiEvent> {
                let repo = Repository::open(repo_path)?;
                let mut commits =
                    service.commit_history(&repo, scope, offset, HISTORY_PAGE_SIZE + 1)?;
                let has_more = commits.len() > HISTORY_PAGE_SIZE;
                commits.truncate(HISTORY_PAGE_SIZE);
                perf_log(
                    "history.commits",
                    started,
                    format!(
                        "tab={} scope={} append={} offset={} commits={} has_more={}",
                        tab_id.0,
                        scope.label(),
                        append,
                        offset,
                        commits.len(),
                        has_more
                    ),
                );
                Ok(UiEvent::HistoryCommitsLoaded {
                    tab_id,
                    commits,
                    append,
                    has_more,
                    scope,
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
        self.reset_uniform_scroll("history-diff-scroll");
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
            let started = Instant::now();
            let result = (|| -> khaslana::Result<UiEvent> {
                let repo = Repository::open(repo_path)?;
                let files = service.commit_files(&repo, &oid)?;
                perf_log(
                    "history.files",
                    started,
                    format!("tab={} files={}", tab_id.0, files.len()),
                );
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
        self.select_history_file_with_reload(path, false);
    }

    fn select_history_file_with_reload(&mut self, path: String, force_reload: bool) {
        let Some(commit_oid) = self.history_selected_commit.clone() else {
            return;
        };
        if !force_reload
            && self.history_selected_file.as_deref() == Some(path.as_str())
            && self.history_diff.is_some()
        {
            return;
        }

        self.history_selected_file = Some(path.clone());
        self.history_diff = None;
        self.history_diff_headers_expanded = false;
        self.reset_uniform_scroll("history-diff-scroll");
        self.history_loading.diff = true;
        self.status = "正在加载提交差异".to_string();

        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        let encoding = self.diff_encoding_choice_for_path(&repo_path);
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        let load_id = self.repository_load_id;

        thread::spawn(move || {
            let started = Instant::now();
            let result = (|| -> khaslana::Result<UiEvent> {
                let repo = Repository::open(repo_path)?;
                let diff =
                    service.commit_file_diff(&repo, &commit_oid, Path::new(&path), encoding)?;
                perf_log(
                    "history.diff",
                    started,
                    format!("tab={} lines={}", tab_id.0, diff.lines.len()),
                );
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
        self.commit_message.clear();
        self.with_repo("提交完成", move |service, repo| {
            service.commit(repo, &CommitMessage::new(message))
        });
    }

    fn load_diff(&mut self, path: String, scope: DiffScope) {
        self.reset_uniform_scroll("diff-scroll");
        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        let encoding = self.diff_encoding_choice_for_path(&repo_path);
        let service = self.service_for_tab(tab_id);
        self.spawn_operation_for_tab(Some(tab_id), "正在加载差异", move || {
            let started = Instant::now();
            let repo = Repository::open(repo_path)?;
            service
                .diff_for_path(&repo, Path::new(&path), scope, encoding)
                .map(|diff| {
                    perf_log(
                        "worktree.diff",
                        started,
                        format!("tab={} lines={}", tab_id.0, diff.lines.len()),
                    );
                    UiEvent::OperationFinished {
                        tab_id: Some(tab_id),
                        message: "差异已加载".to_string(),
                        snapshot: None,
                        diff: Some(diff),
                    }
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

        let credential = if self.credential_form_mode == CredentialFormMode::Ssh {
            GitCredential::SshPassphrase {
                username,
                private_key_path: (!self.credential_use_ssh_agent && !key_path.is_empty())
                    .then_some(key_path),
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

        if !send_credential_response(&pending, Ok(Some(credential))) {
            self.last_error = Some("凭据请求已失效".into());
            return;
        }
        self.show_next_credential_request();
        self.apply_status_event(pending.tab_id, |this| {
            this.status = "凭据已提交，正在继续操作".into();
            this.last_error = None;
        });
        self.reload_credential_records("凭据已提交");
        self.save_remote_credential_bindings();
    }

    fn cancel_credential_request(&mut self) {
        let Some(pending) = self.pending_credential.clone() else {
            return;
        };
        let _ = send_credential_response(
            &pending,
            Err(khaslana::GitError::Credential("已取消凭据输入".into())),
        );
        self.show_next_credential_request();
        self.apply_status_event(pending.tab_id, |this| {
            this.status = "凭据输入已取消".into();
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

    fn credential_kind_button(
        &self,
        label: &'static str,
        mode: CredentialFormMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.credential_form_mode == mode;
        div()
            .id(format!("credential-kind-{label}"))
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
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                this.credential_form_mode = mode;
                cx.notify();
            }))
            .child(label)
    }

    fn toggle_row(
        &self,
        id: &'static str,
        label: &'static str,
        checked: bool,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex()
            .items_center()
            .gap_2()
            .cursor_pointer()
            .on_click(cx.listener(move |this, _event, window, cx| {
                on_click(this, window, cx);
                cx.notify();
            }))
            .child(
                div()
                    .size(px(14.0))
                    .border_1()
                    .border_color(rgb(COLOR_BORDER_STRONG))
                    .bg(if checked {
                        rgb(COLOR_BLUE)
                    } else {
                        rgb(COLOR_SURFACE)
                    }),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .child(label),
            )
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
        let display = field.display_text();
        let selection = field.selected_range();
        let selection_display = selection.clone().map(|range| {
            field.display_byte_for_value_byte(range.start)
                ..field.display_byte_for_value_byte(range.end)
        });
        let caret_display = field.display_byte_for_value_byte(field.caret);
        let field_for_click = field.clone();
        let focus = field.focus.clone();
        let entity = cx.entity();
        div()
            .id(format!("field-{id:?}"))
            .relative()
            .track_focus(&field.focus)
            .on_key_down(cx.listener(move |this, event, window, cx| {
                this.handle_field_key(id, event, window, cx);
                cx.stop_propagation();
            }))
            .px_2()
            .py_1()
            .min_h(if compact { px(26.0) } else { px(32.0) })
            .w_full()
            .flex()
            .items_center()
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
            .cursor(CursorStyle::IBeam)
            .child(
                div()
                    .flex()
                    .items_center()
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .children(input_segments(
                        display,
                        field.placeholder.to_string(),
                        empty,
                        focused,
                        selection_display,
                        caret_display,
                    )),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _cx| {
                        let focus = focus.clone();
                        let field_for_click = field_for_click.clone();
                        let entity = entity.clone();
                        window.on_mouse_event(move |event: &MouseDownEvent, _phase, window, cx| {
                            if event.button != MouseButton::Left
                                || !bounds.contains(&event.position)
                            {
                                return;
                            }
                            window.focus(&focus);
                            let local_x = f32::from(event.position.x - bounds.left()) - 8.0;
                            let byte = field_for_click.byte_for_approx_x(local_x.max(0.0));
                            entity.update(cx, |this, cx| {
                                this.field_mut(id)
                                    .move_caret_to(byte, event.modifiers.shift);
                                cx.notify();
                            });
                            cx.stop_propagation();
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

        let handle = self.scroll_handle("repo-tab-bar-scroll");
        let content = div()
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
            .track_scroll(&handle)
            .children(
                self.tabs
                    .iter()
                    .map(|tab| self.render_repo_tab(tab, cx).into_any_element())
                    .collect::<Vec<_>>(),
            )
            .into_any_element();

        scrollable_frame_intrinsic(
            "repo-tab-bar-scroll",
            ScrollbarMode::Horizontal,
            content,
            handle,
            cx,
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
            .w(px(TAG_MENU_WIDTH))
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
            .w(px(STASH_MENU_WIDTH))
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
        let selected_paths = self.selected_change_paths(menu.scope.clone());
        let all_paths = self.change_paths(menu.scope.clone());
        let all_count = all_paths.len();

        let mut menu_el = div()
            .absolute()
            .left(px(menu.x))
            .top(px(menu.y))
            .w(px(CHANGE_MENU_WIDTH))
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
            });

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
                ))
                .child(menu_separator())
                .child(context_menu_item(
                    "回滚更改...",
                    !self.busy,
                    {
                        let path = menu.path.clone();
                        let scope = menu.scope.clone();
                        move |this| {
                            this.open_discard_change_confirm_dialog(
                                vec![path.clone()],
                                scope.clone(),
                                DiscardTarget::Single,
                            )
                        }
                    },
                    cx,
                ))
                .child(context_menu_item(
                    "回滚指定更改...",
                    selected_count > 0 && !self.busy,
                    {
                        let paths = selected_paths.clone();
                        let scope = menu.scope.clone();
                        move |this| {
                            this.open_discard_change_confirm_dialog(
                                paths.clone(),
                                scope.clone(),
                                DiscardTarget::Selected,
                            )
                        }
                    },
                    cx,
                ))
                .child(context_menu_item(
                    "回滚全部更改...",
                    all_count > 0 && !self.busy,
                    {
                        let paths = all_paths.clone();
                        let scope = menu.scope.clone();
                        move |this| {
                            this.open_discard_change_confirm_dialog(
                                paths.clone(),
                                scope.clone(),
                                DiscardTarget::All,
                            )
                        }
                    },
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
                ))
                .child(menu_separator())
                .child(context_menu_item(
                    "回滚更改...",
                    !self.busy,
                    {
                        let path = menu.path.clone();
                        let scope = menu.scope.clone();
                        move |this| {
                            this.open_discard_change_confirm_dialog(
                                vec![path.clone()],
                                scope.clone(),
                                DiscardTarget::Single,
                            )
                        }
                    },
                    cx,
                ))
                .child(context_menu_item(
                    "回滚指定更改...",
                    selected_count > 0 && !self.busy,
                    {
                        let paths = selected_paths;
                        let scope = menu.scope.clone();
                        move |this| {
                            this.open_discard_change_confirm_dialog(
                                paths.clone(),
                                scope.clone(),
                                DiscardTarget::Selected,
                            )
                        }
                    },
                    cx,
                ))
                .child(context_menu_item(
                    "回滚全部更改...",
                    all_count > 0 && !self.busy,
                    {
                        let paths = all_paths;
                        let scope = menu.scope.clone();
                        move |this| {
                            this.open_discard_change_confirm_dialog(
                                paths.clone(),
                                scope.clone(),
                                DiscardTarget::All,
                            )
                        }
                    },
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
            .w(px(COMMIT_MENU_WIDTH))
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

    pub(crate) fn render_encoding_dropdown(
        &self,
        target: EncodingMenuTarget,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        if self.encoding_menu_target != Some(target) {
            return div().into_any_element();
        }
        let current = self.current_diff_encoding_choice();
        let title = match target {
            EncodingMenuTarget::Worktree => "工作区差异编码",
            EncodingMenuTarget::History => "提交差异编码",
        };

        div()
            .absolute()
            .top(px(38.0))
            .right(px(12.0))
            .w(px(ENCODING_MENU_WIDTH))
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER_STRONG))
            .bg(rgb(COLOR_SURFACE))
            .shadow_lg()
            .flex()
            .flex_col()
            .text_size(px(12.0))
            .occlude()
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
                    .child(title),
            )
            .child(menu_separator())
            .child(self.encoding_menu_item(DiffEncodingChoice::Auto, current, cx))
            .child(self.encoding_menu_item(DiffEncodingChoice::Utf8, current, cx))
            .child(self.encoding_menu_item(DiffEncodingChoice::Gb18030, current, cx))
            .child(self.encoding_menu_item(DiffEncodingChoice::Big5, current, cx))
            .into_any_element()
    }

    fn encoding_menu_item(
        &self,
        choice: DiffEncodingChoice,
        current: DiffEncodingChoice,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = choice == current;
        let label = if selected {
            format!("✓ {}", choice.label())
        } else {
            format!("  {}", choice.label())
        };
        div()
            .id(format!("context-menu-encoding-{}", choice.label()))
            .px_3()
            .py_1()
            .text_color(if selected {
                rgb(COLOR_BLUE_DARK)
            } else {
                rgb(COLOR_TEXT)
            })
            .bg(if selected {
                rgb(COLOR_BLUE_SOFT)
            } else {
                rgb(COLOR_SURFACE)
            })
            .cursor_pointer()
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                cx.stop_propagation();
                this.choose_diff_encoding(choice);
                cx.notify();
            }))
            .child(label)
    }

    fn render_changes(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let (staged_rows, unstaged_rows) = if let Some(snapshot) = self.snapshot.as_ref() {
            let staged_rows = self
                .change_indexes
                .staged
                .iter()
                .filter_map(|index| snapshot.changes.get(*index))
                .cloned()
                .map(|change| {
                    self.change_row(change, DiffScope::Staged, cx)
                        .into_any_element()
                })
                .collect::<Vec<_>>();
            let unstaged_rows = self
                .change_indexes
                .unstaged
                .iter()
                .filter_map(|index| snapshot.changes.get(*index))
                .cloned()
                .map(|change| {
                    self.change_row(change, DiffScope::Unstaged, cx)
                        .into_any_element()
                })
                .collect::<Vec<_>>();
            (staged_rows, unstaged_rows)
        } else {
            (Vec::new(), Vec::new())
        };
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
                has_staged || self.loading.staged(),
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
                cx,
            ))
            .child(div().flex_none().h(px(1.0)).bg(rgb(COLOR_BORDER)))
            .child(self.render_change_section(
                "修改区",
                "unstaged-change-list",
                "修改区加载中...",
                self.loading.unstaged(),
                unstaged_rows,
                has_unstaged || self.loading.unstaged(),
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
                cx,
            ))
    }

    fn render_change_section(
        &self,
        title: &'static str,
        id: &'static str,
        loading_text: &'static str,
        loading: bool,
        rows: Vec<gpui::AnyElement>,
        content_present: bool,
        actions: Vec<gpui::AnyElement>,
        cx: &mut Context<Self>,
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
            .child({
                let handle = self.scroll_handle(id);
                let content = div()
                    .id(id)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap_1()
                    .min_w(px(0.0))
                    .min_h(px(0.0))
                    .p_2()
                    .overflow_scroll()
                    .track_scroll(&handle)
                    .children(rows)
                    .into_any_element();
                scrollable_frame_when(
                    id,
                    ScrollbarMode::Both,
                    content,
                    handle,
                    content_present,
                    cx,
                )
            })
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
                    this.open_change_context_menu(path.clone(), scope.clone(), event, _window);
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

        div()
            .flex()
            .flex_col()
            .flex_1()
            .relative()
            .min_w(px(0.0))
            .min_h(px(260.0))
            .child(self.diff_section_header(title, EncodingMenuTarget::Worktree, cx))
            .child(self.render_virtual_diff(
                "diff-scroll",
                self.diff.clone(),
                self.diff_headers_expanded,
                DiffHeaderTarget::Worktree,
                "请选择一个变更文件查看差异".to_string(),
                cx,
            ))
            .child(self.render_encoding_dropdown(EncodingMenuTarget::Worktree, cx))
    }

    fn render_virtual_diff(
        &self,
        scroll_id: &'static str,
        diff: Option<Arc<FileDiff>>,
        headers_expanded: bool,
        header_target: DiffHeaderTarget,
        empty_message: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let model = diff_render_model_for(diff.as_deref(), headers_expanded);
        let row_count = model.row_count;
        let content_present = diff.is_some() && row_count > 0;
        let width_measure_index = (0..row_count)
            .find(|index| matches!(model.row_at(*index), DiffRenderRow::DiffLine(_)))
            .or_else(|| row_count.checked_sub(1));
        let handle = self.uniform_scroll_handle(scroll_id);
        let list_handle = handle.clone();
        let model_for_list = model.clone();
        let content = div()
            .id(scroll_id)
            .flex()
            .flex_col()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .p_2()
            .font_family("Consolas, monospace")
            .text_size(px(12.0))
            .bg(rgb(COLOR_PANEL_BG))
            .child(
                uniform_list(
                    scroll_id,
                    row_count,
                    cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                        let diff = diff.as_deref();
                        range
                            .map(|index| {
                                this.render_diff_row(
                                    diff,
                                    model_for_list.row_at(index),
                                    headers_expanded,
                                    header_target,
                                    &empty_message,
                                    cx,
                                )
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .track_scroll(&list_handle)
                .with_width_from_item(width_measure_index)
                .with_sizing_behavior(ListSizingBehavior::Auto)
                .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
                .flex_1()
                .min_w(px(0.0))
                .min_h(px(0.0)),
            )
            .into_any_element();

        scrollable_uniform_frame(
            scroll_id,
            ScrollbarMode::Both,
            content,
            handle,
            content_present,
            cx,
        )
    }

    fn render_diff_row(
        &self,
        diff: Option<&FileDiff>,
        row: DiffRenderRow,
        headers_expanded: bool,
        header_target: DiffHeaderTarget,
        empty_message: &str,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        match row {
            DiffRenderRow::HeaderToggle => {
                let summary = if headers_expanded {
                    "Diff 元信息（点击折叠）"
                } else {
                    "Diff 元信息（点击展开）"
                };
                diff_header_toggle(summary, header_target, cx).into_any_element()
            }
            DiffRenderRow::DiffLine(index) => {
                let Some(line) = diff.and_then(|diff| diff.lines.get(index)) else {
                    return diff_line(DiffLineKind::Context, None, None, String::new())
                        .into_any_element();
                };
                if line.kind == DiffLineKind::Header {
                    diff_line(line.kind.clone(), None, None, line.content.clone())
                        .into_any_element()
                } else {
                    diff_line(
                        line.kind.clone(),
                        line.old_lineno,
                        line.new_lineno,
                        line.content.clone(),
                    )
                    .into_any_element()
                }
            }
            DiffRenderRow::Empty => {
                let message = diff
                    .map(|diff| {
                        if diff.is_binary {
                            "二进制文件仅显示元信息"
                        } else {
                            "没有可显示的文本差异"
                        }
                    })
                    .unwrap_or(empty_message);
                diff_line(DiffLineKind::Context, None, None, message.to_string()).into_any_element()
            }
        }
    }

    pub(crate) fn diff_section_header(
        &self,
        title: String,
        target: EncodingMenuTarget,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let diff = match target {
            EncodingMenuTarget::Worktree => self.diff.as_deref(),
            EncodingMenuTarget::History => self.history_diff.as_deref(),
        };
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
                    .min_w(px(0.0))
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(COLOR_TEXT))
                    .truncate()
                    .child(title),
            )
            .child(self.encoding_button(diff, target, cx))
    }

    fn encoding_button(
        &self,
        diff: Option<&FileDiff>,
        target: EncodingMenuTarget,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let requested = self.current_diff_encoding_choice();
        let label = diff
            .map(diff_encoding_label)
            .unwrap_or_else(|| format!("编码：{}", requested.label()));
        div()
            .id(match target {
                EncodingMenuTarget::Worktree => "worktree-diff-encoding",
                EncodingMenuTarget::History => "history-diff-encoding",
            })
            .relative()
            .flex_none()
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(rgb(COLOR_SURFACE))
            .text_color(rgb(COLOR_TEXT_MUTED))
            .text_size(px(11.0))
            .cursor_pointer()
            .hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _event: &MouseDownEvent, _window, cx| {
                    cx.stop_propagation();
                    this.toggle_encoding_menu(target);
                    cx.notify();
                }),
            )
            .child(label)
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
            .when(
                self.credential_form_mode == CredentialFormMode::Https,
                |this| this.child(self.input(FieldId::CredentialSecret, true, window, cx)),
            )
            .when(
                self.credential_form_mode == CredentialFormMode::Ssh,
                |this| {
                    this.child(self.toggle_row(
                        "credential-use-ssh-agent",
                        "使用 SSH agent",
                        self.credential_use_ssh_agent,
                        |this, _, _| this.credential_use_ssh_agent = !this.credential_use_ssh_agent,
                        cx,
                    ))
                    .when(!self.credential_use_ssh_agent, |this| {
                        this.child(self.input(FieldId::CredentialKeyPath, true, window, cx))
                    })
                    .child(self.input(
                        FieldId::CredentialPassphrase,
                        true,
                        window,
                        cx,
                    ))
                },
            )
            .child(self.toggle_row(
                "save-credential",
                "保存到系统凭据管理器",
                self.save_credential,
                |this, _, _| this.save_credential = !this.save_credential,
                cx,
            ))
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
                    .child(self.button("使用凭据", true, |this, _, _| this.use_credentials(), cx))
                    .child(self.button(
                        "取消",
                        true,
                        |this, _, _| this.cancel_credential_request(),
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
            DialogState::ConfirmDiscardChange {
                scope,
                target,
                paths,
            } => self
                .render_confirm_discard_change_dialog(scope, target, paths, cx)
                .into_any_element(),
            DialogState::CredentialManager => {
                self.render_credential_manager_dialog(cx).into_any_element()
            }
            DialogState::CredentialForm { editing } => self
                .render_credential_form_dialog(editing, window, cx)
                .into_any_element(),
            DialogState::RemoteManager => self.render_remote_manager_dialog(cx).into_any_element(),
            DialogState::RemoteForm { editing } => self
                .render_remote_form_dialog(editing, window, cx)
                .into_any_element(),
            DialogState::ConfirmDeleteRemote { name } => self
                .render_confirm_delete_remote_dialog(name, cx)
                .into_any_element(),
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
        let preview = infer_clone_target_path(&self.clone_url.value, &self.clone_path.value)
            .map(|path| path.display().to_string());
        self.dialog_panel("克隆仓库", cx)
            .child(self.input(FieldId::CloneUrl, false, window, cx))
            .child(self.input(FieldId::ClonePath, false, window, cx))
            .child(
                div()
                    .px_2()
                    .text_size(px(12.0))
                    .text_color(rgb(if preview.is_some() {
                        COLOR_TEXT_MUTED
                    } else {
                        COLOR_TEXT_FAINT
                    }))
                    .child(preview.unwrap_or_else(|| {
                        "填写远程仓库 URL 和父文件夹后显示最终代码路径".to_string()
                    })),
            )
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

    fn render_confirm_discard_change_dialog(
        &self,
        scope: DiffScope,
        target: DiscardTarget,
        paths: Vec<String>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let count = paths.len();
        let target_label = match target {
            DiscardTarget::Single => "目标文件".to_string(),
            DiscardTarget::Selected => format!("选定文件（{count} 个）"),
            DiscardTarget::All => match scope {
                DiffScope::Staged => format!("暂存区全部文件（{count} 个）"),
                DiffScope::Unstaged => format!("修改区全部文件（{count} 个）"),
            },
        };
        let preview = discard_paths_preview(&paths);
        let help = match scope {
            DiffScope::Staged => {
                "将丢弃这些文件全部未提交更改，包括暂存区和工作区。新增文件会被删除，删除文件会被恢复。此操作无法从 Khaslana 内撤销。"
            }
            DiffScope::Unstaged => {
                "将仅丢弃这些文件尚未暂存的更改，已暂存内容会保留。未跟踪新增文件会被删除。此操作无法从 Khaslana 内撤销。"
            }
        };
        self.dialog_panel("确认回滚更改", cx)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .child(target_label),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(preview),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child(help),
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
                            let paths = paths.clone();
                            move |this, _, _| {
                                this.discard_change(paths.clone(), scope.clone(), target.clone())
                            }
                        },
                        cx,
                    )),
            )
    }

    fn render_remote_manager_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let remotes = self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.remotes.clone())
            .unwrap_or_default();
        let rows = if remotes.is_empty() {
            vec![placeholder_row("暂无远端。可以点击“新增远端”添加。").into_any_element()]
        } else {
            remotes
                .into_iter()
                .map(|remote| self.remote_manager_row(remote, cx).into_any_element())
                .collect::<Vec<_>>()
        };

        div()
            .id("dialog-远端管理")
            .w(px(820.0))
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
                            .child("远端管理"),
                    )
                    .child(self.button(
                        "新增远端",
                        self.repo_path.is_some() && !self.busy,
                        |this, _, _| this.open_remote_form(None),
                        cx,
                    )),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child("远端地址会同时作为 fetch 和 push URL；凭据只从已保存凭据中选择。"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .min_h(px(0.0))
                    .max_h(px(420.0))
                    .border_1()
                    .border_color(rgb(COLOR_BORDER))
                    .rounded_sm()
                    .child(self.remote_manager_header())
                    .child({
                        let handle = self.scroll_handle("remote-manager-list");
                        let content = div()
                            .id("remote-manager-list")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap_0()
                            .min_w(px(0.0))
                            .min_h(px(0.0))
                            .overflow_y_scroll()
                            .track_scroll(&handle)
                            .children(rows)
                            .into_any_element();
                        scrollable_frame_when(
                            "remote-manager-list",
                            ScrollbarMode::Vertical,
                            content,
                            handle,
                            self.snapshot
                                .as_ref()
                                .is_some_and(|snapshot| !snapshot.remotes.is_empty()),
                            cx,
                        )
                    }),
            )
            .child(div().flex().justify_end().child(self.button(
                "关闭",
                !self.busy,
                |this, _, _| this.close_dialog(),
                cx,
            )))
    }

    fn remote_manager_header(&self) -> impl IntoElement {
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
            .child(div().flex_none().w(px(104.0)).child("名称"))
            .child(div().flex_1().min_w(px(0.0)).child("地址"))
            .child(div().flex_none().w(px(180.0)).child("凭据"))
            .child(div().flex_none().w(px(106.0)).child("操作"))
    }

    fn remote_manager_row(&self, remote: RemoteInfo, cx: &mut Context<Self>) -> impl IntoElement {
        let edit_name = remote.name.clone();
        let delete_name = remote.name.clone();
        let policy = self
            .repo_path
            .as_ref()
            .map(|repo_path| {
                self.remote_credential_policy_for_remote(repo_path, &remote.name, &remote.url)
            })
            .unwrap_or(RemoteCredentialPolicy::AutoMatch);
        let credential_label = match policy {
            RemoteCredentialPolicy::NoCredential => "无凭据".to_string(),
            RemoteCredentialPolicy::Record(record_id) => self
                .credential_records
                .iter()
                .find(|record| record.id == record_id)
                .map(credential_record_label)
                .unwrap_or_else(|| "凭据不存在".to_string()),
            RemoteCredentialPolicy::AutoMatch => self
                .matching_credential_for_remote_url(&remote.url)
                .map(|record| format!("自动：{}", credential_record_label(record)))
                .unwrap_or_else(|| "自动匹配".to_string()),
        };

        div()
            .id(format!("remote-manager-row-{}", remote.name))
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
                    .w(px(104.0))
                    .text_color(rgb(COLOR_TEXT))
                    .truncate()
                    .child(remote.name),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .truncate()
                    .child(remote.url),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(180.0))
                    .text_color(rgb(COLOR_BLUE_DARK))
                    .truncate()
                    .child(credential_label),
            )
            .child(
                div()
                    .flex_none()
                    .w(px(106.0))
                    .flex()
                    .gap_1()
                    .child(self.button(
                        "编辑",
                        !self.busy,
                        move |this, _, _| this.open_remote_form(Some(edit_name.clone())),
                        cx,
                    ))
                    .child(self.button(
                        "删除",
                        !self.busy,
                        move |this, _, _| this.open_delete_remote_confirm(delete_name.clone()),
                        cx,
                    )),
            )
    }

    fn render_remote_form_dialog(
        &self,
        editing: Option<String>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let title = if editing.is_some() {
            "编辑远端"
        } else {
            "新增远端"
        };
        self.dialog_panel(title, cx)
            .w(px(560.0))
            .child(self.input(FieldId::RemoteName, false, window, cx))
            .child(self.input(FieldId::RemoteUrl, false, window, cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(COLOR_TEXT_MUTED))
                            .child("绑定凭据"),
                    )
                    .child(self.remote_credential_picker(cx)),
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
                            this.active_dialog = Some(DialogState::RemoteManager);
                        },
                        cx,
                    ))
                    .child(self.button(
                        "保存",
                        !self.busy,
                        move |this, _, _| this.save_remote(editing.clone()),
                        cx,
                    )),
            )
    }

    fn remote_credential_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let url = self.remote_url.value.trim().to_string();
        let mut rows = Vec::new();
        rows.push(
            self.remote_credential_option(
                RemoteCredentialPolicy::AutoMatch,
                "自动匹配保存凭据".to_string(),
                true,
                cx,
            )
            .into_any_element(),
        );
        rows.push(
            self.remote_credential_option(
                RemoteCredentialPolicy::NoCredential,
                "无凭据".to_string(),
                true,
                cx,
            )
            .into_any_element(),
        );
        rows.extend(
            self.credential_records
                .iter()
                .cloned()
                .map(|record| {
                    let compatible = if url.is_empty() {
                        true
                    } else {
                        match record.scope {
                            CredentialScope::RemoteUrl => {
                                credential_record_is_compatible_with_url(&record, &url)
                            }
                            CredentialScope::Host => {
                                credential_record_matches_remote_url(&record, &url)
                            }
                        }
                    };
                    let mut label = credential_record_label(&record);
                    if record.scope == CredentialScope::Host {
                        label = format!("{label} ({})", credential_display_target(&record));
                    }
                    if !compatible {
                        label.push_str("（不匹配）");
                    }
                    self.remote_credential_option(
                        RemoteCredentialPolicy::Record(record.id),
                        label,
                        compatible,
                        cx,
                    )
                    .into_any_element()
                })
                .collect::<Vec<_>>(),
        );

        div()
            .flex()
            .flex_col()
            .max_h(px(168.0))
            .border_1()
            .border_color(rgb(COLOR_BORDER))
            .rounded_sm()
            .bg(rgb(COLOR_SURFACE))
            .children(rows)
    }

    fn remote_credential_option(
        &self,
        policy: RemoteCredentialPolicy,
        label: String,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.remote_credential_policy == policy;
        let id_label = match &policy {
            RemoteCredentialPolicy::AutoMatch => "auto",
            RemoteCredentialPolicy::NoCredential => "none",
            RemoteCredentialPolicy::Record(record_id) => record_id.as_str(),
        };
        div()
            .id(format!("remote-credential-option-{id_label}"))
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .py_2()
            .border_b_1()
            .border_color(rgb(COLOR_BORDER))
            .bg(if selected {
                rgb(COLOR_ROW_SELECTED)
            } else {
                rgb(COLOR_SURFACE)
            })
            .text_size(px(12.0))
            .text_color(if enabled {
                rgb(COLOR_TEXT)
            } else {
                rgb(COLOR_TEXT_FAINT)
            })
            .cursor_pointer()
            .when(enabled, |this| {
                this.hover(|this| this.bg(rgb(COLOR_BLUE_SOFT)))
            })
            .child(
                div()
                    .flex_none()
                    .size(px(10.0))
                    .rounded_full()
                    .border_1()
                    .border_color(if selected {
                        rgb(COLOR_BLUE_DARK)
                    } else {
                        rgb(COLOR_BORDER)
                    })
                    .bg(if selected {
                        rgb(COLOR_BLUE)
                    } else {
                        rgb(COLOR_SURFACE)
                    }),
            )
            .child(div().flex_1().min_w(px(0.0)).truncate().child(label))
            .on_click(cx.listener(move |this, _event, _window, cx| {
                if enabled {
                    this.remote_credential_policy = policy.clone();
                    cx.notify();
                }
            }))
    }

    fn render_confirm_delete_remote_dialog(
        &self,
        name: String,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.dialog_panel("删除远端", cx)
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT))
                    .child(format!("确认删除远端：{name}")),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(COLOR_TEXT_MUTED))
                    .child("这只会删除当前仓库的远端配置，不会删除任何已保存凭据。"),
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
                            this.active_dialog = Some(DialogState::RemoteManager);
                        },
                        cx,
                    ))
                    .child(self.button(
                        "确认删除",
                        !self.busy,
                        move |this, _, _| this.delete_remote(name.clone()),
                        cx,
                    )),
            )
    }

    fn render_credential_form_dialog(
        &self,
        _editing: Option<String>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        self.dialog_panel("添加凭据", cx)
            .w(px(560.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(COLOR_TEXT_MUTED))
                            .child("类型"),
                    )
                    .child(self.credential_kind_button("HTTPS", CredentialFormMode::Https, cx))
                    .child(self.credential_kind_button("SSH", CredentialFormMode::Ssh, cx)),
            )
            .child(self.input(FieldId::CredentialRemoteUrl, false, window, cx))
            .child(self.input(FieldId::CredentialUsername, false, window, cx))
            .when(
                self.credential_form_mode == CredentialFormMode::Https,
                |this| this.child(self.input(FieldId::CredentialSecret, false, window, cx)),
            )
            .when(
                self.credential_form_mode == CredentialFormMode::Ssh,
                |this| {
                    this.child(self.toggle_row(
                        "credential-form-use-ssh-agent",
                        "使用 SSH agent",
                        self.credential_use_ssh_agent,
                        |this, _, _| this.credential_use_ssh_agent = !this.credential_use_ssh_agent,
                        cx,
                    ))
                    .when(!self.credential_use_ssh_agent, |this| {
                        this.child(self.input(FieldId::CredentialKeyPath, false, window, cx))
                    })
                    .child(self.input(
                        FieldId::CredentialPassphrase,
                        false,
                        window,
                        cx,
                    ))
                },
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
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
                    .justify_end()
                    .gap_2()
                    .child(self.button(
                        "取消",
                        !self.busy,
                        |this, _, _| this.active_dialog = Some(DialogState::CredentialManager),
                        cx,
                    ))
                    .child(self.button(
                        "保存",
                        !self.busy,
                        |this, _, _| this.save_credential_form(),
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
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.button(
                                "添加凭据",
                                !self.busy,
                                |this, _, _| this.open_credential_form(),
                                cx,
                            ))
                            .child(self.button(
                                "刷新",
                                !self.busy,
                                |this, _, _| this.reload_credential_records("凭据列表已刷新"),
                                cx,
                            )),
                    ),
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
                    .child({
                        let handle = self.scroll_handle("credential-record-list");
                        let content = div()
                            .id("credential-record-list")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .gap_0()
                            .min_w(px(0.0))
                            .min_h(px(0.0))
                            .overflow_y_scroll()
                            .track_scroll(&handle)
                            .children(rows)
                            .into_any_element();
                        scrollable_frame(
                            "credential-record-list",
                            ScrollbarMode::Vertical,
                            content,
                            handle,
                            cx,
                        )
                    }),
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
                this.encoding_menu_closed_by_capture = None;
                if this.mouse_down_inside_context_menu(event) {
                    return;
                }
                if this.branch_context_menu.is_some()
                    || this.change_context_menu.is_some()
                    || this.tag_context_menu.is_some()
                    || this.stash_context_menu.is_some()
                    || this.commit_context_menu.is_some()
                    || this.encoding_menu_target.is_some()
                {
                    let closed_encoding_menu = this.encoding_menu_target;
                    this.branch_context_menu = None;
                    this.change_context_menu = None;
                    this.tag_context_menu = None;
                    this.stash_context_menu = None;
                    this.commit_context_menu = None;
                    this.encoding_menu_target = None;
                    this.encoding_menu_closed_by_capture = closed_encoding_menu;
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

fn infer_clone_directory_name(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let without_fragment = trimmed.split('#').next().unwrap_or(trimmed);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment)
        .trim_end_matches('/');
    let path_part = if let Some((_, rest)) = without_query.split_once("://") {
        let (_, path) = rest.split_once('/')?;
        path
    } else {
        without_query
    };
    let last_segment = path_part
        .rsplit(['/', ':'])
        .find(|segment| !segment.trim().is_empty())?;
    let name = last_segment
        .strip_suffix(".git")
        .unwrap_or(last_segment)
        .trim();
    let invalid = name.is_empty()
        || name == "."
        || name == ".."
        || name.chars().any(|ch| {
            matches!(ch, '<' | '>' | '"' | '|' | '?' | '*' | '\\')
                || ch.is_control()
                || ch == std::path::MAIN_SEPARATOR
        });
    (!invalid).then(|| name.to_string())
}

fn infer_clone_target_path(url: &str, parent_path: &str) -> Option<PathBuf> {
    let parent_path = parent_path.trim();
    if parent_path.is_empty() {
        return None;
    }
    infer_clone_directory_name(url).map(|name| PathBuf::from(parent_path).join(name))
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

fn diff_encoding_label(diff: &FileDiff) -> String {
    let base = if diff.encoding.requested == DiffEncodingChoice::Auto {
        format!("编码：自动({})", diff.encoding.resolved.label())
    } else {
        format!("编码：{}", diff.encoding.requested.label())
    };
    if diff.encoding.lossy {
        format!("{base}，有替换")
    } else {
        base
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

fn input_segments(
    display: String,
    placeholder: String,
    empty: bool,
    focused: bool,
    selection: Option<std::ops::Range<usize>>,
    caret: usize,
) -> Vec<gpui::AnyElement> {
    let caret_el = || {
        div()
            .flex_none()
            .w(px(1.0))
            .h(px(16.0))
            .bg(rgb(COLOR_TEXT))
            .into_any_element()
    };
    let text_el = |text: String, color: u32| {
        div()
            .flex_none()
            .text_color(rgb(color))
            .child(text)
            .into_any_element()
    };
    let selected_el = |text: String| {
        div()
            .flex_none()
            .bg(rgb(COLOR_BLUE_SOFT))
            .text_color(rgb(COLOR_TEXT))
            .child(text)
            .into_any_element()
    };

    if empty {
        let mut segments = Vec::new();
        if focused {
            segments.push(caret_el());
        }
        segments.push(text_el(placeholder, COLOR_TEXT_FAINT));
        return segments;
    }

    let mut segments = Vec::new();
    let push_caret = |segments: &mut Vec<gpui::AnyElement>, inserted: &mut bool| {
        if focused && !*inserted {
            segments.push(caret_el());
            *inserted = true;
        }
    };
    let mut caret_inserted = false;

    if let Some(selection) = selection {
        let before = display[..selection.start].to_string();
        let selected = display[selection.clone()].to_string();
        let after = display[selection.end..].to_string();
        if caret <= selection.start {
            push_caret(&mut segments, &mut caret_inserted);
        }
        if !before.is_empty() {
            segments.push(text_el(before, COLOR_TEXT));
        }
        if !selected.is_empty() {
            segments.push(selected_el(selected));
        }
        if caret >= selection.end {
            push_caret(&mut segments, &mut caret_inserted);
        }
        if !after.is_empty() {
            segments.push(text_el(after, COLOR_TEXT));
        }
    } else {
        let before = display[..caret].to_string();
        let after = display[caret..].to_string();
        if !before.is_empty() {
            segments.push(text_el(before, COLOR_TEXT));
        }
        push_caret(&mut segments, &mut caret_inserted);
        if !after.is_empty() {
            segments.push(text_el(after, COLOR_TEXT));
        }
    }

    segments
}

pub(crate) fn clamped_menu_position(
    event: &MouseDownEvent,
    window: &Window,
    width: f32,
    height: f32,
) -> (f32, f32) {
    let position_x: f32 = event.position.x.into();
    let position_y: f32 = event.position.y.into();
    let window_size = window.window_bounds().get_bounds().size;
    let max_x =
        (f32::from(window_size.width) - width - MENU_VIEWPORT_MARGIN).max(MENU_VIEWPORT_MARGIN);
    let max_y =
        (f32::from(window_size.height) - height - MENU_VIEWPORT_MARGIN).max(MENU_VIEWPORT_MARGIN);
    (
        position_x.clamp(MENU_VIEWPORT_MARGIN, max_x),
        position_y.clamp(MENU_VIEWPORT_MARGIN, max_y),
    )
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

    #[test]
    fn diff_encoding_preferences_round_trip() {
        let mut preferences = DiffEncodingPreferences::default();
        preferences
            .repositories
            .insert("c:/work/a".to_string(), DiffEncodingChoice::Gb18030);
        preferences
            .repositories
            .insert("c:/work/b".to_string(), DiffEncodingChoice::Big5);

        let json = serde_json::to_string(&preferences).expect("encode preferences");
        let decoded: DiffEncodingPreferences =
            serde_json::from_str(&json).expect("decode preferences");

        assert_eq!(
            decoded.repositories.get("c:/work/a"),
            Some(&DiffEncodingChoice::Gb18030)
        );
        assert_eq!(
            decoded.repositories.get("c:/work/b"),
            Some(&DiffEncodingChoice::Big5)
        );
        assert_eq!(DiffEncodingChoice::default(), DiffEncodingChoice::Auto);
    }

    #[test]
    fn remote_credential_bindings_round_trip() {
        let bindings = RemoteCredentialBindings {
            remotes: vec![
                RemoteCredentialBinding {
                    repo_path: "c:/work/a".to_string(),
                    remote_name: "origin".to_string(),
                    remote_url: "https://example.com/a.git".to_string(),
                    policy: RemoteCredentialPolicy::NoCredential,
                },
                RemoteCredentialBinding {
                    repo_path: "c:/work/b".to_string(),
                    remote_name: "upstream".to_string(),
                    remote_url: "git@example.com:b.git".to_string(),
                    policy: RemoteCredentialPolicy::Record("record-1".to_string()),
                },
            ],
        };

        let json = serde_json::to_string(&bindings).expect("encode bindings");
        let decoded: RemoteCredentialBindings =
            serde_json::from_str(&json).expect("decode bindings");

        assert_eq!(decoded.remotes, bindings.remotes);
    }

    #[test]
    fn remote_binding_for_request_defaults_to_auto_match() {
        let bindings = Arc::new(Mutex::new(RemoteCredentialBindings::default()));
        let request = CredentialRequest {
            url: "https://example.com/a.git".into(),
            username_from_url: None,
            allowed_types: git2::CredentialType::USER_PASS_PLAINTEXT,
            repo_path: Some(PathBuf::from("C:/work/a")),
            remote_name: Some("origin".into()),
        };

        assert_eq!(
            remote_binding_for_request(&bindings, &request),
            RemoteCredentialPolicy::AutoMatch
        );
    }

    #[test]
    fn remote_binding_for_request_matches_repo_remote_and_url() {
        let bindings = Arc::new(Mutex::new(RemoteCredentialBindings::default()));
        let request = CredentialRequest {
            url: "https://example.com/a.git".into(),
            username_from_url: None,
            allowed_types: git2::CredentialType::USER_PASS_PLAINTEXT,
            repo_path: Some(PathBuf::from("C:/work/a")),
            remote_name: Some("origin".into()),
        };

        set_remote_binding_for_request(
            &bindings,
            &request,
            RemoteCredentialPolicy::Record("record-1".into()),
        );

        assert_eq!(
            remote_binding_for_request(&bindings, &request),
            RemoteCredentialPolicy::Record("record-1".into())
        );

        let changed_url = CredentialRequest {
            url: "https://example.com/renamed.git".into(),
            ..request
        };
        assert_eq!(
            remote_binding_for_request(&bindings, &changed_url),
            RemoteCredentialPolicy::AutoMatch
        );
    }

    #[test]
    fn clone_directory_name_is_inferred_from_remote_url() {
        assert_eq!(
            infer_clone_directory_name("https://github.com/FuturePrayer/khaslana.git"),
            Some("khaslana".to_string())
        );
        assert_eq!(
            infer_clone_directory_name("https://example.invalid/team/repo/"),
            Some("repo".to_string())
        );
        assert_eq!(
            infer_clone_directory_name("git@github.com:FuturePrayer/khaslana.git"),
            Some("khaslana".to_string())
        );
        assert_eq!(
            infer_clone_directory_name("https://example.invalid/team/repo.git?ref=main"),
            Some("repo".to_string())
        );
        assert_eq!(infer_clone_directory_name(""), None);
        assert_eq!(infer_clone_directory_name("https://example.invalid/"), None);
    }

    #[test]
    fn clone_target_path_uses_selected_parent_directory() {
        assert_eq!(
            infer_clone_target_path("https://github.com/example/abc", "D:/dev"),
            Some(PathBuf::from("D:/dev").join("abc"))
        );
        assert_eq!(
            infer_clone_target_path("https://github.com/example/abc.git", "D:/dev/"),
            Some(PathBuf::from("D:/dev/").join("abc"))
        );
        assert_eq!(infer_clone_target_path("", "D:/dev"), None);
        assert_eq!(
            infer_clone_target_path("https://github.com/example/abc", ""),
            None
        );
    }

    #[test]
    fn text_field_edits_at_utf8_char_boundaries() {
        let mut field = TextEditState::for_test("ab你cd", false);

        field.move_caret_to(3, false);
        assert_eq!(field.caret, 2);
        field.insert_text("X", false);
        assert_eq!(field.value, "abX你cd");

        field.delete_backward();
        assert_eq!(field.value, "ab你cd");
        assert_eq!(field.caret, 2);

        field.move_caret_to("ab你".len(), false);
        field.delete_backward();
        assert_eq!(field.value, "abcd");
        assert_eq!(field.caret, 2);

        field.set_value("ab你cd");
        field.move_caret_to(2, false);
        field.delete_forward();
        assert_eq!(field.value, "abcd");
        assert_eq!(field.caret, 2);
    }

    #[test]
    fn text_field_selection_replace_and_navigation_work() {
        let mut field = TextEditState::for_test("abcdef", false);

        field.move_caret_to(2, false);
        field.move_caret_to(5, true);
        assert_eq!(field.selected_text().as_deref(), Some("cde"));

        field.insert_text("X", false);
        assert_eq!(field.value, "abXf");
        assert_eq!(field.caret, 3);
        assert_eq!(field.selected_range(), None);

        field.select_all();
        assert_eq!(field.selected_text().as_deref(), Some("abXf"));
        field.move_left(false);
        assert_eq!(field.caret, 0);
        assert_eq!(field.selected_range(), None);

        field.move_right(false);
        assert_eq!(field.caret, 1);
    }

    #[test]
    fn text_field_single_line_paste_strips_newlines() {
        let mut single_line = TextEditState::for_test("ab", false);
        single_line.move_caret_to(1, false);
        single_line.insert_text("x\ny\r\nz", false);
        assert_eq!(single_line.value, "axyzb");

        let mut multiline = TextEditState::for_test("ab", false);
        multiline.move_caret_to(1, false);
        multiline.insert_text("x\ny", true);
        assert_eq!(multiline.value, "ax\nyb");
    }

    #[test]
    fn text_field_secret_display_masks_and_blocks_copyable_text() {
        let mut field = TextEditState::for_test("密码12", true);

        assert_eq!(field.display_text(), "****");
        assert_eq!(field.display_byte_for_value_byte("密码".len()), 2);

        field.select_all();
        assert_eq!(field.selected_text().as_deref(), Some("密码12"));
        assert_eq!(field.copyable_selected_text(), None);

        field.clear();
        assert!(field.value.is_empty());
        assert_eq!(field.caret, 0);
        assert_eq!(field.selected_range(), None);
    }

    #[test]
    fn text_field_click_position_uses_wide_character_widths() {
        let field = TextEditState::for_test("a你b", false);

        assert_eq!(field.byte_for_approx_x(0.0), 0);
        assert_eq!(field.byte_for_approx_x(8.0), 1);
        assert_eq!(field.byte_for_approx_x(18.0), "a你".len());
        assert_eq!(field.byte_for_approx_x(80.0), "a你b".len());
    }

    fn test_diff(lines: Vec<khaslana::DiffLine>, is_binary: bool) -> FileDiff {
        FileDiff {
            path: "file.txt".to_string(),
            scope: DiffScope::Unstaged,
            is_binary,
            encoding: khaslana::DiffEncodingInfo {
                requested: DiffEncodingChoice::Utf8,
                resolved: DiffEncodingChoice::Utf8,
                lossy: false,
            },
            lines,
        }
    }

    fn test_line(kind: DiffLineKind, content: &str) -> khaslana::DiffLine {
        khaslana::DiffLine {
            kind,
            old_lineno: None,
            new_lineno: None,
            content: content.to_string(),
        }
    }

    #[test]
    fn diff_render_rows_track_headers_and_empty_states() {
        let diff = test_diff(
            vec![
                test_line(DiffLineKind::Header, "diff --git a/file.txt b/file.txt"),
                test_line(DiffLineKind::Header, "index 0000000..1111111"),
                test_line(DiffLineKind::Removed, "-old"),
                test_line(DiffLineKind::Added, "+new"),
            ],
            false,
        );

        assert_eq!(
            diff_render_rows_for(Some(&diff), false),
            vec![
                DiffRenderRow::HeaderToggle,
                DiffRenderRow::DiffLine(2),
                DiffRenderRow::DiffLine(3),
            ]
        );
        assert_eq!(
            diff_render_rows_for(Some(&diff), true),
            vec![
                DiffRenderRow::HeaderToggle,
                DiffRenderRow::DiffLine(0),
                DiffRenderRow::DiffLine(1),
                DiffRenderRow::DiffLine(2),
                DiffRenderRow::DiffLine(3),
            ]
        );

        let empty_text_diff = test_diff(Vec::new(), false);
        let empty_binary_diff = test_diff(Vec::new(), true);
        assert_eq!(
            diff_render_rows_for(Some(&empty_text_diff), false),
            vec![DiffRenderRow::Empty]
        );
        assert_eq!(
            diff_render_rows_for(Some(&empty_binary_diff), false),
            vec![DiffRenderRow::Empty]
        );
        assert_eq!(
            diff_render_rows_for(None, false),
            vec![DiffRenderRow::Empty]
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
