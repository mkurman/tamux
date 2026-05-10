use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn new_subagent_conversation_keeps_header_after_thread_created_without_agent_name() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "domowoj".to_string(),
        name: "Domowoj".to_string(),
        provider: "openai".to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("testing".to_string()),
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

    model.start_new_thread_view_for_agent(Some("domowoj"));
    model.submit_prompt("inspect this".to_string());

    let optimistic = model.current_header_agent_profile();
    assert_eq!(optimistic.agent_label, "Domowoj");

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-domowoj".to_string(),
        title: "inspect this".to_string(),
        agent_name: None,
    });

    let after_created = model.current_header_agent_profile();
    assert_eq!(after_created.agent_label, "Domowoj");
    assert_eq!(after_created.provider, "openai");
    assert_eq!(after_created.model, "gpt-5.4-mini");
}

#[test]
fn new_subagent_thread_view_ignores_background_parent_stream_delta() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-parent".to_string(),
        title: "Parent".to_string(),
    });

    model.start_new_thread_view_for_agent(Some("domowoj"));
    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-parent".to_string(),
        content: "background output".to_string(),
    });

    assert_eq!(model.chat.active_thread_id(), None);
    assert_eq!(model.chat.streaming_content(), "");
}

#[test]
fn new_subagent_conversation_done_clears_footer_activity_after_thread_creation() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "domowoj".to_string(),
        name: "Domowoj".to_string(),
        provider: "openai".to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("testing".to_string()),
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

    model.start_new_thread_view_for_agent(Some("domowoj"));
    model.submit_prompt("inspect this".to_string());
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-domowoj".to_string(),
        title: "inspect this".to_string(),
        agent_name: Some("Domowoj".to_string()),
    });
    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-domowoj".to_string(),
        content: "done".to_string(),
    });
    assert_eq!(model.footer_activity_text().as_deref(), Some("writing"));

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-domowoj".to_string(),
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

    assert!(
        model.footer_activity_text().is_none(),
        "completed subagent reply should clear footer activity"
    );
    assert!(
        !model.assistant_busy(),
        "completed subagent reply should not leave the thread busy"
    );
}

#[test]
fn new_subagent_conversation_keeps_thinking_after_thread_created_until_first_response() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "domowoj".to_string(),
        name: "Domowoj".to_string(),
        provider: "openai".to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("testing".to_string()),
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

    model.start_new_thread_view_for_agent(Some("domowoj"));
    model.submit_prompt("inspect this".to_string());
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-domowoj".to_string(),
        title: "inspect this".to_string(),
        agent_name: Some("Domowoj".to_string()),
    });

    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "thread creation should preserve the pending footer activity until output starts"
    );

    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-domowoj".to_string(),
        content: "done".to_string(),
    });
    assert_eq!(model.footer_activity_text().as_deref(), Some("writing"));
}

#[test]
fn new_subagent_conversation_keeps_thinking_across_reload_before_first_response() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "domowoj".to_string(),
        name: "Domowoj".to_string(),
        provider: "openai".to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("testing".to_string()),
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

    model.start_new_thread_view_for_agent(Some("domowoj"));
    model.submit_prompt("inspect this".to_string());
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-domowoj".to_string(),
        title: "inspect this".to_string(),
        agent_name: Some("Domowoj".to_string()),
    });
    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-domowoj".to_string(),
    });

    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "reload before first response should not clear pending thinking state"
    );
}

#[test]
fn new_subagent_conversation_keeps_reasoning_stream_across_reload_before_first_response() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "domowoj".to_string(),
        name: "Domowoj".to_string(),
        provider: "openai".to_string(),
        model: "gpt-5.4-mini".to_string(),
        role: Some("testing".to_string()),
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

    model.start_new_thread_view_for_agent(Some("domowoj"));
    model.submit_prompt("inspect this".to_string());
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-domowoj".to_string(),
        title: "inspect this".to_string(),
        agent_name: Some("Domowoj".to_string()),
    });
    model.handle_client_event(ClientEvent::Reasoning {
        thread_id: "thread-domowoj".to_string(),
        content: "checking the workspace".to_string(),
    });
    assert_eq!(model.chat.streaming_reasoning(), "checking the workspace");
    assert_eq!(model.footer_activity_text().as_deref(), Some("reasoning"));

    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-domowoj".to_string(),
    });

    assert_eq!(
        model.chat.streaming_reasoning(),
        "checking the workspace",
        "reload should not clear live reasoning before answer text starts"
    );
    assert_eq!(model.footer_activity_text().as_deref(), Some("reasoning"));
}

