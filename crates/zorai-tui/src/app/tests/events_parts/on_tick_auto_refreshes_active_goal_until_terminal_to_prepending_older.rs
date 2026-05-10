use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn on_tick_auto_refreshes_active_goal_until_terminal() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal".to_string(),
            status: Some(task::GoalRunStatus::Running),
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });
    model.config.auto_refresh_interval_secs = 1;

    for _ in 0..21 {
        model.on_tick();
    }

    assert_eq!(
        next_goal_run_detail_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_run_checkpoints_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );

    model
        .tasks
        .reduce(task::TaskAction::GoalRunUpdate(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal".to_string(),
            status: Some(task::GoalRunStatus::Completed),
            ..Default::default()
        }));

    for _ in 0..25 {
        model.on_tick();
    }

    assert!(
        next_goal_run_detail_request(&mut daemon_rx).is_none(),
        "completed goals should stop periodic auto-refresh"
    );
}

#[test]
fn on_tick_auto_refreshes_workspace_until_tasks_done() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.main_pane_view = MainPaneView::Workspace;
    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task(
            "task-1",
            zorai_protocol::WorkspaceTaskStatus::InProgress,
        )],
    );
    model.config.auto_refresh_interval_secs = 1;

    for _ in 0..21 {
        model.on_tick();
    }

    assert!(
        saw_workspace_task_list_command(&mut daemon_rx, "main"),
        "workspace board should auto-refresh while it has active tasks"
    );

    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task(
            "task-1",
            zorai_protocol::WorkspaceTaskStatus::Done,
        )],
    );

    for _ in 0..25 {
        model.on_tick();
    }

    assert!(
        !saw_workspace_task_list_command(&mut daemon_rx, "main"),
        "workspace board should stop periodic auto-refresh once tasks are done"
    );
}

#[test]
fn workspace_task_update_refreshes_visible_workspace_board() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.main_pane_view = MainPaneView::Workspace;
    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task(
            "task-1",
            zorai_protocol::WorkspaceTaskStatus::Todo,
        )],
    );

    model.handle_client_event(ClientEvent::WorkspaceTaskUpdated(workspace_task(
        "task-1",
        zorai_protocol::WorkspaceTaskStatus::InProgress,
    )));

    assert!(
        saw_workspace_task_list_command(&mut daemon_rx, "main"),
        "workspace task status changes should trigger a board refresh"
    );
}

#[test]
fn on_tick_does_not_refresh_spawned_sidebar_tasks_while_thread_is_loading() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-parent".to_string(),
            title: "Parent Thread".to_string(),
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-parent".to_string()));
    model
        .tasks
        .reduce(task::TaskAction::TaskListReceived(vec![task::AgentTask {
            id: "task-child".to_string(),
            title: "Spawned child".to_string(),
            description: "Spawned child task".to_string(),
            thread_id: Some("thread-child".to_string()),
            parent_task_id: Some("task-parent".to_string()),
            parent_thread_id: Some("thread-parent".to_string()),
            created_at: 1,
            status: Some(task::TaskStatus::InProgress),
            progress: 30,
            session_id: None,
            goal_run_id: None,
            goal_step_title: None,
            command: None,
            awaiting_approval_id: None,
            blocked_reason: None,
        }]));
    model.activate_sidebar_tab(SidebarTab::Spawned);
    model.thread_loading_id = Some("thread-parent".to_string());

    for _ in 0..25 {
        model.on_tick();
    }

    assert!(
        !saw_list_tasks_command(&mut daemon_rx),
        "spawned sidebar refresh should stay idle while the active thread is still loading"
    );
}

