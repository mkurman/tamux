use super::select_next_message_from_none_to_thread_detail_refresh_preserves::*;
use crate::state::chat::*;
use zorai_shared::providers::{PROVIDER_ID_GITHUB_COPILOT, PROVIDER_ID_OPENAI};

fn make_thread(id: &str, title: &str) -> AgentThread {
    AgentThread {
        id: id.into(),
        title: title.into(),
        ..Default::default()
    }
}

#[test]
fn delta_appends_to_streaming_content() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "Hello".into(),
    });
    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: " world".into(),
    });
    assert_eq!(state.streaming_content(), "Hello world");
}

#[test]
fn turn_done_finalizes_streaming_into_message() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "Hi".into(),
    });
    state.reduce(ChatAction::TurnDone {
        thread_id: "t1".into(),
        input_tokens: 100,
        output_tokens: 50,
        cost: Some(0.01),
        provider: Some(PROVIDER_ID_OPENAI.into()),
        model: Some("gpt-4o".into()),
        tps: Some(45.0),
        generation_ms: Some(1200),
        reasoning: None,
        provider_final_result_json: Some("result_json".to_string()),
    });
    assert_eq!(state.streaming_content(), "");
    let thread = state.active_thread().unwrap();
    let last = thread.messages.last().unwrap();
    assert_eq!(last.content, "Hi");
    assert_eq!(last.role, MessageRole::Assistant);
}

#[test]
fn scroll_up_locks_scroll() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ScrollChat(5));
    assert!(state.scroll_locked());
    assert_eq!(state.scroll_offset(), 5);
}

#[test]
fn scroll_to_zero_unlocks() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ScrollChat(5));
    state.reduce(ChatAction::ScrollChat(-5));
    assert!(!state.scroll_locked());
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn locked_scroll_offset_grows_with_streamed_newlines() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::ScrollChat(4));

    state.reduce(ChatAction::Delta {
        thread_id: "t1".into(),
        content: "\nnext\nchunk".into(),
    });

    assert_eq!(state.scroll_offset(), 6);
    assert!(state.scroll_locked());
}

#[test]
fn retry_status_replaces_previous_status_for_active_thread() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::SetRetryStatus {
        thread_id: "t1".into(),
        phase: RetryPhase::Retrying,
        attempt: 1,
        max_retries: 3,
        delay_ms: 2_000,
        failure_class: "rate_limit".into(),
        message: "429".into(),
        received_at_tick: 10,
    });
    state.reduce(ChatAction::SetRetryStatus {
        thread_id: "t1".into(),
        phase: RetryPhase::Waiting,
        attempt: 3,
        max_retries: 3,
        delay_ms: 30_000,
        failure_class: "rate_limit".into(),
        message: "retrying automatically".into(),
        received_at_tick: 20,
    });

    let status = state.retry_status().expect("retry status should exist");
    assert_eq!(status.phase, RetryPhase::Waiting);
    assert_eq!(status.delay_ms, 30_000);
    assert_eq!(status.received_at_tick, 20);
}

#[test]
fn copied_message_feedback_expires_after_deadline() {
    let mut state = state_with_messages(1);
    state.mark_message_copied(0, 25);

    assert!(state.is_message_recently_copied(0, 24));

    state.clear_expired_copy_feedback(25);

    assert!(!state.is_message_recently_copied(0, 25));
}

#[test]
fn copied_message_feedback_keeps_render_cache_epoch_stable_until_expiry() {
    let mut state = state_with_messages(1);
    state.mark_message_copied(0, 25);

    let active_epoch = state.render_cache_epoch(10);

    assert_eq!(
        active_epoch,
        state.render_cache_epoch(24),
        "active copy feedback should not invalidate the chat render cache on every tick"
    );

    state.clear_expired_copy_feedback(25);

    assert_eq!(state.render_cache_epoch(25), 0);
}

#[test]
fn thread_list_received_replaces_threads() {
    let mut state = ChatState::new();
    let threads = vec![
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
    ];
    state.reduce(ChatAction::ThreadListReceived(threads));
    assert_eq!(state.threads().len(), 2);
}

