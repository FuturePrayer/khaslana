//! 本视图独立的语义消息资源；用户数据仅通过具名参数传入。
use khaslana::ui_i18n::UiMessage;

pub const NO_USABLE_INTELLIJ_IDEA_INSTALLATION_FOUND_AFTER_CONFIGURATION_RESOLVING_WILL: UiMessage = UiMessage::new("views.external_merge_view.no_usable_intellij_idea_installation_found_after_configuration_resolving_will", "尚未找到可用的 IntelliJ IDEA。配置完成后将继续解决：{path}", "No usable IntelliJ IDEA installation found. After configuration, resolving will continue for: {path}");
pub const CANNOT_START_THE_EXTERNAL_MERGE_TOOL_ERR_SELECT_THE_IDEA: UiMessage = UiMessage::new("views.external_merge_view.cannot_start_the_external_merge_tool_err_select_the_idea", "无法启动外部合并工具：{err}。请选择 IDEA 程序路径，保存后会自动继续当前冲突。", "Cannot start the external merge tool: {err}. Select the IDEA executable; saving will continue the current conflict operation.");
pub const NO_USABLE_INTELLIJ_IDEA_INSTALLATION_FOUND_ERR_ENTER_OR_SELECT: UiMessage = UiMessage::new("views.external_merge_view.no_usable_intellij_idea_installation_found_err_enter_or_select", "未找到可用的 IntelliJ IDEA：{err}。请填写或选择正确的程序路径。", "No usable IntelliJ IDEA installation found: {err}. Enter or select the correct executable path.");
pub const MERGE_TOOL_SETTINGS_SAVED_PATH: UiMessage = UiMessage::new("views.external_merge_view.merge_tool_settings_saved_path", "合并工具设置已保存：{path}", "Merge tool settings saved: {path}");
pub const INTELLIJ_IDEA_FOUND_PATH: UiMessage = UiMessage::new("views.external_merge_view.intellij_idea_found_path", "已找到 IntelliJ IDEA：{path}", "IntelliJ IDEA found: {path}");
pub const CONFIGURE_INTELLIJ_IDEA_BEFORE_ENABLING_ERR_ENTER_A_PATH_OR: UiMessage = UiMessage::new("views.external_merge_view.configure_intellij_idea_before_enabling_err_enter_a_path_or", "启用前需要配置 IntelliJ IDEA：{err}。请填写路径或点击“选择 IDEA 程序”。", "Configure IntelliJ IDEA before enabling: {err}. Enter a path or click “Select IDEA executable”.");
pub const CONFIGURE_INTELLIJ_IDEA_BEFORE_ENABLING_AUTOMATIC_OPENING_ERR_ENTER_A: UiMessage = UiMessage::new("views.external_merge_view.configure_intellij_idea_before_enabling_automatic_opening_err_enter_a", "开启自动打开前需要配置 IntelliJ IDEA：{err}。请填写路径或点击“选择 IDEA 程序”。", "Configure IntelliJ IDEA before enabling automatic opening: {err}. Enter a path or click “Select IDEA executable”.");
pub const IDEA_AVAILABLE: UiMessage = UiMessage::new("views.external_merge_view.idea_available", "✓ IDEA 可用", "✓ IDEA available");
pub const DETECTION_FAILED_RETRY: UiMessage = UiMessage::new("views.external_merge_view.detection_failed_retry", "检测失败 · 重试", "Detection failed · Retry");
pub const DETECT_IDEA: UiMessage = UiMessage::new("views.external_merge_view.detect_idea", "检测 IDEA", "Detect IDEA");
pub const SAVE_AND_CONTINUE: UiMessage = UiMessage::new("views.external_merge_view.save_and_continue", "保存并继续", "Save and continue");
pub const SAVE: UiMessage = UiMessage::new("views.external_merge_view.save", "保存", "Save");
pub const EXTERNAL_MERGE: UiMessage = UiMessage::new("views.external_merge_view.external_merge", "外部合并", "External merge");
pub const ENABLE_INTELLIJ_IDEA_EXTERNAL_MERGE: UiMessage = UiMessage::new("views.external_merge_view.enable_intellij_idea_external_merge", "启用 IntelliJ IDEA 外部合并", "Enable IntelliJ IDEA external merge");
pub const AUTOMATICALLY_OPEN_IDEA_WHEN_SELECTING_A_CONFLICT_FILE: UiMessage = UiMessage::new("views.external_merge_view.automatically_open_idea_when_selecting_a_conflict_file", "选中冲突文件时自动打开 IDEA", "Automatically open IDEA when selecting a conflict file");
pub const IDEA_EXECUTABLE: UiMessage = UiMessage::new("views.external_merge_view.idea_executable", "IDEA 程序", "IDEA executable");
pub const IDEA_PATH: UiMessage = UiMessage::new("views.external_merge_view.idea_path", "IDEA 路径", "IDEA path");
pub const SELECT_IDEA_EXECUTABLE: UiMessage = UiMessage::new("views.external_merge_view.select_idea_executable", "选择 IDEA 程序", "Select IDEA executable");
pub const THE_PATH_IS_OPTIONAL_WHEN_EMPTY_DETECTION_CHECKS_KHASLANA_IDEA: UiMessage = UiMessage::new("views.external_merge_view.the_path_is_optional_when_empty_detection_checks_khaslana_idea", "路径可留空。留空时会依次检测 KHASLANA_IDEA_PATH、PATH 中的 idea64 / idea，以及常见 JetBrains 安装目录。开启或保存时会立即验证；未找到工具时不会静默结束，而是保留当前操作并提示配置。", "The path is optional. When empty, detection checks KHASLANA_IDEA_PATH, idea64 / idea on PATH, and common JetBrains installation folders. Enabling or saving verifies the executable immediately. If it is not found, the pending operation is retained and configuration is requested.");
pub const MERGE_TOOL_SETTINGS_SAVED: UiMessage = UiMessage::new("views.external_merge_view.merge_tool_settings_saved", "合并工具设置已保存", "Merge tool settings saved");
pub const EXTERNAL_MERGE_TOOL_IS_NOT_ENABLED: UiMessage = UiMessage::new("views.external_merge_view.external_merge_tool_is_not_enabled", "外部合并工具未启用", "External merge tool is not enabled");
pub const CONFIGURE_THE_INTELLIJ_IDEA_MERGE_TOOL: UiMessage = UiMessage::new("views.external_merge_view.configure_the_intellij_idea_merge_tool", "需要配置 IntelliJ IDEA 合并工具", "Configure the IntelliJ IDEA merge tool");
pub const ENABLE_THE_EXTERNAL_MERGE_TOOL_BEFORE_SAVING_AND_CONTINUING_THIS: UiMessage = UiMessage::new("views.external_merge_view.enable_the_external_merge_tool_before_saving_and_continuing_this", "请先启用外部合并工具，再保存并继续当前冲突", "Enable the external merge tool before saving and continuing this conflict");
pub const SELECTING_INTELLIJ_IDEA_EXECUTABLE: UiMessage = UiMessage::new("views.external_merge_view.selecting_intellij_idea_executable", "正在选择 IntelliJ IDEA 程序...", "Selecting IntelliJ IDEA executable…");
pub const SELECT_INTELLIJ_IDEA_LAUNCHER: UiMessage = UiMessage::new("views.external_merge_view.select_intellij_idea_launcher", "选择 IntelliJ IDEA 启动程序", "Select IntelliJ IDEA launcher");

