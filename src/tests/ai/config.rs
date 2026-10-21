use super::*;

#[test]
fn validation_errors_keep_identity_and_support_explicit_languages() {
    use crate::language::Language;
    let mut settings = AiProviderSettings::default();
    let error = settings.validate().unwrap_err();
    assert!(error.message_key().is_some());
    assert_eq!(error.display_in(Language::En), "Enter a Base URL");
    assert_eq!(error.display_in(Language::ZhCn), "请填写 Base URL");
    settings.temperature = 3.0;
    let error = settings.validate().unwrap_err();
    assert!(error.display_in(Language::En).contains("between 0 and 2"));
    assert!(error.display_in(Language::ZhCn).contains("0–2"));
}

#[test]
fn default_is_disabled_and_chat_completions() {
    let settings = AiProviderSettings::default();
    assert!(!settings.enabled);
    assert_eq!(settings.api_type, AiApiType::ChatCompletions);
    assert_eq!(settings.api_type.endpoint_path(), "/chat/completions");
    assert!(!settings.is_usable());
}

#[test]
fn validate_rejects_empty_fields() {
    let mut settings = AiProviderSettings::default();
    settings.enabled = true;

    assert!(settings.validate().is_err()); // 缺 base_url 和 model

    settings.base_url = "https://api.openai.com/v1".into();
    assert!(settings.validate().is_err()); // 仍缺 model

    settings.model = "gpt-4o-mini".into();
    assert!(settings.validate().is_ok()); // api_key 可选，不填也能通过
}

#[test]
fn validate_allows_empty_api_key() {
    let mut settings = AiProviderSettings::default();
    settings.enabled = true;
    settings.base_url = "http://localhost:11434".into();
    settings.api_key = "".into(); // 本地模型如 Ollama 不需要 API Key
    settings.model = "llama3".into();
    assert!(settings.validate().is_ok());
}

#[test]
fn validate_rejects_non_http_base_url() {
    let mut settings = AiProviderSettings::default();
    settings.enabled = true;
    settings.base_url = "ftp://example.com".into();
    settings.api_key = "sk-test".into();
    settings.model = "gpt-4o-mini".into();
    let err = settings.validate().unwrap_err().to_string();
    assert!(err.contains("http://"));
}

#[test]
fn json_round_trip() {
    let mut settings = AiProviderSettings::default();
    settings.enabled = true;
    settings.base_url = "https://api.deepseek.com".into();
    settings.api_key = "sk-abc".into();
    settings.model = "deepseek-chat".into();
    settings.temperature = 0.1;
    settings.max_tokens = 1200;

    let json = serde_json::to_string(&settings).unwrap();
    let parsed: AiProviderSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.enabled, settings.enabled);
    assert_eq!(parsed.api_type, settings.api_type);
    assert_eq!(parsed.base_url, settings.base_url);
    assert_eq!(parsed.api_key, settings.api_key);
    assert_eq!(parsed.model, settings.model);
    assert_eq!(parsed.temperature, settings.temperature);
    assert_eq!(parsed.max_tokens, settings.max_tokens);
}

#[test]
fn normalized_base_url_strips_trailing_slash() {
    let mut settings = AiProviderSettings::default();
    settings.base_url = "https://api.openai.com/v1/".into();
    assert_eq!(settings.normalized_base_url(), "https://api.openai.com/v1");
}

#[test]
fn is_usable_requires_enabled_and_valid() {
    let mut settings = AiProviderSettings::default();
    settings.base_url = "https://api.openai.com/v1".into();
    settings.api_key = "sk-test".into();
    settings.model = "gpt-4o-mini".into();

    // 未启用：不可用
    assert!(!settings.is_usable());

    settings.enabled = true;
    assert!(settings.is_usable());
}

#[test]
fn api_paths_and_legacy_serialization_are_compatible() {
    for (kind, path) in [(AiApiType::ChatCompletions, "/chat/completions"),
        (AiApiType::Responses, "/responses"), (AiApiType::AnthropicMessages, "/messages")] {
        assert_eq!(kind.endpoint_path(), path);
        let settings = AiProviderSettings { api_type: kind, reasoning_effort: Some("high".into()), ..Default::default() };
        assert_eq!(serde_json::from_str::<AiProviderSettings>(&serde_json::to_string(&settings).unwrap()).unwrap(), settings);
    }
    assert_eq!(serde_json::to_string(&AiApiType::ChatCompletions).unwrap(), "\"ChatCompletions\"");
    let mut json = serde_json::to_value(AiProviderSettings::default()).unwrap();
    json.as_object_mut().unwrap().remove("reasoning_effort");
    assert_eq!(serde_json::from_value::<AiProviderSettings>(json).unwrap().reasoning_effort, None);
}

#[test]
fn advanced_defaults_and_validation() {
    let mut settings = AiProviderSettings::default();
    assert_eq!((settings.temperature, settings.max_tokens, settings.request_timeout_secs), (0.3, 16_384, 60));
    for temperature in [f32::NAN, f32::INFINITY, -0.1, 2.1] {
        settings.temperature = temperature;
        assert!(settings.validate_advanced().is_err());
    }
    settings.temperature = 2.0;
    assert!(settings.validate_advanced().is_ok());
    settings.max_tokens = 0;
    assert!(settings.validate_advanced().is_err());
    settings.max_tokens = 1;
    settings.request_timeout_secs = 0;
    assert!(settings.validate_advanced().is_err());
}

#[test]
fn temperature_limits_follow_protocol() {
    for api_type in [AiApiType::ChatCompletions, AiApiType::Responses, AiApiType::AnthropicMessages] {
        let mut settings = AiProviderSettings { api_type, temperature: 1.0, ..Default::default() };
        assert!(settings.validate_advanced().is_ok());
        settings.temperature = 1.5;
        assert_eq!(settings.validate_advanced().is_ok(), api_type != AiApiType::AnthropicMessages);
        if api_type == AiApiType::AnthropicMessages {
            assert!(settings.validate_advanced().unwrap_err().to_string().contains("0–1"));
        }
    }
}
