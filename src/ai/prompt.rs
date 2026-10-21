// AI 提示词纯函数；任务启动时冻结两种语言，后续分段不得重读偏好。
use crate::language::{Language, TaskLanguages};

pub(crate) const COMMIT_DIFF_TEXT_LIMIT: usize = 6000;
pub(crate) const WORKFLOW_REQUEST_TEXT_LIMIT: usize = 4000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatRole { System, User, Assistant }

impl ChatRole {
    pub fn as_str(self) -> &'static str {
        match self { Self::System => "system", Self::User => "user", Self::Assistant => "assistant" }
    }
}

/// 只选择宿主文案，不能用于翻译用户输入、源码或工具结果。
pub fn text(language: Language, zh: &'static str, en: &'static str) -> &'static str {
    match language { Language::ZhCn => zh, Language::En => en }
}

fn pair(system: String, user: String, languages: TaskLanguages) -> (ChatMessage, ChatMessage) {
    (ChatMessage { role: ChatRole::System, content: format!("{system}\n\n{}", languages.output_instruction()) },
     ChatMessage { role: ChatRole::User, content: user })
}

pub fn commit_message_prompts(diff_text: &str, hint: Option<&str>) -> (ChatMessage, ChatMessage) {
    commit_message_prompts_with_languages(diff_text, hint, TaskLanguages::default())
}

pub fn commit_message_prompts_with_languages(diff_text: &str, hint: Option<&str>, languages: TaskLanguages) -> (ChatMessage, ChatMessage) {
    let language = languages.system;
    let system = text(language,
        "你是一个资深的 Git 提交信息助手。根据暂存区差异生成简洁、清晰的提交信息。\n要求：\n1. 只输出提交信息正文，不要解释、代码块标记或多余空行。\n2. 第一行是简短摘要（不超过 50 字符）；必要时空一行再写正文要点。\n3. 可使用 Conventional Commits（如 feat/fix/refactor），但不强求。\n4. 关注为什么改，而非改了什么文件。",
        "You are an experienced Git commit message assistant. Generate a concise, clear commit message from the staged diff.\nRequirements:\n1. Output only the commit message, without explanations, code fences or extra blank lines.\n2. Start with a short summary (at most 50 characters); optionally follow a blank line with body bullets.\n3. Conventional Commits (feat/fix/refactor) are optional.\n4. Focus on why the change was made, not a list of changed files.").to_owned();
    let mut user = format!("{}\n\n```diff\n{}\n```", text(language,
        "以下是本次提交的暂存区差异，请生成提交信息：", "Generate a commit message for this staged diff:"),
        truncate_text_in(diff_text, COMMIT_DIFF_TEXT_LIMIT, language));
    if let Some(hint) = hint.filter(|hint| !hint.trim().is_empty()) {
        user.push_str(text(language, "\n\n补充要求：", "\n\nAdditional requirements: "));
        user.push_str(hint);
    }
    pair(system, user, languages)
}

pub fn conflict_merge_prompts(path: &str, diff3_text: &str, segment: Option<(usize, usize)>) -> (ChatMessage, ChatMessage) {
    conflict_merge_prompts_with_languages(path, diff3_text, segment, TaskLanguages::default())
}

pub fn conflict_merge_prompts_with_languages(path: &str, diff3_text: &str, segment: Option<(usize, usize)>, languages: TaskLanguages) -> (ChatMessage, ChatMessage) {
    let language = languages.system;
    let system = text(language,
        "你是一名资深 Git 合并专家，解决 diff3 合并冲突。标记格式：\n<<<<<<< OURS（当前分支）\n||||||| BASE（共同祖先，仅供理解意图）\n=======\n>>>>>>> THEIRS（传入分支）\n要求：\n1. 只输出解决冲突后的完整文本，不要解释、代码块围栏或前后缀。\n2. 综合 OURS 与 THEIRS 的意图，保留两侧语义；BASE 不一定需要保留。\n3. 非冲突上下文必须逐字保留，不增删空行、不改格式、不顺手优化。\n4. 输出不得残留冲突标记（<<<<<<<、|||||||、=======、>>>>>>>）。\n5. 无法确定时优先同时保留两侧内容（OURS 在前），保证语法正确。\n6. 输出语言只约束新生成的自然语言，不翻译文件已有内容、代码或注释。",
        "You are an experienced Git merge expert resolving diff3 conflicts. Marker format:\n<<<<<<< OURS (current branch)\n||||||| BASE (common ancestor, for intent only)\n=======\n>>>>>>> THEIRS (incoming branch)\nRequirements:\n1. Output only the complete resolved text, without explanations, code fences or prefixes/suffixes.\n2. Combine the intent of OURS and THEIRS and preserve both sides' semantics; BASE need not be retained.\n3. Preserve non-conflicting context verbatim: no blank-line changes, reformatting or unrelated cleanup.\n4. Leave no conflict markers (<<<<<<<, |||||||, =======, >>>>>>>).\n5. If uncertain, prefer retaining both sides (OURS first), with valid syntax.\n6. The output language applies only to newly generated natural language; never translate existing file contents, code or comments.").to_owned();
    let heading = match (language, segment) {
        (Language::ZhCn, Some((index, total))) => format!("以下是文件 {path} 的第 {index}/{total} 段（各段首尾相接）。请只输出这一段解决冲突后的完整内容："),
        (Language::En, Some((index, total))) => format!("This is segment {index}/{total} of {path} (segments are concatenated). Output only this segment's complete resolved contents:"),
        (Language::ZhCn, None) => format!("以下是文件 {path} 的完整内容，请输出解决全部冲突后的完整文件："),
        (Language::En, None) => format!("This is the complete contents of {path}. Output the complete file with all conflicts resolved:"),
    };
    pair(system, format!("{heading}\n\n```diff3\n{diff3_text}\n```"), languages)
}

