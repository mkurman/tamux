use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::system_monitor::SystemMonitorDisplay;
use crate::theme::ThemeTokens;
use crate::widgets::token_format::format_token_count;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderHitTarget {
    ApprovalBadge,
    NotificationBell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HeaderUsageDisplay {
    pub(crate) total_thread_tokens: u64,
    pub(crate) current_tokens: u64,
    pub(crate) context_window_tokens: u64,
    pub(crate) compaction_target_tokens: u64,
    pub(crate) compaction_model_window_tokens: Option<u64>,
    pub(crate) utilization_pct: u8,
    pub(crate) total_cost_usd: Option<f64>,
}

const CONTEXT_BAR_WIDTH: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextFillBand {
    Green,
    Orange,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextBarCellKind {
    Fill,
    Empty,
    Marker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContextBarCell {
    ch: char,
    kind: ContextBarCellKind,
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    provider: &str,
    model: &str,
    reasoning_effort: Option<&str>,
    usage: &HeaderUsageDisplay,
    session_duration_secs: Option<u64>,
    theme: &ThemeTokens,
    pending_approvals: usize,
    approvals_open: bool,
    unread_notifications: usize,
    notifications_open: bool,
    system_monitor: Option<&SystemMonitorDisplay>,
) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.fg_dim);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let approvals_width = approval_area_width(pending_approvals);
    let bell_width = bell_area_width(unread_notifications);
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(
                approvals_width
                    .saturating_add(1)
                    .saturating_add(bell_width)
                    .min(inner.width),
            ),
        ])
        .split(inner);
    let title_area = sections[0];
    let badges_area = sections[1];
    let badge_sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(approvals_width.min(badges_area.width)),
            Constraint::Length(1),
            Constraint::Length(bell_width.min(badges_area.width.saturating_sub(approvals_width))),
        ])
        .split(badges_area);
    let approval_area = badge_sections[0];
    let bell_area = badge_sections[2];

    let mut top_spans = vec![
        Span::styled(
            "\u{2591}\u{2592}\u{2593}",
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(" Zorai ", theme.accent_secondary),
        Span::styled(
            "\u{2593}\u{2592}\u{2591} ",
            Style::default().fg(Color::Yellow),
        ),
    ];

    if !provider.is_empty() {
        top_spans.push(Span::raw(provider));
        top_spans.push(Span::raw(" "));
    }

    top_spans.push(Span::styled(model, theme.fg_active));

    if let Some(reasoning_effort) = reasoning_effort.filter(|value| !value.is_empty()) {
        top_spans.push(Span::raw(" ["));
        top_spans.push(Span::styled(reasoning_effort, theme.accent_secondary));
        top_spans.push(Span::raw("]"));
    }

    top_spans.push(Span::raw("  "));
    top_spans.push(Span::styled(
        build_total_usage_label(usage, session_duration_secs),
        theme.fg_dim,
    ));

    let (top_line_area, bottom_line_area) = if title_area.height >= 2 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(title_area);
        (rows[0], Some(rows[1]))
    } else {
        (title_area, None)
    };
    frame.render_widget(
        Paragraph::new(Line::from(top_spans)).alignment(Alignment::Center),
        top_line_area,
    );
    if let Some(system_monitor) = system_monitor {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                build_system_monitor_label(system_monitor),
                theme.fg_dim,
            ))),
            top_line_area,
        );
    }
    if let Some(bottom_line_area) = bottom_line_area {
        frame.render_widget(
            Paragraph::new(context_usage_line(usage, theme)).alignment(Alignment::Center),
            bottom_line_area,
        );
    }

    let approval_render_area = Rect::new(approval_area.x, approval_area.y, approval_area.width, 1);
    let bell_render_area = Rect::new(bell_area.x, bell_area.y, bell_area.width, 1);

    let approval_style = if approvals_open {
        theme.accent_primary
    } else if pending_approvals > 0 {
        theme.accent_danger
    } else {
        theme.fg_dim
    };
    let approval_text = if pending_approvals > 0 {
        format!("⚖ {}", pending_approvals.min(99))
    } else {
        "⚖".to_string()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(approval_text, approval_style)))
            .alignment(Alignment::Right),
        approval_render_area,
    );

    let bell_style = if notifications_open {
        theme.accent_primary
    } else if unread_notifications > 0 {
        theme.accent_secondary
    } else {
        theme.fg_dim
    };
    let bell_text = if unread_notifications > 0 {
        format!("\u{1F514} {}", unread_notifications.min(99))
    } else {
        "\u{1F514}".to_string()
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(bell_text, bell_style))).alignment(Alignment::Right),
        bell_render_area,
    );
}

