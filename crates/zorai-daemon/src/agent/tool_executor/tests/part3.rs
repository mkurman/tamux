use super::super::{
    execute_fetch_url_with_runner,
    execute_search_files_with_runner,
};
use super::part1::successful_exit_status;
use crate::agent::{
    types::AgentConfig,
    AgentEngine,
};
use crate::session_manager::SessionManager;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use tokio::time::Duration;

#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

#[tokio::test]
async fn fetch_url_runtime_uses_default_timeout_on_caller_path() {
    let observed_timeout = Arc::new(Mutex::new(None));
    let observed_timeout_clone = observed_timeout.clone();

    let result = execute_fetch_url_with_runner(
        &serde_json::json!({ "url": "https://example.com" }),
        true,
        move |_url, timeout_seconds| {
            let observed_timeout = observed_timeout_clone.clone();
            async move {
                *observed_timeout
                    .lock()
                    .expect("timeout lock should succeed") = Some(timeout_seconds);
                Ok::<String, anyhow::Error>("<html><body>hello</body></html>".to_string())
            }
        },
        |_url, _timeout_seconds| async move {
            Ok::<String, anyhow::Error>("<html><body>http</body></html>".to_string())
        },
    )
    .await
    .expect("fetch_url should succeed");

    assert_eq!(
        *observed_timeout
            .lock()
            .expect("timeout lock should succeed"),
        Some(300)
    );
    assert_eq!(result, "hello");
}

#[tokio::test]
async fn fetch_url_runtime_clamps_timeout_override_on_caller_path() {
    let observed_browser_timeout = Arc::new(Mutex::new(None));
    let browser_timeout_clone = observed_browser_timeout.clone();
    let observed_http_timeout = Arc::new(Mutex::new(None));
    let http_timeout_clone = observed_http_timeout.clone();

    execute_fetch_url_with_runner(
        &serde_json::json!({ "url": "https://example.com", "timeout_seconds": 999 }),
        true,
        move |_url, timeout_seconds| {
            let observed_browser_timeout = browser_timeout_clone.clone();
            async move {
                *observed_browser_timeout
                    .lock()
                    .expect("timeout lock should succeed") = Some(timeout_seconds);
                Err::<String, anyhow::Error>(anyhow::anyhow!("browser unavailable"))
            }
        },
        move |_url, timeout_seconds| {
            let observed_http_timeout = http_timeout_clone.clone();
            async move {
                *observed_http_timeout
                    .lock()
                    .expect("timeout lock should succeed") = Some(timeout_seconds);
                Ok::<String, anyhow::Error>("<html><body>fallback</body></html>".to_string())
            }
        },
    )
    .await
    .expect("fetch_url should succeed");

    assert_eq!(
        *observed_browser_timeout
            .lock()
            .expect("timeout lock should succeed"),
        Some(600)
    );
    assert_eq!(
        *observed_http_timeout
            .lock()
            .expect("timeout lock should succeed"),
        Some(600)
    );
}

#[tokio::test]
async fn fetch_url_runtime_falls_back_to_http_after_browser_failure() {
    let browser_attempted = Arc::new(Mutex::new(false));
    let browser_attempted_clone = browser_attempted.clone();
    let http_attempted = Arc::new(Mutex::new(false));
    let http_attempted_clone = http_attempted.clone();

    let result = execute_fetch_url_with_runner(
        &serde_json::json!({ "url": "https://example.com" }),
        true,
        move |_url, _timeout_seconds| {
            let browser_attempted = browser_attempted_clone.clone();
            async move {
                *browser_attempted.lock().expect("lock should succeed") = true;
                Err::<String, anyhow::Error>(anyhow::anyhow!("browser failed"))
            }
        },
        move |_url, _timeout_seconds| {
            let http_attempted = http_attempted_clone.clone();
            async move {
                *http_attempted.lock().expect("lock should succeed") = true;
                Ok::<String, anyhow::Error>(
                    "<html><body>fallback content</body></html>".to_string(),
                )
            }
        },
    )
    .await
    .expect("fetch_url should fall back to http");

    assert!(*browser_attempted.lock().expect("lock should succeed"));
    assert!(*http_attempted.lock().expect("lock should succeed"));
    assert_eq!(result, "fallback content");
}

