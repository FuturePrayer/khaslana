//! 设置中心的 Skill、MCP 与浏览器运行环境配置。

#[path = "ui_resources/ai_extensions_view_messages.rs"]
mod messages;

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use gpui::{Context, Div, FontWeight, SharedString, Window, div, prelude::*, px};
use gpui_kit::component::button::Button;
use gpui_kit::component::collapsible::Collapsible;
use gpui_kit::component::{Disableable, IconName, Sizable, h_flex, v_flex};
use khaslana::workflow::browser_runtime::{self, BrowserRuntimeInfo};
use khaslana::workflow::extensions::{
    self, WorkflowMcpConfig, WorkflowMcpServer, WorkflowMcpTool, WorkflowSkillPreview,
    WorkflowToolAccess,
};

use crate::{DialogState, FieldId, RepositoryView, ScrollbarMode, SettingsCategory,
    TextFieldState, UiEvent, dialog_actions, dialog_panel_size, scrollable_frame_when,
    send_ui_event, system::open_directory, tasks::{TaskKind, panic_message},
    ui::theme::{self as ui_theme, rgb}};
use crate::ui::components::{SettingsButtonTone, settings_command_button};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum AiSettingsTab {
    #[default]
    Connection,
    Skill,
    Mcp,
    Runtime,
}

