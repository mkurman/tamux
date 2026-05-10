use super::spawned_agents;
use super::*;
use crate::app::RecentActionVm;
use crate::state::chat::{ChatState, GatewayStatusVm, MessageRole};
use crate::state::sidebar::{SidebarState, SidebarTab};
use crate::state::task::TaskState;
use crate::state::tier::TierState;
use crate::theme::ThemeTokens;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
pub(crate) fn filtered_file_index(
    tasks: &TaskState,
    sidebar: &SidebarState,
    thread_id: Option<&str>,
    path: &str,
) -> Option<usize> {
    build_cached_snapshot(
        Rect::new(0, 0, 80, 0),
        &ChatState::new(),
        sidebar,
        tasks,
        thread_id,
    )
    .filtered_file_index(path)
}

pub(crate) fn selected_pinned_message(
    chat: &ChatState,
    sidebar: &SidebarState,
) -> Option<crate::state::chat::PinnedThreadMessage> {
    build_cached_snapshot(
        Rect::new(0, 0, 80, 0),
        chat,
        sidebar,
        &TaskState::new(),
        None,
    )
    .selected_pinned_message(chat, sidebar.selected_item())
}

pub(crate) fn pinned_message_chars(message: &crate::state::chat::PinnedThreadMessage) -> usize {
    message.content.chars().count()
}

pub(crate) fn pinned_message_role_label(role: MessageRole) -> &'static str {
    match role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
        MessageRole::Tool => "tool",
        MessageRole::Unknown => "unknown",
    }
}

pub(crate) fn pinned_message_snippet(content: &str, width: usize) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let max_len = width.saturating_sub(18).max(8);
    if compact.chars().count() > max_len {
        format!(
            "{}…",
            compact
                .chars()
                .take(max_len.saturating_sub(1))
                .collect::<String>()
        )
    } else {
        compact
    }
}

pub(crate) fn active_thread_pinned_rows(chat: &ChatState) -> PinnedSidebarRows {
    chat.active_thread_pinned_messages()
}

pub(crate) fn pinned_footer_line(theme: &ThemeTokens) -> Line<'static> {
    Line::from(vec![
        Span::styled(" Ctrl+K J", theme.fg_active),
        Span::styled(" jump  ", theme.fg_dim),
        Span::styled("Ctrl+K U", theme.fg_active),
        Span::styled(" unpin  ", theme.fg_dim),
        Span::styled("Ctrl+C", theme.fg_active),
        Span::styled(" copy", theme.fg_dim),
    ])
}

pub(crate) fn spawned_footer_line(theme: &ThemeTokens) -> Line<'static> {
    Line::from(vec![
        Span::styled(" Enter", theme.fg_active),
        Span::styled(" open child  ", theme.fg_dim),
        Span::styled("↑↓", theme.fg_active),
        Span::styled(" move", theme.fg_dim),
    ])
}

pub(crate) fn thread_history_footer_line(theme: &ThemeTokens, depth: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(" Backspace", theme.fg_active),
        Span::styled(" previous thread", theme.fg_dim),
        Span::styled(format!(" ({depth})"), theme.fg_dim),
    ])
}

pub(crate) fn row_from_snapshot(
    snapshot: &CachedSidebarSnapshot,
    index: usize,
    selected: usize,
    theme: &ThemeTokens,
    width: usize,
) -> Option<SidebarRow> {
    let selected_style = Style::default().bg(Color::Indexed(236));

    let row = match &snapshot.body {
        SidebarBodySnapshot::Empty { message } => SidebarRow {
            line: Line::from(Span::styled(message.clone(), theme.fg_dim)),
        },
        SidebarBodySnapshot::Files(items) => {
            let item = items.get(index)?;
            let line = Line::from(vec![
                Span::styled(
                    if index == selected { "> " } else { "  " },
                    theme.accent_primary,
                ),
                Span::styled(format!("[{}]", item.label), theme.fg_dim),
                Span::raw(" "),
                Span::styled(item.display_path.clone(), theme.fg_active),
            ]);
            SidebarRow {
                line: if index == selected {
                    line.style(selected_style)
                } else {
                    line
                },
            }
        }
        SidebarBodySnapshot::Todos(items) => {
            let item = items.get(index)?;
            let line = Line::from(vec![
                Span::styled(
                    if index == selected { "> " } else { "  " },
                    theme.accent_primary,
                ),
                Span::styled(item.marker, theme.fg_dim),
                Span::raw(" "),
                Span::styled(item.text.clone(), theme.fg_active),
            ]);
            SidebarRow {
                line: if index == selected {
                    line.style(selected_style)
                } else {
                    line
                },
            }
        }
        SidebarBodySnapshot::Spawned(items) => {
            let item = items.get(index)?;
            let indent = "  ".repeat(item.depth);
            let marker = if item.is_active {
                "@"
            } else if item.openable {
                ">"
            } else {
                "-"
            };
            let status = if item.live { "live" } else { "done" };
            let max_len = width
                .saturating_sub(indent.chars().count())
                .saturating_sub(12)
                .max(8);
            let title_style = if item.target_thread_id.is_none() && !item.is_active {
                theme.fg_dim
            } else {
                theme.fg_active
            };
            let line = Line::from(vec![
                Span::styled(
                    if index == selected { "> " } else { "  " },
                    theme.accent_primary,
                ),
                Span::raw(indent),
                Span::styled(format!("[{marker}]"), theme.fg_dim),
                Span::raw(" "),
                Span::styled(truncate_tail(&item.title, max_len), title_style),
                Span::styled(format!(" [{status}]"), theme.fg_dim),
            ]);
            SidebarRow {
                line: if index == selected {
                    line.style(selected_style)
                } else {
                    line
                },
            }
        }
        SidebarBodySnapshot::Pinned(items) => {
            let item = items.get(index)?;
            let line = Line::from(vec![
                Span::styled(
                    if index == selected { "> " } else { "  " },
                    theme.accent_primary,
                ),
                Span::styled(item.metadata.clone(), theme.fg_dim),
                Span::raw(" "),
                Span::styled(item.snippet.clone(), theme.fg_active),
            ]);
            SidebarRow {
                line: if index == selected {
                    line.style(selected_style)
                } else {
                    line
                },
            }
        }
    };

    Some(row)
}

