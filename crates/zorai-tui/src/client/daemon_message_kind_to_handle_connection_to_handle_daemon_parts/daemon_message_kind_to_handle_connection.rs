use crate::client::{ClientEvent, DaemonClient};
use anyhow::Result;
use futures::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::{Instant, MissedTickBehavior};
use tokio_util::codec::Framed;
use tracing::{debug, error, info, warn};
#[cfg(not(unix))]
use zorai_protocol::default_tcp_addr;
use zorai_protocol::{ClientMessage, DaemonMessage, ZoraiCodec};
impl DaemonClient {
    fn daemon_message_kind(message: &DaemonMessage) -> &'static str {
        match message {
            DaemonMessage::AgentAsyncCommandCapabilityAck { .. } => {
                "AgentAsyncCommandCapabilityAck"
            }
            DaemonMessage::OperationAccepted { .. } => "OperationAccepted",
            DaemonMessage::AgentEvent { .. } => "AgentEvent",
            DaemonMessage::AgentThreadList { .. } => "AgentThreadList",
            DaemonMessage::AgentThreadDetail { .. } => "AgentThreadDetail",
            DaemonMessage::AgentThreadDetailChunk { .. } => "AgentThreadDetailChunk",
            DaemonMessage::SessionSpawned { .. } => "SessionSpawned",
            DaemonMessage::ApprovalRequired { .. } => "ApprovalRequired",
            DaemonMessage::ApprovalResolved { .. } => "ApprovalResolved",
            DaemonMessage::AgentTaskList { .. } => "AgentTaskList",
            DaemonMessage::AgentGoalRunList { .. } => "AgentGoalRunList",
            DaemonMessage::AgentGoalRunDetail { .. } => "AgentGoalRunDetail",
            DaemonMessage::AgentEventRows { .. } => "AgentEventRows",
            DaemonMessage::AgentExternalRuntimeMigrationResult { .. } => {
                "AgentExternalRuntimeMigrationResult"
            }
            DaemonMessage::Error { .. } => "Error",
            DaemonMessage::AgentError { .. } => "AgentError",
            DaemonMessage::Pong => "Pong",
            _ => "Other",
        }
    }

    #[cfg(unix)]
    pub(crate) fn configure_daemon_bootstrap_command(command: &mut tokio::process::Command) {
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    #[cfg(not(unix))]
    pub(crate) fn configure_daemon_bootstrap_command(_command: &mut tokio::process::Command) {}

    pub(crate) fn next_bootstrap_attempted(bootstrap_attempted: bool, connected: bool) -> bool {
        if connected {
            false
        } else {
            bootstrap_attempted
        }
    }

    async fn probe_daemon_once() -> bool {
        #[cfg(unix)]
        {
            for socket_path in Self::unix_socket_candidates() {
                if let Ok(stream) = UnixStream::connect(&socket_path).await {
                    drop(stream);
                    return true;
                }
            }
            false
        }

        #[cfg(not(unix))]
        {
            let addr = Self::resolve_daemon_addr(&default_tcp_addr());
            tokio::net::TcpStream::connect(&addr).await.is_ok()
        }
    }

    fn daemon_spawn_candidates() -> Vec<std::ffi::OsString> {
        let mut candidates = Vec::new();

        if let Some(path) = std::env::var_os("ZORAI_DAEMON_BIN") {
            if !path.is_empty() {
                candidates.push(path);
            }
        }

        if let Ok(current_exe) = std::env::current_exe() {
            let exe_name = if cfg!(windows) {
                "zorai-daemon.exe"
            } else {
                "zorai-daemon"
            };
            candidates.push(current_exe.with_file_name(exe_name).into_os_string());
        }

        candidates.push(std::ffi::OsString::from(if cfg!(windows) {
            "zorai-daemon.exe"
        } else {
            "zorai-daemon"
        }));
        candidates
    }

    async fn attempt_spawn_daemon() -> bool {
        for candidate in Self::daemon_spawn_candidates() {
            let candidate_display = candidate.to_string_lossy().to_string();
            info!(candidate = %candidate_display, "Attempting daemon bootstrap");

            let mut command = tokio::process::Command::new(&candidate);
            command
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            Self::configure_daemon_bootstrap_command(&mut command);

            let spawn_result = command.spawn();

            match spawn_result {
                Ok(child) => {
                    info!(
                        candidate = %candidate_display,
                        pid = child.id().unwrap_or_default(),
                        "Daemon bootstrap process spawned"
                    );
                    drop(child);
                    for _ in 0..120 {
                        tokio::time::sleep(Duration::from_millis(250)).await;
                        if Self::probe_daemon_once().await {
                            info!(candidate = %candidate_display, "Daemon bootstrap succeeded");
                            return true;
                        }
                    }
                    warn!(candidate = %candidate_display, "Daemon bootstrap command ran but daemon never became ready within 30s");
                }
                Err(err) => {
                    debug!(candidate = %candidate_display, error = %err, "Daemon bootstrap spawn failed");
                }
            }
        }

        false
    }

    pub fn new(event_tx: mpsc::Sender<ClientEvent>) -> Self {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        Self {
            event_tx,
            request_tx,
            request_rx: Mutex::new(Some(request_rx)),
        }
    }

    pub async fn connect(&self) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let Some(mut request_rx) = self
            .request_rx
            .lock()
            .expect("request mutex poisoned")
            .take()
        else {
            return Ok(());
        };

        tokio::spawn(async move {
            let retry_delay = Duration::from_secs(5);
            let mut bootstrap_attempted = false;

            loop {
                let mut connected = false;
                #[cfg(unix)]
                {
                    for socket_path in Self::unix_socket_candidates() {
                        info!(path = %socket_path.display(), "Attempting daemon unix socket");
                        match UnixStream::connect(&socket_path).await {
                            Ok(stream) => {
                                info!("Connected to daemon via unix socket");
                                let _ = event_tx.send(ClientEvent::Connected).await;
                                let framed = Framed::new(stream, ZoraiCodec);
                                Self::handle_connection(framed, event_tx.clone(), &mut request_rx)
                                    .await;
                                connected = true;
                                break;
                            }
                            Err(err) => {
                                debug!(path = %socket_path.display(), error = %err, "Unix socket connect failed");
                            }
                        }
                    }
                }

                #[cfg(not(unix))]
                {
                    let addr = Self::resolve_daemon_addr(&default_tcp_addr());
                    info!(%addr, "Attempting daemon tcp socket");
                    match tokio::net::TcpStream::connect(&addr).await {
                        Ok(stream) => {
                            info!("Connected to daemon via tcp {}", addr);
                            let _ = event_tx.send(ClientEvent::Connected).await;
                            let framed = Framed::new(stream, ZoraiCodec);
                            Self::handle_connection(framed, event_tx.clone(), &mut request_rx)
                                .await;
                            connected = true;
                        }
                        Err(err) => {
                            warn!("Cannot connect to daemon at {} ({})", addr, err);
                            let _ = event_tx.send(ClientEvent::Disconnected).await;
                        }
                    }
                }

                bootstrap_attempted =
                    Self::next_bootstrap_attempted(bootstrap_attempted, connected);

                if !connected && !bootstrap_attempted {
                    bootstrap_attempted = true;
                    if Self::attempt_spawn_daemon().await {
                        continue;
                    }
                }

                let _ = event_tx
                    .send(ClientEvent::Reconnecting {
                        delay_secs: retry_delay.as_secs(),
                    })
                    .await;

                if connected {
                    info!(
                        "Daemon connection closed; retrying in {}s",
                        retry_delay.as_secs()
                    );
                }
                tokio::time::sleep(retry_delay).await;
            }
        });

        Ok(())
    }

    #[cfg(not(unix))]
    fn resolve_daemon_addr(default_addr: &str) -> String {
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
                || std::path::Path::new("/run/WSL").exists()
            {
                if let Ok(contents) = std::fs::read_to_string("/etc/resolv.conf") {
                    for line in contents.lines() {
                        if line.starts_with("nameserver") {
                            if let Some(host_ip) = line.split_whitespace().nth(1) {
                                let port = default_addr.split(':').nth(1).unwrap_or("17563");
                                return format!("{}:{}", host_ip, port);
                            }
                        }
                    }
                }
            }
        }

        default_addr.to_string()
    }

    #[cfg(unix)]
    fn unix_socket_candidates() -> Vec<std::path::PathBuf> {
        let mut candidates = Vec::new();

        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            candidates.push(std::path::PathBuf::from(runtime_dir).join("zorai-daemon.sock"));
        }

        candidates.push(std::path::PathBuf::from("/tmp").join("zorai-daemon.sock"));
        candidates.dedup();
        candidates
    }

    async fn handle_connection<S>(
        framed: Framed<S, ZoraiCodec>,
        event_tx: mpsc::Sender<ClientEvent>,
        request_rx: &mut mpsc::UnboundedReceiver<ClientMessage>,
    ) where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let (mut sink, mut stream) = framed.split();
        let keepalive_interval = Duration::from_secs(5);
        let keepalive_timeout = Duration::from_secs(10);
        let mut ping_tick = tokio::time::interval(keepalive_interval);
        ping_tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut thread_detail_chunks = None;

        for request in [
            ClientMessage::AgentSubscribe,
            ClientMessage::AgentDeclareAsyncCommandCapability {
                capability: zorai_protocol::AsyncCommandCapability {
                    version: 1,
                    supports_operation_acceptance: true,
                },
            },
            ClientMessage::AgentListThreads {
                limit: None,
                offset: None,
                include_internal: true,
                agent_filter: Some(zorai_protocol::AGENT_HANDLE_SVAROG.to_string()),
            },
        ] {
            if let Err(err) = zorai_protocol::validate_client_message_size(&request) {
                error!("Rejected initial oversized daemon request: {}", err);
                let _ = event_tx
                    .send(ClientEvent::Error(format!("Protocol error: {}", err)))
                    .await;
                let _ = event_tx.send(ClientEvent::Disconnected).await;
                return;
            }
            if let Err(err) = sink.send(request).await {
                error!("Failed initial daemon request: {}", err);
                let _ = event_tx
                    .send(ClientEvent::Error(format!("Protocol error: {}", err)))
                    .await;
                let _ = event_tx.send(ClientEvent::Disconnected).await;
                return;
            }
        }

        let last_inbound_ms: Arc<AtomicI64> = Arc::new(AtomicI64::new(now_unix_ms()));
        let last_inbound_for_writer = last_inbound_ms.clone();
        let event_tx_for_writer = event_tx.clone();

        let reader_fut = async {
            loop {
                match stream.next().await {
                    Some(Ok(message)) => {
                        debug!(
                            kind = Self::daemon_message_kind(&message),
                            "Daemon client received daemon message"
                        );
                        last_inbound_ms.store(now_unix_ms(), Ordering::Relaxed);
                        if !Self::handle_daemon_message(
                            message,
                            &event_tx,
                            &mut thread_detail_chunks,
                        )
                        .await
                        {
                            debug!("Daemon client stopping after daemon message handler requested exit");
                            return;
                        }
                    }
                    Some(Err(err)) => {
                        debug!(error = %err, "Daemon client stopping after inbound stream error");
                        let _ = event_tx
                            .send(ClientEvent::Error(format!("Connection error: {}", err)))
                            .await;
                        return;
                    }
                    None => {
                        debug!("Daemon client stopping after daemon stream reached EOF");
                        return;
                    }
                }
            }
        };

        let writer_fut = async {
            let mut awaiting_pong_since: Option<Instant> = None;
            loop {
                tokio::select! {
                    outbound = request_rx.recv() => {
                        match outbound {
                            Some(request) => {
                                let kind = client_message_short_name(&request);
                                if let Err(err) = zorai_protocol::validate_client_message_size(&request) {
                                    debug!(error = %err, "Dropping oversized outbound daemon request");
                                    let _ = event_tx_for_writer
                                        .send(ClientEvent::Error(format!("Send error: {}", err)))
                                        .await;
                                    continue;
                                }
                                let send_started = Instant::now();
                                if let Err(err) = sink.send(request).await {
                                    debug!(error = %err, "Daemon client writer stopping after outbound send error");
                                    let _ = event_tx_for_writer
                                        .send(ClientEvent::Error(format!("Send error: {}", err)))
                                        .await;
                                    return;
                                }
                                info!(
                                    kind,
                                    send_ms = send_started.elapsed().as_millis() as u64,
                                    "writer: sent client message to daemon"
                                );
                            }
                            None => {
                                debug!("Daemon client writer stopping after request channel closed");
                                return;
                            }
                        }
                    }
                    _ = ping_tick.tick() => {
                        let now = Instant::now();
                        let last_inbound_at = Instant::now()
                            .checked_sub(Duration::from_millis(
                                (now_unix_ms() - last_inbound_for_writer.load(Ordering::Relaxed))
                                    .max(0) as u64,
                            ))
                            .unwrap_or_else(Instant::now);

                        if let Some(pending_since) = awaiting_pong_since {
                            if now.duration_since(pending_since) >= keepalive_timeout {
                                debug!(
                                    elapsed_ms = now.duration_since(pending_since).as_millis() as u64,
                                    "Daemon client writer stopping after keepalive timeout"
                                );
                                let _ = event_tx_for_writer
                                    .send(ClientEvent::Error(
                                        "Connection lost: daemon health-check timed out".to_string(),
                                    ))
                                    .await;
                                return;
                            }
                        }

                        if now.duration_since(last_inbound_at) >= keepalive_interval {
                            let ping_send_started = Instant::now();
                            info!(
                                last_inbound_age_ms =
                                    now.duration_since(last_inbound_at).as_millis() as u64,
                                "writer: ping_tick fired, attempting Ping send"
                            );
                            if let Err(err) = sink.send(ClientMessage::Ping).await {
                                debug!(error = %err, "Daemon client writer stopping after keepalive send error");
                                let _ = event_tx_for_writer
                                    .send(ClientEvent::Error(format!(
                                        "Keepalive send error: {}",
                                        err
                                    )))
                                    .await;
                                return;
                            }
                            let ping_send_ms =
                                ping_send_started.elapsed().as_millis() as u64;
                            if ping_send_ms > 1000 {
                                tracing::warn!(
                                    ping_send_ms,
                                    "writer: Ping sink.send blocked >1s — daemon RX buffer full?"
                                );
                            } else {
                                info!(ping_send_ms, "writer: Ping send completed");
                            }
                            awaiting_pong_since = Some(now);
                        } else {
                            awaiting_pong_since = None;
                        }
                    }
                }
            }
        };

        tokio::pin!(reader_fut);
        tokio::pin!(writer_fut);
        tokio::select! {
            _ = &mut reader_fut => {}
            _ = &mut writer_fut => {}
        }

        let _ = event_tx.send(ClientEvent::Disconnected).await;
    }
}

