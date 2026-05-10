use super::super::{
    adapted_timeout_override_for_mode, daemon_tool_timeout_seconds, default_timeout_seconds_for_tool, execute_create_file,
    execute_get_git_line_statuses,
    execute_onecontext_search_with_runner, execute_read_file, execute_tool,
    get_available_tools, get_file_path_arg,
};
use crate::agent::{
    types::{AgentConfig, AgentEvent, ToolCall, ToolFunction},
    AgentEngine,
};
use crate::session_manager::SessionManager;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use tokio::sync::broadcast;
use tokio::time::{timeout, Duration};

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

pub(super) fn successful_exit_status() -> std::process::ExitStatus {
    exit_status_with_code(0)
}

pub(super) fn exit_status_with_code(code: i32) -> std::process::ExitStatus {
    #[cfg(unix)]
    {
        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(windows)]
    {
        std::process::ExitStatus::from_raw(code as u32)
    }
}

#[test]
fn daemon_tool_timeout_uses_300_seconds_for_onecontext_search() {
    assert_eq!(default_timeout_seconds_for_tool("onecontext_search"), 300);
    assert_eq!(
        daemon_tool_timeout_seconds("onecontext_search", &serde_json::json!({})),
        300
    );
}

#[test]
fn daemon_tool_timeout_uses_300_seconds_for_fetch_url() {
    assert_eq!(default_timeout_seconds_for_tool("fetch_url"), 300);
    assert_eq!(
        daemon_tool_timeout_seconds("fetch_url", &serde_json::json!({})),
        300
    );
}

#[test]
fn daemon_tool_timeout_uses_300_seconds_for_web_search() {
    assert_eq!(default_timeout_seconds_for_tool("web_search"), 300);
    assert_eq!(
        daemon_tool_timeout_seconds("web_search", &serde_json::json!({})),
        300
    );
}

#[test]
fn daemon_tool_timeout_uses_600_seconds_for_heavy_media_tools() {
    for tool_name in [
        "analyze_image",
        "generate_image",
        "speech_to_text",
        "text_to_speech",
    ] {
        assert_eq!(
            default_timeout_seconds_for_tool(tool_name),
            600,
            "{tool_name} should default to a long timeout"
        );
        assert_eq!(
            daemon_tool_timeout_seconds(tool_name, &serde_json::json!({})),
            600,
            "{tool_name} should inherit the long timeout"
        );
    }
}

#[test]
fn daemon_tool_timeout_clamps_explicit_override_to_600_seconds() {
    assert_eq!(
        daemon_tool_timeout_seconds(
            "onecontext_search",
            &serde_json::json!({ "timeout_seconds": 999 })
        ),
        600
    );
}

#[test]
fn adapted_timeout_override_tightens_defaults_without_overriding_explicit_value() {
    assert_eq!(
        adapted_timeout_override_for_mode(
            "fetch_url",
            &serde_json::json!({}),
            crate::agent::operator_model::SatisfactionAdaptationMode::Minimal,
        ),
        Some(180)
    );
    assert_eq!(
        adapted_timeout_override_for_mode(
            "fetch_url",
            &serde_json::json!({ "timeout_seconds": 42 }),
            crate::agent::operator_model::SatisfactionAdaptationMode::Minimal,
        ),
        None
    );
}

#[test]
fn onecontext_search_tool_schema_exposes_timeout_seconds() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let onecontext = tools
        .iter()
        .find(|tool| tool.function.name == "onecontext_search")
        .expect("onecontext_search tool should be available");

    let timeout_schema = onecontext
        .function
        .parameters
        .get("properties")
        .and_then(|properties| properties.get("timeout_seconds"))
        .expect("onecontext_search schema should expose timeout_seconds");

    assert_eq!(
        timeout_schema.get("type").and_then(|value| value.as_str()),
        Some("integer")
    );
    assert_eq!(
        timeout_schema
            .get("minimum")
            .and_then(|value| value.as_u64()),
        Some(0)
    );
    assert_eq!(
        timeout_schema
            .get("maximum")
            .and_then(|value| value.as_u64()),
        Some(600)
    );
    assert!(timeout_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 300") && value.contains("max: 600")));
}