#[test]
fn on_tick_does_not_chain_follow_up_older_thread_page_requests_after_reload() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 120,
            loaded_message_start: 20,
            loaded_message_end: 120,
            messages: (20..120)
                .map(|index| crate::state::chat::AgentMessage {
                    id: Some(format!("msg-{index}")),
                    role: crate::state::chat::MessageRole::Assistant,
                    content: format!("msg {index}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model
        .chat
        .reduce(chat::ChatAction::ScrollChat(i32::MAX / 2));

    model.on_tick();

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(100));
        }
        other => panic!("expected first older-page request, got {other:?}"),
    }

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        total_message_count: 240,
        loaded_message_start: 5,
        loaded_message_end: 128,
        messages: (5..128)
            .map(|index| crate::wire::AgentMessage {
                id: Some(format!("msg-{index}")),
                role: crate::wire::MessageRole::Assistant,
                content: format!("msg {index}"),
                timestamp: index as u64,
                message_kind: "normal".to_string(),
                ..Default::default()
            })
            .collect(),
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    model.on_tick();
    assert!(
        next_thread_request(&mut daemon_rx).is_none(),
        "top-of-window reload should debounce follow-up history fetches"
    );

    for _ in 0..(chat::CHAT_HISTORY_FETCH_DEBOUNCE_TICKS - 1) {
        model.on_tick();
    }

    assert!(
        next_thread_request(&mut daemon_rx).is_none(),
        "older-page reload should not chain another history fetch without another user scroll"
    );
}

#[test]
fn on_tick_does_not_repeat_older_thread_page_request_while_pending() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 20;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 100,
            loaded_message_start: 80,
            loaded_message_end: 100,
            messages: (80..100)
                .map(|index| crate::state::chat::AgentMessage {
                    id: Some(format!("msg-{index}")),
                    role: crate::state::chat::MessageRole::Assistant,
                    content: format!("msg {index}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model
        .chat
        .reduce(chat::ChatAction::ScrollChat(i32::MAX / 2));

    model.on_tick();

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(22));
            assert_eq!(message_offset, Some(20));
        }
        other => panic!("expected first older-page request, got {other:?}"),
    }

    for _ in 0..(chat::CHAT_HISTORY_FETCH_DEBOUNCE_TICKS * 3) {
        model.on_tick();
    }

    assert!(
        next_thread_request(&mut daemon_rx).is_none(),
        "older-page fetch must not repeat while the previous request is still pending"
    );
}

#[test]
fn prepending_older_history_releases_the_top_edge_until_user_scrolls_again() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 400,
            loaded_message_start: 277,
            loaded_message_end: 400,
            messages: (277..400)
                .map(|index| crate::state::chat::AgentMessage {
                    id: Some(format!("msg-{index}")),
                    role: crate::state::chat::MessageRole::Assistant,
                    content: format!("msg {index}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model
        .chat
        .reduce(chat::ChatAction::ScrollChat(i32::MAX / 2));

    model.on_tick();

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(123));
        }
        other => panic!("expected first older-page request, got {other:?}"),
    }

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        total_message_count: 400,
        loaded_message_start: 154,
        loaded_message_end: 277,
        messages: (154..277)
            .map(|index| crate::wire::AgentMessage {
                id: Some(format!("msg-{index}")),
                role: crate::wire::MessageRole::Assistant,
                content: format!("msg {index}"),
                timestamp: index as u64,
                message_kind: "normal".to_string(),
                ..Default::default()
            })
            .collect(),
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    for _ in 0..chat::CHAT_HISTORY_FETCH_DEBOUNCE_TICKS {
        model.on_tick();
    }

    assert!(
        next_thread_request(&mut daemon_rx).is_none(),
        "prepend anchor should move the viewport below the new top so history does not auto-fetch again"
    );
}

#[test]
fn prepending_overlapping_older_history_releases_the_top_edge_until_user_scrolls_again() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 240,
            loaded_message_start: 20,
            loaded_message_end: 120,
            messages: (20..120)
                .map(|index| crate::state::chat::AgentMessage {
                    id: Some(format!("msg-{index}")),
                    role: crate::state::chat::MessageRole::Assistant,
                    content: format!("msg {index}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model
        .chat
        .reduce(chat::ChatAction::ScrollChat(i32::MAX / 2));

    model.on_tick();

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(220));
        }
        other => panic!("expected first older-page request, got {other:?}"),
    }

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        total_message_count: 240,
        loaded_message_start: 5,
        loaded_message_end: 128,
        messages: (5..128)
            .map(|index| crate::wire::AgentMessage {
                id: Some(format!("msg-{index}")),
                role: crate::wire::MessageRole::Assistant,
                content: format!("msg {index}"),
                timestamp: index as u64,
                message_kind: "normal".to_string(),
                ..Default::default()
            })
            .collect(),
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    for _ in 0..chat::CHAT_HISTORY_FETCH_DEBOUNCE_TICKS {
        model.on_tick();
    }

    assert!(
        next_thread_request(&mut daemon_rx).is_none(),
        "overlapping prepend should move the viewport below the new top so history does not auto-fetch again"
    );
}

