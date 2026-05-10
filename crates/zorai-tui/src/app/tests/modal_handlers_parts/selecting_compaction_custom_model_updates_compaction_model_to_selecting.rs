use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn selecting_compaction_custom_model_updates_compaction_model() {
    let (mut model, _daemon_rx) = make_model();
    model.config.compaction_custom_provider = PROVIDER_ID_OPENAI.to_string();
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "gpt-5.4".to_string(),
                name: Some("GPT-5.4".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
            crate::state::config::FetchedModel {
                id: "gpt-5.4-mini".to_string(),
                name: Some("GPT-5.4 Mini".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
        ]));

    model.settings_picker_target = Some(SettingsPickerTarget::CompactionCustomModel);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    model.modal.reduce(modal::ModalAction::Navigate(1));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.compaction_custom_model, "gpt-5.4-mini");
}

#[test]
fn selecting_audio_stt_provider_updates_audio_provider_and_opens_model_picker() {
    let (mut model, _daemon_rx) = make_model();
    model.auth.entries = vec![
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_OPENAI.to_string(),
            provider_name: "OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-5.4".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_AZURE_OPENAI.to_string(),
            provider_name: "Azure OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-4.1".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_XAI.to_string(),
            provider_name: "xAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "grok-4".to_string(),
        },
    ];
    model.config.agent_config_raw = Some(serde_json::json!({
        "audio": {
            "stt": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "whisper-1"
            }
        }
    }));

    let target_index = widgets::provider_picker::available_audio_provider_defs(
        &model.auth,
        AudioToolKind::SpeechToText,
    )
    .iter()
    .position(|provider| provider.id == PROVIDER_ID_XAI)
    .expect("provider to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::AudioSttProvider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_audio_provider_defs(
            &model.auth,
            AudioToolKind::SpeechToText,
        )
        .len(),
    );
    if target_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(target_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("stt"))
            .and_then(|stt| stt.get("provider"))
            .and_then(|value| value.as_str()),
        Some(PROVIDER_ID_XAI)
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
        Some("grok-4.3")
    );
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
}

#[test]
fn selecting_audio_stt_model_updates_audio_model() {
    let (mut model, _daemon_rx) = make_model();
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
                id: "gpt-4o-transcribe".to_string(),
                name: Some("GPT-4o Transcribe".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
            crate::state::config::FetchedModel {
                id: "whisper-1".to_string(),
                name: Some("Whisper 1".to_string()),
                context_window: None,
                pricing: None,
                metadata: None,
            },
        ]));

    model.settings_picker_target = Some(SettingsPickerTarget::AudioSttModel);
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
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("stt"))
            .and_then(|stt| stt.get("model"))
            .and_then(|value| value.as_str()),
        Some("gpt-4o-transcribe")
    );
}

#[test]
fn selecting_audio_tts_provider_updates_audio_provider_and_opens_model_picker() {
    let (mut model, _daemon_rx) = make_model();
    model.auth.entries = vec![
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_OPENAI.to_string(),
            provider_name: "OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-5.4".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_AZURE_OPENAI.to_string(),
            provider_name: "Azure OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-4.1".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_XAI.to_string(),
            provider_name: "xAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "grok-4".to_string(),
        },
    ];
    model.config.agent_config_raw = Some(serde_json::json!({
        "audio": {
            "tts": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "gpt-4o-mini-tts"
            }
        }
    }));

    let target_index = widgets::provider_picker::available_audio_provider_defs(
        &model.auth,
        AudioToolKind::TextToSpeech,
    )
    .iter()
    .position(|provider| provider.id == PROVIDER_ID_XAI)
    .expect("provider to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::AudioTtsProvider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_audio_provider_defs(
            &model.auth,
            AudioToolKind::TextToSpeech,
        )
        .len(),
    );
    if target_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(target_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("tts"))
            .and_then(|tts| tts.get("provider"))
            .and_then(|value| value.as_str()),
        Some(PROVIDER_ID_XAI)
    );
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("tts"))
            .and_then(|tts| tts.get("model"))
            .and_then(|value| value.as_str()),
        Some("grok-4.3")
    );
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
}

#[test]
fn authenticated_provider_picker_lists_xai() {
    let (mut model, _daemon_rx) = make_model();
    model.auth.entries = vec![
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_OPENAI.to_string(),
            provider_name: "OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-5.4".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_XAI.to_string(),
            provider_name: "xAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "grok-4".to_string(),
        },
    ];

    let providers = widgets::provider_picker::available_provider_defs(&model.auth);

    assert!(providers
        .iter()
        .any(|provider| provider.id == PROVIDER_ID_XAI));
}

#[test]
fn selecting_audio_tts_model_updates_audio_model() {
    let (mut model, _daemon_rx) = make_model();
    model.config.agent_config_raw = Some(serde_json::json!({
        "audio": {
            "tts": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "gpt-4o-mini-tts"
            }
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "gpt-4o-mini-tts".to_string(),
                name: Some("GPT-4o Mini TTS".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
            crate::state::config::FetchedModel {
                id: "tts-1".to_string(),
                name: Some("TTS 1".to_string()),
                context_window: None,
                pricing: None,
                metadata: None,
            },
        ]));

    model.settings_picker_target = Some(SettingsPickerTarget::AudioTtsModel);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    model.modal.reduce(modal::ModalAction::Navigate(1));

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
            .and_then(|raw| raw.get("audio"))
            .and_then(|audio| audio.get("tts"))
            .and_then(|tts| tts.get("model"))
            .and_then(|value| value.as_str()),
        Some("tts-1")
    );
}

#[test]
fn selecting_image_generation_provider_updates_image_provider_and_opens_model_picker() {
    let (mut model, _daemon_rx) = make_model();
    model.auth.entries = vec![
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_OPENAI.to_string(),
            provider_name: "OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-5.4".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_OPENROUTER.to_string(),
            provider_name: "OpenRouter".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "openai/gpt-5.4".to_string(),
        },
    ];
    model.config.agent_config_raw = Some(serde_json::json!({
        "image": {
            "generation": {
                "provider": PROVIDER_ID_OPENAI,
                "model": "gpt-image-1"
            }
        }
    }));

    let target_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_OPENROUTER)
        .expect("provider to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::ImageGenerationProvider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_provider_defs(&model.auth).len(),
    );
    if target_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(target_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("image"))
            .and_then(|image| image.get("generation"))
            .and_then(|generation| generation.get("provider"))
            .and_then(|value| value.as_str()),
        Some(PROVIDER_ID_OPENROUTER)
    );
    assert_eq!(
        model
            .config
            .agent_config_raw
            .as_ref()
            .and_then(|raw| raw.get("image"))
            .and_then(|image| image.get("generation"))
            .and_then(|generation| generation.get("model"))
            .and_then(|value| value.as_str()),
        Some("openai/gpt-image-1")
    );
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
}