#[tokio::test]
async fn fetch_url_runtime_does_not_fallback_after_browser_timeout_exhausts_budget() {
    let http_attempted = Arc::new(Mutex::new(false));
    let http_attempted_clone = http_attempted.clone();
    let started = std::time::Instant::now();

    let error = execute_fetch_url_with_runner(
        &serde_json::json!({ "url": "https://example.com", "timeout_seconds": 1 }),
        true,
        |_url, timeout_seconds| async move {
            tokio::time::sleep(Duration::from_millis((timeout_seconds * 1000) + 50)).await;
            Ok::<String, anyhow::Error>("<html><body>late browser</body></html>".to_string())
        },
        move |_url, _timeout_seconds| {
            let http_attempted = http_attempted_clone.clone();
            async move {
                *http_attempted.lock().expect("lock should succeed") = true;
                Ok::<String, anyhow::Error>("<html><body>fallback</body></html>".to_string())
            }
        },
    )
    .await
    .expect_err("browser timeout should consume overall budget");

    assert!(error.to_string().contains("fetch_url timed out"));
    assert!(!*http_attempted.lock().expect("lock should succeed"));
    assert!(
        started.elapsed() < Duration::from_millis(1500),
        "overall timeout should not allow a fresh fallback budget"
    );
}

#[tokio::test]
async fn fetch_url_runtime_uses_remaining_budget_for_http_fallback_after_browser_failure() {
    let started = std::time::Instant::now();

    let error = execute_fetch_url_with_runner(
        &serde_json::json!({ "url": "https://example.com", "timeout_seconds": 1 }),
        true,
        |_url, _timeout_seconds| async move {
            tokio::time::sleep(Duration::from_millis(700)).await;
            Err::<String, anyhow::Error>(anyhow::anyhow!("browser failed after delay"))
        },
        |_url, _timeout_seconds| async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ok::<String, anyhow::Error>("<html><body>late fallback</body></html>".to_string())
        },
    )
    .await
    .expect_err("http fallback should only get remaining budget");

    assert!(error.to_string().contains("fetch_url timed out"));
    assert!(
        started.elapsed() < Duration::from_millis(1300),
        "fallback should not receive a fresh full timeout budget"
    );
}

#[tokio::test]
async fn fetch_url_runtime_does_not_fallback_on_browser_timeout_error() {
    let http_attempted = Arc::new(Mutex::new(false));
    let http_attempted_clone = http_attempted.clone();

    let error = execute_fetch_url_with_runner(
        &serde_json::json!({ "url": "https://example.com", "timeout_seconds": 1 }),
        true,
        |_url, _timeout_seconds| async move {
            Err::<String, anyhow::Error>(anyhow::anyhow!(
                "headless browser fetch timed out after 1 seconds"
            ))
        },
        move |_url, _timeout_seconds| {
            let http_attempted = http_attempted_clone.clone();
            async move {
                *http_attempted.lock().expect("lock should succeed") = true;
                Ok::<String, anyhow::Error>("<html><body>fallback</body></html>".to_string())
            }
        },
    )
    .await
    .expect_err("browser timeout error should not fall back to http");

    assert!(error
        .to_string()
        .contains("fetch_url timed out after 1 seconds"));
    assert!(!*http_attempted.lock().expect("lock should succeed"));
}

#[tokio::test]
async fn fetch_url_runtime_returns_timeout_error_when_runner_exceeds_limit() {
    let error = execute_fetch_url_with_runner(
        &serde_json::json!({ "url": "https://example.com", "timeout_seconds": 0 }),
        false,
        |_url, _timeout_seconds| async move {
            Ok::<String, anyhow::Error>("<html><body>browser</body></html>".to_string())
        },
        |_url, _timeout_seconds| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<String, anyhow::Error>("<html><body>late</body></html>".to_string())
        },
    )
    .await
    .expect_err("runner exceeding timeout should return timeout error");

    assert!(error.to_string().contains("fetch_url timed out"));
    assert!(error.to_string().contains("0"));
}

