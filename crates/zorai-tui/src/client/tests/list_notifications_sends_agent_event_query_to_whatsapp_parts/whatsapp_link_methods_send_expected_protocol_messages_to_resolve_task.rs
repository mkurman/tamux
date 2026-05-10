use super::super::drain_request;
use crate::client::{ClientEvent, DaemonClient};
use tokio::sync::mpsc;
use zorai_protocol::{ClientMessage, DaemonMessage};
use zorai_shared::providers::PROVIDER_ID_GITHUB_COPILOT;

pub async fn handle_daemon_message_for_test(
    message: DaemonMessage,
    event_tx: &mpsc::Sender<ClientEvent>,
) -> bool {
    let mut thread_detail_chunks = None;
    DaemonClient::handle_daemon_message(message, event_tx, &mut thread_detail_chunks).await
}

#[test]
fn whatsapp_link_methods_send_expected_protocol_messages() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client.whatsapp_link_start().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentWhatsAppLinkStart
    ));

    client.whatsapp_link_status().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentWhatsAppLinkStatus
    ));

    client.whatsapp_link_subscribe().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentWhatsAppLinkSubscribe
    ));

    client.whatsapp_link_unsubscribe().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentWhatsAppLinkUnsubscribe
    ));

    client.whatsapp_link_reset().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentWhatsAppLinkReset
    ));

    client.whatsapp_link_stop().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentWhatsAppLinkStop
    ));
}

#[test]
fn openai_codex_auth_methods_send_expected_protocol_messages() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client.get_openai_codex_auth_status().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentGetOpenAICodexAuthStatus
    ));

    client.login_openai_codex().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentLoginOpenAICodex
    ));

    client.logout_openai_codex().unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentLogoutOpenAICodex
    ));
}

#[test]
fn pin_methods_send_expected_protocol_messages() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client
        .pin_thread_message_for_compaction("thread-1".to_string(), "message-1".to_string())
        .unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentPinThreadMessageForCompaction { thread_id, message_id }
            if thread_id == "thread-1" && message_id == "message-1"
    ));

    client
        .unpin_thread_message_for_compaction("thread-1".to_string(), "message-1".to_string())
        .unwrap();
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentUnpinThreadMessageForCompaction { thread_id, message_id }
            if thread_id == "thread-1" && message_id == "message-1"
    ));
}

#[test]
fn refresh_requests_thread_list_with_internal_threads_included() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client.refresh().unwrap();

    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentListThreads {
            limit: None,
            offset: None,
            include_internal: true,
            agent_filter: Some(_),
        }
    ));
}

#[test]
fn get_config_requests_agent_config_immediately() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client.get_config().unwrap();

    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentGetConfig
    ));
}

#[test]
fn refresh_services_excludes_agent_config_request() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client.refresh_services().unwrap();

    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentListTasks
    ));
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentListGoalRuns {
            limit: None,
            offset: None,
        }
    ));
    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentHeartbeatGetItems
    ));
    assert!(
        rx.try_recv().is_err(),
        "refresh_services should no longer enqueue AgentGetConfig on the startup-critical lane"
    );
}

#[test]
fn oversized_send_message_is_rejected_before_queueing() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    let err = client
        .send_message(
            Some("thread-oversized".to_string()),
            "x".repeat(zorai_protocol::MAX_IPC_FRAME_SIZE_BYTES + 1024),
            None,
            None,
            None,
        )
        .expect_err("oversized message should be rejected locally");

    assert!(err.to_string().contains("too large for IPC"));
    assert!(
        rx.try_recv().is_err(),
        "oversized request should never be queued"
    );
}

#[test]
fn resolve_task_approval_uses_agent_protocol_message() {
    let (event_tx, _event_rx) = mpsc::channel(8);
    let client = DaemonClient::new(event_tx);
    let mut rx = client.request_rx.lock().unwrap().take().unwrap();

    client
        .resolve_task_approval(
            "policy-escalation-thread_abc-123".to_string(),
            "allow_session".to_string(),
        )
        .unwrap();

    assert!(matches!(
        drain_request(&mut rx),
        ClientMessage::AgentResolveTaskApproval { approval_id, decision }
            if approval_id == "policy-escalation-thread_abc-123"
                && decision == "approve-session"
    ));
}

#[tokio::test]
async fn task_list_accepts_budget_exceeded_status() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentTaskList {
            tasks_json: serde_json::json!([{
                "id": "task-budget",
                "title": "Task budget exceeded",
                "status": "budget_exceeded"
            }])
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected task list event") {
        ClientEvent::TaskList(tasks) => {
            assert_eq!(tasks.len(), 1);
            assert_eq!(
                tasks[0].status,
                Some(crate::wire::TaskStatus::BudgetExceeded)
            );
        }
        other => panic!("expected task list event, got {:?}", other),
    }
}

#[tokio::test]
async fn goal_run_detail_placeholder_payload_carries_requested_id() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentGoalRunDetail {
            goal_run_json: serde_json::json!({
                "id": "goal-1",
            })
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected goal detail event") {
        ClientEvent::GoalRunDetail(Some(goal_run)) => {
            assert_eq!(goal_run.id, "goal-1");
            assert!(goal_run.title.is_empty());
            assert!(goal_run.status.is_none());
        }
        other => panic!("expected goal run detail event, got {:?}", other),
    }
}

