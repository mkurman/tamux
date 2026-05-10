use super::render_provider_tab_to_render_tools_tab::*;
use super::*;
use crate::providers;
use crate::state::config::ConfigState;
use crate::state::settings::SettingsState;
use crate::theme::ThemeTokens;
use ratatui::text::{Line, Span};
pub(crate) fn render_auth_tab<'a>(
    content_width: u16,
    auth: &'a crate::state::auth::AuthState,
    config: &'a ConfigState,
    theme: &ThemeTokens,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        "  Authentication",
        theme.fg_active,
    )));
    lines.push(Line::from(Span::styled(
        "  Provider authentication status",
        theme.fg_dim,
    )));
    lines.push(Line::raw(""));

    if let Some(provider_id) = auth.login_target.as_deref() {
        let provider_name = auth
            .entries
            .iter()
            .find(|entry| entry.provider_id == provider_id)
            .map(|entry| entry.provider_name.clone())
            .or_else(|| providers::find_by_id(provider_id).map(|def| def.name.to_string()))
            .unwrap_or_else(|| provider_id.to_string());
        let masked = "•".repeat(auth.login_buffer.chars().count());
        let display = render_edit_buffer_with_cursor(&masked, auth.login_cursor);

        lines.push(Line::from(Span::styled(
            format!("  Login to {provider_name}"),
            theme.fg_active,
        )));
        lines.push(Line::from(Span::styled(
            "  Enter API key below. Press Enter to save or Esc to cancel.",
            theme.fg_dim,
        )));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::styled("  API Key: ", theme.fg_dim),
            Span::styled(display, theme.accent_primary),
        ]));
        return lines;
    }

    if !auth.loaded {
        lines.push(Line::from(Span::styled(
            "  No providers loaded. Connect to daemon to see status.",
            theme.fg_dim,
        )));
        return lines;
    }

    if auth.entries.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No providers are configured yet. Use the Provider tab to add one.",
            theme.fg_dim,
        )));
        return lines;
    }

    for (i, entry) in auth.entries.iter().enumerate() {
        let is_selected = auth.selected == i;
        let marker = if is_selected { "> " } else { "  " };
        let openai_has_chatgpt_auth = entry.provider_id
            == zorai_shared::providers::PROVIDER_ID_OPENAI
            && config.chatgpt_auth_available;
        let effective_authenticated = entry.authenticated || openai_has_chatgpt_auth;
        let dot_style = if effective_authenticated {
            Style::default().fg(Color::Green)
        } else {
            theme.fg_dim
        };
        let dot = if effective_authenticated {
            "● "
        } else {
            "○ "
        };
        let model_info = if effective_authenticated && !entry.model.is_empty() {
            format!(" ({})", entry.model)
        } else {
            String::new()
        };
        let primary_label = auth_primary_label(entry);
        let test_label = if entry.provider_id == zorai_shared::providers::PROVIDER_ID_OPENAI
            && openai_has_chatgpt_auth
        {
            "[Logout]"
        } else {
            auth_secondary_label(entry)
        };
        let left_width = marker.chars().count()
            + dot.chars().count()
            + entry.provider_name.chars().count()
            + model_info.chars().count();
        let actions_width = primary_label.chars().count() + 1 + test_label.chars().count();
        let spacer =
            " ".repeat((content_width as usize).saturating_sub(left_width + actions_width + 1));

        let line = Line::from(vec![
            Span::styled(
                marker,
                if is_selected {
                    theme.fg_active
                } else {
                    theme.fg_dim
                },
            ),
            Span::styled(dot, dot_style),
            Span::styled(
                entry.provider_name.clone(),
                if is_selected {
                    theme.fg_active
                } else {
                    Style::default().fg(Color::White)
                },
            ),
            Span::styled(model_info, theme.fg_dim),
            Span::raw(spacer),
            Span::styled(
                primary_label,
                if is_selected && auth.actions_focused && auth.action_cursor == 0 {
                    theme.fg_active
                } else {
                    theme.fg_dim
                },
            ),
            Span::raw(" "),
            Span::styled(
                test_label,
                if is_selected && auth.actions_focused && auth.action_cursor == 1 {
                    theme.fg_active
                } else {
                    theme.fg_dim
                },
            ),
        ]);
        lines.push(line);
        if let Some((success, message)) = auth.validation_results.get(&entry.provider_id) {
            let result_style = if *success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            lines.push(Line::from(Span::styled(
                format!("    {message}"),
                result_style,
            )));
        }
    }

    lines.push(Line::raw(""));
    lines.push(Line::from(vec![
        Span::styled("  ", theme.fg_dim),
        Span::styled("↑↓", theme.fg_active),
        Span::styled(" provider  ", theme.fg_dim),
        Span::styled("←→", theme.fg_active),
        Span::styled(" action  ", theme.fg_dim),
        Span::styled("Enter", theme.fg_active),
        Span::styled(" run", theme.fg_dim),
    ]));

    lines
}

