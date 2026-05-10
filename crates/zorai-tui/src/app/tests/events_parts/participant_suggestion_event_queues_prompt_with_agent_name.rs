use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn participant_suggestion_event_queues_prompt_with_agent_name() {
    let mut model = make_model();

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
    assert_eq!(
        model.queued_prompts[0].participant_agent_name.as_deref(),
        Some("Weles")
    );
    assert_eq!(model.queued_prompts[0].display_text(), "Weles: check claim");
}

#[test]
fn auto_response_participant_suggestion_does_not_enter_generic_queued_prompt_modal() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ParticipantSuggestion {
        thread_id: "thread-1".to_string(),
        suggestion: crate::wire::ThreadParticipantSuggestion {
            id: "auto-1".to_string(),
            target_agent_id: "domowoj".to_string(),
            target_agent_name: "Domowoj".to_string(),
            instruction: "Respond to the latest main agent message.".to_string(),
            suggestion_kind: "auto_response".to_string(),
            force_send: false,
            status: "queued".to_string(),
            created_at: 1,
            updated_at: 1,
            auto_send_at: Some(61_000),
            source_message_timestamp: Some(55_000),
            error: None,
        },
    });

    assert!(
        model.queued_prompts.is_empty(),
        "auto-response suggestions should render in the participant banner instead of the generic queued modal"
    );
}

#[test]
fn due_auto_response_on_reopened_thread_dispatches_on_tick() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            id: Some("msg-1".to_string()),
            role: crate::wire::MessageRole::Assistant,
            content: "Main agent reply".to_string(),
            timestamp: 55_000,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "domowoj".to_string(),
            agent_name: "Domowoj".to_string(),
            instruction: "push the work forward".to_string(),
            status: "active".to_string(),
            created_at: 1,
            updated_at: 2,
            last_contribution_at: Some(10),
            deactivated_at: None,
            always_auto_response: false,
        }],
        queued_participant_suggestions: vec![crate::wire::ThreadParticipantSuggestion {
            id: "auto-1".to_string(),
            target_agent_id: "domowoj".to_string(),
            target_agent_name: "Domowoj".to_string(),
            instruction: "Respond to the latest main agent message.".to_string(),
            suggestion_kind: "auto_response".to_string(),
            force_send: false,
            status: "queued".to_string(),
            created_at: 1,
            updated_at: 1,
            auto_send_at: Some(0),
            source_message_timestamp: Some(55_000),
            error: None,
        }],
        ..Default::default()
    })));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    while daemon_rx.try_recv().is_ok() {}

    model.on_tick();

    let mut saw_send = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(
            command,
            DaemonCommand::SendParticipantSuggestion { thread_id, suggestion_id }
                if thread_id == "thread-1" && suggestion_id == "auto-1"
        ) {
            saw_send = true;
            break;
        }
    }
    assert!(saw_send, "expected due auto-response send command on tick");
    assert!(
        model.queued_prompts.is_empty(),
        "tick-driven auto response should bypass the generic queued prompt list"
    );
}

#[test]
fn always_auto_response_participant_suggestion_sends_immediately_on_active_thread() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "domowoj".to_string(),
            agent_name: "Domowoj".to_string(),
            instruction: "push the work forward".to_string(),
            status: "active".to_string(),
            created_at: 1,
            updated_at: 2,
            last_contribution_at: Some(10),
            deactivated_at: None,
            always_auto_response: true,
        }],
        ..Default::default()
    })));
    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::ParticipantSuggestion {
        thread_id: "thread-1".to_string(),
        suggestion: crate::wire::ThreadParticipantSuggestion {
            id: "auto-1".to_string(),
            target_agent_id: "domowoj".to_string(),
            target_agent_name: "Domowoj".to_string(),
            instruction: "Respond to the latest main agent message.".to_string(),
            suggestion_kind: "auto_response".to_string(),
            force_send: false,
            status: "queued".to_string(),
            created_at: 1,
            updated_at: 1,
            auto_send_at: Some(61_000),
            source_message_timestamp: Some(55_000),
            error: None,
        },
    });

    assert!(matches!(
        daemon_rx.try_recv(),
        Ok(DaemonCommand::SendParticipantSuggestion { thread_id, suggestion_id })
            if thread_id == "thread-1" && suggestion_id == "auto-1"
    ));
}

