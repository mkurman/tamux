use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn active_goal_run_file_tab_restores_selected_file_after_partial_refresh() {
    let mut model = active_goal_run_sidebar_model();
    model.activate_goal_sidebar_tab(GoalSidebarTab::Files);
    model.select_goal_sidebar_row(1);

    model.handle_work_context_event(crate::wire::ThreadWorkContext {
        thread_id: "thread-1".to_string(),
        entries: vec![crate::wire::WorkContextEntry {
            path: "/tmp/plan.md".to_string(),
            goal_run_id: Some("goal-1".to_string()),
            is_text: true,
            ..Default::default()
        }],
    });
    assert_eq!(model.goal_sidebar.selected_row(), 0);

    model.handle_work_context_event(crate::wire::ThreadWorkContext {
        thread_id: "thread-1".to_string(),
        entries: vec![
            crate::wire::WorkContextEntry {
                path: "/tmp/plan.md".to_string(),
                goal_run_id: Some("goal-1".to_string()),
                is_text: true,
                ..Default::default()
            },
            crate::wire::WorkContextEntry {
                path: "/tmp/report.md".to_string(),
                goal_run_id: Some("goal-1".to_string()),
                is_text: true,
                ..Default::default()
            },
        ],
    });

    assert_eq!(
        model.goal_sidebar.selected_row(),
        1,
        "file-tab selection should restore the same path when a transient partial refresh drops it temporarily"
    );
}

#[test]
fn sidebar_enter_on_offscreen_pinned_message_requests_containing_page() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 100;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: (200..300)
            .map(|index| crate::wire::AgentMessage {
                id: Some(format!("msg-{index}")),
                role: crate::wire::MessageRole::Assistant,
                content: format!("msg {index}"),
                timestamp: index as u64,
                message_kind: "normal".to_string(),
                ..Default::default()
            })
            .collect(),
        pinned_messages: vec![crate::wire::PinnedThreadMessage {
            message_id: "msg-50".to_string(),
            absolute_index: 50,
            role: crate::wire::MessageRole::User,
            content: "Pinned offscreen".to_string(),
        }],
        total_message_count: 300,
        loaded_message_start: 200,
        loaded_message_end: 300,
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));
    model.focus = FocusArea::Sidebar;
    model
        .sidebar
        .reduce(crate::state::sidebar::SidebarAction::SwitchTab(
            crate::state::sidebar::SidebarTab::Pinned,
        ));

    model.handle_sidebar_enter();

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(100));
            assert_eq!(message_offset, Some(200));
        }
        other => panic!("expected containing-page request, got {other:?}"),
    }
}

#[test]
fn thread_detail_preserves_message_author_metadata() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::Assistant,
            content: "Visible participant post.".to_string(),
            author_agent_id: Some("weles".to_string()),
            author_agent_name: Some("Weles".to_string()),
            timestamp: 1,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    let thread = model
        .chat
        .threads()
        .iter()
        .find(|thread| thread.id == "thread-user")
        .expect("thread detail should populate chat state");
    let message = thread
        .messages
        .first()
        .expect("thread should contain message");
    assert_eq!(message.author_agent_id.as_deref(), Some("weles"));
    assert_eq!(message.author_agent_name.as_deref(), Some("Weles"));
}

#[test]
fn internal_dm_threads_are_retained_in_thread_list_for_internal_picker() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadList(vec![
        crate::wire::AgentThread {
            id: "thread-user".to_string(),
            title: "User Thread".to_string(),
            ..Default::default()
        },
        crate::wire::AgentThread {
            id: "dm:svarog:weles".to_string(),
            title: "Internal DM · Svarog ↔ WELES".to_string(),
            ..Default::default()
        },
    ]));

    let visible_ids: Vec<&str> = model
        .chat
        .threads()
        .iter()
        .map(|thread| thread.id.as_str())
        .collect();
    assert_eq!(visible_ids, vec!["thread-user", "dm:svarog:weles"]);
}

#[test]
fn hidden_handoff_thread_reload_required_is_ignored() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "handoff:thread-user:handoff-1".to_string(),
    });

    assert!(daemon_rx.try_recv().is_err());
}

#[test]
fn active_thread_reload_required_requests_detail_and_sidebar_context() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-user".to_string(),
        title: "User Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-user".to_string(),
    });

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
    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThreadTodos(thread_id)) => {
            assert_eq!(thread_id, "thread-user");
        }
        other => panic!("expected todos request, got {other:?}"),
    }
    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThreadWorkContext(thread_id)) => {
            assert_eq!(thread_id, "thread-user");
        }
        other => panic!("expected work-context request, got {other:?}"),
    }
}

