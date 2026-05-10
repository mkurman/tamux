use super::clicking_footer_queue_indicator_opens_queued_prompts_modal::sample_notification;
use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
#[test]
fn notifications_modal_row_action_focus_uses_left_right_and_enter() {
    let (mut model, _daemon_rx) = make_model();
    model
        .notifications
        .reduce(crate::state::NotificationsAction::Replace(vec![
            sample_notification(None),
        ]));
    model.toggle_notifications_modal();

    let quit = model.handle_key_modal(
        KeyCode::Tab,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert_eq!(model.notifications.selected_row_action_index(), Some(0));

    let quit = model.handle_key_modal(
        KeyCode::Right,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert_eq!(model.notifications.selected_row_action_index(), Some(1));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert!(model
        .notifications
        .selected_item()
        .and_then(|notification| notification.read_at)
        .is_some());
}

#[test]
fn pinned_budget_modal_dismiss_restores_chat_focus() {
    let (mut model, _daemon_rx) = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Pinned".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            id: Some("message-1".to_string()),
            role: chat::MessageRole::Assistant,
            content: "Pinned content".to_string(),
            ..Default::default()
        },
    });
    model.chat.select_message(Some(0));
    model.pending_pinned_budget_exceeded = Some(crate::app::PendingPinnedBudgetExceeded {
        current_pinned_chars: 100,
        pinned_budget_chars: 120,
        candidate_pinned_chars: 160,
    });
    model.modal.reduce(modal::ModalAction::Push(
        modal::ModalKind::PinnedBudgetExceeded,
    ));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::PinnedBudgetExceeded,
    );

    assert!(!quit);
    assert!(model.modal.top().is_none());
    assert_eq!(model.chat.selected_message(), Some(0));
    assert!(model.pending_pinned_budget_exceeded.is_none());
}

#[test]
fn goal_view_action_menu_runtime_assignment_reassign_requires_confirmation() {
    let (mut model, mut daemon_rx) = make_model();
    model.focus = FocusArea::Chat;
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            config::FetchedModel {
                id: "gpt-5.4".to_string(),
                name: Some("GPT-5.4".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
            config::FetchedModel {
                id: "gpt-5.4-mini".to_string(),
                name: Some("GPT-5.4 Mini".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
        ]));
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal One".to_string(),
            status: Some(task::GoalRunStatus::Running),
            goal: "Mission Control runtime edit".to_string(),
            thread_id: Some("thread-1".to_string()),
            root_thread_id: Some("thread-1".to_string()),
            active_thread_id: Some("thread-2".to_string()),
            runtime_assignment_list: vec![
                make_runtime_assignment(
                    zorai_protocol::AGENT_ID_SWAROG,
                    "openai",
                    "gpt-5.4",
                    Some("medium"),
                ),
                make_runtime_assignment("reviewer", "openai", "gpt-5.4", Some("low")),
            ],
            launch_assignment_snapshot: vec![
                make_runtime_assignment(
                    zorai_protocol::AGENT_ID_SWAROG,
                    "openai",
                    "gpt-5.4",
                    Some("medium"),
                ),
                make_runtime_assignment("reviewer", "openai", "gpt-5.4", Some("low")),
            ],
            current_step_owner_profile: Some(make_goal_owner_profile(
                "Swarog",
                "openai",
                "gpt-5.4",
                Some("medium"),
            )),
            ..Default::default()
        }));
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

    let model_edit_index = model
        .goal_action_picker_items()
        .iter()
        .position(|item| item.label() == "Edit Runtime Model")
        .expect("runtime model action should be available");
    if model_edit_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(model_edit_index as i32));
    }

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::GoalStepActionPicker,
    );
    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::GoalComposer));
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    let _ = collect_daemon_commands(&mut daemon_rx);

    navigate_model_picker_to(&mut model, "gpt-5.4-mini");

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );
    assert!(!handled);
    assert_eq!(
        model.modal.top(),
        Some(modal::ModalKind::GoalStepActionPicker)
    );

    let reassign_index = model
        .goal_action_picker_items()
        .iter()
        .position(|item| item.label() == "Reassign Active Step")
        .expect("reassign action should be available");
    if reassign_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(reassign_index as i32));
    }

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
        Some("Reassign the active step with the pending Mission Control roster change?")
    );

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );
    assert!(!handled);
    assert!(
        daemon_rx.try_recv().is_err(),
        "runtime assignment edit should stay TUI-local without daemon command"
    );
    assert_eq!(
        model.goal_mission_control.pending_role_assignments.as_ref(),
        Some(&vec![
            make_runtime_assignment(
                zorai_protocol::AGENT_ID_SWAROG,
                "openai",
                "gpt-5.4-mini",
                Some("medium"),
            ),
            make_runtime_assignment("reviewer", "openai", "gpt-5.4", Some("low")),
        ])
    );
    assert_eq!(
        model.goal_mission_control.pending_runtime_apply_modes,
        vec![
            Some(goal_mission_control::RuntimeAssignmentApplyMode::ReassignActiveStep),
            None,
        ]
    );
}

