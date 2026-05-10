use super::*;
use crate::app::tests::goal_sidebar_tab_cycling_stays_to_collaboration_mouse_clicks_select_rows::goal_sidebar_tab_cycling_stays_mod::*;
use super::super::{rendered_chat_area, unbounded_channel};
use std::sync::mpsc;
#[test]
fn threads_return_to_goal_banner_keeps_conversation_mouse_targets_aligned() {
    let mut model = mission_control_thread_router_model(Some("thread-active"), Some("thread-root"));

    let handled = model.handle_key(KeyCode::Char('o'), KeyModifiers::CONTROL);
    assert!(!handled);

    let conversation_area = model
        .conversation_content_area()
        .expect("conversation content area should be available");
    let (selection_pos, expected_point) = (conversation_area.y
        ..conversation_area.y.saturating_add(conversation_area.height))
        .find_map(|row| {
            (conversation_area.x..conversation_area.x.saturating_add(conversation_area.width))
                .find_map(|column| {
                    let pos = Position::new(column, row);
                    widgets::chat::selection_point_from_mouse(
                        conversation_area,
                        &model.chat,
                        &model.theme,
                        model.tick_counter,
                        pos,
                    )
                    .map(|point| (pos, point))
                })
        })
        .expect("conversation content should expose a selectable point");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: selection_pos.x,
        row: selection_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.chat_drag_anchor_point, Some(expected_point));
}

#[test]
fn threads_return_to_goal_keyboard_restores_goal_run_and_step_selection() {
    let mut model = mission_control_thread_router_model(Some("thread-active"), Some("thread-root"));

    let handled = model.handle_key(KeyCode::Char('o'), KeyModifiers::CONTROL);
    assert!(!handled);

    let handled = model.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            ref goal_run_id,
            step_id: Some(ref step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-2"
    ));
}

#[test]
fn threads_return_to_goal_mouse_restores_goal_run_and_step_selection() {
    let mut model = mission_control_thread_router_model(Some("thread-active"), Some("thread-root"));

    let handled = model.handle_key(KeyCode::Char('o'), KeyModifiers::CONTROL);
    assert!(!handled);

    let button = model
        .conversation_return_to_goal_button_area()
        .expect("return-to-goal button should be rendered");
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

    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            ref goal_run_id,
            step_id: Some(ref step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-2"
    ));
}

#[test]
fn goal_thread_child_thread_sidebar_back_restores_parent_thread_and_keeps_goal_return() {
    let mut model = goal_sidebar_model();
    open_goal_execution_thread(&mut model);

    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-child".to_string(),
        title: "Child worker".to_string(),
    });
    assert!(model
        .chat
        .open_spawned_thread("thread-exec", "thread-child"));
    model.main_pane_view = MainPaneView::Conversation;
    model.focus = FocusArea::Sidebar;
    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));

    let handled = model.handle_key(KeyCode::Backspace, KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.chat.active_thread_id(), Some("thread-exec"));
    assert!(model.mission_control_return_to_goal_target().is_some());

    let handled = model.handle_key(KeyCode::Char('b'), KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun { ref goal_run_id, .. })
            if goal_run_id == "goal-1"
    ));
}

