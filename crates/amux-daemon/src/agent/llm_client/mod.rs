//! LLM API client with SSE streaming support.
//!
//! Supports two API formats:
//! - OpenAI-compatible (`/chat/completions` with Bearer auth) — covers most providers
//! - Anthropic Messages API (`/v1/messages` with `x-api-key` header)

include!("prelude.rs");
include!("api_types.rs");
include!("public_api.rs");
include!("openai_transport.rs");
include!("openai_responses_protocol.rs");
include!("native_assistant.rs");
include!("openai_runtime.rs");
pub mod provider_capabilities;
include!("openai_sse.rs");
include!("anthropic_response_types.rs");
include!("anthropic_request_fields.rs");
include!("anthropic_stream_usage.rs");
include!("anthropic_stream_stop.rs");
include!("anthropic_stream_message_start.rs");
include!("anthropic_stream_upstream_message.rs");
include!("anthropic.rs");
include!("anthropic_batches.rs");
include!("helpers.rs");

#[cfg(test)]
mod tests;
