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
fn conflict_status_message_names_operation_and_resolution_area() {
    assert_eq!(
        conflict_status_message("合并操作已完成", 2),
        "合并产生冲突，请在工作区使用 IDEA 或进入“冲突处理”解决（2 个文件）"
    );
    assert_eq!(
        conflict_status_message("正在同步", 1),
        "操作产生冲突，请在左侧“冲突”区域解决（1 个文件）"
    );
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
