//! 设置中心和共享外观的领域资源；key 不依赖显示文案。
use khaslana::ui_i18n::UiMessage;

pub const CREDENTIALS: UiMessage = UiMessage::new("settings.credentials.title", "凭据管理", "Credentials");
pub const CREDENTIALS_DESCRIPTION: UiMessage = UiMessage::new("settings.credentials.description", "各远端的用户名与访问令牌；密文保存在系统密钥库。", "Usernames and access tokens for remotes. Secrets are stored in the system keyring.");
pub const PROXY: UiMessage = UiMessage::new("settings.proxy.title", "网络代理", "Network proxy");
pub const PROXY_DESCRIPTION: UiMessage = UiMessage::new("settings.proxy.description", "Git 网络操作走哪条代理链路。配置错误会直接报错，不会静默改走直连。", "Configure the proxy for Git network operations. Invalid settings report an error instead of silently connecting directly.");
pub const AI: UiMessage = UiMessage::new("settings.ai.title", "AI 设置", "AI settings");
pub const AI_DESCRIPTION: UiMessage = UiMessage::new("settings.ai.description", "配置 AI 服务连接、Skill、MCP 与浏览器运行环境。", "Configure AI connections, Skills, MCP, and the browser runtime.");
pub const MERGE: UiMessage = UiMessage::new("settings.merge.title", "合并工具", "Merge tool");
pub const MERGE_DESCRIPTION: UiMessage = UiMessage::new("settings.merge.description", "把冲突文件交给外部合并工具处理，并选择需要处理的文件范围。", "Resolve conflicts with an external merge tool and choose which files to process.");
pub const INDEX: UiMessage = UiMessage::new("settings.index.title", "代码索引", "Code index");
pub const INDEX_DESCRIPTION: UiMessage = UiMessage::new("settings.index.description", "为每个仓库建立本机代码知识图谱，供 AI 与 MCP 工具使用。", "Build a local code knowledge graph for each repository for AI and MCP tools.");
pub const APPEARANCE: UiMessage = UiMessage::new("settings.appearance.title", "外观", "Appearance");
pub const APPEARANCE_DESCRIPTION: UiMessage = UiMessage::new("settings.appearance.description", "主题与界面显示偏好。改动即时生效。", "Theme and display preferences. Changes apply immediately.");
pub const UPDATE: UiMessage = UiMessage::new("settings.update.title", "更新设置", "Updates");
pub const UPDATE_DESCRIPTION: UiMessage = UiMessage::new("settings.update.description", "检查新版本与发布渠道；迁移、移动应用数据目录也在这里。", "Check versions and release channels, or migrate and move application data.");
pub const SHORTCUTS: UiMessage = UiMessage::new("settings.shortcuts.title", "快捷键", "Keyboard shortcuts");
pub const SHORTCUTS_DESCRIPTION: UiMessage = UiMessage::new("settings.shortcuts.description", "点击「重新绑定」后按下组合键录入；按 Esc 取消录制。", "Choose Rebind, then press a key combination. Press Esc to cancel recording.");
pub const ABOUT: UiMessage = UiMessage::new("settings.about.title", "关于", "About");
pub const ABOUT_DESCRIPTION: UiMessage = UiMessage::new("settings.about.description", "版本号、发布渠道与版本说明。", "Version, release channel, and release notes.");
pub const DISPLAY_MODE: UiMessage = UiMessage::new("appearance.display_mode", "显示模式", "Display mode");
pub const DISPLAY_DESCRIPTION: UiMessage = UiMessage::new("appearance.display_description", "选择应用主题。跟随系统会在操作系统外观变化时自动切换。", "Choose the application theme. System follows changes to your operating system appearance.");
pub const THEME_MODE: UiMessage = UiMessage::new("appearance.theme_mode", "主题模式", "Theme mode");
pub const ACCENT: UiMessage = UiMessage::new("appearance.accent", "主题色", "Theme color");
pub const ACCENT_DESCRIPTION: UiMessage = UiMessage::new("appearance.accent_description", "主题色影响按钮、选中态、链接和进度条等强调色。", "The theme color accents buttons, selections, links, and progress bars.");
pub const ACCENT_LABEL: UiMessage = UiMessage::new("appearance.accent_label", "强调色", "Accent color");
pub const CURRENT_DISPLAY: UiMessage = UiMessage::new("appearance.current_display", "当前显示", "Current appearance");
pub const CURRENT_THEME: UiMessage = UiMessage::new("appearance.current_theme", "当前生效主题", "Active theme");
pub const SYSTEM: UiMessage = UiMessage::new("appearance.system", "跟随系统", "System");
pub const LIGHT: UiMessage = UiMessage::new("appearance.light", "浅色", "Light");
pub const DARK: UiMessage = UiMessage::new("appearance.dark", "深色", "Dark");
pub const LIGHT_THEME: UiMessage = UiMessage::new("appearance.light_theme", "浅色主题", "Light theme");
pub const DARK_THEME: UiMessage = UiMessage::new("appearance.dark_theme", "深色主题", "Dark theme");
pub const THEME_CHANGED: UiMessage = UiMessage::new("appearance.changed", "外观已切换为{theme}", "Appearance changed to {theme}");
pub const THEME_SAVE_FAILED: UiMessage = UiMessage::new("appearance.save_failed", "主题偏好保存失败：{error}", "Could not save theme preferences: {error}");
pub const ACCENT_CHANGED: UiMessage = UiMessage::new("appearance.accent_changed", "主题色已切换为{color}", "Theme color changed to {color}");
pub const ACCENT_SAVE_FAILED: UiMessage = UiMessage::new("appearance.accent_save_failed", "主题色保存失败：{error}", "Could not save theme color: {error}");

