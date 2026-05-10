#[derive(Debug, Clone, Default)]
pub(crate) struct AnthropicStreamStopMetadata {
    pub(crate) stop_reason: Option<String>,
    pub(crate) stop_sequence: Option<String>,
}

impl AnthropicStreamStopMetadata {
    pub(crate) fn capture_message_delta(&mut self, parsed: &serde_json::Value) {
        self.stop_reason = parsed
            .pointer("/delta/stop_reason")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .or_else(|| self.stop_reason.clone());
        self.stop_sequence = parsed
            .pointer("/delta/stop_sequence")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .or_else(|| self.stop_sequence.clone());
    }
}
