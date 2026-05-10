use super::render_plugins_tab_to_connector_readiness_style::*;
use crate::state::config::ConfigState;
use crate::state::settings::SettingsState;
use crate::theme::ThemeTokens;
use ratatui::text::{Line, Span};
pub(crate) fn render_websearch_tab<'a>(
    settings: &'a SettingsState,
    config: &'a ConfigState,
    theme: &ThemeTokens,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled("  Web Search", theme.fg_active)));
    lines.push(Line::from(Span::styled(
        "  Configure web search tool and providers",
        theme.fg_dim,
    )));
    lines.push(Line::raw(""));

    {
        let is_selected = settings.field_cursor() == 0;
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let check = if config.tool_web_search { "[x]" } else { "[ ]" };
        let check_style = if config.tool_web_search {
            theme.accent_success
        } else {
            theme.fg_dim
        };
        let label_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_active
        };
        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::styled(check, check_style),
            Span::raw(" "),
            Span::styled("Enable Web Search", label_style),
        ];
        if is_selected {
            spans.push(Span::styled("  [Space: toggle]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    {
        let is_selected = settings.field_cursor() == 1;
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let provider_val = if config.search_provider.is_empty() || config.search_provider == "none"
        {
            "none".to_string()
        } else {
            config.search_provider.clone()
        };
        let value_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_active
        };
        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::styled(format!("{:<16} ", "Provider:"), theme.fg_dim),
            Span::styled(provider_val, value_style),
        ];
        if is_selected {
            spans.push(Span::styled("  [Enter: cycle]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    {
        let is_selected = settings.field_cursor() == 2;
        let is_editing =
            settings.is_editing() && settings.editing_field() == Some("duckduckgo_region");
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let display_value: String = if is_editing {
            format!("{}\u{2588}", settings.edit_buffer())
        } else {
            config.duckduckgo_region.clone()
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
            Span::styled(format!("{:<16} ", "DDG Region:"), theme.fg_dim),
            Span::styled(display_value, value_style),
        ];
        if is_selected && !is_editing {
            spans.push(Span::styled("  [Enter: edit]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    {
        let is_selected = settings.field_cursor() == 3;
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let value_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_active
        };
        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::styled(format!("{:<16} ", "DDG Safe:"), theme.fg_dim),
            Span::styled(config.duckduckgo_safe_search.clone(), value_style),
        ];
        if is_selected {
            spans.push(Span::styled("  [Enter: cycle]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    let api_key_fields: [(usize, &str, &str, &str); 3] = [
        (
            4,
            "Firecrawl Key:  ",
            config.firecrawl_api_key.as_str(),
            "firecrawl_api_key",
        ),
        (
            5,
            "Exa Key:        ",
            config.exa_api_key.as_str(),
            "exa_api_key",
        ),
        (
            6,
            "Tavily Key:     ",
            config.tavily_api_key.as_str(),
            "tavily_api_key",
        ),
    ];

    for (idx, label, value, field_name) in &api_key_fields {
        let is_selected = settings.field_cursor() == *idx;
        let is_editing = settings.is_editing() && settings.editing_field() == Some(field_name);
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };

        let display_value: String = if is_editing {
            format!("{}\u{2588}", settings.edit_buffer())
        } else if value.is_empty() {
            "(not set)".to_string()
        } else {
            mask_api_key(value)
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
            spans.push(Span::styled("  [Enter: edit]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    {
        let is_selected = settings.field_cursor() == 7;
        let is_editing =
            settings.is_editing() && settings.editing_field() == Some("search_max_results");
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };

        let display_value: String = if is_editing {
            format!("{}\u{2588}", settings.edit_buffer())
        } else {
            config.search_max_results.to_string()
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
            Span::styled(format!("{:<16} ", "Max Results:"), theme.fg_dim),
            Span::styled(display_value, value_style),
        ];
        if is_selected && !is_editing {
            spans.push(Span::styled("  [Enter: edit]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    {
        let is_selected = settings.field_cursor() == 8;
        let is_editing =
            settings.is_editing() && settings.editing_field() == Some("search_timeout");
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };

        let display_value: String = if is_editing {
            format!("{}\u{2588}", settings.edit_buffer())
        } else {
            format!("{}s", config.search_timeout_secs)
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
            Span::styled(format!("{:<16} ", "Timeout:"), theme.fg_dim),
            Span::styled(display_value, value_style),
        ];
        if is_selected && !is_editing {
            spans.push(Span::styled("  [Enter: edit]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  Web Browsing", theme.fg_active)));

    {
        let is_selected = settings.field_cursor() == 9;
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let provider_val = if config.browse_provider.is_empty() {
            "auto".to_string()
        } else {
            config.browse_provider.clone()
        };
        let value_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_active
        };
        let mut spans = vec![
            Span::styled(marker, marker_style),
            Span::styled(format!("{:<16} ", "Browser:"), theme.fg_dim),
            Span::styled(provider_val, value_style),
        ];
        if is_selected {
            spans.push(Span::styled("  [Enter: cycle]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    lines
}
