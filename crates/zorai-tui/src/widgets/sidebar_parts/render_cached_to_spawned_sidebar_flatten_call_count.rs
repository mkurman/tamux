use super::tab_layout::*;
use super::*;
use crate::app::RecentActionVm;
use crate::state::chat::{ChatState, GatewayStatusVm};
use crate::state::sidebar::{SidebarState, SidebarTab};
use crate::state::task::TaskState;
use crate::state::tier::TierState;
use crate::theme::ThemeTokens;
use ratatui::prelude::*;
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
pub(crate) fn render_cached(
    frame: &mut Frame,
    area: Rect,
    chat: &ChatState,
    sidebar: &SidebarState,
    theme: &ThemeTokens,
    focused: bool,
    gateway_statuses: &[GatewayStatusVm],
    tier: &TierState,
    recent_actions: &[RecentActionVm],
    snapshot: &CachedSidebarSnapshot,
) {
    if area.height < 3 {
        return;
    }

    let gw_lines = if tier.show_gateway_config {
        gateway_status_lines(gateway_statuses, theme)
    } else {
        Vec::new()
    };
    let gw_height = gw_lines.len() as u16;
    let show_spawned = snapshot.show_spawned();
    let show_pinned = snapshot.show_pinned();
    let filter_height = if sidebar.active_tab() == SidebarTab::Files {
        1
    } else {
        0
    };
    let mut footer_lines = Vec::new();
    if chat.can_go_back_thread() {
        footer_lines.push(thread_history_footer_line(
            theme,
            chat.thread_navigation_depth(),
        ));
    }
    if sidebar.active_tab() == SidebarTab::Spawned {
        footer_lines.push(spawned_footer_line(theme));
    }
    if sidebar.active_tab() == SidebarTab::Pinned {
        footer_lines.push(pinned_footer_line(theme));
    }
    let footer_height = footer_lines.len() as u16;

    let ra_lines = recent_actions_lines(recent_actions, theme);
    let ra_height = ra_lines.len() as u16;

    let tier_lines = tier_gated_lines(tier);
    let tier_height = tier_lines.len() as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(filter_height),
            Constraint::Min(1),
            Constraint::Length(gw_height),
            Constraint::Length(ra_height),
            Constraint::Length(tier_height),
            Constraint::Length(footer_height),
        ])
        .split(area);


    for (tab, cell) in tab_cells(chunks[0], show_spawned, show_pinned) {
        let style = if sidebar.active_tab() == tab {
            theme.fg_active.bg(Color::Indexed(236))
        } else {
            theme.fg_dim
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(tab_label(tab), style)))
                .alignment(Alignment::Center),
            cell,
        );
    }

    if filter_height > 0 {
        let filter_text = if sidebar.files_filter().is_empty() {
            " Filter: type to search".to_string()
        } else {
            format!(" Filter: {}", sidebar.files_filter())
        };
        let style = if focused && sidebar.active_tab() == SidebarTab::Files {
            theme.fg_active.bg(Color::Indexed(236))
        } else {
            theme.fg_dim
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(filter_text, style))),
            chunks[1],
        );
    }

    let body_idx = 2;
    let scroll = resolved_scroll(
        snapshot.item_count(),
        sidebar,
        chunks[body_idx].height as usize,
    );
    let rows = visible_rows(
        snapshot,
        sidebar,
        theme,
        chunks[body_idx].width as usize,
        chunks[body_idx].height as usize,
        scroll,
    );
    let paragraph = Paragraph::new(rows.into_iter().map(|row| row.line).collect::<Vec<_>>());
    frame.render_widget(paragraph, chunks[body_idx]);

    if !gw_lines.is_empty() {
        frame.render_widget(Paragraph::new(gw_lines), chunks[body_idx + 1]);
    }

    if !ra_lines.is_empty() {
        frame.render_widget(Paragraph::new(ra_lines), chunks[body_idx + 2]);
    }

    if !tier_lines.is_empty() {
        frame.render_widget(Paragraph::new(tier_lines), chunks[body_idx + 3]);
    }

    if footer_height > 0 {
        frame.render_widget(Paragraph::new(footer_lines), chunks[body_idx + 4]);
    }
}

pub(crate) fn body_item_count(
    tasks: &TaskState,
    chat: &ChatState,
    sidebar: &SidebarState,
    thread_id: Option<&str>,
) -> usize {
    build_cached_snapshot(Rect::new(0, 0, 80, 0), chat, sidebar, tasks, thread_id).item_count()
}

pub(crate) fn hit_test_cached(
    area: Rect,
    sidebar: &SidebarState,
    snapshot: &CachedSidebarSnapshot,
    mouse: Position,
) -> Option<SidebarHitTarget> {
    if area.height < 3
        || mouse.x < area.x
        || mouse.x >= area.x.saturating_add(area.width)
        || mouse.y < area.y
        || mouse.y >= area.y.saturating_add(area.height)
    {
        return None;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(if sidebar.active_tab() == SidebarTab::Files {
                1
            } else {
                0
            }),
            Constraint::Min(1),
        ])
        .split(area);

    if mouse.y == chunks[0].y {
        return tab_hit_test(
            chunks[0],
            mouse.x,
            snapshot.show_spawned(),
            snapshot.show_pinned(),
        )
        .map(SidebarHitTarget::Tab);
    }

    if sidebar.active_tab() == SidebarTab::Files && mouse.y == chunks[1].y {
        return None;
    }
    let body_idx = 2;
    let scroll = resolved_scroll(
        snapshot.item_count(),
        sidebar,
        chunks[body_idx].height as usize,
    );
    let row_idx = scroll + mouse.y.saturating_sub(chunks[body_idx].y) as usize;
    snapshot.row_target(row_idx)
}

#[cfg(test)]
pub(crate) fn hit_test(
    area: Rect,
    chat: &ChatState,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
    mouse: Position,
) -> Option<SidebarHitTarget> {
    let snapshot = build_cached_snapshot(area, chat, sidebar, tasks, thread_id);
    hit_test_cached(area, sidebar, &snapshot, mouse)
}

#[cfg(test)]
pub(crate) fn reset_build_cached_snapshot_call_count() {
    BUILD_CACHED_SNAPSHOT_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
pub(crate) fn build_cached_snapshot_call_count() -> usize {
    BUILD_CACHED_SNAPSHOT_CALLS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_spawned_sidebar_flatten_call_count() {
    spawned_agents::reset_flattened_items_call_count();
}

#[cfg(test)]
pub(crate) fn spawned_sidebar_flatten_call_count() -> usize {
    spawned_agents::flattened_items_call_count()
}
