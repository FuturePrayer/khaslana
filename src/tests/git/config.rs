use git2::{Config, ConfigLevel};
use tempfile::TempDir;

use super::*;
use crate::git::test_support::git_test_support as git_support;

/// 仓库级读写回环：写入后本层级值与生效值一致，来源为 Local。
#[test]
fn local_read_write_roundtrip() {
    let (_dir, repo, service) = git_support::init_repo();
    service
        .write_git_config(
            Some(&repo),
            GitConfigScope::Local,
            &[
                ("user.name".into(), Some("Alice".into())),
                ("user.email".into(), Some("alice@example.com".into())),
            ],
        )
        .unwrap();
    let snapshot = service.read_git_config(Some(&repo), GitConfigScope::Local).unwrap();
    let name = snapshot.item("user.name").unwrap();
    assert_eq!(name.value.as_deref(), Some("Alice"));
    assert_eq!(name.effective.as_deref(), Some("Alice"));
    assert_eq!(name.effective_level, Some(GitConfigLevel::Local));
    assert_eq!(snapshot.item("user.email").unwrap().value.as_deref(), Some("alice@example.com"));
}

/// 清除本层级键：值变 None；清除不存在的键幂等成功。
#[test]
fn local_unset_removes_key_and_is_idempotent() {
    let (_dir, repo, service) = git_support::init_repo();
    service
        .write_git_config(Some(&repo), GitConfigScope::Local, &[("user.name".into(), Some("Alice".into()))])
        .unwrap();
    service
        .write_git_config(Some(&repo), GitConfigScope::Local, &[("user.name".into(), None)])
        .unwrap();
    let snapshot = service.read_git_config(Some(&repo), GitConfigScope::Local).unwrap();
    assert_eq!(snapshot.item("user.name").unwrap().value, None);
    // 再次清除不报错（幂等）。
    service
        .write_git_config(Some(&repo), GitConfigScope::Local, &[("user.name".into(), None)])
        .unwrap();
}

/// 空变更直接成功，不落盘。
#[test]
fn empty_changes_are_noop() {
    let (_dir, repo, service) = git_support::init_repo();
    service.write_git_config(Some(&repo), GitConfigScope::Local, &[]).unwrap();
}

/// Local 作用域必须提供仓库。
#[test]
fn local_scope_requires_repository() {
    let service = git_support::service();
    assert!(service.read_git_config(None, GitConfigScope::Local).is_err());
    assert!(service
        .write_git_config(None, GitConfigScope::Local, &[("user.name".into(), Some("x".into()))])
        .is_err());
}

/// 全局读取冒烟（只读，不写真实全局配置）：快照覆盖全部白名单键。
#[test]
fn global_read_returns_all_keys() {
    let service = git_support::service();
    let snapshot = service.read_git_config(None, GitConfigScope::Global).unwrap();
    assert_eq!(snapshot.items.len(), GIT_CONFIG_KEYS.len());
}

/// 级联与继承：本层级覆盖全局；仅全局设置的键本层级为 None、生效值来源 Global。
/// 用临时文件模拟层级，不碰真实全局配置。
#[test]
fn read_items_cascade_and_inheritance() {
    let dir = TempDir::new().unwrap();
    let global_path = dir.path().join("global");
    let local_path = dir.path().join("local");
    std::fs::write(&global_path, "[user]\nname = Global User\nemail = global@example.com\n").unwrap();
    std::fs::write(&local_path, "[user]\nname = Local User\n").unwrap();
    let mut merged = Config::new().unwrap();
    merged.add_file(&global_path, ConfigLevel::Global, true).unwrap();
    merged.add_file(&local_path, ConfigLevel::Local, true).unwrap();
    let level = merged.open_level(ConfigLevel::Local).unwrap();
    let snapshot = read_items(&level, &merged);

    let name = snapshot.item("user.name").unwrap();
    assert_eq!(name.value.as_deref(), Some("Local User"));
    assert_eq!(name.effective.as_deref(), Some("Local User"));
    assert_eq!(name.effective_level, Some(GitConfigLevel::Local));

    let email = snapshot.item("user.email").unwrap();
    assert_eq!(email.value, None);
    assert_eq!(email.effective.as_deref(), Some("global@example.com"));
    assert_eq!(email.effective_level, Some(GitConfigLevel::Global));
}

/// 布尔与枚举值规范化：yes→true、input 保留、1→true。
#[test]
fn read_value_normalizes_bool_and_enum() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("cfg");
    std::fs::write(&path, "[core]\nquotepath = yes\nautocrlf = input\nlongpaths = 1\n").unwrap();
    let config = Config::open(&path).unwrap();
    assert_eq!(read_value(&config, "core.quotepath").as_deref(), Some("true"));
    assert_eq!(read_value(&config, "core.autocrlf").as_deref(), Some("input"));
    assert_eq!(read_value(&config, "core.longpaths").as_deref(), Some("true"));
    assert_eq!(read_value(&config, "fetch.prune"), None);
}

/// 写入机制（apply_changes）在隔离临时文件上的设值/清除回环。
#[test]
fn apply_changes_set_and_remove_on_temp_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("cfg");
    let mut config = Config::open(&path).unwrap();
    apply_changes(&mut config, &[("user.name".into(), Some("Alice".into()))]).unwrap();
    assert_eq!(config.get_string("user.name").unwrap(), "Alice");
    apply_changes(&mut config, &[("user.name".into(), None)]).unwrap();
    assert!(config.get_string("user.name").is_err());
    apply_changes(&mut config, &[("user.name".into(), None)]).unwrap();
}

/// 校验：拒绝换行/控制字符，保留制表符。
#[test]
fn normalize_changes_rejects_control_characters() {
    let err = normalize_changes(&[("user.name".into(), Some("a\nb".into()))]).unwrap_err();
    assert!(err.is_message(domain_ui_resources::GIT_CONFIG_INVALID_VALUE.key));
    let err = normalize_changes(&[("user.name".into(), Some("a\rb".into()))]).unwrap_err();
    assert!(err.is_message(domain_ui_resources::GIT_CONFIG_INVALID_VALUE.key));
    // 制表符允许。
    assert!(normalize_changes(&[("user.name".into(), Some("a\tb".into()))]).is_ok());
}

/// 校验：白名单外的键被拒绝。
#[test]
fn normalize_changes_rejects_unsupported_key() {
    let err = normalize_changes(&[("core.evil".into(), Some("x".into()))]).unwrap_err();
    assert!(err.is_message(domain_ui_resources::GIT_CONFIG_UNSUPPORTED_KEY.key));
}

/// 规范化：trim 首尾空白；None 保持 None。
#[test]
fn normalize_changes_trims_values() {
    let normalized = normalize_changes(&[
        ("user.name".into(), Some("  Alice  ".into())),
        ("user.email".into(), None),
    ])
    .unwrap();
    assert_eq!(normalized[0].1.as_deref(), Some("Alice"));
    assert_eq!(normalized[1].1, None);
}
