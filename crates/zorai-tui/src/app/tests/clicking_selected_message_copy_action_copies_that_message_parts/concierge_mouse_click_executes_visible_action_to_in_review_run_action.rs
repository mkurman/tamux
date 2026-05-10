use super::*;
use super::super::{build_model, unbounded_channel};
use std::sync::mpsc;
#[test]
fn concierge_mouse_click_executes_visible_action() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.agent_config_loaded = true;
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
                    action_type: "start_goal_run".to_string(),
                    thread_id: None,
                },
            ],
            is_concierge_welcome: true,
            ..Default::default()
        },
    });

    let concierge_area = model.pane_layout().concierge;
    let action_pos = (concierge_area.y..concierge_area.y.saturating_add(concierge_area.height))
        .find_map(|row| {
            (concierge_area.x..concierge_area.x.saturating_add(concierge_area.width)).find_map(
                |column| {
                    let pos = Position::new(column, row);
                    if widgets::concierge::hit_test(
                        concierge_area,
                        model.chat.active_actions(),
                        model.concierge.selected_action,
                        Some(
                            model
                                .current_conversation_agent_profile()
                                .agent_label
                                .as_str(),
                        ),
                        pos,
                    ) == Some(widgets::concierge::ConciergeHitTarget::Action(1))
                    {
                        Some(pos)
                    } else {
                        None
                    }
                },
            )
        })
        .expect("concierge welcome should expose a clickable second action");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: action_pos.x,
        row: action_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::DismissConciergeWelcome) => {}
        other => panic!("expected dismiss command first, got {:?}", other),
    }
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RequestThread {
            thread_id,
            message_limit,
            message_offset,
        }) => {
            assert_eq!(thread_id, "concierge");
            assert_eq!(
                message_limit,
                Some(model.config.tui_chat_history_page_size as usize)
            );
            assert_eq!(message_offset, Some(0));
        }
        other => panic!("expected thread request command, got {:?}", other),
    }
    assert_eq!(model.focus, FocusArea::Input);
    assert_eq!(model.input.buffer(), "/new-goal ");
    assert!(model.chat.active_actions().is_empty());
}

#[test]
fn workspace_view_does_not_show_concierge_action_bar() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.main_pane_view = MainPaneView::Workspace;
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
            actions: vec![chat::MessageAction {
                label: "Continue: research".to_string(),
                action_type: "dismiss".to_string(),
                thread_id: None,
            }],
            is_concierge_welcome: true,
            ..Default::default()
        },
    });

    assert!(!model.actions_bar_visible());
    assert_eq!(model.concierge_banner_height(), 0);
}

#[test]
fn workspace_enter_activates_new_task_not_concierge_action() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.agent_config_loaded = true;
    model.main_pane_view = MainPaneView::Workspace;
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
            actions: vec![chat::MessageAction {
                label: "Dismiss".to_string(),
                action_type: "dismiss".to_string(),
                thread_id: None,
            }],
            is_concierge_welcome: true,
            ..Default::default()
        },
    });

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(
        model.modal.top(),
        Some(modal::ModalKind::WorkspaceCreateTask)
    );
    assert!(cmd_rx.try_recv().is_err());
}

#[test]
fn workspace_command_opens_workspace_picker_and_requests_list() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);

    model.handle_workspace_command("");

    assert_eq!(model.modal.top(), Some(modal::ModalKind::WorkspacePicker));
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::ListWorkspaceSettings) => {}
        other => panic!("expected workspace list request, got {other:?}"),
    }
}

