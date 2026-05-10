use super::super::*;
use super::super::{
    execute_tool,
    get_available_tools,
};
use crate::agent::{
    types::{AgentConfig, ToolCall, ToolFunction},
    AgentEngine,
};
use crate::session_manager::SessionManager;
use tempfile::tempdir;
use tokio::sync::broadcast;

#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

pub(super) fn make_thread(
    id: &str,
    agent_name: Option<&str>,
    title: &str,
    pinned: bool,
    created_at: u64,
    updated_at: u64,
    messages: Vec<crate::agent::types::AgentMessage>,
) -> crate::agent::types::AgentThread {
    crate::agent::types::AgentThread {
        id: id.to_string(),
        agent_name: agent_name.map(ToOwned::to_owned),
        title: title.to_string(),
        messages,
        pinned,
        upstream_thread_id: None,
        upstream_transport: None,
        upstream_provider: None,
        upstream_model: None,
        upstream_assistant_id: None,
        total_input_tokens: 0,
        total_output_tokens: 0,
        created_at,
        updated_at,
    }
}

pub(super) fn weles_internal_message(ts: u64) -> crate::agent::types::AgentMessage {
    let mut message = crate::agent::types::AgentMessage::user(
        crate::agent::agent_identity::build_weles_persona_prompt(
            crate::agent::agent_identity::WELES_GOVERNANCE_SCOPE,
        ),
        ts,
    );
    message.role = crate::agent::types::MessageRole::Assistant;
    message
}

pub(super) fn write_scope_memory_files(
    agent_data_dir: &std::path::Path,
    scope_id: &str,
    soul: &str,
    memory: &str,
    user: &str,
) -> crate::agent::task_prompt::MemoryPaths {
    let paths = crate::agent::task_prompt::memory_paths_for_scope(agent_data_dir, scope_id);
    std::fs::create_dir_all(&paths.memory_dir).expect("create scope memory dir");
    if let Some(parent) = paths.user_path.parent() {
        std::fs::create_dir_all(parent).expect("create shared user dir");
    }
    std::fs::write(&paths.soul_path, soul).expect("write soul");
    std::fs::write(&paths.memory_path, memory).expect("write memory");
    std::fs::write(&paths.user_path, user).expect("write user");
    paths
}

pub(super) async fn current_scope_memory(
    agent_data_dir: &std::path::Path,
) -> crate::agent::types::AgentMemory {
    let paths = crate::agent::task_prompt::memory_paths_for_scope(
        agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
    );
    crate::agent::types::AgentMemory {
        soul: tokio::fs::read_to_string(&paths.soul_path)
            .await
            .expect("read soul"),
        memory: tokio::fs::read_to_string(&paths.memory_path)
            .await
            .expect("read memory"),
        user_profile: tokio::fs::read_to_string(&paths.user_path)
            .await
            .expect("read user"),
    }
}

pub(super) fn build_matching_injection_state(
    memory: &crate::agent::types::AgentMemory,
    paths: &crate::agent::task_prompt::MemoryPaths,
) -> crate::agent::memory_context::PromptMemoryInjectionState {
    let summary =
        crate::agent::memory_context::build_structured_memory_summary(memory, paths, None, None);
    crate::agent::memory_context::build_prompt_memory_injection_state(&summary, false)
}

pub(super) fn sample_task_with_scope(
    id: &str,
    thread_id: Option<&str>,
    scope_id: &str,
) -> crate::agent::types::AgentTask {
    crate::agent::types::AgentTask {
        id: id.to_string(),
        title: id.to_string(),
        description: String::new(),
        status: crate::agent::types::TaskStatus::Queued,
        priority: crate::agent::types::TaskPriority::Normal,
        progress: 0,
        created_at: 0,
        started_at: None,
        completed_at: None,
        error: None,
        result: None,
        thread_id: thread_id.map(str::to_string),
        source: "user".to_string(),
        notify_on_complete: false,
        notify_channels: Vec::new(),
        dependencies: Vec::new(),
        command: None,
        session_id: None,
        goal_run_id: None,
        goal_run_title: None,
        goal_step_id: None,
        goal_step_title: None,
        parent_task_id: None,
        parent_thread_id: None,
        runtime: "daemon".to_string(),
        retry_count: 0,
        max_retries: 3,
        next_retry_at: None,
        scheduled_at: None,
        blocked_reason: None,
        awaiting_approval_id: None,
        policy_fingerprint: None,
        approval_expires_at: None,
        containment_scope: None,
        compensation_status: None,
        compensation_summary: None,
        lane_id: None,
        last_error: None,
        logs: Vec::new(),
        tool_whitelist: None,
        tool_blacklist: None,
        context_budget_tokens: None,
        context_overflow_action: None,
        termination_conditions: None,
        success_criteria: None,
        max_duration_secs: None,
        supervisor_config: None,
        override_provider: None,
        override_model: None,
        override_system_prompt: Some(format!(
            "Agent persona: Test Persona\nAgent persona id: {scope_id}\nYou are Test Persona ({scope_id}) operating as a spawned zorai agent."
        )),
        sub_agent_def_id: None,
    }
}

