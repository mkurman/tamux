use super::super::make_model;
use super::opening_weles_editor_hides_inherited_main_system_prompt_to_feat_skill::focus_settings_field;
use super::*;
use crate::state::settings::SettingsTab;
use crossterm::event::{KeyCode, KeyModifiers};
#[test]
fn concierge_config_serializes_all_fields() {
    let (mut model, mut daemon_rx) = make_model();
    model.concierge.enabled = true;
    model.concierge.detail_level = "daily_briefing".to_string();
    model.concierge.provider = Some("anthropic".to_string());
    model.concierge.model = Some("claude-sonnet-4-5".to_string());
    model.concierge.reasoning_effort = Some("high".to_string());
    model.concierge.auto_cleanup_on_navigate = false;

    model.send_concierge_config();

    let payload = match daemon_rx.try_recv() {
        Ok(DaemonCommand::SetConciergeConfig(payload)) => payload,
        other => panic!("expected SetConciergeConfig, got {other:?}"),
    };
    let json: serde_json::Value = serde_json::from_str(&payload).expect("valid concierge json");
    assert_eq!(json["enabled"], true);
    assert_eq!(json["detail_level"], "daily_briefing");
    assert_eq!(json["provider"], "anthropic");
    assert_eq!(json["model"], "claude-sonnet-4-5");
    assert_eq!(json["reasoning_effort"], "high");
    assert_eq!(json["auto_cleanup_on_navigate"], false);
}

#[test]
fn concierge_config_write_requests_fresh_settings_snapshot() {
    let (mut model, mut daemon_rx) = make_model();
    model.concierge.provider = Some("anthropic".to_string());
    model.concierge.model = Some("claude-sonnet-4-5".to_string());

    model.send_concierge_config();

    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::SetConciergeConfig(_))
    ));
    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::GetConciergeConfig)
    ));
}

fn expect_concierge_config_update(
    daemon_rx: &mut tokio::sync::mpsc::UnboundedReceiver<DaemonCommand>,
) -> serde_json::Value {
    let payload = match daemon_rx.try_recv() {
        Ok(DaemonCommand::SetConciergeConfig(payload)) => payload,
        other => panic!("expected concierge config update, got {other:?}"),
    };
    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::GetConciergeConfig)
    ));
    serde_json::from_str(&payload).expect("json")
}

#[test]
fn concierge_settings_fields_dispatch_expected_actions() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));

    focus_settings_field(&mut model, SettingsTab::Concierge, "concierge_enabled");
    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    let json = expect_concierge_config_update(&mut daemon_rx);
    assert_eq!(json["enabled"], false);

    focus_settings_field(&mut model, SettingsTab::Concierge, "concierge_detail_level");
    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    let json = expect_concierge_config_update(&mut daemon_rx);
    assert_eq!(json["detail_level"], "daily_briefing");

    focus_settings_field(&mut model, SettingsTab::Concierge, "concierge_provider");
    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ProviderPicker));
    model.close_top_modal();

    focus_settings_field(&mut model, SettingsTab::Concierge, "concierge_model");
    model.concierge.provider = Some(PROVIDER_ID_CHUTES.to_string());
    model.config.agent_config_raw = Some(serde_json::json!({
        "providers": {
            PROVIDER_ID_CHUTES: {
                "base_url": "https://llm.chutes.ai/v1",
                "api_key": "chutes-key",
                "auth_source": "api_key"
            }
        }
    }));
    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    match daemon_rx.try_recv() {
        Ok(DaemonCommand::FetchModels {
            provider_id,
            base_url,
            api_key,
            output_modalities,
        }) => {
            assert_eq!(provider_id, PROVIDER_ID_CHUTES);
            assert_eq!(base_url, "https://llm.chutes.ai/v1");
            assert_eq!(api_key, "chutes-key");
            assert_eq!(output_modalities, None);
        }
        other => panic!("expected concierge model fetch command, got {other:?}"),
    }
    model.close_top_modal();

    focus_settings_field(
        &mut model,
        SettingsTab::Concierge,
        "concierge_reasoning_effort",
    );
    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::EffortPicker));
}
