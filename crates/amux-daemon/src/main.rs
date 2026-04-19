pub mod agent;
mod criu;
mod git;
mod governance;
mod history;
mod lsp_client;
mod network;
mod notifications;
mod orchestration;
mod osc;
pub mod plugin;
mod policy;
mod policy_external;
mod pty_session;
mod sandbox;
mod scrub;
mod server;
mod session_manager;
mod snapshot;
mod state;
mod validation;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn init_logging() -> Result<tracing_appender::non_blocking::WorkerGuard> {
    let file_appender = amux_protocol::DailyLogWriter::new("tamux-daemon.log")?;
    let log_path = file_appender.current_path()?;
    let (writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("TAMUX_LOG")
                .or_else(|_| EnvFilter::try_from_env("AMUX_LOG"))
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(writer)
        .with_ansi(false)
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!(panic = %panic_info, "tamux-daemon panicked");
    }));

    tracing::info!(path = %log_path.display(), "daemon log file initialized");
    Ok(guard)
}

const DAEMON_TOKIO_WORKER_STACK_BYTES: usize = 8 * 1024 * 1024;

async fn daemon_main() -> Result<()> {
    if std::env::args().nth(1).as_deref()
        == Some(agent::skill_preflight::SKILL_DISCOVERY_WORKER_ARG)
    {
        return agent::skill_preflight::run_skill_discovery_worker_from_stdio().await;
    }
    if std::env::args().nth(1).as_deref() == Some(agent::ALINE_STARTUP_WORKER_ARG) {
        return agent::run_aline_startup_worker_from_stdio().await;
    }

    let _log_guard = init_logging()?;

    tracing::info!("tamux-daemon starting");

    // Start the IPC server (blocks until shutdown signal).
    server::run().await?;

    Ok(())
}

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(DAEMON_TOKIO_WORKER_STACK_BYTES)
        .build()?
        .block_on(daemon_main())
}
