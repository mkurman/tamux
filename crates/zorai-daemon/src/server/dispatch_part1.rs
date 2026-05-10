use super::*;
use crate::agent::AgentEngine;
use crate::session_manager::SessionManager;
use anyhow::Result;
use std::sync::Arc;
use zorai_protocol::{ClientMessage, DaemonMessage};

pub(crate) async fn dispatch_part1(
    msg: &ClientMessage,
    agent: &Arc<AgentEngine>,
    framed: &mut ConnectionWriter,
    manager: &Arc<SessionManager>,
    attached_rxs: &mut Vec<(
        zorai_protocol::SessionId,
        tokio::sync::broadcast::Receiver<DaemonMessage>,
    )>,
    gateway_connection_state: &mut GatewayConnectionState,
    gateway_ipc_rx: &mut Option<tokio::sync::mpsc::UnboundedReceiver<DaemonMessage>>,
) -> Result<DispatchOutcome>
{
    if !matches!(
        msg,
        ClientMessage::Ping
            | ClientMessage::GatewayRegister { .. }
            | ClientMessage::GatewayAck { .. }
            | ClientMessage::GatewayIncomingEvent { .. }
            | ClientMessage::GatewayCursorUpdate { .. }
            | ClientMessage::GatewayThreadBindingUpdate { .. }
            | ClientMessage::GatewayRouteModeUpdate { .. }
            | ClientMessage::GatewaySendResult { .. }
            | ClientMessage::GatewayHealthUpdate { .. }
            | ClientMessage::SpawnSession { .. }
            | ClientMessage::CloneSession { .. }
            | ClientMessage::AttachSession { .. }
            | ClientMessage::DetachSession { .. }
    ) {
        return Ok(DispatchOutcome::NotMatched);
    }
    let msg = msg.clone();

    match msg {
        ClientMessage::Ping => {
            framed.send(DaemonMessage::Pong).await?;
        }

        ClientMessage::GatewayRegister { registration } => {
            if !matches!(
                gateway_connection_state,
                GatewayConnectionState::Unregistered
            ) {
                framed
                    .send(DaemonMessage::Error {
                        message: "gateway runtime already registered on this connection"
                            .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
            if registration.protocol_version != GATEWAY_IPC_PROTOCOL_VERSION {
                framed
                    .send(DaemonMessage::Error {
                        message: format!(
                            "unsupported gateway protocol version {} (expected {})",
                            registration.protocol_version, GATEWAY_IPC_PROTOCOL_VERSION
                        ),
                    })
                    .await?;
                return Ok(DispatchOutcome::Terminate);
            }
            tracing::info!(
                gateway_id = %registration.gateway_id,
                instance_id = %registration.instance_id,
                protocol_version = registration.protocol_version,
                "gateway runtime registered on daemon socket"
            );
            let bootstrap_correlation_id = format!("gateway-bootstrap:{}", uuid::Uuid::new_v4());
            let payload =
                build_gateway_bootstrap_payload(&agent, bootstrap_correlation_id.clone()).await;
            let (gateway_tx, gateway_rx) = mpsc::unbounded_channel();
            agent.set_gateway_ipc_sender(Some(gateway_tx)).await;
            *gateway_ipc_rx = Some(gateway_rx);
            framed
                .send(DaemonMessage::GatewayBootstrap { payload })
                .await?;
            *gateway_connection_state = GatewayConnectionState::AwaitingBootstrapAck {
                registration,
                bootstrap_correlation_id,
            };
        }

        ClientMessage::GatewayAck { ack } => match &gateway_connection_state {
            GatewayConnectionState::AwaitingBootstrapAck {
                registration,
                bootstrap_correlation_id,
            } if ack.correlation_id == *bootstrap_correlation_id && ack.accepted => {
                tracing::info!(
                    gateway_id = %registration.gateway_id,
                    instance_id = %registration.instance_id,
                    correlation_id = %ack.correlation_id,
                    "gateway runtime bootstrap acknowledged"
                );
                *gateway_connection_state = GatewayConnectionState::Active {
                    registration: registration.clone(),
                };
            }
            GatewayConnectionState::AwaitingBootstrapAck {
                bootstrap_correlation_id,
                ..
            } => {
                framed
                                .send(DaemonMessage::Error {
                                    message: format!(
                                        "invalid gateway bootstrap ack: expected correlation_id {} and accepted=true",
                                        bootstrap_correlation_id
                                    ),
                                })
                                .await?;
            }
            _ => {
                framed
                    .send(DaemonMessage::Error {
                        message: "gateway ack received before gateway registration".to_string(),
                    })
                    .await?;
            }
        },

        ClientMessage::GatewayIncomingEvent { event } => {
            if !gateway_connection_is_active(&gateway_connection_state) {
                framed
                    .send(DaemonMessage::Error {
                        message: "gateway incoming events require a registered gateway connection"
                            .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
            if let Err(error) = enqueue_gateway_incoming_event(&agent, event).await {
                framed
                    .send(DaemonMessage::Error {
                        message: format!("failed to enqueue gateway event: {error}"),
                    })
                    .await?;
            }
        }

        ClientMessage::GatewayCursorUpdate { update } => {
            if !gateway_connection_is_active(&gateway_connection_state) {
                framed
                    .send(DaemonMessage::Error {
                        message: "gateway cursor updates require a registered gateway connection"
                            .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
            if let Err(error) = agent
                .history
                .save_gateway_replay_cursor(
                    &update.platform,
                    &update.channel_id,
                    &update.cursor_value,
                    &update.cursor_type,
                )
                .await
            {
                framed
                    .send(DaemonMessage::Error {
                        message: format!("failed to persist gateway cursor: {error}"),
                    })
                    .await?;
            }
        }

        ClientMessage::GatewayThreadBindingUpdate { update } => {
            if !gateway_connection_is_active(&gateway_connection_state) {
                framed
                    .send(DaemonMessage::Error {
                        message:
                            "gateway thread binding updates require a registered gateway connection"
                                .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
            let result = match update.thread_id.as_deref() {
                Some(thread_id) => {
                    let updated_at = if update.updated_at_ms == 0 {
                        current_time_ms()
                    } else {
                        update.updated_at_ms
                    };
                    agent
                        .history
                        .upsert_gateway_thread_binding(&update.channel_key, thread_id, updated_at)
                        .await
                }
                None => {
                    agent
                        .history
                        .delete_gateway_thread_binding(&update.channel_key)
                        .await
                }
            };
            if let Err(error) = result {
                framed
                    .send(DaemonMessage::Error {
                        message: format!("failed to persist gateway thread binding: {error}"),
                    })
                    .await?;
            }
        }

        ClientMessage::GatewayRouteModeUpdate { update } => {
            if !gateway_connection_is_active(&gateway_connection_state) {
                framed
                    .send(DaemonMessage::Error {
                        message:
                            "gateway route mode updates require a registered gateway connection"
                                .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
            let result = agent
                .history
                .upsert_gateway_route_mode(
                    &update.channel_key,
                    match update.route_mode {
                        GatewayRouteMode::Rarog => "rarog",
                        GatewayRouteMode::Swarog => "swarog",
                    },
                    if update.updated_at_ms == 0 {
                        current_time_ms()
                    } else {
                        update.updated_at_ms
                    },
                )
                .await;
            if let Err(error) = result {
                framed
                    .send(DaemonMessage::Error {
                        message: format!("failed to persist gateway route mode: {error}"),
                    })
                    .await?;
            }
        }

        ClientMessage::GatewaySendResult { result } => {
            if !gateway_connection_is_active(&gateway_connection_state) {
                framed
                    .send(DaemonMessage::Error {
                        message: "gateway send results require a registered gateway connection"
                            .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
            tracing::debug!(
                correlation_id = %result.correlation_id,
                platform = %result.platform,
                ok = result.ok,
                "gateway send result received"
            );
            if result.ok {
                if let Some(channel_key) =
                    gateway_response_channel_key(&result.platform, &result.channel_id)
                {
                    let mut gw_guard = agent.gateway_state.lock().await;
                    if let Some(gateway_state) = gw_guard.as_mut() {
                        if result.platform.eq_ignore_ascii_case("discord") {
                            if let Some(requested_channel_id) =
                                result.requested_channel_id.as_deref()
                            {
                                if requested_channel_id.starts_with("user:")
                                    && requested_channel_id != result.channel_id
                                {
                                    gateway_state.discord_dm_channels_by_user.insert(
                                        requested_channel_id.to_string(),
                                        result.channel_id.clone(),
                                    );
                                }
                            }
                            if let Some(delivery_id) = result.delivery_id.as_ref() {
                                gateway_state.reply_contexts.insert(
                                    channel_key.clone(),
                                    crate::agent::gateway::ThreadContext {
                                        discord_message_id: Some(delivery_id.clone()),
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                        gateway_state
                            .last_response_at
                            .insert(channel_key, current_time_ms());
                    }
                }
            }
            if !agent.complete_gateway_send_result(result.clone()).await {
                tracing::debug!(
                    correlation_id = %result.correlation_id,
                    "gateway send result had no waiting caller"
                );
            }
        }

        ClientMessage::GatewayHealthUpdate { update } => {
            if !gateway_connection_is_active(&gateway_connection_state) {
                framed
                    .send(DaemonMessage::Error {
                        message: "gateway health updates require a registered gateway connection"
                            .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
            let snapshot = update;
            let status_label = match snapshot.status {
                GatewayConnectionStatus::Connected => "connected",
                GatewayConnectionStatus::Disconnected => "disconnected",
                GatewayConnectionStatus::Error => "error",
            };
            tracing::debug!(
                platform = %snapshot.platform,
                status = status_label,
                last_success_at_ms = snapshot.last_success_at_ms,
                last_error_at_ms = snapshot.last_error_at_ms,
                consecutive_failure_count = snapshot.consecutive_failure_count,
                last_error = snapshot.last_error.as_deref().unwrap_or(""),
                current_backoff_secs = snapshot.current_backoff_secs,
                "gateway health update received"
            );
            if let Err(error) = persist_gateway_health_update(&agent, snapshot).await {
                framed
                    .send(DaemonMessage::Error {
                        message: format!("failed to persist gateway health: {error}"),
                    })
                    .await?;
                return Ok(DispatchOutcome::Continue);
            }
        }

        ClientMessage::SpawnSession {
            shell,
            cwd,
            env,
            workspace_id,
            cols,
            rows,
        } => {
            match manager
                .spawn(shell, cwd, workspace_id, env, cols, rows)
                .await
            {
                Ok((id, rx)) => {
                    attached_rxs.push((id, rx));
                    framed.send(DaemonMessage::SessionSpawned { id }).await?;
                }
                Err(e) => {
                    framed
                        .send(DaemonMessage::Error {
                            message: e.to_string(),
                        })
                        .await?;
                }
            }
        }

        ClientMessage::CloneSession {
            source_id,
            workspace_id,
            cols,
            rows,
            replay_scrollback,
            cwd,
        } => {
            match manager
                .clone_session(source_id, workspace_id, cols, rows, replay_scrollback, cwd)
                .await
            {
                Ok((id, rx, active_command)) => {
                    attached_rxs.push((id, rx));
                    framed
                        .send(DaemonMessage::SessionCloned {
                            source_id,
                            id,
                            active_command,
                        })
                        .await?;
                }
                Err(e) => {
                    framed
                        .send(DaemonMessage::Error {
                            message: e.to_string(),
                        })
                        .await?;
                }
            }
        }

        ClientMessage::AttachSession { id } => match manager.subscribe(id).await {
            Ok((rx, alive)) => {
                attached_rxs.push((id, rx));
                framed.send(DaemonMessage::SessionAttached { id }).await?;
                if !alive {
                    framed
                        .send(DaemonMessage::SessionExited {
                            id,
                            exit_code: None,
                        })
                        .await?;
                }
            }
            Err(e) => {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        },

        ClientMessage::DetachSession { id } => {
            attached_rxs.retain(|(sid, _)| *sid != id);
            framed.send(DaemonMessage::SessionDetached { id }).await?;
        }

        _ => unreachable!("message chunk should be exhaustive"),
    }
    Ok(DispatchOutcome::Continue)
}
