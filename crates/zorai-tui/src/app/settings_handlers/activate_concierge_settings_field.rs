use super::*;
impl TuiModel {
    pub(super) fn activate_concierge_settings_field(&mut self, field: &str) -> bool {
        match field {
            "subagent_list" => {
                self.subagents.actions_focused = false;
            }
            "concierge_enabled" => {
                self.concierge.enabled = !self.concierge.enabled;
                self.send_concierge_config();
            }
            "concierge_detail_level" => {
                let levels = [
                    "minimal",
                    "context_summary",
                    "proactive_triage",
                    "daily_briefing",
                ];
                let current_idx = levels
                    .iter()
                    .position(|level| *level == self.concierge.detail_level)
                    .unwrap_or(0);
                self.concierge.detail_level = levels[(current_idx + 1) % levels.len()].to_string();
                self.send_concierge_config();
            }
            "concierge_provider" => {
                self.settings_picker_target = Some(SettingsPickerTarget::ConciergeProvider);
                self.execute_command("provider");
            }
            "concierge_model" => {
                let provider_id = self
                    .concierge
                    .provider
                    .clone()
                    .unwrap_or_else(|| self.config.provider.clone());
                let (base_url, api_key, auth_source) = self.provider_auth_snapshot(&provider_id);
                self.open_provider_backed_model_picker(
                    SettingsPickerTarget::ConciergeModel,
                    provider_id,
                    base_url,
                    api_key,
                    auth_source,
                );
            }
            "concierge_reasoning_effort" => {
                self.settings_picker_target = Some(SettingsPickerTarget::ConciergeReasoningEffort);
                self.execute_command("effort");
            }
            "concierge_openrouter_provider_order" => self
                .open_concierge_openrouter_provider_picker(
                    SettingsPickerTarget::ConciergeOpenRouterPreferredProviders,
                ),
            "concierge_openrouter_provider_ignore" => self
                .open_concierge_openrouter_provider_picker(
                    SettingsPickerTarget::ConciergeOpenRouterExcludedProviders,
                ),
            "concierge_openrouter_allow_fallbacks" => {
                if self.concierge.provider.as_deref() == Some(PROVIDER_ID_OPENROUTER) {
                    self.concierge.openrouter_allow_fallbacks =
                        !self.concierge.openrouter_allow_fallbacks;
                    self.send_concierge_config();
                } else {
                    self.status_line =
                        "OpenRouter provider routing only applies to OpenRouter agents".to_string();
                }
            }
            "managed_security_level" => {
                let levels = ["highest", "moderate", "lowest", "yolo"];
                let current_idx = levels
                    .iter()
                    .position(|level| *level == self.config.managed_security_level)
                    .unwrap_or(2);
                self.config.managed_security_level =
                    levels[(current_idx + 1) % levels.len()].to_string();
                self.sync_config_to_daemon();
            }
            _ => return false,
        }
        true
    }
}