#[tokio::test]
async fn update_memory_from_custom_agent_scope_writes_only_custom_memory() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let agent_data_dir = root.path().join("agent");
    let main_paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Main soul fact\n",
        "# Memory\n\n- Main scope fact\n",
        "# User\n\n- Shared user fact\n",
    );
    let custom_paths = write_scope_memory_files(
        &agent_data_dir,
        "dola",
        "# Soul\n\n- Dola soul fact\n",
        "# Memory\n\n- Dola scope fact\n",
        "# User\n\n- Shared user fact\n",
    );
    let tool_call = ToolCall::with_default_weles_review(
        "tool-update-custom-memory".to_string(),
        ToolFunction {
            name: "update_memory".to_string(),
            arguments: serde_json::json!({
                "target": "memory",
                "mode": "append",
                "content": "- Dola private update"
            })
            .to_string(),
        },
    );

    let result = crate::agent::agent_identity::run_with_agent_scope("dola".to_string(), async {
        execute_tool(
            &tool_call,
            &engine,
            "thread-update-custom-memory",
            None,
            &manager,
            None,
            &event_tx,
            &agent_data_dir,
            &engine.http_client,
            None,
        )
        .await
    })
    .await;

    assert!(
        !result.is_error,
        "custom scope update_memory should succeed: {}",
        result.content
    );
    let custom_memory =
        std::fs::read_to_string(custom_paths.memory_path).expect("read custom memory");
    let main_memory = std::fs::read_to_string(main_paths.memory_path).expect("read main memory");
    assert!(custom_memory.contains("- Dola private update"));
    assert!(!main_memory.contains("- Dola private update"));
}

#[tokio::test]
async fn list_threads_tool_returns_filtered_visible_summaries() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);

    let tools = get_available_tools(&AgentConfig::default(), root.path(), false);
    let list_threads = tools
        .iter()
        .find(|tool| tool.function.name == "list_threads")
        .expect("list_threads tool should be available");
    let properties = list_threads
        .function
        .parameters
        .get("properties")
        .and_then(|value| value.as_object())
        .expect("list_threads schema should expose properties");
    for field in [
        "created_after",
        "created_before",
        "updated_after",
        "updated_before",
        "agent_name",
        "title_query",
        "pinned",
        "include_internal",
        "limit",
        "offset",
    ] {
        assert!(
            properties.contains_key(field),
            "list_threads schema should include {field}"
        );
    }

    let mut threads = engine.threads.write().await;
    threads.insert(
        "thread-alpha".to_string(),
        make_thread(
            "thread-alpha",
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Alpha project thread",
            true,
            120,
            350,
            vec![crate::agent::types::AgentMessage::user(
                "operator message",
                120,
            )],
        ),
    );
    threads.insert(
        "thread-beta".to_string(),
        make_thread(
            "thread-beta",
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Beta project thread",
            false,
            121,
            351,
            vec![crate::agent::types::AgentMessage::user("beta message", 121)],
        ),
    );
    threads.insert(
        "thread-hidden".to_string(),
        make_thread(
            "thread-hidden",
            Some(crate::agent::agent_identity::WELES_AGENT_NAME),
            "Alpha internal thread",
            true,
            122,
            352,
            vec![weles_internal_message(122)],
        ),
    );
    drop(threads);

    let tool_call = ToolCall::with_default_weles_review(
        "tool-list-threads".to_string(),
        ToolFunction {
            name: "list_threads".to_string(),
            arguments: serde_json::json!({
                "created_after": 100,
                "created_before": 200,
                "updated_after": 300,
                "updated_before": 400,
                "agent_name": "main-agent",
                "title_query": " alpha ",
                "pinned": true,
                "limit": 10,
                "offset": 0
            })
            .to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        "thread-list-threads",
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
        "list_threads should succeed with valid filters: {}",
        result.content
    );
    assert!(result.pending_approval.is_none());

    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("list_threads should return JSON");
    let rows = payload
        .as_array()
        .expect("list_threads should return an array");
    assert_eq!(
        rows.len(),
        1,
        "only the matching visible thread should be returned"
    );
    assert_eq!(
        rows[0].get("id").and_then(|value| value.as_str()),
        Some("thread-alpha")
    );
    assert_eq!(
        rows[0].get("title").and_then(|value| value.as_str()),
        Some("Alpha project thread")
    );
    assert_eq!(
        rows[0].get("pinned").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        rows[0]
            .get("messages")
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(0),
        "list_threads should return summary rows without message history"
    );
}