pub const ACCENTS: &[UiMessage] = &[
    UiMessage::new("appearance.accent.indigo", "靛蓝", "Indigo"),
    UiMessage::new("appearance.accent.violet", "紫罗兰", "Violet"),
    UiMessage::new("appearance.accent.rose", "玫红", "Rose"),
    UiMessage::new("appearance.accent.orange", "橙", "Orange"),
    UiMessage::new("appearance.accent.cyan", "青", "Cyan"),
    UiMessage::new("appearance.accent.emerald", "翠绿", "Emerald"),
    UiMessage::new("appearance.accent.graphite", "石墨", "Graphite"),
    UiMessage::new("appearance.accent.gold_brown", "金棕", "Golden brown"),
    UiMessage::new("appearance.accent.sky_blue", "天蓝", "Sky blue"),
];

#[cfg(test)]
#[test]
fn resources_have_unique_keys_and_matching_parameters() {
    khaslana::ui_i18n::validate(&[CREDENTIALS, CREDENTIALS_DESCRIPTION, PROXY, PROXY_DESCRIPTION, AI, AI_DESCRIPTION, MERGE, MERGE_DESCRIPTION, INDEX, INDEX_DESCRIPTION, APPEARANCE, APPEARANCE_DESCRIPTION, UPDATE, UPDATE_DESCRIPTION, SHORTCUTS, SHORTCUTS_DESCRIPTION, ABOUT, ABOUT_DESCRIPTION, DISPLAY_MODE, DISPLAY_DESCRIPTION, THEME_MODE, ACCENT, ACCENT_DESCRIPTION, ACCENT_LABEL, CURRENT_DISPLAY, CURRENT_THEME, SYSTEM, LIGHT, DARK, LIGHT_THEME, DARK_THEME, THEME_CHANGED, THEME_SAVE_FAILED, ACCENT_CHANGED, ACCENT_SAVE_FAILED, ACCENTS[0], ACCENTS[1], ACCENTS[2], ACCENTS[3], ACCENTS[4], ACCENTS[5], ACCENTS[6], ACCENTS[7], ACCENTS[8]]).unwrap();
}
