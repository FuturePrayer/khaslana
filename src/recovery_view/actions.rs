use std::collections::HashSet;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use git2::Repository;
use gpui::Context;
use khaslana::{
    CreateRecoveryBranchRequest, GitError, RecoveryObjectState, RecoveryRepoIdentity, ReflogQuery,
    ReflogScope,
};

use super::state::{
    RecoveryDialog, RecoveryEvent, RecoveryOperation, RecoveryPreview, RecoveryReadKey,
    RecoverySide, metadata_is_current,
};
use crate::{DialogState, RepositoryView, SyntaxSlot, TaskKind, UiEvent, send_ui_event};

fn run_read<T>(task: impl FnOnce() -> khaslana::Result<T>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(task))
        .map_err(|payload| format!("恢复任务异常：{}", crate::tasks::panic_message(payload)))?
        .map_err(|error| error.to_string())
}

fn verify_identity(
    service: &khaslana::GitService,
    repo: &Repository,
    identity: &RecoveryRepoIdentity,
) -> khaslana::Result<()> {
    if service.recovery_repo_identity(repo)? != *identity {
        return Err(GitError::Message(
            "仓库或工作树身份已变化，请刷新恢复记录".into(),
        ));
    }
    Ok(())
}

impl RepositoryView {
    fn recovery_read_key(&self) -> Option<RecoveryReadKey> {
        Some(RecoveryReadKey {
            tab_id: self.active_tab_id()?,
            repo_path: self.repo_path.clone()?,
            generation: self.recovery.generation,
        })
    }

    pub(crate) fn set_history_recovery(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.recovery.show_recovery = enabled;
        if enabled && self.recovery.snapshot.is_none() && !self.recovery.loading {
            self.load_recovery(cx);
        }
        cx.notify();
    }

    pub(crate) fn set_recovery_scope(&mut self, scope: ReflogScope, cx: &mut Context<Self>) {
        if self.recovery.scope == scope {
            return;
        }
        self.recovery.scope = scope;
        self.recovery.snapshot = None;
        self.recovery.entries.clear();
        self.recovery.total = 0;
        self.recovery.next_offset = None;
        self.load_recovery(cx);
    }

    pub(crate) fn set_recovery_days(&mut self, days: Option<i64>, cx: &mut Context<Self>) {
        if self.recovery.days != days {
            self.recovery.days = days;
            self.refresh_recovery_query(cx, false);
        }
    }

