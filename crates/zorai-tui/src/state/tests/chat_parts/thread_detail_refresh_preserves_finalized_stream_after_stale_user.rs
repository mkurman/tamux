use crate::state::chat::*;
#[test]
fn thread_detail_refresh_preserves_finalized_stream_after_stale_user_snapshot() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "Follow up".into(),
            timestamp: 100,
            ..Default::default()
        },
    });
    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "Here is the answer".into(),
    });
    state.reduce(ChatAction::TurnDone {
        thread_id: "t1".into(),
        input_tokens: 1,
        output_tokens: 2,
        cost: None,
        provider: None,
        model: None,
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: None,
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 1,
        loaded_message_start: 0,
        loaded_message_end: 1,
        messages: vec![AgentMessage {
            id: Some("msg-user".into()),
            role: MessageRole::User,
            content: "Follow up".into(),
            timestamp: 200,
            ..Default::default()
        }],
        ..Default::default()
    }));

    let thread = state.active_thread().unwrap();
    assert_eq!(thread.messages.len(), 2);
    assert_eq!(thread.messages[0].content, "Follow up");
    assert_eq!(
        thread.messages[1].content, "Here is the answer",
        "stale detail refresh should not erase the just-finalized assistant stream"
    );
}

#[test]
fn thread_detail_reload_with_short_tail_does_not_wipe_existing_messages() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 4,
        loaded_message_start: 0,
        loaded_message_end: 4,
        messages: (0..4)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("existing {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 1,
        loaded_message_start: 0,
        loaded_message_end: 1,
        messages: vec![AgentMessage {
            id: Some("msg-new".into()),
            role: MessageRole::User,
            content: "new prompt after reload".into(),
            ..Default::default()
        }],
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    let ids = thread
        .messages
        .iter()
        .filter_map(|message| message.id.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec!["msg-0", "msg-1", "msg-2", "msg-3", "msg-new"],
        "short reload snapshots should not replace already loaded history"
    );
}

#[test]
fn empty_thread_detail_does_not_wipe_existing_messages() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Thread".into(),
        total_message_count: 1,
        loaded_message_start: 0,
        loaded_message_end: 1,
        messages: vec![AgentMessage {
            id: Some("msg-0".into()),
            role: MessageRole::Assistant,
            content: "First reply".into(),
            ..Default::default()
        }],
        ..Default::default()
    }));
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::User,
            content: "Follow up".into(),
            ..Default::default()
        },
    });

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Thread".into(),
        agent_name: Some("Dola".into()),
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .unwrap();
    assert_eq!(thread.messages.len(), 2);
    assert_eq!(
        thread
            .messages
            .last()
            .map(|message| message.content.as_str()),
        Some("Follow up")
    );
    assert_eq!(thread.agent_name.as_deref(), Some("Dola"));
}

#[test]
fn empty_thread_detail_preserves_paged_message_window() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Thread".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        messages: (70..120)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("visible {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Thread".into(),
        profile_model: Some("gpt-5.5".into()),
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .unwrap();
    assert_eq!(thread.messages.len(), 50);
    assert_eq!(thread.total_message_count, 120);
    assert_eq!(thread.loaded_message_start, 70);
    assert_eq!(thread.loaded_message_end, 120);
    assert_eq!(thread.profile_model.as_deref(), Some("gpt-5.5"));
    assert_eq!(
        thread
            .messages
            .first()
            .and_then(|message| message.id.as_deref()),
        Some("msg-70")
    );
}

