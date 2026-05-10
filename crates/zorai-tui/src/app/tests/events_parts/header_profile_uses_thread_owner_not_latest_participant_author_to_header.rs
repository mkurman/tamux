use super::done_event_persists_final_reasoning_into_chat_message_to_mission_control::*;
use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn header_profile_uses_thread_owner_not_latest_participant_author() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;
    model.subagents.entries.push(crate::state::SubAgentEntry {
        id: "mokosh".to_string(),
        name: "Mokosh".to_string(),
        provider: "alibaba-coding-plan".to_string(),
        model: "glm-5".to_string(),
        role: Some("testing".to_string()),
        enabled: true,
        builtin: true,
        immutable_identity: true,
        disable_allowed: false,
        delete_allowed: false,
        protected_reason: None,
        reasoning_effort: Some("medium".to_string()),
        openrouter_provider_order: String::new(),
        openrouter_provider_ignore: String::new(),
        openrouter_allow_fallbacks: true,
        raw_json: None,
    });

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-owner-main".to_string(),
        agent_name: Some("Swarog".to_string()),
        title: "Main-owned thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::Assistant,
            content: "Participant contribution".to_string(),
            author_agent_id: Some("mokosh".to_string()),
            author_agent_name: Some("Mokosh".to_string()),
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        total_input_tokens: 10,
        total_output_tokens: 20,
        loaded_message_start: 0,
        loaded_message_end: 1,
        total_message_count: 1,
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-owner-main".to_string(),
    ));

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Swarog");
    assert_eq!(profile.provider, PROVIDER_ID_GITHUB_COPILOT);
    assert_eq!(profile.model, "gpt-5.4");

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.context_window_tokens, 400_000);
}

#[test]
fn header_falls_back_to_goal_planner_owner_profile_for_goal_run_pane() {
    let mut model = make_model();
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
        goal_run_id: "goal-2".to_string(),
        step_id: None,
    });
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_for_header_tests(
            "goal-2",
            None,
            Some(make_goal_owner_profile(
                "Planner Owner",
                "planner-provider",
                "planner-model",
                Some("planner-effort"),
            )),
            None,
        ),
    ));

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Planner Owner");
    assert_eq!(profile.provider, "planner-provider");
    assert_eq!(profile.model, "planner-model");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("planner-effort"));
}

#[test]
fn mission_control_header_falls_back_to_root_thread_runtime_when_active_thread_missing() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-root-fallback".to_string(),
        title: "Root Thread".to_string(),
        agent_name: Some("Swarog".to_string()),
    });
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-root-fallback".to_string(),
        title: "Root Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::Assistant,
            content: "Root output".to_string(),
            cost: Some(0.5),
            input_tokens: 12,
            output_tokens: 18,
            message_kind: "normal".to_string(),
            ..Default::default()
        }],
        total_input_tokens: 12,
        total_output_tokens: 18,
        loaded_message_start: 0,
        loaded_message_end: 1,
        total_message_count: 1,
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-root-fallback".to_string(),
    ));
    if let Some(thread) = model.chat.active_thread_mut() {
        thread.runtime_provider = Some("alibaba-coding-plan".to_string());
        thread.runtime_model = Some("MiniMax-M2.5".to_string());
        thread.runtime_reasoning_effort = Some("high".to_string());
    }

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-root-fallback".to_string(),
        step_id: None,
    });
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-root-fallback".to_string(),
            title: "Goal".to_string(),
            thread_id: Some("thread-legacy".to_string()),
            root_thread_id: Some("thread-root-fallback".to_string()),
            active_thread_id: None,
            ..Default::default()
        }));

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.provider, "alibaba-coding-plan");
    assert_eq!(profile.model, "MiniMax-M2.5");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("high"));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.total_thread_tokens, 30);
    assert_eq!(usage.total_cost_usd, Some(0.5));
    assert_eq!(usage.context_window_tokens, 205_000);
}

#[test]
fn header_uses_thread_profile_metadata_before_runtime_metadata_arrives() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-dazhbog-profile".to_string(),
        title: "Spawned thread".to_string(),
        agent_name: Some("Dazhbog".to_string()),
    });
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-dazhbog-profile".to_string(),
        agent_name: Some("Dazhbog".to_string()),
        profile_provider: Some("alibaba-coding-plan".to_string()),
        profile_model: Some("kimi-k2.5".to_string()),
        profile_reasoning_effort: Some("high".to_string()),
        profile_context_window_tokens: Some(240_000),
        title: "Spawned thread".to_string(),
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    });
    model.chat.reduce(chat::ChatAction::SelectThread(
        "thread-dazhbog-profile".to_string(),
    ));

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Dazhbog");
    assert_eq!(profile.provider, "alibaba-coding-plan");
    assert_eq!(profile.model, "kimi-k2.5");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("high"));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.context_window_tokens, 240_000);
}

