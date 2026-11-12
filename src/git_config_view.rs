//! 设置中心「Git 配置」页：全局 / 当前仓库双作用域读写 git 配置。
//!
//! 设计约束（与代理/AI 设置页同模式）：
//! - 表单真值放 `GitConfigUiState`（跨仓库偏好挂 RepositoryView）；切换作用域、
//!   切换/关闭仓库时经 [`RepositoryView::reload_git_config`] 重置，不污染其他仓库。
//! - 配置值住在 git 自己的配置文件，经 `GitService::{read,write}_git_config` 读写，
//!   不进 AppStorage；读取与保存都走 `TaskKind::Short` + request_id 代际守卫，
//!   迟到结果丢弃，不阻塞 UI。
//! - 每个项显示「所选作用域层级的值」，未设置时描述行给出继承来源；文本留空或
//!   下拉选「未设置」保存即清除该层级键。
//! - 文本输入复用 `TextFieldState` + Kit 宿主（`this.input`）；下拉照语言页用稳定
//!   id + 本地化标签，语言切换只刷新显示项，不重建编辑状态。

#[path = "ui_resources/git_config_view_messages.rs"]
mod messages;

use std::collections::BTreeMap;
use std::sync::Arc;

use git2::Repository;
use gpui::{Context, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_kit::component::select::{Select, SelectEvent, SelectItem, SelectState};
use gpui_kit::component::setting::{SettingGroup, SettingItem};
use gpui_kit::component::{h_flex, v_flex};
use khaslana::language::Language;
use khaslana::{GitConfigItemState, GitConfigLevel, GitConfigScope, GitConfigSnapshot, GitService};

use crate::ui::components::{dialog_actions, segmented_button, settings_segmented_group, tooltip_text};
use crate::ui::theme::{self as ui_theme, rgb};
use crate::{FieldId, RepositoryView, TaskKind, UiEvent, send_ui_event, settings_center};

/// 下拉类配置项（布尔与枚举）。文本项（user.name/user.email/init.defaultBranch）
/// 不走下拉，直接用 Kit 输入。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum GitConfigSelectKey {
    Autocrlf,
    Quotepath,
    Longpaths,
    PullRebase,
    FetchPrune,
    RebaseAutoStash,
}

impl GitConfigSelectKey {
    pub(crate) const ALL: [Self; 6] = [
        Self::Autocrlf,
        Self::Quotepath,
        Self::Longpaths,
        Self::PullRebase,
        Self::FetchPrune,
        Self::RebaseAutoStash,
    ];

    pub(crate) fn config_key(self) -> &'static str {
        match self {
            Self::Autocrlf => "core.autocrlf",
            Self::Quotepath => "core.quotepath",
            Self::Longpaths => "core.longpaths",
            Self::PullRebase => "pull.rebase",
            Self::FetchPrune => "fetch.prune",
            Self::RebaseAutoStash => "rebase.autoStash",
        }
    }

    /// 标准选项（id 为 git 字面值，label 本地化）。布尔项用「开/关」，
    /// 枚举项直接显示 git 字面值（属代码值，不翻译）。
    fn value_options(self) -> Vec<(&'static str, SharedString)> {
        match self {
            Self::Autocrlf => vec![
                ("true", "true".into()),
                ("input", "input".into()),
                ("false", "false".into()),
            ],
            Self::PullRebase => vec![
                ("false", "false".into()),
                ("true", "true".into()),
                ("merges", "merges".into()),
            ],
            Self::Quotepath | Self::Longpaths | Self::FetchPrune | Self::RebaseAutoStash => vec![
                ("true", khaslana::ui_i18n::text(messages::ON).into()),
                ("false", khaslana::ui_i18n::text(messages::OFF).into()),
            ],
        }
    }
}

/// 下拉选项：稳定 id（git 值或 "unset"）+ 本地化标签。
#[derive(Clone)]
pub(crate) struct GitConfigOption {
    id: String,
    label: SharedString,
}

impl SelectItem for GitConfigOption {
    type Value = String;
    fn title(&self) -> SharedString {
        self.label.clone()
    }
    fn value(&self) -> &Self::Value {
        &self.id
    }
}

type GitConfigSelect = gpui::Entity<SelectState<Vec<GitConfigOption>>>;

