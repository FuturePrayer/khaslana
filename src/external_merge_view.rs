#[path = "ui_resources/external_merge_view_messages.rs"]
mod messages;

use std::{
    path::Path,
    rc::Rc,
    sync::{Mutex, OnceLock},
    thread,
};

use gpui::{App, Context, Window, div, prelude::*, px};
use gpui_kit::component::setting::SettingGroup;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, Sizable};

use crate::ui::theme::rgb;
use crate::{
    FieldId, RepositoryView, UiEvent, send_ui_event,
    settings_center::{
        settings_compact_heading, settings_compact_list, settings_item_body,
    },
    ui::{components::dialog_actions, theme as ui_theme},
};

static PENDING_EXTERNAL_MERGE_PATH: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn external_merge_detection_label(
    settings: &khaslana::ExternalMergeSettings,
    detection: Option<&(khaslana::ExternalMergeSettings, bool)>,
) -> &'static str {
    match detection.filter(|(detected_settings, _)| detected_settings == settings) {
        Some((_, true)) => khaslana::ui_i18n::text(messages::IDEA_AVAILABLE),
        Some((_, false)) => khaslana::ui_i18n::text(messages::DETECTION_FAILED_RETRY),
        None => khaslana::ui_i18n::text(messages::DETECT_IDEA),
    }
}

fn pending_external_merge_path_cell() -> &'static Mutex<Option<String>> {
    PENDING_EXTERNAL_MERGE_PATH.get_or_init(|| Mutex::new(None))
}

fn pending_external_merge_path() -> Option<String> {
    pending_external_merge_path_cell()
        .lock()
        .ok()
        .and_then(|pending| pending.clone())
}

fn set_pending_external_merge_path(path: String) {
    if let Ok(mut pending) = pending_external_merge_path_cell().lock() {
        *pending = Some(path);
    }
}

fn take_pending_external_merge_path() -> Option<String> {
    pending_external_merge_path_cell()
        .lock()
        .ok()
        .and_then(|mut pending| pending.take())
}

pub(crate) fn clear_pending_external_merge_path() {
    if let Ok(mut pending) = pending_external_merge_path_cell().lock() {
        *pending = None;
    }
}

