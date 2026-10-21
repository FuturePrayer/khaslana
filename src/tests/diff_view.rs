use super::*;
use khaslana::{GitError, language::Language, ui_i18n::{LocalizedMessage, UiMessage, resolve_in}};

#[test]
fn full_file_limit_identity_survives_string_events_and_language_changes() {
    let message = UiMessage::new("domain.full_file_too_large", "文件过大，无法显示全文视图", "The file is too large to display in full");
    let encoded = diff_task_error(GitError::Localized(LocalizedMessage::new(message, &[])));
    assert!(is_full_file_too_large_error(&encoded));
    assert_eq!(resolve_in(Language::ZhCn, &encoded), message.zh);
    assert_eq!(resolve_in(Language::En, &encoded), message.en);
    // 译文不承担身份：即使正文相同，外部错误或其他资源也不能触发回退。
    assert!(!is_full_file_too_large_error(message.zh));
    assert!(!is_full_file_too_large_error(message.en));
    assert!(!is_full_file_too_large_error(&diff_task_error(GitError::Message(message.zh.into()))));
    let unrelated = UiMessage::new("test.unrelated", message.zh, message.en);
    assert!(!is_full_file_too_large_error(&khaslana::ui_i18n::encode(unrelated, &[])));
}

#[test]
fn full_file_fallback_feedback_is_a_live_bilingual_message() {
    let status = full_file_fallback_status();
    assert_eq!(resolve_in(Language::ZhCn, &status), "文件过大，已回退到差异视图");
    assert_eq!(resolve_in(Language::En, &status), "File is too large; switched back to diff view");
}
