#[tokio::test]
async fn gateway_send_results_use_canonical_discord_dm_channel_keys() {
    let mut config = AgentConfig::default();
    config.gateway.enabled = true;
    let mut conn = spawn_test_connection_with_config(config).await;
    let correlation_id = register_gateway(&mut conn).await;
    acknowledge_gateway_bootstrap(&mut conn, correlation_id).await;
    conn.agent.init_gateway().await;

    let agent = conn.agent.clone();
    let send_task = tokio::spawn(async move {
        agent
            .request_gateway_send(GatewaySendRequest {
                correlation_id: "discord-dm-send".to_string(),
                platform: "discord".to_string(),
                channel_id: "user:123456789".to_string(),
                thread_id: Some("987654321".to_string()),
                content: "hello".to_string(),
            })
            .await
    });

    let request = match conn.recv().await {
        DaemonMessage::GatewaySendRequest { request } => request,
        other => panic!("expected GatewaySendRequest, got {other:?}"),
    };
    assert_eq!(request.channel_id, "user:123456789");

    conn.framed
        .send(ClientMessage::GatewaySendResult {
            result: amux_protocol::GatewaySendResult {
                correlation_id: request.correlation_id.clone(),
                platform: "discord".to_string(),
                channel_id: "DM123".to_string(),
                requested_channel_id: Some("user:123456789".to_string()),
                delivery_id: Some("delivery-1".to_string()),
                ok: true,
                error: None,
                completed_at_ms: 1234,
            },
        })
        .await
        .expect("send discord gateway result");

    let result = send_task
        .await
        .expect("join discord send task")
        .expect("gateway send should complete");
    assert!(result.ok);

    let gw_guard = conn.agent.gateway_state.lock().await;
    let gw = gw_guard.as_ref().expect("gateway state should exist");
    assert!(gw.last_response_at.contains_key("Discord:DM123"));
    assert!(!gw.last_response_at.contains_key("Discord:user:123456789"));
    assert_eq!(
        gw.discord_dm_channels_by_user
            .get("user:123456789")
            .map(String::as_str),
        Some("DM123")
    );
    assert_eq!(
        gw.reply_contexts
            .get("Discord:DM123")
            .and_then(|ctx| ctx.discord_message_id.as_deref()),
        Some("delivery-1")
    );
    drop(gw_guard);

    conn.shutdown().await;
}

#[tokio::test]
async fn whatsapp_link_start_status_stop_send_status_responses() {
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkStatus)
        .await
        .expect("send status request");
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkStatus { state, .. } => {
            assert_eq!(state, "disconnected")
        }
        other => panic!("expected AgentWhatsAppLinkStatus, got {other:?}"),
    }

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkStart)
        .await
        .expect("send start request");
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkStatus { state, .. } => assert_eq!(state, "starting"),
        other => panic!("expected AgentWhatsAppLinkStatus after start, got {other:?}"),
    }

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkStop)
        .await
        .expect("send stop request");
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkStatus { state, .. } => {
            assert_eq!(state, "disconnected")
        }
        other => panic!("expected AgentWhatsAppLinkStatus after stop, got {other:?}"),
    }

    conn.shutdown().await;
}

#[tokio::test]
async fn whatsapp_link_reset_clears_link_state() {
    let mut conn = spawn_test_connection().await;

    conn.agent
        .whatsapp_link
        .broadcast_qr("QR-RESET".to_string(), Some(123))
        .await;
    conn.agent
        .whatsapp_link
        .broadcast_linked(Some("+15551112222".to_string()))
        .await;
    crate::agent::save_persisted_provider_state(
        &conn.agent.history,
        crate::agent::WHATSAPP_LINK_PROVIDER_ID,
        crate::agent::WhatsAppPersistedState {
            linked_phone: Some("+15551112222".to_string()),
            auth_json: Some("{\"session\":true}".to_string()),
            metadata_json: Some("{\"source\":\"server-test\"}".to_string()),
            last_reset_at: None,
            last_linked_at: Some(5),
            updated_at: 6,
        },
    )
    .await
    .expect("persist whatsapp provider state");

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkReset)
        .await
        .expect("send reset request");
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkReset { ok, .. } => assert!(ok),
        other => panic!("expected AgentWhatsAppLinkReset, got {other:?}"),
    }

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkStatus)
        .await
        .expect("send status request after reset");
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkStatus {
            state,
            phone,
            last_error,
        } => {
            assert_eq!(state, "disconnected");
            assert!(phone.is_none());
            assert!(last_error.is_none());
        }
        other => panic!("expected AgentWhatsAppLinkStatus after reset, got {other:?}"),
    }
    assert!(
        crate::agent::load_persisted_provider_state(
            &conn.agent.history,
            crate::agent::WHATSAPP_LINK_PROVIDER_ID,
        )
        .await
        .expect("load persisted provider state")
        .is_none(),
        "reset should remove persisted provider state"
    );

    conn.shutdown().await;
}

