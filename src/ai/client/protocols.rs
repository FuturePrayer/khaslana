//! 协议适配与共享传输。保持同步 ureq，不引入业务线程或第二套代理策略。
use crate::ai::messages::ai_error;
use super::*;
use crate::ai::config::AiApiType;
use serde_json::{Value, json};
mod responses;
mod anthropic;
mod sse;

impl ChatClient {
    /// 非流式工具回合；当前业务优先流式，保留同步协议入口。
    #[allow(dead_code)]
    pub fn request_agent(&self, messages: &[AgentChatMessage], tools: &[ToolSchema], max_tokens: u32) -> Result<AgentTurn, AgentStreamError> {
        let turn = self.request_protocol(messages, tools, max_tokens, false, &mut |_| {}, &|| false)?;
        validate_agent_turn(turn)
    }

    pub(super) fn request_protocol_plain(&self, messages: &[ChatMessage], stream: bool, on_delta: &mut impl FnMut(StreamDelta)) -> KhaslanaResult<ChatResult> {
        let messages = messages.iter().map(|m| match m.role.as_str() {
            "system" | "developer" => AgentChatMessage::System(m.content.clone()),
            "assistant" => AgentChatMessage::Assistant { content: m.content.clone(), tool_calls: vec![] },
            _ => AgentChatMessage::User(m.content.clone()),
        }).collect::<Vec<_>>();
        let turn = self.request_protocol(&messages, &[], self.settings.max_tokens, stream, on_delta, &|| false)
            .map_err(AgentStreamError::into_git_error)?;
        Ok(ChatResult { content: turn.content, reasoning: turn.reasoning })
    }

    pub(super) fn request_protocol(&self, messages: &[AgentChatMessage], tools: &[ToolSchema], max_tokens: u32, stream: bool, on_delta: &mut impl FnMut(StreamDelta), should_stop: &impl Fn() -> bool) -> Result<AgentTurn, AgentStreamError> {
        if should_stop() { return Err(AgentStreamError::fatal_error(ai_error!("ai.protocols.b949b24dc9a1", "AI 请求已中止", "AI request aborted"))); }
        let body = build_body(&self.settings, messages, tools, max_tokens, stream)?;
        let agent = if stream { self.build_streaming_agent() } else { self.build_agent() }.map_err(|e| AgentStreamError::fatal_error(e))?;
        let url = format!("{}{}", self.settings.normalized_base_url(), self.settings.api_type.endpoint_path());
        let mut request = agent.post(&url).header("Content-Type", "application/json").header("User-Agent", AI_USER_AGENT);
        if self.settings.api_type == AiApiType::AnthropicMessages {
            request = request.header("x-api-key", &self.settings.api_key).header("anthropic-version", "2023-06-01");
        } else {
            request = request.header("Authorization", format!("Bearer {}", self.settings.api_key));
        }
        let mut response = request.send_json(body).map_err(classify_agent_http_error)?;
        if should_stop() { return Err(AgentStreamError::fatal_error(ai_error!("ai.protocols.b949b24dc9a1", "AI 请求已中止", "AI request aborted"))); }
        let turn = if stream {
            let mut body = response.into_body();
            sse::read(BufReader::new(body.as_reader()), self.settings.api_type, max_tokens, on_delta, should_stop)?
        } else {
            let value: Value = response.body_mut().read_json().map_err(|e| AgentStreamError::transient_error(ai_error!("ai.protocols.2a3a2b1cc472", "AI 响应解析失败：{e}", "Failed to parse AI response: {e}", "e" => e)))?;
            parse_turn(self.settings.api_type, &value, max_tokens)?
        };
        if should_stop() { return Err(AgentStreamError::fatal_error(ai_error!("ai.protocols.b949b24dc9a1", "AI 请求已中止", "AI request aborted"))); }
        if turn.content.is_empty() && turn.tool_calls.is_empty() && turn.reasoning.is_none() {
            return Err(AgentStreamError::transient_error(agent_turn_empty_error(1, turn.reasoning.is_some())));
        }
        Ok(turn)
    }
}

fn build_body(settings: &AiProviderSettings, messages: &[AgentChatMessage], tools: &[ToolSchema], max_tokens: u32, stream: bool) -> Result<Value, AgentStreamError> {
    // 温度始终按配置发送；模型不兼容应明确失败，不能擅自省略或改值。
    let mut body = json!({"model":settings.model,"temperature":settings.temperature,"stream":stream});
    let effort = settings.reasoning_effort.as_deref().map(str::trim).filter(|s| !s.is_empty());
    match settings.api_type {
        AiApiType::Responses => responses::build(&mut body, messages, tools, max_tokens, effort)?,
        AiApiType::AnthropicMessages => anthropic::build(&mut body, messages, tools, max_tokens, effort)?,
        AiApiType::ChatCompletions => {
            if messages.iter().any(|m| matches!(m, AgentChatMessage::AssistantOpaque { .. })) { return Err(AgentStreamError::fatal_error(ai_error!("ai.protocols.8ea8b5a03e6e", "不能跨 AI 接口回填协议历史，请新建会话", "Cannot replay protocol history across AI APIs; start a new session"))); }
            body["messages"] = serde_json::to_value(messages.iter().map(agent_message_to_request).collect::<Vec<_>>()).unwrap();
            body["max_tokens"] = json!(max_tokens);
            if let Some(effort) = effort { body["reasoning_effort"] = json!(effort); }
            if !tools.is_empty() { body["tools"] = json!(tools.iter().map(|t| json!({"type":"function","function":{"name":t.name,"description":t.description,"parameters":t.parameters}})).collect::<Vec<_>>()); body["tool_choice"] = json!("auto"); }
        }
    }
    Ok(body)
}

