//! Git 配置读写（全局 / 仓库级），供设置中心「Git 配置」页使用。
//!
//! 约束与原因：
//! - 配置值住在 git 自己的配置文件里，经 git2 读写，**不进 AppStorage**；
//!   写入后 `Repository::signature()` 等级联解析自动生效，提交链路无需接线。
//! - 仓库级写入必须经 `open_level(ConfigLevel::Local)` 取单层视图：
//!   `repo.config()` 是多级合并配置，linked worktree 下最高层可能是
//!   Worktree，直接 `set_str` 会写错文件。
//! - 全局写入经 `open_global()`（libgit2 按 git 规则优先已存在的 XDG 文件）；
//!   两个全局文件都不存在时 `open_global` 返回 NotFound，回退
//!   `find_global()` 定位 `$HOME/.gitconfig` 并用 `Config::open` 打开
//!   （磁盘后端对不存在的文件延迟创建，首次写入即建文件）。
//! - 核心逻辑拆成接收 `&Config` / `&mut Config` 的纯函数，测试用
//!   `Config::open(临时文件)` 隔离，不碰真实全局配置（现有测试设施没有
//!   全局级隔离，见 tests/git/partial_stage.rs 的 autocrlf 污染注释）。

use git2::{Config, ConfigLevel, ErrorCode, Repository};

use super::GitService;
use crate::types::domain_ui_resources;
use crate::types::{GitError, Result};
use crate::ui_i18n::LocalizedMessage;

/// 配置作用域：全局用户配置或当前仓库配置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitConfigScope {
    Global,
    Local,
}

/// 生效值的来源层级（用于「继承自全局/系统」提示）。
/// XDG 归入 Global（同属用户级全局文件），Worktree 归入 Local。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitConfigLevel {
    Local,
    Global,
    System,
    Other,
}

/// 设置页暴露的配置项白名单：只允许读写这些键，防止 UI 缺陷写入任意配置。
/// 刻意不含 credential.helper / http.proxy（应用自管凭据与代理）、
/// core.editor / merge.tool（GUI 自有）、gpg 签名与 core.ignorecase（危险）。
pub const GIT_CONFIG_KEYS: &[&str] = &[
    "user.name",
    "user.email",
    "core.autocrlf",
    "core.quotepath",
    "core.longpaths",
    "init.defaultBranch",
    "pull.rebase",
    "fetch.prune",
    "rebase.autoStash",
];

/// 单个配置项的读取结果。
#[derive(Clone, Debug)]
pub struct GitConfigItemState {
    pub key: &'static str,
    /// 所选作用域层级的值；None 表示该层级未设置。
    pub value: Option<String>,
    /// 级联后的生效值（local → global → system）。
    pub effective: Option<String>,
    /// 生效值的来源层级；生效值不存在时为 None。
    pub effective_level: Option<GitConfigLevel>,
}

/// 一次读取的完整快照（按 [`GIT_CONFIG_KEYS`] 顺序）。
#[derive(Clone, Debug, Default)]
pub struct GitConfigSnapshot {
    pub items: Vec<GitConfigItemState>,
}

impl GitConfigSnapshot {
    pub fn item(&self, key: &str) -> Option<&GitConfigItemState> {
        self.items.iter().find(|item| item.key == key)
    }
}

impl GitService {
    /// 读取配置快照。`scope` 为 Local 时必须提供仓库，否则报 NoRepository。
    pub fn read_git_config(
        &self,
        repo: Option<&Repository>,
        scope: GitConfigScope,
    ) -> Result<GitConfigSnapshot> {
        let snapshot = match scope {
            GitConfigScope::Local => {
                let repo = repo.ok_or(GitError::NoRepository)?;
                let merged = repo.config().map_err(config_read_error)?;
                let level = merged.open_level(ConfigLevel::Local).map_err(config_read_error)?;
                read_items(&level, &merged)
            }
            GitConfigScope::Global => {
                let mut merged = Config::open_default().map_err(config_read_error)?;
                let level = open_user_global(&mut merged).map_err(config_read_error)?;
                read_items(&level, &merged)
            }
        };
        Ok(snapshot)
    }

    /// 写入配置变更：`None` 表示清除该层级的键（幂等），`Some` 表示写入。
    /// 先整体校验再落盘，避免校验失败留下半写入状态。
    pub fn write_git_config(
        &self,
        repo: Option<&Repository>,
        scope: GitConfigScope,
        changes: &[(String, Option<String>)],
    ) -> Result<()> {
        let changes = normalize_changes(changes)?;
        if changes.is_empty() {
            return Ok(());
        }
        match scope {
            GitConfigScope::Local => {
                let repo = repo.ok_or(GitError::NoRepository)?;
                let mut level = repo
                    .config()
                    .and_then(|config| config.open_level(ConfigLevel::Local))
                    .map_err(config_write_error)?;
                apply_changes(&mut level, &changes).map_err(config_write_error)
            }
            GitConfigScope::Global => {
                let mut merged = Config::open_default().map_err(config_write_error)?;
                let mut level = open_user_global(&mut merged).map_err(config_write_error)?;
                apply_changes(&mut level, &changes).map_err(config_write_error)
            }
        }
    }
}

