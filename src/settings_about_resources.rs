//! 关于页面的稳定文案资源；发布说明正文与链接地址保持原始数据。
use khaslana::ui_i18n::UiMessage;

pub const GITHUB_REPOSITORY: UiMessage = UiMessage::new("settings.about.github_repository", "GitHub 仓库", "GitHub repository");
pub const GITHUB_DESCRIPTION: UiMessage = UiMessage::new("settings.about.github_description", "查看项目源码、提交 Issue 与参与协作", "Browse the source, report issues, and contribute");
pub const LICENSE: UiMessage = UiMessage::new("settings.about.license", "开源协议 Apache-2.0", "Apache-2.0 license");
pub const LICENSE_DESCRIPTION: UiMessage = UiMessage::new("settings.about.license_description", "查看许可证内容与使用条款", "Read the license and terms of use");
pub const FEEDBACK: UiMessage = UiMessage::new("settings.about.feedback", "问题反馈", "Report an issue");
pub const SUBMIT_ISSUE: UiMessage = UiMessage::new("settings.about.submit_issue", "提交 Issue", "Submit an issue");
pub const COMMUNITY: UiMessage = UiMessage::new("settings.about.community", "社区讨论", "Community");
pub const DISCUSSIONS: UiMessage = UiMessage::new("settings.about.discussions", "GitHub Discussions", "GitHub Discussions");
pub const NO_RELEASE_NOTES: UiMessage = UiMessage::new("settings.about.no_release_notes", "（此版本未附版本说明）", "(No release notes included for this version)");
pub const OPEN_LINK: UiMessage = UiMessage::new("settings.about.open_link", "{title}（在默认浏览器中打开）", "{title} (open in default browser)");
pub const OPEN_LINK_TOOLTIP: UiMessage = UiMessage::new("settings.about.open_link_tooltip", "在默认浏览器中打开：{url}", "Open in default browser: {url}");
pub const VERSION: UiMessage = UiMessage::new("settings.about.version", "版本", "Version");
pub const VERSION_DESCRIPTION: UiMessage = UiMessage::new("settings.about.version_description", "更新渠道与自动检查可在「更新设置」中配置。", "Configure update channels and automatic checks in Update settings.");
pub const PROJECT_LINKS: UiMessage = UiMessage::new("settings.about.project_links", "项目链接", "Project links");
pub const PROJECT_DESCRIPTION: UiMessage = UiMessage::new("settings.about.project_description", "开源项目，欢迎 Star 与参与贡献。", "An open-source project. Stars and contributions are welcome.");
pub const OTHER_INFORMATION: UiMessage = UiMessage::new("settings.about.other_information", "其他信息", "Other information");
pub const CURRENT_VERSION: UiMessage = UiMessage::new("settings.about.current_version", "当前版本", "Version");
pub const RELEASE_CHANNEL: UiMessage = UiMessage::new("settings.about.release_channel", "发布渠道", "Channel");
pub const RUNTIME: UiMessage = UiMessage::new("settings.about.runtime", "运行环境", "Platform");

// 检索词保留双语，不随当前界面语言变化，且不参与渲染或业务判断。
pub const VERSION_KEYWORDS: &[&str] = &["版本", "版本说明", "更新设置", "正式版", "测试版", "version", "release notes", "update settings", "stable", "beta"];
pub const PROJECT_KEYWORDS: &[&str] = &["项目链接", "GitHub", "开源协议", "Apache-2.0", "project links", "license"];
pub const INFORMATION_KEYWORDS: &[&str] = &["其他信息", "系统", "架构", "运行环境", "问题反馈", "社区讨论", "information", "system", "architecture", "platform", "issues", "community"];