#[tokio::test]
async fn search_files_runtime_clamps_timeout_override_on_caller_path() {
    let observed_timeout = Arc::new(Mutex::new(None));
    let observed_timeout_clone = observed_timeout.clone();

    let result = execute_search_files_with_runner(
        &serde_json::json!({ "pattern": "needle", "timeout_seconds": 999 }),
        move |request| {
            let observed_timeout = observed_timeout_clone.clone();
            async move {
                *observed_timeout
                    .lock()
                    .expect("timeout lock should succeed") = Some(request.timeout_seconds);
                Ok::<super::SearchFilesCommandOutput, anyhow::Error>(
                    super::SearchFilesCommandOutput {
                        status: successful_exit_status(),
                        stdout: b"file.rs:1:needle\n".to_vec(),
                        stderr: Vec::new(),
                        truncated: false,
                    },
                )
            }
        },
    )
    .await
    .expect("search_files should succeed");

    assert_eq!(
        *observed_timeout
            .lock()
            .expect("timeout lock should succeed"),
        Some(600)
    );
    assert_eq!(result, "file.rs:1:needle");
}

#[tokio::test]
async fn search_files_runtime_returns_timeout_error_when_runner_exceeds_limit() {
    let error = execute_search_files_with_runner(
        &serde_json::json!({ "pattern": "needle", "timeout_seconds": 0 }),
        |_| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<super::SearchFilesCommandOutput, anyhow::Error>(super::SearchFilesCommandOutput {
                status: successful_exit_status(),
                stdout: Vec::new(),
                stderr: Vec::new(),
                truncated: false,
            })
        },
    )
    .await
    .expect_err("runner exceeding timeout should return timeout error");

    assert!(error.to_string().contains("search timed out"));
    assert!(error.to_string().contains("0"));
}

#[test]
fn fetch_url_request_parses_optional_profile_id() {
    let request = super::fetch_url_request(&serde_json::json!({
        "url": "https://example.com",
        "profile_id": " main-work "
    }))
    .expect("fetch_url request should parse");

    assert_eq!(request.profile_id.as_deref(), Some("main-work"));
}

#[test]
fn build_headless_browser_args_adds_profile_dir_for_supported_browser() {
    let browser = super::HeadlessBrowser {
        kind: "chrome",
        bin: "chrome".to_string(),
        args_prefix: vec!["--headless=new".to_string(), "--dump-dom".to_string()],
        profile_dir_arg_prefix: Some("--user-data-dir="),
    };

    let args = super::build_headless_browser_args(
        &browser,
        "https://example.com",
        Some("/tmp/browser-profile"),
    )
    .expect("chrome should accept profile dir");

    assert_eq!(
        args,
        vec![
            "--headless=new".to_string(),
            "--dump-dom".to_string(),
            "--user-data-dir=/tmp/browser-profile".to_string(),
            "https://example.com".to_string(),
        ]
    );
}

#[test]
fn build_headless_browser_args_rejects_profile_dir_for_unsupported_browser() {
    let browser = super::HeadlessBrowser {
        kind: "lightpanda",
        bin: "lightpanda".to_string(),
        args_prefix: vec!["fetch".to_string()],
        profile_dir_arg_prefix: None,
    };

    let error = super::build_headless_browser_args(
        &browser,
        "https://example.com",
        Some("/tmp/browser-profile"),
    )
    .expect_err("unsupported browser should reject profile dir");

    assert!(error
        .to_string()
        .contains("does not support browser profile directories"));
}

