if matches!(
        &msg,
        ClientMessage::AgentSetOperatorProfileConsent{ .. } |
        ClientMessage::AgentGetOperatorModel |
        ClientMessage::AgentResetOperatorModel |
        ClientMessage::AgentGetCausalTraceReport{ .. } |
        ClientMessage::AgentGetCounterfactualReport{ .. } |
        ClientMessage::AgentGetMemoryProvenanceReport{ .. } |
        ClientMessage::AgentGetProvenanceReport{ .. } |
        ClientMessage::AgentGenerateSoc2Artifact{ .. } |
        ClientMessage::AgentExecuteMemoryTool{ .. } |
        ClientMessage::AgentGetCollaborationSessions{ .. } |
        ClientMessage::AgentGetDivergentSession{ .. } |
        ClientMessage::AgentListGeneratedTools |
        ClientMessage::AgentSynthesizeTool{ .. } |
        ClientMessage::AgentRunGeneratedTool{ .. } |
        ClientMessage::AgentPromoteGeneratedTool{ .. } |
        ClientMessage::AgentActivateGeneratedTool{ .. } |
        ClientMessage::AgentRetireGeneratedTool{ .. } |
        ClientMessage::AgentGetProviderAuthStates |
        ClientMessage::AgentLoginProvider{ .. } |
        ClientMessage::AgentLogoutProvider{ .. } |
        ClientMessage::AgentGetOpenAICodexAuthStatus |
        ClientMessage::AgentLoginOpenAICodex |
        ClientMessage::AgentLogoutOpenAICodex
    ) {
        match msg {
                ClientMessage::AgentSetOperatorProfileConsent {
                    consent_key,
                    granted,
                } => {
                    match agent
                        .set_operator_profile_consent(&consent_key, granted)
                        .await
                    {
                        Ok(updated_fields) => {
                            framed
                                .send(DaemonMessage::AgentOperatorProfileSessionCompleted {
                                    session_id: "consent-update".to_string(),
                                    updated_fields,
                                })
                                .await
                                .ok();
                        }
                        Err(error) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!(
                                        "failed to set operator profile consent: {error}"
                                    ),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentGetOperatorModel => match agent.operator_model_json().await {
                    Ok(model_json) => {
                        framed
                            .send(DaemonMessage::AgentOperatorModel { model_json })
                            .await
                            .ok();
                    }
                    Err(e) => {
                        framed
                            .send(DaemonMessage::AgentError {
                                message: format!("failed to load operator model: {e}"),
                            })
                            .await
                            .ok();
                    }
                },

                ClientMessage::AgentResetOperatorModel => {
                    match agent.reset_operator_model().await {
                        Ok(()) => {
                            framed
                                .send(DaemonMessage::AgentOperatorModelReset { ok: true })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to reset operator model: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentGetCausalTraceReport { option_type, limit } => {
                    match agent
                        .causal_trace_report(&option_type, limit.unwrap_or(20))
                        .await
                    {
                        Ok(report) => {
                            let report_json =
                                serde_json::to_string(&report).unwrap_or_else(|_| "{}".into());
                            framed
                                .send(DaemonMessage::AgentCausalTraceReport { report_json })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to build causal trace report: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentGetCounterfactualReport {
                    option_type,
                    command_family,
                    limit,
                } => match agent
                    .counterfactual_report(&option_type, &command_family, limit.unwrap_or(20))
                    .await
                {
                    Ok(report) => {
                        let report_json =
                            serde_json::to_string(&report).unwrap_or_else(|_| "{}".into());
                        framed
                            .send(DaemonMessage::AgentCounterfactualReport { report_json })
                            .await
                            .ok();
                    }
                    Err(e) => {
                        framed
                            .send(DaemonMessage::AgentError {
                                message: format!("failed to build counterfactual report: {e}"),
                            })
                            .await
                            .ok();
                    }
                },

                ClientMessage::AgentGetMemoryProvenanceReport { target, limit } => {
                    match agent
                        .history
                        .memory_provenance_report(target.as_deref(), limit.unwrap_or(25) as usize)
                        .await
                    {
                        Ok(report) => {
                            let report_json =
                                serde_json::to_string(&report).unwrap_or_else(|_| "{}".into());
                            framed
                                .send(DaemonMessage::AgentMemoryProvenanceReport { report_json })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!(
                                        "failed to build memory provenance report: {e}"
                                    ),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentConfirmMemoryProvenanceEntry { entry_id } => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    match agent
                        .history
                        .confirm_memory_provenance_entry(&entry_id, now_ms)
                        .await
                    {
                        Ok(true) => {
                            framed
                                .send(DaemonMessage::AgentMemoryProvenanceConfirmed {
                                    entry_id,
                                    confirmed_at: now_ms,
                                })
                                .await
                                .ok();
                        }
                        Ok(false) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: "memory provenance entry not found".to_string(),
                                })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to confirm memory provenance entry: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentRetractMemoryProvenanceEntry { entry_id } => {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    match agent
                        .history
                        .retract_memory_provenance_entry(&entry_id, now_ms)
                        .await
                    {
                        Ok(true) => {
                            framed
                                .send(DaemonMessage::AgentMemoryProvenanceRetracted {
                                    entry_id,
                                    retracted_at: now_ms,
                                })
                                .await
                                .ok();
                        }
                        Ok(false) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: "memory provenance entry not found".to_string(),
                                })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to retract memory provenance entry: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentGetProvenanceReport { limit } => {
                    match agent
                        .provenance_report_json(limit.unwrap_or(50) as usize)
                        .await
                    {
                        Ok(report_json) => {
                            framed
                                .send(DaemonMessage::AgentProvenanceReport { report_json })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to build provenance report: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentSemanticQuery { args_json } => {
                    let args = serde_json::from_str::<serde_json::Value>(&args_json)
                        .unwrap_or_else(|_| serde_json::json!({}));
                    match agent.semantic_query_text(&args).await {
                        Ok(content) => {
                            framed
                                .send(DaemonMessage::AgentSemanticQueryResult { content })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to execute semantic query: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentExecuteMemoryTool { tool_name, args_json } => {
                    match serde_json::from_str::<serde_json::Value>(&args_json) {
                        Ok(args) => {
                            match crate::agent::tool_executor::execute_memory_tool_for_mcp(
                                &tool_name,
                                &args,
                                &agent,
                                &agent.data_dir,
                            )
                            .await
                            {
                                Ok(content) => {
                                    framed
                                        .send(DaemonMessage::AgentMemoryToolResult { content })
                                        .await
                                        .ok();
                                }
                                Err(error) => {
                                    framed
                                        .send(DaemonMessage::AgentError {
                                            message: format!(
                                                "failed to execute memory tool `{tool_name}`: {error}"
                                            ),
                                        })
                                        .await
                                        .ok();
                                }
                            }
                        }
                        Err(error) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!(
                                        "invalid memory tool arguments for `{tool_name}`: {error}"
                                    ),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentGenerateSoc2Artifact { period_days } => {
                    match agent
                        .generate_soc2_artifact(period_days.unwrap_or(30))
                        .await
                    {
                        Ok(artifact_path) => {
                            framed
                                .send(DaemonMessage::AgentSoc2Artifact { artifact_path })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to generate SOC2 artifact: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentGetCollaborationSessions { parent_task_id } => {
                    match agent
                        .collaboration_sessions_json(parent_task_id.as_deref())
                        .await
                    {
                        Ok(sessions) => {
                            framed
                                .send(DaemonMessage::AgentCollaborationSessions {
                                    sessions_json: sessions.to_string(),
                                })
                                .await
                                .ok();
                        }
                        Err(error) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!(
                                        "failed to read collaboration sessions: {error}"
                                    ),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentVoteOnCollaborationDisagreement {
                    parent_task_id,
                    disagreement_id,
                    task_id,
                    position,
                    confidence,
                } => {
                    match agent
                        .vote_on_collaboration_disagreement(
                            &parent_task_id,
                            &disagreement_id,
                            &task_id,
                            &position,
                            confidence,
                        )
                        .await
                    {
                        Ok(report) => {
                            framed
                                .send(DaemonMessage::AgentCollaborationVoteResult {
                                    report_json: report.to_string(),
                                })
                                .await
                                .ok();
                        }
                        Err(error) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!(
                                        "failed to vote on collaboration disagreement: {error}"
                                    ),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentGetDivergentSession { session_id } => {
                    match agent.get_divergent_session(&session_id).await {
                        Ok(session_payload) => {
                            framed
                                .send(DaemonMessage::AgentDivergentSession {
                                    session_json: session_payload.to_string(),
                                })
                                .await
                                .ok();
                        }
                        Err(error) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!(
                                        "failed to read divergent session {session_id}: {error}"
                                    ),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentListGeneratedTools => {
                    match agent.list_generated_tools_json().await {
                        Ok(tools_json) => {
                            framed
                                .send(DaemonMessage::AgentGeneratedTools { tools_json })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to list generated tools: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentSynthesizeTool { request_json } => {
                    if !background_daemon_pending.has_capacity(BackgroundSubsystem::AgentWork) {
                        background_daemon_pending.note_rejection(BackgroundSubsystem::AgentWork);
                        framed
                            .send(DaemonMessage::Error {
                                message: "agent_work background queue is full".to_string(),
                            })
                            .await
                            .ok();
                        continue;
                    }

                    let operation = operation_registry().accept_operation(
                        OPERATION_KIND_SYNTHESIZE_TOOL,
                        Some(synthesize_tool_dedup_key(&agent, &request_json)),
                    );

                    framed
                        .send(DaemonMessage::OperationAccepted {
                            operation_id: operation.operation_id.clone(),
                            kind: operation.kind.clone(),
                            dedup: operation.dedup.clone(),
                            revision: operation.revision,
                        })
                        .await
                        .ok();

                    let operation_id = Some(operation.operation_id.clone());
                    let result_operation_id = operation_id.clone();
                    let agent = agent.clone();
                    let background_daemon_tx =
                        background_daemon_queues.sender(BackgroundSubsystem::AgentWork);
                    spawn_background_operation(
                        BackgroundSubsystem::AgentWork,
                        operation_id,
                        background_daemon_tx,
                        &mut background_daemon_pending,
                        async move {
                            #[cfg(test)]
                            if let Some(delay) = agent.take_test_synthesize_tool_delay().await {
                                tokio::time::sleep(delay).await;
                                return BackgroundOperationOutput::Failed(DaemonMessage::AgentError {
                                    message: "failed to synthesize generated tool: timed out"
                                        .to_string(),
                                });
                            }

                            match agent.synthesize_tool_json(&request_json).await {
                                Ok(result_json) => {
                                    BackgroundOperationOutput::Completed(
                                        DaemonMessage::AgentGeneratedToolResult {
                                            operation_id: result_operation_id.clone(),
                                            tool_name: None,
                                            result_json,
                                        },
                                    )
                                }
                                Err(e) => BackgroundOperationOutput::Failed(
                                    DaemonMessage::AgentError {
                                        message: format!(
                                            "failed to synthesize generated tool: {e}"
                                        ),
                                    },
                                ),
                            }
                        },
                    );
                }

                ClientMessage::AgentRunGeneratedTool {
                    tool_name,
                    args_json,
                } => match agent
                    .run_generated_tool_json(&tool_name, &args_json, None)
                    .await
                {
                    Ok(result_json) => {
                        framed
                            .send(DaemonMessage::AgentGeneratedToolResult {
                                operation_id: None,
                                tool_name: Some(tool_name),
                                result_json,
                            })
                            .await
                            .ok();
                    }
                    Err(e) => {
                        framed
                            .send(DaemonMessage::AgentError {
                                message: format!("failed to run generated tool: {e}"),
                            })
                            .await
                            .ok();
                    }
                },

                ClientMessage::AgentPromoteGeneratedTool { tool_name } => {
                    match agent.promote_generated_tool_json(&tool_name).await {
                        Ok(result_json) => {
                            framed
                                .send(DaemonMessage::AgentGeneratedToolResult {
                                    operation_id: None,
                                    tool_name: Some(tool_name),
                                    result_json,
                                })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to promote generated tool: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                ClientMessage::AgentActivateGeneratedTool { tool_name } => {
                    match agent.activate_generated_tool_json(&tool_name).await {
                        Ok(result_json) => {
                            framed
                                .send(DaemonMessage::AgentGeneratedToolResult {
                                    operation_id: None,
                                    tool_name: Some(tool_name),
                                    result_json,
                                })
                                .await
                                .ok();
                        }
                        Err(e) => {
                            framed
                                .send(DaemonMessage::AgentError {
                                    message: format!("failed to activate generated tool: {e}"),
                                })
                                .await
                                .ok();
                        }
                    }
                }

                    ClientMessage::AgentRetireGeneratedTool { tool_name } => {
                        match agent.retire_generated_tool_json(&tool_name).await {
                            Ok(result_json) => {
                                framed
                                    .send(DaemonMessage::AgentGeneratedToolResult {
                                        operation_id: None,
                                        tool_name: Some(tool_name),
                                        result_json,
                                    })
                                    .await
                                    .ok();
                            }
                            Err(e) => {
                                framed
                                    .send(DaemonMessage::AgentError {
                                        message: format!("failed to retire generated tool: {e}"),
                                    })
                                    .await
                                    .ok();
                            }
                        }
                    }

                ClientMessage::AgentGetProviderAuthStates => {
                    let states = agent.get_provider_auth_states().await;
                    let json = serde_json::to_string(&states).unwrap_or_default();
                    framed
                        .send(DaemonMessage::AgentProviderAuthStates { states_json: json })
                        .await?;
                }

                ClientMessage::AgentLoginProvider {
                    provider_id,
                    api_key,
                    base_url,
                    auth_source,
                } => {
                    if provider_id == amux_shared::providers::PROVIDER_ID_GITHUB_COPILOT
                        && matches!(auth_source.as_deref(), Some("github_copilot"))
                    {
                        match crate::agent::copilot_auth::begin_github_copilot_auth_flow() {
                            Ok(result) => {
                                let message = match result {
                                    crate::agent::copilot_auth::GithubCopilotAuthFlowResult::AlreadyAvailable => {
                                        "GitHub Copilot auth already available".to_string()
                                    }
                                    crate::agent::copilot_auth::GithubCopilotAuthFlowResult::ImportedFromGhCli => {
                                        "Imported GitHub Copilot auth from GitHub CLI".to_string()
                                    }
                                    crate::agent::copilot_auth::GithubCopilotAuthFlowResult::Started => {
                                        "Started GitHub Copilot browser login".to_string()
                                    }
                                };
                                framed
                                    .send(DaemonMessage::AgentProviderLoginResult {
                                        provider_id: provider_id.clone(),
                                        success: true,
                                        message: Some(message),
                                    })
                                    .await?;
                            }
                            Err(error) => {
                                framed
                                    .send(DaemonMessage::AgentProviderLoginResult {
                                        provider_id: provider_id.clone(),
                                        success: false,
                                        message: Some(error.to_string()),
                                    })
                                    .await?;
                            }
                        }

                        let states = agent.get_provider_auth_states().await;
                        let json = serde_json::to_string(&states).unwrap_or_default();
                        framed
                            .send(DaemonMessage::AgentProviderAuthStates { states_json: json })
                            .await?;
                        continue;
                    }

                    // Surgical update: modify only the target provider's key.
                    let mut config = agent.get_config().await;
                    let entry = config
                        .providers
                        .entry(provider_id.clone())
                        .or_insert_with(|| {
                            let def = crate::agent::types::get_provider_definition(&provider_id);
                            crate::agent::types::ProviderConfig {
                                base_url: if base_url.is_empty() {
                                    def.map(|d| d.default_base_url.to_string())
                                        .unwrap_or_default()
                                } else {
                                    base_url.clone()
                                },
                                model: def.map(|d| d.default_model.to_string()).unwrap_or_default(),
                                api_key: String::new(),
                                assistant_id: String::new(),
                                auth_source: auth_source
                                    .as_deref()
                                    .map(|value| match value {
                                        "chatgpt_subscription" => {
                                            crate::agent::types::AuthSource::ChatgptSubscription
                                        }
                                        "github_copilot" => {
                                            crate::agent::types::AuthSource::GithubCopilot
                                        }
                                        _ => crate::agent::types::AuthSource::ApiKey,
                                    })
                                    .unwrap_or(crate::agent::types::AuthSource::ApiKey),
                                api_transport:
                                    crate::agent::types::default_api_transport_for_provider(
                                        &provider_id,
                                    ),
                                reasoning_effort: "high".into(),
                                context_window_tokens: 128_000,
                                response_schema: None,
                                stop_sequences: None,
                                temperature: None,
                                top_p: None,
                                top_k: None,
                                metadata: None,
                                service_tier: None,
                                container: None,
                                inference_geo: None,
                                cache_control: None,
                                max_tokens: None,
                                anthropic_tool_choice: None,
                                output_effort: None,
                            }
                        });
                    entry.api_key = api_key;
                    if !base_url.is_empty() {
                        entry.base_url = base_url;
                    }
                    if let Some(auth_source) = auth_source.as_deref() {
                        entry.auth_source = match auth_source {
                            "chatgpt_subscription" => {
                                crate::agent::types::AuthSource::ChatgptSubscription
                            }
                            "github_copilot" => crate::agent::types::AuthSource::GithubCopilot,
                            _ => crate::agent::types::AuthSource::ApiKey,
                        };
                    }
                    agent.set_config(config).await;

                    let states = agent.get_provider_auth_states().await;
                    let json = serde_json::to_string(&states).unwrap_or_default();
                    framed
                        .send(DaemonMessage::AgentProviderAuthStates { states_json: json })
                        .await?;
                }

                ClientMessage::AgentLogoutProvider { provider_id } => {
                    if provider_id == amux_shared::providers::PROVIDER_ID_GITHUB_COPILOT {
                        let _ = crate::agent::copilot_auth::clear_stored_github_copilot_auth();
                    }
                    let mut config = agent.get_config().await;
                    if let Some(entry) = config.providers.get_mut(&provider_id) {
                        entry.api_key.clear();
                    }
                    if config.provider == provider_id {
                        config.api_key.clear();
                    }
                    agent.set_config(config).await;

                    let states = agent.get_provider_auth_states().await;
                    let json = serde_json::to_string(&states).unwrap_or_default();
                    framed
                        .send(DaemonMessage::AgentProviderAuthStates { states_json: json })
                        .await?;
                }

                ClientMessage::AgentGetOpenAICodexAuthStatus => {
                    let status_json = serde_json::to_string(
                        &crate::agent::openai_codex_auth::openai_codex_auth_status(true),
                    )
                    .unwrap_or_else(|_| "{}".to_string());
                    framed
                        .send(DaemonMessage::AgentOpenAICodexAuthStatus { status_json })
                        .await?;
                }

                ClientMessage::AgentLoginOpenAICodex => {
                    match crate::agent::openai_codex_auth::begin_openai_codex_auth_login() {
                        Ok(result) => {
                            let result_json = serde_json::to_string(&result)
                                .unwrap_or_else(|_| "{}".to_string());
                            framed
                                .send(DaemonMessage::AgentOpenAICodexAuthLoginResult {
                                    result_json,
                                })
                                .await?;

                            if result.auth_url.is_some()
                                && crate::agent::openai_codex_auth::mark_openai_codex_auth_completion_started()
                            {
                                let background_daemon_tx =
                                    background_daemon_queues.sender(BackgroundSubsystem::AgentWork);
                                background_daemon_pending.increment(BackgroundSubsystem::AgentWork);
                                std::thread::spawn(move || {
                                    let status_json = serde_json::to_string(
                                        &crate::agent::openai_codex_auth::complete_browser_auth(),
                                    )
                                    .unwrap_or_else(|_| "{}".to_string());
                                    let _ = background_daemon_tx.send(BackgroundSignal::Deliver(
                                        DaemonMessage::AgentOpenAICodexAuthStatus { status_json },
                                    ));
                                    let _ = background_daemon_tx.send(BackgroundSignal::Finished);
                                });
                            }
                        }
                        Err(error) => {
                            let result_json = serde_json::to_string(
                                &crate::agent::openai_codex_auth::openai_codex_auth_error_status(
                                    &error.to_string(),
                                ),
                            )
                            .unwrap_or_else(|_| "{}".to_string());
                            framed
                                .send(DaemonMessage::AgentOpenAICodexAuthLoginResult {
                                    result_json,
                                })
                                .await?;
                        }
                    }
                }

                ClientMessage::AgentLogoutOpenAICodex => {
                    match crate::agent::openai_codex_auth::logout_openai_codex_auth() {
                        Ok(()) => {
                            framed
                                .send(DaemonMessage::AgentOpenAICodexAuthLogoutResult {
                                    ok: true,
                                    error: None,
                                })
                                .await?;
                        }
                        Err(error) => {
                            framed
                                .send(DaemonMessage::AgentOpenAICodexAuthLogoutResult {
                                    ok: false,
                                    error: Some(
                                        crate::agent::openai_codex_auth::openai_codex_auth_error_message(
                                            &error.to_string(),
                                        ),
                                    ),
                                })
                                .await?;
                        }
                    }
                }

            _ => unreachable!("message chunk should be exhaustive"),
        }
        continue;
    }