#[test]
fn workspace_picker_enter_switches_workspace_and_loads_board() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.open_workspace_picker();
    let _ = cmd_rx.try_recv();
    model.handle_client_event(ClientEvent::WorkspaceSettingsList(vec![
        zorai_protocol::WorkspaceSettings {
            workspace_id: "alpha".to_string(),
            workspace_root: None,
            operator: zorai_protocol::WorkspaceOperator::User,
            repo_monitor_enabled: false,
            repo_monitor_include_dirs: vec![],
            repo_monitor_exclude_dirs: vec![],
            created_at: 1,
            updated_at: 1,
        },
        zorai_protocol::WorkspaceSettings {
            workspace_id: "beta".to_string(),
            workspace_root: None,
            operator: zorai_protocol::WorkspaceOperator::Svarog,
            repo_monitor_enabled: false,
            repo_monitor_include_dirs: vec![],
            repo_monitor_exclude_dirs: vec![],
            created_at: 2,
            updated_at: 2,
        },
    ]));
    model.modal.reduce(modal::ModalAction::Navigate(2));

    model.submit_workspace_picker();

    assert_eq!(model.modal.top(), None);
    assert!(matches!(model.main_pane_view, MainPaneView::Workspace));
    assert_eq!(model.workspace.workspace_id(), "beta");
    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::GetWorkspaceSettings { workspace_id }) => {
                assert_eq!(workspace_id, "beta");
                break;
            }
            other => panic!("expected settings request, got {other:?}"),
        }
    }
    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::ListWorkspaceTasks {
                workspace_id,
                include_deleted,
            }) => {
                assert_eq!(workspace_id, "beta");
                assert!(!include_deleted);
                break;
            }
            other => panic!("expected task list request, got {other:?}"),
        }
    }
}

#[test]
fn workspace_picker_click_switches_workspace_and_loads_board() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.open_workspace_picker();
    let _ = cmd_rx.try_recv();
    model.handle_client_event(ClientEvent::WorkspaceSettingsList(vec![
        zorai_protocol::WorkspaceSettings {
            workspace_id: "alpha".to_string(),
            workspace_root: None,
            operator: zorai_protocol::WorkspaceOperator::User,
            repo_monitor_enabled: false,
            repo_monitor_include_dirs: vec![],
            repo_monitor_exclude_dirs: vec![],
            created_at: 1,
            updated_at: 1,
        },
        zorai_protocol::WorkspaceSettings {
            workspace_id: "beta".to_string(),
            workspace_root: None,
            operator: zorai_protocol::WorkspaceOperator::Svarog,
            repo_monitor_enabled: false,
            repo_monitor_include_dirs: vec![],
            repo_monitor_exclude_dirs: vec![],
            created_at: 2,
            updated_at: 2,
        },
    ]));

    let (_, overlay_area) = model
        .current_modal_area()
        .expect("workspace picker should have an overlay area");
    let inner = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .inner(overlay_area);
    let beta_row = inner.y.saturating_add(5);
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: inner.x.saturating_add(2),
        row: beta_row,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.modal.top(), None);
    assert!(matches!(model.main_pane_view, MainPaneView::Workspace));
    assert_eq!(model.workspace.workspace_id(), "beta");
    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::GetWorkspaceSettings { workspace_id }) => {
                assert_eq!(workspace_id, "beta");
                break;
            }
            other => panic!("expected settings request, got {other:?}"),
        }
    }
    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::ListWorkspaceTasks {
                workspace_id,
                include_deleted,
            }) => {
                assert_eq!(workspace_id, "beta");
                assert!(!include_deleted);
                break;
            }
            other => panic!("expected task list request, got {other:?}"),
        }
    }
}

#[test]
fn workspace_picker_selection_ignores_delayed_concierge_welcome() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.agent_config_loaded = true;
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
            actions: vec![chat::MessageAction {
                label: "Dismiss".to_string(),
                action_type: "dismiss".to_string(),
                thread_id: None,
            }],
            is_concierge_welcome: true,
            ..Default::default()
        },
    });
    model
        .concierge
        .reduce(crate::state::ConciergeAction::WelcomeReceived {
            content: "Welcome".to_string(),
            actions: vec![crate::state::ConciergeActionVm {
                label: "Dismiss".to_string(),
                action_type: "dismiss".to_string(),
                thread_id: None,
            }],
        });
    model.open_workspace_picker();
    let _ = cmd_rx.try_recv();
    model.handle_client_event(ClientEvent::WorkspaceSettingsList(vec![
        zorai_protocol::WorkspaceSettings {
            workspace_id: "alpha".to_string(),
            workspace_root: None,
            operator: zorai_protocol::WorkspaceOperator::User,
            repo_monitor_enabled: false,
            repo_monitor_include_dirs: vec![],
            repo_monitor_exclude_dirs: vec![],
            created_at: 1,
            updated_at: 1,
        },
        zorai_protocol::WorkspaceSettings {
            workspace_id: "beta".to_string(),
            workspace_root: None,
            operator: zorai_protocol::WorkspaceOperator::Svarog,
            repo_monitor_enabled: false,
            repo_monitor_include_dirs: vec![],
            repo_monitor_exclude_dirs: vec![],
            created_at: 2,
            updated_at: 2,
        },
    ]));
    model.modal.reduce(modal::ModalAction::Navigate(2));

    model.submit_workspace_picker();
    model.handle_concierge_welcome_event(
        "Late welcome".to_string(),
        vec![crate::state::ConciergeActionVm {
            label: "Start new session".to_string(),
            action_type: "start_new".to_string(),
            thread_id: None,
        }],
    );

    assert!(matches!(model.main_pane_view, MainPaneView::Workspace));
    assert_eq!(model.workspace.workspace_id(), "beta");
    assert!(!model.concierge.welcome_visible);
    assert!(model.chat.active_actions().is_empty());
    assert!(
        matches!(
            cmd_rx.try_recv(),
            Ok(DaemonCommand::DismissConciergeWelcome)
        ),
        "workspace navigation should dismiss the active concierge welcome"
    );
}

