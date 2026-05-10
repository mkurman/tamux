use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
use tokio::sync::mpsc::unbounded_channel;
#[test]
fn status_diagnostics_warning_mentions_sync_state() {
    let mut model = make_model();
    model.handle_client_event(ClientEvent::StatusDiagnostics {
        operator_profile_sync_state: "dirty".to_string(),
        operator_profile_sync_dirty: true,
        operator_profile_scheduler_fallback: false,
        diagnostics_json: r#"{"operator_profile_sync_state":"dirty","aline":{"available":true}}"#
            .to_string(),
    });
    assert!(
        model.status_line.contains("sync state: dirty"),
        "status line should expose dirty sync diagnostics"
    );
    assert!(
        model
            .status_modal_diagnostics_json
            .as_deref()
            .is_some_and(|diagnostics| diagnostics.contains("\"aline\"")),
        "status diagnostics should retain the raw payload for the status modal"
    );
}

#[test]
fn status_diagnostics_warning_mentions_mesh_state() {
    let mut model = make_model();
    model.handle_client_event(ClientEvent::StatusDiagnostics {
        operator_profile_sync_state: "clean".to_string(),
        operator_profile_sync_dirty: false,
        operator_profile_scheduler_fallback: false,
        diagnostics_json: r#"{"skill_mesh":{"backend":"mesh","state":"degraded"}}"#.to_string(),
    });
    assert!(
        model.status_line.contains("skill mesh: degraded"),
        "status line should expose degraded mesh diagnostics"
    );
}

#[test]
fn full_status_event_caches_snapshot_for_status_modal() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::StatusSnapshot(
        crate::client::AgentStatusSnapshotVm {
            tier: "mission_control".to_string(),
            activity: "waiting_for_operator".to_string(),
            active_thread_id: Some("thread-1".to_string()),
            active_goal_run_id: Some("goal-1".to_string()),
            active_goal_run_title: Some("Close release gap".to_string()),
            provider_health_json: r#"{"openai":{"can_execute":true,"trip_count":0}}"#.to_string(),
            gateway_statuses_json: r#"{"slack":{"status":"connected"}}"#.to_string(),
            recent_actions_json:
                r#"[{"action_type":"tool_call","summary":"Ran status","timestamp":1712345678}]"#
                    .to_string(),
        },
    ));
    model.handle_client_event(ClientEvent::StatusDiagnostics {
        operator_profile_sync_state: "clean".to_string(),
        operator_profile_sync_dirty: false,
        operator_profile_scheduler_fallback: false,
        diagnostics_json: r#"{"aline":{"available":true,"watcher_state":"running"}}"#.to_string(),
    });

    let snapshot = model
        .status_modal_snapshot
        .as_ref()
        .expect("status snapshot should be cached");
    assert_eq!(snapshot.tier, "mission_control");
    assert_eq!(snapshot.activity, "waiting_for_operator");
    assert_eq!(snapshot.active_thread_id.as_deref(), Some("thread-1"));
    assert_eq!(model.recent_actions.len(), 1);
    assert_eq!(model.recent_actions[0].summary, "Ran status");
    assert!(model.status_modal_body().contains("Watcher:"));
}

#[test]
fn status_query_failure_resolves_loading_modal() {
    let mut model = make_model();
    model.status_modal_loading = true;

    model.handle_client_event(ClientEvent::Error("boom".to_string()));

    assert!(!model.status_modal_loading);
}

#[test]
fn status_modal_failure_replaces_loading_body_with_error_text() {
    let mut model = make_model();
    model.open_status_modal_loading();

    model.handle_client_event(ClientEvent::Error("daemon unavailable".to_string()));

    assert!(model.status_modal_body().contains("daemon unavailable"));
}

#[test]
fn pin_budget_exceeded_event_opens_app_flow() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadMessagePinResult(
        crate::client::ThreadMessagePinResultVm {
            ok: false,
            thread_id: "thread-1".to_string(),
            message_id: "message-1".to_string(),
            error: Some("pinned_budget_exceeded".to_string()),
            current_pinned_chars: 100,
            pinned_budget_chars: 120,
            candidate_pinned_chars: Some(160),
        },
    ));

    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::PinnedBudgetExceeded)
    );
    let payload = model
        .pending_pinned_budget_exceeded
        .as_ref()
        .expect("budget payload should be stored");
    assert_eq!(payload.current_pinned_chars, 100);
    assert_eq!(payload.pinned_budget_chars, 120);
    assert_eq!(payload.candidate_pinned_chars, 160);
}

#[test]
fn generic_pin_error_stays_status_only() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ThreadMessagePinResult(
        crate::client::ThreadMessagePinResultVm {
            ok: false,
            thread_id: "thread-1".to_string(),
            message_id: "message-1".to_string(),
            error: Some("message_not_found".to_string()),
            current_pinned_chars: 0,
            pinned_budget_chars: 120,
            candidate_pinned_chars: None,
        },
    ));

    assert!(model.pending_pinned_budget_exceeded.is_none());
    assert_ne!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::PinnedBudgetExceeded)
    );
    assert_eq!(model.status_line, "Pin failed: message_not_found");
}

