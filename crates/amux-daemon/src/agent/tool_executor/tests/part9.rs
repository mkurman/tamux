#[tokio::test]
async fn fetch_url_openapi_spec_emits_openapi_tool_synthesis_proposal_notice() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind openapi spec server");
    let addr = listener.local_addr().expect("openapi spec local addr");
    let spec_body = serde_json::json!({
        "openapi": "3.0.0",
        "servers": [{"url": format!("http://{addr}")}],
        "paths": {
            "/status": {
                "get": {
                    "operationId": "getStatus",
                    "summary": "Fetch status",
                    "parameters": [
                        {
                            "name": "verbose",
                            "in": "query",
                            "required": false,
                            "schema": {"type": "boolean"},
                            "description": "Verbose output"
                        }
                    ]
                }
            }
        }
    })
    .to_string();

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let spec_body = spec_body.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 8192];
                let _ = socket.read(&mut buffer).await.expect("read spec request");
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    spec_body.len(),
                    spec_body
                );
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write spec response");
            });
        }
    });

    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    config.extra.insert(
        "browse_provider".to_string(),
        serde_json::Value::String("none".to_string()),
    );
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let spec_url = format!("http://{addr}/openapi.json");
    let result = execute_tool(
        &ToolCall {
            id: "call-fetch-openapi-spec".to_string(),
            function: ToolFunction {
                name: "fetch_url".to_string(),
                arguments: serde_json::json!({
                    "url": spec_url,
                    "max_length": 10_000,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-openapi-gap",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "fetch_url should succeed: {}",
        result.content
    );

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected synthesis proposal workflow notice")
        .expect("workflow notice should be received");
    match notice {
        AgentEvent::WorkflowNotice {
            kind,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "tool-synthesis-proposal");
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details
                    .get("proposal_kind")
                    .and_then(|value| value.as_str()),
                Some("openapi")
            );
            let synth_args = details
                .get("synthesize_tool_args")
                .expect("proposal should include synthesize_tool args");
            assert_eq!(
                synth_args.get("kind").and_then(|value| value.as_str()),
                Some("openapi")
            );
            assert_eq!(
                synth_args.get("target").and_then(|value| value.as_str()),
                Some(spec_url.as_str())
            );
            assert_eq!(
                synth_args
                    .get("operation_id")
                    .and_then(|value| value.as_str()),
                Some("getStatus")
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn fetch_url_openapi_spec_emits_activate_notice_when_equivalent_generated_tool_is_new() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind openapi spec server");
    let addr = listener.local_addr().expect("openapi spec local addr");
    let spec_body = serde_json::json!({
        "openapi": "3.0.0",
        "servers": [{"url": format!("http://{addr}")}],
        "paths": {
            "/status": {
                "get": {
                    "operationId": "getStatus",
                    "summary": "Fetch status"
                }
            }
        }
    })
    .to_string();

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let spec_body = spec_body.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 8192];
                let _ = socket.read(&mut buffer).await.expect("read spec request");
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    spec_body.len(),
                    spec_body
                );
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write spec response");
            });
        }
    });

    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    config.extra.insert(
        "browse_provider".to_string(),
        serde_json::Value::String("none".to_string()),
    );
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let spec_url = format!("http://{addr}/openapi.json");
    engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "openapi",
                "target": spec_url,
                "operation_id": "getStatus",
                "name": "getstatus",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated OpenAPI tool");

    let result = execute_tool(
        &ToolCall {
            id: "call-fetch-openapi-existing".to_string(),
            function: ToolFunction {
                name: "fetch_url".to_string(),
                arguments: serde_json::json!({
                    "url": format!("http://{addr}/openapi.json"),
                    "max_length": 10_000,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-openapi-gap-existing",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "fetch_url should succeed: {}",
        result.content
    );

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected existing-tool status notice")
        .expect("workflow notice should be received");
    match notice {
        AgentEvent::WorkflowNotice {
            kind,
            message,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "tool-synthesis-proposal");
            assert!(message.contains("Activate it"));
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details.get("reason").and_then(|value| value.as_str()),
                Some("existing_equivalent_generated_tool")
            );
            assert_eq!(
                details
                    .get("source_reason")
                    .and_then(|value| value.as_str()),
                Some("fetched_openapi_get_gap")
            );
            assert_eq!(
                details
                    .get("recommended_action")
                    .and_then(|value| value.as_str()),
                Some("activate_generated_tool")
            );
            assert_eq!(
                details
                    .get("proposal_kind")
                    .and_then(|value| value.as_str()),
                Some("openapi")
            );
            assert_eq!(
                details
                    .get("existing_tool")
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_str()),
                Some("new")
            );
            assert_eq!(
                details.get("target").and_then(|value| value.as_str()),
                Some(spec_url.as_str())
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn fetch_url_openapi_spec_emits_reuse_notice_when_equivalent_generated_tool_is_active() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind openapi spec server");
    let addr = listener.local_addr().expect("openapi spec local addr");
    let spec_body = serde_json::json!({
        "openapi": "3.0.0",
        "servers": [{"url": format!("http://{addr}")}],
        "paths": {
            "/status": {
                "get": {
                    "operationId": "getStatus",
                    "summary": "Fetch status"
                }
            }
        }
    })
    .to_string();

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let spec_body = spec_body.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 8192];
                let _ = socket.read(&mut buffer).await.expect("read spec request");
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    spec_body.len(),
                    spec_body
                );
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write spec response");
            });
        }
    });

    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    config.extra.insert(
        "browse_provider".to_string(),
        serde_json::Value::String("none".to_string()),
    );
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let spec_url = format!("http://{addr}/openapi.json");
    let synth = engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "openapi",
                "target": spec_url,
                "operation_id": "getStatus",
                "name": "getstatus",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated OpenAPI tool");
    let record: serde_json::Value = serde_json::from_str(&synth).expect("parse synth record");
    let tool_name = record
        .get("id")
        .and_then(|value| value.as_str())
        .expect("generated tool id");
    engine
        .activate_generated_tool_json(tool_name)
        .await
        .expect("activate generated OpenAPI tool");

    let result = execute_tool(
        &ToolCall {
            id: "call-fetch-openapi-existing-active".to_string(),
            function: ToolFunction {
                name: "fetch_url".to_string(),
                arguments: serde_json::json!({
                    "url": format!("http://{addr}/openapi.json"),
                    "max_length": 10_000,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-openapi-gap-existing-active",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "fetch_url should succeed: {}",
        result.content
    );

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected existing-tool status notice")
        .expect("workflow notice should be received");
    match notice {
        AgentEvent::WorkflowNotice {
            kind,
            message,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "tool-synthesis-proposal");
            assert!(message.contains("already active"));
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details.get("reason").and_then(|value| value.as_str()),
                Some("existing_equivalent_generated_tool")
            );
            assert_eq!(
                details
                    .get("source_reason")
                    .and_then(|value| value.as_str()),
                Some("fetched_openapi_get_gap")
            );
            assert_eq!(
                details
                    .get("recommended_action")
                    .and_then(|value| value.as_str()),
                Some("use_existing_generated_tool")
            );
            assert_eq!(
                details
                    .get("proposal_kind")
                    .and_then(|value| value.as_str()),
                Some("openapi")
            );
            assert_eq!(
                details
                    .get("existing_tool")
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_str()),
                Some("active")
            );
            assert_eq!(
                details.get("target").and_then(|value| value.as_str()),
                Some(spec_url.as_str())
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn fetch_url_openapi_spec_updates_existing_tool_notice_after_status_transition() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind openapi spec server");
    let addr = listener.local_addr().expect("openapi spec local addr");
    let spec_body = serde_json::json!({
        "openapi": "3.0.0",
        "servers": [{"url": format!("http://{addr}")}],
        "paths": {
            "/status": {
                "get": {
                    "operationId": "getStatus",
                    "summary": "Fetch status"
                }
            }
        }
    })
    .to_string();

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let spec_body = spec_body.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 8192];
                let _ = socket.read(&mut buffer).await.expect("read spec request");
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    spec_body.len(),
                    spec_body
                );
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write spec response");
            });
        }
    });

    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    config.extra.insert(
        "browse_provider".to_string(),
        serde_json::Value::String("none".to_string()),
    );
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(16);

    let spec_url = format!("http://{addr}/openapi.json");
    let synth = engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "openapi",
                "target": spec_url,
                "operation_id": "getStatus",
                "name": "getstatus",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated OpenAPI tool");
    let record: serde_json::Value = serde_json::from_str(&synth).expect("parse synth record");
    let tool_name = record
        .get("id")
        .and_then(|value| value.as_str())
        .expect("generated tool id");

    // First fetch should notice the equivalent NEW tool and recommend activation.
    let first_result = execute_tool(
        &ToolCall {
            id: "call-fetch-openapi-existing-status-new".to_string(),
            function: ToolFunction {
                name: "fetch_url".to_string(),
                arguments: serde_json::json!({
                    "url": format!("http://{addr}/openapi.json"),
                    "max_length": 10_000,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-openapi-gap-status-transition",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;
    assert!(
        !first_result.is_error,
        "first fetch_url should succeed: {}",
        first_result.content
    );

    let first_notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected first existing-tool status notice")
        .expect("workflow notice should be received");
    match first_notice {
        AgentEvent::WorkflowNotice {
            details: Some(details),
            ..
        } => {
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details
                    .get("recommended_action")
                    .and_then(|value| value.as_str()),
                Some("activate_generated_tool")
            );
            assert_eq!(
                details
                    .get("existing_tool")
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_str()),
                Some("new")
            );
        }
        other => panic!("expected first workflow notice, got {other:?}"),
    }

    // Transition status to active.
    engine
        .activate_generated_tool_json(tool_name)
        .await
        .expect("activate generated OpenAPI tool");

    // Second fetch on the SAME thread should surface updated reuse guidance.
    let second_result = execute_tool(
        &ToolCall {
            id: "call-fetch-openapi-existing-status-active".to_string(),
            function: ToolFunction {
                name: "fetch_url".to_string(),
                arguments: serde_json::json!({
                    "url": format!("http://{addr}/openapi.json"),
                    "max_length": 10_000,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-openapi-gap-status-transition",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;
    assert!(
        !second_result.is_error,
        "second fetch_url should succeed: {}",
        second_result.content
    );

    let second_notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected second existing-tool status notice after activation")
        .expect("workflow notice should be received");
    match second_notice {
        AgentEvent::WorkflowNotice {
            details: Some(details),
            ..
        } => {
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details
                    .get("recommended_action")
                    .and_then(|value| value.as_str()),
                Some("use_existing_generated_tool")
            );
            assert_eq!(
                details
                    .get("existing_tool")
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_str()),
                Some("active")
            );
        }
        other => panic!("expected second workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn unknown_cli_like_tool_emits_tool_synthesis_proposal_notice() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);
    let http_client = reqwest::Client::new();

    let result = execute_tool(
        &ToolCall {
            id: "call-tool-gap-cargo-check".to_string(),
            function: ToolFunction {
                name: "cargo_check".to_string(),
                arguments: serde_json::json!({}).to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-tool-gap",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &http_client,
        None,
    )
    .await;

    assert!(result.is_error, "unknown tool should still error");
    assert!(result.content.contains("Unknown tool: cargo_check"));

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected synthesis proposal workflow notice")
        .expect("workflow notice should be received");
    match notice {
        AgentEvent::WorkflowNotice {
            kind,
            message,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "tool-synthesis-proposal");
            assert!(message.contains("cargo_check"));
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details.get("missing_tool").and_then(|value| value.as_str()),
                Some("cargo_check")
            );
            let synth_args = details
                .get("synthesize_tool_args")
                .expect("proposal should include synthesize_tool args");
            assert_eq!(
                synth_args.get("kind").and_then(|value| value.as_str()),
                Some("cli")
            );
            assert_eq!(
                synth_args.get("target").and_then(|value| value.as_str()),
                Some("cargo check")
            );
            assert_eq!(
                synth_args.get("name").and_then(|value| value.as_str()),
                Some("cargo_check")
            );
            assert_eq!(
                synth_args.get("activate").and_then(|value| value.as_bool()),
                Some(false)
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn risky_unknown_cli_like_tool_does_not_emit_tool_synthesis_proposal_notice() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);
    let http_client = reqwest::Client::new();

    let result = execute_tool(
        &ToolCall {
            id: "call-tool-gap-cargo-install".to_string(),
            function: ToolFunction {
                name: "cargo_install".to_string(),
                arguments: serde_json::json!({}).to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-tool-gap-risky",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &http_client,
        None,
    )
    .await;

    assert!(result.is_error, "unknown tool should still error");
    assert!(result.content.contains("Unknown tool: cargo_install"));
    assert!(
        timeout(Duration::from_millis(150), event_rx.recv())
            .await
            .is_err(),
        "risky or mutating CLI-like gaps should not emit automatic synthesis proposals"
    );
}

#[tokio::test]
async fn duplicate_unknown_cli_gap_notice_is_suppressed_per_thread() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);
    let http_client = reqwest::Client::new();

    for call_id in ["call-tool-gap-1", "call-tool-gap-2"] {
        let result = execute_tool(
            &ToolCall {
                id: call_id.to_string(),
                function: ToolFunction {
                    name: "cargo_check".to_string(),
                    arguments: serde_json::json!({}).to_string(),
                },
                weles_review: None,
            },
            &engine,
            "thread-tool-gap-dedupe",
            None,
            &manager,
            None,
            &event_tx,
            root.path(),
            &http_client,
            None,
        )
        .await;
        assert!(result.is_error, "unknown tool should still error");
    }

    let first_notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected first synthesis proposal notice")
        .expect("workflow notice should be received");
    match first_notice {
        AgentEvent::WorkflowNotice { kind, .. } => {
            assert_eq!(kind, "tool-synthesis-proposal");
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }

    assert!(
        timeout(Duration::from_millis(150), event_rx.recv())
            .await
            .is_err(),
        "duplicate missing-tool proposals in the same thread should be suppressed"
    );
}

#[tokio::test]
async fn successful_safe_shell_command_emits_tool_synthesis_proposal_notice() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let repo_root = root.path().join("repo-shell-gap");
    std::fs::create_dir_all(&repo_root).expect("create repo root");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_root)
        .output()
        .expect("git init should succeed");

    let result = execute_tool(
        &ToolCall {
            id: "call-shell-gap-git-status".to_string(),
            function: ToolFunction {
                name: "bash_command".to_string(),
                arguments: serde_json::json!({
                    "command": "git status --short",
                    "cwd": repo_root,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-shell-gap",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "safe shell command should succeed before proposal surfacing: {}",
        result.content
    );

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected synthesis proposal workflow notice")
        .expect("workflow notice should be received");

    match notice {
        AgentEvent::WorkflowNotice {
            kind,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "tool-synthesis-proposal");
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details.get("reason").and_then(|value| value.as_str()),
                Some("successful_safe_shell_cli_gap")
            );
            let synth_args = details
                .get("synthesize_tool_args")
                .expect("proposal should include synthesize_tool args");
            assert_eq!(
                synth_args.get("target").and_then(|value| value.as_str()),
                Some("git status")
            );
            assert_eq!(
                synth_args.get("name").and_then(|value| value.as_str()),
                Some("git_status")
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn successful_risky_shell_command_does_not_emit_tool_synthesis_proposal_notice() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let result = execute_tool(
        &ToolCall {
            id: "call-shell-gap-cargo-install".to_string(),
            function: ToolFunction {
                name: "bash_command".to_string(),
                arguments: serde_json::json!({
                    "command": "printf ok",
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-shell-gap-risky",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "shell command should succeed: {}",
        result.content
    );
    assert!(
        timeout(Duration::from_millis(150), event_rx.recv())
            .await
            .is_err(),
        "non-matching shell commands should not emit synthesis proposals"
    );
}

#[tokio::test]
async fn duplicate_successful_shell_gap_notice_is_suppressed_per_thread() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let repo_root = root.path().join("repo-shell-gap-dedupe");
    std::fs::create_dir_all(&repo_root).expect("create repo root");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_root)
        .output()
        .expect("git init should succeed");

    for call_id in ["call-shell-gap-1", "call-shell-gap-2"] {
        let result = execute_tool(
            &ToolCall {
                id: call_id.to_string(),
                function: ToolFunction {
                    name: "bash_command".to_string(),
                    arguments: serde_json::json!({
                        "command": "git status --short",
                        "cwd": repo_root,
                    })
                    .to_string(),
                },
                weles_review: None,
            },
            &engine,
            "thread-shell-gap-dedupe",
            None,
            &manager,
            None,
            &event_tx,
            root.path(),
            &engine.http_client,
            None,
        )
        .await;
        assert!(
            !result.is_error,
            "shell command should succeed: {}",
            result.content
        );
    }

    let mut synthesis_notice_count = 0;
    for _ in 0..4 {
        if let Ok(Ok(AgentEvent::WorkflowNotice { kind, .. })) =
            timeout(Duration::from_millis(150), event_rx.recv()).await
        {
            if kind == "tool-synthesis-proposal" {
                synthesis_notice_count += 1;
            }
        } else {
            break;
        }
    }

    assert_eq!(
        synthesis_notice_count, 1,
        "duplicate successful shell gaps in the same thread should emit only one synthesis proposal"
    );
}

#[tokio::test]
async fn repeated_shell_fallback_pattern_upgrades_tool_synthesis_proposal_notice() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let repo_root = root.path().join("repo-shell-gap-repeated");
    std::fs::create_dir_all(&repo_root).expect("create repo root");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_root)
        .output()
        .expect("git init should succeed");

    let first = execute_tool(
        &ToolCall {
            id: "call-shell-gap-repeated-1".to_string(),
            function: ToolFunction {
                name: "bash_command".to_string(),
                arguments: serde_json::json!({
                    "command": "git status --short",
                    "cwd": repo_root,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-shell-gap-repeated",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;
    assert!(
        !first.is_error,
        "shell command should succeed: {}",
        first.content
    );

    let first_notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected first workflow notice")
        .expect("workflow notice should be received");
    match first_notice {
        AgentEvent::WorkflowNotice {
            details: Some(details),
            ..
        } => {
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details.get("reason").and_then(|value| value.as_str()),
                Some("successful_safe_shell_cli_gap")
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }

    {
        let mut model = engine.operator_model.write().await;
        model
            .implicit_feedback
            .fallback_histogram
            .insert("search_files -> bash_command".to_string(), 3);
        model.implicit_feedback.top_tool_fallbacks =
            vec!["search_files -> bash_command".to_string()];
    }

    let second = execute_tool(
        &ToolCall {
            id: "call-shell-gap-repeated-2".to_string(),
            function: ToolFunction {
                name: "bash_command".to_string(),
                arguments: serde_json::json!({
                    "command": "git status --short",
                    "cwd": repo_root,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-shell-gap-repeated",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;
    assert!(
        !second.is_error,
        "shell command should succeed: {}",
        second.content
    );

    let second_notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected repeated-pattern workflow notice")
        .expect("workflow notice should be received");
    match second_notice {
        AgentEvent::WorkflowNotice {
            kind,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "tool-synthesis-proposal");
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details.get("reason").and_then(|value| value.as_str()),
                Some("repeated_safe_shell_fallback_cli_gap")
            );
            assert_eq!(
                details
                    .get("matched_fallback")
                    .and_then(|value| value.as_str()),
                Some("search_files -> bash_command")
            );
            assert_eq!(
                details
                    .get("fallback_count")
                    .and_then(|value| value.as_u64()),
                Some(3)
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn unknown_cli_like_tool_does_not_emit_proposal_when_equivalent_generated_tool_exists() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let synth = engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "cli",
                "target": "cargo check",
                "name": "cargo_check",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated tool record");
    let record: serde_json::Value = serde_json::from_str(&synth).expect("parse synth record");
    let tool_name = record
        .get("id")
        .and_then(|value| value.as_str())
        .expect("generated tool id");
    engine
        .activate_generated_tool_json(tool_name)
        .await
        .expect("activate generated tool");

    let result = execute_tool(
        &ToolCall {
            id: "call-tool-gap-existing-cargo-check".to_string(),
            function: ToolFunction {
                name: "cargo_check".to_string(),
                arguments: serde_json::json!({}).to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-tool-gap-existing",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(result.is_error, "unknown tool should still error");
    assert!(result.content.contains("Unknown tool: cargo_check"));
    assert!(
        timeout(Duration::from_millis(150), event_rx.recv())
            .await
            .is_err(),
        "existing generated tool should suppress fresh synthesis proposal notices"
    );
}

#[tokio::test]
async fn successful_safe_shell_command_does_not_emit_proposal_when_equivalent_generated_tool_exists(
) {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "cli",
                "target": "git status",
                "name": "git_status",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated tool record");

    let repo_root = root.path().join("repo-shell-gap-existing");
    std::fs::create_dir_all(&repo_root).expect("create repo root");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_root)
        .output()
        .expect("git init should succeed");

    let result = execute_tool(
        &ToolCall {
            id: "call-shell-gap-existing-git-status".to_string(),
            function: ToolFunction {
                name: "bash_command".to_string(),
                arguments: serde_json::json!({
                    "command": "git status --short",
                    "cwd": repo_root,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-shell-gap-existing",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "shell command should succeed: {}",
        result.content
    );
    assert!(
        timeout(Duration::from_millis(150), event_rx.recv())
            .await
            .is_err(),
        "existing equivalent generated tool should suppress shell fallback synthesis proposal notices"
    );
}

#[tokio::test]
async fn unknown_cli_like_tool_emits_activate_notice_when_equivalent_generated_tool_is_new() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "cli",
                "target": "cargo check",
                "name": "cargo_check",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated tool record");

    let result = execute_tool(
        &ToolCall {
            id: "call-tool-gap-existing-new".to_string(),
            function: ToolFunction {
                name: "cargo_check".to_string(),
                arguments: serde_json::json!({}).to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-tool-gap-existing-new",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(result.is_error, "unknown tool should still error");

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected existing-tool status notice")
        .expect("workflow notice should be received");
    match notice {
        AgentEvent::WorkflowNotice {
            kind,
            message,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "tool-synthesis-proposal");
            assert!(message.contains("Activate it"));
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details.get("reason").and_then(|value| value.as_str()),
                Some("existing_equivalent_generated_tool")
            );
            assert_eq!(
                details
                    .get("recommended_action")
                    .and_then(|value| value.as_str()),
                Some("activate_generated_tool")
            );
            assert_eq!(
                details
                    .get("existing_tool")
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_str()),
                Some("new")
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn successful_safe_shell_command_emits_reuse_notice_when_equivalent_generated_tool_is_active()
{
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let synth = engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "cli",
                "target": "git status",
                "name": "git_status",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated tool record");
    let record: serde_json::Value = serde_json::from_str(&synth).expect("parse synth record");
    let tool_name = record
        .get("id")
        .and_then(|value| value.as_str())
        .expect("generated tool id");
    engine
        .activate_generated_tool_json(tool_name)
        .await
        .expect("activate generated tool");

    let repo_root = root.path().join("repo-shell-gap-existing-active");
    std::fs::create_dir_all(&repo_root).expect("create repo root");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_root)
        .output()
        .expect("git init should succeed");

    let result = execute_tool(
        &ToolCall {
            id: "call-shell-gap-existing-active".to_string(),
            function: ToolFunction {
                name: "bash_command".to_string(),
                arguments: serde_json::json!({
                    "command": "git status --short",
                    "cwd": repo_root,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-shell-gap-existing-active",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "shell command should succeed: {}",
        result.content
    );

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected existing-tool status notice")
        .expect("workflow notice should be received");
    match notice {
        AgentEvent::WorkflowNotice {
            message,
            details: Some(details),
            ..
        } => {
            assert!(message.contains("already active"));
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details
                    .get("recommended_action")
                    .and_then(|value| value.as_str()),
                Some("use_existing_generated_tool")
            );
            assert_eq!(
                details
                    .get("existing_tool")
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_str()),
                Some("active")
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn successful_safe_shell_command_emits_promote_notice_when_equivalent_generated_tool_is_promotable(
) {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.tool_synthesis.enabled = true;
    config.tool_synthesis.sandbox.allow_filesystem = true;
    let engine = AgentEngine::new_test(manager.clone(), config, root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);

    let synth = engine
        .synthesize_tool_json(
            &serde_json::json!({
                "kind": "cli",
                "target": "git status",
                "name": "git_status",
                "activate": false,
            })
            .to_string(),
        )
        .await
        .expect("synthesize generated tool record");
    let record: serde_json::Value = serde_json::from_str(&synth).expect("parse synth record");
    let tool_name = record
        .get("id")
        .and_then(|value| value.as_str())
        .expect("generated tool id");
    engine
        .activate_generated_tool_json(tool_name)
        .await
        .expect("activate generated tool");

    let generated_tool_args = serde_json::json!({});
    for _ in 0..3 {
        engine
            .run_generated_tool_json(
                tool_name,
                &generated_tool_args.to_string(),
                Some("thread-a"),
            )
            .await
            .expect("run generated tool to reach promotable status");
    }

    let repo_root = root.path().join("repo-shell-gap-existing-promotable");
    std::fs::create_dir_all(&repo_root).expect("create repo root");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo_root)
        .output()
        .expect("git init should succeed");

    let result = execute_tool(
        &ToolCall {
            id: "call-shell-gap-existing-promotable".to_string(),
            function: ToolFunction {
                name: "bash_command".to_string(),
                arguments: serde_json::json!({
                    "command": "git status --short",
                    "cwd": repo_root,
                })
                .to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-shell-gap-existing-promotable",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "shell command should succeed: {}",
        result.content
    );

    let notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("expected existing-tool status notice")
        .expect("workflow notice should be received");
    match notice {
        AgentEvent::WorkflowNotice {
            message,
            details: Some(details),
            ..
        } => {
            assert!(message.contains("already promotable"));
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("notice details should be json");
            assert_eq!(
                details
                    .get("recommended_action")
                    .and_then(|value| value.as_str()),
                Some("promote_generated_tool")
            );
            assert_eq!(
                details
                    .get("existing_tool")
                    .and_then(|value| value.get("status"))
                    .and_then(|value| value.as_str()),
                Some("promotable")
            );
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn read_memory_includes_thread_structural_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-graph-neighbors";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_301,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:demo",
            "demo",
            "package",
            Some("cargo package from Cargo.toml"),
            1_717_180_302,
        )
        .await
        .expect("persist package node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:package:cargo:demo",
            "file_in_package",
            2.0,
            1_717_180_303,
        )
        .await
        .expect("persist graph edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-graph-neighbors".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 5,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "read_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_memory should return JSON");
    let neighbors = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("graph neighbors should be present");
    assert!(neighbors.iter().any(|item| {
        item.get("node_id").and_then(|value| value.as_str()) == Some("node:package:cargo:demo")
            && item.get("relation_type").and_then(|value| value.as_str()) == Some("file_in_package")
    }));
}

#[tokio::test]
async fn search_memory_uses_preferred_structural_refs_for_graph_lookup_without_memory_graph() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-preferred-structural-lookup";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory preferred structural lookup",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search parser graph",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/lib.rs".to_string(),
                    relative_path: "src/lib.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/parser.rs".to_string(),
                    relative_path: "src/parser.rs".to_string(),
                },
            ],
            edges: vec![crate::agent::context::structural_memory::StructuralEdge {
                from: "node:file:src/lib.rs".to_string(),
                to: "node:file:src/parser.rs".to_string(),
                kind: "imported_file".to_string(),
            }],
            ..Default::default()
        },
    );

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-preferred-structural-lookup".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "parser",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let preferred_refs = payload
        .get("thread_structural_memory")
        .and_then(|value| value.get("preferred_refs"))
        .and_then(|value| value.as_array())
        .expect("search_memory should expose preferred structural refs");
    assert!(preferred_refs
        .iter()
        .any(|value| value.as_str() == Some("node:file:src/lib.rs")));
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    assert!(matches.iter().any(|item| {
        item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
            && item
                .get("snippet")
                .and_then(|value| value.as_str())
                .is_some_and(|snippet| {
                    snippet.contains("Graph lookup from node:file:src/lib.rs")
                        && snippet.contains("src/parser.rs")
                })
    }));
}

#[tokio::test]
async fn search_memory_matches_thread_structural_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-graph-neighbors";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory graph neighbors",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search graph neighbors",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_401,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:graph-demo",
            "graph-demo",
            "package",
            Some("package linked from structural graph"),
            1_717_180_402,
        )
        .await
        .expect("persist package node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:package:cargo:graph-demo",
            "file_in_package",
            2.0,
            1_717_180_403,
        )
        .await
        .expect("persist graph edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-graph-neighbors".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "graph-demo",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    assert!(matches.iter().any(|item| {
        item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
            && item
                .get("snippet")
                .and_then(|value| value.as_str())
                .is_some_and(|snippet| snippet.contains("graph-demo"))
    }));
}

#[tokio::test]
async fn search_memory_exposes_thread_structural_graph_neighbors_metadata() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-graph-neighbor-metadata";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory graph neighbor metadata",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search graph neighbor metadata",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_451,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:graph-metadata-demo",
            "graph-metadata-demo",
            "package",
            Some("package linked from structural graph metadata"),
            1_717_180_452,
        )
        .await
        .expect("persist package node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:package:cargo:graph-metadata-demo",
            "file_in_package",
            2.0,
            1_717_180_453,
        )
        .await
        .expect("persist graph edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-graph-neighbor-metadata".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "graph-metadata-demo",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let thread_structural_memory = payload
        .get("thread_structural_memory")
        .expect("search_memory should expose thread structural memory metadata");
    let neighbors = thread_structural_memory
        .get("graph_neighbors")
        .and_then(|value| value.as_array())
        .expect("search_memory should expose graph neighbors metadata");
    assert!(neighbors.iter().any(|item| {
        item.get("node_id").and_then(|value| value.as_str())
            == Some("node:package:cargo:graph-metadata-demo")
            && item.get("relation_type").and_then(|value| value.as_str()) == Some("file_in_package")
    }));
}

#[tokio::test]
async fn read_memory_includes_second_hop_thread_structural_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-second-hop-graph-neighbors";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_501,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:demo-two-hop",
            "demo-two-hop",
            "package",
            Some("intermediate cargo package"),
            1_717_180_502,
        )
        .await
        .expect("persist package node");
    engine
        .history
        .upsert_memory_node(
            "node:task:graph-two-hop-task",
            "graph-two-hop-task",
            "task",
            Some("second hop task node"),
            1_717_180_503,
        )
        .await
        .expect("persist task node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:package:cargo:demo-two-hop",
            "file_in_package",
            2.0,
            1_717_180_504,
        )
        .await
        .expect("persist file/package edge");
    engine
        .history
        .upsert_memory_edge(
            "node:package:cargo:demo-two-hop",
            "node:task:graph-two-hop-task",
            "package_supports_task",
            1.5,
            1_717_180_505,
        )
        .await
        .expect("persist package/task edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-second-hop-graph-neighbors".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 6,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "read_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_memory should return JSON");
    let neighbors = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("graph neighbors should be present");
    assert!(neighbors.iter().any(|item| {
        item.get("node_id").and_then(|value| value.as_str()) == Some("node:task:graph-two-hop-task")
    }));
}

#[tokio::test]
async fn search_memory_matches_second_hop_thread_structural_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-second-hop-graph-neighbors";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory second hop graph neighbors",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search second hop graph neighbors",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_601,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:demo-second-hop",
            "demo-second-hop",
            "package",
            Some("intermediate cargo package"),
            1_717_180_602,
        )
        .await
        .expect("persist package node");
    engine
        .history
        .upsert_memory_node(
            "node:error:two-hop:unique-needle",
            "two-hop-needle",
            "error",
            Some("unique second hop memory graph needle"),
            1_717_180_603,
        )
        .await
        .expect("persist error node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:package:cargo:demo-second-hop",
            "file_in_package",
            2.0,
            1_717_180_604,
        )
        .await
        .expect("persist file/package edge");
    engine
        .history
        .upsert_memory_edge(
            "node:package:cargo:demo-second-hop",
            "node:error:two-hop:unique-needle",
            "package_linked_error",
            1.5,
            1_717_180_605,
        )
        .await
        .expect("persist package/error edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-second-hop-graph-neighbors".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "unique second hop memory graph needle",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    assert!(matches.iter().any(|item| {
        item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
            && item
                .get("snippet")
                .and_then(|value| value.as_str())
                .is_some_and(|snippet| snippet.contains("unique second hop memory graph needle"))
    }));
}

#[tokio::test]
async fn read_memory_includes_third_hop_thread_structural_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-third-hop-graph-neighbors";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_701,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:demo-three-hop",
            "demo-three-hop",
            "package",
            Some("first hop package"),
            1_717_180_702,
        )
        .await
        .expect("persist package node");
    engine
        .history
        .upsert_memory_node(
            "node:task:graph-middle-hop",
            "graph-middle-hop",
            "task",
            Some("second hop task"),
            1_717_180_703,
        )
        .await
        .expect("persist middle hop task node");
    engine
        .history
        .upsert_memory_node(
            "node:error:graph-third-hop-needle",
            "graph-third-hop-needle",
            "error",
            Some("third hop graph neighbor target"),
            1_717_180_704,
        )
        .await
        .expect("persist third hop node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:package:cargo:demo-three-hop",
            "file_in_package",
            3.0,
            1_717_180_705,
        )
        .await
        .expect("persist file/package edge");
    engine
        .history
        .upsert_memory_edge(
            "node:package:cargo:demo-three-hop",
            "node:task:graph-middle-hop",
            "package_supports_task",
            2.0,
            1_717_180_706,
        )
        .await
        .expect("persist package/task edge");
    engine
        .history
        .upsert_memory_edge(
            "node:task:graph-middle-hop",
            "node:error:graph-third-hop-needle",
            "task_hit_error",
            1.0,
            1_717_180_707,
        )
        .await
        .expect("persist task/error edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-third-hop-graph-neighbors".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 8,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "read_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_memory should return JSON");
    let neighbors = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("graph neighbors should be present");
    let retained = neighbors
        .iter()
        .find(|item| {
            item.get("node_id").and_then(|value| value.as_str())
                == Some("node:error:graph-third-hop-needle")
        })
        .expect("third-hop graph neighbor should be present");
    assert_eq!(
        retained.get("hop_count").and_then(|value| value.as_u64()),
        Some(3),
        "third-hop graph neighbor should expose the retained path depth"
    );
    assert!(
        retained
            .get("path_summary")
            .and_then(|value| value.as_str())
            .is_some_and(|summary| {
                summary.contains("src/lib.rs")
                    && summary.contains("demo-three-hop")
                    && summary.contains("graph-middle-hop")
                    && summary.contains("graph-third-hop-needle")
            }),
        "third-hop graph neighbor should expose the connecting path summary"
    );
    let path = retained
        .get("path")
        .and_then(|value| value.as_array())
        .expect("third-hop graph neighbor should expose the retained path");
    assert_eq!(
        path.len(),
        4,
        "path should include seed, intermediate hops, and target"
    );
    assert_eq!(
        path[1]
            .get("relation_type")
            .and_then(|value| value.as_str()),
        Some("file_in_package")
    );
    assert_eq!(
        path[2]
            .get("relation_type")
            .and_then(|value| value.as_str()),
        Some("package_supports_task")
    );
    assert_eq!(
        path[3]
            .get("relation_type")
            .and_then(|value| value.as_str()),
        Some("task_hit_error")
    );
}

#[tokio::test]
async fn search_memory_matches_third_hop_thread_structural_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-third-hop-graph-neighbors";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory third hop graph neighbors",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search third hop graph neighbors",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_801,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:demo-search-three-hop",
            "demo-search-three-hop",
            "package",
            Some("first hop package"),
            1_717_180_802,
        )
        .await
        .expect("persist package node");
    engine
        .history
        .upsert_memory_node(
            "node:task:search-third-hop-middle",
            "search-third-hop-middle",
            "task",
            Some("second hop task"),
            1_717_180_803,
        )
        .await
        .expect("persist middle hop task node");
    engine
        .history
        .upsert_memory_node(
            "node:error:third-hop-search-needle",
            "third-hop-search-needle",
            "error",
            Some("unique third hop memory graph needle"),
            1_717_180_804,
        )
        .await
        .expect("persist third hop node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:package:cargo:demo-search-three-hop",
            "file_in_package",
            3.0,
            1_717_180_805,
        )
        .await
        .expect("persist file/package edge");
    engine
        .history
        .upsert_memory_edge(
            "node:package:cargo:demo-search-three-hop",
            "node:task:search-third-hop-middle",
            "package_supports_task",
            2.0,
            1_717_180_806,
        )
        .await
        .expect("persist package/task edge");
    engine
        .history
        .upsert_memory_edge(
            "node:task:search-third-hop-middle",
            "node:error:third-hop-search-needle",
            "task_hit_error",
            1.0,
            1_717_180_807,
        )
        .await
        .expect("persist task/error edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-third-hop-graph-neighbors".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "unique third hop memory graph needle",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let neighbors = payload
        .get("thread_structural_memory")
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("search_memory should expose graph neighbors metadata");
    let retained = neighbors
        .iter()
        .find(|item| {
            item.get("node_id").and_then(|value| value.as_str())
                == Some("node:error:third-hop-search-needle")
        })
        .expect("search metadata should retain the third-hop graph path");
    assert_eq!(
        retained.get("hop_count").and_then(|value| value.as_u64()),
        Some(3),
        "search metadata should preserve the retained path depth"
    );
    assert!(
        retained
            .get("path_summary")
            .and_then(|value| value.as_str())
            .is_some_and(|summary| {
                summary.contains("src/lib.rs")
                    && summary.contains("demo-search-three-hop")
                    && summary.contains("search-third-hop-middle")
                    && summary.contains("third-hop-search-needle")
            }),
        "search metadata should expose the connecting path summary"
    );
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    assert!(matches.iter().any(|item| {
        item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
            && item
                .get("snippet")
                .and_then(|value| value.as_str())
                .is_some_and(|snippet| snippet.contains("unique third hop memory graph needle"))
    }));
}

#[tokio::test]
async fn read_memory_excludes_structural_seed_nodes_from_deeper_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-excludes-structural-seed-graph-neighbors";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            workspace_seeds: vec![crate::agent::context::structural_memory::WorkspaceSeed {
                node_id: "node:package:cargo:seed-package".to_string(),
                relative_path: "Cargo.toml".to_string(),
                kind: "package".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_180_901,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:seed-package",
            "seed-package",
            "package",
            Some("structural seed package"),
            1_717_180_902,
        )
        .await
        .expect("persist seed package node");
    engine
        .history
        .upsert_memory_node(
            "node:task:loop-middle",
            "loop-middle",
            "task",
            Some("middle hop task"),
            1_717_180_903,
        )
        .await
        .expect("persist middle hop node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:loop-middle",
            "file_supports_task",
            3.0,
            1_717_180_904,
        )
        .await
        .expect("persist file/task edge");
    engine
        .history
        .upsert_memory_edge(
            "node:task:loop-middle",
            "node:package:cargo:seed-package",
            "task_in_package",
            2.0,
            1_717_180_905,
        )
        .await
        .expect("persist task/package edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-excludes-structural-seed-graph-neighbors".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 8,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "read_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_memory should return JSON");
    let neighbors = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("graph neighbors should be present");
    assert!(neighbors.iter().any(|item| {
        item.get("node_id").and_then(|value| value.as_str()) == Some("node:task:loop-middle")
    }));
    assert!(
        !neighbors.iter().any(|item| {
            item.get("node_id").and_then(|value| value.as_str())
                == Some("node:package:cargo:seed-package")
                && item.get("relation_type").and_then(|value| value.as_str())
                    == Some("task_in_package")
        }),
        "structural seed node should not resurface as a deeper graph neighbor"
    );
}

#[tokio::test]
async fn search_memory_excludes_structural_seed_nodes_from_deeper_graph_neighbors() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-excludes-structural-seed-graph-neighbors";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory excludes structural seed graph neighbors",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search structural seed graph neighbors",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            workspace_seeds: vec![crate::agent::context::structural_memory::WorkspaceSeed {
                node_id: "node:package:cargo:search-seed-package".to_string(),
                relative_path: "Cargo.toml".to_string(),
                kind: "package".to_string(),
            }],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file"),
            1_717_181_001,
        )
        .await
        .expect("persist file node");
    engine
        .history
        .upsert_memory_node(
            "node:package:cargo:search-seed-package",
            "search-seed-package",
            "package",
            Some("structural seed package"),
            1_717_181_002,
        )
        .await
        .expect("persist seed package node");
    engine
        .history
        .upsert_memory_node(
            "node:task:search-loop-middle",
            "search-loop-middle",
            "task",
            Some("middle hop task unique-seed-needle"),
            1_717_181_003,
        )
        .await
        .expect("persist middle hop node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:search-loop-middle",
            "file_supports_task",
            3.0,
            1_717_181_004,
        )
        .await
        .expect("persist file/task edge");
    engine
        .history
        .upsert_memory_edge(
            "node:task:search-loop-middle",
            "node:package:cargo:search-seed-package",
            "task_in_package",
            2.0,
            1_717_181_005,
        )
        .await
        .expect("persist task/package edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-excludes-structural-seed-graph-neighbors".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "unique-seed-needle",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    assert!(
        matches.iter().any(|item| {
            item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
                && item.get("source").and_then(|value| value.as_str())
                    == Some("node:task:search-loop-middle")
                && item
                    .get("snippet")
                    .and_then(|value| value.as_str())
                    .is_some_and(|snippet| snippet.contains("unique-seed-needle"))
        }),
        "search should still find the middle-hop task summary"
    );
    assert!(
        !matches.iter().any(|item| {
            item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
                && item.get("source").and_then(|value| value.as_str())
                    == Some("node:package:cargo:search-seed-package")
                && item
                    .get("snippet")
                    .and_then(|value| value.as_str())
                    .is_some_and(|snippet| snippet.contains("via task in package"))
        }),
        "search results should not include graph-neighbor resurfacing of the structural seed node"
    );
}

#[tokio::test]
async fn read_memory_deduplicates_deeper_graph_neighbors_reached_from_multiple_seeds() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-dedup-deeper-graph-neighbors";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/lib.rs".to_string(),
                    relative_path: "src/lib.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/main.rs".to_string(),
                    relative_path: "src/main.rs".to_string(),
                },
            ],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file one"),
            1_717_181_101,
        )
        .await
        .expect("persist first file node");
    engine
        .history
        .upsert_memory_node(
            "node:file:src/main.rs",
            "src/main.rs",
            "file",
            Some("observed file two"),
            1_717_181_102,
        )
        .await
        .expect("persist second file node");
    engine
        .history
        .upsert_memory_node(
            "node:task:shared-deep-task",
            "shared-deep-task",
            "task",
            Some("shared deeper task node"),
            1_717_181_103,
        )
        .await
        .expect("persist shared deep task node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:shared-deep-task",
            "file_supports_task",
            2.0,
            1_717_181_104,
        )
        .await
        .expect("persist first file/task edge");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/main.rs",
            "node:task:shared-deep-task",
            "file_supports_task",
            1.5,
            1_717_181_105,
        )
        .await
        .expect("persist second file/task edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-dedup-deeper-graph-neighbors".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 8,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "read_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_memory should return JSON");
    let neighbors = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("graph neighbors should be present");
    let duplicate_count = neighbors
        .iter()
        .filter(|item| {
            item.get("node_id").and_then(|value| value.as_str())
                == Some("node:task:shared-deep-task")
        })
        .count();
    assert_eq!(
        duplicate_count, 1,
        "shared deeper node should appear only once in graph neighbors even when reachable from multiple structural seeds"
    );
}

#[tokio::test]
async fn search_memory_deduplicates_deeper_graph_neighbors_reached_from_multiple_seeds() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-dedup-deeper-graph-neighbors";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory dedup deeper graph neighbors",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search dedup deeper graph neighbors",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/lib.rs".to_string(),
                    relative_path: "src/lib.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/main.rs".to_string(),
                    relative_path: "src/main.rs".to_string(),
                },
            ],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file one"),
            1_717_181_201,
        )
        .await
        .expect("persist first file node");
    engine
        .history
        .upsert_memory_node(
            "node:file:src/main.rs",
            "src/main.rs",
            "file",
            Some("observed file two"),
            1_717_181_202,
        )
        .await
        .expect("persist second file node");
    engine
        .history
        .upsert_memory_node(
            "node:task:shared-search-deep-task",
            "shared-search-deep-task",
            "task",
            Some("shared dedup search unique-graph-dedup-needle"),
            1_717_181_203,
        )
        .await
        .expect("persist shared deep task node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:shared-search-deep-task",
            "file_supports_task",
            2.0,
            1_717_181_204,
        )
        .await
        .expect("persist first file/task edge");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/main.rs",
            "node:task:shared-search-deep-task",
            "file_supports_task",
            1.5,
            1_717_181_205,
        )
        .await
        .expect("persist second file/task edge");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-dedup-deeper-graph-neighbors".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "unique-graph-dedup-needle",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    let duplicate_count = matches
        .iter()
        .filter(|item| {
            item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
                && item.get("source").and_then(|value| value.as_str())
                    == Some("node:task:shared-search-deep-task")
        })
        .count();
    assert_eq!(
        duplicate_count, 1,
        "shared deeper node should surface only once in search results even when reachable from multiple structural seeds"
    );
}

#[tokio::test]
async fn read_memory_prefers_stronger_edge_when_shared_neighbor_is_reached_multiple_times() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-prefers-stronger-edge-shared-neighbor";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/lib.rs".to_string(),
                    relative_path: "src/lib.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/main.rs".to_string(),
                    relative_path: "src/main.rs".to_string(),
                },
            ],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file one"),
            1_717_181_301,
        )
        .await
        .expect("persist first file node");
    engine
        .history
        .upsert_memory_node(
            "node:file:src/main.rs",
            "src/main.rs",
            "file",
            Some("observed file two"),
            1_717_181_302,
        )
        .await
        .expect("persist second file node");
    engine
        .history
        .upsert_memory_node(
            "node:task:weighted-shared-neighbor",
            "weighted-shared-neighbor",
            "task",
            Some("shared neighbor for edge weight preference"),
            1_717_181_303,
        )
        .await
        .expect("persist shared neighbor node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:weighted-shared-neighbor",
            "weak_file_supports_task",
            1.0,
            1_717_181_304,
        )
        .await
        .expect("persist weaker edge first");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/main.rs",
            "node:task:weighted-shared-neighbor",
            "strong_file_supports_task",
            5.0,
            1_717_181_305,
        )
        .await
        .expect("persist stronger edge second");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-prefers-stronger-edge-shared-neighbor".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 8,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "read_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_memory should return JSON");
    let neighbors = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("graph neighbors should be present");
    let retained = neighbors
        .iter()
        .find(|item| {
            item.get("node_id").and_then(|value| value.as_str())
                == Some("node:task:weighted-shared-neighbor")
        })
        .expect("shared neighbor should be present once");
    assert_eq!(
        retained
            .get("relation_type")
            .and_then(|value| value.as_str()),
        Some("strong_file_supports_task"),
        "when a shared neighbor is reachable multiple times, the stronger edge should determine the retained relation"
    );
}

#[tokio::test]
async fn search_memory_prefers_stronger_edge_when_shared_neighbor_is_reached_multiple_times() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-prefers-stronger-edge-shared-neighbor";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory prefers stronger edge shared neighbor",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search stronger edge shared neighbor",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/lib.rs".to_string(),
                    relative_path: "src/lib.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/main.rs".to_string(),
                    relative_path: "src/main.rs".to_string(),
                },
            ],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file one"),
            1_717_181_401,
        )
        .await
        .expect("persist first file node");
    engine
        .history
        .upsert_memory_node(
            "node:file:src/main.rs",
            "src/main.rs",
            "file",
            Some("observed file two"),
            1_717_181_402,
        )
        .await
        .expect("persist second file node");
    engine
        .history
        .upsert_memory_node(
            "node:task:weighted-search-shared-neighbor",
            "weighted-search-shared-neighbor",
            "task",
            Some("shared search neighbor unique-weighted-edge-needle"),
            1_717_181_403,
        )
        .await
        .expect("persist shared neighbor node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:weighted-search-shared-neighbor",
            "weak_file_supports_task",
            1.0,
            1_717_181_404,
        )
        .await
        .expect("persist weaker edge first");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/main.rs",
            "node:task:weighted-search-shared-neighbor",
            "strong_file_supports_task",
            5.0,
            1_717_181_405,
        )
        .await
        .expect("persist stronger edge second");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-prefers-stronger-edge-shared-neighbor".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "unique-weighted-edge-needle",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    let retained = matches
        .iter()
        .find(|item| {
            item.get("layer").and_then(|value| value.as_str()) == Some("thread_structural_memory")
                && item.get("source").and_then(|value| value.as_str())
                    == Some("node:task:weighted-search-shared-neighbor")
        })
        .expect("shared search neighbor should be present once");
    assert!(
        retained
            .get("snippet")
            .and_then(|value| value.as_str())
            .is_some_and(|snippet| snippet.contains("via strong file supports task")),
        "search result snippet should reflect the stronger retained edge"
    );
}

#[tokio::test]
async fn read_memory_orders_graph_neighbors_by_strongest_retained_edge_weight() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-orders-graph-neighbors-by-weight";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/lib.rs".to_string(),
                    relative_path: "src/lib.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/main.rs".to_string(),
                    relative_path: "src/main.rs".to_string(),
                },
            ],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file one"),
            1_717_181_501,
        )
        .await
        .expect("persist first file node");
    engine
        .history
        .upsert_memory_node(
            "node:file:src/main.rs",
            "src/main.rs",
            "file",
            Some("observed file two"),
            1_717_181_502,
        )
        .await
        .expect("persist second file node");
    engine
        .history
        .upsert_memory_node(
            "node:task:lower-weight-neighbor",
            "lower-weight-neighbor",
            "task",
            Some("lower weight neighbor"),
            1_717_181_503,
        )
        .await
        .expect("persist lower neighbor node");
    engine
        .history
        .upsert_memory_node(
            "node:task:higher-weight-neighbor",
            "higher-weight-neighbor",
            "task",
            Some("higher weight neighbor"),
            1_717_181_504,
        )
        .await
        .expect("persist higher neighbor node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:lower-weight-neighbor",
            "file_supports_task",
            2.0,
            1_717_181_505,
        )
        .await
        .expect("persist lower-weight edge first");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/main.rs",
            "node:task:higher-weight-neighbor",
            "file_supports_task",
            6.0,
            1_717_181_506,
        )
        .await
        .expect("persist higher-weight edge second");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-orders-graph-neighbors-by-weight".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 8,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "read_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_memory should return JSON");
    let neighbors = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_neighbors"))
        .and_then(|value| value.as_array())
        .expect("graph neighbors should be present");
    assert!(
        neighbors.len() >= 2,
        "expected at least two graph neighbors"
    );
    assert_eq!(
        neighbors[0]
            .get("node_id")
            .and_then(|value| value.as_str()),
        Some("node:task:higher-weight-neighbor"),
        "higher-weight retained neighbor should be ordered before lower-weight neighbor regardless of seed traversal order"
    );
}

