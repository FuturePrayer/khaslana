//! 关于页只展示构建信息与项目入口，不改变更新源或更新渠道偏好。

use gpui::{App, Context, Div, FontWeight, SharedString, Window, div, prelude::*, px};
use gpui_kit::assets::IconName;
use gpui_kit::component::{Icon, Size, Sizable, h_flex, v_flex};
use gpui_kit::component::setting::SettingGroup;

use crate::{RepositoryView, ScrollbarMode, open_url, scrollable_frame_when, update,
    settings_center::{settings_compact_heading, settings_compact_item, settings_compact_list},
    ui::{components::{SettingsButtonTone, settings_content_button},
        theme::{self as ui_theme, rgb}}};

#[cfg(test)]
#[path = "tests/settings_about.rs"]
mod tests;

const NOTES_SCROLL_ID: &str = "about-release-notes";
const PROJECT_URL: &str = "https://github.com/Neglecton/khaslana";
const LICENSE_URL: &str = "https://www.apache.org/licenses/LICENSE-2.0";
const ISSUES_URL: &str = "https://github.com/Neglecton/khaslana/issues";
const DISCUSSIONS_URL: &str = "https://github.com/Neglecton/khaslana/discussions";

#[derive(Clone, Copy)]
struct AboutLink {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    url: &'static str,
    icon: IconName,
}

const PROJECT_LINKS: [AboutLink; 2] = [
    AboutLink { id: "about-github", title: "GitHub 仓库", description: "查看项目源码、提交 Issue 与参与协作", url: PROJECT_URL, icon: IconName::Github },
    AboutLink { id: "about-license", title: "开源协议 Apache-2.0", description: "查看许可证内容与使用条款", url: LICENSE_URL, icon: IconName::FileText },
];
const COMMUNITY_LINKS: [AboutLink; 2] = [
    AboutLink { id: "about-issues", title: "问题反馈", description: "提交 Issue", url: ISSUES_URL, icon: IconName::Info },
    AboutLink { id: "about-discussions", title: "社区讨论", description: "GitHub Discussions", url: DISCUSSIONS_URL, icon: IconName::MessageSquare },
];

/// 宽度来自实际卡片内容区，单位为逻辑像素；不猜测可拖拽侧栏的宽度。
fn about_is_compact(content_width: f32) -> bool {
    !content_width.is_finite() || content_width < 480.0
}

fn release_notes_display(notes: &str) -> &str {
    if notes.trim().is_empty() { "（此版本未附版本说明）" } else { notes }
}

fn platform_display_name(platform: &str) -> &str {
    match platform {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        _ => platform,
    }
}

fn system_description() -> String {
    format!("{} · {}", platform_display_name(std::env::consts::OS), std::env::consts::ARCH)
}

fn caption(text: impl Into<SharedString>) -> Div {
    div().text_size(px(ui_theme::TYPE_META)).line_height(px(16.0))
        .text_color(rgb(ui_theme::CONTENT_SECONDARY)).child(text.into())
}

fn icon(name: IconName, size: f32) -> Icon {
    Icon::new(name).with_size(Size::Size(px(size)))
}

fn circular_icon(name: IconName) -> Div {
    div().flex_none().size(px(36.0)).rounded(px(ui_theme::RADIUS_PILL))
        .bg(rgb(ui_theme::PRIMARY_SUBTLE)).text_color(rgb(ui_theme::PRIMARY))
        .flex().items_center().justify_center().child(icon(name, 18.0))
}

fn card_heading(name: IconName, title: &'static str, description: &'static str) -> Div {
    h_flex().min_w(px(0.0)).gap_3().items_center()
        .child(circular_icon(name))
        .child(settings_compact_heading(title, Some(description.into())).min_w(px(0.0)).flex_1())
}

/// 首帧保守使用单列；实际布局后仅在跨断点时重绘，避免布局测量反馈循环。
fn responsive_card(
    id: &'static str, window: &mut Window, cx: &mut App,
    build: impl FnOnce(bool, &mut App) -> Div,
) -> Div {
    let compact_state = window.use_keyed_state(id, cx, |_, _| true);
    let compact = *compact_state.read(cx);
    div().w_full().min_w(px(0.0)).child(build(compact, cx).w_full().min_w(px(0.0)))
        .on_children_prepainted(move |bounds, _, cx| {
            let Some(bounds) = bounds.first() else { return; };
            let next = about_is_compact(f32::from(bounds.size.width));
            if next != *compact_state.read(cx) {
                let state = compact_state.clone();
                cx.defer(move |cx| state.update(cx, |value, cx| {
                    if *value != next { *value = next; cx.notify(); }
                }));
            }
        })
}

fn external_link(link: AboutLink, detailed: bool, cx: &App) -> impl IntoElement {
    settings_content_button(link.id, format!("{}（在默认浏览器中打开）", link.title),
        if detailed { SettingsButtonTone::Secondary } else { SettingsButtonTone::Quiet }, true, cx)
        .w_full().min_w(px(0.0))
        .when(detailed, |this| this.h_auto().min_h(px(64.0)).py(px(10.0)))
        .when(!detailed, |this| this.h_auto().min_h(px(32.0)).py(px(6.0))
            .px_0().text_color(rgb(ui_theme::PRIMARY)))
        .tooltip(format!("在默认浏览器中打开：{}", link.url))
        .child(h_flex().w_full().min_w(px(0.0)).items_center().gap_2()
            .when(detailed, |this| this.child(circular_icon(link.icon)))
            .child(v_flex().flex_1().min_w(px(0.0)).items_start().gap_1()
                .child(h_flex().min_w(px(0.0)).items_center().gap_1()
                    .child(div().text_size(px(ui_theme::TYPE_BODY)).font_weight(FontWeight::MEDIUM)
                        .when(!detailed, |this| this.text_color(rgb(ui_theme::PRIMARY)))
                        .child(if detailed { link.title } else { link.description }))
                    .child(icon(IconName::ExternalLink, 12.0)))
                .when(detailed, |this| this.child(caption(link.description))))
            .when(detailed, |this| this.child(icon(IconName::ChevronRight, 16.0))))
        // 只绑定 Kit 的点击通道；Enter / Space 由 Kit 转换，不重复添加键盘回调。
        .on_click(move |_, _, _| open_url(link.url))
}

