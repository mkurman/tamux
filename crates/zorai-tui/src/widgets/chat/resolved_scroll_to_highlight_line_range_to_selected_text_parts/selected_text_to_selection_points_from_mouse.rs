use super::super::render_streaming_markdown_to_message_block_style_to_message_action::*;
use super::super::resolved_scroll_to_highlight_line_range_to_selected_text_to_selection::*;
use super::super::selection_point_from_snapshot_to_render::*;
use super::super::*;
use crate::state::chat::ChatState;
use crate::theme::ThemeTokens;
#[cfg(test)]
pub(crate) fn selected_text(
    area: Rect,
    chat: &ChatState,
    theme: &ThemeTokens,
    current_tick: u64,
    start: SelectionPoint,
    end: SelectionPoint,
) -> Option<String> {
    let snapshot = selection_snapshot(area, chat, theme, current_tick, false)?;
    if snapshot.total_lines == 0 {
        return None;
    }

    let (start_point, end_point) =
        if start.row <= end.row || (start.row == end.row && start.col <= end.col) {
            (start, end)
        } else {
            (end, start)
        };
    let start_row = start_point.row.min(snapshot.total_lines.saturating_sub(1));
    let end_row = end_point.row.min(snapshot.total_lines.saturating_sub(1));
    let start_col = start_point.col;
    let end_col = end_point.col;

    if start_row == end_row && start_col == end_col {
        return None;
    }

    let mut lines = Vec::new();

    for row in start_row..=end_row {
        let rendered = snapshot_line_at(&snapshot, row)?;
        let (plain, content_start, content_end) = rendered_line_content_bounds(rendered);
        let content_width = content_end.saturating_sub(content_start);
        let from = if row == start_row {
            start_col.min(content_width)
        } else {
            0
        };
        let to = if row == end_row {
            end_col.min(content_width).max(from)
        } else {
            content_width
        };

        lines.push(display_slice(
            &plain,
            content_start.saturating_add(from),
            content_start.saturating_add(to),
        ));
    }

    let text = lines.join("\n");
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
pub(crate) fn selection_point_from_mouse(
    area: Rect,
    chat: &ChatState,
    theme: &ThemeTokens,
    current_tick: u64,
    mouse: Position,
) -> Option<SelectionPoint> {
    let snapshot = selection_snapshot(area, chat, theme, current_tick, false)?;
    selection_point_from_snapshot(&snapshot, mouse)
}

pub(crate) fn selection_points_from_mouse(
    area: Rect,
    chat: &ChatState,
    theme: &ThemeTokens,
    current_tick: u64,
    start: Position,
    end: Position,
    retry_wait_start_selected: bool,
) -> Option<(SelectionPoint, SelectionPoint)> {
    let snapshot = selection_snapshot(area, chat, theme, current_tick, retry_wait_start_selected)?;
    Some((
        selection_point_from_snapshot(&snapshot, start)?,
        selection_point_from_snapshot(&snapshot, end)?,
    ))
}