#[test]
fn goal_view_action_menu_runtime_assignment_cancel_clears_pending_confirmation_state() {
    let (mut model, mut daemon_rx_cmd) = make_model();
    model.connected = true;
    model.focus = FocusArea::Chat;
    model
        .config
        .reduce(config::ConfigAction::ModelsFetched(vec![
            config::FetchedModel {
                id: "gpt-5.4".to_string(),
                name: Some("GPT-5.4".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
            config::FetchedModel {
                id: "gpt-5.4-mini".to_string(),
                name: Some("GPT-5.4 Mini".to_string()),
                context_window: Some(128_000),
                pricing: None,
                metadata: None,
            },
        ]));
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal One".to_string(),
            status: Some(task::GoalRunStatus::Running),
            goal: "Mission Control runtime edit".to_string(),
            thread_id: Some("thread-1".to_string()),
            root_thread_id: Some("thread-1".to_string()),
            active_thread_id: Some("thread-2".to_string()),
            runtime_assignment_list: vec![
                make_runtime_assignment(
                    zorai_protocol::AGENT_ID_SWAROG,
                    "openai",
                    "gpt-5.4",
                    Some("medium"),
                ),
                make_runtime_assignment("reviewer", "openai", "gpt-5.4", Some("low")),
            ],
            launch_assignment_snapshot: vec![
                make_runtime_assignment(
                    zorai_protocol::AGENT_ID_SWAROG,
                    "openai",
                    "gpt-5.4",
                    Some("medium"),
                ),
                make_runtime_assignment("reviewer", "openai", "gpt-5.4", Some("low")),
            ],
            current_step_owner_profile: Some(make_goal_owner_profile(
                "Swarog",
                "openai",
                "gpt-5.4",
                Some("medium"),
            )),
            ..Default::default()
        }));
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

    let model_edit_index = model
        .goal_action_picker_items()
        .iter()
        .position(|item| item.label() == "Edit Runtime Model")
        .expect("runtime model action should be available");
    if model_edit_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(model_edit_index as i32));
    }

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::GoalStepActionPicker,
    );
    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ModelPicker));
    let _ = collect_daemon_commands(&mut daemon_rx_cmd);

    navigate_model_picker_to(&mut model, "gpt-5.4-mini");

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::ModelPicker,
    );
    assert!(!handled);
    assert_eq!(
        model.modal.top(),
        Some(modal::ModalKind::GoalStepActionPicker)
    );

    let reassign_index = model
        .goal_action_picker_items()
        .iter()
        .position(|item| item.label() == "Reassign Active Step")
        .expect("reassign action should be available");
    if reassign_index > 0 {
        model
            .modal
            .reduce(modal::ModalAction::Navigate(reassign_index as i32));
    }

    let handled = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::GoalStepActionPicker,
    );
    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ChatActionConfirm));

    let handled = model.handle_key_modal(
        KeyCode::Esc,
        KeyModifiers::NONE,
        modal::ModalKind::ChatActionConfirm,
    );
    assert!(!handled);
    assert!(model.goal_mission_control.pending_runtime_change.is_none());
    assert!(
        daemon_rx_cmd.try_recv().is_err(),
        "runtime assignment edit should stay TUI-local without daemon command"
    );

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });
    model.focus = FocusArea::Chat;

    let labels: Vec<_> = model
        .goal_action_picker_items()
        .iter()
        .map(|item| item.label())
        .collect();
    assert!(labels.contains(&"Edit Runtime Model"), "{labels:?}");
    assert!(!labels.contains(&"Reassign Active Step"), "{labels:?}");
}
