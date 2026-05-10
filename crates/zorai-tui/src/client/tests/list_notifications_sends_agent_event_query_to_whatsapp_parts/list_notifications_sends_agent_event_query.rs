use super::super::drain_request;
use super::whatsapp_link_methods_send_expected_protocol_messages_to_resolve_task::handle_daemon_message_for_test;
use crate::client::AgentStatusSnapshotVm;
use crate::client::{ClientEvent, DaemonClient};
use tokio::sync::mpsc;
use zorai_protocol::{ClientMessage, DaemonMessage};
use zorai_shared::providers::*;
#[tokio::test]
async fn done_event_parses_provider_final_result_payload() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "done",
            "thread_id": "thread-1",
            "input_tokens": 10,
            "output_tokens": 20,
            "provider_final_result": {
                "provider": "open_ai_responses",
                "id": "resp_tui_done"
            }
        }),
        &event_tx,
    )
    .await;

    match event_rx.recv().await.expect("expected done event") {
        ClientEvent::Done {
            provider_final_result_json,
            ..
        } => {
            let json = provider_final_result_json.expect("expected provider final result");
            let value: serde_json::Value =
                serde_json::from_str(&json).expect("parse provider final result json");
            assert_eq!(
                value.get("provider").and_then(|v| v.as_str()),
                Some("open_ai_responses")
            );
            assert_eq!(
                value.get("id").and_then(|v| v.as_str()),
                Some("resp_tui_done")
            );
        }
        other => panic!("expected done event, got {:?}", other),
    }
}

#[tokio::test]
async fn daemon_agent_error_is_forwarded_to_client_error_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentError {
            message: "protected mutation: cannot change WELES name".to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected error event") {
        ClientEvent::Error(message) => {
            assert_eq!(message, "protected mutation: cannot change WELES name");
        }
        other => panic!("expected error event, got {:?}", other),
    }
}

#[tokio::test]
async fn daemon_operation_accepted_is_ignored_without_error() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::OperationAccepted {
            operation_id: "op-tui-1".to_string(),
            kind: "agent_set_sub_agent".to_string(),
            dedup: None,
            revision: 1,
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    assert!(
        event_rx.try_recv().is_err(),
        "operation acceptance should not emit a user-visible TUI event"
    );
}

#[tokio::test]
async fn daemon_provider_validation_with_operation_id_emits_provider_validation_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentProviderValidation {
            operation_id: Some("op-provider-validation-1".to_string()),
            provider_id: PROVIDER_ID_OPENAI.to_string(),
            valid: false,
            error: Some("bad key".to_string()),
            models_json: None,
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx
        .recv()
        .await
        .expect("expected provider validation event")
    {
        ClientEvent::ProviderValidation {
            provider_id,
            valid,
            error,
        } => {
            assert_eq!(provider_id, PROVIDER_ID_OPENAI);
            assert!(!valid);
            assert_eq!(error.as_deref(), Some("bad key"));
        }
        other => panic!("expected provider validation event, got {:?}", other),
    }
}

#[tokio::test]
async fn daemon_models_response_with_operation_id_emits_models_fetched_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentModelsResponse {
            operation_id: Some("op-fetch-models-1".to_string()),
            models_json: r#"[{"id":"gpt-5.4-mini","name":"GPT-5.4 Mini","provider":"openai"}]"#
                .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx
        .recv()
        .await
        .expect("expected models fetched event")
    {
        ClientEvent::ModelsFetched(models) => {
            assert_eq!(models.len(), 1);
            assert_eq!(models[0].id, "gpt-5.4-mini");
            assert_eq!(models[0].name.as_deref(), Some("GPT-5.4 Mini"));
            assert_eq!(models[0].context_window, None);
            assert!(models[0].pricing.is_none());
            assert!(models[0].metadata.is_none());
        }
        other => panic!("expected models fetched event, got {:?}", other),
    }
}

