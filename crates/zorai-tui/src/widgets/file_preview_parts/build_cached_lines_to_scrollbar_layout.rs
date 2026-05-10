use super::scroll_offset_from_thumb_offset_to_file_preview_cache_key::*;
use crate::app::ChatFilePreviewTarget;
use crate::state::task::TaskState;
use crate::terminal_graphics::{active_protocol, TerminalImageOverlaySpec, TerminalImageProtocol};
use crate::theme::ThemeTokens;
use crate::widgets::image_preview;
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::sync::Arc;

pub(super) fn build_cached_lines(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    terminal_graphics: bool,
) -> CachedFilePreviewLines {
    let cache_scroll = if terminal_graphics { 0 } else { 1 };
    let header_lines = Arc::new(build_header_lines(target, theme));
    let body_area = file_preview_body_area(area);
    let body_lines = build_body_lines(body_area, tasks, target, theme, cache_scroll);
    if body_area.height == 0
        || body_area.width <= SCROLLBAR_WIDTH
        || body_lines.len() <= body_area.height as usize
    {
        let max_scroll = body_lines.len().saturating_sub(body_area.height as usize);
        return CachedFilePreviewLines {
            key: file_preview_cache_key(area, tasks, target, theme, terminal_graphics),
            header_lines,
            max_scroll,
            body_lines: Arc::new(body_lines),
            body_area,
        };
    }

    let content_body_area = Rect::new(
        body_area.x,
        body_area.y,
        body_area.width.saturating_sub(SCROLLBAR_WIDTH),
        body_area.height,
    );
    let body_lines = build_body_lines(content_body_area, tasks, target, theme, cache_scroll);
    CachedFilePreviewLines {
        key: file_preview_cache_key(area, tasks, target, theme, terminal_graphics),
        max_scroll: body_lines
            .len()
            .saturating_sub(content_body_area.height as usize),
        header_lines,
        body_lines: Arc::new(body_lines),
        body_area: content_body_area,
    }
}

#[cfg(not(test))]
pub(super) const FILE_PREVIEW_CACHE_CAPACITY: usize = 8;
#[cfg(test)]
pub(super) const FILE_PREVIEW_CACHE_CAPACITY: usize = 64;