#[test]
fn thread_history_stack_pushes_and_pops_in_order() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        make_thread("thread-a", "A"),
        make_thread("thread-b", "B"),
        make_thread("thread-c", "C"),
    ]));
    state.reduce(ChatAction::SelectThread("thread-a".into()));

    assert!(state.open_spawned_thread("thread-a", "thread-b"));
    assert!(state.open_spawned_thread("thread-b", "thread-c"));
    assert_eq!(
        state.thread_history_stack(),
        &["thread-a".to_string(), "thread-b".to_string()]
    );

    assert_eq!(state.go_back_thread(), Some("thread-b".to_string()));
    assert_eq!(state.active_thread_id(), Some("thread-b"));
    assert_eq!(state.thread_history_stack(), &["thread-a".to_string()]);

    assert_eq!(state.go_back_thread(), Some("thread-a".to_string()));
    assert_eq!(state.active_thread_id(), Some("thread-a"));
    assert!(state.thread_history_stack().is_empty());
}

#[test]
fn thread_history_stack_suppresses_duplicate_top_entries() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        make_thread("thread-a", "A"),
        make_thread("thread-b", "B"),
    ]));
    state.reduce(ChatAction::SelectThread("thread-a".into()));

    assert!(state.open_spawned_thread("thread-a", "thread-b"));
    assert!(state.open_spawned_thread("thread-a", "thread-b"));

    assert_eq!(state.thread_history_stack(), &["thread-a".to_string()]);
}

#[test]
fn thread_history_stack_allows_opening_unloaded_spawned_thread() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![make_thread(
        "thread-a", "A",
    )]));
    state.reduce(ChatAction::SelectThread("thread-a".into()));

    assert!(state.open_spawned_thread("thread-a", "thread-b"));
    assert_eq!(state.active_thread_id(), Some("thread-b"));
    assert_eq!(state.thread_history_stack(), &["thread-a".to_string()]);
}

#[test]
fn spawned_thread_navigation_bumps_render_revision() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        make_thread("thread-a", "A"),
        make_thread("thread-b", "B"),
    ]));
    state.reduce(ChatAction::SelectThread("thread-a".into()));
    let before_open = state.render_revision();

    assert!(state.open_spawned_thread("thread-a", "thread-b"));
    assert!(
        state.render_revision() > before_open,
        "opening a spawned thread changes the rendered transcript"
    );

    let before_back = state.render_revision();
    assert_eq!(state.go_back_thread(), Some("thread-a".to_string()));
    assert!(
        state.render_revision() > before_back,
        "returning to the previous thread changes the rendered transcript"
    );
}

#[test]
fn thread_history_stack_survives_ordinary_selection_and_prunes_deleted_threads() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        make_thread("thread-a", "A"),
        make_thread("thread-b", "B"),
        make_thread("thread-c", "C"),
    ]));
    state.reduce(ChatAction::SelectThread("thread-a".into()));

    assert!(state.open_spawned_thread("thread-a", "thread-b"));
    assert!(state.open_spawned_thread("thread-b", "thread-c"));
    assert_eq!(state.thread_navigation_depth(), 2);

    state.reduce(ChatAction::SelectThread("thread-c".into()));
    assert_eq!(
        state.thread_history_stack(),
        &["thread-a".to_string(), "thread-b".to_string()]
    );
    assert_eq!(state.go_back_thread(), Some("thread-b".to_string()));
    assert_eq!(state.active_thread_id(), Some("thread-b"));

    assert!(state.open_spawned_thread("thread-b", "thread-c"));
    state.reduce(ChatAction::ThreadDeleted {
        thread_id: "thread-c".into(),
    });
    assert!(
        state.thread_history_stack().is_empty(),
        "deleting the active thread should reset spawned-thread history"
    );
    assert_eq!(state.go_back_thread(), None);
}

