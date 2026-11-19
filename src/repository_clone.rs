//! 克隆任务编排：全局单任务，克隆期间不创建仓库 tab，成功后才打开。
//!
//! 进度只进克隆弹窗（`dialog_view::render_clone_dialog` 按任务状态渲染），
//! 不写任何 tab 的 status，避免串到工作流 Console 或原仓库状态栏。
//! 失败留在弹窗内供重试；关闭弹窗（取消/放弃）UI 停留在原仓库。
//! 任务身份与互斥参照 AI 思考任务（`ai_view::start_ai_thinking_task`）：
//! 运行态占位，终态事件按 id 寻址，迟到事件静默丢弃。

use crate::*;
// 复用 repository_operations 挂载的操作反馈资源，避免重复编译同一文件。
use crate::repository_operations::feedback_messages;
use khaslana::ui_i18n::{encode, format, text_in, format_in};
use khaslana::{OperationEvent, ProgressEmitter, RepoPath};

#[cfg(test)]
#[path = "tests/clone_task.rs"]
mod tests;

/// 克隆任务阶段：运行中（含已取消但终态事件未到达）/ 失败待重试。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClonePhase {
    Running,
    Failed,
}

pub(crate) struct CloneTask {
    pub(crate) id: u64,
    pub(crate) url: String,
    pub(crate) path: PathBuf,
    pub(crate) options: khaslana::CloneOptions,
    pub(crate) phase: ClonePhase,
    /// 编码后的阶段文案（正在克隆/已接收对象/检出文件/同步子模块）。
    pub(crate) progress_message: String,
    /// 对象传输进度；total=0 表示当前阶段没有确定态进度，只显示文案。
    pub(crate) received_objects: usize,
    pub(crate) total_objects: usize,
    pub(crate) error: Option<String>,
    /// 用户已请求取消：终态事件据此静默收尾，不弹错误。
    pub(crate) cancelled: bool,
    pub(crate) cancel_flag: Arc<AtomicBool>,
    /// 失败时刻目标目录是否存在（残留）：重试只清理本任务产生的残留，
    /// 失败后外部新建的同名目录受 TARGET_EXISTS 保护，不得删除。
    pub(crate) leftover_at_failure: bool,
}

/// 重试前的残留清理决策（纯函数，便于回归测试）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CloneRetryCleanup {
    /// 目录是本任务失败留下的残留：重试前删除。
    RemoveLeftover,
    /// 目录不存在：直接克隆。
    NoCleanup,
    /// 失败时不存在、现在却存在：外部新建内容，禁止删除，按目标已存在报错。
    BlockTargetExists,
}

pub(crate) fn clone_retry_cleanup(leftover_at_failure: bool, exists_now: bool) -> CloneRetryCleanup {
    match (leftover_at_failure, exists_now) {
        (_, false) => CloneRetryCleanup::NoCleanup,
        (true, true) => CloneRetryCleanup::RemoveLeftover,
        (false, true) => CloneRetryCleanup::BlockTargetExists,
    }
}

/// 克隆进度桥：与 `TabProgress` 不同，不写 tab status，数字保留给进度条。
#[derive(Clone)]
struct CloneProgressEmitter {
    tx: Sender<UiEvent>,
    task_id: u64,
}

impl ProgressEmitter for CloneProgressEmitter {
    fn emit(&self, event: OperationEvent) {
        let (message, done, total) = match event {
            OperationEvent::Started(message)
            | OperationEvent::Progress(message)
            | OperationEvent::Finished(message) => (message, 0, 0),
            OperationEvent::TransferProgress { received_objects, total_objects } => (
                format(
                    khaslana::types::domain_ui_resources::RECEIVED_OBJECTS,
                    &[
                        ("value0", &received_objects.to_string()),
                        ("value1", &total_objects.to_string()),
                    ],
                ),
                received_objects,
                total_objects,
            ),
        };
        send_ui_event(
            &self.tx,
            UiEvent::CloneProgress { task_id: self.task_id, message, done, total },
        );
    }
}

/// 尽力清理失败/取消留下的半成品目录。
///
/// 调用前提：目标目录在首次发起克隆前不存在（表单 TARGET_EXISTS 校验），
/// 因此现存的目录只能是本任务产生的残留。这不是对已打开仓库的工作区操作，
/// 不走 worktree_compat 包装；删除失败（如 Windows 文件占用）仅记录日志，
/// 重试路径会再次尝试并把失败原因报给用户。
fn cleanup_partial_clone(path: &Path) {
    if path.exists()
        && let Err(err) = std::fs::remove_dir_all(path)
    {
        tracing::warn!("清理克隆残留目录失败 {}: {err}", path.display());
    }
}

