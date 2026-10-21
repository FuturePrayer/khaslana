//! 内嵌 MCP stdio 服务器（协议行为逐条对照 codebase-memory-mcp 的
//! `src/mcp/mcp.c`：换行分隔 JSON 帧 + Content-Length 兼容、单行紧凑 JSON
//! text 载荷 + structuredContent 双通道、业务错误 isError + 自纠错 hint、
//! 字符串/数字 id 原样回显、单线程串行处理）。
//!
//! 两种启动形态（对齐参考项目「单服务器 + 项目注册表」的多仓库语义）：
//! - `khaslana mcp <仓库路径>`：单仓库模式，启动即校验并后台保障索引，
//!   工具调用固定查该仓库（per-仓库各挂一条的高级用法，repo 参数被忽略）。
//! - `khaslana mcp`：多仓库模式（推荐，MCP 配置与仓库零耦合），启动时
//!   不绑定仓库；工具调用经可选 `repo` 参数解析目标（仓库绝对路径，或
//!   `list_projects` 返回的 repo 键），未传时若仅一个已索引仓库自动选中、
//!   多个则报错列出清单让模型自纠错；首次触达的仓库自动后台建索引。
//!
//! 入口 `run(repo_path)`：进入 stdin 消息循环直到 EOF。stdout 只走协议，
//! 进度/日志一律 stderr。
//!
//! 由 `main()` 顶部的 `khaslana mcp [仓库路径]` 子命令分支调用（先于 GUI
//! 启动），供 Claude Code / Cursor / ZCode 等 AI 工具作为 MCP 服务器挂载，
//! 查询本机已索引仓库的代码知识图谱。

use crate::types::domain_ui_resources;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::queries::{self, DetailOutcome, TraceDirection, TraceOutcome};
use super::store::{IndexStats, read_index_stats};
use super::search::{SearchOptions, search_symbols_with_options};
use crate::types::Result;

/// 服务器支持的 MCP 协议版本（协商：客户端声明版本在列表内则回显，否则回最新）。
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];
const SERVER_NAME: &str = "khaslana-code-index";
/// 单帧字节上限（防恶意/损坏输入撑爆内存；对齐参考项目 10MB）。
const MAX_FRAME_BYTES: usize = 10 * 1024 * 1024;
/// trace_path / 详情查询的默认 BFS 节点上限（对齐参考项目 MCP_BFS_LIMIT）。
const DEFAULT_MAX_NODES: usize = 100;

#[path = "mcp_transport.rs"]
mod transport;

/// 单个仓库的查询上下文：根目录（按 repo 键解析且 meta 缺路径时可能未知）
/// 与索引库路径。
#[derive(Clone)]
struct RepoContext {
    repo_root: Option<PathBuf>,
    db_path: PathBuf,
    repo_key: String,
}

/// 服务器状态。`fixed` 为单仓库模式的固定仓库；多仓库模式为 None，按每次
/// 工具调用的 repo 参数解析。
pub struct McpServer {
    fixed: Option<RepoContext>,
    data_dir: PathBuf,
    coordinator: super::jobs::IndexCoordinator,
    auto_refresh: bool,

}

#[cfg(test)]
#[path = "../tests/code_index_mcp_jobs.rs"]
mod jobs_tests;

#[cfg(test)]
#[path = "../tests/code_index_mcp_protocol.rs"]
mod protocol_tests;

impl Drop for McpServer {
    fn drop(&mut self) { self.coordinator.stop(); }
}

