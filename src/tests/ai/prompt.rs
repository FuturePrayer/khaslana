use super::*;

#[test]
fn all_prompt_language_pairs_preserve_payload_and_contracts() {
    let payload = "用户原文 Keep ${target} 中文路径/README.md";
    for system in [Language::ZhCn, Language::En] {
        for output in [Language::ZhCn, Language::En] {
            let languages = TaskLanguages { system, output };
            let (commit, user) = commit_message_prompts_with_languages(payload, Some(payload), languages);
            assert!(commit.content.ends_with(languages.output_instruction()));
            assert!(user.content.contains(payload));
            assert_eq!(commit.role, ChatRole::System);
            assert_eq!(user.role, ChatRole::User);
            let (merge, user) = conflict_merge_prompts_with_languages("中文.rs", payload, Some((2, 3)), languages);
            assert!(merge.content.ends_with(languages.output_instruction()));
            assert!(merge.content.contains("<<<<<<<"));
            assert!(user.content.contains(payload));
            assert!(user.content.contains("2/3"));
            for version in [WorkflowPromptVersion::V1, WorkflowPromptVersion::V2] {
                let (workflow, user) = workflow_template_prompts_with_languages(payload, Some(payload), version, languages);
                assert!(workflow.content.ends_with(languages.output_instruction()));
                assert!(user.content.contains(payload));
                assert!(workflow.content.contains("filterBranches"));
                let expected = match (system, version) {
                    (Language::ZhCn, WorkflowPromptVersion::V1) => "version 字段恒为 1",
                    (Language::ZhCn, WorkflowPromptVersion::V2) => "version 字段恒为 2",
                    (Language::En, WorkflowPromptVersion::V1) => "version must be 1",
                    (Language::En, WorkflowPromptVersion::V2) => "version must be 2",
                };
                assert!(workflow.content.contains(expected));
                if system == Language::En {
                    assert!(!workflow.content.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)));
                }
            }
            if system == Language::En {
                assert!(commit.content.starts_with("You are"));
                assert!(merge.content.starts_with("You are"));
            }
        }
    }
}

#[test]
fn segmented_merge_keeps_language_snapshot_and_assistant_history_verbatim() {
    for system in [Language::ZhCn, Language::En] {
        for output in [Language::ZhCn, Language::En] {
            let languages = TaskLanguages { system, output };
            let (first_system, first) = conflict_merge_prompts_with_languages("中文.rs", "段一\n", Some((1, 2)), languages);
            let (second_system, second) = conflict_merge_prompts_with_languages("中文.rs", "段二\n", Some((2, 2)), languages);
            assert_eq!(first_system, second_system);
            let history = vec![(first, ChatMessage { role: ChatRole::Assistant, content: "原样 resolved\n".into() })];
            let messages = crate::ai::merge::build_segment_messages(second_system, &history, second, 20_000);
            assert!(messages[0].content.ends_with(languages.output_instruction()));
            assert_eq!(messages[2].content, "原样 resolved\n");
            assert!(messages[3].content.contains("段二\n"));
        }
    }
}

#[test]
fn english_workflow_reference_covers_all_actions_and_expression_methods() {
    for token in ["checkout", "fetch", "pull", "createBranch", "guardRemoteBranch", "merge", "push",
        "ensureClean", "assertBranch", "filterBranches", "deleteBranches", "mcp.call", "skill.run", "js.run",
        "requireCleanWorktree", "browserGuard", "autoDiscover", "cachedTools", "saveAs", "dateGroup",
        "olderThanMonths", "dryRun", "skipCurrent", "run.sourceKind", "git.initialBranch",
        "compact", "nth:index", "replace:from:to", "slug", "suffix:n", "alt"] {
        assert!(WORKFLOW_DOC_EN.contains(token), "missing English reference token: {token}");
    }
}

