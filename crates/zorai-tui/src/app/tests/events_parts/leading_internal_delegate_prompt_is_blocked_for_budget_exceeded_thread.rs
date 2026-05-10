use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
use tokio::sync::mpsc::unbounded_channel;
use zorai_shared::providers::*;
#[test]
fn leading_internal_delegate_prompt_is_blocked_for_budget_exceeded_thread() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-budget".to_string(),
        title: "Budget exceeded".to_string(),
        ..Default::default()
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-budget".to_string()));
    model.handle_task_update_event(crate::wire::AgentTask {
        id: "task-budget".to_string(),
        title: "Budget exceeded child".to_string(),
        thread_id: Some("thread-budget".to_string()),
        status: Some(crate::wire::TaskStatus::BudgetExceeded),
        blocked_reason: Some("execution budget exceeded for this thread".to_string()),
        created_at: 42,
        ..Default::default()
    });
    while daemon_rx.try_recv().is_ok() {}

    model.submit_prompt("!weles verify the auth regression".to_string());

    assert_eq!(
        model.input.buffer(),
        "!weles verify the auth regression",
        "blocked internal delegate should preserve operator text in the input"
    );
    let (notice, _) = model
        .input_notice_style()
        .expect("blocked internal delegate should surface an input notice");
    assert!(
        notice.contains("Thread budget exceeded"),
        "expected budget exceeded notice, got: {notice}"
    );
    while let Ok(command) = daemon_rx.try_recv() {
        assert!(
            !matches!(command, DaemonCommand::InternalDelegate { .. }),
            "budget-exceeded thread should not emit an internal delegate command: {command:?}"
        );
    }
}

#[test]
fn known_agent_directive_aliases_keep_builtin_entries_canonical_lowercase() {
    let (model, _) = make_model_with_daemon_rx();
    let aliases = model.known_agent_directive_aliases();

    assert!(aliases.iter().any(|alias| alias == "swarozyc"));
    assert!(aliases.iter().any(|alias| alias == "mokosh"));
    assert!(aliases.iter().any(|alias| alias == "veles"));
    assert!(!aliases.iter().any(|alias| alias == "Swarozyc"));
    assert!(!aliases.iter().any(|alias| alias == "Mokosh"));
    assert!(!aliases.iter().any(|alias| alias == "Dazhbog"));
}

#[test]
fn leading_internal_delegate_prompt_routes_veles_alias_to_internal_command() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.submit_prompt("!veles verify the auth regression".to_string());

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::InternalDelegate {
            thread_id,
            target_agent_id,
            content,
            ..
        }) => {
            assert_eq!(thread_id.as_deref(), Some("thread-1"));
            assert_eq!(target_agent_id, "veles");
            assert_eq!(content, "verify the auth regression");
        }
        other => panic!("expected internal delegate command, got {:?}", other),
    }
    assert!(
        model
            .chat
            .active_thread()
            .expect("thread should remain selected")
            .messages
            .is_empty(),
        "internal delegation should not append a visible user turn"
    );
}

#[test]
fn leading_participant_prompt_routes_to_participant_command() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.submit_prompt("@weles verify claims before answering".to_string());

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::ThreadParticipantCommand {
            thread_id,
            target_agent_id,
            action,
            instruction,
            ..
        }) => {
            assert_eq!(thread_id, "thread-1");
            assert_eq!(target_agent_id, "weles");
            assert_eq!(action, "upsert");
            assert_eq!(
                instruction.as_deref(),
                Some("verify claims before answering")
            );
        }
        other => panic!("expected participant command, got {:?}", other),
    }
    assert!(
        model
            .chat
            .active_thread()
            .expect("thread should remain selected")
            .messages
            .is_empty(),
        "participant registration should not append a visible user turn"
    );
    let (notice, _) = model
        .input_notice_style()
        .expect("participant command should surface a visible notice");
    assert!(
        notice.contains("Weles"),
        "expected agent name in notice, got: {notice}"
    );
    assert!(
        notice.contains("joined") || notice.contains("updated"),
        "expected participant update wording in notice, got: {notice}"
    );
}