/// 「Git 配置」页的表单与作用域状态。
pub(crate) struct GitConfigUiState {
    pub(crate) scope: GitConfigScope,
    pub(crate) user_name: crate::TextFieldState,
    pub(crate) user_email: crate::TextFieldState,
    pub(crate) default_branch: crate::TextFieldState,
    /// 下拉表单值：选项 id（"unset" 或 git 字面值）。
    select_values: BTreeMap<GitConfigSelectKey, String>,
    /// Kit 下拉宿主（惰性创建，见 ensure_git_config_selects）。
    selects: Vec<(GitConfigSelectKey, GitConfigSelect)>,
    /// 每个下拉最近一次构建选项时的 (语言, 当前值) 签名，避免每帧 set_items。
    select_signature: BTreeMap<GitConfigSelectKey, (Language, String)>,
    /// 已加载快照：dirty 比较与继承说明的基准。
    loaded: Option<GitConfigSnapshot>,
    pub(crate) request_id: u64,
    pub(crate) loading: bool,
    pub(crate) saving: bool,
}

impl GitConfigUiState {
    pub(crate) fn new(cx: &mut Context<RepositoryView>) -> Self {
        let mut select_values = BTreeMap::new();
        for key in GitConfigSelectKey::ALL {
            select_values.insert(key, UNSET.to_string());
        }
        Self {
            scope: GitConfigScope::Global,
            user_name: crate::TextFieldState::localized(cx, messages::USER_NAME_PLACEHOLDER),
            user_email: crate::TextFieldState::localized(cx, messages::USER_EMAIL_PLACEHOLDER),
            default_branch: crate::TextFieldState::localized(cx, messages::DEFAULT_BRANCH_PLACEHOLDER),
            select_values,
            selects: Vec::new(),
            select_signature: BTreeMap::new(),
            loaded: None,
            request_id: 0,
            loading: false,
            saving: false,
        }
    }

    fn select_value(&self, key: GitConfigSelectKey) -> String {
        self.select_values.get(&key).cloned().unwrap_or_else(|| UNSET.to_string())
    }
}

/// 「未设置」选项 id：保存时映射为清除该层级键。
pub(crate) const UNSET: &str = "unset";

/// 控件列固定宽度：下拉与文本输入统一宽度、统一左右位置，避免随描述文字
/// 长短参差不齐；文本列 flex_1 + min_w(0) 负责换行吸收剩余空间。
const GIT_CONFIG_CONTROL_WIDTH: f32 = 240.0;

/// 选项 id → git 值（"unset" → None 表示清除）。
pub(crate) fn option_id_to_value(id: &str) -> Option<String> {
    if id == UNSET { None } else { Some(id.to_string()) }
}

/// git 值 → 选项 id（None → "unset"）。
pub(crate) fn value_to_option_id(value: Option<&str>) -> String {
    value.unwrap_or(UNSET).to_string()
}

