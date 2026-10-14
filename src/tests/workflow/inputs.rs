use super::*;
use crate::parse_workflow_json5;

fn input(source: &str) -> WorkflowInputDefinition {
    parse_workflow_json5(&format!("{{version:1,inputs:{{field:{source}}},steps:[{{op:'ensureClean'}}]}}"))
        .unwrap().inputs.remove("field").unwrap()
}

#[test]
fn legacy_input_stays_text_and_serializes_without_new_fields() {
    let definition = input("{label:'名称',default:'旧值'}");
    assert_eq!(definition.kind, WorkflowInputKind::Text);
    assert!(definition.options.is_empty());
    let value = serde_json::to_value(definition).unwrap();
    assert!(value.get("type").is_none());
    assert!(value.get("options").is_none());
}

#[test]
fn selection_uses_values_not_labels_and_preserves_option_order() {
    let definition = input("{type:'select',options:[{label:'验证环境',value:'uat'},{label:'生产环境',value:'prod'}]}");
    assert_eq!(definition.options[0].value, "uat");
    assert!(definition.validate_value("field", "").is_err());
    assert!(definition.validate_value("field", "uat").is_ok());
    assert!(definition.validate_value("field", "验证环境").is_err());
    assert!(definition.validate_value("field", "unknown").is_err());
}

#[test]
fn date_requires_a_real_iso_calendar_date() {
    let definition = input("{type:'date',required:false}");
    for valid in ["", "2024-02-29", "2026-10-12"] { assert!(definition.validate_value("field", valid).is_ok()); }
    for invalid in ["2025-02-29", "2026-13-01", "2026-1-01", "2026-10-12 ", "12/10/2026"] {
        assert!(definition.validate_value("field", invalid).is_err(), "{invalid}");
    }
}

#[test]
fn invalid_option_definitions_and_defaults_are_rejected() {
    for source in ["{type:'unknown'}", "{type:'select'}", "{type:'select',options:[{label:'',value:'a'}]}",
        "{type:'select',options:[{label:'A',value:'a'},{label:'B',value:'a'}]}",
        "{type:'select',default:'b',options:[{label:'A',value:'a'}]}", "{type:'date',default:'2026-02-30'}"] {
        assert!(parse_workflow_json5(&format!("{{version:1,inputs:{{field:{source}}},steps:[{{op:'ensureClean'}}]}}")).is_err());
    }
}

#[test]
fn multiline_and_expression_defaults_roundtrip_losslessly() {
    let definition = input("{type:'textarea',default:'  第一行\\n第二行  '}");
    assert_eq!(definition.default.as_deref(), Some("  第一行\n第二行  "));
    let decoded: WorkflowInputDefinition = serde_json::from_value(serde_json::to_value(&definition).unwrap()).unwrap();
    assert_eq!(decoded, definition);
    assert!(input("{type:'date',default:'${date:%Y-%m-%d}'}").validate_value("field", "invalid").is_err());
}
