//! 应用 UI 的稳定消息身份与语言桥接；用户数据只作为参数，不参与资源查找。
use std::sync::atomic::{AtomicBool, Ordering};
use crate::language::Language;

static ENGLISH: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiMessage {
    pub key: &'static str,
    pub zh: &'static str,
    pub en: &'static str,
}
impl UiMessage {
    pub const fn new(key: &'static str, zh: &'static str, en: &'static str) -> Self {
        Self { key, zh, en }
    }
}
pub fn set_language(language: Language) {
    ENGLISH.store(language == Language::En, Ordering::Relaxed);
}
pub fn language() -> Language {
    if ENGLISH.load(Ordering::Relaxed) { Language::En } else { Language::ZhCn }
}
pub fn text(message: UiMessage) -> &'static str { text_in(language(), message) }
pub fn text_in(language: Language, message: UiMessage) -> &'static str {
    if language == Language::En && !message.en.is_empty() { message.en } else { message.zh }
}
pub fn format(message: UiMessage, args: &[(&str, &str)]) -> String {
    format_in(language(), message, args)
}
pub fn format_in(language: Language, message: UiMessage, args: &[(&str, &str)]) -> String {
    let mut rest = text_in(language, message);
    let mut result = String::with_capacity(rest.len());
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}') else { result.push_str(&rest[start..]); return result; };
        let name = &rest[start + 1..start + end];
        if let Some((_, value)) = args.iter().find(|(key, _)| *key == name) { result.push_str(value); }
        else { result.push_str(&rest[start..=start + end]); }
        rest = &rest[start + end + 1..];
    }
    result.push_str(rest);
    result
}

/// 长驻状态保留资源身份及原始参数，切换语言不重启任务、不改写用户数据。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalizedMessage {
    pub message: UiMessage,
    pub args: Vec<(String, String)>,
}
impl LocalizedMessage {
    pub fn new(message: UiMessage, args: &[(&str, &str)]) -> Self {
        Self { message, args: args.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect() }
    }
    pub fn resolve(&self) -> String {
        let args: Vec<_> = self.args.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect();
        format(self.message, &args)
    }
}

/// 只识别应用显式封装的结构化状态；普通用户数据和外部错误原样返回。
const MESSAGE_PREFIX: &str = "\u{1e}khaslana-ui:";
#[derive(serde::Serialize, serde::Deserialize)]
struct StoredMessage {
    key: String,
    zh: String,
    en: String,
    args: Vec<(String, String)>,
    #[serde(default)]
    messages: Vec<(String, String)>,
}
pub fn encode(message: UiMessage, args: &[(&str, &str)]) -> String {
    let stored = StoredMessage { key: message.key.into(), zh: message.zh.into(), en: message.en.into(),
        args: args.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect(), messages: Vec::new() };
    std::format!("{MESSAGE_PREFIX}{}", serde_json::to_string(&stored).expect("UI 状态只包含字符串"))
}
/// 只有调用方明确声明的应用消息参数可重译；普通 data 参数永不解析。
pub fn encode_with_messages(message: UiMessage, data: &[(&str, &str)], messages: &[(&str, &str)]) -> String {
    let stored = StoredMessage { key: message.key.into(), zh: message.zh.into(), en: message.en.into(),
        args: data.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect(),
        messages: messages.iter().map(|(key, value)| ((*key).into(), (*value).into())).collect() };
    std::format!("{MESSAGE_PREFIX}{}", serde_json::to_string(&stored).expect("UI 状态只包含字符串"))
}
pub fn message_key(value: &str) -> Option<String> {
    serde_json::from_str::<StoredMessage>(value.strip_prefix(MESSAGE_PREFIX)?).ok().map(|message| message.key)
}
pub fn resolve(value: &str) -> String { resolve_in(language(), value) }
pub fn resolve_in(language: Language, value: &str) -> String { resolve_message(language, value, 0) }
fn resolve_message(language: Language, value: &str, depth: usize) -> String {
    let Some(stored) = value.strip_prefix(MESSAGE_PREFIX).and_then(|json| serde_json::from_str::<StoredMessage>(json).ok()) else { return value.into(); };
    let template = if language == Language::En && !stored.en.is_empty() { &stored.en } else { &stored.zh };
    // 递归仅限显式消息槽；用户数据即使含结构前缀也不解释。深度限制防异常状态膨胀。
    let nested: Vec<_> = stored.messages.iter().map(|(key, value)| (key.as_str(),
        if depth < 8 { resolve_message(language, value, depth + 1) } else { String::new() })).collect();
    let mut args: Vec<_> = stored.args.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect();
    args.extend(nested.iter().map(|(key, value)| (*key, value.as_str())));
    interpolate(template, &args)
}
fn interpolate(mut rest: &str, args: &[(&str, &str)]) -> String {
    let mut result = String::with_capacity(rest.len());
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        let Some(end) = rest[start..].find('}') else { result.push_str(&rest[start..]); return result; };
        let name = &rest[start + 1..start + end];
        if let Some((_, value)) = args.iter().find(|(key, _)| *key == name) { result.push_str(value); }
        else { result.push_str(&rest[start..=start + end]); }
        rest = &rest[start + end + 1..];
    }
    result.push_str(rest);
    result
}

/// 每个资源模块可用其注册表调用此校验，确保 key 唯一及双语具名参数一致。
pub fn validate(messages: &[UiMessage]) -> Result<(), String> {
    let mut keys = std::collections::HashSet::new();
    fn parameters(text: &str) -> std::collections::BTreeSet<&str> {
        text.split('{').skip(1).filter_map(|part| part.split_once('}').map(|(name, _)| name)).collect()
    }
    for message in messages {
        if !keys.insert(message.key) { return Err(std::format!("duplicate UI key: {}", message.key)); }
        if message.en.is_empty() || message.zh.is_empty() { return Err(std::format!("missing UI translation: {}", message.key)); }
        if parameters(message.zh) != parameters(message.en) { return Err(std::format!("UI parameter mismatch: {}", message.key)); }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/ui_i18n.rs"]
mod tests;