/// 文本框内容 → git 值：trim 后为空表示清除。
pub(crate) fn text_to_value(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

/// 配置操作不需要凭据/进度/代理，构造最小 GitService（不依赖活动 tab）。
fn git_config_service() -> GitService {
    GitService::new(
        Arc::new(khaslana::PromptCredentialProvider::memory_only(|_| Ok(None))),
        Arc::new(khaslana::NoopProgress),
    )
}

/// 构建下拉选项：标准选项 + 「未设置」；当前值不在标准选项内（如
/// pull.rebase=only）时补一个原样选项，保证回显与往返不丢值。
fn build_git_config_options(key: GitConfigSelectKey, current: &str) -> Vec<GitConfigOption> {
    let mut options = vec![GitConfigOption {
        id: UNSET.to_string(),
        label: khaslana::ui_i18n::text(messages::UNSET).into(),
    }];
    for (id, label) in key.value_options() {
        options.push(GitConfigOption { id: id.to_string(), label });
    }
    if current != UNSET && !options.iter().any(|option| option.id == current) {
        options.push(GitConfigOption { id: current.to_string(), label: current.into() });
    }
    options
}

impl RepositoryView {
    /// 惰性创建并同步六个下拉宿主（每帧经 ensure_kit_fields 调用）。
    pub(crate) fn ensure_git_config_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let language = self.ui_language;
        for key in GitConfigSelectKey::ALL {
            let current = self.git_config_ui.select_value(key);
            let options = build_git_config_options(key, &current);
            let existing = self.git_config_ui.selects.iter().find(|(k, _)| *k == key).map(|(_, state)| state.clone());
            let state = match existing {
                Some(state) => state,
                None => {
                    let state = cx.new(|cx| SelectState::new(options.clone(), None, window, cx));
                    cx.subscribe_in(&state, window, move |this, _, event: &SelectEvent<Vec<GitConfigOption>>, _, cx| {
                        let SelectEvent::Confirm(Some(value)) = event else { return };
                        this.git_config_ui.select_values.insert(key, value.clone());
                        cx.notify();
                    }).detach();
                    self.git_config_ui.selects.push((key, state.clone()));
                    state
                }
            };
            // 语言或当前值变化才重建选项，避免每帧 set_items。
            let signature = (language, current.clone());
            if self.git_config_ui.select_signature.get(&key) != Some(&signature) {
                state.update(cx, |state, cx| state.set_items(options, window, cx));
                self.git_config_ui.select_signature.insert(key, signature);
            }
            if state.read(cx).selected_value() != Some(&current) {
                state.update(cx, |state, cx| state.set_selected_value(&current, window, cx));
            }
        }
    }

    /// 切换作用域并重新加载（Local 无仓库时不可切换）。
    pub(crate) fn set_git_config_scope(&mut self, scope: GitConfigScope) {
        if self.git_config_ui.scope == scope {
            return;
        }
        if scope == GitConfigScope::Local && self.repo_path.is_none() {
            return;
        }
        self.git_config_ui.scope = scope;
        self.reload_git_config();
    }

    /// 后台加载配置快照（request_id 代际守卫）。
    pub(crate) fn reload_git_config(&mut self) {
        if self.git_config_ui.scope == GitConfigScope::Local && self.repo_path.is_none() {
            self.git_config_ui.scope = GitConfigScope::Global;
        }
        self.git_config_ui.request_id = self.git_config_ui.request_id.wrapping_add(1);
        let request_id = self.git_config_ui.request_id;
        self.git_config_ui.loading = true;
        let scope = self.git_config_ui.scope;
        let path = self.repo_path.clone();
        let service = git_config_service();
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Short, move || {
            let result = (|| -> khaslana::Result<GitConfigSnapshot> {
                let repo = match scope {
                    GitConfigScope::Local => {
                        let path = path.ok_or(khaslana::GitError::NoRepository)?;
                        Some(Repository::open(path)?)
                    }
                    GitConfigScope::Global => None,
                };
                service.read_git_config(repo.as_ref(), scope)
            })()
            .map_err(|error| error.to_string());
            send_ui_event(&tx, UiEvent::GitConfigLoaded { request_id, result });
        });
    }

    /// 收集表单相对已加载快照的差异（只提交改动项）。
    pub(crate) fn git_config_form_changes(&self) -> Vec<(String, Option<String>)> {
        let Some(loaded) = self.git_config_ui.loaded.as_ref() else {
            return Vec::new();
        };
        let mut changes = Vec::new();
        let mut consider = |key: &str, form: Option<String>| {
            let baseline = loaded.item(key).and_then(|item| item.value.clone());
            if form != baseline {
                changes.push((key.to_string(), form));
            }
        };
        consider("user.name", text_to_value(&self.git_config_ui.user_name.value));
        consider("user.email", text_to_value(&self.git_config_ui.user_email.value));
        consider("init.defaultBranch", text_to_value(&self.git_config_ui.default_branch.value));
        for key in GitConfigSelectKey::ALL {
            let form = option_id_to_value(&self.git_config_ui.select_value(key));
            consider(key.config_key(), form);
        }
        changes
    }

    pub(crate) fn git_config_form_is_dirty(&self) -> bool {
        !self.git_config_form_changes().is_empty()
    }

    /// 保存差异：后台写入并重读快照一并返回（request_id 代际守卫）。
    pub(crate) fn save_git_config(&mut self) {
        if self.git_config_ui.loading || self.git_config_ui.saving {
            return;
        }
        let changes = self.git_config_form_changes();
        if changes.is_empty() {
            return;
        }
        if self.git_config_ui.scope == GitConfigScope::Local && self.repo_path.is_none() {
            self.last_error = Some(khaslana::ui_i18n::encode(messages::NO_REPOSITORY_OPEN, &[]));
            return;
        }
        self.git_config_ui.request_id = self.git_config_ui.request_id.wrapping_add(1);
        let request_id = self.git_config_ui.request_id;
        self.git_config_ui.saving = true;
        let scope = self.git_config_ui.scope;
        let path = self.repo_path.clone();
        let service = git_config_service();
        let tx = self.tx.clone();
        self.tasks.spawn(TaskKind::Short, move || {
            let result = (|| -> khaslana::Result<GitConfigSnapshot> {
                let repo = match scope {
                    GitConfigScope::Local => {
                        let path = path.ok_or(khaslana::GitError::NoRepository)?;
                        Some(Repository::open(path)?)
                    }
                    GitConfigScope::Global => None,
                };
                service.write_git_config(repo.as_ref(), scope, &changes)?;
                service.read_git_config(repo.as_ref(), scope)
            })()
            .map_err(|error| error.to_string());
            send_ui_event(&tx, UiEvent::GitConfigSaved { request_id, result });
        });
    }

    /// 用快照回填表单（加载/保存成功后调用）。
    fn reset_git_config_form_from_snapshot(&mut self, snapshot: GitConfigSnapshot) {
        let value_of = |key: &str| snapshot.item(key).and_then(|item| item.value.clone()).unwrap_or_default();
        self.git_config_ui.user_name.set_value(value_of("user.name"));
        self.git_config_ui.user_email.set_value(value_of("user.email"));
        self.git_config_ui.default_branch.set_value(value_of("init.defaultBranch"));
        for key in GitConfigSelectKey::ALL {
            let value = snapshot.item(key.config_key()).and_then(|item| item.value.as_deref());
            self.git_config_ui.select_values.insert(key, value_to_option_id(value));
        }
        self.git_config_ui.loaded = Some(snapshot);
    }

    /// 加载完成事件处理（代际守卫；迟到结果丢弃）。
    pub(crate) fn handle_git_config_loaded(
        &mut self,
        request_id: u64,
        result: std::result::Result<GitConfigSnapshot, String>,
        cx: &mut Context<Self>,
    ) {
        if self.git_config_ui.request_id != request_id {
            return;
        }
        self.git_config_ui.loading = false;
        match result {
            Ok(snapshot) => {
                self.reset_git_config_form_from_snapshot(snapshot);
                self.last_error = None;
            }
            Err(error) => {
                self.last_error = Some(error);
            }
        }
        cx.notify();
    }

    /// 保存完成事件处理：成功回填并按 last_error 提示，失败带错误 toast。
    pub(crate) fn handle_git_config_saved(
        &mut self,
        request_id: u64,
        result: std::result::Result<GitConfigSnapshot, String>,
        cx: &mut Context<Self>,
    ) {
        if self.git_config_ui.request_id != request_id {
            return;
        }
        self.git_config_ui.saving = false;
        match result {
            Ok(snapshot) => {
                self.reset_git_config_form_from_snapshot(snapshot);
                self.last_error = None;
                self.status = khaslana::ui_i18n::encode(messages::GIT_CONFIG_SAVED, &[]);
            }
            Err(error) => {
                self.last_error = Some(error);
            }
        }
        if self.settings_center == Some(crate::SettingsCategory::GitConfig) {
            self.notify_settings_save(khaslana::ui_i18n::text(messages::GIT_CONFIG_SAVED), cx);
        }
        cx.notify();
    }

    fn git_config_item(&self, key: &str) -> Option<&GitConfigItemState> {
        self.git_config_ui.loaded.as_ref()?.item(key)
    }

    /// 继承说明：本层级未设置时给出生效值与来源；已设置则无说明。
    fn git_config_inheritance_caption(&self, key: &str) -> Option<SharedString> {
        let item = self.git_config_item(key)?;
        if item.value.is_some() {
            return None;
        }
        match (&item.effective, item.effective_level) {
            (Some(value), level) => {
                let source = match level {
                    Some(GitConfigLevel::Local) => khaslana::ui_i18n::text(messages::SOURCE_LOCAL),
                    Some(GitConfigLevel::Global) => khaslana::ui_i18n::text(messages::SOURCE_GLOBAL),
                    Some(GitConfigLevel::System) => khaslana::ui_i18n::text(messages::SOURCE_SYSTEM),
                    _ => khaslana::ui_i18n::text(messages::SOURCE_OTHER),
                };
                Some(khaslana::ui_i18n::format(messages::INHERITED_FROM, &[("source", source), ("value", value)]).into())
            }
            (None, _) => Some(khaslana::ui_i18n::text(messages::NOT_SET_AT_LEVEL).into()),
        }
    }

    /// 当前生效提交身份说明（两项齐全给身份，否则给警告）。
    fn git_config_identity_caption(&self) -> SharedString {
        let name = self.git_config_item("user.name").and_then(|item| item.effective.clone()).unwrap_or_default();
        let email = self.git_config_item("user.email").and_then(|item| item.effective.clone()).unwrap_or_default();
        if !name.trim().is_empty() && !email.trim().is_empty() {
            let identity = format!("{name} <{email}>");
            khaslana::ui_i18n::format(messages::EFFECTIVE_IDENTITY, &[("identity", &identity)]).into()
        } else {
            khaslana::ui_i18n::text(messages::IDENTITY_NOT_SET).into()
        }
    }

    fn git_config_scope_button(
        &self,
        label: &'static str,
        scope: GitConfigScope,
        enabled: bool,
        disabled_reason: Option<&'static str>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.git_config_ui.scope == scope;
        segmented_button(format!("git-config-scope-{label}"), selected, enabled)
            .when_some(disabled_reason, |this, reason| {
                this.tooltip(move |_window, cx| tooltip_text(reason, cx))
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                if enabled {
                    this.set_git_config_scope(scope);
                    cx.notify();
                }
            }))
            .child(label)
    }

    fn render_git_config_select(&self, key: GitConfigSelectKey) -> gpui::AnyElement {
        let Some(state) = self.git_config_ui.selects.iter().find(|(k, _)| *k == key).map(|(_, state)| state.clone()) else {
            return div().into_any_element();
        };
        let label = git_config_select_label(key);
        Select::new(&state)
            .accessibility_label(label)
            .w_full()
            .h(px(34.0))
            .text_size(px(ui_theme::TYPE_BODY))
            .bg(rgb(ui_theme::WB_INPUT_SURFACE))
            .border_color(rgb(ui_theme::BORDER_MUTED))
            .rounded(px(ui_theme::RADIUS_SM))
            .into_any_element()
    }

    /// 「Git 配置」页的分组（Kit SettingGroup 列表）。
    pub(crate) fn settings_git_config_groups(
        &self,
        _window: &Window,
        cx: &mut Context<Self>,
    ) -> Vec<SettingGroup> {
        let has_repo = self.repo_path.is_some();
        let busy = self.git_config_ui.loading || self.git_config_ui.saving;
        let scope_enabled = !busy;
        let local_disabled_reason = if !has_repo {
            Some(khaslana::ui_i18n::text(messages::NO_REPOSITORY_OPEN))
        } else if busy {
            Some(khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT))
        } else {
            None
        };
        let global_disabled_reason = busy.then_some(khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT));

        let mut groups = Vec::new();

        // 作用域 + 生效身份。
        groups.push(
            SettingGroup::new()
                .item(settings_center::settings_group_heading(
                    khaslana::ui_i18n::text(messages::SCOPE),
                    Some(khaslana::ui_i18n::text(messages::SCOPE_HELP).into()),
                ))
                .item({
                    let view = cx.entity();
                    SettingItem::render(move |_options, _window, cx| {
                        view.update(cx, |this, cx| {
                            settings_segmented_group()
                                .child(this.git_config_scope_button(khaslana::ui_i18n::text(messages::GLOBAL), GitConfigScope::Global, scope_enabled, global_disabled_reason, cx))
                                .child(this.git_config_scope_button(khaslana::ui_i18n::text(messages::CURRENT_REPO), GitConfigScope::Local, scope_enabled && has_repo, local_disabled_reason, cx))
                        })
                    }).keywords([khaslana::ui_i18n::text(messages::SCOPE)])
                })
                .item({
                    let view = cx.entity();
                    SettingItem::render(move |_options, _window, cx| {
                        view.update(cx, |this, _cx| {
                            div().text_size(px(ui_theme::TYPE_META)).line_height(px(16.0))
                                .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                                .child(this.git_config_identity_caption())
                        })
                    })
                }),
        );

        // 提交身份。
        groups.push(
            SettingGroup::new()
                .item(settings_center::settings_group_heading(
                    khaslana::ui_i18n::text(messages::IDENTITY_GROUP),
                    Some(khaslana::ui_i18n::text(messages::IDENTITY_GROUP_DESC).into()),
                ))
                .item(self.git_config_text_item(
                    khaslana::ui_i18n::text(messages::USER_NAME),
                    khaslana::ui_i18n::text(messages::USER_NAME_DESC),
                    "user.name",
                    FieldId::GitConfigUserName,
                    cx,
                ))
                .item(self.git_config_text_item(
                    khaslana::ui_i18n::text(messages::USER_EMAIL),
                    khaslana::ui_i18n::text(messages::USER_EMAIL_DESC),
                    "user.email",
                    FieldId::GitConfigUserEmail,
                    cx,
                )),
        );

        // 文件与换行。
        groups.push(
            SettingGroup::new()
                .item(settings_center::settings_group_heading(
                    khaslana::ui_i18n::text(messages::FILES_GROUP),
                    Some(khaslana::ui_i18n::text(messages::FILES_GROUP_DESC).into()),
                ))
                .item(self.git_config_select_item(khaslana::ui_i18n::text(messages::AUTOCRLF), khaslana::ui_i18n::text(messages::AUTOCRLF_DESC), GitConfigSelectKey::Autocrlf, cx))
                .item(self.git_config_select_item(khaslana::ui_i18n::text(messages::QUOTEPATH), khaslana::ui_i18n::text(messages::QUOTEPATH_DESC), GitConfigSelectKey::Quotepath, cx))
                .item(self.git_config_select_item(khaslana::ui_i18n::text(messages::LONGPATHS), khaslana::ui_i18n::text(messages::LONGPATHS_DESC), GitConfigSelectKey::Longpaths, cx)),
        );

        // 分支与同步。
        groups.push(
            SettingGroup::new()
                .item(settings_center::settings_group_heading(
                    khaslana::ui_i18n::text(messages::BRANCH_GROUP),
                    Some(khaslana::ui_i18n::text(messages::BRANCH_GROUP_DESC).into()),
                ))
                .item(self.git_config_text_item(
                    khaslana::ui_i18n::text(messages::DEFAULT_BRANCH),
                    khaslana::ui_i18n::text(messages::DEFAULT_BRANCH_DESC),
                    "init.defaultBranch",
                    FieldId::GitConfigDefaultBranch,
                    cx,
                ))
                .item(self.git_config_select_item(khaslana::ui_i18n::text(messages::PULL_REBASE), khaslana::ui_i18n::text(messages::PULL_REBASE_DESC), GitConfigSelectKey::PullRebase, cx))
                .item(self.git_config_select_item(khaslana::ui_i18n::text(messages::FETCH_PRUNE), khaslana::ui_i18n::text(messages::FETCH_PRUNE_DESC), GitConfigSelectKey::FetchPrune, cx))
                .item(self.git_config_select_item(khaslana::ui_i18n::text(messages::REBASE_AUTOSTASH), khaslana::ui_i18n::text(messages::REBASE_AUTOSTASH_DESC), GitConfigSelectKey::RebaseAutoStash, cx)),
        );

        // 保存栏。
        let dirty = self.git_config_form_is_dirty();
        let save_enabled = dirty && !busy;
        let status_caption = if self.git_config_ui.loading {
            khaslana::ui_i18n::text(messages::LOADING)
        } else if self.git_config_ui.saving {
            khaslana::ui_i18n::text(messages::SAVING)
        } else if dirty {
            khaslana::ui_i18n::text(messages::UNSAVED_CHANGES)
        } else {
            khaslana::ui_i18n::text(messages::NO_CHANGES)
        };
        let scope_hint = if self.git_config_ui.scope == GitConfigScope::Local {
            khaslana::ui_i18n::text(messages::SAVE_SCOPE_LOCAL)
        } else {
            khaslana::ui_i18n::text(messages::SAVE_SCOPE_GLOBAL)
        };
        groups.push(
            SettingGroup::new()
                .item({
                    let view = cx.entity();
                    SettingItem::render(move |_options, _window, cx| {
                        view.update(cx, |this, cx| {
                            v_flex().w_full().gap(px(ui_theme::SPACE_2))
                                .child(div().text_size(px(ui_theme::TYPE_META)).line_height(px(16.0))
                                    .text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(scope_hint))
                                .child(dialog_actions()
                                    .child(div().flex_1().text_size(px(ui_theme::TYPE_META)).line_height(px(16.0))
                                        .text_color(rgb(ui_theme::CONTENT_TERTIARY)).child(status_caption))
                                    .child(this.primary_button(
                                        khaslana::ui_i18n::text(messages::SAVE),
                                        save_enabled,
                                        |this, _, cx| {
                                            this.save_git_config();
                                            cx.notify();
                                        },
                                        cx,
                                    )))
                        })
                    }).keywords([khaslana::ui_i18n::text(messages::SAVE)])
                }),
        );

        groups
    }

    /// 文本配置项行：标题 + 静态说明 + 继承说明 + 右侧输入框。
    fn git_config_text_item(
        &self,
        title: &'static str,
        description: &'static str,
        key: &'static str,
        field: FieldId,
        cx: &mut Context<Self>,
    ) -> SettingItem {
        let view = cx.entity();
        let caption = self.git_config_inheritance_caption(key);
        SettingItem::render(move |_options, window, cx| {
            view.update(cx, |this, cx| {
                let control = this.input(field, false, window, cx);
                git_config_row(title, Some(description.into()), caption.clone(), control)
            })
        }).keywords([title])
    }

    /// 下拉配置项行：标题 + 静态说明 + 继承说明 + 右侧下拉。
    fn git_config_select_item(
        &self,
        title: &'static str,
        description: &'static str,
        key: GitConfigSelectKey,
        cx: &mut Context<Self>,
    ) -> SettingItem {
        let view = cx.entity();
        let caption = self.git_config_inheritance_caption(key.config_key());
        SettingItem::render(move |_options, _window, cx| {
            view.update(cx, |this, _cx| {
                let control = this.render_git_config_select(key);
                git_config_row(title, Some(description.into()), caption.clone(), control)
            })
        }).keywords([title])
    }
}

