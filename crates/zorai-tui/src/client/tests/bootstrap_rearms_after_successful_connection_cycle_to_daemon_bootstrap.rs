use super::whatsapp_link_methods_send_expected_protocol_messages_to_resolve_task::handle_daemon_message_for_test;
use crate::client::{ClientEvent, DaemonClient};
use serde_json::Value;
use tokio::sync::mpsc;
use zorai_protocol::DaemonMessage;
#[tokio::test]
async fn daemon_operator_model_replies_emit_client_events() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentOperatorModel {
            model_json: serde_json::json!({
                "version": "1.0",
                "session_count": 4,
                "risk_fingerprint": {
                    "auto_approve_categories": ["git"]
                }
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
        .expect("expected operator model event")
    {
        ClientEvent::OperatorModelSummary { model_json } => {
            let parsed: Value = serde_json::from_str(&model_json).expect("valid operator model");
            assert_eq!(parsed["version"], "1.0");
            assert_eq!(parsed["session_count"], 4);
        }
        other => panic!("expected operator model summary event, got {:?}", other),
    }

    let should_continue = handle_daemon_message_for_test(
        DaemonMessage::AgentOperatorModelReset { ok: true },
        &event_tx,
    )
    .await;

    assert!(should_continue);
    match event_rx
        .recv()
        .await
        .expect("expected operator model reset event")
    {
        ClientEvent::OperatorModelReset { ok } => {
            assert!(ok);
        }
        other => panic!("expected operator model reset event, got {:?}", other),
    }
}

#[test]
fn bootstrap_rearms_after_successful_connection_cycle() {
    assert!(
        !DaemonClient::next_bootstrap_attempted(true, true),
        "a successful connection cycle should re-arm daemon bootstrap for the next disconnect"
    );
    assert!(
        !DaemonClient::next_bootstrap_attempted(false, false),
        "before any bootstrap attempt or successful connection, the latch should stay clear"
    );
    assert!(
        DaemonClient::next_bootstrap_attempted(true, false),
        "once bootstrap already ran in the current disconnected cycle, it should not rerun immediately"
    );
}

#[cfg(unix)]
#[test]
fn daemon_bootstrap_command_detaches_child_session_on_unix() {
    let parent_sid = unsafe { libc::getsid(0) };
    assert!(parent_sid > 0, "parent session id should be available");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let child_sid = runtime.block_on(async {
        let mut command = tokio::process::Command::new("sh");
        command
            .arg("-c")
            .arg("ps -o sid= -p $$")
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped());
        DaemonClient::configure_daemon_bootstrap_command(&mut command);

        let output = command.output().await.expect("spawn detached child");
        assert!(output.status.success(), "child should exit cleanly");

        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<libc::pid_t>()
            .expect("parse child session id")
    });

    assert_ne!(
        child_sid, parent_sid,
        "daemon bootstrap child should run in a detached session"
    );
}