#[test]
fn read_skill_tool_schema_avoids_top_level_combinators() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let read_skill = tools
        .iter()
        .find(|tool| tool.function.name == "read_skill")
        .expect("read_skill tool should be available");

    assert_eq!(
        read_skill
            .function
            .parameters
            .get("type")
            .and_then(|value| value.as_str()),
        Some("object")
    );
    for forbidden in ["oneOf", "anyOf", "allOf", "enum", "not"] {
        assert!(
            !read_skill.function.parameters.get(forbidden).is_some(),
            "read_skill schema must not expose top-level {forbidden}"
        );
    }
    assert!(read_skill
        .function
        .parameters
        .get("properties")
        .and_then(|properties| properties.get("skills"))
        .is_some());
}

#[test]
fn web_search_tool_schema_exposes_timeout_seconds() {
    let mut config = AgentConfig::default();
    config.tools.web_search = true;
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let web_search = tools
        .iter()
        .find(|tool| tool.function.name == "web_search")
        .expect("web_search tool should be available");

    let timeout_schema = web_search
        .function
        .parameters
        .get("properties")
        .and_then(|properties| properties.get("timeout_seconds"))
        .expect("web_search schema should expose timeout_seconds");

    assert_eq!(
        timeout_schema.get("type").and_then(|value| value.as_str()),
        Some("integer")
    );
    assert_eq!(
        timeout_schema
            .get("minimum")
            .and_then(|value| value.as_u64()),
        Some(0)
    );
    assert_eq!(
        timeout_schema
            .get("maximum")
            .and_then(|value| value.as_u64()),
        Some(600)
    );
    assert!(timeout_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 300") && value.contains("max: 600")));
}

#[test]
fn fetch_url_tool_schema_exposes_timeout_seconds() {
    let mut config = AgentConfig::default();
    config.tools.web_browse = true;
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let fetch_url = tools
        .iter()
        .find(|tool| tool.function.name == "fetch_url")
        .expect("fetch_url tool should be available");

    let timeout_schema = fetch_url
        .function
        .parameters
        .get("properties")
        .and_then(|properties| properties.get("timeout_seconds"))
        .expect("fetch_url schema should expose timeout_seconds");

    assert_eq!(
        timeout_schema.get("type").and_then(|value| value.as_str()),
        Some("integer")
    );
    assert_eq!(
        timeout_schema
            .get("minimum")
            .and_then(|value| value.as_u64()),
        Some(0)
    );
    assert_eq!(
        timeout_schema
            .get("maximum")
            .and_then(|value| value.as_u64()),
        Some(600)
    );
    assert!(timeout_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 300") && value.contains("max: 600")));
}

#[test]
fn web_and_terminal_tool_descriptions_distinguish_text_reads_from_binary_downloads() {
    let mut config = AgentConfig::default();
    config.tools.web_browse = true;
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);

    let fetch_url = tools
        .iter()
        .find(|tool| tool.function.name == "fetch_url")
        .expect("fetch_url tool should be available");
    let run_terminal_command = tools
        .iter()
        .find(|tool| tool.function.name == "run_terminal_command")
        .expect("run_terminal_command tool should be available");
    let execute_managed_command = tools
        .iter()
        .find(|tool| tool.function.name == "execute_managed_command")
        .expect("execute_managed_command tool should be available");

    assert!(fetch_url.function.description.contains("text content"));
    assert!(run_terminal_command
        .function
        .description
        .contains("large or binary downloads"));
    assert!(execute_managed_command
        .function
        .description
        .contains("large or binary downloads"));
}

#[test]
fn search_files_tool_schema_exposes_timeout_seconds() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let search_files = tools
        .iter()
        .find(|tool| tool.function.name == "search_files")
        .expect("search_files tool should be available");

    let timeout_schema = search_files
        .function
        .parameters
        .get("properties")
        .and_then(|properties| properties.get("timeout_seconds"))
        .expect("search_files schema should expose timeout_seconds");

    assert_eq!(
        timeout_schema.get("type").and_then(|value| value.as_str()),
        Some("integer")
    );
    assert_eq!(
        timeout_schema
            .get("minimum")
            .and_then(|value| value.as_u64()),
        Some(0)
    );
    assert_eq!(
        timeout_schema
            .get("maximum")
            .and_then(|value| value.as_u64()),
        Some(600)
    );
    assert!(timeout_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 120") && value.contains("max: 600")));
}

#[test]
fn summary_alias_tool_is_exposed() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);

    assert!(tools.iter().any(|tool| tool.function.name == "summary"));
}

