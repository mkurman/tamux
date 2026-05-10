use super::*;
use crate::theme::ThemeTokens;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::HashSet;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
pub(crate) fn render_column_tasks(
    frame: &mut Frame,
    area: Rect,
    column: &crate::state::workspace::WorkspaceColumn,
    notice_summaries: &std::collections::HashMap<String, String>,
    expanded_task_ids: &HashSet<String>,
    scroll: usize,
    selected: Option<&WorkspaceBoardHitTarget>,
    theme: &ThemeTokens,
) {
    if column.tasks.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("No tasks", theme.fg_dim))),
            area,
        );
        return;
    }
    let scroll = scroll.min(column.tasks.len().saturating_sub(1));
    for (index, task) in column.tasks.iter().enumerate().skip(scroll) {
        let rect =
            task_card_rect_with_scroll(area, &column.tasks, expanded_task_ids, scroll, index);
        if rect.height < 3 || rect.y >= area.y.saturating_add(area.height) {
            continue;
        }
        let task_selected = selected.is_some_and(|target| match target {
            WorkspaceBoardHitTarget::Task { task_id, .. }
            | WorkspaceBoardHitTarget::Action { task_id, .. } => task_id == &task.id,
            WorkspaceBoardHitTarget::Column { .. } | WorkspaceBoardHitTarget::Toolbar(_) => false,
        });
        let card = Block::default()
            .borders(Borders::ALL)
            .border_style(task_card_border_style(
                task,
                notice_summaries.get(&task.id),
                task_selected,
                theme,
            ));
        frame.render_widget(
            Paragraph::new(task_lines_with_actions(
                task,
                notice_summaries,
                selected,
                theme,
                Some(block_inner(rect).width),
                expanded_task_ids.contains(&task.id),
            ))
            .block(card),
            rect,
        );
    }
}

pub(crate) fn task_card_border_style(
    task: &zorai_protocol::WorkspaceTask,
    notice_summary: Option<&String>,
    selected: bool,
    theme: &ThemeTokens,
) -> Style {
    if selected {
        return theme.accent_primary;
    }
    if task.status == zorai_protocol::WorkspaceTaskStatus::Done {
        return theme.accent_success;
    }
    if notice_summary.is_some_and(|summary| task_notice_is_failed(summary)) {
        return theme.accent_danger;
    }
    theme.fg_dim
}

pub(crate) fn task_notice_is_failed(summary: &str) -> bool {
    summary.starts_with("runtime_failed:")
        || summary.starts_with("run_failed:")
        || summary.starts_with("review_failed:")
}

#[cfg(test)]
pub(crate) fn task_lines(
    task: &zorai_protocol::WorkspaceTask,
    notice_summaries: &std::collections::HashMap<String, String>,
    selected: Option<&WorkspaceBoardHitTarget>,
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    task_lines_with_actions(task, notice_summaries, selected, theme, None, false)
}

pub(crate) fn task_lines_with_actions(
    task: &zorai_protocol::WorkspaceTask,
    notice_summaries: &std::collections::HashMap<String, String>,
    selected: Option<&WorkspaceBoardHitTarget>,
    theme: &ThemeTokens,
    card_body_width: Option<u16>,
    expanded: bool,
) -> Vec<Line<'static>> {
    let assignee = task
        .assignee
        .as_ref()
        .map(|actor| format!("{actor:?}"))
        .unwrap_or_else(|| "unassigned".to_string());
    let id_line = notice_summaries
        .get(&task.id)
        .map(|notice| format!("id {} · {notice}", short_task_id(&task.id)))
        .unwrap_or_else(|| format!("id {}", short_task_id(&task.id)));
    let controls_line = action_line(
        task,
        selected,
        theme,
        &[
            (WorkspaceBoardAction::OpenRuntime, "[Open]"),
            (
                WorkspaceBoardAction::ToggleActions,
                if expanded {
                    "[Hide actions]"
                } else {
                    "[Actions]"
                },
            ),
        ],
    );
    let run_line = if task.assignee.is_some() {
        action_line(
            task,
            selected,
            theme,
            &[
                (WorkspaceBoardAction::Run, "[Run]"),
                (WorkspaceBoardAction::Pause, "[Pause]"),
                (WorkspaceBoardAction::Stop, "[Stop]"),
            ],
        )
    } else {
        action_line(
            task,
            selected,
            theme,
            &[
                (WorkspaceBoardAction::RunBlocked, "[Blocked]"),
                (WorkspaceBoardAction::Pause, "[Pause]"),
                (WorkspaceBoardAction::Stop, "[Stop]"),
            ],
        )
    };
    let mut lines = title_lines(&task.title, card_body_width, theme);
    lines.extend([
        Line::from(Span::styled(
            format!("{:?} · {:?}", task.task_type, task.priority),
            theme.fg_dim,
        )),
        Line::from(Span::styled(id_line, theme.fg_dim)),
        Line::from(Span::styled(format!("→ {assignee}"), theme.fg_dim)),
    ]);
    if !expanded {
        lines.push(controls_line);
        return lines;
    }
    lines.extend([
        run_line,
        action_line(
            task,
            selected,
            theme,
            &[
                (WorkspaceBoardAction::MoveNext, "[Move]"),
                (WorkspaceBoardAction::Review, "[Review]"),
            ],
        ),
        action_line(
            task,
            selected,
            theme,
            &[
                (WorkspaceBoardAction::Assign, "[Assign]"),
                (WorkspaceBoardAction::Reviewer, "[Reviewer]"),
            ],
        ),
        action_line(
            task,
            selected,
            theme,
            &[
                (WorkspaceBoardAction::Details, "[Details]"),
                (WorkspaceBoardAction::History, "[History]"),
                (WorkspaceBoardAction::Edit, "[Edit]"),
                (WorkspaceBoardAction::Delete, "[Delete]"),
            ],
        ),
        controls_line,
    ]);
    lines
}

