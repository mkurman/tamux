use super::selection::*;
use super::*;
use crate::state::sidebar::SidebarTab;
use crate::state::task::TaskState;
use crate::terminal_graphics::TerminalImageOverlaySpec;
use crate::theme::ThemeTokens;
use crate::widgets::chat::SelectionPoint;
use crate::widgets::image_preview;
use ratatui::prelude::*;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
pub(crate) fn selection_points_from_mouse(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
    start: Position,
    end: Position,
) -> Option<(SelectionPoint, SelectionPoint)> {
    let snapshot = selection_snapshot(
        area,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    )?;
    Some((
        selection_point_from_snapshot(&snapshot, start)?,
        selection_point_from_snapshot(&snapshot, end)?,
    ))
}

pub(crate) fn selection_point_from_mouse(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
    mouse: Position,
) -> Option<SelectionPoint> {
    let snapshot = selection_snapshot(
        area,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    )?;
    selection_point_from_snapshot(&snapshot, mouse)
}

pub(crate) fn selected_text(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
    start: SelectionPoint,
    end: SelectionPoint,
) -> Option<String> {
    let snapshot = selection_snapshot(
        area,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
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
        let rendered = snapshot.all_lines.get(row)?;
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

pub(crate) fn hit_test(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    scroll: usize,
    mouse: Position,
    theme: &ThemeTokens,
) -> Option<WorkContextHitTarget> {
    if active_tab == SidebarTab::Files {
        let snapshot =
            sticky_files_snapshot(area, tasks, thread_id, selected_index, theme, scroll)?;
        if !snapshot.header_area.contains(mouse) {
            return None;
        }
        let row = mouse
            .y
            .saturating_sub(snapshot.header_area.y)
            .min(snapshot.header_area.height.saturating_sub(1)) as usize;
        return snapshot.header_lines.get(row).and_then(|line| {
            if line.close_preview {
                Some(WorkContextHitTarget::ClosePreview)
            } else {
                None
            }
        });
    }

    let layout = scrollbar_layout(
        area,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    );
    let content = layout.map(|layout| layout.content).unwrap_or(area);
    if !content.contains(mouse) {
        return None;
    }

    let row_index = layout
        .map(|layout| layout.scroll)
        .unwrap_or(scroll)
        .saturating_add(mouse.y.saturating_sub(content.y) as usize);
    build_lines(
        content,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    )
    .get(row_index)
    .and_then(|line| {
        if line.close_preview {
            Some(WorkContextHitTarget::ClosePreview)
        } else {
            None
        }
    })
}

pub(crate) fn render(
    frame: &mut Frame,
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
    mouse_selection: Option<(SelectionPoint, SelectionPoint)>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if active_tab == SidebarTab::Files {
        let Some(mut snapshot) =
            sticky_files_snapshot(area, tasks, thread_id, selected_index, theme, scroll)
        else {
            return;
        };

        if let Some((start, end)) = mouse_selection {
            let (start_point, end_point) =
                if start.row <= end.row || (start.row == end.row && start.col <= end.col) {
                    (start, end)
                } else {
                    (end, start)
                };
            let highlight = Style::default().bg(Color::Indexed(31));
            for row in start_point.row..=end_point.row {
                let line = if row < snapshot.header_lines.len() {
                    snapshot.header_lines.get_mut(row)
                } else {
                    snapshot
                        .body_lines
                        .get_mut(row.saturating_sub(snapshot.header_lines.len()))
                };
                if let Some(line) = line {
                    let line_width = line_display_width(&line.line);
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
                    highlight_line_range(&mut line.line, from, to, highlight);
                }
            }
        }

        if snapshot.header_area.height > 0 {
            let visible_header = snapshot
                .header_lines
                .into_iter()
                .take(snapshot.header_area.height as usize)
                .map(|line| line.line)
                .collect::<Vec<_>>();
            frame.render_widget(Paragraph::new(visible_header), snapshot.header_area);
        }

        let visible_body = snapshot
            .body_lines
            .into_iter()
            .skip(snapshot.scroll)
            .take(snapshot.body_area.height as usize)
            .map(|line| line.line)
            .collect::<Vec<_>>();

        if let Some(layout) = snapshot.layout {
            frame.render_widget(Paragraph::new(visible_body), layout.content);

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
        } else {
            frame.render_widget(Paragraph::new(visible_body), snapshot.body_area);
        }
        return;
    }

    let layout = scrollbar_layout(
        area,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    );
    let content = layout.map(|layout| layout.content).unwrap_or(area);
    let resolved_scroll = layout.map(|layout| layout.scroll).unwrap_or(scroll);
    let mut all_lines = build_lines(
        content,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        resolved_scroll,
    );

    if let Some((start, end)) = mouse_selection {
        let (start_point, end_point) =
            if start.row <= end.row || (start.row == end.row && start.col <= end.col) {
                (start, end)
            } else {
                (end, start)
            };
        let highlight = Style::default().bg(Color::Indexed(31));
        for row in start_point.row..=end_point.row {
            if let Some(line) = all_lines.get_mut(row) {
                let line_width = line_display_width(&line.line);
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
                highlight_line_range(&mut line.line, from, to, highlight);
            }
        }
    }

    let visible = all_lines
        .into_iter()
        .skip(resolved_scroll)
        .take(content.height as usize)
        .map(|line| line.line)
        .collect::<Vec<_>>();

    if let Some(layout) = layout {
        frame.render_widget(Paragraph::new(visible), layout.content);

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
    } else {
        frame.render_widget(Paragraph::new(visible), content);
    }
}

pub(crate) fn max_scroll(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
) -> usize {
    if area.width == 0 || area.height == 0 {
        return 0;
    }

    if active_tab == SidebarTab::Files {
        return sticky_files_snapshot(area, tasks, thread_id, selected_index, theme, 0)
            .map(|snapshot| snapshot.max_scroll)
            .unwrap_or(0);
    }

    scrollbar_layout(area, tasks, thread_id, active_tab, selected_index, theme, 0)
        .map(|layout| layout.max_scroll)
        .unwrap_or_else(|| {
            build_lines(area, tasks, thread_id, active_tab, selected_index, theme, 0)
                .len()
                .saturating_sub(area.height as usize)
        })
}

pub(crate) fn scrollbar_layout(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<WorkContextScrollbarLayout> {
    if area.width <= SCROLLBAR_WIDTH || area.height == 0 {
        return None;
    }

    if active_tab == SidebarTab::Files {
        return sticky_files_snapshot(area, tasks, thread_id, selected_index, theme, scroll)
            .and_then(|snapshot| snapshot.layout);
    }

    let full_lines = build_lines(
        area,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    );
    if full_lines.len() <= area.height as usize {
        return None;
    }

    let content = Rect::new(
        area.x,
        area.y,
        area.width.saturating_sub(SCROLLBAR_WIDTH),
        area.height,
    );
    let all_lines = build_lines(
        content,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    );
    scrollbar_layout_from_metrics(area, all_lines.len(), scroll)
}

pub(crate) fn terminal_image_overlay_spec(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<TerminalImageOverlaySpec> {
    if active_tab != SidebarTab::Files || scroll != 0 {
        return None;
    }

    let thread_id = thread_id?;
    let context = tasks.work_context_for_thread(thread_id)?;
    let entry = context.entries.get(selected_index)?;
    if !uses_terminal_graphics(&entry.path, entry.repo_root.as_deref(), active_tab, scroll) {
        return None;
    }

    let content =
        sticky_files_snapshot(area, tasks, Some(thread_id), selected_index, theme, scroll)
            .map(|snapshot| snapshot.body_area)
            .unwrap_or(area);
    let path = image_preview::resolve_local_image_path(&entry.path)?;
    let image_row = content.y.saturating_add(TERMINAL_IMAGE_HEADER_LINES);
    let image_rows = content.height.saturating_sub(TERMINAL_IMAGE_HEADER_LINES);
    if content.width == 0 || image_rows == 0 {
        return None;
    }

    Some(TerminalImageOverlaySpec {
        path,
        column: content.x,
        row: image_row,
        cols: content.width,
        rows: image_rows,
    })
}
