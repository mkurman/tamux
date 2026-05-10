use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn selecting_custom_provider_focuses_model_field_for_inline_entry() {
    let (mut model, _daemon_rx) = make_model();
    let custom_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_CUSTOM)
        .expect("custom provider to exist");

    model
        .settings
        .reduce(SettingsAction::SwitchTab(SettingsTab::Provider));
    model.settings_picker_target = Some(SettingsPickerTarget::Provider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_provider_defs(&model.auth).len(),
    );
    if custom_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(custom_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.provider, PROVIDER_ID_CUSTOM);
    assert_eq!(model.settings.current_field_name(), "model");
    assert_eq!(model.settings.field_cursor(), 3);
}

#[test]
fn provider_picker_filters_to_authenticated_entries_plus_custom() {
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
            provider_id: "groq".to_string(),
            provider_name: "Groq".to_string(),
            authenticated: false,
            auth_source: "api_key".to_string(),
            model: "llama".to_string(),
        },
    ];

    let defs = widgets::provider_picker::available_provider_defs(&model.auth);
    assert!(defs
        .iter()
        .any(|provider| provider.id == PROVIDER_ID_OPENAI));
    assert!(defs
        .iter()
        .any(|provider| provider.id == PROVIDER_ID_CUSTOM));
    assert!(!defs.iter().any(|provider| provider.id == "groq"));
}

#[test]
fn model_command_skips_remote_fetch_for_static_provider_catalogs() {
    let (mut model, mut daemon_rx) = make_model();
    model.config.provider = PROVIDER_ID_ALIBABA_CODING_PLAN.to_string();
    model.config.base_url = "https://coding-intl.dashscope.aliyuncs.com/v1".to_string();
    model.config.model = "qwen3.6-plus".to_string();
    model.config.auth_source = "api_key".to_string();
    model.config.api_key = "dashscope-key".to_string();

    model.execute_command("model");

    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    while let Ok(command) = daemon_rx.try_recv() {
        if let DaemonCommand::FetchModels { .. } = command {
            panic!("static providers should not trigger remote model fetches");
        }
    }
}

#[test]
fn provider_picker_skips_remote_fetch_for_static_provider_catalogs() {
    let (mut model, mut daemon_rx) = make_model();
    model.auth.entries = vec![crate::state::auth::ProviderAuthEntry {
        provider_id: PROVIDER_ID_ALIBABA_CODING_PLAN.to_string(),
        provider_name: "Alibaba Coding Plan".to_string(),
        authenticated: true,
        auth_source: "api_key".to_string(),
        model: "qwen3.6-plus".to_string(),
    }];

    let alibaba_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_ALIBABA_CODING_PLAN)
        .expect("alibaba-coding-plan to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::Provider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_provider_defs(&model.auth).len(),
    );
    if alibaba_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(alibaba_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.provider, PROVIDER_ID_ALIBABA_CODING_PLAN);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    while let Ok(command) = daemon_rx.try_recv() {
        if let DaemonCommand::FetchModels { .. } = command {
            panic!("static providers should not trigger remote model fetches");
        }
    }
}

#[test]
fn provider_picker_fetches_remote_models_for_chutes() {
    let (mut model, mut daemon_rx) = make_model();
    model.auth.entries = vec![crate::state::auth::ProviderAuthEntry {
        provider_id: PROVIDER_ID_CHUTES.to_string(),
        provider_name: "Chutes".to_string(),
        authenticated: true,
        auth_source: "api_key".to_string(),
        model: "deepseek-ai/DeepSeek-R1".to_string(),
    }];
    model.config.agent_config_raw = Some(serde_json::json!({
        "providers": {
            PROVIDER_ID_CHUTES: {
                "base_url": "https://llm.chutes.ai/v1",
                "api_key": "chutes-key",
                "auth_source": "api_key"
            }
        }
    }));

    let chutes_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_CHUTES)
        .expect("chutes to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::Provider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_provider_defs(&model.auth).len(),
    );
    if chutes_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(chutes_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.provider, PROVIDER_ID_CHUTES);
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
        other => panic!("expected FetchModels for Chutes provider picker, got {other:?}"),
    }
}

#[test]
fn provider_picker_uses_chatgpt_subscription_auth_without_remote_model_fetch() {
    let (mut model, mut daemon_rx) = make_model();
    model.auth.entries = vec![crate::state::auth::ProviderAuthEntry {
        provider_id: PROVIDER_ID_OPENAI.to_string(),
        provider_name: "OpenAI".to_string(),
        authenticated: true,
        auth_source: "chatgpt_subscription".to_string(),
        model: "gpt-5.4".to_string(),
    }];
    model.config.chatgpt_auth_available = true;
    model.config.chatgpt_auth_source = Some("zorai-daemon".to_string());

    let openai_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_OPENAI)
        .expect("openai to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::Provider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_provider_defs(&model.auth).len(),
    );
    if openai_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(openai_index as i32));
    }

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.provider, PROVIDER_ID_OPENAI);
    assert_eq!(model.config.auth_source, "chatgpt_subscription");
    assert_eq!(model.config.api_transport, "responses");
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    while let Ok(command) = daemon_rx.try_recv() {
        if let DaemonCommand::FetchModels { .. } = command {
            panic!("chatgpt subscription auth should not trigger remote model fetches");
        }
    }
}

