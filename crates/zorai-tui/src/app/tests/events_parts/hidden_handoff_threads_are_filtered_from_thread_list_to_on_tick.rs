use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn hidden_handoff_threads_are_filtered_from_thread_list() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadList(vec![
        crate::wire::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            ..Default::default()
        },
        crate::wire::AgentThread {
            id: "handoff:thread-user:handoff-1".to_string(),
            title: "Handoff · Svarog -> Weles".to_string(),
            ..Default::default()
        },
    ]));

    let visible_ids: Vec<&str> = model
        .chat
        .threads()
        .iter()
        .map(|thread| thread.id.as_str())
        .collect();
    assert_eq!(visible_ids, vec!["thread-user"]);
}

#[test]
fn thread_list_requests_detail_for_selected_thread_with_only_summary_data() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;

    model.chat.reduce(chat::ChatAction::ThreadListReceived(vec![
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            ..Default::default()
        },
    ]));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.handle_client_event(ClientEvent::ThreadList(vec![crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        ..Default::default()
    }]));

    assert_eq!(model.thread_loading_id.as_deref(), Some("thread-user"));
    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThread {
            thread_id,
            message_limit,
            message_offset,
        }) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(0));
        }
        other => panic!("expected thread detail request, got {other:?}"),
    }
}

#[test]
fn thread_detail_clears_loading_state() {
    let mut model = make_model();
    model.thread_loading_id = Some("thread-user".to_string());

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::Assistant,
            content: "Loaded".to_string(),
            timestamp: 1,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    assert!(model.thread_loading_id.is_none());
}

#[test]
fn workspace_task_update_does_not_reopen_loading_after_empty_thread_detail_arrives() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 77;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "workspace-thread:task-1".to_string(),
            title: "Workspace Task".to_string(),
            messages: Vec::new(),
            ..Default::default()
        },
    ));
    model.chat.reduce(chat::ChatAction::SelectThread(
        "workspace-thread:task-1".to_string(),
    ));
    model.thread_loading_id = Some("workspace-thread:task-1".to_string());

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "workspace-thread:task-1".to_string(),
        title: "Workspace Task".to_string(),
        messages: Vec::new(),
        created_at: 1,
        updated_at: 1,
        total_message_count: 0,
        loaded_message_start: 0,
        loaded_message_end: 0,
        ..Default::default()
    })));
    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::WorkspaceTaskUpdated(
        zorai_protocol::WorkspaceTask {
            id: "task-1".to_string(),
            workspace_id: "main".to_string(),
            title: "Workspace Task".to_string(),
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            description: "Description".to_string(),
            definition_of_done: None,
            priority: zorai_protocol::WorkspacePriority::Low,
            status: zorai_protocol::WorkspaceTaskStatus::InProgress,
            sort_order: 1,
            reporter: zorai_protocol::WorkspaceActor::User,
            assignee: Some(zorai_protocol::WorkspaceActor::Agent(
                zorai_protocol::AGENT_ID_SWAROG.to_string(),
            )),
            reviewer: Some(zorai_protocol::WorkspaceActor::User),
            thread_id: Some("workspace-thread:task-1".to_string()),
            goal_run_id: None,
            runtime_history: Vec::new(),
            created_at: 1,
            updated_at: 2,
            started_at: Some(2),
            completed_at: None,
            deleted_at: None,
            last_notice_id: None,
        },
    ));

    assert_eq!(next_thread_request(&mut daemon_rx), None);
    assert!(
        model.thread_loading_id.is_none(),
        "workspace sync after an empty detail should not put the open thread back into loading"
    );
}

#[test]
fn missing_thread_detail_clears_active_loading_state_and_refreshes() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "workspace-thread:task-1".to_string(),
            title: "Workspace Task".to_string(),
            messages: Vec::new(),
            ..Default::default()
        },
    ));
    model.chat.reduce(chat::ChatAction::SelectThread(
        "workspace-thread:task-1".to_string(),
    ));
    model.thread_loading_id = Some("workspace-thread:task-1".to_string());

    model.handle_client_event(ClientEvent::ThreadDetail(None));

    assert!(model.thread_loading_id.is_none());
    assert!(
        std::iter::from_fn(|| daemon_rx.try_recv().ok())
            .any(|command| matches!(command, DaemonCommand::Refresh)),
        "a missing active thread should trigger a daemon refresh"
    );

    model.handle_client_event(ClientEvent::ThreadList(Vec::new()));
    assert_eq!(
        model.chat.active_thread_id(),
        Some("workspace-thread:task-1"),
        "refreshes before hydration should not drop the workspace runtime placeholder"
    );
}