#[test]
fn on_tick_requests_next_older_goal_run_page_when_scrolled_to_top_of_loaded_window() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Paged Goal".to_string(),
            loaded_step_start: 20,
            loaded_step_end: 40,
            total_step_count: 40,
            loaded_event_start: 60,
            loaded_event_end: 120,
            total_event_count: 120,
            steps: (20..40)
                .map(|idx| task::GoalRunStep {
                    id: format!("step-{idx}"),
                    title: format!("Step {idx}"),
                    instructions: format!("instructions {idx}"),
                    order: idx as u32,
                    ..Default::default()
                })
                .collect(),
            events: (60..120)
                .map(|idx| task::GoalRunEvent {
                    id: format!("event-{idx}"),
                    message: format!("event {idx}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }));

    model.on_tick();

    match next_goal_run_page_request(&mut daemon_rx) {
        Some((goal_run_id, step_offset, step_limit, event_offset, event_limit)) => {
            assert_eq!(goal_run_id, "goal-1");
            assert_eq!(step_offset, Some(0));
            assert_eq!(step_limit, Some(20));
            assert_eq!(event_offset, Some(0));
            assert_eq!(event_limit, Some(60));
        }
        other => panic!("expected older goal-run page request, got {other:?}"),
    }
}

#[test]
fn prepending_older_goal_run_history_releases_top_edge_until_user_scrolls_again() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Paged Goal".to_string(),
            loaded_step_start: 20,
            loaded_step_end: 40,
            total_step_count: 40,
            loaded_event_start: 60,
            loaded_event_end: 120,
            total_event_count: 120,
            steps: (20..40)
                .map(|idx| task::GoalRunStep {
                    id: format!("step-{idx}"),
                    title: format!("Step {idx}"),
                    instructions: format!("instructions {idx}"),
                    order: idx as u32,
                    ..Default::default()
                })
                .collect(),
            events: (60..120)
                .map(|idx| task::GoalRunEvent {
                    id: format!("event-{idx}"),
                    message: format!("event {idx}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }));

    model.on_tick();

    match next_goal_run_page_request(&mut daemon_rx) {
        Some((goal_run_id, step_offset, step_limit, event_offset, event_limit)) => {
            assert_eq!(goal_run_id, "goal-1");
            assert_eq!(step_offset, Some(0));
            assert_eq!(step_limit, Some(20));
            assert_eq!(event_offset, Some(0));
            assert_eq!(event_limit, Some(60));
        }
        other => panic!("expected initial older goal-run page request, got {other:?}"),
    }

    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Paged Goal".to_string(),
            loaded_step_start: 0,
            loaded_step_end: 20,
            total_step_count: 40,
            loaded_event_start: 0,
            loaded_event_end: 60,
            total_event_count: 120,
            steps: (0..20)
                .map(|idx| task::GoalRunStep {
                    id: format!("step-{idx}"),
                    title: format!("Step {idx}"),
                    instructions: format!("instructions {idx}"),
                    order: idx as u32,
                    ..Default::default()
                })
                .collect(),
            events: (0..60)
                .map(|idx| task::GoalRunEvent {
                    id: format!("event-{idx}"),
                    message: format!("event {idx}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }));
    model.handle_goal_run_detail_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Paged Goal".to_string(),
        loaded_step_start: 0,
        loaded_step_end: 20,
        total_step_count: 40,
        loaded_event_start: 0,
        loaded_event_end: 60,
        total_event_count: 120,
        steps: (0..20)
            .map(|idx| crate::wire::GoalRunStep {
                id: format!("step-{idx}"),
                position: idx,
                title: format!("Step {idx}"),
                instructions: format!("instructions {idx}"),
                ..Default::default()
            })
            .collect(),
        events: (0..60)
            .map(|idx| crate::wire::GoalRunEvent {
                id: format!("event-{idx}"),
                message: format!("event {idx}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    });

    model.on_tick();

    assert!(
        next_goal_run_page_request(&mut daemon_rx).is_none(),
        "prepend anchor should move the viewport below the new top so goal history does not auto-fetch again"
    );
}
