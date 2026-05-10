use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn selecting_image_generation_model_updates_image_model() {
    let (mut model, _daemon_rx) = make_model();
    model.config.agent_config_raw = Some(serde_json::json!({
        "image": {
            "generation": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "gpt-image-1"
            }
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "openai/gpt-image-1".to_string(),
                name: Some("OpenAI GPT Image 1".to_string()),
                context_window: None,
                pricing: Some(crate::state::config::FetchedModelPricing {
                    image: Some("0.00001".to_string()),
                    ..Default::default()
                }),
                metadata: Some(serde_json::json!({
                    "output_modalities": ["image"]
                })),
            },
            crate::state::config::FetchedModel {
                id: "gpt-4o-mini".to_string(),
                name: Some("GPT-4o Mini".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: Some(serde_json::json!({
                    "output_modalities": ["text"]
                })),
            },
        ]));

    model.settings_picker_target = Some(SettingsPickerTarget::ImageGenerationModel);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );

    assert!(!quit);
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("image"))
            .and_then(|image| image.get("generation"))
            .and_then(|generation| generation.get("model"))
            .and_then(|value| value.as_str()),
        Some("gpt-image-1")
    );
}

#[test]
fn selecting_main_image_capable_model_enables_vision() {
    let (mut model, mut daemon_rx) = make_model();
    model.config.tool_vision = false;
    model.config.model.clear();
    model.config.agent_config_raw = Some(serde_json::json!({
        "tools": {
            "vision": false
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "gpt-4.1-image".to_string(),
                name: Some("GPT 4.1 Image".to_string()),
                context_window: Some(128_000),
                pricing: Some(crate::state::config::FetchedModelPricing {
                    image: Some("0.00001".to_string()),
                    ..Default::default()
                }),
                metadata: Some(serde_json::json!({
                    "architecture": {
                        "input_modalities": ["text", "image"],
                        "output_modalities": ["text"]
                    }
                })),
            },
        ]));
    model.settings_picker_target = Some(SettingsPickerTarget::Model);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    navigate_model_picker_to(&mut model, "gpt-4.1-image");

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.model, "gpt-4.1-image");
    assert!(model.config.tool_vision);
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("tools"))
            .and_then(|tools| tools.get("vision"))
            .and_then(|value| value.as_bool()),
        Some(true)
    );

    let commands = collect_daemon_commands(&mut daemon_rx);
    assert!(commands.iter().any(|command| {
        matches!(
            command,
            DaemonCommand::SetConfigItem { key_path, value_json }
                if key_path == "/tools/vision" && value_json == "true"
        )
    }));
}

#[test]
fn selecting_main_audio_capable_model_prompts_for_stt_reuse() {
    let (mut model, mut daemon_rx) = make_model();
    model.config.model.clear();
    model.config.agent_config_raw = Some(serde_json::json!({
        "audio": {
            "stt": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "whisper-1"
            }
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "gpt-4o-audio-preview".to_string(),
                name: Some("GPT-4o Audio Preview".to_string()),
                context_window: Some(128_000),
                pricing: Some(crate::state::config::FetchedModelPricing {
                    audio: Some("0.000032".to_string()),
                    ..Default::default()
                }),
                metadata: Some(serde_json::json!({
                    "architecture": {
                        "input_modalities": ["text", "audio"],
                        "output_modalities": ["text", "audio"]
                    }
                })),
            },
        ]));
    model.settings_picker_target = Some(SettingsPickerTarget::Model);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    navigate_model_picker_to(&mut model, "gpt-4o-audio-preview");

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.model, "gpt-4o-audio-preview");
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Selected model supports audio. Use it as the STT model too?")
    );
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("stt"))
            .and_then(|stt| stt.get("model"))
            .and_then(|value| value.as_str()),
        Some("whisper-1")
    );

    let commands = collect_daemon_commands(&mut daemon_rx);
    assert!(!commands.iter().any(|command| {
        matches!(
            command,
            DaemonCommand::SetConfigItem { key_path, .. } if key_path == "/audio/stt/model"
        )
    }));
}

