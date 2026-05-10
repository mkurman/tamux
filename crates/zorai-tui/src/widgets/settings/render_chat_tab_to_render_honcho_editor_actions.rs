use super::render_gateway_text_field::*;
use super::render_plugins_tab_to_connector_readiness_style::*;
use crate::state::config::ConfigState;
use crate::state::settings::SettingsState;
use crate::theme::ThemeTokens;
use ratatui::text::{Line, Span};
pub(crate) fn render_chat_tab<'a>(
    settings: &'a SettingsState,
    config: &'a ConfigState,
    theme: &ThemeTokens,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled("  Chat", theme.fg_active)));
    lines.push(Line::from(Span::styled(
        "  Configure streaming and memory",
        theme.fg_dim,
    )));
    lines.push(Line::raw(""));

    let toggles: [(usize, bool, &str); 3] = [
        (0, config.enable_streaming, "Streaming"),
        (1, config.enable_conversation_memory, "Conversation Memory"),
        (2, config.enable_honcho_memory, "Honcho Memory"),
    ];
    for (idx, enabled, name) in &toggles {
        let is_selected = settings.field_cursor() == *idx;
        let check = if *enabled { "[x]" } else { "[ ]" };
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let check_style = if *enabled {
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
            Span::styled(*name, label_style),
        ];
        if is_selected {
            if *idx == 2 {
                spans.push(Span::styled("  [Space: toggle, Enter: edit]", theme.fg_dim));
            } else {
                spans.push(Span::styled("  [Space: toggle]", theme.fg_dim));
            }
        }
        lines.push(Line::from(spans));
    }

    let capability_toggles: [(usize, bool, &str); 10] = [
        (3, config.anticipatory_enabled, "Anticipatory Support"),
        (4, config.anticipatory_morning_brief, "Morning Brief"),
        (
            5,
            config.anticipatory_predictive_hydration,
            "Predictive Hydration",
        ),
        (6, config.anticipatory_stuck_detection, "Stuck Detection"),
        (7, config.operator_model_enabled, "Operator Model"),
        (
            8,
            config.operator_model_allow_message_statistics,
            "Message Statistics",
        ),
        (
            9,
            config.operator_model_allow_approval_learning,
            "Approval Learning",
        ),
        (
            10,
            config.operator_model_allow_attention_tracking,
            "Attention Tracking",
        ),
        (
            11,
            config.operator_model_allow_implicit_feedback,
            "Implicit Feedback",
        ),
        (12, config.collaboration_enabled, "Collaboration"),
    ];
    for (idx, enabled, name) in &capability_toggles {
        let is_selected = settings.field_cursor() == *idx;
        let check = if *enabled { "[x]" } else { "[ ]" };
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let check_style = if *enabled {
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
            Span::styled(*name, label_style),
        ];
        if is_selected {
            spans.push(Span::styled("  [Space: toggle]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    let field_13_selected = settings.field_cursor() == 13;
    lines.push(Line::from(vec![
        Span::styled(
            if field_13_selected { "> " } else { "  " },
            if field_13_selected {
                theme.accent_primary
            } else {
                theme.fg_dim
            },
        ),
        Span::styled(format!("{:<16} ", "Compliance:"), theme.fg_dim),
        Span::styled(
            config.compliance_mode.as_str(),
            if field_13_selected {
                theme.accent_primary
            } else {
                theme.fg_active
            },
        ),
        if field_13_selected {
            Span::styled("  [Enter: cycle]", theme.fg_dim)
        } else {
            Span::raw("")
        },
    ]));

    for (idx, label, value, field_name) in [(
        14usize,
        "Retention Days: ",
        config.compliance_retention_days.to_string(),
        "compliance_retention_days",
    )] {
        render_gateway_text_field(
            settings, theme, &mut lines, idx, label, &value, field_name, false,
        );
    }

    for (idx, enabled, name) in [
        (
            15usize,
            config.compliance_sign_all_events,
            "Sign All Events",
        ),
        (16usize, config.tool_synthesis_enabled, "Tool Synthesis"),
        (
            17usize,
            config.tool_synthesis_require_activation,
            "Require Activation",
        ),
    ] {
        let is_selected = settings.field_cursor() == idx;
        let check = if enabled { "[x]" } else { "[ ]" };
        let marker = if is_selected { "> " } else { "  " };
        let marker_style = if is_selected {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let check_style = if enabled {
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
            Span::styled(name, label_style),
        ];
        if is_selected {
            spans.push(Span::styled("  [Space: toggle]", theme.fg_dim));
        }
        lines.push(Line::from(spans));
    }

    if let Some(editor) = config.honcho_editor.as_ref() {
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            "  Honcho Memory Settings",
            theme.fg_dim,
        )));
        lines.push(render_honcho_editor_toggle(
            editor,
            crate::state::config::HonchoEditorField::Enabled,
            "Enabled",
            editor.enabled,
            theme,
        ));
        lines.push(render_honcho_editor_text_field(
            settings,
            editor,
            crate::state::config::HonchoEditorField::ApiKey,
            "API Key",
            &editor.api_key,
            "honcho_editor_api_key",
            true,
            theme,
        ));
        lines.push(render_honcho_editor_text_field(
            settings,
            editor,
            crate::state::config::HonchoEditorField::BaseUrl,
            "Base URL",
            &editor.base_url,
            "honcho_editor_base_url",
            false,
            theme,
        ));
        lines.push(render_honcho_editor_text_field(
            settings,
            editor,
            crate::state::config::HonchoEditorField::WorkspaceId,
            "Workspace",
            &editor.workspace_id,
            "honcho_editor_workspace_id",
            false,
            theme,
        ));
        lines.push(render_honcho_editor_actions(editor, theme));
        lines.push(Line::raw(""));
    }

    for (idx, label, value, field_name) in [
        (
            18usize,
            "Tool Limit:     ",
            config.tool_synthesis_max_generated_tools.to_string(),
            "tool_synthesis_max_generated_tools",
        ),
        (
            19usize,
            "Visible Msgs:   ",
            config.tui_chat_history_page_size.to_string(),
            "tui_chat_history_page_size",
        ),
        (
            20usize,
            "Restore Hours:  ",
            config.participant_observer_restore_window_hours.to_string(),
            "participant_observer_restore_window_hours",
        ),
    ] {
        render_gateway_text_field(
            settings, theme, &mut lines, idx, label, &value, field_name, false,
        );
    }

    for (idx, label) in [
        (21usize, "Inspect Operator Model"),
        (22usize, "Reset Operator Model"),
        (23usize, "Inspect Collaboration"),
        (24usize, "Inspect Generated Tools"),
    ] {
        let is_selected = settings.field_cursor() == idx;
        lines.push(Line::from(vec![
            Span::styled(
                if is_selected { "> " } else { "  " },
                if is_selected {
                    theme.accent_primary
                } else {
                    theme.fg_dim
                },
            ),
            Span::styled(
                format!("{label:<22}"),
                if is_selected {
                    theme.accent_primary
                } else {
                    theme.fg_active
                },
            ),
            if is_selected {
                Span::styled("  [Enter: run]", theme.fg_dim)
            } else {
                Span::raw("")
            },
        ]));
    }

    lines
}

pub(crate) fn render_honcho_editor_toggle<'a>(
    editor: &crate::state::config::HonchoEditorState,
    field: crate::state::config::HonchoEditorField,
    label: &'a str,
    enabled: bool,
    theme: &ThemeTokens,
) -> Line<'a> {
    let is_selected = editor.field == field;
    let marker = if is_selected { "  > " } else { "    " };
    let check = if enabled { "[x]" } else { "[ ]" };
    let mut spans = vec![
        Span::styled(
            marker,
            if is_selected {
                theme.accent_primary
            } else {
                theme.fg_dim
            },
        ),
        Span::styled(
            check,
            if enabled {
                theme.accent_success
            } else {
                theme.fg_dim
            },
        ),
        Span::raw(" "),
        Span::styled(
            label,
            if is_selected {
                theme.accent_primary
            } else {
                theme.fg_active
            },
        ),
    ];
    if is_selected {
        spans.push(Span::styled("  [Space: toggle]", theme.fg_dim));
    }
    Line::from(spans)
}

