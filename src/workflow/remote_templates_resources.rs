//! 远端模板目录与安装校验；模板内容和文件名只作为原始参数。
use crate::ui_i18n::UiMessage;
pub const TEMPLATE_COUNT_LIMIT: UiMessage = UiMessage::new("workflow.remote_templates.count_limit", "远端工作流模板数量超过限制", "The remote workflow template count exceeds the limit");
pub const TEMPLATE_TOO_LARGE: UiMessage = UiMessage::new("workflow.remote_templates.template_too_large", "远端工作流模板过大：{name}", "Remote workflow template is too large: {name}");
pub const TOTAL_SIZE_LIMIT: UiMessage = UiMessage::new("workflow.remote_templates.total_size_limit", "远端工作流模板总大小超过限制", "The total size of remote workflow templates exceeds the limit");
pub const INVALID_NAMED_UTF8: UiMessage = UiMessage::new("workflow.remote_templates.invalid_named_utf8", "远端工作流模板不是 UTF-8：{name}", "Remote workflow template is not UTF-8: {name}");
pub const EMPTY_CATALOG: UiMessage = UiMessage::new("workflow.remote_templates.empty_catalog", "远端目录没有可用的工作流模板", "The remote directory has no usable workflow templates");
pub const INVALID_FILE_NAME: UiMessage = UiMessage::new("workflow.remote_templates.invalid_file_name", "远端工作流模板文件名无效", "Invalid remote workflow template file name");
pub const INVALID_UTF8: UiMessage = UiMessage::new("workflow.remote_templates.invalid_utf8", "远端工作流模板不是 UTF-8", "Remote workflow template is not UTF-8");
pub const LOCAL_TEMPLATE_EXISTS: UiMessage = UiMessage::new("workflow.remote_templates.local_template_exists", "本地已存在同名模板：{name}", "A local template with the same name already exists: {name}");
