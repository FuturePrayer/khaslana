use super::*;

#[test]
fn conflict_workbench_uses_flat_semantic_surface_roles() {
    assert_eq!(
        conflict_workbench_surfaces(),
        ConflictWorkbenchSurfaces {
            canvas: ui_theme::SURFACE_CANVAS,
            rail: ui_theme::RAIL_SURFACE,
            header: ui_theme::TITLEBAR_SURFACE,
            content: ui_theme::SURFACE_BASE,
            border: ui_theme::BORDER_MUTED,
        }
    );
}

#[test]
fn conflict_paths_follow_snapshot_conflict_order() {
    let mut snapshot = RepositorySnapshot::default();
    snapshot.conflicts = vec!["a.txt".to_string(), "dir/b.txt".to_string()];

    assert_eq!(
        conflict_paths(Some(&snapshot)),
        vec!["a.txt".to_string(), "dir/b.txt".to_string()]
    );
    assert!(conflict_paths(None).is_empty());
}

#[test]
fn conflict_status_message_uses_operation_identity_and_keeps_bilingual_count() {
    use khaslana::{language::Language, ui_i18n::{message_key, resolve_in}};
    let merge = conflict_status_message(crate::operation_ui_resources::MERGE_OPERATION_COMPLETED.key, 2);
    assert_eq!(message_key(&merge).as_deref(), Some(messages::MERGE_CONFLICTS_OCCURRED_USE_IDEA_FROM_THE_WORKING_TREE_OR.key));
    assert_eq!(resolve_in(Language::ZhCn, &merge), "合并产生冲突，请在工作区使用 IDEA 或进入“冲突处理”解决（2 个文件）");
    assert_eq!(resolve_in(Language::En, &merge), "Merge conflicts occurred. Use IDEA from the working tree or open Conflicts to resolve them (2 files).");
    let other = conflict_status_message(crate::operation_ui_resources::REBASE_COMPLETED.key, 1);
    assert_eq!(resolve_in(Language::ZhCn, &other), "操作产生冲突，请在左侧“冲突”区域解决（1 个文件）");
    assert_eq!(resolve_in(Language::En, &other), "The operation caused conflicts. Resolve them in Conflicts on the left (1 files).");
    // 中文展示标签即使包含“合并”，也不是业务操作身份。
    let legacy_label = conflict_status_message("合并操作已完成", 3);
    assert_eq!(message_key(&legacy_label).as_deref(), Some(messages::OPERATION_CONFLICTS.key));
}

#[test]
fn conflict_plain_line_model_preserves_empty_and_trailing_lines() {
    let model = ConflictPlainLineModel::new("base\n\nend\n");

    assert_eq!(model.line_count(), 4);
    assert_eq!(model.line_text(0), "base");
    assert_eq!(model.line_text(1), "");
    assert_eq!(model.line_text(2), "end");
    assert_eq!(model.line_text(3), "");
}