#[tokio::test]
async fn whatsapp_link_subscribe_then_unsubscribe_stops_forwarding() {
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkSubscribe)
        .await
        .expect("send subscribe request");
    assert!(
        matches!(
            conn.recv().await,
            DaemonMessage::AgentWhatsAppLinkStatus { .. }
        ),
        "subscribe should replay status snapshot"
    );

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkUnsubscribe)
        .await
        .expect("send unsubscribe request");
    conn.framed
        .send(ClientMessage::Ping)
        .await
        .expect("send ping barrier");
    assert!(
        matches!(conn.recv().await, DaemonMessage::Pong),
        "ping barrier should confirm unsubscribe was processed"
    );
    conn.agent
        .whatsapp_link
        .broadcast_qr("QR-UNSUB".to_string(), Some(123))
        .await;

    let maybe_msg = timeout(Duration::from_millis(150), conn.framed.next()).await;
    assert!(
        maybe_msg.is_err(),
        "no whatsapp link event should be forwarded after unsubscribe"
    );

    conn.shutdown().await;
}

#[tokio::test]
async fn whatsapp_link_subscription_replay_status_then_incremental_events() {
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkSubscribe)
        .await
        .expect("send subscribe request");
    assert!(
        matches!(
            conn.recv().await,
            DaemonMessage::AgentWhatsAppLinkStatus { .. }
        ),
        "first replayed event should be status snapshot"
    );

    conn.agent
        .whatsapp_link
        .broadcast_qr("QR-ORDER".to_string(), Some(111))
        .await;
    conn.agent
        .whatsapp_link
        .broadcast_linked(Some("+15550001111".to_string()))
        .await;
    conn.agent
        .whatsapp_link
        .broadcast_error("recoverable".to_string(), true)
        .await;
    conn.agent
        .whatsapp_link
        .broadcast_disconnected(Some("operator_cancelled".to_string()))
        .await;

    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkQr { ascii_qr, .. } => assert_eq!(ascii_qr, "QR-ORDER"),
        other => panic!("expected AgentWhatsAppLinkQr, got {other:?}"),
    }
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinked { phone } => {
            assert_eq!(phone.as_deref(), Some("+15550001111"))
        }
        other => panic!("expected AgentWhatsAppLinked, got {other:?}"),
    }
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkError {
            message,
            recoverable,
        } => {
            assert_eq!(message, "recoverable");
            assert!(recoverable);
        }
        other => panic!("expected AgentWhatsAppLinkError, got {other:?}"),
    }
    match conn.recv().await {
        DaemonMessage::AgentWhatsAppLinkDisconnected { reason } => {
            assert_eq!(reason.as_deref(), Some("operator_cancelled"))
        }
        other => panic!("expected AgentWhatsAppLinkDisconnected, got {other:?}"),
    }

    conn.shutdown().await;
}

