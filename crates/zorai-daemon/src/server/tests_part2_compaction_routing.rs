use super::*;
#[test]
fn manual_compaction_workflow_notice_forwards_only_to_subscribed_target_thread() {
    let event = AgentEvent::WorkflowNotice {
        thread_id: "thread-compaction".to_string(),
        kind: "manual-compaction".to_string(),
        message: "Manual compaction applied.".to_string(),
        details: Some("{\"split_at\":5,\"total_message_count\":12}".to_string()),
    };

    let subscribed = std::collections::HashSet::from(["thread-compaction".to_string()]);
    let other_thread = std::collections::HashSet::from(["thread-other".to_string()]);
    let no_threads = std::collections::HashSet::new();

    assert!(
        super::super::should_forward_agent_event(&event, &subscribed),
        "compaction notice should forward to the subscribed target thread"
    );
    assert!(
        !super::super::should_forward_agent_event(&event, &other_thread),
        "compaction notice should not forward to a different subscribed thread"
    );
    assert!(
        !super::super::should_forward_agent_event(&event, &no_threads),
        "compaction notice should not behave like a global event"
    );
}

#[test]
fn lag_recovery_synthesizes_thread_reload_required_for_each_subscribed_thread() {
    let mut subscribed = std::collections::HashSet::new();
    subscribed.insert("thread-a".to_string());
    subscribed.insert("thread-b".to_string());

    let events = super::super::lag_recovery_thread_reload_events(&subscribed);
    assert_eq!(
        events.len(),
        2,
        "lag recovery should synthesize one event per subscribed thread"
    );

    let mut recovered_ids: Vec<String> = events
        .into_iter()
        .map(|event| match event {
            AgentEvent::ThreadReloadRequired { thread_id } => thread_id,
            other => panic!("expected ThreadReloadRequired, got {other:?}"),
        })
        .collect();
    recovered_ids.sort();
    assert_eq!(recovered_ids, vec!["thread-a", "thread-b"]);
}

#[test]
fn lag_recovery_with_no_subscribed_threads_returns_no_events() {
    let subscribed: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let events = super::super::lag_recovery_thread_reload_events(&subscribed);
    assert!(
        events.is_empty(),
        "no subscribed threads → no recovery events"
    );
}

#[test]
fn thread_reload_required_for_compaction_forwards_only_to_subscribed_target_thread() {
    let event = AgentEvent::ThreadReloadRequired {
        thread_id: "thread-compaction".to_string(),
    };

    let subscribed = std::collections::HashSet::from(["thread-compaction".to_string()]);
    let other_thread = std::collections::HashSet::from(["thread-other".to_string()]);

    assert!(
        super::super::should_forward_agent_event(&event, &subscribed),
        "compaction reload should forward to the subscribed target thread"
    );
    assert!(
        !super::super::should_forward_agent_event(&event, &other_thread),
        "compaction reload should not forward to a different subscribed thread"
    );
}