fn build_total_usage_label(
    usage: &HeaderUsageDisplay,
    session_duration_secs: Option<u64>,
) -> String {
    let mut label = format!("⚡ {}", format_token_count(usage.total_thread_tokens));
    if let Some(total_cost_usd) = usage.total_cost_usd {
        label.push(' ');
        label.push_str(&format!("${total_cost_usd:.4}"));
    }
    if let Some(session_duration_secs) = session_duration_secs {
        label.push_str("  🕒 ");
        label.push_str(&format_session_duration(session_duration_secs));
    }
    label
}

fn format_session_duration(duration_secs: u64) -> String {
    let days = duration_secs / 86_400;
    let hours = (duration_secs % 86_400) / 3_600;
    let minutes = (duration_secs % 3_600) / 60;
    let seconds = duration_secs % 60;

    if days > 0 {
        format!("{days}d {hours:02}h")
    } else if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else if minutes > 0 {
        format!("{minutes}m")
    } else {
        format!("{seconds}s")
    }
}

fn build_system_monitor_label(system_monitor: &SystemMonitorDisplay) -> String {
    let used_gib = bytes_to_gib(system_monitor.memory_used_bytes);
    let total_gib = bytes_to_gib(system_monitor.memory_total_bytes);
    let mut label = format!(
        "CPU {:.0}%  MEM {:.1}/{:.0}G",
        system_monitor.cpu_percent, used_gib, total_gib
    );
    if let Some(gpu_percent) = system_monitor.gpu_percent {
        label.push_str(&format!("  GPU {:.0}%", gpu_percent));
        if let (Some(gpu_used_bytes), Some(gpu_total_bytes)) = (
            system_monitor.gpu_memory_used_bytes,
            system_monitor.gpu_memory_total_bytes,
        ) {
            label.push_str(&format!(
                " {:.1}/{:.0}G",
                bytes_to_gib(gpu_used_bytes),
                bytes_to_gib(gpu_total_bytes)
            ));
        }
    }
    label
}

fn bytes_to_gib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}

#[cfg(test)]
fn build_context_usage_label(usage: &HeaderUsageDisplay) -> String {
    let clamped_pct = usage.utilization_pct.min(100);
    let bar: String = context_bar_cells(usage)
        .iter()
        .map(|cell| cell.ch)
        .collect();
    let context_window = format_token_count(usage.context_window_tokens.max(1));
    format!("ctx [{bar}] {clamped_pct}% [{context_window}]")
}

fn context_usage_line(usage: &HeaderUsageDisplay, theme: &ThemeTokens) -> Line<'static> {
    let band = context_fill_band(usage);
    let fill_style = context_fill_style(band, theme);
    let pct = usage.utilization_pct.min(100);
    let context_window = format!(
        "[{}]",
        format_token_count(usage.context_window_tokens.max(1))
    );

    let mut spans = Vec::with_capacity(CONTEXT_BAR_WIDTH + 8);
    spans.push(Span::styled("ctx ", theme.fg_dim));
    spans.push(Span::styled("[", theme.fg_dim));
    for cell in context_bar_cells(usage) {
        let style = match cell.kind {
            ContextBarCellKind::Fill => fill_style,
            ContextBarCellKind::Empty => theme.fg_dim,
            ContextBarCellKind::Marker => theme.accent_primary,
        };
        spans.push(Span::styled(cell.ch.to_string(), style));
    }
    spans.push(Span::styled("] ", theme.fg_dim));
    spans.push(Span::styled(format!("{pct}%"), fill_style));
    spans.push(Span::styled(" ", theme.fg_dim));
    spans.push(Span::styled(context_window, theme.fg_dim));
    if let Some(compaction_window) = usage.compaction_model_window_tokens {
        spans.push(Span::styled(
            format!(" cmpct[{}]", format_token_count(compaction_window.max(1))),
            theme.fg_dim,
        ));
    }
    Line::from(spans)
}

