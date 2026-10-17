use super::state::{AiConnectionTestState, ConnectionTestOutcome};
use khaslana::AiProviderSettings;

fn settings() -> AiProviderSettings {
    AiProviderSettings {
        enabled: true,
        base_url: "https://example.test/v1".into(),
        model: "test-model".into(),
        ..Default::default()
    }
}

#[test]
fn stale_completion_does_not_finish_new_test() {
    let mut state = AiConnectionTestState::default();
    let first = state.begin(settings());
    let second = state.begin(settings());
    assert!(!state.finish(first, Ok("旧结果".into())));
    assert!(state.is_running());
    assert!(state.finish(second, Ok("新结果".into())));
    assert!(matches!(state.outcome_for(&settings()), ConnectionTestOutcome::Passed(message) if message == "新结果"));
}

#[test]
fn changed_connection_never_displays_old_success() {
    let mut state = AiConnectionTestState::default();
    let original = settings();
    let request = state.begin(original.clone());
    assert!(state.finish(request, Ok("通过".into())));
    for field in ["address", "key", "model"] {
        let mut edited = original.clone();
        match field {
            "address" => edited.base_url = "https://other.test/v1".into(),
            "key" => edited.api_key = "new-secret".into(),
            _ => edited.model = "other-model".into(),
        }
        assert!(matches!(state.outcome_for(&edited), ConnectionTestOutcome::Idle));
    }
    assert!(matches!(state.outcome_for(&original), ConnectionTestOutcome::Passed(_)));
}

#[test]
fn editing_during_test_preserves_pending_request_and_scopes_failure() {
    let mut state = AiConnectionTestState::default();
    let original = settings();
    let request = state.begin(original.clone());
    let mut edited = original.clone();
    edited.model = "edited-model".into();
    assert!(matches!(state.outcome_for(&edited), ConnectionTestOutcome::Idle));
    assert!(state.is_running());
    assert!(state.finish(request, Err("服务不可用".into())));
    assert!(!state.is_running());
    assert!(matches!(state.outcome_for(&edited), ConnectionTestOutcome::Idle));
    assert!(matches!(state.outcome_for(&original), ConnectionTestOutcome::Failed(error) if error == "服务不可用"));
}

#[test]
fn duplicate_completion_cannot_overwrite_result() {
    let mut state = AiConnectionTestState::default();
    let request = state.begin(settings());
    assert!(state.finish(request, Err("连接失败".into())));
    assert!(!state.finish(request, Ok("迟到结果".into())));
    assert!(matches!(state.outcome_for(&settings()), ConnectionTestOutcome::Failed(_)));
}

#[test]
fn global_reset_invalidates_in_flight_completion() {
    let mut state = AiConnectionTestState::default();
    let request = state.begin(settings());
    state.interrupt("后台任务异常".into());
    assert!(!state.is_running());
    assert!(!state.finish(request, Ok("迟到结果".into())));
    assert!(matches!(state.outcome_for(&settings()), ConnectionTestOutcome::Failed(_)));
    let next = state.begin(settings());
    assert!(!state.finish(request, Err("旧失败".into())));
    assert!(state.is_running());
    assert!(state.finish(next, Ok("通过".into())));
}

#[test]
fn form_parser_trims_effort_and_keeps_custom_values() {
    use super::state::{parse_form_settings, form_is_dirty};
    let saved = settings();
    let unchanged = parse_form_settings(saved.clone(), " 0.3 ", &saved.max_tokens.to_string(), "60", "  ");
    assert!(!form_is_dirty(&unchanged, &saved));
    let parsed = parse_form_settings(saved.clone(), "1", "800", "120", " vendor-value ").unwrap();
    assert_eq!(parsed.reasoning_effort.as_deref(), Some("vendor-value"));
    assert!(form_is_dirty(&Ok(parsed), &saved));
    for (temp, tokens, timeout) in [("", "4000", "60"), ("NaN", "4000", "60"),
        ("inf", "4000", "60"), ("-0.1", "4000", "60"), ("2.1", "4000", "60"),
        ("0.3", "0", "60"), ("0.3", "1.5", "60"), ("0.3", "4294967296", "60"),
        ("0.3", "4000", "0"), ("0.3", "4000", "-1"),
        ("0.3", "4000", "18446744073709551616")] {
        let invalid = parse_form_settings(saved.clone(), temp, tokens, timeout, "");
        assert!(invalid.is_err(), "{temp}/{tokens}/{timeout}");
        assert!(form_is_dirty(&invalid, &saved));
    }
}

#[test]
fn connection_snapshot_includes_protocol_and_every_advanced_parameter() {
    let mut state = AiConnectionTestState::default();
    let saved = settings();
    let request = state.begin(saved.clone());
    assert!(state.finish(request, Ok("通过".into())));
    let mut variants = vec![saved.clone(); 5];
    variants[0].api_type = khaslana::AiApiType::Responses;
    variants[1].temperature = 1.0;
    variants[2].max_tokens = 800;
    variants[3].request_timeout_secs = 120;
    variants[4].reasoning_effort = Some("high".into());
    for edited in variants {
        assert!(matches!(state.outcome_for(&edited), ConnectionTestOutcome::Idle));
    }
}
