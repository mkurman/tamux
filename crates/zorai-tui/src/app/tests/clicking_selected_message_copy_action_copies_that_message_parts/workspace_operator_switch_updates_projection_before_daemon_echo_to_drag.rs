use super::*;
use crate::app::tests::goal_sidebar_tab_cycling_stays_to_collaboration_mouse_clicks_select_rows::goal_sidebar_tab_cycling_stays_mod::*;
use crate::app::tests::clicking_selected_message_copy_action_copies_that_message_to_click::in_review_open_action_opens_queued_review_task_thread_to_workspace::*;
use super::super::{build_model, unbounded_channel};
use std::sync::mpsc;
#[test]
fn workspace_operator_switch_updates_projection_before_daemon_echo() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, _cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model
        .workspace
        .set_settings(workspace_settings_for_operator(
            zorai_protocol::WorkspaceOperator::Svarog,
        ));

    model.switch_workspace_operator_from_ui(zorai_protocol::WorkspaceOperator::User);

    assert_eq!(
        model.workspace.operator(),
        zorai_protocol::WorkspaceOperator::User
    );
    assert_eq!(
        model.workspace.projection().operator,
        zorai_protocol::WorkspaceOperator::User
    );
}

#[test]
fn workspace_drag_todo_to_in_progress_moves_unassigned_task_without_running() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.width = 120;
    model.height = 40;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model
        .workspace
        .set_settings(workspace_settings_for_operator(
            zorai_protocol::WorkspaceOperator::User,
        ));
    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task_for_board(
            "todo-1",
            zorai_protocol::WorkspaceTaskStatus::Todo,
            None,
        )],
    );
    let start = workspace_hit_position(&model, |target| {
        matches!(
            target,
            widgets::workspace_board::WorkspaceBoardHitTarget::Task { task_id, status }
                if task_id == "todo-1" && status == zorai_protocol::WorkspaceTaskStatus::Todo
        )
    });
    let drop = workspace_hit_position(&model, |target| {
        matches!(
            target,
            widgets::workspace_board::WorkspaceBoardHitTarget::Column { status }
                if status == zorai_protocol::WorkspaceTaskStatus::InProgress
        )
    });

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: start.x,
        row: start.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: drop.x,
        row: drop.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::MoveWorkspaceTask(request)) => {
            assert_eq!(request.task_id, "todo-1");
            assert_eq!(
                request.status,
                zorai_protocol::WorkspaceTaskStatus::InProgress
            );
        }
        other => panic!("expected move command, got {other:?}"),
    }
    assert!(
        cmd_rx.try_recv().is_err(),
        "drag should not auto-run unassigned tasks"
    );
}

#[test]
fn workspace_drag_from_collapsed_action_row_moves_task() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.width = 120;
    model.height = 40;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model
        .workspace
        .set_settings(workspace_settings_for_operator(
            zorai_protocol::WorkspaceOperator::User,
        ));
    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task_for_board(
            "todo-1",
            zorai_protocol::WorkspaceTaskStatus::Todo,
            None,
        )],
    );
    let chat_area = model.pane_layout().chat;
    let board_inner = Rect::new(
        chat_area.x.saturating_add(1),
        chat_area.y.saturating_add(1),
        chat_area.width.saturating_sub(2),
        chat_area.height.saturating_sub(2),
    );
    let board_area = Rect::new(
        board_inner.x,
        board_inner.y.saturating_add(1),
        board_inner.width,
        board_inner.height.saturating_sub(1),
    );
    let column_width = board_area.width / 4;
    let todo_column = Rect::new(board_area.x, board_area.y, column_width, board_area.height);
    let todo_body = Rect::new(
        todo_column.x.saturating_add(1),
        todo_column.y.saturating_add(1),
        todo_column.width.saturating_sub(2),
        todo_column.height.saturating_sub(2),
    );
    let task_body = Rect::new(
        todo_body.x.saturating_add(1),
        todo_body.y.saturating_add(1),
        todo_body.width.saturating_sub(2),
        8,
    );
    let start = Position::new(task_body.x + 8, task_body.y + 4);
    let drop = workspace_hit_position(&model, |target| {
        matches!(
            target,
            widgets::workspace_board::WorkspaceBoardHitTarget::Column { status }
                if status == zorai_protocol::WorkspaceTaskStatus::InProgress
        )
    });

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: start.x,
        row: start.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: drop.x,
        row: drop.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::MoveWorkspaceTask(request)) => {
            assert_eq!(request.task_id, "todo-1");
            assert_eq!(
                request.status,
                zorai_protocol::WorkspaceTaskStatus::InProgress
            );
        }
        other => panic!("expected move command, got {other:?}"),
    }
    assert!(
        cmd_rx.try_recv().is_err(),
        "drag should not auto-run unassigned tasks"
    );
}