#[test]
fn workspace_task_update_retries_empty_active_runtime_thread() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 77;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "workspace-thread:task-1".to_string(),
            title: "Workspace Task".to_string(),
            messages: Vec::new(),
            ..Default::default()
        },
    ));
    model.chat.reduce(chat::ChatAction::SelectThread(
        "workspace-thread:task-1".to_string(),
    ));

    model.handle_client_event(ClientEvent::WorkspaceTaskUpdated(
        zorai_protocol::WorkspaceTask {
            id: "task-1".to_string(),
            workspace_id: "main".to_string(),
            title: "Workspace Task".to_string(),
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            description: "Description".to_string(),
            definition_of_done: None,
            priority: zorai_protocol::WorkspacePriority::Low,
            status: zorai_protocol::WorkspaceTaskStatus::InProgress,
            sort_order: 1,
            reporter: zorai_protocol::WorkspaceActor::User,
            assignee: Some(zorai_protocol::WorkspaceActor::Agent(
                zorai_protocol::AGENT_ID_SWAROG.to_string(),
            )),
            reviewer: Some(zorai_protocol::WorkspaceActor::User),
            thread_id: Some("workspace-thread:task-1".to_string()),
            goal_run_id: None,
            runtime_history: Vec::new(),
            created_at: 1,
            updated_at: 2,
            started_at: Some(2),
            completed_at: None,
            deleted_at: None,
            last_notice_id: None,
        },
    ));

    assert_eq!(
        next_thread_request(&mut daemon_rx),
        Some(("workspace-thread:task-1".to_string(), Some(77), Some(0)))
    );
    assert_eq!(
        model.thread_loading_id.as_deref(),
        Some("workspace-thread:task-1")
    );
}

#[test]
fn workspace_task_update_does_not_retry_runtime_thread_after_missing_detail() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 77;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "workspace-thread:task-1".to_string(),
            title: "Workspace Task".to_string(),
            messages: Vec::new(),
            ..Default::default()
        },
    ));
    model.chat.reduce(chat::ChatAction::SelectThread(
        "workspace-thread:task-1".to_string(),
    ));
    model.thread_loading_id = Some("workspace-thread:task-1".to_string());

    model.handle_client_event(ClientEvent::ThreadDetail(None));
    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::WorkspaceTaskUpdated(
        zorai_protocol::WorkspaceTask {
            id: "task-1".to_string(),
            workspace_id: "main".to_string(),
            title: "Workspace Task".to_string(),
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            description: "Description".to_string(),
            definition_of_done: None,
            priority: zorai_protocol::WorkspacePriority::Low,
            status: zorai_protocol::WorkspaceTaskStatus::InProgress,
            sort_order: 1,
            reporter: zorai_protocol::WorkspaceActor::User,
            assignee: Some(zorai_protocol::WorkspaceActor::Agent(
                zorai_protocol::AGENT_ID_SWAROG.to_string(),
            )),
            reviewer: Some(zorai_protocol::WorkspaceActor::User),
            thread_id: Some("workspace-thread:task-1".to_string()),
            goal_run_id: None,
            runtime_history: Vec::new(),
            created_at: 1,
            updated_at: 2,
            started_at: Some(2),
            completed_at: None,
            deleted_at: None,
            last_notice_id: None,
        },
    ));

    assert_eq!(next_thread_request(&mut daemon_rx), None);
    assert!(
        model.thread_loading_id.is_none(),
        "missing workspace runtime thread should not be put back into loading"
    );
}

