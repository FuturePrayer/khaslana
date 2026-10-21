use super::*;

#[test]
fn credential_actions_column_reserves_both_buttons_and_spacing() {
    // 12px 字号下，中文标签各 24px；英文按较宽的字形预算覆盖 Test / Delete。
    for (test_width, delete_width) in [(24.0, 24.0), (32.0, 44.0), (48.0, 48.0)] {
        let required = credential_actions_column_width(test_width, delete_width);
        assert!(CREDENTIAL_TABLE_OP_COL_W >= required);
        let available_for_labels = CREDENTIAL_TABLE_OP_COL_W - 16.0 - 4.0 - 2.0 * 26.0;
        assert!(available_for_labels >= test_width + delete_width);
        assert!(required >= 120.0);
    }
    assert!(credential_actions_column_width(32.0, 44.0) > 120.0,
        "旧操作列宽无法容纳英文两按钮与完整内边距");
    assert_eq!(CREDENTIAL_TABLE_OP_COL_W, 168.0);
}

#[test]
fn credential_test_validation_errors_are_resolved_before_display() {
    use khaslana::{StoredCredentialKind, ui_i18n::{message_key, text}};
    for (url, expected) in [
        ("", crate::app_ui_resources::TEST_URL_REQUIRED),
        ("   ", crate::app_ui_resources::TEST_URL_REQUIRED),
        ("git@example.com:team/repository.git", crate::app_ui_resources::TEST_HTTPS),
    ] {
        let error = crate::validate_credential_test_url(
            StoredCredentialKind::HttpsUserPass, "https://example.com", url,
        ).expect_err("真实验证器应拒绝空地址或错误协议");
        assert_eq!(message_key(&error).as_deref(), Some(expected.key));
        let displayed = credential_test_error_text(&error);
        assert_eq!(displayed, text(expected));
        assert_ne!(displayed, error);
        assert!(message_key(&displayed).is_none(), "显示内容不能仍是编码消息");
    }
    let external = "外部错误 {path}: invalid URL";
    assert_eq!(credential_test_error_text(external), external);
}
