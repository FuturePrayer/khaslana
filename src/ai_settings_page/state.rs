use khaslana::{AiApiType, AiProviderSettings};
use khaslana::ui_i18n::encode;
use super::messages;

pub(crate) const API_TYPES: [AiApiType; 3] = [
    AiApiType::ChatCompletions, AiApiType::Responses, AiApiType::AnthropicMessages,
];

/// 文本先严格解析再验证；无效输入绝不偷偷回退到已保存值。
pub(crate) fn parse_form_settings(
    mut settings: AiProviderSettings,
    temperature: &str,
    max_tokens: &str,
    timeout: &str,
    reasoning_effort: &str,
) -> Result<AiProviderSettings, String> {
    settings.temperature = temperature.trim().parse()
        .map_err(|_| encode(messages::TEMPERATURE_PARSE_ERROR, &[]))?;
    settings.max_tokens = max_tokens.trim().parse()
        .map_err(|_| encode(messages::MAX_OUTPUT_PARSE_ERROR, &[]))?;
    settings.request_timeout_secs = timeout.trim().parse()
        .map_err(|_| encode(messages::TIMEOUT_PARSE_ERROR, &[]))?;
    let effort = reasoning_effort.trim();
    settings.reasoning_effort = (!effort.is_empty()).then(|| effort.to_string());
    settings.validate_advanced().map_err(|error| error.to_string())?;
    Ok(settings)
}

pub(crate) fn form_is_dirty(
    form: &Result<AiProviderSettings, String>, saved: &AiProviderSettings,
) -> bool {
    form.as_ref().map_or(true, |settings| settings != saved)
}

#[derive(Default)]
pub(crate) enum ConnectionTestOutcome {
    #[default]
    Idle,
    Running,
    Passed(String),
    Failed(String),
}

/// 连接测试只展示同一份参数的结果，不能借用全局 Git 操作的状态。
#[derive(Default)]
pub(crate) struct AiConnectionTestState {
    request_id: u64,
    settings: Option<AiProviderSettings>,
    outcome: ConnectionTestOutcome,
}

impl AiConnectionTestState {
    pub(crate) fn begin(&mut self, settings: AiProviderSettings) -> u64 {
        self.request_id = self.request_id.wrapping_add(1);
        self.settings = Some(settings);
        self.outcome = ConnectionTestOutcome::Running;
        self.request_id
    }

    pub(crate) fn finish(&mut self, request_id: u64, result: Result<String, String>) -> bool {
        if request_id != self.request_id || !self.is_running() {
            return false;
        }
        self.outcome = match result {
            Ok(message) => ConnectionTestOutcome::Passed(message),
            Err(error) => ConnectionTestOutcome::Failed(error),
        };
        true
    }

    pub(crate) fn is_running(&self) -> bool {
        matches!(self.outcome, ConnectionTestOutcome::Running)
    }

    /// 调度器全局复位后，旧连接事件不能再解锁后来开始的操作。
    pub(crate) fn interrupt(&mut self, error: String) {
        if self.is_running() {
            self.request_id = self.request_id.wrapping_add(1);
            self.outcome = ConnectionTestOutcome::Failed(error);
        }
    }

    pub(crate) fn outcome_for(&self, settings: &AiProviderSettings) -> &ConnectionTestOutcome {
        if self.settings.as_ref() == Some(settings) {
            &self.outcome
        } else {
            &ConnectionTestOutcome::Idle
        }
    }
}
