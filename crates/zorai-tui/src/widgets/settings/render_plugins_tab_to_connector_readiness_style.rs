use super::*;
use crate::state::settings::{PluginListItem, PluginSettingsState, SettingsState};
use crate::theme::ThemeTokens;
use ratatui::text::{Line, Span};
pub(crate) fn render_plugins_tab<'a>(
    settings: &'a SettingsState,
    plugin_state: &PluginSettingsState,
    _content_width: u16,
    theme: &ThemeTokens,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    if plugin_state.list_mode {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled("  Plugins", theme.fg_active)));
        lines.push(Line::from(Span::styled(
            "  Manage installed plugins and their settings.",
            theme.fg_dim,
        )));
        lines.push(Line::raw(""));

        if plugin_state.plugins.is_empty() {
            lines.push(Line::from(Span::styled(
                "  No plugins. Run `zorai plugin add <name>` to install.",
                theme.fg_dim,
            )));
            return lines;
        }

        for (i, plugin) in plugin_state.plugins.iter().enumerate() {
            let is_selected = i == plugin_state.selected_index;
            let marker = if is_selected { "> " } else { "  " };
            let checkbox = if plugin.enabled { "[x]" } else { "[ ]" };
            let auth_status = connector_readiness_label(plugin);
            let auth_style = connector_readiness_style(plugin, theme);
            let name_style = if is_selected {
                theme.accent_primary
            } else if plugin.enabled {
                theme.fg_active
            } else {
                theme.fg_dim
            };
            let meta_style = theme.fg_dim;

            lines.push(Line::from(vec![
                Span::styled(
                    marker,
                    if is_selected {
                        theme.accent_primary
                    } else {
                        theme.fg_dim
                    },
                ),
                Span::styled(
                    format!("{} ", checkbox),
                    if plugin.enabled {
                        theme.accent_primary
                    } else {
                        meta_style
                    },
                ),
                Span::styled(plugin.name.clone(), name_style),
                Span::styled(format!("  v{}", plugin.version), meta_style),
                Span::styled(
                    plugin
                        .connector_kind
                        .as_ref()
                        .map(|kind| format!("  [{}]", kind))
                        .unwrap_or_default(),
                    meta_style,
                ),
                Span::styled(format!("  {}", auth_status), auth_style),
            ]));
        }
    } else {
        let Some(plugin) = plugin_state.selected_plugin() else {
            lines.push(Line::from(Span::styled(
                "  No plugin selected.",
                theme.fg_dim,
            )));
            return lines;
        };

        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} v{}", plugin.name, plugin.version),
                theme.fg_active,
            ),
            Span::styled("  [Esc] Back", theme.fg_dim),
        ]));
        if let Some(ref desc) = plugin.description {
            lines.push(Line::from(Span::styled(
                format!("  {}", desc),
                theme.fg_dim,
            )));
        }
        lines.push(Line::raw(""));
        if let Some(kind) = plugin.connector_kind.as_deref() {
            lines.push(Line::from(Span::styled(
                format!("  Connector: {}", kind),
                theme.fg_active,
            )));
            lines.push(Line::from(Span::styled(
                format!("  Readiness: {}", connector_readiness_label(plugin)),
                connector_readiness_style(plugin, theme),
            )));
            if let Some(message) = plugin.readiness_message.as_deref() {
                lines.push(Line::from(Span::styled(
                    format!("  Status: {}", message),
                    theme.fg_dim,
                )));
            }
            if let Some(hint) = plugin.recovery_hint.as_deref() {
                lines.push(Line::from(Span::styled(
                    format!("  Recovery: {}", hint),
                    theme.fg_dim,
                )));
            }
            if let Some(hint) = plugin.setup_hint.as_deref() {
                lines.push(Line::from(Span::styled(
                    format!("  Setup: {}", hint),
                    theme.fg_dim,
                )));
            }
            if let Some(docs_path) = plugin.docs_path.as_deref() {
                lines.push(Line::from(Span::styled(
                    format!("  Docs: {}", docs_path),
                    theme.fg_dim,
                )));
            }
            if !plugin.workflow_primitives.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  Primitives: {}", plugin.workflow_primitives.join(", ")),
                    theme.fg_dim,
                )));
            }
            if !plugin.read_actions.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  Read actions: {}", plugin.read_actions.join(", ")),
                    theme.fg_dim,
                )));
            }
            if !plugin.write_actions.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  Write actions: {}", plugin.write_actions.join(", ")),
                    theme.fg_dim,
                )));
            }
            lines.push(Line::raw(""));
        }

        for (i, field) in plugin_state.schema_fields.iter().enumerate() {
            let is_active = !plugin_state.list_mode && i == plugin_state.detail_cursor;
            let marker = if is_active { "> " } else { "  " };
            let required_mark = if field.required { " *" } else { "" };
            let label = if field.label.is_empty() {
                field.key.clone()
            } else {
                field.label.clone()
            };

            let value = if settings.is_editing() && settings.editing_field() == Some(&field.key) {
                if field.secret {
                    render_edit_buffer_with_cursor(settings.edit_buffer(), settings.edit_cursor())
                } else {
                    render_edit_buffer_with_cursor(settings.edit_buffer(), settings.edit_cursor())
                }
            } else if field.secret {
                let raw = plugin_state.value_for_key(&field.key).unwrap_or("");
                if raw.is_empty() {
                    "(not set)".to_string()
                } else {
                    "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}".to_string()
                }
            } else {
                plugin_state
                    .value_for_key(&field.key)
                    .unwrap_or("(not set)")
                    .to_string()
            };

            let marker_style = if is_active {
                theme.accent_primary
            } else {
                theme.fg_dim
            };
            let value_style = if is_active {
                theme.accent_primary
            } else {
                theme.fg_dim
            };

            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(
                    format!("{:<18}{}", format!("{}{}", label, required_mark), " "),
                    theme.fg_dim,
                ),
                Span::styled(value, value_style),
            ]));
        }

        let action_offset = plugin_state.schema_fields.len();
        if plugin.has_api {
            let btn_idx = action_offset;
            let is_active = plugin_state.detail_cursor == btn_idx;
            let marker = if is_active { "> " } else { "  " };
            let marker_style = if is_active {
                theme.accent_primary
            } else {
                theme.fg_dim
            };
            lines.push(Line::raw(""));
            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(
                    "[Test Connection]",
                    if is_active {
                        theme.accent_primary
                    } else {
                        theme.fg_active
                    },
                ),
            ]));
            if let Some((success, ref msg)) = plugin_state.test_result {
                let result_style = if success {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };
                lines.push(Line::from(Span::styled(
                    format!("    {}", msg),
                    result_style,
                )));
            }
        }
        if plugin.has_auth {
            let btn_idx = action_offset + if plugin.has_api { 1 } else { 0 };
            let is_active = plugin_state.detail_cursor == btn_idx;
            let marker = if is_active { "> " } else { "  " };
            let marker_style = if is_active {
                theme.accent_primary
            } else {
                theme.fg_dim
            };
            let connect_label = if plugin.auth_status == "not_configured" {
                "[Connect]"
            } else {
                "[Reconnect]"
            };
            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(
                    connect_label,
                    if is_active {
                        theme.accent_primary
                    } else {
                        theme.fg_active
                    },
                ),
            ]));
        }
    }

    lines
}