#[tokio::test]
async fn read_memory_skips_fresh_injected_base_markdown_by_default() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-fresh";
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

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-fresh".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({}).to_string(),
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
    assert_eq!(
        payload.get("scope").and_then(|value| value.as_str()),
        Some("memory")
    );
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_injected"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_stale"))
            .and_then(|value| value.as_bool()),
        Some(false)
    );
    assert!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .is_none(),
        "fresh injected base markdown should be skipped by default"
    );
    assert!(
        payload
            .get("layers_skipped")
            .and_then(|value| value.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("layer").and_then(|value| value.as_str()) == Some("base_markdown")
                    && item.get("reason").and_then(|value| value.as_str())
                        == Some("already_injected_fresh")
            })),
        "base markdown should be marked as skipped due to fresh injection"
    );
    assert!(
        payload
            .get("results")
            .and_then(|value| value.get("operator_profile_json"))
            .is_some(),
        "read_memory should still return non-markdown layers"
    );
}

#[tokio::test]
async fn read_memory_includes_structural_graph_lookup_results() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-structural-graph";
    let agent_data_dir = root.path().join("agent");

    engine
        .history
        .create_thread(&zorai_protocol::AgentDbThread {
            id: thread_id.to_string(),
            workspace_id: None,
            surface_id: None,
            pane_id: None,
            agent_name: Some(crate::agent::agent_identity::MAIN_AGENT_NAME.to_string()),
            title: "Structural graph lookup thread".to_string(),
            created_at: 1,
            updated_at: 1,
            message_count: 0,
            total_tokens: 0,
            last_preview: String::new(),
            metadata_json: None,
        })
        .await
        .expect("seed thread row");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Structural graph lookup thread",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "show structural graph",
                1,
            )],
        ),
    );

    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            workspace_seed_scan_complete: true,
            language_hints: vec!["rust".to_string()],
            workspace_seeds: vec![crate::agent::context::structural_memory::WorkspaceSeed {
                node_id: "node:file:Cargo.toml".to_string(),
                relative_path: "Cargo.toml".to_string(),
                kind: "cargo_manifest".to_string(),
            }],
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "node:file:src/lib.rs".to_string(),
                relative_path: "src/lib.rs".to_string(),
            }],
            edges: vec![crate::agent::context::structural_memory::StructuralEdge {
                from: "node:file:src/lib.rs".to_string(),
                to: "node:package:cargo:demo".to_string(),
                kind: "file_in_package".to_string(),
            }],
        },
    );

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-structural-graph".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "include_thread_structural_memory": true,
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
    let graph_lookup = payload
        .get("results")
        .and_then(|value| value.get("thread_structural_memory"))
        .and_then(|value| value.get("graph_lookup"))
        .and_then(|value| value.as_array())
        .expect("thread structural memory should expose graph_lookup");
    assert!(!graph_lookup.is_empty(), "graph_lookup should not be empty");
    assert!(graph_lookup.iter().any(|item| {
        item.get("node_id").and_then(|value| value.as_str()) == Some("node:package:cargo:demo")
            && item.get("relation_kind").and_then(|value| value.as_str()) == Some("file_in_package")
    }));
}

