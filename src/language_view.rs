//! 语言设置保留稳定选项身份，不重建仓库视图或编辑实体。
use gpui::{Context, Entity, SharedString, Window, prelude::*, px};
use gpui_kit::component::{select::{Select, SelectEvent, SelectItem, SelectState}, setting::SettingGroup};
use khaslana::{language::{LanguagePreferences, PromptLanguagePreference, UiLanguagePreference}, language_resources::{MessageKey, template, format_message}};
use crate::{RepositoryView, ui::theme::{self as theme, rgb}};

#[derive(Clone)]
pub(crate) struct LanguageOption {
    id: &'static str,
    label: SharedString,
}
impl SelectItem for LanguageOption {
    type Value = &'static str;
    fn title(&self) -> SharedString { self.label.clone() }
    fn value(&self) -> &Self::Value { &self.id }
}

pub(crate) type LanguageSelect = Entity<SelectState<Vec<LanguageOption>>>;

impl RepositoryView {
    pub(crate) fn load_languages(storage: &khaslana::AppStorage) -> LanguagePreferences {
        storage.load_language_preferences()
            .inspect_err(|error| tracing::warn!("language preferences load skipped: {error}"))
            .unwrap_or_default()
    }

    pub(crate) fn ensure_language_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let language = self.ui_language;
        for prompt in [false, true] {
            let options = if prompt {
                vec![("follow-ui", MessageKey::FollowUi), ("zh-CN", MessageKey::SimplifiedChinese), ("en", MessageKey::English)]
            } else {
                vec![("system", MessageKey::System), ("zh-CN", MessageKey::SimplifiedChinese), ("en", MessageKey::English)]
            }.into_iter().map(|(id, key)| LanguageOption { id, label: template(language, key).into() }).collect::<Vec<_>>();
            let slot = if prompt { &mut self.prompt_language_select } else { &mut self.ui_language_select };
            if slot.is_none() {
                let state = cx.new(|cx| SelectState::new(options.clone(), None, window, cx));
                cx.subscribe_in(&state, window, move |this, _, event: &SelectEvent<Vec<LanguageOption>>, window, cx| {
                    let SelectEvent::Confirm(Some(value)) = event else { return };
                    let result = if prompt {
                        this.storage.save_prompt_language(PromptLanguagePreference::from_id(value))
                    } else {
                        this.storage.save_ui_language(UiLanguagePreference::from_id(value))
                    };
                    if let Err(error) = result {
                        this.notify_error(format_message(this.ui_language, MessageKey::SaveFailed, &[("error", &error.to_string())]), cx);
                        cx.notify();
                        return;
                    }
                    if prompt {
                        this.language_preferences.prompt = PromptLanguagePreference::from_id(value);
                    } else {
                        this.language_preferences.ui = UiLanguagePreference::from_id(value);
                        // 即使重复选择 System 也重新检测，不订阅实时操作系统通知。
                        this.ui_language = this.language_preferences.ui.resolve();
                        gpui_kit::component::set_locale(this.ui_language.kit_locale());
                        khaslana::ui_i18n::set_language(this.ui_language);
                        // 原生托盘菜单不走 GPUI 重绘，只更新文字以保留菜单身份与事件路由。
                        #[cfg(windows)]
                        if let Some(tray) = this.tray.as_mut() {
                            tray.refresh_language();
                        }
                        window.refresh();
                    }
                    cx.notify();
                }).detach();
                *slot = Some(state);
            }
            let state = slot.as_ref().expect("语言选择器已经初始化");
            let selected = if prompt { self.language_preferences.prompt.id() } else { self.language_preferences.ui.id() };
            state.update(cx, |state, cx| {
                // 仅语言切换才刷新显示项，避免每帧 set_items 引起重复通知。
                if self.language_select_locale != Some(language) {
                    state.set_items(options, window, cx);
                }
                if state.selected_value() != Some(&selected) {
                    state.set_selected_value(&selected, window, cx);
                }
            });
        }
        self.language_select_locale = Some(language);
    }

    pub(crate) fn language_settings_group(&self, prompt: bool, cx: &mut Context<Self>) -> SettingGroup {
        let language = self.ui_language;
        let title = template(language, if prompt { MessageKey::PromptLanguage } else { MessageKey::LanguageHeading });
        let description = template(language, if prompt { MessageKey::PromptDescription } else { MessageKey::LanguageDescription });
        let view = cx.entity();
        SettingGroup::new()
            .item(crate::settings_center::settings_group_heading(title, Some(description.into())))
            .item(crate::settings_center::settings_item_row(
                template(language, if prompt { MessageKey::PromptLanguage } else { MessageKey::UiLanguage }), None,
                move |_, _, cx| view.update(cx, |this, _| {
                    let state = if prompt { &this.prompt_language_select } else { &this.ui_language_select };
                    state.as_ref().map(|state| Select::new(state)
                        .accessibility_label(template(this.ui_language, if prompt { MessageKey::PromptLanguage } else { MessageKey::UiLanguage }))
                        .w(px(200.0)).h(px(34.0)).text_size(px(theme::TYPE_BODY))
                        .bg(rgb(theme::WB_INPUT_SURFACE)).border_color(rgb(theme::BORDER_MUTED))
                        .rounded(px(theme::RADIUS_SM)).into_any_element())
                        .unwrap_or_else(|| gpui::div().into_any_element())
                })
            ))
    }
}