#[tokio::test]
async fn skill_discover_returns_ranked_candidates() {
    let mut conn = spawn_test_connection().await;

    let skill_dir = conn
        .agent
        .history
        .data_dir()
        .join("skills")
        .join("generated")
        .join("systematic-debugging");
    std::fs::create_dir_all(&skill_dir).expect("create skill dir");
    let skill_path = skill_dir.join("SKILL.md");
    std::fs::write(
        &skill_path,
        r#"---
name: systematic-debugging
description: Debug panic failures in Rust workspaces.
keywords: [debug, panic, rust]
triggers: [panic, crash]
---

# Systematic Debugging

Use this when debugging a panic in a Rust workspace.
"#,
    )
    .expect("write skill document");

    let record = conn
        .agent
        .history
        .register_skill_document(&skill_path)
        .await
        .expect("register skill document");
    for _ in 0..14 {
        conn.agent
            .history
            .record_skill_variant_use(&record.variant_id, Some(true))
            .await
            .expect("record successful skill use");
    }
    for _ in 0..2 {
        conn.agent
            .history
            .record_skill_variant_use(&record.variant_id, Some(false))
            .await
            .expect("record failed skill use");
    }

    conn.framed
        .send(ClientMessage::SkillDiscover {
            query: "debug panic".to_string(),
            session_id: None,
            limit: 3,
            cursor: None,
        })
        .await
        .expect("send skill discover request");

    let result = loop {
        match conn.recv_with_timeout(Duration::from_secs(2)).await {
            DaemonMessage::SkillDiscoverResult { result_json } => {
                break serde_json::from_str::<amux_protocol::SkillDiscoveryResultPublic>(
                    &result_json,
                )
                .expect("parse discovery result")
            }
            DaemonMessage::CwdChanged { .. } => continue,
            DaemonMessage::Output { .. }
            | DaemonMessage::CommandStarted { .. }
            | DaemonMessage::CommandFinished { .. } => continue,
            other => panic!("expected SkillDiscoverResult, got {other:?}"),
        }
    };

    assert_eq!(result.query, "debug panic");
    assert_eq!(result.confidence_tier, "strong");
    assert_eq!(result.recommended_action, "read_skill systematic-debugging");
    assert!(
        result.workspace_tags.iter().any(|tag| tag == "rust"),
        "expected workspace tags to include rust: {:?}",
        result.workspace_tags
    );
    assert_eq!(result.candidates.len(), 1);
    assert_eq!(result.candidates[0].skill_name, "systematic-debugging");
    assert_eq!(result.candidates[0].status, "active");
    assert_eq!(result.candidates[0].confidence_tier, "strong");
    assert_eq!(result.candidates[0].use_count, 16);
    assert_eq!(result.candidates[0].success_count, 14);
    assert_eq!(result.candidates[0].failure_count, 2);
    assert!(
        result.candidates[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("successful uses")),
        "expected usage rationale in discovery reasons: {:?}",
        result.candidates[0].reasons
    );

    conn.shutdown().await;
}

#[tokio::test]
async fn whatsapp_link_subscriber_is_cleaned_up_on_disconnect_without_unsubscribe() {
    let mut conn = spawn_test_connection().await;
    conn.framed
        .send(ClientMessage::AgentWhatsAppLinkSubscribe)
        .await
        .expect("send subscribe request");
    assert!(
        matches!(
            conn.recv().await,
            DaemonMessage::AgentWhatsAppLinkStatus { .. }
        ),
        "subscribe should replay status snapshot"
    );

    assert_eq!(
        conn.agent.whatsapp_link.subscriber_count().await,
        1,
        "subscriber should be registered after subscribe"
    );

    let agent = conn.agent.clone();
    conn.shutdown().await;

    assert_eq!(
        agent.whatsapp_link.subscriber_count().await,
        0,
        "subscriber should be removed when connection exits"
    );
}

#[tokio::test]
async fn divergent_ipc_get_session_returns_completion_payload() {
    let mut conn = spawn_test_connection().await;
    let thread_id = "thread-divergent-server";
    let session_id = conn
        .agent
        .start_divergent_session("evaluate rollout strategy", None, thread_id, None)
        .await
        .expect("start divergent session");

    // Record contributions and complete session to synthesize retrieval payload.
    let framing_labels = vec!["analytical-lens".to_string(), "pragmatic-lens".to_string()];
    for (idx, label) in framing_labels.iter().enumerate() {
        conn.agent
            .record_divergent_contribution(
                &session_id,
                label,
                if idx == 0 {
                    "Prefer conservative phased rollout"
                } else {
                    "Prefer fast rollout with rollback hooks"
                },
            )
            .await
            .expect("contribution recording should succeed");
    }
    conn.agent
        .complete_divergent_session(&session_id)
        .await
        .expect("session completion should succeed");

    conn.framed
        .send(ClientMessage::AgentGetDivergentSession {
            session_id: session_id.clone(),
        })
        .await
        .expect("send retrieval request");

    let payload = match conn.recv().await {
        DaemonMessage::AgentDivergentSession { session_json } => {
            serde_json::from_str::<serde_json::Value>(&session_json)
                .expect("session payload should decode")
        }
        other => panic!("expected AgentDivergentSession, got {other:?}"),
    };
    assert_eq!(
        payload.get("session_id").and_then(|v| v.as_str()),
        Some(session_id.as_str())
    );
    assert_eq!(
        payload.get("status").and_then(|v| v.as_str()),
        Some("complete")
    );
    assert!(payload
        .get("tensions_markdown")
        .and_then(|v| v.as_str())
        .is_some_and(|v| !v.is_empty()));
    assert!(payload
        .get("mediator_prompt")
        .and_then(|v| v.as_str())
        .is_some_and(|v| !v.is_empty()));

    conn.shutdown().await;
}

