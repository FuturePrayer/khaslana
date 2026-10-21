use std::collections::BTreeMap;
#[path = "v2_resources.rs"]
mod deep_messages;

use std::sync::Arc;

use git2::Repository;
use serde_json::{Map, Value};

use crate::{GitError, GitService, Result};

use super::{WorkflowRunControl, WorkflowStep};

/// 宿主动作的接口；具体 MCP、JS、Skill 适配器由后续模块显式注册。
pub trait WorkflowAction: Send + Sync {
    /// 工作流结束时关闭共享外部会话，清理问题作为提示而非操作失败。
    fn finish(&self) -> Vec<String> { Vec::new() }
    fn preview(
        &self,
        service: &GitService,
        repo: &Repository,
        arguments: &Value,
    ) -> Result<WorkflowActionPreview>;

    fn execute(
        &self,
        service: &GitService,
        repo: &mut Repository,
        arguments: &Value,
    ) -> Result<WorkflowActionResult>;

    /// 旧动作只在启动前检查取消；耗时的外部动作应覆盖此方法并在执行中检查。
    fn execute_with_control(
        &self,
        service: &GitService,
        repo: &mut Repository,
        arguments: &Value,
        control: &WorkflowRunControl,
    ) -> Result<WorkflowActionResult> {
        control.check_cancelled()?;
        self.execute(service, repo, arguments)
    }

    /// 长运行扩展动作可逐条回报审计明细，失败时已回报的内容仍保留在运行日志。
    fn execute_with_control_and_progress(
        &self,
        service: &GitService,
        repo: &mut Repository,
        arguments: &Value,
        control: &WorkflowRunControl,
        _progress: &mut dyn FnMut(String),
    ) -> Result<WorkflowActionResult> {
        self.execute_with_control(service, repo, arguments, control)
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowActionPreview {
    pub summary: String,
    pub details: Vec<String>,
    /// 只读且可确定的输出才在预览时提供。
    pub output: Option<Value>,
}

#[derive(Clone, Debug, Default)]
pub struct WorkflowActionResult {
    pub details: Vec<String>,
    pub output: Option<Value>,
}

#[derive(Default)]
pub struct WorkflowActionRegistry {
    actions: BTreeMap<String, Arc<dyn WorkflowAction>>,
}

impl WorkflowActionRegistry {
    pub fn finish(&self) -> Vec<String> {
        self.actions.values().flat_map(|action| action.finish()).collect()
    }
    pub fn register(&mut self, name: impl Into<String>, action: Arc<dyn WorkflowAction>) -> Result<()> {
        let name = name.into();
        if name.trim().is_empty() || name.starts_with("git.") {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::INVALID_WORKFLOW_ACTION_NAME, &[("name", &(name).to_string())])));
        }
        if self.actions.contains_key(&name) {
            return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::WORKFLOW_ACTION_REGISTERED_MORE_THAN_ONCE, &[("name", &(name).to_string())])));
        }
        self.actions.insert(name, action);
        Ok(())
    }

    pub(crate) fn get(&self, name: &str) -> Result<&dyn WorkflowAction> {
        self.actions
            .get(name)
            .map(Arc::as_ref)
            .ok_or_else(|| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::WORKFLOW_ACTION_IS_NOT_REGISTERED, &[("name", &(name).to_string())])))
    }
}

pub(super) fn empty_json_object() -> Value {
    Value::Object(Map::new())
}

/// 内置动作转换为已有 v1 步骤，以便共用同一套预览、GitService 与业务守卫。
pub(super) fn builtin_step(uses: &str, arguments: &Value) -> Result<Option<WorkflowStep>> {
    let op = match uses {
        "git.checkout" => "checkout",
        "git.fetch" => "fetch",
        "git.pull" => "pull",
        "git.createBranch" => "createBranch",
        "git.merge" => "merge",
        "git.push" => "push",
        "git.guardRemoteBranch" => "guardRemoteBranch",
        "git.ensureClean" => "ensureClean",
        "git.assertBranch" => "assertBranch",
        "git.filterBranches" => "filterBranches",
        "git.deleteBranches" => "deleteBranches",
        _ => return Ok(None),
    };
    let Value::Object(fields) = arguments else {
        return Err(GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::THE_WITH_ARGUMENT_OF_WORKFLOW_ACTION_MUST_BE_AN_OBJECT, &[("uses", &(uses).to_string())])));
    };
    let mut fields = fields.clone();
    fields.insert("op".into(), Value::String(op.into()));
    serde_json::from_value(Value::Object(fields))
        .map(Some)
        .map_err(|err| GitError::Localized(crate::ui_i18n::LocalizedMessage::new(deep_messages::INVALID_ARGUMENTS_FOR_WORKFLOW_ACTION, &[("uses", &(uses).to_string()), ("err", &(err).to_string())])))
}
