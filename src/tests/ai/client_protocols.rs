use super::*;

#[test]
fn owned_protocol_errors_are_bilingual_and_provider_details_are_verbatim() {
    use crate::language::Language;
    let provider = "供应商原文 keep English ${do_not_translate}";
    let error = protocol_error(&json!({"type":"invalid_request_error","message":provider}));
    assert!(!error.retryable());
    for language in [Language::ZhCn, Language::En] {
        let display = error.display_in(language);
        assert!(display.contains(provider));
        assert_eq!(display.starts_with("AI response error:"), language == Language::En);
    }
    let error = error.into_git_error();
    assert!(matches!(error, GitError::Localized(_)));
    assert!(error.display_in(Language::En).contains(provider));
    let responses = parse_turn(AiApiType::Responses, &json!({"status":"completed"}), 123).unwrap_err();
    assert!(responses.display_in(Language::En).contains("missing output"));
    assert!(responses.display_in(Language::ZhCn).contains("缺少 output"));
    let anthropic = parse_turn(AiApiType::AnthropicMessages, &json!({"stop_reason":"end_turn"}), 123).unwrap_err();
    assert!(anthropic.display_in(Language::En).contains("missing content"));
    assert!(anthropic.display_in(Language::ZhCn).contains("缺少 content"));
    let sse_error = sse::read(std::io::Cursor::new(""), AiApiType::ChatCompletions, 123, &mut |_| {}, &|| false).unwrap_err();
    assert!(sse_error.display_in(Language::En).contains("no end marker"));
    assert!(sse_error.display_in(Language::ZhCn).contains("未收到结束标记"));
}

#[test]
fn truncation_and_empty_errors_retain_message_identity_and_retry_policy() {
    use crate::language::Language;
    let limited = agent_stream_truncation_message(true, Some("length"), 123).unwrap();
    assert_eq!(limited.retry_ceiling(), 1);
    assert!(limited.display_in(Language::En).contains("123"));
    assert!(limited.display_in(Language::ZhCn).contains("123"));
    let interrupted = agent_stream_truncation_message(false, Some("stop"), 123).unwrap();
    assert_eq!(interrupted.retry_ceiling(), AGENT_STREAM_MAX_RETRIES);
    let plain = plain_stream_truncation_error(true, Some("length"), 123).unwrap();
    assert!(plain.message_key().is_some());
    assert!(plain.display_in(Language::En).contains("incomplete"));
    assert!(agent_turn_empty_error(0, false).display_in(Language::En).contains("no valid chunks"));
    assert!(agent_turn_empty_error(1, true).display_in(Language::En).contains("only reasoning"));
}
use std::io::{Read, Write};
use std::net::TcpListener;

