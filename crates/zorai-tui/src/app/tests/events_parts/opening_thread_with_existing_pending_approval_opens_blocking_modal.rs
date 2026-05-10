use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn opening_thread_with_existing_pending_approval_opens_blocking_modal() {
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

    model.open_thread_conversation("thread-2".to_string());

    assert_eq!(model.modal.top(), Some(modal::ModalKind::ApprovalOverlay));
    assert_eq!(model.approval.selected_approval_id(), Some("approval-2"));
}

#[test]
fn task_list_hydrates_pending_approvals_from_awaiting_approval_tasks() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Hydrated Thread".to_string(),
    });

    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated approval".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::TaskStatus::AwaitingApproval),
        awaiting_approval_id: Some("approval-1".to_string()),
        blocked_reason: Some("waiting for operator approval".to_string()),
        ..Default::default()
    }]);

    let approval = model
        .approval
        .approval_by_id("approval-1")
        .expect("task snapshot should hydrate approval queue");
    assert_eq!(approval.task_id, "task-1");
    assert_eq!(approval.thread_title.as_deref(), Some("Hydrated Thread"));
}

#[test]
fn goal_run_update_hydrates_pending_approval_when_task_snapshot_is_missing() {
    let mut model = make_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Goal Thread".to_string(),
    });

    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal plan review".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::GoalRunStatus::AwaitingApproval),
        current_step_title: Some("review plan".to_string()),
        approval_count: 1,
        awaiting_approval_id: Some("approval-1".to_string()),
        ..Default::default()
    });

    let approval = model
        .approval
        .approval_by_id("approval-1")
        .expect("goal run awaiting approval should hydrate approval queue");
    assert_eq!(approval.thread_id.as_deref(), Some("thread-1"));
    assert_eq!(approval.thread_title.as_deref(), Some("Goal Thread"));
    assert_eq!(approval.task_title.as_deref(), Some("Goal plan review"));
}

#[test]
fn goal_run_started_requests_authoritative_refresh_and_hydrates_pending_approval() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Goal Thread".to_string(),
    });

    model.handle_goal_run_started_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal plan review".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::GoalRunStatus::AwaitingApproval),
        current_step_title: Some("review plan".to_string()),
        approval_count: 1,
        awaiting_approval_id: Some("approval-1".to_string()),
        ..Default::default()
    });

    assert!(matches!(
        &model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun { goal_run_id, step_id: None })
            if goal_run_id == "goal-1"
    ));
    assert_eq!(
        next_goal_run_detail_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_run_checkpoints_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(
        model.pending_goal_hydration_refreshes.contains("goal-1"),
        "started goal should remain pending until authoritative hydration lands"
    );

    let approval = model
        .approval
        .approval_by_id("approval-1")
        .expect("started goal awaiting approval should hydrate approval queue");
    assert_eq!(approval.thread_id.as_deref(), Some("thread-1"));
    assert_eq!(approval.thread_title.as_deref(), Some("Goal Thread"));
    assert_eq!(approval.task_title.as_deref(), Some("Goal plan review"));
    assert_eq!(model.modal.top(), Some(modal::ModalKind::ApprovalOverlay));
    assert_eq!(model.approval.selected_approval_id(), Some("approval-1"));
}

#[test]
fn approval_resolution_requests_authoritative_refresh_for_visible_goal_run() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.handle_goal_run_detail_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal plan review".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::GoalRunStatus::AwaitingApproval),
        current_step_title: Some("review plan".to_string()),
        approval_count: 1,
        awaiting_approval_id: Some("approval-1".to_string()),
        ..Default::default()
    });
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.handle_approval_resolved_event("approval-1".to_string(), "approved".to_string());

    assert_eq!(
        next_goal_run_detail_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_run_checkpoints_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
}

#[test]
fn task_list_event_preserves_spawned_tree_metadata() {
    let mut model = make_model();

    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated child".to_string(),
        thread_id: Some("thread-1".to_string()),
        parent_task_id: Some("task-root".to_string()),
        parent_thread_id: Some("thread-root".to_string()),
        created_at: 42,
        ..Default::default()
    }]);

    let task = model
        .tasks
        .task_by_id("task-1")
        .expect("task should be present after hydration");
    assert_eq!(task.parent_task_id.as_deref(), Some("task-root"));
    assert_eq!(task.parent_thread_id.as_deref(), Some("thread-root"));
    assert_eq!(task.created_at, 42);
}

#[test]
fn fallback_task_update_preserves_spawned_tree_metadata() {
    let mut model = make_model();

    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated child".to_string(),
        thread_id: Some("thread-1".to_string()),
        parent_task_id: Some("task-root".to_string()),
        parent_thread_id: Some("thread-root".to_string()),
        created_at: 42,
        ..Default::default()
    }]);

    model.handle_task_update_event(crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated child".to_string(),
        status: Some(crate::wire::TaskStatus::InProgress),
        progress: 75,
        ..Default::default()
    });

    let task = model
        .tasks
        .task_by_id("task-1")
        .expect("task should still be present after update");
    assert_eq!(task.parent_task_id.as_deref(), Some("task-root"));
    assert_eq!(task.parent_thread_id.as_deref(), Some("thread-root"));
    assert_eq!(task.created_at, 42);
}

