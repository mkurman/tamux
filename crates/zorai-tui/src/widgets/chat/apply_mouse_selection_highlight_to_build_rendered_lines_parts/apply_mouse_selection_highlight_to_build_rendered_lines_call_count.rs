use super::super::build_rendered_lines_to_build_visible_window_from_snapshot_to_apply::*;
use super::super::render_streaming_markdown_to_message_block_style_to_message_action::*;
use super::super::resolved_scroll_to_highlight_line_range_to_selected_text_to_selection::*;
use super::super::*;
use crate::state::chat::ChatState;
use crate::theme::ThemeTokens;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
pub(crate) fn apply_mouse_selection_highlight(
    _snapshot: &SelectionSnapshot,
    visible_lines: &mut [RenderedChatLine],
    padding: usize,
    start_idx: usize,
    end_idx: usize,
    mouse_selection: Option<(SelectionPoint, SelectionPoint)>,
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
    let highlight = Style::default().bg(Color::Indexed(31));
    let visible_last = end_idx.saturating_sub(1);
    let range_start = start_point.row.max(start_idx);
    let range_end = end_point.row.min(visible_last);

    if range_start > range_end {
        return;
    }

    for absolute_row in range_start..=range_end {
        let visible_row = padding + absolute_row.saturating_sub(start_idx);
        if let Some(line) = visible_lines.get_mut(visible_row) {
            let rendered = RenderedChatLine {
                line: line.line.clone(),
                message_index: line.message_index,
                kind: line.kind,
            };
            let (_, content_start, content_end) = rendered_line_content_bounds(&rendered);
            let content_width = content_end.saturating_sub(content_start);
            let from = if absolute_row == start_point.row {
                content_start.saturating_add(start_point.col.min(content_width))
            } else {
                content_start
            };
            let to = if absolute_row == end_point.row {
                content_start
                    .saturating_add(end_point.col.min(content_width))
                    .max(from)
            } else {
                content_end
            };
            highlight_line_range(&mut line.line, from, to, highlight);
        }
    }
}

pub(crate) fn render_snapshot(
    frame: &mut Frame,
    snapshot: &SelectionSnapshot,
    chat: &ChatState,
    theme: &ThemeTokens,
    mouse_selection: Option<(SelectionPoint, SelectionPoint)>,
) {
    let (mut visible_lines, padding, start_idx, end_idx) =
        build_visible_window_from_snapshot(snapshot, chat);
    apply_selected_message_highlight(&mut visible_lines, chat.selected_message());
    apply_mouse_selection_highlight(
        snapshot,
        &mut visible_lines,
        padding,
        start_idx,
        end_idx,
        mouse_selection,
    );

    let visible_lines = visible_lines
        .into_iter()
        .map(|line| line.line)
        .collect::<Vec<_>>();
    let scroll = resolved_scroll(
        chat,
        snapshot.total_lines,
        snapshot.inner.height as usize,
        &snapshot.message_line_ranges,
    );
    if let Some(layout) =
        scrollbar_layout_from_metrics(snapshot.inner, snapshot.total_lines, scroll)
    {
        frame.render_widget(Paragraph::new(visible_lines), layout.content);

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
        frame.render_widget(Paragraph::new(visible_lines), snapshot.inner);
    }
}

pub fn render_cached(
    frame: &mut Frame,
    _area: Rect,
    chat: &ChatState,
    theme: &ThemeTokens,
    snapshot: &CachedSelectionSnapshot,
    mouse_selection: Option<(SelectionPoint, SelectionPoint)>,
) {
    render_snapshot(frame, &snapshot.0, chat, theme, mouse_selection);
}

#[cfg(test)]
pub fn reset_build_rendered_lines_call_count() {
    BUILD_RENDERED_LINES_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
pub fn build_rendered_lines_call_count() -> usize {
    BUILD_RENDERED_LINES_CALLS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub fn reset_build_transcript_metrics_call_count() {
    BUILD_TRANSCRIPT_METRICS_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
pub fn build_transcript_metrics_call_count() -> usize {
    BUILD_TRANSCRIPT_METRICS_CALLS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn reset_assistant_responder_labels_call_count() {
    ASSISTANT_RESPONDER_LABELS_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
pub(crate) fn assistant_responder_labels_call_count() -> usize {
    ASSISTANT_RESPONDER_LABELS_CALLS.with(std::cell::Cell::get)
}
