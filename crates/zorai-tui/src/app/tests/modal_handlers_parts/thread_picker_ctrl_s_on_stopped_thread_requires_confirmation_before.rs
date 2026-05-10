use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
#[test]
fn thread_picker_ctrl_s_on_stopped_thread_requires_confirmation_before_resume() {
    let (mut model, mut daemon_rx) = make_model();
    model.chat.reduce(chat::ChatAction::ThreadListReceived(vec![
        chat::AgentThread {
            id: "thread-1".into(),
            agent_name: Some("Svarog".into()),
            title: "Thread One".into(),
            messages: vec![chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: "partial answer [stopped]".into(),
                ..Default::default()
            }],
            total_message_count: 1,
            loaded_message_end: 1,
            ..Default::default()
        },
    ]));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ThreadPicker));
    model.sync_thread_picker_item_count();
    model.modal.reduce(modal::ModalAction::Navigate(1));

    let quit = model.handle_key_modal(
        KeyCode::Char('s'),
        KeyModifiers::CONTROL,
        modal::ModalKind::ThreadPicker,
    );

    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected send-message resume command"),
        DaemonCommand::SendMessage {
            thread_id,
            content,
            ..
        } if thread_id.as_deref() == Some("thread-1") && content == "continue"
    ));
}

#[test]
fn goal_picker_delete_requires_confirmation_before_sending_delete_goal() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .tasks
        .reduce(task::TaskAction::GoalRunListReceived(vec![make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Cancelled,
        )]));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::GoalPicker));
    model.sync_goal_picker_item_count();
    model.modal.reduce(modal::ModalAction::Navigate(1));

    let quit = model.handle_key_modal(
        KeyCode::Delete,
        KeyModifiers::NONE,
        modal::ModalKind::GoalPicker,
    );

    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert!(
        daemon_rx.try_recv().is_err(),
        "delete should wait for confirmation"
    );

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected delete-goal command"),
        DaemonCommand::DeleteGoalRun { goal_run_id } if goal_run_id == "goal-1"
    ));
}

#[test]
fn goal_picker_ctrl_s_running_goal_requires_confirmation_before_pause() {
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

    let quit = model.handle_key_modal(
        KeyCode::Char('s'),
        KeyModifiers::CONTROL,
        modal::ModalKind::GoalPicker,
    );

    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Pause goal run \"Goal One\"?")
    );

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected control-goal command"),
        DaemonCommand::ControlGoalRun {
            goal_run_id,
            action,
            ..
        }
            if goal_run_id == "goal-1" && action == "pause"
    ));
    assert_eq!(model.status_line, "Pausing goal run...");
}

#[test]
fn goal_picker_ctrl_s_from_handle_key_routes_to_pause_confirmation() {
    let (mut model, _daemon_rx) = make_model();
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

    let handled = model.handle_key(KeyCode::Char('s'), KeyModifiers::CONTROL);

    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Pause goal run \"Goal One\"?")
    );
}

#[test]
fn goal_picker_ctrl_s_paused_goal_requires_confirmation_before_resume() {
    let (mut model, mut daemon_rx) = make_model();
    model
        .tasks
        .reduce(task::TaskAction::GoalRunListReceived(vec![make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Paused,
        )]));
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::GoalPicker));
    model.sync_goal_picker_item_count();
    model.modal.reduce(modal::ModalAction::Navigate(1));

    let quit = model.handle_key_modal(
        KeyCode::Char('s'),
        KeyModifiers::CONTROL,
        modal::ModalKind::GoalPicker,
    );

    assert!(!quit);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );

    assert!(!quit);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected control-goal command"),
        DaemonCommand::ControlGoalRun {
            goal_run_id,
            action,
            ..
        }
            if goal_run_id == "goal-1" && action == "resume"
    ));
}

#[test]
fn goal_view_ctrl_s_paused_goal_requires_confirmation_before_resume() {
    let (mut model, _daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Paused,
        )));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let handled = model.handle_key(KeyCode::Char('s'), KeyModifiers::CONTROL);

    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Resume goal run \"Goal One\"?")
    );
}

#[test]
fn goal_view_action_menu_can_pause_running_goal_without_step_selection() {
    let (mut model, mut daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Running,
        )));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
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
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Pause goal run \"Goal One\"?")
    );

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
            step_index: None,
        } if goal_run_id == "goal-1" && action == "pause"
    ));
}

#[test]
fn goal_view_action_menu_can_delete_terminal_goal_without_picker() {
    let (mut model, mut daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Completed,
        )));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    assert!(model
        .goal_action_picker_items()
        .contains(&crate::app::commands::GoalActionPickerItem::DeleteGoal));

    let handled = model.handle_key(KeyCode::Char('a'), KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(
        model.modal.top(),
        Some(modal::ModalKind::GoalStepActionPicker)
    );
    let delete_index = model
        .goal_action_picker_items()
        .iter()
        .position(|item| *item == crate::app::commands::GoalActionPickerItem::DeleteGoal)
        .expect("delete action should be present");
    model
        .modal
        .reduce(modal::ModalAction::Navigate(delete_index as i32));

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::GoalStepActionPicker,
    );
    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));
    assert_eq!(
        model
            .pending_chat_action_confirm
            .as_ref()
            .map(PendingConfirmAction::modal_body)
            .as_deref(),
        Some("Delete goal run \"Goal One\"?")
    );

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );
    assert!(!handled);
    assert!(matches!(
        daemon_rx.try_recv().expect("expected delete-goal command"),
        DaemonCommand::DeleteGoalRun { goal_run_id } if goal_run_id == "goal-1"
    ));
}

#[test]
fn goal_view_retry_uses_current_step_without_explicit_step_selection() {
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
    if let Some(run) = model.tasks.goal_run_by_id_mut("goal-1") {
        run.current_step_index = 1;
        run.current_step_title = Some("Deploy".to_string());
    }
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
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
fn goal_view_retry_from_prompt_without_steps_opens_confirmation() {
    let (mut model, _daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Failed,
        )));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
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
        Some("Retry goal \"Goal One\" from the current prompt?")
    );
}

#[test]
fn goal_view_ctrl_r_reruns_from_prompt_without_steps() {
    let (mut model, _daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Failed,
        )));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
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
        Some("Rerun goal \"Goal One\" from the current prompt?")
    );
}

#[test]
fn goal_view_shift_r_requests_authoritative_goal_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Running,
        )));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let handled = model.handle_key(KeyCode::Char('R'), KeyModifiers::SHIFT);

    assert!(!handled);
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
fn goal_workspace_refresh_action_requests_authoritative_goal_refresh() {
    let (mut model, mut daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(make_goal_run(
            "goal-1",
            "Goal One",
            task::GoalRunStatus::Running,
        )));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let handled = model.activate_goal_workspace_action(
        crate::widgets::goal_workspace::GoalWorkspaceAction::RefreshGoal,
    );

    assert!(handled);
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