#[test]
fn commit_message_prompts_contains_diff_and_system_rules() {
    let (system, user) = commit_message_prompts("diff --git a/x b/x", None);
    assert_eq!(system.role, ChatRole::System);
    assert!(system.content.contains("提交信息"));
    assert_eq!(user.role, ChatRole::User);
    assert!(user.content.contains("diff --git a/x b/x"));
}

#[test]
fn commit_message_prompts_includes_hint_when_provided() {
    let (_system, user) = commit_message_prompts("diff", Some("只看接口"));
    assert!(user.content.contains("补充要求"));
    assert!(user.content.contains("只看接口"));
}

#[test]
fn commit_message_prompts_omits_hint_section_when_empty() {
    let (_system, user) = commit_message_prompts("diff", Some("   "));
    assert!(!user.content.contains("补充要求"));
}

#[test]
fn commit_message_prompts_truncates_long_diff() {
    let long = "a".repeat(COMMIT_DIFF_TEXT_LIMIT * 2);
    let (_system, user) = commit_message_prompts(&long, None);
    assert!(user.content.contains("已截断"));
}

#[test]
fn truncate_text_keeps_short_text_intact() {
    assert_eq!(truncate_text("hello", 10), "hello");
}

#[test]
fn truncate_text_appends_notice_when_too_long() {
    let result = truncate_text("abcdefgh", 3);
    assert!(result.starts_with("abc"));
    assert!(result.contains("已截断"));
}

#[test]
fn conflict_merge_prompts_whole_file_mode() {
    let (system, user) = conflict_merge_prompts("src/lib.rs", "<<<<<<< OURS\nx\n", None);
    assert_eq!(system.role, ChatRole::System);
    // 系统提示说明标记格式与输出约束（不得残留冲突标记、非冲突行逐字保留）。
    assert!(system.content.contains("diff3"));
    assert!(system.content.contains("冲突标记"));
    assert!(system.content.contains("逐字保留"));
    assert_eq!(user.role, ChatRole::User);
    assert!(user.content.contains("src/lib.rs"));
    assert!(user.content.contains("完整内容"));
    // 整文件模式不出现分段说明。
    assert!(!user.content.contains("第 "));
    assert!(user.content.contains("<<<<<<< OURS\nx"));
}

#[test]
fn conflict_merge_prompts_segment_mode_marks_position() {
    let (_system, user) = conflict_merge_prompts("a.txt", "block body", Some((2, 5)));
    assert!(user.content.contains("第 2/5 段"));
    assert!(user.content.contains("只输出这一段"));
    assert!(user.content.contains("block body"));
}

#[test]
fn workflow_template_prompts_embeds_doc_and_request() {
    let (system, user) = workflow_template_prompts("基于 master 创建 release 分支并推送", None);
    assert_eq!(system.role, ChatRole::System);
    // 文档关键章节嵌入（AI 参考工作流使用文档）
    assert!(system.content.contains("支持的步骤"));
    assert!(system.content.contains("filterBranches"));
    assert!(system.content.contains("version 字段恒为 1"));
    assert_eq!(user.role, ChatRole::User);
    // 新建模式：不含现有模板标记，含需求
    assert!(!user.content.contains("【现有模板】"));
    assert!(user.content.contains("基于 master 创建 release 分支并推送"));
}

#[test]
fn workflow_template_prompts_edit_mode_includes_current_definition() {
    let current = r#"{ version: 1, name: "旧模板", steps: [{ op: "ensureClean" }] }"#;
    let (_system, user) = workflow_template_prompts("加一个推送步骤", Some(current));
    assert!(user.content.contains("【现有模板】"));
    assert!(user.content.contains("旧模板"));
    assert!(user.content.contains("加一个推送步骤"));
}

#[test]
fn workflow_template_prompts_trims_and_rejects_empty_request() {
    // 空白需求 trim 后仍生成 prompt（守卫在 UI 层做），但内容不含空白原文
    let (_system, user) = workflow_template_prompts("   ", None);
    assert!(user.content.contains("【功能需求】"));
}