fn settings(api: AiApiType) -> AiProviderSettings {
    AiProviderSettings { api_type: api, model: "fixture-model".into(), api_key: "fixture-key".into(), temperature: 1.0, max_tokens: 1234, reasoning_effort: Some("  custom-effort  ".into()), ..Default::default() }
}
fn tools() -> Vec<ToolSchema> { vec![ToolSchema { name: "read", description: "读取", parameters: json!({"type":"object","properties":{"path":{"type":"string"}}}) }] }
fn user() -> Vec<AgentChatMessage> { vec![AgentChatMessage::System("规则".into()), AgentChatMessage::User("输入".into())] }
fn response_output() -> Value { json!([
    {"id":"reason_1","type":"reasoning","summary":[{"type":"summary_text","text":"思考"}],"encrypted_content":"encrypted-final"},
    {"id":"fc_1","type":"function_call","call_id":"call_1","name":"read","arguments":"{\"path\":\"a\"}","status":"completed"}
]) }
fn anthropic_blocks() -> Value { json!([
    {"type":"thinking","thinking":"思考","signature":"signature-final"},
    {"type":"redacted_thinking","data":"opaque-secret"},
    {"type":"tool_use","id":"call_1","name":"read","input":{"path":"a"}}
]) }
fn event(kind: &str, value: Value) -> String { format!("event: {kind}\r\ndata:{}\r\n\r\n", value) }
fn responses_stream() -> String {
    event("response.output_item.added", json!({"type":"response.output_item.added","output_index":0,"item":{"type":"reasoning","encrypted_content":"partial"}}))
    + &event("response.reasoning_summary_text.delta", json!({"type":"response.reasoning_summary_text.delta","delta":"思考"}))
    + &event("response.output_item.done", json!({"type":"response.output_item.done","output_index":0,"item":response_output()[0]}))
    + &event("response.function_call_arguments.delta", json!({"type":"response.function_call_arguments.delta","output_index":1,"delta":"{\"path\":"}))
    + &event("response.output_item.done", json!({"type":"response.output_item.done","output_index":1,"item":response_output()[1]}))
    + &event("response.completed", json!({"type":"response.completed","response":{"status":"completed","output":response_output()}}))
}
fn anthropic_stream() -> String {
    let mut s = event("message_start", json!({"type":"message_start","message":{"role":"assistant","content":[],"stop_reason":null}}));
    for (i, b) in anthropic_blocks().as_array().unwrap().iter().enumerate() {
        let mut start = b.clone();
        if i == 0 { start["signature"] = json!(""); start["thinking"] = json!(""); }
        if i == 2 { start["input"] = json!({}); }
        s += &event("content_block_start", json!({"type":"content_block_start","index":i,"content_block":start}));
        if i == 0 {
            s += &event("content_block_delta", json!({"type":"content_block_delta","index":i,"delta":{"type":"thinking_delta","thinking":"思考"}}));
            s += &event("content_block_delta", json!({"type":"content_block_delta","index":i,"delta":{"type":"signature_delta","signature":"signature-"}}));
            s += &event("content_block_delta", json!({"type":"content_block_delta","index":i,"delta":{"type":"signature_delta","signature":"final"}}));
        }
        if i == 2 {
            for part in ["{\"path\":", "\"a\"}"] { s += &event("content_block_delta", json!({"type":"content_block_delta","index":i,"delta":{"type":"input_json_delta","partial_json":part}})); }
        }
        s += &event("content_block_stop", json!({"type":"content_block_stop","index":i}));
    }
    s += &event("message_delta", json!({"type":"message_delta","delta":{"stop_reason":"tool_use"}}));
    s += &event("message_stop", json!({"type":"message_stop"}));
    s
}

/// 本地真实 HTTP fixture 验证 ureq 请求、认证、端点、序列化和 SSE 整条链路。
/// 线程仅属于测试服务端，不进入业务代码。
fn fixture(replies: Vec<(u16, String, bool)>, test: impl FnOnce(String)) -> Vec<(String, Value)> {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let mut requests = vec![];
        for (status, body, stream) in replies {
            let (mut socket, _) = listener.accept().unwrap();
            socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            let mut bytes = vec![];
            let mut buf = [0; 1024];
            let (header_end, length) = loop {
                let n = socket.read(&mut buf).unwrap(); assert!(n > 0); bytes.extend_from_slice(&buf[..n]);
                if let Some(i) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let h = String::from_utf8_lossy(&bytes[..i]);
                    let len = h.lines().find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse::<usize>().unwrap())).unwrap();
                    break (i + 4, len);
                }
            };
            while bytes.len() < header_end + length { let n = socket.read(&mut buf).unwrap(); assert!(n > 0); bytes.extend_from_slice(&buf[..n]); }
            requests.push((String::from_utf8(bytes[..header_end].to_vec()).unwrap(), serde_json::from_slice(&bytes[header_end..header_end+length]).unwrap()));
            let content_type = if stream { "text/event-stream" } else { "application/json" };
            write!(socket, "HTTP/1.1 {status} fixture\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
        requests
    });
    test(base);
    server.join().unwrap()
}

