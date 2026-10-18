use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use super::state::{RecoveryOperation, RecoveryReadKey, RecoverySide};
use super::view::{recovery_canvas_width, recovery_layout};
use crate::{RepoTabId, RepoTabState, TextFieldState};

fn tab(id: u64, path: &str) -> RepoTabState {
    let cx = gpui::TestAppContext::single();
    let field = cx.update(|cx| TextFieldState::new(cx, "提交信息"));
    RepoTabState::new(RepoTabId(id), Some(PathBuf::from(path)), field)
}

#[test]
fn recovery_read_results_require_tab_path_and_generation() {
    let mut target = tab(1, "C:/repos/a");
    target.recovery.generation = 8;
    let key = RecoveryReadKey {
        tab_id: target.id,
        repo_path: target.repo_path.clone().unwrap(),
        generation: 8,
    };
    assert!(key.matches(&target));
    let other = tab(2, "C:/repos/a");
    assert!(!key.matches(&other));
    target.repo_path = Some(PathBuf::from("C:/repos/b"));
    assert!(!key.matches(&target));
    target.repo_path = Some(PathBuf::from("C:/repos/a"));
    target.recovery.generation += 1;
    assert!(!key.matches(&target));
}

#[test]
fn recovery_fields_and_selection_are_isolated_per_repository() {
    let mut a = tab(1, "C:/repos/a");
    let b = tab(2, "C:/repos/b");
    assert_ne!(a.recovery.search.uid, b.recovery.search.uid);
    assert_ne!(a.recovery.branch_name.uid, b.recovery.branch_name.uid);
    assert_ne!(a.recovery.search.uid, a.recovery.branch_name.uid);
    a.recovery.search.set_value("reset");
    a.recovery.show_recovery = true;
    a.recovery.preview.open = true;
    assert!(b.recovery.search.value.is_empty());
    assert!(!b.recovery.show_recovery);
    assert!(!b.recovery.preview.open);
    assert_eq!(b.recovery.days, Some(30));
}

#[test]
fn closing_repository_cancels_queued_tasks_and_invalidates_read_results() {
    let mut target = tab(1, "C:/repos/a");
    let reading = Arc::new(AtomicBool::new(false));
    let writing = Arc::new(AtomicBool::new(false));
    target.recovery.cancel = Some(reading.clone());
    target.recovery.operation = Some(RecoveryOperation {
        id: 3,
        cancel: writing.clone(),
    });
    target.recovery.loading = true;
    target.recovery.querying = true;
    let key = RecoveryReadKey {
        tab_id: target.id,
        repo_path: target.repo_path.clone().unwrap(),
        generation: target.recovery.generation,
    };
    target.recovery.cancel_tasks();
    assert!(reading.load(Ordering::Relaxed));
    assert!(writing.load(Ordering::Relaxed));
    assert!(!key.matches(&target));
    assert!(!target.recovery.loading);
    assert!(!target.recovery.querying);
    // 取消并不声称写入未发生；运行槽直到对应结果回传才收尾。
    assert!(target.recovery.operation.is_some());
}

#[test]
fn late_recovery_completion_cannot_release_the_new_operation() {
    let mut target = tab(1, "C:/repos/a");
    target.recovery.operation = Some(RecoveryOperation {
        id: 8,
        cancel: Arc::new(AtomicBool::new(false)),
    });
    assert!(!target.recovery.finish_operation(7));
    assert_eq!(target.recovery.operation.as_ref().unwrap().id, 8);
    assert!(target.recovery.finish_operation(8));
    assert!(!target.recovery.finish_operation(8));
}

#[test]
fn recovery_target_side_keeps_both_full_oids() {
    let entry = khaslana::ReflogEntry {
        id: "HEAD:2".into(),
        ref_name: "HEAD".into(),
        old_oid: "a".repeat(40),
        new_oid: "b".repeat(40),
        actor: "用户".into(),
        actor_email: "user@example.invalid".into(),
        timestamp: 1,
        timezone_offset: 480,
        message: "reset: moving to main".into(),
        lossy: false,
    };
    assert_eq!(RecoverySide::Before.oid(&entry), entry.old_oid);
    assert_eq!(RecoverySide::After.oid(&entry), entry.new_oid);
    assert_eq!(RecoverySide::Before.label(), "操作前");
    assert_eq!(RecoverySide::After.label(), "操作后");
}

#[test]
fn recovery_layout_collapses_before_detail_and_commands_become_inaccessible() {
    for width in [0.0, 664.0, 860.0, 1039.0] {
        let layout = recovery_layout(width);
        assert!(!layout.dual_pane);
        assert!(layout.list_width >= 0.0);
        assert!(layout.detail_width >= 0.0);
    }
    for width in [1040.0, 1264.0, 1600.0] {
        let layout = recovery_layout(width);
        assert!(layout.dual_pane);
        assert!((320.0..=440.0).contains(&layout.list_width));
        assert!(layout.detail_width >= 500.0);
        assert!(layout.list_width + layout.detail_width <= width);
    }
    assert!(recovery_canvas_width(1360.0, 320.0) < recovery_canvas_width(1360.0, 48.0));
    assert!(recovery_canvas_width(0.0, 48.0) >= 0.0);
}