#[test]
fn goal_run_input_routes_prompt_to_goal_main_thread_before_active_step_thread() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;

    for (thread_id, title) in [
        ("thread-user", "User Thread"),
        ("thread-root", "Goal Root Thread"),
        ("thread-goal", "Goal Main Thread"),
        ("thread-step", "Goal Step Thread"),
    ] {
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: thread_id.to_string(),
            title: title.to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::ThreadDetailReceived(chat::AgentThread {
                id: thread_id.to_string(),
                title: title.to_string(),
                ..Default::default()
            }));
    }
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal Title".to_string(),
            thread_id: Some("thread-goal".to_string()),
            root_thread_id: Some("thread-root".to_string()),
            active_thread_id: Some("thread-step".to_string()),
            goal: "Ship release".to_string(),
            current_step_title: Some("Implement".to_string()),
            steps: vec![task::GoalRunStep {
                id: "step-1".to_string(),
                title: "Implement".to_string(),
                order: 0,
                ..Default::default()
            }],
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-1".to_string()),
    });
    model.focus = FocusArea::Input;
    model.input.set_text("follow the current step");

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::SendMessage {
            thread_id, content, ..
        }) => {
            assert_eq!(thread_id.as_deref(), Some("thread-goal"));
            assert_eq!(content, "follow the current step");
        }
        other => panic!("expected send-message command, got {other:?}"),
    }
    assert_eq!(model.chat.active_thread_id(), Some("thread-goal"));
    assert_eq!(
        model
            .chat
            .active_thread()
            .and_then(|thread| thread.messages.last())
            .map(|message| message.content.as_str()),
        Some("follow the current step")
    );
}

#[test]
fn goal_thread_file_preview_escape_prefers_parent_thread_over_goal() {
    let mut model = goal_sidebar_model();
    open_goal_execution_thread(&mut model);

    model.open_file_preview_path("/tmp/thread-child-preview.txt".to_string());
    assert!(matches!(model.main_pane_view, MainPaneView::FilePreview(_)));

    let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.chat.active_thread_id(), Some("thread-exec"));
}

#[test]
fn goal_thread_file_preview_close_button_prefers_parent_thread_over_goal() {
    let mut model = goal_sidebar_model();
    open_goal_execution_thread(&mut model);

    model.open_file_preview_path("/tmp/thread-child-preview.txt".to_string());
    assert!(matches!(model.main_pane_view, MainPaneView::FilePreview(_)));

    let chat_area = rendered_chat_area(&model);
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chat_area.x.saturating_add(1),
        row: chat_area.y,
        modifiers: KeyModifiers::NONE,
    });

    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.chat.active_thread_id(), Some("thread-exec"));
}

#[test]
fn goal_thread_work_context_escape_prefers_parent_thread_over_goal() {
    let mut model = goal_sidebar_model();
    open_goal_execution_thread(&mut model);
    model.tasks.reduce(task::TaskAction::WorkContextReceived(
        task::ThreadWorkContext {
            thread_id: "thread-exec".to_string(),
            entries: vec![task::WorkContextEntry {
                path: "/tmp/thread-exec-runtime.rs".to_string(),
                is_text: true,
                ..Default::default()
            }],
        },
    ));
    model.activate_sidebar_tab(SidebarTab::Files);
    model.focus = FocusArea::Sidebar;

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::WorkContext));

    let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.chat.active_thread_id(), Some("thread-exec"));
}

#[test]
fn goal_run_input_falls_back_to_active_goal_thread_when_main_thread_is_missing() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;

    for (thread_id, title) in [
        ("thread-user", "User Thread"),
        ("thread-step", "Goal Step Thread"),
    ] {
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: thread_id.to_string(),
            title: title.to_string(),
        });
        model
            .chat
            .reduce(chat::ChatAction::ThreadDetailReceived(chat::AgentThread {
                id: thread_id.to_string(),
                title: title.to_string(),
                ..Default::default()
            }));
    }
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal Title".to_string(),
            active_thread_id: Some("thread-step".to_string()),
            goal: "Ship release".to_string(),
            current_step_title: Some("Implement".to_string()),
            steps: vec![task::GoalRunStep {
                id: "step-1".to_string(),
                title: "Implement".to_string(),
                order: 0,
                ..Default::default()
            }],
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-1".to_string()),
    });
    model.focus = FocusArea::Input;
    model.input.set_text("follow the current step");

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    match cmd_rx.try_recv() {
        Ok(DaemonCommand::SendMessage {
            thread_id, content, ..
        }) => {
            assert_eq!(thread_id.as_deref(), Some("thread-step"));
            assert_eq!(content, "follow the current step");
        }
        other => panic!("expected send-message command, got {other:?}"),
    }
    assert_eq!(model.chat.active_thread_id(), Some("thread-step"));
}

