use super::build_rendered_lines_to_build_visible_window_from_snapshot_to_apply::*;
use super::render_streaming_markdown_to_message_block_style_to_message_action::*;
use super::resolved_scroll_to_highlight_line_range_to_selected_text_to_selection::*;
use super::*;
use crate::state::chat::{
    ChatHitTarget, ChatState, MessageRole, RetryPhase,
};
use crate::theme::ThemeTokens;
use unicode_width::UnicodeWidthStr;
pub(crate) fn selection_point_from_snapshot(
    snapshot: &SelectionSnapshot,
    mouse: Position,
) -> Option<SelectionPoint> {
    let inner = snapshot.inner;

    let clamped_x = mouse.x.clamp(
        inner.x,
        inner.x.saturating_add(inner.width).saturating_sub(1),
    );
    let clamped_y = mouse.y.clamp(
        inner.y,
        inner.y.saturating_add(inner.height).saturating_sub(1),
    );
    let rel_row = clamped_y.saturating_sub(inner.y) as usize;
    let rel_col = clamped_x.saturating_sub(inner.x) as usize;

    if rel_row < snapshot.padding {
        return None;
    }

    let row = {
        let visible_count = snapshot.end_idx.saturating_sub(snapshot.start_idx).max(1);
        snapshot.start_idx
            + rel_row
                .saturating_sub(snapshot.padding)
                .min(visible_count.saturating_sub(1))
    };
    let row = nearest_content_row(snapshot, row)?;
    let rendered = snapshot_line_at(snapshot, row)?;
    let (_, content_start, content_end) = rendered_line_content_bounds(rendered);
    let content_width = content_end.saturating_sub(content_start);
    let content_col = rel_col.saturating_sub(content_start).min(content_width);

    Some(SelectionPoint {
        row,
        col: content_col,
    })
}

pub(crate) fn toggle_button_hit(hit: &RenderedChatLine, inner: Rect, mouse: Position) -> bool {
    let content_col = mouse.x.saturating_sub(inner.x) as usize;
    let (_, content_start, _) = rendered_line_content_bounds(hit);
    content_col >= content_start
        && content_col < content_start.saturating_add(TOGGLE_BUTTON_HIT_WIDTH)
}

#[allow(dead_code)]
pub fn hit_test(
    area: Rect,
    chat: &ChatState,
    theme: &ThemeTokens,
    current_tick: u64,
    mouse: Position,
) -> Option<ChatHitTarget> {
    let snapshot = selection_snapshot(area, chat, theme, current_tick, false)?;
    hit_test_snapshot(&snapshot, chat, current_tick, mouse)
}

pub fn hit_test_from_cached_snapshot(
    snapshot: &CachedSelectionSnapshot,
    chat: &ChatState,
    current_tick: u64,
    mouse: Position,
) -> Option<ChatHitTarget> {
    hit_test_snapshot(&snapshot.0, chat, current_tick, mouse)
}