fn context_fill_style(band: ContextFillBand, theme: &ThemeTokens) -> Style {
    match band {
        ContextFillBand::Green => theme.accent_success,
        ContextFillBand::Orange => theme.accent_secondary,
        ContextFillBand::Red => theme.accent_danger,
    }
}

fn context_fill_band(usage: &HeaderUsageDisplay) -> ContextFillBand {
    let pct = usage.utilization_pct.min(100);
    let compaction_threshold_pct = usage
        .compaction_target_tokens
        .saturating_mul(100)
        .checked_div(usage.context_window_tokens.max(1))
        .unwrap_or(100)
        .min(100) as u8;

    if pct < 40 {
        ContextFillBand::Green
    } else if pct < 60 {
        ContextFillBand::Orange
    } else if pct < compaction_threshold_pct {
        ContextFillBand::Red
    } else {
        ContextFillBand::Red
    }
}

fn context_bar_cells(usage: &HeaderUsageDisplay) -> Vec<ContextBarCell> {
    let filled = usage
        .current_tokens
        .saturating_mul(CONTEXT_BAR_WIDTH as u64)
        .checked_div(usage.context_window_tokens.max(1))
        .unwrap_or(0)
        .min(CONTEXT_BAR_WIDTH as u64) as usize;
    let marker_index = usage
        .compaction_target_tokens
        .saturating_mul(CONTEXT_BAR_WIDTH as u64)
        .div_ceil(usage.context_window_tokens.max(1))
        .clamp(1, CONTEXT_BAR_WIDTH as u64)
        .saturating_sub(1) as usize;

    let mut cells = Vec::with_capacity(CONTEXT_BAR_WIDTH);
    for index in 0..CONTEXT_BAR_WIDTH {
        let (ch, kind) = if index == marker_index {
            ('|', ContextBarCellKind::Marker)
        } else if index < filled {
            ('=', ContextBarCellKind::Fill)
        } else {
            ('-', ContextBarCellKind::Empty)
        };
        cells.push(ContextBarCell { ch, kind });
    }
    cells
}

pub fn hit_test(
    area: Rect,
    pending_approvals: usize,
    unread_notifications: usize,
    position: Position,
) -> Option<HeaderHitTarget> {
    let inner = Block::default().borders(Borders::BOTTOM).inner(area);
    let approvals_width = approval_area_width(pending_approvals).min(inner.width);
    let bell_width = bell_area_width(unread_notifications).min(inner.width);
    let bell_x = inner
        .x
        .saturating_add(inner.width.saturating_sub(bell_width));
    let approval_x = bell_x.saturating_sub(1).saturating_sub(approvals_width);
    let approval_area = Rect::new(approval_x, inner.y, approvals_width, inner.height);
    let bell_area = Rect::new(bell_x, inner.y, bell_width, inner.height);

    if position.x >= approval_area.x
        && position.x < approval_area.x.saturating_add(approval_area.width)
        && position.y >= approval_area.y
        && position.y < approval_area.y.saturating_add(approval_area.height)
    {
        Some(HeaderHitTarget::ApprovalBadge)
    } else if position.x >= bell_area.x
        && position.x < bell_area.x.saturating_add(bell_area.width)
        && position.y >= bell_area.y
        && position.y < bell_area.y.saturating_add(bell_area.height)
    {
        Some(HeaderHitTarget::NotificationBell)
    } else {
        None
    }
}

fn approval_area_width(pending_approvals: usize) -> u16 {
    if pending_approvals > 9 {
        6
    } else {
        4
    }
}

