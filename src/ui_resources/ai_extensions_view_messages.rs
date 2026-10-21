//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const STAGE_FAILED_UNEXPECTEDLY_PAYLOAD: UiMessage = UiMessage::new("views.ai_extensions_view.stage_failed_unexpectedly_payload", "{stage}异常：{payload}", "{stage} failed unexpectedly: {payload}");
pub const FAILED_TO_OPEN_THE_SKILL_FOLDER_ERROR: UiMessage = UiMessage::new("views.ai_extensions_view.failed_to_open_the_skill_folder_error", "打开 Skill 目录失败：{error}", "Failed to open the Skill folder: {error}");
pub const SKILL_NAME_INSTALLED: UiMessage = UiMessage::new("views.ai_extensions_view.skill_name_installed", "Skill {name} 已安装", "Skill {name} installed");
pub const SKILL_NAME_REMOVED: UiMessage = UiMessage::new("views.ai_extensions_view.skill_name_removed", "Skill {name} 已移除", "Skill {name} removed");
pub const MCP_SERVICE_NAME_SAVED: UiMessage = UiMessage::new("views.ai_extensions_view.mcp_service_name_saved", "MCP 服务 {name} 已保存", "MCP service {name} saved");
pub const MCP_SERVICE_NAME_REMOVED: UiMessage = UiMessage::new("views.ai_extensions_view.mcp_service_name_removed", "MCP 服务 {name} 已移除", "MCP service {name} removed");
pub const FILES_FILES: UiMessage = UiMessage::new("views.ai_extensions_view.files_files", "文件：{files}", "Files: {files}");
pub const REMOVE_NAME: UiMessage = UiMessage::new("views.ai_extensions_view.remove_name", "确定移除 {name}？", "Remove {name}?");
pub const MCP_TOOLS_TOOLS_RECORDED_ENABLED_COUNT_ALLOWED: UiMessage = UiMessage::new("views.ai_extensions_view.mcp_tools_tools_recorded_enabled_count_allowed", "工具记录 {mcp_tools} 个 · 白名单 {enabled_count} 个", "{mcp_tools} tools recorded · {enabled_count} allowed");
pub const EXAMPLE_MY_LOCAL_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.example_my_local_service", "例如 my-local-service", "Example: my-local-service");
pub const EXAMPLE_NPX_UVX_OR_THE_FULL_SERVICE_EXECUTABLE_PATH: UiMessage = UiMessage::new("views.ai_extensions_view.example_npx_uvx_or_the_full_service_executable_path", "例如 npx、uvx 或服务程序完整路径", "Example: npx, uvx, or the full service executable path");
pub const ONE_STARTUP_ARGUMENT_PER_LINE: UiMessage = UiMessage::new("views.ai_extensions_view.one_startup_argument_per_line", "每行一个启动参数", "One startup argument per line");
pub const CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY: UiMessage = UiMessage::new("views.ai_extensions_view.cannot_locate_the_application_data_directory", "无法定位应用数据目录", "Cannot locate the application data directory");
pub const READ_AI_EXTENSION_SETTINGS: UiMessage = UiMessage::new("views.ai_extensions_view.read_ai_extension_settings", "读取 AI 扩展设置", "Read AI extension settings");
pub const PREVIEW_SKILL: UiMessage = UiMessage::new("views.ai_extensions_view.preview_skill", "预览 Skill", "Preview Skill");
pub const SELECT_A_FOLDER_CONTAINING_SKILL_MD: UiMessage = UiMessage::new("views.ai_extensions_view.select_a_folder_containing_skill_md", "选择包含 SKILL.md 的文件夹", "Select a folder containing SKILL.md");
pub const INSTALL_SKILL: UiMessage = UiMessage::new("views.ai_extensions_view.install_skill", "安装 Skill", "Install Skill");
pub const REMOVE_SKILL: UiMessage = UiMessage::new("views.ai_extensions_view.remove_skill", "移除 Skill", "Remove Skill");
pub const TEST_MCP_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.test_mcp_service", "测试 MCP 服务", "Test MCP service");
pub const THE_STARTUP_COMMAND_OR_ARGUMENTS_CHANGED_TEST_THE_CONNECTION_AGAIN: UiMessage = UiMessage::new("views.ai_extensions_view.the_startup_command_or_arguments_changed_test_the_connection_again", "启动命令或参数已更改，请重新测试连接", "The startup command or arguments changed. Test the connection again.");
pub const SAVE_MCP_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.save_mcp_service", "保存 MCP 服务", "Save MCP service");
pub const REMOVE_MCP_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.remove_mcp_service", "移除 MCP 服务", "Remove MCP service");
pub const UPDATE_MCP_SERVICE_STATUS: UiMessage = UiMessage::new("views.ai_extensions_view.update_mcp_service_status", "更新 MCP 服务状态", "Update MCP service status");
pub const INSTALL_LOCAL_SKILL: UiMessage = UiMessage::new("views.ai_extensions_view.install_local_skill", "安装本地 Skill", "Install local Skill");
pub const READ_SKILL_MD_FROM_A_FOLDER_REVIEW_ITS_CONTENTS_THEN: UiMessage = UiMessage::new("views.ai_extensions_view.read_skill_md_from_a_folder_review_its_contents_then", "从文件夹读取 SKILL.md，确认内容后导入应用数据目录。", "Read SKILL.md from a folder, review its contents, then import it into the application data directory.");
pub const NO_FOLDER_SELECTED: UiMessage = UiMessage::new("views.ai_extensions_view.no_folder_selected", "尚未选择文件夹", "No folder selected");
pub const SELECTING: UiMessage = UiMessage::new("views.ai_extensions_view.selecting", "正在选择…", "Selecting…");
pub const SELECT_FOLDER: UiMessage = UiMessage::new("views.ai_extensions_view.select_folder", "选择文件夹", "Select folder");
pub const IMPORTED_SKILLS_CAN_BE_VIEWED_AND_REMOVED_RUNNING_A_SKILL: UiMessage = UiMessage::new("views.ai_extensions_view.imported_skills_can_be_viewed_and_removed_running_a_skill", "导入后可查看和移除；运行 Skill 时仍需确认工具权限。", "Imported Skills can be viewed and removed. Running a Skill still requires tool permission approval.");
pub const CANCEL: UiMessage = UiMessage::new("views.ai_extensions_view.cancel", "取消", "Cancel");
pub const CONFIRM_INSTALLATION: UiMessage = UiMessage::new("views.ai_extensions_view.confirm_installation", "确认安装", "Confirm installation");
pub const SKILL_DETAILS: UiMessage = UiMessage::new("views.ai_extensions_view.skill_details", "Skill 详情", "Skill details");
pub const REQUIRED_MCP_TOOLS_ARE_SHOWN_FOR_APPROVAL_BEFORE_THE_WORKFLOW: UiMessage = UiMessage::new("views.ai_extensions_view.required_mcp_tools_are_shown_for_approval_before_the_workflow", "工作流调用前会展示所需 MCP 工具并请求确认。", "Required MCP tools are shown for approval before the workflow runs.");
pub const CLOSE: UiMessage = UiMessage::new("views.ai_extensions_view.close", "关闭", "Close");
pub const THIS_DELETES_THE_SKILL_FOLDER_FROM_THE_APPLICATION_DATA_DIRECTORY: UiMessage = UiMessage::new("views.ai_extensions_view.this_deletes_the_skill_folder_from_the_application_data_directory", "将删除应用数据目录中的该 Skill 文件夹。", "This deletes the Skill folder from the application data directory.");
pub const CONFIRM_REMOVAL: UiMessage = UiMessage::new("views.ai_extensions_view.confirm_removal", "确认移除", "Confirm removal");
pub const ENTER_A_COMMAND_AND_ARGUMENTS_TO_SAVE_THE_SERVICE_CONNECTS: UiMessage = UiMessage::new("views.ai_extensions_view.enter_a_command_and_arguments_to_save_the_service_connects", "填写命令和参数即可保存；首次使用时连接。测试连接只读取工具列表。", "Enter a command and arguments to save. The service connects on first use. Connection tests only read the tool list.");
pub const SERVICE_ID: UiMessage = UiMessage::new("views.ai_extensions_view.service_id", "服务 ID", "Service ID");
pub const STARTUP_COMMAND: UiMessage = UiMessage::new("views.ai_extensions_view.startup_command", "启动命令", "Startup command");
pub const YOU_CAN_ENTER_NPX_DIRECTLY_WITHOUT_CMD_PUT_EACH_ARGUMENT: UiMessage = UiMessage::new("views.ai_extensions_view.you_can_enter_npx_directly_without_cmd_put_each_argument", "可直接填写 npx，无需添加 .cmd；每个参数单独一行，无需加引号。", "You can enter npx directly without .cmd. Put each argument on its own line without quotes.");
pub const STARTUP_ARGUMENTS_ONE_PER_LINE: UiMessage = UiMessage::new("views.ai_extensions_view.startup_arguments_one_per_line", "启动参数（每行一个）", "Startup arguments (one per line)");
pub const CONNECTION_TEST_SUCCEEDED: UiMessage = UiMessage::new("views.ai_extensions_view.connection_test_succeeded", "本次连接测试成功", "Connection test succeeded");
pub const THIS_CONFIGURATION_HAS_NOT_BEEN_VERIFIED_YOU_CAN_STILL_SAVE: UiMessage = UiMessage::new("views.ai_extensions_view.this_configuration_has_not_been_verified_you_can_still_save", "尚未验证当前配置，可直接保存", "This configuration has not been verified; you can still save it");
pub const TEST_CONNECTION: UiMessage = UiMessage::new("views.ai_extensions_view.test_connection", "测试连接", "Test connection");
pub const CONNECTING_TO_THE_MCP_SERVICE_AND_READING_ITS_TOOL_LIST: UiMessage = UiMessage::new("views.ai_extensions_view.connecting_to_the_mcp_service_and_reading_its_tool_list", "正在连接 MCP 服务并读取工具列表…", "Connecting to the MCP service and reading its tool list…");
pub const AUTOMATICALLY_DISCOVER_TOOLS_APPROVAL_IS_STILL_REQUIRED_BEFORE_RUNNING: UiMessage = UiMessage::new("views.ai_extensions_view.automatically_discover_tools_approval_is_still_required_before_running", "自动发现工具（运行前仍需授权）", "Automatically discover tools (approval is still required before running)");
pub const DISABLE_TOOLS_YOU_DO_NOT_NEED_READ_WRITE_TYPES_ARE: UiMessage = UiMessage::new("views.ai_extensions_view.disable_tools_you_do_not_need_read_write_types_are", "可关闭不需要的工具；读写类型用于运行前的提示，未知工具默认按写入处理。", "Disable tools you do not need. Read/write types are shown before running; unknown tools are treated as writes.");
pub const READ: UiMessage = UiMessage::new("views.ai_extensions_view.read", "读取", "Read");
pub const WRITE: UiMessage = UiMessage::new("views.ai_extensions_view.write", "写入", "Write");
pub const COLLAPSE_ADVANCED_OPTIONS: UiMessage = UiMessage::new("views.ai_extensions_view.collapse_advanced_options", "收起高级配置", "Collapse advanced options");
pub const ADVANCED_OPTIONS_OPTIONAL: UiMessage = UiMessage::new("views.ai_extensions_view.advanced_options_optional", "高级配置（可选）", "Advanced options (optional)");
pub const ENABLE_AT_LEAST_ONE_TOOL_BEFORE_SAVING_THE_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.enable_at_least_one_tool_before_saving_the_service", "请至少启用一个工具后保存服务", "Enable at least one tool before saving the service");
pub const UNKNOWN_DISCOVERED_TOOLS_REQUEST_WRITE_ACCESS_WITH_DISCOVERY_OFF_ONLY: UiMessage = UiMessage::new("views.ai_extensions_view.unknown_discovered_tools_request_write_access_with_discovery_off_only", "自动发现的未知工具按写入请求授权；关闭自动发现后只允许白名单工具。", "Unknown discovered tools request write access. With discovery off, only allowed tools are available.");
pub const EDIT_MCP_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.edit_mcp_service", "编辑 MCP 服务", "Edit MCP service");
pub const ADD_MCP_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.add_mcp_service", "添加 MCP 服务", "Add MCP service");
pub const SAVE_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.save_service", "保存服务", "Save service");
pub const WORKFLOW_STEPS_REFERENCING_THIS_SERVICE_WILL_NO_LONGER_RUN: UiMessage = UiMessage::new("views.ai_extensions_view.workflow_steps_referencing_this_service_will_no_longer_run", "工作流中引用此服务的步骤将无法再运行。", "Workflow steps referencing this service will no longer run.");
pub const BUILT_IN_BROWSER_EDGE_SERVICE: UiMessage = UiMessage::new("views.ai_extensions_view.built_in_browser_edge_service", "内置 browser.edge 服务", "Built-in browser.edge service");
pub const RUNTIME_COMPONENTS_ARE_READY: UiMessage = UiMessage::new("views.ai_extensions_view.runtime_components_are_ready", "运行组件已就绪", "Runtime components are ready");
pub const RUNTIME_COMPONENTS_ARE_NOT_INSTALLED_DOWNLOAD_AND_ENABLE_THEM_ON: UiMessage = UiMessage::new("views.ai_extensions_view.runtime_components_are_not_installed_download_and_enable_them_on", "运行组件尚未安装，可在运行环境页下载并启用", "Runtime components are not installed. Download and enable them on the Runtime page.");
pub const CLIENT_MANAGED_MICROSOFT_EDGE_MCP_APPROVAL_IS_REQUESTED_BEFORE_EACH: UiMessage = UiMessage::new("views.ai_extensions_view.client_managed_microsoft_edge_mcp_approval_is_requested_before_each", "客户端管理的 Microsoft Edge MCP；工作流调用前会逐次确认。", "Client-managed Microsoft Edge MCP; approval is requested before each workflow call.");

