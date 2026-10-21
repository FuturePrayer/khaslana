use super::*;

use crate::types::{BlameCommitInfo, ChangeState};
use crate::git::test_support::git_test_support as git_support;
use serde_json::{Value, json};

// fixture 只绑定回环随机端口，记录真实业务入口发出的请求；等待有界，失败不挂住测试。
fn review_http_fixture(
    responses: Vec<String>,
) -> (String, std::thread::JoinHandle<Vec<Value>>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let mut requests = Vec::new();
        for response in responses {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "未收到预期的评审 HTTP 请求");
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(err) => panic!("评审 fixture 接收失败：{err}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut bytes = Vec::new();
            let header_end = loop {
                let mut buffer = [0; 4096];
                let count = stream.read(&mut buffer).unwrap();
                assert_ne!(count, 0, "评审请求头提前结束");
                bytes.extend_from_slice(&buffer[..count]);
                if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    break end + 4;
                }
                assert!(bytes.len() < 64 * 1024, "评审请求头过大");
            };
            let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
            let length = headers.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().unwrap())
            }).expect("评审请求应发送 Content-Length");
            assert!(length < 1024 * 1024);
            while bytes.len() < header_end + length {
                let mut buffer = [0; 4096];
                let count = stream.read(&mut buffer).unwrap();
                assert_ne!(count, 0, "评审请求正文提前结束");
                bytes.extend_from_slice(&buffer[..count]);
            }
            requests.push(serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap());
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.len(), response).unwrap();
        }
        requests
    });
    (base_url, server)
}

fn run_fixture_review(settings: crate::ai::config::AiProviderSettings) -> AiReviewResult {
    run_fixture_review_in(settings, TaskLanguages::default())
}

fn run_fixture_review_in(settings: crate::ai::config::AiProviderSettings, languages: TaskLanguages) -> AiReviewResult {
    let (dir, repo, service) = git_support::init_repo();
    git_support::write_file(dir.path(), "review.txt", "需要评审的内容\n");
    let oid = git_support::commit_all(&repo, "评审测试初始提交");
    let input = ReviewAgentInput {
        repo_path: dir.path().to_path_buf(),
        target_display_name: "feature/review-fixture".into(),
        target_commit_oid: oid.to_string(),
        compare_files: Vec::new(),
    };
    let client = ChatClient::new(settings, None);
    let mut completed = Vec::new();
    let result = run_review_agent_with_languages(
        &input, &service, &client,
        &std::sync::atomic::AtomicBool::new(false),
        &mut |event| {
            if let AgentEvent::Done(result) = event { completed.push(result.content); }
        },
        languages,
    ).unwrap().expect("评审应完成而非取消");
    assert_eq!(completed, vec![result.content.clone()]);
    result
}

#[test]
fn review_tool_schemas_change_descriptions_only_not_protocol_contracts() {
    fn erase_descriptions(value: &mut Value) {
        match value {
            Value::Object(object) => {
                object.remove("description");
                for value in object.values_mut() { erase_descriptions(value); }
            }
            Value::Array(values) => for value in values { erase_descriptions(value); },
            _ => {}
        }
    }
    let zh = tool_schemas_in(Language::ZhCn);
    let en = tool_schemas_in(Language::En);
    assert_eq!(zh.len(), en.len());
    for (zh, en) in zh.iter().zip(&en) {
        assert_eq!(zh.name, en.name);
        assert_ne!(zh.description, en.description);
        assert!(!en.description.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c)));
        let mut zh_params = zh.parameters.clone();
        let mut en_params = en.parameters.clone();
        erase_descriptions(&mut zh_params);
        erase_descriptions(&mut en_params);
        assert_eq!(zh_params, en_params);
    }
}

#[test]
fn review_retry_keeps_frozen_language_messages_and_contract() {
    let languages = TaskLanguages { system: Language::En, output: Language::ZhCn };
    let partial = format!("data: {}\n\n", json!({
        "choices":[{"index":0,"delta":{"content":"partial must be discarded"},"finish_reason":null}]
    }));
    let complete = format!("data: {}\n\ndata: [DONE]\n\n", json!({
        "choices":[{"index":0,"delta":{"content":"最终结果"},"finish_reason":"stop"}]
    }));
    let (base_url, server) = review_http_fixture(vec![partial, complete]);
    let result = run_fixture_review_in(crate::ai::config::AiProviderSettings {
        enabled: true, base_url, model: "retry-language-fixture".into(), request_timeout_secs: 5,
        ..Default::default()
    }, languages);
    assert_eq!(result.content, "最终结果");
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["messages"], requests[1]["messages"]);
    assert_eq!(requests[0]["tools"], requests[1]["tools"]);
    assert!(requests[1]["messages"][0]["content"].as_str().unwrap().ends_with(languages.output_instruction()));
}

