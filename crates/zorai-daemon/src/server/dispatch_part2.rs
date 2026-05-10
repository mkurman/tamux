use super::*;
use crate::agent::AgentEngine;
use crate::session_manager::SessionManager;
use anyhow::Result;
use std::sync::Arc;
use zorai_protocol::{ClientMessage, DaemonMessage};

pub(crate) async fn dispatch_part2(
    msg: &ClientMessage,
    agent: &Arc<AgentEngine>,
    framed: &mut ConnectionWriter,
    manager: &Arc<SessionManager>,
    attached_rxs: &mut Vec<(
        zorai_protocol::SessionId,
        tokio::sync::broadcast::Receiver<DaemonMessage>,
    )>,
) -> Result<DispatchOutcome>
{
    if !matches!(
        msg,
        ClientMessage::KillSession { .. }
            | ClientMessage::Input { .. }
            | ClientMessage::ExecuteManagedCommand { .. }
            | ClientMessage::ResolveApproval { .. }
            | ClientMessage::Resize { .. }
            | ClientMessage::ListSessions
            | ClientMessage::GetScrollback { .. }
            | ClientMessage::AnalyzeSession { .. }
            | ClientMessage::SearchHistory { .. }
            | ClientMessage::AppendCommandLog { .. }
            | ClientMessage::CompleteCommandLog { .. }
            | ClientMessage::QueryCommandLog { .. }
            | ClientMessage::ClearCommandLog
            | ClientMessage::CreateAgentThread { .. }
            | ClientMessage::DeleteAgentThread { .. }
            | ClientMessage::ListAgentThreads
            | ClientMessage::GetAgentThread { .. }
            | ClientMessage::AddAgentMessage { .. }
            | ClientMessage::DeleteAgentMessages { .. }
            | ClientMessage::RestoreAgentMessages { .. }
    ) {
        return Ok(DispatchOutcome::NotMatched);
    }
    let msg = msg.clone();

    match msg {
        ClientMessage::KillSession { id } => {
            attached_rxs.retain(|(sid, _)| *sid != id);
            let session_info = manager
                .list()
                .await
                .into_iter()
                .find(|session| session.id == id);
            let recent_output = manager.get_analysis_text(id, Some(40)).await.ok();
            match manager.kill(id).await {
                Ok(()) => {
                    let session_id_str = id.to_string();
                    let (session_summary, entities) = build_session_end_episode_payload(
                        &session_id_str,
                        session_info.as_ref(),
                        recent_output.as_deref(),
                    );
                    if let Err(e) = agent
                        .record_session_end_episode(&session_id_str, &session_summary, entities)
                        .await
                    {
                        tracing::warn!(session_id = %session_id_str, error = %e, "failed to record session end episode");
                    }
                    framed.send(DaemonMessage::SessionKilled { id }).await?;
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

        ClientMessage::Input { id, data } => {
            if let Err(e) = manager.write_input(id, &data).await {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        }

        ClientMessage::ExecuteManagedCommand {
            id,
            request,
            client_surface,
        } => {
            if matches!(client_surface, Some(zorai_protocol::ClientSurface::Tui)) {
                framed
                    .send(DaemonMessage::Error {
                        message: "managed terminal execution is reserved for Electron clients"
                            .to_string(),
                    })
                    .await?;
                return Ok(DispatchOutcome::Terminate);
            }
            match manager.execute_managed_command(id, request).await {
                Ok(message) => {
                    if let DaemonMessage::ApprovalRequired { approval, .. } = &message {
                        let pending = crate::agent::types::ToolPendingApproval {
                            approval_id: approval.approval_id.clone(),
                            execution_id: approval.execution_id.clone(),
                            command: approval.command.clone(),
                            rationale: approval.rationale.clone(),
                            risk_level: approval.risk_level.clone(),
                            blast_radius: approval.blast_radius.clone(),
                            reasons: approval.reasons.clone(),
                            session_id: Some(id.to_string()),
                        };
                        if let Err(error) = agent.record_operator_approval_requested(&pending).await
                        {
                            tracing::warn!(
                                approval_id = %approval.approval_id,
                                "failed to record operator approval request: {error}"
                            );
                        }
                        agent
                            .record_provenance_event(
                                "approval_requested",
                                "managed command requested approval",
                                serde_json::json!({
                                    "approval_id": approval.approval_id,
                                    "session_id": id.to_string(),
                                    "command": approval.command,
                                    "risk_level": approval.risk_level,
                                    "blast_radius": approval.blast_radius,
                                }),
                                None,
                                None,
                                None,
                                Some(approval.approval_id.as_str()),
                                None,
                            )
                            .await;
                    }
                    framed.send(message).await?;
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

        ClientMessage::ResolveApproval {
            id,
            approval_id,
            decision,
        } => match manager.resolve_approval(id, &approval_id, decision).await {
            Ok(messages) => {
                let _ = agent
                    .record_operator_approval_resolution(&approval_id, decision)
                    .await;
                let _ = agent
                    .handle_task_approval_resolution(&approval_id, decision)
                    .await;
                agent
                    .record_provenance_event(
                        match decision {
                            zorai_protocol::ApprovalDecision::ApproveOnce
                            | zorai_protocol::ApprovalDecision::ApproveSession => {
                                "approval_granted"
                            }
                            zorai_protocol::ApprovalDecision::Deny => "approval_denied",
                        },
                        "operator resolved approval request",
                        serde_json::json!({
                            "approval_id": approval_id,
                            "session_id": id.to_string(),
                            "decision": format!("{decision:?}").to_lowercase(),
                        }),
                        None,
                        None,
                        None,
                        Some(approval_id.as_str()),
                        None,
                    )
                    .await;
                for message in messages {
                    framed.send(message).await?;
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

        ClientMessage::Resize { id, cols, rows } => {
            if let Err(e) = manager.resize(id, cols, rows).await {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        }

        ClientMessage::ListSessions => {
            let sessions = manager.list().await;
            framed.send(DaemonMessage::SessionList { sessions }).await?;
        }

        ClientMessage::GetScrollback { id, max_lines } => {
            match manager.get_scrollback(id, max_lines).await {
                Ok(data) => {
                    let (data, truncated) = cap_scrollback_for_ipc(id, data);
                    if truncated {
                        tracing::warn!(session_id = %id, "truncated scrollback to fit IPC frame limit");
                    }
                    framed.send(DaemonMessage::Scrollback { id, data }).await?;
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

        ClientMessage::AnalyzeSession { id, max_lines } => {
            match manager.get_analysis_text(id, max_lines).await {
                Ok(text) => {
                    let (result, truncated) = cap_analysis_result_for_ipc(id, text);
                    if truncated {
                        tracing::warn!(session_id = %id, "truncated analysis result to fit IPC frame limit");
                    }
                    framed
                        .send(DaemonMessage::AnalysisResult { id, result })
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

        ClientMessage::SearchHistory { query, limit } => {
            match agent
                .search_history_semantic_first(&query, limit.unwrap_or(8).max(1))
                .await
            {
                Ok((summary, hits)) => {
                    let (summary, hits, truncated) =
                        cap_history_search_result_for_ipc(&query, summary, hits);
                    if truncated {
                        tracing::warn!(query = %query, "truncated history search result to fit IPC frame limit");
                    }
                    framed
                        .send(DaemonMessage::HistorySearchResult {
                            query,
                            summary,
                            hits,
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

        ClientMessage::AppendCommandLog { entry_json } => {
            match serde_json::from_str::<zorai_protocol::CommandLogEntry>(&entry_json) {
                Ok(entry) => match manager.append_command_log(&entry).await {
                    Ok(()) => {
                        framed.send(DaemonMessage::CommandLogAck).await?;
                    }
                    Err(e) => {
                        framed
                            .send(DaemonMessage::Error {
                                message: e.to_string(),
                            })
                            .await?;
                    }
                },
                Err(e) => {
                    framed
                        .send(DaemonMessage::Error {
                            message: format!("invalid command log payload: {e}"),
                        })
                        .await?;
                }
            }
        }

        ClientMessage::CompleteCommandLog {
            id,
            exit_code,
            duration_ms,
        } => match manager
            .complete_command_log(&id, exit_code, duration_ms)
            .await
        {
            Ok(()) => {
                framed.send(DaemonMessage::CommandLogAck).await?;
            }
            Err(e) => {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        },

        ClientMessage::QueryCommandLog {
            workspace_id,
            pane_id,
            limit,
        } => match manager
            .query_command_log(workspace_id.as_deref(), pane_id.as_deref(), limit)
            .await
        {
            Ok(entries) => {
                let entries_json = serde_json::to_string(&entries).unwrap_or_default();
                framed
                    .send(DaemonMessage::CommandLogEntries { entries_json })
                    .await?;
            }
            Err(e) => {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        },

        ClientMessage::ClearCommandLog => match manager.clear_command_log().await {
            Ok(()) => {
                framed.send(DaemonMessage::CommandLogAck).await?;
            }
            Err(e) => {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        },

        ClientMessage::CreateAgentThread { thread_json } => {
            match serde_json::from_str::<zorai_protocol::AgentDbThread>(&thread_json) {
                Ok(thread) => match manager.create_agent_thread(&thread).await {
                    Ok(()) => {
                        framed.send(DaemonMessage::AgentDbMessageAck).await?;
                    }
                    Err(e) => {
                        framed
                            .send(DaemonMessage::Error {
                                message: e.to_string(),
                            })
                            .await?;
                    }
                },
                Err(e) => {
                    framed
                        .send(DaemonMessage::Error {
                            message: format!("invalid agent thread payload: {e}"),
                        })
                        .await?;
                }
            }
        }

        ClientMessage::DeleteAgentThread { thread_id } => {
            match manager.delete_agent_thread(&thread_id).await {
                Ok(()) => {
                    framed.send(DaemonMessage::AgentDbMessageAck).await?;
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

        ClientMessage::ListAgentThreads => match manager
            .list_agent_threads_filtered(&crate::history::AgentThreadListQuery {
                limit: Some(AGENT_DB_THREAD_LIST_WINDOW),
                ..crate::history::AgentThreadListQuery::default()
            })
            .await
        {
            Ok(threads) => {
                let threads_json = serde_json::to_string(&threads).unwrap_or_default();
                framed
                    .send(DaemonMessage::AgentDbThreadList { threads_json })
                    .await?;
            }
            Err(e) => {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        },

        ClientMessage::GetAgentThread {
            thread_id,
            include_deleted,
        } => {
            let started = std::time::Instant::now();
            let (thread_result, messages_result) = if include_deleted {
                tokio::join!(
                    manager.get_agent_thread(&thread_id),
                    manager.list_agent_messages_with_deleted(
                        &thread_id,
                        Some(AGENT_DB_THREAD_DETAIL_MESSAGE_WINDOW),
                    ),
                )
            } else {
                tokio::join!(
                    manager.get_agent_thread(&thread_id),
                    manager.list_agent_messages(
                        &thread_id,
                        Some(AGENT_DB_THREAD_DETAIL_MESSAGE_WINDOW),
                    ),
                )
            };
            match thread_result {
                Ok(thread) => {
                    let messages = messages_result?;
                    let db_elapsed = started.elapsed().as_millis() as u64;
                    let message_count = messages.len();
                    let thread_id_for_log = thread_id.clone();
                    let cap_started = std::time::Instant::now();
                    let ((thread_json, messages_json), truncated) =
                        tokio::task::spawn_blocking(move || {
                            cap_agent_db_thread_detail_for_ipc(thread, messages)
                        })
                        .await
                        .unwrap_or_else(|_| ((String::new(), String::new()), true));
                    tracing::debug!(
                        thread_id = %thread_id_for_log,
                        message_count,
                        db_ms = db_elapsed,
                        cap_ms = cap_started.elapsed().as_millis() as u64,
                        truncated,
                        "GetAgentThread served"
                    );
                    if truncated {
                        tracing::warn!(
                            thread_id = %thread_id_for_log,
                            "truncated agent db thread detail to fit IPC frame limit"
                        );
                    }
                    framed
                        .send(DaemonMessage::AgentDbThreadDetail {
                            thread_json,
                            messages_json,
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

        ClientMessage::AddAgentMessage { message_json } => {
            match serde_json::from_str::<zorai_protocol::AgentDbMessage>(&message_json) {
                Ok(message) => match manager.add_agent_message(&message).await {
                    Ok(()) => {
                        framed.send(DaemonMessage::AgentDbMessageAck).await?;
                    }
                    Err(e) => {
                        framed
                            .send(DaemonMessage::Error {
                                message: e.to_string(),
                            })
                            .await?;
                    }
                },
                Err(e) => {
                    framed
                        .send(DaemonMessage::Error {
                            message: format!("invalid agent message payload: {e}"),
                        })
                        .await?;
                }
            }
        }

        ClientMessage::DeleteAgentMessages {
            thread_id,
            message_ids,
        } => match agent.delete_thread_messages(&thread_id, &message_ids).await {
            Ok(deleted) => {
                tracing::info!(
                    thread_id = %thread_id,
                    deleted,
                    "deleted agent messages"
                );
                framed.send(DaemonMessage::AgentDbMessageAck).await?;
            }
            Err(e) => {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        },

        ClientMessage::RestoreAgentMessages {
            thread_id,
            message_ids,
        } => match agent
            .restore_thread_messages(&thread_id, &message_ids)
            .await
        {
            Ok(restored) => {
                tracing::info!(
                    thread_id = %thread_id,
                    restored,
                    "restored soft-deleted agent messages"
                );
                framed.send(DaemonMessage::AgentDbMessageAck).await?;
            }
            Err(e) => {
                framed
                    .send(DaemonMessage::Error {
                        message: e.to_string(),
                    })
                    .await?;
            }
        },

        _ => unreachable!("message chunk should be exhaustive"),
    }
    Ok(DispatchOutcome::Continue)
}