#[test]
fn selecting_compaction_weles_provider_updates_provider_and_opens_model_picker() {
    let (mut model, mut daemon_rx) = make_model();
    model.auth.entries = vec![
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_OPENAI.to_string(),
            provider_name: "OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-5.4".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_ALIBABA_CODING_PLAN.to_string(),
            provider_name: "Alibaba Coding Plan".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "qwen3.6-plus".to_string(),
        },
    ];

    let target_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_ALIBABA_CODING_PLAN)
        .expect("provider to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::CompactionWelesProvider);
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
        model.config.compaction_weles_provider,
        PROVIDER_ID_ALIBABA_CODING_PLAN
    );
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    while let Ok(command) = daemon_rx.try_recv() {
        if let DaemonCommand::FetchModels { .. } = command {
            panic!("static providers should not trigger remote model fetches");
        }
    }
}

#[test]
fn selecting_compaction_weles_model_updates_compaction_model() {
    let (mut model, _daemon_rx) = make_model();
    model.config.compaction_weles_provider = PROVIDER_ID_OPENAI.to_string();
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

    model.settings_picker_target = Some(SettingsPickerTarget::CompactionWelesModel);
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
    assert_eq!(model.config.compaction_weles_model, "gpt-5.4-mini");
}

#[test]
fn selecting_embedding_model_applies_dimension_from_model_settings() {
    let (mut model, mut daemon_rx) = make_model();
    model.config.agent_config_raw = Some(serde_json::json!({
        "semantic": {
            "embedding": {
                "provider": PROVIDER_ID_DEEPSEEK,
                "model": "old-embedding-model",
                "dimensions": 1536
            }
        }
    }));
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            crate::state::config::FetchedModel {
                id: "vendor/new-embed-model".to_string(),
                name: Some("New Embed Model".to_string()),
                context_window: Some(8192),
                pricing: None,
                metadata: Some(serde_json::json!({
                    "architecture": {
                        "output_modalities": ["embeddings"]
                    },
                    "settings": {
                        "dimensions": 2048
                    }
                })),
            },
        ]));

    model.settings_picker_target = Some(SettingsPickerTarget::EmbeddingModel);
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
        model.config.semantic_embedding_model(),
        "vendor/new-embed-model"
    );
    assert_eq!(model.config.semantic_embedding_dimensions(), 2048);

    assert!(matches!(
        daemon_rx.try_recv().expect("expected embedding model command"),
        DaemonCommand::SetConfigItem {
            key_path,
            value_json,
        } if key_path == "/semantic/embedding/model" && value_json == "\"vendor/new-embed-model\""
    ));
    assert!(matches!(
        daemon_rx.try_recv().expect("expected embedding dimensions command"),
        DaemonCommand::SetConfigItem {
            key_path,
            value_json,
        } if key_path == "/semantic/embedding/dimensions" && value_json == "2048"
    ));
}

#[test]
fn selecting_compaction_custom_provider_updates_provider_and_opens_model_picker() {
    let (mut model, mut daemon_rx) = make_model();
    model.auth.entries = vec![
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_OPENAI.to_string(),
            provider_name: "OpenAI".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "gpt-5.4".to_string(),
        },
        crate::state::auth::ProviderAuthEntry {
            provider_id: PROVIDER_ID_ALIBABA_CODING_PLAN.to_string(),
            provider_name: "Alibaba Coding Plan".to_string(),
            authenticated: true,
            auth_source: "api_key".to_string(),
            model: "qwen3.6-plus".to_string(),
        },
    ];

    let target_index = widgets::provider_picker::available_provider_defs(&model.auth)
        .iter()
        .position(|provider| provider.id == PROVIDER_ID_ALIBABA_CODING_PLAN)
        .expect("provider to exist");

    model.settings_picker_target = Some(SettingsPickerTarget::CompactionCustomProvider);
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
        model.config.compaction_custom_provider,
        PROVIDER_ID_ALIBABA_CODING_PLAN
    );
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    while let Ok(command) = daemon_rx.try_recv() {
        if let DaemonCommand::FetchModels { .. } = command {
            panic!("static providers should not trigger remote model fetches");
        }
    }
}

#[test]
fn selecting_compaction_custom_provider_copies_current_provider_transport() {
    let (mut model, _daemon_rx) = make_model();
    model.auth.entries = vec![crate::state::auth::ProviderAuthEntry {
        provider_id: PROVIDER_ID_QWEN.to_string(),
        provider_name: "Qwen".to_string(),
        authenticated: true,
        auth_source: "api_key".to_string(),
        model: "qwen-max".to_string(),
    }];
    model.config.provider = PROVIDER_ID_QWEN.to_string();
    model.config.auth_source = "api_key".to_string();
    model.config.api_transport = "chat_completions".to_string();

    model.settings_picker_target = Some(SettingsPickerTarget::CompactionCustomProvider);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
    model.modal.set_picker_item_count(
        widgets::provider_picker::available_provider_defs(&model.auth).len(),
    );

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ProviderPicker,
    );

    assert!(!quit);
    assert_eq!(model.config.compaction_custom_provider, PROVIDER_ID_QWEN);
    assert_eq!(
        model.config.compaction_custom_api_transport,
        "chat_completions"
    );
}