#[test]
fn review_agent_four_language_pairs_survive_real_tool_round_trip() {
    for system in [Language::ZhCn, Language::En] {
        for output in [Language::ZhCn, Language::En] {
            let languages = TaskLanguages { system, output };
            let event = |delta: Value| format!("data: {}\n\ndata: [DONE]\n\n", json!({
                "choices":[{"index":0,"delta":delta,"finish_reason":"stop"}]
            }));
            let (base_url, server) = review_http_fixture(vec![
                event(json!({"tool_calls":[{"index":0,"id":"call_language","type":"function","function":{"name":"get_file_tree","arguments":"{}"}}]})),
                event(json!({"content":"原样 result"})),
            ]);
            let result = run_fixture_review_in(crate::ai::config::AiProviderSettings {
                enabled: true, base_url, model: "language-fixture".into(), request_timeout_secs: 5,
                ..Default::default()
            }, languages);
            assert_eq!(result.content, "原样 result");
            let requests = server.join().unwrap();
            assert_eq!(requests.len(), 2);
            for request in &requests {
                let system_text = request["messages"][0]["content"].as_str().unwrap();
                assert!(system_text.ends_with(languages.output_instruction()));
                assert_eq!(system_text.starts_with("You are"), system == Language::En);
                assert_eq!(request["tools"][0]["function"]["name"], "read_lines");
            }
            assert_eq!(requests[0]["messages"][0], requests[1]["messages"][0]);
            assert_eq!(requests[1]["messages"][3]["tool_call_id"], "call_language");
            assert!(requests[1]["messages"][3]["content"].as_str().unwrap().contains("review.txt"));
        }
    }
}

#[test]
fn review_agent_sends_custom_generation_settings_and_completes() {
    let response = format!("data: {}\n\ndata: [DONE]\n\n", json!({
        "choices": [{"index":0,"delta":{"content":"评审完成，无严重问题。"},"finish_reason":"stop"}]
    }));
    let (base_url, server) = review_http_fixture(vec![response]);
    let result = run_fixture_review(crate::ai::config::AiProviderSettings {
        enabled: true, base_url, model: "review-fixture-model".into(),
        max_tokens: 12_345, temperature: 0.75,
        reasoning_effort: Some("high".into()), request_timeout_secs: 5,
        ..Default::default()
    });
    assert_eq!(result.content, "评审完成，无严重问题。");
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request["max_tokens"], 12_345);
    assert_eq!(request["temperature"], 0.75);
    assert_eq!(request["reasoning_effort"], "high");
    assert_eq!(request["model"], "review-fixture-model");
    assert_eq!(request["stream"], true);
    assert!(request["tools"].as_array().is_some_and(|tools| !tools.is_empty()));
    assert!(request["messages"][1]["content"].as_str().unwrap().contains("feature/review-fixture"));
}

#[test]
fn review_agent_responses_replays_opaque_history_after_real_repository_tool() {
    let output = json!([
        {"type":"reasoning","id":"rs_review","summary":[{"type":"summary_text","text":"先查看目标提交的文件树"}],"encrypted_content":"opaque-review-state"},
        {"type":"function_call","id":"fc_review","call_id":"call_review_tree","name":"get_file_tree","arguments":"{}","status":"completed"}
    ]);
    let completed = |output: Value| format!("event: response.completed\ndata: {}\n\n", json!({
        "type":"response.completed", "response":{"id":"resp_review","status":"completed","output":output}
    }));
    let (base_url, server) = review_http_fixture(vec![
        completed(output.clone()),
        completed(json!([{"type":"message","role":"assistant","status":"completed","content":[{"type":"output_text","text":"已核对文件树，评审完成。"}]}])),
    ]);
    let result = run_fixture_review(crate::ai::config::AiProviderSettings {
        enabled: true, api_type: crate::ai::config::AiApiType::Responses,
        base_url, model: "responses-review-fixture".into(),
        max_tokens: 23_456, temperature: 0.5,
        reasoning_effort: Some("medium".into()), request_timeout_secs: 5,
        ..Default::default()
    });
    assert_eq!(result.content, "已核对文件树，评审完成。");
    assert!(result.steps.iter().any(|step| matches!(step,
        AiReviewStep::ToolCall { name, result_excerpt, error: false, .. }
        if name == "get_file_tree" && result_excerpt.contains("review.txt")
    )));
    let requests = server.join().unwrap();
    assert_eq!(requests.len(), 2);
    for request in &requests {
        assert_eq!(request["max_output_tokens"], 23_456);
        assert_eq!(request["temperature"], 0.5);
        assert_eq!(request["reasoning"]["effort"], "medium");
        assert_eq!(request["stream"], true);
    }
    let replay = requests[1]["input"].as_array().unwrap();
    // 必须由业务循环携带完整原生项，而不只是重建可见文本和工具调用。
    assert_eq!(replay[2], output[0]);
    assert_eq!(replay[3], output[1]);
    assert_eq!(replay[4]["type"], "function_call_output");
    assert_eq!(replay[4]["call_id"], "call_review_tree");
    assert!(replay[4]["output"].as_str().unwrap().contains("review.txt"));
}

