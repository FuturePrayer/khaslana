//! Windows 任务栏按钮进度叠加层（ITaskbarList3）。
//!
//! 与资源管理器拷贝文件、IDEA 编译同一原生能力：shell 在任务栏按钮上
//! 绘制绿色确定态进度条、不定态扫动或红色错误态，样式由系统统一提供。
//! 全部长任务聚合为单一状态（优先级：克隆 > 更新下载 > 活动 tab 操作 >
//! 代码索引），数值变化去重后才发 COM 调用。主线程 STA 已由 gpui 的
//! OleInitialize 就绪，这里不得重复初始化 COM。非 Windows 平台整体 no-op。

use crate::*;

/// 任务栏进度状态（跨平台纯数据，聚合逻辑可单测）。
#[cfg(any(windows, test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TaskbarProgressState {
    /// 无进度（TBPF_NOPROGRESS）。
    None,
    /// 不定态扫动（TBPF_INDETERMINATE）：有工作但无确定总数。
    Indeterminate,
    /// 确定态绿色进度条（TBPF_NORMAL + SetProgressValue）。
    Determinate { done: u64, total: u64 },
    /// 红色错误态（TBPF_ERROR）：克隆失败待重试期间。
    Error,
}

/// 聚合输入：全部读自 UI 层现成字段，不新增事件形状。
#[cfg(any(windows, test))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TaskbarProgressInputs {
    pub clone_running: bool,
    pub clone_failed: bool,
    pub clone_done: u64,
    pub clone_total: u64,
    pub update_downloading: bool,
    pub update_done: u64,
    pub update_total: u64,
    /// 活动 tab 的网络/工作流等操作（has_active_loading），只有不定态。
    pub active_tab_loading: bool,
    pub index_running: bool,
    pub index_done: u64,
    pub index_total: u64,
}

/// 聚合优先级：克隆失败红 > 克隆 > 更新下载 > 活动 tab 扫动 > 索引。
/// 取消中的克隆不算运行/失败态，向下落穿到其他进度源。
#[cfg(any(windows, test))]
pub(crate) fn aggregate_taskbar_progress(inputs: &TaskbarProgressInputs) -> TaskbarProgressState {
    if inputs.clone_failed {
        return TaskbarProgressState::Error;
    }
    if inputs.clone_running {
        return determinate_or_marquee(inputs.clone_done, inputs.clone_total);
    }
    if inputs.update_downloading {
        return determinate_or_marquee(inputs.update_done, inputs.update_total);
    }
    if inputs.active_tab_loading {
        return TaskbarProgressState::Indeterminate;
    }
    if inputs.index_running {
        return determinate_or_marquee(inputs.index_done, inputs.index_total);
    }
    TaskbarProgressState::None
}

#[cfg(any(windows, test))]
fn determinate_or_marquee(done: u64, total: u64) -> TaskbarProgressState {
    if total > 0 {
        TaskbarProgressState::Determinate { done: done.min(total), total }
    } else {
        TaskbarProgressState::Indeterminate
    }
}

/// 克隆源映射：取消中的任务不算运行/失败态（落穿到其他进度源）。
#[cfg(any(windows, test))]
fn clone_source(clone: Option<&repository_clone::CloneTask>) -> (bool, bool, u64, u64) {
    match clone {
        Some(task) if !task.cancelled => (
            task.phase == repository_clone::ClonePhase::Running,
            task.phase == repository_clone::ClonePhase::Failed,
            task.received_objects as u64,
            task.total_objects as u64,
        ),
        _ => (false, false, 0, 0),
    }
}