#[test]
fn selecting_main_model_with_only_generic_audio_metadata_does_not_prompt_for_stt_reuse() {
    let (mut model, mut daemon_rx) = make_model();
    model.config.model.clear();
    model.config.agent_config_raw = Some(serde_json::json!({
        "audio": {
            "stt": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "whisper-1"
            }
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "openai/generic-audio".to_string(),
                name: Some("Generic Audio".to_string()),
                context_window: Some(128_000),
                pricing: Some(crate::state::config::FetchedModelPricing {
                    audio: Some("0.000032".to_string()),
                    ..Default::default()
                }),
                metadata: Some(serde_json::json!({
                    "modalities": ["text", "audio"]
                })),
            },
        ]));
    model.settings_picker_target = Some(SettingsPickerTarget::Model);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    navigate_model_picker_to(&mut model, "openai/generic-audio");

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.model, "openai/generic-audio");
    assert_ne!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("stt"))
            .and_then(|stt| stt.get("model"))
            .and_then(|value| value.as_str()),
        Some("whisper-1")
    );

    let commands = collect_daemon_commands(&mut daemon_rx);
    assert!(!commands.iter().any(|command| {
        matches!(
            command,
            DaemonCommand::SetConfigItem { key_path, .. } if key_path == "/audio/stt/model"
        )
    }));
}

#[test]
fn selecting_main_model_with_nondirectional_modality_string_does_not_prompt_for_stt_reuse() {
    let (mut model, mut daemon_rx) = make_model();
    model.config.model.clear();
    model.config.agent_config_raw = Some(serde_json::json!({
        "audio": {
            "stt": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "whisper-1"
            }
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "openai/plain-modality-audio".to_string(),
                name: Some("Plain Modality Audio".to_string()),
                context_window: Some(128_000),
                pricing: Some(crate::state::config::FetchedModelPricing {
                    audio: Some("0.000032".to_string()),
                    ..Default::default()
                }),
                metadata: Some(serde_json::json!({
                    "architecture": {
                        "modality": "text+audio"
                    }
                })),
            },
        ]));
    model.settings_picker_target = Some(SettingsPickerTarget::Model);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    navigate_model_picker_to(&mut model, "openai/plain-modality-audio");

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.model, "openai/plain-modality-audio");
    assert_ne!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("stt"))
            .and_then(|stt| stt.get("model"))
            .and_then(|value| value.as_str()),
        Some("whisper-1")
    );

    let commands = collect_daemon_commands(&mut daemon_rx);
    assert!(!commands.iter().any(|command| {
        matches!(
            command,
            DaemonCommand::SetConfigItem { key_path, .. } if key_path == "/audio/stt/model"
        )
    }));
}

#[test]
fn accepting_audio_model_stt_reuse_updates_stt_model() {
    let (mut model, mut daemon_rx) = make_model();
    model.config.model.clear();
    model.config.agent_config_raw = Some(serde_json::json!({
        "audio": {
            "stt": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "whisper-1"
            }
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "gpt-4o-audio-preview".to_string(),
                name: Some("GPT-4o Audio Preview".to_string()),
                context_window: Some(128_000),
                pricing: Some(crate::state::config::FetchedModelPricing {
                    audio: Some("0.000032".to_string()),
                    ..Default::default()
                }),
                metadata: Some(serde_json::json!({
                    "architecture": {
                        "input_modalities": ["text", "audio"],
                        "output_modalities": ["text", "audio"]
                    }
                })),
            },
        ]));
    model.settings_picker_target = Some(SettingsPickerTarget::Model);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    navigate_model_picker_to(&mut model, "gpt-4o-audio-preview");

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );
    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );

    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("stt"))
            .and_then(|stt| stt.get("provider"))
            .and_then(|value| value.as_str()),
        Some(PROVIDER_ID_OPENAI)
    );
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("stt"))
            .and_then(|stt| stt.get("model"))
            .and_then(|value| value.as_str()),
        Some("gpt-4o-audio-preview")
    );

    let commands = collect_daemon_commands(&mut daemon_rx);
    assert!(commands.iter().any(|command| {
        matches!(
            command,
            DaemonCommand::SetConfigItem { key_path, value_json }
                if key_path == "/audio/stt/model"
                    && value_json == "\"gpt-4o-audio-preview\""
        )
    }));
    assert!(!commands.iter().any(|command| {
        matches!(
            command,
            DaemonCommand::SetConfigItem { key_path, .. } if key_path == "/audio/stt/provider"
        )
    }));
}
