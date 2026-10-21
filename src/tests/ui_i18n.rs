use super::*;

const SAMPLE: UiMessage = UiMessage::new("test.parameter", "无法打开 {path}：{error}", "Could not open {path}: {error}");

#[test]
fn complete_sentence_parameters_are_not_recursively_interpolated() {
    assert_eq!(format_in(Language::En, SAMPLE, &[("path", "用户/{error}.rs"), ("error", "外部错误 {path}")]),
        "Could not open 用户/{error}.rs: 外部错误 {path}");
}

#[test]
fn encoded_status_translates_lazily_and_keeps_identity() {
    let status = encode(SAMPLE, &[("path", "用户/{error}.rs"), ("error", "原始错误")]);
    assert_eq!(message_key(&status).as_deref(), Some("test.parameter"));
    assert_eq!(resolve_in(Language::ZhCn, &status), "无法打开 用户/{error}.rs：原始错误");
    assert_eq!(resolve_in(Language::En, &status), "Could not open 用户/{error}.rs: 原始错误");
    assert_eq!(resolve_in(Language::En, "用户数据与原始外部错误"), "用户数据与原始外部错误");
}

#[test]
fn resource_registry_rejects_duplicate_keys_and_parameter_drift() {
    assert!(validate(&[SAMPLE]).is_ok());
    assert!(validate(&[SAMPLE, SAMPLE]).is_err());
    assert!(validate(&[UiMessage::new("test.bad", "{path}", "{error}")]).is_err());
}

#[test]
fn missing_english_uses_chinese_baseline() {
    assert_eq!(text_in(Language::En, UiMessage::new("test.fallback", "保留中文", "")), "保留中文");
}

#[test]
fn persistent_message_keeps_identity_and_original_arguments() {
    let message = LocalizedMessage::new(SAMPLE, &[("path", "用户数据"), ("error", "原始错误")]);
    assert_eq!(message.message.key, "test.parameter");
    assert_eq!(message.args[0].1, "用户数据");
    assert_eq!(format_in(Language::ZhCn, message.message, &[("path", "用户数据"), ("error", "原始错误")]), "无法打开 用户数据：原始错误");
}

#[test]
fn only_explicit_message_parameters_are_resolved() {
    let inner = encode(SAMPLE, &[("path", "用户路径"), ("error", "外部错误")]);
    let outer = UiMessage::new("test.nested", "操作失败：{error}，数据 {data}", "Operation failed: {error}; data {data}");
    let status = encode_with_messages(outer, &[("data", &inner)], &[("error", &inner)]);
    let english = resolve_in(Language::En, &status);
    assert!(english.starts_with("Operation failed: Could not open 用户路径: 外部错误; data "));
    assert!(english.ends_with(&inner));
    assert_eq!(message_key(&status).as_deref(), Some("test.nested"));
}
