//! 输入始终是字符串；控件类型只约束编辑和校验，不改变变量插值协议。
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
            Self::Text => "单行文本", Self::Textarea => "多行文本",
            Self::Select => "单选下拉", Self::Date => "日期",
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
                return Err(GitError::Message(format!("工作流下拉变量「{name}」至少需要一个选项")));
            }
            let mut values = BTreeSet::new();
            for option in &self.options {
                if option.label.trim().is_empty() || option.value.trim().is_empty()
                    || !values.insert(&option.value) {
                    return Err(GitError::Message(format!("工作流下拉变量「{name}」的选项名称或值为空，或值重复")));
                }
            }
        } else if !self.options.is_empty() {
            return Err(GitError::Message(format!("工作流变量「{name}」只有单选下拉可以配置 options")));
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
            return Err(GitError::Message(format!("请填写工作流变量：{label}")));
        }
        self.validate_nonempty_value(label, value)
    }

    fn validate_nonempty_value(&self, label: &str, value: &str) -> Result<()> {
        if value.trim().is_empty() { return Ok(()); }
        match self.kind {
            WorkflowInputKind::Select if !self.options.iter().any(|option| option.value == value) =>
                Err(GitError::Message(format!("请选择工作流变量「{label}」的有效选项"))),
            WorkflowInputKind::Date if NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .is_err() || value.len() != 10 || !value.as_bytes().iter().enumerate()
                    .all(|(i, b)| if i == 4 || i == 7 { *b == b'-' } else { b.is_ascii_digit() }) =>
                Err(GitError::Message(format!("工作流变量「{label}」必须是有效的 YYYY-MM-DD 日期"))),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "../tests/workflow/inputs.rs"]
mod tests;
