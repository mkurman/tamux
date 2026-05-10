use crate::state::sidebar::SidebarTab;
use crate::theme::ThemeTokens;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};

pub(super) fn visible_tabs(show_spawned: bool, show_pinned: bool) -> Vec<SidebarTab> {
    let mut tabs = vec![SidebarTab::Todos, SidebarTab::Files];
    if show_spawned {
        tabs.push(SidebarTab::Spawned);
    }
    if show_pinned {
        tabs.push(SidebarTab::Pinned);
    }
    tabs
}

pub(crate) fn tab_hit_test(
    tab_area: Rect,
    mouse_x: u16,
    show_spawned: bool,
    show_pinned: bool,
) -> Option<SidebarTab> {
    tab_cells(tab_area, show_spawned, show_pinned)
        .into_iter()
        .find_map(|(tab, rect)| {
            (mouse_x >= rect.x && mouse_x < rect.x.saturating_add(rect.width)).then_some(tab)
        })
}

pub(crate) fn tab_cells(
    tab_area: Rect,
    show_spawned: bool,
    show_pinned: bool,
) -> Vec<(SidebarTab, Rect)> {
    let tabs = visible_tabs(show_spawned, show_pinned);
    if tabs.is_empty() {
        return Vec::new();
    }
    let percent = 100 / tabs.len() as u16;
    let mut constraints = vec![Constraint::Percentage(percent); tabs.len()];
    if let Some(last) = constraints.last_mut() {
        *last = Constraint::Min(0);
    }
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(tab_area);
    tabs.into_iter().zip(chunks.iter().copied()).collect()
}

pub(crate) fn tab_label(tab: SidebarTab) -> &'static str {
    match tab {
        SidebarTab::Files => " Files ",
        SidebarTab::Todos => " Todos ",
        SidebarTab::Spawned => " Spawned ",
        SidebarTab::Pinned => " Pinned ",
    }
}

#[allow(dead_code)]
pub(super) fn tab_hint_line(theme: &ThemeTokens) -> Line<'static> {
    let mut spans = vec![
        Span::styled("[", theme.accent_primary),
        Span::styled(" todos ", theme.fg_dim),
        Span::styled("]", theme.accent_primary),
        Span::styled("  ", theme.fg_dim),
        Span::styled("[", theme.accent_primary),
        Span::styled(" files ", theme.fg_dim),
        Span::styled("]", theme.accent_primary),
    ];
    spans.extend([
        Span::styled("  ", theme.fg_dim),
        Span::styled("[", theme.accent_primary),
        Span::styled(" spawned ", theme.fg_dim),
        Span::styled("]", theme.accent_primary),
        Span::styled("  ", theme.fg_dim),
        Span::styled("[", theme.accent_primary),
        Span::styled(" pinned ", theme.fg_dim),
        Span::styled("]", theme.accent_primary),
        Span::styled("  click tab", theme.fg_dim),
    ]);
    Line::from(spans)
}
