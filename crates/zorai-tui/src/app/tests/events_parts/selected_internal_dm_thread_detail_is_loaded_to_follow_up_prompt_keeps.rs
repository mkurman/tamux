use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn selected_internal_dm_thread_detail_is_loaded() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "dm:svarog:weles".to_string(),
        title: "Internal DM · Svarog ↔ WELES".to_string(),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "dm:svarog:weles".to_string(),
    ));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "dm:svarog:weles".to_string(),
        title: "Internal DM · Svarog ↔ WELES".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::Assistant,
            content: "Keep reviewing the migration plan.".to_string(),
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
        .find(|thread| thread.id == "dm:svarog:weles")
        .expect("selected internal dm thread should remain in chat state");
    assert_eq!(model.chat.active_thread_id(), Some("dm:svarog:weles"));
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(
        thread.messages[0].content,
        "Keep reviewing the migration plan."
    );
}

#[test]
fn selected_internal_dm_thread_detail_with_weles_persona_marker_is_loaded() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "dm:svarog:weles".to_string(),
        title: "Internal DM · Svarog ↔ WELES".to_string(),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "dm:svarog:weles".to_string(),
    ));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "dm:svarog:weles".to_string(),
        title: "Internal DM · Svarog ↔ WELES".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::Assistant,
            content: "Agent persona id: weles\n\nKeep reviewing the migration plan.".to_string(),
            timestamp: 1,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        total_message_count: 150,
        loaded_message_start: 50,
        loaded_message_end: 150,
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    let thread = model
        .chat
        .threads()
        .iter()
        .find(|thread| thread.id == "dm:svarog:weles")
        .expect("selected internal dm thread should remain in chat state");
    assert_eq!(model.chat.active_thread_id(), Some("dm:svarog:weles"));
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.total_message_count, 150);
    assert_eq!(thread.loaded_message_start, 50);
    assert_eq!(thread.loaded_message_end, 150);
}

#[test]
fn internal_dm_tool_activity_does_not_block_normal_thread_completion() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-user".to_string(),
        title: "User Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.handle_client_event(ClientEvent::ToolCall {
        thread_id: "dm:svarog:weles".to_string(),
        call_id: "internal-call".to_string(),
        name: "message_agent".to_string(),
        arguments: "{}".to_string(),
        weles_review: None,
    });
    assert!(
        model.chat.active_tool_calls().is_empty(),
        "internal tool calls should not enter the visible running-tool tracker"
    );
    model.handle_client_event(ClientEvent::ToolCall {
        thread_id: "thread-user".to_string(),
        call_id: "user-call".to_string(),
        name: "bash_command".to_string(),
        arguments: "{\"command\":\"pwd\"}".to_string(),
        weles_review: None,
    });
    assert_eq!(model.chat.active_tool_calls().len(), 1);

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "dm:svarog:weles".to_string(),
        title: "Internal DM · Swarog ↔ WELES".to_string(),
        agent_name: None,
    });
    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-user".to_string(),
        input_tokens: 0,
        output_tokens: 0,
        cost: None,
        provider: None,
        model: None,
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: Some("result_json".to_string()),
    });

    assert!(
        model.chat.active_tool_calls().is_empty(),
        "visible thread completion should still clear running tools"
    );
    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
}

#[test]
fn inactive_thread_events_do_not_replace_selected_thread_activity_badge() {
    let mut model = make_model();
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

    model.handle_client_event(ClientEvent::Reasoning {
        thread_id: "thread-other".to_string(),
        content: "background reasoning".to_string(),
    });
    model.handle_client_event(ClientEvent::ToolCall {
        thread_id: "thread-other".to_string(),
        call_id: "background-call".to_string(),
        name: "bash_command".to_string(),
        arguments: "{\"command\":\"pwd\"}".to_string(),
        weles_review: None,
    });

    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
    assert!(model.footer_activity_text().is_none());
    assert_eq!(model.chat.streaming_content(), "");
    assert_eq!(model.chat.streaming_reasoning(), "");
    assert!(model.chat.active_tool_calls().is_empty());
}

#[test]
fn inactive_thread_workflow_notice_does_not_replace_selected_thread_footer_activity() {
    let mut model = make_model();
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

    model.handle_client_event(ClientEvent::Reasoning {
        thread_id: "thread-user".to_string(),
        content: "active reasoning".to_string(),
    });
    assert_eq!(model.footer_activity_text().as_deref(), Some("reasoning"));

    model.handle_client_event(ClientEvent::WorkflowNotice {
        thread_id: Some("thread-other".to_string()),
        kind: "skill-gate".to_string(),
        message: "background skill gate".to_string(),
        details: Some(r#"{"recommended_skill":"onecontext"}"#.to_string()),
    });

    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("reasoning"),
        "background workflow notices must not replace the selected thread footer activity"
    );
}

#[test]
fn thread_footer_activity_remains_scoped_when_switching_between_busy_threads() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
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

    model.submit_prompt("investigate the auth regression".to_string());
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::WorkflowNotice {
        thread_id: Some("thread-other".to_string()),
        kind: "skill-gate".to_string(),
        message: "background skill gate".to_string(),
        details: Some(r#"{"recommended_skill":"onecontext"}"#.to_string()),
    });

    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "selected thread should keep its own footer activity after background updates"
    );

    model.open_thread_conversation("thread-other".to_string());
    while daemon_rx.try_recv().is_ok() {}
    assert_eq!(model.footer_activity_text().as_deref(), Some("skill gate"));

    model.open_thread_conversation("thread-user".to_string());
    while daemon_rx.try_recv().is_ok() {}
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));
}

#[test]
fn optimistic_new_thread_keeps_thinking_during_stale_thread_list_refresh() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;

    model.submit_prompt("investigate the auth regression".to_string());
    let optimistic_thread_id = model
        .chat
        .active_thread_id()
        .map(str::to_string)
        .expect("submit should create an optimistic thread");
    assert!(optimistic_thread_id.starts_with("local-"));
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadList(vec![]));

    assert_eq!(
        model.chat.active_thread_id(),
        Some(optimistic_thread_id.as_str())
    );
    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "stale thread list refresh should not clear pending thinking on an optimistic thread"
    );
}

#[test]
fn reopened_thread_keeps_thinking_during_stale_thread_list_refresh() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
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

    model.submit_prompt("investigate the auth regression".to_string());
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadList(vec![crate::wire::AgentThread {
        id: "thread-other".to_string(),
        title: "Other Thread".to_string(),
        ..Default::default()
    }]));

    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "stale thread list refresh should not clear pending thinking on a reopened thread"
    );
}

#[test]
fn follow_up_prompt_keeps_thinking_during_stale_thread_list_refresh() {
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
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadList(vec![]));

    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "stale thread list refresh should not clear pending thinking on a follow-up prompt"
    );
}

#[test]
fn follow_up_prompt_keeps_thinking_across_reload_after_stale_thread_detail_replaces_tail() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::User,
                content: "First question".to_string(),
                timestamp: 1,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
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
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-user".to_string(),
        title: "User Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::User,
                content: "First question".to_string(),
                timestamp: 100,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: "First answer".to_string(),
                timestamp: 200,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        total_message_count: 2,
        loaded_message_start: 0,
        loaded_message_end: 2,
        created_at: 1,
        updated_at: 2,
        ..Default::default()
    })));
    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-user".to_string(),
    });

    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "reload should preserve thinking even if a stale thread detail temporarily drops the optimistic prompt tail"
    );
}