impl McpServer {
    /// 初始化。`Some(repo_path)` 为单仓库模式：校验仓库 → 索引保障（无库
    /// 自动全量建、有库增量检查，后台线程防 initialize 超时）；`None` 为
    /// 多仓库模式：只解析数据目录，仓库在工具调用时按 repo 参数解析。
    /// 失败返回 Err（调用方写 stderr 后退出）。
    pub fn new(repo_path: Option<&Path>) -> Result<Self> {
        let data_dir = crate::storage::active_data_dir()
            .ok_or_else(|| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::THE_DATA_DIRECTORY_IS_UNAVAILABLE_CANNOT_LOCATE_THE_INDEX_DATABASE, &[])))?;
        let mut server = Self {
            fixed: None,
            data_dir,
            coordinator: super::jobs::IndexCoordinator::default(),
            auto_refresh: true,
        };
        if let Some(repo_path) = repo_path {
            let ctx = Self::context_from_root(&server.data_dir, repo_path)?;
            server.ensure_index_background(&ctx);
            server.fixed = Some(ctx);
        }
        Ok(server)
    }

    /// 测试专用：单仓库模式（跳过仓库校验与索引保障）。
    #[cfg(test)]
    pub(crate) fn for_test(repo_root: &Path, db_path: PathBuf) -> Self {
        Self {
            fixed: Some(RepoContext {
                repo_root: Some(repo_root.to_path_buf()),
                db_path,
                repo_key: String::new(),
            }),
            data_dir: std::env::temp_dir(),
            coordinator: super::jobs::IndexCoordinator::default(),
            auto_refresh: false,
        }
    }

    /// 测试专用：多仓库模式（不绑定仓库），data_dir 指向临时数据目录。
    #[cfg(test)]
    pub(crate) fn for_multi_test(data_dir: PathBuf) -> Self {
        Self {
            fixed: None,
            data_dir,
            coordinator: super::jobs::IndexCoordinator::default(),
            auto_refresh: false,
        }
    }

    /// 由仓库根目录构造查询上下文（canonicalize + git2 校验 + repo 键定位库）。
    fn context_from_root(data_dir: &Path, repo_path: &Path) -> Result<RepoContext> {
        let repo_root = repo_path
            .canonicalize()
            .map_err(|e| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::REPOSITORY_PATH_UNAVAILABLE, &[("value0", &format!("{}", repo_path.display())), ("e", &format!("{}", e))])))?;
        let repo = git2::Repository::open(&repo_root)
            .map_err(|e| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::IS_NOT_A_GIT_REPOSITORY, &[("value0", &format!("{}", repo_root.display())), ("e", &format!("{}", e))])))?;
        let repo_root = repo.workdir().ok_or_else(|| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::A_BARE_REPOSITORY_HAS_NO_WORKING_TREE_CANNOT_BUILD_A_WORKING_TREE_CODE_INDEX, &[])))?
            .canonicalize().map_err(|e| crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::REPOSITORY_WORKING_TREE_UNAVAILABLE, &[("e", &format!("{}", e))])))?;
        let repo_key = crate::ai::review_store::repo_key(&repo_root.to_string_lossy());
        let db_path = super::open_index_db_path(data_dir, &repo_key)?;
        Ok(RepoContext {
            repo_root: Some(repo_root),
            db_path,
            repo_key,
        })
    }

    /// 仓库键格式校验（repo_key = FNV-1a 32 位小写十六进制 8 字符）。
    fn is_repo_key(spec: &str) -> bool {
        spec.len() == 8 && spec.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// 解析 repo 参数：优先按仓库路径（存在但非 Git 仓库显式报错，不静默）；
    /// 路径不存在时回退按 8 位 repo 键在数据目录中定位索引库。
    fn context_from_spec(&self, spec: &str) -> std::result::Result<RepoContext, Value> {
        let path = Path::new(spec);
        if path.exists() {
            return Self::context_from_root(&self.data_dir, path)
                .map_err(|e| json!({ "error": e.to_string() }));
        }
        if Self::is_repo_key(spec) {
            let key = spec.to_ascii_lowercase();
            let db_path = self.data_dir.join("code-index").join(&key).join("index.db");
            if db_path.is_file() {
                // 路由只需要仓库路径；不能每次 search/trace 都先 COUNT 全库节点和边。
                use rusqlite::OptionalExtension;
                let store = super::store::open_read_only_if_exists(&db_path)
                    .map_err(|error| json!({ "error": error.to_string() }))?
                    .ok_or_else(|| json!({ "error": crate::ui_i18n::text(domain_ui_resources::THE_INDEX_DATABASE_IS_UNAVAILABLE_OR_ITS_VERSION_IS_INCOMPATIBLE), "repo": key }))?;
                let root: Option<String> = store.conn.query_row("SELECT value FROM meta WHERE key='repo_path'", [], |row| row.get(0))
                    .optional().map_err(|error| json!({ "error": crate::ui_i18n::format(domain_ui_resources::FAILED_TO_READ_THE_REPOSITORY_PATH, &[("error", &format!("{}", error))]) }))?;
                return Ok(RepoContext { repo_root: root.filter(|root| !root.is_empty()).map(PathBuf::from),
                    db_path, repo_key: key });
            }
        }
        Err(json!({
            "error": crate::ui_i18n::format(domain_ui_resources::THE_REPO_ARGUMENT_IS_NEITHER_AN_EXISTING_REPOSITORY_PATH_NOR_A_KNOWN_INDEX_DATABASE, &[("spec", &format!("{}", spec))]),
            "hint": crate::ui_i18n::text(domain_ui_resources::PROVIDE_AN_ABSOLUTE_REPOSITORY_PATH_OR_CALL_LIST_PROJECTS_TO_SEE_AVAILABLE_REPOSITORI),
        }))
    }

    /// 由 repo 键 + 已读统计构造上下文：根目录取 meta repo_path（旧库可能
    /// 为空 → None，需要根目录的工具会给出明确错误，下次索引落盘补写）。
    fn context_from_entry(data_dir: &Path, repo_key: &str, stats: &IndexStats) -> RepoContext {
        RepoContext {
            repo_root: if stats.repo_path.is_empty() {
                None
            } else {
                Some(PathBuf::from(&stats.repo_path))
            },
            db_path: data_dir.join("code-index").join(repo_key).join("index.db"),
            repo_key: repo_key.to_string(),
        }
    }

    /// 枚举数据目录下全部已索引仓库（code-index/<键>/index.db 且统计可读），
    /// 按最近索引时间倒序。
    fn list_project_entries(data_dir: &Path) -> Vec<(String, IndexStats)> {
        let mut entries = Vec::new();
        let Ok(dirs) = std::fs::read_dir(data_dir.join("code-index")) else {
            return entries;
        };
        for dir in dirs.flatten() {
            let db_path = dir.path().join("index.db");
            if !db_path.is_file() {
                continue;
            }
            let Some(stats) = read_index_stats(&db_path).ok().flatten() else {
                continue;
            };
            entries.push((dir.file_name().to_string_lossy().to_string(), stats));
        }
        entries.sort_by(|a, b| b.1.indexed_at.cmp(&a.1.indexed_at).then(a.0.cmp(&b.0)));
        entries
    }

    /// 解析工具调用的目标仓库：单仓库模式固定返回（repo 参数被忽略）；
    /// 多仓库模式按 repo 参数解析，未传时仅一个已索引仓库自动选中、多个
    /// 报错列出清单让模型自纠错。
    fn resolve_context(&self, repo_arg: Option<&str>) -> std::result::Result<RepoContext, Value> {
        if let Some(fixed) = &self.fixed {
            return Ok(fixed.clone());
        }
        let Some(spec) = repo_arg else {
            let entries = Self::list_project_entries(&self.data_dir);
            return match entries.len() {
                0 => Err(json!({
                    "error": crate::ui_i18n::text(domain_ui_resources::NO_REPOSITORIES_HAVE_BEEN_INDEXED_YET),
                    "hint": crate::ui_i18n::text(domain_ui_resources::CALL_REFRESH_INDEX_WITH_REPO_SET_TO_AN_ABSOLUTE_REPOSITORY_PATH_TO_BUILD_THE_INDEX),
                })),
                1 => Ok(Self::context_from_entry(
                    &self.data_dir,
                    &entries[0].0,
                    &entries[0].1,
                )),
                _ => {
                    let projects: Vec<Value> = entries
                        .iter()
                        .map(|(key, stats)| project_entry_json(key, stats))
                        .collect();
                    Err(json!({
                        "error": crate::ui_i18n::format(domain_ui_resources::THE_REPO_ARGUMENT_WAS_OMITTED_BUT_INDEXED_REPOSITORIES_EXIST_LOCALLY, &[("value0", &format!("{}", projects.len()))]),
                        "projects": projects,
                        "hint": crate::ui_i18n::text(domain_ui_resources::PROVIDE_REPO_AS_AN_ABSOLUTE_REPOSITORY_PATH_OR_ONE_OF_THE_REPO_KEYS_ABOVE_CALL_LIST_P),
                    }))
                }
            };
        };
        let ctx = self.context_from_spec(spec)?;
        Ok(ctx)
    }

    /// 查询触发后台索引检查：按仓库排队、成功检查节流、失败指数退避。
    /// 无库自动全量建、有库增量检查。大仓库全量
    /// 建索引可达分钟级，走专用任务池防 MCP 客户端 initialize/工具超时；建立
    /// 期间查询得到「索引尚未建立」错误 + hint，属预期行为。
    fn ensure_index_background(&self, ctx: &RepoContext) {
        if !self.auto_refresh { return; }
        let Some(repo_root) = ctx.repo_root.clone() else { return; };
        let db_path = ctx.db_path.clone();
        let repo_key = ctx.repo_key.clone();
        self.coordinator.schedule(repo_key.clone(), false, move |cancel| {
            let result = Self::run_index_inner(&repo_root, &db_path, false, cancel, |message| {
                eprintln!("[khaslana-mcp] {message}");
            });
            if let Err(error) = &result { eprintln!("[khaslana-mcp] 后台索引失败（{repo_key}）：{error}"); }
            result.map_err(|error| error.to_string())
        });
    }

    fn indexing(&self, key: &str) -> bool {
        self.coordinator.status(key)["active"].as_bool().unwrap_or(false)
    }

    fn run_index_inner(
        repo_root: &Path,
        db_path: &Path,
        force_full: bool,
        cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
        progress: fn(String),
    ) -> Result<()> {
        let mut options = super::PipelineOptions::new(
            cancel,
            Box::new(move |p| progress(crate::ui_i18n::resolve(&p.message))),
        );
        match super::run_index(repo_root, db_path, force_full, &mut options)? {
            super::RunOutcome::Completed(stats) => progress(crate::ui_i18n::format(domain_ui_resources::INDEXING_COMPLETE_FILES_SYMBOLS_EDGES, &[("value0", &format!("{}", stats.files)), ("value1", &format!("{}", stats.symbols)), ("value2", &format!("{}", stats.edges))])),
            super::RunOutcome::Unchanged => progress(crate::ui_i18n::text(domain_ui_resources::INDEX_UNCHANGED).to_string()),
            super::RunOutcome::Cancelled => return Err(crate::types::GitError::Localized(crate::ui_i18n::LocalizedMessage::new(domain_ui_resources::INDEXING_CANCELLED_THE_PREVIOUS_INDEX_WAS_RETAINED, &[]))),
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // 工具分发
    // ------------------------------------------------------------------

    /// tools/call 分发：返回 MCP content 信封（业务错误 isError:true）。
    /// list_projects 不依赖仓库；其余工具先按 repo 参数解析目标仓库。
    fn call_tool(&self, name: &str, arguments: &Value) -> Value {
        if !matches!(name, "list_projects" | "search_symbols" | "search_graph" | "get_symbol_detail" | "get_code_snippet" | "trace_path" | "get_architecture" | "detect_changes" | "index_status" | "check_index_coverage" | "refresh_index") {
            return text_result(&json!({ "error": crate::ui_i18n::format(domain_ui_resources::UNKNOWN_TOOL, &[("name", &format!("{}", name))]), "hint": crate::ui_i18n::text(domain_ui_resources::SEE_TOOLS_LIST_FOR_AVAILABLE_TOOLS) }), true);
        }
        if name == "list_projects" {
            return match self.tool_list_projects() {
                Ok(value) => text_result(&value, false),
                Err(value) => text_result(&value, true),
            };
        }
        let repo_arg = Self::arg_str(arguments, "repo");
        let ctx = match self.resolve_context(repo_arg) {
            Ok(ctx) => ctx,
            Err(value) => return text_result(&value, true),
        };
        if name != "refresh_index" { self.ensure_index_background(&ctx); }
        let guarded = matches!(name, "search_symbols" | "search_graph" | "get_symbol_detail" | "get_code_snippet" | "trace_path" | "get_architecture" | "check_index_coverage" | "detect_changes");
        let generation = if guarded {
            match super::store::index_generation(&ctx.db_path) {
                Ok(generation) => generation,
                Err(error) => return text_result(&json!({ "error": error.to_string() }), true),
            }
        } else { None };
        if guarded && Self::arg_str(arguments, "generation").is_some_and(|expected| generation.as_deref() != Some(expected)) {
            return text_result(&json!({ "error": crate::ui_i18n::text(domain_ui_resources::THE_INDEX_WAS_REFRESHED_THE_PREVIOUS_PAGINATION_GENERATION_IS_NO_LONGER_VALID), "hint": crate::ui_i18n::text(domain_ui_resources::REMOVE_GENERATION_AND_RESTART_THE_QUERY_AT_OFFSET_0) }), true);
        }
        let result = match name {
            "search_symbols" | "search_graph" => self.tool_search_symbols(&ctx, arguments, name == "search_graph"),
            "get_symbol_detail" => self.tool_symbol_detail(&ctx, arguments),
            "get_code_snippet" => self.tool_code_snippet(&ctx, arguments),
            "trace_path" => self.tool_trace_path(&ctx, arguments),
            "get_architecture" => self.tool_architecture(&ctx),
            "detect_changes" => self.tool_detect_changes(&ctx, arguments),
            "index_status" => self.tool_index_status(&ctx),
            "check_index_coverage" => self.tool_coverage(&ctx, arguments),
            "refresh_index" => self.tool_refresh_index(&ctx, arguments),
            other => {
                return text_result(
                    &json!({
                        "error": crate::ui_i18n::format(domain_ui_resources::UNKNOWN_TOOL_2, &[("other", &format!("{}", other))]),
                        "hint": crate::ui_i18n::text(domain_ui_resources::SEE_TOOLS_LIST_FOR_AVAILABLE_TOOLS),
                    }),
                    true,
                );
            }
        };
        match result {
            Ok(mut value) => {
                if guarded {
                    let current = match super::store::index_generation(&ctx.db_path) {
                        Ok(generation) => generation,
                        Err(error) => return text_result(&json!({ "error": error.to_string() }), true),
                    };
                    if generation != current {
                        return text_result(&json!({ "error": crate::ui_i18n::text(domain_ui_resources::THE_INDEX_CHANGED_DURING_THE_QUERY), "hint": crate::ui_i18n::text(domain_ui_resources::RESTART_THE_QUERY_AT_OFFSET_0) }), true);
                    }
                    value["generation"] = json!(generation);
                }
                text_result(&value, false)
            },
            Err(mut value) => {
                // 后台索引建立期间查询会命中「索引尚未建立」——补自纠错 hint
                // （仅当正在建的就是当前仓库，多仓库模式不误伤其他仓库报错）。
                if self.indexing(&ctx.repo_key) {
                    value["hint"] =
                        json!(crate::ui_i18n::text(domain_ui_resources::THE_INDEX_IS_BEING_BUILT_OR_REFRESHED_IN_THE_BACKGROUND_RETRY_LATER_AND_USE_INDEX_STA));
                }
                text_result(&value, true)
            }
        }
    }

    fn arg_str<'a>(arguments: &'a Value, key: &str) -> Option<&'a str> {
        arguments.get(key).and_then(|v| v.as_str())
    }

    fn arg_usize(arguments: &Value, key: &str) -> Option<usize> {
        arguments
            .get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
    }

    fn missing_arg_error(key: &str) -> Value {
        json!({ "error": crate::ui_i18n::format(domain_ui_resources::MISSING_REQUIRED_ARGUMENT, &[("key", &format!("{}", key))]), "hint": crate::ui_i18n::text(domain_ui_resources::SEE_THIS_TOOL_S_INPUTSCHEMA_IN_TOOLS_LIST) })
    }

    fn tool_search_symbols(
        &self,
        ctx: &RepoContext,
        args: &Value,
        structural: bool,
    ) -> std::result::Result<Value, Value> {
        let query = Self::arg_str(args, "query");
        if query.is_none() && (!structural || ["name_pattern", "qn_pattern", "file_pattern", "label"]
            .iter().all(|key| Self::arg_str(args, key).is_none())) {
            return Err(Self::missing_arg_error(crate::ui_i18n::text(domain_ui_resources::QUERY_OR_NAME_PATTERN_QN_PATTERN_FILE_PATTERN_LABEL)));
        }
        let limit = Self::arg_usize(args, "limit").unwrap_or(10).clamp(1, 200);
        let offset = Self::arg_usize(args, "offset").unwrap_or(0);
        let options = SearchOptions {
            query, label: Self::arg_str(args, "label"),
            file_pattern: Self::arg_str(args, "file_pattern"),
            name_pattern: Self::arg_str(args, "name_pattern"),
            qn_pattern: Self::arg_str(args, "qn_pattern"), limit, offset,
        };
        let (all, total) = search_symbols_with_options(&ctx.db_path, &options)
            .map_err(|e| json!({ "error": e.to_string() }))?;
        let count = all.len();
        let next_offset = offset.saturating_add(count);
        let has_more = next_offset < total;
        let mut result = json!({ "repo": ctx.repo_key, "total": total, "results": all,
            "offset": offset, "returned": count, "has_more": has_more });
        if Self::arg_str(args, "detail") != Some("full") {
            for hit in result["results"].as_array_mut().unwrap() {
                hit.as_object_mut().unwrap().remove("docstring");
            }
        }
        if has_more { result["next_offset"] = json!(next_offset); }
        if count == 0 {
            result["hint"] = if total > 0 {
                json!(crate::ui_i18n::text(domain_ui_resources::OFFSET_EXCEEDS_THE_TOTAL_RESULT_COUNT_RESTART_THE_QUERY_AT_OFFSET_0))
            } else {
                json!(crate::ui_i18n::text(domain_ui_resources::NO_MATCHES_SHORTEN_QUERY_OR_USE_SEARCH_GRAPH_WITH_A_NAME_PATTERN_REGEX_OR_FILE_PATTER))
            };
        }
        Ok(result)
    }

    fn tool_code_snippet(&self, ctx: &RepoContext, args: &Value) -> std::result::Result<Value, Value> {
        let Some(name) = Self::arg_str(args, "qualified_name") else {
            return Err(Self::missing_arg_error("qualified_name"));
        };
        let mut result = self.tool_definition(ctx, &json!({ "name": name }), false)?;
        if let Some(object) = result.as_object_mut() {
            object.remove("callers");
            object.remove("callees");
            if !object.contains_key("source") && !object.contains_key("suggestions") {
                object.insert("source_note".to_string(), json!(crate::ui_i18n::text(domain_ui_resources::CANNOT_READ_THE_SOURCE_FILE_MAKE_SURE_REPO_IS_AN_EXISTING_ABSOLUTE_REPOSITORY_PATH_RE)));
            }
        }
        Ok(result)
    }

    fn tool_symbol_detail(
        &self,
        ctx: &RepoContext,
        args: &Value,
    ) -> std::result::Result<Value, Value> {
        self.tool_definition(ctx, args, true)
    }

    fn tool_definition(&self, ctx: &RepoContext, args: &Value, relations: bool) -> std::result::Result<Value, Value> {
        let Some(name) = Self::arg_str(args, "name") else {
            return Err(Self::missing_arg_error("name"));
        };
        let outcome = super::sql_queries::definition_detail(&ctx.db_path, ctx.repo_root.as_deref(), name, relations)
            .map_err(|e| json!({ "error": e.to_string() }))?;
        match outcome {
            DetailOutcome::Found(detail) => Ok(serde_json::to_value(&*detail)
                .map_err(|e| json!({ "error": crate::ui_i18n::format(domain_ui_resources::FAILED_TO_SERIALIZE_DETAILS, &[("e", &format!("{}", e))]) }))?),
            DetailOutcome::Ambiguous(candidates) => Ok(json!({
                "status": "ambiguous",
                "message": crate::ui_i18n::format(domain_ui_resources::THERE_ARE_DEFINITIONS_WITH_THIS_NAME_QUERY_AN_EXACT_QUALIFIED_NAME, &[("value0", &format!("{}", candidates.len()))]),
                "suggestions": candidates,
            })),
            DetailOutcome::NotFound => Err(json!({
                "error": "symbol not found",
                "name": name,
                "hint": crate::ui_i18n::text(domain_ui_resources::USE_SEARCH_SYMBOLS_FIRST_TO_FIND_THE_CORRECT_NAME),
            })),
        }
    }

    fn tool_trace_path(
        &self,
        ctx: &RepoContext,
        args: &Value,
    ) -> std::result::Result<Value, Value> {
        let Some(function_name) = Self::arg_str(args, "function_name") else {
            return Err(Self::missing_arg_error("function_name"));
        };
        let direction = TraceDirection::parse(Self::arg_str(args, "direction").unwrap_or("both"));
        let depth = Self::arg_usize(args, "depth").unwrap_or(3).clamp(1, 8) as u32;
        let limit = Self::arg_usize(args, "limit").unwrap_or(DEFAULT_MAX_NODES).clamp(1, 200);
        let offset = Self::arg_usize(args, "offset").unwrap_or(0);
        let risk_labels = args
            .get("risk_labels")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let outcome = queries::trace_calls_page(
            &ctx.db_path,
            function_name,
            direction,
            depth,
            limit,
            offset,
        )
        .map_err(|e| json!({ "error": e.to_string() }))?;
        match outcome {
            TraceOutcome::Found(mut result) => {
                if !risk_labels {
                    for hop in result.callers.iter_mut().chain(result.callees.iter_mut()) {
                        hop.risk = "";
                    }
                }
                let mut value = serde_json::to_value(&result)
                    .map_err(|e| json!({ "error": crate::ui_i18n::format(domain_ui_resources::FAILED_TO_SERIALIZE_RESULTS, &[("e", &format!("{}", e))]) }))?;
                if result.has_more { value["next_offset"] = json!(offset.saturating_add(limit)); }
                if Self::arg_str(args, "detail") == Some("full") {
                    value["coverage_note"] = json!(crate::ui_i18n::text(domain_ui_resources::TOTAL_COUNTS_DISTINCT_NODES_WITHIN_THE_REQUESTED_DEPTH_NOT_CALLS_FOR_HOP_1_CALL_SITES));
                } else {
                    for key in ["callers", "callees"] {
                        if let Some(hops) = value.get_mut(key).and_then(Value::as_array_mut) {
                            for hop in hops {
                                // 缺省调用文件为该行的 file_path；跨文件的出向边仍保留真实调用位置。
                                if hop["call_sites"].is_object() && hop["call_sites"]["file_path"] == hop["file_path"] {
                                    hop["call_sites"].as_object_mut().unwrap().remove("file_path");
                                }
                                if hop["risk"] == "" { hop.as_object_mut().unwrap().remove("risk"); }
                            }
                        }
                    }
                }
                Ok(value)
            }
            TraceOutcome::Ambiguous(candidates) => Ok(json!({
                "status": "ambiguous",
                "message": crate::ui_i18n::format(domain_ui_resources::THERE_ARE_CALLABLE_DEFINITIONS_WITH_THIS_NAME_QUERY_AN_EXACT_QUALIFIED_NAME, &[("value0", &format!("{}", candidates.len()))]),
                "suggestions": candidates,
            })),
            TraceOutcome::NotFound => Err(json!({
                "error": "function not found",
                "function_name": function_name,
                "hint": crate::ui_i18n::text(domain_ui_resources::USE_SEARCH_SYMBOLS_FIRST_TO_FIND_THE_CORRECT_NAME),
            })),
        }
    }

    fn tool_architecture(&self, ctx: &RepoContext) -> std::result::Result<Value, Value> {
        let overview =
            queries::index_overview(&ctx.db_path).map_err(|e| json!({ "error": e.to_string() }))?;
        serde_json::to_value(&overview).map_err(|e| json!({ "error": crate::ui_i18n::format(domain_ui_resources::SERIALIZATION_FAILED, &[("e", &format!("{}", e))]) }))
    }

    fn tool_detect_changes(
        &self,
        ctx: &RepoContext,
        args: &Value,
    ) -> std::result::Result<Value, Value> {
        let Some(repo_root) = ctx.repo_root.as_ref() else {
            return Err(json!({
                "error": crate::ui_i18n::text(domain_ui_resources::THIS_INDEX_DATABASE_HAS_NO_REPOSITORY_PATH_METADATA_CANNOT_READ_WORKING_TREE_STATUS),
                "hint": crate::ui_i18n::text(domain_ui_resources::SET_REPO_TO_AN_ABSOLUTE_REPOSITORY_PATH_OR_CALL_REFRESH_INDEX_WITH_THAT_PATH_TO_RESTO),
            }));
        };
        let scope = Self::arg_str(args, "scope").unwrap_or("symbols");
        let base_branch = Self::arg_str(args, "base_branch");
        let changed_files = queries::changed_files_via_git(repo_root, base_branch)
            .map_err(|e| json!({ "error": e.to_string() }))?;
        let mut result = match scope {
            "files" => json!({
                "changed_files": changed_files,
                "changed_count": changed_files.len(),
            }),
            _ => {
                // symbols：文件 + 受影响符号 + 上游调用方展开。
                let report = queries::impacted_symbols_for_files(&ctx.db_path, &changed_files, 1)
                    .map_err(|e| json!({ "error": e.to_string() }))?;
                serde_json::to_value(&report)
                    .map_err(|e| json!({ "error": crate::ui_i18n::format(domain_ui_resources::SERIALIZATION_FAILED, &[("e", &format!("{}", e))]) }))?
            }
        };
        if changed_files.is_empty() {
            result["hint"] = json!(crate::ui_i18n::text(domain_ui_resources::NO_CHANGED_FILES_DETECTED_BETWEEN_THE_WORKING_TREE_AND_THE_BASELINE));
        }
        Ok(result)
    }

    fn tool_index_status(&self, ctx: &RepoContext) -> std::result::Result<Value, Value> {
        let indexing = self.indexing(&ctx.repo_key);
        let stats =
            read_index_stats(&ctx.db_path).map_err(|e| json!({ "error": e.to_string(), "status": "error",
                "repo": ctx.repo_key, "indexing": indexing, "refresh": self.coordinator.status(&ctx.repo_key) }))?;
        match stats {
            Some(stats) => {
                let mut value = stats_to_json(&stats);
                value["repo"] = json!(ctx.repo_key);
                value["indexing"] = json!(indexing);
                value["refresh"] = self.coordinator.status(&ctx.repo_key);
                if indexing { value["status"] = json!("indexing"); }
                if stats.needs_rebuild { value["hint"] = json!(crate::ui_i18n::text(domain_ui_resources::THE_INDEX_NEEDS_REBUILDING_FOR_AN_UPGRADE_WAIT_FOR_THE_BACKGROUND_REFRESH_OR_CALL_REF)); }
                value["coverage_note"] = json!(crate::ui_i18n::text(domain_ui_resources::THE_INDEX_USES_TREE_SITTER_AND_HEURISTIC_CALL_RESOLUTION_NORMAL_STATISTICS_DO_NOT_MEA));
                Ok(value)
            }
            None => Ok(json!({
                "status": if indexing { "indexing" } else { "empty" },
                "indexing": indexing,
                "repo": ctx.repo_key,
                "refresh": self.coordinator.status(&ctx.repo_key),
                "hint": crate::ui_i18n::text(domain_ui_resources::THE_INDEX_IS_EMPTY_CALL_REFRESH_INDEX_TO_BUILD_A_FULL_INDEX_PROVIDE_REPO_IN_MULTI_REP),
            })),
        }
    }

    fn tool_list_projects(&self) -> std::result::Result<Value, Value> {
        let entries = Self::list_project_entries(&self.data_dir);
        let projects: Vec<Value> = entries
            .iter()
            .map(|(key, stats)| project_entry_json(key, stats))
            .collect();
        let count = projects.len();
        let mut result = json!({ "total": count, "projects": projects });
        if count == 0 {
            result["hint"] = json!(
                crate::ui_i18n::text(domain_ui_resources::NO_REPOSITORIES_HAVE_BEEN_INDEXED_YET_CALL_REFRESH_INDEX_WITH_REPO_SET_TO_AN_ABSOLUTE)
            );
        } else {
            result["hint"] = json!(
                crate::ui_i18n::text(domain_ui_resources::FOR_OTHER_TOOLS_SELECT_THE_REPOSITORY_WITH_REPO_AN_ABSOLUTE_PATH_OR_A_REPO_KEY_SHOWN)
            );
        }
        Ok(result)
    }

    fn tool_coverage(&self, ctx: &RepoContext, args: &Value) -> std::result::Result<Value, Value> {
        let strings = |key: &str| -> std::result::Result<Vec<String>, Value> {
            let Some(value) = args.get(key) else { return Ok(Vec::new()); };
            let Some(items) = value.as_array() else { return Err(json!({ "error": crate::ui_i18n::format(domain_ui_resources::MUST_BE_AN_ARRAY_OF_STRINGS, &[("key", &format!("{}", key))]) })); };
            items.iter().map(|value| value.as_str().map(str::to_string)
                .ok_or_else(|| json!({ "error": crate::ui_i18n::format(domain_ui_resources::MUST_BE_AN_ARRAY_OF_STRINGS, &[("key", &format!("{}", key))]) }))).collect()
        };
        super::coverage::check_coverage(&ctx.db_path, ctx.repo_root.as_deref(), &strings("paths")?, &strings("scopes")?,
            Self::arg_usize(args, "offset").unwrap_or(0), Self::arg_usize(args, "limit").unwrap_or(50).clamp(1, 200))
            .map_err(|error| json!({ "error": error.to_string() }))
    }

    fn tool_refresh_index(
        &self,
        ctx: &RepoContext,
        args: &Value,
    ) -> std::result::Result<Value, Value> {
        let mode = Self::arg_str(args, "mode").unwrap_or("incremental");
        let force_full = match mode {
            "incremental" => false,
            "full" => true,
            other => {
                return Err(
                    json!({ "error": crate::ui_i18n::format(domain_ui_resources::UNKNOWN_MODE_EXPECTED_INCREMENTAL_OR_FULL, &[("other", &format!("{}", other))]) }),
                );
            }
        };
        let Some(repo_root) = ctx.repo_root.clone() else {
            return Err(json!({
                "error": crate::ui_i18n::text(domain_ui_resources::THIS_INDEX_DATABASE_HAS_NO_REPOSITORY_PATH_METADATA_CANNOT_LOCATE_THE_REPOSITORY_ROOT),
                "hint": crate::ui_i18n::text(domain_ui_resources::SET_REPO_TO_AN_ABSOLUTE_REPOSITORY_PATH_AND_REFRESH_AGAIN),
            }));
        };
        let db_path = ctx.db_path.clone();
        let scheduled = self.coordinator.schedule(ctx.repo_key.clone(), true, move |cancel| {
            Self::run_index_inner(&repo_root, &db_path, force_full, cancel, |message| {
                eprintln!("[khaslana-mcp] {message}");
            }).map_err(|error| error.to_string())
        });
        if !scheduled && (force_full || !self.indexing(&ctx.repo_key)) {
            return Err(json!({ "error": crate::ui_i18n::text(domain_ui_resources::REFRESH_WAS_NOT_SCHEDULED_A_TASK_FOR_THIS_REPOSITORY_IS_STILL_ACTIVE_OR_THE_SCHEDULER),
                "repo": ctx.repo_key, "scheduled": false, "refresh": self.coordinator.status(&ctx.repo_key),
                "hint": crate::ui_i18n::text(domain_ui_resources::USE_INDEX_STATUS_TO_WAIT_FOR_THE_CURRENT_TASK_TO_FINISH_THEN_CALL_REFRESH_INDEX_AGAIN) }));
        }
        // stdio 串行分发不能等分钟级建库；小任务保留直接返回结果的兼容行为。
        if !self.coordinator.wait_timeout(&ctx.repo_key, std::time::Duration::from_millis(100)) {
            return Ok(json!({ "status": "indexing", "indexing": true, "repo": ctx.repo_key,
                "scheduled": scheduled, "refresh": self.coordinator.status(&ctx.repo_key),
                "hint": crate::ui_i18n::text(domain_ui_resources::INDEXING_IS_STILL_RUNNING_IN_THE_BACKGROUND_USE_INDEX_STATUS_TO_CHECK_COMPLETION_OR_F) }));
        }
        if let Some(error) = self.coordinator.status(&ctx.repo_key)["last_error"].as_str() {
            return Err(json!({ "error": error }));
        }

        let stats = read_index_stats(&ctx.db_path)
            .map_err(|e| json!({ "error": e.to_string() }))?
            .ok_or_else(|| json!({ "error": crate::ui_i18n::text(domain_ui_resources::STATISTICS_ARE_STILL_EMPTY_AFTER_INDEXING_COMPLETED) }))?;
        let mut value = stats_to_json(&stats);
        value["status"] = json!("ready");
        value["repo"] = json!(ctx.repo_key);
        Ok(value)
    }

    // ------------------------------------------------------------------
    // 协议消息处理
    // ------------------------------------------------------------------

    /// 处理一条 JSON-RPC 消息；通知类返回 None（不产生响应）。
    pub fn handle_message(&self, line: &str) -> Option<String> {
        let parsed: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => {
                return Some(jsonrpc_error(&Value::Null, -32700, "Parse error"));
            }
        };
        let valid_id = parsed.get("id").is_none_or(|id| id.is_string() || id.is_i64() || id.is_u64());
        if !parsed.is_object() || parsed["jsonrpc"] != "2.0" || !parsed["method"].is_string() || !valid_id {
            return Some(jsonrpc_error(&Value::Null, -32600, "Invalid Request"));
        }
        // 无 id 的消息按 JSON-RPC 通知处理：不产生任何响应。
        let Some(id) = parsed.get("id").cloned() else {
            return None;
        };
        let Some(method) = parsed.get("method").and_then(|m| m.as_str()) else {
            return Some(jsonrpc_error(
                &id,
                -32600,
                "Invalid Request: missing method",
            ));
        };
        if parsed.get("params").is_some_and(|params| !params.is_object()) {
            return Some(jsonrpc_error(&id, -32602, crate::ui_i18n::text(domain_ui_resources::PARAMS_MUST_BE_AN_OBJECT)));
        }

        match method {
            "initialize" => {
                let requested = parsed
                    .pointer("/params/protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let negotiated = if SUPPORTED_PROTOCOL_VERSIONS.contains(&requested) {
                    requested
                } else {
                    SUPPORTED_PROTOCOL_VERSIONS[0]
                };
                Some(jsonrpc_result(
                    &id,
                    &json!({
                        "protocolVersion": negotiated,
                        "capabilities": { "tools": { "listChanged": false } },
                        "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
                        "instructions": "Select repo via list_projects; omit only for a single repo. Single-repo servers ignore repo overrides; ambiguous selection lists choices. Search definitions, then pass returned qualified_name to get_code_snippet or trace_path; resolve ambiguity via suggestions, never guess symbols from tool names. When has_more, keep filters and use offset=next_offset plus generation; on stale generation restart at offset=0 without generation. Check evidence paths with check_index_coverage; read source for partial, stale or missing coverage. Tree-sitter and heuristic calls are incomplete: empty results do not prove absence; clean coverage does not prove completeness. Queries trigger background incremental checks; index_status.refresh reports queue, errors and backoff. After refresh_index returns indexing, poll index_status for completion/failure, do not refresh repeatedly.",
                    }),
                ))
            }
            "ping" => Some(jsonrpc_result(&id, &json!({}))),
            "tools/list" => Some(jsonrpc_result(&id, &json!({ "tools": tool_definitions() }))),
            "tools/call" => {
                let name = parsed
                    .pointer("/params/name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let arguments = parsed
                    .pointer("/params/arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                let Some(definition) = tool_definitions().iter().find(|tool| tool["name"] == name) else {
                    return Some(jsonrpc_error(&id, -32602, &crate::ui_i18n::format(domain_ui_resources::UNKNOWN_TOOL, &[("name", &format!("{}", name))])));
                };
                if let Err(error) = validate_arguments(&definition["inputSchema"], &arguments, "arguments") {
                    return Some(jsonrpc_error(&id, -32602, &error));
                }
                Some(jsonrpc_result(&id, &self.call_tool(name, &arguments)))
            }
            _ => Some(jsonrpc_error(&id, -32601, "Method not found")),
        }
    }
}