#[test]
fn thread_history_stack_prunes_missing_threads_on_list_replacement() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        make_thread("thread-a", "A"),
        make_thread("thread-b", "B"),
        make_thread("thread-c", "C"),
    ]));
    state.reduce(ChatAction::SelectThread("thread-a".into()));

    assert!(state.open_spawned_thread("thread-a", "thread-b"));
    assert_eq!(state.thread_history_stack(), &["thread-a".to_string()]);

    state.reduce(ChatAction::ThreadListReceived(vec![make_thread(
        "thread-c", "C",
    )]));
    assert!(state.thread_history_stack().is_empty());
    assert_eq!(state.go_back_thread(), None);
}

#[test]
fn thread_list_received_preserves_existing_messages_when_summary_is_empty() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "First".into(),
        messages: vec![AgentMessage {
            role: MessageRole::Assistant,
            content: "Existing detail".into(),
            ..Default::default()
        }],
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadListReceived(vec![AgentThread {
        id: "t1".into(),
        title: "First renamed".into(),
        messages: Vec::new(),
        ..Default::default()
    }]));

    let thread = state.threads().first().expect("thread should exist");
    assert_eq!(thread.title, "First renamed");
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.messages[0].content, "Existing detail");
}

#[test]
fn thread_list_received_preserves_existing_profile_metadata_when_summary_is_empty() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "First".into(),
        agent_name: Some("Svarog".into()),
        profile_provider: Some(PROVIDER_ID_GITHUB_COPILOT.into()),
        profile_model: Some("gpt-5.4".into()),
        profile_reasoning_effort: Some("xhigh".into()),
        profile_context_window_tokens: Some(400_000),
        messages: vec![AgentMessage {
            role: MessageRole::Assistant,
            content: "Existing detail".into(),
            ..Default::default()
        }],
        ..Default::default()
    }));

    state.reduce(ChatAction::ThreadListReceived(vec![AgentThread {
        id: "t1".into(),
        title: "First renamed".into(),
        messages: Vec::new(),
        ..Default::default()
    }]));

    let thread = state.threads().first().expect("thread should exist");
    assert_eq!(thread.agent_name.as_deref(), Some("Svarog"));
    assert_eq!(
        thread.profile_provider.as_deref(),
        Some(PROVIDER_ID_GITHUB_COPILOT)
    );
    assert_eq!(thread.profile_model.as_deref(), Some("gpt-5.4"));
    assert_eq!(thread.profile_reasoning_effort.as_deref(), Some("xhigh"));
    assert_eq!(thread.profile_context_window_tokens, Some(400_000));
}

#[test]
fn duplicate_thread_created_preserves_loaded_history_window() {
    let mut state = ChatState::new();
    let messages = (20..120)
        .map(|index| AgentMessage {
            role: MessageRole::Assistant,
            content: format!("message {index}"),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Existing".into(),
        messages,
        total_message_count: 120,
        loaded_message_start: 20,
        loaded_message_end: 120,
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Existing".into(),
    });

    let thread = state.active_thread().expect("thread should still exist");
    assert_eq!(thread.total_message_count, 120);
    assert_eq!(thread.loaded_message_start, 20);
    assert_eq!(thread.loaded_message_end, 120);
    assert_eq!(
        thread
            .messages
            .first()
            .map(|message| message.content.as_str()),
        Some("message 20")
    );
}

#[test]
fn thread_detail_hydrates_pinned_for_compaction() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Pinned".into(),
        messages: vec![AgentMessage {
            id: Some("m1".into()),
            role: MessageRole::User,
            content: "keep this".into(),
            pinned_for_compaction: true,
            ..Default::default()
        }],
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    let thread = state.active_thread().expect("thread should exist");
    assert!(thread.messages[0].pinned_for_compaction);
}

#[test]
fn thread_refresh_updates_pinned_for_compaction_flags() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Pinned".into(),
        messages: vec![AgentMessage {
            id: Some("m1".into()),
            role: MessageRole::User,
            content: "keep this".into(),
            pinned_for_compaction: false,
            ..Default::default()
        }],
        loaded_message_end: 1,
        total_message_count: 1,
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Pinned".into(),
        messages: vec![AgentMessage {
            id: Some("m1".into()),
            role: MessageRole::User,
            content: "keep this".into(),
            pinned_for_compaction: true,
            ..Default::default()
        }],
        loaded_message_end: 1,
        total_message_count: 1,
        ..Default::default()
    }));

    let thread = state.active_thread().expect("thread should exist");
    assert!(thread.messages[0].pinned_for_compaction);
}

