//! SSE 按事件而非单行解码；保留多行 data，只有协议终态才算成功。
use crate::ai::messages::ai_error;
use super::*;
use std::collections::BTreeMap;

pub(super) fn read(reader: impl BufRead, api: AiApiType, max_tokens: u32, on_delta: &mut impl FnMut(StreamDelta), should_stop: &impl Fn() -> bool) -> Result<AgentTurn, AgentStreamError> {
    let mut state = State::default();
    let mut data = vec![];
    let mut event = String::new();
    for line in reader.lines() {
        if should_stop() { return Err(AgentStreamError::fatal_error(ai_error!("ai.sse.b949b24dc9a1", "AI 请求已中止", "AI request aborted"))); }
        let line = line.map_err(|e| AgentStreamError::transient_error(ai_error!("ai.sse.a58f803ebe5b", "AI 流读取失败：{e}", "Failed to read AI stream: {e}", "e" => e)))?;
        if should_stop() { return Err(AgentStreamError::fatal_error(ai_error!("ai.sse.b949b24dc9a1", "AI 请求已中止", "AI request aborted"))); }
        if line.is_empty() {
            if !data.is_empty() {
                state.push(api, &event, &data.join("\n"), max_tokens, on_delta)?;
                data.clear();
                if state.done { break; }
            }
            event.clear();
        } else if let Some(v) = line.strip_prefix("data:") { data.push(v.strip_prefix(' ').unwrap_or(v).to_owned()); }
        else if let Some(v) = line.strip_prefix("event:") { event = v.trim().into(); }
    }
    // 未以空行结尾的末事件仍解析；但不能凭 EOF 合成结束标记。
    if !state.done && !data.is_empty() { state.push(api, &event, &data.join("\n"), max_tokens, on_delta)?; }
    if !state.done { return Err(AgentStreamError::transient_error(ai_error!("ai.sse.cfa846e07650", "AI 响应流在完成前被中断（未收到结束标记）", "AI response stream was interrupted before completion (no end marker)"))); }
    match api {
        AiApiType::Responses => parse_turn(api, &state.final_value, max_tokens),
        AiApiType::AnthropicMessages => {
            if !state.open_blocks.is_empty() { return Err(AgentStreamError::transient_error(ai_error!("ai.sse.c9142ed1fd8b", "AI Anthropic 内容块未完整结束", "AI Anthropic content block did not finish"))); }
            let blocks = state.blocks.into_values().collect::<Vec<_>>();
            parse_turn(api, &json!({"content":blocks,"stop_reason":state.finish}), max_tokens)
        }
        AiApiType::ChatCompletions => {
            check_finish(state.finish.as_deref(), max_tokens)?;
            let mut calls = vec![];
            for c in state.blocks.into_values() { calls.push(call(&c, "id", &c["function"], "arguments")?); }
            let (content, reasoning) = split_response_message_parts(Some(state.content), (!state.reasoning.is_empty()).then_some(state.reasoning));
            Ok(AgentTurn { content: content.trim().into(), reasoning, tool_calls: calls, protocol_history: None })
        }
    }
}

