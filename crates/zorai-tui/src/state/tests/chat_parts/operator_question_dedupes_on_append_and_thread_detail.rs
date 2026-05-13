use crate::state::chat::*;

#[test]
fn append_message_dedupes_existing_operator_question_id() {
    // Why this matters: the daemon's OperatorQuestion event can arrive twice
    // (observed in zorai-daemon log 2026-05-12 11:25:43 where the agent event
    // bus emitted some events in rapid succession). Each delivery dispatches
    // ChatAction::AppendMessage. Without idempotency the TUI renders the
    // question text and "Responder:" header twice per question — the bug the
    // user reported with a screenshot. Idempotency keyed on
    // operator_question_id is the only field guaranteed unique per question
    // at the time the live event fires (id is None until persistence).
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    let message = AgentMessage {
        role: MessageRole::Assistant,
        content: "Pick A or B".into(),
        is_operator_question: true,
        operator_question_id: Some("oq_dup".into()),
        ..Default::default()
    };
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: message.clone(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message,
    });

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(
        thread.messages.len(),
        1,
        "second AppendMessage with the same operator_question_id must be ignored"
    );
}

#[test]
fn append_message_keeps_distinct_operator_questions() {
    // Why this matters: the idempotency check must not collapse two genuinely
    // different operator questions into one. We key strictly on the
    // operator_question_id string.
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Q1".into(),
            is_operator_question: true,
            operator_question_id: Some("oq_one".into()),
            ..Default::default()
        },
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Q2".into(),
            is_operator_question: true,
            operator_question_id: Some("oq_two".into()),
            ..Default::default()
        },
    });

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    assert_eq!(thread.messages.len(), 2);
}

#[test]
fn thread_detail_received_dedupes_operator_question_id_after_merge() {
    // Why this matters: if a live OperatorQuestion event already appended a
    // message locally (id=None) and the daemon snapshot arrives later with
    // the same question persisted under a fresh row id, merge_thread_window
    // would otherwise keep both because it indexes by absolute position.
    // dedupe_operator_question_messages must collapse them.
    let mut state = ChatState::new();
    state.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    state.reduce(ChatAction::AppendMessage {
        thread_id: "t1".into(),
        message: AgentMessage {
            id: None,
            role: MessageRole::Assistant,
            content: "Pick A or B".into(),
            is_operator_question: true,
            operator_question_id: Some("oq_xyz".into()),
            ..Default::default()
        },
    });

    let incoming = AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        messages: vec![AgentMessage {
            id: Some("db_row_1".into()),
            role: MessageRole::Assistant,
            content: "Pick A or B".into(),
            is_operator_question: true,
            operator_question_id: Some("oq_xyz".into()),
            timestamp: 42,
            ..Default::default()
        }],
        total_message_count: 1,
        loaded_message_start: 0,
        loaded_message_end: 1,
        ..Default::default()
    };
    state.reduce(ChatAction::ThreadDetailReceived(incoming));

    let thread = state
        .threads()
        .iter()
        .find(|thread| thread.id == "t1")
        .expect("thread should exist");
    let matches: Vec<&AgentMessage> = thread
        .messages
        .iter()
        .filter(|message| message.operator_question_id.as_deref() == Some("oq_xyz"))
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "snapshot merge must not duplicate a question already present locally"
    );
    assert_eq!(
        thread.loaded_message_end,
        thread.loaded_message_start + thread.messages.len(),
        "loaded window must stay consistent with messages.len() after dedup"
    );
}
