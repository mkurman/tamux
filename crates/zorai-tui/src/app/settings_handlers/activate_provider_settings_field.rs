use super::*;
use crate::providers;
impl TuiModel {
    pub(super) fn activate_provider_settings_field(&mut self, field: &str) -> bool {
        match field {
            "provider" => {
                self.open_provider_picker(SettingsPickerTarget::Provider);
            }
            "model" => {
                if self.config.provider == PROVIDER_ID_CUSTOM {
                    self.begin_custom_model_edit();
                } else {
                    self.open_provider_backed_model_picker(
                        SettingsPickerTarget::Model,
                        self.config.provider.clone(),
                        self.config.base_url.clone(),
                        self.config.api_key.clone(),
                        self.config.auth_source.clone(),
                    );
                }
            }
            "auth_source" => {
                let supported = providers::supported_auth_sources_for(&self.config.provider);
                let current_idx = supported
                    .iter()
                    .position(|source| *source == self.config.auth_source)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % supported.len().max(1);
                self.config.auth_source = supported
                    .get(next_idx)
                    .copied()
                    .unwrap_or("api_key")
                    .to_string();
                if self.config.provider == PROVIDER_ID_OPENAI
                    && self.config.auth_source == "chatgpt_subscription"
                {
                    self.refresh_openai_auth_status();
                }
                self.refresh_provider_models_for_current_auth();
                if self.config.provider == PROVIDER_ID_OPENAI
                    && self.config.auth_source == "chatgpt_subscription"
                {
                    self.config.api_transport = "responses".to_string();
                }
                self.sync_config_to_daemon();
            }
            "api_transport" => {
                if let Some(fixed_transport) =
                    providers::fixed_transport_for_model(&self.config.provider, &self.config.model)
                {
                    let transport_label = match fixed_transport {
                        "native_assistant" => "native assistant",
                        "anthropic_messages" => "anthropic messages",
                        "responses" => "responses",
                        _ => "chat completions",
                    };
                    self.status_line = format!("This model uses {transport_label} only.");
                    return true;
                }
                if providers::uses_fixed_anthropic_messages(
                    &self.config.provider,
                    &self.config.model,
                ) {
                    self.status_line =
                        "This provider uses the Anthropic messages protocol.".to_string();
                    return true;
                }
                let supported = providers::supported_transports_for(&self.config.provider);
                if supported.len() <= 1 {
                    let only = supported.first().copied().unwrap_or("chat_completions");
                    let transport_label = match only {
                        "native_assistant" => "native assistant",
                        "anthropic_messages" => "anthropic messages",
                        "responses" => "responses",
                        _ => "chat completions",
                    };
                    let provider_name = providers::find_by_id(&self.config.provider)
                        .map(|def| def.name)
                        .unwrap_or("This provider");
                    self.status_line = format!("{provider_name} supports {transport_label} only.");
                    return true;
                }
                let current_idx = supported
                    .iter()
                    .position(|transport| *transport == self.config.api_transport)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % supported.len().max(1);
                self.config.api_transport = supported
                    .get(next_idx)
                    .copied()
                    .unwrap_or("chat_completions")
                    .to_string();
                if self.config.provider == PROVIDER_ID_OPENAI
                    && self.config.auth_source == "chatgpt_subscription"
                {
                    self.config.api_transport = "responses".to_string();
                }
                self.sync_config_to_daemon();
            }
            "assistant_id" => self
                .settings
                .start_editing("assistant_id", &self.config.assistant_id.clone()),
            "reasoning_effort" => self.execute_command("effort"),
            "base_url" => self
                .settings
                .start_editing("base_url", &self.config.base_url.clone()),
            "openrouter_provider_order" => self.open_openrouter_provider_picker(
                SettingsPickerTarget::OpenRouterPreferredProviders,
            ),
            "openrouter_provider_ignore" => self
                .open_openrouter_provider_picker(SettingsPickerTarget::OpenRouterExcludedProviders),
            "openrouter_allow_fallbacks" => {
                if self.config.provider == PROVIDER_ID_OPENROUTER {
                    self.config.openrouter_allow_fallbacks =
                        !self.config.openrouter_allow_fallbacks;
                    self.sync_config_to_daemon();
                } else {
                    self.status_line =
                        "OpenRouter provider routing only applies to OpenRouter".to_string();
                }
            }
            "openrouter_response_cache_enabled" => {
                if self.config.provider == PROVIDER_ID_OPENROUTER {
                    self.config.openrouter_response_cache_enabled =
                        !self.config.openrouter_response_cache_enabled;
                    self.sync_config_to_daemon();
                } else {
                    self.status_line =
                        "OpenRouter response caching only applies to OpenRouter".to_string();
                }
            }
            _ => return false,
        }
        true
    }
}