#[test]
fn pending_search_cannot_append_with_an_old_page_cursor() {
    let mut target = tab(1, "C:/repos/a");
    let old = khaslana::ReflogQuery {
        search: "reset".into(),
        since: Some(100),
    };
    assert_eq!(
        target.recovery.page_query(old.clone(), false),
        Some(old.clone())
    );
    target.recovery.next_offset = Some(100);
    target.recovery.search.set_value("rebase");
    target.recovery.search_changed = Some(std::time::Instant::now());
    let new = khaslana::ReflogQuery {
        search: "rebase".into(),
        since: Some(101),
    };
    assert!(target.recovery.page_query(new.clone(), true).is_none());
    assert!(target.recovery.search_changed.is_some());
    assert_eq!(target.recovery.active_query.as_ref(), Some(&old));
    assert_eq!(target.recovery.page_query(new.clone(), false), Some(new));
    assert!(target.recovery.search_changed.is_none());
}

#[test]
fn later_pages_keep_the_original_time_boundary() {
    let mut target = tab(1, "C:/repos/a");
    let query = khaslana::ReflogQuery {
        search: String::new(),
        since: Some(100),
    };
    target.recovery.page_query(query.clone(), false);
    let later = khaslana::ReflogQuery {
        since: Some(105),
        ..query.clone()
    };
    assert_eq!(target.recovery.page_query(later, true), Some(query));
}

#[test]
fn reopening_preview_invalidates_old_files_and_diff_even_when_request_numbers_repeat() {
    let mut target = tab(1, "C:/repos/a");
    let first_session = target.recovery.begin_preview();
    target.recovery.preview.request_id = 1;
    target.recovery.preview.open = false;
    let second_session = target.recovery.begin_preview();
    target.recovery.preview.request_id = 1;
    assert_ne!(first_session, second_session);
    assert!(target.recovery.preview.open);
    assert!(target.recovery.preview.loading_files);
    assert_eq!(target.recovery.selection_id, second_session);
}

#[test]
fn short_windows_use_compact_controls_without_changing_width_layout() {
    use super::view::recovery_is_compact;
    for height in [0.0, 520.0, 600.0, 679.0] {
        assert!(recovery_is_compact(height));
    }
    for height in [680.0, 820.0, 1080.0] {
        assert!(!recovery_is_compact(height));
    }
    assert!(recovery_is_compact(f32::NAN));
    assert!(!recovery_layout(800.0).dual_pane);
    assert!(recovery_layout(1200.0).dual_pane);
}

#[test]
fn completed_recovery_dialog_is_not_restored_after_cancelling_window_close() {
    use crate::DialogState;
    use crate::repository_core::window_close_dialog_to_restore;
    assert_eq!(
        window_close_dialog_to_restore(Some(DialogState::CreateRecoveryBranch), false),
        None
    );
    assert_eq!(
        window_close_dialog_to_restore(Some(DialogState::CreateRecoveryBranch), true),
        Some(DialogState::CreateRecoveryBranch)
    );
    assert_eq!(
        window_close_dialog_to_restore(Some(DialogState::CreateBranch), false),
        Some(DialogState::CreateBranch)
    );
    assert_eq!(window_close_dialog_to_restore(None, false), None);
}

#[test]
fn completing_a_recovery_does_not_make_old_metadata_current() {
    use super::state::metadata_is_current;
    let mut target = tab(1, "C:/repos/a");
    target.repository_load_id = 9;
    target.recovery.operation = Some(RecoveryOperation {
        id: 3,
        cancel: Arc::new(AtomicBool::new(false)),
    });
    assert!(target.recovery.finish_operation(3));
    assert!(!metadata_is_current(8, target.repository_load_id, false));
    assert!(!metadata_is_current(9, target.repository_load_id, true));
    assert!(metadata_is_current(9, target.repository_load_id, false));
}

#[test]
fn creation_completion_does_not_reuse_a_newer_recovery_context() {
    let mut target = tab(1, "C:/repos/a");
    target.recovery.generation = 3;
    let generation = target.recovery.generation;
    let selection_id = target.recovery.selection_id;
    target.recovery.operation = Some(RecoveryOperation {
        id: 5,
        cancel: Arc::new(AtomicBool::new(false)),
    });
    assert!(target.recovery.context_is_current(generation, selection_id));
    let new_selection = target.recovery.begin_preview();
    assert!(target.recovery.finish_operation(5));
    assert!(!target.recovery.context_is_current(generation, selection_id));
    assert!(
        target
            .recovery
            .context_is_current(generation, new_selection)
    );
    target.recovery.generation += 1;
    assert!(
        !target
            .recovery
            .context_is_current(generation, new_selection)
    );
    assert!(target.recovery.preview.open);
    assert!(target.recovery.preview.loading_files);
}
