use super::*;
use crate::repository_clone::{ClonePhase, CloneTask};
use std::sync::atomic::AtomicBool;

fn inputs() -> TaskbarProgressInputs {
    TaskbarProgressInputs::default()
}

fn clone_task(phase: ClonePhase, cancelled: bool, done: usize, total: usize) -> CloneTask {
    CloneTask {
        id: 1,
        url: "https://example.invalid/repo.git".into(),
        path: PathBuf::from("target"),
        options: khaslana::CloneOptions::default(),
        phase,
        progress_message: String::new(),
        received_objects: done,
        total_objects: total,
        error: None,
        cancelled,
        cancel_flag: Arc::new(AtomicBool::new(false)),
        leftover_at_failure: false,
    }
}

#[test]
fn no_active_task_clears_taskbar_progress() {
    assert_eq!(aggregate_taskbar_progress(&inputs()), TaskbarProgressState::None);
}

#[test]
fn clone_transfer_is_determinate_and_checkout_phase_is_marquee() {
    let mut i = inputs();
    i.clone_running = true;
    i.clone_done = 30;
    i.clone_total = 100;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 30, total: 100 }
    );
    // 检出/子模块阶段没有确定总数：不定态扫动。
    i.clone_done = 0;
    i.clone_total = 0;
    assert_eq!(aggregate_taskbar_progress(&i), TaskbarProgressState::Indeterminate);
}

#[test]
fn clone_done_is_clamped_to_total() {
    let mut i = inputs();
    i.clone_running = true;
    i.clone_done = 120;
    i.clone_total = 100;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 100, total: 100 }
    );
}

#[test]
fn clone_failure_shows_error_even_with_other_sources_active() {
    let mut i = inputs();
    i.clone_failed = true;
    i.update_downloading = true;
    i.update_done = 1;
    i.update_total = 2;
    i.active_tab_loading = true;
    assert_eq!(aggregate_taskbar_progress(&i), TaskbarProgressState::Error);
}

#[test]
fn clone_source_maps_phase_and_drops_cancelled_task() {
    let running = clone_task(ClonePhase::Running, false, 3, 10);
    assert_eq!(clone_source(Some(&running)), (true, false, 3, 10));
    let failed = clone_task(ClonePhase::Failed, false, 0, 0);
    assert_eq!(clone_source(Some(&failed)), (false, true, 0, 0));
    // 取消中的克隆不占用任务栏，落穿到其他进度源。
    let cancelled = clone_task(ClonePhase::Running, true, 3, 10);
    assert_eq!(clone_source(Some(&cancelled)), (false, false, 0, 0));
    assert_eq!(clone_source(None), (false, false, 0, 0));

    let mut i = inputs();
    i.index_running = true;
    i.index_done = 5;
    i.index_total = 8;
    let (running_flag, failed_flag, done, total) = clone_source(Some(&cancelled));
    i.clone_running = running_flag;
    i.clone_failed = failed_flag;
    i.clone_done = done;
    i.clone_total = total;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 5, total: 8 }
    );
}

#[test]
fn update_download_determinate_and_unknown_total_marquee() {
    let mut i = inputs();
    i.update_downloading = true;
    i.update_done = 1_048_576;
    i.update_total = 4_194_304;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 1_048_576, total: 4_194_304 }
    );
    i.update_total = 0;
    assert_eq!(aggregate_taskbar_progress(&i), TaskbarProgressState::Indeterminate);
}

#[test]
fn active_tab_operation_is_marquee_and_outranks_index() {
    let mut i = inputs();
    i.active_tab_loading = true;
    i.index_running = true;
    i.index_done = 5;
    i.index_total = 8;
    assert_eq!(aggregate_taskbar_progress(&i), TaskbarProgressState::Indeterminate);
}

#[test]
fn index_progress_is_determinate() {
    let mut i = inputs();
    i.index_running = true;
    i.index_done = 5;
    i.index_total = 8;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 5, total: 8 }
    );
}

#[test]
fn priority_order_clone_update_active_tab_index() {
    let mut i = inputs();
    i.clone_running = true;
    i.clone_done = 1;
    i.clone_total = 4;
    i.update_downloading = true;
    i.update_done = 1;
    i.update_total = 2;
    i.active_tab_loading = true;
    i.index_running = true;
    i.index_done = 5;
    i.index_total = 8;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 1, total: 4 }
    );
    i.clone_running = false;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 1, total: 2 }
    );
    i.update_downloading = false;
    assert_eq!(aggregate_taskbar_progress(&i), TaskbarProgressState::Indeterminate);
    i.active_tab_loading = false;
    assert_eq!(
        aggregate_taskbar_progress(&i),
        TaskbarProgressState::Determinate { done: 5, total: 8 }
    );
    i.index_running = false;
    assert_eq!(aggregate_taskbar_progress(&i), TaskbarProgressState::None);
}