pub(crate) fn action_line(
    task: &zorai_protocol::WorkspaceTask,
    selected: Option<&WorkspaceBoardHitTarget>,
    theme: &ThemeTokens,
    actions: &[(WorkspaceBoardAction, &'static str)],
) -> Line<'static> {
    let spans = actions
        .iter()
        .enumerate()
        .flat_map(|(index, (action, label))| {
            let is_selected = selected.is_some_and(|target| {
                target
                    == &WorkspaceBoardHitTarget::Action {
                        task_id: task.id.clone(),
                        status: task.status.clone(),
                        action: action.clone(),
                    }
            });
            let style = if is_selected {
                theme.accent_primary
            } else {
                theme.fg_dim
            };
            let mut spans = vec![Span::styled(*label, style)];
            if index + 1 < actions.len() {
                spans.push(Span::raw(" "));
            }
            spans
        })
        .collect::<Vec<_>>();
    Line::from(spans)
}

pub(crate) fn title_lines(
    title: &str,
    card_body_width: Option<u16>,
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    wrapped_title_text(title, card_body_width)
        .into_iter()
        .map(|line| Line::from(Span::styled(line, theme.fg_active)))
        .collect()
}

pub(crate) fn wrapped_title_text(title: &str, card_body_width: Option<u16>) -> Vec<String> {
    let Some(width) = card_body_width.map(usize::from).filter(|width| *width > 0) else {
        return vec![title.to_string()];
    };
    wrap_text_to_width(title, width, TASK_TITLE_MAX_LINES)
}

pub(crate) fn title_row_count(title: &str, card_body_width: u16) -> u16 {
    wrapped_title_text(title, Some(card_body_width))
        .len()
        .max(1) as u16
}

pub(crate) fn wrap_text_to_width(text: &str, width: usize, max_lines: usize) -> Vec<String> {
    if max_lines == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if lines.len() >= max_lines {
            break;
        }
        if UnicodeWidthStr::width(word) > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
                if lines.len() >= max_lines {
                    break;
                }
            }
            for chunk in split_word_to_width(word, width) {
                lines.push(chunk);
                if lines.len() >= max_lines {
                    break;
                }
            }
            continue;
        }
        let next_width = if current.is_empty() {
            UnicodeWidthStr::width(word)
        } else {
            UnicodeWidthStr::width(current.as_str()) + 1 + UnicodeWidthStr::width(word)
        };
        if next_width <= width {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
            continue;
        }
        lines.push(std::mem::take(&mut current));
        if lines.len() >= max_lines {
            break;
        }
        current.push_str(word);
    }
    if lines.len() < max_lines && !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

