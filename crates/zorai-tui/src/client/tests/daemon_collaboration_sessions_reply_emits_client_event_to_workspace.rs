use super::whatsapp_link_methods_send_expected_protocol_messages_to_resolve_task::handle_daemon_message_for_test;
use crate::client::{ClientEvent, DaemonClient};
use serde_json::Value;
use tokio::sync::mpsc;
use zorai_protocol::DaemonMessage;
#[tokio::test]
async fn daemon_collaboration_sessions_reply_emits_client_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentCollaborationSessions {
            sessions_json: serde_json::json!([
                {
                    "id": "session-1",
                    "parent_task_id": "task-1",
                    "disagreements": [
                        {
                            "topic": "deployment strategy",
                            "positions": ["roll forward", "roll back"]
                        }
                    ]
                }
            ])
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx
        .recv()
        .await
        .expect("expected collaboration sessions event")
    {
        ClientEvent::CollaborationSessions { sessions_json } => {
            let parsed: Value =
                serde_json::from_str(&sessions_json).expect("valid collaboration sessions json");
            assert_eq!(parsed[0]["id"], "session-1");
        }
        other => panic!("expected collaboration sessions event, got {:?}", other),
    }
}

#[tokio::test]
async fn daemon_generated_tools_reply_emits_client_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentGeneratedTools {
            tools_json: serde_json::json!([
                {
                    "id": "tool-1",
                    "name": "tool-1",
                    "status": "active"
                }
            ])
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx
        .recv()
        .await
        .expect("expected generated tools event")
    {
        ClientEvent::GeneratedTools { tools_json } => {
            let parsed: Value =
                serde_json::from_str(&tools_json).expect("valid generated tools json");
            assert_eq!(parsed[0]["id"], "tool-1");
            assert_eq!(parsed[0]["status"], "active");
        }
        other => panic!("expected generated tools event, got {:?}", other),
    }
}

#[tokio::test]
async fn daemon_collaboration_vote_result_reply_emits_client_event() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentCollaborationVoteResult {
            report_json: serde_json::json!({
                "session_id": "session-1",
                "resolution": "resolved"
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
        .expect("expected collaboration vote result event")
    {
        ClientEvent::CollaborationVoteResult { report_json } => {
            let parsed: Value =
                serde_json::from_str(&report_json).expect("valid collaboration vote result json");
            assert_eq!(parsed["session_id"], "session-1");
            assert_eq!(parsed["resolution"], "resolved");
        }
        other => panic!("expected collaboration vote result event, got {:?}", other),
    }
}

#[tokio::test]
async fn workspace_task_agent_event_emits_workspace_task_update() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentEvent {
            event_json: serde_json::json!({
                "type": "workspace_task_update",
                "task": {
                    "id": "wtask-1",
                    "workspace_id": "main",
                    "title": "Ship board",
                    "task_type": "thread",
                    "description": "Finish live workspace updates",
                    "definition_of_done": null,
                    "priority": "low",
                    "status": "in_progress",
                    "sort_order": 2,
                    "reporter": "user",
                    "assignee": { "agent": "svarog" },
                    "reviewer": "user",
                    "thread_id": "workspace-thread:wtask-1",
                    "goal_run_id": null,
                    "created_at": 10,
                    "updated_at": 20,
                    "started_at": 15,
                    "completed_at": null,
                    "deleted_at": null,
                    "last_notice_id": "wnotice-1"
                }
            })
            .to_string(),
        },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    let event = tokio::time::timeout(std::time::Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected workspace update before timeout")
        .expect("expected workspace update");
    match event {
        ClientEvent::WorkspaceTaskUpdated(task) => {
            assert_eq!(task.id, "wtask-1");
            assert_eq!(task.workspace_id, "main");
            assert_eq!(task.status, zorai_protocol::WorkspaceTaskStatus::InProgress);
            assert_eq!(
                task.assignee,
                Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string()))
            );
        }
        other => panic!("expected workspace task update event, got {:?}", other),
    }
}

#[tokio::test]
async fn workspace_notice_agent_event_emits_workspace_notice_update() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "workspace_notice_update",
            "notice": {
                "id": "wnotice-1",
                "workspace_id": "main",
                "task_id": "wtask-1",
                "notice_type": "review_failed",
                "message": "Needs clearer acceptance checks",
                "actor": "user",
                "created_at": 30
            }
        }),
        &event_tx,
    )
    .await;

    let event = tokio::time::timeout(std::time::Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected workspace notice before timeout")
        .expect("expected workspace notice");
    match event {
        ClientEvent::WorkspaceNoticeUpdated(notice) => {
            assert_eq!(notice.id, "wnotice-1");
            assert_eq!(notice.task_id, "wtask-1");
            assert_eq!(notice.notice_type, "review_failed");
        }
        other => panic!("expected workspace notice update event, got {:?}", other),
    }
}