#[test]
fn sidebar_falls_back_to_todo_after_last_pin_removed() {
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
    model.sidebar.reduce(sidebar::SidebarAction::SwitchTab(
        sidebar::SidebarTab::Pinned,
    ));

    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Pinned".to_string(),
        messages: vec![crate::wire::AgentMessage {
            id: Some("message-1".to_string()),
            role: crate::wire::MessageRole::Assistant,
            content: "Pinned content".to_string(),
            pinned_for_compaction: false,
            ..Default::default()
        }],
        loaded_message_end: 1,
        total_message_count: 1,
        ..Default::default()
    });

    assert_eq!(model.sidebar.active_tab(), sidebar::SidebarTab::Todos);
}

#[test]
fn status_modal_latest_response_replaces_stale_content() {
    let mut model = make_model();
    model.open_status_modal_loading();
    model.handle_client_event(ClientEvent::StatusDiagnostics {
        operator_profile_sync_state: "older".to_string(),
        operator_profile_sync_dirty: true,
        operator_profile_scheduler_fallback: false,
        diagnostics_json: r#"{"aline":{"available":false,"watcher_state":"unknown"}}"#.to_string(),
    });
    model.handle_client_event(ClientEvent::StatusSnapshot(
        crate::client::AgentStatusSnapshotVm {
            tier: "mission_control".to_string(),
            activity: "older".to_string(),
            active_thread_id: None,
            active_goal_run_id: None,
            active_goal_run_title: None,
            provider_health_json: "{}".to_string(),
            gateway_statuses_json: "{}".to_string(),
            recent_actions_json: "[]".to_string(),
        },
    ));
    model.handle_client_event(ClientEvent::StatusDiagnostics {
        operator_profile_sync_state: "clean".to_string(),
        operator_profile_sync_dirty: false,
        operator_profile_scheduler_fallback: false,
        diagnostics_json: r#"{"aline":{"available":true,"watcher_state":"running"}}"#.to_string(),
    });
    model.handle_client_event(ClientEvent::StatusSnapshot(
        crate::client::AgentStatusSnapshotVm {
            tier: "mission_control".to_string(),
            activity: "newer".to_string(),
            active_thread_id: None,
            active_goal_run_id: None,
            active_goal_run_title: None,
            provider_health_json: "{}".to_string(),
            gateway_statuses_json: "{}".to_string(),
            recent_actions_json: "[]".to_string(),
        },
    ));

    assert!(model.status_modal_body().contains("newer"));
    assert!(model.status_modal_body().contains("running"));
}

#[test]
fn repeated_gateway_status_does_not_keep_overwriting_status_line() {
    let mut model = make_model();
    model.status_line = "Prompt sent".to_string();

    model.handle_client_event(ClientEvent::GatewayStatus {
        platform: "discord".to_string(),
        status: "disconnected".to_string(),
        last_error: Some("socket closed".to_string()),
        consecutive_failures: 1,
    });
    assert_eq!(model.status_line, "🌐 Gateway discord: disconnected");

    model.status_line = "Prompt sent".to_string();
    model.handle_client_event(ClientEvent::GatewayStatus {
        platform: "discord".to_string(),
        status: "disconnected".to_string(),
        last_error: Some("socket closed".to_string()),
        consecutive_failures: 2,
    });

    assert_eq!(
        model.status_line, "Prompt sent",
        "repeated gateway status should not keep stealing the footer"
    );
}

#[test]
fn upgrade_notification_updates_status_line_and_inbox() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::NotificationUpsert(
        zorai_protocol::ZoraiUpdateStatus::from_versions("0.2.3", "0.2.4")
            .expect("status should parse valid versions")
            .into_notification(100),
    ));

    assert_eq!(model.notifications.unread_count(), 1);
    assert_eq!(model.status_line, "🔔 zorai 0.2.4 is available");
}

#[test]
fn operator_profile_question_event_shows_onboarding_notice() {
    let mut model = make_model();
    model.handle_client_event(ClientEvent::OperatorProfileSessionStarted {
        session_id: "sess-1".to_string(),
        kind: "first_run_onboarding".to_string(),
    });
    model.handle_client_event(ClientEvent::OperatorProfileQuestion {
        session_id: "sess-1".to_string(),
        question_id: "name".to_string(),
        field_key: "name".to_string(),
        prompt: "What should I call you?".to_string(),
        input_kind: "text".to_string(),
        optional: false,
    });

    assert!(model.should_show_operator_profile_onboarding());
    assert_eq!(
        model
            .operator_profile
            .question
            .as_ref()
            .map(|q| q.field_key.as_str()),
        Some("name")
    );
}

