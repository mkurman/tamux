use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
#[test]
fn selected_goal_step_r_opens_retry_confirmation() {
    let (mut model, _daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_with_steps(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Failed,
            vec![
                task::GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    order: 0,
                    status: Some(task::GoalRunStatus::Completed),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-2".to_string(),
                    title: "Deploy".to_string(),
                    order: 1,
                    status: Some(task::GoalRunStatus::Failed),
                    ..Default::default()
                },
            ],
        ),
    ));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-2".to_string()),
    });

    let handled = model.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Retry step 2 \"Deploy\" in goal \"Goal One\"?")
    );
}

#[test]
fn selected_goal_step_ctrl_r_opens_rerun_confirmation() {
    let (mut model, _daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_with_steps(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Failed,
            vec![
                task::GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    order: 0,
                    status: Some(task::GoalRunStatus::Completed),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-2".to_string(),
                    title: "Deploy".to_string(),
                    order: 1,
                    status: Some(task::GoalRunStatus::Failed),
                    ..Default::default()
                },
            ],
        ),
    ));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-2".to_string()),
    });

    let handled = model.handle_key(KeyCode::Char('r'), KeyModifiers::CONTROL);

    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Rerun from step 2 \"Deploy\" in goal \"Goal One\"?")
    );
}

#[test]
fn selected_goal_step_shift_r_lowercase_key_requests_authoritative_goal_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_with_steps(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Failed,
            vec![
                task::GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    order: 0,
                    status: Some(task::GoalRunStatus::Completed),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-2".to_string(),
                    title: "Deploy".to_string(),
                    order: 1,
                    status: Some(task::GoalRunStatus::Failed),
                    ..Default::default()
                },
            ],
        ),
    ));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-2".to_string()),
    });

    let handled = model.handle_key(KeyCode::Char('r'), KeyModifiers::SHIFT);

    assert!(!handled);
    assert_eq!(model.modal.top(), None);
    assert_eq!(
        next_goal_run_detail_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_run_checkpoints_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(matches!(daemon_rx.try_recv(), Ok(DaemonCommand::Refresh)));
    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::RefreshServices)
    ));
}

#[test]
fn selected_goal_step_action_menu_can_send_retry_step() {
    let (mut model, mut daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_with_steps(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Failed,
            vec![
                task::GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    order: 0,
                    status: Some(task::GoalRunStatus::Completed),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-2".to_string(),
                    title: "Deploy".to_string(),
                    order: 1,
                    status: Some(task::GoalRunStatus::Failed),
                    ..Default::default()
                },
            ],
        ),
    ));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-2".to_string()),
    });

    let handled = model.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.modal.top(),
        Some(modal::ModalKind::GoalStepActionPicker)
    );

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::GoalStepActionPicker,
    );
    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );
    assert!(!handled);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected control-goal command"),
        DaemonCommand::ControlGoalRun {
            goal_run_id,
            action,
            step_index: Some(1),
        } if goal_run_id == "goal-1" && action == "retry_step"
    ));
}

#[test]
fn thread_picker_shift_r_requests_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ThreadPicker));
    model.sync_thread_picker_item_count();

    let handled = model.handle_key_modal(
        KeyCode::Char('R'),
        KeyModifiers::SHIFT,
        modal::ModalKind::ThreadPicker,
    );

    assert!(!handled);
    assert!(matches!(daemon_rx.try_recv(), Ok(DaemonCommand::Refresh)));
}

#[test]
fn thread_picker_shift_r_lowercase_key_requests_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ThreadPicker));
    model.sync_thread_picker_item_count();

    let handled = model.handle_key_modal(
        KeyCode::Char('r'),
        KeyModifiers::SHIFT,
        modal::ModalKind::ThreadPicker,
    );

    assert!(!handled);
    assert!(matches!(daemon_rx.try_recv(), Ok(DaemonCommand::Refresh)));
}

#[test]
fn goal_picker_shift_r_requests_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::GoalPicker));
    model.sync_goal_picker_item_count();

    let handled = model.handle_key_modal(
        KeyCode::Char('R'),
        KeyModifiers::SHIFT,
        modal::ModalKind::GoalPicker,
    );

    assert!(!handled);
    assert!(matches!(daemon_rx.try_recv(), Ok(DaemonCommand::Refresh)));
}

#[test]
fn goal_picker_shift_r_requests_goal_run_list_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::GoalPicker));
    model.sync_goal_picker_item_count();

    let handled = model.handle_key_modal(
        KeyCode::Char('R'),
        KeyModifiers::SHIFT,
        modal::ModalKind::GoalPicker,
    );

    assert!(!handled);
    assert!(matches!(daemon_rx.try_recv(), Ok(DaemonCommand::Refresh)));
    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::RefreshServices)
    ));
}

