//! 本视图独立的语义消息资源；配置键名（user.name 等）作为代码原样保留，
//! 用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const SCOPE: UiMessage = UiMessage::new("views.git_config_view.scope", "作用域", "Scope");
pub const SCOPE_HELP: UiMessage = UiMessage::new("views.git_config_view.scope_help", "选择要查看和编辑的配置层级。仓库级配置优先于全局配置生效。", "Choose the configuration level to view and edit. Repository-level configuration takes precedence over global configuration.");
pub const GLOBAL: UiMessage = UiMessage::new("views.git_config_view.global", "全局", "Global");
pub const CURRENT_REPO: UiMessage = UiMessage::new("views.git_config_view.current_repo", "当前仓库", "Current repository");
pub const NO_REPOSITORY_OPEN: UiMessage = UiMessage::new("views.git_config_view.no_repository_open", "尚未打开仓库，无法编辑仓库级配置", "No repository is open; repository-level configuration is unavailable");

pub const IDENTITY_GROUP: UiMessage = UiMessage::new("views.git_config_view.identity_group", "提交身份", "Commit identity");
pub const IDENTITY_GROUP_DESC: UiMessage = UiMessage::new("views.git_config_view.identity_group_desc", "提交、修补提交与标签使用的作者姓名和邮箱。", "Author name and email used for commits, amends, and tags.");
pub const FILES_GROUP: UiMessage = UiMessage::new("views.git_config_view.files_group", "文件与换行", "Files & line endings");
pub const FILES_GROUP_DESC: UiMessage = UiMessage::new("views.git_config_view.files_group_desc", "跨平台换行处理与文件名显示。", "Cross-platform line-ending handling and file name display.");
pub const BRANCH_GROUP: UiMessage = UiMessage::new("views.git_config_view.branch_group", "分支与同步", "Branch & sync");
pub const BRANCH_GROUP_DESC: UiMessage = UiMessage::new("views.git_config_view.branch_group_desc", "新仓库默认分支、拉取与获取行为。", "Default branch for new repositories, plus pull and fetch behavior.");

pub const USER_NAME: UiMessage = UiMessage::new("views.git_config_view.user_name", "用户名", "User name");
pub const USER_NAME_DESC: UiMessage = UiMessage::new("views.git_config_view.user_name_desc", "user.name · 提交时使用的作者姓名。", "user.name · Author name used for commits.");
pub const USER_EMAIL: UiMessage = UiMessage::new("views.git_config_view.user_email", "邮箱", "Email");
pub const USER_EMAIL_DESC: UiMessage = UiMessage::new("views.git_config_view.user_email_desc", "user.email · 提交时使用的作者邮箱。", "user.email · Author email used for commits.");
pub const AUTOCRLF: UiMessage = UiMessage::new("views.git_config_view.autocrlf", "换行符转换", "Line-ending conversion");
pub const AUTOCRLF_DESC: UiMessage = UiMessage::new("views.git_config_view.autocrlf_desc", "core.autocrlf · true 检出转 CRLF、提交转 LF；input 仅提交转 LF；false 不转换。", "core.autocrlf · true converts to CRLF on checkout and LF on commit; input converts to LF on commit only; false disables conversion.");
pub const QUOTEPATH: UiMessage = UiMessage::new("views.git_config_view.quotepath", "路径转义", "Quote paths");
pub const QUOTEPATH_DESC: UiMessage = UiMessage::new("views.git_config_view.quotepath_desc", "core.quotepath · 关闭后中文等非 ASCII 文件名原样显示，不再转义为八进制。", "core.quotepath · When off, non-ASCII file names (such as Chinese) display as-is instead of octal escapes.");
pub const LONGPATHS: UiMessage = UiMessage::new("views.git_config_view.longpaths", "长路径", "Long paths");
pub const LONGPATHS_DESC: UiMessage = UiMessage::new("views.git_config_view.longpaths_desc", "core.longpaths · Windows 上启用超过 260 字符的路径支持。", "core.longpaths · Enables support for paths longer than 260 characters on Windows.");
pub const DEFAULT_BRANCH: UiMessage = UiMessage::new("views.git_config_view.default_branch", "默认分支", "Default branch");
pub const DEFAULT_BRANCH_DESC: UiMessage = UiMessage::new("views.git_config_view.default_branch_desc", "init.defaultBranch · 新建仓库时的初始分支名，留空使用 git 默认（master）。", "init.defaultBranch · Initial branch name for new repositories; leave empty to use the git default (master).");
pub const PULL_REBASE: UiMessage = UiMessage::new("views.git_config_view.pull_rebase", "拉取变基", "Pull rebase");
pub const PULL_REBASE_DESC: UiMessage = UiMessage::new("views.git_config_view.pull_rebase_desc", "pull.rebase · true 拉取时变基，false 合并，merges 保留合并提交。", "pull.rebase · true rebases on pull, false merges, merges preserves merge commits.");
pub const FETCH_PRUNE: UiMessage = UiMessage::new("views.git_config_view.fetch_prune", "获取时清理", "Fetch prune");
pub const FETCH_PRUNE_DESC: UiMessage = UiMessage::new("views.git_config_view.fetch_prune_desc", "fetch.prune · 获取后删除远端已不存在的本地远端跟踪分支。", "fetch.prune · After fetching, remove remote-tracking branches that no longer exist on the remote.");
pub const REBASE_AUTOSTASH: UiMessage = UiMessage::new("views.git_config_view.rebase_autostash", "变基自动贮藏", "Rebase autostash");
pub const REBASE_AUTOSTASH_DESC: UiMessage = UiMessage::new("views.git_config_view.rebase_autostash_desc", "rebase.autoStash · 变基前自动贮藏未提交改动，结束后恢复。", "rebase.autoStash · Automatically stash uncommitted changes before a rebase and restore them afterwards.");