#[test]
fn unconfigured_mokosh_participant_prompt_opens_setup_and_retries_after_model_selection() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.auth.entries = vec![crate::state::auth::ProviderAuthEntry {
        provider_id: zorai_shared::providers::PROVIDER_ID_ALIBABA_CODING_PLAN.to_string(),
        provider_name: "Alibaba Coding Plan".to_string(),
        authenticated: true,
        auth_source: "api_key".to_string(),
        model: "qwen3.6-plus".to_string(),
    }];
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.submit_prompt("@mokosh keep flow moving".to_string());

    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::ProviderPicker)
    );
    assert!(
        daemon_rx.try_recv().is_err(),
        "setup should happen before any daemon command is emitted"
    );

    let provider_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| {
            provider.id == zorai_shared::providers::PROVIDER_ID_ALIBABA_CODING_PLAN
        })
        .expect("provider to exist");
    if provider_index > 0 {
        model
            .modal
            .reduce(crate::state::modal::ModalAction::Navigate(
                provider_index as i32,
            ));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        crate::state::modal::ModalKind::ProviderPicker,
    );
    assert!(!quit);
    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::ModelPicker)
    );

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        crate::state::modal::ModalKind::ModelPicker,
    );
    assert!(!quit);

    match daemon_rx
        .try_recv()
        .expect("expected targeted builtin persona config command")
    {
        DaemonCommand::SetTargetAgentProviderModel {
            target_agent_id,
            provider_id,
            model,
        } => {
            assert_eq!(target_agent_id, "mokosh");
            assert_eq!(
                provider_id,
                zorai_shared::providers::PROVIDER_ID_ALIBABA_CODING_PLAN
            );
            assert!(
                !model.trim().is_empty(),
                "setup flow should choose a concrete model"
            );
        }
        other => panic!("expected builtin persona provider/model command, got {other:?}"),
    }

    match daemon_rx
        .try_recv()
        .expect("expected retried participant command after setup")
    {
        DaemonCommand::ThreadParticipantCommand {
            thread_id,
            target_agent_id,
            action,
            instruction,
            ..
        } => {
            assert_eq!(thread_id, "thread-1");
            assert_eq!(target_agent_id, "mokosh");
            assert_eq!(action, "upsert");
            assert_eq!(instruction.as_deref(), Some("keep flow moving"));
        }
        other => panic!("expected participant command, got {other:?}"),
    }
}

#[test]
fn unconfigured_builtin_participant_prompt_opens_setup_and_retries_after_model_selection() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.auth.entries = vec![crate::state::auth::ProviderAuthEntry {
        provider_id: zorai_shared::providers::PROVIDER_ID_ALIBABA_CODING_PLAN.to_string(),
        provider_name: "Alibaba Coding Plan".to_string(),
        authenticated: true,
        auth_source: "api_key".to_string(),
        model: "qwen3.6-plus".to_string(),
    }];
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.submit_prompt("@swarozyc verify claims before answering".to_string());

    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::ProviderPicker)
    );
    assert!(
        daemon_rx.try_recv().is_err(),
        "setup should happen before any daemon command is emitted"
    );

    let provider_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| {
            provider.id == zorai_shared::providers::PROVIDER_ID_ALIBABA_CODING_PLAN
        })
        .expect("provider to exist");
    if provider_index > 0 {
        model
            .modal
            .reduce(crate::state::modal::ModalAction::Navigate(
                provider_index as i32,
            ));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        crate::state::modal::ModalKind::ProviderPicker,
    );
    assert!(!quit);
    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::ModelPicker)
    );

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        crate::state::modal::ModalKind::ModelPicker,
    );
    assert!(!quit);

    match daemon_rx
        .try_recv()
        .expect("expected targeted builtin persona config command")
    {
        DaemonCommand::SetTargetAgentProviderModel {
            target_agent_id,
            provider_id,
            model,
        } => {
            assert_eq!(target_agent_id, "swarozyc");
            assert_eq!(
                provider_id,
                zorai_shared::providers::PROVIDER_ID_ALIBABA_CODING_PLAN
            );
            assert!(
                !model.trim().is_empty(),
                "setup flow should choose a concrete model"
            );
        }
        other => panic!("expected builtin persona provider/model command, got {other:?}"),
    }

    match daemon_rx
        .try_recv()
        .expect("expected retried participant command after setup")
    {
        DaemonCommand::ThreadParticipantCommand {
            thread_id,
            target_agent_id,
            action,
            instruction,
            ..
        } => {
            assert_eq!(thread_id, "thread-1");
            assert_eq!(target_agent_id, "swarozyc");
            assert_eq!(action, "upsert");
            assert_eq!(
                instruction.as_deref(),
                Some("verify claims before answering")
            );
        }
        other => panic!("expected participant command, got {other:?}"),
    }
}

