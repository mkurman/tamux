use super::*;
pub(super) struct TestExchange {
    pub(super) result: std::result::Result<StoredOpenAICodexAuth, String>,
}

pub(super) const CODEX_CLI_AUTH_FIXTURE_JSON: &str = r#"{"tokens":{"access_token":"header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdC0xIn0sImV4cCI6NDEwMjQ0NDgwMH0.signature","refresh_token":"refresh-token"}}"#;

impl OpenAICodexExchange for TestExchange {
    fn exchange_authorization_code(
        &self,
        _code: &str,
        _verifier: &str,
    ) -> Result<StoredOpenAICodexAuth> {
        match &self.result {
            Ok(auth) => Ok(auth.clone()),
            Err(message) => Err(anyhow::anyhow!(message.clone())),
        }
    }
}

pub(super) fn stored_auth_fixture() -> StoredOpenAICodexAuth {
    StoredOpenAICodexAuth {
        provider: Some("openai-codex".to_string()),
        auth_mode: Some("chatgpt_subscription".to_string()),
        access_token: "header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdC0xIn0sImV4cCI6NDEwMjQ0NDgwMH0.signature".to_string(),
        refresh_token: "refresh-token".to_string(),
        account_id: Some("acct-1".to_string()),
        expires_at: Some(4_102_444_800_000),
        source: Some("zorai".to_string()),
        updated_at: Some(4_102_444_800_000),
        created_at: Some(4_102_444_800_000),
    }
}

pub(super) fn write_codex_cli_auth_fixture(path: &std::path::Path) {
    std::fs::write(path, CODEX_CLI_AUTH_FIXTURE_JSON).expect("write codex auth fixture");
}

pub(super) fn set_test_auth_env(root: &std::path::Path, cli_auth_path: &std::path::Path) {
    std::env::set_var("ZORAI_PROVIDER_AUTH_DB_PATH", root.join("provider-auth.db"));
    std::env::set_var("ZORAI_CODEX_CLI_AUTH_PATH", cli_auth_path);
}

pub(super) fn prepare_openai_auth_test(
    root: &std::path::Path,
    cli_auth_name: &str,
) -> std::path::PathBuf {
    let cli_auth_path = root.join(cli_auth_name);
    set_test_auth_env(root, &cli_auth_path);
    clear_openai_codex_auth_test_state();
    cli_auth_path
}

pub(super) fn extract_state_from_auth_url(auth_url: &str) -> String {
    url::Url::parse(auth_url)
        .expect("auth url should parse")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .expect("auth url should contain state")
}

pub(super) fn wait_for_listener_and_send_callback(state: &str, code: &str) {
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
