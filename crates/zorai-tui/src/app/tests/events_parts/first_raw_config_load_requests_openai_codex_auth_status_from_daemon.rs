use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn first_raw_config_load_requests_openai_codex_auth_status_from_daemon() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.connected = true;
    model.agent_config_loaded = false;

    model.handle_agent_config_raw_event(serde_json::json!({
        "provider": PROVIDER_ID_OPENAI,
        "base_url": "https://api.openai.com/v1",
        "model": "gpt-5.4",
    }));

    let mut saw_auth_status = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(command, DaemonCommand::GetOpenAICodexAuthStatus) {
            saw_auth_status = true;
            break;
        }
    }

    assert!(
        saw_auth_status,
        "first config load should release deferred codex auth refresh"
    );
}

#[test]
fn provider_auth_states_overlay_chatgpt_auth_when_openai_is_configured_for_chatgpt_subscription() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_OPENAI.to_string();
    model.config.auth_source = "chatgpt_subscription".to_string();
    model.config.chatgpt_auth_available = true;
    model.config.chatgpt_auth_source = Some("zorai-daemon".to_string());

    model.handle_provider_auth_states_event(vec![crate::state::ProviderAuthEntry {
        provider_id: PROVIDER_ID_OPENAI.to_string(),
        provider_name: "OpenAI".to_string(),
        authenticated: false,
        auth_source: "api_key".to_string(),
        model: "gpt-5.4".to_string(),
    }]);

    let openai = model
        .auth
        .entries
        .iter()
        .find(|entry| entry.provider_id == PROVIDER_ID_OPENAI)
        .expect("openai auth entry should exist");
    assert!(
        openai.authenticated,
        "chatgpt daemon auth should surface as connected"
    );
    assert_eq!(openai.auth_source, "chatgpt_subscription");
}

#[test]
fn provider_auth_states_overlay_chatgpt_auth_for_openai_even_when_another_provider_is_active() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.chatgpt_auth_available = true;
    model.config.chatgpt_auth_source = Some("zorai-daemon".to_string());

    model.handle_provider_auth_states_event(vec![crate::state::ProviderAuthEntry {
        provider_id: PROVIDER_ID_OPENAI.to_string(),
        provider_name: "OpenAI".to_string(),
        authenticated: false,
        auth_source: "api_key".to_string(),
        model: "gpt-5.4".to_string(),
    }]);

    let openai = model
        .auth
        .entries
        .iter()
        .find(|entry| entry.provider_id == PROVIDER_ID_OPENAI)
        .expect("openai auth entry should exist");
    assert!(
        openai.authenticated,
        "chatgpt daemon auth should keep openai selectable in provider picker"
    );
    assert_eq!(openai.auth_source, "chatgpt_subscription");
}

#[test]
fn provider_validation_event_updates_auth_result_and_status_line() {
    let mut model = make_model();
    model.auth.entries = vec![crate::state::ProviderAuthEntry {
        provider_id: PROVIDER_ID_OPENAI.to_string(),
        provider_name: "OpenAI".to_string(),
        authenticated: true,
        auth_source: "api_key".to_string(),
        model: "gpt-5.4".to_string(),
    }];
    model.auth.validating = Some(PROVIDER_ID_OPENAI.to_string());

    model.handle_client_event(ClientEvent::ProviderValidation {
        provider_id: PROVIDER_ID_OPENAI.to_string(),
        valid: false,
        error: Some("bad key".to_string()),
    });

    assert_eq!(model.auth.validating, None);
    assert_eq!(model.status_line, "OpenAI test failed: bad key");
    assert_eq!(
        model
            .auth
            .validation_results
            .get(PROVIDER_ID_OPENAI)
            .cloned(),
        Some((false, "Error: bad key".to_string()))
    );
}

