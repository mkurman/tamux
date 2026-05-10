use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
use zorai_shared::providers::*;
#[test]
fn done_event_persists_final_reasoning_into_chat_message() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::Delta {
        thread_id: "thread-1".to_string(),
        content: "Answer".to_string(),
    });

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-1".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: None,
        provider: Some(PROVIDER_ID_GITHUB_COPILOT.to_string()),
        model: Some("gpt-5.4".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: Some("Final reasoning summary".to_string()),
        provider_final_result_json: Some("result_json".to_string()),
    });

    let thread = model.chat.active_thread().expect("thread should exist");
    let last = thread
        .messages
        .last()
        .expect("assistant message should exist");
    assert_eq!(last.reasoning.as_deref(), Some("Final reasoning summary"));
}

#[test]
fn done_event_does_not_force_authoritative_refresh_for_participant_threads() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::User,
            content: "hello".to_string(),
            timestamp: 1,
            ..Default::default()
        }],
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "weles".to_string(),
            agent_name: "Weles".to_string(),
            instruction: "verify claims".to_string(),
            status: "active".to_string(),
            created_at: 1,
            updated_at: 1,
            last_contribution_at: None,
            deactivated_at: None,
            always_auto_response: false,
        }],
        created_at: 1,
        updated_at: 1,
        ..Default::default()
    })));

    while daemon_rx.try_recv().is_ok() {}

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-1".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: None,
        provider: Some(PROVIDER_ID_GITHUB_COPILOT.to_string()),
        model: Some("gpt-5.4".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: Some("result_json".to_string()),
    });

    assert!(
        next_thread_request(&mut daemon_rx).is_none(),
        "participant-thread done should rely on daemon reload events instead of forcing a TUI refresh"
    );
}

#[test]
fn thread_detail_event_hydrates_pinned_for_compaction_from_wire() {
    let mut model = make_model();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Pinned".to_string(),
        messages: vec![crate::wire::AgentMessage {
            id: Some("message-1".to_string()),
            role: crate::wire::MessageRole::Assistant,
            content: "Pinned content".to_string(),
            pinned_for_compaction: true,
            ..Default::default()
        }],
        loaded_message_end: 1,
        total_message_count: 1,
        ..Default::default()
    });

    let thread = model.chat.active_thread().expect("thread should exist");
    assert!(thread.messages[0].pinned_for_compaction);
}

#[test]
fn thread_detail_event_hydrates_tool_output_preview_path_from_wire() {
    let mut model = make_model();

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Preview".to_string(),
        messages: vec![crate::wire::AgentMessage {
            id: Some("message-1".to_string()),
            role: crate::wire::MessageRole::Tool,
            tool_name: Some("bash_command".to_string()),
            tool_status: Some("done".to_string()),
            tool_output_preview_path: Some(
                "/tmp/.zorai/.cache/tools/thread-thread-1/bash_command-1700000123.txt".to_string(),
            ),
            content: "Tool result saved to preview file".to_string(),
            ..Default::default()
        }],
        loaded_message_end: 1,
        total_message_count: 1,
        ..Default::default()
    });

    let thread = model.chat.active_thread().expect("thread should exist");
    assert_eq!(
        thread.messages[0].tool_output_preview_path.as_deref(),
        Some("/tmp/.zorai/.cache/tools/thread-thread-1/bash_command-1700000123.txt")
    );
}

#[test]
fn stale_retry_status_after_done_does_not_restore_retrying_placeholder() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::User,
            content: "Retry this".to_string(),
            ..Default::default()
        },
    });

    model.handle_client_event(ClientEvent::RetryStatus {
        thread_id: "thread-1".to_string(),
        phase: "retrying".to_string(),
        attempt: 1,
        max_retries: 3,
        delay_ms: 1_000,
        failure_class: "temporary_upstream".to_string(),
        message: "upstream timeout".to_string(),
    });
    assert_eq!(model.footer_activity_text().as_deref(), Some("retrying"));
    assert!(model.chat.retry_status().is_some());

    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-1".to_string(),
        content: "Recovered answer".to_string(),
    });

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-1".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: None,
        provider: Some(PROVIDER_ID_GITHUB_COPILOT.to_string()),
        model: Some("gpt-5.4".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: Some("result_json".to_string()),
    });

    assert!(
        model.footer_activity_text().is_none(),
        "done should clear the retrying placeholder"
    );
    assert!(
        model.chat.retry_status().is_none(),
        "done should clear visible retry status"
    );

    model.handle_client_event(ClientEvent::RetryStatus {
        thread_id: "thread-1".to_string(),
        phase: "retrying".to_string(),
        attempt: 1,
        max_retries: 3,
        delay_ms: 1_000,
        failure_class: "temporary_upstream".to_string(),
        message: "late retry status".to_string(),
    });

    assert!(
        model.footer_activity_text().is_none(),
        "late retry events for a completed turn must not restore the retrying placeholder"
    );
    assert!(
        model.chat.retry_status().is_none(),
        "late retry events for a completed turn must not restore retry status UI"
    );
}

