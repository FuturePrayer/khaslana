use super::*;

fn workflow_bindings(entries: &[(&str, &str, bool)]) -> khaslana::WorkflowShortcutBindings {
    khaslana::WorkflowShortcutBindings {
        bindings: entries
            .iter()
            .map(|(file, keystroke, background)| {
                (
                    file.to_string(),
                    khaslana::WorkflowShortcutBinding {
                        keystroke: keystroke.to_string(),
                        background: *background,
                    },
                )
            })
            .collect(),
    }
}

#[test]
fn format_keystroke_ctrl_shift_combo() {
    assert_eq!(format_keystroke("ctrl-shift-f"), "Ctrl+Shift+F");
    assert_eq!(format_keystroke("ctrl-shift-l"), "Ctrl+Shift+L");
}

#[test]
fn format_keystroke_single_key() {
    assert_eq!(format_keystroke("f5"), "F5");
}

#[test]
fn format_keystroke_ctrl_comma() {
    assert_eq!(format_keystroke("ctrl-,"), "Ctrl+,");
}

#[test]
fn format_keystroke_ctrl_number() {
    assert_eq!(format_keystroke("ctrl-1"), "Ctrl+1");
}

#[test]
fn find_conflict_detects_other_action() {
    // 默认绑定中 refresh=f5, fetch=ctrl-shift-f。
    let bindings = crate::default_shortcut_bindings();
    // 给 refresh 绑定 ctrl-shift-f（fetch 的默认）应冲突到 fetch。
    let conflict = crate::find_keystroke_conflict(
        &bindings,
        &workflow_bindings(&[]),
        &crate::ShortcutRecordingTarget::App(ShortcutAction::Refresh),
        "ctrl-shift-f",
    );
    assert_eq!(
        conflict,
        Some(crate::ShortcutConflict::App(ShortcutAction::Fetch))
    );
}

#[test]
fn find_conflict_no_self_conflict() {
    let bindings = crate::default_shortcut_bindings();
    // 一个动作的当前绑定不与自己冲突。
    let conflict = crate::find_keystroke_conflict(
        &bindings,
        &workflow_bindings(&[]),
        &crate::ShortcutRecordingTarget::App(ShortcutAction::Refresh),
        "f5",
    );
    assert_eq!(conflict, None);
}

#[test]
fn find_conflict_covers_static_and_workflow_in_both_directions() {
    let app = crate::default_shortcut_bindings();
    let workflows = workflow_bindings(&[("sync.json5", "ctrl-alt-1", false)]);

    // 录制静态动作时撞工作流绑定。
    let conflict = crate::find_keystroke_conflict(
        &app,
        &workflows,
        &crate::ShortcutRecordingTarget::App(ShortcutAction::Refresh),
        "ctrl-alt-1",
    );
    assert_eq!(
        conflict,
        Some(crate::ShortcutConflict::Workflow("sync.json5".to_string()))
    );

    // 录制工作流绑定时撞静态动作（含其默认键位）。
    let conflict = crate::find_keystroke_conflict(
        &app,
        &workflows,
        &crate::ShortcutRecordingTarget::Workflow {
            file: "release.json5".to_string(),
        },
        "f5",
    );
    assert_eq!(
        conflict,
        Some(crate::ShortcutConflict::App(ShortcutAction::Refresh))
    );

    // 工作流 ↔ 工作流（排除自身）。
    let conflict = crate::find_keystroke_conflict(
        &app,
        &workflows,
        &crate::ShortcutRecordingTarget::Workflow {
            file: "release.json5".to_string(),
        },
        "ctrl-alt-1",
    );
    assert_eq!(
        conflict,
        Some(crate::ShortcutConflict::Workflow("sync.json5".to_string()))
    );

    // 同一模板重录自己的键位不冲突。
    let conflict = crate::find_keystroke_conflict(
        &app,
        &workflows,
        &crate::ShortcutRecordingTarget::Workflow {
            file: "sync.json5".to_string(),
        },
        "ctrl-alt-1",
    );
    assert_eq!(conflict, None);
}

#[test]
fn prune_workflow_shortcut_bindings_drops_invalid_entries() {
    let app = crate::default_shortcut_bindings();
    let bindings = workflow_bindings(&[
        ("ok.json5", "ctrl-alt-1", true),
        ("clash.json5", "f5", false),       // 撞静态 refresh 默认键
        ("dup.json5", "ctrl-alt-1", false), // 与 ok.json5 同键位（字母序在前者保留）
        ("bad.json5", "", false),           // 空键位
    ]);

    let (pruned, changed) = crate::prune_workflow_shortcut_bindings(&bindings, &app);
    assert!(changed);
    // 同键位重复：BTreeMap 字母序遍历，先到的 dup.json5 保留。
    let binding = pruned
        .bindings
        .get("dup.json5")
        .expect("first duplicate kept");
    assert_eq!(binding.keystroke, "ctrl-alt-1");
    assert!(!binding.background);
    assert!(!pruned.bindings.contains_key("clash.json5"));
    assert!(!pruned.bindings.contains_key("ok.json5"));
    assert!(!pruned.bindings.contains_key("bad.json5"));

    // 已干净的映射不报告变化。
    let (again, changed_again) = crate::prune_workflow_shortcut_bindings(&pruned, &app);
    assert!(!changed_again);
    assert_eq!(again, pruned);
}

