use super::*;

#[test]
fn resources_are_complete_and_parameters_match() {
    for key in ALL_KEYS {
        let zh = template(Language::ZhCn, *key);
        let en = template(Language::En, *key);
        assert!(!zh.is_empty());
        assert!(!en.is_empty());
        let parameters = |text: &str| -> Vec<String> {
            text.split('{').skip(1).filter_map(|part| part.split_once('}'))
                .map(|(name, _)| name.to_owned()).collect()
        };
        assert_eq!(parameters(zh), parameters(en), "{key:?}");
    }
}

#[test]
fn arguments_are_not_translated_or_recursively_interpolated() {
    let error = "用户数据 {error} C:/工作区/code.rs";
    assert_eq!(format_message(Language::En, MessageKey::SaveFailed, &[("error", error)]),
        format!("Could not save language preferences: {error}"));
}
