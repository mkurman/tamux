use super::normalize_provider_auth_source_falls_back_for_invalid_values_to_apply::*;
use crate::state::DaemonCommand;
use zorai_shared::providers::*;
#[test]
fn apply_config_json_loads_nested_compaction_settings() {
    let mut model = make_model();

    model.apply_config_json(&serde_json::json!({
        "provider": PROVIDER_ID_OPENAI,
        "providers": {
            "openai": {
                "base_url": "https://api.openai.com/v1",
                "model": "gpt-5.4",
                "auth_source": "api_key",
                "api_transport": "responses"
            }
        },
        "builtin_sub_agents": {
            "weles": {
                "provider": "openai",
                "model": "gpt-5.4-mini",
                "reasoning_effort": "medium",
                "max_concurrent_reviews": 5
            }
        },
        "compaction": {
            "strategy": "custom_model",
            "weles": {
                "provider": "openai",
                "model": "gpt-5.4-mini",
                "reasoning_effort": "medium"
            },
            "custom_model": {
                "provider": "openrouter",
                "base_url": "https://openrouter.ai/api/v1",
                "model": "arcee-ai/trinity-large-thinking",
                "api_key": "sk-compaction",
                "assistant_id": "assistant-compaction",
                "auth_source": "api_key",
                "api_transport": "chat_completions",
                "reasoning_effort": "xhigh",
                "context_window_tokens": 222000
            }
        }
    }));

    assert_eq!(model.config.compaction_strategy, "custom_model");
    assert_eq!(model.config.weles_max_concurrent_reviews, 5);
    assert_eq!(model.config.compaction_weles_provider, PROVIDER_ID_OPENAI);
    assert_eq!(model.config.compaction_weles_model, "gpt-5.4-mini");
    assert_eq!(
        model.config.compaction_custom_provider,
        PROVIDER_ID_OPENROUTER
    );
    assert_eq!(
        model.config.compaction_custom_base_url,
        "https://openrouter.ai/api/v1"
    );
    assert_eq!(
        model.config.compaction_custom_model,
        "arcee-ai/trinity-large-thinking"
    );
    assert_eq!(model.config.compaction_custom_api_key, "sk-compaction");
    assert_eq!(
        model.config.compaction_custom_assistant_id,
        "assistant-compaction"
    );
    assert_eq!(
        model.config.compaction_custom_api_transport,
        "chat_completions"
    );
    assert_eq!(model.config.compaction_custom_reasoning_effort, "xhigh");
    assert_eq!(model.config.compaction_custom_context_window_tokens, 222000);
}

#[test]
fn load_saved_settings_requests_daemon_openai_auth_status() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.load_saved_settings();

    let mut saw_auth_status = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(command, DaemonCommand::GetOpenAICodexAuthStatus) {
            saw_auth_status = true;
            break;
        }
    }

    assert!(saw_auth_status, "expected daemon auth status refresh");
}

#[test]
fn apply_config_json_does_not_consult_local_openai_auth_state() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.chatgpt_auth_available = true;
    model.config.chatgpt_auth_source = Some("stale-local".to_string());

    model.apply_config_json(&serde_json::json!({
        "provider": PROVIDER_ID_OPENAI,
        "providers": {
            "openai": {
                "base_url": "https://api.openai.com/v1",
                "model": "gpt-5.4",
                "auth_source": "chatgpt_subscription",
                "api_transport": "responses"
            }
        }
    }));

    let mut saw_auth_status = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(command, DaemonCommand::GetOpenAICodexAuthStatus) {
            saw_auth_status = true;
            break;
        }
    }

    assert!(saw_auth_status, "expected daemon auth status refresh");
    assert!(model.config.chatgpt_auth_available);
    assert_eq!(
        model.config.chatgpt_auth_source.as_deref(),
        Some("stale-local")
    );
}

#[test]
fn sync_config_to_daemon_does_not_emit_null_api_key_for_advanced_edits() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.agent_config_loaded = true;
    model.config.provider = PROVIDER_ID_OPENAI.to_string();
    model.config.base_url = "https://api.openai.com/v1".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.api_key = "sk-live".to_string();
    model.config.auth_source = "api_key".to_string();
    model.config.api_transport = "responses".to_string();
    model.config.max_context_messages = 123;
    model.config.tui_chat_history_page_size = 222;
    model.config.agent_config_raw = Some(serde_json::json!({
        "provider": "openai",
        "base_url": "https://api.openai.com/v1",
        "model": "gpt-5.4",
        "api_key": "sk-live",
        "auth_source": "api_key",
        "api_transport": "responses",
        "max_context_messages": 100,
        "tui_chat_history_page_size": 100,
        "providers": {
            "openai": {
                "base_url": "https://api.openai.com/v1",
                "model": "gpt-5.4",
                "api_key": "sk-live",
                "auth_source": "api_key",
                "api_transport": "responses",
                "reasoning_effort": ""
            }
        }
    }));

    model.sync_config_to_daemon();

    let mut emitted_max_context_messages = false;
    let mut emitted_tui_chat_history_page_size = false;
    while let Ok(command) = daemon_rx.try_recv() {
        match command {
            DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            } => {
                assert_ne!(
                    key_path, "/api_key",
                    "advanced edits must not null out api_key"
                );
                if key_path == "/max_context_messages" {
                    emitted_max_context_messages = true;
                    assert_eq!(value_json, "123");
                }
                if key_path == "/tui_chat_history_page_size" {
                    emitted_tui_chat_history_page_size = true;
                    assert_eq!(value_json, "222");
                }
            }
            DaemonCommand::SetProviderModel { .. } => {}
            other => panic!("unexpected daemon command: {other:?}"),
        }
    }

    assert!(
        emitted_max_context_messages,
        "expected max_context_messages update to be emitted"
    );
    assert!(
        emitted_tui_chat_history_page_size,
        "expected tui_chat_history_page_size update to be emitted"
    );
}

