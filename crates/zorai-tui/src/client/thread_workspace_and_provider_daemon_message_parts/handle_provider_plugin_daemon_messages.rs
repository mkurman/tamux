use crate::client::OpenAICodexAuthStatusVm;
use crate::client::{ClientEvent, DaemonClient};
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::warn;
use zorai_protocol::DaemonMessage;
impl DaemonClient {
    pub(crate) async fn handle_provider_plugin_daemon_messages(
        message: DaemonMessage,
        event_tx: &mpsc::Sender<ClientEvent>,
    ) {
        match message {
            DaemonMessage::AgentProviderAuthStates { states_json } => {
                let states: Vec<serde_json::Value> =
                    serde_json::from_str(&states_json).unwrap_or_default();
                let entries = states
                    .iter()
                    .filter_map(|v| {
                        Some(crate::state::ProviderAuthEntry {
                            provider_id: v.get("provider_id")?.as_str()?.to_string(),
                            provider_name: v.get("provider_name")?.as_str()?.to_string(),
                            authenticated: v.get("authenticated")?.as_bool()?,
                            auth_source: v
                                .get("auth_source")
                                .and_then(|s| s.as_str())
                                .unwrap_or("api_key")
                                .to_string(),
                            model: v
                                .get("model")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string(),
                        })
                    })
                    .collect();
                let _ = event_tx
                    .send(ClientEvent::ProviderAuthStates(entries))
                    .await;
            }
            DaemonMessage::AgentProviderCatalog { catalog_json } => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&catalog_json) {
                    if let Some(diagnostics) = value
                        .get("custom_provider_report")
                        .and_then(|report| report.get("diagnostics"))
                        .and_then(|diagnostics| diagnostics.as_array())
                    {
                        for diagnostic in diagnostics {
                            warn!("Custom provider configuration issue: {}", diagnostic);
                        }
                    }
                }
            }
            DaemonMessage::AgentOpenAICodexAuthStatus { status_json } => {
                match serde_json::from_str::<OpenAICodexAuthStatusVm>(&status_json) {
                    Ok(status) => {
                        let _ = event_tx
                            .send(ClientEvent::OpenAICodexAuthStatus(status))
                            .await;
                    }
                    Err(err) => warn!("Failed to parse OpenAI Codex auth status: {}", err),
                }
            }
            DaemonMessage::AgentOpenAICodexAuthLoginResult { result_json } => {
                match serde_json::from_str::<OpenAICodexAuthStatusVm>(&result_json) {
                    Ok(status) => {
                        let _ = event_tx
                            .send(ClientEvent::OpenAICodexAuthLoginResult(status))
                            .await;
                    }
                    Err(err) => warn!("Failed to parse OpenAI Codex auth login result: {}", err),
                }
            }
            DaemonMessage::AgentOpenAICodexAuthLogoutResult { ok, error } => {
                let _ = event_tx
                    .send(ClientEvent::OpenAICodexAuthLogoutResult { ok, error })
                    .await;
            }
            DaemonMessage::AgentProviderValidation {
                operation_id: _,
                provider_id,
                valid,
                error,
                ..
            } => {
                let _ = event_tx
                    .send(ClientEvent::ProviderValidation {
                        provider_id,
                        valid,
                        error,
                    })
                    .await;
            }
            DaemonMessage::AgentSubAgentList { sub_agents_json } => {
                let items: Vec<serde_json::Value> =
                    serde_json::from_str(&sub_agents_json).unwrap_or_default();
                let entries = items
                    .iter()
                    .filter_map(|v| {
                        Some(crate::state::SubAgentEntry {
                            id: v.get("id")?.as_str()?.to_string(),
                            name: v.get("name")?.as_str()?.to_string(),
                            provider: v.get("provider")?.as_str()?.to_string(),
                            model: v.get("model")?.as_str()?.to_string(),
                            role: v.get("role").and_then(|s| s.as_str()).map(String::from),
                            enabled: v.get("enabled").and_then(|b| b.as_bool()).unwrap_or(true),
                            builtin: v.get("builtin").and_then(|b| b.as_bool()).unwrap_or(false),
                            immutable_identity: v
                                .get("immutable_identity")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false),
                            disable_allowed: v
                                .get("disable_allowed")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(true),
                            delete_allowed: v
                                .get("delete_allowed")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(true),
                            protected_reason: v
                                .get("protected_reason")
                                .and_then(|s| s.as_str())
                                .map(String::from),
                            reasoning_effort: v
                                .get("reasoning_effort")
                                .and_then(|s| s.as_str())
                                .map(String::from),
                            openrouter_provider_order:
                                crate::state::subagents::openrouter_provider_list_from_json(
                                    v,
                                    "openrouter_provider_order",
                                ),
                            openrouter_provider_ignore:
                                crate::state::subagents::openrouter_provider_list_from_json(
                                    v,
                                    "openrouter_provider_ignore",
                                ),
                            openrouter_allow_fallbacks: v
                                .get("openrouter_allow_fallbacks")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(true),
                            raw_json: Some(v.clone()),
                        })
                    })
                    .collect();
                let _ = event_tx.send(ClientEvent::SubAgentList(entries)).await;
            }
            DaemonMessage::AgentSubAgentUpdated { sub_agent_json } => {
                let v: serde_json::Value =
                    serde_json::from_str(&sub_agent_json).unwrap_or_default();
                let entry = crate::state::SubAgentEntry {
                    id: v
                        .get("id")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    name: v
                        .get("name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    provider: v
                        .get("provider")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    model: v
                        .get("model")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string(),
                    role: v.get("role").and_then(|s| s.as_str()).map(String::from),
                    enabled: v.get("enabled").and_then(|b| b.as_bool()).unwrap_or(true),
                    builtin: v.get("builtin").and_then(|b| b.as_bool()).unwrap_or(false),
                    immutable_identity: v
                        .get("immutable_identity")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(false),
                    disable_allowed: v
                        .get("disable_allowed")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(true),
                    delete_allowed: v
                        .get("delete_allowed")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(true),
                    protected_reason: v
                        .get("protected_reason")
                        .and_then(|s| s.as_str())
                        .map(String::from),
                    reasoning_effort: v
                        .get("reasoning_effort")
                        .and_then(|s| s.as_str())
                        .map(String::from),
                    openrouter_provider_order:
                        crate::state::subagents::openrouter_provider_list_from_json(
                            &v,
                            "openrouter_provider_order",
                        ),
                    openrouter_provider_ignore:
                        crate::state::subagents::openrouter_provider_list_from_json(
                            &v,
                            "openrouter_provider_ignore",
                        ),
                    openrouter_allow_fallbacks: v
                        .get("openrouter_allow_fallbacks")
                        .and_then(|b| b.as_bool())
                        .unwrap_or(true),
                    raw_json: Some(v),
                };
                let _ = event_tx.send(ClientEvent::SubAgentUpdated(entry)).await;
            }
            DaemonMessage::AgentSubAgentRemoved { sub_agent_id } => {
                let _ = event_tx
                    .send(ClientEvent::SubAgentRemoved { sub_agent_id })
                    .await;
            }
            DaemonMessage::AgentConciergeConfig { config_json } => {
                match serde_json::from_str::<Value>(&config_json) {
                    Ok(raw) => {
                        let _ = event_tx.send(ClientEvent::ConciergeConfig(raw)).await;
                    }
                    Err(err) => warn!("Failed to parse concierge config response: {}", err),
                }
            }
            DaemonMessage::PluginListResult { plugins } => {
                let _ = event_tx.send(ClientEvent::PluginList(plugins)).await;
            }
            DaemonMessage::PluginGetResult {
                plugin,
                settings_schema,
            } => {
                let _ = event_tx
                    .send(ClientEvent::PluginGet {
                        plugin,
                        settings_schema,
                    })
                    .await;
            }
            DaemonMessage::PluginSettingsResult {
                plugin_name,
                settings,
            } => {
                let _ = event_tx
                    .send(ClientEvent::PluginSettings {
                        plugin_name,
                        settings,
                    })
                    .await;
            }
            DaemonMessage::PluginTestConnectionResult {
                plugin_name,
                success,
                message,
            } => {
                let _ = event_tx
                    .send(ClientEvent::PluginTestConnection {
                        plugin_name,
                        success,
                        message,
                    })
                    .await;
            }
            DaemonMessage::PluginActionResult { success, message } => {
                let _ = event_tx
                    .send(ClientEvent::PluginAction { success, message })
                    .await;
            }
            DaemonMessage::PluginCommandsResult { commands } => {
                let _ = event_tx.send(ClientEvent::PluginCommands(commands)).await;
            }
            DaemonMessage::PluginOAuthUrl { name, url } => {
                let _ = event_tx
                    .send(ClientEvent::PluginOAuthUrl { name, url })
                    .await;
            }
            DaemonMessage::PluginOAuthComplete {
                operation_id: _,
                name,
                success,
                error,
            } => {
                let _ = event_tx
                    .send(ClientEvent::PluginOAuthComplete {
                        name,
                        success,
                        error,
                    })
                    .await;
            }
            _ => unreachable!("provider/plugin daemon message dispatch should be exhaustive"),
        }
    }
}
