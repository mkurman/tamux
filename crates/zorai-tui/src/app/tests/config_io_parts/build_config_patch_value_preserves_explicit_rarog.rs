use super::normalize_provider_auth_source_falls_back_for_invalid_values_to_apply::*;
use zorai_shared::providers::*;
#[test]
fn build_config_patch_value_keeps_inheritance_during_first_time_main_provider_setup() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.base_url = "https://models.github.ai/inference".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.api_transport = "chat_completions".to_string();
    model.config.agent_config_raw = Some(serde_json::json!({
        "providers": {
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
    assert!(json["concierge"].get("provider").is_none());
    assert!(json["concierge"].get("model").is_none());
    assert!(json["builtin_sub_agents"]["weles"]
        .get("provider")
        .is_none());
    assert!(json["builtin_sub_agents"]["weles"].get("model").is_none());
}

#[test]
fn build_config_patch_value_preserves_explicit_rarog_and_weles_overrides_on_main_switch() {
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
            },
            "custom-rarog": {
                "base_url": "https://rarog.example/v1",
                "model": "rarog-model",
                "auth_source": "api_key",
                "api_transport": "chat_completions"
            },
            "custom-weles": {
                "base_url": "https://weles.example/v1",
                "model": "weles-model",
                "auth_source": "api_key",
                "api_transport": "chat_completions"
            }
        },
        "concierge": {
            "enabled": true,
            "detail_level": "context_summary",
            "provider": "custom-rarog",
            "model": "rarog-model"
        },
        "builtin_sub_agents": {
            "weles": {
                "enabled": true,
                "provider": "custom-weles",
                "model": "weles-model"
            }
        }
    }));

    let json = model.build_config_patch_value();

    assert_eq!(json["concierge"]["provider"], "custom-rarog");
    assert_eq!(json["concierge"]["model"], "rarog-model");
    assert_eq!(
        json["builtin_sub_agents"]["weles"]["provider"],
        "custom-weles"
    );
    assert_eq!(json["builtin_sub_agents"]["weles"]["model"], "weles-model");
}