#[test]
fn all_protocols_share_budget_temperature_effort_and_tool_schema() {
    for api in [AiApiType::ChatCompletions, AiApiType::Responses, AiApiType::AnthropicMessages] {
        let mut settings = settings(api);
        let body = build_body(&settings, &user(), &tools(), 777, true).unwrap();
        assert_eq!(body["temperature"], 1.0);
        assert_eq!(body["stream"], true);
        match api {
            AiApiType::ChatCompletions => { assert_eq!(body["reasoning_effort"], "custom-effort"); assert_eq!(body["max_tokens"], 777); assert_eq!(body["tools"][0]["function"]["name"], "read"); }
            AiApiType::Responses => { assert_eq!(body["reasoning"]["effort"], "custom-effort"); assert_eq!(body["max_output_tokens"], 777); assert_eq!(body["tools"][0]["name"], "read"); assert_eq!(body["include"][0], "reasoning.encrypted_content"); assert_eq!(body["store"], false); }
            AiApiType::AnthropicMessages => { assert_eq!(body["output_config"]["effort"], "custom-effort"); assert!(body.get("thinking").is_none()); assert_eq!(body["max_tokens"], 777); assert_eq!(body["tools"][0]["input_schema"], tools()[0].parameters); assert_eq!(body["system"][0]["text"], "规则"); }
        }
        settings.reasoning_effort = Some("  ".into());
        let body = build_body(&settings, &user(), &[], 100, false).unwrap();
        for key in ["reasoning", "reasoning_effort", "output_config", "tools", "tool_choice"] { assert!(body.get(key).is_none(), "{key}"); }
    }
}

#[test]
fn opaque_history_roundtrips_without_losing_signed_or_encrypted_items() {
    for (api, value, key) in [
        (AiApiType::Responses, json!({"status":"completed","output":response_output()}), "input"),
        (AiApiType::AnthropicMessages, json!({"stop_reason":"tool_use","content":anthropic_blocks()}), "messages"),
    ] {
        let turn = parse_turn(api, &value, 100).unwrap();
        assert_eq!(turn.tool_calls[0].id, "call_1");
        let message = turn.into_message();
        assert!(message.context_bytes() > 50);
        let messages = vec![message.clone(), AgentChatMessage::Tool { tool_call_id: "call_1".into(), content: "result1".into() }, AgentChatMessage::Tool { tool_call_id: "call_2".into(), content: "result2".into() }];
        let body = build_body(&settings(api), &messages, &[], 100, false).unwrap();
        if api == AiApiType::Responses { assert_eq!(body[key][0], value["output"][0]); assert_eq!(body[key][1], value["output"][1]); assert_eq!(body[key][2]["call_id"], "call_1"); }
        else { assert_eq!(body[key][0]["content"], value["content"]); assert_eq!(body[key][1]["content"].as_array().unwrap().len(), 2); }
        let other = if api == AiApiType::Responses { AiApiType::AnthropicMessages } else { AiApiType::Responses };
        assert!(!build_body(&settings(other), &[message], &[], 100, false).unwrap_err().retryable());
    }
}

#[test]
fn responses_and_anthropic_agent_http_tool_roundtrip_nonstream_and_stream() {
    for api in [AiApiType::Responses, AiApiType::AnthropicMessages] {
        for stream in [false, true] {
            let first = match (api, stream) {
                (AiApiType::Responses, true) => responses_stream(),
                (AiApiType::Responses, false) => json!({"status":"completed","output":response_output()}).to_string(),
                (_, true) => anthropic_stream(),
                (_, false) => json!({"stop_reason":"tool_use","content":anthropic_blocks()}).to_string(),
            };
            let final_response = if api == AiApiType::Responses { json!({"status":"completed","output":[{"type":"message","role":"assistant","content":[{"type":"output_text","text":"结论"}]}]}) } else { json!({"stop_reason":"end_turn","content":[{"type":"text","text":"结论"}]}) };
            let requests = fixture(vec![(200, first, stream), (200, final_response.to_string(), false)], |base| {
                let client = ChatClient::new(AiProviderSettings { base_url: base, ..settings(api) }, None);
                let mut history = user();
                let mut deltas = vec![];
                let turn = if stream { client.request_agent_stream(&history, &tools(), 999, &mut |d| deltas.push(d)).unwrap() } else { client.request_agent(&history, &tools(), 999).unwrap() };
                assert_eq!(turn.tool_calls[0].arguments, "{\"path\":\"a\"}");
                assert_eq!(turn.reasoning.as_deref(), Some("思考"));
                if stream { assert!(deltas.iter().any(|d| matches!(d, StreamDelta::Reasoning(s) if s == "思考"))); }
                history.push(turn.into_message());
                history.push(AgentChatMessage::Tool { tool_call_id: "call_1".into(), content: "data".into() });
                assert_eq!(client.request_agent(&history, &[], client.max_output_tokens()).unwrap().content, "结论");
            });
            let headers = requests[0].0.to_ascii_lowercase();
            assert!(headers.contains(&format!("post {} http/1.1", api.endpoint_path())));
            assert!(headers.contains("user-agent: khaslana/"));
            if api == AiApiType::AnthropicMessages {
                assert!(headers.contains("x-api-key: fixture-key")); assert!(headers.contains("anthropic-version: 2023-06-01")); assert!(!headers.contains("authorization:"));
                assert_eq!(requests[1].1["messages"][1]["content"], anthropic_blocks());
            } else { assert!(headers.contains("authorization: bearer fixture-key")); assert_eq!(requests[1].1["input"][2], response_output()[0]); }
        }
    }
}