impl AiSettingsTab {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Connection => 0,
            Self::Skill => 1,
            Self::Mcp => 2,
            Self::Runtime => 3,
        }
    }

    pub(crate) fn from_index(index: usize) -> Self {
        match index {
            1 => Self::Skill,
            2 => Self::Mcp,
            3 => Self::Runtime,
            _ => Self::Connection,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct McpToolDraft {
    pub(crate) name: String,
    pub(crate) allowed: bool,
    pub(crate) access: WorkflowToolAccess,
}

fn discovered_mcp_tools(names: Vec<String>, previous: &[McpToolDraft],
    new_server: bool) -> Vec<McpToolDraft> {
    let previous: BTreeMap<_, _> = previous.iter()
        .map(|tool| (tool.name.as_str(), (tool.allowed, tool.access))).collect();
    names.into_iter().map(|name| {
        // 新服务开箱即用；旧配置的白名单与本次手动关闭的工具不能被重测覆盖。
        let (allowed, access) = previous.get(name.as_str()).copied()
            .unwrap_or((new_server, WorkflowToolAccess::Write));
        McpToolDraft { name, allowed, access }
    }).collect()
}

pub(crate) struct AiExtensionsUiState {
    pub(crate) connection_test: crate::ai_settings_page::state::AiConnectionTestState,
    pub(crate) save_error: Option<String>,
    pub(crate) tab: AiSettingsTab,
    pub(crate) skills: Vec<(String, String)>,
    pub(crate) mcp: WorkflowMcpConfig,
    pub(crate) runtime: Option<BrowserRuntimeInfo>,
    pub(crate) loading: bool,
    pub(crate) load_request_id: u64,
    pub(crate) error: Option<String>,
    pub(crate) skill_preview: Option<WorkflowSkillPreview>,
    pub(crate) skill_preview_request_id: u64,
    pub(crate) skill_picker_open: bool,
    pub(crate) edit_mcp_name: Option<String>,
    pub(crate) mcp_id: TextFieldState,
    pub(crate) mcp_command: TextFieldState,
    pub(crate) mcp_args: TextFieldState,
    pub(crate) mcp_tools: Vec<McpToolDraft>,
    pub(crate) mcp_tools_expanded: bool,
    pub(crate) mcp_auto_discover: bool,
    pub(crate) mcp_enabled: bool,
    pub(crate) mcp_verified: Option<(String, Vec<String>)>,
    pub(crate) mcp_testing: bool,
    pub(crate) mcp_test_request_id: u64,
    pub(crate) action_busy: bool,
    pub(crate) action_request_id: u64,
}

impl AiExtensionsUiState {
    pub(crate) fn new(cx: &mut Context<RepositoryView>) -> Self {
        Self {
            connection_test: Default::default(),
            save_error: None,
            tab: AiSettingsTab::Connection,
            skills: Vec::new(),
            mcp: WorkflowMcpConfig::default(),
            runtime: None,
            loading: false,
            load_request_id: 0,
            error: None,
            skill_preview: None,
            skill_preview_request_id: 0,
            skill_picker_open: false,
            edit_mcp_name: None,
            mcp_id: TextFieldState::localized(cx, messages::EXAMPLE_MY_LOCAL_SERVICE),
            mcp_command: TextFieldState::localized(cx, messages::EXAMPLE_NPX_UVX_OR_THE_FULL_SERVICE_EXECUTABLE_PATH),
            mcp_args: TextFieldState::localized(cx, messages::ONE_STARTUP_ARGUMENT_PER_LINE),
            mcp_tools: Vec::new(),
            mcp_tools_expanded: false,
            mcp_auto_discover: true,
            mcp_enabled: true,
            mcp_verified: None,
            mcp_testing: false,
            mcp_test_request_id: 0,
            action_busy: false,
            action_request_id: 0,
        }
    }
}

fn guarded<T>(stage: &'static str, action: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    catch_unwind(AssertUnwindSafe(action))
        .unwrap_or_else(|payload| Err(khaslana::ui_i18n::format(messages::STAGE_FAILED_UNEXPECTEDLY_PAYLOAD, &[("stage", &(stage).to_string()), ("payload", &(panic_message(payload)).to_string())])))
}

fn primary_text(text: impl Into<SharedString>) -> Div {
    div().text_size(px(ui_theme::TYPE_BODY))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(text.into())
}

fn meta_text(text: impl Into<SharedString>) -> Div {
    div().text_size(px(ui_theme::TYPE_META))
        .text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(text.into())
}

fn error_text(text: impl Into<SharedString>) -> Div {
    let text: SharedString = text.into();
    let text = khaslana::ui_i18n::resolve(text.as_ref());
    div().text_size(px(ui_theme::TYPE_META))
        .text_color(rgb(ui_theme::DESTRUCTIVE)).child(text)
}

fn settings_row(title: impl Into<SharedString>, description: impl Into<SharedString>) -> Div {
    h_flex().w_full().min_w(px(0.0)).items_center().justify_between().gap_3()
        .px_3().py_3().rounded(px(ui_theme::RADIUS_XS))
        .border_1().border_color(rgb(ui_theme::BORDER_MUTED))
        .bg(rgb(ui_theme::SURFACE_BASE))
        .child(v_flex().flex_1().min_w(px(0.0)).gap_1()
            .child(primary_text(title).truncate())
            .child(meta_text(description).truncate()))
}

impl RepositoryView {
    pub(crate) fn refresh_ai_extensions(&mut self) {
        self.load_ai_extensions(false);
    }

    pub(crate) fn inspect_ai_runtime(&mut self) {
        self.load_ai_extensions(true);
    }

    fn load_ai_extensions(&mut self, inspect_runtime: bool) {
        let Some(data_dir) = khaslana::storage::active_data_dir() else {
            self.ai_extensions.error = Some(khaslana::ui_i18n::encode(messages::CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY, &[]));
            return;
        };
        self.ai_extensions.load_request_id = self.ai_extensions.load_request_id.wrapping_add(1);
        let request_id = self.ai_extensions.load_request_id;
        self.ai_extensions.loading = true;
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::External, move || {
            let result = guarded(khaslana::ui_i18n::text(messages::READ_AI_EXTENSION_SETTINGS), || {
                let skills = extensions::discover_skill_packages(&data_dir)
                    .map_err(|err| err.to_string())?;
                let mcp = extensions::load_user_mcp_config(&data_dir)
                    .map_err(|err| err.to_string())?;
                let runtime = if inspect_runtime { browser_runtime::inspect(&data_dir) }
                    else { browser_runtime::inspect_cached(&data_dir) };
                Ok((skills, mcp, runtime))
            });
            send_ui_event(&tx, UiEvent::AiExtensionsLoaded { request_id, result });
        });
    }

    pub(crate) fn apply_ai_extensions_loaded(&mut self, request_id: u64,
        result: Result<(Vec<(String, String)>, WorkflowMcpConfig, BrowserRuntimeInfo), String>) {
        if request_id != self.ai_extensions.load_request_id { return; }
        self.ai_extensions.loading = false;
        match result {
            Ok((skills, mcp, runtime)) => {
                self.ai_extensions.skills = skills;
                self.ai_extensions.mcp = mcp;
                self.ai_extensions.runtime = Some(runtime);
                self.ai_extensions.error = None;
            }
            Err(error) => self.ai_extensions.error = Some(error),
        }
    }

    pub(crate) fn select_ai_settings_tab(&mut self, tab: AiSettingsTab) {
        if tab != AiSettingsTab::Connection && !self.ai_settings.enabled { return; }
        self.ai_extensions.tab = tab;
        if tab == AiSettingsTab::Runtime { self.refresh_ai_extensions(); }
    }

    fn can_edit_ai_extensions(&self) -> bool {
        self.settings_center == Some(SettingsCategory::Ai) && self.ai_settings.enabled
    }

    pub(crate) fn open_ai_skill_folder(&mut self, cx: &mut Context<Self>) {
        if !self.can_edit_ai_extensions() { return; }
        let Some(data_dir) = khaslana::storage::active_data_dir() else {
            self.notify_error(khaslana::ui_i18n::text(messages::CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY), cx);
            return;
        };
        let folder = data_dir.join("workflow-skills");
        if let Err(error) = std::fs::create_dir_all(&folder).and_then(|()| open_directory(&folder)) {
            self.notify_error(khaslana::ui_i18n::format(messages::FAILED_TO_OPEN_THE_SKILL_FOLDER_ERROR, &[("error", &(error).to_string())]), cx);
        }
    }

    pub(crate) fn open_ai_skill_import(&mut self) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy
            || self.ai_extensions.skill_picker_open { return; }
        self.ai_extensions.skill_preview = None;
        self.ai_extensions.skill_preview_request_id =
            self.ai_extensions.skill_preview_request_id.wrapping_add(1);
        self.ai_extensions.error = None;
        self.ai_extensions.action_request_id = self.ai_extensions.action_request_id.wrapping_add(1);
        self.active_dialog = Some(DialogState::AiSkillImport);
    }

    pub(crate) fn pick_ai_skill_folder(&mut self) {
        if self.active_dialog != Some(DialogState::AiSkillImport)
            || self.ai_extensions.action_busy || self.ai_extensions.skill_picker_open { return; }
        self.ai_extensions.skill_preview_request_id =
            self.ai_extensions.skill_preview_request_id.wrapping_add(1);
        let request_id = self.ai_extensions.skill_preview_request_id;
        self.ai_extensions.skill_picker_open = true;
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::External, move || {
            let result = guarded(khaslana::ui_i18n::text(messages::PREVIEW_SKILL), || {
                let folder = rfd::FileDialog::new().set_title(khaslana::ui_i18n::text(messages::SELECT_A_FOLDER_CONTAINING_SKILL_MD))
                    .pick_folder();
                folder.map(|folder| extensions::preview_skill_folder(&folder)
                    .map_err(|err| err.to_string())).transpose()
            });
            send_ui_event(&tx, UiEvent::AiSkillPreviewed { request_id, result });
        });
    }

    pub(crate) fn apply_ai_skill_preview(&mut self, request_id: u64,
        result: Result<Option<WorkflowSkillPreview>, String>) {
        if request_id != self.ai_extensions.skill_preview_request_id { return; }
        self.ai_extensions.skill_picker_open = false;
        if self.active_dialog != Some(DialogState::AiSkillImport) { return; }
        match result {
            Ok(Some(preview)) => {
                self.ai_extensions.skill_preview = Some(preview);
                self.ai_extensions.error = None;
            }
            Ok(None) => {}
            Err(error) => self.ai_extensions.error = Some(error),
        }
    }

    pub(crate) fn install_ai_skill(&mut self) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy
            || self.ai_extensions.skill_picker_open
            || self.active_dialog != Some(DialogState::AiSkillImport) { return; }
        let Some(preview) = self.ai_extensions.skill_preview.as_ref() else { return; };
        let Some(data_dir) = khaslana::storage::active_data_dir() else {
            self.ai_extensions.error = Some(khaslana::ui_i18n::encode(messages::CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY, &[]));
            return;
        };
        let source = preview.source.clone();
        let expected_sha256 = preview.content_sha256.clone();
        self.start_ai_extension_change(khaslana::ui_i18n::text(messages::INSTALL_SKILL), move || {
            extensions::install_skill_folder(&data_dir, &source, &expected_sha256)
                .map(|preview| khaslana::ui_i18n::format(messages::SKILL_NAME_INSTALLED, &[("name", &(preview.name).to_string())]))
                .map_err(|err| err.to_string())
        });
    }

    pub(crate) fn open_ai_skill_details(&mut self, name: String) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy { return; }
        self.active_dialog = Some(DialogState::AiSkillDetails { name });
    }

    pub(crate) fn remove_ai_skill(&mut self, name: String) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy { return; }
        let Some(data_dir) = khaslana::storage::active_data_dir() else {
            self.ai_extensions.error = Some(khaslana::ui_i18n::encode(messages::CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY, &[]));
            return;
        };
        self.start_ai_extension_change(khaslana::ui_i18n::text(messages::REMOVE_SKILL), move || {
            extensions::remove_skill_package(&data_dir, &name)
                .map(|()| khaslana::ui_i18n::format(messages::SKILL_NAME_REMOVED, &[("name", &(name).to_string())]))
                .map_err(|err| err.to_string())
        });
    }

    pub(crate) fn open_ai_mcp_form(&mut self, name: Option<String>) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy { return; }
        let existing = name.as_ref().and_then(|name| self.ai_extensions.mcp.servers.get(name));
        self.ai_extensions.mcp_id.set_value(name.clone().unwrap_or_default());
        self.ai_extensions.mcp_command.set_value(existing.map(|server| server.command.clone()).unwrap_or_default());
        self.ai_extensions.mcp_args.set_value(existing.map(|server| server.args.join("\n")).unwrap_or_default());
        self.ai_extensions.mcp_tools = existing.map(|server| server.tools.iter()
            .map(|(name, tool)| McpToolDraft { name: name.clone(), allowed: true,
                access: tool.access }).collect()).unwrap_or_default();
        self.ai_extensions.mcp_auto_discover = existing.is_none_or(|server| server.auto_discover);
        self.ai_extensions.mcp_enabled = existing.is_none_or(|server| server.enabled);
        if let Some(server) = existing {
            let names = server.cached_tools.iter().cloned()
                .chain(server.tools.keys().cloned()).collect::<std::collections::BTreeSet<_>>();
            self.ai_extensions.mcp_tools = discovered_mcp_tools(names.into_iter().collect(),
                &self.ai_extensions.mcp_tools, server.auto_discover);
        }
        self.ai_extensions.edit_mcp_name = name;
        self.ai_extensions.mcp_tools_expanded = false;
        self.ai_extensions.mcp_verified = None;
        self.ai_extensions.error = None;
        self.ai_extensions.mcp_testing = false;
        self.ai_extensions.mcp_test_request_id = self.ai_extensions.mcp_test_request_id.wrapping_add(1);
        self.ai_extensions.action_request_id = self.ai_extensions.action_request_id.wrapping_add(1);
        self.active_dialog = Some(DialogState::AiMcpForm);
    }

    fn ai_mcp_args(&self) -> Vec<String> {
        self.ai_extensions.mcp_args.value.lines().map(str::trim)
            .filter(|value| !value.is_empty()).map(str::to_owned).collect()
    }

    pub(crate) fn test_ai_mcp_form(&mut self) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.mcp_testing
            || self.active_dialog != Some(DialogState::AiMcpForm) { return; }
        let command = self.ai_extensions.mcp_command.value.trim().to_owned();
        let args = self.ai_mcp_args();
        self.ai_extensions.mcp_test_request_id = self.ai_extensions.mcp_test_request_id.wrapping_add(1);
        let request_id = self.ai_extensions.mcp_test_request_id;
        self.ai_extensions.mcp_testing = true;
        self.ai_extensions.mcp_verified = None;
        self.ai_extensions.error = None;
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::External, move || {
            let result = guarded(khaslana::ui_i18n::text(messages::TEST_MCP_SERVICE), ||
                extensions::inspect_mcp_server_tools(&command, &args).map_err(|err| err.to_string()));
            send_ui_event(&tx, UiEvent::AiMcpToolsInspected { request_id, command, args, result });
        });
    }

    pub(crate) fn apply_ai_mcp_tools(&mut self, request_id: u64, command: String,
        args: Vec<String>, result: Result<Vec<String>, String>) {
        if request_id != self.ai_extensions.mcp_test_request_id
            || self.active_dialog != Some(DialogState::AiMcpForm) { return; }
        self.ai_extensions.mcp_testing = false;
        if command != self.ai_extensions.mcp_command.value.trim() || args != self.ai_mcp_args() {
            self.ai_extensions.error = Some(khaslana::ui_i18n::encode(messages::THE_STARTUP_COMMAND_OR_ARGUMENTS_CHANGED_TEST_THE_CONNECTION_AGAIN, &[]));
            return;
        }
        match result {
            Ok(names) => {
                self.ai_extensions.mcp_tools = discovered_mcp_tools(names,
                    &self.ai_extensions.mcp_tools, self.ai_extensions.mcp_auto_discover);
                self.ai_extensions.mcp_verified = Some((command, args));
                self.ai_extensions.error = None;
            }
            Err(error) => self.ai_extensions.error = Some(error),
        }
    }

    pub(crate) fn save_ai_mcp_form(&mut self) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy
            || self.ai_extensions.mcp_testing || self.active_dialog != Some(DialogState::AiMcpForm) { return; }
        let Some(data_dir) = khaslana::storage::active_data_dir() else {
            self.ai_extensions.error = Some(khaslana::ui_i18n::encode(messages::CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY, &[]));
            return;
        };
        let name = self.ai_extensions.mcp_id.value.trim().to_owned();
        let command = self.ai_extensions.mcp_command.value.trim().to_owned();
        let args = self.ai_mcp_args();
        let verified = self.ai_extensions.mcp_verified.as_ref() == Some(&(command.clone(), args.clone()));
        let cached_tools = if verified {
            self.ai_extensions.mcp_tools.iter().map(|tool| tool.name.clone()).collect()
        } else {
            self.ai_extensions.edit_mcp_name.as_ref()
                .and_then(|name| self.ai_extensions.mcp.servers.get(name))
                .filter(|server| server.command == command && server.args == args)
                .map(|server| server.cached_tools.clone()).unwrap_or_default()
        };
        let server = WorkflowMcpServer {
            command,
            args,
            tools: self.ai_extensions.mcp_tools.iter().filter(|tool| tool.allowed)
                .map(|tool| (tool.name.clone(), WorkflowMcpTool { access: tool.access }))
                .collect(),
            enabled: self.ai_extensions.mcp_enabled,
            auto_discover: self.ai_extensions.mcp_auto_discover,
            cached_tools,
        };
        let previous_name = self.ai_extensions.edit_mcp_name.clone();
        self.start_ai_extension_change(khaslana::ui_i18n::text(messages::SAVE_MCP_SERVICE), move || {
            extensions::upsert_user_mcp_server(&data_dir, previous_name.as_deref(), &name, server)
                .map(|()| khaslana::ui_i18n::format(messages::MCP_SERVICE_NAME_SAVED, &[("name", &(name).to_string())]))
                .map_err(|err| err.to_string())
        });
    }

    pub(crate) fn remove_ai_mcp_server(&mut self, name: String) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy { return; }
        let Some(data_dir) = khaslana::storage::active_data_dir() else {
            self.ai_extensions.error = Some(khaslana::ui_i18n::encode(messages::CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY, &[]));
            return;
        };
        self.start_ai_extension_change(khaslana::ui_i18n::text(messages::REMOVE_MCP_SERVICE), move || {
            extensions::remove_user_mcp_server(&data_dir, &name)
                .map(|()| khaslana::ui_i18n::format(messages::MCP_SERVICE_NAME_REMOVED, &[("name", &(name).to_string())]))
                .map_err(|err| err.to_string())
        });
    }

    pub(crate) fn set_ai_mcp_server_enabled(&mut self, name: String, enabled: bool) {
        if !self.can_edit_ai_extensions() || self.ai_extensions.action_busy { return; }
        let Some(data_dir) = khaslana::storage::active_data_dir() else { return; };
        self.start_ai_extension_change(khaslana::ui_i18n::text(messages::UPDATE_MCP_SERVICE_STATUS), move || {
            extensions::set_user_mcp_server_enabled(&data_dir, &name, enabled)
                .map(|()| khaslana::ui_i18n::format(if enabled { messages::MCP_ENABLED } else { messages::MCP_DISABLED }, &[("name", &name)]))
                .map_err(|err| err.to_string())
        });
    }

    fn start_ai_extension_change(&mut self, stage: &'static str,
        action: impl FnOnce() -> Result<String, String> + Send + 'static) {
        self.ai_extensions.action_request_id = self.ai_extensions.action_request_id.wrapping_add(1);
        let request_id = self.ai_extensions.action_request_id;
        self.ai_extensions.action_busy = true;
        self.ai_extensions.error = None;
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::External, move || {
            let result = guarded(stage, action);
            send_ui_event(&tx, UiEvent::AiExtensionChanged { request_id, result });
        });
    }

    pub(crate) fn apply_ai_extension_change(&mut self, request_id: u64,
        result: Result<String, String>, cx: &mut Context<Self>) {
        if request_id != self.ai_extensions.action_request_id { return; }
        self.ai_extensions.action_busy = false;
        match result {
            Ok(message) => {
                if matches!(self.active_dialog, Some(DialogState::AiSkillImport
                    | DialogState::AiSkillDetails { .. } | DialogState::AiMcpForm
                    | DialogState::AiMcpRemove { .. } | DialogState::AiSkillRemove { .. })) {
                    self.close_dialog();
                }
                self.notify_success(message, cx);
                self.refresh_ai_extensions();
            }
            Err(error) => {
                self.ai_extensions.error = Some(error.clone());
                self.notify_error(error, cx);
            }
        }
        cx.notify();
    }

    pub(crate) fn start_ai_runtime_download(&mut self) {
        if !self.can_edit_ai_extensions() { return; }
        self.start_browser_runtime_download_for_settings();
    }
}

