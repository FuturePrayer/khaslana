#[path = "../ui_resources/recovery_view_state_messages.rs"]
mod messages;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

use khaslana::{
    BranchInfo, CommitFileChange, CreateRecoveryBranchOutcome, FileDiff, RecoveryCommitDetails,
    RecoveryObjectState, RecoveryRepoIdentity, ReflogEntry, ReflogPage, ReflogScope,
    ReflogSnapshot, ReflogSources,
};

use crate::{RepoTabId, SharedSyntaxSpans, TextFieldState};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum RecoverySide {
    Before,
    #[default]
    After,
}

impl RecoverySide {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Before => khaslana::ui_i18n::text(messages::BEFORE_OPERATION),
            Self::After => khaslana::ui_i18n::text(messages::AFTER_OPERATION),
        }
    }

    pub(super) fn oid(self, entry: &ReflogEntry) -> &str {
        match self {
            Self::Before => &entry.old_oid,
            Self::After => &entry.new_oid,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RecoveryPreview {
    pub(crate) open: bool,
    pub(crate) files: Vec<CommitFileChange>,
    pub(crate) selected_file: Option<String>,
    pub(crate) diff: Option<Arc<FileDiff>>,
    pub(crate) diff_syntax: Option<Arc<SharedSyntaxSpans>>,
    pub(crate) headers_expanded: bool,
    pub(crate) loading_files: bool,
    pub(crate) loading_diff: bool,
    pub(crate) request_id: u64,
    pub(crate) error: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct RecoveryDialog {
    pub identity: RecoveryRepoIdentity,
    pub oid: String,
    pub source: String,
    pub side: RecoverySide,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) struct RecoveryOperation {
    pub id: u64,
    pub cancel: Arc<AtomicBool>,
}

#[derive(Clone, Debug)]
pub(crate) struct RecoveryState {
    pub(crate) show_recovery: bool,
    pub(crate) list_focus: gpui::FocusHandle,
    pub(crate) search: TextFieldState,
    pub(crate) branch_name: TextFieldState,
    pub(crate) preview: RecoveryPreview,
    pub(super) scope: ReflogScope,
    pub(super) days: Option<i64>,
    pub(super) sources: Option<ReflogSources>,
    pub(super) snapshot: Option<Arc<ReflogSnapshot>>,
    pub(super) entries: Vec<ReflogEntry>,
    pub(super) total: usize,
    pub(super) next_offset: Option<usize>,
    pub(super) loading: bool,
    pub(super) querying: bool,
    pub(super) error: Option<String>,
    pub(super) generation: u64,
    pub(super) query_id: u64,
    pub(super) selection_id: u64,
    pub(super) cancel: Option<Arc<AtomicBool>>,
    pub(super) search_changed: Option<Instant>,
    pub(super) active_query: Option<khaslana::ReflogQuery>,
    pub(super) selected: Option<String>,
    pub(super) side: RecoverySide,
    pub(super) details: Option<RecoveryCommitDetails>,
    pub(super) details_loading: bool,
    pub(super) availability: HashMap<String, RecoveryObjectState>,
    pub(super) dialog: Option<RecoveryDialog>,
    pub(super) operation: Option<RecoveryOperation>,
    pub(super) next_operation_id: u64,
    pub(super) narrow_details: bool,
    pub(super) last_created: Option<String>,
    pub(super) branches_refresh_pending: bool,
    pub(super) branches_refreshing: bool,
}

impl RecoveryState {
    pub(crate) fn new(seed: &TextFieldState) -> Self {
        let mut search = seed.empty_sibling();
        search.set_placeholder_message(messages::SEARCH_OPERATIONS_REFERENCES_ACTORS_OR_OIDS);
        let mut branch_name = seed.empty_sibling();
        branch_name.set_placeholder_message(messages::RECOVERY_BRANCH_NAME);
        Self {
            show_recovery: false,
            list_focus: seed.focus.clone(),
            search,
            branch_name,
            preview: RecoveryPreview::default(),
            scope: ReflogScope::Head,
            days: Some(30),
            sources: None,
            snapshot: None,
            entries: Vec::new(),
            total: 0,
            next_offset: None,
            loading: false,
            querying: false,
            error: None,
            generation: 0,
            query_id: 0,
            selection_id: 0,
            cancel: None,
            search_changed: None,
            active_query: None,
            selected: None,
            side: RecoverySide::After,
            details: None,
            details_loading: false,
            availability: HashMap::new(),
            dialog: None,
            operation: None,
            next_operation_id: 0,
            narrow_details: false,
            last_created: None,
            branches_refresh_pending: false,
            branches_refreshing: false,
        }
    }

    pub(crate) fn cancel_tasks(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        if let Some(operation) = &self.operation {
            operation.cancel.store(true, Ordering::Relaxed);
        }
        self.generation = self.generation.wrapping_add(1);
        self.loading = false;
        self.querying = false;
        self.details_loading = false;
        self.search_changed = None;
        self.branches_refresh_pending = false;
    }

    pub(super) fn page_query(
        &mut self,
        query: khaslana::ReflogQuery,
        append: bool,
    ) -> Option<khaslana::ReflogQuery> {
        if append {
            if self.search_changed.is_some() {
                return None;
            }
            return self.active_query.clone();
        }
        self.search_changed = None;
        self.active_query = Some(query.clone());
        Some(query)
    }

    pub(super) fn begin_preview(&mut self) -> u64 {
        self.selection_id = self.selection_id.wrapping_add(1);
        self.preview = RecoveryPreview {
            open: true,
            loading_files: true,
            ..RecoveryPreview::default()
        };
        self.selection_id
    }

    pub(crate) fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    pub(crate) fn has_operation(&self) -> bool {
        self.operation.is_some()
    }

    pub(super) fn context_is_current(&self, read_generation: u64, selection_id: u64) -> bool {
        self.generation == read_generation && self.selection_id == selection_id
    }

    pub(super) fn finish_operation(&mut self, operation_id: u64) -> bool {
        if self.operation.as_ref().map(|operation| operation.id) != Some(operation_id) {
            return false;
        }
        self.operation = None;
        true
    }

    pub(super) fn selected_entry(&self) -> Option<&ReflogEntry> {
        let selected = self.selected.as_deref()?;
        self.entries.iter().find(|entry| entry.id == selected)
    }

    pub(super) fn selected_oid(&self) -> Option<&str> {
        self.selected_entry().map(|entry| self.side.oid(entry))
    }
}

pub(super) fn metadata_is_current(captured: u64, current: u64, busy: bool) -> bool {
    captured == current && !busy
}

#[derive(Clone, Debug)]
pub(crate) struct RecoveryReadKey {
    pub tab_id: RepoTabId,
    pub repo_path: PathBuf,
    pub generation: u64,
}

impl RecoveryReadKey {
    pub(super) fn matches(&self, tab: &crate::RepoTabState) -> bool {
        tab.id == self.tab_id
            && tab.repo_path.as_ref() == Some(&self.repo_path)
            && tab.recovery.generation == self.generation
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RecoveryEvent {
    Loaded {
        key: RecoveryReadKey,
        result: Result<(ReflogSources, Arc<ReflogSnapshot>), String>,
    },
    Queried {
        key: RecoveryReadKey,
        query_id: u64,
        offset: usize,
        page: ReflogPage,
    },
    Objects {
        key: RecoveryReadKey,
        values: Vec<(String, RecoveryObjectState)>,
    },
    Details {
        key: RecoveryReadKey,
        selection_id: u64,
        oid: String,
        result: Result<RecoveryCommitDetails, String>,
    },
    Files {
        key: RecoveryReadKey,
        selection_id: u64,
        oid: String,
        result: Result<Vec<CommitFileChange>, String>,
    },
    Diff {
        key: RecoveryReadKey,
        selection_id: u64,
        request_id: u64,
        oid: String,
        path: String,
        result: Result<FileDiff, String>,
    },
    Created {
        tab_id: RepoTabId,
        repo_path: PathBuf,
        operation_id: u64,
        metadata_generation: u64,
        read_generation: u64,
        selection_id: u64,
        result: Result<CreateRecoveryBranchOutcome, String>,
        branches: Result<Vec<BranchInfo>, String>,
    },
    Branches {
        tab_id: RepoTabId,
        repo_path: PathBuf,
        metadata_generation: u64,
        result: Result<Vec<BranchInfo>, String>,
    },
    QueryFailed {
        key: RecoveryReadKey,
        query_id: u64,
        error: String,
    },
}