#[test]
fn openai_codex_auth_status_event_clears_stale_modal_state() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.openai_auth_url = Some("https://stale.example/login".to_string());
    model.openai_auth_status_text = Some("stale".to_string());
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::OpenAIAuth));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));

    model.handle_client_event(ClientEvent::OpenAICodexAuthStatus(
        crate::client::OpenAICodexAuthStatusVm {
            available: false,
            auth_mode: Some("chatgpt_subscription".to_string()),
            account_id: None,
            expires_at: None,
            source: Some("zorai-daemon".to_string()),
            error: Some("Timed out waiting for callback".to_string()),
            auth_url: None,
            status: Some("error".to_string()),
        },
    ));

    assert!(model.openai_auth_url.is_none());
    assert_eq!(
        model.openai_auth_status_text.as_deref(),
        Some("Timed out waiting for callback")
    );
    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::OpenAIAuth)
    );
    model.close_top_modal();
    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::CommandPalette)
    );
    assert!(matches!(
        daemon_rx
            .try_recv()
            .expect("expected auth state refresh after status"),
        DaemonCommand::GetProviderAuthStates
    ));
    assert_eq!(model.status_line, "Timed out waiting for callback");
}

#[test]
fn openai_codex_auth_status_event_removes_all_stale_nested_openai_modals() {
    let mut model = make_model();
    model.openai_auth_url = Some("https://stale.example/login".to_string());
    model.openai_auth_status_text = Some("stale".to_string());
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::OpenAIAuth));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::OpenAIAuth));

    model.handle_client_event(ClientEvent::OpenAICodexAuthStatus(
        crate::client::OpenAICodexAuthStatusVm {
            available: false,
            auth_mode: Some("chatgpt_subscription".to_string()),
            account_id: None,
            expires_at: None,
            source: Some("zorai-daemon".to_string()),
            error: Some("Timed out waiting for callback".to_string()),
            auth_url: None,
            status: Some("error".to_string()),
        },
    ));

    assert_eq!(model.modal.top(), Some(modal::ModalKind::OpenAIAuth));
    model.close_top_modal();
    assert_eq!(model.modal.top(), Some(modal::ModalKind::CommandPalette));
    model.close_top_modal();
    assert_eq!(model.modal.top(), Some(modal::ModalKind::Settings));
}

#[test]
fn disconnect_and_reconnect_clear_openai_auth_modal_even_when_nested() {
    let mut model = make_model();
    model.openai_auth_url = Some("https://auth.openai.com/oauth/authorize?flow=tui".to_string());
    model.openai_auth_status_text = Some("pending".to_string());
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::OpenAIAuth));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::CommandPalette));

    model.handle_disconnected_event();

    assert!(model.openai_auth_url.is_none());
    assert!(model.openai_auth_status_text.is_none());
    assert_eq!(model.modal.top(), Some(modal::ModalKind::CommandPalette));

    model.openai_auth_url = Some("https://auth.openai.com/oauth/authorize?flow=tui".to_string());
    model.openai_auth_status_text = Some("pending".to_string());
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::OpenAIAuth));

    model.handle_reconnecting_event(3);

    assert!(model.openai_auth_url.is_none());
    assert!(model.openai_auth_status_text.is_none());
    assert_eq!(model.modal.top(), Some(modal::ModalKind::CommandPalette));
}

#[test]
fn disconnect_clears_transport_send_errors_from_error_state() {
    let mut model = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ErrorViewer));

    model.handle_client_event(ClientEvent::Error(
        "Send error: Broken pipe (os error 32)".to_string(),
    ));
    assert_eq!(
        model.last_error.as_deref(),
        Some("Send error: Broken pipe (os error 32)")
    );
    assert!(model.error_active);

    model.handle_client_event(ClientEvent::Disconnected);

    assert!(model.last_error.is_none());
    assert!(!model.error_active);
    assert_ne!(model.modal.top(), Some(modal::ModalKind::ErrorViewer));
    assert_eq!(model.status_line, "Disconnected from daemon");
}
