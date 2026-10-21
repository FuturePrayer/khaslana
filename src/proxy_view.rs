#[path = "ui_resources/proxy_view_messages.rs"]
mod messages;

use gpui::{Context, IntoElement, Window, div, prelude::*, px};
use khaslana::NetworkProxyMode;

use gpui_kit::component::setting::{SettingGroup, SettingItem};

use crate::ui::{components::tooltip_text, theme::rgb};
use crate::{
    FieldId, RepositoryView,
    ui::{
        components::{dialog_actions, segmented_button},
        theme as ui_theme,
    },
};

impl RepositoryView {
    /// 网络代理页的分组（Kit `SettingGroup` 列表）。
    ///
    /// 「自定义地址」只在自定义代理模式下存在，分组数量随模式变化：Kit 的
    /// `SettingsFilter::selected_index` 会自动清掉不存在分组的选中态，
    /// 侧栏子导航与内容区同步增减，不会出现悬空高亮。
    pub(crate) fn settings_proxy_groups(
        &self,
        _window: &Window,
        cx: &mut Context<Self>,
    ) -> Vec<SettingGroup> {
        let view = cx.entity();
        let custom_enabled = self.proxy_mode == NetworkProxyMode::Custom;
        let remote_label = self
            .current_remote()
            .map(|remote| khaslana::ui_i18n::format(messages::THE_TEST_WILL_CONNECT_TO_THE_CURRENT_REMOTE_REMOTE, &[("remote", &(remote).to_string())]))
            .unwrap_or_else(|| khaslana::ui_i18n::text(messages::OPEN_A_REPOSITORY_WITH_A_REMOTE_BEFORE_TESTING_THE_PROXY).to_string());

        let mut groups = vec![SettingGroup::new()
            .item(crate::settings_center::settings_group_heading(
                khaslana::ui_i18n::text(messages::PROXY_MODE),
                Some(proxy_mode_help(self.proxy_mode).into()),
            ))
            .item(crate::settings_center::settings_item_row(
                khaslana::ui_i18n::text(messages::MODE),
                None,
                move |_options, _window, cx| {
                    view.update(cx, |this, cx| {
                        crate::ui::components::settings_segmented_group()
                            .child(this.proxy_mode_button(khaslana::ui_i18n::text(messages::NO_PROXY), NetworkProxyMode::Disabled, cx))
                            .child(this.proxy_mode_button(khaslana::ui_i18n::text(messages::USE_SYSTEM_PROXY), NetworkProxyMode::System, cx))
                            .child(this.proxy_mode_button(khaslana::ui_i18n::text(messages::CUSTOM_PROXY), NetworkProxyMode::Custom, cx))
                    })
                },
            ))];

        if custom_enabled {
            groups.push(
                SettingGroup::new()
                    .item(crate::settings_center::settings_group_heading(
                        khaslana::ui_i18n::text(messages::CUSTOM_ADDRESSES),
                        Some(khaslana::ui_i18n::text(messages::FOR_NOW_INCLUDE_PROXY_AUTHENTICATION_IN_THE_URL_FOR_EXAMPLE).into()),
                    ))
                    .item({
                        let view = cx.entity();
                        crate::settings_center::settings_item_row(
                            khaslana::ui_i18n::text(messages::HTTP_PROXY),
                            None,
                            move |_options, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.input(FieldId::ProxyHttpUrl, false, window, cx)
                                })
                            },
                        )
                    })
                    .item({
                        let view = cx.entity();
                        crate::settings_center::settings_item_row(
                            khaslana::ui_i18n::text(messages::HTTPS_PROXY),
                            None,
                            move |_options, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.input(FieldId::ProxyHttpsUrl, false, window, cx)
                                })
                            },
                        )
                    })
                    .item({
                        let view = cx.entity();
                        crate::settings_center::settings_item_row(
                            khaslana::ui_i18n::text(messages::SOCKS5_PROXY),
                            None,
                            move |_options, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.input(FieldId::ProxySocks5Url, false, window, cx)
                                })
                            },
                        )
                    }),
            );
        }

        groups.push(
            SettingGroup::new()
                .item(crate::settings_center::settings_group_heading(
                    khaslana::ui_i18n::text(messages::CONNECTION_TEST),
                    Some(khaslana::ui_i18n::text(messages::TEST_THE_PROXY_CONNECTION_BEFORE_SAVING).into()),
                ))
                .item({
                    let view = cx.entity();
                    crate::settings_center::settings_item_row(
                        khaslana::ui_i18n::text(messages::CURRENT_REMOTE),
                        None,
                        move |_options, _window, cx| {
                            view.update(cx, |_this, _cx| {
                                div()
                                    .text_size(px(ui_theme::TYPE_BODY))
                                    .line_height(px(18.0))
                                    .text_color(rgb(ui_theme::CONTENT_SECONDARY))
                                    .child(remote_label.clone())
                            })
                        },
                    )
                })
                .item({
                    let view = cx.entity();
                    SettingItem::render(move |_options, _window, cx| {
                        view.update(cx, |this, cx| {
                            dialog_actions()
                                .child(this.button(
                                    khaslana::ui_i18n::text(messages::TEST_PROXY),
                                    !this.busy,
                                    |this, _, _| this.test_network_proxy_settings(),
                                    cx,
                                ))
                                .child(this.primary_button(
                                    khaslana::ui_i18n::text(messages::SAVE),
                                    !this.busy,
                                    |this, _, cx| {
                                        this.save_network_proxy_settings();
                                        this.notify_settings_save(khaslana::ui_i18n::text(messages::PROXY_SETTINGS_SAVED), cx);
                                    },
                                    cx,
                                ))
                        })
                    }).keywords([khaslana::ui_i18n::text(messages::TEST_PROXY), khaslana::ui_i18n::text(messages::SAVE)])
                }),
        );

        groups
    }

    fn proxy_mode_button(
        &self,
        label: &'static str,
        mode: NetworkProxyMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.proxy_mode == mode;
        let enabled = !self.busy;
        let disabled_reason = proxy_mode_disabled_reason(enabled);
        segmented_button(format!("proxy-mode-{label}"), selected, enabled)
            .when_some(disabled_reason, |this, reason| {
                this.tooltip(move |_window, cx| tooltip_text(reason, cx))
            })
            .on_click(cx.listener(move |this, _event, _window, cx| {
                if enabled && !this.busy {
                    this.set_proxy_mode(mode);
                    cx.notify();
                }
            }))
            .child(label)
    }
}

fn proxy_mode_disabled_reason(enabled: bool) -> Option<&'static str> {
    (!enabled).then_some(khaslana::ui_i18n::text(messages::AN_OPERATION_IS_IN_PROGRESS_PLEASE_WAIT))
}

fn proxy_mode_help(mode: NetworkProxyMode) -> &'static str {
    match mode {
        NetworkProxyMode::Disabled => khaslana::ui_i18n::text(messages::GIT_NETWORK_OPERATIONS_CONNECT_DIRECTLY_IGNORING_GIT_CONFIGURATION_AND_ENVIRONMENT),
        NetworkProxyMode::System => {
            khaslana::ui_i18n::text(messages::USE_LIBGIT2_AUTOMATIC_PROXY_SELECTION_GIT_PROXY_CONFIGURATION_FIRST_THEN)
        }
        NetworkProxyMode::Custom => {
            khaslana::ui_i18n::text(messages::SELECT_A_CUSTOM_PROXY_BY_REMOTE_PROTOCOL_HTTP_HTTPS_REMOTES)
        }
    }
}

#[cfg(test)]
#[path = "tests/proxy_view.rs"]
mod tests;