#[test]
fn recording_visual_state_only_marks_the_active_action() {
    let refresh_target = crate::ShortcutRecordingTarget::App(ShortcutAction::Refresh);
    assert!(shortcut_row_is_recording(
        Some(&refresh_target),
        ShortcutAction::Refresh
    ));
    assert!(!shortcut_row_is_recording(
        Some(&refresh_target),
        ShortcutAction::Fetch
    ));
    assert!(!shortcut_row_is_recording(None, ShortcutAction::Refresh));
    // 工作流录制目标不点亮任何静态动作行。
    let workflow_target = crate::ShortcutRecordingTarget::Workflow {
        file: "sync.json5".to_string(),
    };
    assert!(!shortcut_row_is_recording(
        Some(&workflow_target),
        ShortcutAction::Refresh
    ));
}

#[test]
fn reset_default_is_disabled_during_recording_with_reason() {
    let target = crate::ShortcutRecordingTarget::App(ShortcutAction::Refresh);
    assert!(shortcut_reset_enabled(None));
    assert!(!shortcut_reset_enabled(Some(&target)));
    assert_eq!(
        shortcut_reset_disabled_reason(Some(&target)),
        Some("请先结束快捷键录制")
    );
    assert_eq!(shortcut_reset_disabled_reason(None), None);
}

#[test]
fn format_keystroke_double_press_sequences() {
    assert_eq!(format_keystroke("shift shift"), "Shift Shift");
    assert_eq!(format_keystroke("control control"), "Ctrl Ctrl");
    assert_eq!(format_keystroke("alt alt"), "Alt Alt");
    // 序列拆分不影响既有单键显示。
    assert_eq!(format_keystroke("ctrl-shift-f"), "Ctrl+Shift+F");
    assert_eq!(format_keystroke("f5"), "F5");
}

#[test]
fn valid_shortcut_strings_cover_single_keys_and_modifier_double_press() {
    for key in DOUBLE_PRESS_MODIFIER_KEYS {
        assert!(is_valid_shortcut_string(&format!("{key} {key}")));
    }
    assert!(is_valid_shortcut_string("ctrl-p"));
    assert!(is_valid_shortcut_string("f5"));
    assert!(is_valid_shortcut_string("ctrl-,"));
    // 普通按键连按会拦截打字，混合修饰键、三段与空串一律拒绝。
    assert!(!is_valid_shortcut_string("p p"));
    assert!(!is_valid_shortcut_string("f5 f5"));
    assert!(!is_valid_shortcut_string("shift control"));
    assert!(!is_valid_shortcut_string("shift shift shift"));
    assert!(!is_valid_shortcut_string(""));
    assert!(!is_valid_shortcut_string("   "));
    assert!(!is_valid_shortcut_string("not-a-key"));
}

#[test]
fn modifier_only_keystroke_guard_matches_synthesized_taps() {
    for key in ["shift", "control", "alt", "platform", "function"] {
        let ks = gpui::Keystroke::parse(key).unwrap();
        assert!(is_modifier_only_keystroke(&ks), "{key}");
    }
    // 带修饰键的组合与普通按键不是轻点修饰键。
    assert!(!is_modifier_only_keystroke(
        &gpui::Keystroke::parse("ctrl-shift").unwrap()
    ));
    assert!(!is_modifier_only_keystroke(
        &gpui::Keystroke::parse("a").unwrap()
    ));
}

#[test]
fn prune_keeps_modifier_double_press_and_drops_invalid_sequences() {
    let app = crate::default_shortcut_bindings();
    let bindings = workflow_bindings(&[
        ("double.json5", "shift shift", true),
        ("letters.json5", "p p", false),      // 普通按键连按：拒绝
        ("mixed.json5", "shift control", false), // 混合修饰键：拒绝
        ("triple.json5", "alt alt alt", false),  // 三段：拒绝
    ]);
    let (pruned, changed) = crate::prune_workflow_shortcut_bindings(&bindings, &app);
    assert!(changed);
    let binding = pruned.bindings.get("double.json5").expect("double press kept");
    assert_eq!(binding.keystroke, "shift shift");
    assert!(binding.background);
    assert!(!pruned.bindings.contains_key("letters.json5"));
    assert!(!pruned.bindings.contains_key("mixed.json5"));
    assert!(!pruned.bindings.contains_key("triple.json5"));
}

#[test]
fn double_press_conflicts_are_detected_in_both_directions() {
    let mut app = crate::default_shortcut_bindings();
    app.bindings.insert(
        ShortcutAction::OpenCodeSearch.action_id().to_string(),
        "shift shift".to_string(),
    );
    // 工作流录制撞静态连按键。
    let conflict = crate::find_keystroke_conflict(
        &app,
        &workflow_bindings(&[]),
        &crate::ShortcutRecordingTarget::Workflow {
            file: "sync.json5".to_string(),
        },
        "shift shift",
    );
    assert_eq!(
        conflict,
        Some(crate::ShortcutConflict::App(ShortcutAction::OpenCodeSearch))
    );
    // 静态动作恢复默认前撞工作流连按键。
    let conflict = crate::find_keystroke_conflict(
        &app,
        &workflow_bindings(&[("sync.json5", "alt alt", false)]),
        &crate::ShortcutRecordingTarget::App(ShortcutAction::Refresh),
        "alt alt",
    );
    assert_eq!(
        conflict,
        Some(crate::ShortcutConflict::Workflow("sync.json5".to_string()))
    );
    // 连按串与单键串互不误报。
    assert_eq!(
        crate::find_keystroke_conflict(
            &app,
            &workflow_bindings(&[]),
            &crate::ShortcutRecordingTarget::App(ShortcutAction::Refresh),
            "shift",
        ),
        None
    );
}
