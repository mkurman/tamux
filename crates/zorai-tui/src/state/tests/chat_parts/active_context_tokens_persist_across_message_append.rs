use crate::state::chat::*;

#[test]
fn active_context_window_tokens_persist_across_message_append() {
    // Why this matters: header shows context-utilization % from
    // active_context_window_tokens. Before the fix, every AppendMessage reset
    // it to None — header flashed 0% between turns. The daemon only sends a
    // fresh ContextWindowUpdate after the next request, so we must keep the
    // last-known value across local mutations to avoid the visible swing.
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::ContextWindowUpdated {
        thread_id: "t1".into(),
        active_context_window_start: 0,
        active_context_window_end: 5,
        active_context_window_tokens: 50_000,
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "another reply".into(),
            timestamp: 100,
            ..Default::default()
        },
    });

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(
        thread.active_context_window_tokens,
        Some(50_000),
        "tokens must persist across message append; otherwise header flashes 0%"
    );
}

#[test]
fn active_context_window_tokens_persist_across_compaction_until_daemon_update() {
    // Why this matters: CompactionApplied used to null tokens, then the daemon
    // sent a fresh ContextWindowUpdate immediately after. Any UI render between
    // those two events showed 0%. We now keep the last-known value so the
    // worst case is a brief stale (slightly-too-high) reading rather than 0%.
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::ContextWindowUpdated {
        thread_id: "t1".into(),
        active_context_window_start: 0,
        active_context_window_end: 50,
        active_context_window_tokens: 200_000,
    });
    state.reduce(ChatAction::CompactionApplied {
        thread_id: "t1".into(),
        active_compaction_window_start: 40,
        total_message_count: 51,
    });

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(
        thread.active_context_window_tokens,
        Some(200_000),
        "tokens must persist through CompactionApplied; daemon's ContextWindowUpdate will correct shortly"
    );
}
