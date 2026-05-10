use super::super::*;
use super::workspace_board_hit_test_tracks::*;
use ratatui::layout::{Position, Rect};
use zorai_protocol::WorkspaceTaskStatus;

#[test]
fn workspace_board_hit_test_tracks_extended_task_actions() {
    let state = workspace_with_assigned_task();
    let mut expanded = std::collections::HashSet::new();
    expanded.insert("workspace-task-1".to_string());
    let area = Rect::new(0, 0, 140, 24);
    let inner = block_inner(area);
    let columns = workspace_column_areas(board_area(inner));
    let projection = state.projection();
    let todo_tasks = &projection.columns[0].tasks;
    let todo_body = block_inner(columns[0]);
    let task_body = block_inner(task_card_rect(todo_body, todo_tasks, &expanded, 0));

    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x + 7, task_body.y + TASK_PRIMARY_ACTION_ROW)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::Pause,
        })
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x, task_body.y + TASK_DELETE_ACTION_ROW)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::Details,
        })
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x + 10, task_body.y + TASK_DELETE_ACTION_ROW)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::History,
        })
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x + 20, task_body.y + TASK_DELETE_ACTION_ROW)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::Edit,
        })
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x + 27, task_body.y + TASK_DELETE_ACTION_ROW)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::Delete,
        })
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x + 9, task_body.y + TASK_ASSIGN_ACTION_ROW)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::Reviewer,
        })
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x, task_body.y + TASK_DELETE_ACTION_ROW + 1)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::OpenRuntime,
        })
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(task_body.x + 8, task_body.y + TASK_DELETE_ACTION_ROW + 1)
        ),
        Some(WorkspaceBoardHitTarget::Action {
            task_id: "workspace-task-1".to_string(),
            status: WorkspaceTaskStatus::Todo,
            action: WorkspaceBoardAction::ToggleActions,
        })
    );
}

#[test]
fn workspace_board_hit_test_tracks_toolbar_actions() {
    let state = workspace_with_task();
    let expanded = std::collections::HashSet::new();
    let area = Rect::new(0, 0, 100, 20);
    let inner = block_inner(area);

    assert_eq!(
        hit_test(area, &state, &expanded, Position::new(inner.x, inner.y)),
        Some(WorkspaceBoardHitTarget::Toolbar(
            WorkspaceBoardToolbarAction::NewTask
        ))
    );
    assert_eq!(
        hit_test(
            area,
            &state,
            &expanded,
            Position::new(inner.x + 13, inner.y)
        ),
        Some(WorkspaceBoardHitTarget::Toolbar(
            WorkspaceBoardToolbarAction::ToggleOperator
        ))
    );
}

#[test]
fn workspace_toolbar_has_single_operator_switcher_label() {
    let label = toolbar_label(zorai_protocol::WorkspaceOperator::User);

    assert_eq!(label, "[New task] [operator: User]");
    assert!(!label.contains("[Refresh]"));
    assert!(!label.contains("[Auto]"));
    assert!(!label.contains("[User]"));
    assert!(!label.contains("  operator:"));
}