#[test]
fn active_thread_pinned_messages_follow_thread_order() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Pinned".into(),
        messages: vec![
            AgentMessage {
                id: Some("m1".into()),
                role: MessageRole::User,
                content: "unpinned".into(),
                pinned_for_compaction: false,
                ..Default::default()
            },
            AgentMessage {
                id: Some("m2".into()),
                role: MessageRole::Assistant,
                content: "first pin".into(),
                pinned_for_compaction: true,
                ..Default::default()
            },
            AgentMessage {
                id: Some("m3".into()),
                role: MessageRole::User,
                content: "second pin".into(),
                pinned_for_compaction: true,
                ..Default::default()
            },
        ],
        loaded_message_end: 3,
        total_message_count: 3,
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    let pinned = state.active_thread_pinned_messages();
    assert_eq!(pinned.len(), 2);
    assert_eq!(pinned[0].absolute_index, 1);
    assert_eq!(pinned[0].message_id, "m2");
    assert_eq!(pinned[1].absolute_index, 2);
    assert_eq!(pinned[1].message_id, "m3");
}

#[test]
fn thread_detail_hydrates_pinned_summaries_outside_loaded_window() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Pinned".into(),
        messages: vec![AgentMessage {
            id: Some("m3".into()),
            role: MessageRole::Assistant,
            content: "latest".into(),
            pinned_for_compaction: false,
            ..Default::default()
        }],
        pinned_messages: vec![PinnedThreadMessage {
            message_id: "m1".into(),
            absolute_index: 0,
            role: MessageRole::User,
            content: "offscreen pin".into(),
        }],
        loaded_message_start: 2,
        loaded_message_end: 3,
        total_message_count: 3,
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    let pinned = state.active_thread_pinned_messages();
    assert!(state.active_thread_has_pinned_messages());
    assert_eq!(pinned.len(), 1);
    assert_eq!(pinned[0].message_id, "m1");
    assert_eq!(pinned[0].absolute_index, 0);
    assert_eq!(pinned[0].content, "offscreen pin");
}

#[test]
fn local_unpin_removes_offscreen_pinned_summary_immediately() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Pinned".into(),
        messages: vec![AgentMessage {
            id: Some("m3".into()),
            role: MessageRole::Assistant,
            content: "latest".into(),
            pinned_for_compaction: false,
            ..Default::default()
        }],
        pinned_messages: vec![PinnedThreadMessage {
            message_id: "m1".into(),
            absolute_index: 0,
            role: MessageRole::User,
            content: "offscreen pin".into(),
        }],
        loaded_message_start: 2,
        loaded_message_end: 3,
        total_message_count: 3,
        ..Default::default()
    }));
    state.reduce(ChatAction::SelectThread("t1".into()));

    state.reduce(ChatAction::UnpinMessageForCompaction {
        thread_id: "t1".into(),
        message_id: "m1".into(),
        absolute_index: Some(0),
    });

    assert!(!state.active_thread_has_pinned_messages());
    assert!(state.active_thread_pinned_messages().is_empty());
}

#[test]
fn thread_created_moves_new_thread_to_front() {
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadListReceived(vec![
        AgentThread {
            id: "older".into(),
            title: "Older".into(),
            updated_at: 10,
            ..Default::default()
        },
        AgentThread {
            id: "oldest".into(),
            title: "Oldest".into(),
            updated_at: 5,
            ..Default::default()
        },
    ]));

    state.reduce(ChatAction::ThreadCreated {
        thread_id: "new".into(),
        title: "Newest".into(),
    });

    let threads = state.threads();
    assert_eq!(threads[0].id, "new");
    assert_eq!(threads[1].id, "older");
    assert_eq!(threads[2].id, "oldest");
    assert_eq!(state.active_thread_id(), Some("new"));
}
