use super::done_event_persists_final_reasoning_into_chat_message_to_mission_control::*;
use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn header_goal_run_usage_defaults_when_goal_thread_missing() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-active".to_string(),
        title: "Active Thread".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-active".to_string()));
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-active".to_string(),
        title: "Active Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::User,
            content: "active thread content that should not be borrowed".repeat(50),
            cost: Some(9.99),
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        total_input_tokens: 900,
        total_output_tokens: 1_100,
        loaded_message_start: 0,
        loaded_message_end: 1,
        total_message_count: 1,
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    });

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-missing-thread".to_string(),
        step_id: None,
    });
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_for_header_tests(
            "goal-missing-thread",
            Some("goal-thread-missing"),
            Some(make_goal_owner_profile(
                "Planner Owner",
                "planner-provider",
                "planner-model",
                None,
            )),
            None,
        ),
    ));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.total_thread_tokens, 0);
    assert_eq!(usage.current_tokens, 0);
    assert_eq!(usage.total_cost_usd, None);
    assert_eq!(usage.context_window_tokens, 400_000);
    assert!(usage.compaction_target_tokens > 0);
    assert_eq!(usage.utilization_pct, 0);
}

#[test]
fn header_goal_run_uses_generic_config_defaults_when_owner_profiles_missing() {
    let mut model = make_model();
    model.config.provider = "provider-generic".to_string();
    model.config.model = "model-generic".to_string();
    model.config.reasoning_effort = "low".to_string();
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-active".to_string(),
        title: "Active Thread".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-active".to_string()));
    if let Some(thread) = model.chat.active_thread_mut() {
        thread.runtime_provider = Some("conversation-provider".to_string());
        thread.runtime_model = Some("conversation-model".to_string());
        thread.runtime_reasoning_effort = Some("conversation-effort".to_string());
    }

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-generic".to_string(),
        step_id: None,
    });
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_for_header_tests("goal-generic", None, None, None),
    ));

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Swarog");
    assert_eq!(profile.provider, "provider-generic");
    assert_eq!(profile.model, "model-generic");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("low"));
}

#[test]
fn header_usage_summary_uses_runtime_model_context_window_for_rarog() {
    let mut model = make_model();
    model.concierge.provider = Some("alibaba-coding-plan".to_string());
    model.concierge.model = Some("qwen3.6-plus".to_string());

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-rarog-usage".to_string(),
        title: "Rarog Thread".to_string(),
        agent_name: Some("Rarog".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-rarog-usage".to_string(),
    ));
    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-rarog-usage".to_string(),
        content: "Runtime answer".to_string(),
    });
    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-rarog-usage".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: Some(0.25),
        provider: Some("alibaba-coding-plan".to_string()),
        model: Some("MiniMax-M2.5".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: None,
    });
    model.handle_client_event(ClientEvent::ContextWindowUpdate {
        thread_id: "thread-rarog-usage".to_string(),
        active_context_window_start: 0,
        active_context_window_end: 1,
        active_context_window_tokens: 30,
    });

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.context_window_tokens, 205_000);
    assert_eq!(usage.compaction_target_tokens, 164_000);
    let total_cost = usage
        .total_cost_usd
        .expect("header should expose total cost");
    assert!(
        (total_cost - 0.25).abs() < 1e-9,
        "expected summed total cost to be 0.25, got {total_cost}"
    );
    assert_eq!(
        usage.current_tokens, 30,
        "header should use the daemon-reported active context window tokens"
    );
    assert!(usage.utilization_pct <= 100);
}