#[cfg(test)]
const ALL_MESSAGES: &[UiMessage] = &[
    NO_USABLE_INTELLIJ_IDEA_INSTALLATION_FOUND_AFTER_CONFIGURATION_RESOLVING_WILL,
    CANNOT_START_THE_EXTERNAL_MERGE_TOOL_ERR_SELECT_THE_IDEA,
    NO_USABLE_INTELLIJ_IDEA_INSTALLATION_FOUND_ERR_ENTER_OR_SELECT,
    MERGE_TOOL_SETTINGS_SAVED_PATH,
    INTELLIJ_IDEA_FOUND_PATH,
    CONFIGURE_INTELLIJ_IDEA_BEFORE_ENABLING_ERR_ENTER_A_PATH_OR,
    CONFIGURE_INTELLIJ_IDEA_BEFORE_ENABLING_AUTOMATIC_OPENING_ERR_ENTER_A,
    IDEA_AVAILABLE,
    DETECTION_FAILED_RETRY,
    DETECT_IDEA,
    SAVE_AND_CONTINUE,
    SAVE,
    EXTERNAL_MERGE,
    ENABLE_INTELLIJ_IDEA_EXTERNAL_MERGE,
    AUTOMATICALLY_OPEN_IDEA_WHEN_SELECTING_A_CONFLICT_FILE,
    IDEA_EXECUTABLE,
    IDEA_PATH,
    SELECT_IDEA_EXECUTABLE,
    THE_PATH_IS_OPTIONAL_WHEN_EMPTY_DETECTION_CHECKS_KHASLANA_IDEA,
    MERGE_TOOL_SETTINGS_SAVED,
    EXTERNAL_MERGE_TOOL_IS_NOT_ENABLED,
    CONFIGURE_THE_INTELLIJ_IDEA_MERGE_TOOL,
    ENABLE_THE_EXTERNAL_MERGE_TOOL_BEFORE_SAVING_AND_CONTINUING_THIS,
    SELECTING_INTELLIJ_IDEA_EXECUTABLE,
    SELECT_INTELLIJ_IDEA_LAUNCHER,
];

#[cfg(test)]
#[path = "../tests/view_resources.rs"]
mod tests;
