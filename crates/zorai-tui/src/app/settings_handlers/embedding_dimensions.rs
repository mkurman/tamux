use super::*;
impl TuiModel {
    pub(crate) fn set_embedding_dimensions_config(&mut self, dimensions: u32) {
        self.send_daemon_command(DaemonCommand::SetConfigItem {
            key_path: "/semantic/embedding/dimensions".to_string(),
            value_json: dimensions.to_string(),
        });
        if let Some(ref mut raw) = self.config.agent_config_raw {
            if raw.get("semantic").is_none() {
                raw["semantic"] = serde_json::json!({});
            }
            if raw["semantic"].get("embedding").is_none() {
                raw["semantic"]["embedding"] = serde_json::json!({});
            }
            raw["semantic"]["embedding"]["dimensions"] = serde_json::json!(dimensions);
        }
    }
}
