use crate::state::chat::*;
use zorai_shared::providers::*;
#[test]
fn select_thread_with_empty_id_clears_active_selection() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::SelectThread("t1".into()));

    state.reduce(ChatAction::SelectThread(String::new()));

    assert_eq!(state.active_thread_id(), None);
    assert!(state.active_thread().is_none());
}

#[test]
fn tool_call_tracks_running_tool() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "c1".into(),
        name: "bash_command".into(),
        args: "ls".into(),
        weles_review: None,
    });
    assert_eq!(state.active_tool_calls().len(), 1);
    assert_eq!(state.active_tool_calls()[0].status, ToolCallStatus::Running);
}

#[test]
fn tool_result_updates_status() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "c1".into(),
        name: "bash_command".into(),
        args: "ls".into(),
        weles_review: None,
    });
    state.reduce(ChatAction::ToolResult {
        thread_id: "t1".into(),
        call_id: "c1".into(),
        name: "bash_command".into(),
        content: "file.txt".into(),
        is_error: false,
        weles_review: None,
    });
    assert_eq!(state.active_tool_calls()[0].status, ToolCallStatus::Done);
}

#[test]
fn reasoning_and_tool_calls_preserve_transcript_continuity() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.reduce(ChatAction::Reasoning {
        thread_id: "t1".into(),
        content: "First reasoning".into(),
    });
    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "call-1".into(),
        name: "tool_one".into(),
        args: "{}".into(),
        weles_review: None,
    });
    state.reduce(ChatAction::ToolResult {
        thread_id: "t1".into(),
        call_id: "call-1".into(),
        name: "tool_one".into(),
        content: "done".into(),
        is_error: false,
        weles_review: None,
    });
    state.reduce(ChatAction::Reasoning {
        thread_id: "t1".into(),
        content: "Second reasoning".into(),
    });
    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "call-2".into(),
        name: "tool_two".into(),
        args: "{}".into(),
        weles_review: None,
    });
    state.reduce(ChatAction::ToolResult {
        thread_id: "t1".into(),
        call_id: "call-2".into(),
        name: "tool_two".into(),
        content: "done".into(),
        is_error: false,
        weles_review: None,
    });
    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "call-3".into(),
        name: "tool_three".into(),
        args: "{}".into(),
        weles_review: None,
    });
    state.reduce(ChatAction::ToolResult {
        thread_id: "t1".into(),
        call_id: "call-3".into(),
        name: "tool_three".into(),
        content: "done".into(),
        is_error: false,
        weles_review: None,
    });
    state.reduce(ChatAction::Reasoning {
        thread_id: "t1".into(),
        content: "Final reasoning".into(),
    });
    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "Final message".into(),
    });
    state.reduce(ChatAction::TurnDone {
        thread_id: "t1".into(),
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

    let thread = state.active_thread().expect("thread should exist");
    let transcript: Vec<(MessageRole, Option<&str>, &str)> = thread
        .messages
        .iter()
        .map(|message| {
            (
                message.role,
                message.tool_name.as_deref(),
                message
                    .reasoning
                    .as_deref()
                    .unwrap_or(message.content.as_str()),
            )
        })
        .collect();

    assert_eq!(
        transcript,
        vec![
            (MessageRole::Assistant, None, "First reasoning"),
            (MessageRole::Tool, Some("tool_one"), "done"),
            (MessageRole::Assistant, None, "Second reasoning"),
            (MessageRole::Tool, Some("tool_two"), "done"),
            (MessageRole::Tool, Some("tool_three"), "done"),
            (MessageRole::Assistant, None, "Final reasoning"),
        ]
    );

    let final_message = thread.messages.last().expect("final message should exist");
    assert_eq!(final_message.content, "Final message");
    assert_eq!(final_message.reasoning.as_deref(), Some("Final reasoning"));
}

#[test]
fn tool_messages_store_weles_review_metadata() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    let review = WelesReviewMetaVm {
        weles_reviewed: true,
        verdict: "allow".into(),
        reasons: vec!["operator approved".into()],
        audit_id: Some("audit-1".into()),
        security_override_mode: Some("yolo".into()),
    };

    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "c1".into(),
        name: "bash_command".into(),
        args: "ls".into(),
        weles_review: Some(review.clone()),
    });
    state.reduce(ChatAction::ToolResult {
        thread_id: "t1".into(),
        call_id: "c1".into(),
        name: "bash_command".into(),
        content: "file.txt".into(),
        is_error: false,
        weles_review: Some(review.clone()),
    });

    let thread = state.active_thread().expect("thread should exist");
    let tool_message = thread
        .messages
        .iter()
        .find(|message| message.tool_call_id.as_deref() == Some("c1"))
        .expect("tool message should exist");

    let stored = tool_message
        .weles_review
        .as_ref()
        .expect("weles review should be stored on tool message");
    assert!(stored.weles_reviewed);
    assert_eq!(stored.verdict, "allow");
    assert_eq!(stored.audit_id.as_deref(), Some("audit-1"));
    assert_eq!(stored.security_override_mode.as_deref(), Some("yolo"));
}

#[test]
fn turn_done_uses_final_reasoning_when_no_reasoning_delta_was_streamed() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "Answer".into(),
    });

    state.reduce(ChatAction::TurnDone {
        thread_id: "t1".into(),
        input_tokens: 100,
        output_tokens: 50,
        cost: Some(0.01),
        provider: Some(PROVIDER_ID_GITHUB_COPILOT.into()),
        model: Some("gpt-5.4".into()),
        tps: Some(45.0),
        generation_ms: Some(1200),
        reasoning: Some("Final reasoning summary".into()),
        provider_final_result_json: Some("result_json".to_string()),
    });

    let thread = state.active_thread().unwrap();
    let last = thread.messages.last().unwrap();
    assert_eq!(last.reasoning.as_deref(), Some("Final reasoning summary"));
}

