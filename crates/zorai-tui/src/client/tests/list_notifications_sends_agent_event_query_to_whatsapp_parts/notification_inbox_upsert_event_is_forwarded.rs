use crate::client::{ClientEvent, DaemonClient};
use tokio::sync::mpsc;
#[tokio::test]
async fn notification_inbox_upsert_event_is_forwarded() {
    let (event_tx, mut event_rx) = mpsc::channel(8);

    DaemonClient::dispatch_agent_event(
        serde_json::json!({
            "type": "notification_inbox_upsert",
            "notification": {
                "id": "n1",
                "source": "plugin_auth",
                "kind": "plugin_needs_reconnect",
                "title": "Reconnect Gmail",
                "body": "Reconnect required.",
                "subtitle": "gmail",
                "severity": "warning",
                "created_at": 10,
                "updated_at": 20,
                "read_at": null,
                "archived_at": null,
                "deleted_at": null,
                "actions": [],
                "metadata_json": null
            }
        }),
        &event_tx,
    )
    .await;

    match event_rx.recv().await.expect("expected notification event") {
        ClientEvent::NotificationUpsert(notification) => {
            assert_eq!(notification.id, "n1");
            assert_eq!(notification.source, "plugin_auth");
            assert_eq!(notification.title, "Reconnect Gmail");
        }
        other => panic!("expected notification upsert, got {:?}", other),
    }
}
