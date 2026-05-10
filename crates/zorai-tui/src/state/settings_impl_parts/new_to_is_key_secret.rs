use super::*;
impl PluginSettingsState {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            selected_index: 0,
            schema_fields: Vec::new(),
            settings_values: Vec::new(),
            list_mode: true,
            test_result: None,
            loading: false,
            detail_cursor: 0,
        }
    }

    pub fn selected_plugin(&self) -> Option<&PluginListItem> {
        self.plugins.get(self.selected_index)
    }

    /// field_count in detail mode = number of schema fields + action buttons
    pub fn detail_field_count(&self) -> usize {
        self.schema_fields.len()
            + if self.selected_plugin().map_or(false, |p| p.has_api) {
                1
            } else {
                0
            }
            + if self.selected_plugin().map_or(false, |p| p.has_auth) {
                1
            } else {
                0
            }
    }

    /// Get the current value for a schema field key.
    pub fn value_for_key(&self, key: &str) -> Option<&str> {
        self.settings_values
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, v, _)| v.as_str())
    }

    /// Check if a key is a secret field.
    pub fn is_key_secret(&self, key: &str) -> bool {
        self.settings_values
            .iter()
            .find(|(k, _, _)| k == key)
            .map_or(false, |(_, _, is_secret)| *is_secret)
    }
}
