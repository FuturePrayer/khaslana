//! MCP 命令预检错误，不改写命令或参数。
use crate::ui_i18n::UiMessage;
pub const INVALID_COMMAND: UiMessage = UiMessage::new("workflow.mcp_process.invalid_command", "请填写有效的 MCP 启动命令", "Enter a valid MCP launch command");
#[cfg(windows)]
pub const COMMAND_NOT_FOUND: UiMessage = UiMessage::new("workflow.mcp_process.command_not_found", "找不到 MCP 启动命令 {command}；请确认已安装并加入 PATH，或填写完整路径。使用 npx 需安装 Node.js（含 npm），安装后重启 Khaslana。", "MCP launch command {command} was not found. Install it and add it to PATH, or enter its full path. Using npx requires Node.js (including npm); restart Khaslana after installation.");