#[test]
fn created_runtime_thread_after_missing_detail_retries_active_workspace_thread() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 77;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "workspace-thread:task-1".to_string(),
            title: "Workspace Task".to_string(),
            messages: Vec::new(),
            ..Default::default()
        },
    ));
    model.chat.reduce(chat::ChatAction::SelectThread(
        "workspace-thread:task-1".to_string(),
    ));
    model.thread_loading_id = Some("workspace-thread:task-1".to_string());

    model.handle_client_event(ClientEvent::ThreadDetail(None));
    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "workspace-thread:task-1".to_string(),
        title: "Workspace Task".to_string(),
        agent_name: Some("Svarog".to_string()),
    });

    assert_eq!(
        next_thread_request(&mut daemon_rx),
        Some(("workspace-thread:task-1".to_string(), Some(77), Some(0)))
    );
    assert_eq!(
        model.thread_loading_id.as_deref(),
        Some("workspace-thread:task-1")
    );
}

#[test]
fn on_tick_requests_next_older_thread_page_when_scrolled_to_top_of_loaded_window() {
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

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThread {
            thread_id,
            message_limit,
            message_offset,
        }) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(100));
        }
        other => panic!("expected older-page request, got {other:?}"),
    }
}

#[test]
fn on_tick_requests_older_thread_page_after_scroll_up_when_loaded_window_fits_viewport() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 12,
            loaded_message_start: 10,
            loaded_message_end: 12,
            messages: (10..12)
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
    model.chat.reduce(chat::ChatAction::ScrollChat(3));

    model.on_tick();

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(2));
        }
        other => panic!("expected older-page request, got {other:?}"),
    }
}

#[test]
fn on_tick_does_not_refill_underfilled_active_thread_window_without_scroll() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 20;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 80,
            loaded_message_start: 58,
            loaded_message_end: 80,
            messages: (58..80)
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
    model.chat.delete_active_message(0);

    model.on_tick();

    assert!(
        next_thread_request(&mut daemon_rx).is_none(),
        "delete backfill should be threshold-driven instead of running an underfilled-window fetch loop"
    );
}

#[test]
fn pending_older_thread_fetch_retries_after_cooldown_without_response() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 20;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 120,
            loaded_message_start: 100,
            loaded_message_end: 120,
            messages: (100..120)
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
    assert!(
        next_thread_request(&mut daemon_rx).is_some(),
        "first top-edge tick should request older messages"
    );

    for _ in 0..chat::CHAT_HISTORY_FETCH_DEBOUNCE_TICKS {
        model.on_tick();
    }
    model.on_tick();

    assert!(
        next_thread_request(&mut daemon_rx).is_some(),
        "pending older fetch should retry after cooldown if no detail arrives"
    );
}

#[test]
fn on_tick_requests_older_thread_page_when_streaming_reasoning_extends_below_loaded_window() {
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
    model.chat.reduce(chat::ChatAction::Reasoning {
        thread_id: "thread-user".to_string(),
        content: (0..20)
            .map(|index| format!("thinking line {index}"))
            .collect::<Vec<_>>()
            .join("\n"),
    });

    model.on_tick();

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThread {
            thread_id,
            message_limit,
            message_offset,
        }) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(100));
        }
        other => panic!("expected older-page request, got {other:?}"),
    }
}

#[test]
fn mouse_scroll_up_on_streaming_activity_row_requests_older_thread_page() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.agent_config_loaded = true;
    model.config.tui_chat_history_page_size = 123;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            total_message_count: 12,
            loaded_message_start: 10,
            loaded_message_end: 12,
            messages: (10..12)
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
    model.set_active_thread_activity("thinking");
    let activity_row = model.pane_layout().concierge.y.saturating_add(1);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: 4,
        row: activity_row,
        modifiers: KeyModifiers::NONE,
    });
    model.on_tick();

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(2));
        }
        other => panic!("expected older-page request, got {other:?}"),
    }
}

#[test]
fn on_tick_refreshes_spawned_sidebar_tasks_on_cooldown() {
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

    model.on_tick();
    assert!(
        saw_list_tasks_command(&mut daemon_rx),
        "spawned sidebar should refresh as soon as the cooldown is eligible"
    );

    for _ in 0..19 {
        model.on_tick();
    }

    assert!(
        !saw_list_tasks_command(&mut daemon_rx),
        "spawned sidebar should not refresh again before the cooldown elapses"
    );

    model.on_tick();
    assert!(
        saw_list_tasks_command(&mut daemon_rx),
        "spawned sidebar should request another task refresh once the cooldown elapses"
    );
}