#[test]
fn operator_profile_boolean_question_defaults_to_yes_without_filling_input() {
    let mut model = make_model();
    model.handle_client_event(ClientEvent::OperatorProfileSessionStarted {
        session_id: "sess-1".to_string(),
        kind: "first_run_onboarding".to_string(),
    });
    model.handle_client_event(ClientEvent::OperatorProfileQuestion {
        session_id: "sess-1".to_string(),
        question_id: "enabled".to_string(),
        field_key: "enabled".to_string(),
        prompt: "Enable operator modeling overall?".to_string(),
        input_kind: "boolean".to_string(),
        optional: false,
    });

    assert_eq!(model.input.buffer(), "");
    assert_eq!(model.operator_profile.bool_answer, Some(true));
    assert_eq!(
        model.current_operator_profile_select_options(),
        Some(&["yes", "no"][..])
    );
}

#[test]
fn operator_profile_progress_requests_next_question() {
    let (_event_tx, event_rx) = std::sync::mpsc::channel();
    let (daemon_tx, mut daemon_rx) = unbounded_channel();
    let mut model = TuiModel::new(event_rx, daemon_tx);
    model.handle_client_event(ClientEvent::OperatorProfileSessionStarted {
        session_id: "sess-1".to_string(),
        kind: "first_run_onboarding".to_string(),
    });

    model.handle_client_event(ClientEvent::OperatorProfileProgress {
        session_id: "sess-1".to_string(),
        answered: 1,
        remaining: 2,
        completion_ratio: 0.33,
    });

    let mut found_next = false;
    while let Ok(command) = daemon_rx.try_recv() {
        if matches!(
            command,
            crate::state::DaemonCommand::NextOperatorProfileQuestion { .. }
        ) {
            found_next = true;
            break;
        }
    }
    assert!(found_next, "progress should trigger next-question command");
}

#[test]
fn weles_health_update_surfaces_degraded_status() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::WelesHealthUpdate {
        state: "degraded".to_string(),
        reason: Some("WELES review unavailable for guarded actions".to_string()),
        checked_at: 77,
    });

    assert_eq!(
        model
            .weles_health
            .as_ref()
            .map(|health| health.state.as_str()),
        Some("degraded")
    );
    assert!(
        model.status_line.contains("WELES degraded"),
        "status line should mention degraded WELES health"
    );
}

#[test]
fn models_fetched_updates_picker_count_for_open_model_picker() {
    let mut model = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
    model.modal.set_picker_item_count(1);

    model.handle_client_event(ClientEvent::ModelsFetched(vec![
        crate::wire::FetchedModel {
            id: "m1".to_string(),
            name: Some("Model One".to_string()),
            context_window: Some(128_000),
            pricing: None,
            metadata: None,
        },
        crate::wire::FetchedModel {
            id: "m2".to_string(),
            name: Some("Model Two".to_string()),
            context_window: Some(128_000),
            pricing: None,
            metadata: None,
        },
        crate::wire::FetchedModel {
            id: "m3".to_string(),
            name: Some("Model Three".to_string()),
            context_window: Some(128_000),
            pricing: None,
            metadata: None,
        },
    ]));

    model.modal.reduce(modal::ModalAction::Navigate(1));
    model.modal.reduce(modal::ModalAction::Navigate(1));

    assert_eq!(model.modal.picker_cursor(), 2);
}

#[test]
fn approval_required_in_current_thread_opens_blocking_modal() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Active Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "WELES review".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::TaskStatus::AwaitingApproval),
        awaiting_approval_id: Some("approval-1".to_string()),
        ..Default::default()
    }]);

    model.handle_client_event(ClientEvent::ApprovalRequired {
        approval_id: "approval-1".to_string(),
        command: "git push".to_string(),
        rationale: Some("Push release branch to origin".to_string()),
        reasons: vec!["network access requested".to_string()],
        risk_level: "high".to_string(),
        blast_radius: "repo".to_string(),
    });

    assert_eq!(model.modal.top(), Some(modal::ModalKind::ApprovalOverlay));
    assert_eq!(model.approval.selected_approval_id(), Some("approval-1"));
}

#[test]
fn approval_required_in_background_thread_shows_notice_without_modal() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Active Thread".to_string(),
    });
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-2".to_string(),
        title: "Background Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-2".to_string(),
        title: "WELES review".to_string(),
        thread_id: Some("thread-2".to_string()),
        status: Some(crate::wire::TaskStatus::AwaitingApproval),
        awaiting_approval_id: Some("approval-2".to_string()),
        ..Default::default()
    }]);

    model.handle_client_event(ClientEvent::ApprovalRequired {
        approval_id: "approval-2".to_string(),
        command: "git clone".to_string(),
        rationale: Some("Clone support repository into workspace".to_string()),
        reasons: vec!["network access requested".to_string()],
        risk_level: "medium".to_string(),
        blast_radius: "workspace".to_string(),
    });

    assert_eq!(model.modal.top(), None);
    assert_eq!(model.approval.pending_approvals().len(), 1);
    assert!(model
        .input_notice_style()
        .expect("approval banner should be visible")
        .0
        .contains("Ctrl+A"));
}
