//! 输入始终是字符串；控件类型只约束编辑和校验，不改变变量插值协议。
#[path = "inputs_resources.rs"]
mod deep_messages;

use std::collections::BTreeSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::WorkflowInputDefinition;
use crate::{GitError, Result};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowInputKind {
    #[default]
    Text,
    Textarea,
    Select,
    Date,
}

impl WorkflowInputKind {
    pub fn is_text(&self) -> bool { *self == Self::Text }

    pub fn label(self) -> &'static str {
        match self {
            Self::Text => crate::ui_i18n::text(deep_messages::SINGLE_LINE_TEXT), Self::Textarea => crate::ui_i18n::text(deep_messages::MULTILINE_TEXT),
            Self::Select => crate::ui_i18n::text(deep_messages::SINGLE_SELECT_DROPDOWN), Self::Date => crate::ui_i18n::text(deep_messages::DATE),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct WorkflowInputOption {
    pub label: String,
    pub value: String,
}

impl WorkflowInputDefinition {
    pub(super) fn validate_definition(&self, name: &str) -> Result<()> {
        if self.kind == WorkflowInputKind::Select {
            if self.options.is_empty() {
                return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::WORKFLOW_DROPDOWN_VARIABLE_REQUIRES_AT_LEAST_ONE_OPTION, &[("name", &(name).to_string())])));
            }
            let mut values = BTreeSet::new();
            for option in &self.options {
                if option.label.trim().is_empty() || option.value.trim().is_empty()
                    || !values.insert(&option.value) {
                    return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::WORKFLOW_DROPDOWN_VARIABLE_HAS_AN_EMPTY_OPTION_LABEL_OR_VALUE_OR_DUPLICATE, &[("name", &(name).to_string())])));
                }
            }
        } else if !self.options.is_empty() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::ONLY_SINGLE_SELECT_WORKFLOW_VARIABLE_CAN_DEFINE_OPTIONS, &[("name", &(name).to_string())])));
        }
        // 含表达式的默认值必须在运行上下文展开后再检查。
        if let Some(value) = self.default.as_deref().filter(|value| !value.contains("${")) {
            self.validate_nonempty_value(name, value)?;
        }
        Ok(())
    }

    pub fn validate_value(&self, name: &str, value: &str) -> Result<()> {
        let label = self.label.as_deref().filter(|label| !label.trim().is_empty()).unwrap_or(name);
        if self.required && value.trim().is_empty() {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::ENTER_A_VALUE_FOR_WORKFLOW_VARIABLE, &[("label", &(label).to_string())])));
        }
        self.validate_nonempty_value(label, value)
    }

    fn validate_nonempty_value(&self, label: &str, value: &str) -> Result<()> {
        if value.trim().is_empty() { return Ok(()); }
        match self.kind {
            WorkflowInputKind::Select if !self.options.iter().any(|option| option.value == value) =>
                Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::SELECT_A_VALID_OPTION_FOR_WORKFLOW_VARIABLE, &[("label", &(label).to_string())]))),
            WorkflowInputKind::Date if NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .is_err() || value.len() != 10 || !value.as_bytes().iter().enumerate()
                    .all(|(i, b)| if i == 4 || i == 7 { *b == b'-' } else { b.is_ascii_digit() }) =>
                Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::WORKFLOW_VARIABLE_MUST_BE_A_VALID_YYYY_MM_DD_DATE, &[("label", &(label).to_string())]))),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "../tests/workflow/inputs.rs"]
mod tests;
