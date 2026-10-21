//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const QUEUED_POSITION_N: UiMessage = UiMessage::new("views.code_index_view.queued_position_n", "排队中 · 第 {n} 位", "Queued · Position {n}");
pub const ENTRIES_REPOSITORIES_FILTER_BY_NAME_OR_PATH: UiMessage = UiMessage::new("views.code_index_view.entries_repositories_filter_by_name_or_path", "共 {entries} 个仓库；可按名称或路径过滤。", "{entries} repositories; filter by name or path.");
pub const THE_INDEX_USES_TREE_SITTER_TO_PARSE_THE_WORKING_TREE: UiMessage = UiMessage::new("views.code_index_view.the_index_uses_tree_sitter_to_parse_the_working_tree", "索引基于 tree-sitter 解析仓库工作区（遵循 .gitignore），提取文件、符号、调用与导入关系存入本机 SQLite 库；数据位于应用数据目录 code-index/ 下，按仓库哈希隔离。支持符号提取的语言：Rust、Python、JavaScript、TypeScript/TSX、Go、Java、C、C++、C#、PHP、Kotlin；其余文本文件仅登记为文件节点。", "The index uses tree-sitter to parse the working tree (respecting .gitignore), storing files, symbols, calls, and imports in a local SQLite database; data is stored under code-index/ in the application data directory, isolated by repository hash. Symbol extraction supports Rust, Python, JavaScript, TypeScript/TSX, Go, Java, C, C++, C#, PHP, and Kotlin. Other text files are registered as file nodes only.");
pub const INDEX_DISABLED_DATA_RETAINED_S: UiMessage = UiMessage::new("views.code_index_view.index_disabled_data_retained_s", "索引已停用，数据保留 · {s}", "Index disabled; data retained · {s}");
pub const FAILED_TO_SAVE_INDEX_SETTINGS_ERR: UiMessage = UiMessage::new("views.code_index_view.failed_to_save_index_settings_err", "保存索引设置失败：{err}", "Failed to save index settings: {err}");
pub const INDEX_SKIPPED_REPOSITORY_PATH_UNAVAILABLE_REPO_KEY: UiMessage = UiMessage::new("views.code_index_view.index_skipped_repository_path_unavailable_repo_key", "跳过索引：仓库路径不可用 {repo_key}", "Index skipped: repository path unavailable {repo_key}");
pub const FAILED_TO_DELETE_INDEX_DATA_ERR: UiMessage = UiMessage::new("views.code_index_view.failed_to_delete_index_data_err", "删除索引数据失败：{err}", "Failed to delete index data: {err}");
pub const CODE_INDEX_MESSAGE: UiMessage = UiMessage::new("views.code_index_view.code_index_message", "代码索引：{message}", "Code index: {message}");
pub const CODE_INDEX_COMPLETE_FILES_FILES_SYMBOLS_SYMBOLS_EDGES_RELATIONS: UiMessage = UiMessage::new("views.code_index_view.code_index_complete_files_files_symbols_symbols_edges_relations", "代码索引完成：{files} 文件 · {symbols} 符号 · {edges} 关系", "Code index complete: {files} files · {symbols} symbols · {edges} relations");
pub const CODE_INDEX_UPDATED_FILES_FILES_SYMBOLS_SYMBOLS_EDGES_RELATIONS: UiMessage = UiMessage::new("views.code_index_view.code_index_updated_files_files_symbols_symbols_edges_relations", "代码索引已更新：{files} 文件 · {symbols} 符号 · {edges} 关系", "Code index updated: {files} files · {symbols} symbols · {edges} relations");
pub const CODE_INDEXING_FAILED_ERROR: UiMessage = UiMessage::new("views.code_index_view.code_indexing_failed_error", "代码索引失败:{error}", "Code indexing failed: {error}");
pub const INDEXING_TASK_EXITED_UNEXPECTEDLY_PAYLOAD: UiMessage = UiMessage::new("views.code_index_view.indexing_task_exited_unexpectedly_payload", "索引任务异常退出：{payload}", "Indexing task exited unexpectedly: {payload}");
pub const FILES_FILES_SYMBOLS_SYMBOLS_EDGES_RELATIONS_CALLS_CALLS_F64_MB: UiMessage = UiMessage::new("views.code_index_view.files_files_symbols_symbols_edges_relations_calls_calls_f64_mb", "{files} 文件 · {symbols} 符号 · {edges} 关系（调用 {calls}）· {f64} MB · {indexed_at} · 分支 {branch}", "{files} files · {symbols} symbols · {edges} relations ({calls} calls) · {f64} MB · {indexed_at} · Branch {branch}");
pub const DELETE_THE_CODE_INDEX_DATA_FOR_DISPLAY_NAME: UiMessage = UiMessage::new("views.code_index_view.delete_the_code_index_data_for_display_name", "确认删除 {display_name} 的代码索引数据？", "Delete the code index data for {display_name}?");
pub const CANCELLING: UiMessage = UiMessage::new("views.code_index_view.cancelling", "取消中", "Cancelling");
pub const INDEXING: UiMessage = UiMessage::new("views.code_index_view.indexing", "索引中", "Indexing");
pub const INDEXED: UiMessage = UiMessage::new("views.code_index_view.indexed", "已索引", "Indexed");
pub const DISABLED: UiMessage = UiMessage::new("views.code_index_view.disabled", "已停用", "Disabled");
pub const NOT_INDEXED: UiMessage = UiMessage::new("views.code_index_view.not_indexed", "未索引", "Not indexed");
pub const MCP_SERVER: UiMessage = UiMessage::new("views.code_index_view.mcp_server", "MCP 服务器", "MCP server");
pub const MULTI_REPOSITORY_MODE_ONE_CONFIGURATION_SERVES_ALL_INDEXED_REPOSITORIES: UiMessage = UiMessage::new("views.code_index_view.multi_repository_mode_one_configuration_serves_all_indexed_repositories", "多仓库模式：一条配置服务所有已索引仓库", "Multi-repository mode: one configuration serves all indexed repositories");
pub const EXPOSE_CODE_INDEXING_AS_MCP_TOOLS_TO_EXTERNAL_AI_CLIENTS: UiMessage = UiMessage::new("views.code_index_view.expose_code_indexing_as_mcp_tools_to_external_ai_clients", "把代码索引以 MCP 工具暴露给外部 AI 工具（Claude Code / Cursor / ZCode 等）。命令：khaslana mcp（多仓库模式，工具按 repo 参数选择目标；也可写 khaslana mcp <仓库路径> 固定单仓库）；启动后首次使用仓库时自动建立或增量刷新索引。", "Expose code indexing as MCP tools to external AI clients (Claude Code / Cursor / ZCode, etc.). Command: khaslana mcp (multi-repository mode; tools select the target using the repo parameter). Use khaslana mcp <repository-path> to pin one repository. The first use of a repository automatically builds or incrementally refreshes its index.");
pub const STARTUP_COMMAND: UiMessage = UiMessage::new("views.code_index_view.startup_command", "启动命令", "Startup command");
pub const COPY_MCP_CONFIGURATION: UiMessage = UiMessage::new("views.code_index_view.copy_mcp_configuration", "复制 MCP 配置", "Copy MCP configuration");
pub const MCP_CONFIGURATION_COPIED: UiMessage = UiMessage::new("views.code_index_view.mcp_configuration_copied", "已复制 MCP 配置", "MCP configuration copied");
pub const REPOSITORIES: UiMessage = UiMessage::new("views.code_index_view.repositories", "仓库", "Repositories");
pub const FILTER_REPOSITORIES: UiMessage = UiMessage::new("views.code_index_view.filter_repositories", "过滤仓库", "Filter repositories");
pub const ENABLE_ALL: UiMessage = UiMessage::new("views.code_index_view.enable_all", "全部开启", "Enable all");
pub const DISABLE_ALL: UiMessage = UiMessage::new("views.code_index_view.disable_all", "全部关闭", "Disable all");
pub const ONLY_AFFECTS_FILTERED_REPOSITORIES_DISABLING_KEEPS_INDEX_DATA: UiMessage = UiMessage::new("views.code_index_view.only_affects_filtered_repositories_disabling_keeps_index_data", "仅作用于当前筛选结果；关闭保留索引数据", "Only affects filtered repositories; disabling keeps index data");
pub const NO_MATCHING_REPOSITORIES_FOR_BULK_OPERATIONS: UiMessage = UiMessage::new("views.code_index_view.no_matching_repositories_for_bulk_operations", "没有匹配仓库，无法批量操作", "No matching repositories for bulk operations");
pub const BULK_INDEXING: UiMessage = UiMessage::new("views.code_index_view.bulk_indexing", "批量索引", "Bulk indexing");
pub const NO_MATCHING_REPOSITORIES: UiMessage = UiMessage::new("views.code_index_view.no_matching_repositories", "没有匹配的仓库。", "No matching repositories.");
pub const NO_REPOSITORIES_ARE_OPEN_OPEN_ONE_FROM_THE_REPOSITORY_SWITCHER: UiMessage = UiMessage::new("views.code_index_view.no_repositories_are_open_open_one_from_the_repository_switcher", "当前没有打开的仓库。先在仓库切换下拉中打开一个仓库，再回到此页启用索引。", "No repositories are open. Open one from the repository switcher, then return here to enable indexing.");
pub const INDEXING_2: UiMessage = UiMessage::new("views.code_index_view.indexing_2", "正在索引…", "Indexing…");
pub const NO_INDEX_YET_A_FULL_INDEX_WILL_BE_BUILT_AUTOMATICALLY: UiMessage = UiMessage::new("views.code_index_view.no_index_yet_a_full_index_will_be_built_automatically", "尚未建立索引 · 即将自动建立全量索引", "No index yet · A full index will be built automatically");
pub const NOT_INDEXED_ENABLE_THE_SWITCH_TO_BUILD_A_FULL_INDEX: UiMessage = UiMessage::new("views.code_index_view.not_indexed_enable_the_switch_to_build_a_full_index", "未建立索引 · 开启开关后自动建立全量索引", "Not indexed · Enable the switch to build a full index automatically");
pub const CANCEL: UiMessage = UiMessage::new("views.code_index_view.cancel", "取消", "Cancel");
pub const INCREMENTAL_UPDATE: UiMessage = UiMessage::new("views.code_index_view.incremental_update", "增量更新", "Incremental update");
pub const REBUILD_INDEX: UiMessage = UiMessage::new("views.code_index_view.rebuild_index", "重建索引", "Rebuild index");
pub const DELETE_INDEX_DATA: UiMessage = UiMessage::new("views.code_index_view.delete_index_data", "删除索引数据", "Delete index data");
pub const CANCELLING_2: UiMessage = UiMessage::new("views.code_index_view.cancelling_2", "取消中…", "Cancelling…");
pub const ONLY_ONE_INDEXING_TASK_CAN_RUN_AT_A_TIME: UiMessage = UiMessage::new("views.code_index_view.only_one_indexing_task_can_run_at_a_time", "同一时间只能有一个索引任务", "Only one indexing task can run at a time");
pub const CODE_INDEXING_DISABLED_EXISTING_INDEX_DATA_RETAINED: UiMessage = UiMessage::new("views.code_index_view.code_indexing_disabled_existing_index_data_retained", "代码索引已关闭（已有索引数据保留）", "Code indexing disabled (existing index data retained)");
pub const CODE_INDEX_UNAVAILABLE_CANNOT_CREATE_THE_DATA_DIRECTORY: UiMessage = UiMessage::new("views.code_index_view.code_index_unavailable_cannot_create_the_data_directory", "代码索引不可用：数据目录无法创建", "Code index unavailable: cannot create the data directory");
pub const PREPARING_INDEX: UiMessage = UiMessage::new("views.code_index_view.preparing_index", "准备索引…", "Preparing index…");
pub const CANCELLING_CODE_INDEXING: UiMessage = UiMessage::new("views.code_index_view.cancelling_code_indexing", "正在取消代码索引…", "Cancelling code indexing…");
pub const CODE_INDEXING_CANCELLED: UiMessage = UiMessage::new("views.code_index_view.code_indexing_cancelled", "代码索引已取消", "Code indexing cancelled");
pub const CANNOT_DELETE_DATA_WHILE_INDEXING_OR_CANCELLATION_IS_IN_PROGRESS: UiMessage = UiMessage::new("views.code_index_view.cannot_delete_data_while_indexing_or_cancellation_is_in_progress", "索引运行或取消中，暂不能删除数据", "Cannot delete data while indexing or cancellation is in progress");
pub const INDEX_DATA_DELETED: UiMessage = UiMessage::new("views.code_index_view.index_data_deleted", "索引数据已删除", "Index data deleted");
pub const FAILED_TO_READ_CODE_INDEX_SETTINGS: UiMessage = UiMessage::new("views.code_index_view.failed_to_read_code_index_settings", "读取代码索引设置失败", "Failed to read code index settings");
pub const CODE_INDEX_NO_CONTENT_CHANGES: UiMessage = UiMessage::new("views.code_index_view.code_index_no_content_changes", "代码索引：内容无变化", "Code index: no content changes");
pub const A_FULL_INDEX_MUST_BE_REBUILT_TO_RESTORE_SEARCH_THE: UiMessage = UiMessage::new("views.code_index_view.a_full_index_must_be_rebuilt_to_restore_search_the", "删除后需要重新建立全量索引才能恢复搜索能力；不影响仓库本身。", "A full index must be rebuilt to restore search. The repository itself is not affected.");
pub const CONFIRM_DELETION: UiMessage = UiMessage::new("views.code_index_view.confirm_deletion", "确认删除", "Confirm deletion");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    QUEUED_POSITION_N,
    ENTRIES_REPOSITORIES_FILTER_BY_NAME_OR_PATH,
    THE_INDEX_USES_TREE_SITTER_TO_PARSE_THE_WORKING_TREE,
    INDEX_DISABLED_DATA_RETAINED_S,
    FAILED_TO_SAVE_INDEX_SETTINGS_ERR,
    INDEX_SKIPPED_REPOSITORY_PATH_UNAVAILABLE_REPO_KEY,
    FAILED_TO_DELETE_INDEX_DATA_ERR,
    CODE_INDEX_MESSAGE,
    CODE_INDEX_COMPLETE_FILES_FILES_SYMBOLS_SYMBOLS_EDGES_RELATIONS,
    CODE_INDEX_UPDATED_FILES_FILES_SYMBOLS_SYMBOLS_EDGES_RELATIONS,
    CODE_INDEXING_FAILED_ERROR,
    INDEXING_TASK_EXITED_UNEXPECTEDLY_PAYLOAD,
    FILES_FILES_SYMBOLS_SYMBOLS_EDGES_RELATIONS_CALLS_CALLS_F64_MB,
    DELETE_THE_CODE_INDEX_DATA_FOR_DISPLAY_NAME,
    CANCELLING,
    INDEXING,
    INDEXED,
    DISABLED,
    NOT_INDEXED,
    MCP_SERVER,
    MULTI_REPOSITORY_MODE_ONE_CONFIGURATION_SERVES_ALL_INDEXED_REPOSITORIES,
    EXPOSE_CODE_INDEXING_AS_MCP_TOOLS_TO_EXTERNAL_AI_CLIENTS,
    STARTUP_COMMAND,
    COPY_MCP_CONFIGURATION,
    MCP_CONFIGURATION_COPIED,
    REPOSITORIES,
    FILTER_REPOSITORIES,
    ENABLE_ALL,
    DISABLE_ALL,
    ONLY_AFFECTS_FILTERED_REPOSITORIES_DISABLING_KEEPS_INDEX_DATA,
    NO_MATCHING_REPOSITORIES_FOR_BULK_OPERATIONS,
    BULK_INDEXING,
    NO_MATCHING_REPOSITORIES,
    NO_REPOSITORIES_ARE_OPEN_OPEN_ONE_FROM_THE_REPOSITORY_SWITCHER,
    INDEXING_2,
    NO_INDEX_YET_A_FULL_INDEX_WILL_BE_BUILT_AUTOMATICALLY,
    NOT_INDEXED_ENABLE_THE_SWITCH_TO_BUILD_A_FULL_INDEX,
    CANCEL,
    INCREMENTAL_UPDATE,
    REBUILD_INDEX,
    DELETE_INDEX_DATA,
    CANCELLING_2,
    ONLY_ONE_INDEXING_TASK_CAN_RUN_AT_A_TIME,
    CODE_INDEXING_DISABLED_EXISTING_INDEX_DATA_RETAINED,
    CODE_INDEX_UNAVAILABLE_CANNOT_CREATE_THE_DATA_DIRECTORY,
    PREPARING_INDEX,
    CANCELLING_CODE_INDEXING,
    CODE_INDEXING_CANCELLED,
    CANNOT_DELETE_DATA_WHILE_INDEXING_OR_CANCELLATION_IS_IN_PROGRESS,
    INDEX_DATA_DELETED,
    FAILED_TO_READ_CODE_INDEX_SETTINGS,
    CODE_INDEX_NO_CONTENT_CHANGES,
    A_FULL_INDEX_MUST_BE_REBUILT_TO_RESTORE_SEARCH_THE,
    CONFIRM_DELETION,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