impl RepositoryView {
    /// 克隆专用服务：凭据/主机信任请求不绑定 tab（None），代理沿用全局设置。
    fn service_for_clone(&self, task_id: u64, cancel_flag: Arc<AtomicBool>) -> GitService {
        GitService::new(
            Arc::new(TabCredentialProvider::new(
                self.credential_store.clone(),
                self.storage.clone(),
                self.remote_credential_bindings.clone(),
                self.tx.clone(),
                None,
                self.proxy_settings.clone(),
            )),
            Arc::new(CloneProgressEmitter { tx: self.tx.clone(), task_id }),
        )
        .with_proxy_settings(self.proxy_settings.clone())
        .with_host_trust_provider(Arc::new(repository_ssh_ui::TabHostTrustProvider {
            tx: self.tx.clone(),
            tab_id: None,
        }))
        .with_cancel_flag(cancel_flag)
    }

    /// 发起克隆（表单校验通过后调用）。任务槽占用期间拒绝第二个克隆。
    pub(crate) fn start_clone_task(
        &mut self,
        url: String,
        path: PathBuf,
        options: khaslana::CloneOptions,
    ) {
        if self.clone_task.is_some() {
            self.last_error = Some(encode(feedback_messages::CLONE_ALREADY_RUNNING, &[]));
            return;
        }
        let id = self.next_clone_task_id;
        self.next_clone_task_id = self.next_clone_task_id.wrapping_add(1).max(1);
        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.clone_task = Some(CloneTask {
            id,
            url: url.clone(),
            path: path.clone(),
            options,
            phase: ClonePhase::Running,
            progress_message: encode(feedback_messages::CLONING, &[]),
            received_objects: 0,
            total_objects: 0,
            error: None,
            cancelled: false,
            cancel_flag: cancel_flag.clone(),
            leftover_at_failure: false,
        });
        self.last_error = None;
        // 克隆弹窗保持打开，渲染随任务状态切到进度态；不 close_popups。
        self.spawn_clone_attempt(id, url, path, options, cancel_flag, false);
    }

    /// 失败后就地重试：分配新任务 id（旧_attempt 的迟到事件按 id 丢弃），
    /// 先按残留决策清理目标目录再重新克隆。
    pub(crate) fn retry_clone(&mut self) {
        let Some(task) = self.clone_task.as_ref() else {
            return;
        };
        if task.phase != ClonePhase::Failed || task.cancelled {
            return;
        }
        let url = task.url.clone();
        let path = task.path.clone();
        let options = task.options;
        let cleanup = clone_retry_cleanup(task.leftover_at_failure, path.exists());
        if cleanup == CloneRetryCleanup::BlockTargetExists {
            if let Some(task) = self.clone_task.as_mut() {
                task.error = Some(encode(feedback_messages::TARGET_EXISTS, &[]));
            }
            return;
        }
        let id = self.next_clone_task_id;
        self.next_clone_task_id = self.next_clone_task_id.wrapping_add(1).max(1);
        let cancel_flag = Arc::new(AtomicBool::new(false));
        if let Some(task) = self.clone_task.as_mut() {
            task.id = id;
            task.phase = ClonePhase::Running;
            task.progress_message = encode(feedback_messages::CLONING, &[]);
            task.received_objects = 0;
            task.total_objects = 0;
            task.error = None;
            task.cancelled = false;
            task.cancel_flag = cancel_flag.clone();
            task.leftover_at_failure = false;
        }
        self.last_error = None;
        self.spawn_clone_attempt(
            id,
            url,
            path,
            options,
            cancel_flag,
            cleanup == CloneRetryCleanup::RemoveLeftover,
        );
    }

    /// 取消克隆（进行中关闭弹窗/Esc/取消按钮）。
    ///
    /// 任务槽保留到终态事件：libgit2 只在进度回调检查点中止，若立即释放槽
    /// 并允许新克隆，两个任务会竞争同一目标目录。取消后 UI 停留在原仓库。
    pub(crate) fn cancel_clone(&mut self) {
        if let Some(task) = self.clone_task.as_mut() {
            task.cancelled = true;
            task.cancel_flag.store(true, Ordering::Relaxed);
        }
        if self.active_dialog == Some(DialogState::CloneRepo) {
            self.active_dialog = None;
            self.request_overlay_focus_restore();
        }
    }