#[tokio::test]
async fn mcp_read_memory_uses_explicit_thread_id_for_injection_state() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");
    let thread_id = "thread-mcp-explicit";

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- MCP explicit thread fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;

    let content = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({ "thread_id": thread_id }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect("mcp read_memory should succeed");

    let payload: serde_json::Value =
        serde_json::from_str(&content).expect("mcp read_memory should return JSON");
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_injected"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .is_none(),
        "explicit thread_id should reuse that thread's injection state"
    );
}

#[tokio::test]
async fn mcp_read_memory_without_thread_id_stays_thread_agnostic() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- MCP fallback fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            "thread-mcp-existing",
            build_matching_injection_state(&memory, &paths),
        )
        .await;

    let content = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({}),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect("mcp read_memory should succeed without thread_id");

    let payload: serde_json::Value =
        serde_json::from_str(&content).expect("mcp read_memory should return JSON");
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_injected"))
            .and_then(|value| value.as_bool()),
        Some(false),
        "omitting thread_id should not borrow a random tracked thread"
    );
    assert_eq!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str()),
        Some("# Memory\n\n- MCP fallback fact\n")
    );
}

#[tokio::test]
async fn mcp_read_memory_uses_thread_scope_for_non_main_threads() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");
    let thread_id = "thread-mcp-domowoj";

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Main soul fact\n",
        "# Memory\n\n- Main scope fact\n",
        "# User\n\n- Shared user fact\n",
    );
    let domowoj_paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::DOMOWOJ_AGENT_ID,
        "# Soul\n\n- Domowoj soul fact\n",
        "# Memory\n\n- Domowoj scope fact\n",
        "# User\n\n- Shared user fact\n",
    );
    let domowoj_memory = crate::agent::memory::load_memory_for_scope(
        &agent_data_dir,
        crate::agent::agent_identity::DOMOWOJ_AGENT_ID,
    )
    .await
    .expect("load domowoj memory");
    engine
        .set_thread_handoff_state(
            thread_id,
            crate::agent::ThreadHandoffState {
                origin_agent_id: crate::agent::agent_identity::MAIN_AGENT_ID.to_string(),
                active_agent_id: crate::agent::agent_identity::DOMOWOJ_AGENT_ID.to_string(),
                responder_stack: vec![crate::agent::ThreadResponderFrame {
                    agent_id: crate::agent::agent_identity::DOMOWOJ_AGENT_ID.to_string(),
                    agent_name: crate::agent::agent_identity::DOMOWOJ_AGENT_NAME.to_string(),
                    entered_at: 1,
                    entered_via_handoff_event_id: None,
                    linked_thread_id: None,
                }],
                events: Vec::new(),
                pending_approval_id: None,
            },
        )
        .await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            crate::agent::memory_context::build_prompt_memory_injection_state(
                &crate::agent::memory_context::build_structured_memory_summary(
                    &domowoj_memory,
                    &domowoj_paths,
                    None,
                    None,
                ),
                false,
            ),
        )
        .await;

    let content = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({
            "thread_id": thread_id,
            "include_already_injected": true
        }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect("mcp read_memory should succeed for non-main thread");

    let payload: serde_json::Value =
        serde_json::from_str(&content).expect("mcp read_memory should return JSON");
    assert_eq!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str()),
        Some("# Memory\n\n- Domowoj scope fact\n"),
        "explicit thread_id should align memory scope with the thread's active agent"
    );
}

