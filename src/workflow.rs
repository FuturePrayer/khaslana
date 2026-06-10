use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::{DateTime, Local};
use git2::Repository;
use serde::Deserialize;

use crate::{
    BranchName, GitError, GitService, RemoteName, RepositorySnapshot, Result, WorktreeChange,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinition {
    pub version: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub defaults: WorkflowDefaults,
    #[serde(default)]
    pub inputs: BTreeMap<String, WorkflowInputDefinition>,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
    pub steps: Vec<WorkflowStep>,
}

impl WorkflowDefinition {
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "未命名工作流".to_string())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowInputDefinition {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default = "default_workflow_input_required")]
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefaults {
    #[serde(default = "default_require_clean_worktree")]
    pub require_clean_worktree: bool,
}

impl Default for WorkflowDefaults {
    fn default() -> Self {
        Self {
            require_clean_worktree: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum WorkflowStep {
    Checkout {
        branch: String,
    },
    Fetch {
        #[serde(default)]
        remote: Option<String>,
    },
    Pull {
        #[serde(default)]
        remote: Option<String>,
    },
    CreateBranch {
        name: String,
        #[serde(default)]
        from: Option<String>,
        #[serde(default = "default_create_branch_checkout")]
        checkout: bool,
    },
    Merge {
        branch: String,
    },
    Push {
        #[serde(default)]
        remote: Option<String>,
        #[serde(default)]
        branch: Option<String>,
        #[serde(default = "default_set_upstream")]
        set_upstream: bool,
    },
    EnsureClean,
    AssertBranch {
        branch: String,
    },
}

#[derive(Clone, Debug)]
pub struct WorkflowRunOptions {
    pub default_remote: String,
    pub input_vars: BTreeMap<String, String>,
}

impl Default for WorkflowRunOptions {
    fn default() -> Self {
        Self {
            default_remote: "origin".to_string(),
            input_vars: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowRunResult {
    pub name: String,
    pub steps_run: usize,
    pub snapshot: RepositorySnapshot,
}

#[derive(Clone, Debug)]
pub struct WorkflowPreview {
    pub name: String,
    pub steps: Vec<WorkflowPreviewStep>,
}

#[derive(Clone, Debug)]
pub struct WorkflowPreviewStep {
    pub index: usize,
    pub op: &'static str,
    pub summary: String,
}

#[derive(Clone, Debug)]
pub enum WorkflowProgressEvent {
    Started {
        name: String,
        total: usize,
    },
    StepStarted {
        index: usize,
        total: usize,
        label: String,
    },
    StepFinished {
        index: usize,
        total: usize,
        label: String,
    },
    Finished {
        name: String,
        total: usize,
    },
}

pub struct WorkflowExecutor<'a> {
    service: &'a GitService,
}

impl<'a> WorkflowExecutor<'a> {
    pub fn new(service: &'a GitService) -> Self {
        Self { service }
    }

    pub fn preview(
        &self,
        repo: &Repository,
        definition: &WorkflowDefinition,
        options: &WorkflowRunOptions,
    ) -> Result<WorkflowPreview> {
        validate_definition(definition)?;
        validate_input_values(definition, options)?;
        let context = WorkflowEvalContext::new(self.service, repo);
        let mut resolver = WorkflowResolver::new(self.service, repo, definition, options, &context);
        let steps = definition
            .steps
            .iter()
            .enumerate()
            .map(|(index, step)| {
                Ok(WorkflowPreviewStep {
                    index,
                    op: step.op_name(),
                    summary: step.resolved_summary(&mut resolver)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(WorkflowPreview {
            name: definition.display_name(),
            steps,
        })
    }

    pub fn resolve_template(
        &self,
        repo: &Repository,
        definition: &WorkflowDefinition,
        options: &WorkflowRunOptions,
        template: &str,
    ) -> Result<String> {
        validate_definition(definition)?;
        let context = WorkflowEvalContext::new(self.service, repo);
        let mut resolver = WorkflowResolver::new(self.service, repo, definition, options, &context);
        resolver.interpolate(template)
    }

    pub fn run<F>(
        &self,
        repo: &mut Repository,
        definition: &WorkflowDefinition,
        options: WorkflowRunOptions,
        mut progress: F,
    ) -> Result<WorkflowRunResult>
    where
        F: FnMut(WorkflowProgressEvent),
    {
        validate_definition(definition)?;
        validate_input_values(definition, &options)?;
        if definition.defaults.require_clean_worktree {
            ensure_clean_worktree(self.service, repo)?;
        }

        let name = definition.display_name();
        let total = definition.steps.len();
        progress(WorkflowProgressEvent::Started {
            name: name.clone(),
            total,
        });

        let context = WorkflowEvalContext::new(self.service, repo);
        let mut steps_run = 0;
        let mut last_snapshot = None;

        for (index, step) in definition.steps.iter().enumerate() {
            let resolved_step = {
                let mut resolver =
                    WorkflowResolver::new(self.service, repo, definition, &options, &context);
                step.resolve(&mut resolver)?
            };
            let label = resolved_step.summary();
            progress(WorkflowProgressEvent::StepStarted {
                index,
                total,
                label: label.clone(),
            });

            last_snapshot = Some(resolved_step.execute(self.service, repo)?);
            steps_run += 1;

            progress(WorkflowProgressEvent::StepFinished {
                index,
                total,
                label,
            });
        }

        let snapshot = match last_snapshot {
            Some(snapshot) => snapshot,
            None => self.service.snapshot_after_operation(repo)?,
        };
        progress(WorkflowProgressEvent::Finished {
            name: name.clone(),
            total,
        });
        Ok(WorkflowRunResult {
            name,
            steps_run,
            snapshot,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedWorkflowStep {
    Checkout {
        branch: String,
    },
    Fetch {
        remote: String,
    },
    Pull {
        remote: String,
    },
    CreateBranch {
        name: String,
        from: Option<String>,
        checkout: bool,
    },
    Merge {
        branch: String,
    },
    Push {
        remote: String,
        branch: String,
        set_upstream: bool,
    },
    EnsureClean,
    AssertBranch {
        branch: String,
    },
}

impl WorkflowStep {
    pub fn op_name(&self) -> &'static str {
        match self {
            WorkflowStep::Checkout { .. } => "checkout",
            WorkflowStep::Fetch { .. } => "fetch",
            WorkflowStep::Pull { .. } => "pull",
            WorkflowStep::CreateBranch { .. } => "createBranch",
            WorkflowStep::Merge { .. } => "merge",
            WorkflowStep::Push { .. } => "push",
            WorkflowStep::EnsureClean => "ensureClean",
            WorkflowStep::AssertBranch { .. } => "assertBranch",
        }
    }

    fn resolved_summary(&self, resolver: &mut WorkflowResolver<'_, '_>) -> Result<String> {
        Ok(self.resolve(resolver)?.summary())
    }

    fn resolve(&self, resolver: &mut WorkflowResolver<'_, '_>) -> Result<ResolvedWorkflowStep> {
        match self {
            WorkflowStep::Checkout { branch } => Ok(ResolvedWorkflowStep::Checkout {
                branch: resolver.interpolate(branch)?,
            }),
            WorkflowStep::Fetch { remote } => Ok(ResolvedWorkflowStep::Fetch {
                remote: resolver.remote_name(remote)?,
            }),
            WorkflowStep::Pull { remote } => Ok(ResolvedWorkflowStep::Pull {
                remote: resolver.remote_name(remote)?,
            }),
            WorkflowStep::CreateBranch {
                name,
                from,
                checkout,
            } => {
                let from = from
                    .as_ref()
                    .map(|from| resolver.interpolate(from))
                    .transpose()?;
                Ok(ResolvedWorkflowStep::CreateBranch {
                    name: resolver.interpolate(name)?,
                    from,
                    checkout: *checkout,
                })
            }
            WorkflowStep::Merge { branch } => Ok(ResolvedWorkflowStep::Merge {
                branch: resolver.interpolate(branch)?,
            }),
            WorkflowStep::Push {
                remote,
                branch,
                set_upstream,
            } => Ok(ResolvedWorkflowStep::Push {
                remote: resolver.remote_name(remote)?,
                branch: resolver.branch_or_current(branch)?,
                set_upstream: *set_upstream,
            }),
            WorkflowStep::EnsureClean => Ok(ResolvedWorkflowStep::EnsureClean),
            WorkflowStep::AssertBranch { branch } => Ok(ResolvedWorkflowStep::AssertBranch {
                branch: resolver.interpolate(branch)?,
            }),
        }
    }
}

impl ResolvedWorkflowStep {
    fn summary(&self) -> String {
        match self {
            ResolvedWorkflowStep::Checkout { branch } => format!("切换到分支 {branch}"),
            ResolvedWorkflowStep::Fetch { remote } => format!("获取远端 {remote}"),
            ResolvedWorkflowStep::Pull { remote } => format!("拉取远端 {remote}"),
            ResolvedWorkflowStep::CreateBranch {
                name,
                from,
                checkout,
            } => {
                let from = from.clone().unwrap_or_else(|| "当前 HEAD".to_string());
                let suffix = if *checkout { "并切换" } else { "" };
                format!("基于 {from} 创建分支 {name}{suffix}")
            }
            ResolvedWorkflowStep::Merge { branch } => format!("合并分支 {branch}"),
            ResolvedWorkflowStep::Push { remote, branch, .. } => {
                format!("推送分支 {branch} 到 {remote}")
            }
            ResolvedWorkflowStep::EnsureClean => "检查工作区干净".to_string(),
            ResolvedWorkflowStep::AssertBranch { branch } => {
                format!("确认当前分支是 {branch}")
            }
        }
    }

    fn execute(&self, service: &GitService, repo: &mut Repository) -> Result<RepositorySnapshot> {
        match self {
            ResolvedWorkflowStep::Checkout { branch } => {
                service.checkout_branch(repo, &BranchName::new(branch.clone()))
            }
            ResolvedWorkflowStep::Fetch { remote } => {
                service.fetch(repo, &RemoteName::new(remote.clone()))
            }
            ResolvedWorkflowStep::Pull { remote } => {
                service.pull(repo, &RemoteName::new(remote.clone()))
            }
            ResolvedWorkflowStep::CreateBranch {
                name,
                from,
                checkout,
            } => service.create_branch_from(
                repo,
                &BranchName::new(name.clone()),
                from.as_ref()
                    .map(|from| BranchName::new(from.clone()))
                    .as_ref(),
                *checkout,
            ),
            ResolvedWorkflowStep::Merge { branch } => {
                service.merge_branch(repo, &BranchName::new(branch.clone()))
            }
            ResolvedWorkflowStep::Push {
                remote,
                branch,
                set_upstream,
            } => service.push_branch(
                repo,
                &RemoteName::new(remote.clone()),
                &BranchName::new(branch.clone()),
                *set_upstream,
            ),
            ResolvedWorkflowStep::EnsureClean => {
                ensure_clean_worktree(service, repo)?;
                service.snapshot_after_operation(repo)
            }
            ResolvedWorkflowStep::AssertBranch { branch } => {
                let actual = service.current_branch(repo).ok_or_else(|| {
                    GitError::Message("当前 HEAD 未指向本地分支，无法确认分支".into())
                })?;
                if actual != *branch {
                    return Err(GitError::Message(format!(
                        "当前分支是 {actual}，不是工作流要求的 {branch}"
                    )));
                }
                service.snapshot_after_operation(repo)
            }
        }
    }
}

pub fn parse_workflow_json5(content: &str) -> Result<WorkflowDefinition> {
    let definition = json5::from_str::<WorkflowDefinition>(content)
        .map_err(|err| GitError::Message(format!("工作流 JSON5 解析失败：{err}")))?;
    validate_definition(&definition)?;
    Ok(definition)
}

fn validate_definition(definition: &WorkflowDefinition) -> Result<()> {
    if definition.version != 1 {
        return Err(GitError::Message(format!(
            "不支持的工作流版本：{}",
            definition.version
        )));
    }
    if definition.steps.is_empty() {
        return Err(GitError::Message("工作流至少需要一个步骤".into()));
    }
    for key in definition.inputs.keys() {
        validate_input_name(key)?;
    }
    Ok(())
}

fn validate_input_name(name: &str) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err(GitError::Message("工作流输入变量名不能为空".into()));
    }
    if name == "run.id"
        || name == "git.initialBranch"
        || name == "git.currentBranch"
        || name == "git.head"
        || name == "git.repoName"
        || name.starts_with("date:")
        || name.starts_with("run.startedAt:")
        || name.starts_with("git.")
        || name.starts_with("run.")
    {
        return Err(GitError::Message(format!(
            "工作流输入变量不能使用内置变量名：{name}"
        )));
    }
    Ok(())
}

fn validate_input_values(
    definition: &WorkflowDefinition,
    options: &WorkflowRunOptions,
) -> Result<()> {
    for (name, input) in &definition.inputs {
        if input.required
            && options
                .input_vars
                .get(name)
                .is_none_or(|value| value.trim().is_empty())
        {
            let label = input
                .label
                .as_deref()
                .filter(|label| !label.trim().is_empty())
                .unwrap_or(name);
            return Err(GitError::Message(format!("请填写工作流变量：{label}")));
        }
    }
    Ok(())
}

fn ensure_clean_worktree(service: &GitService, repo: &Repository) -> Result<()> {
    let changes = service.status_full(repo)?;
    if changes.is_empty() {
        return Ok(());
    }
    Err(GitError::Message(format!(
        "工作区存在未提交更改，不能运行该工作流：{}",
        changes_preview(&changes)
    )))
}

fn changes_preview(changes: &[WorktreeChange]) -> String {
    let mut preview = changes
        .iter()
        .take(5)
        .map(|change| change.path.clone())
        .collect::<Vec<_>>()
        .join(", ");
    if changes.len() > 5 {
        preview.push_str(&format!(" 等 {} 个文件", changes.len()));
    }
    preview
}

struct WorkflowEvalContext {
    started_at: DateTime<Local>,
    run_id: String,
    initial_branch: Option<String>,
    repo_name: String,
}

impl WorkflowEvalContext {
    fn new(service: &GitService, repo: &Repository) -> Self {
        let started_at = Local::now();
        Self {
            run_id: format!("{}", started_at.timestamp_millis()),
            initial_branch: service.current_branch(repo),
            repo_name: repo_display_name(repo),
            started_at,
        }
    }
}

struct WorkflowResolver<'a, 'repo> {
    service: &'a GitService,
    repo: &'repo Repository,
    definition: &'a WorkflowDefinition,
    options: &'a WorkflowRunOptions,
    context: &'a WorkflowEvalContext,
}

impl<'a, 'repo> WorkflowResolver<'a, 'repo> {
    fn new(
        service: &'a GitService,
        repo: &'repo Repository,
        definition: &'a WorkflowDefinition,
        options: &'a WorkflowRunOptions,
        context: &'a WorkflowEvalContext,
    ) -> Self {
        Self {
            service,
            repo,
            definition,
            options,
            context,
        }
    }

    fn current_branch(&self) -> Result<String> {
        self.service.current_branch(self.repo).ok_or_else(|| {
            GitError::Message("当前 HEAD 未指向本地分支，无法解析 git.currentBranch".into())
        })
    }

    fn head_oid(&self) -> Result<String> {
        let head = self.repo.head()?;
        let target = head
            .target()
            .ok_or_else(|| GitError::Message("当前 HEAD 没有目标提交".into()))?;
        Ok(target.to_string())
    }

    fn remote_name(&mut self, remote: &Option<String>) -> Result<String> {
        match remote {
            Some(remote) => self.interpolate(remote),
            None => Ok(self.options.default_remote.clone()),
        }
    }

    fn branch_or_current(&mut self, branch: &Option<String>) -> Result<String> {
        match branch {
            Some(branch) => self.interpolate(branch),
            None => self.current_branch(),
        }
    }

    fn interpolate(&mut self, template: &str) -> Result<String> {
        self.interpolate_with_stack(template, &mut BTreeSet::new())
    }

    fn interpolate_with_stack(
        &mut self,
        template: &str,
        stack: &mut BTreeSet<String>,
    ) -> Result<String> {
        let mut output = String::new();
        let mut rest = template;

        while let Some(start) = rest.find("${") {
            output.push_str(&rest[..start]);
            let expression_start = start + 2;
            let Some(end) = rest[expression_start..].find('}') else {
                return Err(GitError::Message(format!(
                    "变量表达式缺少结束符：{template}"
                )));
            };
            let expression_end = expression_start + end;
            let expression = rest[expression_start..expression_end].trim();
            output.push_str(&self.resolve_expression(expression, stack)?);
            rest = &rest[expression_end + 1..];
        }

        output.push_str(rest);
        Ok(output)
    }

    fn resolve_expression(
        &mut self,
        expression: &str,
        stack: &mut BTreeSet<String>,
    ) -> Result<String> {
        if let Some(format) = expression.strip_prefix("date:") {
            return Ok(self.context.started_at.format(format).to_string());
        }
        if let Some(format) = expression.strip_prefix("run.startedAt:") {
            return Ok(self.context.started_at.format(format).to_string());
        }

        match expression {
            "run.id" => return Ok(self.context.run_id.clone()),
            "git.initialBranch" => {
                return self.context.initial_branch.clone().ok_or_else(|| {
                    GitError::Message("当前 HEAD 未指向本地分支，无法解析 git.initialBranch".into())
                });
            }
            "git.currentBranch" => return self.current_branch(),
            "git.head" => return self.head_oid(),
            "git.repoName" => return Ok(self.context.repo_name.clone()),
            _ => {}
        }

        if let Some(value) = self.definition.vars.get(expression) {
            if !stack.insert(expression.to_string()) {
                return Err(GitError::Message(format!(
                    "工作流变量存在循环引用：{expression}"
                )));
            }
            if let Some(input) = self.options.input_vars.get(expression) {
                stack.remove(expression);
                return Ok(input.clone());
            }
            let resolved = self.interpolate_with_stack(value, stack);
            stack.remove(expression);
            return resolved;
        }
        if let Some(value) = self.options.input_vars.get(expression) {
            return Ok(value.clone());
        }

        Err(GitError::Message(format!("未知工作流变量：{expression}")))
    }
}

fn repo_display_name(repo: &Repository) -> String {
    repo.workdir()
        .or_else(|| repo.path().parent())
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "repository".to_string())
}

fn default_require_clean_worktree() -> bool {
    true
}

fn default_workflow_input_required() -> bool {
    true
}

fn default_create_branch_checkout() -> bool {
    true
}

fn default_set_upstream() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;

    use git2::{IndexAddOption, Oid, RepositoryInitOptions, Signature};
    use tempfile::TempDir;

    use super::*;
    use crate::NoopProgress;
    use crate::credentials::PromptCredentialProvider;

    fn service() -> GitService {
        GitService::new(
            Arc::new(PromptCredentialProvider::memory_only(|_| Ok(None))),
            Arc::new(NoopProgress),
        )
    }

    fn init_repo() -> (TempDir, Repository, GitService) {
        let dir = TempDir::new().unwrap();
        let mut options = RepositoryInitOptions::new();
        options.initial_head("main");
        let repo = Repository::init_opts(dir.path(), &options).unwrap();
        configure_user(&repo);
        (dir, repo, service())
    }

    fn configure_user(repo: &Repository) {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config
            .set_str("user.email", "test@example.invalid")
            .unwrap();
    }

    fn write_file(root: &Path, path: &str, body: &str) {
        fs::write(root.join(path), body).unwrap();
    }

    fn signature() -> Signature<'static> {
        Signature::now("Test User", "test@example.invalid").unwrap()
    }

    fn commit_all(repo: &Repository, message: &str) -> Oid {
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let signature = signature();
        let parents = repo
            .head()
            .ok()
            .and_then(|head| head.peel_to_commit().ok())
            .into_iter()
            .collect::<Vec<_>>();
        let parent_refs = parents.iter().collect::<Vec<_>>();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )
        .unwrap()
    }

    #[test]
    fn parses_json5_with_comments_and_variables() {
        let definition = parse_workflow_json5(
            r#"
            {
              version: 1,
              name: "demo",
              vars: {
                target: "release/${date:%Y%m%d}",
              },
              // comment
              steps: [
                { op: "checkout", branch: "main" },
                { op: "createBranch", name: "${target}", from: "main", checkout: true },
              ],
            }
            "#,
        )
        .unwrap();

        assert_eq!(definition.display_name(), "demo");
        assert_eq!(definition.steps.len(), 2);
    }

    #[test]
    fn parses_workflow_inputs() {
        let definition = parse_workflow_json5(
            r#"
            {
              version: 1,
              inputs: {
                target: {
                  label: "目标分支",
                  description: "运行前填写",
                  default: "feature/${date:%Y%m%d}",
                },
                optionalName: { required: false },
              },
              steps: [{ op: "createBranch", name: "${target}" }],
            }
            "#,
        )
        .unwrap();

        let target = definition.inputs.get("target").unwrap();
        assert_eq!(target.label.as_deref(), Some("目标分支"));
        assert_eq!(target.description.as_deref(), Some("运行前填写"));
        assert!(target.required);
        assert!(!definition.inputs.get("optionalName").unwrap().required);
    }

    #[test]
    fn rejects_unknown_version() {
        let err =
            parse_workflow_json5("{ version: 99, steps: [{ op: \"ensureClean\" }] }").unwrap_err();
        assert!(err.to_string().contains("不支持的工作流版本"));
    }

    #[test]
    fn rejects_builtin_input_names() {
        let err = parse_workflow_json5(
            r#"
            {
              version: 1,
              inputs: { "git.currentBranch": { default: "main" } },
              steps: [{ op: "ensureClean" }],
            }
            "#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("内置变量名"));
    }

    #[test]
    fn detects_variable_cycles() {
        let (dir, repo, service) = init_repo();
        write_file(dir.path(), "README.md", "hello\n");
        commit_all(&repo, "initial");
        let definition = parse_workflow_json5(
            r#"
            {
              version: 1,
              vars: { a: "${b}", b: "${a}" },
              steps: [{ op: "assertBranch", branch: "${a}" }],
            }
            "#,
        )
        .unwrap();
        let executor = WorkflowExecutor::new(&service);
        let err = executor
            .preview(&repo, &definition, &WorkflowRunOptions::default())
            .unwrap_err();
        assert!(err.to_string().contains("循环引用"));
    }

    #[test]
    fn input_values_override_vars() {
        let (dir, repo, service) = init_repo();
        write_file(dir.path(), "README.md", "hello\n");
        commit_all(&repo, "initial");
        let definition = parse_workflow_json5(
            r#"
            {
              version: 1,
              inputs: { target: { default: "from-input" } },
              vars: { target: "from-vars" },
              steps: [{ op: "createBranch", name: "${target}" }],
            }
            "#,
        )
        .unwrap();
        let options = WorkflowRunOptions {
            input_vars: BTreeMap::from([("target".to_string(), "chosen".to_string())]),
            ..WorkflowRunOptions::default()
        };

        let preview = WorkflowExecutor::new(&service)
            .preview(&repo, &definition, &options)
            .unwrap();

        assert_eq!(
            preview.steps[0].summary,
            "基于 当前 HEAD 创建分支 chosen并切换"
        );
    }

    #[test]
    fn required_input_values_must_not_be_empty() {
        let (dir, repo, service) = init_repo();
        write_file(dir.path(), "README.md", "hello\n");
        commit_all(&repo, "initial");
        let definition = parse_workflow_json5(
            r#"
            {
              version: 1,
              inputs: { target: { label: "目标分支" } },
              steps: [{ op: "createBranch", name: "${target}" }],
            }
            "#,
        )
        .unwrap();

        let err = WorkflowExecutor::new(&service)
            .preview(&repo, &definition, &WorkflowRunOptions::default())
            .unwrap_err();

        assert!(err.to_string().contains("请填写工作流变量：目标分支"));
    }

    #[test]
    fn resolves_input_defaults_with_existing_variables() {
        let (dir, repo, service) = init_repo();
        write_file(dir.path(), "README.md", "hello\n");
        commit_all(&repo, "initial");
        let definition = parse_workflow_json5(
            r#"
            {
              version: 1,
              vars: { prefix: "feature" },
              inputs: { target: { default: "${prefix}/${git.initialBranch}" } },
              steps: [{ op: "createBranch", name: "${target}" }],
            }
            "#,
        )
        .unwrap();
        let default = WorkflowExecutor::new(&service)
            .resolve_template(
                &repo,
                &definition,
                &WorkflowRunOptions::default(),
                definition.inputs["target"].default.as_deref().unwrap(),
            )
            .unwrap();

        assert_eq!(default, "feature/main");
    }

    #[test]
    fn workflow_creates_branch_merges_and_asserts_dynamic_variables() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "README.md", "base\n");
        commit_all(&repo, "initial");

        service
            .create_branch_from(
                &mut repo,
                &BranchName::new("B"),
                Some(&BranchName::new("main")),
                true,
            )
            .unwrap();
        write_file(dir.path(), "feature.txt", "feature\n");
        commit_all(&repo, "feature");
        service
            .checkout_branch(&mut repo, &BranchName::new("main"))
            .unwrap();

        let definition = parse_workflow_json5(
            r#"
            {
              version: 1,
              vars: {
                target: "A-${git.initialBranch}",
              },
              steps: [
                { op: "checkout", branch: "main" },
                { op: "createBranch", name: "${target}", from: "main", checkout: true },
                { op: "merge", branch: "B" },
                { op: "assertBranch", branch: "${target}" },
              ],
            }
            "#,
        )
        .unwrap();

        let executor = WorkflowExecutor::new(&service);
        let mut events = Vec::new();
        let result = executor
            .run(
                &mut repo,
                &definition,
                WorkflowRunOptions::default(),
                |event| events.push(event),
            )
            .unwrap();

        assert_eq!(result.steps_run, 4);
        assert_eq!(service.current_branch(&repo).as_deref(), Some("A-main"));
        assert!(dir.path().join("feature.txt").exists());
        assert!(events.len() >= 6);
    }

    #[test]
    fn default_clean_worktree_check_rejects_dirty_repo() {
        let (dir, mut repo, service) = init_repo();
        write_file(dir.path(), "README.md", "base\n");
        commit_all(&repo, "initial");
        write_file(dir.path(), "README.md", "dirty\n");
        let definition =
            parse_workflow_json5("{ version: 1, steps: [{ op: \"ensureClean\" }] }").unwrap();
        let executor = WorkflowExecutor::new(&service);

        let err = executor
            .run(
                &mut repo,
                &definition,
                WorkflowRunOptions::default(),
                |_| {},
            )
            .unwrap_err();

        assert!(err.to_string().contains("工作区存在未提交更改"));
    }
}
