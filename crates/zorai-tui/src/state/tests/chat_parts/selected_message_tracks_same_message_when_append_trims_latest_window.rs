use super::select_next_message_from_none_to_thread_detail_refresh_preserves::*;
use crate::state::chat::*;
#[test]
fn selected_message_tracks_same_message_when_append_trims_latest_window() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 100,
        loaded_message_start: 0,
        loaded_message_end: 100,
        messages: (0..100)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.select_message(Some(80));

    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            id: Some("msg-100".into()),
            role: MessageRole::Assistant,
            content: "msg 100".into(),
            ..Default::default()
        },
    });

    let selected_index = state.selected_message().expect("selection should survive");
    let thread = state.active_thread().expect("thread should exist");
    assert_eq!(selected_index, 79);
    assert_eq!(
        thread.messages[selected_index].id.as_deref(),
        Some("msg-80")
    );
}

#[test]
fn resolve_message_ref_ignores_absolute_indexes_before_loaded_window() {
    let thread = AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 8,
        loaded_message_start: 5,
        loaded_message_end: 8,
        messages: (5..8)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };

    let message_ref = StoredMessageRef {
        thread_id: "t1".into(),
        message_id: None,
        absolute_index: 2,
    };

    assert_eq!(resolve_message_ref(&thread, &message_ref), None);
}

#[test]
fn expanded_reasoning_and_tools_track_same_messages_across_window_updates() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 100,
        loaded_message_start: 0,
        loaded_message_end: 100,
        messages: (0..100)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: if index == 80 {
                    MessageRole::Assistant
                } else if index == 90 {
                    MessageRole::Tool
                } else {
                    MessageRole::User
                },
                content: format!("msg {index}"),
                reasoning: (index == 80).then(|| "reasoning".to_string()),
                tool_name: (index == 90).then(|| "bash".to_string()),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));
    state.toggle_reasoning(80);
    state.toggle_tool_expansion(90);

    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            id: Some("msg-100".into()),
            role: MessageRole::Assistant,
            content: "msg 100".into(),
            ..Default::default()
        },
    });

    let thread = state.active_thread().expect("thread should exist");
    let expanded_reasoning = state.expanded_reasoning();
    let expanded_tools = state.expanded_tools();
    let reasoning_index = expanded_reasoning
        .iter()
        .copied()
        .next()
        .expect("reasoning expansion should survive");
    let tool_index = expanded_tools
        .iter()
        .copied()
        .next()
        .expect("tool expansion should survive");
    assert_eq!(
        thread.messages[reasoning_index].id.as_deref(),
        Some("msg-80")
    );
    assert_eq!(thread.messages[tool_index].id.as_deref(), Some("msg-90"));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 101,
        loaded_message_start: 0,
        loaded_message_end: 50,
        messages: (0..50)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    let thread = state.active_thread().expect("thread should exist");
    let expanded_reasoning = state.expanded_reasoning();
    let expanded_tools = state.expanded_tools();
    let reasoning_index = expanded_reasoning
        .iter()
        .copied()
        .next()
        .expect("reasoning expansion should survive prepend");
    let tool_index = expanded_tools
        .iter()
        .copied()
        .next()
        .expect("tool expansion should survive prepend");
    assert_eq!(
        thread.messages[reasoning_index].id.as_deref(),
        Some("msg-80")
    );
    assert_eq!(thread.messages[tool_index].id.as_deref(), Some("msg-90"));
}

#[test]
fn select_prev_message_clamps_at_zero() {
    let mut state = state_with_messages(3);
    state.select_message(Some(0));
    state.select_prev_message();
    assert_eq!(state.selected_message(), Some(0));
}

#[test]
fn clear_selection() {
    let mut state = state_with_messages(3);
    state.select_message(Some(1));
    state.select_message(None);
    assert_eq!(state.selected_message(), None);
}

#[test]
fn toggle_message_selection_clears_when_same_message_clicked() {
    let mut state = state_with_messages(3);
    state.toggle_message_selection(1);
    assert_eq!(state.selected_message(), Some(1));
    state.toggle_message_selection(1);
    assert_eq!(state.selected_message(), None);
}

#[test]
fn toggle_tool_expansion() {
    let mut state = state_with_messages(1);
    assert!(!state.expanded_tools().contains(&0));
    state.toggle_tool_expansion(0);
    assert!(state.expanded_tools().contains(&0));
    state.toggle_tool_expansion(0);
    assert!(!state.expanded_tools().contains(&0));
}

#[test]
fn toggle_tool_expansion_independent() {
    let mut state = state_with_messages(2);
    state.toggle_tool_expansion(0);
    state.toggle_tool_expansion(1);
    assert!(state.expanded_tools().contains(&0));
    assert!(state.expanded_tools().contains(&1));
    state.toggle_tool_expansion(0);
    assert!(!state.expanded_tools().contains(&0));
    assert!(state.expanded_tools().contains(&1));
}
