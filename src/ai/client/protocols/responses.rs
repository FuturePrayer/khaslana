//! 基于 openai-node 官方 responses.ts：完整 output 项按顺序原样回填。
use crate::ai::messages::ai_error;
use super::*;

pub(super) fn build(body: &mut Value, messages: &[AgentChatMessage], tools: &[ToolSchema], max_tokens: u32, effort: Option<&str>) -> Result<(), AgentStreamError> {
    let mut input = vec![];
    for message in messages {
        match message {
            AgentChatMessage::System(s) => input.push(json!({"role":"system","content":s})),
            AgentChatMessage::User(s) => input.push(json!({"role":"user","content":s})),
            AgentChatMessage::Tool { tool_call_id, content } => input.push(json!({"type":"function_call_output","call_id":tool_call_id,"output":content})),
            AgentChatMessage::AssistantOpaque { history: AgentProtocolHistory::Responses(items), .. } => input.extend(items.iter().cloned()),
            AgentChatMessage::AssistantOpaque { .. } => return Err(AgentStreamError::fatal_error(ai_error!("ai.responses.8ea8b5a03e6e", "不能跨 AI 接口回填协议历史，请新建会话", "Cannot replay protocol history across AI APIs; start a new session"))),
            AgentChatMessage::Assistant { content, tool_calls } => {
                if !content.is_empty() { input.push(json!({"role":"assistant","content":content})); }
                input.extend(tool_calls.iter().map(|c| json!({"type":"function_call","call_id":c.id,"name":c.name,"arguments":c.arguments})));
            }
        }
    }
    body["input"] = json!(input);
    body["max_output_tokens"] = json!(max_tokens);
    body["store"] = json!(false);
    body["include"] = json!(["reasoning.encrypted_content"]);
    if let Some(effort) = effort { body["reasoning"] = json!({"effort":effort}); }
    if !tools.is_empty() {
        // 官方默认 strict=true 会收紧任意工具 schema；这里显式保留应用现有非严格参数语义。
        body["tools"] = json!(tools.iter().map(|t| json!({"type":"function","name":t.name,"description":t.description,"parameters":t.parameters,"strict":false})).collect::<Vec<_>>());
        body["tool_choice"] = json!("auto");
    }
    Ok(())
}

pub(super) fn parse(value: &Value, max_tokens: u32) -> Result<AgentTurn, AgentStreamError> {
    if value["status"] != "completed" {
        check_finish(value["incomplete_details"]["reason"].as_str(), max_tokens)?;
        return Err(AgentStreamError::transient_error(ai_error!("ai.responses.2d465c80c9b9", "AI Responses 未完整完成：{arg0}", "AI Responses did not complete: {arg0}", "arg0" => value["status"])));
    }
    let items = value["output"].as_array().ok_or_else(|| AgentStreamError::transient_error(ai_error!("ai.responses.35780f65b059", "AI Responses 缺少 output", "AI Responses is missing output")))?;
    let mut content = String::new();
    let mut reasoning = String::new();
    let mut calls = vec![];
    for item in items {
        if item["status"] == "incomplete" || item["status"] == "in_progress" { return Err(AgentStreamError::transient_error(ai_error!("ai.responses.f41946077825", "AI Responses 输出项未完整完成", "AI Responses output item did not complete"))); }
        match item["type"].as_str() {
            Some("message") => {
                if let Some(parts) = item["content"].as_array() { for part in parts {
                    match part["type"].as_str() {
                        Some("output_text") => content.push_str(part["text"].as_str().unwrap_or_default()),
                        Some("refusal") => return Err(AgentStreamError::fatal_error(ai_error!("ai.responses.288a8de369fe", "AI 拒绝回答：{arg0}", "AI refused to answer: {arg0}", "arg0" => part["refusal"].as_str().unwrap_or_default()))),
                        _ => {}
                    }
                } }
            }
            Some("reasoning") => for key in ["summary", "content"] {
                if let Some(parts) = item[key].as_array() { for part in parts { reasoning.push_str(part["text"].as_str().unwrap_or_default()); } }
            },
            Some("function_call") => calls.push(call(item, "call_id", item, "arguments")?),
            _ => {}
        }
    }
    let (content, reasoning) = split_response_message_parts(Some(content), (!reasoning.is_empty()).then_some(reasoning));
    Ok(AgentTurn { content: content.trim().into(), reasoning, tool_calls: calls, protocol_history: Some(AgentProtocolHistory::Responses(items.clone())) })
}