pub(crate) fn visible_rows(
    snapshot: &CachedSidebarSnapshot,
    sidebar: &SidebarState,
    theme: &ThemeTokens,
    width: usize,
    body_height: usize,
    scroll: usize,
) -> Vec<SidebarRow> {
    let end = scroll
        .saturating_add(body_height)
        .min(snapshot.item_count());
    (scroll..end)
        .filter_map(|index| {
            row_from_snapshot(snapshot, index, sidebar.selected_item(), theme, width)
        })
        .collect()
}

pub(crate) fn has_spawned_tab(
    tasks: &TaskState,
    chat: &ChatState,
    thread_id: Option<&str>,
) -> bool {
    spawned_agents::has_content(tasks, thread_id) || chat.can_go_back_thread()
}

pub(crate) fn visible_tabs(
    tasks: &TaskState,
    chat: &ChatState,
    thread_id: Option<&str>,
) -> Vec<SidebarTab> {
    tab_layout::visible_tabs(
        has_spawned_tab(tasks, chat, thread_id),
        chat.active_thread_has_pinned_messages(),
    )
}

pub(crate) fn selected_spawned_thread_id(
    tasks: &TaskState,
    sidebar: &SidebarState,
    thread_id: Option<&str>,
) -> Option<String> {
    spawned_agents::selected_thread_id(tasks, sidebar.selected_item(), thread_id)
}

pub(crate) fn first_openable_spawned_index(
    tasks: &TaskState,
    thread_id: Option<&str>,
) -> Option<usize> {
    spawned_agents::first_openable_index(tasks, thread_id)
}

pub(crate) fn resolved_scroll(
    item_count: usize,
    sidebar: &SidebarState,
    body_height: usize,
) -> usize {
    let max_scroll = item_count.saturating_sub(body_height);
    let mut scroll = sidebar.scroll_offset().min(max_scroll);
    let selected = sidebar.selected_item().min(item_count.saturating_sub(1));
    if selected < scroll {
        scroll = selected;
    } else if selected >= scroll.saturating_add(body_height) {
        scroll = selected.saturating_add(1).saturating_sub(body_height);
    }
    scroll.min(max_scroll)
}