pub(super) fn snapshot_with_cache(
    cache: &mut FilePreviewRenderCache,
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<FilePreviewSnapshot> {
    if area.width == 0 || area.height == 0 {
        return None;
    }

    let terminal_graphics = uses_terminal_graphics(target, scroll);
    let key = file_preview_cache_key(area, tasks, target, theme, terminal_graphics);
    let cached = cache
        .lines
        .iter()
        .find(|cached| cached.key == key)
        .cloned()
        .unwrap_or_else(|| {
            let cached = build_cached_lines(area, tasks, target, theme, terminal_graphics);
            cache.lines.push(cached.clone());
            if cache.lines.len() > FILE_PREVIEW_CACHE_CAPACITY {
                cache.lines.remove(0);
            }
            cached
        });
    let body_track_area = file_preview_body_area(area);
    let layout = scrollbar_layout_from_metrics(body_track_area, cached.body_lines.len(), scroll);
    let resolved_scroll = layout
        .map(|layout| layout.scroll)
        .unwrap_or_else(|| scroll.min(cached.max_scroll));

    Some(FilePreviewSnapshot {
        header_lines: cached.header_lines,
        body_lines: cached.body_lines,
        scroll: resolved_scroll,
        body_area: layout
            .map(|layout| layout.content)
            .unwrap_or(cached.body_area),
        layout,
        max_scroll: cached.max_scroll,
    })
}

pub(super) fn snapshot(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<FilePreviewSnapshot> {
    let mut cache = lock_file_preview_cache();
    snapshot_with_cache(&mut cache, area, tasks, target, theme, scroll)
}

pub(super) fn selection_snapshot(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<SelectionSnapshot> {
    let snapshot = snapshot(area, tasks, target, theme, scroll)?;
    let header_height = FILE_PREVIEW_HEADER_LINES.min(area.height);
    let header_area = Rect::new(area.x, area.y, area.width, header_height);
    if header_area.height == 0
        && (snapshot.body_lines.is_empty()
            || snapshot.body_area.width == 0
            || snapshot.body_area.height == 0)
    {
        return None;
    }
    Some(SelectionSnapshot {
        header_lines: snapshot.header_lines,
        body_lines: snapshot.body_lines,
        scroll: snapshot.scroll,
        header_area,
        body_area: snapshot.body_area,
    })
}

pub(super) fn selection_point_from_snapshot(
    snapshot: &SelectionSnapshot,
    mouse: Position,
) -> Option<crate::widgets::chat::SelectionPoint> {
    if snapshot.header_area.contains(mouse) {
        let clamped_x = mouse.x.clamp(
            snapshot.header_area.x,
            snapshot
                .header_area
                .x
                .saturating_add(snapshot.header_area.width)
                .saturating_sub(1),
        );
        let visible_header_rows = snapshot.header_area.height as usize;
        let row = (mouse.y.saturating_sub(snapshot.header_area.y) as usize)
            .min(visible_header_rows.saturating_sub(1))
            .min(snapshot.header_lines.len().saturating_sub(1));
        let col = clamped_x.saturating_sub(snapshot.header_area.x) as usize;
        let width = line_display_width(snapshot.header_lines.get(row)?);
        return Some(crate::widgets::chat::SelectionPoint {
            row,
            col: col.min(width),
        });
    }

    let area = snapshot.body_area;
    if area.width == 0 || area.height == 0 || snapshot.body_lines.is_empty() {
        return None;
    }
    let clamped_x = mouse
        .x
        .clamp(area.x, area.x.saturating_add(area.width).saturating_sub(1));
    let clamped_y = mouse
        .y
        .clamp(area.y, area.y.saturating_add(area.height).saturating_sub(1));
    let body_row = snapshot
        .scroll
        .saturating_add(clamped_y.saturating_sub(area.y) as usize)
        .min(snapshot.body_lines.len().saturating_sub(1));
    let row = snapshot.header_lines.len().saturating_add(body_row);
    let col = clamped_x.saturating_sub(area.x) as usize;
    let width = line_display_width(snapshot.body_lines.get(body_row)?);
    Some(crate::widgets::chat::SelectionPoint {
        row,
        col: col.min(width),
    })
}

pub(super) fn selection_line<'a>(
    snapshot: &'a SelectionSnapshot,
    row: usize,
) -> Option<&'a Line<'static>> {
    if row < snapshot.header_lines.len() {
        snapshot.header_lines.get(row)
    } else {
        snapshot
            .body_lines
            .get(row.saturating_sub(snapshot.header_lines.len()))
    }
}

pub fn selection_point_from_mouse(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
    mouse: Position,
) -> Option<crate::widgets::chat::SelectionPoint> {
    let snapshot = selection_snapshot(area, tasks, target, theme, scroll)?;
    selection_point_from_snapshot(&snapshot, mouse)
}

pub fn selection_points_from_mouse(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
    start: Position,
    end: Position,
) -> Option<(
    crate::widgets::chat::SelectionPoint,
    crate::widgets::chat::SelectionPoint,
)> {
    let snapshot = selection_snapshot(area, tasks, target, theme, scroll)?;
    Some((
        selection_point_from_snapshot(&snapshot, start)?,
        selection_point_from_snapshot(&snapshot, end)?,
    ))
}

pub fn selected_text(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
    start: crate::widgets::chat::SelectionPoint,
    end: crate::widgets::chat::SelectionPoint,
) -> Option<String> {
    let snapshot = selection_snapshot(area, tasks, target, theme, scroll)?;
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
        let line = selection_line(&snapshot, row)?;
        let plain = line_plain_text(line);
        let width = line_display_width(line);
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

pub(super) fn apply_mouse_selection_highlight_to_line(
    row: usize,
    line: &mut Line<'static>,
    mouse_selection: Option<(
        crate::widgets::chat::SelectionPoint,
        crate::widgets::chat::SelectionPoint,
    )>,
) {
    let Some((start, end)) = mouse_selection else {
        return;
    };
    let (start_point, end_point) =
        if start.row <= end.row || (start.row == end.row && start.col <= end.col) {
            (start, end)
        } else {
            (end, start)
        };
    if row < start_point.row || row > end_point.row {
        return;
    }

    let line_width = line_display_width(line);
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
    highlight_line_range(line, from, to, Style::default().bg(Color::Indexed(31)));
}

pub(super) fn uses_terminal_graphics(target: &ChatFilePreviewTarget, scroll: usize) -> bool {
    scroll == 0
        && target.repo_root.is_none()
        && active_protocol() != TerminalImageProtocol::None
        && image_preview::resolve_local_image_path(&target.path)
            .as_deref()
            .is_some_and(image_preview::is_previewable_image_path)
}

pub(super) fn push_terminal_graphics_placeholder(
    lines: &mut Vec<Line<'static>>,
    image_preview_height: usize,
    theme: &ThemeTokens,
) {
    lines.push(Line::from(vec![
        Span::styled("Image: ", theme.fg_dim),
        Span::styled("high-quality terminal preview", theme.fg_active),
    ]));
    for _ in 1..image_preview_height {
        lines.push(Line::raw(""));
    }
}

pub fn terminal_image_overlay_spec(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<TerminalImageOverlaySpec> {
    if !uses_terminal_graphics(target, scroll) {
        return None;
    }

    let body = snapshot(area, tasks, target, theme, scroll)?.body_area;
    let path = image_preview::resolve_local_image_path(&target.path)?;
    let image_row = body.y.saturating_add(TERMINAL_IMAGE_HEADER_LINES);
    let image_rows = body.height.saturating_sub(TERMINAL_IMAGE_HEADER_LINES);
    if body.width == 0 || image_rows == 0 {
        return None;
    }

    Some(TerminalImageOverlaySpec {
        path,
        column: body.x,
        row: image_row,
        cols: body.width,
        rows: image_rows,
    })
}

pub fn hit_test(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    scroll: usize,
    mouse: Position,
    theme: &ThemeTokens,
) -> Option<FilePreviewHitTarget> {
    let _ = snapshot(area, tasks, target, theme, scroll)?;
    if !area.contains(mouse) {
        return None;
    }
    if mouse.y == area.y {
        Some(FilePreviewHitTarget::ClosePreview)
    } else {
        None
    }
}

pub(crate) fn scrollbar_scroll_offset_for_pointer(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
    pointer_row: u16,
    grab_offset: u16,
) -> Option<usize> {
    let layout = snapshot(area, tasks, target, theme, scroll)?.layout?;
    let track_span = layout.scrollbar.height.saturating_sub(layout.thumb.height);
    let clamped_row = pointer_row.clamp(
        layout.scrollbar.y,
        layout
            .scrollbar
            .y
            .saturating_add(layout.scrollbar.height)
            .saturating_sub(1),
    );
    let desired_top = clamped_row
        .saturating_sub(layout.scrollbar.y)
        .saturating_sub(grab_offset)
        .min(track_span);

    Some(scroll_offset_from_thumb_offset(
        desired_top,
        track_span,
        layout.max_scroll,
    ))
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
    mouse_selection: Option<(
        crate::widgets::chat::SelectionPoint,
        crate::widgets::chat::SelectionPoint,
    )>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let Some(snapshot) = snapshot(area, tasks, target, theme, scroll) else {
        return;
    };
    let header_height = FILE_PREVIEW_HEADER_LINES.min(area.height);
    if header_height > 0 {
        let header_area = Rect::new(area.x, area.y, area.width, header_height);
        let visible_header = snapshot
            .header_lines
            .iter()
            .take(header_height as usize)
            .enumerate()
            .map(|(row, line)| {
                let mut line = line.clone();
                apply_mouse_selection_highlight_to_line(row, &mut line, mouse_selection);
                line
            })
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(visible_header), header_area);
    }

    let body_row_offset = snapshot.header_lines.len();
    let visible = snapshot
        .body_lines
        .iter()
        .enumerate()
        .skip(snapshot.scroll)
        .take(snapshot.body_area.height as usize)
        .map(|(row, line)| {
            let mut line = line.clone();
            apply_mouse_selection_highlight_to_line(
                body_row_offset.saturating_add(row),
                &mut line,
                mouse_selection,
            );
            line
        })
        .collect::<Vec<_>>();

    if let Some(layout) = snapshot.layout {
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
        frame.render_widget(Paragraph::new(visible), snapshot.body_area);
    }
}

pub fn max_scroll(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
) -> usize {
    snapshot(area, tasks, target, theme, 0)
        .map(|snapshot| snapshot.max_scroll)
        .unwrap_or(0)
}

pub(crate) fn scrollbar_layout(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<FilePreviewScrollbarLayout> {
    snapshot(area, tasks, target, theme, scroll)?.layout
}