struct EnvGuard {
    vars: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn new(names: &[&'static str]) -> Self {
        Self {
            vars: names
                .iter()
                .map(|name| (*name, std::env::var(name).ok()))
                .collect(),
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (name, value) in self.vars.drain(..) {
            if let Some(value) = value {
                std::env::set_var(name, value);
            } else {
                std::env::remove_var(name);
            }
        }
    }
}

fn prepare_server_openai_codex_auth_test(root: &std::path::Path) {
    std::env::set_var("TAMUX_PROVIDER_AUTH_DB_PATH", root.join("provider-auth.db"));
    std::env::set_var(
        "TAMUX_CODEX_CLI_AUTH_PATH",
        root.join("missing-codex-auth.json"),
    );
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

fn setup_server_openai_codex_auth_test() -> (tempfile::TempDir, EnvGuard) {
    let temp_dir = tempfile::tempdir().expect("tempdir should succeed");
    let env_guard = EnvGuard::new(&["TAMUX_PROVIDER_AUTH_DB_PATH", "TAMUX_CODEX_CLI_AUTH_PATH"]);
    prepare_server_openai_codex_auth_test(temp_dir.path());
    (temp_dir, env_guard)
}

fn parse_json(raw: &str) -> serde_json::Value {
    serde_json::from_str(raw).expect("json payload should decode")
}

#[tokio::test]
async fn inspect_prompt_returns_sectioned_main_agent_prompt() {
    let mut config = AgentConfig::default();
    config.system_prompt = "Custom operator prompt".to_string();
    config.provider = "openai".to_string();
    config.model = "gpt-5.4-mini".to_string();
    let mut conn = spawn_test_connection_with_config(config).await;

    conn.framed
        .send(ClientMessage::AgentInspectPrompt { agent_id: None })
        .await
        .expect("send inspect prompt request");

    let payload = match conn.recv().await {
        DaemonMessage::AgentPromptInspection { prompt_json } => parse_json(&prompt_json),
        other => panic!("expected AgentPromptInspection, got {other:?}"),
    };

    assert_eq!(
        payload.get("agent_id").and_then(|value| value.as_str()),
        Some("swarog")
    );
    assert_eq!(
        payload.get("agent_name").and_then(|value| value.as_str()),
        Some("Svarog")
    );

    let sections = payload
        .get("sections")
        .and_then(|value| value.as_array())
        .expect("sections should be an array");
    assert!(
        sections.iter().any(|section| {
            section.get("id").and_then(|value| value.as_str()) == Some("base_prompt")
                && section.get("content").and_then(|value| value.as_str())
                    == Some("Custom operator prompt")
        }),
        "base prompt section should reflect the configured operator prompt"
    );

    let final_prompt = payload
        .get("final_prompt")
        .and_then(|value| value.as_str())
        .expect("final prompt should be present");
    assert!(final_prompt.contains("Custom operator prompt"));
    assert!(final_prompt.contains("## Local Skills"));
    assert!(final_prompt.contains("## Runtime Identity"));

    conn.shutdown().await;
}

#[tokio::test]
async fn force_compact_request_queues_continuation_and_cancels_active_stream() {
    let mut conn = spawn_test_connection().await;
    let thread_id = "thread-force-compact";

    conn.agent.threads.write().await.insert(
        thread_id.to_string(),
        AgentThread {
            id: thread_id.to_string(),
            agent_name: Some(crate::agent::agent_identity::MAIN_AGENT_NAME.to_string()),
            title: "Force compact".to_string(),
            messages: vec![
                AgentMessage::user("original user request", 1),
                AgentMessage::user("follow-up user request", 2),
            ],
            pinned: false,
            upstream_thread_id: None,
            upstream_transport: None,
            upstream_provider: None,
            upstream_model: None,
            upstream_assistant_id: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            created_at: 1,
            updated_at: 2,
        },
    );
    let (generation, token, retry_now) = conn.agent.begin_stream_cancellation(thread_id).await;
    {
        let mut streams = conn.agent.stream_cancellations.lock().await;
        streams.insert(
            thread_id.to_string(),
            StreamCancellationEntry {
                generation,
                token: token.clone(),
                retry_now,
                started_at: 1,
                last_progress_at: 1,
                last_progress_kind: StreamProgressKind::Content,
                last_progress_excerpt: "working".to_string(),
            },
        );
    }

    conn.framed
        .send(ClientMessage::AgentForceCompact {
            thread_id: thread_id.to_string(),
        })
        .await
        .expect("send force compact request");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert!(
        token.is_cancelled(),
        "force compact should cancel active stream"
    );
    let queued_for_thread = conn
        .agent
        .deferred_visible_thread_continuations_for(thread_id)
        .await;
    assert_eq!(queued_for_thread.len(), 1);
    assert!(queued_for_thread[0].force_compaction);

    conn.shutdown().await;
}

#[tokio::test]
async fn force_compact_reports_failure_when_thread_has_no_user_message() {
    let mut conn = spawn_test_connection().await;
    let thread_id = "thread-force-compact-no-user";

    let mut assistant = AgentMessage::user("assistant-only context", 1);
    assistant.role = crate::agent::types::MessageRole::Assistant;
    conn.agent.threads.write().await.insert(
        thread_id.to_string(),
        AgentThread {
            id: thread_id.to_string(),
            agent_name: Some(crate::agent::agent_identity::MAIN_AGENT_NAME.to_string()),
            title: "Force compact without user replay".to_string(),
            messages: vec![assistant],
            pinned: false,
            upstream_thread_id: None,
            upstream_transport: None,
            upstream_provider: None,
            upstream_model: None,
            upstream_assistant_id: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            created_at: 1,
            updated_at: 1,
        },
    );

    conn.framed
        .send(ClientMessage::AgentSubscribe)
        .await
        .expect("subscribe to agent events");

    conn.framed
        .send(ClientMessage::AgentForceCompact {
            thread_id: thread_id.to_string(),
        })
        .await
        .expect("send force compact request");

    let message = timeout(Duration::from_millis(250), async {
        loop {
            match conn.recv().await {
                DaemonMessage::AgentEvent { event_json } => {
                    let parsed: serde_json::Value =
                        serde_json::from_str(&event_json).expect("parse workflow notice");
                    if parsed.get("type").and_then(|value| value.as_str())
                        == Some("workflow_notice")
                    {
                        return parsed;
                    }
                }
                other => panic!("expected workflow notice, got {other:?}"),
            }
        }
    })
    .await
    .expect("manual compaction failure should be surfaced");

    assert_eq!(
        message.get("thread_id").and_then(|value| value.as_str()),
        Some(thread_id)
    );
    assert_eq!(
        message.get("kind").and_then(|value| value.as_str()),
        Some("manual-compaction")
    );
    assert!(
        message
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .contains("failed"),
        "expected manual compaction failure message, got {message:?}"
    );

    conn.shutdown().await;
}

fn extract_state_from_auth_url(auth_url: &str) -> String {
    url::Url::parse(auth_url)
        .expect("auth url should parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("auth url should contain state")
}

fn wait_for_listener_and_send_callback(state: &str, code: &str) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);

    loop {
        match std::net::TcpStream::connect("127.0.0.1:1455") {
            Ok(mut stream) => {
                use std::io::{Read, Write};

                let request = format!(
                        "GET /auth/callback?state={state}&code={code} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
                    );
                stream
                    .write_all(request.as_bytes())
                    .expect("callback request should write");
                let mut response = String::new();
                let _ = stream.read_to_string(&mut response);
                return;
            }
            Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
                if std::time::Instant::now() >= deadline {
                    panic!("callback listener did not become ready in time");
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(error) => panic!("callback connection should succeed: {error}"),
        }
    }
}

#[tokio::test]
async fn openai_codex_auth_status_request_returns_status_payload() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (_temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentGetOpenAICodexAuthStatus)
        .await
        .expect("send auth status request");

    let status = match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthStatus { status_json } => parse_json(&status_json),
        other => panic!("expected AgentOpenAICodexAuthStatus, got {other:?}"),
    };
    assert_eq!(
        status.get("available").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(status.get("status").and_then(|v| v.as_str()), None);
    assert!(status.get("authUrl").is_none());
    assert!(status.get("error").is_none());

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn openai_codex_auth_login_request_returns_login_payload() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (_temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLoginOpenAICodex)
        .await
        .expect("send auth login request");

    let login = match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { result_json } => parse_json(&result_json),
        other => panic!("expected AgentOpenAICodexAuthLoginResult, got {other:?}"),
    };
    assert_eq!(
        login.get("available").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        login.get("status").and_then(|v| v.as_str()),
        Some("pending")
    );
    assert!(login
        .get("authUrl")
        .and_then(|v| v.as_str())
        .is_some_and(|value| value.starts_with("https://auth.openai.com/oauth/authorize")));
    assert!(login.get("error").is_none());

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn openai_codex_auth_login_request_starts_browser_callback_completion_flow() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (_temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLoginOpenAICodex)
        .await
        .expect("send auth login request");

    let state = match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { result_json } => {
            let login = parse_json(&result_json);
            extract_state_from_auth_url(
                login
                    .get("authUrl")
                    .and_then(|value| value.as_str())
                    .expect("login should include auth url"),
            )
        }
        other => panic!("expected AgentOpenAICodexAuthLoginResult, got {other:?}"),
    };