#[tokio::test]
async fn goal_run_update_placeholder_payload_is_marked_sparse() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentEvent {
            event_json: serde_json::json!({
                "type": "goal_run_update",
                "goal_run_id": "goal-1",
                "message": "Goal update",
                "current_step_index": 4
            })
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected goal update event") {
        ClientEvent::GoalRunUpdate(goal_run) => {
            assert_eq!(goal_run.id, "goal-1");
            assert_eq!(goal_run.title, "Goal run update");
            assert!(goal_run.last_error.is_none());
            assert_eq!(goal_run.current_step_index, 4);
            assert!(goal_run.sparse_update);
        }
        other => panic!("expected goal run update event, got {:?}", other),
    }
}

#[tokio::test]
async fn todo_update_applies_top_level_step_index_to_items_missing_it() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "todo_update",
            "thread_id": "thread-1",
            "goal_run_id": "goal-1",
            "step_index": 2,
            "items": [
                {
                    "id": "todo-1",
                    "content": "Verify note contents",
                    "status": "in_progress",
                    "position": 0
                }
            ]
        }),
        &event_tx,
    )
    .await;

    match event_rx.recv().await.expect("expected thread todos event") {
        ClientEvent::ThreadTodos {
            thread_id,
            goal_run_id,
            step_index,
            items,
        } => {
            assert_eq!(thread_id, "thread-1");
            assert_eq!(goal_run_id.as_deref(), Some("goal-1"));
            assert_eq!(step_index, Some(2));
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].id, "todo-1");
            assert_eq!(items[0].step_index, Some(2));
        }
        other => panic!("expected thread todos event, got {:?}", other),
    }
}

#[tokio::test]
async fn context_window_update_agent_event_emits_client_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "context_window_update",
            "thread_id": "thread-1",
            "active_context_window_start": 2,
            "active_context_window_end": 7,
            "active_context_window_tokens": 1234
        }),
        &event_tx,
    )
    .await;

    match event_rx
        .recv()
        .await
        .expect("expected context window event")
    {
        ClientEvent::ContextWindowUpdate {
            thread_id,
            active_context_window_start,
            active_context_window_end,
            active_context_window_tokens,
        } => {
            assert_eq!(thread_id, "thread-1");
            assert_eq!(active_context_window_start, 2);
            assert_eq!(active_context_window_end, 7);
            assert_eq!(active_context_window_tokens, 1234);
        }
        other => panic!("expected context window event, got {:?}", other),
    }
}

#[tokio::test]
async fn checkpoint_list_event_carries_goal_id_when_empty() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentCheckpointList {
            goal_run_id: "goal-1".to_string(),
            checkpoints_json: "[]".to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected checkpoints event") {
        ClientEvent::GoalRunCheckpoints {
            goal_run_id,
            checkpoints,
        } => {
            assert_eq!(goal_run_id, "goal-1");
            assert!(checkpoints.is_empty());
        }
        other => panic!("expected checkpoints event, got {:?}", other),
    }
}

#[tokio::test]
async fn goal_run_controlled_ack_emits_client_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentGoalRunControlled {
            goal_run_id: "goal-1".to_string(),
            ok: true,
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx.recv().await.expect("expected goal control event") {
        ClientEvent::GoalRunControlled { goal_run_id, ok } => {
            assert_eq!(goal_run_id, "goal-1");
            assert!(ok);
        }
        other => panic!("expected goal control event, got {:?}", other),
    }
}

#[tokio::test]
async fn done_event_parses_reasoning_payload() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "done",
            "thread_id": "thread-1",
            "input_tokens": 10,
            "output_tokens": 20,
            "provider": PROVIDER_ID_GITHUB_COPILOT,
            "model": "gpt-5.4",
            "reasoning": "Final reasoning summary"
        }),
        &event_tx,
    )
    .await;

    match event_rx.recv().await.expect("expected done event") {
        ClientEvent::Done {
            thread_id,
            reasoning,
            ..
        } => {
            assert_eq!(thread_id, "thread-1");
            assert_eq!(reasoning.as_deref(), Some("Final reasoning summary"));
        }
        other => panic!("expected done event, got {:?}", other),
    }
}

#[tokio::test]
async fn thread_message_pin_result_emits_budget_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    handle_daemon_message_for_test(
        DaemonMessage::AgentThreadMessagePinResult {
            result_json: serde_json::json!({
                "ok": false,
                "thread_id": "thread-1",
                "message_id": "message-1",
                "error": "pinned_budget_exceeded",
                "current_pinned_chars": 100,
                "pinned_budget_chars": 120,
                "candidate_pinned_chars": 160
            })
            .to_string(),
        },
        &event_tx,
    )
    .await;

    match event_rx.recv().await.expect("expected pin result event") {
        ClientEvent::ThreadMessagePinResult(result) => {
            assert!(!result.ok);
            assert_eq!(result.thread_id, "thread-1");
            assert_eq!(result.message_id, "message-1");
            assert_eq!(result.error.as_deref(), Some("pinned_budget_exceeded"));
            assert_eq!(result.current_pinned_chars, 100);
            assert_eq!(result.pinned_budget_chars, 120);
            assert_eq!(result.candidate_pinned_chars, Some(160));
        }
        other => panic!("expected pin result event, got {:?}", other),
    }
}
