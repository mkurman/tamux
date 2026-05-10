use super::*;
impl TuiModel {
    pub(crate) fn subagent_editor_system_prompt_override(
        &self,
        entry: &crate::state::SubAgentEntry,
        raw: &serde_json::Value,
    ) -> String {
        let raw_prompt = raw
            .get("system_prompt")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let Some(config_raw) = self.config.agent_config_raw.as_ref() else {
            return raw_prompt.to_string();
        };

        if entry.id == "weles_builtin" && entry.builtin {
            if let Some(configured_override) = config_raw
                .get("builtin_sub_agents")
                .and_then(|value| value.get("weles"))
                .and_then(|value| value.get("system_prompt"))
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                return configured_override.to_string();
            }

            let main_prompt = config_raw
                .get("system_prompt")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if !main_prompt.is_empty() && raw_prompt == main_prompt {
                return String::new();
            }
        }

        raw_prompt.to_string()
    }

    pub(crate) fn close_subagent_editor(&mut self) {
        self.subagents.editor = None;
        self.settings.reduce(SettingsAction::CancelEdit);
    }
}
