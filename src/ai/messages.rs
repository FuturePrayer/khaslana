//! AI 与扩展宿主的应用消息。原始供应商及用户数据只放入参数，不做翻译。

macro_rules! ai_error {
    ($key:literal, $zh:literal, $en:literal $(, $name:literal => $value:expr)* $(,)?) => {
        crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(
            crate::ui_i18n::UiMessage::new($key, $zh, $en),
            &[$(($name, &$value.to_string())),*],
        ))
    };
}
pub(crate) use ai_error;

macro_rules! ai_text {
    ($key:literal, $zh:literal, $en:literal $(, $name:literal => $value:expr)* $(,)?) => {
        crate::ui_i18n::format(crate::ui_i18n::UiMessage::new($key, $zh, $en),
            &[$(($name, &$value.to_string())),*])
    };
}
pub(crate) use ai_text;

macro_rules! ai_text_in {
    ($language:expr, $key:literal, $zh:literal, $en:literal $(, $name:literal => $value:expr)* $(,)?) => {
        crate::ui_i18n::format_in($language, crate::ui_i18n::UiMessage::new($key, $zh, $en),
            &[$(($name, &$value.to_string())),*])
    };
}
pub(crate) use ai_text_in;