fn info_row(label: &'static str, value: impl Into<SharedString>) -> Div {
    h_flex().w_full().min_w(px(0.0)).flex_wrap().items_center().gap_2()
        .child(caption(label).w(px(56.0)).flex_none())
        .child(div().text_size(px(ui_theme::TYPE_BODY)).text_color(rgb(ui_theme::CONTENT_PRIMARY))
            .child(value.into()))
}

impl RepositoryView {
    pub(crate) fn settings_about_groups(&self, cx: &mut Context<Self>) -> Vec<SettingGroup> {
        let version = format!("Khaslana v{}", env!("CARGO_PKG_VERSION"));
        let channel = update::current_channel();
        let notes = release_notes_display(update::current_release_notes()).to_owned();
        let view = cx.entity();
        let version_card = SettingGroup::new().item(settings_compact_item(
            &["版本", "版本说明", "更新设置", "正式版", "测试版"],
            move |_, window, cx| {
                let version = version.clone();
                let notes = notes.clone();
                responsive_card("about-version-layout", window, cx, |compact, cx| {
                    let badge = div().flex_none().px(px(6.0)).py(px(2.0))
                        .rounded(px(ui_theme::RADIUS_PILL)).text_size(px(ui_theme::TYPE_META))
                        .bg(rgb(if channel == "测试版" { ui_theme::FEEDBACK_WARNING_BG } else { ui_theme::PRIMARY_SUBTLE }))
                        .text_color(rgb(if channel == "测试版" { ui_theme::FEEDBACK_WARNING_TEXT } else { ui_theme::PRIMARY }))
                        .child(channel);
                    let header = div().flex().gap_3().min_w(px(0.0))
                        .when(compact, |this| this.flex_col())
                        .when(!compact, |this| this.items_center().justify_between())
                        .child(card_heading(IconName::Info, "版本", "更新渠道与自动检查可在「更新设置」中配置。").flex_1())
                        .child(h_flex().flex_none().flex_wrap().gap_2().items_center()
                            .child(div().text_size(px(ui_theme::TYPE_BODY)).font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(ui_theme::CONTENT_PRIMARY)).child(version)).child(badge));
                    let notes_body = view.update(cx, |this, cx| {
                        let handle = this.scroll_handle(NOTES_SCROLL_ID);
                        let content = div().id(NOTES_SCROLL_ID).size_full().overflow_y_scroll()
                            .track_scroll(&handle).text_size(px(ui_theme::TYPE_BODY))
                            .line_height(px(18.0)).text_color(rgb(ui_theme::CONTENT_PRIMARY))
                            .child(crate::markdown_view::render_markdown(&notes));
                        v_flex().w_full().h(px(220.0)).min_h(px(0.0)).overflow_hidden()
                            .child(scrollable_frame_when(NOTES_SCROLL_ID, ScrollbarMode::Vertical,
                                content.into_any_element(), handle, true, cx))
                    });
                    settings_compact_list().gap_3().child(header).child(notes_body)
                })
            },
        ));
        let project_card = SettingGroup::new().item(settings_compact_item(
            &["项目链接", "GitHub", "开源协议", "Apache-2.0"],
            |_, window, cx| responsive_card("about-project-layout", window, cx, |compact, cx| {
                settings_compact_list().gap_3()
                    .child(card_heading(IconName::Link, "项目链接", "开源项目，欢迎 Star 与参与贡献。"))
                    .child(div().flex().gap_3().min_w(px(0.0))
                        .when(compact, |this| this.flex_col())
                        .children(PROJECT_LINKS.into_iter().map(|link| {
                            div().min_w(px(0.0)).when(compact, |this| this.w_full())
                                .when(!compact, |this| this.flex_1())
                                .child(external_link(link, true, cx))
                        })))
            }),
        ));
        let info_card = SettingGroup::new().item(settings_compact_item(
            &["其他信息", "系统", "架构", "运行环境", "问题反馈", "社区讨论"],
            |_, window, cx| responsive_card("about-info-layout", window, cx, |compact, cx| {
                settings_compact_list().gap_3()
                    .child(h_flex().min_w(px(0.0)).items_center().gap_3()
                        .child(circular_icon(IconName::Info))
                        .child(settings_compact_heading("其他信息", None)))
                    .child(div().flex().gap_3().min_w(px(0.0))
                        .when(compact, |this| this.flex_col())
                        .child(settings_compact_list().min_w(px(0.0))
                            .when(!compact, |this| this.flex_1())
                            .child(info_row("当前版本", format!("v{}", env!("CARGO_PKG_VERSION"))))
                            .child(info_row("发布渠道", update::current_channel()))
                            .child(info_row("运行环境", system_description())))
                        .child(v_flex().min_w(px(0.0)).gap_1()
                            .when(!compact, |this| this.flex_1())
                            .children(COMMUNITY_LINKS.into_iter().map(|link| {
                                h_flex().w_full().min_w(px(0.0)).flex_wrap().items_center().gap_2()
                                    .child(caption(link.title).w(px(56.0)).flex_none())
                                    .child(div().flex_1().min_w(px(0.0))
                                        .child(external_link(link, false, cx)))
                            }))))
            }),
        ));
        vec![version_card, project_card, info_card]
    }
}