#[tokio::test]
async fn search_memory_orders_graph_neighbors_by_strongest_retained_edge_weight() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-memory-orders-graph-neighbors-by-weight";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search memory orders graph neighbors by weight",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search graph neighbor weight ordering",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/lib.rs".to_string(),
                    relative_path: "src/lib.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "node:file:src/main.rs".to_string(),
                    relative_path: "src/main.rs".to_string(),
                },
            ],
            ..Default::default()
        },
    );
    engine
        .history
        .upsert_memory_node(
            "node:file:src/lib.rs",
            "src/lib.rs",
            "file",
            Some("observed file one"),
            1_717_181_601,
        )
        .await
        .expect("persist first file node");
    engine
        .history
        .upsert_memory_node(
            "node:file:src/main.rs",
            "src/main.rs",
            "file",
            Some("observed file two"),
            1_717_181_602,
        )
        .await
        .expect("persist second file node");
    engine
        .history
        .upsert_memory_node(
            "node:task:aaa-lower-weight-neighbor",
            "aaa-lower-weight-neighbor",
            "task",
            Some("equal-score graph ordering needle"),
            1_717_181_603,
        )
        .await
        .expect("persist lower neighbor node");
    engine
        .history
        .upsert_memory_node(
            "node:task:zzz-higher-weight-neighbor",
            "zzz-higher-weight-neighbor",
            "task",
            Some("equal-score graph ordering needle"),
            1_717_181_604,
        )
        .await
        .expect("persist higher neighbor node");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/lib.rs",
            "node:task:aaa-lower-weight-neighbor",
            "file_supports_task",
            2.0,
            1_717_181_605,
        )
        .await
        .expect("persist lower-weight edge first");
    engine
        .history
        .upsert_memory_edge(
            "node:file:src/main.rs",
            "node:task:zzz-higher-weight-neighbor",
            "file_supports_task",
            6.0,
            1_717_181_606,
        )
        .await
        .expect("persist higher-weight edge second");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-memory-orders-graph-neighbors-by-weight".to_string(),
        ToolFunction {
            name: "search_memory".to_string(),
            arguments: serde_json::json!({
                "query": "equal-score graph ordering needle",
                "limit": 5,
                "include_base_markdown": false,
                "include_operator_profile_json": false,
                "include_operator_model_summary": false,
                "include_thread_structural_memory": true
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        thread_id,
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "search_memory should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_memory should return JSON");
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_memory should return matches");
    assert!(matches.len() >= 2, "expected at least two search matches");
    assert_eq!(
        matches[0]
            .get("source")
            .and_then(|value| value.as_str()),
        Some("node:task:zzz-higher-weight-neighbor"),
        "when graph-backed search candidates tie textually, the higher-weight retained neighbor should rank first"
    );
}

#[tokio::test]
async fn execute_tool_records_behavioral_event_for_successful_outcome() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);

    let file_path = root.path().join("behavioral-success.txt");
    std::fs::write(&file_path, "hello behavioral event\n").expect("write success fixture");

    let result = execute_tool(
        &ToolCall::with_default_weles_review(
            "tool-read-success-behavioral".to_string(),
            ToolFunction {
                name: "read_file".to_string(),
                arguments: serde_json::json!({
                    "path": file_path,
                    "limit": 20,
                })
                .to_string(),
            },
        ),
        &engine,
        "thread-tool-success",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(!result.is_error, "read_file should succeed: {}", result.content);

    let events = engine
        .history
        .list_agent_events(Some("behavioral"), Some("thread-tool-success"), Some(20))
        .await
        .expect("behavioral events should list");
    let row = events
        .into_iter()
        .find(|row| row.kind == "tool_execution_outcome")
        .expect("tool execution outcome event should exist");
    let payload: serde_json::Value =
        serde_json::from_str(&row.payload_json).expect("behavioral payload should be valid json");
    assert_eq!(payload.get("thread_id").and_then(|v| v.as_str()), Some("thread-tool-success"));
    let nested = payload.get("payload").expect("payload envelope should contain payload field");
    assert_eq!(nested.get("tool_call_id").and_then(|v| v.as_str()), Some("tool-read-success-behavioral"));
    assert_eq!(nested.get("requested_tool_name").and_then(|v| v.as_str()), Some("read_file"));
    assert_eq!(nested.get("dispatch_tool_name").and_then(|v| v.as_str()), Some("read_file"));
    assert_eq!(nested.get("status").and_then(|v| v.as_str()), Some("success"));
    assert_eq!(nested.get("is_error").and_then(|v| v.as_bool()), Some(false));
}