#[test]
fn active_thread_reload_required_invalidates_stale_header_context_tokens() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-compacted".to_string(),
        title: "Compacted".to_string(),
        total_message_count: 2,
        loaded_message_start: 0,
        loaded_message_end: 2,
        active_context_window_start: Some(0),
        active_context_window_end: Some(2),
        active_context_window_tokens: Some(239_700_000),
        messages: vec![
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: "summary".to_string(),
                message_kind: "compaction_artifact".to_string(),
                compaction_payload: Some("P".repeat(40)),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::User,
                content: "C".repeat(80),
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-compacted".to_string(),
    ));
    while daemon_rx.try_recv().is_ok() {}

    assert_eq!(
        model.current_header_usage_summary().current_tokens,
        239_700_000
    );

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-compacted".to_string(),
    });

    assert_eq!(
        model.current_header_usage_summary().current_tokens,
        0,
        "reload should clear stale daemon context tokens while the authoritative detail is pending"
    );
    assert!(matches!(
        next_thread_request(&mut daemon_rx),
        Some((thread_id, _, _)) if thread_id == "thread-compacted"
    ));
}

#[test]
fn goal_header_thread_reload_required_refreshes_header_thread_even_when_not_selected() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "selected-thread".to_string(),
        title: "Selected".to_string(),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "selected-thread".to_string(),
    ));
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "goal-thread".to_string(),
        title: "Goal Thread".to_string(),
        total_message_count: 2,
        loaded_message_start: 0,
        loaded_message_end: 2,
        active_context_window_start: Some(0),
        active_context_window_end: Some(2),
        active_context_window_tokens: Some(239_700_000),
        messages: vec![
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: "summary".to_string(),
                message_kind: "compaction_artifact".to_string(),
                compaction_payload: Some("P".repeat(40)),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::User,
                content: "C".repeat(80),
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-reload".to_string(),
            title: "Goal".to_string(),
            active_thread_id: Some("goal-thread".to_string()),
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-reload".to_string(),
        step_id: None,
    });
    while daemon_rx.try_recv().is_ok() {}

    assert_eq!(
        model.current_header_usage_summary().current_tokens,
        239_700_000
    );

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "goal-thread".to_string(),
    });

    assert_eq!(model.chat.active_thread_id(), Some("selected-thread"));
    assert_eq!(model.current_header_usage_summary().current_tokens, 0);
    assert!(matches!(
        next_thread_request(&mut daemon_rx),
        Some((thread_id, _, _)) if thread_id == "goal-thread"
    ));
}

#[test]
fn active_thread_reload_required_reuses_loaded_page_window_when_viewing_older_history() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-user".to_string(),
            title: "Discord Alice".to_string(),
            total_message_count: 400,
            loaded_message_start: 154,
            loaded_message_end: 277,
            messages: (154..277)
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

    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-user".to_string(),
    });

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThread {
            thread_id,
            message_limit,
            message_offset,
        }) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(136));
            assert_eq!(message_offset, Some(123));
        }
        other => panic!("expected current page refresh request, got {other:?}"),
    }
}

#[test]
fn participant_managed_thread_reload_requests_expanded_latest_page() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "weles".to_string(),
            agent_name: "Weles".to_string(),
            instruction: "verify claims".to_string(),
            status: "active".to_string(),
            created_at: 1,
            updated_at: 1,
            last_contribution_at: None,
            deactivated_at: None,
            always_auto_response: false,
        }],
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-user".to_string(),
    });

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::RequestThread {
            thread_id,
            message_limit,
            message_offset,
        }) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(246));
            assert_eq!(message_offset, Some(0));
        }
        other => {
            panic!("expected expanded participant-managed thread detail request, got {other:?}")
        }
    }
}

#[test]
fn inactive_thread_reload_required_does_not_interrupt_selected_thread() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-user".to_string(),
        title: "User Thread".to_string(),
    });
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-other".to_string(),
        title: "Other Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));
    model.handle_client_event(ClientEvent::ToolCall {
        thread_id: "thread-user".to_string(),
        call_id: "user-call".to_string(),
        name: "bash_command".to_string(),
        arguments: "{\"command\":\"pwd\"}".to_string(),
        weles_review: None,
    });

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-other".to_string(),
    });

    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
    assert_eq!(model.chat.active_tool_calls().len(), 1);
    assert!(daemon_rx.try_recv().is_err());
}
