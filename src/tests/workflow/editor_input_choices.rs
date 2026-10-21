use super::*;

#[test]
fn input_kind_ids_are_stable_and_independent_of_translated_titles() {
    for (id, kind) in [("text", WorkflowInputKind::Text), ("textarea", WorkflowInputKind::Textarea), ("select", WorkflowInputKind::Select), ("date", WorkflowInputKind::Date)] {
        assert_eq!(kind_id(kind), id);
        assert_eq!(kind_from_id(id), Some(kind));
        let chinese = WorkflowInputKindOption { id, title: "中文标题".into() };
        let english = WorkflowInputKindOption { id, title: "English title".into() };
        assert_eq!(chinese.value(), english.value());
        assert_ne!(chinese.title(), english.title());
    }
    assert_eq!(kind_from_id("单行文本"), None);
    assert_eq!(kind_from_id("Single-line text"), None);
}
