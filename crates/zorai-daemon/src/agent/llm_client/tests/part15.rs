use super::*;
#[test]
fn responses_protocol_request_serializes_minimal_create_body() {
    let request = OpenAiResponsesCreateRequest {
        model: "gpt-5.4".to_string(),
        instructions: Some("system prompt".to_string()),
        input: vec![OpenAiResponsesInputItem::Message(
            OpenAiResponsesInputMessage {
                role: "user".to_string(),
                content: OpenAiResponsesInputContent::Text("ping".to_string()),
            },
        )],
        previous_response_id: None,
        tools: Vec::new(),
        tool_choice: None,
        text: None,
        reasoning: None,
        store: None,
        include: Vec::new(),
        stream: true,
    };

    let serialized = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(
        serialized,
        serde_json::json!({
            "model": "gpt-5.4",
            "instructions": "system prompt",
            "input": [
                {
                    "role": "user",
                    "content": "ping"
                }
            ],
            "stream": true
        })
    );
}

#[test]
fn responses_protocol_request_serializes_previous_response_id_when_present() {
    let request = OpenAiResponsesCreateRequest {
        model: "gpt-5.4".to_string(),
        instructions: None,
        input: vec![OpenAiResponsesInputItem::Message(
            OpenAiResponsesInputMessage {
                role: "user".to_string(),
                content: OpenAiResponsesInputContent::Text("ping".to_string()),
            },
        )],
        previous_response_id: Some("resp_123".to_string()),
        tools: Vec::new(),
        tool_choice: None,
        text: None,
        reasoning: None,
        store: None,
        include: Vec::new(),
        stream: true,
    };

    let serialized = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(serialized["previous_response_id"], "resp_123");
}

#[test]
fn responses_protocol_request_omits_instructions_when_none() {
    let request = OpenAiResponsesCreateRequest {
        model: "gpt-5.4".to_string(),
        instructions: None,
        input: vec![OpenAiResponsesInputItem::Message(
            OpenAiResponsesInputMessage {
                role: "user".to_string(),
                content: OpenAiResponsesInputContent::Text("ping".to_string()),
            },
        )],
        previous_response_id: None,
        tools: Vec::new(),
        tool_choice: None,
        text: None,
        reasoning: None,
        store: None,
        include: Vec::new(),
        stream: true,
    };

    let serialized = serde_json::to_value(&request).expect("serialize request");

    assert!(serialized.get("instructions").is_none());
}

#[test]
fn responses_protocol_completed_response_round_trips_with_error_shape() {
    let original = serde_json::json!({
        "id": "resp_123",
        "object": "response",
        "status": "completed",
        "output": [],
        "usage": {
            "input_tokens": 1,
            "output_tokens": 2,
            "total_tokens": 3
        },
        "error": null
    });

    let parsed: OpenAiResponsesTerminalResponse =
        serde_json::from_value(original).expect("deserialize terminal response");
    let serialized = serde_json::to_value(&parsed).expect("serialize terminal response");

    assert_eq!(parsed.status, "completed");
    assert_eq!(parsed.object, "response");
    assert!(parsed.error.is_none());
    assert_eq!(
        serialized,
        serde_json::json!({
            "id": "resp_123",
            "object": "response",
            "status": "completed",
            "output": [],
            "usage": {
                "input_tokens": 1,
                "output_tokens": 2,
                "total_tokens": 3
            },
            "error": null
        })
    );
}

#[test]
fn responses_protocol_function_call_item_round_trips() {
    let original = serde_json::json!({
        "type": "function_call",
        "call_id": "call_123",
        "name": "lookup_weather",
        "arguments": "{\"city\":\"Berlin\"}"
    });

    let parsed: OpenAiResponsesInputItem =
        serde_json::from_value(original.clone()).expect("deserialize function call item");
    let serialized = serde_json::to_value(&parsed).expect("serialize function call item");

    assert_eq!(
        parsed,
        OpenAiResponsesInputItem::FunctionCall(OpenAiResponsesFunctionCall::new(
            "call_123".to_string(),
            "lookup_weather".to_string(),
            "{\"city\":\"Berlin\"}".to_string(),
        ))
    );
    assert_eq!(serialized, original);
}

#[test]
fn responses_protocol_function_call_output_item_round_trips_with_blocks() {
    let original = serde_json::json!({
        "type": "function_call_output",
        "call_id": "call_123",
        "output": [
            {
                "type": "output_text",
                "text": "72F and sunny"
            }
        ]
    });

    let parsed: OpenAiResponsesInputItem =
        serde_json::from_value(original.clone()).expect("deserialize function call output item");
    let serialized = serde_json::to_value(&parsed).expect("serialize function call output item");

    assert_eq!(
        parsed,
        OpenAiResponsesInputItem::FunctionCallOutput(OpenAiResponsesFunctionCallOutput::new(
            "call_123".to_string(),
            OpenAiResponsesInputContent::Blocks(vec![serde_json::json!({
                "type": "output_text",
                "text": "72F and sunny"
            })]),
        ))
    );
    assert_eq!(serialized, original);
}

#[test]
fn responses_protocol_message_item_round_trips_with_blocks_content() {
    let original = serde_json::json!({
        "role": "assistant",
        "content": [
            {
                "type": "output_text",
                "text": "hello"
            },
            {
                "type": "input_image",
                "image_url": "https://example.test/cat.png"
            }
        ]
    });

    let parsed: OpenAiResponsesInputItem =
        serde_json::from_value(original.clone()).expect("deserialize message item with blocks");
    let serialized = serde_json::to_value(&parsed).expect("serialize message item with blocks");

    assert_eq!(
        parsed,
        OpenAiResponsesInputItem::Message(OpenAiResponsesInputMessage {
            role: "assistant".to_string(),
            content: OpenAiResponsesInputContent::Blocks(vec![
                serde_json::json!({
                    "type": "output_text",
                    "text": "hello"
                }),
                serde_json::json!({
                    "type": "input_image",
                    "image_url": "https://example.test/cat.png"
                }),
            ]),
        })
    );
    assert_eq!(serialized, original);
}

#[test]
fn responses_protocol_terminal_response_round_trips_non_null_error() {
    let original = serde_json::json!({
        "id": "resp_456",
        "object": "response",
        "status": "failed",
        "output": [],
        "usage": {
            "input_tokens": 4,
            "output_tokens": 0,
            "total_tokens": 4
        },
        "error": {
            "code": "rate_limit_exceeded",
            "message": "Rate limit exceeded"
        }
    });

    let parsed: OpenAiResponsesTerminalResponse =
        serde_json::from_value(original.clone()).expect("deserialize failed terminal response");
    let serialized = serde_json::to_value(&parsed).expect("serialize failed terminal response");

    assert_eq!(parsed.status, "failed");
    assert_eq!(
        parsed.error,
        Some(OpenAiResponsesTerminalError {
            code: "rate_limit_exceeded".to_string(),
            message: "Rate limit exceeded".to_string(),
        })
    );
    assert_eq!(serialized, original);
}
