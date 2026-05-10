use crate::agent::CompletionServerToolUsage;

#[derive(Debug, Clone, Default)]
pub(crate) struct AnthropicStreamUsage {
    pub(crate) input_tokens: u64,
    pub(crate) output_tokens: u64,
    pub(crate) cache_creation_input_tokens: Option<u64>,
    pub(crate) cache_read_input_tokens: Option<u64>,
    pub(crate) server_tool_use: Option<CompletionServerToolUsage>,
}

impl AnthropicStreamUsage {
    pub(crate) fn capture_message_start(&mut self, parsed: &serde_json::Value) {
        self.input_tokens = parsed
            .pointer("/message/usage/input_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(self.input_tokens);
        self.cache_creation_input_tokens = parsed
            .pointer("/message/usage/cache_creation_input_tokens")
            .and_then(|value| value.as_u64())
            .or(self.cache_creation_input_tokens);
        self.cache_read_input_tokens = parsed
            .pointer("/message/usage/cache_read_input_tokens")
            .and_then(|value| value.as_u64())
            .or(self.cache_read_input_tokens);
        self.server_tool_use = extract_stream_server_tool_usage(parsed.pointer("/message/usage"))
            .or_else(|| self.server_tool_use.clone());
    }

    pub(crate) fn capture_message_delta(&mut self, parsed: &serde_json::Value) {
        self.input_tokens = parsed
            .pointer("/usage/input_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(self.input_tokens);
        self.output_tokens = parsed
            .pointer("/usage/output_tokens")
            .and_then(|value| value.as_u64())
            .unwrap_or(self.output_tokens);
        self.cache_creation_input_tokens = parsed
            .pointer("/usage/cache_creation_input_tokens")
            .and_then(|value| value.as_u64())
            .or(self.cache_creation_input_tokens);
        self.cache_read_input_tokens = parsed
            .pointer("/usage/cache_read_input_tokens")
            .and_then(|value| value.as_u64())
            .or(self.cache_read_input_tokens);
        self.server_tool_use = extract_stream_server_tool_usage(parsed.pointer("/usage"))
            .or_else(|| self.server_tool_use.clone());
    }
}

fn extract_stream_server_tool_usage(
    usage: Option<&serde_json::Value>,
) -> Option<CompletionServerToolUsage> {
    let server_tool_use = usage?.get("server_tool_use")?;
    Some(CompletionServerToolUsage {
        web_fetch_requests: server_tool_use
            .get("web_fetch_requests")
            .and_then(|value| value.as_u64()),
        web_search_requests: server_tool_use
            .get("web_search_requests")
            .and_then(|value| value.as_u64()),
    })
}