#[tokio::test]
async fn agent_status_response_emits_full_status_event_and_diagnostics() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
            DaemonMessage::AgentStatusResponse {
                tier: "mission_control".to_string(),
                feature_flags_json: "{}".to_string(),
                activity: "waiting_for_operator".to_string(),
                active_thread_id: Some("thread-1".to_string()),
                active_goal_run_id: Some("goal-1".to_string()),
                active_goal_run_title: Some("Close release gap".to_string()),
                provider_health_json: r#"{"openai":{"can_execute":true,"trip_count":0}}"#.to_string(),
                gateway_statuses_json: r#"{"slack":{"status":"connected"}}"#.to_string(),
                recent_actions_json: r#"[{"action_type":"tool_call","summary":"Ran status","timestamp":1712345678}]"#.to_string(),
                diagnostics_json: r#"{"operator_profile_sync_state":"dirty","operator_profile_sync_dirty":true,"operator_profile_scheduler_fallback":false}"#.to_string(),
            },
            &event_tx,
        )
        .await;

    assert!(should_continue);

    match event_rx.recv().await.expect("expected first event") {
        ClientEvent::StatusSnapshot(AgentStatusSnapshotVm {
            tier,
            activity,
            active_thread_id,
            active_goal_run_title,
            provider_health_json,
            ..
        }) => {
            assert_eq!(tier, "mission_control");
            assert_eq!(activity, "waiting_for_operator");
            assert_eq!(active_thread_id.as_deref(), Some("thread-1"));
            assert_eq!(active_goal_run_title.as_deref(), Some("Close release gap"));
            assert!(provider_health_json.contains("openai"));
        }
        other => panic!("expected status snapshot event, got {:?}", other),
    }

    match event_rx.recv().await.expect("expected second event") {
        ClientEvent::StatusDiagnostics {
            operator_profile_sync_state,
            operator_profile_sync_dirty,
            operator_profile_scheduler_fallback,
            diagnostics_json,
        } => {
            assert_eq!(operator_profile_sync_state, "dirty");
            assert!(operator_profile_sync_dirty);
            assert!(!operator_profile_scheduler_fallback);
            assert!(diagnostics_json.contains("operator_profile_sync_state"));
        }
        other => panic!("expected status diagnostics event, got {:?}", other),
    }
}

#[tokio::test]
async fn agent_prompt_inspection_emits_prompt_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentPromptInspection {
            prompt_json: serde_json::json!({
                "agent_id": "swarog",
                "agent_name": "Svarog",
                "provider_id": "openai",
                "model": "gpt-5.4-mini",
                "sections": [{
                    "id": "base_prompt",
                    "title": "Base Prompt",
                    "content": "Custom operator prompt",
                }],
                "final_prompt": "Custom operator prompt\n\n## Runtime Identity",
            })
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx
        .recv()
        .await
        .expect("expected prompt inspection event")
    {
        ClientEvent::PromptInspection(prompt) => {
            assert_eq!(prompt.agent_id, "swarog");
            assert_eq!(prompt.agent_name, "Svarog");
            assert_eq!(prompt.sections.len(), 1);
            assert!(prompt.final_prompt.contains("## Runtime Identity"));
        }
        other => panic!("expected prompt inspection event, got {:?}", other),
    }
}

