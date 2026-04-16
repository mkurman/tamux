use amux_protocol::{AmuxCodec, ClientMessage};
use anyhow::Result;
use futures::SinkExt;
use tokio_util::codec::Framed;

use super::emit_agent_event;
use crate::client::agent_protocol::AgentBridgeCommand;

pub(super) async fn handle_line<T>(framed: &mut Framed<T, AmuxCodec>, line: &str) -> Result<bool>
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let command: AgentBridgeCommand = match serde_json::from_str(line) {
        Ok(cmd) => cmd,
        Err(error) => {
            let err_json =
                serde_json::json!({"type":"error","message":format!("invalid command: {error}")});
            emit_agent_event(&err_json.to_string())?;
            return Ok(true);
        }
    };

    match command {
        AgentBridgeCommand::SendMessage {
            thread_id,
            content,
            session_id,
            context_messages,
        } => {
            let context_messages_json =
                context_messages.and_then(|msgs| serde_json::to_string(&msgs).ok());
            framed
                .send(ClientMessage::AgentSendMessage {
                    thread_id,
                    content,
                    session_id,
                    context_messages_json,
                    client_surface: Some(amux_protocol::ClientSurface::Electron),
                    target_agent_id: None,
                })
                .await?;
        }
        AgentBridgeCommand::InternalDelegate {
            thread_id,
            target_agent_id,
            content,
            session_id,
        } => {
            framed
                .send(ClientMessage::AgentInternalDelegate {
                    thread_id,
                    target_agent_id,
                    content,
                    session_id,
                    client_surface: Some(amux_protocol::ClientSurface::Electron),
                })
                .await?;
        }
        AgentBridgeCommand::ThreadParticipantCommand {
            thread_id,
            target_agent_id,
            action,
            instruction,
            session_id,
        } => {
            framed
                .send(ClientMessage::AgentThreadParticipantCommand {
                    thread_id,
                    target_agent_id,
                    action,
                    instruction,
                    session_id,
                    client_surface: Some(amux_protocol::ClientSurface::Electron),
                })
                .await?;
        }
        AgentBridgeCommand::StopStream { thread_id } => {
            framed
                .send(ClientMessage::AgentStopStream { thread_id })
                .await?;
        }
        AgentBridgeCommand::ListThreads => {
            framed.send(ClientMessage::AgentListThreads).await?;
        }
        AgentBridgeCommand::GetThread { thread_id } => {
            framed
                .send(ClientMessage::AgentGetThread {
                    thread_id,
                    message_limit: None,
                    message_offset: None,
                })
                .await?;
        }
        AgentBridgeCommand::DeleteThread { thread_id } => {
            framed
                .send(ClientMessage::AgentDeleteThread { thread_id })
                .await?;
        }
        AgentBridgeCommand::PinThreadMessageForCompaction {
            thread_id,
            message_id,
        } => {
            framed
                .send(ClientMessage::AgentPinThreadMessageForCompaction {
                    thread_id,
                    message_id,
                })
                .await?;
        }
        AgentBridgeCommand::UnpinThreadMessageForCompaction {
            thread_id,
            message_id,
        } => {
            framed
                .send(ClientMessage::AgentUnpinThreadMessageForCompaction {
                    thread_id,
                    message_id,
                })
                .await?;
        }
        AgentBridgeCommand::AddTask {
            title,
            description,
            priority,
            command,
            session_id,
            scheduled_at,
            dependencies,
        } => {
            framed
                .send(ClientMessage::AgentAddTask {
                    title,
                    description,
                    priority: priority.unwrap_or_else(|| "normal".into()),
                    command,
                    session_id,
                    scheduled_at,
                    dependencies,
                })
                .await?;
        }
        AgentBridgeCommand::CancelTask { task_id } => {
            framed
                .send(ClientMessage::AgentCancelTask { task_id })
                .await?;
        }
        AgentBridgeCommand::ListTasks => {
            framed.send(ClientMessage::AgentListTasks).await?;
        }
        AgentBridgeCommand::ListRuns => {
            framed.send(ClientMessage::AgentListRuns).await?;
        }
        AgentBridgeCommand::GetRun { run_id } => {
            framed.send(ClientMessage::AgentGetRun { run_id }).await?;
        }
        AgentBridgeCommand::StartGoalRun {
            goal,
            title,
            thread_id,
            session_id,
            priority,
            client_request_id,
            autonomy_level,
        } => {
            framed
                .send(ClientMessage::AgentStartGoalRun {
                    goal,
                    title,
                    thread_id,
                    session_id,
                    priority,
                    client_request_id,
                    autonomy_level,
                    client_surface: Some(amux_protocol::ClientSurface::Electron),
                })
                .await?;
        }
        AgentBridgeCommand::ListGoalRuns => {
            framed.send(ClientMessage::AgentListGoalRuns).await?;
        }
        AgentBridgeCommand::GetGoalRun { goal_run_id } => {
            framed
                .send(ClientMessage::AgentGetGoalRun { goal_run_id })
                .await?;
        }
        AgentBridgeCommand::ControlGoalRun {
            goal_run_id,
            action,
            step_index,
        } => {
            framed
                .send(ClientMessage::AgentControlGoalRun {
                    goal_run_id,
                    action,
                    step_index,
                })
                .await?;
        }
        AgentBridgeCommand::ListTodos => {
            framed.send(ClientMessage::AgentListTodos).await?;
        }
        AgentBridgeCommand::GetTodos { thread_id } => {
            framed
                .send(ClientMessage::AgentGetTodos { thread_id })
                .await?;
        }
        AgentBridgeCommand::GetWorkContext { thread_id } => {
            framed
                .send(ClientMessage::AgentGetWorkContext { thread_id })
                .await?;
        }
        AgentBridgeCommand::GetGitDiff {
            repo_path,
            file_path,
        } => {
            framed
                .send(ClientMessage::GetGitDiff {
                    repo_path,
                    file_path,
                })
                .await?;
        }
        AgentBridgeCommand::GetFilePreview { path, max_bytes } => {
            framed
                .send(ClientMessage::GetFilePreview { path, max_bytes })
                .await?;
        }
        AgentBridgeCommand::GetConfig => {
            framed.send(ClientMessage::AgentGetConfig).await?;
        }
        AgentBridgeCommand::SetConfigItem {
            key_path,
            value_json,
        } => {
            framed
                .send(ClientMessage::AgentSetConfigItem {
                    key_path,
                    value_json,
                })
                .await?;
        }
        AgentBridgeCommand::SetProviderModel { provider_id, model } => {
            framed
                .send(ClientMessage::AgentSetProviderModel { provider_id, model })
                .await?;
        }
        AgentBridgeCommand::SetTargetAgentProviderModel {
            target_agent_id,
            provider_id,
            model,
        } => {
            framed
                .send(ClientMessage::AgentSetTargetAgentProviderModel {
                    target_agent_id,
                    provider_id,
                    model,
                })
                .await?;
        }
        AgentBridgeCommand::HeartbeatGetItems => {
            framed.send(ClientMessage::AgentHeartbeatGetItems).await?;
        }
        AgentBridgeCommand::HeartbeatSetItems { items_json } => {
            framed
                .send(ClientMessage::AgentHeartbeatSetItems { items_json })
                .await?;
        }
        AgentBridgeCommand::ResolveTaskApproval {
            approval_id,
            decision,
        } => {
            framed
                .send(ClientMessage::AgentResolveTaskApproval {
                    approval_id,
                    decision,
                })
                .await?;
        }
        AgentBridgeCommand::ValidateProvider {
            provider_id,
            base_url,
            api_key,
            auth_source,
        } => {
            framed
                .send(ClientMessage::AgentValidateProvider {
                    provider_id,
                    base_url,
                    api_key,
                    auth_source,
                })
                .await?;
        }
        AgentBridgeCommand::LoginProvider {
            provider_id,
            api_key,
            base_url,
            auth_source,
        } => {
            framed
                .send(ClientMessage::AgentLoginProvider {
                    provider_id,
                    api_key,
                    base_url,
                    auth_source,
                })
                .await?;
        }
        AgentBridgeCommand::LogoutProvider { provider_id } => {
            framed
                .send(ClientMessage::AgentLogoutProvider { provider_id })
                .await?;
        }
        AgentBridgeCommand::GetProviderAuthStates => {
            framed
                .send(ClientMessage::AgentGetProviderAuthStates)
                .await?;
        }
        AgentBridgeCommand::GetOpenAICodexAuthStatus => {
            framed
                .send(ClientMessage::AgentGetOpenAICodexAuthStatus)
                .await?;
        }
        AgentBridgeCommand::LoginOpenAICodex => {
            framed.send(ClientMessage::AgentLoginOpenAICodex).await?;
        }
        AgentBridgeCommand::LogoutOpenAICodex => {
            framed.send(ClientMessage::AgentLogoutOpenAICodex).await?;
        }
        AgentBridgeCommand::SetSubAgent { sub_agent_json } => {
            framed
                .send(ClientMessage::AgentSetSubAgent { sub_agent_json })
                .await?;
        }
        AgentBridgeCommand::RemoveSubAgent { sub_agent_id } => {
            framed
                .send(ClientMessage::AgentRemoveSubAgent { sub_agent_id })
                .await?;
        }
        AgentBridgeCommand::ListSubAgents => {
            framed.send(ClientMessage::AgentListSubAgents).await?;
        }
        AgentBridgeCommand::GetConciergeConfig => {
            framed.send(ClientMessage::AgentGetConciergeConfig).await?;
        }
        AgentBridgeCommand::SetConciergeConfig { config_json } => {
            framed
                .send(ClientMessage::AgentSetConciergeConfig { config_json })
                .await?;
        }
        AgentBridgeCommand::DismissConciergeWelcome => {
            framed
                .send(ClientMessage::AgentDismissConciergeWelcome)
                .await?;
        }
        AgentBridgeCommand::RequestConciergeWelcome => {
            framed
                .send(ClientMessage::AgentRequestConciergeWelcome)
                .await?;
        }
        AgentBridgeCommand::AuditDismiss { entry_id } => {
            framed
                .send(ClientMessage::AuditDismiss { entry_id })
                .await?;
        }
        AgentBridgeCommand::QueryAudits {
            action_types,
            since,
            limit,
        } => {
            framed
                .send(ClientMessage::AuditQuery {
                    action_types,
                    since,
                    limit,
                })
                .await?;
        }
        AgentBridgeCommand::GetProvenanceReport { limit } => {
            framed
                .send(ClientMessage::AgentGetProvenanceReport { limit })
                .await?;
        }
        AgentBridgeCommand::GetMemoryProvenanceReport { target, limit } => {
            framed
                .send(ClientMessage::AgentGetMemoryProvenanceReport { target, limit })
                .await?;
        }
        AgentBridgeCommand::ConfirmMemoryProvenanceEntry { entry_id } => {
            framed
                .send(ClientMessage::AgentConfirmMemoryProvenanceEntry { entry_id })
                .await?;
        }
        AgentBridgeCommand::RetractMemoryProvenanceEntry { entry_id } => {
            framed
                .send(ClientMessage::AgentRetractMemoryProvenanceEntry { entry_id })
                .await?;
        }
        AgentBridgeCommand::GetCollaborationSessions { parent_task_id } => {
            framed
                .send(ClientMessage::AgentGetCollaborationSessions { parent_task_id })
                .await?;
        }
        AgentBridgeCommand::ListGeneratedTools => {
            framed.send(ClientMessage::AgentListGeneratedTools).await?;
        }
        AgentBridgeCommand::RunGeneratedTool {
            tool_name,
            args_json,
        } => {
            framed
                .send(ClientMessage::AgentRunGeneratedTool {
                    tool_name,
                    args_json,
                })
                .await?;
        }
        AgentBridgeCommand::PromoteGeneratedTool { tool_name } => {
            framed
                .send(ClientMessage::AgentPromoteGeneratedTool { tool_name })
                .await?;
        }
        AgentBridgeCommand::ActivateGeneratedTool { tool_name } => {
            framed
                .send(ClientMessage::AgentActivateGeneratedTool { tool_name })
                .await?;
        }
        AgentBridgeCommand::RetireGeneratedTool { tool_name } => {
            framed
                .send(ClientMessage::AgentRetireGeneratedTool { tool_name })
                .await?;
        }
        AgentBridgeCommand::VoteOnCollaborationDisagreement {
            parent_task_id,
            disagreement_id,
            task_id,
            position,
            confidence,
        } => {
            framed
                .send(ClientMessage::AgentVoteOnCollaborationDisagreement {
                    parent_task_id,
                    disagreement_id,
                    task_id,
                    position,
                    confidence,
                })
                .await?;
        }
        AgentBridgeCommand::GetStatistics { window } => {
            framed
                .send(ClientMessage::AgentStatisticsQuery { window })
                .await?;
        }
        AgentBridgeCommand::GetStatus => {
            framed.send(ClientMessage::AgentStatusQuery).await?;
        }
        AgentBridgeCommand::InspectPrompt { agent_id } => {
            framed
                .send(ClientMessage::AgentInspectPrompt { agent_id })
                .await?;
        }
        AgentBridgeCommand::SetTierOverride { tier } => {
            framed
                .send(ClientMessage::AgentSetTierOverride { tier })
                .await?;
        }
        AgentBridgeCommand::PluginList => {
            framed.send(ClientMessage::PluginList {}).await?;
        }
        AgentBridgeCommand::PluginGetDetail { name } => {
            framed.send(ClientMessage::PluginGet { name }).await?;
        }
        AgentBridgeCommand::PluginEnableCmd { name } => {
            framed.send(ClientMessage::PluginEnable { name }).await?;
        }
        AgentBridgeCommand::PluginDisableCmd { name } => {
            framed.send(ClientMessage::PluginDisable { name }).await?;
        }
        AgentBridgeCommand::PluginGetSettings { name } => {
            framed
                .send(ClientMessage::PluginGetSettings { name })
                .await?;
        }
        AgentBridgeCommand::PluginUpdateSettings {
            plugin_name,
            key,
            value,
            is_secret,
        } => {
            framed
                .send(ClientMessage::PluginUpdateSettings {
                    plugin_name,
                    key,
                    value,
                    is_secret,
                })
                .await?;
        }
        AgentBridgeCommand::PluginTestConnection { name } => {
            framed
                .send(ClientMessage::PluginTestConnection { name })
                .await?;
        }
        AgentBridgeCommand::PluginOAuthStart { name } => {
            framed
                .send(ClientMessage::PluginOAuthStart { name })
                .await?;
        }
        AgentBridgeCommand::WhatsAppLinkStart => {
            framed.send(ClientMessage::AgentWhatsAppLinkStart).await?;
        }
        AgentBridgeCommand::WhatsAppLinkStop => {
            framed.send(ClientMessage::AgentWhatsAppLinkStop).await?;
        }
        AgentBridgeCommand::WhatsAppLinkStatus => {
            framed.send(ClientMessage::AgentWhatsAppLinkStatus).await?;
        }
        AgentBridgeCommand::WhatsAppLinkSubscribe => {
            framed
                .send(ClientMessage::AgentWhatsAppLinkSubscribe)
                .await?;
        }
        AgentBridgeCommand::WhatsAppLinkUnsubscribe => {
            framed
                .send(ClientMessage::AgentWhatsAppLinkUnsubscribe)
                .await?;
        }
        AgentBridgeCommand::StartOperatorProfileSession { kind } => {
            framed
                .send(ClientMessage::AgentStartOperatorProfileSession { kind })
                .await?;
        }
        AgentBridgeCommand::NextOperatorProfileQuestion { session_id } => {
            framed
                .send(ClientMessage::AgentNextOperatorProfileQuestion { session_id })
                .await?;
        }
        AgentBridgeCommand::SubmitOperatorProfileAnswer {
            session_id,
            question_id,
            answer_json,
        } => {
            framed
                .send(ClientMessage::AgentSubmitOperatorProfileAnswer {
                    session_id,
                    question_id,
                    answer_json,
                })
                .await?;
        }
        AgentBridgeCommand::SkipOperatorProfileQuestion {
            session_id,
            question_id,
            reason,
        } => {
            framed
                .send(ClientMessage::AgentSkipOperatorProfileQuestion {
                    session_id,
                    question_id,
                    reason,
                })
                .await?;
        }
        AgentBridgeCommand::DeferOperatorProfileQuestion {
            session_id,
            question_id,
            defer_until_unix_ms,
        } => {
            framed
                .send(ClientMessage::AgentDeferOperatorProfileQuestion {
                    session_id,
                    question_id,
                    defer_until_unix_ms,
                })
                .await?;
        }
        AgentBridgeCommand::AnswerQuestion {
            question_id,
            answer,
        } => {
            framed
                .send(ClientMessage::AgentAnswerQuestion {
                    question_id,
                    answer,
                })
                .await?;
        }
        AgentBridgeCommand::GetOperatorProfileSummary => {
            framed
                .send(ClientMessage::AgentGetOperatorProfileSummary)
                .await?;
        }
        AgentBridgeCommand::SetOperatorProfileConsent {
            consent_key,
            granted,
        } => {
            framed
                .send(ClientMessage::AgentSetOperatorProfileConsent {
                    consent_key,
                    granted,
                })
                .await?;
        }
        AgentBridgeCommand::ExplainAction {
            action_id,
            step_index,
        } => {
            framed
                .send(ClientMessage::AgentExplainAction {
                    action_id,
                    step_index,
                })
                .await?;
        }
        AgentBridgeCommand::StartDivergentSession {
            problem_statement,
            thread_id,
            goal_run_id,
            custom_framings_json,
        } => {
            framed
                .send(ClientMessage::AgentStartDivergentSession {
                    problem_statement,
                    thread_id,
                    goal_run_id,
                    custom_framings_json,
                })
                .await?;
        }
        AgentBridgeCommand::GetDivergentSession { session_id } => {
            framed
                .send(ClientMessage::AgentGetDivergentSession { session_id })
                .await?;
        }
        AgentBridgeCommand::Shutdown => {
            framed.send(ClientMessage::AgentUnsubscribe).await?;
            return Ok(false);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::handle_line;
    use crate::client::agent_protocol::AgentBridgeCommand;
    use amux_protocol::{AmuxCodec, ClientMessage, DaemonCodec};
    use bytes::BytesMut;
    use futures::{SinkExt, StreamExt};
    use tokio_util::codec::Framed;
    use tokio_util::codec::{Decoder, Encoder};

    async fn emitted_client_message(line: &str) -> ClientMessage {
        let (client_side, server_side) = tokio::io::duplex(1024);
        let mut bridge = Framed::new(client_side, AmuxCodec);
        let mut daemon = Framed::new(server_side, DaemonCodec);

        let (handle_result, message_result) =
            tokio::join!(handle_line(&mut bridge, line), daemon.next());

        assert!(handle_result.expect("bridge command should be handled"));

        message_result
            .expect("expected outbound client message")
            .expect("codec should decode client message")
    }

    async fn assert_emitted_client_message(line: &str, expected: ClientMessage) {
        let message = emitted_client_message(line).await;
        assert_eq!(
            std::mem::discriminant(&message),
            std::mem::discriminant(&expected)
        );
    }

    #[tokio::test]
    async fn openai_codex_auth_status_command_maps_to_client_message() {
        assert_emitted_client_message(
            r#"{"type":"openai-codex-auth-status"}"#,
            ClientMessage::AgentGetOpenAICodexAuthStatus,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_codex_auth_login_command_maps_to_client_message() {
        assert_emitted_client_message(
            r#"{"type":"openai-codex-auth-login"}"#,
            ClientMessage::AgentLoginOpenAICodex,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_codex_auth_logout_command_maps_to_client_message() {
        assert_emitted_client_message(
            r#"{"type":"openai-codex-auth-logout"}"#,
            ClientMessage::AgentLogoutOpenAICodex,
        )
        .await;
    }

    #[test]
    fn send_message_command_deserializes() {
        let command: AgentBridgeCommand = serde_json::from_str(
            r#"{"type":"send-message","thread_id":"thread-1","content":"hello","session_id":null,"context_messages":null}"#,
        )
        .expect("send-message command should deserialize");

        match command {
            AgentBridgeCommand::SendMessage {
                thread_id,
                content,
                session_id,
                context_messages,
            } => {
                assert_eq!(thread_id.as_deref(), Some("thread-1"));
                assert_eq!(content, "hello");
                assert!(session_id.is_none());
                assert!(context_messages.is_none());
            }
            other => panic!("expected SendMessage command, got {other:?}"),
        }
    }

    #[test]
    fn internal_delegate_command_deserializes() {
        let command: AgentBridgeCommand = serde_json::from_str(
            r#"{"type":"internal-delegate","thread_id":"thread-1","target_agent_id":"weles","content":"verify this","session_id":null}"#,
        )
        .expect("internal-delegate command should deserialize");

        match command {
            AgentBridgeCommand::InternalDelegate {
                thread_id,
                target_agent_id,
                content,
                session_id,
            } => {
                assert_eq!(thread_id.as_deref(), Some("thread-1"));
                assert_eq!(target_agent_id, "weles");
                assert_eq!(content, "verify this");
                assert!(session_id.is_none());
            }
            other => panic!("expected InternalDelegate command, got {other:?}"),
        }
    }

    #[test]
    fn thread_participant_command_deserializes() {
        let command: AgentBridgeCommand = serde_json::from_str(
            r#"{"type":"thread-participant-command","thread_id":"thread-1","target_agent_id":"weles","action":"upsert","instruction":"verify claims","session_id":null}"#,
        )
        .expect("thread-participant-command should deserialize");

        match command {
            AgentBridgeCommand::ThreadParticipantCommand {
                thread_id,
                target_agent_id,
                action,
                instruction,
                session_id,
            } => {
                assert_eq!(thread_id, "thread-1");
                assert_eq!(target_agent_id, "weles");
                assert_eq!(action, "upsert");
                assert_eq!(instruction.as_deref(), Some("verify claims"));
                assert!(session_id.is_none());
            }
            other => panic!("expected ThreadParticipantCommand, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_message_command_emits_agent_send_message_frame() {
        let message = emitted_client_message(
            r#"{"type":"send-message","thread_id":"thread-1","content":"hello","session_id":null,"context_messages":null}"#,
        )
        .await;

        match message {
            ClientMessage::AgentSendMessage {
                thread_id,
                content,
                session_id,
                context_messages_json,
                client_surface,
                target_agent_id,
            } => {
                assert_eq!(thread_id.as_deref(), Some("thread-1"));
                assert_eq!(content, "hello");
                assert!(session_id.is_none());
                assert!(context_messages_json.is_none());
                assert_eq!(client_surface, Some(amux_protocol::ClientSurface::Electron));
                assert!(target_agent_id.is_none());
            }
            other => panic!("expected AgentSendMessage, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn internal_delegate_command_emits_internal_delegate_frame() {
        let message = emitted_client_message(
            r#"{"type":"internal-delegate","thread_id":"thread-1","target_agent_id":"weles","content":"verify this","session_id":null}"#,
        )
        .await;

        match message {
            ClientMessage::AgentInternalDelegate {
                thread_id,
                target_agent_id,
                content,
                session_id,
                client_surface,
            } => {
                assert_eq!(thread_id.as_deref(), Some("thread-1"));
                assert_eq!(target_agent_id, "weles");
                assert_eq!(content, "verify this");
                assert!(session_id.is_none());
                assert_eq!(client_surface, Some(amux_protocol::ClientSurface::Electron));
            }
            other => panic!("expected AgentInternalDelegate, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn participant_command_emits_thread_participant_frame() {
        let message = emitted_client_message(
            r#"{"type":"thread-participant-command","thread_id":"thread-1","target_agent_id":"weles","action":"upsert","instruction":"verify claims","session_id":null}"#,
        )
        .await;

        match message {
            ClientMessage::AgentThreadParticipantCommand {
                thread_id,
                target_agent_id,
                action,
                instruction,
                session_id,
                client_surface,
            } => {
                assert_eq!(thread_id, "thread-1");
                assert_eq!(target_agent_id, "weles");
                assert_eq!(action, "upsert");
                assert_eq!(instruction.as_deref(), Some("verify claims"));
                assert!(session_id.is_none());
                assert_eq!(client_surface, Some(amux_protocol::ClientSurface::Electron));
            }
            other => panic!("expected AgentThreadParticipantCommand, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn direct_agent_send_message_frame_decodes_cleanly() {
        let message = ClientMessage::AgentSendMessage {
            thread_id: Some("thread-1".to_string()),
            content: "hello".to_string(),
            session_id: None,
            context_messages_json: None,
            client_surface: Some(amux_protocol::ClientSurface::Electron),
            target_agent_id: None,
        };

        let mut encoded = BytesMut::new();
        AmuxCodec
            .encode(message.clone(), &mut encoded)
            .expect("codec should encode AgentSendMessage in memory");
        let decoded = DaemonCodec
            .decode(&mut encoded)
            .expect("codec decode should not error in memory")
            .expect("codec should decode AgentSendMessage from memory buffer");
        assert!(matches!(decoded, ClientMessage::AgentSendMessage { .. }));

        let (client_side, server_side) = tokio::io::duplex(4096);
        let mut bridge = Framed::new(client_side, AmuxCodec);
        let mut daemon = Framed::new(server_side, DaemonCodec);

        bridge
            .send(message)
            .await
            .expect("direct send should succeed");

        match daemon.next().await {
            Some(Ok(ClientMessage::AgentSendMessage { content, .. })) => {
                assert_eq!(content, "hello");
            }
            other => panic!("expected direct AgentSendMessage decode, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn direct_agent_get_thread_frame_decodes_cleanly() {
        let (client_side, server_side) = tokio::io::duplex(4096);
        let mut bridge = Framed::new(client_side, AmuxCodec);
        let mut daemon = Framed::new(server_side, DaemonCodec);

        bridge
            .send(ClientMessage::AgentGetThread {
                thread_id: "thread-1".to_string(),
                message_limit: None,
                message_offset: None,
            })
            .await
            .expect("direct send should succeed");

        match daemon.next().await {
            Some(Ok(ClientMessage::AgentGetThread {
                thread_id,
                message_limit,
                message_offset,
            })) => {
                assert_eq!(thread_id, "thread-1");
                assert!(message_limit.is_none());
                assert!(message_offset.is_none());
            }
            other => panic!("expected direct AgentGetThread decode, got {other:?}"),
        }
    }
}