/// 读取索引统计为 JSON（index_status / refresh_index / list_projects 共用）。
fn stats_to_json(stats: &IndexStats) -> Value {
    json!({
        "status": if stats.needs_rebuild { "rebuild_required" } else { "ready" },
        "generation": stats.generation,
        "needs_rebuild": stats.needs_rebuild,
        "coverage": stats.coverage,
        "nodes": stats.nodes,
        "edges": stats.edges,
        "files": stats.files,
        "symbols": stats.symbols,
        "calls": stats.calls,
        "branch": stats.branch,
        "mode": stats.mode,
        "indexed_at": stats.indexed_at,
        "duration_ms": stats.duration_ms,
        "db_bytes": stats.db_bytes,
        "repo_path": stats.repo_path,
    })
}

/// list_projects / 多仓库错误提示共用的条目 JSON（统计 + repo 键）。
fn project_entry_json(repo_key: &str, stats: &IndexStats) -> Value {
    let mut value = stats_to_json(stats);
    value["repo"] = json!(repo_key);
    value
}

/// MCP content 信封：text 单行紧凑 JSON；非错误且可解析为对象时附
/// structuredContent（对齐参考项目 cbm_mcp_text_result 的双通道）。
fn text_result(value: &Value, is_error: bool) -> Value {
    let text = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
    let mut result = json!({ "content": [ { "type": "text", "text": text } ] });
    if is_error {
        result["isError"] = json!(true);
    } else if value.is_object() {
        result["structuredContent"] = value.clone();
    }
    result
}