#[test]
fn turn_done_does_not_append_reasoning_only_duplicate_of_flushed_content() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "Perfect. That confirms it is truly online now.".into(),
    });
    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "call-1".into(),
        name: "bash_command".into(),
        args: "{}".into(),
        weles_review: None,
    });
    state.reduce(ChatAction::ToolResult {
        thread_id: "t1".into(),
        call_id: "call-1".into(),
        name: "bash_command".into(),
        content: "done".into(),
        is_error: false,
        weles_review: None,
    });

    state.reduce(ChatAction::TurnDone {
        thread_id: "t1".into(),
        input_tokens: 100,
        output_tokens: 50,
        cost: None,
        provider: None,
        model: None,
        tps: None,
        generation_ms: None,
        reasoning: Some("Perfect. That confirms it is truly online now.".into()),
        provider_final_result_json: Some("result_json".to_string()),
    });

    let thread = state.active_thread().unwrap();
    let duplicate_count = thread
        .messages
        .iter()
        .filter(|message| {
            message.role == MessageRole::Assistant
                && (message.content == "Perfect. That confirms it is truly online now."
                    || message.reasoning.as_deref()
                        == Some("Perfect. That confirms it is truly online now."))
        })
        .count();
    assert_eq!(
        duplicate_count, 1,
        "final reasoning identical to already flushed assistant content should not create a second visible block"
    );
}

#[test]
fn new_thread_clears_active() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    assert!(state.active_thread_id().is_some());
    state.reduce(ChatAction::NewThread);
    assert!(state.active_thread_id().is_none());
}

#[test]
fn background_delta_after_new_thread_does_not_reselect_previous_thread() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "First".into(),
    });
    state.reduce(ChatAction::NewThread);

    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "background output".into(),
    });

    assert_eq!(state.active_thread_id(), None);
    assert_eq!(state.streaming_content(), "");
}

#[test]
fn select_thread_changes_active() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        AgentThread {
            id: "t1".into(),
            title: "First".into(),
            ..Default::default()
        },
        AgentThread {
            id: "t2".into(),
            title: "Second".into(),
            ..Default::default()
        },
    ]));
    state.reduce(ChatAction::SelectThread("t2".into()));
    assert_eq!(state.active_thread_id(), Some("t2"));
}

#[test]
fn reselecting_thread_resets_scroll_lock_and_offset() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![AgentThread {
        id: "t1".into(),
        title: "First".into(),
        ..Default::default()
    }]));
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.reduce(ChatAction::ScrollChat(5));

    assert!(state.scroll_locked());
    assert_eq!(state.scroll_offset(), 5);

    state.reduce(ChatAction::SelectThread("t1".into()));

    assert!(!state.scroll_locked());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn inactive_thread_streaming_does_not_pollute_selected_thread_view() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        AgentThread {
            id: "t1".into(),
            title: "First".into(),
            ..Default::default()
        },
        AgentThread {
            id: "t2".into(),
            title: "Second".into(),
            ..Default::default()
        },
    ]));
    state.reduce(ChatAction::SelectThread("t2".into()));

    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "background output".into(),
    });
    state.reduce(ChatAction::Reasoning {
        thread_id: "t1".into(),
        content: "background reasoning".into(),
    });
    state.reduce(ChatAction::ToolCall {
        thread_id: "t1".into(),
        call_id: "call-1".into(),
        name: "bash_command".into(),
        args: "ls".into(),
        weles_review: None,
    });

    assert_eq!(state.active_thread_id(), Some("t2"));
    assert_eq!(state.streaming_content(), "");
    assert_eq!(state.streaming_reasoning(), "");
    assert!(state.active_tool_calls().is_empty());
}

#[test]
fn inactive_thread_done_finalizes_background_stream_on_origin_thread() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        AgentThread {
            id: "t1".into(),
            title: "First".into(),
            ..Default::default()
        },
        AgentThread {
            id: "t2".into(),
            title: "Second".into(),
            ..Default::default()
        },
    ]));
    state.reduce(ChatAction::SelectThread("t2".into()));

    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "background output".into(),
    });
    state.reduce(ChatAction::Reasoning {
        thread_id: "t1".into(),
        content: "background reasoning".into(),
    });
    state.reduce(ChatAction::TurnDone {
        thread_id: "t1".into(),
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

    assert_eq!(state.active_thread_id(), Some("t2"));
    assert_eq!(state.streaming_content(), "");
    assert_eq!(state.streaming_reasoning(), "");

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("origin thread should exist");
    let last = thread
        .messages
        .last()
        .expect("background reply should be recorded");
    assert_eq!(last.role, MessageRole::Assistant);
    assert_eq!(last.content, "background output");
    assert_eq!(last.reasoning.as_deref(), Some("background reasoning"));
}

#[test]
fn thread_detail_keeps_local_messages_with_actions() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "concierge".into(),
        title: "Concierge".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "concierge".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Welcome".into(),
            actions: vec![MessageAction {
                label: "Continue".into(),
                action_type: "continue_session".into(),
                thread_id: Some("t1".into()),
            }],
            is_concierge_welcome: true,
            ..Default::default()
        },
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "concierge".into(),
        title: "Concierge".into(),
        messages: vec![AgentMessage {
            role: MessageRole::Assistant,
            content: "Welcome".into(),
            ..Default::default()
        }],
        ..Default::default()
    }));

    let thread = state
        .active_thread()
        .expect("concierge thread should exist");
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.messages[0].actions.len(), 1);
}