#[test]
fn header_uses_rarog_daemon_runtime_metadata_after_first_reply() {
    let mut model = make_model();
    model.concierge.provider = Some("alibaba-coding-plan".to_string());
    model.concierge.model = Some("qwen3.6-plus".to_string());
    model.concierge.reasoning_effort = Some("none".to_string());

    model.handle_client_event(ClientEvent::ThreadCreated {
        thread_id: "thread-rarog".to_string(),
        title: "Rarog Thread".to_string(),
        agent_name: Some("Rarog".to_string()),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-rarog".to_string()));
    model.handle_client_event(ClientEvent::Delta {
        thread_id: "thread-rarog".to_string(),
        content: "Runtime answer".to_string(),
    });

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-rarog".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: None,
        provider: Some("alibaba-coding-plan".to_string()),
        model: Some("MiniMax-M2.5".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: Some(r#"{"reasoning":{"effort":"low"}}"#.to_string()),
    });

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.agent_label, "Rarog");
    assert_eq!(profile.provider, "alibaba-coding-plan");
    assert_eq!(profile.model, "MiniMax-M2.5");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("low"));
}

pub(super) fn make_goal_owner_profile(
    agent_label: &str,
    provider: &str,
    model: &str,
    reasoning_effort: Option<&str>,
) -> task::GoalRuntimeOwnerProfile {
    task::GoalRuntimeOwnerProfile {
        agent_label: agent_label.to_string(),
        provider: provider.to_string(),
        model: model.to_string(),
        reasoning_effort: reasoning_effort.map(str::to_string),
    }
}

pub(super) fn make_goal_run_for_header_tests(
    goal_run_id: &str,
    thread_id: Option<&str>,
    planner_owner_profile: Option<task::GoalRuntimeOwnerProfile>,
    current_step_owner_profile: Option<task::GoalRuntimeOwnerProfile>,
) -> task::GoalRun {
    task::GoalRun {
        id: goal_run_id.to_string(),
        title: "Goal".to_string(),
        thread_id: thread_id.map(str::to_string),
        planner_owner_profile,
        current_step_owner_profile,
        ..Default::default()
    }
}

pub(super) fn make_goal_assignment(
    role_id: &str,
    provider: &str,
    model: &str,
    reasoning_effort: Option<&str>,
) -> task::GoalAgentAssignment {
    task::GoalAgentAssignment {
        role_id: role_id.to_string(),
        enabled: true,
        provider: provider.to_string(),
        model: model.to_string(),
        reasoning_effort: reasoning_effort.map(str::to_string),
        inherit_from_main: false,
    }
}

#[test]
fn header_uses_goal_current_step_owner_profile_for_goal_run_pane() {
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
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });
    model.tasks.reduce(task::TaskAction::GoalRunDetailReceived(
        make_goal_run_for_header_tests(
            "goal-1",
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
    assert_eq!(profile.agent_label, "Current Step Owner");
    assert_eq!(profile.provider, "step-provider");
    assert_eq!(profile.model, "step-model");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("step-effort"));
}

#[test]
fn mission_control_header_prefers_active_execution_thread_runtime_and_usage() {
    let mut model = make_model();
    model.config.provider = PROVIDER_ID_GITHUB_COPILOT.to_string();
    model.config.auth_source = "github_copilot".to_string();
    model.config.model = "gpt-5.4".to_string();
    model.config.context_window_tokens = 400_000;

    for (thread_id, title, input_tokens, output_tokens, cost, provider, model_id, effort) in [
        (
            "thread-root",
            "Root Thread",
            40_u64,
            60_u64,
            Some(1.0_f64),
            "openai",
            "gpt-5.4",
            Some("medium"),
        ),
        (
            "thread-active",
            "Active Thread",
            10_u64,
            20_u64,
            Some(0.25_f64),
            "alibaba-coding-plan",
            "MiniMax-M2.5",
            Some("high"),
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
                cost,
                input_tokens,
                output_tokens,
                message_kind: "normal".to_string(),
                ..Default::default()
            }],
            total_input_tokens: input_tokens,
            total_output_tokens: output_tokens,
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
            thread.runtime_reasoning_effort = effort.map(str::to_string);
        }
    }

    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-mission-control-active".to_string(),
        step_id: None,
    });
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-mission-control-active".to_string(),
            title: "Goal".to_string(),
            thread_id: Some("thread-root".to_string()),
            root_thread_id: Some("thread-root".to_string()),
            active_thread_id: Some("thread-active".to_string()),
            planner_owner_profile: Some(make_goal_owner_profile(
                "Planner Owner",
                "planner-provider",
                "planner-model",
                Some("low"),
            )),
            current_step_owner_profile: Some(make_goal_owner_profile(
                "Current Step Owner",
                "step-provider",
                "step-model",
                Some("medium"),
            )),
            ..Default::default()
        }));

    let profile = model.current_header_agent_profile();
    assert_eq!(profile.provider, "openai");
    assert_eq!(profile.model, "gpt-5.4");
    assert_eq!(profile.reasoning_effort.as_deref(), Some("medium"));

    let usage = model.current_header_usage_summary();
    assert_eq!(usage.total_thread_tokens, 100);
    assert_eq!(usage.total_cost_usd, Some(1.0));
    assert_eq!(usage.context_window_tokens, 1_000_000);
}
