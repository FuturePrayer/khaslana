//! 各视图的独立资源模块复用同一约束测试，不依赖全局 UI 语言状态。
use super::ALL_MESSAGES;

#[test]
fn message_keys_and_parameters_are_consistent() {
    let mut keys = std::collections::BTreeSet::new();
    fn parameters(text: &str) -> std::collections::BTreeSet<&str> {
        text.split('{').skip(1)
            .filter_map(|part| part.split_once('}').map(|(name, _)| name))
            .collect()
    }
    for message in ALL_MESSAGES {
        assert!(keys.insert(message.key), "duplicate key: {}", message.key);
        assert!(!message.zh.is_empty() && !message.en.is_empty(), "{}", message.key);
        assert_eq!(parameters(message.zh), parameters(message.en), "{}", message.key);
        let names = parameters(message.zh);
        let args: Vec<_> = names.iter().map(|name| (*name, "用户/{literal}.rs")).collect();
        for language in [khaslana::language::Language::ZhCn, khaslana::language::Language::En] {
            let rendered = khaslana::ui_i18n::format_in(language, *message, &args);
            for name in &names {
                assert!(!rendered.contains(&format!("{{{name}}}")), "{}: {name}", message.key);
            }
        }
    }
}
