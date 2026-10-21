//! 语言偏好、系统语言匹配与任务语言快照。系统调用与匹配算法分离。

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    ZhCn,
    En,
}

impl Language {
    pub const fn kit_locale(self) -> &'static str {
        match self { Self::ZhCn => "zh-CN", Self::En => "en" }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiLanguagePreference {
    #[default]
    System,
    ZhCn,
    En,
}

impl UiLanguagePreference {
    pub const fn id(self) -> &'static str {
        match self { Self::System => "system", Self::ZhCn => "zh-CN", Self::En => "en" }
    }
    pub fn from_id(value: &str) -> Self {
        match value { "zh-CN" => Self::ZhCn, "en" => Self::En, _ => Self::System }
    }
    pub fn resolve(self) -> Language {
        match self {
            Self::System => preferred_language(system_preferred_languages().iter().map(String::as_str)),
            Self::ZhCn => Language::ZhCn,
            Self::En => Language::En,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PromptLanguagePreference {
    #[default]
    FollowUi,
    ZhCn,
    En,
}

impl PromptLanguagePreference {
    pub const fn id(self) -> &'static str {
        match self { Self::FollowUi => "follow-ui", Self::ZhCn => "zh-CN", Self::En => "en" }
    }
    pub fn from_id(value: &str) -> Self {
        match value { "zh-CN" => Self::ZhCn, "en" => Self::En, _ => Self::FollowUi }
    }
    pub const fn resolve(self, ui: Language) -> Language {
        match self { Self::FollowUi => ui, Self::ZhCn => Language::ZhCn, Self::En => Language::En }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LanguagePreferences {
    pub ui: UiLanguagePreference,
    pub prompt: PromptLanguagePreference,
}

/// 在任务启动处构造后按值传入后台；不得在重试或分段时重读全局偏好。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskLanguages {
    pub system: Language,
    pub output: Language,
}

impl TaskLanguages {
    pub const fn new(ui: Language, prompt: PromptLanguagePreference) -> Self {
        Self { system: prompt.resolve(ui), output: ui }
    }
    pub const fn output_instruction(self) -> &'static str {
        match (self.system, self.output) {
            (Language::En, Language::En) => "Write all natural-language output in English. Do not translate code, identifiers, paths, or JSON contract keys.",
            (Language::En, Language::ZhCn) => "Write all natural-language output in Simplified Chinese, even though these system instructions are in English. Do not translate code, identifiers, paths, or JSON contract keys.",
            (Language::ZhCn, Language::En) => "所有自然语言结果必须使用英语。不要翻译代码、标识符、路径或 JSON 契约键。",
            (Language::ZhCn, Language::ZhCn) => "所有自然语言结果必须使用简体中文。不要翻译代码、标识符、路径或 JSON 契约键。",
        }
    }
}

/// 仅匹配明确支持的语言；繁体（包括显式 Hant）不能误判为简体。
pub fn supported_locale(locale: &str) -> Option<Language> {
    let locale = locale.trim().split(['.', '@']).next().unwrap_or_default();
    let normalized = locale.replace('_', "-").to_ascii_lowercase();
    let parts: Vec<_> = normalized.split('-').collect();
    match parts.first().copied()? {
        "en" => Some(Language::En),
        "zh" if !parts.iter().any(|p| matches!(*p, "hant" | "tw" | "hk" | "mo"))
            && parts.iter().any(|p| matches!(*p, "hans" | "cn" | "sg")) => Some(Language::ZhCn),
        _ => None,
    }
}

pub fn preferred_language<'a>(locales: impl IntoIterator<Item = &'a str>) -> Language {
    locales.into_iter().find_map(supported_locale).unwrap_or_default()
}

#[cfg(windows)]
pub fn system_preferred_languages() -> Vec<String> {
    // 使用用户 UI 首选列表，而不是格式区域或进程环境变量。
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserPreferredUILanguages(flags: u32, count: *mut u32, buffer: *mut u16, size: *mut u32) -> i32;
    }
    let mut count = 0;
    let mut size = 0;
    // MUI_LANGUAGE_NAME = 8；先查询长度，再读取双 NUL 结尾的 MULTI_SZ。
    unsafe {
        if GetUserPreferredUILanguages(8, &mut count, std::ptr::null_mut(), &mut size) == 0 || size == 0 {
            return Vec::new();
        }
        let mut buffer = vec![0u16; size as usize];
        if GetUserPreferredUILanguages(8, &mut count, buffer.as_mut_ptr(), &mut size) == 0 {
            return Vec::new();
        }
        buffer[..(size as usize).min(buffer.len())].split(|c| *c == 0)
            .take_while(|part| !part.is_empty()).map(String::from_utf16_lossy).collect()
    }
}

#[cfg(target_os = "macos")]
pub fn system_preferred_languages() -> Vec<String> {
    let Ok(output) = std::process::Command::new("/usr/bin/defaults")
        .args(["read", "-g", "AppleLanguages"])
        .output()
    else { return Vec::new() };
    if !output.status.success() { return Vec::new(); }
    String::from_utf8_lossy(&output.stdout).lines()
        .map(|line| line.trim().trim_end_matches(',').trim_matches('"'))
        .filter(|line| !line.is_empty() && !matches!(*line, "(" | ")"))
        .map(str::to_owned).collect()
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn system_preferred_languages() -> Vec<String> {
    // LANGUAGE 是桌面会话的有序语言偏好；未设置时遵循 POSIX locale 优先级。
    if let Ok(value) = std::env::var("LANGUAGE") {
        let languages: Vec<_> = value.split(':').filter(|v| !v.trim().is_empty())
            .map(str::to_owned).collect();
        if !languages.is_empty() { return languages; }
    }
    ["LC_ALL", "LC_MESSAGES", "LANG"].into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .find(|value| !value.trim().is_empty())
        .into_iter().collect()
}

#[cfg(test)]
#[path = "tests/language.rs"]
mod tests;
