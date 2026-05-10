use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn follow_up_prompt_deduplicates_latest_page_when_wire_start_is_missing() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                id: Some("msg-118".to_string()),
                role: crate::wire::MessageRole::User,
                content: "Earlier question".to_string(),
                timestamp: 118,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                id: Some("msg-119".to_string()),
                role: crate::wire::MessageRole::Assistant,
                content: "Earlier answer".to_string(),
                timestamp: 119,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        total_message_count: 120,
        loaded_message_end: 120,
        ..Default::default()
    })));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.submit_prompt("follow-up question".to_string());
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                id: Some("msg-118".to_string()),
                role: crate::wire::MessageRole::User,
                content: "Earlier question".to_string(),
                timestamp: 118,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                id: Some("msg-119".to_string()),
                role: crate::wire::MessageRole::Assistant,
                content: "Earlier answer".to_string(),
                timestamp: 119,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                id: Some("msg-120".to_string()),
                role: crate::wire::MessageRole::User,
                content: "follow-up question".to_string(),
                timestamp: 120,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        total_message_count: 121,
        loaded_message_end: 121,
        ..Default::default()
    })));

    let thread = model
        .chat
        .active_thread()
        .expect("thread should stay active");
    assert_eq!(
        thread
            .messages
            .iter()
            .filter(|message| message.role == chat::MessageRole::User
                && message.content == "follow-up question")
            .count(),
        1,
        "persisted latest-page echo should replace the optimistic prompt"
    );
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));
}

#[test]
fn follow_up_prompt_keeps_reasoning_stream_across_reload_before_first_response() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                id: Some("msg-user-1".to_string()),
                role: crate::wire::MessageRole::User,
                content: "First question".to_string(),
                timestamp: 1,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                id: Some("msg-assistant-1".to_string()),
                role: crate::wire::MessageRole::Assistant,
                content: "First answer".to_string(),
                timestamp: 2,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        created_at: 1,
        updated_at: 2,
        ..Default::default()
    })));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.submit_prompt("follow-up question".to_string());
    model.handle_client_event(ClientEvent::Reasoning {
        thread_id: "thread-user".to_string(),
        content: "thinking about the follow-up".to_string(),
    });

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-user".to_string(),
    });

    assert_eq!(
        model.chat.streaming_reasoning(),
        "thinking about the follow-up",
        "reload should not clear live reasoning on follow-up prompts"
    );
    assert_eq!(model.footer_activity_text().as_deref(), Some("reasoning"));
}

#[test]
fn participant_playground_activity_surfaces_only_for_active_parent_thread() {
    let mut model = make_model();
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-other".to_string(),
        title: "Other Thread".to_string(),
        agent_name: None,
    });
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        thread_participants: vec![
            crate::wire::ThreadParticipantState {
                agent_id: "domowoj".to_string(),
                agent_name: "Domowoj".to_string(),
                instruction: "Look for gaps".to_string(),
                status: "active".to_string(),
                created_at: 1,
                updated_at: 1,
                last_contribution_at: None,
                deactivated_at: None,
                always_auto_response: false,
            },
            crate::wire::ThreadParticipantState {
                agent_id: "weles".to_string(),
                agent_name: "Weles".to_string(),
                instruction: "Verify risky changes".to_string(),
                status: "active".to_string(),
                created_at: 1,
                updated_at: 1,
                last_contribution_at: None,
                deactivated_at: None,
                always_auto_response: false,
            },
        ],
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.handle_client_event(ClientEvent::Reasoning {
        thread_id: "playground:domowoj:thread-user".to_string(),
        content: "Hidden participant reasoning".to_string(),
    });
    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("Domowoj crafting response")
    );

    model.handle_client_event(ClientEvent::ToolCall {
        thread_id: "playground:weles:thread-user".to_string(),
        call_id: "hidden-call".to_string(),
        name: "bash_command".to_string(),
        arguments: "{\"command\":\"pwd\"}".to_string(),
        weles_review: None,
    });
    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("Domowoj +1 crafting responses")
    );

    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-other".to_string()));
    assert!(
        model.footer_activity_text().is_none(),
        "participant playground activity should stay scoped to the selected visible thread"
    );
}