#[test]
fn read_file_tool_schema_exposes_offset_and_limit_defaults() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let read_file = tools
        .iter()
        .find(|tool| tool.function.name == "read_file")
        .expect("read_file tool should be available");

    let properties = read_file
        .function
        .parameters
        .get("properties")
        .expect("read_file schema should expose properties");

    let offset_schema = properties
        .get("offset")
        .expect("read_file schema should expose offset");
    assert_eq!(
        offset_schema.get("type").and_then(|value| value.as_str()),
        Some("integer")
    );
    assert!(offset_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 0")));

    let limit_schema = properties
        .get("limit")
        .expect("read_file schema should expose limit");
    assert_eq!(
        limit_schema.get("type").and_then(|value| value.as_str()),
        Some("integer")
    );
    assert!(limit_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 250")));
}

#[tokio::test]
async fn read_file_uses_default_offset_zero_and_limit_250() {
    let root = tempdir().expect("tempdir");
    let file_path = root.path().join("sample.txt");
    let body = (0..300)
        .map(|index| format!("line-{index:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    tokio::fs::write(&file_path, body)
        .await
        .expect("write sample file");

    let result = execute_read_file(&serde_json::json!({
        "path": file_path,
    }))
    .await
    .expect("read file should succeed");

    assert!(result.starts_with("line-000\nline-001"));
    assert!(result.contains("line-249"));
    assert!(!result.contains("line-250\n"));
    assert!(result.contains("truncated, showing 250 of 300 lines"));
}

#[tokio::test]
async fn read_file_honors_offset_and_limit_window() {
    let root = tempdir().expect("tempdir");
    let file_path = root.path().join("sample.txt");
    let body = (0..20)
        .map(|index| format!("line-{index:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    tokio::fs::write(&file_path, body)
        .await
        .expect("write sample file");

    let result = execute_read_file(&serde_json::json!({
        "path": file_path,
        "offset": 5,
        "limit": 3,
    }))
    .await
    .expect("read file should succeed");

    assert_eq!(result, "line-005\nline-006\nline-007");
}

#[test]
fn get_file_path_arg_skips_blank_path_and_falls_back_to_filename() {
    let args = serde_json::json!({
        "path": "   ",
        "filename": "notes.md"
    });

    assert_eq!(get_file_path_arg(&args), Some("notes.md"));
}

#[tokio::test]
async fn create_file_uses_filename_and_cwd_when_path_is_blank() {
    let root = tempdir().expect("tempdir");
    let file_path = root.path().join("notes.md");

    let result = execute_create_file(&serde_json::json!({
        "path": "",
        "filename": "notes.md",
        "cwd": root.path(),
        "content": "hello world"
    }))
    .await
    .expect("create_file should fall back to filename when path is blank");

    assert!(result.contains(&file_path.display().to_string()));
    assert_eq!(
        tokio::fs::read_to_string(&file_path)
            .await
            .expect("read created file"),
        "hello world"
    );
}

#[tokio::test]
async fn create_file_accepts_filename_when_it_is_already_a_path() {
    let root = tempdir().expect("tempdir");
    let file_path = root.path().join("nested").join("notes.md");

    let result = execute_create_file(&serde_json::json!({
        "path": "",
        "filename": file_path,
        "cwd": "/tmp/should-not-matter",
        "content": "hello world"
    }))
    .await
    .expect("create_file should accept a full path in filename");

    assert!(result.contains(&file_path.display().to_string()));
    assert_eq!(
        tokio::fs::read_to_string(&file_path)
            .await
            .expect("read created file"),
        "hello world"
    );
}

#[test]
fn get_git_line_statuses_tool_schema_exposes_line_window_arguments() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let line_statuses = tools
        .iter()
        .find(|tool| tool.function.name == "get_git_line_statuses")
        .expect("get_git_line_statuses tool should be available");

    let properties = line_statuses
        .function
        .parameters
        .get("properties")
        .expect("get_git_line_statuses schema should expose properties");

    let start_line_schema = properties
        .get("start_line")
        .expect("get_git_line_statuses schema should expose start_line");
    assert_eq!(
        start_line_schema
            .get("type")
            .and_then(|value| value.as_str()),
        Some("integer")
    );
    assert!(start_line_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 1")));

    let limit_schema = properties
        .get("limit")
        .expect("get_git_line_statuses schema should expose limit");
    assert_eq!(
        limit_schema.get("type").and_then(|value| value.as_str()),
        Some("integer")
    );
    assert!(limit_schema
        .get("description")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.contains("default: 250")));
}

#[tokio::test]
async fn get_git_line_statuses_reports_modified_and_added_current_lines() {
    let root = tempdir().expect("tempdir");
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root.path())
        .output()
        .expect("git init should succeed");
    std::process::Command::new("git")
        .args(["config", "user.name", "zorai tests"])
        .current_dir(root.path())
        .output()
        .expect("git config user.name should succeed");
    std::process::Command::new("git")
        .args(["config", "user.email", "zorai@example.com"])
        .current_dir(root.path())
        .output()
        .expect("git config user.email should succeed");

    let file_path = root.path().join("sample.txt");
    tokio::fs::write(&file_path, "alpha\nbeta\ngamma\n")
        .await
        .expect("write initial file");
    std::process::Command::new("git")
        .args(["add", "sample.txt"])
        .current_dir(root.path())
        .output()
        .expect("git add should succeed");
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(root.path())
        .output()
        .expect("git commit should succeed");

    tokio::fs::write(&file_path, "alpha\nbeta-updated\ngamma\ndelta\n")
        .await
        .expect("write updated file");

    let result = execute_get_git_line_statuses(&serde_json::json!({
        "path": file_path,
        "start_line": 1,
        "limit": 4,
    }))
    .await
    .expect("get_git_line_statuses should succeed");

    let payload: serde_json::Value =
        serde_json::from_str(&result).expect("tool result should be valid json");
    let statuses = payload
        .get("statuses")
        .and_then(|value| value.as_array())
        .expect("tool result should include statuses");
    let compact = statuses
        .iter()
        .map(|entry| {
            (
                entry
                    .get("line")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0),
                entry
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or(""),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        compact,
        vec![
            (1, "unchanged"),
            (2, "modified"),
            (3, "unchanged"),
            (4, "added")
        ]
    );
}

#[tokio::test]
async fn summary_alias_dispatches_to_semantic_query_summary_kind() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, mut event_rx) = broadcast::channel(8);
    let http_client = reqwest::Client::new();

    let alias_args = serde_json::json!({
        "path": root.path(),
        "limit": 5
    });
    let semantic_args = serde_json::json!({
        "kind": "summary",
        "path": root.path(),
        "limit": 5
    });

    let alias_result = execute_tool(
        &ToolCall {
            id: "call-summary".to_string(),
            function: ToolFunction {
                name: "summary".to_string(),
                arguments: alias_args.to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-summary",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &http_client,
        None,
    )
    .await;

    let semantic_result = execute_tool(
        &ToolCall {
            id: "call-semantic-summary".to_string(),
            function: ToolFunction {
                name: "semantic_query".to_string(),
                arguments: semantic_args.to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-semantic",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &http_client,
        None,
    )
    .await;

    assert!(
        !alias_result.is_error,
        "summary alias should succeed: {}",
        alias_result.content
    );
    assert!(
        !semantic_result.is_error,
        "semantic_query summary should succeed: {}",
        semantic_result.content
    );
    assert_eq!(alias_result.content, semantic_result.content);

    let workflow_notice = timeout(Duration::from_millis(250), event_rx.recv())
        .await
        .expect("summary alias should emit workflow notice")
        .expect("workflow notice should be received");
    match workflow_notice {
        AgentEvent::WorkflowNotice {
            kind,
            details: Some(details),
            ..
        } => {
            assert_eq!(kind, "semantic-query");
            let details: serde_json::Value =
                serde_json::from_str(&details).expect("workflow notice details should be json");
            assert_eq!(
                details.get("kind").and_then(|value| value.as_str()),
                Some("summary")
            );
            assert_eq!(
                details.get("limit").and_then(|value| value.as_u64()),
                Some(5)
            );
            assert!(details
                .get("path")
                .and_then(|value| value.as_str())
                .is_some_and(|value| value == root.path().to_string_lossy()));
        }
        other => panic!("expected workflow notice, got {other:?}"),
    }
}

#[tokio::test]
async fn get_current_datetime_dispatch_returns_local_and_utc_timestamps() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager.clone(), AgentConfig::default(), root.path()).await;
    let (event_tx, _event_rx) = broadcast::channel(8);
    let http_client = reqwest::Client::new();

    let result = execute_tool(
        &ToolCall {
            id: "call-current-datetime".to_string(),
            function: ToolFunction {
                name: "get_current_datetime".to_string(),
                arguments: serde_json::json!({}).to_string(),
            },
            weles_review: None,
        },
        &engine,
        "thread-current-datetime",
        None,
        &manager,
        None,
        &event_tx,
        root.path(),
        &http_client,
        None,
    )
    .await;

    assert!(
        !result.is_error,
        "get_current_datetime should succeed: {}",
        result.content
    );
    assert!(result.content.contains("Current datetime:"));
    assert!(result.content.contains("Local:"));
    assert!(result.content.contains("UTC:"));
    assert!(result.content.contains("Unix timestamp (ms):"));
}

#[tokio::test]
async fn onecontext_search_runtime_uses_default_timeout_on_caller_path() {
    let observed_timeout = Arc::new(Mutex::new(None));
    let observed_timeout_clone = observed_timeout.clone();

    let result = execute_onecontext_search_with_runner(
        &serde_json::json!({ "query": "timeout policy" }),
        true,
        move |request| {
            let observed_timeout = observed_timeout_clone.clone();
            async move {
                *observed_timeout
                    .lock()
                    .expect("timeout lock should succeed") = Some(request.timeout_seconds);
                Ok::<std::process::Output, anyhow::Error>(std::process::Output {
                    status: successful_exit_status(),
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                })
            }
        },
    )
    .await
    .expect("onecontext search should succeed");

    assert_eq!(
        *observed_timeout
            .lock()
            .expect("timeout lock should succeed"),
        Some(300)
    );
    assert!(result.contains("No OneContext matches for \"timeout policy\""));
}

#[test]
fn fetch_url_tool_schema_exposes_profile_id() {
    let mut config = AgentConfig::default();
    config.tools.web_browse = true;
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let fetch_url = tools
        .iter()
        .find(|tool| tool.function.name == "fetch_url")
        .expect("fetch_url tool should be available");

    let profile_schema = fetch_url
        .function
        .parameters
        .get("properties")
        .and_then(|properties| properties.get("profile_id"))
        .expect("fetch_url schema should expose profile_id");

    assert_eq!(
        profile_schema.get("type").and_then(|value| value.as_str()),
        Some("string")
    );
}

#[test]
fn setup_web_browsing_schema_does_not_expose_dead_profile_id() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let setup_web_browsing = tools
        .iter()
        .find(|tool| tool.function.name == "setup_web_browsing")
        .expect("setup_web_browsing tool should be available");

    let properties = setup_web_browsing
        .function
        .parameters
        .get("properties")
        .expect("setup_web_browsing schema should expose properties");

    assert!(
        properties.get("profile_id").is_none(),
        "setup_web_browsing should not advertise unsupported profile_id"
    );
}

#[test]
fn list_trigger_fire_history_tool_schema_matches_runtime_statuses() {
    let config = AgentConfig::default();
    let temp_dir = std::env::temp_dir();
    let tools = get_available_tools(&config, &temp_dir, false);
    let trigger_history = tools
        .iter()
        .find(|tool| tool.function.name == "list_trigger_fire_history")
        .expect("list_trigger_fire_history tool should be available");

    let status_schema = trigger_history
        .function
        .parameters
        .get("properties")
        .and_then(|properties| properties.get("status"))
        .expect("list_trigger_fire_history schema should expose status");

    let statuses = status_schema
        .get("enum")
        .and_then(|value| value.as_array())
        .expect("status schema should expose enum values")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        statuses,
        vec!["fired", "succeeded", "failed", "suppressed", "dead_letter"]
    );
    assert!(trigger_history.function.description.contains("dead_letter"));
    assert!(trigger_history.function.description.contains("suppressed"));
}
