use super::part5_support::*;
use super::*;
use crate::agent::provider_auth_store;
use crate::test_support::EnvGuard;
use tempfile::tempdir;
#[test]
fn logout_during_pending_cancels_flow() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["ZORAI_PROVIDER_AUTH_DB_PATH", "ZORAI_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    begin_openai_codex_auth_login().expect("login should start");

    logout_openai_codex_auth().expect("logout should succeed");
    let status = openai_codex_auth_status(false);

    assert_eq!(status.status, None);
    assert!(!status.available);
    assert!(status.auth_url.is_none());
}

#[test]
fn logout_tombstone_blocks_codex_import() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["ZORAI_PROVIDER_AUTH_DB_PATH", "ZORAI_CODEX_CLI_AUTH_PATH"]);
    std::env::set_var(
        "ZORAI_PROVIDER_AUTH_DB_PATH",
        temp_dir.path().join("provider-auth.db"),
    );
    let codex_auth_path = prepare_openai_auth_test(temp_dir.path(), "codex-auth.json");
    write_codex_cli_auth_fixture(&codex_auth_path);

    logout_openai_codex_auth().expect("logout should succeed");
    assert!(tombstone_present_for_tests());
    assert!(import_codex_cli_auth_if_present()
        .expect("import should not error")
        .is_none());

    reset_openai_codex_auth_runtime_for_tests();
    assert!(tombstone_present_for_tests());
    assert!(import_codex_cli_auth_if_present()
        .expect("import should not error")
        .is_none());
}

#[test]
fn explicit_login_clears_tombstone() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["ZORAI_PROVIDER_AUTH_DB_PATH", "ZORAI_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    logout_openai_codex_auth().expect("logout should succeed");
    assert!(tombstone_present_for_tests());

    let login = begin_openai_codex_auth_login().expect("login should start");

    assert_eq!(login.status.as_deref(), Some("pending"));
    assert!(!tombstone_present_for_tests());
}

#[test]
fn successful_login_reports_error_when_persistence_fails() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["ZORAI_PROVIDER_AUTH_DB_PATH", "ZORAI_CODEX_CLI_AUTH_PATH"]);
    prepare_openai_auth_test(temp_dir.path(), "missing-codex-auth.json");
    begin_openai_codex_auth_login().expect("login should start");
    std::env::set_var("ZORAI_PROVIDER_AUTH_DB_PATH", temp_dir.path());

    let status = complete_openai_codex_auth_with_code_for_tests(
        "good-code",
        &TestExchange {
            result: Ok(stored_auth_fixture()),
        },
    );

    assert_eq!(status.status.as_deref(), Some("error"));
    assert!(!status.available);
    assert_eq!(
        status.error.as_deref(),
        Some("OpenAI authentication could not be saved. Please try again.")
    );
}

#[test]
fn status_refresh_reports_import_persistence_failure() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["ZORAI_PROVIDER_AUTH_DB_PATH", "ZORAI_CODEX_CLI_AUTH_PATH"]);
    let codex_auth_path = temp_dir.path().join("codex-auth.json");
    let invalid_parent = temp_dir.path().join("not-a-dir");
    std::fs::write(&invalid_parent, "blocking parent").expect("write blocking file");
    std::env::set_var(
        "ZORAI_PROVIDER_AUTH_DB_PATH",
        invalid_parent.join("provider-auth.db"),
    );
    std::env::set_var("ZORAI_CODEX_CLI_AUTH_PATH", &codex_auth_path);
    clear_openai_codex_auth_test_state();
    write_codex_cli_auth_fixture(&codex_auth_path);

    let status = openai_codex_auth_status(true);

    assert_eq!(status.status.as_deref(), Some("error"));
    assert!(!status.available);
    assert_eq!(
        status.error.as_deref(),
        Some("OpenAI authentication could not be saved. Please try again.")
    );
}

#[test]
fn codex_status_payload_omits_secrets() {
    let _lock = provider_auth_store::provider_auth_test_env_lock();
    let temp_dir = tempdir().expect("tempdir should succeed");
    let _env_guard = EnvGuard::new(&["ZORAI_PROVIDER_AUTH_DB_PATH", "ZORAI_CODEX_CLI_AUTH_PATH"]);
    set_test_auth_env(
        temp_dir.path(),
        &temp_dir.path().join("missing-codex-auth.json"),
    );
    reset_openai_codex_auth_runtime_for_tests();
    begin_openai_codex_auth_login().expect("login should start");
    let pending_json =
        serde_json::to_value(begin_openai_codex_auth_login().expect("pending login"))
            .expect("serialize pending");
    assert!(pending_json.get("accessToken").is_none());
    assert!(pending_json.get("refreshToken").is_none());

    complete_openai_codex_auth_with_code_for_tests(
        "good-code",
        &TestExchange {
            result: Ok(stored_auth_fixture()),
        },
    );
    let status_json =
        serde_json::to_value(openai_codex_auth_status(false)).expect("serialize status");
    assert!(status_json.get("accessToken").is_none());
    assert!(status_json.get("refreshToken").is_none());
}