pub const UNSET: UiMessage = UiMessage::new("views.git_config_view.unset", "未设置", "Not set");
pub const ON: UiMessage = UiMessage::new("views.git_config_view.on", "开", "On");
pub const OFF: UiMessage = UiMessage::new("views.git_config_view.off", "关", "Off");

pub const INHERITED_FROM: UiMessage = UiMessage::new("views.git_config_view.inherited_from", "继承自{source}：{value}", "Inherited from {source}: {value}");
pub const NOT_SET_AT_LEVEL: UiMessage = UiMessage::new("views.git_config_view.not_set_at_level", "此层级未设置", "Not set at this level");
pub const SOURCE_GLOBAL: UiMessage = UiMessage::new("views.git_config_view.source_global", "全局", "global");
pub const SOURCE_LOCAL: UiMessage = UiMessage::new("views.git_config_view.source_local", "本仓库", "this repository");
pub const SOURCE_SYSTEM: UiMessage = UiMessage::new("views.git_config_view.source_system", "系统", "system");
pub const SOURCE_OTHER: UiMessage = UiMessage::new("views.git_config_view.source_other", "其他层级", "another level");

pub const EFFECTIVE_IDENTITY: UiMessage = UiMessage::new("views.git_config_view.effective_identity", "当前生效身份：{identity}", "Active identity: {identity}");
pub const IDENTITY_NOT_SET: UiMessage = UiMessage::new("views.git_config_view.identity_not_set", "尚未配置完整的提交身份，提交将使用回退身份 Khaslana。建议填写用户名和邮箱。", "A complete commit identity is not configured; commits will use the fallback identity Khaslana. Set a user name and email.");

pub const SAVE: UiMessage = UiMessage::new("views.git_config_view.save", "保存", "Save");
pub const GIT_CONFIG_SAVED: UiMessage = UiMessage::new("views.git_config_view.git_config_saved", "Git 配置已保存", "Git configuration saved");
pub const UNSAVED_CHANGES: UiMessage = UiMessage::new("views.git_config_view.unsaved_changes", "有未保存的改动", "Unsaved changes");
pub const NO_CHANGES: UiMessage = UiMessage::new("views.git_config_view.no_changes", "没有改动", "No changes");
pub const LOADING: UiMessage = UiMessage::new("views.git_config_view.loading", "加载中…", "Loading…");
pub const SAVING: UiMessage = UiMessage::new("views.git_config_view.saving", "保存中…", "Saving…");
pub const SAVE_SCOPE_GLOBAL: UiMessage = UiMessage::new("views.git_config_view.save_scope_global", "保存到全局配置，对所有仓库生效。留空保存将清除该层级的配置。", "Saves to global configuration, applying to all repositories. Saving an empty value clears it at this level.");
pub const SAVE_SCOPE_LOCAL: UiMessage = UiMessage::new("views.git_config_view.save_scope_local", "保存到当前仓库配置，仅对本仓库生效。留空保存将清除该层级的配置。", "Saves to the current repository's configuration only. Saving an empty value clears it at this level.");
pub const AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT: UiMessage = UiMessage::new("views.git_config_view.an_operation_is_in_progress_please_wait", "当前操作进行中，请稍候", "An operation is in progress. Please wait.");

pub const USER_NAME_PLACEHOLDER: UiMessage = UiMessage::new("views.git_config_view.user_name_placeholder", "例如：张三", "e.g. Jane Doe");
pub const USER_EMAIL_PLACEHOLDER: UiMessage = UiMessage::new("views.git_config_view.user_email_placeholder", "例如：zhangsan@example.com", "e.g. jane@example.com");
pub const DEFAULT_BRANCH_PLACEHOLDER: UiMessage = UiMessage::new("views.git_config_view.default_branch_placeholder", "例如：main", "e.g. main");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    SCOPE,
    SCOPE_HELP,
    GLOBAL,
    CURRENT_REPO,
    NO_REPOSITORY_OPEN,
    IDENTITY_GROUP,
    IDENTITY_GROUP_DESC,
    FILES_GROUP,
    FILES_GROUP_DESC,
    BRANCH_GROUP,
    BRANCH_GROUP_DESC,
    USER_NAME,
    USER_NAME_DESC,
    USER_EMAIL,
    USER_EMAIL_DESC,
    AUTOCRLF,
    AUTOCRLF_DESC,
    QUOTEPATH,
    QUOTEPATH_DESC,
    LONGPATHS,
    LONGPATHS_DESC,
    DEFAULT_BRANCH,
    DEFAULT_BRANCH_DESC,
    PULL_REBASE,
    PULL_REBASE_DESC,
    FETCH_PRUNE,
    FETCH_PRUNE_DESC,
    REBASE_AUTOSTASH,
    REBASE_AUTOSTASH_DESC,
    UNSET,
    ON,
    OFF,
    INHERITED_FROM,
    NOT_SET_AT_LEVEL,
    SOURCE_GLOBAL,
    SOURCE_LOCAL,
    SOURCE_SYSTEM,
    SOURCE_OTHER,
    EFFECTIVE_IDENTITY,
    IDENTITY_NOT_SET,
    SAVE,
    GIT_CONFIG_SAVED,
    UNSAVED_CHANGES,
    NO_CHANGES,
    LOADING,
    SAVING,
    SAVE_SCOPE_GLOBAL,
    SAVE_SCOPE_LOCAL,
    AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT,
    USER_NAME_PLACEHOLDER,
    USER_EMAIL_PLACEHOLDER,
    DEFAULT_BRANCH_PLACEHOLDER,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