#[test]
fn header_profile_tracks_repeated_swarog_config_changes_after_runtime_metadata_exists() {
    let mut model = make_model();
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-config".to_string(),
        title: "Config".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-config".to_string()));

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-config".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: None,
        provider: Some("provider-old".to_string()),
        model: Some("model-old".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: None,
    });

    let initial = model.current_header_agent_profile();
    assert_eq!(initial.provider, "provider-old");
    assert_eq!(initial.model, "model-old");

    model.handle_agent_config_event(crate::wire::AgentConfigSnapshot {
        provider: "provider-a".to_string(),
        base_url: "https://example.invalid/v1".to_string(),
        model: "model-a".to_string(),
        api_key: String::new(),
        assistant_id: String::new(),
        auth_source: "api_key".to_string(),
        api_transport: "responses".to_string(),
        reasoning_effort: "high".to_string(),
        context_window_tokens: 200_000,
    });

    let first = model.current_header_agent_profile();
    assert_eq!(first.provider, "provider-a");
    assert_eq!(first.model, "model-a");

    model.handle_agent_config_event(crate::wire::AgentConfigSnapshot {
        provider: "provider-b".to_string(),
        base_url: "https://example.invalid/v1".to_string(),
        model: "model-b".to_string(),
        api_key: String::new(),
        assistant_id: String::new(),
        auth_source: "api_key".to_string(),
        api_transport: "responses".to_string(),
        reasoning_effort: "medium".to_string(),
        context_window_tokens: 400_000,
    });

    let second = model.current_header_agent_profile();
    assert_eq!(second.provider, "provider-b");
    assert_eq!(second.model, "model-b");
}

#[test]
fn header_usage_summary_uses_primary_threshold_for_heuristic_compaction() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;
    model.config.compact_threshold_pct = 80;
    model.config.compaction_strategy = "heuristic".to_string();

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-heuristic-target".to_string(),
        title: "Heuristic".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-heuristic-target".to_string(),
    ));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-heuristic-target".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::User,
            content: "A".repeat(20_000),
            ..Default::default()
        },
    });

    let usage = model.current_header_usage_summary();
    assert_eq!(
        usage.compaction_target_tokens, 320_000,
        "heuristic compaction should use the main model threshold"
    );
    assert_eq!(usage.context_window_tokens, 400_000);
}

#[test]
fn header_profile_tracks_weles_subagent_updates_after_runtime_metadata_exists() {
    let mut model = make_model();
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "weles_builtin".to_string(),
        name: "WELES".to_string(),
        provider: "provider-old".to_string(),
        model: "model-old".to_string(),
        role: Some("testing".to_string()),
        enabled: true,
        builtin: true,
        immutable_identity: true,
        disable_allowed: false,
        delete_allowed: false,
        protected_reason: Some("Built-in reviewer".to_string()),
        reasoning_effort: Some("medium".to_string()),
        openrouter_provider_order: String::new(),
        openrouter_provider_ignore: String::new(),
        openrouter_allow_fallbacks: true,
        raw_json: None,
    });
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-weles-runtime".to_string(),
        title: "Reviewer Runtime".to_string(),
        agent_name: Some("Weles".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-weles-runtime".to_string(),
    ));
    if let Some(thread) = model.chat.active_thread_mut() {
        thread.runtime_provider = Some("provider-runtime".to_string());
        thread.runtime_model = Some("model-runtime".to_string());
    }

    let before = model.current_header_agent_profile();
    assert_eq!(before.provider, "provider-runtime");
    assert_eq!(before.model, "model-runtime");

    model.handle_subagent_updated_event(crate::state::SubAgentEntry {
        id: "weles_builtin".to_string(),
        name: "WELES".to_string(),
        provider: "provider-new".to_string(),
        model: "model-new".to_string(),
        role: Some("testing".to_string()),
        enabled: true,
        builtin: true,
        immutable_identity: true,
        disable_allowed: false,
        delete_allowed: false,
        protected_reason: Some("Built-in reviewer".to_string()),
        reasoning_effort: Some("high".to_string()),
        openrouter_provider_order: String::new(),
        openrouter_provider_ignore: String::new(),
        openrouter_allow_fallbacks: true,
        raw_json: None,
    });

    let after = model.current_header_agent_profile();
    assert_eq!(after.provider, "provider-new");
    assert_eq!(after.model, "model-new");
    assert_eq!(after.reasoning_effort.as_deref(), Some("high"));
}