pub(crate) fn hit_test_snapshot(
    snapshot: &SelectionSnapshot,
    chat: &ChatState,
    current_tick: u64,
    mouse: Position,
) -> Option<ChatHitTarget> {
    let inner = snapshot.inner;

    if mouse.x < inner.x
        || mouse.y < inner.y
        || mouse.x >= inner.x.saturating_add(inner.width)
        || mouse.y >= inner.y.saturating_add(inner.height)
    {
        return None;
    }
    let rel_row = mouse.y.saturating_sub(inner.y) as usize;
    if rel_row < snapshot.padding {
        return None;
    }
    let visible_count = snapshot.end_idx.saturating_sub(snapshot.start_idx).max(1);
    let absolute_row = snapshot.start_idx
        + rel_row
            .saturating_sub(snapshot.padding)
            .min(visible_count.saturating_sub(1));
    let resolved_row = match snapshot_line_at(snapshot, absolute_row) {
        Some(line) if !matches!(line.kind, RenderedLineKind::Padding) => absolute_row,
        Some(_) => nearest_message_content_row(snapshot, absolute_row)?,
        None => return None,
    };
    let hit = snapshot_line_at(snapshot, resolved_row)?;

    if matches!(
        hit.kind,
        RenderedLineKind::ToolToggle
            | RenderedLineKind::MessageBody
            | RenderedLineKind::ImageAttachment
    ) {
        if let Some(message_index) = hit.message_index {
            let message = chat
                .active_thread()
                .and_then(|thread| thread.messages.get(message_index))?;
            if message.role == MessageRole::Tool {
                let content_col = mouse.x.saturating_sub(inner.x) as usize;
                let (_, content_start, _) = rendered_line_content_bounds(hit);
                let action_col = content_col.saturating_sub(content_start);
                if let Some(chip) = tool_file_chip(message) {
                    let line_text = rendered_line_plain_text(hit);
                    let chip_text = format!("[{}]", chip.label);
                    if let Some(chip_byte_start) = line_text.find(&chip_text) {
                        let chip_start = UnicodeWidthStr::width(&line_text[..chip_byte_start]);
                        let chip_width = UnicodeWidthStr::width(chip_text.as_str());
                        if action_col >= chip_start
                            && action_col < chip_start.saturating_add(chip_width)
                        {
                            return Some(ChatHitTarget::ToolFilePath { message_index });
                        }
                    }
                }
            } else if matches!(hit.kind, RenderedLineKind::ImageAttachment)
                && crate::widgets::chat::message_image_preview_path(message).is_some()
            {
                return Some(ChatHitTarget::MessageImage { message_index });
            }
        }
    }

    match hit.kind {
        RenderedLineKind::RetryAction => {
            let content_col = mouse.x.saturating_sub(inner.x) as usize;
            let (_, content_start, _) = rendered_line_content_bounds(hit);
            let action_col = content_col.saturating_sub(content_start);
            let status = chat.retry_status()?;
            match status.phase {
                RetryPhase::Retrying => {
                    let label_width = UnicodeWidthStr::width("[Stop]");
                    if action_col < label_width {
                        Some(ChatHitTarget::RetryStop)
                    } else {
                        None
                    }
                }
                RetryPhase::Waiting => {
                    let yes_label =
                        format!("[Yes {}s]", retry_wait_remaining_secs(status, current_tick));
                    let yes_width = UnicodeWidthStr::width(yes_label.as_str());
                    if action_col < yes_width {
                        Some(ChatHitTarget::RetryStartNow)
                    } else {
                        let no_start = yes_width.saturating_add(1);
                        let no_width = UnicodeWidthStr::width("[No]");
                        if action_col >= no_start && action_col < no_start.saturating_add(no_width)
                        {
                            Some(ChatHitTarget::RetryStop)
                        } else {
                            None
                        }
                    }
                }
            }
        }
        RenderedLineKind::ReasoningToggle => {
            let message_index = hit.message_index?;
            if toggle_button_hit(hit, inner, mouse) {
                Some(ChatHitTarget::ReasoningToggle(message_index))
            } else {
                Some(ChatHitTarget::Message(message_index))
            }
        }
        RenderedLineKind::ToolToggle => {
            let message_index = hit.message_index?;
            if toggle_button_hit(hit, inner, mouse) {
                Some(ChatHitTarget::ToolToggle(message_index))
            } else {
                Some(ChatHitTarget::Message(message_index))
            }
        }
        RenderedLineKind::ActionBar => {
            let message_index = hit.message_index?;
            let content_col = mouse.x.saturating_sub(inner.x) as usize;
            let (_, content_start, _) = rendered_line_content_bounds(hit);
            let message = chat
                .active_thread()
                .and_then(|thread| thread.messages.get(message_index))?;
            action_hit_target(
                chat,
                message_index,
                message,
                content_col.saturating_sub(content_start),
                current_tick,
            )
        }
        RenderedLineKind::MessageBody
        | RenderedLineKind::ImageAttachment
        | RenderedLineKind::ReasoningContent
        | RenderedLineKind::ToolDetail => Some(ChatHitTarget::Message(hit.message_index?)),
        RenderedLineKind::Padding | RenderedLineKind::Streaming | RenderedLineKind::RetryStatus => {
            None
        }
    }
}

pub fn render(
    frame: &mut Frame,
    area: Rect,
    chat: &ChatState,
    theme: &ThemeTokens,
    current_tick: u64,
    retry_wait_start_selected: bool,
    _focused: bool,
    mouse_selection: Option<(SelectionPoint, SelectionPoint)>,
) {
    let inner = content_inner(area);

    if chat.active_thread().is_none() && chat.streaming_content().is_empty() {
        crate::widgets::splash::render(frame, inner, theme);
        return;
    }
    let Some(snapshot) =
        selection_snapshot(area, chat, theme, current_tick, retry_wait_start_selected)
    else {
        return;
    };
    render_snapshot(frame, &snapshot, chat, theme, mouse_selection);
}