    let callback_thread = std::thread::spawn(|| {
        wait_for_listener_and_send_callback("wrong-state", "bad-code");
    });

    let status = match conn.recv_with_timeout(Duration::from_secs(1)).await {
        DaemonMessage::AgentOpenAICodexAuthStatus { status_json } => parse_json(&status_json),
        other => panic!("expected AgentOpenAICodexAuthStatus, got {other:?}"),
    };
    callback_thread.join().expect("callback thread should join");

    assert_eq!(state.is_empty(), false);
    assert_eq!(
        status.get("status").and_then(|value| value.as_str()),
        Some("error")
    );
    assert_eq!(
        status.get("error").and_then(|value| value.as_str()),
        Some("OpenAI authentication failed. Please try signing in again.")
    );

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn openai_codex_auth_login_request_helper_failure_returns_login_result_payload() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    std::env::set_var("TAMUX_PROVIDER_AUTH_DB_PATH", temp_dir.path());
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLoginOpenAICodex)
        .await
        .expect("send auth login request");

    let login = match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { result_json } => parse_json(&result_json),
        other => panic!("expected AgentOpenAICodexAuthLoginResult, got {other:?}"),
    };
    assert_eq!(login.get("status").and_then(|v| v.as_str()), Some("error"));
    assert_eq!(
        login.get("available").and_then(|v| v.as_bool()),
        Some(false)
    );
    assert_eq!(
        login.get("error").and_then(|v| v.as_str()),
        Some("OpenAI authentication failed. Please try signing in again.")
    );

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn openai_codex_auth_login_request_replies_immediately_without_operation_accepted() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (_temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLoginOpenAICodex)
        .await
        .expect("send auth login request");

    match conn.recv_with_timeout(Duration::from_millis(250)).await {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { .. } => {}
        DaemonMessage::OperationAccepted { .. } => {
            panic!("login should reply immediately instead of OperationAccepted")
        }
        other => panic!("expected immediate login reply, got {other:?}"),
    }

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn openai_codex_auth_logout_request_returns_logout_payload() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (_temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLogoutOpenAICodex)
        .await
        .expect("send auth logout request");

    match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLogoutResult { ok, error } => {
            assert!(ok);
            assert!(error.is_none());
        }
        other => panic!("expected AgentOpenAICodexAuthLogoutResult, got {other:?}"),
    }

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn openai_codex_auth_logout_helper_failure_returns_sanitized_error_payload() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    std::env::set_var("TAMUX_PROVIDER_AUTH_DB_PATH", temp_dir.path());
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLogoutOpenAICodex)
        .await
        .expect("send auth logout request");

    match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLogoutResult { ok, error } => {
            assert!(!ok);
            assert_eq!(
                error.as_deref(),
                Some("OpenAI authentication failed. Please try signing in again.")
            );
        }
        other => panic!("expected AgentOpenAICodexAuthLogoutResult, got {other:?}"),
    }

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn openai_codex_auth_logout_during_pending_clears_pending_state() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (_temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLoginOpenAICodex)
        .await
        .expect("send auth login request");
    match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { .. } => {}
        other => panic!("expected AgentOpenAICodexAuthLoginResult, got {other:?}"),
    }

    conn.framed
        .send(ClientMessage::AgentLogoutOpenAICodex)
        .await
        .expect("send auth logout request while pending");
    match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLogoutResult { ok, error } => {
            assert!(ok);
            assert!(error.is_none());
        }
        other => panic!("expected AgentOpenAICodexAuthLogoutResult, got {other:?}"),
    }

    conn.framed
        .send(ClientMessage::AgentGetOpenAICodexAuthStatus)
        .await
        .expect("send auth status request after logout");
    let status = match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthStatus { status_json } => parse_json(&status_json),
        other => panic!("expected AgentOpenAICodexAuthStatus, got {other:?}"),
    };
    assert_eq!(status.get("status").and_then(|v| v.as_str()), None);
    assert!(status.get("authUrl").is_none());

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}