#[cfg(any(windows, test))]
impl RepositoryView {
    pub(crate) fn taskbar_progress_inputs(&self) -> TaskbarProgressInputs {
        let (clone_running, clone_failed, clone_done, clone_total) =
            clone_source(self.clone_task.as_ref());
        TaskbarProgressInputs {
            clone_running,
            clone_failed,
            clone_done,
            clone_total,
            update_downloading: self.update_downloading,
            update_done: self.update_download_done,
            update_total: self.update_download_total,
            active_tab_loading: self.has_active_loading(),
            index_running: self.code_index_task.is_some(),
            index_done: self.code_index_progress_done as u64,
            index_total: self.code_index_progress_total as u64,
        }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::TaskbarProgressState;
    use gpui::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
    use windows::Win32::UI::Shell::{
        ITaskbarList3, TaskbarList, TBPF_ERROR, TBPF_INDETERMINATE, TBPF_NOPROGRESS, TBPF_NORMAL,
    };

    /// 任务栏进度控制器：与托盘控制器同生命周期，由 RepositoryView 在 UI 线程持有。
    pub(crate) struct TaskbarController {
        list: ITaskbarList3,
        hwnd: HWND,
        /// 上次成功发送的状态：去重，避免每帧/每 tick 重复 COM 调用。
        last: Option<TaskbarProgressState>,
    }

    impl TaskbarController {
        pub(crate) fn new(window: &Window) -> Result<Self, String> {
            let handle = HasWindowHandle::window_handle(window).map_err(|err| format!("{err}"))?;
            let RawWindowHandle::Win32(handle) = handle.as_raw() else {
                return Err("current window is not a Win32 window".to_string());
            };
            let hwnd = HWND(handle.hwnd.get() as *mut _);
            // SAFETY：主线程 STA 已由 gpui OleInitialize 初始化；句柄来自当前
            // 存活的主窗口，创建与后续调用都在其 UI 线程。
            let list: ITaskbarList3 =
                unsafe { CoCreateInstance(&TaskbarList, None, CLSCTX_INPROC_SERVER) }
                    .map_err(|err| format!("{err}"))?;
            unsafe { list.HrInit() }.map_err(|err| format!("{err}"))?;
            Ok(Self { list, hwnd, last: None })
        }

        /// 清空去重缓存：托盘隐藏后任务栏按钮被 shell 重建，恢复时需重发进度。
        pub(crate) fn invalidate(&mut self) {
            self.last = None;
        }

        pub(crate) fn apply(&mut self, state: TaskbarProgressState) {
            if self.last == Some(state) {
                return;
            }
            // SAFETY：hwnd 为同一视图持有的主窗口；COM 调用发生在 UI 线程。
            let result = unsafe {
                match state {
                    TaskbarProgressState::None => {
                        self.list.SetProgressState(self.hwnd, TBPF_NOPROGRESS)
                    }
                    TaskbarProgressState::Indeterminate => {
                        self.list.SetProgressState(self.hwnd, TBPF_INDETERMINATE)
                    }
                    TaskbarProgressState::Error => {
                        self.list.SetProgressState(self.hwnd, TBPF_ERROR)
                    }
                    TaskbarProgressState::Determinate { done, total } => self
                        .list
                        .SetProgressState(self.hwnd, TBPF_NORMAL)
                        .and_then(|()| self.list.SetProgressValue(self.hwnd, done, total.max(1))),
                }
            };
            if let Err(err) = result {
                // shell 瞬态失败不缓存，下次同步重试；debug 级避免 tick 节拍刷屏。
                tracing::debug!("taskbar progress update failed: {err}");
                return;
            }
            self.last = Some(state);
        }
    }
}

#[cfg(windows)]
pub(crate) use windows_impl::TaskbarController;

impl RepositoryView {
    /// 窗口创建回调挂接（与 attach_window_to_tray 同时机）。
    #[cfg(windows)]
    pub(crate) fn attach_taskbar_progress(&mut self, window: &Window) {
        match windows_impl::TaskbarController::new(window) {
            Ok(controller) => self.taskbar = Some(controller),
            Err(err) => {
                // 装饰性能力：无 shell 环境（如 Server Core）静默降级，不弹错误。
                tracing::warn!("taskbar progress unavailable: {err}");
                self.taskbar = None;
            }
        }
    }

    #[cfg(not(windows))]
    pub(crate) fn attach_taskbar_progress(&mut self, _window: &Window) {}

    /// 事件批处理末尾与 UiTick 兜底节拍调用；去重保证 COM 只在变化时发送。
    #[cfg(windows)]
    pub(crate) fn sync_taskbar_progress(&mut self) {
        let state = aggregate_taskbar_progress(&self.taskbar_progress_inputs());
        if let Some(controller) = self.taskbar.as_mut() {
            controller.apply(state);
        }
    }

    #[cfg(not(windows))]
    pub(crate) fn sync_taskbar_progress(&mut self) {}

    #[cfg(windows)]
    pub(crate) fn invalidate_taskbar_progress(&mut self) {
        if let Some(controller) = self.taskbar.as_mut() {
            controller.invalidate();
        }
    }

    #[cfg(not(windows))]
    pub(crate) fn invalidate_taskbar_progress(&mut self) {}
}

#[cfg(test)]
#[path = "tests/taskbar_progress.rs"]
mod tests;