/// 打开用户全局配置的单层视图（读写共用，保证显示与写入 targeting 同一文件）。
/// 全局文件都不存在时回退 `$HOME/.gitconfig`（磁盘后端延迟创建）。
fn open_user_global(default: &mut Config) -> std::result::Result<Config, git2::Error> {
    match default.open_global() {
        Ok(config) => Ok(config),
        Err(err) if err.code() == ErrorCode::NotFound => {
            let path = Config::find_global()?;
            Config::open(&path)
        }
        Err(err) => Err(err),
    }
}

/// 布尔键：git 允许 yes/no/1/0/on/off 等多种写法，统一读成 canonical
/// "true"/"false"，UI 下拉才能稳定回显与往返。
const BOOL_KEYS: &[&str] = &[
    "core.quotepath",
    "core.longpaths",
    "fetch.prune",
    "rebase.autoStash",
];

/// 逐项读取「本层级值 + 生效值 + 生效来源」。读取失败（键不存在、多值键等）
/// 一律按未设置处理，不中断整页加载。
fn read_items(level: &Config, merged: &Config) -> GitConfigSnapshot {
    let items = GIT_CONFIG_KEYS
        .iter()
        .map(|key| {
            let value = read_value(level, key);
            let effective = read_value(merged, key);
            let effective_level = merged.get_entry(key).ok().map(|entry| map_level(entry.level()));
            GitConfigItemState {
                key,
                value,
                effective,
                effective_level,
            }
        })
        .collect();
    GitConfigSnapshot { items }
}

/// 读取单个键并规范化：布尔键经 `get_bool` 归一；autocrlf/pull.rebase 这类
/// 枚举把 true/false 的多种写法归一，保留 input/merges 等字面量；其余原样 trim。
fn read_value(config: &Config, key: &str) -> Option<String> {
    if BOOL_KEYS.contains(&key) {
        return config
            .get_bool(key)
            .ok()
            .map(|value| if value { "true" } else { "false" }.to_string());
    }
    let raw = config.get_string(key).ok()?;
    let trimmed = raw.trim();
    match key {
        "core.autocrlf" | "pull.rebase" => {
            if key == "core.autocrlf" && trimmed.eq_ignore_ascii_case("input") {
                return Some("input".to_string());
            }
            match Config::parse_bool(trimmed) {
                Ok(value) => Some(if value { "true" } else { "false" }.to_string()),
                Err(_) => Some(trimmed.to_string()),
            }
        }
        _ => Some(trimmed.to_string()),
    }
}

fn map_level(level: ConfigLevel) -> GitConfigLevel {
    match level {
        ConfigLevel::Local | ConfigLevel::Worktree => GitConfigLevel::Local,
        ConfigLevel::Global | ConfigLevel::XDG => GitConfigLevel::Global,
        ConfigLevel::System | ConfigLevel::ProgramData => GitConfigLevel::System,
        _ => GitConfigLevel::Other,
    }
}

/// 校验并规范化变更：键必须在白名单内，值 trim 后不得含换行/控制字符
/// （git 配置值单行存储；制表符允许）。
fn normalize_changes(
    changes: &[(String, Option<String>)],
) -> Result<Vec<(String, Option<String>)>> {
    let mut normalized = Vec::with_capacity(changes.len());
    for (key, value) in changes {
        if !GIT_CONFIG_KEYS.contains(&key.as_str()) {
            return Err(GitError::Localized(LocalizedMessage::new(
                domain_ui_resources::GIT_CONFIG_UNSUPPORTED_KEY,
                &[("key", key)],
            )));
        }
        let value = match value {
            None => None,
            Some(value) => {
                let value = value.trim();
                if value
                    .chars()
                    .any(|c| c == '\n' || c == '\r' || (c.is_control() && c != '\t'))
                {
                    return Err(GitError::Localized(LocalizedMessage::new(
                        domain_ui_resources::GIT_CONFIG_INVALID_VALUE,
                        &[("key", key)],
                    )));
                }
                Some(value.to_string())
            }
        };
        normalized.push((key.clone(), value));
    }
    Ok(normalized)
}

/// 落盘已规范化的变更。清除不存在的键视为幂等成功（并发外部修改时不报错）。
fn apply_changes(config: &mut Config, changes: &[(String, Option<String>)]) -> std::result::Result<(), git2::Error> {
    for (key, value) in changes {
        match value {
            Some(value) => config.set_str(key, value)?,
            None => match config.remove(key) {
                Ok(()) => {}
                Err(err) if err.code() == ErrorCode::NotFound => {}
                Err(err) => return Err(err),
            },
        }
    }
    Ok(())
}

fn config_read_error(err: git2::Error) -> GitError {
    GitError::Localized(LocalizedMessage::new(
        domain_ui_resources::GIT_CONFIG_READ_FAILED,
        &[("detail", &err.to_string())],
    ))
}

fn config_write_error(err: git2::Error) -> GitError {
    GitError::Localized(LocalizedMessage::new(
        domain_ui_resources::GIT_CONFIG_WRITE_FAILED,
        &[("detail", &err.to_string())],
    ))
}

#[cfg(test)]
#[path = "../tests/git/config.rs"]
mod tests;