pub(crate) fn render_honcho_editor_text_field<'a>(
    settings: &SettingsState,
    editor: &crate::state::config::HonchoEditorState,
    field: crate::state::config::HonchoEditorField,
    label: &'a str,
    value: &str,
    field_name: &'a str,
    password: bool,
    theme: &ThemeTokens,
) -> Line<'a> {
    let is_selected = editor.field == field;
    let is_editing = settings.is_editing() && settings.editing_field() == Some(field_name);
    let display_value = if is_editing {
        format!("{}\u{2588}", settings.edit_buffer())
    } else if value.is_empty() {
        "(not set)".to_string()
    } else if password {
        mask_api_key(value)
    } else {
        value.to_string()
    };
    let mut spans = vec![
        Span::styled(
            if is_selected { "  > " } else { "    " },
            if is_selected {
                theme.accent_primary
            } else {
                theme.fg_dim
            },
        ),
        Span::styled(format!("{label:<12} "), theme.fg_dim),
        Span::styled(
            display_value,
            if is_editing {
                theme.fg_active
            } else if is_selected {
                theme.accent_primary
            } else {
                theme.fg_active
            },
        ),
    ];
    if is_selected && !is_editing {
        spans.push(Span::styled("  [Enter: edit]", theme.fg_dim));
    }
    Line::from(spans)
}

pub(crate) fn render_honcho_editor_actions<'a>(
    editor: &crate::state::config::HonchoEditorState,
    theme: &ThemeTokens,
) -> Line<'a> {
    let save_selected = editor.field == crate::state::config::HonchoEditorField::Save;
    let cancel_selected = editor.field == crate::state::config::HonchoEditorField::Cancel;
    Line::from(vec![
        Span::styled(
            if save_selected || cancel_selected {
                "  > "
            } else {
                "    "
            },
            if save_selected || cancel_selected {
                theme.accent_primary
            } else {
                theme.fg_dim
            },
        ),
        Span::styled(
            "[Save]",
            if save_selected {
                theme.accent_primary
            } else {
                theme.fg_dim
            },
        ),
        Span::raw("  "),
        Span::styled(
            "[Cancel]",
            if cancel_selected {
                theme.accent_primary
            } else {
                theme.fg_dim
            },
        ),
    ])
}
