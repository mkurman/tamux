use super::*;
use ratatui::prelude::*;
pub(crate) fn contains(area: Rect, position: Position) -> bool {
    position.x >= area.x
        && position.x < area.x.saturating_add(area.width)
        && position.y >= area.y
        && position.y < area.y.saturating_add(area.height)
}

pub(crate) fn action_at_x(
    row: u16,
    body_x: u16,
    position_x: u16,
    run_blocked: bool,
) -> Option<WorkspaceBoardAction> {
    let x = position_x.saturating_sub(body_x);
    match row {
        TASK_PRIMARY_ACTION_ROW if run_blocked => match x {
            0..=8 => Some(WorkspaceBoardAction::RunBlocked),
            10..=16 => Some(WorkspaceBoardAction::Pause),
            18..=23 => Some(WorkspaceBoardAction::Stop),
            _ => None,
        },
        TASK_PRIMARY_ACTION_ROW => match x {
            0..=4 => Some(WorkspaceBoardAction::Run),
            6..=12 => Some(WorkspaceBoardAction::Pause),
            14..=19 => Some(WorkspaceBoardAction::Stop),
            _ => None,
        },
        TASK_SECONDARY_ACTION_ROW => match x {
            0..=5 => Some(WorkspaceBoardAction::MoveNext),
            7..=14 => Some(WorkspaceBoardAction::Review),
            _ => None,
        },
        TASK_ASSIGN_ACTION_ROW => match x {
            0..=7 => Some(WorkspaceBoardAction::Assign),
            9..=18 => Some(WorkspaceBoardAction::Reviewer),
            _ => None,
        },
        TASK_DELETE_ACTION_ROW => match x {
            0..=8 => Some(WorkspaceBoardAction::Details),
            10..=18 => Some(WorkspaceBoardAction::History),
            20..=25 => Some(WorkspaceBoardAction::Edit),
            27..=34 => Some(WorkspaceBoardAction::Delete),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn collapsed_controls_action_at_x(
    body_x: u16,
    position_x: u16,
) -> Option<WorkspaceBoardAction> {
    let x = position_x.saturating_sub(body_x);
    match x {
        0..=5 => Some(WorkspaceBoardAction::OpenRuntime),
        7..=15 => Some(WorkspaceBoardAction::ToggleActions),
        _ => None,
    }
}

pub(crate) fn expanded_footer_action_at_x(
    body_x: u16,
    position_x: u16,
) -> Option<WorkspaceBoardAction> {
    let x = position_x.saturating_sub(body_x);
    match x {
        0..=5 => Some(WorkspaceBoardAction::OpenRuntime),
        7..=20 => Some(WorkspaceBoardAction::ToggleActions),
        _ => None,
    }
}

pub(crate) fn toolbar_action_at_x(
    body_x: u16,
    position_x: u16,
) -> Option<WorkspaceBoardToolbarAction> {
    let x = position_x.saturating_sub(body_x);
    match x {
        0..=9 => Some(WorkspaceBoardToolbarAction::NewTask),
        11..=27 => Some(WorkspaceBoardToolbarAction::ToggleOperator),
        _ => None,
    }
}