pub(crate) fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        return "(not set)".to_string();
    }
    let chars: Vec<char> = key.chars().collect();
    let len = chars.len();
    if len <= 7 {
        return "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}".to_string();
    }
    let prefix: String = chars[..3].iter().collect();
    let suffix: String = chars[len - 4..].iter().collect();
    format!(
        "{}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}{}",
        prefix, suffix
    )
}

pub(crate) fn connector_readiness_label(plugin: &PluginListItem) -> String {
    match plugin.readiness_state.as_str() {
        "ready" => "Ready".to_string(),
        "needs_setup" => "Setup".to_string(),
        "needs_reconnect" => "Reconnect".to_string(),
        "degraded" => "Degraded".to_string(),
        "misconfigured" => "Fix setup".to_string(),
        "disabled" => "Disabled".to_string(),
        "unavailable" => "Unavailable".to_string(),
        _ if !plugin.has_auth => "Configured".to_string(),
        _ => match plugin.auth_status.as_str() {
            "connected" => "OK".to_string(),
            "refreshable" => "Auto-refresh".to_string(),
            "needs_reconnect" => "Reconnect".to_string(),
            _ => "Setup".to_string(),
        },
    }
}

pub(crate) fn connector_readiness_style(plugin: &PluginListItem, theme: &ThemeTokens) -> Style {
    match plugin.readiness_state.as_str() {
        "ready" => Style::default().fg(Color::Green),
        "degraded" => Style::default().fg(Color::Yellow),
        "needs_reconnect" | "misconfigured" => Style::default().fg(Color::Red),
        "disabled" => theme.fg_dim,
        _ if !plugin.has_auth => theme.fg_dim,
        _ => match plugin.auth_status.as_str() {
            "connected" => Style::default().fg(Color::Green),
            "refreshable" => Style::default().fg(Color::Yellow),
            "needs_reconnect" => Style::default().fg(Color::Red),
            _ => theme.fg_dim,
        },
    }
}