fn blame_commit() -> BlameCommitInfo {
    BlameCommitInfo {
        oid: "0123456789abcdef0123456789abcdef01234567".into(),
        short_oid: "0123456".into(),
        author: "张三".into(),
        time: 1_755_000_000,
        summary: "初始提交".into(),
    }
}

#[test]
fn clamp_line_range_defaults_and_window() {
    // 默认整个文件。
    assert_eq!(clamp_line_range(100, None, None), (1, 100));
    // 指定范围钳制在文件内。
    assert_eq!(clamp_line_range(100, Some(10), Some(30)), (10, 30));
    // end < start 抬到 start。
    assert_eq!(clamp_line_range(100, Some(50), Some(20)), (50, 50));
    // 超出文件的 start/end 钳到边界。
    assert_eq!(clamp_line_range(100, Some(0), Some(999)), (1, 100));
    // 窗口最宽 400 行。
    assert_eq!(clamp_line_range(1000, None, None), (1, 400));
    assert_eq!(clamp_line_range(1000, Some(900), Some(999)), (900, 999));
    assert_eq!(clamp_line_range(1000, Some(900), None), (900, 1000));
    // 空文件。
    assert_eq!(clamp_line_range(0, None, None), (1, 0));
}

#[test]
fn truncate_result_chars_annotates_only_when_over() {
    let short = "abc".repeat(10);
    assert_eq!(truncate_result_chars(&short, 100), short);

    let long = "x".repeat(300);
    let truncated = truncate_result_chars(&long, 100);
    assert!(truncated.starts_with(&"x".repeat(100)));
    assert!(truncated.contains("（结果过长，已截断）"));
}

#[test]
fn initial_prompt_full_diffs_within_budget() {
    let entries = vec![
        InitialDiffEntry {
            path: "src/lib.rs".into(),
            status_label: "M",
            patch: "+modified\n".into(),
        },
        InitialDiffEntry {
            path: "docs.md".into(),
            status_label: "A",
            patch: "+added line\n".into(),
        },
    ];
    let prompt = assemble_initial_user_prompt("feature/x", &entries);
    assert!(prompt.contains("目标分支：feature/x"));
    assert!(prompt.contains("变更文件（2 个）："));
    assert!(prompt.contains("- [M] src/lib.rs"));
    assert!(prompt.contains("- [A] docs.md"));
    // 预算内：diff 原样保留，无截断标注。
    assert!(prompt.contains("+modified\n"));
    assert!(!prompt.contains("已截断"));
}

#[test]
fn initial_prompt_truncates_per_file_over_budget() {
    let big_patch = "x".repeat(20_000);
    let entries = vec![
        InitialDiffEntry {
            path: "big1.rs".into(),
            status_label: "M",
            patch: big_patch.clone(),
        },
        InitialDiffEntry {
            path: "big2.rs".into(),
            status_label: "M",
            patch: big_patch,
        },
    ];
    let prompt = assemble_initial_user_prompt("feature/x", &entries);
    // 总量 40K > 30K 预算：进入逐文件截断模式。
    assert!(prompt.contains("已截断"));
    // 单文件保留 INITIAL_PER_FILE_CHARS 字符 + 标注。
    assert!(prompt.contains(&"x".repeat(INITIAL_PER_FILE_CHARS)));
    assert!(prompt.matches(&"x".repeat(INITIAL_PER_FILE_CHARS)).count() >= 2);
}