/// 下拉的可访问标签（复用项标题）。
fn git_config_select_label(key: GitConfigSelectKey) -> &'static str {
    match key {
        GitConfigSelectKey::Autocrlf => khaslana::ui_i18n::text(messages::AUTOCRLF),
        GitConfigSelectKey::Quotepath => khaslana::ui_i18n::text(messages::QUOTEPATH),
        GitConfigSelectKey::Longpaths => khaslana::ui_i18n::text(messages::LONGPATHS),
        GitConfigSelectKey::PullRebase => khaslana::ui_i18n::text(messages::PULL_REBASE),
        GitConfigSelectKey::FetchPrune => khaslana::ui_i18n::text(messages::FETCH_PRUNE),
        GitConfigSelectKey::RebaseAutoStash => khaslana::ui_i18n::text(messages::REBASE_AUTOSTASH),
    }
}

/// 配置项行布局（复刻 settings_item_body，但说明与继承说明支持动态文案）：
/// 文本列 flex_1 + min_w(0) 吸收剩余宽度、显示不下自动换行；控件列固定宽度
/// flex_none，保证每行下拉/输入框左右位置与宽度完全对齐，不被文字挤出去。
/// 说明 TYPE_META Secondary，继承说明 Tertiary。
fn git_config_row(
    title: &'static str,
    description: Option<SharedString>,
    caption: Option<SharedString>,
    control: impl IntoElement,
) -> gpui::Div {
    h_flex()
        .w_full()
        .items_center()
        .gap_3()
        .child(
            v_flex()
                .flex_1()
                .min_w(px(0.0))
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(ui_theme::TYPE_BODY))
                        .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                        .child(title),
                )
                .when_some(description, |this, text| {
                    this.child(
                        div()
                            .text_size(px(ui_theme::TYPE_META))
                            .line_height(px(16.0))
                            .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                            .child(text),
                    )
                })
                .when_some(caption, |this, text| {
                    this.child(
                        div()
                            .text_size(px(ui_theme::TYPE_META))
                            .line_height(px(16.0))
                            .text_color(rgb(ui_theme::CONTENT_TERTIARY))
                            .child(text),
                    )
                }),
        )
        .child(
            div()
                .w(px(GIT_CONFIG_CONTROL_WIDTH))
                .flex_none()
                .child(control),
        )
}

#[cfg(test)]
#[path = "tests/git_config_view.rs"]
mod tests;