#[test]
fn goal_picker_shift_r_lowercase_key_requests_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::GoalPicker));
    model.sync_goal_picker_item_count();

    let handled = model.handle_key_modal(
        KeyCode::Char('r'),
        KeyModifiers::SHIFT,
        modal::ModalKind::GoalPicker,
    );

    assert!(!handled);
    assert!(matches!(daemon_rx.try_recv(), Ok(DaemonCommand::Refresh)));
}

#[test]
fn goal_picker_open_selected_running_goal_starts_background_hydration() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .tasks
        .reduce(task::TaskAction::GoalRunListReceived(vec![make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Running,
        )]));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::GoalPicker));
    model.sync_goal_picker_item_count();
    model.modal.reduce(modal::ModalAction::Navigate(1));

    model.handle_modal_enter(modal::ModalKind::GoalPicker);

    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun { ref goal_run_id, step_id: None })
            if goal_run_id == "goal-1"
    ));
    assert_eq!(
        next_goal_run_detail_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_run_checkpoints_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(
        model.pending_goal_hydration_refreshes.contains("goal-1"),
        "opening a live goal should keep background hydration armed for new timeline events"
    );
}

#[test]
fn sidebar_goal_enter_opens_selected_child_task() {
    let (mut model, _daemon_rx) = seed_goal_sidebar_model();
    model.goal_sidebar.cycle_tab_right();
    model.goal_sidebar.cycle_tab_right();
    model.goal_sidebar.navigate(1, 2);

    let handled = model.handle_goal_sidebar_enter();

    assert!(handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::Task { task_id }) if task_id == "task-2"
    ));
}

#[test]
fn sidebar_goal_enter_skips_stale_child_task_ids() {
    let (mut model, _daemon_rx) = seed_goal_sidebar_model();
    if let Some(run) = model.tasks.goal_run_by_id_mut("goal-1") {
        run.child_task_ids = vec!["missing-task".to_string(), "task-1".to_string()];
    }
    model.goal_sidebar.cycle_tab_right();
    model.goal_sidebar.cycle_tab_right();

    let handled = model.handle_goal_sidebar_enter();

    assert!(handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::Task { task_id }) if task_id == "task-1"
    ));
}

#[test]
fn sidebar_goal_enter_opens_selected_work_context_file() {
    let (mut model, _daemon_rx) = seed_goal_sidebar_model();
    model.goal_sidebar.cycle_tab_right();
    model.goal_sidebar.cycle_tab_right();
    model.goal_sidebar.cycle_tab_right();
    model.goal_sidebar.navigate(1, 2);

    let handled = model.handle_goal_sidebar_enter();

    assert!(handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert!(matches!(model.main_pane_view, MainPaneView::WorkContext));
    assert_eq!(
        model.tasks.selected_work_path("thread-1"),
        Some("/tmp/second.md")
    );
}

#[test]
fn sidebar_goal_enter_checkpoint_with_step_index_selects_goal_step() {
    let (mut model, _daemon_rx) = seed_goal_sidebar_model();
    model.goal_sidebar.cycle_tab_right();

    let handled = model.handle_goal_sidebar_enter();

    assert!(handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            goal_run_id,
            step_id: Some(step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-2"
    ));
}

#[test]
fn sidebar_goal_enter_checkpoint_without_step_index_is_non_destructive() {
    let (mut model, _daemon_rx) = seed_goal_sidebar_model();
    model.goal_sidebar.cycle_tab_right();
    model.goal_sidebar.navigate(1, 2);

    let handled = model.handle_goal_sidebar_enter();

    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Sidebar);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            goal_run_id,
            step_id: Some(step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-1"
    ));
}

#[test]
fn goal_run_task_view_bracket_keys_cycle_selected_step() {
    let (mut model, _daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_with_steps(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Running,
            vec![
                task::GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    order: 0,
                    status: Some(task::GoalRunStatus::Completed),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-2".to_string(),
                    title: "Execute".to_string(),
                    order: 1,
                    status: Some(task::GoalRunStatus::Running),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-3".to_string(),
                    title: "Verify".to_string(),
                    order: 2,
                    status: Some(task::GoalRunStatus::Queued),
                    ..Default::default()
                },
            ],
        ),
    ));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-2".to_string()),
    });

    let handled = model.handle_key(KeyCode::Char(']'), KeyModifiers::NONE);
    assert!(!handled);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            ref goal_run_id,
            step_id: Some(ref step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-3"
    ));

    let handled = model.handle_key(KeyCode::Char('['), KeyModifiers::NONE);
    assert!(!handled);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            ref goal_run_id,
            step_id: Some(ref step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-2"
    ));
}