pub const MCP_ENABLED: UiMessage = UiMessage::new("views.ai_extensions_view.mcp_enabled", "MCP 服务 {name} 已启用", "MCP service {name} enabled");

pub const MCP_DISABLED: UiMessage = UiMessage::new("views.ai_extensions_view.mcp_disabled", "MCP 服务 {name} 已禁用", "MCP service {name} disabled");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    STAGE_FAILED_UNEXPECTEDLY_PAYLOAD,
    FAILED_TO_OPEN_THE_SKILL_FOLDER_ERROR,
    SKILL_NAME_INSTALLED,
    SKILL_NAME_REMOVED,
    MCP_SERVICE_NAME_SAVED,
    MCP_SERVICE_NAME_REMOVED,
    FILES_FILES,
    REMOVE_NAME,
    MCP_TOOLS_TOOLS_RECORDED_ENABLED_COUNT_ALLOWED,
    EXAMPLE_MY_LOCAL_SERVICE,
    EXAMPLE_NPX_UVX_OR_THE_FULL_SERVICE_EXECUTABLE_PATH,
    ONE_STARTUP_ARGUMENT_PER_LINE,
    CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY,
    READ_AI_EXTENSION_SETTINGS,
    PREVIEW_SKILL,
    SELECT_A_FOLDER_CONTAINING_SKILL_MD,
    INSTALL_SKILL,
    REMOVE_SKILL,
    TEST_MCP_SERVICE,
    THE_STARTUP_COMMAND_OR_ARGUMENTS_CHANGED_TEST_THE_CONNECTION_AGAIN,
    SAVE_MCP_SERVICE,
    REMOVE_MCP_SERVICE,
    UPDATE_MCP_SERVICE_STATUS,
    INSTALL_LOCAL_SKILL,
    READ_SKILL_MD_FROM_A_FOLDER_REVIEW_ITS_CONTENTS_THEN,
    NO_FOLDER_SELECTED,
    SELECTING,
    SELECT_FOLDER,
    IMPORTED_SKILLS_CAN_BE_VIEWED_AND_REMOVED_RUNNING_A_SKILL,
    CANCEL,
    CONFIRM_INSTALLATION,
    SKILL_DETAILS,
    REQUIRED_MCP_TOOLS_ARE_SHOWN_FOR_APPROVAL_BEFORE_THE_WORKFLOW,
    CLOSE,
    THIS_DELETES_THE_SKILL_FOLDER_FROM_THE_APPLICATION_DATA_DIRECTORY,
    CONFIRM_REMOVAL,
    ENTER_A_COMMAND_AND_ARGUMENTS_TO_SAVE_THE_SERVICE_CONNECTS,
    SERVICE_ID,
    STARTUP_COMMAND,
    YOU_CAN_ENTER_NPX_DIRECTLY_WITHOUT_CMD_PUT_EACH_ARGUMENT,
    STARTUP_ARGUMENTS_ONE_PER_LINE,
    CONNECTION_TEST_SUCCEEDED,
    THIS_CONFIGURATION_HAS_NOT_BEEN_VERIFIED_YOU_CAN_STILL_SAVE,
    TEST_CONNECTION,
    CONNECTING_TO_THE_MCP_SERVICE_AND_READING_ITS_TOOL_LIST,
    AUTOMATICALLY_DISCOVER_TOOLS_APPROVAL_IS_STILL_REQUIRED_BEFORE_RUNNING,
    DISABLE_TOOLS_YOU_DO_NOT_NEED_READ_WRITE_TYPES_ARE,
    READ,
    WRITE,
    COLLAPSE_ADVANCED_OPTIONS,
    ADVANCED_OPTIONS_OPTIONAL,
    ENABLE_AT_LEAST_ONE_TOOL_BEFORE_SAVING_THE_SERVICE,
    UNKNOWN_DISCOVERED_TOOLS_REQUEST_WRITE_ACCESS_WITH_DISCOVERY_OFF_ONLY,
    EDIT_MCP_SERVICE,
    ADD_MCP_SERVICE,
    SAVE_SERVICE,
    WORKFLOW_STEPS_REFERENCING_THIS_SERVICE_WILL_NO_LONGER_RUN,
    BUILT_IN_BROWSER_EDGE_SERVICE,
    RUNTIME_COMPONENTS_ARE_READY,
    RUNTIME_COMPONENTS_ARE_NOT_INSTALLED_DOWNLOAD_AND_ENABLE_THEM_ON,
    CLIENT_MANAGED_MICROSOFT_EDGE_MCP_APPROVAL_IS_REQUESTED_BEFORE_EACH,
    MCP_ENABLED,
    MCP_DISABLED,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
