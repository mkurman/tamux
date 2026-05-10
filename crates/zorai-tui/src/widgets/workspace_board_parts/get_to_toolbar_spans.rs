use super::*;
use crate::state::workspace::WorkspaceState;
use crate::theme::ThemeTokens;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::HashSet;
use zorai_protocol::WorkspaceTaskStatus;

pub(crate) const TASK_COLLAPSED_ROW_HEIGHT: u16 = 7;
pub(crate) const TASK_EXPANDED_ROW_HEIGHT: u16 = 11;
pub(crate) const TASK_PRIMARY_ACTION_ROW: u16 = 4;
pub(crate) const TASK_SECONDARY_ACTION_ROW: u16 = 5;
pub(crate) const TASK_ASSIGN_ACTION_ROW: u16 = 6;
pub(crate) const TASK_DELETE_ACTION_ROW: u16 = 7;
pub(crate) const TASK_TITLE_MAX_LINES: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceBoardToolbarAction {
    NewTask,
    ToggleOperator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceBoardAction {
    ToggleActions,
    Run,
    RunBlocked,
    Pause,
    Stop,
    MoveNext,
    Review,
    Assign,
    Reviewer,
    Details,
    OpenRuntime,
    History,
    Edit,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceBoardHitTarget {
    Column {
        status: zorai_protocol::WorkspaceTaskStatus,
    },
    Task {
        task_id: String,
        status: zorai_protocol::WorkspaceTaskStatus,
    },
    Action {
        task_id: String,
        status: zorai_protocol::WorkspaceTaskStatus,
        action: WorkspaceBoardAction,
    },
    Toolbar(WorkspaceBoardToolbarAction),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceBoardScroll {
    todo: usize,
    in_progress: usize,
    in_review: usize,
    done: usize,
}

impl WorkspaceBoardScroll {
    pub fn get(&self, status: &WorkspaceTaskStatus) -> usize {
        *self.slot(status)
    }

    pub fn set(&mut self, status: &WorkspaceTaskStatus, value: usize) {
        *self.slot_mut(status) = value;
    }

    fn slot(&self, status: &WorkspaceTaskStatus) -> &usize {
        match status {
            WorkspaceTaskStatus::Todo => &self.todo,
            WorkspaceTaskStatus::InProgress => &self.in_progress,
            WorkspaceTaskStatus::InReview => &self.in_review,
            WorkspaceTaskStatus::Done => &self.done,
        }
    }

    fn slot_mut(&mut self, status: &WorkspaceTaskStatus) -> &mut usize {
        match status {
            WorkspaceTaskStatus::Todo => &mut self.todo,
            WorkspaceTaskStatus::InProgress => &mut self.in_progress,
            WorkspaceTaskStatus::InReview => &mut self.in_review,
            WorkspaceTaskStatus::Done => &mut self.done,
        }
    }
}

pub(crate) fn render_with_scroll(
    frame: &mut Frame,
    area: Rect,
    workspace: &WorkspaceState,
    expanded_task_ids: &HashSet<String>,
    column_scrolls: &WorkspaceBoardScroll,
    selected: Option<&WorkspaceBoardHitTarget>,
    theme: &ThemeTokens,
    focused: bool,
) {
    let projection = workspace.projection();
    let title = format!(
        " Workspace {}{} ",
        projection.workspace_id,
        projection
            .filter_summary
            .as_ref()
            .map(|summary| format!(" · filter {summary}"))
            .unwrap_or_default()
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(if focused {
            theme.accent_primary
        } else {
            theme.fg_dim
        });
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width < 30 || inner.height < 4 {
        return;
    }

    render_toolbar(frame, inner, projection.operator.clone(), selected, theme);
    let board = board_area(inner);
    let columns = workspace_column_areas(board);

    for (index, column) in projection.columns.iter().enumerate() {
        let column_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ({}) ", column.title, column.tasks.len()))
            .border_style(theme.fg_dim);
        let column_inner = column_block.inner(columns[index]);
        frame.render_widget(column_block, columns[index]);
        render_column_tasks(
            frame,
            column_inner,
            column,
            &projection.notice_summaries,
            expanded_task_ids,
            column_scrolls.get(&column.status),
            selected,
            theme,
        );
    }
}

pub(crate) fn selectable_targets(
    workspace: &WorkspaceState,
    expanded_task_ids: &HashSet<String>,
) -> Vec<WorkspaceBoardHitTarget> {
    let mut targets = vec![
        WorkspaceBoardHitTarget::Toolbar(WorkspaceBoardToolbarAction::NewTask),
        WorkspaceBoardHitTarget::Toolbar(WorkspaceBoardToolbarAction::ToggleOperator),
    ];
    for column in &workspace.projection().columns {
        for task in &column.tasks {
            targets.push(WorkspaceBoardHitTarget::Task {
                task_id: task.id.clone(),
                status: column.status.clone(),
            });
            targets.push(WorkspaceBoardHitTarget::Action {
                task_id: task.id.clone(),
                status: column.status.clone(),
                action: WorkspaceBoardAction::OpenRuntime,
            });
            targets.push(WorkspaceBoardHitTarget::Action {
                task_id: task.id.clone(),
                status: column.status.clone(),
                action: WorkspaceBoardAction::ToggleActions,
            });
            if !expanded_task_ids.contains(&task.id) {
                continue;
            }
            let run_action = if task.assignee.is_some() {
                WorkspaceBoardAction::Run
            } else {
                WorkspaceBoardAction::RunBlocked
            };
            for action in [
                run_action,
                WorkspaceBoardAction::Pause,
                WorkspaceBoardAction::Stop,
                WorkspaceBoardAction::MoveNext,
                WorkspaceBoardAction::Review,
                WorkspaceBoardAction::Assign,
                WorkspaceBoardAction::Reviewer,
                WorkspaceBoardAction::Details,
                WorkspaceBoardAction::History,
                WorkspaceBoardAction::Edit,
                WorkspaceBoardAction::Delete,
            ] {
                targets.push(WorkspaceBoardHitTarget::Action {
                    task_id: task.id.clone(),
                    status: column.status.clone(),
                    action,
                });
            }
        }
    }
    targets
}

pub(crate) fn step_selection(
    workspace: &WorkspaceState,
    expanded_task_ids: &HashSet<String>,
    current: Option<&WorkspaceBoardHitTarget>,
    delta: i32,
) -> Option<WorkspaceBoardHitTarget> {
    let targets = selectable_targets(workspace, expanded_task_ids);
    if targets.is_empty() {
        return None;
    }
    let current_index = current
        .and_then(|selected| targets.iter().position(|target| target == selected))
        .unwrap_or(0);
    let last = targets.len().saturating_sub(1) as i32;
    let next = (current_index as i32 + delta).clamp(0, last) as usize;
    targets.get(next).cloned()
}

pub(crate) fn hit_test_with_scroll(
    area: Rect,
    workspace: &WorkspaceState,
    expanded_task_ids: &HashSet<String>,
    column_scrolls: &WorkspaceBoardScroll,
    position: Position,
) -> Option<WorkspaceBoardHitTarget> {
    let inner = block_inner(area);
    if !contains(inner, position) || inner.width < 30 || inner.height < 4 {
        return None;
    }
    if position.y == inner.y {
        return toolbar_action_at_x(inner.x, position.x).map(WorkspaceBoardHitTarget::Toolbar);
    }
    let board = board_area(inner);
    let columns = workspace_column_areas(board);
    for (index, rect) in columns.iter().enumerate() {
        if !contains(*rect, position) {
            continue;
        }
        let column = workspace.projection().columns.get(index)?;
        let body = block_inner(*rect);
        if !contains(body, position) {
            return Some(WorkspaceBoardHitTarget::Column {
                status: column.status.clone(),
            });
        }
        let Some((task_index, card_rect)) = task_card_at_position_with_scroll(
            body,
            &column.tasks,
            expanded_task_ids,
            column_scrolls.get(&column.status),
            position,
        ) else {
            return Some(WorkspaceBoardHitTarget::Column {
                status: column.status.clone(),
            });
        };
        let Some(task) = column.tasks.get(task_index) else {
            return Some(WorkspaceBoardHitTarget::Column {
                status: column.status.clone(),
            });
        };
        let card_body = block_inner(card_rect);
        let rows = task_action_rows(task, card_body.width);
        if !contains(card_body, position) {
            return Some(WorkspaceBoardHitTarget::Task {
                task_id: task.id.clone(),
                status: column.status.clone(),
            });
        }
        let row_in_task = position.y.saturating_sub(card_body.y);
        let actions_expanded = expanded_task_ids.contains(&task.id);
        if !actions_expanded && row_in_task == rows.primary {
            if let Some(action) = collapsed_controls_action_at_x(card_body.x, position.x) {
                return Some(WorkspaceBoardHitTarget::Action {
                    task_id: task.id.clone(),
                    status: column.status.clone(),
                    action,
                });
            }
        }
        if actions_expanded && rows.action_rows().contains(&row_in_task) {
            if let Some(action) = action_at_x(
                rows.base_row(row_in_task),
                card_body.x,
                position.x,
                task.assignee.is_none(),
            ) {
                return Some(WorkspaceBoardHitTarget::Action {
                    task_id: task.id.clone(),
                    status: column.status.clone(),
                    action,
                });
            }
        }
        if actions_expanded && row_in_task == rows.footer {
            if let Some(action) = expanded_footer_action_at_x(card_body.x, position.x) {
                return Some(WorkspaceBoardHitTarget::Action {
                    task_id: task.id.clone(),
                    status: column.status.clone(),
                    action,
                });
            }
        }
        return Some(WorkspaceBoardHitTarget::Task {
            task_id: task.id.clone(),
            status: column.status.clone(),
        });
    }
    None
}

pub(crate) fn column_status_at_position(
    area: Rect,
    workspace: &WorkspaceState,
    position: Position,
) -> Option<WorkspaceTaskStatus> {
    let inner = block_inner(area);
    if !contains(inner, position) || inner.width < 30 || inner.height < 4 {
        return None;
    }
    let board = board_area(inner);
    let columns = workspace_column_areas(board);
    for (index, rect) in columns.iter().enumerate() {
        if contains(*rect, position) {
            return workspace
                .projection()
                .columns
                .get(index)
                .map(|column| column.status.clone());
        }
    }
    None
}

pub(crate) fn scroll_for_target(
    area: Rect,
    workspace: &WorkspaceState,
    expanded_task_ids: &HashSet<String>,
    column_scrolls: &WorkspaceBoardScroll,
    target: &WorkspaceBoardHitTarget,
) -> WorkspaceBoardScroll {
    let mut next = column_scrolls.clone();
    let Some((status, task_id)) = target_task(target) else {
        return next;
    };
    let inner = block_inner(area);
    if inner.width < 30 || inner.height < 4 {
        return next;
    }
    let board = board_area(inner);
    let columns = workspace_column_areas(board);
    let projection = workspace.projection();
    let Some((column_index, column)) = projection
        .columns
        .iter()
        .enumerate()
        .find(|(_, column)| column.status == status)
    else {
        return next;
    };
    let Some(task_index) = column.tasks.iter().position(|task| task.id == task_id) else {
        return next;
    };
    let body = block_inner(columns[column_index]);
    let current = next.get(&status).min(column.tasks.len().saturating_sub(1));
    let desired =
        scroll_offset_for_task(body, &column.tasks, expanded_task_ids, current, task_index);
    next.set(&status, desired);
    next
}

pub(crate) fn stepped_scroll_for_status(
    workspace: &WorkspaceState,
    current: &WorkspaceBoardScroll,
    status: &WorkspaceTaskStatus,
    delta: i32,
) -> WorkspaceBoardScroll {
    let mut next = current.clone();
    let max_scroll = workspace
        .projection()
        .columns
        .iter()
        .find(|column| column.status == *status)
        .map(|column| column.tasks.len().saturating_sub(1))
        .unwrap_or(0);
    let current = next.get(status) as i32;
    let value = (current + delta).clamp(0, max_scroll as i32) as usize;
    next.set(status, value);
    next
}

pub(crate) fn target_task(target: &WorkspaceBoardHitTarget) -> Option<(WorkspaceTaskStatus, &str)> {
    match target {
        WorkspaceBoardHitTarget::Task { task_id, status }
        | WorkspaceBoardHitTarget::Action {
            task_id, status, ..
        } => Some((status.clone(), task_id.as_str())),
        WorkspaceBoardHitTarget::Column { .. } | WorkspaceBoardHitTarget::Toolbar(_) => None,
    }
}

pub(crate) fn render_toolbar(
    frame: &mut Frame,
    area: Rect,
    operator: zorai_protocol::WorkspaceOperator,
    selected: Option<&WorkspaceBoardHitTarget>,
    theme: &ThemeTokens,
) {
    let label = toolbar_label(operator.clone());
    let spans = toolbar_spans(operator, selected, theme);
    frame.render_widget(
        Paragraph::new(Line::from(spans)).wrap(Wrap { trim: false }),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let _ = label;
}

pub(crate) fn toolbar_label(operator: zorai_protocol::WorkspaceOperator) -> String {
    format!("[New task] [operator: {operator:?}]")
}

pub(crate) fn toolbar_spans(
    operator: zorai_protocol::WorkspaceOperator,
    selected: Option<&WorkspaceBoardHitTarget>,
    theme: &ThemeTokens,
) -> Vec<Span<'static>> {
    [
        (
            WorkspaceBoardToolbarAction::NewTask,
            "[New task]".to_string(),
        ),
        (
            WorkspaceBoardToolbarAction::ToggleOperator,
            format!("[operator: {operator:?}]"),
        ),
    ]
    .into_iter()
    .enumerate()
    .flat_map(|(index, (action, label))| {
        let style = if selected == Some(&WorkspaceBoardHitTarget::Toolbar(action.clone())) {
            theme.accent_primary
        } else {
            theme.fg_dim
        };
        let mut spans = vec![Span::styled(label, style)];
        if index < 2 {
            spans.push(Span::raw(" "));
        }
        spans
    })
    .collect()
}