#[test]
fn thread_detail_refresh_replaces_window_when_overlapping_message_ids_shift() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 4,
        loaded_message_start: 0,
        loaded_message_end: 4,
        messages: (0..4)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::Assistant,
                content: format!("before {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 4,
        loaded_message_start: 0,
        loaded_message_end: 4,
        messages: vec![
            AgentMessage {
                id: Some("msg-1".into()),
                role: MessageRole::Assistant,
                content: "shifted 1".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-2".into()),
                role: MessageRole::Assistant,
                content: "shifted 2".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-3".into()),
                role: MessageRole::Assistant,
                content: "shifted 3".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-4".into()),
                role: MessageRole::Assistant,
                content: "shifted 4".into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .unwrap();
    let ids = thread
        .messages
        .iter()
        .filter_map(|message| message.id.as_deref())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec!["msg-1", "msg-2", "msg-3", "msg-4"],
        "overlapping windows with shifted message IDs should prefer the fresh authoritative mapping"
    );
}

#[test]
fn thread_detail_derives_compaction_boundary_and_preserves_it_after_collapse() {
    let mut state = ChatState::new();
    state.set_history_page_size(2);
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 5,
        loaded_message_start: 0,
        loaded_message_end: 5,
        messages: vec![
            AgentMessage {
                id: Some("msg-0".into()),
                role: MessageRole::User,
                content: "before".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-1".into()),
                role: MessageRole::Assistant,
                content: "compacted".into(),
                message_kind: "compaction_artifact".into(),
                compaction_payload: Some("Older context compacted".into()),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-2".into()),
                role: MessageRole::Assistant,
                content: "after 1".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-3".into()),
                role: MessageRole::User,
                content: "after 2".into(),
                ..Default::default()
            },
            AgentMessage {
                id: Some("msg-4".into()),
                role: MessageRole::Assistant,
                content: "after 3".into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    let before = state.active_thread().expect("thread should exist");
    assert_eq!(
        before.active_compaction_window_start,
        Some(1),
        "thread detail should derive the latest compaction boundary from loaded messages"
    );

    state.schedule_history_collapse(0, 0);
    state.maybe_collapse_history(0);

    let after = state.active_thread().expect("thread should exist");
    assert_eq!(after.loaded_message_start, 3);
    assert_eq!(after.messages.len(), 2);
    assert_eq!(
        after.active_compaction_window_start,
        Some(1),
        "history collapse should preserve the absolute compaction boundary even after trimming the artifact"
    );
}

#[test]
fn collapse_history_keeps_latest_page_only() {
    let mut state = ChatState::new();
    state.set_history_page_size(50);
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 20,
        loaded_message_end: 120,
        messages: (20..120)
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

    state.schedule_history_collapse(10, 5);
    state.maybe_collapse_history(14);
    assert_eq!(state.active_thread().expect("thread").messages.len(), 100);

    state.maybe_collapse_history(15);
    let thread = state.active_thread().expect("thread should exist");
    assert_eq!(thread.loaded_message_start, 70);
    assert_eq!(thread.loaded_message_end, 120);
    assert_eq!(thread.messages.len(), 50);
    assert_eq!(
        thread
            .messages
            .first()
            .and_then(|message| message.id.as_deref()),
        Some("msg-70")
    );
}

#[test]
fn collapse_history_waits_when_viewport_is_locked_at_zero_offset() {
    let mut state = ChatState::new();
    state.set_history_page_size(50);
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 0,
        loaded_message_end: 120,
        messages: (0..120)
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
    state.preserve_prepend_scroll_anchor(0);

    state.schedule_history_collapse(10, 5);
    state.maybe_collapse_history(15);

    let thread = state.active_thread().expect("thread should exist");
    assert_eq!(thread.loaded_message_start, 0);
    assert_eq!(thread.loaded_message_end, 120);
    assert_eq!(thread.messages.len(), 120);
    assert_eq!(
        thread
            .messages
            .first()
            .and_then(|message| message.id.as_deref()),
        Some("msg-0")
    );
}

#[test]
fn selected_message_tracks_same_message_when_older_page_is_prepended() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 70,
        loaded_message_end: 120,
        messages: (70..120)
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
    state.select_message(Some(10));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        total_message_count: 120,
        loaded_message_start: 20,
        loaded_message_end: 70,
        messages: (20..70)
            .map(|index| AgentMessage {
                id: Some(format!("msg-{index}")),
                role: MessageRole::User,
                content: format!("msg {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }));

    let selected_index = state.selected_message().expect("selection should survive");
    let thread = state.active_thread().expect("thread should exist");
    assert_eq!(selected_index, 60);
    assert_eq!(
        thread.messages[selected_index].id.as_deref(),
        Some("msg-80")
    );
}