#[test]
fn reopened_thread_with_always_auto_response_dispatches_queued_suggestion_immediately() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            id: Some("msg-1".to_string()),
            role: crate::wire::MessageRole::Assistant,
            content: "Main agent reply".to_string(),
            timestamp: 55_000,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "domowoj".to_string(),
            agent_name: "Domowoj".to_string(),
            instruction: "push the work forward".to_string(),
            status: "active".to_string(),
            created_at: 1,
            updated_at: 2,
            last_contribution_at: Some(10),
            deactivated_at: None,
            always_auto_response: true,
        }],
        queued_participant_suggestions: vec![crate::wire::ThreadParticipantSuggestion {
            id: "auto-1".to_string(),
            target_agent_id: "domowoj".to_string(),
            target_agent_name: "Domowoj".to_string(),
            instruction: "Respond to the latest main agent message.".to_string(),
            suggestion_kind: "auto_response".to_string(),
            force_send: false,
            status: "queued".to_string(),
            created_at: 1,
            updated_at: 1,
            auto_send_at: Some(61_000),
            source_message_timestamp: Some(55_000),
            error: None,
        }],
        ..Default::default()
    })));

    let mut saw_send = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(
            command,
            DaemonCommand::SendParticipantSuggestion { thread_id, suggestion_id }
                if thread_id == "thread-1" && suggestion_id == "auto-1"
        ) {
            saw_send = true;
            break;
        }
    }
    assert!(
        saw_send,
        "reopening a thread with an always auto-response participant should send the queued suggestion immediately"
    );
}

#[test]
fn active_thread_detail_requests_auto_response_for_latest_main_reply() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                id: Some("msg-1".to_string()),
                role: crate::wire::MessageRole::User,
                content: "Keep going".to_string(),
                timestamp: 1,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                id: Some("msg-2".to_string()),
                role: crate::wire::MessageRole::Assistant,
                content: "I finished the patch; the next step is verifying the diff.".to_string(),
                timestamp: 2,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        thread_participants: vec![
            crate::wire::ThreadParticipantState {
                agent_id: "weles".to_string(),
                agent_name: "Weles".to_string(),
                instruction: "verify claims".to_string(),
                status: "active".to_string(),
                created_at: 1,
                updated_at: 2,
                last_contribution_at: Some(10),
                deactivated_at: None,
                always_auto_response: false,
            },
            crate::wire::ThreadParticipantState {
                agent_id: "domowoj".to_string(),
                agent_name: "Domowoj".to_string(),
                instruction: "push the work forward".to_string(),
                status: "active".to_string(),
                created_at: 1,
                updated_at: 3,
                last_contribution_at: Some(20),
                deactivated_at: None,
                always_auto_response: false,
            },
        ],
        ..Default::default()
    })));

    let mut saw_auto_response = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(
            command,
            DaemonCommand::ThreadParticipantCommand {
                thread_id,
                target_agent_id,
                action,
                instruction,
                ..
            } if thread_id == "thread-1"
                && target_agent_id == "domowoj"
                && action == "auto_response"
                && instruction.is_none()
        ) {
            saw_auto_response = true;
            break;
        }
    }

    assert!(
        saw_auto_response,
        "opening the active thread should request an auto-response for the most active participant"
    );
}

#[test]
fn done_event_on_active_thread_requests_auto_response_for_live_main_reply() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            id: Some("msg-1".to_string()),
            role: crate::wire::MessageRole::User,
            content: "Keep going".to_string(),
            timestamp: 1,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        thread_participants: vec![
            crate::wire::ThreadParticipantState {
                agent_id: "weles".to_string(),
                agent_name: "Weles".to_string(),
                instruction: "verify claims".to_string(),
                status: "active".to_string(),
                created_at: 1,
                updated_at: 2,
                last_contribution_at: Some(10),
                deactivated_at: None,
                always_auto_response: false,
            },
            crate::wire::ThreadParticipantState {
                agent_id: "domowoj".to_string(),
                agent_name: "Domowoj".to_string(),
                instruction: "push the work forward".to_string(),
                status: "active".to_string(),
                created_at: 1,
                updated_at: 3,
                last_contribution_at: Some(20),
                deactivated_at: None,
                always_auto_response: false,
            },
        ],
        ..Default::default()
    })));
    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-1".to_string(),
        content: "I finished the patch; the next step is verifying the diff.".to_string(),
    });
    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-1".to_string(),
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

    let mut saw_auto_response = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(
            command,
            DaemonCommand::ThreadParticipantCommand {
                thread_id,
                target_agent_id,
                action,
                instruction,
                ..
            } if thread_id == "thread-1"
                && target_agent_id == "domowoj"
                && action == "auto_response"
                && instruction.is_none()
        ) {
            saw_auto_response = true;
            break;
        }
    }

    assert!(
        saw_auto_response,
        "finishing a live main-agent reply on the open thread should request auto-response immediately"
    );
}

#[test]
fn participant_suggestion_does_not_auto_flush_as_user_message_after_done() {
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

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-1".to_string(),
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
        daemon_rx.try_recv().is_err(),
        "participant suggestions must not auto-submit through the normal send-message path"
    );
    assert_eq!(model.queued_prompts.len(), 1);
    assert_eq!(
        model.queued_prompts[0].suggestion_id.as_deref(),
        Some("sugg-1")
    );
}