#[test]
fn initial_prompt_total_capped_in_truncation_mode() {
    // 200 个文件 × 20K：旧实现总量 = 200 × 4K = 800K 撑爆上下文；
    // 现在截断模式下总量仍受 INITIAL_DIFF_BUDGET_CHARS 二次封顶。
    let big_patch = "x".repeat(20_000);
    let entries: Vec<InitialDiffEntry> = (0..200)
        .map(|index| InitialDiffEntry {
            path: format!("file{index}.rs"),
            status_label: "M",
            patch: big_patch.clone(),
        })
        .collect();
    let prompt = assemble_initial_user_prompt("feature/x", &entries);
    // x 只来自 diff 本体；头部与截断标注现在一并计入预算扣减，
    // 总量严格不超过预算（不再为标注留每文件 20 字符余量）。
    let x_count = prompt.matches('x').count();
    assert!(
        x_count <= INITIAL_DIFF_BUDGET_CHARS,
        "x 字符总量 {x_count} 超出二次封顶"
    );
    // 预算耗尽后的文件降级为「仅清单 + read_diff 引导」。
    assert!(prompt.contains("差异未附：初始预算已用尽，请用 read_diff 获取"));
}

#[test]
fn tool_budget_force_finish_at_each_limit() {
    let mut budget = ToolBudget::default();
    assert!(!budget.force_finish());

    budget.rounds = MAX_TOOL_ROUNDS;
    assert!(budget.force_finish());

    budget.rounds = 0;
    budget.calls = MAX_TOOL_CALLS_TOTAL;
    assert!(budget.force_finish());

    budget.calls = 0;
    budget.note_result(MAX_TOTAL_TOOL_RESULT_CHARS);
    assert!(budget.force_finish());

    // 记账饱和不 panic。
    budget.note_result(usize::MAX);
    assert!(budget.result_chars >= MAX_TOTAL_TOOL_RESULT_CHARS);
}

#[test]
fn tool_budget_limit_reason_names_tripped_limit() {
    // 未触顶没有原因。
    let mut budget = ToolBudget::default();
    assert_eq!(budget.limit_reason(), None);

    // 三条线各自命名，首个触顶的优先（轮次 > 次数 > 体积）。
    budget.rounds = MAX_TOOL_ROUNDS;
    assert_eq!(budget.limit_reason(), Some("工具调用轮次上限"));

    budget.calls = MAX_TOOL_CALLS_TOTAL;
    assert_eq!(budget.limit_reason(), Some("工具调用轮次上限"));

    budget.rounds = 0;
    assert_eq!(budget.limit_reason(), Some("工具调用次数上限"));

    budget.calls = 0;
    budget.note_result(MAX_TOTAL_TOOL_RESULT_CHARS);
    assert_eq!(budget.limit_reason(), Some("工具结果累计体积上限"));
}

#[test]
fn blame_hunks_format_committed_and_uncommitted() {
    let hunks = vec![
        BlameHunkInfo {
            commit: Some(blame_commit()),
            start_line: 1,
            line_count: 3,
        },
        BlameHunkInfo {
            commit: None,
            start_line: 4,
            line_count: 2,
        },
    ];
    let text = format_blame_hunks(&hunks);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("L1-L3 0123456 张三 "));
    assert!(lines[0].ends_with(" 初始提交"));
    assert_eq!(lines[1], "L4-L5 （工作区未提交）");
}

#[test]
fn search_hits_and_history_format() {
    let hits = vec![CodeSearchMatch {
        path: "src/lib.rs".into(),
        lineno: 12,
        line: "fn target() {}".into(),
    }];
    assert_eq!(format_search_hits(&hits), "src/lib.rs:12: fn target() {}\n");

    let commit = CommitInfo {
        oid: "0123456789abcdef0123456789abcdef01234567".into(),
        short_oid: "0123456".into(),
        summary: "修复".into(),
        message: "修复".into(),
        author: "李四".into(),
        author_email: None,
        committer: "李四".into(),
        committer_email: None,
        time: 1_755_000_000,
        parents: vec![],
        refs: vec![],
    };
    let text = format_history_lines(std::slice::from_ref(&commit));
    assert!(text.starts_with("0123456 修复 — 李四, "));
}