#[test]
fn workspace_task_open_thread_renders_return_to_workspace_and_b_restores_board() {
    let mut model = build_model();
    model.width = 120;
    model.height = 40;
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model
        .workspace
        .set_settings(workspace_settings_for_operator(
            zorai_protocol::WorkspaceOperator::User,
        ));
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "workspace-thread:thread-task".to_string(),
        title: "Workspace task thread".to_string(),
    });
    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task_for_board(
            "thread-task",
            zorai_protocol::WorkspaceTaskStatus::InProgress,
            Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
        )],
    );

    model.open_workspace_task_runtime("thread-task".to_string());

    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    let plain = render_chat_plain(&mut model);
    assert!(plain.contains("Return to workspace"), "{plain}");

    let handled = model.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::Workspace));
}

#[test]
fn workspace_task_open_thread_uses_subagent_assignee_as_responder_hint() {
    let mut model = build_model();
    model.width = 120;
    model.height = 40;
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "dola".to_string(),
        name: "Dola".to_string(),
        provider: "openai".to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("implementation".to_string()),
        enabled: true,
        builtin: false,
        immutable_identity: false,
        disable_allowed: true,
        delete_allowed: true,
        protected_reason: None,
        reasoning_effort: Some("medium".to_string()),
        openrouter_provider_order: String::new(),
        openrouter_provider_ignore: String::new(),
        openrouter_allow_fallbacks: true,
        raw_json: None,
    });
    model
        .workspace
        .set_settings(workspace_settings_for_operator(
            zorai_protocol::WorkspaceOperator::User,
        ));
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "workspace-thread:dola-task".to_string(),
        title: "Workspace task thread".to_string(),
    });
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "workspace-thread:dola-task".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "Dola is handling the task".to_string(),
            ..Default::default()
        },
    });
    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task_for_board(
            "dola-task",
            zorai_protocol::WorkspaceTaskStatus::InProgress,
            Some(zorai_protocol::WorkspaceActor::Subagent("dola".to_string())),
        )],
    );

    model.open_workspace_task_runtime("dola-task".to_string());

    let plain = render_chat_plain(&mut model);
    assert!(plain.contains("Responder: Dola"), "{plain}");
    assert!(!plain.contains("Responder: Svarog"), "{plain}");
}

#[test]
fn workspace_task_open_goal_renders_return_to_workspace_and_b_restores_board() {
    let mut model = goal_sidebar_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model
        .workspace
        .set_settings(workspace_settings_for_operator(
            zorai_protocol::WorkspaceOperator::User,
        ));
    let mut workspace_task = workspace_task_for_board(
        "goal-task",
        zorai_protocol::WorkspaceTaskStatus::InProgress,
        Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
    );
    workspace_task.goal_run_id = Some("goal-1".to_string());
    model
        .workspace
        .set_tasks("main".to_string(), vec![workspace_task]);

    model.open_workspace_task_runtime("goal-task".to_string());

    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun { ref goal_run_id, .. })
            if goal_run_id == "goal-1"
    ));
    let plain = render_chat_plain(&mut model);
    assert!(plain.contains("Return to workspace"), "{plain}");

    let button = model
        .task_return_to_workspace_button_area()
        .expect("workspace return button should be rendered");
    let click_column = button.x.saturating_add(1);
    let click_row = button.y;

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click_column,
        row: click_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: click_column,
        row: click_row,
        modifiers: KeyModifiers::NONE,
    });

    assert!(matches!(model.main_pane_view, MainPaneView::Workspace));
}

#[test]
fn dismissing_concierge_welcome_returns_to_local_landing() {
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

    model.run_concierge_action(crate::state::ConciergeActionVm {
        label: "Dismiss".to_string(),
        action_type: "dismiss".to_string(),
        thread_id: None,
    });

    assert_eq!(model.chat.active_thread_id(), None);
    assert!(model.should_show_local_landing());
    assert_eq!(model.focus, FocusArea::Input);
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::DismissConciergeWelcome) => {}
        other => panic!("expected dismiss command, got {:?}", other),
    }
    assert!(cmd_rx.try_recv().is_err(), "unexpected follow-up command");
}

#[test]
fn drag_selection_keeps_original_anchor_point_when_chat_scrolls() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
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
            content: (1..=80)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
    let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
        .chain(chat_area.y..preferred_row)
        .find(|row| {
            widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(3, *row),
            )
            .is_some()
        })
        .expect("chat transcript should expose at least one selectable row");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 3,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });
    let anchor_point = model
        .chat_drag_anchor_point
        .expect("mouse down should capture a document anchor point");

    for _ in 0..4 {
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
    }

    let current_point = model
        .chat_drag_current_point
        .expect("dragging should keep updating the current document point");
    assert_eq!(
        model.chat_drag_anchor_point,
        Some(anchor_point),
        "autoscroll should not rewrite the original selection anchor"
    );
    assert!(
            current_point.row < anchor_point.row,
            "dragging upward with autoscroll should extend the selection into older transcript rows: anchor={anchor_point:?} current={current_point:?}"
        );
}
