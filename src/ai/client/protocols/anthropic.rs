//! 基于 anthropic-sdk-typescript messages.ts。签名及 redacted_thinking 块不改写。
use super::*;

pub(super) fn build(body: &mut Value, messages: &[AgentChatMessage], tools: &[ToolSchema], max_tokens: u32, effort: Option<&str>) -> Result<(), AgentStreamError> {
    let mut system = vec![];
    let mut turns: Vec<Value> = vec![];
    for message in messages {
        let (role, blocks) = match message {
            AgentChatMessage::System(s) => { system.push(json!({"type":"text","text":s})); continue; }
            AgentChatMessage::User(s) => ("user", vec![json!({"type":"text","text":s})]),
            AgentChatMessage::Tool { tool_call_id, content } => ("user", vec![json!({"type":"tool_result","tool_use_id":tool_call_id,"content":content})]),
            AgentChatMessage::AssistantOpaque { history: AgentProtocolHistory::AnthropicMessages(items), .. } => ("assistant", items.clone()),
            AgentChatMessage::AssistantOpaque { .. } => return Err(AgentStreamError::fatal("不能跨 AI 接口回填协议历史，请新建会话".into())),
            AgentChatMessage::Assistant { content, tool_calls } => {
                let mut blocks = vec![];
                if !content.is_empty() { blocks.push(json!({"type":"text","text":content})); }
                for c in tool_calls {
                    let input: Value = serde_json::from_str(&c.arguments).map_err(|_| AgentStreamError::fatal("工具历史参数不是有效 JSON".into()))?;
                    blocks.push(json!({"type":"tool_use","id":c.id,"name":c.name,"input":input}));
                }
                ("assistant", blocks)
            }
        };
        // 并行工具结果必须在同一 user 回合的开头，不能各自拆成独立回合。
        if let Some(last) = turns.last_mut().filter(|v| v["role"] == role) {
            last["content"].as_array_mut().unwrap().extend(blocks);
        } else { turns.push(json!({"role":role,"content":blocks})); }
    }
    body["messages"] = json!(turns);
    body["max_tokens"] = json!(max_tokens);
    if !system.is_empty() { body["system"] = json!(system); }
    // effort 不等同于开启 thinking。SDK 将二者独立声明，省略 thinking 的语义由模型决定。
    if let Some(effort) = effort { body["output_config"] = json!({"effort":effort}); }
    if !tools.is_empty() {
        body["tools"] = json!(tools.iter().map(|t| json!({"name":t.name,"description":t.description,"input_schema":t.parameters})).collect::<Vec<_>>());
        body["tool_choice"] = json!({"type":"auto"});
    }
    Ok(())
}

pub(super) fn parse(value: &Value, max_tokens: u32) -> Result<AgentTurn, AgentStreamError> {
    let reason = value["stop_reason"].as_str();
    check_finish(reason, max_tokens)?;
    if !matches!(reason, Some("end_turn" | "tool_use" | "stop_sequence")) {
        return Err(AgentStreamError::transient(format!("AI Anthropic 回合未完成：{}", value["stop_reason"])));
    }
    let blocks = value["content"].as_array().ok_or_else(|| AgentStreamError::transient("AI Anthropic 缺少 content".into()))?;
    let mut content = String::new();
    let mut reasoning = String::new();
    let mut calls = vec![];
    for block in blocks {
        match block["type"].as_str() {
            Some("text") => content.push_str(block["text"].as_str().unwrap_or_default()),
            Some("thinking") => {
                if block["signature"].as_str().is_none_or(str::is_empty) { return Err(AgentStreamError::transient("AI 思考块缺少签名，无法安全继续会话".into())); }
                reasoning.push_str(block["thinking"].as_str().unwrap_or_default());
            }
            Some("tool_use") => calls.push(call(block, "id", block, "input")?),
            _ => {}
        }
    }
    if (reason == Some("tool_use")) != !calls.is_empty() { return Err(AgentStreamError::transient("AI 工具调用与终态不一致".into())); }
    let (content, reasoning) = split_response_message_parts(Some(content), (!reasoning.is_empty()).then_some(reasoning));
    Ok(AgentTurn { content: content.trim().into(), reasoning, tool_calls: calls, protocol_history: Some(AgentProtocolHistory::AnthropicMessages(blocks.clone())) })
}