    /// 失败后用户关闭弹窗放弃重试：释放任务槽并尽力清理残留目录。
    /// 弹窗本体的关闭（置空 active_dialog、焦点恢复）由 close_dialog 常规路径完成。
    pub(crate) fn dismiss_failed_clone(&mut self) {
        let Some(task) = self.clone_task.take() else {
            return;
        };
        if task.leftover_at_failure && task.path.exists() {
            let path = task.path;
            self.tasks.spawn(TaskKind::Short, move || cleanup_partial_clone(&path));
        }
    }

    fn spawn_clone_attempt(
        &mut self,
        task_id: u64,
        url: String,
        path: PathBuf,
        options: khaslana::CloneOptions,
        cancel_flag: Arc<AtomicBool>,
        cleanup_leftover: bool,
    ) {
        let service = self.service_for_clone(task_id, cancel_flag.clone());
        let tx = self.tx.clone();
        // 错误文案语言在任务启动时快照，后台不重读全局语言偏好。
        let language = self.ui_language;
        self.tasks.spawn(TaskKind::Long, move || {
            if cleanup_leftover && path.exists() {
                if let Err(err) = std::fs::remove_dir_all(&path) {
                    let error = format_in(
                        language,
                        feedback_messages::CLONE_CLEANUP_FAILED,
                        &[("error", &err.to_string())],
                    );
                    send_ui_event(&tx, UiEvent::CloneFailed { task_id, error, cancelled: false });
                    return;
                }
            }
            let result =
                service.clone_repo_with_options(&url, &RepoPath::new(path.clone()), options);
            // 取消与否按 flag 判定，不从错误译文反推业务分支。
            if cancel_flag.load(Ordering::Relaxed) {
                cleanup_partial_clone(&path);
                let error = text_in(
                    language,
                    khaslana::types::domain_ui_resources::CLONE_CANCELLED,
                )
                .to_string();
                send_ui_event(&tx, UiEvent::CloneFailed { task_id, error, cancelled: true });
                return;
            }
            match result {
                Ok(snapshot) => {
                    send_ui_event(&tx, UiEvent::CloneFinished { task_id, snapshot });
                }
                Err(err) => {
                    cleanup_partial_clone(&path);
                    send_ui_event(
                        &tx,
                        UiEvent::CloneFailed { task_id, error: err.display_in(language), cancelled: false },
                    );
                }
            }
        });
    }

    pub(crate) fn handle_clone_progress(
        &mut self,
        task_id: u64,
        message: String,
        done: usize,
        total: usize,
    ) {
        let Some(task) = self.clone_task.as_mut() else {
            return;
        };
        if task.id != task_id || task.phase != ClonePhase::Running {
            return;
        }
        task.progress_message = message;
        task.received_objects = done;
        task.total_objects = total;
    }

    /// 克隆成功：此时才创建并激活仓库 tab，随后复用操作完成事件链
    /// （快照应用、历史刷新、全量状态补全、分支同步、完成通知）。
    pub(crate) fn handle_clone_finished(
        &mut self,
        task_id: u64,
        snapshot: RepositorySnapshot,
        cx: &mut Context<Self>,
    ) {
        let Some(task) = self.clone_task.as_ref() else {
            return;
        };
        if task.id != task_id {
            return;
        }
        let path = task.path.clone();
        self.clone_task = None;
        if self.active_dialog == Some(DialogState::CloneRepo) {
            self.close_dialog();
        }
        // 记录最近打开时间，供仓库切换下拉排序（与 open_repo/activate_tab 一致）。
        let _ = self.storage.upsert_recent_repo(&path);
        let tab_id = self.ensure_tab_for_path(path);
        self.handle_ui_event(
            UiEvent::OperationFinished {
                tab_id: Some(tab_id),
                message: encode(crate::operation_ui_resources::CLONED, &[]),
                snapshot: Some(snapshot),
                diff: None,
            },
            cx,
        );
    }

    pub(crate) fn handle_clone_failed(
        &mut self,
        task_id: u64,
        error: String,
        cancelled: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(task) = self.clone_task.as_mut() else {
            return;
        };
        if task.id != task_id {
            return;
        }
        if cancelled || task.cancelled {
            // 用户取消：弹窗已关闭，静默释放任务槽，不弹错误、不建 tab。
            self.clone_task = None;
            return;
        }
        task.phase = ClonePhase::Failed;
        task.error = Some(error);
        task.leftover_at_failure = task.path.exists();
        // 正常路径弹窗保持打开；若已被其他流程置空则重新打开承载错误与重试。
        // SSH 主机验证弹窗（VerifySshHost）不覆盖——它响应后会恢复克隆弹窗。
        if self.active_dialog.is_none() {
            self.active_dialog = Some(DialogState::CloneRepo);
        }
        cx.notify();
    }
}