#[test]
fn participant_playground_done_refreshes_active_visible_thread_and_surfaces_reply() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.config.tui_chat_history_page_size = 123;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            id: Some("msg-1".to_string()),
            role: crate::wire::MessageRole::Assistant,
            content: "Main agent reply".to_string(),
            timestamp: 1,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "domowoj".to_string(),
            agent_name: "Domowoj".to_string(),
            instruction: "Look for weak spots".to_string(),
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

    model.handle_client_event(ClientEvent::Delta {
        thread_id: "playground:domowoj:thread-user".to_string(),
        content: "Drafting visible reply".to_string(),
    });
    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("Domowoj crafting response")
    );

    model.handle_client_event(ClientEvent::Done {
        thread_id: "playground:domowoj:thread-user".to_string(),
        input_tokens: 0,
        output_tokens: 0,
        cost: None,
        provider: None,
        model: None,
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: None,
    });

    match next_thread_request(&mut daemon_rx) {
        Some((thread_id, message_limit, message_offset)) => {
            assert_eq!(thread_id, "thread-user");
            assert_eq!(message_limit, Some(246));
            assert_eq!(message_offset, Some(0));
        }
        other => {
            panic!("expected active visible thread refresh after playground done, got {other:?}")
        }
    }
    assert!(
        model.footer_activity_text().is_none(),
        "playground completion should clear the footer activity line"
    );

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                id: Some("msg-1".to_string()),
                role: crate::wire::MessageRole::Assistant,
                content: "Main agent reply".to_string(),
                timestamp: 1,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                id: Some("msg-2".to_string()),
                role: crate::wire::MessageRole::Assistant,
                content: "Visible participant reply".to_string(),
                author_agent_id: Some("domowoj".to_string()),
                author_agent_name: Some("Domowoj".to_string()),
                timestamp: 2,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "domowoj".to_string(),
            agent_name: "Domowoj".to_string(),
            instruction: "Look for weak spots".to_string(),
            status: "active".to_string(),
            created_at: 1,
            updated_at: 2,
            last_contribution_at: Some(2),
            deactivated_at: None,
            always_auto_response: false,
        }],
        created_at: 1,
        updated_at: 2,
        ..Default::default()
    })));

    let thread = model.chat.active_thread().expect("thread should exist");
    assert!(
        thread.messages.iter().any(|message| {
            message.content == "Visible participant reply"
                && message.author_agent_name.as_deref() == Some("Domowoj")
        }),
        "authoritative refresh should surface participant-authored visible replies"
    );
}

#[test]
fn queued_prompt_flushes_after_last_tool_result_before_turn_done() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.handle_client_event(ClientEvent::ToolCall {
        thread_id: "thread-1".to_string(),
        call_id: "call-1".to_string(),
        name: "bash_command".to_string(),
        arguments: "{\"command\":\"pwd\"}".to_string(),
        weles_review: None,
    });

    model.submit_prompt("stay on the migration task".to_string());
    assert_eq!(model.queued_prompts.len(), 1);
    assert!(daemon_rx.try_recv().is_err());

    model.handle_client_event(ClientEvent::ToolResult {
        thread_id: "thread-1".to_string(),
        call_id: "call-1".to_string(),
        name: "bash_command".to_string(),
        content: "/repo".to_string(),
        is_error: false,
        weles_review: None,
    });

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::SendMessage {
            thread_id, content, ..
        }) => {
            assert_eq!(thread_id.as_deref(), Some("thread-1"));
            assert_eq!(content, "stay on the migration task");
        }
        other => panic!("expected queued send after tool result, got {:?}", other),
    }
    assert!(
        model.queued_prompts.is_empty(),
        "queued prompt should flush as soon as the last tool finishes"
    );
}

#[test]
fn prompt_during_text_stream_without_running_tools_waits_for_done() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-1".to_string(),
        content: "Partial answer".to_string(),
    });
    assert!(
        model.chat.active_tool_calls().is_empty(),
        "plain streaming should not fabricate running tools"
    );

    model.submit_prompt("switch to the auth bug instead".to_string());
    assert_eq!(model.queued_prompts.len(), 1);
    assert!(daemon_rx.try_recv().is_err());

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-1".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: None,
        provider: None,
        model: None,
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: Some("result_json".to_string()),
    });

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::SendMessage {
            thread_id, content, ..
        }) => {
            assert_eq!(thread_id.as_deref(), Some("thread-1"));
            assert_eq!(content, "switch to the auth bug instead");
        }
        other => panic!(
            "expected queued send after done when text is streaming, got {:?}",
            other
        ),
    }
    assert!(
        model.queued_prompts.is_empty(),
        "message should flush once the streaming assistant message completes"
    );
}
