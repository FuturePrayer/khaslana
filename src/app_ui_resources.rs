//! 应用命令、数据展示与凭据守卫的稳定资源。
use khaslana::ui_i18n::UiMessage;
pub const REFRESH: UiMessage = UiMessage::new("commands.refresh", "刷新", "Refresh");
pub const FETCH: UiMessage = UiMessage::new("commands.fetch", "获取", "Fetch");
pub const PULL: UiMessage = UiMessage::new("commands.pull", "拉取", "Pull");
pub const PUSH: UiMessage = UiMessage::new("commands.push", "推送", "Push");
pub const STASH: UiMessage = UiMessage::new("commands.stash", "贮藏", "Stash");
pub const SUBMODULE: UiMessage = UiMessage::new("commands.submodule", "子模块", "Submodules");
pub const SETTINGS: UiMessage = UiMessage::new("commands.settings", "设置", "Settings");
pub const WORKTREE: UiMessage = UiMessage::new("commands.worktree", "工作区", "Worktree");
pub const HISTORY: UiMessage = UiMessage::new("commands.history", "提交记录", "History");
pub const WORKFLOW: UiMessage = UiMessage::new("commands.workflow", "工作流", "Workflows");
pub const EXPLORER: UiMessage = UiMessage::new("commands.explorer", "在资源管理器中打开仓库", "Open repository in file manager");
pub const BROWSER: UiMessage = UiMessage::new("commands.browser", "以浏览器打开当前远端", "Open current remote in browser");
pub const SEARCH: UiMessage = UiMessage::new("commands.search", "全局搜索", "Global search");
pub const WORKFLOW_CONFLICT: UiMessage = UiMessage::new("shortcuts.workflow_conflict", "工作流 {file}", "Workflow {file}");
pub const BRANCH_TARGET: UiMessage = UiMessage::new("checkout.branch_target", "分支 {name}", "Branch {name}");
pub const REMOTE_TARGET: UiMessage = UiMessage::new("checkout.remote_target", "远端分支 {name}", "Remote branch {name}");
pub const TAG_TARGET: UiMessage = UiMessage::new("checkout.tag_target", "标签 {name}", "Tag {name}");
pub const MORE_FILES: UiMessage = UiMessage::new("files.more", "... 以及另外 {count} 个文件", "... and {count} more files");
pub const READY: UiMessage = UiMessage::new("app.ready", "就绪", "Ready");
pub const UNNAMED_REPOSITORY: UiMessage = UiMessage::new("repository.unnamed", "未命名仓库", "Unnamed repository");
pub const GITEE_SAVE_FAILED: UiMessage = UiMessage::new("credentials.gitee.renewal_save_failed", "Gitee 令牌已续期，但续期材料保存失败（{error}）；请重新登录 Gitee 恢复自动续期", "Gitee token renewed, but renewal data could not be saved ({error}). Sign in to Gitee again to restore automatic renewal.");
pub const GITEE_RENEWED: UiMessage = UiMessage::new("credentials.gitee.renewed", "Gitee 令牌已自动续期", "Gitee token renewed automatically");
pub const GITEE_RENEW_FAILED: UiMessage = UiMessage::new("credentials.gitee.renew_failed", "Gitee 令牌自动续期失败：{error}；请重新登录 Gitee 更新凭据", "Gitee token renewal failed: {error}. Sign in to Gitee again to update credentials.");
pub const CREDENTIAL_CANCELLED: UiMessage = UiMessage::new("credentials.input.cancelled", "凭据输入已取消", "Credential entry cancelled");
pub const TEST_URL_REQUIRED: UiMessage = UiMessage::new("credentials.test.url_required", "需要填写测试地址", "Enter a test URL");
pub const TEST_HTTPS: UiMessage = UiMessage::new("credentials.test.https_required", "该记录是 HTTPS 凭据，测试地址必须是 http(s) 地址", "This record contains HTTPS credentials; the test URL must use HTTP or HTTPS");
pub const TEST_SSH: UiMessage = UiMessage::new("credentials.test.ssh_required", "该记录是 SSH 凭据，测试地址必须是 SSH 地址（git@主机:仓库 或 ssh://）", "This record contains SSH credentials; use an SSH address (git@host:repository or ssh://)");
pub const TEST_HOST: UiMessage = UiMessage::new("credentials.test.host_mismatch", "该凭据绑定 {record_host}，令牌不通用，不能用其它站点（{host}）的地址测试", "These credentials are bound to {record_host}. Tokens cannot be tested on another site ({host}).");
pub const TEST_URL_INVALID: UiMessage = UiMessage::new("credentials.test.url_invalid", "无法解析测试地址", "Could not parse the test URL");
pub const SHORTCUT_TAKEN: UiMessage = UiMessage::new("shortcuts.taken", "快捷键 {keystroke} 已被「{action}」占用", "Shortcut {keystroke} is already assigned to {action}");
pub const ENCODING_AUTO: UiMessage = UiMessage::new("diff.encoding.auto", "编码：自动({encoding})", "Encoding: Auto ({encoding})");
pub const ENCODING: UiMessage = UiMessage::new("diff.encoding.selected", "编码：{encoding}", "Encoding: {encoding}");
pub const ENCODING_REPLACED: UiMessage = UiMessage::new("diff.encoding.replaced", "{encoding}，有替换", "{encoding}, replacements present");
pub const UPDATE_CHECK_FAILED: UiMessage = UiMessage::new("update.check.failed", "检查更新失败：{error}", "Could not check for updates: {error}");
pub const UPDATE_NEW: UiMessage = UiMessage::new("update.new", "发现新版本 v{version}，点击查看更新", "New version v{version} available. Click to view update.");

#[cfg(test)]
#[test]
fn resources_have_unique_keys_and_matching_parameters() {
    khaslana::ui_i18n::validate(&[REFRESH, FETCH, PULL, PUSH, STASH, SUBMODULE, SETTINGS, WORKTREE, HISTORY, WORKFLOW, EXPLORER, BROWSER, SEARCH, WORKFLOW_CONFLICT, BRANCH_TARGET, REMOTE_TARGET, TAG_TARGET, MORE_FILES, READY, UNNAMED_REPOSITORY, GITEE_SAVE_FAILED, GITEE_RENEWED, GITEE_RENEW_FAILED, CREDENTIAL_CANCELLED, TEST_URL_REQUIRED, TEST_HTTPS, TEST_SSH, TEST_HOST, TEST_URL_INVALID, SHORTCUT_TAKEN, ENCODING_AUTO, ENCODING, ENCODING_REPLACED, UPDATE_CHECK_FAILED, UPDATE_NEW]).unwrap();
}