#[tokio::test]
async fn mcp_read_memory_rejects_unknown_thread_id() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");

    let error = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({ "thread_id": "thread-does-not-exist" }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect_err("unknown thread_id should be rejected");
    assert!(
        error.to_string().contains("unknown thread_id"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn mcp_read_memory_rejects_non_string_thread_id() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");

    let error = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({ "thread_id": 7 }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect_err("non-string thread_id should be rejected");
    assert!(
        error.to_string().contains("'thread_id' must be a string"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn mcp_read_memory_rejects_unknown_task_id() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");

    let error = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({ "task_id": "task-does-not-exist" }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect_err("unknown task_id should be rejected");
    assert!(
        error.to_string().contains("unknown task_id"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn mcp_read_memory_rejects_non_string_task_id() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");

    let error = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({ "task_id": 7 }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect_err("non-string task_id should be rejected");
    assert!(
        error.to_string().contains("'task_id' must be a string"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn mcp_read_memory_rejects_non_object_args() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");

    let error = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!(null),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect_err("non-object args should be rejected");
    assert!(
        error
            .to_string()
            .contains("memory tool arguments must be a JSON object"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn mcp_read_memory_treats_blank_selectors_as_absent() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Blank selector fact\n",
        "# User\n\n- Stable user fact\n",
    );

    let content = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({ "thread_id": "   ", "task_id": "" }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect("blank selectors should be treated as absent");

    let payload: serde_json::Value =
        serde_json::from_str(&content).expect("mcp read_memory should return JSON");
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_injected"))
            .and_then(|value| value.as_bool()),
        Some(false)
    );
}

#[tokio::test]
async fn mcp_read_memory_task_id_overrides_thread_context_entirely() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;
    let agent_data_dir = root.path().join("agent");
    let thread_id = "thread-mcp-task-overrides";
    let task_id = "task-domowoj-scope";

    let main_paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Main soul fact\n",
        "# Memory\n\n- Main thread fact\n",
        "# User\n\n- Shared user fact\n",
    );
    let main_memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&main_memory, &main_paths),
        )
        .await;
    engine.tasks.lock().await.push_back(sample_task_with_scope(
        task_id,
        Some(thread_id),
        crate::agent::agent_identity::DOMOWOJ_AGENT_ID,
    ));
    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::DOMOWOJ_AGENT_ID,
        "# Soul\n\n- Domowoj soul fact\n",
        "# Memory\n\n- Task scope fact\n",
        "# User\n\n- Shared user fact\n",
    );

    let content = execute_memory_tool_for_mcp(
        "read_memory",
        &serde_json::json!({
            "thread_id": thread_id,
            "task_id": task_id,
            "include_already_injected": true
        }),
        &engine,
        &agent_data_dir,
    )
    .await
    .expect("task_id should override thread context");

    let payload: serde_json::Value =
        serde_json::from_str(&content).expect("mcp read_memory should return JSON");
    assert_eq!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str()),
        Some("# Memory\n\n- Task scope fact\n")
    );
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_injected"))
            .and_then(|value| value.as_bool()),
        Some(false),
        "task_id override should ignore thread-scoped injection state"
    );
}

#[tokio::test]
async fn read_memory_rejects_non_object_args_in_direct_tool_path() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let agent_data_dir = root.path().join("agent");

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Direct path fact\n",
        "# User\n\n- Stable user fact\n",
    );

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-null-args".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: "null".to_string(),
        },
    );

    let result = execute_tool(
        &tool_call,
        &engine,
        "thread-read-memory-null-args",
        None,
        &manager,
        None,
        &event_tx,
        &agent_data_dir,
        &engine.http_client,
        None,
    )
    .await;

    assert!(result.is_error, "non-object args should be rejected");
    assert!(
        result
            .content
            .contains("memory tool arguments must be a JSON object"),
        "unexpected error: {}",
        result.content
    );
}

#[tokio::test]
async fn read_memory_skip_uses_memory_layer_freshness_not_combined_bootstrap_hash() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-layer-freshness";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let initial_memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&initial_memory, &paths),
        )
        .await;
    std::fs::write(&paths.user_path, "# User\n\n- Updated user fact\n").expect("rewrite user");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-layer-freshness".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({}).to_string(),
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
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_stale"))
            .and_then(|value| value.as_bool()),
        Some(false),
        "read_memory staleness should be scoped to MEMORY.md rather than the combined bootstrap hash"
    );
    assert!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .is_none(),
        "read_memory should still skip MEMORY.md when only USER.md changed"
    );
}

#[tokio::test]
async fn read_memory_can_force_return_of_already_injected_base_markdown() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-force";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "# Memory\n\n- Forced include memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-force".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "include_already_injected": true
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
    assert_eq!(
        payload.get("scope").and_then(|value| value.as_str()),
        Some("memory")
    );
    assert_eq!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str()),
        Some("# Memory\n\n- Forced include memory fact\n")
    );
    assert!(
        payload
            .get("layers_consulted")
            .and_then(|value| value.as_array())
            .is_some_and(|items| items
                .iter()
                .any(|item| item.as_str() == Some("base_markdown"))),
        "forced include should consult base markdown even when already injected"
    );
}

