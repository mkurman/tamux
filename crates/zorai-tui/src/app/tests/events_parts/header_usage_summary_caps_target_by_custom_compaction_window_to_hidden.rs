use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn header_usage_summary_caps_target_by_custom_compaction_window() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;
    model.config.compact_threshold_pct = 80;
    model.config.compaction_strategy = "custom_model".to_string();
    model.config.compaction_custom_context_window_tokens = 160_000;

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-custom-target".to_string(),
        title: "Custom".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-custom-target".to_string(),
    ));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.compaction_target_tokens, 128_000);
    assert_eq!(usage.context_window_tokens, 400_000);
}

#[test]
fn header_usage_summary_does_not_estimate_after_compaction_artifact() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-compaction".to_string(),
        title: "Compaction".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-compaction".to_string(),
    ));

    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-compaction".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::User,
            content: "A".repeat(4_000),
            ..Default::default()
        },
    });
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-compaction".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "B".repeat(4_000),
            cost: Some(0.10),
            ..Default::default()
        },
    });

    let before = model.current_header_usage_summary();

    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-compaction".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "rule based".to_string(),
            message_kind: "compaction_artifact".to_string(),
            compaction_payload: Some("Older context compacted for continuity".to_string()),
            ..Default::default()
        },
    });
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-compaction".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "short follow-up".to_string(),
            cost: Some(0.15),
            ..Default::default()
        },
    });

    let after = model.current_header_usage_summary();
    assert_eq!(
        before.current_tokens, 0,
        "header should not estimate active context usage before daemon context state arrives"
    );
    assert_eq!(
        after.current_tokens, 0,
        "header should not estimate active context usage after compaction without daemon context state"
    );
    let total_cost = after
        .total_cost_usd
        .expect("header should include summed total cost after compaction");
    assert!(
        (total_cost - 0.25).abs() < 1e-9,
        "expected summed total cost to stay at 0.25, got {total_cost}"
    );
}

#[test]
fn header_usage_summary_ignores_loaded_messages_before_known_compaction_boundary() {
    let mut model = make_model();

    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-boundary".to_string(),
            title: "Boundary".to_string(),
            total_message_count: 4,
            loaded_message_start: 0,
            loaded_message_end: 4,
            active_compaction_window_start: Some(2),
            messages: vec![
                crate::state::chat::AgentMessage {
                    role: crate::state::chat::MessageRole::User,
                    content: "A".repeat(400),
                    ..Default::default()
                },
                crate::state::chat::AgentMessage {
                    role: crate::state::chat::MessageRole::Assistant,
                    content: "B".repeat(400),
                    ..Default::default()
                },
                crate::state::chat::AgentMessage {
                    role: crate::state::chat::MessageRole::Assistant,
                    content: "C".repeat(400),
                    ..Default::default()
                },
                crate::state::chat::AgentMessage {
                    role: crate::state::chat::MessageRole::User,
                    content: "D".repeat(400),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
    ));
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-boundary".to_string(),
    ));

    let usage = model.current_header_usage_summary();
    assert_eq!(
        usage.current_tokens, 0,
        "header should not estimate active context usage from loaded messages without daemon context state"
    );
}

#[test]
fn header_usage_summary_does_not_estimate_on_legacy_visible_compaction_artifact() {
    let mut model = make_model();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-legacy-compaction".to_string(),
        title: "Legacy Compaction".to_string(),
        total_message_count: 4,
        loaded_message_start: 0,
        loaded_message_end: 4,
        messages: vec![
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::User,
                content: "A".repeat(4_000),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: "B".repeat(4_000),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: "Pre-compaction context: ~842,460 / 400,000 tokens (threshold 320,000)\nTrigger: token-threshold\nStrategy: custom model generated summary.".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: "short follow-up".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-legacy-compaction".to_string(),
    ));

    let usage = model.current_header_usage_summary();
    assert_eq!(
        usage.current_tokens, 0,
        "legacy compaction artifacts should not drive a fallback header context estimate without daemon fields"
    );
}

#[test]
fn header_usage_summary_prefers_daemon_active_context_window_tokens() {
    let mut model = make_model();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-authoritative-context".to_string(),
        title: "Authoritative Context".to_string(),
        total_message_count: 4,
        loaded_message_start: 3,
        loaded_message_end: 4,
        active_context_window_start: Some(2),
        active_context_window_end: Some(4),
        active_context_window_tokens: Some(54),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::User,
            content: "C".repeat(80),
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-authoritative-context".to_string(),
    ));

    let usage = model.current_header_usage_summary();
    assert_eq!(
        usage.current_tokens, 54,
        "header should use daemon-calculated active context tokens instead of estimating the loaded page"
    );
}