#[test]
fn all_protocols_plain_http_nonstream_and_stream() {
    for api in [AiApiType::ChatCompletions, AiApiType::Responses, AiApiType::AnthropicMessages] {
        let json_response = match api {
            AiApiType::ChatCompletions => json!({"choices":[{"finish_reason":"stop","message":{"content":"正文"}}]}),
            AiApiType::Responses => json!({"status":"completed","output":[{"type":"message","content":[{"type":"output_text","text":"正文"}]}]}),
            AiApiType::AnthropicMessages => json!({"stop_reason":"end_turn","content":[{"type":"text","text":"正文"}]}),
        };
        let stream = match api {
            AiApiType::ChatCompletions => "data:{\"choices\":[{\"delta\":{\"content\":\"正文\"},\"finish_reason\":\"stop\"}]}\n\ndata:[DONE]\n\n".into(),
            AiApiType::Responses => event("response.output_text.delta", json!({"type":"response.output_text.delta","delta":"正文"})) + &event("response.completed", json!({"type":"response.completed","response":json_response})),
            AiApiType::AnthropicMessages => event("message_start", json!({"type":"message_start"}))
                + &event("content_block_start", json!({"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}))
                + &event("content_block_delta", json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"正文"}}))
                + &event("content_block_stop", json!({"type":"content_block_stop","index":0}))
                + &event("message_delta", json!({"type":"message_delta","delta":{"stop_reason":"end_turn"}}))
                + &event("message_stop", json!({"type":"message_stop"})),
        };
        fixture(vec![(200, json_response.to_string(), false), (200, stream, true)], |base| {
            let client = ChatClient::new(AiProviderSettings { base_url: base, ..settings(api) }, None);
            let messages = vec![ChatMessage { role: crate::ai::prompt::ChatRole::User, content: "输入".into() }];
            assert_eq!(client.request(&messages).unwrap().content, "正文");
            let mut deltas = vec![];
            assert_eq!(client.request_stream(&messages, &mut |d| deltas.push(d)).unwrap().content, "正文");
            assert!(deltas.iter().any(|d| matches!(d, StreamDelta::Content(s) if s == "正文")));
        });
    }
}

#[test]
fn streams_reject_errors_malformed_data_truncation_and_incomplete_arguments() {
    let cases = [
        (AiApiType::Responses, event("response.incomplete", json!({"type":"response.incomplete","response":{"incomplete_details":{"reason":"max_output_tokens"}}})), 1),
        (AiApiType::Responses, event("response.output_text.delta", json!({"type":"response.output_text.delta","delta":"半截"})), 3),
        (AiApiType::Responses, event("response.failed", json!({"type":"response.failed","response":{"error":{"code":"invalid_api_key","message":"bad key"}}})), 0),
        (AiApiType::AnthropicMessages, event("error", json!({"type":"error","error":{"type":"overloaded_error","message":"busy"}})), 3),
        (AiApiType::AnthropicMessages, anthropic_stream().replace("\"stop_reason\":\"tool_use\"", "\"stop_reason\":\"max_tokens\""), 1),
        (AiApiType::AnthropicMessages, anthropic_stream().replace("signature-final", "" ).replace("signature-", "").replace("\"signature\":\"final\"", "\"signature\":\"\""), 3),
        (AiApiType::ChatCompletions, "data:{\"error\":{\"message\":\"upstream failed\"}}\n\ndata:[DONE]\n\n".into(), 3),
        (AiApiType::ChatCompletions, "data:{broken\n\ndata:[DONE]\n\n".into(), 3),
        (AiApiType::ChatCompletions, "data:{\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"c\",\"function\":{\"name\":\"read\",\"arguments\":\"{\"}}]},\"finish_reason\":\"tool_calls\"}]}\n\ndata:[DONE]\n\n".into(), 3),
    ];
    for (api, response, ceiling) in cases {
        fixture(vec![(200, response, true)], |base| {
            let client = ChatClient::new(AiProviderSettings { base_url: base, ..settings(api) }, None);
            let err = client.request_agent_stream(&user(), &tools(), 100, &mut |_| {}).unwrap_err();
            assert_eq!(err.retry_ceiling(), ceiling, "{api:?}: {err}");
        });
    }
}

