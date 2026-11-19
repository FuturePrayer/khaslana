use super::*;
use khaslana::OperationEvent;

#[test]
fn retry_cleanup_only_removes_leftover_created_by_failed_attempt() {
    // 目录不存在：直接克隆。
    assert_eq!(clone_retry_cleanup(false, false), CloneRetryCleanup::NoCleanup);
    assert_eq!(clone_retry_cleanup(true, false), CloneRetryCleanup::NoCleanup);
    // 失败时留下的残留：重试前清理。
    assert_eq!(clone_retry_cleanup(true, true), CloneRetryCleanup::RemoveLeftover);
    // 失败时不存在、现在却存在：外部新建内容，禁止删除。
    assert_eq!(clone_retry_cleanup(false, true), CloneRetryCleanup::BlockTargetExists);
}

#[test]
fn clone_progress_emitter_keeps_numbers_and_task_identity() {
    let (tx, rx) = async_channel::unbounded();
    let emitter = CloneProgressEmitter { tx, task_id: 7 };

    emitter.emit(OperationEvent::TransferProgress { received_objects: 3, total_objects: 10 });
    match rx.recv_blocking().unwrap() {
        UiEvent::CloneProgress { task_id, message, done, total } => {
            assert_eq!(task_id, 7);
            assert_eq!((done, total), (3, 10));
            // 文案与数字分离：译文含对象计数，业务不依赖译文反解析。
            let resolved = khaslana::ui_i18n::resolve(&message);
            assert!(resolved.contains("3/10"), "进度文案应含对象计数：{resolved}");
        }
        other => panic!("期望 CloneProgress，实际 {other:?}"),
    }

    // 阶段文案（检出/子模块）没有确定态数字，进度条回到空轨道。
    emitter.emit(OperationEvent::Progress("checkout".into()));
    match rx.recv_blocking().unwrap() {
        UiEvent::CloneProgress { task_id, done, total, .. } => {
            assert_eq!(task_id, 7);
            assert_eq!((done, total), (0, 0));
        }
        other => panic!("期望 CloneProgress，实际 {other:?}"),
    }
}
