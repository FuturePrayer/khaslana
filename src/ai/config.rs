use crate::ai::messages::ai_error;
// AI 供应商配置：接口类型、连接参数、模型选择。

use serde::{Deserialize, Serialize};

use crate::types::Result as KhaslanaResult;

/// AI 供应商接口类型。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiApiType {
    #[default]
    /// OpenAI 兼容的 `/chat/completions` 接口。
    ChatCompletions,
    /// OpenAI `/responses` 接口。
    Responses,
    /// Anthropic `/messages` 接口。
    AnthropicMessages,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiProviderSettings {
    /// 是否启用 AI 功能；默认关闭，用户配置后显式开启。
    pub enabled: bool,
    pub api_type: AiApiType,
    /// 例如 https://api.openai.com/v1 或 https://api.deepseek.com。
    pub base_url: String,
    /// API Key；第一版明文存配置，后续可迁移到 keyring。
    pub api_key: String,
    /// 模型名，例如 gpt-4o-mini / deepseek-chat。
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    /// 请求超时秒数，覆盖网络较慢或 reasoning 模型长输出场景。
    pub request_timeout_secs: u64,
    /// 服务端推理强度；允许供应商自定义值，旧存档缺省为 None。
    #[serde(default)]
    pub reasoning_effort: Option<String>,
}

impl AiApiType {
    /// 该接口类型对应的 API 路径（相对于 base_url）。
    pub fn endpoint_path(self) -> &'static str {
        match self {
            Self::ChatCompletions => "/chat/completions",
            Self::Responses => "/responses",
            Self::AnthropicMessages => "/messages",
        }
    }

    /// 设置弹窗中显示的接口类型中文名。
    pub fn label(self) -> &'static str {
        match self {
            Self::ChatCompletions => "Chat Completions (/chat/completions)",
            Self::Responses => "Responses (/responses)",
            Self::AnthropicMessages => "Anthropic Messages (/messages)",
        }
    }
}

impl Default for AiProviderSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            api_type: AiApiType::default(),
            base_url: String::new(),
            api_key: String::new(),
            model: String::new(),
            temperature: 0.3,
            // 所有 AI 场景的单次请求共用此输出上限；推理模型的思考与正文
            // 可能共享预算，用户可根据模型要求在高级设置中调整。
            max_tokens: 16_384,
            request_timeout_secs: 60,
            reasoning_effort: None,
        }
    }
}

impl AiProviderSettings {
    /// 高级数值按协议校验，保存与测试不得绕过；模型更严约束由服务端判断。
    pub fn validate_advanced(&self) -> KhaslanaResult<()> {
        let max_temperature = if self.api_type == AiApiType::AnthropicMessages { 1.0 } else { 2.0 };
        if !self.temperature.is_finite() || !(0.0..=max_temperature).contains(&self.temperature) {
            return Err(ai_error!("ai.config.dc86ee108bc4", "温度必须是 0–{max_temperature} 之间的有限数值", "Temperature must be a finite number between 0 and {max_temperature}", "max_temperature" => max_temperature));
        }
        if self.max_tokens == 0 {
            return Err(ai_error!("ai.config.df92f63a590a", "最大输出必须是大于 0 的整数", "Maximum output must be an integer greater than 0"));
        }
        if self.request_timeout_secs == 0 {
            return Err(ai_error!("ai.config.a3ff0291adb1", "请求超时必须是大于 0 的整数秒数", "Request timeout must be an integer number of seconds greater than 0"));
        }
        Ok(())
    }

    /// 校验配置字段；保留消息身份，在 UI 或任务边界按目标语言展示。
    pub fn validate(&self) -> KhaslanaResult<()> {
        self.validate_advanced()?;
        if self.base_url.trim().is_empty() {
            return Err(ai_error!("ai.config.92a068b1cbd1", "请填写 Base URL", "Enter a Base URL"));
        }
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(ai_error!("ai.config.f63e701a8b7b", "Base URL 必须以 http:// 或 https:// 开头", "Base URL must start with http:// or https://"));
        }
        // API Key 允许为空（部分自部署/本地模型如 Ollama 不需要鉴权）。
        if self.model.trim().is_empty() {
            return Err(ai_error!("ai.config.090051697c46", "请填写模型名称", "Enter a model name"));
        }
        Ok(())
    }

    /// 是否已配置到可用状态（启用且字段校验通过）。
    pub fn is_usable(&self) -> bool {
        self.enabled && self.validate().is_ok()
    }

    /// 去掉 base_url 末尾多余的斜杠，避免拼出 `//chat/completions`。
    pub fn normalized_base_url(&self) -> String {
        self.base_url.trim().trim_end_matches('/').to_string()
    }
}

#[cfg(test)]
#[path = "../tests/ai/config.rs"]
mod tests;