#[tokio::test]
async fn github_copilot_login_provider_with_browser_auth_returns_login_result() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let db_root = tempfile::tempdir().expect("temp dir");
    let script_path = db_root.path().join("gh-mock.sh");
    std::fs::write(
        &script_path,
        "#!/bin/sh\nif [ \"$1\" = \"auth\" ] && [ \"$2\" = \"login\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"auth\" ] && [ \"$2\" = \"token\" ]; then\n  printf 'ghu_test_token\\n'\n  exit 0\nfi\nexit 1\n",
    )
    .expect("write gh mock");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)
            .expect("script metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("chmod gh mock");
    }

    std::env::set_var("TAMUX_PROVIDER_AUTH_DB_PATH", db_root.path().join("provider-auth.db"));
    std::env::set_var("PATH", format!("{}:{}", db_root.path().display(), std::env::var("PATH").unwrap_or_default()));
    std::fs::copy(&script_path, db_root.path().join("gh")).expect("install gh mock");

    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLoginProvider {
            provider_id: "github-copilot".to_string(),
            api_key: String::new(),
            base_url: "https://api.githubcopilot.com".to_string(),
            auth_source: Some("github_copilot".to_string()),
        })
        .await
        .expect("send copilot login request");

    match conn.recv().await {
        DaemonMessage::AgentProviderLoginResult {
            provider_id,
            success,
            message,
        } => {
            assert_eq!(provider_id, "github-copilot");
            assert!(success);
            assert_eq!(message.as_deref(), Some("Started GitHub Copilot browser login"));
        }
        other => panic!("expected AgentProviderLoginResult, got {other:?}"),
    }

    match conn.recv().await {
        DaemonMessage::AgentProviderAuthStates { states_json } => {
            let states: serde_json::Value = serde_json::from_str(&states_json).expect("states json");
            let copilot = states
                .as_array()
                .and_then(|items| {
                    items.iter().find(|item| {
                        item.get("provider_id").and_then(|v| v.as_str())
                            == Some("github-copilot")
                    })
                })
                .expect("copilot state present");
            assert_eq!(copilot.get("authenticated").and_then(|v| v.as_bool()), Some(true));
        }
        other => panic!("expected AgentProviderAuthStates, got {other:?}"),
    }

    conn.shutdown().await;
    std::env::remove_var("TAMUX_PROVIDER_AUTH_DB_PATH");
}

