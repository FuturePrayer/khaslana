use super::*;

/// 选项 id 与 git 值互转：unset ↔ None，其余原样。
#[test]
fn option_id_value_roundtrip() {
    assert_eq!(option_id_to_value(UNSET), None);
    assert_eq!(option_id_to_value("true"), Some("true".to_string()));
    assert_eq!(option_id_to_value("input"), Some("input".to_string()));
    assert_eq!(value_to_option_id(None), UNSET);
    assert_eq!(value_to_option_id(Some("false")), "false");
    for id in [UNSET, "true", "false", "input", "merges"] {
        assert_eq!(value_to_option_id(option_id_to_value(id).as_deref()), id);
    }
}

/// 文本框内容转 git 值：trim 后为空表示清除。
#[test]
fn text_to_value_trims_and_clears() {
    assert_eq!(text_to_value(""), None);
    assert_eq!(text_to_value("   "), None);
    assert_eq!(text_to_value("  Alice  "), Some("Alice".to_string()));
    assert_eq!(text_to_value("main"), Some("main".to_string()));
}

/// 下拉键到 git 配置键的映射稳定。
#[test]
fn select_key_config_mapping() {
    assert_eq!(GitConfigSelectKey::Autocrlf.config_key(), "core.autocrlf");
    assert_eq!(GitConfigSelectKey::Quotepath.config_key(), "core.quotepath");
    assert_eq!(GitConfigSelectKey::Longpaths.config_key(), "core.longpaths");
    assert_eq!(GitConfigSelectKey::PullRebase.config_key(), "pull.rebase");
    assert_eq!(GitConfigSelectKey::FetchPrune.config_key(), "fetch.prune");
    assert_eq!(GitConfigSelectKey::RebaseAutoStash.config_key(), "rebase.autoStash");
    assert_eq!(GitConfigSelectKey::ALL.len(), 6);
}

/// 选项构建：含「未设置」；非标准当前值补原样选项，标准值不重复添加。
#[test]
fn build_options_includes_unset_and_unknown_value() {
    let options = build_git_config_options(GitConfigSelectKey::PullRebase, "unset");
    assert!(options.iter().any(|option| option.id == UNSET));
    assert!(options.iter().any(|option| option.id == "merges"));
    assert!(!options.iter().any(|option| option.id == "only"));

    // 非标准值 only：补一个原样选项保证回显。
    let options = build_git_config_options(GitConfigSelectKey::PullRebase, "only");
    assert!(options.iter().any(|option| option.id == "only"));

    // 标准值 true：不额外重复添加。
    let options = build_git_config_options(GitConfigSelectKey::Quotepath, "true");
    assert_eq!(options.iter().filter(|option| option.id == "true").count(), 1);
}
