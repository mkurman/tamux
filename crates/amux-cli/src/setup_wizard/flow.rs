use super::*;

const SETUP_PROBE_TIMEOUT_SECS: u64 = 5;

pub(super) fn parse_fetch_models_terminal_response(msg: DaemonMessage) -> Option<Result<String>> {
    match msg {
        DaemonMessage::OperationAccepted { .. } => None,
        DaemonMessage::AgentModelsResponse { models_json, .. } => Some(Ok(models_json)),
        DaemonMessage::AgentError { message } | DaemonMessage::Error { message } => {
            Some(Err(anyhow::anyhow!(message)))
        }
        _ => None,
    }
}

pub(super) fn parse_set_config_item_response(msg: DaemonMessage) -> Option<Result<()>> {
    match msg {
        DaemonMessage::OperationAccepted { .. } | DaemonMessage::AgentConfigResponse { .. } => {
            Some(Ok(()))
        }
        DaemonMessage::AgentError { message } | DaemonMessage::Error { message } => {
            Some(Err(anyhow::anyhow!(message)))
        }
        _ => None,
    }
}

async fn recv_fetch_models_terminal_response(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
) -> Result<String> {
    loop {
        let msg = wizard_recv(framed).await?;
        if let Some(result) = parse_fetch_models_terminal_response(msg) {
            return result;
        }
    }
}

pub(super) async fn set_config_item(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    key_path: impl Into<String>,
    value_json: impl Into<String>,
) -> Result<()> {
    wizard_send(
        framed,
        ClientMessage::AgentSetConfigItem {
            key_path: key_path.into(),
            value_json: value_json.into(),
        },
    )
    .await?;

    loop {
        let msg = wizard_recv(framed).await?;
        if let Some(result) = parse_set_config_item_response(msg) {
            return result;
        }
    }
}

pub(super) async fn select_provider(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
) -> Result<ProviderSelection> {
    wizard_send(framed, ClientMessage::AgentGetProviderAuthStates).await?;
    let providers: Vec<ProviderAuthState> = match wizard_recv(framed).await? {
        DaemonMessage::AgentProviderAuthStates { states_json } => {
            serde_json::from_str(&states_json)
                .context("Failed to parse provider auth states from daemon")?
        }
        other => anyhow::bail!("Unexpected daemon response: {other:?}"),
    };

    if providers.is_empty() {
        anyhow::bail!("Daemon returned empty provider list. Is the daemon running correctly?");
    }

    let provider_items: Vec<(&str, &str)> = providers
        .iter()
        .map(|p| (p.provider_name.as_str(), p.provider_id.as_str()))
        .collect();
    let provider_idx = select_list("Select your LLM provider:", &provider_items, false, 0)?
        .expect("provider selection is required");
    let selected = &providers[provider_idx];

    Ok(ProviderSelection {
        provider_id: selected.provider_id.clone(),
        provider_name: selected.provider_name.clone(),
        base_url: selected.base_url.clone(),
        default_model: selected.model.clone(),
        auth_source: selected.auth_source.clone(),
    })
}

async fn configure_api_key(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    provider: &ProviderSelection,
    selected_provider: &ProviderAuthState,
) -> Result<String> {
    if is_local_provider(&provider.provider_id) {
        println!("Local provider -- no API key needed.");
        return Ok(String::new());
    }

    if selected_provider.authenticated {
        println!(
            "{} is already authenticated.",
            provider.provider_name.clone().bold()
        );
        let replace_idx = select_list(
            "Replace API key?",
            &[("No, keep existing key", ""), ("Yes, enter a new key", "")],
            false,
            0,
        )?
        .unwrap_or(0);
        if replace_idx == 0 {
            println!("Keeping existing API key.");
            return Ok("(existing)".to_string());
        }
    }

    let prompt = if selected_provider.authenticated {
        format!("Enter new API key for {}", provider.provider_name)
    } else {
        format!("Enter API key for {}", provider.provider_name)
    };
    let api_key = text_input(&prompt, "", true)?.unwrap_or_default();
    if api_key.is_empty() {
        println!("No API key entered. You can set it later with `tamux setup`.");
        return Ok(String::new());
    }

    let writes = if selected_provider.authenticated {
        vec![
            (
                "provider",
                serde_json::to_string(&provider.provider_id).unwrap_or_default(),
            ),
            (
                "api_key",
                serde_json::to_string(&api_key).unwrap_or_default(),
            ),
        ]
    } else {
        vec![
            (
                "provider",
                serde_json::to_string(&provider.provider_id).unwrap_or_default(),
            ),
            (
                "api_key",
                serde_json::to_string(&api_key).unwrap_or_default(),
            ),
            (
                "base_url",
                serde_json::to_string(&provider.base_url).unwrap_or_default(),
            ),
            (
                "model",
                serde_json::to_string(&provider.default_model).unwrap_or_default(),
            ),
        ]
    };

    for (key, val) in writes {
        set_config_item(framed, format!("/{key}"), val)
            .await
            .with_context(|| format!("Failed to set {key}"))?;
    }
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    println!(
        "{}",
        if selected_provider.authenticated {
            "API key updated."
        } else {
            "API key saved."
        }
    );
    Ok(api_key)
}

