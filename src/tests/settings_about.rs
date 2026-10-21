use super::*;

#[test]
fn responsive_layout_uses_content_logical_pixels_and_exact_boundary() {
    for width in [0.0, 180.0, 320.0, 479.99] {
        assert!(about_is_compact(width), "内容宽度 {width} 应为单列");
    }
    for width in [480.0, 600.0, 900.0] {
        assert!(!about_is_compact(width), "内容宽度 {width} 应为双列");
    }
    // DPI 不参与断点计算：传入同一逻辑宽度，100%–200% 的结果一致。
    for scale in [1.0, 1.25, 1.5, 2.0] {
        assert!(about_is_compact(400.0 * scale / scale));
        assert!(!about_is_compact(520.0 * scale / scale));
    }
}

#[test]
fn invalid_layout_measurements_fall_back_to_single_column() {
    for width in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(about_is_compact(width));
    }
}

#[test]
fn project_destinations_and_interaction_ids_are_fixed_and_distinct() {
    let links: Vec<_> = PROJECT_LINKS.into_iter().chain(COMMUNITY_LINKS).collect();
    assert_eq!(links.iter().map(|link| link.url).collect::<Vec<_>>(), vec![
        "https://github.com/Neglecton/khaslana",
        "https://www.apache.org/licenses/LICENSE-2.0",
        "https://github.com/Neglecton/khaslana/issues",
        "https://github.com/Neglecton/khaslana/discussions",
    ]);
    let ids: std::collections::HashSet<_> = links.iter().map(|link| link.id).collect();
    assert_eq!(ids.len(), links.len());
    for link in links {
        assert!(!link.title.zh.is_empty());
        assert!(!link.description.zh.is_empty());
        assert!(link.url.starts_with("https://"));
        assert!(!link.url.contains(['"', '\r', '\n', '\t']));
    }
}

#[test]
fn project_and_community_links_keep_approved_titles_and_visible_values() {
    assert_eq!(PROJECT_LINKS[0].description.zh, "查看项目源码、提交 Issue 与参与协作");
    assert_eq!(PROJECT_LINKS[0].icon, IconName::Github);
    assert_eq!(PROJECT_LINKS[1].title.zh, "开源协议 Apache-2.0");
    assert_eq!(PROJECT_LINKS[1].description.zh, "查看许可证内容与使用条款");
    assert_eq!(COMMUNITY_LINKS.map(|link| (link.title.zh, link.description.zh)), [
        ("问题反馈", "提交 Issue"),
        ("社区讨论", "GitHub Discussions"),
    ]);
}

#[test]
fn release_notes_keep_markdown_and_only_replace_blank_content() {
    let markdown = "## 版本说明\n\n- **新增**功能\n";
    assert_eq!(release_notes_display(markdown), markdown);
    for blank in ["", " \n\t"] {
        assert_eq!(release_notes_display(blank), text(messages::NO_RELEASE_NOTES));
        assert_eq!(khaslana::ui_i18n::text_in(khaslana::language::Language::ZhCn, messages::NO_RELEASE_NOTES), "（此版本未附版本说明）");
        assert_eq!(khaslana::ui_i18n::text_in(khaslana::language::Language::En, messages::NO_RELEASE_NOTES), "(No release notes included for this version)");
    }
}

#[test]
fn system_information_uses_build_target_instead_of_hardcoded_windows_x64() {
    assert_eq!(system_description(), format!("{} · {}", platform_display_name(std::env::consts::OS), std::env::consts::ARCH));
    assert_eq!(platform_display_name("windows"), "Windows");
    assert_eq!(platform_display_name("macos"), "macOS");
    assert_eq!(platform_display_name("linux"), "Linux");
    assert_eq!(platform_display_name("freebsd"), "freebsd");
}

#[test]
fn about_link_resources_have_unique_keys_and_both_languages() {
    let mut keys = std::collections::HashSet::new();
    for link in PROJECT_LINKS.into_iter().chain(COMMUNITY_LINKS) {
        for message in [link.title, link.description] {
            assert!(keys.insert(message.key), "重复消息身份：{}", message.key);
            assert!(!message.zh.is_empty());
            assert!(!message.en.is_empty());
        }
    }
}
