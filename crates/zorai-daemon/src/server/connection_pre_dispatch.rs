use super::*;
use crate::agent::AgentEngine;
use anyhow::Result;
use futures::stream::SplitStream;
use futures::StreamExt;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, mpsc};
use tokio_util::codec::Framed;
use zorai_protocol::{DaemonCodec, DaemonMessage, SessionId};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn pre_dispatch<S>(
    agent: &Arc<AgentEngine>,
    stream: &mut SplitStream<Framed<S, DaemonCodec>>,
    framed: &mut ConnectionWriter,
    _plugin_manager: &Arc<crate::plugin::PluginManager>,
    attached_rxs: &mut Vec<(SessionId, broadcast::Receiver<DaemonMessage>)>,
    client_agent_threads: &mut HashSet<String>,
    last_concierge_welcome_fingerprint: &mut Option<String>,
    agent_event_rx: &mut Option<broadcast::Receiver<crate::agent::types::AgentEvent>>,
    background_daemon_queues: &mut BackgroundSubsystemQueues,
    background_daemon_pending: &mut BackgroundPendingCounts,
    whatsapp_link_rx: &mut Option<
        broadcast::Receiver<crate::agent::types::WhatsAppLinkRuntimeEvent>,
    >,
    whatsapp_link_snapshot_replayed: &mut bool,
    gateway_ipc_rx: &mut Option<mpsc::UnboundedReceiver<DaemonMessage>>,
    gateway_connection_state: &GatewayConnectionState,
) -> Result<PreDispatchOutcome>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    const MAX_EVENT_DRAIN_PER_CALL: usize = 256;
    let mut drained_event_count: usize = 0;

    let msg = {
        if let Some(ref mut rx) = agent_event_rx {
            while drained_event_count < MAX_EVENT_DRAIN_PER_CALL {
                match rx.try_recv() {
                    Ok(event) => {
                        if should_forward_agent_event(&event, &client_agent_threads) {
                            if let Some(fingerprint) = concierge_welcome_fingerprint(&event) {
                                if last_concierge_welcome_fingerprint.as_deref()
                                    == Some(fingerprint.as_str())
                                {
                                    continue;
                                }
                                *last_concierge_welcome_fingerprint = Some(fingerprint);
                            }
                            if let Some((json, truncated)) = cap_agent_event_for_ipc(&event) {
                                if truncated {
                                    tracing::warn!(
                                        thread_id = agent_event_thread_id(&event).unwrap_or(""),
                                        "truncated agent event to fit IPC frame limit"
                                    );
                                }
                                tracing::debug!(
                                    event_type = match &event {
                                        crate::agent::types::AgentEvent::Notification {
                                            ..
                                        } => "notification",
                                        crate::agent::types::AgentEvent::AnticipatoryUpdate {
                                            ..
                                        } => "anticipatory_update",
                                        crate::agent::types::AgentEvent::ConciergeWelcome {
                                            ..
                                        } => "concierge_welcome",
                                        crate::agent::types::AgentEvent::WorkflowNotice {
                                            ..
                                        } => "workflow_notice",
                                        crate::agent::types::AgentEvent::ThreadCreated {
                                            ..
                                        } => "thread_created",
                                        crate::agent::types::AgentEvent::ThreadReloadRequired {
                                            ..
                                        } => "thread_reload_required",
                                        crate::agent::types::AgentEvent::TaskUpdate { .. } =>
                                            "task_update",
                                        crate::agent::types::AgentEvent::GoalRunUpdate {
                                            ..
                                        } => "goal_run_update",
                                        _ => "other",
                                    },
                                    payload_bytes = json.len(),
                                    "forwarding agent event to client"
                                );
                                framed
                                    .send(DaemonMessage::AgentEvent { event_json: json })
                                    .await?;
                                drained_event_count += 1;
                            }
                        }
                    }
                    Err(broadcast::error::TryRecvError::Lagged(n)) => {
                        tracing::warn!(
                            skipped = n,
                            recovering_threads = client_agent_threads.len(),
                            "agent event broadcast lagged — synthesizing ThreadReloadRequired for subscribed threads"
                        );
                        for event in lag_recovery_thread_reload_events(client_agent_threads) {
                            if let Some((json, _truncated)) = cap_agent_event_for_ipc(&event) {
                                framed
                                    .send(DaemonMessage::AgentEvent { event_json: json })
                                    .await?;
                            }
                        }
                        break;
                    }
                    _ => break,
                }
            }
        }
        if let Some(ref mut rx) = whatsapp_link_rx {
            loop {
                match rx.try_recv() {
                    Ok(event) => match event {
                        crate::agent::types::WhatsAppLinkRuntimeEvent::Status(snapshot) => {
                            tracing::debug!(
                                state = %snapshot.state,
                                has_error = snapshot.last_error.is_some(),
                                "forwarding whatsapp runtime status to client"
                            );
                            if !*whatsapp_link_snapshot_replayed {
                                *whatsapp_link_snapshot_replayed = true;
                                framed
                                    .send(DaemonMessage::AgentWhatsAppLinkStatus {
                                        state: snapshot.state,
                                        phone: snapshot.phone,
                                        last_error: snapshot.last_error,
                                    })
                                    .await?;
                            }
                        }
                        crate::agent::types::WhatsAppLinkRuntimeEvent::Qr {
                            ascii_qr,
                            expires_at_ms,
                        } => {
                            tracing::debug!(
                                qr_len = ascii_qr.len(),
                                expires_at_ms,
                                "forwarding whatsapp runtime qr to client"
                            );
                            framed
                                .send(DaemonMessage::AgentWhatsAppLinkQr {
                                    ascii_qr,
                                    expires_at_ms,
                                })
                                .await?;
                        }
                        crate::agent::types::WhatsAppLinkRuntimeEvent::Linked { phone } => {
                            tracing::debug!(
                                phone = phone.as_deref().unwrap_or(""),
                                "forwarding whatsapp runtime linked to client"
                            );
                            framed
                                .send(DaemonMessage::AgentWhatsAppLinked { phone })
                                .await?;
                        }
                        crate::agent::types::WhatsAppLinkRuntimeEvent::Error {
                            message,
                            recoverable,
                        } => {
                            tracing::debug!(
                                recoverable,
                                message = %message,
                                "forwarding whatsapp runtime error to client"
                            );
                            framed
                                .send(DaemonMessage::AgentWhatsAppLinkError {
                                    message,
                                    recoverable,
                                })
                                .await?;
                        }
                        crate::agent::types::WhatsAppLinkRuntimeEvent::Disconnected { reason } => {
                            tracing::debug!(
                                reason = reason.as_deref().unwrap_or(""),
                                "forwarding whatsapp runtime disconnected to client"
                            );
                            framed
                                .send(DaemonMessage::AgentWhatsAppLinkDisconnected { reason })
                                .await?;
                        }
                    },
                    Err(broadcast::error::TryRecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "whatsapp link broadcast lagged");
                        break;
                    }
                    _ => break,
                }
            }
        }
        if let Some(ref mut rx) = gateway_ipc_rx {
            loop {
                match rx.try_recv() {
                    Ok(daemon_msg) => {
                        framed.send(daemon_msg).await?;
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                }
            }
        }
        for subsystem in BackgroundSubsystem::ALL {
            loop {
                match background_daemon_queues.try_recv(subsystem) {
                    Ok(BackgroundSignal::Deliver(daemon_msg)) => {
                        framed.send(daemon_msg).await?;
                    }
                    Ok(BackgroundSignal::Finished) => {
                        background_daemon_pending.decrement(subsystem);
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
                }
            }
        }

        let has_subscriptions = !attached_rxs.is_empty()
            || agent_event_rx.is_some()
            || whatsapp_link_rx.is_some()
            || gateway_ipc_rx.is_some()
            || background_daemon_pending.any();
        let msg = if !has_subscriptions {
            match stream.next().await {
                Some(Ok(msg)) => Some(msg),
                Some(Err(e)) => {
                    if gateway_connection_is_tracked(&gateway_connection_state) {
                        agent
                            .record_gateway_ipc_loss("gateway connection error")
                            .await;
                    }
                    return Err(e.into());
                }
                None => {
                    if gateway_connection_is_tracked(&gateway_connection_state) {
                        agent
                            .record_gateway_ipc_loss("gateway connection closed")
                            .await;
                    }
                    return Ok(PreDispatchOutcome::Terminate);
                }
            }
        } else {
            let mut forwarded = false;
            let mut closed_sessions = Vec::new();
            for (sid, rx) in attached_rxs.iter_mut() {
                loop {
                    match rx.try_recv() {
                        Ok(daemon_msg) => {
                            framed.send(daemon_msg).await?;
                            forwarded = true;
                        }
                        Err(broadcast::error::TryRecvError::Empty) => break,
                        Err(broadcast::error::TryRecvError::Lagged(n)) => {
                            tracing::warn!(session = %sid, skipped = n, "broadcast lagged");
                            break;
                        }
                        Err(broadcast::error::TryRecvError::Closed) => {
                            framed
                                .send(DaemonMessage::SessionExited {
                                    id: *sid,
                                    exit_code: None,
                                })
                                .await?;
                            closed_sessions.push(*sid);
                            forwarded = true;
                            break;
                        }
                    }
                }
            }
            if !closed_sessions.is_empty() {
                attached_rxs.retain(|(sid, _)| !closed_sessions.contains(sid));
            }

            match tokio::time::timeout(
                std::time::Duration::from_millis(if forwarded { 10 } else { 50 }),
                stream.next(),
            )
            .await
            {
                Ok(Some(Ok(msg))) => Some(msg),
                Ok(Some(Err(e))) => {
                    if gateway_connection_is_tracked(&gateway_connection_state) {
                        agent
                            .record_gateway_ipc_loss("gateway connection error")
                            .await;
                    }
                    return Err(e.into());
                }
                Ok(None) => {
                    if gateway_connection_is_tracked(&gateway_connection_state) {
                        agent
                            .record_gateway_ipc_loss("gateway connection closed")
                            .await;
                    }
                    return Ok(PreDispatchOutcome::Terminate);
                }
                Err(_) => None,
            }
        };
        msg
    };
    Ok(PreDispatchOutcome::Msg(msg))
}