async fn persist_provider_auth_source(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    provider_id: &str,
    auth_source: &str,
) -> Result<()> {
    let value_json = serde_json::to_string(auth_source).unwrap_or_default();
    set_config_item(
        framed,
        format!("/providers/{provider_id}/auth_source"),
        value_json.clone(),
    )
    .await
    .with_context(|| format!("Failed to set auth_source for {provider_id}"))?;
    set_config_item(framed, "/auth_source", value_json)
        .await
        .context("Failed to set active auth_source")?;
    Ok(())
}

async fn configure_provider_auth(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    provider: &ProviderSelection,
    selected_provider: &ProviderAuthState,
) -> Result<(String, String)> {
    if provider.auth_source == "github_copilot" {
        let options = if selected_provider.authenticated {
            vec![
                ("Keep existing browser login", "github_copilot_keep"),
                ("Use browser login", "github_copilot"),
                ("Use API token instead", "api_key"),
            ]
        } else {
            vec![
                ("Use browser login", "github_copilot"),
                ("Use API token instead", "api_key"),
            ]
        };
        let choice = select_list(
            "How should GitHub Copilot authenticate?",
            &options,
            false,
            0,
        )?
        .expect("provider auth selection is required");
        let selected_auth_source = options[choice].1;

        if selected_auth_source == "github_copilot_keep" {
            persist_provider_auth_source(framed, &provider.provider_id, "github_copilot").await?;
            println!("Keeping existing GitHub Copilot browser login.");
            return Ok(("(existing)".to_string(), "github_copilot".to_string()));
        }

        if selected_auth_source == "github_copilot" {
            persist_provider_auth_source(framed, &provider.provider_id, "github_copilot").await?;
            let (success, message) = login_provider_on_stream(
                framed,
                &provider.provider_id,
                "",
                &provider.base_url,
                Some("github_copilot"),
            )
            .await?;
            if success {
                if let Some(message) = message {
                    println!("{message}.");
                }
                println!(
                    "If the browser flow is still in progress, complete it and rerun `tamux setup` if validation fails."
                );
                return Ok((String::new(), "github_copilot".to_string()));
            }
            anyhow::bail!(
                "{}",
                message.unwrap_or_else(|| "GitHub Copilot login failed".to_string())
            );
        }

        let api_key = configure_api_key(framed, provider, selected_provider).await?;
        persist_provider_auth_source(framed, &provider.provider_id, "api_key").await?;
        return Ok((api_key, "api_key".to_string()));
    }

    let api_key = configure_api_key(framed, provider, selected_provider).await?;
    persist_provider_auth_source(framed, &provider.provider_id, &provider.auth_source).await?;
    Ok((api_key, provider.auth_source.clone()))
}

async fn fetch_selected_provider_state(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    provider_id: &str,
) -> Result<ProviderAuthState> {
    wizard_send(framed, ClientMessage::AgentGetProviderAuthStates).await?;
    let providers: Vec<ProviderAuthState> = match wizard_recv(framed).await? {
        DaemonMessage::AgentProviderAuthStates { states_json } => {
            serde_json::from_str(&states_json)
                .context("Failed to parse provider auth states from daemon")?
        }
        other => anyhow::bail!("Unexpected daemon response: {other:?}"),
    };
    providers
        .into_iter()
        .find(|p| p.provider_id == provider_id)
        .ok_or_else(|| anyhow::anyhow!("selected provider disappeared from daemon state"))
}

