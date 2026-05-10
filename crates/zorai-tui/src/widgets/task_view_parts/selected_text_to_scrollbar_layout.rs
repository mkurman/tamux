use super::selection::*;
use super::*;
use crate::state::sidebar::SidebarItemTarget;
use crate::state::task::*;
use crate::theme::ThemeTokens;
use crate::widgets::chat::SelectionPoint;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
pub(crate) fn selected_text(
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    scroll: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
    start: SelectionPoint,
    end: SelectionPoint,
) -> Option<String> {
    let snapshot = selection_snapshot(
        area,
        tasks,
        target,
        theme,
        scroll,
        show_live_todos,
        show_timeline,
        show_files,
    )?;
    let (start_point, end_point) =
        if start.row <= end.row || (start.row == end.row && start.col <= end.col) {
            (start, end)
        } else {
            (end, start)
        };
    if start_point == end_point {
        return None;
    }

    let mut lines = Vec::new();
    for row in start_point.row..=end_point.row {
        let rendered = snapshot.rows.get(row)?;
        let plain = line_plain_text(&rendered.line);
        let width = line_display_width(&rendered.line);
        let from = if row == start_point.row {
            start_point.col.min(width)
        } else {
            0
        };
        let to = if row == end_point.row {
            end_point.col.min(width).max(from)
        } else {
            width
        };
        lines.push(display_slice(&plain, from, to));
    }

    let text = lines.join("\n");
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    _focused: bool,
    scroll: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
    current_tick: u64,
    mouse_selection: Option<(SelectionPoint, SelectionPoint)>,
) {
    let inner = content_inner(area);

    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // Single-pass row build. The previous structure called `rows_for_width`
    // up to three times per frame: scrollbar_layout did it twice (full
    // width + content width), then the render path called it again. With
    // an active goal-run forcing fast-tick redraws, that meant 15+ row-
    // tree rebuilds per second — the user-visible 2× CPU + delayed input.
    //
    // Strategy: compute rows ONCE at content_width (inner.width minus the
    // scrollbar gutter). Wrapping at the narrower width gives ≥ rows than
    // at inner.width, so "fits at content_width" implies "fits at full
    // width" — safe to use this for both the scrollbar-needed decision
    // and the actual render. Worst case: 1 call (was 3).
    let content_width = inner.width.saturating_sub(SCROLLBAR_WIDTH).max(1) as usize;
    let mut lines = rows_for_width(
        tasks,
        target,
        theme,
        content_width,
        show_live_todos,
        show_timeline,
        show_files,
        Some(current_tick),
    );

    if let Some(layout) = scrollbar_layout_from_metrics(inner, lines.len(), scroll) {
        if let Some((start, end)) = mouse_selection {
            let (start_point, end_point) =
                if start.row <= end.row || (start.row == end.row && start.col <= end.col) {
                    (start, end)
                } else {
                    (end, start)
                };
            let highlight = Style::default().bg(Color::Indexed(31));
            for row in start_point.row..=end_point.row {
                if let Some(rendered) = lines.get_mut(row) {
                    let line_width = line_display_width(&rendered.line);
                    let from = if row == start_point.row {
                        start_point.col.min(line_width)
                    } else {
                        0
                    };
                    let to = if row == end_point.row {
                        end_point.col.min(line_width).max(from)
                    } else {
                        line_width
                    };
                    highlight_line_range(&mut rendered.line, from, to, highlight);
                }
            }
        }
        let lines = lines.into_iter().map(|row| row.line).collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines).scroll((layout.scroll as u16, 0));
        frame.render_widget(paragraph, layout.content);

        let scrollbar_lines = (0..layout.scrollbar.height)
            .map(|offset| {
                let y = layout.scrollbar.y.saturating_add(offset);
                let (glyph, style) = if y >= layout.thumb.y
                    && y < layout.thumb.y.saturating_add(layout.thumb.height)
                {
                    ("█", theme.accent_primary)
                } else {
                    ("│", theme.fg_dim)
                };
                Line::from(Span::styled(glyph, style))
            })
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(scrollbar_lines), layout.scrollbar);
        return;
    }

    // No scrollbar needed: render the same content_width-wrapped rows
    // directly across `inner` (the small visual gap on the right where
    // the scrollbar would be is invisible — empty space — and saves
    // a second row-build pass).
    if let Some((start, end)) = mouse_selection {
        let (start_point, end_point) =
            if start.row <= end.row || (start.row == end.row && start.col <= end.col) {
                (start, end)
            } else {
                (end, start)
            };
        let highlight = Style::default().bg(Color::Indexed(31));
        for row in start_point.row..=end_point.row {
            if let Some(rendered) = lines.get_mut(row) {
                let line_width = line_display_width(&rendered.line);
                let from = if row == start_point.row {
                    start_point.col.min(line_width)
                } else {
                    0
                };
                let to = if row == end_point.row {
                    end_point.col.min(line_width).max(from)
                } else {
                    line_width
                };
                highlight_line_range(&mut rendered.line, from, to, highlight);
            }
        }
    }
    let max_scroll = lines.len().saturating_sub(inner.height as usize);
    let lines = lines.into_iter().map(|row| row.line).collect::<Vec<_>>();
    let paragraph = Paragraph::new(lines).scroll((scroll.min(max_scroll) as u16, 0));
    frame.render_widget(paragraph, inner);
}

pub(crate) fn max_scroll(
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
) -> usize {
    let inner = content_inner(area);
    if inner.width == 0 || inner.height == 0 {
        return 0;
    }

    scrollbar_layout(
        area,
        tasks,
        target,
        theme,
        0,
        show_live_todos,
        show_timeline,
        show_files,
    )
    .map(|layout| layout.max_scroll)
    .unwrap_or_else(|| {
        let rows = rows_for_width(
            tasks,
            target,
            theme,
            inner.width as usize,
            show_live_todos,
            show_timeline,
            show_files,
            None,
        );
        rows.len().saturating_sub(inner.height as usize)
    })
}

pub(crate) fn scrollbar_layout(
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    scroll: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
) -> Option<TaskViewScrollbarLayout> {
    let inner = content_inner(area);
    if inner.width <= SCROLLBAR_WIDTH || inner.height == 0 {
        return None;
    }

    let full_rows = rows_for_width(
        tasks,
        target,
        theme,
        inner.width as usize,
        show_live_todos,
        show_timeline,
        show_files,
        None,
    );
    if full_rows.len() <= inner.height as usize {
        return None;
    }

    let content_width = inner.width.saturating_sub(SCROLLBAR_WIDTH) as usize;
    let rows = rows_for_width(
        tasks,
        target,
        theme,
        content_width,
        show_live_todos,
        show_timeline,
        show_files,
        None,
    );
    scrollbar_layout_from_metrics(inner, rows.len(), scroll)
}