#[test]
fn subagent_error_requests_refresh_to_clear_rejected_optimistic_state() {
    let (_event_tx, event_rx) = std::sync::mpsc::channel();
    let (daemon_tx, mut daemon_rx) = unbounded_channel();
    let mut model = TuiModel::new(event_rx, daemon_tx);
    model.subagents.entries = vec![crate::state::SubAgentEntry {
        id: "weles_builtin".to_string(),
        name: "Legacy WELES".to_string(),
        provider: PROVIDER_ID_OPENAI.to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("testing".to_string()),
        enabled: true,
        builtin: false,
        immutable_identity: false,
        disable_allowed: true,
        delete_allowed: true,
        protected_reason: None,
        reasoning_effort: None,
        openrouter_provider_order: String::new(),
        openrouter_provider_ignore: String::new(),
        openrouter_allow_fallbacks: true,
        raw_json: Some(serde_json::json!({
            "id": "weles_builtin",
            "name": "Legacy WELES"
        })),
    }];

    model.handle_client_event(ClientEvent::Error(
        "protected mutation: reserved built-in sub-agent".to_string(),
    ));

    assert_eq!(
        model.subagents.entries.len(),
        1,
        "stale optimistic entry remains until refresh arrives"
    );
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("subagent error should request authoritative refresh"),
        DaemonCommand::ListSubAgents
    ));
}

#[test]
fn openai_codex_auth_events_update_config_and_modal_state() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.handle_client_event(ClientEvent::OpenAICodexAuthStatus(
        crate::client::OpenAICodexAuthStatusVm {
            available: false,
            auth_mode: Some("chatgpt_subscription".to_string()),
            account_id: None,
            expires_at: None,
            source: Some("zorai-daemon".to_string()),
            error: None,
            auth_url: None,
            status: Some("pending".to_string()),
        },
    ));

    assert!(!model.config.chatgpt_auth_available);
    assert_eq!(
        model.config.chatgpt_auth_source.as_deref(),
        Some("zorai-daemon")
    );

    model.handle_client_event(ClientEvent::OpenAICodexAuthLoginResult(
        crate::client::OpenAICodexAuthStatusVm {
            available: false,
            auth_mode: Some("chatgpt_subscription".to_string()),
            account_id: None,
            expires_at: None,
            source: Some("zorai-daemon".to_string()),
            error: None,
            auth_url: Some("https://auth.openai.com/oauth/authorize?flow=tui".to_string()),
            status: Some("pending".to_string()),
        },
    ));

    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::OpenAIAuth)
    );
    assert_eq!(
        model.openai_auth_url.as_deref(),
        Some("https://auth.openai.com/oauth/authorize?flow=tui")
    );
    assert!(model
        .openai_auth_status_text
        .as_deref()
        .is_some_and(|text| text.contains("complete ChatGPT authentication")));
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected auth state refresh after status"),
        DaemonCommand::GetProviderAuthStates
    ));
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected auth state refresh after login"),
        DaemonCommand::GetProviderAuthStates
    ));

    model.handle_client_event(ClientEvent::OpenAICodexAuthLogoutResult {
        ok: true,
        error: None,
    });

    assert!(!model.config.chatgpt_auth_available);
    assert!(model.config.chatgpt_auth_source.is_none());
    assert!(model.openai_auth_url.is_none());
    assert!(model.openai_auth_status_text.is_none());
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected auth state refresh after logout"),
        DaemonCommand::GetProviderAuthStates
    ));
    assert_eq!(model.status_line, "ChatGPT subscription auth cleared");
}
