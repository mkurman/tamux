use super::*;
use crate::commands::common::find_sibling_binary;

pub(super) fn wizard_startup_messages() -> Vec<ClientMessage> {
    vec![ClientMessage::AgentDeclareAsyncCommandCapability {
        capability: amux_protocol::AsyncCommandCapability {
            version: 1,
            supports_operation_acceptance: true,
        },
    }]
}

async fn initialize_wizard_connection<T>(
    mut framed: Framed<T, AmuxCodec>,
) -> Result<Framed<T, AmuxCodec>>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    for msg in wizard_startup_messages() {
        framed.send(msg).await?;
    }
    Ok(framed)
}

#[cfg(unix)]
pub(super) async fn wizard_connect(
) -> Result<Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path = std::path::PathBuf::from(runtime_dir).join("tamux-daemon.sock");
    let stream = tokio::net::UnixStream::connect(&path)
        .await
        .with_context(|| format!("cannot connect to daemon at {}", path.display()))?;
    initialize_wizard_connection(Framed::new(stream, AmuxCodec)).await
}

#[cfg(windows)]
pub(super) async fn wizard_connect(
) -> Result<Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>> {
    let addr = amux_protocol::default_tcp_addr();
    let stream = tokio::net::TcpStream::connect(&addr)
        .await
        .with_context(|| format!("cannot connect to daemon on {addr}"))?;
    initialize_wizard_connection(Framed::new(stream, AmuxCodec)).await
}

pub(super) async fn wizard_send(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    msg: ClientMessage,
) -> Result<()> {
    framed.send(msg).await.map_err(Into::into)
}

pub(super) async fn wizard_recv(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
) -> Result<DaemonMessage> {
    loop {
        let msg = framed
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("daemon closed connection"))?
            .map_err(anyhow::Error::from)?;

        if should_ignore_wizard_message(&msg) {
            continue;
        }

        return Ok(msg);
    }
}

pub(super) fn should_ignore_wizard_message(msg: &DaemonMessage) -> bool {
    matches!(msg, DaemonMessage::AgentAsyncCommandCapabilityAck { .. })
}

pub(super) fn parse_provider_validation_terminal_response(
    msg: DaemonMessage,
) -> Option<Result<(bool, Option<String>)>> {
    match msg {
        DaemonMessage::OperationAccepted { .. } => None,
        DaemonMessage::AgentProviderValidation { valid, error, .. } => Some(Ok((valid, error))),
        DaemonMessage::Error { message } => Some(Err(anyhow::anyhow!(message))),
        _ => None,
    }
}

pub(super) fn parse_provider_login_terminal_response(
    msg: DaemonMessage,
) -> Option<Result<(bool, Option<String>)>> {
    match msg {
        DaemonMessage::AgentProviderLoginResult {
            success, message, ..
        } => Some(Ok((success, message))),
        DaemonMessage::Error { message } | DaemonMessage::AgentError { message } => {
            Some(Err(anyhow::anyhow!(message)))
        }
        _ => None,
    }
}

pub(super) async fn login_provider_on_stream(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    provider_id: &str,
    api_key: &str,
    base_url: &str,
    auth_source: Option<&str>,
) -> Result<(bool, Option<String>)> {
    wizard_send(
        framed,
        ClientMessage::AgentLoginProvider {
            provider_id: provider_id.to_string(),
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            auth_source: auth_source.map(str::to_string),
        },
    )
    .await?;

    loop {
        let msg = wizard_recv(framed).await?;
        if let Some(result) = parse_provider_login_terminal_response(msg) {
            return result;
        }
    }
}

pub(super) async fn validate_provider_on_stream(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    provider_id: &str,
    auth_source: &str,
) -> Result<bool> {
    wizard_send(
        framed,
        ClientMessage::AgentValidateProvider {
            provider_id: provider_id.to_string(),
            base_url: String::new(),
            api_key: String::new(),
            auth_source: auth_source.to_string(),
        },
    )
    .await?;

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match tokio::time::timeout_at(deadline, wizard_recv(framed)).await {
            Ok(Ok(msg)) => {
                if let Some(result) = parse_provider_validation_terminal_response(msg) {
                    let (valid, error) = result?;
                    if !valid {
                        if let Some(err) = error {
                            println!("Validation error: {err}");
                        }
                    }
                    return Ok(valid);
                }
            }
            Ok(Err(e)) => anyhow::bail!("Connection error: {e}"),
            Err(_) => anyhow::bail!("Timed out (30s)"),
        }
    }
}

pub(super) async fn read_config_key(key: &str) -> Option<String> {
    let mut conn = wizard_connect().await.ok()?;
    wizard_send(&mut conn, ClientMessage::AgentGetConfig)
        .await
        .ok()?;
    match wizard_recv(&mut conn).await.ok()? {
        DaemonMessage::AgentConfigResponse { config_json } => {
            let val: serde_json::Value = serde_json::from_str(&config_json).ok()?;
            let mut current = &val;
            for part in key.split('.') {
                current = current.get(part)?;
            }
            match current {
                serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
                serde_json::Value::Bool(b) => Some(b.to_string()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                _ => None,
            }
        }
        _ => None,
    }
}

pub(crate) async fn ensure_daemon_running() -> Result<()> {
    if wizard_connect().await.is_ok() {
        return Ok(());
    }

    println!("Starting daemon...");
    let mut cmd = std::process::Command::new(find_sibling_binary("tamux-daemon"));
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    if let Err(e) = cmd.spawn() {
        anyhow::bail!("Could not start daemon: {e}\nPlease start it manually with: tamux-daemon");
    }

    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if wizard_connect().await.is_ok() {
            println!("Daemon started.");
            return Ok(());
        }
    }

    anyhow::bail!(
        "Daemon did not become reachable within 5 seconds.\n\
         Please start it manually with: tamux-daemon"
    )
}