#[tokio::test]
async fn read_memory_skips_blank_base_markdown_when_injected_copy_is_fresh() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-blank";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Stable soul fact\n",
        "",
        "# User\n\n- Stable user fact\n",
    );
    let memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&memory, &paths),
        )
        .await;

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-blank".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({}).to_string(),
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
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_stale"))
            .and_then(|value| value.as_bool()),
        Some(false),
        "blank MEMORY.md should still be recognized as already injected and fresh"
    );
    assert!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .is_none(),
        "blank but fresh MEMORY.md should still be skipped by default"
    );
}

#[tokio::test]
async fn read_soul_returns_base_markdown_when_injected_copy_is_stale() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-soul-stale";
    let agent_data_dir = root.path().join("agent");

    let paths = write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Original soul fact\n",
        "# Memory\n\n- Stable memory fact\n",
        "# User\n\n- Stable user fact\n",
    );
    let initial_memory = current_scope_memory(&agent_data_dir).await;
    engine
        .set_thread_memory_injection_state(
            thread_id,
            build_matching_injection_state(&initial_memory, &paths),
        )
        .await;
    std::fs::write(&paths.soul_path, "# Soul\n\n- Updated soul fact\n").expect("rewrite soul");

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-soul-stale".to_string(),
        ToolFunction {
            name: "read_soul".to_string(),
            arguments: serde_json::json!({}).to_string(),
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
        "read_soul should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("read_soul should return JSON");
    assert_eq!(
        payload.get("scope").and_then(|value| value.as_str()),
        Some("soul")
    );
    assert_eq!(
        payload
            .get("injection_state")
            .and_then(|value| value.get("base_layer_stale"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        payload
            .get("results")
            .and_then(|value| value.get("base_markdown"))
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str()),
        Some("# Soul\n\n- Updated soul fact\n")
    );
}

#[tokio::test]
async fn read_memory_marks_truncated_when_language_hints_overflow_limit() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-language-hints";
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
            language_hints: vec!["rust".to_string(), "typescript".to_string()],
            ..Default::default()
        },
    );

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-language-hints".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 1
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
    assert_eq!(
        payload.get("truncated").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        payload
            .get("results")
            .and_then(|value| value.get("thread_structural_memory"))
            .and_then(|value| value.get("language_hints"))
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(1),
        "language hints should still be limited by limit_per_layer"
    );
}