#[cfg(test)]
#[path = "tests/ai_extensions_view.rs"]
mod tests;

impl RepositoryView {
    pub(crate) fn render_ai_skill_import_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let preview = self.ai_extensions.skill_preview.as_ref();
        let busy = self.ai_extensions.action_busy;
        let picking = self.ai_extensions.skill_picker_open;
        self.dialog_panel(khaslana::ui_i18n::text(messages::INSTALL_LOCAL_SKILL), cx)
            .w(px(570.0))
            .child(meta_text(khaslana::ui_i18n::text(messages::READ_SKILL_MD_FROM_A_FOLDER_REVIEW_ITS_CONTENTS_THEN)))
            .child(h_flex().w_full().justify_between().gap_2().items_center()
                .child(meta_text(preview.map(|item| item.source.display().to_string())
                    .unwrap_or_else(|| khaslana::ui_i18n::text(messages::NO_FOLDER_SELECTED).into())).flex_1().min_w(px(0.0)).truncate())
                .child(self.button(if picking { khaslana::ui_i18n::text(messages::SELECTING) } else { khaslana::ui_i18n::text(messages::SELECT_FOLDER) }, !busy && !picking,
                    |this, _, _| this.pick_ai_skill_folder(), cx)))
            .when_some(preview, |this, preview| {
                this.child(v_flex().w_full().gap_2().p_3()
                    .rounded(px(ui_theme::RADIUS_XS)).border_1()
                    .border_color(rgb(ui_theme::BORDER_MUTED))
                    .child(primary_text(&preview.name))
                    .child(meta_text(&preview.description))
                    .child(meta_text(khaslana::ui_i18n::format(messages::FILES_FILES, &[("files", &(preview.files.join(" · ")).to_string())]))))
            })
            .when_some(self.ai_extensions.error.as_ref(), |this, error| {
                this.child(error_text(error.clone()))
            })
            .child(meta_text(khaslana::ui_i18n::text(messages::IMPORTED_SKILLS_CAN_BE_VIEWED_AND_REMOVED_RUNNING_A_SKILL)))
            .child(dialog_actions()
                .child(self.button(khaslana::ui_i18n::text(messages::CANCEL), !busy, |this, _, _| this.close_dialog(), cx))
                .child(self.primary_button(khaslana::ui_i18n::text(messages::CONFIRM_INSTALLATION), preview.is_some() && !busy && !picking,
                    |this, _, _| this.install_ai_skill(), cx)))
    }

    pub(crate) fn render_ai_skill_details_dialog(&self, name: &str,
        cx: &mut Context<Self>) -> impl IntoElement {
        let description = self.ai_extensions.skills.iter()
            .find(|(candidate, _)| candidate == name)
            .map(|(_, description)| description.clone()).unwrap_or_default();
        let folder = khaslana::storage::active_data_dir()
            .map(|path| path.join("workflow-skills").join(name).display().to_string())
            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::CANNOT_LOCATE_THE_APPLICATION_DATA_DIRECTORY).into());
        let name_for_remove = name.to_owned();
        self.dialog_panel(khaslana::ui_i18n::text(messages::SKILL_DETAILS), cx).w(px(560.0))
            .child(primary_text(name.to_owned()))
            .child(meta_text(description))
            .child(meta_text(folder))
            .child(meta_text(khaslana::ui_i18n::text(messages::REQUIRED_MCP_TOOLS_ARE_SHOWN_FOR_APPROVAL_BEFORE_THE_WORKFLOW)))
            .child(dialog_actions()
                .child(self.button(khaslana::ui_i18n::text(messages::CLOSE), true, |this, _, _| this.close_dialog(), cx))
                .child(self.danger_button(khaslana::ui_i18n::text(messages::REMOVE_SKILL), !self.ai_extensions.action_busy,
                    move |this, _, cx| {
                        this.active_dialog = Some(DialogState::AiSkillRemove {
                            name: name_for_remove.clone() });
                        cx.notify();
                    }, cx)))
    }

    pub(crate) fn render_ai_skill_remove_dialog(&self, name: &str,
        cx: &mut Context<Self>) -> impl IntoElement {
        let name_for_remove = name.to_owned();
        self.dialog_panel(khaslana::ui_i18n::text(messages::REMOVE_SKILL), cx)
            .child(primary_text(khaslana::ui_i18n::format(messages::REMOVE_NAME, &[("name", &(name).to_string())])))
            .child(meta_text(khaslana::ui_i18n::text(messages::THIS_DELETES_THE_SKILL_FOLDER_FROM_THE_APPLICATION_DATA_DIRECTORY)))
            .when_some(self.ai_extensions.error.as_ref(), |this, error| {
                this.child(error_text(error.clone()))
            })
            .child(dialog_actions()
                .child(self.button(khaslana::ui_i18n::text(messages::CANCEL), !self.ai_extensions.action_busy,
                    |this, _, _| this.close_dialog(), cx))
                .child(self.danger_button(khaslana::ui_i18n::text(messages::CONFIRM_REMOVAL), !self.ai_extensions.action_busy,
                    move |this, _, _| this.remove_ai_skill(name_for_remove.clone()), cx)))
    }

    pub(crate) fn render_ai_mcp_form_dialog(&self, window: &Window,
        cx: &mut Context<Self>) -> impl IntoElement {
        let editing = self.ai_extensions.edit_mcp_name.is_some();
        let busy = self.ai_extensions.action_busy;
        let testing = self.ai_extensions.mcp_testing;
        let verified = self.ai_extensions.mcp_verified.as_ref().is_some_and(|(command, args)|
            command == self.ai_extensions.mcp_command.value.trim()
                && *args == self.ai_mcp_args());
        let selected_tools = self.ai_extensions.mcp_tools.iter().any(|tool| tool.allowed);
        let enabled_count = self.ai_extensions.mcp_tools.iter().filter(|tool| tool.allowed).count();
        let (panel_width, panel_height) = dialog_panel_size(window, 590.0, 620.0);
        let handle = self.scroll_handle("ai-mcp-form-scroll");
        let mut body = v_flex().w_full().gap_2()
            .child(meta_text(khaslana::ui_i18n::text(messages::ENTER_A_COMMAND_AND_ARGUMENTS_TO_SAVE_THE_SERVICE_CONNECTS)))
            .child(primary_text(khaslana::ui_i18n::text(messages::SERVICE_ID)))
            .child(self.input(FieldId::AiMcpServerId, false, window, cx))
            .child(primary_text(khaslana::ui_i18n::text(messages::STARTUP_COMMAND)))
            .child(self.input(FieldId::AiMcpCommand, false, window, cx))
            .child(meta_text(khaslana::ui_i18n::text(messages::YOU_CAN_ENTER_NPX_DIRECTLY_WITHOUT_CMD_PUT_EACH_ARGUMENT)))
            .child(primary_text(khaslana::ui_i18n::text(messages::STARTUP_ARGUMENTS_ONE_PER_LINE)))
            .child(self.input(FieldId::AiMcpArgs, false, window, cx))
            .child(h_flex().w_full().justify_between().items_center()
                .child(meta_text(if verified { khaslana::ui_i18n::text(messages::CONNECTION_TEST_SUCCEEDED) } else { khaslana::ui_i18n::text(messages::THIS_CONFIGURATION_HAS_NOT_BEEN_VERIFIED_YOU_CAN_STILL_SAVE) }))
                .child(self.button(khaslana::ui_i18n::text(messages::TEST_CONNECTION), !busy && !testing,
                    |this, _, _| this.test_ai_mcp_form(), cx)));
        if testing { body = body.child(meta_text(khaslana::ui_i18n::text(messages::CONNECTING_TO_THE_MCP_SERVICE_AND_READING_ITS_TOOL_LIST))); }
        let form_request_id = self.ai_extensions.mcp_test_request_id;
        body = body.child(h_flex().w_full().items_center().justify_between().gap_2()
            .child(meta_text(khaslana::ui_i18n::text(messages::AUTOMATICALLY_DISCOVER_TOOLS_APPROVAL_IS_STILL_REQUIRED_BEFORE_RUNNING)))
            .child(self.toggle_switch("ai-mcp-auto-discover", self.ai_extensions.mcp_auto_discover,
                busy || testing, move |this, checked, _, _| {
                    if this.active_dialog == Some(DialogState::AiMcpForm)
                        && this.ai_extensions.mcp_test_request_id == form_request_id {
                        this.ai_extensions.mcp_auto_discover = checked;
                    }
                }, cx)));
        let mut tools = v_flex().w_full().gap_2()
            .child(meta_text(khaslana::ui_i18n::text(messages::DISABLE_TOOLS_YOU_DO_NOT_NEED_READ_WRITE_TYPES_ARE)));
        for (index, tool) in self.ai_extensions.mcp_tools.iter().enumerate() {
            let access_view = cx.entity();
            let form_request_id = self.ai_extensions.mcp_test_request_id;
            tools = tools.child(h_flex().w_full().items_center().justify_between().gap_2()
                .py_2().border_b_1().border_color(rgb(ui_theme::BORDER_MUTED))
                .child(h_flex().flex_1().min_w(px(0.0)).items_center().gap_2()
                    .child(self.toggle_switch(format!("ai-mcp-tool-{index}"), tool.allowed,
                        busy || testing, move |this, checked, _, _| {
                            if this.active_dialog == Some(DialogState::AiMcpForm)
                                && this.ai_extensions.mcp_test_request_id == form_request_id
                                && let Some(tool) = this.ai_extensions.mcp_tools.get_mut(index) {
                                tool.allowed = checked;
                                this.ai_extensions.mcp_auto_discover = false;
                            }
                        }, cx))
                    .child(primary_text(tool.name.clone()).truncate()))
                .child(Button::new(format!("ai-mcp-access-{index}")).small()
                    .disabled(busy || testing)
                    .label(if tool.access == WorkflowToolAccess::Read { khaslana::ui_i18n::text(messages::READ) } else { khaslana::ui_i18n::text(messages::WRITE) })
                    .on_click(move |_, _, cx| {
                        access_view.update(cx, |this, cx| {
                            if this.active_dialog == Some(DialogState::AiMcpForm)
                                && this.ai_extensions.mcp_test_request_id == form_request_id
                                && let Some(tool) = this.ai_extensions.mcp_tools.get_mut(index) {
                                tool.access = if tool.access == WorkflowToolAccess::Read {
                                    WorkflowToolAccess::Write
                                } else { WorkflowToolAccess::Read };
                            }
                            cx.notify();
                        });
                    })));
        }
        if verified || !self.ai_extensions.mcp_tools.is_empty() {
            let expanded = self.ai_extensions.mcp_tools_expanded;
            let view = cx.entity();
            let form_request_id = self.ai_extensions.mcp_test_request_id;
            body = body.child(Collapsible::new().w_full().gap_2().open(expanded)
                .child(h_flex().w_full().items_center().justify_between().gap_2()
                    .child(meta_text(khaslana::ui_i18n::format(messages::MCP_TOOLS_TOOLS_RECORDED_ENABLED_COUNT_ALLOWED, &[("mcp_tools", &(self.ai_extensions.mcp_tools.len()).to_string()), ("enabled_count", &(enabled_count).to_string())])))
                    .child(settings_command_button("ai-mcp-tools-advanced",
                        if expanded { khaslana::ui_i18n::text(messages::COLLAPSE_ADVANCED_OPTIONS) } else { khaslana::ui_i18n::text(messages::ADVANCED_OPTIONS_OPTIONAL) },
                        SettingsButtonTone::Secondary, true, cx)
                        .icon(if expanded { IconName::ChevronUp } else { IconName::ChevronDown })
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                if this.active_dialog == Some(DialogState::AiMcpForm)
                                    && this.ai_extensions.mcp_test_request_id == form_request_id {
                                    this.ai_extensions.mcp_tools_expanded = !expanded;
                                    cx.notify();
                                }
                            });
                        })))
                .content(tools));
            if !self.ai_extensions.mcp_auto_discover && !selected_tools {
                body = body.child(error_text(khaslana::ui_i18n::text(messages::ENABLE_AT_LEAST_ONE_TOOL_BEFORE_SAVING_THE_SERVICE)));
            }
        }
        if let Some(error) = &self.ai_extensions.error {
            body = body.child(error_text(error.clone()));
        }
        body = body.child(meta_text(khaslana::ui_i18n::text(messages::UNKNOWN_DISCOVERED_TOOLS_REQUEST_WRITE_ACCESS_WITH_DISCOVERY_OFF_ONLY)));
        let viewport = div().id("ai-mcp-form-viewport").size_full().min_h(px(0.0))
            .overflow_y_scroll().track_scroll(&handle).child(body.flex_none());
        self.dialog_panel(if editing { khaslana::ui_i18n::text(messages::EDIT_MCP_SERVICE) } else { khaslana::ui_i18n::text(messages::ADD_MCP_SERVICE) }, cx)
            .w(panel_width).h(panel_height).overflow_hidden()
            .child(scrollable_frame_when("ai-mcp-form-scroll", ScrollbarMode::Vertical,
                viewport.into_any_element(), handle, true, cx))
            .child(dialog_actions().flex_none().bg(rgb(ui_theme::WB_PANEL))
                .child(self.button(khaslana::ui_i18n::text(messages::CANCEL), !busy && !testing,
                    |this, _, _| this.close_dialog(), cx))
                .child(self.primary_button(khaslana::ui_i18n::text(messages::SAVE_SERVICE), !busy && !testing
                    && !self.ai_extensions.mcp_id.value.trim().is_empty()
                    && !self.ai_extensions.mcp_command.value.trim().is_empty()
                    && (self.ai_extensions.mcp_auto_discover || selected_tools),
                    |this, _, _| this.save_ai_mcp_form(), cx)))
    }

    pub(crate) fn render_ai_mcp_remove_dialog(&self, name: &str,
        cx: &mut Context<Self>) -> impl IntoElement {
        let name_for_remove = name.to_owned();
        self.dialog_panel(khaslana::ui_i18n::text(messages::REMOVE_MCP_SERVICE), cx)
            .child(primary_text(khaslana::ui_i18n::format(messages::REMOVE_NAME, &[("name", &(name).to_string())])))
            .child(meta_text(khaslana::ui_i18n::text(messages::WORKFLOW_STEPS_REFERENCING_THIS_SERVICE_WILL_NO_LONGER_RUN)))
            .when_some(self.ai_extensions.error.as_ref(), |this, error| {
                this.child(error_text(error.clone()))
            })
            .child(dialog_actions()
                .child(self.button(khaslana::ui_i18n::text(messages::CANCEL), !self.ai_extensions.action_busy,
                    |this, _, _| this.close_dialog(), cx))
                .child(self.danger_button(khaslana::ui_i18n::text(messages::CONFIRM_REMOVAL), !self.ai_extensions.action_busy,
                    move |this, _, _| this.remove_ai_mcp_server(name_for_remove.clone()), cx)))
    }

    pub(crate) fn render_ai_mcp_builtin_details_dialog(&self,
        cx: &mut Context<Self>) -> impl IntoElement {
        let ready = self.ai_extensions.runtime.as_ref()
            .is_some_and(|runtime| runtime.mcp_ready);
        let mut panel = self.dialog_panel(khaslana::ui_i18n::text(messages::BUILT_IN_BROWSER_EDGE_SERVICE), cx).w(px(520.0))
            .child(meta_text(if ready { khaslana::ui_i18n::text(messages::RUNTIME_COMPONENTS_ARE_READY) } else {
                khaslana::ui_i18n::text(messages::RUNTIME_COMPONENTS_ARE_NOT_INSTALLED_DOWNLOAD_AND_ENABLE_THEM_ON)
            }))
            .child(meta_text(khaslana::ui_i18n::text(messages::CLIENT_MANAGED_MICROSOFT_EDGE_MCP_APPROVAL_IS_REQUESTED_BEFORE_EACH)));
        for (name, tool) in extensions::builtin_browser_server().tools {
            let access = match tool.access {
                WorkflowToolAccess::Read => khaslana::ui_i18n::text(messages::READ),
                WorkflowToolAccess::Write => khaslana::ui_i18n::text(messages::WRITE),
            };
            panel = panel.child(settings_row(name, access));
        }
        panel.child(dialog_actions()
            .child(self.button(khaslana::ui_i18n::text(messages::CLOSE), true, |this, _, _| this.close_dialog(), cx)))
    }
}
