use super::*;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesCreateRequest {
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    pub input: Vec<OpenAiResponsesInputItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<OpenAiResponsesTool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<OpenAiResponsesToolChoice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<OpenAiResponsesTextConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<OpenAiResponsesReasoning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,
    pub stream: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesTool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum OpenAiResponsesToolChoice {
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesTextConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<OpenAiResponsesTextFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesTextFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    pub name: String,
    pub strict: bool,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesReasoning {
    pub effort: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum OpenAiResponsesInputItem {
    Message(OpenAiResponsesInputMessage),
    FunctionCall(OpenAiResponsesFunctionCall),
    FunctionCallOutput(OpenAiResponsesFunctionCallOutput),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesInputMessage {
    pub role: String,
    pub content: OpenAiResponsesInputContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum OpenAiResponsesInputContent {
    Text(String),
    Blocks(Vec<serde_json::Value>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesFunctionCall {
    #[serde(rename = "type")]
    pub(crate) item_type: OpenAiResponsesFunctionCallItemType,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

impl OpenAiResponsesFunctionCall {
    #[allow(dead_code)]
    pub(crate) fn new(call_id: String, name: String, arguments: String) -> Self {
        Self {
            item_type: OpenAiResponsesFunctionCallItemType::FunctionCall,
            call_id,
            name,
            arguments,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesFunctionCallOutput {
    #[serde(rename = "type")]
    pub(crate) item_type: OpenAiResponsesFunctionCallOutputItemType,
    pub call_id: String,
    pub output: OpenAiResponsesInputContent,
}

impl OpenAiResponsesFunctionCallOutput {
    #[allow(dead_code)]
    pub(crate) fn new(call_id: String, output: OpenAiResponsesInputContent) -> Self {
        Self {
            item_type: OpenAiResponsesFunctionCallOutputItemType::FunctionCallOutput,
            call_id,
            output,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum OpenAiResponsesFunctionCallItemType {
    #[serde(rename = "function_call")]
    FunctionCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum OpenAiResponsesFunctionCallOutputItemType {
    #[serde(rename = "function_call_output")]
    FunctionCallOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesTerminalResponse {
    pub id: String,
    pub object: String,
    pub status: String,
    pub output: Vec<serde_json::Value>,
    pub usage: OpenAiResponsesResponseUsage,
    #[serde(default)]
    pub error: Option<OpenAiResponsesTerminalError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesResponseUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesTerminalError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamResponseRef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamTerminalResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAiResponsesResponseUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<OpenAiResponsesTerminalError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamErrorPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamOutputItemEvent {
    #[serde(default)]
    pub output_index: u32,
    pub item: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamFunctionCallArgumentsDeltaEvent {
    #[serde(default)]
    pub output_index: u32,
    pub delta: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamUnknownEvent {
    pub event_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamCreatedEvent {
    pub response: OpenAiResponsesStreamResponseRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamTextDeltaEvent {
    pub delta: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamReasoningSummaryTextDeltaEvent {
    pub delta: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamTerminalEvent {
    pub response: OpenAiResponsesStreamTerminalResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct OpenAiResponsesStreamErrorEvent {
    pub error: OpenAiResponsesStreamErrorPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OpenAiResponsesStreamEvent {
    ResponseCreated(OpenAiResponsesStreamCreatedEvent),
    ResponseOutputTextDelta(OpenAiResponsesStreamTextDeltaEvent),
    ResponseReasoningSummaryTextDelta(OpenAiResponsesStreamReasoningSummaryTextDeltaEvent),
    ResponseOutputItemAdded(OpenAiResponsesStreamOutputItemEvent),
    ResponseOutputItemDone(OpenAiResponsesStreamOutputItemEvent),
    ResponseFunctionCallArgumentsDelta(OpenAiResponsesStreamFunctionCallArgumentsDeltaEvent),
    ResponseCompleted(OpenAiResponsesStreamTerminalEvent),
    ResponseIncomplete(OpenAiResponsesStreamTerminalEvent),
    ResponseFailed(OpenAiResponsesStreamTerminalEvent),
    Error(OpenAiResponsesStreamErrorEvent),
    ChatCompletionsMismatch { payload: serde_json::Value },
    Unknown(OpenAiResponsesStreamUnknownEvent),
}

impl OpenAiResponsesStreamEvent {
    #[allow(dead_code)]
    pub(crate) fn event_type(&self) -> &str {
        match self {
            Self::ResponseCreated(_) => "response.created",
            Self::ResponseOutputTextDelta(_) => "response.output_text.delta",
            Self::ResponseReasoningSummaryTextDelta(_) => "response.reasoning_summary_text.delta",
            Self::ResponseOutputItemAdded(_) => "response.output_item.added",
            Self::ResponseOutputItemDone(_) => "response.output_item.done",
            Self::ResponseFunctionCallArgumentsDelta(_) => "response.function_call_arguments.delta",
            Self::ResponseCompleted(_) => "response.completed",
            Self::ResponseIncomplete(_) => "response.incomplete",
            Self::ResponseFailed(_) => "response.failed",
            Self::Error(_) => "error",
            Self::ChatCompletionsMismatch { .. } => "chat_completions_mismatch",
            Self::Unknown(event) => &event.event_type,
        }
    }

    pub(crate) fn is_recognized_responses_event(&self) -> bool {
        matches!(
            self,
            Self::ResponseCreated(_)
                | Self::ResponseOutputTextDelta(_)
                | Self::ResponseReasoningSummaryTextDelta(_)
                | Self::ResponseOutputItemAdded(_)
                | Self::ResponseOutputItemDone(_)
                | Self::ResponseFunctionCallArgumentsDelta(_)
                | Self::ResponseCompleted(_)
                | Self::ResponseIncomplete(_)
                | Self::ResponseFailed(_)
                | Self::Error(_)
        )
    }
}

pub(crate) fn parse_openai_responses_stream_event(
    payload: serde_json::Value,
) -> serde_json::Result<OpenAiResponsesStreamEvent> {
    if payload.get("choices").is_some() {
        return Ok(OpenAiResponsesStreamEvent::ChatCompletionsMismatch { payload });
    }

    let event_type = payload
        .get("type")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    match event_type.as_str() {
        "response.created" => {
            serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::ResponseCreated)
        }
        "response.output_text.delta" => {
            serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::ResponseOutputTextDelta)
        }
        "response.reasoning_summary_text.delta" => serde_json::from_value(payload)
            .map(OpenAiResponsesStreamEvent::ResponseReasoningSummaryTextDelta),
        "response.output_item.added" => {
            serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::ResponseOutputItemAdded)
        }
        "response.output_item.done" => {
            serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::ResponseOutputItemDone)
        }
        "response.function_call_arguments.delta" => serde_json::from_value(payload)
            .map(OpenAiResponsesStreamEvent::ResponseFunctionCallArgumentsDelta),
        "response.completed" => {
            serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::ResponseCompleted)
        }
        "response.incomplete" => {
            serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::ResponseIncomplete)
        }
        "response.failed" => {
            serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::ResponseFailed)
        }
        "error" => serde_json::from_value(payload).map(OpenAiResponsesStreamEvent::Error),
        _ => Ok(OpenAiResponsesStreamEvent::Unknown(
            OpenAiResponsesStreamUnknownEvent {
                event_type,
                payload,
            },
        )),
    }
}