pub(crate) fn gateway_status_lines(
    statuses: &[GatewayStatusVm],
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    let active: Vec<&GatewayStatusVm> = statuses
        .iter()
        .filter(|s| s.status != "disconnected")
        .collect();
    if active.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        " Gateway",
        Style::default()
            .fg(Color::Indexed(245))
            .add_modifier(ratatui::style::Modifier::BOLD),
    )));

    for gw in &active {
        let (indicator, color) = match gw.status.as_str() {
            "connected" => ("\u{25CF}", Color::Green),
            "error" => ("\u{25CF}", Color::Red),
            _ => ("\u{25CF}", Color::Indexed(245)),
        };
        let platform_label = match gw.platform.as_str() {
            "slack" => "Slack",
            "discord" => "Discord",
            "telegram" => "Telegram",
            other => other,
        };
        let mut spans = vec![
            Span::styled("  ", theme.fg_dim),
            Span::styled(indicator.to_string(), Style::default().fg(color)),
            Span::raw(" "),
            Span::styled(platform_label.to_string(), theme.fg_active),
        ];
        if gw.status == "error" {
            if let Some(ref err) = gw.last_error {
                let truncated: String = err.chars().take(30).collect();
                spans.push(Span::styled(
                    format!(" ({})", truncated),
                    Style::default().fg(Color::Red),
                ));
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

pub(crate) fn recent_actions_lines(
    actions: &[RecentActionVm],
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    if actions.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        " Recent Actions",
        Style::default()
            .fg(Color::Indexed(245))
            .add_modifier(ratatui::style::Modifier::BOLD),
    )));
    for action in actions.iter().take(3) {
        let icon = match action.action_type.as_str() {
            "stale_todo" => "\u{2611}",
            "stuck_goal" => "\u{26A0}",
            "morning_brief" => "\u{2600}",
            _ => "\u{25CB}",
        };
        let mut summary = action.summary.clone();
        if summary.chars().count() > 40 {
            summary = format!("{}...", summary.chars().take(37).collect::<String>());
        }
        lines.push(Line::from(vec![
            Span::styled("  ", theme.fg_dim),
            Span::styled(icon.to_string(), theme.fg_dim),
            Span::raw(" "),
            Span::styled(summary, theme.fg_active),
        ]));
    }
    lines
}

/// Render a dimmed one-line placeholder for a tier-locked sidebar section (D-05).
pub(crate) fn tier_placeholder_line(label: &str, required_tier: &str) -> Line<'static> {
    let dim = Style::default().fg(Color::DarkGray);
    Line::from(vec![
        Span::styled("  \u{25B6} ", dim),
        Span::styled(label.to_string(), dim),
        Span::styled(format!("  [{}]", required_tier.replace('_', " ")), dim),
    ])
}

/// Collect tier-gated placeholder lines for hidden sidebar sections.
pub(crate) fn tier_gated_lines(tier: &TierState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if !tier.show_goal_runs {
        lines.push(tier_placeholder_line("Goal Runs", "familiar"));
    }
    if !tier.show_task_queue {
        lines.push(tier_placeholder_line("Task Queue", "familiar"));
    }
    if !tier.show_gateway_config {
        lines.push(tier_placeholder_line("Gateway", "familiar"));
    }
    if !tier.show_subagents {
        lines.push(tier_placeholder_line("Sub-Agents", "power user"));
    }
    if !tier.show_memory_controls {
        lines.push(tier_placeholder_line("Memory", "expert"));
    }
    lines
}

#[cfg(test)]
pub(crate) fn agent_status_line(
    activity: Option<&str>,
    tier: &str,
    weles_health: Option<&crate::client::WelesHealthVm>,
) -> Line<'static> {
    let status_span = match activity {
        Some("thinking" | "reasoning" | "writing") => {
            Span::styled("\u{25CF} Thinking", Style::default().fg(Color::Yellow))
        }
        Some(s) if s.starts_with('\u{2699}') => {
            Span::styled(format!("\u{25CF} {}", s), Style::default().fg(Color::Blue))
        }
        Some("waiting_for_approval") => Span::styled(
            "\u{25CF} Awaiting approval",
            Style::default().fg(Color::Rgb(255, 165, 0)),
        ),
        Some("running_goal" | "goal_running") => {
            Span::styled("\u{25CF} Running goal", Style::default().fg(Color::Green))
        }
        Some("idle") | None => Span::styled("\u{25CF} Idle", Style::default().fg(Color::DarkGray)),
        Some(other) => Span::styled(
            format!("\u{25CF} {}", other),
            Style::default().fg(Color::DarkGray),
        ),
    };

    let tier_label = match tier {
        "newcomer" => "",
        "familiar" => " [familiar]",
        "power_user" => " [power user]",
        "expert" => " [expert]",
        _ => "",
    };

    let mut spans = vec![Span::raw(" "), status_span];
    if !tier_label.is_empty() {
        spans.push(Span::styled(
            tier_label.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }
    if weles_health.is_some_and(|health| health.state.eq_ignore_ascii_case("degraded")) {
        spans.push(Span::styled(
            "  [WELES degraded]".to_string(),
            Style::default().fg(Color::LightYellow),
        ));
    }
    Line::from(spans)
}

#[cfg(test)]
pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    chat: &ChatState,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
    theme: &ThemeTokens,
    focused: bool,
    gateway_statuses: &[GatewayStatusVm],
    tier: &TierState,
    _agent_activity: Option<&str>,
    _weles_health: Option<&crate::client::WelesHealthVm>,
    recent_actions: &[RecentActionVm],
) {
    let snapshot = build_cached_snapshot(area, chat, sidebar, tasks, thread_id);
    render_cached(
        frame,
        area,
        chat,
        sidebar,
        theme,
        focused,
        gateway_statuses,
        tier,
        recent_actions,
        &snapshot,
    );
}
