//! 工作流本模块的稳定可见消息。
use khaslana::ui_i18n::UiMessage;
pub const ADVANCED_ARGUMENTS_ARE_NOT_VALID_JSON5: UiMessage = UiMessage::new("workflow_editor.v2.advanced_arguments_are_not_valid_json5", "高级参数不是有效的 JSON5：{error}", "Advanced arguments are not valid JSON5: {error}");
pub const INVALID_FORMAT: UiMessage = UiMessage::new("workflow_editor.v2.invalid_format", "{value0}格式错误：{error}", "Invalid {value0} format: {error}");
pub const ENTER: UiMessage = UiMessage::new("workflow_editor.v2.enter", "请填写{value0}", "Enter {value0}");
pub const ADVANCED_ARGUMENTS_MUST_BE_AN_OBJECT: UiMessage = UiMessage::new("workflow_editor.v2.advanced_arguments_must_be_an_object", "高级参数必须是对象", "Advanced arguments must be an object");
pub const MCP_TOOL: UiMessage = UiMessage::new("workflow_editor.v2.mcp_tool", "MCP 工具", "MCP tool");
pub const JAVASCRIPT_SCRIPT: UiMessage = UiMessage::new("workflow_editor.v2.javascript_script", "JavaScript 脚本", "JavaScript script");