fn jsonrpc_result(id: &Value, result: &Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn jsonrpc_error(id: &Value, code: i64, message: &str) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }).to_string()
}

/// 只校验本服务工具清单使用的 schema 子集，避免文档与实际参数规则各维护一套。
fn validate_arguments(schema: &Value, value: &Value, path: &str) -> std::result::Result<(), String> {
    let valid_type = match schema["type"].as_str() {
        Some("object") => value.is_object(),
        Some("array") => value.is_array(),
        Some("string") => value.is_string(),
        Some("integer") => value.is_i64() || value.is_u64(),
        Some("boolean") => value.is_boolean(),
        _ => true,
    };
    if !valid_type { return Err(crate::ui_i18n::format(domain_ui_resources::INVALID_TYPE_FOR_EXPECTED, &[("path", &format!("{}", path)), ("value0", &format!("{}", schema["type"]))])); }
    if let Some(values) = schema["enum"].as_array() {
        if !values.contains(value) { return Err(crate::ui_i18n::format(domain_ui_resources::IS_NOT_ONE_OF_THE_ALLOWED_VALUES, &[("path", &format!("{}", path))])); }
    }
    if let Some(minimum) = schema["minimum"].as_i64() {
        if value.as_i64().is_some_and(|number| number < minimum) {
            return Err(crate::ui_i18n::format(domain_ui_resources::MUST_NOT_BE_LESS_THAN, &[("path", &format!("{}", path)), ("minimum", &format!("{}", minimum))]));
        }
    }
    if let Some(maximum) = schema["maximum"].as_u64() {
        if value.as_u64().is_some_and(|number| number > maximum) {
            return Err(crate::ui_i18n::format(domain_ui_resources::MUST_NOT_EXCEED, &[("path", &format!("{}", path)), ("maximum", &format!("{}", maximum))]));
        }
    }
    if let Some(required) = schema["required"].as_array() {
        for key in required.iter().filter_map(Value::as_str) {
            if value.get(key).is_none() { return Err(crate::ui_i18n::format(domain_ui_resources::MISSING_REQUIRED_ARGUMENT_2, &[("path", &format!("{}", path)), ("key", &format!("{}", key))])); }
        }
    }
    if let Some(choices) = schema["anyOf"].as_array() {
        if !choices.iter().any(|choice| validate_arguments(choice, value, path).is_ok()) {
            return Err(crate::ui_i18n::format(domain_ui_resources::IS_MISSING_REQUIRED_QUERY_CRITERIA_SEE_TOOLS_LIST, &[("path", &format!("{}", path))]));
        }
    }
    if let Some(properties) = schema["properties"].as_object() {
        for (key, property) in properties {
            if let Some(value) = value.get(key) { validate_arguments(property, value, &format!("{path}.{key}"))?; }
        }
    }
    if let Some(items) = value.as_array() {
        for (index, item) in items.iter().enumerate() {
            validate_arguments(&schema["items"], item, &format!("{path}[{index}]"))?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 工具定义：中文展示标题，紧凑英文供模型读取；默认值由 schema 表达。
// ---------------------------------------------------------------------------

fn tool_definitions() -> &'static [Value] {
    // 标题缓存按语言分开，工具名称与 schema 身份不随语言变化。
    static ZH: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    static EN: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    let cache = match crate::ui_i18n::language() {
        crate::language::Language::ZhCn => &ZH,
        crate::language::Language::En => &EN,
    };
    cache.get_or_init(build_tool_definitions)
}

fn build_tool_definitions() -> Vec<Value> {
    let output_schema = json!({ "type": "object", "additionalProperties": true });
    // 多仓库模式共用的可选 repo 参数说明。
    let repo_prop = json!({
        "type": "string",
        "description": "Absolute worktree path or list_projects repo key; omit for a single repo."
    });
    let with_repo = |mut schema: Value| {
        schema["properties"]["repo"] = repo_prop.clone();
        schema
    };
    let mut definitions = vec![
        json!({
            "name": "list_projects",
            "title": crate::ui_i18n::text(domain_ui_resources::PROJECTS),
            "description": "List local indexed repos: keys, paths, file/symbol/edge counts and index times. Select a repo for subsequent tools.",
            "inputSchema": { "type": "object", "properties": {} },
            "outputSchema": output_schema,
        }),
        json!({
            "name": "search_symbols",
            "title": crate::ui_i18n::text(domain_ui_resources::SYMBOL_SEARCH),
            "description": "Search definition names/paths/signatures/docs. Exact names rank first; relax all-words only if empty. Filters AND. Returns locations, signatures and pages; full adds docs. Use qualified_name for snippet/trace. No match is not proof of absence.",
            "inputSchema": with_repo(json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Names/paths/signatures/docs; splits camelCase/snake_case." },
                    "label": { "type": "string", "description": "Node label: Function/Method/Class/Struct/Interface/Enum/Trait/Type/Field." },
                    "file_pattern": { "type": "string", "description": "Repo-relative glob (* ? [...]), e.g. src/git/*; not regex." },
                    "name_pattern": { "type": "string", "description": "Name regex, e.g. ^(push|pull).*" },
                    "qn_pattern": { "type": "string", "description": "Qualified-name regex for module/type scope." },
                    "offset": { "type": "integer", "minimum": 0, "default": 0, "description": "Use next_offset with unchanged filters and generation." },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 200, "default": 10 },
                    "detail": { "type": "string", "enum": ["compact", "full"], "default": "compact" }
                },
                "required": ["query"]
            })),
            "outputSchema": output_schema,
        }),
        json!({
            "name": "get_symbol_detail",
            "title": crate::ui_i18n::text(domain_ui_resources::SYMBOL_DETAILS),
            "description": "Definition location, direct callers/callees and source (max 200 lines). On status=ambiguous, retry a suggestions qualified_name. Use names from search, not tool names.",
            "inputSchema": with_repo(json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Symbol name or exact qualified_name from search." }
                },
                "required": ["name"]
            })),
            "outputSchema": output_schema,
        }),
        json!({
            "name": "trace_path",
            "title": crate::ui_i18n::text(domain_ui_resources::CALL_TRACING),
            "description": "Traverse CALLS with risk labels. Totals count distinct nodes within depth, not calls; directions page independently. hop=1 call_sites has count/lines (max 100, truncated); missing call_sites.file_path uses row file_path. Heuristic links may miss calls. full restores all fields and notes.",
            "inputSchema": with_repo(json!({
                "type": "object",
                "properties": {
                    "function_name": { "type": "string", "description": "Function/method name; prefer qualified_name from search." },
                    "direction": { "type": "string", "enum": ["inbound", "outbound", "both"], "default": "both", "description": "inbound=callers; outbound=callees." },
                    "depth": { "type": "integer", "minimum": 1, "maximum": 8, "default": 3, "description": "BFS depth." },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 200, "default": 100, "description": "Nodes per direction." },
                    "offset": { "type": "integer", "minimum": 0, "default": 0, "description": "Skip per direction; use next_offset with unchanged args and generation." },
                    "risk_labels": { "type": "boolean", "default": true },
                    "detail": { "type": "string", "enum": ["compact", "full"], "default": "compact" }
                },
                "required": ["function_name"]
            })),
            "outputSchema": output_schema,
        }),
        json!({
            "name": "get_architecture",
            "title": crate::ui_i18n::text(domain_ui_resources::ARCHITECTURE_OVERVIEW),
            "description": "Repo overview: node/edge totals and types, languages, top-level folder symbol density and top 10 fan-in hotspots.",
            "inputSchema": with_repo(json!({ "type": "object", "properties": {} })),
            "outputSchema": output_schema,
        }),
        json!({
            "name": "detect_changes",
            "title": crate::ui_i18n::text(domain_ui_resources::CHANGE_IMPACT_ANALYSIS),
            "description": "Impact of unstaged, staged and untracked changes: files, affected symbols and upstream callers with risk labels. Optional base_branch includes committed changes via three-dot diff.",
            "inputSchema": with_repo(json!({
                "type": "object",
                "properties": {
                    "scope": { "type": "string", "enum": ["files", "symbols"], "default": "symbols", "description": "files=paths only; symbols=files, symbols and callers." },
                    "base_branch": { "type": "string", "description": "Three-dot diff base; includes working-tree changes." }
                }
            })),
            "outputSchema": output_schema,
        }),
        json!({
            "name": "index_status",
            "title": crate::ui_i18n::text(domain_ui_resources::INDEX_STATUS),
            "description": "Index stats and this process's refresh queue/errors/backoff. If empty or needs_rebuild, use refresh_index; old relations are blocked until rebuilt.",
            "inputSchema": with_repo(json!({ "type": "object", "properties": {} })),
            "outputSchema": output_schema,
        }),
        json!({
            "name": "refresh_index",
            "title": crate::ui_i18n::text(domain_ui_resources::REFRESH_INDEX),
            "description": "Refresh index; first build needs an absolute repo path. Returns ready or indexing: poll index_status, do not repeat refresh. An active local job is reused for incremental; full is rejected until it finishes.",
            "inputSchema": with_repo(json!({
                "type": "object",
                "properties": {
                    "mode": { "type": "string", "enum": ["incremental", "full"], "default": "incremental", "description": "incremental=mtime+size check; full=rebuild." }
                }
            })),
            "outputSchema": output_schema,
        }),
    ];
    let mut search_graph = definitions[1].clone();
    search_graph["name"] = json!("search_graph");
    search_graph["title"] = json!(crate::ui_i18n::text(domain_ui_resources::STRUCTURE_AND_KEYWORD_SEARCH));
    search_graph["description"] = json!("Find definitions by keywords and/or filters (AND). Without query, supply a regex/glob/label. Returns locations, signatures and total/has_more/next_offset; full adds docs. Use qualified_name for snippet/trace. No match is not proof of absence.");
    search_graph["inputSchema"].as_object_mut().unwrap().remove("required");
    search_graph["inputSchema"]["anyOf"] = json!([
        { "required": ["query"] }, { "required": ["name_pattern"] },
        { "required": ["qn_pattern"] }, { "required": ["file_pattern"] }, { "required": ["label"] }
    ]);
    let mut snippet = definitions[2].clone();
    snippet["name"] = json!("get_code_snippet");
    snippet["title"] = json!(crate::ui_i18n::text(domain_ui_resources::DEFINITION_SOURCE));
    snippet["description"] = json!("Read a definition found by search_graph/search_symbols, not a search tool. Returns signature, docs and source.lines/start_line (max 200; source.truncated). On ambiguity, retry a suggestions qualified_name. For calls use trace_path.");
    let name_schema = snippet["inputSchema"]["properties"]["name"].clone();
    snippet["inputSchema"]["properties"].as_object_mut().unwrap().remove("name");
    snippet["inputSchema"]["properties"]["qualified_name"] = name_schema;
    snippet["inputSchema"]["required"] = json!(["qualified_name"]);
    definitions.extend([search_graph, snippet]);
    definitions.push(json!({
        "name": "check_index_coverage", "title": crate::ui_i18n::text(domain_ui_resources::INDEX_COVERAGE_CHECK),
        "description": "Check evidence files: coverage status, parse gaps, unresolved calls and metadata freshness. unresolved_no_candidate=no same-language repo definition; unresolved_with_candidates=target uncertain. Neither is a parse error rate. Read source for gaps/stale/missing coverage; clean status is no proof of completeness. Supply paths or scopes.",
        "inputSchema": with_repo(json!({ "type": "object", "properties": {
            "paths": { "type": "array", "items": { "type": "string" }, "description": "Exact repo-relative file paths." },
            "scopes": { "type": "array", "items": { "type": "string" }, "description": "Directory prefixes; '.' covers the repo." },
            "offset": { "type": "integer", "minimum": 0, "default": 0, "description": "Use next_offset with unchanged args and generation." },
            "limit": { "type": "integer", "minimum": 1, "maximum": 200, "default": 50 }
        }, "anyOf": [{ "required": ["paths"] }, { "required": ["scopes"] }] })),
        "outputSchema": output_schema
    }));
    for definition in &mut definitions {
        let read_only = definition["name"] != "refresh_index";
        definition["annotations"] = json!({ "readOnlyHint": read_only, "destructiveHint": false,
            "idempotentHint": true, "openWorldHint": false });
        if definition["inputSchema"]["properties"].get("offset").is_some() {
            definition["inputSchema"]["properties"]["offset"]["maximum"] = json!(i64::MAX);
        }
        if matches!(definition["name"].as_str(), Some("search_symbols" | "search_graph" | "trace_path" | "get_code_snippet" | "get_symbol_detail" | "get_architecture" | "check_index_coverage" | "detect_changes")) {
            definition["inputSchema"]["properties"]["generation"] = json!({ "type": "string", "description": "Reuse result generation; if stale, omit and restart at offset=0." });
        }
    }
    definitions
}

// ---------------------------------------------------------------------------
// stdio 主循环
// ---------------------------------------------------------------------------

/// MCP 服务器入口：进入 stdin 消息循环直到 EOF。`Some(repo_path)` 为单仓库
/// 模式，`None` 为多仓库模式（见模块文档）。返回进程退出码。
pub fn run(repo_path: Option<&Path>) -> i32 {
    let server = match McpServer::new(repo_path) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("[khaslana-mcp] 启动失败：{error}");
            return 2;
        }
    };
    eprintln!(
        "[khaslana-mcp] {} 就绪（{}，{} 个工具）",
        SERVER_NAME,
        if repo_path.is_some() {
            "单仓库模式"
        } else {
            "多仓库模式"
        },
        tool_definitions().len()
    );

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    match transport::serve(&server, &mut stdin.lock(), &mut stdout.lock()) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("[khaslana-mcp] 连接结束：{error}");
            1
        }
    }
}