#[tokio::test]
async fn read_memory_marks_truncated_when_structural_entries_overflow_across_sources() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-read-memory-structural-overflow";
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
                    node_id: "file:a".to_string(),
                    relative_path: "a.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "file:b".to_string(),
                    relative_path: "b.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "file:c".to_string(),
                    relative_path: "c.rs".to_string(),
                },
                crate::agent::context::structural_memory::ObservedFileNode {
                    node_id: "file:d".to_string(),
                    relative_path: "d.rs".to_string(),
                },
            ],
            workspace_seeds: vec![
                crate::agent::context::structural_memory::WorkspaceSeed {
                    node_id: "seed:e".to_string(),
                    relative_path: "e.toml".to_string(),
                    kind: "manifest".to_string(),
                },
                crate::agent::context::structural_memory::WorkspaceSeed {
                    node_id: "seed:f".to_string(),
                    relative_path: "f.toml".to_string(),
                    kind: "manifest".to_string(),
                },
                crate::agent::context::structural_memory::WorkspaceSeed {
                    node_id: "seed:g".to_string(),
                    relative_path: "g.toml".to_string(),
                    kind: "manifest".to_string(),
                },
                crate::agent::context::structural_memory::WorkspaceSeed {
                    node_id: "seed:h".to_string(),
                    relative_path: "h.toml".to_string(),
                    kind: "manifest".to_string(),
                },
            ],
            ..Default::default()
        },
    );

    let tool_call = ToolCall::with_default_weles_review(
        "tool-read-memory-structural-overflow".to_string(),
        ToolFunction {
            name: "read_memory".to_string(),
            arguments: serde_json::json!({
                "limit_per_layer": 5
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
    assert_eq!(
        payload.get("truncated").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        payload
            .get("results")
            .and_then(|value| value.get("thread_structural_memory"))
            .and_then(|value| value.get("entries"))
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(5),
        "entries should still respect the requested limit"
    );
}

#[tokio::test]
async fn search_memory_tools_are_exposed() {
    let root = tempdir().expect("tempdir");

    let tools = get_available_tools(&AgentConfig::default(), root.path(), false);
    for tool_name in ["search_memory", "search_user", "search_soul"] {
        let tool = tools
            .iter()
            .find(|tool| tool.function.name == tool_name)
            .unwrap_or_else(|| panic!("{tool_name} should be available"));
        let properties = tool
            .function
            .parameters
            .get("properties")
            .and_then(|value| value.as_object())
            .expect("search tool schema should expose properties");
        for field in [
            "query",
            "limit",
            "include_already_injected",
            "include_base_markdown",
            "include_operator_profile_json",
            "include_operator_model_summary",
            "include_thread_structural_memory",
        ] {
            assert!(
                properties.contains_key(field),
                "{tool_name} schema should include {field}"
            );
        }
        assert!(
            tool.function
                .parameters
                .get("required")
                .and_then(|value| value.as_array())
                .is_some_and(|items| items.iter().any(|item| item.as_str() == Some("query"))),
            "{tool_name} should require query"
        );
    }
}

#[tokio::test]
async fn search_user_honors_layer_toggles() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _) = broadcast::channel(8);
    let thread_id = "thread-search-user-layer-toggles";
    let agent_data_dir = root.path().join("agent");
    engine.threads.write().await.insert(
        thread_id.to_string(),
        make_thread(
            thread_id,
            Some(crate::agent::agent_identity::MAIN_AGENT_NAME),
            "Search user layer toggles",
            false,
            1,
            1,
            vec![crate::agent::types::AgentMessage::user(
                "search user layers",
                1,
            )],
        ),
    );

    write_scope_memory_files(
        &agent_data_dir,
        crate::agent::agent_identity::MAIN_AGENT_ID,
        "# Soul\n\n- Soul material without the query\n",
        "# Memory\n\n- Memory material without the query\n",
        "# User\n\n- toggle-query base line\n",
    );
    engine.thread_structural_memories.write().await.insert(
        thread_id.to_string(),
        crate::agent::context::structural_memory::ThreadStructuralMemory {
            observed_files: vec![crate::agent::context::structural_memory::ObservedFileNode {
                node_id: "file:toggle".to_string(),
                relative_path: "toggle-query.rs".to_string(),
            }],
            ..Default::default()
        },
    );

    let tool_call = ToolCall::with_default_weles_review(
        "tool-search-user-layer-toggles".to_string(),
        ToolFunction {
            name: "search_user".to_string(),
            arguments: serde_json::json!({
                "query": "toggle-query",
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
        "search_user should succeed: {}",
        result.content
    );
    let payload: serde_json::Value =
        serde_json::from_str(&result.content).expect("search_user should return JSON");
    assert_eq!(
        payload.get("scope").and_then(|value| value.as_str()),
        Some("user")
    );
    assert_eq!(
        payload
            .get("layers_consulted")
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .collect::<Vec<_>>()
            }),
        Some(vec!["thread_structural_memory"])
    );
    assert!(
        payload
            .get("layers_skipped")
            .and_then(|value| value.as_array())
            .is_some_and(|items| items.iter().any(|item| {
                item.get("layer").and_then(|value| value.as_str()) == Some("base_markdown")
                    && item.get("reason").and_then(|value| value.as_str()) == Some("disabled")
            })),
        "disabled base markdown should be recorded as skipped"
    );
    let matches = payload
        .get("matches")
        .and_then(|value| value.as_array())
        .expect("search_user should return matches");
    assert_eq!(
        matches.len(),
        1,
        "only the enabled layer should contribute matches"
    );
    assert_eq!(
        matches[0].get("layer").and_then(|value| value.as_str()),
        Some("thread_structural_memory")
    );
    assert!(
        matches[0]
            .get("score")
            .and_then(|value| value.as_u64())
            .is_some(),
        "search matches should expose a deterministic score"
    );
    assert!(
        matches[0]
            .get("freshness")
            .and_then(|value| value.get("status"))
            .and_then(|value| value.as_str())
            .is_some(),
        "search matches should expose freshness metadata"
    );
}