impl RepositoryView {
    /// 合并工具页的分组（Kit `SettingGroup` 列表）。
    ///
    /// 对应旧弹窗 `render_external_merge_settings_dialog` 的迁移：「外部合并」=
    /// pending 待续提示 + 两个开关；「IDEA 程序」= 路径输入 + 浏览按钮 +
    /// 说明 + last_error + 检测/保存按钮组。`save_label` / `detection_label`
    /// 的计算时机与旧弹窗一致：分组每次渲染重建，捕获值等价于渲染期读取。
    ///
    /// 每组用 [`localized_settings_compact_item`] 整卡单条目自排 8px 行距（对齐旧弹窗
    /// `gap_2`）；Kit `SettingGroup` 逐条排的 16px 行距会把表单撑散。
    pub(crate) fn settings_external_merge_groups(
        &self,
        _window: &Window,
        cx: &mut Context<Self>,
    ) -> Vec<SettingGroup> {
        // render 闭包必须 'static，每个闭包各持一份 Entity 克隆（廉价句柄）。
        let view = cx.entity();
        // 开关写回闭包经 Rc 共享：渲染闭包是 Fn（Kit 每次渲染调用），
        // on_click 每帧各 move 一份克隆。
        let enabled_value_view = view.clone();
        let enabled_set: Rc<dyn Fn(bool, &mut App)> = {
            let view = view.clone();
            Rc::new(move |value: bool, cx: &mut App| {
                view.update(cx, |this, cx| {
                    this.set_external_merge_enabled_form_with_detection(value);
                    cx.notify();
                });
            })
        };
        let auto_open_value_view = view.clone();
        let auto_open_set: Rc<dyn Fn(bool, &mut App)> = {
            let view = view.clone();
            Rc::new(move |value: bool, cx: &mut App| {
                view.update(cx, |this, cx| {
                    this.set_external_merge_auto_open_form_with_detection(value);
                    cx.notify();
                });
            })
        };
        let path_view = view.clone();
        let browse_view = view.clone();
        let actions_view = view.clone();
        let pending_path = pending_external_merge_path();
        let save_label = if pending_path.is_some() {
            khaslana::ui_i18n::text(messages::SAVE_AND_CONTINUE)
        } else {
            khaslana::ui_i18n::text(messages::SAVE)
        };
        let detection_label = external_merge_detection_label(
            &self.external_merge_form_settings(),
            self.external_merge_detection.as_ref(),
        );
        let has_last_error = self.last_error.is_some();
        let last_error_text = self.last_error.clone().unwrap_or_default();

        vec![
            SettingGroup::new().item(localized_settings_compact_item(
                &[
                    khaslana::ui_i18n::text(messages::EXTERNAL_MERGE),
                    khaslana::ui_i18n::text(messages::ENABLE_INTELLIJ_IDEA_EXTERNAL_MERGE),
                    khaslana::ui_i18n::text(messages::AUTOMATICALLY_OPEN_IDEA_WHEN_SELECTING_A_CONFLICT_FILE),
                ],
                move |options, _window, cx| {
                    let enabled = enabled_value_view.read(cx).external_merge_enabled_form;
                    let auto_open = auto_open_value_view.read(cx).external_merge_auto_open_form;
                    let mut list = settings_compact_list()
                        .child(settings_compact_heading(khaslana::ui_i18n::text(messages::EXTERNAL_MERGE), None));
                    // 冲突解决被 IDEA 缺失阻断时的待续提示：沿用旧弹窗的 when_some 条件。
                    if let Some(path) = pending_path.clone() {
                        list = list.child(
                            div()
                                .px_3()
                                .py_2()
                                .rounded(px(ui_theme::RADIUS_XS))
                                .border_1()
                                .border_color(rgb(ui_theme::BORDER_STRONG))
                                .bg(rgb(ui_theme::SURFACE_SUNKEN))
                                .text_size(px(12.0))
                                .line_height(px(18.0))
                                .text_color(rgb(ui_theme::CONTENT_PRIMARY))
                                .child(khaslana::ui_i18n::format(messages::NO_USABLE_INTELLIJ_IDEA_INSTALLATION_FOUND_AFTER_CONFIGURATION_RESOLVING_WILL, &[("path", &(path).to_string())])),
                        );
                    }
                    let set_enabled = enabled_set.clone();
                    let set_auto_open = auto_open_set.clone();
                    list.child(settings_item_body(
                        khaslana::ui_i18n::text(messages::ENABLE_INTELLIJ_IDEA_EXTERNAL_MERGE),
                        None,
                        Switch::new("merge-enabled")
                            .checked(enabled)
                            .disabled(options.is_disabled())
                            .with_size(options.size())
                            .on_click(move |checked: &bool, _, cx: &mut App| {
                                set_enabled(*checked, cx)
                            }),
                    ))
                    .child(settings_item_body(
                        khaslana::ui_i18n::text(messages::AUTOMATICALLY_OPEN_IDEA_WHEN_SELECTING_A_CONFLICT_FILE),
                        None,
                        Switch::new("merge-auto-open")
                            .checked(auto_open)
                            .disabled(options.is_disabled())
                            .with_size(options.size())
                            .on_click(move |checked: &bool, _, cx: &mut App| {
                                set_auto_open(*checked, cx)
                            }),
                    ))
                },
            )),
            SettingGroup::new().item(localized_settings_compact_item(
                &[khaslana::ui_i18n::text(messages::IDEA_EXECUTABLE), khaslana::ui_i18n::text(messages::IDEA_PATH), khaslana::ui_i18n::text(messages::SELECT_IDEA_EXECUTABLE), khaslana::ui_i18n::text(messages::DETECT_IDEA), khaslana::ui_i18n::text(messages::SAVE)],
                move |_options, window, cx| {
                    let path_input = path_view.update(cx, |this, cx| {
                        div().w(px(280.0)).max_w_full().child(this.input(
                            FieldId::ExternalMergeIntellijPath,
                            false,
                            window,
                            cx,
                        ))
                    });
                    // 按钮元素的 opaque 类型捕获 `this` / `cx` 的借用，不能
                    // 直接作为行内容返回（要求 'static）；包一层 div 让按钮
                    // 以子元素存入，类型即与借用脱钩。
                    let browse_button = browse_view.update(cx, |this, cx| {
                        div().child(this.button(
                            khaslana::ui_i18n::text(messages::SELECT_IDEA_EXECUTABLE),
                            !this.busy,
                            |this, _, _| this.browse_external_merge_executable(),
                            cx,
                        ))
                    });
                    let mut list = settings_compact_list()
                        .child(settings_compact_heading(khaslana::ui_i18n::text(messages::IDEA_EXECUTABLE), None))
                        .child(settings_item_body(khaslana::ui_i18n::text(messages::IDEA_PATH), None, path_input))
                        .child(browse_button)
                        .child(
                            div()
                                .text_size(px(12.0))
                                .line_height(px(18.0))
                                .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                                .child(khaslana::ui_i18n::text(messages::THE_PATH_IS_OPTIONAL_WHEN_EMPTY_DETECTION_CHECKS_KHASLANA_IDEA)),
                        );
                    // last_error 行沿用旧弹窗的 when 条件。
                    if has_last_error {
                        list = list.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(ui_theme::DESTRUCTIVE))
                                .child(last_error_text.clone()),
                        );
                    }
                    let actions = actions_view.update(cx, |this, cx| {
                        dialog_actions()
                            .child(this.button(
                                detection_label,
                                !this.busy,
                                |this, _, _| this.test_external_merge_settings_from_form(),
                                cx,
                            ))
                            .child(this.primary_button(
                                save_label,
                                !this.busy,
                                |this, _, cx| {
                                    this.save_external_merge_settings_from_form_and_resume();
                                    this.notify_settings_save(khaslana::ui_i18n::text(messages::MERGE_TOOL_SETTINGS_SAVED), cx);
                                },
                                cx,
                            ))
                    });
                    list.child(actions)
                },
            )),
        ]
    }

    pub(crate) fn request_external_merge_for_path(&mut self, path: String) -> bool {
        let validation = if self.external_merge_settings.enabled {
            khaslana::external_merge::resolve_intellij_idea_command_with_settings(
                &self.external_merge_settings,
            )
        } else {
            Err(khaslana::GitError::Message(khaslana::ui_i18n::text(messages::EXTERNAL_MERGE_TOOL_IS_NOT_ENABLED).into()))
        };

        if let Err(err) = validation {
            set_pending_external_merge_path(path);
            self.open_external_merge_settings();
            if !self.external_merge_enabled_form {
                self.set_external_merge_enabled_form(true);
            }
            self.status = khaslana::ui_i18n::encode(messages::CONFIGURE_THE_INTELLIJ_IDEA_MERGE_TOOL, &[]);
            self.last_error = Some(khaslana::ui_i18n::encode(messages::CANNOT_START_THE_EXTERNAL_MERGE_TOOL_ERR_SELECT_THE_IDEA, &[("err", &(err).to_string())]));
            return false;
        }

        self.start_external_merge_operation(path);
        true
    }

    fn start_external_merge_operation(&mut self, path: String) {
        let settings = self.external_merge_settings.clone();
        self.diff = None;
        self.diff_headers_expanded = false;
        self.reset_uniform_scroll("diff-scroll");
        self.with_repo_blocking(
            crate::operation_ui_resources::EXTERNAL_MERGE_APPLIED,
            move |service, repo| {
                service.resolve_conflict_with_intellij_idea_settings(
                    repo,
                    Path::new(&path),
                    &settings,
                )
            },
        );
    }

    pub(crate) fn save_external_merge_settings_from_form_and_resume(&mut self) {
        let settings = self.external_merge_form_settings();
        let has_pending_merge = pending_external_merge_path().is_some();

        if has_pending_merge && !settings.enabled {
            self.last_error = Some(khaslana::ui_i18n::encode(messages::ENABLE_THE_EXTERNAL_MERGE_TOOL_BEFORE_SAVING_AND_CONTINUING_THIS, &[]));
            return;
        }

        let resolved_path = if settings.enabled {
            match khaslana::external_merge::resolve_intellij_idea_command_with_settings(&settings) {
                Ok(path) => Some(path),
                Err(err) => {
                    self.status = khaslana::ui_i18n::encode(messages::CONFIGURE_THE_INTELLIJ_IDEA_MERGE_TOOL, &[]);
                    self.last_error = Some(khaslana::ui_i18n::encode(messages::NO_USABLE_INTELLIJ_IDEA_INSTALLATION_FOUND_ERR_ENTER_OR_SELECT, &[("err", &(err).to_string())]));
                    return;
                }
            }
        } else {
            None
        };

        self.external_merge_settings = settings;
        self.save_external_merge_settings();
        self.status = resolved_path
            .map(|path| khaslana::ui_i18n::encode(messages::MERGE_TOOL_SETTINGS_SAVED_PATH, &[("path", &(path.display()).to_string())]))
            .unwrap_or_else(|| khaslana::ui_i18n::encode(messages::MERGE_TOOL_SETTINGS_SAVED, &[]));
        self.last_error = None;

        if let Some(path) = take_pending_external_merge_path() {
            self.start_external_merge_operation(path);
        }
    }

    pub(crate) fn set_external_merge_enabled_form_with_detection(&mut self, enabled: bool) {
        self.set_external_merge_enabled_form(enabled);
        if !enabled {
            return;
        }

        let settings = self.external_merge_form_settings();
        match khaslana::external_merge::resolve_intellij_idea_command_with_settings(&settings) {
            Ok(path) => {
                self.external_merge_detection = Some((settings, true));
                self.status = khaslana::ui_i18n::encode(messages::INTELLIJ_IDEA_FOUND_PATH, &[("path", &(path.display()).to_string())]);
                self.last_error = None;
            }
            Err(err) => {
                self.external_merge_detection = Some((settings, false));
                self.status = khaslana::ui_i18n::encode(messages::CONFIGURE_THE_INTELLIJ_IDEA_MERGE_TOOL, &[]);
                self.last_error = Some(khaslana::ui_i18n::encode(messages::CONFIGURE_INTELLIJ_IDEA_BEFORE_ENABLING_ERR_ENTER_A_PATH_OR, &[("err", &(err).to_string())]));
            }
        }
    }

    pub(crate) fn set_external_merge_auto_open_form_with_detection(&mut self, enabled: bool) {
        self.set_external_merge_auto_open_form(enabled);
        if !enabled {
            return;
        }

        let settings = self.external_merge_form_settings();
        match khaslana::external_merge::resolve_intellij_idea_command_with_settings(&settings) {
            Ok(path) => {
                self.external_merge_detection = Some((settings, true));
                self.status = khaslana::ui_i18n::encode(messages::INTELLIJ_IDEA_FOUND_PATH, &[("path", &(path.display()).to_string())]);
                self.last_error = None;
            }
            Err(err) => {
                self.external_merge_detection = Some((settings, false));
                self.status = khaslana::ui_i18n::encode(messages::CONFIGURE_THE_INTELLIJ_IDEA_MERGE_TOOL, &[]);
                self.last_error = Some(khaslana::ui_i18n::encode(messages::CONFIGURE_INTELLIJ_IDEA_BEFORE_ENABLING_AUTOMATIC_OPENING_ERR_ENTER_A, &[("err", &(err).to_string())]));
            }
        }
    }

    pub(crate) fn browse_external_merge_executable(&mut self) {
        self.status = khaslana::ui_i18n::encode(messages::SELECTING_INTELLIJ_IDEA_EXECUTABLE, &[]);
        self.last_error = None;
        let tx = self.tx.clone();
        // Windows 原生文件框有自己的 COM / 消息循环，不能阻塞在 GPUI 事件回调中。
        thread::spawn(move || {
            let dialog = rfd::FileDialog::new().set_title(khaslana::ui_i18n::text(messages::SELECT_INTELLIJ_IDEA_LAUNCHER));
            #[cfg(windows)]
            let dialog = dialog.add_filter("IntelliJ IDEA", &["exe", "bat", "cmd"]);
            let path = dialog.pick_file();
            send_ui_event(&tx, UiEvent::ExternalMergeExecutableSelected { path });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection_button_only_shows_result_for_current_settings() {
        let settings = khaslana::ExternalMergeSettings {
            enabled: true,
            auto_open_intellij: false,
            intellij_path: "idea64.exe".into(),
        };
        let succeeded = (settings.clone(), true);
        assert_eq!(
            external_merge_detection_label(&settings, Some(&succeeded)),
            "✓ IDEA 可用"
        );

        let mut changed = settings.clone();
        changed.intellij_path = "other.exe".into();
        assert_eq!(
            external_merge_detection_label(&changed, Some(&succeeded)),
            "检测 IDEA"
        );
    }
}

// Kit 设置项即时复制关键词；局部桥接不要求临时翻译数组具有静态生命周期。
fn localized_settings_compact_item<E, F>(keywords: &[&'static str], body: F) -> gpui_kit::component::setting::SettingItem
where F: Fn(&gpui_kit::component::setting::RenderOptions, &mut Window, &mut App) -> E + 'static, E: IntoElement {
    gpui_kit::component::setting::SettingItem::render(body).keywords(keywords.iter().copied())
}