#[tokio::test]
async fn resolve_fetch_browser_profile_rejects_unusable_health_state() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;

    let profile = crate::agent::types::BrowserProfile {
        profile_id: "browser-profile-main".to_string(),
        label: "Main Browser".to_string(),
        profile_dir: "/tmp/browser-profile-main".to_string(),
        browser_kind: Some("chrome".to_string()),
        workspace_id: None,
        health_state: crate::agent::types::BrowserProfileHealth::RepairNeeded,
        created_at: 1_777_230_000,
        updated_at: 1_777_230_100,
        last_used_at: None,
        last_auth_success_at: None,
        last_auth_failure_at: Some(1_777_230_200),
        last_auth_failure_reason: Some("cookies expired".to_string()),
    };
    engine
        .history
        .upsert_browser_profile(&profile)
        .await
        .expect("profile should persist");

    let error = super::resolve_fetch_browser_profile(&engine, "browser-profile-main")
        .await
        .expect_err("repair-needed profile should be rejected");

    assert!(error.to_string().contains("not usable for fetch_url"));
    assert!(error.to_string().contains("repair_needed"));
}

#[tokio::test]
async fn resolve_fetch_browser_profile_reclassifies_and_rejects_expired_profile() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;

    let now_ms = crate::agent::now_millis();
    let sixty_days_ago = now_ms.saturating_sub(60 * 24 * 60 * 60 * 1000);
    let profile = crate::agent::types::BrowserProfile {
        profile_id: "browser-profile-expired".to_string(),
        label: "Expired Browser".to_string(),
        profile_dir: "/tmp/browser-profile-expired".to_string(),
        browser_kind: Some("chrome".to_string()),
        workspace_id: None,
        health_state: crate::agent::types::BrowserProfileHealth::Healthy,
        created_at: sixty_days_ago,
        updated_at: sixty_days_ago,
        last_used_at: Some(sixty_days_ago),
        last_auth_success_at: Some(sixty_days_ago),
        last_auth_failure_at: None,
        last_auth_failure_reason: None,
    };
    engine
        .history
        .upsert_browser_profile(&profile)
        .await
        .expect("profile should persist");

    let error = super::resolve_fetch_browser_profile(&engine, "browser-profile-expired")
        .await
        .expect_err("expired profile should be rejected");

    assert!(error.to_string().contains("not usable for fetch_url"));
    assert!(error.to_string().contains("expired"));

    let updated = engine
        .history
        .get_browser_profile("browser-profile-expired")
        .await
        .expect("lookup should succeed")
        .expect("profile should exist");
    assert_eq!(updated.health_state, "expired");
}

#[tokio::test]
async fn record_browser_profile_fetch_success_updates_last_used_at() {
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let engine = AgentEngine::new_test(manager, AgentConfig::default(), root.path()).await;

    let profile = crate::agent::types::BrowserProfile {
        profile_id: "browser-profile-main".to_string(),
        label: "Main Browser".to_string(),
        profile_dir: "/tmp/browser-profile-main".to_string(),
        browser_kind: Some("chrome".to_string()),
        workspace_id: None,
        health_state: crate::agent::types::BrowserProfileHealth::Healthy,
        created_at: 1_777_230_000,
        updated_at: 1_777_230_100,
        last_used_at: None,
        last_auth_success_at: None,
        last_auth_failure_at: None,
        last_auth_failure_reason: None,
    };
    engine
        .history
        .upsert_browser_profile(&profile)
        .await
        .expect("profile should persist");

    let row = engine
        .history
        .get_browser_profile("browser-profile-main")
        .await
        .expect("lookup should succeed")
        .expect("profile should exist");
    assert_eq!(row.last_used_at, None);

    super::record_browser_profile_fetch_success(&engine, &row)
        .await
        .expect("success should update metadata");

    let updated = engine
        .history
        .get_browser_profile("browser-profile-main")
        .await
        .expect("lookup should succeed")
        .expect("profile should exist");
    assert!(updated.last_used_at.is_some());
    assert!(updated.updated_at >= row.updated_at);
}