#[test]
fn header_profile_switches_on_handoff_append_and_clears_stale_runtime() {
    let mut model = make_model();
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "weles_builtin".to_string(),
        name: "Weles".to_string(),
        provider: "provider-weles".to_string(),
        model: "model-weles".to_string(),
        role: Some("review".to_string()),
        enabled: true,
        builtin: true,
        immutable_identity: true,
        disable_allowed: false,
        delete_allowed: false,
        protected_reason: Some("Built-in reviewer".to_string()),
        reasoning_effort: Some("high".to_string()),
        openrouter_provider_order: String::new(),
        openrouter_provider_ignore: String::new(),
        openrouter_allow_fallbacks: true,
        raw_json: None,
    });
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-handoff".to_string(),
        title: "Handoff".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-handoff".to_string()));
    if let Some(thread) = model.chat.active_thread_mut() {
        thread.runtime_provider = Some("provider-runtime".to_string());
        thread.runtime_model = Some("model-runtime".to_string());
    }

    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-handoff".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::System,
            content: "[[handoff_event]]{\"to_agent_id\":\"weles\",\"to_agent_name\":\"Weles\"}\nSwarog handed this thread to Weles.".to_string(),
            ..Default::default()
        },
    });

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Swarog");
    assert_eq!(profile.provider, model.config.provider);
    assert_eq!(profile.model, model.config.model);
    let thread = model
        .chat
        .active_thread()
        .expect("thread should remain active");
    assert_eq!(thread.runtime_provider, None);
    assert_eq!(thread.runtime_model, None);
}

#[test]
fn header_profile_resets_on_return_handoff_after_thread_detail_refresh() {
    let mut model = make_model();
    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-return".to_string(),
        title: "Return".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-return".to_string()));
    if let Some(thread) = model.chat.active_thread_mut() {
        thread.runtime_provider = Some("provider-weles-runtime".to_string());
        thread.runtime_model = Some("model-weles-runtime".to_string());
    }
    model.config.provider = "provider-svarog".to_string();
    model.config.model = "model-svarog".to_string();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-return".to_string(),
        title: "Return".to_string(),
        agent_name: Some("Swarog".to_string()),
        messages: vec![
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::System,
                content: "[[handoff_event]]{\"to_agent_id\":\"weles\",\"to_agent_name\":\"Weles\"}\nSwarog handed this thread to Weles.".to_string(),
                timestamp: 1,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::System,
                content: "[[handoff_event]]{\"to_agent_id\":\"svarog\",\"to_agent_name\":\"Swarog\"}\nWeles handed this thread to Swarog.".to_string(),
                timestamp: 2,
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        total_message_count: 2,
        loaded_message_start: 0,
        loaded_message_end: 2,
        created_at: 1,
        updated_at: 2,
        ..Default::default()
    });

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Swarog");
    assert_eq!(profile.provider, "provider-svarog");
    assert_eq!(profile.model, "model-svarog");
    let thread = model
        .chat
        .active_thread()
        .expect("thread should remain active");
    assert_eq!(thread.runtime_provider, None);
    assert_eq!(thread.runtime_model, None);
}

#[test]
fn header_usage_summary_caps_target_by_weles_compaction_window() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;
    model.config.compact_threshold_pct = 80;
    model.config.compaction_strategy = "weles".to_string();
    model.config.compaction_weles_provider = "minimax-coding-plan".to_string();
    model.config.compaction_weles_model = "MiniMax-M2.7".to_string();

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-weles-target".to_string(),
        title: "Reviewer Target".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-weles-target".to_string(),
    ));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.compaction_target_tokens, 164_000);
    assert_eq!(usage.context_window_tokens, 400_000);
}