#[test]
fn tool_args_summary_uses_task_output_language_without_translating_query_or_history() {
    let arguments = r#"{"query":"中文 query[0-9]+","is_regex":true}"#;
    let history = AiReviewStep::ToolCall {
        name: "search_code".into(), args_summary: "search_code 旧查询（正则）".into(),
        result_excerpt: "原始工具结果".into(), error: false,
    };
    for system in [Language::ZhCn, Language::En] {
        for output in [Language::ZhCn, Language::En] {
            let languages = TaskLanguages { system, output };
            assert_eq!(tool_args_summary_in("search_code", arguments, languages.output),
                match output {
                    Language::ZhCn => "search_code 中文 query[0-9]+（正则）",
                    Language::En => "search_code 中文 query[0-9]+ (regex)",
                });
            assert_eq!(tool_args_summary_in("search_code", r#"{"query":"用户 query","is_regex":false}"#, languages.output),
                "search_code 用户 query");
            assert_eq!(history.summary_in(languages.output), "search_code 旧查询（正则）");
        }
    }
}

#[test]
fn tool_args_summary_variants() {
    assert_eq!(
        tool_args_summary(
            "read_lines",
            r#"{"path":"src/lib.rs","start":100,"end":180}"#
        ),
        "read_lines src/lib.rs:100-180"
    );
    assert_eq!(
        tool_args_summary("read_lines", r#"{"path":"src/lib.rs"}"#),
        "read_lines src/lib.rs"
    );
    assert_eq!(tool_args_summary("get_file_tree", "{}"), "get_file_tree /");
    assert_eq!(
        tool_args_summary("search_code", r#"{"query":"TODO","is_regex":true}"#),
        "search_code TODO（正则）"
    );
    // 超长参数截到 60 字符。
    let long_query = "q".repeat(100);
    let summary = tool_args_summary("search_code", &format!(r#"{{"query":"{long_query}"}}"#));
    assert!(summary.contains('…') && summary.chars().count() < 80);
    // 坏 JSON 容错：路径显示为空而不是 panic。
    assert_eq!(tool_args_summary("read_diff", "not json"), "read_diff ");
    assert_eq!(tool_args_summary("mystery", "{}"), "mystery");
}

#[test]
fn patch_text_renders_line_kinds() {
    let diff = FileDiff {
        path: "a.txt".into(),
        scope: crate::types::DiffScope::Staged,
        is_binary: false,
        untracked: false,
        old_size: None,
        new_size: None,
        encoding: crate::types::DiffEncodingInfo {
            requested: DiffEncodingChoice::Auto,
            resolved: DiffEncodingChoice::Utf8,
            lossy: false,
        },
        lines: vec![
            crate::types::DiffLine {
                kind: crate::types::DiffLineKind::Header,
                content: "--- a.txt".into(),
                old_lineno: None,
                new_lineno: None,
                hunk_index: 0,
            },
            crate::types::DiffLine {
                kind: crate::types::DiffLineKind::Context,
                content: "keep".into(),
                old_lineno: Some(1),
                new_lineno: Some(1),
                hunk_index: 0,
            },
            crate::types::DiffLine {
                kind: crate::types::DiffLineKind::Removed,
                content: "old".into(),
                old_lineno: Some(2),
                new_lineno: None,
                hunk_index: 0,
            },
            crate::types::DiffLine {
                kind: crate::types::DiffLineKind::Added,
                content: "new".into(),
                old_lineno: None,
                new_lineno: Some(2),
                hunk_index: 0,
            },
        ],
    };
    let text = file_diff_to_patch_text(&diff);
    assert_eq!(text, "--- a.txt\n keep\n-old\n+new\n");

    // 二进制文件给占位说明。
    let binary = FileDiff {
        path: "img.png".into(),
        is_binary: true,
        lines: Vec::new(),
        ..diff
    };
    assert_eq!(
        file_diff_to_patch_text(&binary),
        "（img.png 是二进制文件）\n"
    );
}

/// 变更状态标签与初始上下文的对接：ChangeState::label 输出直接进清单。
#[test]
fn change_state_labels_feed_initial_entries() {
    assert_eq!(ChangeState::Added.label(), "A");
    assert_eq!(ChangeState::Renamed.label(), "R");
    let entries = vec![InitialDiffEntry {
        path: "renamed.rs".into(),
        status_label: ChangeState::Renamed.label(),
        patch: String::new(),
    }];
    let prompt = assemble_initial_user_prompt("t", &entries);
    assert!(prompt.contains("- [R] renamed.rs"));
    assert!(prompt.contains("（无文本差异）"));
}