#[test]
fn sync_config_to_daemon_emits_managed_execution_security_level_for_advanced_toggle() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.agent_config_loaded = true;
    model.config.provider = PROVIDER_ID_OPENAI.to_string();
    model.config.base_url = "https://api.openai.com/v1".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.api_key = "sk-live".to_string();
    model.config.auth_source = "api_key".to_string();
    model.config.api_transport = "responses".to_string();
    model.config.managed_security_level = "yolo".to_string();
    model.config.agent_config_raw = Some(serde_json::json!({
        "provider": "openai",
        "base_url": "https://api.openai.com/v1",
        "model": "gpt-5.4",
        "api_key": "sk-live",
        "auth_source": "api_key",
        "api_transport": "responses",
        "managed_execution": {
            "sandbox_enabled": false,
            "security_level": "highest"
        },
        "providers": {
            "openai": {
                "base_url": "https://api.openai.com/v1",
                "model": "gpt-5.4",
                "api_key": "sk-live",
                "auth_source": "api_key",
                "api_transport": "responses",
                "reasoning_effort": ""
            }
        }
    }));

    model.sync_config_to_daemon();

    let mut emitted_security_level = false;
    while let Ok(command) = daemon_rx.try_recv() {
        match command {
            DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            } => {
                assert_ne!(
                    key_path, "/managed_security_level",
                    "advanced tab should use canonical managed_execution path"
                );
                if key_path == "/managed_execution/security_level" {
                    emitted_security_level = true;
                    assert_eq!(value_json, "\"yolo\"");
                }
            }
            DaemonCommand::SetProviderModel { .. } => {}
            other => panic!("unexpected daemon command: {other:?}"),
        }
    }

    assert!(
        emitted_security_level,
        "expected managed_execution security level update to be emitted"
    );
}

#[test]
fn sync_config_to_daemon_emits_provider_model_and_base_url_items_for_provider_switch() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.agent_config_loaded = true;
    model.config.provider = PROVIDER_ID_OPENROUTER.to_string();
    model.config.base_url = "https://openrouter.ai/api/v1".to_string();
    model.config.model = "arcee-ai/trinity-large-preview:free".to_string();
    model.config.auth_source = "api_key".to_string();
    model.config.api_transport = "chat_completions".to_string();
    model.config.reasoning_effort = "high".to_string();
    model.config.context_window_tokens = 1_000_000;
    model.config.agent_config_raw = Some(serde_json::json!({
        "provider": "minimax-coding-plan",
        "base_url": "https://api.minimax.io/anthropic",
        "model": "MiniMax-M2.7",
        "api_key": "sk-cp-test",
        "auth_source": "api_key",
        "api_transport": "chat_completions",
        "providers": {
            "minimax-coding-plan": {
                "base_url": "https://api.minimax.io/anthropic",
                "model": "MiniMax-M2.7",
                "api_key": "sk-cp-test",
                "auth_source": "api_key",
                "api_transport": "chat_completions",
                "reasoning_effort": "high",
                "context_window_tokens": 205000
            },
            "openrouter": {
                "base_url": "https://openrouter.ai/api/v1",
                "model": "arcee-ai/trinity-large-preview:free",
                "api_key": "sk-or-test",
                "auth_source": "api_key",
                "api_transport": "chat_completions",
                "reasoning_effort": "high",
                "context_window_tokens": 1000000
            }
        }
    }));

    model.sync_config_to_daemon();

    let mut saw_provider = false;
    let mut saw_model = false;
    let mut saw_base_url = false;
    while let Ok(command) = daemon_rx.try_recv() {
        match command {
            DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            } => match key_path.as_str() {
                "/provider" => {
                    saw_provider = true;
                    assert_eq!(value_json, format!("\"{}\"", PROVIDER_ID_OPENROUTER));
                }
                "/model" => {
                    saw_model = true;
                    assert_eq!(value_json, "\"arcee-ai/trinity-large-preview:free\"");
                }
                "/base_url" => {
                    saw_base_url = true;
                    assert_eq!(value_json, "\"https://openrouter.ai/api/v1\"");
                }
                _ => {}
            },
            DaemonCommand::SetProviderModel { .. } => {
                panic!("provider switches should persist through config items")
            }
            other => panic!("unexpected daemon command: {other:?}"),
        }
    }

    assert!(saw_provider, "expected /provider update to be emitted");
    assert!(saw_model, "expected /model update to be emitted");
    assert!(saw_base_url, "expected /base_url update to be emitted");
}