pub(crate) fn render_agent_tab<'a>(
    settings: &'a SettingsState,
    config: &'a ConfigState,
    theme: &ThemeTokens,
) -> Vec<Line<'a>> {
    let mut lines = render_provider_tab(settings, config, theme);

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", zorai_protocol::AGENT_NAME_SWAROG),
        theme.fg_active,
    )));
    lines.push(Line::from(Span::styled(
        "  Main agent identity and behavior",
        theme.fg_dim,
    )));
    lines.push(Line::raw(""));

    lines.push(Line::from(vec![
        Span::styled("  Fixed Name        ", theme.fg_dim),
        Span::styled(zorai_protocol::AGENT_NAME_SWAROG, theme.fg_active),
    ]));

    let system_prompt = if let Some(raw) = config.agent_config_raw() {
        raw.get("system_prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    let editable_fields: [(usize, &str, String, &str, &str); 1] = [(
        8,
        "System Prompt ",
        system_prompt,
        "system_prompt",
        " [Enter: edit]",
    )];

    for (idx, label, value, field_name, hint) in &editable_fields {
        let is_selected = settings.field_cursor() == *idx;
        let is_editing = settings.is_editing() && settings.editing_field() == Some(field_name);
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };

        if *field_name == "system_prompt" && is_editing && settings.is_textarea() {
            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(*label, theme.fg_dim),
                Span::styled(" [Ctrl+S/Ctrl+Enter: save, Esc: cancel]", theme.fg_dim),
            ]));
            lines.push(Line::from(Span::styled(
                "  ╭──────────────────────────────────────────╮",
                theme.fg_dim,
            )));
            for buf_line in settings.edit_buffer().split('\n') {
                lines.push(Line::from(vec![
                    Span::styled("  │ ", theme.fg_dim),
                    Span::styled(buf_line.to_string(), theme.fg_active),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("  │ ", theme.fg_dim),
                Span::raw("\u{2588}"),
            ]));
            lines.push(Line::from(Span::styled(
                "  ╰──────────────────────────────────────────╯",
                theme.fg_dim,
            )));
            continue;
        }

        if *field_name == "system_prompt" && !is_editing {
            let preview = if value.is_empty() {
                "(not set)".to_string()
            } else {
                let first_lines: Vec<&str> = value.lines().take(2).collect();
                let preview = first_lines.join(" ");
                if preview.chars().count() > 45 {
                    let truncated: String = preview.chars().take(42).collect();
                    format!("{}...", truncated)
                } else if value.lines().count() > 2 {
                    format!("{} ...", preview)
                } else {
                    preview
                }
            };
            let hint_text = if is_selected { " [Enter: edit]" } else { "" };
            lines.push(Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(*label, theme.fg_dim),
                Span::styled(
                    preview,
                    if is_selected {
                        theme.fg_active
                    } else {
                        theme.fg_dim
                    },
                ),
                Span::styled(hint_text.to_string(), theme.fg_dim),
            ]));
            continue;
        }

        let display_value: String = if is_editing {
            format!("{}\u{2588}", settings.edit_buffer())
        } else if value.is_empty() {
            "(not set)".to_string()
        } else {
            let v = value.as_str();
            if v.chars().count() > 40 {
                let truncated: String = v.chars().take(37).collect();
                format!("{}...", truncated)
            } else {
                v.to_string()
            }
        };

        let value_style = if is_editing {
            theme.fg_active
        } else if is_selected {
            theme.accent_primary
        } else {
            theme.fg_active
        };

        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::styled(format!("{:<16} ", label), theme.fg_dim),
            Span::styled(display_value, value_style),
        ];
        if is_selected && !is_editing {
            spans.push(Span::styled(*hint, theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    {
        let is_selected = settings.field_cursor() == 9;
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let value_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        lines.push(Line::from(vec![
            Span::styled(marker, marker_style),
            Span::styled("Backend           ", theme.fg_dim),
            Span::styled("daemon", value_style),
        ]));
    }

    lines
}