#[test]
fn header_usage_summary_uses_delivered_context_window_update_before_messages() {
    let mut model = make_model();
    model.config.provider = "unknown-provider".to_string();
    model.config.model = "unknown-model".to_string();
    model.config.context_window_tokens = 1_000;

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-delivered-context".to_string(),
        title: "Delivered Context".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-delivered-context".to_string(),
    ));

    assert_eq!(model.current_header_usage_summary().current_tokens, 0);

    model.handle_client_event(ClientEvent::ContextWindowUpdate {
        thread_id: "thread-delivered-context".to_string(),
        active_context_window_start: 0,
        active_context_window_end: 1,
        active_context_window_tokens: 24_000,
    });

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.current_tokens, 24_000);
    assert!(usage.utilization_pct > 0);
}

#[test]
fn header_usage_summary_does_not_estimate_context_tokens_from_loaded_history() {
    let mut model = make_model();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-history-context".to_string(),
        title: "History Context".to_string(),
        total_message_count: 6,
        loaded_message_start: 4,
        loaded_message_end: 6,
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::Assistant,
            content: "latest".to_string(),
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-history-context".to_string(),
    ));

    let before = model.current_header_usage_summary();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-history-context".to_string(),
        title: "History Context".to_string(),
        total_message_count: 6,
        loaded_message_start: 0,
        loaded_message_end: 4,
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::User,
            content: "older ".repeat(8_000),
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });

    let after = model.current_header_usage_summary();
    assert_eq!(before.current_tokens, 0);
    assert_eq!(
        after.current_tokens, before.current_tokens,
        "loading older history should not fake active context usage when daemon context state is absent"
    );
}

#[test]
fn header_usage_summary_preserves_daemon_context_tokens_when_loading_history() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-turn-usage".to_string(),
        title: "Turn Usage".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-turn-usage".to_string(),
    ));
    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-turn-usage".to_string(),
        content: "latest response".to_string(),
    });
    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-turn-usage".to_string(),
        input_tokens: 12_000,
        output_tokens: 3_000,
        cost: Some(0.25),
        provider: Some("openai".to_string()),
        model: Some("gpt-5.4".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: None,
    });
    model.handle_client_event(ClientEvent::ContextWindowUpdate {
        thread_id: "thread-turn-usage".to_string(),
        active_context_window_start: 0,
        active_context_window_end: 1,
        active_context_window_tokens: 15_000,
    });

    let before = model.current_header_usage_summary();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-turn-usage".to_string(),
        title: "Turn Usage".to_string(),
        total_message_count: 4,
        loaded_message_start: 0,
        loaded_message_end: 2,
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::User,
            content: "older ".repeat(8_000),
            input_tokens: 80_000,
            output_tokens: 20_000,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });

    let after = model.current_header_usage_summary();
    assert_eq!(
        before.current_tokens, 15_000,
        "header should use the daemon-reported active context window tokens"
    );
    assert_eq!(
        after.current_tokens, before.current_tokens,
        "loading older history should not replace authoritative daemon context tokens with per-message estimates"
    );
}

#[test]
fn internal_dm_thread_created_does_not_hijack_active_thread() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-user".to_string(),
        title: "User Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "dm:svarog:weles".to_string(),
        title: "Internal DM · Swarog ↔ WELES".to_string(),
        agent_name: None,
    });

    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
}

#[test]
fn hidden_handoff_thread_created_does_not_hijack_active_thread() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-user".to_string(),
        title: "User Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "handoff:thread-user:handoff-1".to_string(),
        title: "Handoff · Svarog -> Weles".to_string(),
        agent_name: None,
    });

    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
    assert!(
        model
            .chat
            .threads()
            .iter()
            .all(|thread| thread.id != "handoff:thread-user:handoff-1"),
        "hidden handoff threads should not be added to visible chat state"
    );
}

#[test]
fn thread_created_event_preserves_agent_name_for_responder_fallback() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-weles".to_string(),
        title: "Governance".to_string(),
        agent_name: Some("Weles".to_string()),
    });

    let thread = model
        .chat
        .threads()
        .iter()
        .find(|thread| thread.id == "thread-weles")
        .expect("thread should be added to chat state");

    assert_eq!(thread.agent_name.as_deref(), Some("Weles"));
}

#[test]
fn hidden_handoff_thread_detail_is_ignored() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-user".to_string(),
        title: "User Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-user".to_string()));

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "handoff:thread-user:handoff-1".to_string(),
        title: "Handoff · Svarog -> Weles".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::System,
            content: "{\"kind\":\"thread_handoff_context\"}".to_string(),
            timestamp: 1,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    assert_eq!(model.chat.active_thread_id(), Some("thread-user"));
    assert!(
        model
            .chat
            .threads()
            .iter()
            .all(|thread| thread.id != "handoff:thread-user:handoff-1"),
        "hidden handoff thread detail should not populate visible chat state"
    );
}
