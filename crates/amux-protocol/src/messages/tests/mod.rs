use super::*;
use crate::codec::{AmuxCodec, DaemonCodec};
use bytes::BytesMut;
use serde::Serialize;
use tokio_util::codec::{Decoder, Encoder};

fn assert_bincode_variant_index<T: Serialize>(value: &T, expected_index: u32) {
    let bytes = bincode::serialize(value).unwrap();
    assert!(
        bytes.len() >= 4,
        "bincode payload must include a variant index"
    );
    assert_eq!(
        &bytes[..4],
        &expected_index.to_le_bytes(),
        "variant index changed in the wire format"
    );
}

fn bincode_variant_index<T: Serialize>(value: &T) -> u32 {
    let bytes = bincode::serialize(value).unwrap();
    u32::from_le_bytes(bytes[..4].try_into().unwrap())
}

#[test]
fn agent_provider_validation_bincode_roundtrip() {
    let msg = DaemonMessage::AgentProviderValidation {
        operation_id: None,
        provider_id: "openrouter".to_string(),
        valid: true,
        error: None,
        models_json: None,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentProviderValidation {
            operation_id,
            provider_id,
            valid,
            error,
            models_json,
        } => {
            assert!(operation_id.is_none());
            assert_eq!(provider_id, "openrouter");
            assert!(valid);
            assert!(error.is_none());
            assert!(models_json.is_none());
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn agent_provider_validation_codec_roundtrip() {
    let msg = DaemonMessage::AgentProviderValidation {
        operation_id: Some("op-provider-1".to_string()),
        provider_id: "openrouter".to_string(),
        valid: true,
        error: None,
        models_json: None,
    };
    let mut daemon_codec = DaemonCodec;
    let mut buf = BytesMut::new();
    daemon_codec.encode(msg.clone(), &mut buf).unwrap();
    let mut client_codec = AmuxCodec;
    let decoded = client_codec.decode(&mut buf).unwrap().unwrap();
    match decoded {
        DaemonMessage::AgentProviderValidation {
            operation_id,
            provider_id,
            valid,
            error,
            models_json,
        } => {
            assert_eq!(operation_id.as_deref(), Some("op-provider-1"));
            assert_eq!(provider_id, "openrouter");
            assert!(valid);
            assert!(error.is_none());
            assert!(models_json.is_none());
        }
        other => panic!("decoded wrong variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_provider_validation_with_operation_id() {
    let msg = DaemonMessage::AgentProviderValidation {
        operation_id: Some("op-provider-1".to_string()),
        provider_id: "openrouter".to_string(),
        valid: false,
        error: Some("invalid api key".to_string()),
        models_json: None,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentProviderValidation {
            operation_id,
            provider_id,
            valid,
            error,
            ..
        } => {
            assert_eq!(operation_id.as_deref(), Some("op-provider-1"));
            assert_eq!(provider_id, "openrouter");
            assert!(!valid);
            assert_eq!(error.as_deref(), Some("invalid api key"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_models_response_with_operation_id() {
    let msg = DaemonMessage::AgentModelsResponse {
        operation_id: Some("op-models-1".to_string()),
        models_json: r#"[{"id":"gpt-5.4","label":"GPT-5.4"}]"#.to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentModelsResponse {
            operation_id,
            models_json,
        } => {
            assert_eq!(operation_id.as_deref(), Some("op-models-1"));
            assert!(models_json.contains("gpt-5.4"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_tool_list() {
    let msg = DaemonMessage::AgentToolList {
        result: ToolListResultPublic {
            total: 1,
            limit: 20,
            offset: 0,
            items: vec![ToolDescriptorPublic {
                name: "read_file".to_string(),
                description: "Read file contents".to_string(),
                required: vec!["path".to_string()],
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                })
                .to_string(),
            }],
        },
    };

    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentToolList { result } => {
            assert_eq!(result.total, 1);
            assert_eq!(result.items.len(), 1);
            assert_eq!(result.items[0].name, "read_file");
            let parameters: serde_json::Value =
                serde_json::from_str(&result.items[0].parameters).unwrap();
            assert_eq!(
                parameters
                    .get("required")
                    .and_then(|value| value.as_array())
                    .map(|items| items.len()),
                Some(1)
            );
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_tool_search_result() {
    let msg = DaemonMessage::AgentToolSearchResult {
        result: ToolSearchResultPublic {
            query: "file".to_string(),
            total: 1,
            limit: 20,
            offset: 0,
            items: vec![ToolSearchMatchPublic {
                name: "read_file".to_string(),
                description: "Read file contents".to_string(),
                required: vec!["path".to_string()],
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    }
                })
                .to_string(),
                score: 91,
                matched_fields: vec!["name".to_string()],
            }],
        },
    };

    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentToolSearchResult { result } => {
            assert_eq!(result.query, "file");
            assert_eq!(result.items.len(), 1);
            assert_eq!(result.items[0].score, 91);
            let parameters: serde_json::Value =
                serde_json::from_str(&result.items[0].parameters).unwrap();
            assert_eq!(
                parameters
                    .get("properties")
                    .and_then(|value| value.as_object())
                    .map(|properties| properties.contains_key("path")),
                Some(true)
            );
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_generated_tool_result_with_operation_id() {
    let msg = DaemonMessage::AgentGeneratedToolResult {
        operation_id: Some("op-tool-1".to_string()),
        tool_name: None,
        result_json: r#"{"id":"generated_echo","status":"new"}"#.to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentGeneratedToolResult {
            operation_id,
            tool_name,
            result_json,
        } => {
            assert_eq!(operation_id.as_deref(), Some("op-tool-1"));
            assert!(tool_name.is_none());
            assert!(result_json.contains("generated_echo"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_divergent_session_started_with_operation_id() {
    let msg = DaemonMessage::AgentDivergentSessionStarted {
        operation_id: Some("op-divergent-1".to_string()),
        session_json: serde_json::json!({
            "session_id": "div-123",
            "status": "started",
        })
        .to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentDivergentSessionStarted {
            operation_id,
            session_json,
        } => {
            assert_eq!(operation_id.as_deref(), Some("op-divergent-1"));
            let payload: serde_json::Value = serde_json::from_str(&session_json).unwrap();
            assert_eq!(payload["session_id"], "div-123");
            assert_eq!(payload["status"], "started");
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn client_message_supports_participant_suggestion_send() {
    let msg = ClientMessage::AgentSendParticipantSuggestion {
        thread_id: "thread-1".to_string(),
        suggestion_id: "sugg-1".to_string(),
        session_id: None,
        client_surface: None,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        ClientMessage::AgentSendParticipantSuggestion {
            thread_id,
            suggestion_id,
            ..
        } if thread_id == "thread-1" && suggestion_id == "sugg-1"
    ));
}

#[test]
fn client_message_supports_participant_suggestion_dismiss() {
    let msg = ClientMessage::AgentDismissParticipantSuggestion {
        thread_id: "thread-1".to_string(),
        suggestion_id: "sugg-1".to_string(),
        session_id: None,
        client_surface: None,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        ClientMessage::AgentDismissParticipantSuggestion {
            thread_id,
            suggestion_id,
            ..
        } if thread_id == "thread-1" && suggestion_id == "sugg-1"
    ));
}

#[test]
fn client_message_roundtrips_agent_semantic_query() {
    let msg = ClientMessage::AgentSemanticQuery {
        args_json: serde_json::json!({
            "kind": "packages",
            "root": "/tmp/workspace",
            "limit": 10,
        })
        .to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::AgentSemanticQuery { args_json } => {
            let payload: serde_json::Value = serde_json::from_str(&args_json).unwrap();
            assert_eq!(payload["kind"], "packages");
            assert_eq!(payload["root"], "/tmp/workspace");
            assert_eq!(payload["limit"], 10);
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_semantic_query_result() {
    let msg = DaemonMessage::AgentSemanticQueryResult {
        content: "## Packages\n- cargo: tamux-daemon".to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentSemanticQueryResult { content } => {
            assert!(content.contains("Packages"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn client_message_roundtrips_agent_execute_memory_tool() {
    let msg = ClientMessage::AgentExecuteMemoryTool {
        tool_name: "search_memory".to_string(),
        args_json: serde_json::json!({
            "query": "operator preferences",
            "limit": 5,
        })
        .to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::AgentExecuteMemoryTool {
            tool_name,
            args_json,
        } => {
            let payload: serde_json::Value = serde_json::from_str(&args_json).unwrap();
            assert_eq!(tool_name, "search_memory");
            assert_eq!(payload["query"], "operator preferences");
            assert_eq!(payload["limit"], 5);
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_memory_tool_result() {
    let msg = DaemonMessage::AgentMemoryToolResult {
        content: serde_json::json!({
            "scope": "memory",
            "matches": [],
            "truncated": false,
        })
        .to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentMemoryToolResult { content } => {
            let payload: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(payload["scope"], "memory");
            assert_eq!(payload["truncated"], false);
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn client_message_roundtrips_agent_vote_on_collaboration_disagreement() {
    let msg = ClientMessage::AgentVoteOnCollaborationDisagreement {
        parent_task_id: "task-1".to_string(),
        disagreement_id: "disagree-1".to_string(),
        task_id: "operator".to_string(),
        position: "recommend".to_string(),
        confidence: Some(1.0),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::AgentVoteOnCollaborationDisagreement {
            parent_task_id,
            disagreement_id,
            task_id,
            position,
            confidence,
        } => {
            assert_eq!(parent_task_id, "task-1");
            assert_eq!(disagreement_id, "disagree-1");
            assert_eq!(task_id, "operator");
            assert_eq!(position, "recommend");
            assert_eq!(confidence, Some(1.0));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_collaboration_vote_result() {
    let msg = DaemonMessage::AgentCollaborationVoteResult {
        report_json: serde_json::json!({
            "session_id": "session-1",
            "resolution": "resolved"
        })
        .to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentCollaborationVoteResult { report_json } => {
            let payload: serde_json::Value = serde_json::from_str(&report_json).unwrap();
            assert_eq!(payload["session_id"], "session-1");
            assert_eq!(payload["resolution"], "resolved");
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn client_message_roundtrips_agent_confirm_memory_provenance_entry() {
    let msg = ClientMessage::AgentConfirmMemoryProvenanceEntry {
        entry_id: "old-confirmable".to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::AgentConfirmMemoryProvenanceEntry { entry_id } => {
            assert_eq!(entry_id, "old-confirmable");
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_memory_provenance_confirmed() {
    let msg = DaemonMessage::AgentMemoryProvenanceConfirmed {
        entry_id: "old-confirmable".to_string(),
        confirmed_at: 123,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentMemoryProvenanceConfirmed {
            entry_id,
            confirmed_at,
        } => {
            assert_eq!(entry_id, "old-confirmable");
            assert_eq!(confirmed_at, 123);
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn async_command_capability_roundtrips() {
    let payload = AsyncCommandCapability {
        version: 1,
        supports_operation_acceptance: true,
    };
    let bytes = bincode::serialize(&payload).unwrap();
    let decoded: AsyncCommandCapability = bincode::deserialize(&bytes).unwrap();
    assert_eq!(decoded.version, 1);
    assert!(decoded.supports_operation_acceptance);
}

#[test]
fn client_message_roundtrips_async_command_capability() {
    let msg = ClientMessage::AgentDeclareAsyncCommandCapability {
        capability: AsyncCommandCapability {
            version: 1,
            supports_operation_acceptance: true,
        },
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        ClientMessage::AgentDeclareAsyncCommandCapability { .. }
    ));
}

#[test]
fn daemon_message_roundtrips_async_command_capability_ack() {
    let msg = DaemonMessage::AgentAsyncCommandCapabilityAck {
        capability: AsyncCommandCapability {
            version: 1,
            supports_operation_acceptance: true,
        },
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        DaemonMessage::AgentAsyncCommandCapabilityAck { .. }
    ));
}

#[test]
fn operation_status_snapshot_roundtrips() {
    let snapshot = OperationStatusSnapshot {
        operation_id: "op-1".to_string(),
        kind: "concierge_welcome".to_string(),
        state: OperationLifecycleState::Accepted,
        dedup: Some("concierge:default".to_string()),
        revision: 0,
    };
    let bytes = bincode::serialize(&snapshot).unwrap();
    let decoded: OperationStatusSnapshot = bincode::deserialize(&bytes).unwrap();
    assert_eq!(decoded.operation_id, "op-1");
    assert!(matches!(decoded.state, OperationLifecycleState::Accepted));
}

#[test]
fn daemon_message_roundtrips_operation_accepted() {
    let msg = DaemonMessage::OperationAccepted {
        operation_id: "op-1".to_string(),
        kind: "concierge_welcome".to_string(),
        dedup: Some("concierge:default".to_string()),
        revision: 0,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(decoded, DaemonMessage::OperationAccepted { .. }));
}

#[test]
fn client_message_roundtrips_operation_status_query() {
    let msg = ClientMessage::AgentGetOperationStatus {
        operation_id: "op-1".to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        ClientMessage::AgentGetOperationStatus { .. }
    ));
}

#[test]
fn client_message_roundtrips_explain_action() {
    let msg = ClientMessage::AgentExplainAction {
        action_id: "missing-action".to_string(),
        step_index: None,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(decoded, ClientMessage::AgentExplainAction { .. }));
}

#[test]
fn client_message_roundtrips_start_divergent_session() {
    let msg = ClientMessage::AgentStartDivergentSession {
        problem_statement: "compare rollout strategies".to_string(),
        thread_id: "thread-div-1".to_string(),
        goal_run_id: None,
        custom_framings_json: None,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        ClientMessage::AgentStartDivergentSession { .. }
    ));
}

#[test]
fn daemon_message_roundtrips_operation_status_snapshot() {
    let msg = DaemonMessage::OperationStatus {
        snapshot: OperationStatusSnapshot {
            operation_id: "op-1".to_string(),
            kind: "concierge_welcome".to_string(),
            state: OperationLifecycleState::Started,
            dedup: Some("concierge:default".to_string()),
            revision: 1,
        },
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(decoded, DaemonMessage::OperationStatus { .. }));
}

#[test]
fn daemon_message_keeps_agent_thread_deleted_after_older_thread_responses() {
    let direct_message_index = bincode_variant_index(&DaemonMessage::AgentDirectMessageResponse {
        target: "operator".to_string(),
        thread_id: "thread-1".to_string(),
        response: "ok".to_string(),
        session_id: None,
        provider_final_result_json: None,
    });
    let deleted_index = bincode_variant_index(&DaemonMessage::AgentThreadDeleted {
        thread_id: "thread-1".to_string(),
        deleted: true,
    });

    assert!(
        deleted_index > direct_message_index,
        "new daemon variants must be appended to preserve older wire indices"
    );
}

#[test]
fn client_message_roundtrips_effective_config_state_query() {
    let msg = ClientMessage::AgentGetEffectiveConfigState;
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        ClientMessage::AgentGetEffectiveConfigState
    ));
}

#[test]
fn daemon_message_roundtrips_effective_config_state() {
    let msg = DaemonMessage::AgentEffectiveConfigState {
        state_json: r#"{"reconcile":{"state":"reconciling","desired_revision":2,"effective_revision":1,"last_error":null},"gateway_runtime_connected":false}"#.to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        DaemonMessage::AgentEffectiveConfigState { .. }
    ));
}

#[test]
fn client_message_roundtrips_subsystem_metrics_query() {
    let msg = ClientMessage::AgentGetSubsystemMetrics;
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(decoded, ClientMessage::AgentGetSubsystemMetrics));
}

#[test]
fn client_message_roundtrips_get_openai_codex_auth_status() {
    let msg = ClientMessage::AgentGetOpenAICodexAuthStatus;
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        ClientMessage::AgentGetOpenAICodexAuthStatus
    ));
}

#[test]
fn client_message_roundtrips_login_openai_codex() {
    let msg = ClientMessage::AgentLoginOpenAICodex;
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(decoded, ClientMessage::AgentLoginOpenAICodex));
}

#[test]
fn client_message_roundtrips_logout_openai_codex() {
    let msg = ClientMessage::AgentLogoutOpenAICodex;
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(decoded, ClientMessage::AgentLogoutOpenAICodex));
}

#[test]
fn daemon_message_roundtrips_subsystem_metrics_response() {
    let msg = DaemonMessage::AgentSubsystemMetrics {
        metrics_json: r#"{"plugin_io":{"current_depth":1,"max_depth":2,"rejection_count":1,"accepted_count":3,"started_count":3,"completed_count":1,"failed_count":2,"accepted_to_started_samples":3,"started_to_terminal_samples":3,"last_accepted_to_started_ms":1,"last_started_to_terminal_ms":2}}"#.to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    assert!(matches!(
        decoded,
        DaemonMessage::AgentSubsystemMetrics { .. }
    ));
}

#[test]
fn daemon_message_roundtrips_openai_codex_auth_status() {
    let msg = DaemonMessage::AgentOpenAICodexAuthStatus {
        status_json: r#"{"provider":"openai_codex","state":"authenticated","last_checked_at":"2026-04-01T12:00:00Z"}"#.to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentOpenAICodexAuthStatus { status_json } => {
            assert_eq!(
                status_json,
                r#"{"provider":"openai_codex","state":"authenticated","last_checked_at":"2026-04-01T12:00:00Z"}"#
            );
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_openai_codex_auth_login_result() {
    let msg = DaemonMessage::AgentOpenAICodexAuthLoginResult {
        result_json: r#"{"provider":"openai_codex","login_url":"https://auth.openai.example/device","expires_in_seconds":900}"#.to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { result_json } => {
            assert_eq!(
                result_json,
                r#"{"provider":"openai_codex","login_url":"https://auth.openai.example/device","expires_in_seconds":900}"#
            );
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_openai_codex_auth_logout_result() {
    let msg = DaemonMessage::AgentOpenAICodexAuthLogoutResult {
        ok: false,
        error: Some("no cached codex credentials found".to_string()),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentOpenAICodexAuthLogoutResult { ok, error } => {
            assert!(!ok);
            assert_eq!(error.as_deref(), Some("no cached codex credentials found"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn client_message_roundtrips_login_provider_with_auth_source() {
    let msg = ClientMessage::AgentLoginProvider {
        provider_id: "github-copilot".to_string(),
        api_key: String::new(),
        base_url: "https://api.githubcopilot.com".to_string(),
        auth_source: Some("github_copilot".to_string()),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::AgentLoginProvider {
            provider_id,
            api_key,
            base_url,
            auth_source,
        } => {
            assert_eq!(provider_id, "github-copilot");
            assert!(api_key.is_empty());
            assert_eq!(base_url, "https://api.githubcopilot.com");
            assert_eq!(auth_source.as_deref(), Some("github_copilot"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_provider_login_result() {
    let msg = DaemonMessage::AgentProviderLoginResult {
        provider_id: "github-copilot".to_string(),
        success: true,
        message: Some("Started GitHub Copilot browser login".to_string()),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentProviderLoginResult {
            provider_id,
            success,
            message,
        } => {
            assert_eq!(provider_id, "github-copilot");
            assert!(success);
            assert_eq!(message.as_deref(), Some("Started GitHub Copilot browser login"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_plugin_api_call_result_with_operation_id() {
    let msg = DaemonMessage::PluginApiCallResult {
        operation_id: Some("op-plugin-1".to_string()),
        plugin_name: "api-test".to_string(),
        endpoint_name: "slow".to_string(),
        success: false,
        result: "timed out".to_string(),
        error_type: Some("timeout".to_string()),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::PluginApiCallResult { operation_id, .. } => {
            assert_eq!(operation_id.as_deref(), Some("op-plugin-1"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_skill_import_result_with_operation_id() {
    let msg = DaemonMessage::SkillImportResult {
        operation_id: Some("op-skill-import-1".to_string()),
        success: true,
        message: "Imported community skill 'test-skill' as draft.".to_string(),
        variant_id: Some("variant-1".to_string()),
        scan_verdict: Some("warn".to_string()),
        findings_count: 0,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::SkillImportResult { operation_id, .. } => {
            assert_eq!(operation_id.as_deref(), Some("op-skill-import-1"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_skill_publish_result_with_operation_id() {
    let msg = DaemonMessage::SkillPublishResult {
        operation_id: Some("op-skill-publish-1".to_string()),
        success: true,
        message: "Published skill 'publish-test'.".to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::SkillPublishResult { operation_id, .. } => {
            assert_eq!(operation_id.as_deref(), Some("op-skill-publish-1"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_agent_explanation_with_operation_id() {
    let msg = DaemonMessage::AgentExplanation {
        operation_id: Some("op-explain-1".to_string()),
        explanation_json: r#"{"action_id":"missing-action","source":"fallback"}"#.to_string(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::AgentExplanation {
            operation_id,
            explanation_json,
        } => {
            assert_eq!(operation_id.as_deref(), Some("op-explain-1"));
            assert!(explanation_json.contains("missing-action"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn daemon_message_roundtrips_plugin_oauth_complete_with_operation_id() {
    let msg = DaemonMessage::PluginOAuthComplete {
        operation_id: Some("op-oauth-1".to_string()),
        name: "oauth-test".to_string(),
        success: true,
        error: None,
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::PluginOAuthComplete {
            operation_id,
            name,
            success,
            error,
        } => {
            assert_eq!(operation_id.as_deref(), Some("op-oauth-1"));
            assert_eq!(name, "oauth-test");
            assert!(success);
            assert!(error.is_none());
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn agent_get_thread_round_trip_preserves_message_page_arguments() {
    let msg = ClientMessage::AgentGetThread {
        thread_id: "thread-1".to_string(),
        message_limit: Some(50),
        message_offset: Some(100),
    };

    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::AgentGetThread {
            thread_id,
            message_limit,
            message_offset,
        } => {
            assert_eq!(thread_id, "thread-1");
            assert_eq!(message_limit, Some(50));
            assert_eq!(message_offset, Some(100));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

fn sample_skill_variant() -> SkillVariantPublic {
    SkillVariantPublic {
        variant_id: "sv-001".to_string(),
        skill_name: "git_rebase_workflow".to_string(),
        variant_name: "v1".to_string(),
        relative_path: "drafts/git_rebase_workflow/SKILL.md".to_string(),
        status: "active".to_string(),
        use_count: 12,
        success_count: 10,
        failure_count: 2,
        context_tags: vec!["git".to_string(), "rebase".to_string()],
        created_at: 1700000000,
        updated_at: 1700001000,
    }
}

fn sample_community_skill_entry() -> CommunitySkillEntry {
    CommunitySkillEntry {
        name: "git-rebase-workflow".to_string(),
        description: "Safely rebase a feature branch".to_string(),
        version: "1.2.0".to_string(),
        publisher_id: "abcd1234efgh5678".to_string(),
        publisher_verified: true,
        success_rate: 0.93,
        use_count: 42,
        content_hash: "d34db33f".to_string(),
        tamux_version: "0.1.10".to_string(),
        maturity_at_publish: "proven".to_string(),
        tags: vec!["git".to_string(), "workflow".to_string()],
        published_at: 1700001234,
    }
}

fn sample_skill_discovery_candidate() -> SkillDiscoveryCandidatePublic {
    SkillDiscoveryCandidatePublic {
        variant_id: "local:git_rebase_workflow:v1".to_string(),
        skill_name: "git_rebase_workflow".to_string(),
        variant_name: "v1".to_string(),
        relative_path: "drafts/git_rebase_workflow/SKILL.md".to_string(),
        status: "active".to_string(),
        score: 0.94,
        confidence_tier: "strong".to_string(),
        reasons: vec!["matches git rebase workflow".to_string()],
        matched_intents: vec!["git rebase workflow".to_string()],
        matched_trigger_phrases: vec!["rebase".to_string()],
        context_tags: vec!["git".to_string(), "rebase".to_string()],
        risk_level: "low".to_string(),
        trust_tier: "trusted_builtin".to_string(),
        source_kind: "builtin".to_string(),
        recommended_action: "read_skill git_rebase_workflow".to_string(),
        use_count: 12,
        success_count: 10,
        failure_count: 2,
    }
}

fn sample_skill_discovery_result() -> SkillDiscoveryResultPublic {
    SkillDiscoveryResultPublic {
        query: "git rebase workflow".to_string(),
        normalized_intent: "git rebase workflow".to_string(),
        required: true,
        confidence_tier: "strong".to_string(),
        recommended_action: "read_skill git_rebase_workflow".to_string(),
        requires_approval: false,
        mesh_state: "fresh".to_string(),
        rationale: vec!["matched git rebase workflow".to_string()],
        capability_family: vec!["development".to_string(), "git".to_string()],
        explicit_rationale_required: false,
        workspace_tags: vec!["git".to_string(), "rebase".to_string()],
        candidates: vec![sample_skill_discovery_candidate()],
        next_cursor: Some("cursor:git-rebase".to_string()),
    }
}

#[test]
fn minimal_skill_discovery_result_deserializes_with_defaults() {
    let result_json = serde_json::json!({
        "query": "debug panic",
        "normalized_intent": "debug panic",
        "confidence_tier": "strong",
        "recommended_action": "read_skill",
        "requires_approval": false,
        "mesh_state": "fresh",
        "rationale": ["matched debug intent"],
        "capability_family": ["development", "debugging"],
        "candidates": [{
            "skill_name": "systematic-debugging",
            "score": 93.0,
            "reasons": ["matched debug", "workspace rust", "active variant"],
            "matched_intents": ["debug panic"],
            "matched_trigger_phrases": ["panic"],
            "risk_level": "low",
            "trust_tier": "trusted_builtin",
            "source_kind": "builtin",
            "recommended_action": "read_skill systematic-debugging"
        }]
    })
    .to_string();

    let result: SkillDiscoveryResultPublic = serde_json::from_str(&result_json).unwrap();
    assert_eq!(result.query, "debug panic");
    assert_eq!(result.normalized_intent, "debug panic");
    assert!(!result.required);
    assert_eq!(result.confidence_tier, "strong");
    assert_eq!(result.recommended_action, "read_skill");
    assert!(!result.requires_approval);
    assert_eq!(result.mesh_state, "fresh");
    assert_eq!(result.rationale, vec!["matched debug intent".to_string()]);
    assert_eq!(
        result.capability_family,
        vec!["development".to_string(), "debugging".to_string()]
    );
    assert!(!result.explicit_rationale_required);
    assert!(result.workspace_tags.is_empty());
    assert_eq!(result.candidates.len(), 1);
    let candidate = &result.candidates[0];
    assert_eq!(candidate.variant_id, "");
    assert_eq!(candidate.skill_name, "systematic-debugging");
    assert_eq!(candidate.variant_name, "");
    assert_eq!(candidate.relative_path, "");
    assert_eq!(candidate.status, "");
    assert!((candidate.score - 93.0).abs() < f64::EPSILON);
    assert_eq!(candidate.confidence_tier, "");
    assert_eq!(
        candidate.reasons,
        vec![
            "matched debug".to_string(),
            "workspace rust".to_string(),
            "active variant".to_string()
        ]
    );
    assert!(candidate.context_tags.is_empty());
    assert_eq!(candidate.matched_intents, vec!["debug panic".to_string()]);
    assert_eq!(candidate.matched_trigger_phrases, vec!["panic".to_string()]);
    assert_eq!(candidate.risk_level, "low");
    assert_eq!(candidate.trust_tier, "trusted_builtin");
    assert_eq!(candidate.source_kind, "builtin");
    assert_eq!(
        candidate.recommended_action,
        "read_skill systematic-debugging"
    );
    assert_eq!(candidate.use_count, 0);
    assert_eq!(candidate.success_count, 0);
    assert_eq!(candidate.failure_count, 0);
}

#[test]
fn gateway_register_round_trip() {
    let msg = ClientMessage::GatewayRegister {
        registration: GatewayRegistration {
            gateway_id: "gateway-main".to_string(),
            instance_id: "instance-01".to_string(),
            protocol_version: 1,
            supported_platforms: vec![
                "slack".to_string(),
                "discord".to_string(),
                "telegram".to_string(),
            ],
            process_id: Some(4242),
        },
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::GatewayRegister { registration } => {
            assert_eq!(registration.gateway_id, "gateway-main");
            assert_eq!(registration.instance_id, "instance-01");
            assert_eq!(registration.protocol_version, 1);
            assert_eq!(registration.supported_platforms.len(), 3);
            assert_eq!(registration.process_id, Some(4242));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn gateway_bootstrap_round_trip() {
    let msg = DaemonMessage::GatewayBootstrap {
        payload: GatewayBootstrapPayload {
            bootstrap_correlation_id: "boot-1".to_string(),
            feature_flags: vec![
                "gateway_runtime_ownership".to_string(),
                "gateway_route_persistence".to_string(),
            ],
            providers: vec![GatewayProviderBootstrap {
                platform: "slack".to_string(),
                enabled: true,
                credentials_json: r#"{"token":"secret"}"#.to_string(),
                config_json: r#"{"channel_filter":"C123"}"#.to_string(),
            }],
            continuity: GatewayContinuityState {
                cursors: vec![GatewayCursorState {
                    platform: "slack".to_string(),
                    channel_id: "C123".to_string(),
                    cursor_value: "1712345678.000100".to_string(),
                    cursor_type: "message_ts".to_string(),
                    updated_at_ms: 1_710_000_000_000,
                }],
                thread_bindings: vec![GatewayThreadBindingState {
                    channel_key: "Slack:C123".to_string(),
                    thread_id: Some("thread-123".to_string()),
                    updated_at_ms: 1_710_000_000_001,
                }],
                route_modes: vec![GatewayRouteModeState {
                    channel_key: "Slack:C123".to_string(),
                    route_mode: GatewayRouteMode::Rarog,
                    updated_at_ms: 1_710_000_000_002,
                }],
                health_snapshots: vec![GatewayHealthState {
                    platform: "slack".to_string(),
                    status: GatewayConnectionStatus::Error,
                    last_success_at_ms: Some(1_710_000_000_000),
                    last_error_at_ms: Some(1_710_000_000_100),
                    consecutive_failure_count: 2,
                    last_error: Some("timeout".to_string()),
                    current_backoff_secs: 30,
                }],
            },
        },
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::GatewayBootstrap { payload } => {
            assert_eq!(payload.bootstrap_correlation_id, "boot-1");
            assert_eq!(payload.feature_flags.len(), 2);
            assert_eq!(payload.providers.len(), 1);
            assert_eq!(payload.providers[0].platform, "slack");
            assert_eq!(payload.continuity.cursors.len(), 1);
            assert_eq!(payload.continuity.thread_bindings.len(), 1);
            assert_eq!(payload.continuity.route_modes.len(), 1);
            assert_eq!(payload.continuity.health_snapshots.len(), 1);
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn skill_variant_result_round_trip() {
    let msg = DaemonMessage::SkillListResult {
        variants: vec![sample_skill_variant()],
        next_cursor: Some("cursor:variant-2".to_string()),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::SkillListResult {
            variants,
            next_cursor,
        } => {
            assert_eq!(variants.len(), 1);
            assert_eq!(variants[0].skill_name, "git_rebase_workflow");
            assert_eq!(variants[0].status, "active");
            assert_eq!(next_cursor.as_deref(), Some("cursor:variant-2"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn skill_search_result_round_trip() {
    let msg = DaemonMessage::SkillSearchResult {
        entries: vec![sample_community_skill_entry()],
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::SkillSearchResult { entries } => {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].name, "git-rebase-workflow");
            assert!(entries[0].publisher_verified);
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn client_message_agent_logout_openai_codex_preserves_pre_change_wire_discriminant() {
    let msg = ClientMessage::AgentLogoutOpenAICodex;
    assert_bincode_variant_index(&msg, 166);
}

#[test]
fn daemon_message_agent_openai_codex_auth_logout_result_preserves_pre_change_wire_discriminant() {
    let msg = DaemonMessage::AgentOpenAICodexAuthLogoutResult {
        ok: true,
        error: None,
    };
    assert_bincode_variant_index(&msg, 136);
}

#[test]
fn skill_discover_round_trip() {
    let expected_session_id = SessionId::new_v4();
    let msg = ClientMessage::SkillDiscover {
        query: "git rebase workflow".to_string(),
        session_id: Some(expected_session_id),
        limit: 5,
        cursor: Some("cursor:git-rebase".to_string()),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::SkillDiscover {
            query,
            session_id,
            limit,
            cursor,
        } => {
            assert_eq!(query, "git rebase workflow");
            assert_eq!(session_id, Some(expected_session_id));
            assert_eq!(limit, 5);
            assert_eq!(cursor.as_deref(), Some("cursor:git-rebase"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn skill_discover_result_round_trip() {
    let payload = sample_skill_discovery_result();
    let msg = DaemonMessage::SkillDiscoverResult {
        result_json: serde_json::to_string(&payload).unwrap(),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: DaemonMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        DaemonMessage::SkillDiscoverResult { result_json } => {
            let result: SkillDiscoveryResultPublic = serde_json::from_str(&result_json).unwrap();
            assert_eq!(result.query, "git rebase workflow");
            assert_eq!(result.normalized_intent, "git rebase workflow");
            assert!(result.required);
            assert_eq!(result.confidence_tier, "strong");
            assert_eq!(result.recommended_action, "read_skill git_rebase_workflow");
            assert!(!result.requires_approval);
            assert_eq!(result.mesh_state, "fresh");
            assert_eq!(
                result.rationale,
                vec!["matched git rebase workflow".to_string()]
            );
            assert_eq!(
                result.capability_family,
                vec!["development".to_string(), "git".to_string()]
            );
            assert!(!result.explicit_rationale_required);
            assert_eq!(result.workspace_tags, vec!["git", "rebase"]);
            assert_eq!(result.candidates.len(), 1);
            assert_eq!(result.next_cursor.as_deref(), Some("cursor:git-rebase"));
            assert_eq!(
                result.candidates[0].variant_id,
                "local:git_rebase_workflow:v1"
            );
            assert_eq!(result.candidates[0].skill_name, "git_rebase_workflow");
            assert_eq!(result.candidates[0].variant_name, "v1");
            assert_eq!(
                result.candidates[0].relative_path,
                "drafts/git_rebase_workflow/SKILL.md"
            );
            assert_eq!(result.candidates[0].status, "active");
            assert!((result.candidates[0].score - 0.94).abs() < f64::EPSILON);
            assert_eq!(result.candidates[0].confidence_tier, "strong");
            assert_eq!(
                result.candidates[0].reasons,
                vec!["matches git rebase workflow".to_string()]
            );
            assert_eq!(
                result.candidates[0].matched_intents,
                vec!["git rebase workflow".to_string()]
            );
            assert_eq!(
                result.candidates[0].matched_trigger_phrases,
                vec!["rebase".to_string()]
            );
            assert_eq!(result.candidates[0].context_tags, vec!["git", "rebase"]);
            assert_eq!(result.candidates[0].risk_level, "low");
            assert_eq!(result.candidates[0].trust_tier, "trusted_builtin");
            assert_eq!(result.candidates[0].source_kind, "builtin");
            assert_eq!(
                result.candidates[0].recommended_action,
                "read_skill git_rebase_workflow"
            );
            assert_eq!(result.candidates[0].use_count, 12);
            assert_eq!(result.candidates[0].success_count, 10);
            assert_eq!(result.candidates[0].failure_count, 2);
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn skill_list_round_trip_preserves_cursor() {
    let msg = ClientMessage::SkillList {
        status: Some("active".to_string()),
        limit: 25,
        cursor: Some("cursor:active-page-2".to_string()),
    };
    let bytes = bincode::serialize(&msg).unwrap();
    let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
    match decoded {
        ClientMessage::SkillList {
            status,
            limit,
            cursor,
        } => {
            assert_eq!(status.as_deref(), Some("active"));
            assert_eq!(limit, 25);
            assert_eq!(cursor.as_deref(), Some("cursor:active-page-2"));
        }
        other => panic!("unexpected variant: {:?}", other),
    }
}

#[test]
fn approval_payload_deserializes_legacy_shape_with_defaults() {
    let legacy_json = serde_json::json!({
        "approval_id": "apr_legacy",
        "execution_id": "exec_legacy",
        "command": "echo hello",
        "rationale": "legacy payload",
        "risk_level": "medium",
        "blast_radius": "current session",
        "reasons": ["network access requested"],
        "workspace_id": "workspace-a",
        "allow_network": true
    })
    .to_string();

    let payload: ApprovalPayload = serde_json::from_str(&legacy_json).unwrap();
    assert_eq!(payload.approval_id, "apr_legacy");
    assert_eq!(payload.execution_id, "exec_legacy");
    assert_eq!(payload.transition_kind, None);
    assert_eq!(payload.policy_fingerprint, None);
    assert_eq!(payload.expires_at, None);
    assert!(payload.constraints.is_empty());
    assert_eq!(payload.scope_summary, None);
}

#[test]
fn approval_payload_omits_new_governance_fields_when_absent() {
    let payload = ApprovalPayload {
        approval_id: "apr_minimal".to_string(),
        execution_id: "exec_minimal".to_string(),
        command: "echo hello".to_string(),
        rationale: "minimal payload".to_string(),
        risk_level: "low".to_string(),
        blast_radius: "current session".to_string(),
        reasons: vec!["safe command".to_string()],
        workspace_id: Some("workspace-a".to_string()),
        allow_network: false,
        transition_kind: None,
        policy_fingerprint: None,
        expires_at: None,
        constraints: Vec::new(),
        scope_summary: None,
    };

    let json = serde_json::to_value(&payload).unwrap();
    let object = json.as_object().unwrap();
    assert!(!object.contains_key("transition_kind"));
    assert!(!object.contains_key("policy_fingerprint"));
    assert!(!object.contains_key("expires_at"));
    assert!(!object.contains_key("constraints"));
    assert!(!object.contains_key("scope_summary"));
}

#[test]
fn approval_payload_round_trips_governance_metadata() {
    let payload = ApprovalPayload {
        approval_id: "apr_governed".to_string(),
        execution_id: "exec_governed".to_string(),
        command: "sudo terraform destroy".to_string(),
        rationale: "apply risky infra change".to_string(),
        risk_level: "high".to_string(),
        blast_radius: "network and workspace".to_string(),
        reasons: vec!["risk score exceeded approval threshold".to_string()],
        workspace_id: Some("workspace-a".to_string()),
        allow_network: true,
        transition_kind: Some("managed_command_dispatch".to_string()),
        policy_fingerprint: Some("fingerprint-123".to_string()),
        expires_at: Some(1_717_171_717),
        constraints: vec!["serial_only_execution".to_string()],
        scope_summary: Some("managed transition".to_string()),
    };

    let encoded = serde_json::to_string(&payload).unwrap();
    let decoded: ApprovalPayload = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        decoded.transition_kind.as_deref(),
        Some("managed_command_dispatch")
    );
    assert_eq!(
        decoded.policy_fingerprint.as_deref(),
        Some("fingerprint-123")
    );
    assert_eq!(decoded.expires_at, Some(1_717_171_717));
    assert_eq!(
        decoded.constraints,
        vec!["serial_only_execution".to_string()]
    );
    assert_eq!(decoded.scope_summary.as_deref(), Some("managed transition"));
}