fn bell_area_width(unread_notifications: usize) -> u16 {
    if unread_notifications > 9 {
        8
    } else {
        6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_usage_label_includes_thread_tokens_and_cost() {
        let label = build_total_usage_label(
            &HeaderUsageDisplay {
                total_thread_tokens: 21_000_000,
                current_tokens: 64_000,
                context_window_tokens: 128_000,
                compaction_target_tokens: 102_400,
                compaction_model_window_tokens: None,
                utilization_pct: 50,
                total_cost_usd: Some(1.25),
            },
            None,
        );

        assert!(
            label.contains("21.0M tok"),
            "label should include cumulative thread tokens: {label}"
        );
        assert!(
            label.starts_with("⚡ "),
            "label should prefix cumulative thread tokens with a thunder icon: {label}"
        );
        assert!(
            label.contains("$1.2500"),
            "label should include total cost: {label}"
        );
    }

    #[test]
    fn total_usage_label_includes_friendly_session_duration() {
        let label = build_total_usage_label(
            &HeaderUsageDisplay {
                total_thread_tokens: 141_300_000,
                current_tokens: 240_000,
                context_window_tokens: 400_000,
                compaction_target_tokens: 320_000,
                compaction_model_window_tokens: None,
                utilization_pct: 60,
                total_cost_usd: None,
            },
            Some(65 * 60),
        );

        assert!(
            label.contains("141.3M tok"),
            "label should keep cumulative thread tokens: {label}"
        );
        assert!(
            label.contains("🕒 1h 05m"),
            "label should include a clock icon with friendly elapsed time: {label}"
        );
    }

    #[test]
    fn friendly_session_duration_formats_seconds_minutes_hours_and_days() {
        assert_eq!(format_session_duration(42), "42s");
        assert_eq!(format_session_duration(5 * 60), "5m");
        assert_eq!(format_session_duration(65 * 60), "1h 05m");
        assert_eq!(format_session_duration(26 * 60 * 60), "1d 02h");
    }

    #[test]
    fn context_usage_label_includes_progress_percent_full_context_and_compaction_marker() {
        let label = build_context_usage_label(&HeaderUsageDisplay {
            total_thread_tokens: 21_000_000,
            current_tokens: 64_000,
            context_window_tokens: 400_000,
            compaction_target_tokens: 320_000,
            compaction_model_window_tokens: None,
            utilization_pct: 16,
            total_cost_usd: Some(1.25),
        });

        assert!(
            label.contains("ctx"),
            "label should identify context usage: {label}"
        );
        assert!(
            label.contains("16%"),
            "label should include utilization percent against the full context: {label}"
        );
        assert!(
            label.contains("[400.0k tok]"),
            "label should include full context window tokens: {label}"
        );
        assert!(
            label.contains("|"),
            "label should include a compaction marker: {label}"
        );
        assert!(
            label.contains("="),
            "label should include a visible progress fill: {label}"
        );
    }

    #[test]
    fn context_usage_line_surfaces_compaction_model_window_when_present() {
        // Why this matters: when the compaction model (Weles or CustomModel) has
        // a smaller window than the main model, every input that exceeds it
        // legitimately falls back to heuristic compaction. Without surfacing
        // this in the header, the user has no way to understand why their
        // configured LLM compaction never seems to run.
        let theme = ThemeTokens::default();
        let line = context_usage_line(
            &HeaderUsageDisplay {
                total_thread_tokens: 0,
                current_tokens: 32_000,
                context_window_tokens: 200_000,
                compaction_target_tokens: 160_000,
                compaction_model_window_tokens: Some(8_000),
                utilization_pct: 16,
                total_cost_usd: None,
            },
            &theme,
        );
        let plain: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(
            plain.contains("cmpct[8.0k tok]"),
            "compaction window must appear when configured smaller than primary: {plain}"
        );
    }

    #[test]
    fn context_usage_line_omits_compaction_model_window_when_absent() {
        // Heuristic strategy has no separate compaction model — header should
        // stay clean rather than render a redundant marker.
        let theme = ThemeTokens::default();
        let line = context_usage_line(
            &HeaderUsageDisplay {
                total_thread_tokens: 0,
                current_tokens: 32_000,
                context_window_tokens: 200_000,
                compaction_target_tokens: 160_000,
                compaction_model_window_tokens: None,
                utilization_pct: 16,
                total_cost_usd: None,
            },
            &theme,
        );
        let plain: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(
            !plain.contains("cmpct"),
            "no compaction marker should appear when None: {plain}"
        );
    }

    #[test]
    fn context_usage_label_clamps_overflowing_progress_to_full_bar() {
        let label = build_context_usage_label(&HeaderUsageDisplay {
            total_thread_tokens: 21_000_000,
            current_tokens: 500_000,
            context_window_tokens: 400_000,
            compaction_target_tokens: 320_000,
            compaction_model_window_tokens: None,
            utilization_pct: 100,
            total_cost_usd: None,
        });

        assert!(
            label.contains("100%"),
            "overflow should clamp to 100 percent: {label}"
        );
        assert!(
            label.contains("[===============|====]"),
            "overflow should render as a full context bar with a compaction marker: {label}"
        );
    }

    #[test]
    fn context_fill_band_uses_green_orange_and_red_thresholds() {
        assert_eq!(
            context_fill_band(&HeaderUsageDisplay {
                total_thread_tokens: 0,
                current_tokens: 0,
                context_window_tokens: 400_000,
                compaction_target_tokens: 320_000,
                compaction_model_window_tokens: None,
                utilization_pct: 39,
                total_cost_usd: None,
            }),
            ContextFillBand::Green
        );
        assert_eq!(
            context_fill_band(&HeaderUsageDisplay {
                total_thread_tokens: 0,
                current_tokens: 0,
                context_window_tokens: 400_000,
                compaction_target_tokens: 320_000,
                compaction_model_window_tokens: None,
                utilization_pct: 55,
                total_cost_usd: None,
            }),
            ContextFillBand::Orange
        );
        assert_eq!(
            context_fill_band(&HeaderUsageDisplay {
                total_thread_tokens: 0,
                current_tokens: 0,
                context_window_tokens: 400_000,
                compaction_target_tokens: 320_000,
                compaction_model_window_tokens: None,
                utilization_pct: 70,
                total_cost_usd: None,
            }),
            ContextFillBand::Red
        );
        assert_eq!(
            context_fill_band(&HeaderUsageDisplay {
                total_thread_tokens: 0,
                current_tokens: 0,
                context_window_tokens: 400_000,
                compaction_target_tokens: 320_000,
                compaction_model_window_tokens: None,
                utilization_pct: 90,
                total_cost_usd: None,
            }),
            ContextFillBand::Red
        );
    }

    #[test]
    fn system_monitor_label_formats_cpu_memory_and_gpu_when_present() {
        let label = build_system_monitor_label(&SystemMonitorDisplay {
            cpu_percent: 12.4,
            memory_used_bytes: 8 * 1024 * 1024 * 1024,
            memory_total_bytes: 32 * 1024 * 1024 * 1024,
            gpu_percent: Some(47.6),
            gpu_memory_used_bytes: Some(3277 * 1024 * 1024),
            gpu_memory_total_bytes: Some(24 * 1024 * 1024 * 1024),
        });

        assert_eq!(label, "CPU 12%  MEM 8.0/32G  GPU 48% 3.2/24G");
    }

    #[test]
    fn system_monitor_label_omits_gpu_when_absent() {
        let label = build_system_monitor_label(&SystemMonitorDisplay {
            cpu_percent: 4.0,
            memory_used_bytes: 512 * 1024 * 1024,
            memory_total_bytes: 2 * 1024 * 1024 * 1024,
            gpu_percent: None,
            gpu_memory_used_bytes: None,
            gpu_memory_total_bytes: None,
        });

        assert_eq!(label, "CPU 4%  MEM 0.5/2G");
    }

    #[test]
    fn bell_hit_test_detects_click_inside_bell_area() {
        let area = Rect::new(0, 0, 80, 3);
        let hit = hit_test(area, 0, 3, Position::new(78, 1));
        assert_eq!(hit, Some(HeaderHitTarget::NotificationBell));
    }

    #[test]
    fn approval_hit_test_detects_click_inside_approval_area() {
        let area = Rect::new(0, 0, 80, 3);
        let hit = hit_test(area, 2, 0, Position::new(72, 1));
        assert_eq!(hit, Some(HeaderHitTarget::ApprovalBadge));
    }
}