#[tokio::test]
async fn execute_tool_records_behavioral_event_for_error_outcome() {
    let root = tempdir().expect("tempdir should succeed");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);

    let result = execute_tool(
        &ToolCall::with_default_weles_review(
            "tool-read-error-behavioral".to_string(),
            ToolFunction {
                name: "read_file".to_string(),
                arguments: serde_json::json!({
                    "path": root.path().join("missing-file.txt"),
                    "limit": 20,
                })
                .to_string(),
            },
        ),
        &engine,
        "thread-tool-error",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &engine.http_client,
        None,
    )
    .await;

    assert!(result.is_error, "missing file should fail");

    let events = engine
        .history
        .list_agent_events(Some("behavioral"), Some("thread-tool-error"), Some(20))
        .await
        .expect("behavioral events should list");
    let row = events
        .into_iter()
        .find(|row| row.kind == "tool_execution_outcome")
        .expect("tool execution outcome event should exist");
    let payload: serde_json::Value =
        serde_json::from_str(&row.payload_json).expect("behavioral payload should be valid json");
    assert_eq!(payload.get("thread_id").and_then(|v| v.as_str()), Some("thread-tool-error"));
    let nested = payload.get("payload").expect("payload envelope should contain payload field");
    assert_eq!(nested.get("tool_call_id").and_then(|v| v.as_str()), Some("tool-read-error-behavioral"));
    assert_eq!(nested.get("requested_tool_name").and_then(|v| v.as_str()), Some("read_file"));
    assert_eq!(nested.get("dispatch_tool_name").and_then(|v| v.as_str()), Some("read_file"));
    assert_eq!(nested.get("status").and_then(|v| v.as_str()), Some("error"));
    assert_eq!(nested.get("is_error").and_then(|v| v.as_bool()), Some(true));
}
