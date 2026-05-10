use super::super::{build_model, unbounded_channel};
use super::*;
use std::sync::mpsc;
#[test]
fn goal_composer_opens_mission_control_preflight_with_main_agent_defaults() {
    let mut model = build_model();
    model.config.provider = "openai".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.reasoning_effort = "low".to_string();

    model.open_new_goal_view();

    assert!(matches!(model.main_pane_view, MainPaneView::GoalComposer));
    assert_eq!(model.input.buffer(), "");
    assert_eq!(
        model.goal_mission_control.preset_source_label,
        "Main agent inheritance"
    );
    assert_eq!(
        model.goal_mission_control.main_assignment.provider,
        "openai"
    );
    assert_eq!(model.goal_mission_control.main_assignment.model, "gpt-5.4");
    assert_eq!(
        model
            .goal_mission_control
            .main_assignment
            .reasoning_effort
            .as_deref(),
        Some("low")
    );
    assert_eq!(model.goal_mission_control.role_assignments.len(), 1);
    assert!(
        !model.goal_mission_control.save_as_default_pending,
        "save-as-default should start disabled"
    );
}

#[test]
fn goal_composer_loads_previous_goal_settings_as_defaults_when_present() {
    let mut model = build_model();
    model.config.provider = "fallback-provider".to_string();
    model.config.model = "fallback-model".to_string();
    model.config.reasoning_effort = "fallback-effort".to_string();
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Previous Goal".to_string(),
            updated_at: 42,
            launch_assignment_snapshot: vec![
                task::GoalAgentAssignment {
                    role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
                    enabled: true,
                    provider: "previous-provider".to_string(),
                    model: "previous-model".to_string(),
                    reasoning_effort: Some("medium".to_string()),
                    inherit_from_main: false,
                },
                task::GoalAgentAssignment {
                    role_id: "planner".to_string(),
                    enabled: true,
                    provider: "planner-provider".to_string(),
                    model: "planner-model".to_string(),
                    reasoning_effort: Some("high".to_string()),
                    inherit_from_main: false,
                },
            ],
            runtime_assignment_list: vec![task::GoalAgentAssignment {
                role_id: "planner".to_string(),
                enabled: true,
                provider: "planner-provider".to_string(),
                model: "planner-model".to_string(),
                reasoning_effort: Some("high".to_string()),
                inherit_from_main: false,
            }],
            ..Default::default()
        }));

    model.open_new_goal_view();

    assert_eq!(
        model.goal_mission_control.preset_source_label,
        "Previous goal snapshot"
    );
    assert_eq!(
        model.goal_mission_control.main_assignment.provider,
        "previous-provider"
    );
    assert_eq!(
        model.goal_mission_control.main_assignment.model,
        "previous-model"
    );
    assert_eq!(
        model
            .goal_mission_control
            .main_assignment
            .reasoning_effort
            .as_deref(),
        Some("medium")
    );
    assert_eq!(model.goal_mission_control.role_assignments.len(), 2);
    assert_eq!(
        model.goal_mission_control.role_assignments[1].provider,
        "planner-provider"
    );
}

#[test]
fn goal_composer_enter_launches_from_mission_control_preflight_state_instead_of_raw_input_mode() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.main_pane_view = MainPaneView::GoalComposer;
    model.focus = FocusArea::Input;
    model.goal_mission_control.prompt_text = "Mission Control goal".to_string();
    model.input.set_text("raw input that should be ignored");

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::DismissConciergeWelcome) => {}
        other => panic!(
            "expected concierge dismissal before launch, got {:?}",
            other
        ),
    }
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::StartGoalRun {
            goal,
            thread_id,
            session_id,
            ..
        }) => {
            assert_eq!(goal, "Mission Control goal");
            assert_eq!(thread_id, None);
            assert_eq!(session_id, None);
        }
        other => panic!("expected start-goal-run command, got {:?}", other),
    }
    assert_eq!(
        model.status_line, "Starting goal run...",
        "preflight submission should launch the goal run"
    );
}

#[test]
fn goal_command_opens_blank_preflight_even_when_goal_run_is_selected() {
    let mut model = build_model();
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Existing Goal".to_string(),
            updated_at: 42,
            launch_assignment_snapshot: vec![
                task::GoalAgentAssignment {
                    role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
                    enabled: true,
                    provider: "runtime-provider".to_string(),
                    model: "runtime-model".to_string(),
                    reasoning_effort: Some("high".to_string()),
                    inherit_from_main: false,
                },
                task::GoalAgentAssignment {
                    role_id: "planner".to_string(),
                    enabled: true,
                    provider: "planner-provider".to_string(),
                    model: "planner-model".to_string(),
                    reasoning_effort: Some("medium".to_string()),
                    inherit_from_main: false,
                },
            ],
            runtime_assignment_list: vec![task::GoalAgentAssignment {
                role_id: "planner".to_string(),
                enabled: true,
                provider: "planner-provider".to_string(),
                model: "planner-model".to_string(),
                reasoning_effort: Some("medium".to_string()),
                inherit_from_main: false,
            }],
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.execute_command("new-goal");

    assert!(matches!(model.main_pane_view, MainPaneView::GoalComposer));
    assert_eq!(model.focus, FocusArea::Input);
    assert_eq!(model.goal_mission_control.runtime_goal_run_id, None);
    assert_eq!(model.goal_mission_control.prompt_text, "");
    assert_eq!(
        model.goal_mission_control.preset_source_label,
        "Previous goal snapshot"
    );
}