    pub(crate) fn load_recovery(&mut self, cx: &mut Context<Self>) {
        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        if let Some(cancel) = self.recovery.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.recovery.generation = self.recovery.generation.wrapping_add(1);
        self.recovery.query_id = self.recovery.query_id.wrapping_add(1);
        self.recovery.selection_id = self.recovery.selection_id.wrapping_add(1);
        self.recovery.details = None;
        self.recovery.details_loading = false;
        self.recovery.preview = RecoveryPreview::default();
        self.recovery.availability.clear();
        self.recovery.loading = true;
        self.recovery.querying = false;
        self.recovery.search_changed = None;
        self.recovery.error = None;
        let cancel = Arc::new(AtomicBool::new(false));
        self.recovery.cancel = Some(cancel.clone());
        let key = self.recovery_read_key().expect("已确认仓库标签页");
        let scope = self.recovery.scope.clone();
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Long, move || {
            let result = run_read(|| {
                let repo = Repository::open(&repo_path)?;
                let sources = service.reflog_sources(&repo)?;
                let snapshot = service.read_reflog_snapshot(&repo, &scope, &cancel)?;
                if snapshot.identity != sources.identity {
                    return Err(GitError::Message("读取期间仓库身份变化，请重试".into()));
                }
                Ok((sources, Arc::new(snapshot)))
            });
            send_ui_event(
                &tx,
                UiEvent::Recovery(RecoveryEvent::Loaded { key, result }),
            );
        });
        cx.notify();
    }

    pub(crate) fn cancel_recovery_read(&mut self) {
        if let Some(cancel) = self.recovery.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.recovery.generation = self.recovery.generation.wrapping_add(1);
        self.recovery.loading = false;
        self.recovery.querying = false;
        self.recovery.details_loading = false;
        self.recovery.preview.loading_files = false;
        self.recovery.preview.loading_diff = false;
        self.recovery.search_changed = None;
        self.recovery.error = Some("读取已取消，保留上次可用结果".into());
    }

    pub(crate) fn recovery_search_changed(&mut self) {
        self.recovery.search_changed = Some(Instant::now());
    }

    pub(crate) fn recovery_branch_name_changed(&mut self) {
        if let Some(dialog) = self.recovery.dialog.as_mut() {
            dialog.error = None;
        }
    }

    pub(crate) fn tick_recovery(&mut self, cx: &mut Context<Self>) {
        self.refresh_recovery_branches();
        if self.recovery.show_recovery
            && self
                .recovery
                .search_changed
                .is_some_and(|at| at.elapsed() >= Duration::from_millis(250))
        {
            self.refresh_recovery_query(cx, false);
        }
    }

    fn refresh_recovery_branches(&mut self) {
        if !self.recovery.branches_refresh_pending || self.recovery.branches_refreshing || self.busy
        {
            return;
        }
        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        let Some(identity) = self
            .recovery
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.identity.clone())
        else {
            return;
        };
        let metadata_generation = self.repository_load_id;
        self.recovery.branches_refresh_pending = false;
        self.recovery.branches_refreshing = true;
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Short, move || {
            let result = run_read(|| {
                let repo = Repository::open(&repo_path)?;
                verify_identity(&service, &repo, &identity)?;
                service.branches(&repo)
            });
            send_ui_event(
                &tx,
                UiEvent::Recovery(RecoveryEvent::Branches {
                    tab_id,
                    repo_path,
                    metadata_generation,
                    result,
                }),
            );
        });
    }

    pub(crate) fn refresh_recovery_query(&mut self, cx: &mut Context<Self>, append: bool) {
        if self.recovery.loading || (append && self.recovery.querying) {
            return;
        }
        let Some(snapshot) = self.recovery.snapshot.clone() else {
            return;
        };
        let Some(key) = self.recovery_read_key() else {
            return;
        };
        let offset = if append {
            let Some(offset) = self.recovery.next_offset else {
                return;
            };
            offset
        } else {
            0
        };
        let requested_query = ReflogQuery {
            search: self.recovery.search.value.trim().to_string(),
            since: self.recovery.days.map(|days| {
                chrono::Utc::now()
                    .timestamp()
                    .saturating_sub(days.saturating_mul(86_400))
            }),
        };
        let Some(query) = self.recovery.page_query(requested_query, append) else {
            return;
        };
        self.recovery.query_id = self.recovery.query_id.wrapping_add(1);
        let query_id = self.recovery.query_id;
        self.recovery.querying = true;
        self.recovery.error = None;
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Long, move || {
            let result = run_read(|| Ok(snapshot.query(&query, offset, 100)));
            let event = match result {
                Ok(page) => RecoveryEvent::Queried {
                    key,
                    query_id,
                    offset,
                    page,
                },
                Err(error) => RecoveryEvent::QueryFailed {
                    key,
                    query_id,
                    error,
                },
            };
            send_ui_event(&tx, UiEvent::Recovery(event));
        });
        cx.notify();
    }

    fn load_recovery_page_objects(&mut self, offset: usize) {
        let Some(key) = self.recovery_read_key() else {
            return;
        };
        let Some(snapshot) = self.recovery.snapshot.clone() else {
            return;
        };
        let mut seen = HashSet::new();
        let oids: Vec<String> = self
            .recovery
            .entries
            .iter()
            .skip(offset)
            .flat_map(|entry| [&entry.old_oid, &entry.new_oid])
            .filter(|oid| {
                !self.recovery.availability.contains_key(*oid) && seen.insert((*oid).clone())
            })
            .cloned()
            .collect();
        if oids.is_empty() {
            return;
        }
        let service = self.service_for_tab(key.tab_id);
        let tx = self.tx.clone();
        let cancel = self.recovery.cancel.clone();
        self.tasks.spawn(TaskKind::Short, move || {
            let values = run_read(|| {
                let repo = Repository::open(&key.repo_path)?;
                verify_identity(&service, &repo, &snapshot.identity)?;
                let mut values = Vec::new();
                for oid in oids {
                    if cancel
                        .as_ref()
                        .is_some_and(|flag| flag.load(Ordering::Relaxed))
                    {
                        break;
                    }
                    let state = service
                        .recovery_commit_details(&repo, &oid)
                        .map(|details| details.state)
                        .unwrap_or_else(|error| RecoveryObjectState::Unreadable(error.to_string()));
                    values.push((oid, state));
                }
                Ok(values)
            })
            .unwrap_or_default();
            send_ui_event(
                &tx,
                UiEvent::Recovery(RecoveryEvent::Objects { key, values }),
            );
        });
    }

    pub(crate) fn select_recovery_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(entry) = self.recovery.entries.get(index) else {
            return;
        };
        self.recovery.selected = Some(entry.id.clone());
        self.recovery.side = RecoverySide::After;
        self.recovery.narrow_details = true;
        self.load_recovery_details(cx);
    }

    pub(crate) fn select_recovery_side(&mut self, side: RecoverySide, cx: &mut Context<Self>) {
        if self.recovery.side != side {
            self.recovery.side = side;
            self.load_recovery_details(cx);
        }
    }

    fn load_recovery_details(&mut self, cx: &mut Context<Self>) {
        self.recovery.selection_id = self.recovery.selection_id.wrapping_add(1);
        self.recovery.details = None;
        self.recovery.preview = RecoveryPreview::default();
        let Some(oid) = self.recovery.selected_oid().map(str::to_string) else {
            return;
        };
        let Some(key) = self.recovery_read_key() else {
            return;
        };
        let Some(snapshot) = self.recovery.snapshot.clone() else {
            return;
        };
        let selection_id = self.recovery.selection_id;
        self.recovery.details_loading = true;
        let service = self.service_for_tab(key.tab_id);
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Short, move || {
            let result = run_read(|| {
                let repo = Repository::open(&key.repo_path)?;
                verify_identity(&service, &repo, &snapshot.identity)?;
                service.recovery_commit_details(&repo, &oid)
            });
            send_ui_event(
                &tx,
                UiEvent::Recovery(RecoveryEvent::Details {
                    key,
                    selection_id,
                    oid,
                    result,
                }),
            );
        });
        cx.notify();
    }

    pub(crate) fn show_recovery_commit(&mut self, cx: &mut Context<Self>) {
        if !self
            .recovery
            .details
            .as_ref()
            .is_some_and(|details| details.state == RecoveryObjectState::Available)
        {
            return;
        }
        let Some(oid) = self.recovery.selected_oid().map(str::to_string) else {
            return;
        };
        let Some(key) = self.recovery_read_key() else {
            return;
        };
        let Some(snapshot) = self.recovery.snapshot.clone() else {
            return;
        };
        let selection_id = self.recovery.begin_preview();
        self.reset_uniform_scroll("recovery-file-list");
        self.reset_uniform_scroll("recovery-diff-scroll");
        let service = self.service_for_tab(key.tab_id);
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Short, move || {
            let result = run_read(|| {
                let repo = Repository::open(&key.repo_path)?;
                verify_identity(&service, &repo, &snapshot.identity)?;
                service.commit_files(&repo, &oid)
            });
            send_ui_event(
                &tx,
                UiEvent::Recovery(RecoveryEvent::Files {
                    key,
                    selection_id,
                    oid,
                    result,
                }),
            );
        });
        cx.notify();
    }

    pub(crate) fn select_recovery_file(&mut self, path: String) {
        if !self
            .recovery
            .preview
            .files
            .iter()
            .any(|file| file.path == path)
        {
            return;
        }
        let Some(oid) = self.recovery.selected_oid().map(str::to_string) else {
            return;
        };
        let Some(key) = self.recovery_read_key() else {
            return;
        };
        let Some(snapshot) = self.recovery.snapshot.clone() else {
            return;
        };
        let selection_id = self.recovery.selection_id;
        self.recovery.preview.request_id = self.recovery.preview.request_id.wrapping_add(1);
        let request_id = self.recovery.preview.request_id;
        self.recovery.preview.selected_file = Some(path.clone());
        self.recovery.preview.diff = None;
        self.recovery.preview.diff_syntax = None;
        self.recovery.preview.headers_expanded = false;
        self.recovery.preview.loading_diff = true;
        self.recovery.preview.error = None;
        self.reset_uniform_scroll("recovery-diff-scroll");
        let encoding = self.diff_encoding_choice_for_path(&key.repo_path);
        let full_context = self.full_file_view;
        let service = self.service_for_tab(key.tab_id);
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Short, move || {
            let result = run_read(|| {
                let repo = Repository::open(&key.repo_path)?;
                verify_identity(&service, &repo, &snapshot.identity)?;
                service.commit_file_diff(
                    &repo,
                    &oid,
                    std::path::Path::new(&path),
                    full_context,
                    encoding,
                )
            });
            send_ui_event(
                &tx,
                UiEvent::Recovery(RecoveryEvent::Diff {
                    key,
                    selection_id,
                    request_id,
                    oid,
                    path,
                    result,
                }),
            );
        });
    }

    pub(crate) fn reload_recovery_diff(&mut self) {
        if !self.recovery.preview.open {
            return;
        }
        if let Some(path) = self.recovery.preview.selected_file.clone() {
            self.select_recovery_file(path);
        }
    }

    pub(crate) fn open_recovery_branch_dialog(&mut self, cx: &mut Context<Self>) {
        if self.busy || self.recovery.operation.is_some() {
            return;
        }
        let Some(details) = self
            .recovery
            .details
            .as_ref()
            .filter(|details| details.state == RecoveryObjectState::Available)
        else {
            return;
        };
        let oid = details.oid.clone();
        let Some(entry) = self.recovery.selected_entry() else {
            return;
        };
        let source = entry.ref_name.clone();
        let Some(snapshot) = self.recovery.snapshot.as_ref() else {
            return;
        };
        let identity = snapshot.identity.clone();
        let side = self.recovery.side;
        self.close_popups();
        let name = format!(
            "recovery/{}-{}",
            chrono::Local::now().format("%Y%m%d-%H%M"),
            &oid[..oid.len().min(8)]
        );
        self.recovery.branch_name = self.recovery.branch_name.empty_sibling().with_value(name);
        self.recovery.dialog = Some(RecoveryDialog {
            identity,
            oid,
            source,
            side,
            error: None,
        });
        self.active_dialog = Some(DialogState::CreateRecoveryBranch);
        cx.notify();
    }

    pub(crate) fn close_recovery_dialog(&mut self) {
        if let Some(operation) = &self.recovery.operation {
            operation.cancel.store(true, Ordering::Relaxed);
        }
        self.recovery.dialog = None;
    }

    pub(crate) fn recovery_branch_error(&self) -> Option<String> {
        self.recovery.dialog.as_ref()?;
        let name = self.recovery.branch_name.value.trim();
        let Some(tab_id) = self.active_tab_id() else {
            return Some("仓库已关闭".into());
        };
        if let Err(error) = self
            .service_for_tab(tab_id)
            .validate_recovery_branch_name(name)
        {
            return Some(error.to_string());
        }
        let exists = self.snapshot.as_ref().is_some_and(|snapshot| {
            snapshot.branches.iter().any(|branch| {
                branch.kind == khaslana::BranchKind::Local
                    && if cfg!(windows) {
                        branch.name.eq_ignore_ascii_case(name)
                    } else {
                        branch.name == name
                    }
            })
        });
        if exists {
            return Some("分支名称已存在，请使用其他名称".into());
        }
        None
    }

    pub(crate) fn submit_recovery_branch(&mut self, cx: &mut Context<Self>) {
        if self.active_dialog != Some(DialogState::CreateRecoveryBranch)
            || self.busy
            || self.recovery.operation.is_some()
        {
            return;
        }
        if let Some(error) = self.recovery_branch_error() {
            if let Some(dialog) = self.recovery.dialog.as_mut() {
                dialog.error = Some(error);
            }
            cx.notify();
            return;
        }
        let Some(dialog) = self.recovery.dialog.clone() else {
            return;
        };
        let Some(tab_id) = self.active_tab_id() else {
            return;
        };
        let Some(repo_path) = self.repo_path.clone() else {
            return;
        };
        self.recovery.next_operation_id = self.recovery.next_operation_id.wrapping_add(1);
        let operation_id = self.recovery.next_operation_id;
        let metadata_generation = self.repository_load_id;
        let read_generation = self.recovery.generation;
        let selection_id = self.recovery.selection_id;
        if let Some(dialog) = self.recovery.dialog.as_mut() {
            dialog.error = None;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        self.recovery.operation = Some(RecoveryOperation {
            id: operation_id,
            cancel: cancel.clone(),
        });
        let request = CreateRecoveryBranchRequest {
            identity: dialog.identity,
            target_oid: dialog.oid,
            branch_name: self.recovery.branch_name.value.trim().to_string(),
        };
        self.status = "正在创建恢复分支".into();
        let service = self.service_for_tab(tab_id);
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Long, move || {
            let result = run_read(|| {
                if cancel.load(Ordering::Relaxed) {
                    return Err(GitError::Message("已取消创建，未执行引用写入".into()));
                }
                let repo = Repository::open(&repo_path)?;
                if cancel.load(Ordering::Relaxed) {
                    return Err(GitError::Message("已取消创建，未执行引用写入".into()));
                }
                service.create_recovery_branch(&repo, &request)
            });
            // 创建成功后再做独立元数据刷新；刷新异常不能抹去已经发生的创建。
            let branches = if result.is_ok() {
                run_read(|| {
                    let repo = Repository::open(&repo_path)?;
                    verify_identity(&service, &repo, &request.identity)?;
                    service.branches(&repo)
                })
            } else {
                Ok(Vec::new())
            };
            send_ui_event(
                &tx,
                UiEvent::Recovery(RecoveryEvent::Created {
                    tab_id,
                    repo_path,
                    operation_id,
                    metadata_generation,
                    read_generation,
                    selection_id,
                    result,
                    branches,
                }),
            );
        });
        cx.notify();
    }

    pub(crate) fn handle_recovery_event(&mut self, event: RecoveryEvent, cx: &mut Context<Self>) {
        match event {
            RecoveryEvent::Loaded { key, result } => {
                self.with_tab_context(key.tab_id, |this| {
                    if !key.matches(this.active_tab_state()) {
                        return;
                    }
                    this.recovery.loading = false;
                    match result {
                        Ok((sources, snapshot)) => {
                            this.recovery.sources = Some(sources);
                            this.recovery.snapshot = Some(snapshot);
                            this.refresh_recovery_query(cx, false);
                        }
                        Err(error) => {
                            this.recovery.error =
                                Some(format!("读取恢复记录失败：{error}。保留上次可用结果。"))
                        }
                    }
                });
            }
            RecoveryEvent::Queried {
                key,
                query_id,
                offset,
                page,
            } => {
                self.with_tab_context(key.tab_id, |this| {
                    if !key.matches(this.active_tab_state()) || query_id != this.recovery.query_id {
                        return;
                    }
                    this.recovery.querying = false;
                    this.recovery.total = page.total;
                    this.recovery.next_offset = page.next_offset;
                    if offset == 0 {
                        this.reset_uniform_scroll("recovery-record-list");
                        this.recovery.entries = page.entries;
                    } else {
                        this.recovery.entries.extend(page.entries);
                    }
                    let selected_exists = this.recovery.selected_entry().is_some();
                    if !selected_exists || offset == 0 {
                        let selected_index = this.recovery.selected.as_ref().and_then(|id| {
                            this.recovery
                                .entries
                                .iter()
                                .position(|entry| &entry.id == id)
                        });
                        this.recovery.selected = None;
                        this.recovery.selection_id = this.recovery.selection_id.wrapping_add(1);
                        this.recovery.details = None;
                        this.recovery.details_loading = false;
                        this.recovery.preview = RecoveryPreview::default();
                        if !this.recovery.entries.is_empty() {
                            let narrow_details = this.recovery.narrow_details;
                            this.select_recovery_entry(selected_index.unwrap_or(0), cx);
                            this.recovery.narrow_details = narrow_details;
                        }
                    }
                    this.load_recovery_page_objects(offset);
                });
            }
            RecoveryEvent::Branches {
                tab_id,
                repo_path,
                metadata_generation,
                result,
            } => {
                self.with_tab_context(tab_id, |this| {
                    if this.repo_path.as_ref() != Some(&repo_path) {
                        return;
                    }
                    this.recovery.branches_refreshing = false;
                    if !metadata_is_current(metadata_generation, this.repository_load_id, this.busy)
                    {
                        this.recovery.branches_refresh_pending = true;
                        return;
                    }
                    match result {
                        Ok(branches) => {
                            if let Some(snapshot) = this.snapshot.as_mut() {
                                snapshot.branches = branches;
                            }
                            this.history_refs_cache = None;
                        }
                        Err(error) => {
                            this.last_error =
                                Some(format!("恢复分支已创建，分支列表刷新失败：{error}"));
                        }
                    }
                });
            }
            RecoveryEvent::QueryFailed {
                key,
                query_id,
                error,
            } => {
                self.with_tab_context(key.tab_id, |this| {
                    if key.matches(this.active_tab_state()) && query_id == this.recovery.query_id {
                        this.recovery.querying = false;
                        this.recovery.error = Some(error);
                    }
                });
            }
            RecoveryEvent::Objects { key, values } => {
                self.with_tab_context(key.tab_id, |this| {
                    if key.matches(this.active_tab_state()) {
                        this.recovery.availability.extend(values);
                    }
                });
            }
            RecoveryEvent::Details {
                key,
                selection_id,
                oid,
                result,
            } => {
                self.with_tab_context(key.tab_id, |this| {
                    if !key.matches(this.active_tab_state())
                        || selection_id != this.recovery.selection_id
                        || this.recovery.selected_oid() != Some(&oid)
                    {
                        return;
                    }
                    this.recovery.details_loading = false;
                    this.recovery.details =
                        Some(
                            result.unwrap_or_else(|error| khaslana::RecoveryCommitDetails {
                                oid,
                                state: RecoveryObjectState::Unreadable(error),
                                commit: None,
                            }),
                        );
                });
            }
            RecoveryEvent::Files {
                key,
                selection_id,
                oid,
                result,
            } => {
                self.with_tab_context(key.tab_id, |this| {
                    if !key.matches(this.active_tab_state())
                        || selection_id != this.recovery.selection_id
                        || this.recovery.selected_oid() != Some(&oid)
                        || !this.recovery.preview.open
                    {
                        return;
                    }
                    this.recovery.preview.loading_files = false;
                    match result {
                        Ok(files) => {
                            let first = files.first().map(|file| file.path.clone());
                            this.recovery.preview.files = files;
                            if let Some(path) = first {
                                this.select_recovery_file(path);
                            }
                        }
                        Err(error) => this.recovery.preview.error = Some(error),
                    }
                });
            }
            RecoveryEvent::Diff {
                key,
                selection_id,
                request_id,
                oid,
                path,
                result,
            } => {
                self.with_tab_context(key.tab_id, |this| {
                    if !key.matches(this.active_tab_state())
                        || selection_id != this.recovery.selection_id
                        || request_id != this.recovery.preview.request_id
                        || !this.recovery.preview.open
                        || this.recovery.selected_oid() != Some(&oid)
                        || this.recovery.preview.selected_file.as_ref() != Some(&path)
                    {
                        return;
                    }
                    this.recovery.preview.loading_diff = false;
                    match result {
                        Ok(diff) => {
                            this.recovery.preview.diff = Some(Arc::new(diff));
                            this.recovery.preview.diff_syntax = None;
                            this.schedule_syntax_highlight(SyntaxSlot::RecoveryDiff);
                        }
                        Err(error) => {
                            if this.full_file_view
                                && error.contains(khaslana::FULL_FILE_TOO_LARGE_MESSAGE)
                            {
                                this.full_file_view = false;
                                this.status = "文件过大，已回退到差异视图".into();
                                this.reload_recovery_diff();
                            } else {
                                this.recovery.preview.error = Some(error);
                            }
                        }
                    }
                });
            }
            RecoveryEvent::Created {
                tab_id,
                repo_path,
                operation_id,
                metadata_generation,
                read_generation,
                selection_id,
                result,
                branches,
            } => {
                let dialog_visible = self.active_tab_id() == Some(tab_id)
                    && self.active_dialog == Some(DialogState::CreateRecoveryBranch);
                let mut feedback = None;
                let mut close = false;
                self.with_tab_context(tab_id, |this| {
                    if this.repo_path.as_ref() != Some(&repo_path)
                        || !this.recovery.finish_operation(operation_id)
                    {
                        return;
                    }
                    let current_metadata = metadata_is_current(
                        metadata_generation,
                        this.repository_load_id,
                        this.busy,
                    );
                    let current_recovery = this
                        .recovery
                        .context_is_current(read_generation, selection_id)
                        && this.main_mode == crate::MainMode::History
                        && this.recovery.show_recovery;
                    match result {
                        Ok(outcome) => {
                            let mut warning = outcome.verification_warning;
                            if current_metadata {
                                match branches {
                                    Ok(branches) => {
                                        if let Some(snapshot) = this.snapshot.as_mut() {
                                            snapshot.branches = branches;
                                        }
                                    }
                                    Err(error) => {
                                        warning = Some(format!(
                                            "{}分支列表刷新失败：{error}",
                                            warning
                                                .map(|text| format!("{text}；"))
                                                .unwrap_or_default()
                                        ))
                                    }
                                }
                            } else {
                                this.recovery.branches_refresh_pending = true;
                            }
                            if current_metadata && current_recovery {
                                this.selected_branch = Some(outcome.branch_name.clone());
                            }
                            this.recovery.last_created = Some(outcome.branch_name.clone());
                            this.history_refs_cache = None;
                            this.recovery.dialog = None;
                            let message = format!(
                                "恢复分支 {} 已创建，当前分支未切换{}",
                                outcome.branch_name,
                                warning
                                    .as_ref()
                                    .map(|text| format!("；{text}"))
                                    .unwrap_or_default()
                            );
                            if current_metadata {
                                this.status = message.clone();
                                this.last_error = warning.clone();
                            }
                            feedback = Some((message, warning.is_some(), false));
                            close = true;
                            if current_metadata && current_recovery {
                                this.load_recovery(cx);
                            }
                        }
                        Err(error) => {
                            if let Some(dialog) = this.recovery.dialog.as_mut() {
                                dialog.error = Some(error.clone());
                            }
                            if current_metadata {
                                this.status = "恢复分支未创建".into();
                                this.last_error = Some(error.clone());
                            }
                            feedback = Some((format!("创建恢复分支失败：{error}"), false, true));
                        }
                    }
                });
                if close && dialog_visible {
                    self.active_dialog = None;
                    self.request_overlay_focus_restore();
                }
                if let Some((message, warning, error)) = feedback {
                    if error {
                        self.notify_error(message, cx);
                    } else if warning {
                        self.notify_warning(message, cx);
                    } else {
                        self.notify_success(message, cx);
                    }
                }
            }
        }
        cx.notify();
    }
}
