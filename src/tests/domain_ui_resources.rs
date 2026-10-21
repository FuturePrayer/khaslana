use super::*;
use crate::language::Language;
use crate::types::GitError;
use crate::ui_i18n::{LocalizedMessage, format_in, text_in};

#[test]
fn all_domain_resources_have_unique_keys_and_matching_parameters() {
    assert!(crate::ui_i18n::validate(ALL).is_ok());
    assert!(ALL.len() >= 380);
}

#[test]
fn domain_error_keeps_identity_and_displays_in_both_languages() {
    let error = GitError::Localized(LocalizedMessage::new(
        INVALID_REMOTE_NAME,
        &[("name", "用户/{name}")],
    ));
    assert!(error.is_message(INVALID_REMOTE_NAME.key));
    assert_eq!(error.display_in(Language::ZhCn), "远端名称无效：用户/{name}");
    assert_eq!(error.display_in(Language::En), "Invalid remote name: 用户/{name}");
    assert_eq!(error.message_key(), Some(INVALID_REMOTE_NAME.key));
}

#[test]
fn credential_category_and_external_error_payload_are_preserved() {
    let error = GitError::LocalizedCredential(LocalizedMessage::new(
        FAILED_TO_READ_SYSTEM_CREDENTIALS,
        &[("err", "外部错误 {err}")],
    ));
    assert_eq!(error.display_in(Language::ZhCn), "凭据错误：系统凭据读取失败：外部错误 {err}");
    assert_eq!(error.display_in(Language::En), "Credential error: Failed to read system credentials: 外部错误 {err}");
    let external = GitError::from(std::io::Error::other("用户/原始 IO 错误"));
    assert!(std::error::Error::source(&external).is_some());
    assert_eq!(external.display_in(Language::En), "I/O error: 用户/原始 IO 错误");
    let raw = GitError::Message("原始用户内容".into());
    assert_eq!(raw.display_in(Language::En), "原始用户内容");
}

#[test]
fn guard_identity_is_independent_of_display_language() {
    let error = GitError::Localized(LocalizedMessage::new(FULL_FILE_TOO_LARGE, &[]));
    assert_eq!(error.message_key(), Some("domain.full_file_too_large"));
    assert_eq!(error.display_in(Language::ZhCn), "文件过大，无法显示全文视图");
    assert_eq!(error.display_in(Language::En), "The file is too large to display in full");
    assert_eq!(text_in(Language::En, EMPTY_COMMIT_MESSAGE), "Commit message cannot be empty");
    assert_eq!(GitError::EmptyCommit.message_key(), Some(EMPTY_COMMIT.key));
}

#[test]
fn stash_reference_braces_are_not_reinterpreted() {
    assert_eq!(format_in(Language::En, DELETING_STASH, &[("stash_ref", "stash@{3}")]), "Deleting stash@{3}");
    assert_eq!(format_in(Language::ZhCn, DELETING_STASH, &[("stash_ref", "stash@{3}")]), "正在删除贮藏 stash@{3}");
}