fn parse_turn(api: AiApiType, value: &Value, max_tokens: u32) -> Result<AgentTurn, AgentStreamError> {
    if !value["error"].is_null() { return Err(protocol_error(&value["error"])); }
    match api {
        AiApiType::Responses => responses::parse(value, max_tokens),
        AiApiType::AnthropicMessages => anthropic::parse(value, max_tokens),
        AiApiType::ChatCompletions => {
            let choice = value["choices"].as_array().and_then(|a| a.first()).ok_or_else(|| AgentStreamError::transient_error(ai_error!("ai.protocols.7e841b466edc", "AI 响应中没有 choices", "AI response has no choices")))?;
            check_finish(choice["finish_reason"].as_str(), max_tokens)?;
            let m = &choice["message"];
            let (content, reasoning) = split_response_message_parts(m["content"].as_str().map(str::to_owned), m["reasoning_content"].as_str().map(str::to_owned));
            let mut calls = vec![];
            if let Some(items) = m["tool_calls"].as_array() { for c in items { calls.push(call(c, "id", &c["function"], "arguments")?); } }
            Ok(AgentTurn { content: content.trim().into(), reasoning, tool_calls: calls, protocol_history: None })
        }
    }
}

fn check_finish(reason: Option<&str>, max_tokens: u32) -> Result<(), AgentStreamError> {
    match reason {
        Some("length" | "max_tokens" | "max_output_tokens" | "model_context_window_exceeded") => Err(AgentStreamError::transient_once_error(ai_error!("ai.protocols.aa9008bcefe0", "AI 输出触及 max_tokens 上限（{max_tokens}）被截断", "AI output was truncated at the max_tokens limit ({max_tokens})", "max_tokens" => max_tokens))),
        Some("content_filter" | "refusal") => Err(AgentStreamError::fatal_error(ai_error!("ai.protocols.98e67532853a", "AI 输出被安全策略拒绝，内容不完整", "AI output was rejected by a safety policy and is incomplete"))),
        _ => Ok(()),
    }
}
fn protocol_error(v: &Value) -> AgentStreamError {
    let Some(message) = v["message"].as_str() else {
        return AgentStreamError::transient_error(ai_error!("ai.protocol.provider_error", "供应商返回错误", "Provider returned an error"));
    };
    match v["type"].as_str().or(v["code"].as_str()) {
        Some("invalid_request_error" | "authentication_error" | "permission_error" | "not_found_error" | "invalid_api_key" | "unsupported_parameter" | "invalid_value" | "invalid_request") => AgentStreamError::fatal_error(ai_error!("ai.protocols.da5be4b9ab17", "AI 响应错误：{message}", "AI response error: {message}", "message" => message)),
        _ => AgentStreamError::transient_error(ai_error!("ai.protocols.da5be4b9ab17", "AI 响应错误：{message}", "AI response error: {message}", "message" => message)),
    }
}
fn call(v: &Value, id_key: &str, function: &Value, arguments_key: &str) -> Result<AgentToolCall, AgentStreamError> {
    let id = v[id_key].as_str().filter(|s| !s.is_empty()).ok_or_else(|| AgentStreamError::transient_error(ai_error!("ai.protocols.257291087736", "AI 工具调用缺少身份", "AI tool call is missing its identity")))?;
    let name = function["name"].as_str().filter(|s| !s.is_empty()).ok_or_else(|| AgentStreamError::transient_error(ai_error!("ai.protocols.69634c56a089", "AI 工具调用缺少名称", "AI tool call is missing its name")))?;
    let args = &function[arguments_key];
    let arguments = args.as_str().map(str::to_owned).unwrap_or_else(|| args.to_string());
    let parsed = serde_json::from_str::<Value>(&arguments).map_err(|_| AgentStreamError::transient_error(ai_error!("ai.protocols.45052fa9af44", "AI 工具参数被截断或不是有效 JSON", "AI tool arguments are truncated or invalid JSON")))?;
    if !parsed.is_object() { return Err(AgentStreamError::transient_error(ai_error!("ai.protocols.96143672aaf4", "AI 工具参数必须为 JSON 对象", "AI tool arguments must be a JSON object"))); }
    Ok(AgentToolCall { id: id.into(), name: name.into(), arguments })
}

#[cfg(test)]
#[path = "../../tests/ai/client_protocols.rs"]
mod tests;

// 普通请求允许仅有思考链，由生成业务层给出用途相关错误；agent 必须有可执行回合。
pub(super) fn validate_agent_turn(turn: AgentTurn) -> Result<AgentTurn, AgentStreamError> {
    if turn.content.is_empty() && turn.tool_calls.is_empty() {
        return Err(AgentStreamError::transient_error(agent_turn_empty_error(1, turn.reasoning.is_some())));
    }
    Ok(turn)
}