async fn configure_model(
    framed: &mut Framed<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin, AmuxCodec>,
    provider: &ProviderSelection,
    api_key_saved: &str,
    tier_string: &str,
    summary: &mut SetupSummary,
) -> Result<()> {
    println!();
    let existing_model = read_config_key("model").await;
    let user_wants_replace = if let Some(ref model) = existing_model {
        println!(
            "Svarog's model is already configured ({}).",
            model.clone().bold()
        );
        match select_list(
            "Replace model?",
            &[
                ("No, keep existing model", ""),
                ("Yes, choose a new model", ""),
            ],
            false,
            0,
        )? {
            Some(1) => true,
            _ => {
                println!("Keeping existing model.");
                summary.model = Some(model.clone());
                false
            }
        }
    } else {
        false
    };

    let show_model_picker =
        user_wants_replace || (existing_model.is_none() && tier_shows_step(tier_string, "model"));

    if show_model_picker {
        println!("Fetching available models...");
        wizard_send(
            framed,
            ClientMessage::AgentFetchModels {
                provider_id: provider.provider_id.clone(),
                base_url: provider.base_url.clone(),
                api_key: api_key_saved.to_string(),
            },
        )
        .await
        .context("Failed to fetch models")?;

        match recv_fetch_models_terminal_response(framed).await {
            Ok(models_json) => {
                let models: Vec<String> = serde_json::from_str(&models_json).unwrap_or_default();
                if !models.is_empty() {
                    let items: Vec<(&str, &str)> =
                        models.iter().map(|m| (m.as_str(), "")).collect();
                    if let Some(idx) =
                        select_list("Select default Svarog's model:", &items, true, 0)?
                    {
                        let chosen = &models[idx];
                        set_config_item(framed, "/model", format!("\"{}\"", chosen))
                            .await
                            .context("Failed to set Svarog's model")?;
                        summary.model = Some(chosen.clone());
                    } else {
                        println!("Skipped -- using default Svarog's model.");
                    }
                } else {
                    let fallback = if provider.default_model.is_empty() {
                        ""
                    } else {
                        &provider.default_model
                    };
                    if let Some(m) =
                        text_input("Enter model name (or Esc to skip)", fallback, false)?
                    {
                        if !m.is_empty() {
                            set_config_item(framed, "/model", format!("\"{}\"", m))
                                .await
                                .context("Failed to set Svarog's model")?;
                            summary.model = Some(m);
                        } else {
                            println!("Skipped -- using default Svarog's model.");
                        }
                    } else {
                        println!("Skipped -- using default Svarog's model.");
                    }
                }
            }
            Err(error) => {
                let message = error.to_string();
                println!("Could not fetch models: {message}");
                let fallback = if provider.default_model.is_empty() {
                    ""
                } else {
                    &provider.default_model
                };
                if let Some(m) = text_input("Enter model name (or Esc to skip)", fallback, false)? {
                    if !m.is_empty() {
                        set_config_item(framed, "/model", format!("\"{}\"", m))
                            .await
                            .context("Failed to set Svarog's model")?;
                        summary.model = Some(m);
                    } else {
                        println!("Skipped -- using default Svarog's model.");
                    }
                } else {
                    println!("Skipped -- using default Svarog's model.");
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    } else if existing_model.is_none() && !provider.default_model.is_empty() {
        set_config_item(framed, "/model", format!("\"{}\"", provider.default_model))
            .await
            .context("Failed to set default Svarog's model")?;
        summary.model = Some(provider.default_model.clone());
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SetupProbe {
    Ready,
    NeedsSetup,
    DaemonUnavailable,
}

pub(crate) fn setup_probe_from_config_json(config_json: &str) -> SetupProbe {
    let value: serde_json::Value = match serde_json::from_str(config_json) {
        Ok(v) => v,
        Err(_) => return SetupProbe::NeedsSetup,
    };
    if matches!(value.get("provider").and_then(|v| v.as_str()), Some(s) if !s.is_empty()) {
        SetupProbe::Ready
    } else {
        SetupProbe::NeedsSetup
    }
}

pub(crate) async fn probe_setup_via_ipc() -> SetupProbe {
    let mut framed = match wizard_connect().await {
        Ok(f) => f,
        Err(_) => return SetupProbe::DaemonUnavailable,
    };
    if wizard_send(&mut framed, ClientMessage::AgentGetConfig)
        .await
        .is_err()
    {
        return SetupProbe::DaemonUnavailable;
    }
    match tokio::time::timeout(
        std::time::Duration::from_secs(SETUP_PROBE_TIMEOUT_SECS),
        wizard_recv(&mut framed),
    )
    .await
    {
        Ok(Ok(DaemonMessage::AgentConfigResponse { config_json })) => {
            setup_probe_from_config_json(&config_json)
        }
        _ => SetupProbe::DaemonUnavailable,
    }
}

pub async fn run_setup_wizard() -> Result<PostSetupAction> {
    ensure_daemon_running().await?;
    let mut framed = wizard_connect()
        .await
        .context("Failed to connect to daemon for setup")?;

    println!();
    println!("{}", "Svarog, Rarog, Weles -- Agents That Live".bold());
    println!("First-time setup");
    println!();

    let tier_items = [
        ("Just getting started", "newcomer"),
        ("I've used chatbots and assistants", "familiar"),
        ("I run automations and scripting", "power_user"),
        ("I build agent systems", "expert"),
    ];
    let tier_idx = select_list(
        "How familiar are you with AI agents?",
        &tier_items,
        false,
        0,
    )?
    .expect("tier selection is required");
    let tier_string = tier_items[tier_idx].1.to_string();
    wizard_send(
        &mut framed,
        ClientMessage::AgentSetTierOverride {
            tier: Some(tier_string.clone()),
        },
    )
    .await
    .context("Failed to set tier override")?;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    println!();

    let provider = select_provider(&mut framed).await?;
    let selected_provider =
        fetch_selected_provider_state(&mut framed, &provider.provider_id).await?;
    println!();
    let (api_key_saved, selected_auth_source) =
        configure_provider_auth(&mut framed, &provider, &selected_provider).await?;
    println!();

    set_config_item(
        &mut framed,
        "/provider",
        format!("\"{}\"", provider.provider_id),
    )
    .await
    .context("Failed to set active provider")?;

    let mut summary = SetupSummary::default();
    configure_model(
        &mut framed,
        &provider,
        &api_key_saved,
        &tier_string,
        &mut summary,
    )
    .await?;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    configure_advanced_agents(&mut framed, &tier_string, &mut summary).await?;

    if !is_local_provider(&provider.provider_id) && !api_key_saved.is_empty() {
        println!();
        println!("Testing connection to {}...", provider.provider_name);
        match validate_provider_on_stream(&mut framed, &provider.provider_id, &selected_auth_source)
            .await
        {
            Ok(true) => println!("{}", "Connection successful!".with(style::Color::Green)),
            Ok(false) => {
                println!("Provider validation returned invalid. You can retry with `tamux setup`.");
            }
            Err(e) => {
                println!("Could not test connection: {e}");
                println!("You can retry later with `tamux setup`.");
            }
        }
    }

    println!();
    let security_items = [
        ("Ask for risky actions only (strict)", "highest"),
        ("Ask for risky actions only", "moderate"),
        ("Ask for destructive actions only", "lowest"),
        ("I trust it, minimize interruptions", "yolo"),
    ];
    let security_idx = select_list(
        "How cautious should tamux be?",
        &security_items,
        false,
        default_security_index(&tier_string),
    )?
    .expect("security selection is required");
    let (security_level_str, security_level_label) = security_level_from_index(security_idx);
    set_config_item(
        &mut framed,
        "/managed_execution/security_level",
        format!("\"{}\"", security_level_str),
    )
    .await
    .context("Failed to set security level")?;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    configure_web_search(&mut framed, &mut summary).await?;
    configure_gateway(&mut framed, &mut summary).await?;

    if tier_shows_step(&tier_string, "data_dir") {
        println!();
        let data_dir = amux_protocol::amux_data_dir();
        println!("Data stored at: {}", data_dir.display());
    }

    print_summary_and_choose_action(&provider.provider_name, security_level_label, &summary).await
}