#[test]
fn new_thread_generic_workflow_notice_does_not_break_thinking_preservation() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;

    model.start_new_thread_view();
    model.submit_prompt("do you have generate image now?".to_string());
    assert_eq!(model.footer_activity_text().as_deref(), Some("thinking"));

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-main".to_string(),
        title: "do you have generate image now?".to_string(),
        agent_name: None,
    });
    model.handle_client_event(ClientEvent::WorkflowNotice {
        thread_id: Some("thread-main".to_string()),
        kind: "transport-fallback".to_string(),
        message: "provider switched transport".to_string(),
        details: Some(r#"{"to":"responses"}"#.to_string()),
    });
    model.handle_client_event(ClientEvent::ThreadReloadRequired {
        thread_id: "thread-main".to_string(),
    });

    assert_eq!(
        model.footer_activity_text().as_deref(),
        Some("thinking"),
        "non-activity workflow notices should not let reload clear thinking before output"
    );
}

#[test]
fn thread_detail_prunes_stale_participant_prompts_after_daemon_removes_suggestion() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.handle_client_event(ClientEvent::ParticipantSuggestion {
        thread_id: "thread-1".to_string(),
        suggestion: crate::wire::ThreadParticipantSuggestion {
            id: "sugg-1".to_string(),
            target_agent_id: "weles".to_string(),
            target_agent_name: "Weles".to_string(),
            instruction: "check claim".to_string(),
            suggestion_kind: "prepared_message".to_string(),
            force_send: false,
            status: "queued".to_string(),
            created_at: 1,
            updated_at: 1,
            auto_send_at: None,
            source_message_timestamp: None,
            error: None,
        },
    });
    assert_eq!(model.queued_prompts.len(), 1);

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        messages: vec![],
        queued_participant_suggestions: vec![],
        created_at: 1,
        updated_at: 2,
        ..Default::default()
    });

    assert!(
        model.queued_prompts.is_empty(),
        "thread detail should clear stale participant prompts once the daemon no longer reports them"
    );
}

#[test]
fn queued_participant_send_now_stops_stream_and_sends_participant_command() {
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
        content: "streaming".to_string(),
    });
    model.queue_participant_suggestion(
        "thread-1".to_string(),
        "sugg-1".to_string(),
        "weles".to_string(),
        "Weles".to_string(),
        "urgent fix".to_string(),
        true,
    );
    model.open_queued_prompts_modal();

    model.execute_selected_queued_prompt_action();

    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::StopStream { thread_id }) if thread_id == "thread-1"
    ));
    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::SendParticipantSuggestion { thread_id, suggestion_id })
            if thread_id == "thread-1" && suggestion_id == "sugg-1"
    ));
}

#[test]
fn follow_up_prompt_after_cancel_keeps_processing_new_events_on_same_thread() {
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
    model.cancelled_thread_id = Some("thread-1".to_string());
    model.chat.reduce(chat::ChatAction::ForceStopStreaming);

    model.submit_prompt("follow up".to_string());

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::SendMessage {
            thread_id, content, ..
        }) => {
            assert_eq!(thread_id.as_deref(), Some("thread-1"));
            assert_eq!(content, "follow up");
        }
        other => panic!("expected follow-up send on same thread, got {:?}", other),
    }

    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-1".to_string(),
        content: "Visible answer".to_string(),
    });

    assert_eq!(
        model.chat.streaming_content(),
        "Visible answer",
        "new stream chunks on the same thread should not be dropped after a cancelled turn"
    );
}

#[test]
fn leading_internal_delegate_prompt_routes_to_internal_command() {
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

    model.submit_prompt("!weles verify the auth regression".to_string());

    match daemon_rx.try_recv() {
        Ok(DaemonCommand::InternalDelegate {
            thread_id,
            target_agent_id,
            content,
            ..
        }) => {
            assert_eq!(thread_id.as_deref(), Some("thread-1"));
            assert_eq!(target_agent_id, "weles");
            assert_eq!(content, "verify the auth regression");
        }
        other => panic!("expected internal delegate command, got {:?}", other),
    }
    assert!(
        model
            .chat
            .active_thread()
            .expect("thread should remain selected")
            .messages
            .is_empty(),
        "internal delegation should not append a visible user turn"
    );
}