#[test]
fn mission_control_header_context_window_tracks_active_execution_thread_changes() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;

    for (thread_id, title, provider, model_id) in [
        ("thread-root-window", "Root Thread", "openai", "gpt-5.4"),
        (
            "thread-active-window",
            "Active Thread",
            "alibaba-coding-plan",
            "MiniMax-M2.5",
        ),
    ] {
        model.handle_client_event(ClientEvent::ThreadCreated {
            thread_id: thread_id.to_string(),
            title: title.to_string(),
            agent_name: Some("Swarog".to_string()),
        });
        model.handle_thread_detail_event(crate::wire::AgentThread {
            id: thread_id.to_string(),
            title: title.to_string(),
            messages: vec![crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: format!("{title} output"),
                input_tokens: 10,
                output_tokens: 10,
                message_kind: "normal".to_string(),
                ..Default::default()
            }],
            total_input_tokens: 10,
            total_output_tokens: 10,
            loaded_message_start: 0,
            loaded_message_end: 1,
            total_message_count: 1,
            created_at: 1,
            updated_at: 1,
            ..Default::default()
        });
        model
            .chat
            .reduce(chat::ChatAction::SelectThread(thread_id.to_string()));
        if let Some(thread) = model.chat.active_thread_mut() {
            thread.runtime_provider = Some(provider.to_string());
            thread.runtime_model = Some(model_id.to_string());
        }
    }

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-window-switch".to_string(),
        step_id: None,
    });
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-window-switch".to_string(),
            title: "Goal".to_string(),
            thread_id: Some("thread-root-window".to_string()),
            root_thread_id: Some("thread-root-window".to_string()),
            active_thread_id: Some("thread-root-window".to_string()),
            ..Default::default()
        }));

    let initial = model.current_header_usage_summary();
    assert_eq!(initial.context_window_tokens, 1_000_000);

    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-window-switch".to_string(),
            title: "Goal".to_string(),
            thread_id: Some("thread-root-window".to_string()),
            root_thread_id: Some("thread-root-window".to_string()),
            active_thread_id: Some("thread-active-window".to_string()),
            ..Default::default()
        }));

    let updated = model.current_header_usage_summary();
    assert_eq!(updated.context_window_tokens, 1_000_000);
}

#[test]
fn header_goal_run_pane_ignores_unrelated_conversation_runtime_metadata() {
    let mut model = make_model();
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
        goal_run_id: "goal-3".to_string(),
        step_id: None,
    });
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_for_header_tests(
            "goal-3",
            None,
            Some(make_goal_owner_profile(
                "Planner Owner",
                "planner-provider",
                "planner-model",
                Some("planner-effort"),
            )),
            Some(make_goal_owner_profile(
                "Current Step Owner",
                "step-provider",
                "step-model",
                Some("step-effort"),
            )),
        ),
    ));

    let profile = model.current_header_agent_profile();
    assert_ne!(profile.provider, "conversation-provider");
    assert_ne!(profile.model, "conversation-model");
    assert_ne!(
        profile.reasoning_effort.as_deref(),
        Some("conversation-effort")
    );
    assert_eq!(profile.provider, "step-provider");
    assert_eq!(profile.model, "step-model");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("step-effort"));
}

#[test]
fn mission_control_header_falls_back_to_launch_snapshot_before_generic_defaults() {
    let mut model = make_model();
    model.config.provider = "provider-generic".to_string();
    model.config.model = "model-generic".to_string();
    model.config.reasoning_effort = "low".to_string();
    model.config.context_window_tokens = 400_000;

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-launch-snapshot".to_string(),
        step_id: None,
    });
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-launch-snapshot".to_string(),
            title: "Goal".to_string(),
            launch_assignment_snapshot: vec![make_goal_assignment(
                zorai_protocol::AGENT_ID_SWAROG,
                "alibaba-coding-plan",
                "MiniMax-M2.5",
                Some("high"),
            )],
            ..Default::default()
        }));

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Swarog");
    assert_eq!(profile.provider, "alibaba-coding-plan");
    assert_eq!(profile.model, "MiniMax-M2.5");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("high"));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.total_thread_tokens, 0);
    assert_eq!(usage.context_window_tokens, 205_000);
}

#[test]
fn header_uses_goal_thread_usage_when_goal_run_thread_exists() {
    let mut model = make_model();
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
            content: "active thread content that should not be used".repeat(50),
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
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-goal".to_string(),
        title: "Goal Thread".to_string(),
        messages: vec![
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::User,
                content: "goal".to_string(),
                cost: Some(0.25),
                message_kind: "normal".to_string(),
                ..Default::default()
            },
            crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: "result".to_string(),
                cost: Some(0.75),
                message_kind: "normal".to_string(),
                ..Default::default()
            },
        ],
        total_input_tokens: 40,
        total_output_tokens: 60,
        loaded_message_start: 0,
        loaded_message_end: 2,
        total_message_count: 2,
        created_at: 2,
        updated_at: 2,
        ..Default::default()
    });

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-usage".to_string(),
        step_id: None,
    });
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_for_header_tests(
            "goal-usage",
            Some("thread-goal"),
            Some(make_goal_owner_profile(
                "Planner Owner",
                "planner-provider",
                "planner-model",
                None,
            )),
            Some(make_goal_owner_profile(
                "Current Step Owner",
                "step-provider",
                "step-model",
                None,
            )),
        ),
    ));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.total_thread_tokens, 100);
    assert_eq!(usage.total_cost_usd, Some(1.0));
    assert_eq!(usage.current_tokens, 0);
    assert_ne!(usage.total_thread_tokens, 2_000);
    assert_ne!(usage.total_cost_usd, Some(9.99));
}