#[tokio::test]
async fn daemon_openai_codex_auth_replies_emit_client_events() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentOpenAICodexAuthStatus {
            status_json: serde_json::json!({
                "available": false,
                "authMode": "chatgpt_subscription",
                "source": "zorai-daemon",
                "status": "pending",
                "authUrl": "https://auth.openai.com/oauth/authorize?code=123"
            })
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected auth status event") {
        ClientEvent::OpenAICodexAuthStatus(status) => {
            assert!(!status.available);
            assert_eq!(status.auth_mode.as_deref(), Some("chatgpt_subscription"));
            assert_eq!(status.source.as_deref(), Some("zorai-daemon"));
            assert_eq!(status.status.as_deref(), Some("pending"));
            assert!(status
                .auth_url
                .as_deref()
                .is_some_and(|url| url.starts_with("https://auth.openai.com/oauth/authorize")));
        }
        other => panic!("expected auth status event, got {:?}", other),
    }

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentOpenAICodexAuthLoginResult {
            result_json: serde_json::json!({
                "available": false,
                "authMode": "chatgpt_subscription",
                "source": "zorai-daemon",
                "status": "pending",
                "authUrl": "https://auth.openai.com/oauth/authorize?code=456"
            })
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected auth login event") {
        ClientEvent::OpenAICodexAuthLoginResult(status) => {
            assert_eq!(status.status.as_deref(), Some("pending"));
            assert_eq!(status.source.as_deref(), Some("zorai-daemon"));
            assert!(status
                .auth_url
                .as_deref()
                .is_some_and(|url| url.contains("code=456")));
        }
        other => panic!("expected auth login event, got {:?}", other),
    }

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentOpenAICodexAuthLogoutResult {
            ok: true,
            error: None,
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected auth logout event") {
        ClientEvent::OpenAICodexAuthLogoutResult { ok, error } => {
            assert!(ok);
            assert!(error.is_none());
        }
        other => panic!("expected auth logout event, got {:?}", other),
    }
}

#[tokio::test]
async fn weles_health_update_event_parses_degraded_payload() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "weles_health_update",
            "state": "degraded",
            "reason": "WELES review unavailable for guarded actions",
            "checked_at": 321
        }),
        &event_tx,
    )
    .await;

    match event_rx.recv().await.expect("expected weles health event") {
        ClientEvent::WelesHealthUpdate {
            state,
            reason,
            checked_at,
        } => {
            assert_eq!(state, "degraded");
            assert_eq!(checked_at, 321);
            assert_eq!(
                reason.as_deref(),
                Some("WELES review unavailable for guarded actions")
            );
        }
        other => panic!("expected weles health update, got {:?}", other),
    }
}

#[tokio::test]
async fn hidden_handoff_thread_reload_event_is_filtered() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "thread_reload_required",
            "thread_id": "handoff:thread-user:handoff-1"
        }),
        &event_tx,
    )
    .await;

    assert!(event_rx.try_recv().is_err());
}

#[tokio::test]
async fn internal_dm_thread_reload_event_is_forwarded() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "thread_reload_required",
            "thread_id": "dm:svarog:weles"
        }),
        &event_tx,
    )
    .await;

    match tokio::time::timeout(std::time::Duration::from_millis(100), event_rx.recv())
        .await
        .expect("internal dm reload event should arrive")
        .expect("expected internal dm reload event")
    {
        ClientEvent::ThreadReloadRequired { thread_id } => {
            assert_eq!(thread_id, "dm:svarog:weles");
        }
        other => panic!("expected thread reload event, got {:?}", other),
    }
}

#[tokio::test]
async fn internal_dm_done_event_is_forwarded() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "done",
            "thread_id": "dm:svarog:weles",
            "input_tokens": 1,
            "output_tokens": 2,
            "reasoning": "internal reasoning"
        }),
        &event_tx,
    )
    .await;

    match tokio::time::timeout(std::time::Duration::from_millis(100), event_rx.recv())
        .await
        .expect("internal dm done event should arrive")
        .expect("expected internal dm done event")
    {
        ClientEvent::Done {
            thread_id,
            input_tokens,
            output_tokens,
            reasoning,
            ..
        } => {
            assert_eq!(thread_id, "dm:svarog:weles");
            assert_eq!(input_tokens, 1);
            assert_eq!(output_tokens, 2);
            assert_eq!(reasoning.as_deref(), Some("internal reasoning"));
        }
        other => panic!("expected done event, got {:?}", other),
    }
}

#[test]
fn list_notifications_sends_agent_event_query() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client.list_notifications().unwrap();

    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::ListAgentEvents {
            category: Some(category),
            pane_id: None,
            limit: Some(500),
        } if category == "notification"
    ));
}