#[test]
fn goal_composer_prompt_typing_updates_preflight_prompt_text() {
    let mut model = build_model();
    model.execute_command("new-goal");

    let handled_g = model.handle_key(KeyCode::Char('g'), KeyModifiers::NONE);
    let handled_o = model.handle_key(KeyCode::Char('o'), KeyModifiers::NONE);

    assert!(!handled_g);
    assert!(!handled_o);
    assert_eq!(model.input.buffer(), "go");
    assert_eq!(model.goal_mission_control.prompt_text, "go");
}

#[test]
fn goal_composer_enter_expands_multiline_paste_before_launching() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.execute_command("new-goal");

    model.handle_paste("first line\nsecond line".to_string());

    assert!(model.input.buffer().contains("\x00PASTE:0\x00"));
    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    let mut launched_goal = None;
    while let Ok(command) = cmd_rx.try_recv() {
        if let DaemonCommand::StartGoalRun { goal, .. } = command {
            launched_goal = Some(goal);
            break;
        }
    }
    assert_eq!(
        launched_goal.as_deref(),
        Some("first line\nsecond line"),
        "goal launch should receive the expanded paste content"
    );
}

#[test]
fn goal_composer_escape_cancels_preflight_and_restores_previous_goal_view() {
    let mut model = build_model();
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Existing Goal".to_string(),
            updated_at: 42,
            ..Default::default()
        }));
    let previous = SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    };
    model.main_pane_view = MainPaneView::Task(previous.clone());

    model.execute_command("new-goal");
    let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Chat);
    match &model.main_pane_view {
        MainPaneView::Task(target) => assert_eq!(target, &previous),
        other => panic!("expected previous goal pane after cancel, got {:?}", other),
    }
}

#[test]
fn goal_composer_arrow_keys_navigate_preflight_role_assignments() {
    let mut model = build_model();
    model.goal_mission_control = goal_mission_control::GoalMissionControlState::from_goal_snapshot(
        vec![
            task::GoalAgentAssignment {
                role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
                enabled: true,
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                reasoning_effort: Some("low".to_string()),
                inherit_from_main: false,
            },
            task::GoalAgentAssignment {
                role_id: "planner".to_string(),
                enabled: true,
                provider: "openai".to_string(),
                model: "gpt-5.4-mini".to_string(),
                reasoning_effort: Some("medium".to_string()),
                inherit_from_main: false,
            },
        ],
        task::GoalAgentAssignment::default(),
        "Previous goal snapshot",
    );
    model.main_pane_view = MainPaneView::GoalComposer;
    model.focus = FocusArea::Chat;

    let handled = model.handle_key(KeyCode::Down, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(
        model.goal_mission_control.selected_runtime_assignment_index,
        1
    );
}

#[test]
fn goal_composer_provider_hotkey_opens_picker_for_selected_assignment() {
    let mut model = build_model();
    model.goal_mission_control = goal_mission_control::GoalMissionControlState::from_goal_snapshot(
        vec![task::GoalAgentAssignment {
            role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
            enabled: true,
            provider: "openai".to_string(),
            model: "gpt-5.4".to_string(),
            reasoning_effort: Some("low".to_string()),
            inherit_from_main: false,
        }],
        task::GoalAgentAssignment::default(),
        "Main agent inheritance",
    );
    model.main_pane_view = MainPaneView::GoalComposer;
    model.focus = FocusArea::Chat;

    let handled = model.handle_key(KeyCode::Char('p'), KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ProviderPicker));
    assert_eq!(
        model.goal_mission_control.pending_runtime_edit,
        Some(goal_mission_control::RuntimeAssignmentEditRequest {
            row_index: 0,
            field: goal_mission_control::RuntimeAssignmentEditField::Provider,
        })
    );
}

#[test]
fn goal_view_m_hotkey_opens_runtime_mission_control_editor() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Existing Goal".to_string(),
            status: Some(task::GoalRunStatus::Running),
            runtime_assignment_list: vec![task::GoalAgentAssignment {
                role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
                enabled: true,
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                reasoning_effort: Some("low".to_string()),
                inherit_from_main: false,
            }],
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let handled = model.handle_key(KeyCode::Char('m'), KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::GoalComposer));
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_mission_control.runtime_goal_run_id.as_deref(),
        Some("goal-1")
    );
}

#[test]
fn concierge_arrow_keys_navigate_visible_actions() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "concierge".to_string(),
        title: "Concierge".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("concierge".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "concierge".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "Welcome".to_string(),
            actions: vec![
                chat::MessageAction {
                    label: "One".to_string(),
                    action_type: "dismiss".to_string(),
                    thread_id: None,
                },
                chat::MessageAction {
                    label: "Two".to_string(),
                    action_type: "dismiss".to_string(),
                    thread_id: None,
                },
            ],
            is_concierge_welcome: true,
            ..Default::default()
        },
    });

    let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.concierge.selected_action, 1);
}

#[test]
fn selected_message_arrow_keys_navigate_inline_actions() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::User,
            content: "first".to_string(),
            ..Default::default()
        },
    });
    model.chat.select_message(Some(0));
    model.chat.select_message_action(0);

    let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.selected_message_action(), 1);
}

#[test]
fn retry_wait_keyboard_can_select_yes_and_trigger_immediate_retry() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_retry_status_event(
        "thread-1".to_string(),
        "waiting".to_string(),
        1,
        1,
        30_000,
        "transport".to_string(),
        "upstream transport error".to_string(),
    );

    let handled = model.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert!(!handled);

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);
    assert!(
        model.chat.retry_status().is_none(),
        "retry prompt should clear locally once retry-now is triggered"
    );

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RetryStreamNow { thread_id }) => assert_eq!(thread_id, "thread-1"),
        other => panic!("expected retry-now command, got {:?}", other),
    }
}