#[test]
fn budget_exceeded_active_thread_surfaces_persistent_footer_notice() {
    let mut model = make_model();
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-child".to_string(),
        title: "Child thread".to_string(),
        ..Default::default()
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-child".to_string()));

    model.handle_task_update_event(crate::wire::AgentTask {
        id: "task-child".to_string(),
        title: "Child task".to_string(),
        thread_id: Some("thread-child".to_string()),
        status: Some(crate::wire::TaskStatus::BudgetExceeded),
        blocked_reason: Some("execution budget exceeded for this thread".to_string()),
        created_at: 42,
        ..Default::default()
    });

    let notice = model
        .active_thread_budget_exceeded_notice()
        .expect("budget exceeded thread should surface footer notice");
    assert!(
        notice.contains("Thread budget exceeded"),
        "expected budget notice, got: {notice}"
    );
    assert!(
        notice.contains("thread-child"),
        "expected thread id in notice, got: {notice}"
    );
}

#[test]
fn submit_prompt_blocks_budget_exceeded_active_thread() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.connected = true;
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-child".to_string(),
        title: "Child thread".to_string(),
        ..Default::default()
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-child".to_string()));
    model.handle_task_update_event(crate::wire::AgentTask {
        id: "task-child".to_string(),
        title: "Child task".to_string(),
        thread_id: Some("thread-child".to_string()),
        status: Some(crate::wire::TaskStatus::BudgetExceeded),
        blocked_reason: Some("execution budget exceeded for this thread".to_string()),
        created_at: 42,
        ..Default::default()
    });
    while daemon_rx.try_recv().is_ok() {}

    model.submit_prompt("continue from here".to_string());

    assert_eq!(
        model.input.buffer(),
        "continue from here",
        "blocked submit should preserve operator text in the input"
    );
    let (notice, _) = model
        .input_notice_style()
        .expect("blocked submit should surface an input notice");
    assert!(
        notice.contains("Thread budget exceeded"),
        "expected budget exceeded notice, got: {notice}"
    );
    while let Ok(command) = daemon_rx.try_recv() {
        assert!(
            !matches!(command, DaemonCommand::SendMessage { .. }),
            "budget-exceeded thread should not emit a send command: {command:?}"
        );
    }
}

#[test]
fn unrelated_sync_does_not_clear_event_backed_pending_approval() {
    let mut model = make_model();

    model.handle_client_event(ClientEvent::ApprovalRequired {
        approval_id: "approval-1".to_string(),
        command: "git push".to_string(),
        rationale: Some("Push branch".to_string()),
        reasons: vec!["network access requested".to_string()],
        risk_level: "high".to_string(),
        blast_radius: "repo".to_string(),
    });

    model.handle_thread_list_event(vec![crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        ..Default::default()
    }]);

    assert!(
        model.approval.approval_by_id("approval-1").is_some(),
        "thread sync should not discard live approvals without an explicit resolution"
    );
}

#[test]
fn task_list_clears_approval_when_same_task_no_longer_waits_for_it() {
    let mut model = make_model();

    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated approval".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::TaskStatus::AwaitingApproval),
        awaiting_approval_id: Some("approval-1".to_string()),
        blocked_reason: Some("waiting for operator approval".to_string()),
        ..Default::default()
    }]);
    assert!(model.approval.approval_by_id("approval-1").is_some());

    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated approval".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::TaskStatus::Queued),
        awaiting_approval_id: None,
        ..Default::default()
    }]);

    assert!(
        model.approval.approval_by_id("approval-1").is_none(),
        "task snapshot should clear approvals only when the same task explicitly drops them"
    );
}

#[test]
fn task_update_clearing_approval_does_not_rehydrate_on_later_sync() {
    let mut model = make_model();

    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated approval".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::TaskStatus::AwaitingApproval),
        awaiting_approval_id: Some("approval-1".to_string()),
        blocked_reason: Some("waiting for operator approval".to_string()),
        ..Default::default()
    }]);
    assert!(model.approval.approval_by_id("approval-1").is_some());

    model.handle_task_update_event(crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Hydrated approval".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::TaskStatus::Queued),
        awaiting_approval_id: None,
        ..Default::default()
    });

    assert!(
        model.approval.approval_by_id("approval-1").is_none(),
        "task update should clear the pending approval immediately"
    );

    model.handle_thread_list_event(vec![crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Thread".to_string(),
        ..Default::default()
    }]);

    assert!(
        model.approval.approval_by_id("approval-1").is_none(),
        "later thread sync should not resurrect approvals that the task already cleared"
    );
}

#[test]
fn task_list_hydrates_policy_escalation_rationale_from_thread_messages() {
    let mut model = make_model();
    model.handle_thread_detail_event(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Hydrated Thread".to_string(),
        messages: vec![crate::wire::AgentMessage {
            role: crate::wire::MessageRole::System,
            content: "Policy escalation requested operator guidance: Cloning scientific skills repository from GitHub as part of WELES governance review task".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });

    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "WELES".to_string(),
        thread_id: Some("thread-1".to_string()),
        status: Some(crate::wire::TaskStatus::AwaitingApproval),
        awaiting_approval_id: Some("approval-1".to_string()),
        blocked_reason: Some(
            "waiting for operator approval: orchestrator_policy_escalation".to_string(),
        ),
        ..Default::default()
    }]);

    let approval = model
        .approval
        .approval_by_id("approval-1")
        .expect("task snapshot should hydrate approval queue");
    assert_eq!(
        approval.rationale.as_deref(),
        Some(
            "Cloning scientific skills repository from GitHub as part of WELES governance review task"
        )
    );
}