#[tokio::test]
async fn openai_codex_auth_login_after_error_returns_fresh_pending_payload() {
    let _lock = crate::agent::provider_auth_test_env_lock();
    let (_temp_dir, _env_guard) = setup_server_openai_codex_auth_test();
    let mut conn = spawn_test_connection().await;

    conn.framed
        .send(ClientMessage::AgentLoginOpenAICodex)
        .await
        .expect("send auth login request");
    let first_login = match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { result_json } => parse_json(&result_json),
        other => panic!("expected AgentOpenAICodexAuthLoginResult, got {other:?}"),
    };
    crate::agent::openai_codex_auth::mark_openai_codex_auth_timeout_for_tests();

    conn.framed
        .send(ClientMessage::AgentLoginOpenAICodex)
        .await
        .expect("send auth login request after error");
    let second_login = match conn.recv().await {
        DaemonMessage::AgentOpenAICodexAuthLoginResult { result_json } => parse_json(&result_json),
        other => panic!("expected AgentOpenAICodexAuthLoginResult, got {other:?}"),
    };

    assert_eq!(
        second_login.get("status").and_then(|v| v.as_str()),
        Some("pending")
    );
    assert!(second_login.get("error").is_none());
    assert_ne!(second_login.get("authUrl"), first_login.get("authUrl"));

    conn.shutdown().await;
    crate::agent::openai_codex_auth::clear_openai_codex_auth_test_state();
}