#[derive(Default)]
struct State {
    done: bool,
    started: bool,
    finish: Option<String>,
    content: String,
    reasoning: String,
    final_value: Value,
    blocks: BTreeMap<usize, Value>,
    open_blocks: std::collections::BTreeSet<usize>,
    arguments: BTreeMap<usize, String>,
}
impl State {
    fn push(&mut self, api: AiApiType, event: &str, data: &str, max_tokens: u32, on_delta: &mut impl FnMut(StreamDelta)) -> Result<(), AgentStreamError> {
        if api == AiApiType::ChatCompletions && data.trim() == "[DONE]" { self.done = true; return Ok(()); }
        let v: Value = serde_json::from_str(data).map_err(|e| AgentStreamError::transient_error(ai_error!("ai.sse.2212fdfd4c9c", "AI SSE 数据格式异常，内容可能被截断：{e}", "Invalid AI SSE data; content may be truncated: {e}", "e" => e)))?;
        if !v["error"].is_null() { return Err(protocol_error(&v["error"])); }
        let kind = v["type"].as_str().unwrap_or(event);
        if kind == "error" { return Err(protocol_error(&v)); }
        match api {
            AiApiType::Responses => match kind {
                "response.output_text.delta" | "response.refusal.delta" => {
                    if kind == "response.refusal.delta" { return Err(AgentStreamError::fatal_error(ai_error!("ai.sse.b89112c652dc", "AI 拒绝回答", "AI refused to answer"))); }
                    emit(&v["delta"], false, on_delta);
                }
                "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => emit(&v["delta"], true, on_delta),
                "response.output_item.done" => {
                    let index = index(&v, "output_index")?;
                    self.blocks.insert(index, v["item"].clone());
                }
                "response.completed" => {
                    self.final_value = v["response"].clone();
                    // 以 done 项的完整签名为准，added 项可能只有部分密文。
                    if self.final_value["output"].is_null() {
                        self.final_value["output"] = json!(self.blocks.values().collect::<Vec<_>>());
                    } else {
                        for (i, item) in &self.blocks {
                            if self.final_value["output"].get(*i) != Some(item) {
                                return Err(AgentStreamError::transient_error(ai_error!("ai.sse.b1b0cef7f23d", "AI Responses 完成事件与完整输出项不一致", "AI Responses completion event does not match complete output items")));
                            }
                        }
                    }
                    self.done = true;
                }
                "response.incomplete" | "response.failed" => {
                    if !v["response"]["error"].is_null() { return Err(protocol_error(&v["response"]["error"])); }
                    check_finish(v["response"]["incomplete_details"]["reason"].as_str(), max_tokens)?;
                    return Err(AgentStreamError::transient_error(ai_error!("ai.sse.5bf2eadae466", "AI Responses 流未完整完成：{kind}", "AI Responses stream did not complete: {kind}", "kind" => kind)));
                }
                _ => {}
            },
            AiApiType::AnthropicMessages => match kind {
                "message_start" => { if self.started { return Err(AgentStreamError::transient_error(ai_error!("ai.sse.ae869b6a4978", "AI Anthropic 重复回合起点", "AI Anthropic has a duplicate turn start"))); } self.started = true; }
                "content_block_start" => {
                    let i = index(&v, "index")?;
                    if !self.started || self.blocks.contains_key(&i) { return Err(AgentStreamError::transient_error(ai_error!("ai.sse.aa61acb7bd48", "AI Anthropic 内容块顺序异常", "AI Anthropic content blocks are out of order"))); }
                    self.blocks.insert(i, v["content_block"].clone());
                    self.open_blocks.insert(i);
                    let b = &v["content_block"];
                    if b["type"] == "text" { emit(&b["text"], false, on_delta); }
                    if b["type"] == "thinking" { emit(&b["thinking"], true, on_delta); }
                }
                "content_block_delta" => {
                    let i = index(&v, "index")?;
                    if !self.open_blocks.contains(&i) { return Err(AgentStreamError::transient_error(ai_error!("ai.sse.9b9dde067dd4", "AI Anthropic 收到孤立内容增量", "AI Anthropic received an orphan content delta"))); }
                    let b = self.blocks.get_mut(&i).unwrap();
                    let d = &v["delta"];
                    match d["type"].as_str() {
                        Some("text_delta") => { append(b, "text", &d["text"]); emit(&d["text"], false, on_delta); }
                        Some("thinking_delta") => { append(b, "thinking", &d["thinking"]); emit(&d["thinking"], true, on_delta); }
                        Some("signature_delta") => append(b, "signature", &d["signature"]),
                        Some("input_json_delta") => self.arguments.entry(i).or_default().push_str(d["partial_json"].as_str().unwrap_or_default()),
                        _ => {}
                    }
                }
                "content_block_stop" => {
                    let i = index(&v, "index")?;
                    if !self.open_blocks.remove(&i) { return Err(AgentStreamError::transient_error(ai_error!("ai.sse.d51370b4c7dd", "AI Anthropic 内容块重复或孤立终态", "AI Anthropic content block has a duplicate or orphan terminal state"))); }
                    if let Some(args) = self.arguments.remove(&i) {
                        self.blocks.get_mut(&i).unwrap()["input"] = serde_json::from_str(&args).map_err(|_| AgentStreamError::transient_error(ai_error!("ai.sse.7d10992541ce", "AI 工具参数被截断", "AI tool arguments were truncated")))?;
                    }
                }
                "message_delta" => { self.finish = v["delta"]["stop_reason"].as_str().map(str::to_owned); check_finish(self.finish.as_deref(), max_tokens)?; }
                "message_stop" => { if !self.started || self.finish.is_none() { return Err(AgentStreamError::transient_error(ai_error!("ai.sse.b3f66c0f9c22", "AI Anthropic 缺少回合终态", "AI Anthropic is missing the turn terminal state"))); } self.done = true; }
                _ => {}
            },
            AiApiType::ChatCompletions => {
                if let Some(choices) = v["choices"].as_array() { for c in choices {
                    if c["index"].as_u64().unwrap_or(0) != 0 { continue; }
                    if let Some(reason) = c["finish_reason"].as_str() { self.finish = Some(reason.into()); check_finish(Some(reason), max_tokens)?; }
                    let d = &c["delta"];
                    if let Some(text) = d["content"].as_str() { self.content.push_str(text); emit(&d["content"], false, on_delta); }
                    if let Some(text) = d["reasoning_content"].as_str() { self.reasoning.push_str(text); emit(&d["reasoning_content"], true, on_delta); }
                    if let Some(calls) = d["tool_calls"].as_array() { for t in calls {
                        let i = t["index"].as_u64().unwrap_or(0) as usize;
                        let b = self.blocks.entry(i).or_insert_with(|| json!({"function":{"name":"","arguments":""}}));
                        if b["id"].is_null() && t["id"].is_string() { b["id"] = t["id"].clone(); }
                        append(&mut b["function"], "name", &t["function"]["name"]);
                        append(&mut b["function"], "arguments", &t["function"]["arguments"]);
                    } }
                } }
            }
        }
        Ok(())
    }
}
fn index(v: &Value, field: &str) -> Result<usize, AgentStreamError> { v[field].as_u64().map(|n| n as usize).ok_or_else(|| AgentStreamError::transient_error(ai_error!("ai.sse.1ec310103654", "AI SSE 内容块缺少索引", "AI SSE content block is missing an index"))) }
fn append(v: &mut Value, key: &str, delta: &Value) { if let Some(delta) = delta.as_str() { let mut s = v[key].as_str().unwrap_or_default().to_owned(); s.push_str(delta); v[key] = json!(s); } }
fn emit(v: &Value, reasoning: bool, on_delta: &mut impl FnMut(StreamDelta)) { if let Some(s) = v.as_str().filter(|s| !s.is_empty()) { on_delta(if reasoning { StreamDelta::Reasoning(s.into()) } else { StreamDelta::Content(s.into()) }); } }