#[test]
fn workspace_refresh_key_reloads_current_board() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;

    let handled = model.handle_key(KeyCode::Char('r'), KeyModifiers::NONE);

    assert!(!handled);
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::GetWorkspaceSettings { workspace_id }) => {
            assert_eq!(workspace_id, "main");
        }
        other => panic!("expected settings request, got {other:?}"),
    }
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::ListWorkspaceTasks {
            workspace_id,
            include_deleted,
        }) => {
            assert_eq!(workspace_id, "main");
            assert!(!include_deleted);
        }
        other => panic!("expected task list request, got {other:?}"),
    }
}

#[test]
fn workspace_arrow_navigation_skips_removed_refresh_button() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;

    model.handle_key(KeyCode::Right, KeyModifiers::NONE);
    model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert_eq!(
        model.workspace_board_selection,
        Some(widgets::workspace_board::WorkspaceBoardHitTarget::Toolbar(
            widgets::workspace_board::WorkspaceBoardToolbarAction::ToggleOperator
        ))
    );
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::SetWorkspaceOperator { workspace_id, .. }) => {
            assert_eq!(workspace_id, "main");
        }
        other => panic!("expected operator update request, got {other:?}"),
    }
}

#[test]
fn in_review_run_action_opens_queued_review_task_thread() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model.workspace.set_tasks(
        "main".to_string(),
        vec![zorai_protocol::WorkspaceTask {
            id: "wtask-1".to_string(),
            workspace_id: "main".to_string(),
            title: "Review me".to_string(),
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            description: "Do it".to_string(),
            definition_of_done: None,
            priority: zorai_protocol::WorkspacePriority::Low,
            status: zorai_protocol::WorkspaceTaskStatus::InReview,
            sort_order: 1,
            reporter: zorai_protocol::WorkspaceActor::User,
            assignee: Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
            reviewer: Some(zorai_protocol::WorkspaceActor::Subagent("qa".to_string())),
            thread_id: Some("workspace-thread".to_string()),
            goal_run_id: None,
            runtime_history: Vec::new(),
            created_at: 1,
            updated_at: 1,
            started_at: None,
            completed_at: None,
            deleted_at: None,
            last_notice_id: None,
        }],
    );
    model
        .workspace
        .set_notices(vec![zorai_protocol::WorkspaceNotice {
            id: "notice-1".to_string(),
            workspace_id: "main".to_string(),
            task_id: "wtask-1".to_string(),
            notice_type: "review_requested".to_string(),
            message:
                "Workspace task review requested from subagent:qa; queued review task review-task-1"
                    .to_string(),
            actor: Some(zorai_protocol::WorkspaceActor::Subagent("qa".to_string())),
            created_at: 2,
        }]);
    model
        .tasks
        .reduce(task::TaskAction::TaskUpdate(task::AgentTask {
            id: "review-task-1".to_string(),
            title: "Review".to_string(),
            thread_id: Some("review-thread-1".to_string()),
            ..Default::default()
        }));

    model.activate_workspace_task_action(
        "wtask-1".to_string(),
        zorai_protocol::WorkspaceTaskStatus::InReview,
        widgets::workspace_board::WorkspaceBoardAction::Run,
    );

    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::RequestThread { thread_id, .. }) => {
                assert_eq!(thread_id, "review-thread-1");
                break;
            }
            other => panic!("expected review thread request, got {other:?}"),
        }
    }
}