#[test]
fn http_status_and_proxy_errors_keep_existing_retry_policy() {
    for (status, retryable) in [(400, false), (401, false), (429, true), (503, true)] {
        fixture(vec![(status, "{}".into(), false)], |base| {
            let client = ChatClient::new(AiProviderSettings { base_url: base, ..settings(AiApiType::Responses) }, None);
            assert_eq!(client.request_agent(&user(), &[], 100).unwrap_err().retryable(), retryable);
        });
    }
    let client = ChatClient::new(settings(AiApiType::Responses), Some("not a proxy URL".into()));
    assert!(!client.request_agent(&user(), &[], 100).unwrap_err().retryable());
}

#[test]
fn cancellation_checked_before_transport_and_between_events() {
    let client = ChatClient::new(settings(AiApiType::Responses), None);
    assert!(!client.request_agent_stream_until(&user(), &[], 100, &mut |_| {}, &|| true).unwrap_err().retryable());
    let cancelled = std::cell::Cell::new(false);
    let stream = responses_stream();
    let err = sse::read(std::io::Cursor::new(stream), AiApiType::Responses, 100, &mut |_| cancelled.set(true), &|| cancelled.get()).unwrap_err();
    assert!(err.message().contains("中止"));
    assert!(!err.retryable());
}

#[test]
fn responses_terminal_items_must_match_and_arguments_must_be_complete() {
    let mismatch = responses_stream();
    let mut terminal: Value = json!({"type":"response.completed","response":{"status":"completed","output":response_output()}});
    terminal["response"]["output"][0]["encrypted_content"] = json!("different");
    let prefix = mismatch.split("event: response.completed").next().unwrap();
    let malformed = prefix.to_owned() + &event("response.completed", terminal);
    assert!(sse::read(std::io::Cursor::new(malformed), AiApiType::Responses, 100, &mut |_| {}, &|| false).unwrap_err().message().contains("不一致"));
    let mut output = response_output();
    output[1]["arguments"] = json!("{\"path\":");
    assert!(parse_turn(AiApiType::Responses, &json!({"status":"completed","output":output}), 100).unwrap_err().message().contains("参数"));
}

#[test]
fn plain_reasoning_only_is_preserved_but_agent_rejects_it() {
    let body = json!({"status":"completed","output":[{"type":"reasoning","summary":[{"text":"思考"}],"encrypted_content":"opaque"}]});
    fixture(vec![(200, event("response.completed", json!({"type":"response.completed","response":body})), true)], |base| {
        let client = ChatClient::new(AiProviderSettings { base_url: base, ..settings(AiApiType::Responses) }, None);
        let result = client.request_stream(&[ChatMessage { role: crate::ai::prompt::ChatRole::User, content: "输入".into() }], &mut |_| {}).unwrap();
        assert!(result.content.is_empty());
        assert_eq!(result.reasoning.as_deref(), Some("思考"));
    });
    assert!(validate_agent_turn(parse_turn(AiApiType::Responses, &body, 100).unwrap()).is_err());
}

#[test]
fn sse_handles_multiline_data_comments_and_ignores_ping() {
    let stream = ": keepalive\r\nevent: ping\r\ndata: {\"type\":\"ping\"}\r\n\r\nevent: response.completed\r\ndata: {\"type\":\"response.completed\",\r\ndata: \"response\":{\"status\":\"completed\",\"output\":[]}}\r\n\r\n";
    assert!(sse::read(std::io::Cursor::new(stream), AiApiType::Responses, 100, &mut |_| {}, &|| false).is_ok());
}