#[test]
fn sync_config_to_daemon_emits_second_provider_switch_after_returning_from_custom() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.agent_config_loaded = true;
    model.config.provider = PROVIDER_ID_MINIMAX_CODING_PLAN.to_string();
    model.config.base_url = "https://api.minimax.io/anthropic".to_string();
    model.config.model = "MiniMax-M2.7".to_string();
    model.config.auth_source = "api_key".to_string();
    model.config.api_transport = "chat_completions".to_string();
    model.config.reasoning_effort = "high".to_string();
    model.config.context_window_tokens = 205_000;
    model.config.agent_config_raw = Some(serde_json::json!({
        "provider": "minimax-coding-plan",
        "base_url": "https://api.minimax.io/anthropic",
        "model": "MiniMax-M2.7",
        "auth_source": "api_key",
        "api_transport": "chat_completions",
        "providers": {
            "minimax-coding-plan": {
                "base_url": "https://api.minimax.io/anthropic",
                "model": "MiniMax-M2.7",
                "auth_source": "api_key",
                "api_transport": "chat_completions",
                "reasoning_effort": "high",
                "context_window_tokens": 205000
            },
            "custom": {
                "base_url": "",
                "model": "",
                "auth_source": "api_key",
                "api_transport": "responses",
                "reasoning_effort": "high",
                "context_window_tokens": 128000
            }
        }
    }));

    model.config.provider = PROVIDER_ID_CUSTOM.to_string();
    model.config.base_url = "http://localhost:11434/v1".to_string();
    model.config.model = "llama3.1".to_string();
    model.config.api_transport = "chat_completions".to_string();
    model.config.context_window_tokens = 128_000;

    model.sync_config_to_daemon();

    while daemon_rx.try_recv().is_ok() {}

    model.config.provider = PROVIDER_ID_MINIMAX_CODING_PLAN.to_string();
    model.config.base_url = "https://api.minimax.io/anthropic".to_string();
    model.config.model = "MiniMax-M2.7".to_string();
    model.config.auth_source = "api_key".to_string();
    model.config.api_transport = "chat_completions".to_string();
    model.config.context_window_tokens = 205_000;

    model.sync_config_to_daemon();

    let mut saw_provider = false;
    let mut saw_model = false;
    let mut saw_base_url = false;
    while let Ok(command) = daemon_rx.try_recv() {
        match command {
            DaemonCommand::SetConfigItem {
                key_path,
                value_json,
            } => match key_path.as_str() {
                "/provider" => {
                    saw_provider = true;
                    assert_eq!(
                        value_json,
                        format!("\"{}\"", PROVIDER_ID_MINIMAX_CODING_PLAN)
                    );
                }
                "/model" => {
                    saw_model = true;
                    assert_eq!(value_json, "\"MiniMax-M2.7\"");
                }
                "/base_url" => {
                    saw_base_url = true;
                    assert_eq!(value_json, "\"https://api.minimax.io/anthropic\"");
                }
                _ => {}
            },
            DaemonCommand::SetProviderModel { .. } => {
                panic!("provider switches should persist through config items")
            }
            other => panic!("unexpected daemon command: {other:?}"),
        }
    }

    assert!(
        saw_provider,
        "expected second provider switch to emit /provider update"
    );
    assert!(
        saw_model,
        "expected second provider switch to emit /model update"
    );
    assert!(
        saw_base_url,
        "expected second provider switch to emit /base_url update"
    );
}

#[test]
fn build_config_patch_value_pins_inherited_rarog_and_weles_when_main_provider_changes_after_setup()
{
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.base_url = "https://models.github.ai/inference".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.api_transport = "chat_completions".to_string();
    model.config.agent_config_raw = Some(serde_json::json!({
        "provider": "openai",
        "base_url": "https://api.openai.com/v1",
        "model": "gpt-4.1",
        "auth_source": "api_key",
        "api_transport": "responses",
        "providers": {
            "openai": {
                "base_url": "https://api.openai.com/v1",
                "model": "gpt-4.1",
                "auth_source": "api_key",
                "api_transport": "responses"
            },
            "github-copilot": {
                "base_url": "https://models.github.ai/inference",
                "model": "gpt-5.4",
                "auth_source": "github_copilot",
                "api_transport": "chat_completions"
            }
        },
        "concierge": {
            "enabled": true,
            "detail_level": "context_summary"
        },
        "builtin_sub_agents": {
            "weles": {
                "enabled": true
            }
        }
    }));

    let json = model.build_config_patch_value();

    assert_eq!(json["provider"], PROVIDER_ID_GITHUB_COPILOT);
    assert_eq!(json["model"], "gpt-5.4");
    assert_eq!(json["concierge"]["provider"], "openai");
    assert_eq!(json["concierge"]["model"], "gpt-4.1");
    assert_eq!(json["builtin_sub_agents"]["weles"]["provider"], "openai");
    assert_eq!(json["builtin_sub_agents"]["weles"]["model"], "gpt-4.1");
}