const WORKFLOW_DOC_TEXT: &str = include_str!("../../docs/workflows.md");
const WORKFLOW_DOC_EN: &str = include_str!("../../docs/workflows.en.md");

/// 版本是结构化输入，不依赖对已翻译句子的字符串替换。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkflowPromptVersion { V1, V2 }

pub fn workflow_template_prompts(request: &str, current_definition_json5: Option<&str>) -> (ChatMessage, ChatMessage) {
    workflow_template_prompts_with_languages(request, current_definition_json5, WorkflowPromptVersion::V1, TaskLanguages::default())
}

pub fn workflow_template_prompts_with_languages(request: &str, current_definition_json5: Option<&str>, version: WorkflowPromptVersion, languages: TaskLanguages) -> (ChatMessage, ChatMessage) {
    let language = languages.system;
    let mut system = text(language,
        "你是 Khaslana Git 客户端的工作流模板生成助手。按用户需求生成可直接保存的 JSON5 工作流。严格遵循以下完整文档的格式、步骤、变量与限制：\n\n",
        "You generate workflow templates for the Khaslana Git client. Produce a ready-to-save JSON5 workflow meeting the user's requirements. Follow this reference for formats, steps, variables and constraints:\n\n").to_owned();
    system.push_str(match language { Language::ZhCn => WORKFLOW_DOC_TEXT, Language::En => WORKFLOW_DOC_EN });
    system.push_str(text(language,
        "\n\n输出要求：\n1. 只输出一个完整 JSON5 对象，不要解释、前后缀或 markdown 围栏。\n",
        "\n\nOutput requirements:\n1. Output exactly one complete JSON5 object, without explanations, prefixes/suffixes or markdown fences.\n"));
    system.push_str(match (language, version) {
        (Language::ZhCn, WorkflowPromptVersion::V1) => "2. version 字段恒为 1。\n3. steps 至少一个，op 只能使用文档「支持的步骤」章节列出的 Git 类型，不使用 invoke。\n",
        (Language::En, WorkflowPromptVersion::V1) => "2. version must be 1.\n3. Include at least one step. op must be a Git type from Supported steps; do not use invoke.\n",
        (Language::ZhCn, WorkflowPromptVersion::V2) => "2. version 字段恒为 2。\n3. steps 至少一个；可使用文档支持的 Git 步骤或 invoke。invoke 必须有唯一 id、uses 和对象 with，可选 saveAs；uses 可为 git.*、js.run、skill.run。MCP 是 Skill 或 JS 调用的工具，不新增独立 mcp.call 步骤，仅保留当前模板已有的 mcp.call 兼容步骤。纯 JS 无需 MCP，只有任务或脚本确实需要时才声明 tools，不自动添加工具。\n",
        (Language::En, WorkflowPromptVersion::V2) => "2. version must be 2.\n3. Include at least one supported Git step or invoke. Each invoke needs a unique id, uses and an object with; saveAs is optional. uses may be git.*, js.run or skill.run. MCP tools belong inside Skill or JS: do not add standalone mcp.call steps; preserve existing legacy mcp.call steps only. Pure JS needs no MCP; declare tools only when actually needed by the task or script, never automatically.\n",
    });
    system.push_str(text(language,
        "4. 变量统一使用 ${...}；inputs 的键不得以 git. / run. / date: 开头。\n5. label、说明和新生成的任务描述使用输出语言，而非系统提示语言；现有用户内容除需求涉及部分外原样保留。\n6. inputs.type 只用 text、textarea、select、date；select 必须有非空 options，每项 label 与唯一非空 value；日期为 YYYY-MM-DD。要求用户明确选择时不设默认值。",
        "4. Use ${...} for variables; inputs keys must not start with git. / run. / date:.\n5. Use the output language, not the system instruction language, for labels, descriptions and newly generated tasks. Preserve existing user content unless the request requires a change.\n6. inputs.type is text, textarea, select or date. select needs nonempty options with labels and unique nonempty values; dates use YYYY-MM-DD. Omit defaults when an explicit choice is required."));
    let mut user = match current_definition_json5 {
        Some(current) => format!("{}\n\n{}\n```json5\n{current}\n```\n\n", text(language,
            "请按新需求修改现有模板，未涉及部分保持不变。", "Modify this template for the new requirements; keep unrelated content unchanged."),
            text(language, "【现有模板】", "[Current template]")),
        None => text(language, "请从零生成一个新的工作流模板。\n\n", "Generate a new workflow template from scratch.\n\n").to_owned(),
    };
    user.push_str(text(language, "【功能需求】\n", "[Requirements]\n"));
    user.push_str(&truncate_text_in(request.trim(), WORKFLOW_REQUEST_TEXT_LIMIT, language));
    pair(system, user, languages)
}

#[cfg(test)]
pub(crate) fn truncate_text(text: &str, limit: usize) -> String {
    truncate_text_in(text, limit, Language::ZhCn)
}

pub(crate) fn truncate_text_in(value: &str, limit: usize, language: Language) -> String {
    if value.chars().count() <= limit { return value.to_owned(); }
    let truncated: String = value.chars().take(limit).collect();
    format!("{truncated}\n\n{}", text(language, "…（差异过长，已截断）", "… (text exceeded the limit and was truncated)"))
}

#[cfg(test)]
#[path = "../tests/ai/prompt.rs"]
mod tests;
