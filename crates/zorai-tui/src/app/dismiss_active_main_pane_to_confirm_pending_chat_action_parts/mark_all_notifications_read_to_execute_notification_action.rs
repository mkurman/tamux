use super::*;
impl TuiModel {
    pub(crate) fn mark_all_notifications_read(&mut self) {
        let ids = self
            .notifications
            .active_items()
            .into_iter()
            .filter(|item| item.read_at.is_none())
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            self.mark_notification_read(&id);
        }
    }

    pub(crate) fn archive_read_notifications(&mut self) {
        let ids = self
            .notifications
            .active_items()
            .into_iter()
            .filter(|item| item.read_at.is_some())
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();
        for id in ids {
            self.archive_notification(&id);
        }
    }

    pub(crate) fn execute_notification_row_action(
        &mut self,
        notification_id: &str,
        action_index: usize,
    ) {
        match action_index {
            0 => self.toggle_notification_expand(notification_id.to_string()),
            1 => self.mark_notification_read(notification_id),
            2 => self.archive_notification(notification_id),
            3 => self.delete_notification(notification_id),
            other => {
                self.execute_notification_action(notification_id, "", Some(other.saturating_sub(4)))
            }
        }
    }

    pub(crate) fn execute_notification_action(
        &mut self,
        notification_id: &str,
        action_id: &str,
        action_index: Option<usize>,
    ) {
        let Some(notification) = self
            .notifications
            .all_items()
            .iter()
            .find(|item| item.id == notification_id)
            .cloned()
        else {
            return;
        };
        let action = action_index
            .and_then(|index| notification.actions.get(index).cloned())
            .or_else(|| {
                notification
                    .actions
                    .iter()
                    .find(|candidate| candidate.id == action_id)
                    .cloned()
            });
        let Some(action) = action else {
            return;
        };
        self.mark_notification_read(notification_id);
        match action.action_type.as_str() {
            "open_thread" => {
                if let Some(thread_id) = action.target.as_deref() {
                    self.close_top_modal();
                    self.open_thread_conversation(thread_id.to_string());
                    self.status_line = format!("Opened thread {}", thread_id);
                }
            }
            "open_plugin_settings" => {
                self.open_settings_tab(SettingsTab::Plugins);
                if let Some(plugin_name) = action.target.as_deref() {
                    let selected_index = self
                        .plugin_settings
                        .plugins
                        .iter()
                        .position(|plugin| plugin.name == plugin_name);
                    if let Some(index) = selected_index {
                        self.plugin_settings.selected_index = index;
                    }
                    self.plugin_settings.list_mode = selected_index.is_none();
                    self.plugin_settings.detail_cursor = 0;
                    self.plugin_settings.test_result = None;
                    self.plugin_settings.schema_fields.clear();
                    self.plugin_settings.settings_values.clear();
                    self.send_daemon_command(DaemonCommand::PluginGet(plugin_name.to_string()));
                    self.send_daemon_command(DaemonCommand::PluginGetSettings(
                        plugin_name.to_string(),
                    ));
                    self.status_line = format!("Opened plugin settings for {}", plugin_name);
                }
            }
            _ => {
                self.status_line = format!("Notification action unavailable: {}", action.label);
            }
        }
    }
}