#[test]
fn goal_run_input_blocks_plain_prompt_without_goal_thread_target() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;

    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-user".to_string(),
        title: "User Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::ThreadDetailReceived(chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            ..Default::default()
        }));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal Title".to_string(),
            goal: "Ship release".to_string(),
            current_step_title: Some("Implement".to_string()),
            steps: vec![task::GoalRunStep {
                id: "step-1".to_string(),
                title: "Implement".to_string(),
                order: 0,
                ..Default::default()
            }],
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-1".to_string()),
    });
    model.focus = FocusArea::Input;
    model.input.set_text("follow the current step");

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert!(
        cmd_rx.try_recv().is_err(),
        "goal pane should not send plain text without a goal thread target"
    );
    assert_eq!(
        model.status_line,
        "Goal input accepts only slash commands until an active goal thread is available"
    );
    assert_eq!(model.input.buffer(), "follow the current step");
    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
}

#[test]
fn esc_from_goal_run_keeps_user_in_goals_view() {
    let mut model = goal_sidebar_model();
    model.focus = FocusArea::Chat;
    model.tasks.reduce(task::TaskAction::SelectWorkPath {
        thread_id: "thread-1".to_string(),
        path: None,
    });

    let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun { ref goal_run_id, .. }) if goal_run_id == "goal-1"
    ));
    assert_eq!(model.focus, FocusArea::Chat);
}

#[test]
fn goal_workspace_mouse_wheel_scrolls_plan_rows() {
    let mut model = goal_sidebar_model();
    if let Some(run) = model.tasks.goal_run_by_id_mut("goal-1") {
        run.steps = (1..=60)
            .map(|idx| task::GoalRunStep {
                id: format!("step-{idx}"),
                title: format!("Step {idx}"),
                order: idx - 1,
                ..Default::default()
            })
            .collect();
    }
    model.focus = FocusArea::Chat;

    let chat_area = model.pane_layout().chat;

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: chat_area.x.saturating_add(2),
        row: chat_area.y.saturating_add(6),
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.focus, FocusArea::Chat);
    assert!(model.pane_layout().sidebar.is_none());
    assert_eq!(model.goal_workspace.plan_scroll(), 3);
}

#[test]
fn goal_workspace_mouse_wheel_scrolls_timeline_and_details_rows() {
    let mut model = goal_sidebar_model();
    if let Some(run) = model.tasks.goal_run_by_id_mut("goal-1") {
        run.events = (0..30)
            .map(|idx| task::GoalRunEvent {
                id: format!("event-{idx}"),
                phase: "execution".to_string(),
                message: format!("event {idx} with wrapped timeline details"),
                details: Some(format!(
                    "details line for event {idx} that should wrap in the timeline panel"
                )),
                step_index: Some(1),
                ..Default::default()
            })
            .collect();
    }
    model.tasks.reduce(task::TaskAction::WorkContextReceived(
        task::ThreadWorkContext {
            thread_id: "thread-1".to_string(),
            entries: (0..30)
                .map(|idx| task::WorkContextEntry {
                    path: format!("/tmp/file-{idx}.md"),
                    goal_run_id: Some("goal-1".to_string()),
                    is_text: true,
                    ..Default::default()
                })
                .collect(),
        },
    ));
    model.focus = FocusArea::Chat;

    let chat_area = model.pane_layout().chat;
    let (_, timeline, details) = goal_workspace_click_targets(chat_area);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: timeline.x,
        row: timeline.y.saturating_add(2),
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(model.goal_workspace.timeline_scroll(), 3);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: details.x,
        row: details.y.saturating_add(2),
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(model.goal_workspace.detail_scroll(), 3);
}