fn client_message_short_name(msg: &ClientMessage) -> &'static str {
    use ClientMessage::*;
    match msg {
        Ping => "Ping",
        AgentSubscribe => "AgentSubscribe",
        AgentUnsubscribe => "AgentUnsubscribe",
        AgentGetConfig => "AgentGetConfig",
        AgentListThreads { .. } => "AgentListThreads",
        AgentListTasks => "AgentListTasks",
        AgentListGoalRuns { .. } => "AgentListGoalRuns",
        AgentListSubAgents => "AgentListSubAgents",
        AgentGetConciergeConfig => "AgentGetConciergeConfig",
        AgentRequestConciergeWelcome => "AgentRequestConciergeWelcome",
        AgentDeclareAsyncCommandCapability { .. } => "AgentDeclareAsyncCommandCapability",
        AgentGetProviderAuthStates => "AgentGetProviderAuthStates",
        AgentHeartbeatGetItems => "AgentHeartbeatGetItems",
        AgentGetThread { .. } => "AgentGetThread",
        ListAgentEvents { .. } => "ListAgentEvents",
        UpsertAgentEvent { .. } => "UpsertAgentEvent",
        PluginList { .. } => "PluginList",
        PluginListCommands { .. } => "PluginListCommands",
        _ => "<other>",
    }
}

fn now_unix_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
