//! 稳定语义键资源。整句具名参数替换，不将用户数据再次送入翻译器。
use crate::language::Language;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKey {
    LanguageHeading,
    LanguageDescription,
    UiLanguage,
    PromptLanguage,
    PromptDescription,
    System,
    FollowUi,
    SimplifiedChinese,
    English,
    SaveFailed,
}

pub const ALL_KEYS: &[MessageKey] = &[
    MessageKey::LanguageHeading, MessageKey::LanguageDescription, MessageKey::UiLanguage,
    MessageKey::PromptLanguage, MessageKey::PromptDescription, MessageKey::System,
    MessageKey::FollowUi, MessageKey::SimplifiedChinese, MessageKey::English, MessageKey::SaveFailed,
];

pub fn template(language: Language, key: MessageKey) -> &'static str {
    use MessageKey::*;
    // 明确的中文基线同时作为缺失英文资源的 fallback；不使用中文原文作 key。
    let zh = match key {
        LanguageHeading => "语言",
        LanguageDescription => "选择界面语言。跟随系统会在启动和重新选择时检测用户首选语言。",
        UiLanguage => "界面语言",
        PromptLanguage => "AI 提示词语言",
        PromptDescription => "仅影响应用自有 system 提示词；自然语言结果始终使用任务启动时的界面语言。此偏好独立即选即存。",
        System => "跟随系统",
        FollowUi => "跟随界面",
        SimplifiedChinese => "简体中文",
        English => "English",
        SaveFailed => "语言偏好保存失败：{error}",
    };
    if language == Language::ZhCn { return zh; }
    match key {
        LanguageHeading => "Language",
        LanguageDescription => "Choose the interface language. System checks your preferred UI languages at startup and when selected again.",
        UiLanguage => "Interface language",
        PromptLanguage => "AI prompt language",
        PromptDescription => "Only changes application-owned system prompts. Natural-language results use the interface language captured when the task starts. Saved independently on selection.",
        System => "System",
        FollowUi => "Follow interface",
        SimplifiedChinese => "简体中文",
        English => "English",
        SaveFailed => "Could not save language preferences: {error}",
    }
}

/// 一次扫描资源模板，参数中的花括号不参与二次插值。
pub fn format_message(language: Language, key: MessageKey, args: &[(&str, &str)]) -> String {
    let text = template(language, key);
    let mut result = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}') else { result.push_str(&rest[start..]); return result; };
        let name = &rest[start + 1..start + end];
        if let Some((_, value)) = args.iter().find(|(key, _)| *key == name) {
            result.push_str(value);
        } else {
            result.push_str(&rest[start..=start + end]);
        }
        rest = &rest[start + end + 1..];
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
#[path = "tests/language_resources.rs"]
mod tests;