pub(crate) fn split_word_to_width(word: &str, width: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;
    for ch in word.chars() {
        let char_width = ch.width().unwrap_or(0);
        if current_width > 0 && current_width + char_width > width {
            chunks.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(ch);
        current_width += char_width;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TaskActionRows {
    pub(crate) primary: u16,
    pub(crate) secondary: u16,
    pub(crate) assign: u16,
    pub(crate) delete: u16,
    pub(crate) footer: u16,
}

impl TaskActionRows {
    pub(crate) fn action_rows(self) -> [u16; 4] {
        [self.primary, self.secondary, self.assign, self.delete]
    }

    pub(crate) fn base_row(self, row: u16) -> u16 {
        row.saturating_sub(self.primary.saturating_sub(TASK_PRIMARY_ACTION_ROW))
    }
}

pub(crate) fn task_action_rows(
    task: &zorai_protocol::WorkspaceTask,
    card_body_width: u16,
) -> TaskActionRows {
    let offset = title_row_count(&task.title, card_body_width).saturating_sub(1);
    TaskActionRows {
        primary: TASK_PRIMARY_ACTION_ROW.saturating_add(offset),
        secondary: TASK_SECONDARY_ACTION_ROW.saturating_add(offset),
        assign: TASK_ASSIGN_ACTION_ROW.saturating_add(offset),
        delete: TASK_DELETE_ACTION_ROW.saturating_add(offset),
        footer: TASK_DELETE_ACTION_ROW
            .saturating_add(1)
            .saturating_add(offset),
    }
}

pub(crate) fn short_task_id(task_id: &str) -> String {
    task_id.chars().take(12).collect()
}

pub(crate) fn workspace_column_areas(inner: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner)
}

pub(crate) fn task_card_height(
    task: &zorai_protocol::WorkspaceTask,
    card_body_width: u16,
    expanded_task_ids: &HashSet<String>,
) -> u16 {
    let title_extra_rows = title_row_count(&task.title, card_body_width).saturating_sub(1);
    if expanded_task_ids.contains(&task.id) {
        TASK_EXPANDED_ROW_HEIGHT.saturating_add(title_extra_rows)
    } else {
        TASK_COLLAPSED_ROW_HEIGHT.saturating_add(title_extra_rows)
    }
}

pub(crate) fn task_card_rect_with_scroll(
    body: Rect,
    tasks: &[zorai_protocol::WorkspaceTask],
    expanded_task_ids: &HashSet<String>,
    scroll: usize,
    index: usize,
) -> Rect {
    if index < scroll {
        return Rect::new(body.x, body.y, body.width, 0);
    }
    let card_body_width = body.width.saturating_sub(2);
    let offset = tasks
        .iter()
        .skip(scroll)
        .take(index.saturating_sub(scroll))
        .fold(0u16, |offset, task| {
            offset.saturating_add(task_card_height(task, card_body_width, expanded_task_ids))
        });
    let y = body.y.saturating_add(offset);
    let height = tasks
        .get(index)
        .map(|task| task_card_height(task, card_body_width, expanded_task_ids))
        .unwrap_or(TASK_COLLAPSED_ROW_HEIGHT);
    Rect::new(
        body.x,
        y,
        body.width,
        height.min(body.y.saturating_add(body.height).saturating_sub(y)),
    )
}

pub(crate) fn scroll_offset_for_task(
    body: Rect,
    tasks: &[zorai_protocol::WorkspaceTask],
    expanded_task_ids: &HashSet<String>,
    current: usize,
    task_index: usize,
) -> usize {
    let max_scroll = tasks.len().saturating_sub(1);
    let mut scroll = current.min(max_scroll);
    if task_index < scroll {
        return task_index;
    }
    while scroll < task_index {
        let offset = task_offset_from_scroll(body, tasks, expanded_task_ids, scroll, task_index);
        let height = tasks
            .get(task_index)
            .map(|task| task_card_height(task, body.width.saturating_sub(2), expanded_task_ids))
            .unwrap_or(TASK_COLLAPSED_ROW_HEIGHT);
        if offset.saturating_add(height) <= body.height {
            break;
        }
        scroll += 1;
    }
    scroll.min(max_scroll)
}

pub(crate) fn task_offset_from_scroll(
    body: Rect,
    tasks: &[zorai_protocol::WorkspaceTask],
    expanded_task_ids: &HashSet<String>,
    scroll: usize,
    index: usize,
) -> u16 {
    let card_body_width = body.width.saturating_sub(2);
    tasks
        .iter()
        .skip(scroll)
        .take(index.saturating_sub(scroll))
        .fold(0u16, |offset, task| {
            offset.saturating_add(task_card_height(task, card_body_width, expanded_task_ids))
        })
}

pub(crate) fn task_card_at_position_with_scroll(
    body: Rect,
    tasks: &[zorai_protocol::WorkspaceTask],
    expanded_task_ids: &HashSet<String>,
    scroll: usize,
    position: Position,
) -> Option<(usize, Rect)> {
    if !contains(body, position) {
        return None;
    }
    let scroll = scroll.min(tasks.len().saturating_sub(1));
    for index in scroll..tasks.len() {
        let rect = task_card_rect_with_scroll(body, tasks, expanded_task_ids, scroll, index);
        if contains(rect, position) {
            return Some((index, rect));
        }
        if rect.y >= body.y.saturating_add(body.height) {
            break;
        }
    }
    None
}

pub(crate) fn board_area(inner: Rect) -> Rect {
    Rect::new(
        inner.x,
        inner.y.saturating_add(1),
        inner.width,
        inner.height.saturating_sub(1),
    )
}

pub(crate) fn block_inner(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(1),
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    )
}
