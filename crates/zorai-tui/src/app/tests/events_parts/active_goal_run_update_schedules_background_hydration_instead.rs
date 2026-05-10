use super::idle_tick_does_not_request_redraw_to_first_raw_config_load_triggers::*;
use crate::app::*;
#[test]
fn active_goal_run_update_schedules_background_hydration_instead_of_immediate_refresh() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    });

    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(
        next_goal_run_detail_request(&mut daemon_rx).is_none(),
        "active-goal updates should no longer emit immediate detail refreshes"
    );
    assert!(
        next_goal_run_checkpoints_request(&mut daemon_rx).is_none(),
        "active-goal updates should no longer emit immediate checkpoint refreshes"
    );
}

#[test]
fn goal_run_controlled_ack_requests_authoritative_goal_refresh() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.handle_client_event(ClientEvent::GoalRunControlled {
        goal_run_id: "goal-1".to_string(),
        ok: true,
    });

    assert_eq!(
        next_goal_run_detail_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_run_checkpoints_request(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(model.status_line, "Goal run updated");
}

#[test]
fn failed_goal_run_control_ack_does_not_refresh_goal() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.handle_client_event(ClientEvent::GoalRunControlled {
        goal_run_id: "goal-1".to_string(),
        ok: false,
    });

    assert!(next_goal_run_detail_request(&mut daemon_rx).is_none());
    assert!(next_goal_run_checkpoints_request(&mut daemon_rx).is_none());
    assert_eq!(model.status_line, "Goal run update failed");
}

#[test]
fn repeated_active_goal_updates_only_schedule_background_hydration_once_per_burst() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    });
    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    });

    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(
        next_goal_hydration_schedule(&mut daemon_rx).is_none(),
        "duplicate active-goal updates should coalesce into one pending hydration request"
    );
}

#[test]
fn active_goal_hydration_reschedules_after_authoritative_detail_arrives() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    });
    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );

    model.handle_goal_run_detail_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        ..Default::default()
    });
    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    });

    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1"),
        "authoritative detail should clear the pending marker so later updates can reschedule"
    );
}

#[test]
fn goal_detail_placeholder_clears_pending_hydration_for_the_requested_goal() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    });
    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(
        model.pending_goal_hydration_refreshes.contains("goal-1"),
        "update should leave the goal pending until an authoritative response arrives"
    );

    model.schedule_goal_hydration_refresh("goal-2".to_string());
    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-2")
    );
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-2".to_string(),
        step_id: None,
    });

    model.handle_client_event(ClientEvent::GoalRunDetail(Some(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        ..Default::default()
    })));

    assert!(
        !model.pending_goal_hydration_refreshes.contains("goal-1"),
        "placeholder detail should clear the stale pending hydration marker for the requested goal"
    );
    assert!(
        model.pending_goal_hydration_refreshes.contains("goal-2"),
        "changing panes before the empty response arrives should not clear the wrong goal"
    );
}

#[test]
fn goal_run_list_refresh_reconciles_pending_hydration_against_present_goals() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();

    model.schedule_goal_hydration_refresh("goal-1".to_string());
    model.schedule_goal_hydration_refresh("goal-2".to_string());
    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-2")
    );
    assert_eq!(model.pending_goal_hydration_refreshes.len(), 2);

    model.handle_goal_run_list_event(vec![crate::wire::GoalRun {
        id: "goal-2".to_string(),
        title: "Goal Two".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    }]);

    assert!(
        !model.pending_goal_hydration_refreshes.contains("goal-1"),
        "goal list refresh should drop pending hydration for goals no longer present"
    );
    assert!(
        model.pending_goal_hydration_refreshes.contains("goal-2"),
        "goal list refresh should preserve pending hydration for goals still present"
    );
}

#[test]
fn empty_goal_checkpoint_refresh_clears_pending_hydration_for_requested_goal() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();

    model.schedule_goal_hydration_refresh("goal-1".to_string());
    model.handle_client_event(ClientEvent::GoalRunCheckpoints {
        goal_run_id: "goal-1".to_string(),
        checkpoints: vec![],
    });

    assert!(
        !model.pending_goal_hydration_refreshes.contains("goal-1"),
        "empty checkpoint lists should still clear the exact pending goal hydration marker"
    );
}

#[test]
fn goal_hydration_schedule_failed_clears_pending_marker_via_client_event() {
    let (mut model, _daemon_rx) = make_model_with_daemon_rx();

    model.schedule_goal_hydration_refresh("goal-1".to_string());
    model.handle_client_event(ClientEvent::GoalHydrationScheduleFailed {
        goal_run_id: "goal-1".to_string(),
    });

    assert!(
        !model.pending_goal_hydration_refreshes.contains("goal-1"),
        "background hydration failure should clear the exact pending marker"
    );
}

#[test]
fn goal_scoped_thread_todos_for_active_goal_schedule_background_hydration() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.handle_goal_run_detail_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        steps: vec![crate::wire::GoalRunStep {
            id: "step-1".to_string(),
            position: 0,
            title: "Plan".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Child task".to_string(),
        thread_id: Some("thread-task".to_string()),
        goal_run_id: Some("goal-1".to_string()),
        ..Default::default()
    }]);
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.handle_client_event(ClientEvent::ThreadTodos {
        thread_id: "thread-task".to_string(),
        goal_run_id: Some("goal-1".to_string()),
        step_index: Some(0),
        items: vec![crate::wire::TodoItem {
            id: "todo-1".to_string(),
            content: "child thread todo".to_string(),
            status: Some(crate::wire::TodoStatus::InProgress),
            position: 0,
            step_index: Some(0),
            ..Default::default()
        }],
    });

    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(
        model.pending_goal_hydration_refreshes.contains("goal-1"),
        "goal-scoped todo updates should arm background hydration for the visible goal"
    );
}

#[test]
fn work_context_for_active_goal_linked_thread_schedules_background_hydration() {
    let (mut model, mut daemon_rx) = make_model_with_daemon_rx();
    model.handle_goal_run_detail_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal One".to_string(),
        status: Some(crate::wire::GoalRunStatus::Running),
        steps: vec![crate::wire::GoalRunStep {
            id: "step-1".to_string(),
            position: 0,
            title: "Plan".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Child task".to_string(),
        thread_id: Some("thread-task".to_string()),
        goal_run_id: Some("goal-1".to_string()),
        ..Default::default()
    }]);
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    model.handle_work_context_event(crate::wire::ThreadWorkContext {
        thread_id: "thread-task".to_string(),
        entries: vec![crate::wire::WorkContextEntry {
            path: "/tmp/skill_injection_gap.md".to_string(),
            goal_run_id: Some("goal-1".to_string()),
            step_index: Some(0),
            is_text: true,
            ..Default::default()
        }],
    });

    assert_eq!(
        next_goal_hydration_schedule(&mut daemon_rx).as_deref(),
        Some("goal-1")
    );
    assert!(
        model.pending_goal_hydration_refreshes.contains("goal-1"),
        "work-context updates should arm background hydration for the visible goal"
    );
}

#[test]
fn active_goal_run_goal_refresh_preserves_or_clamps_goal_sidebar_selection() {
    let mut model = active_goal_run_sidebar_model();
    model.activate_goal_sidebar_tab(GoalSidebarTab::Tasks);
    model.select_goal_sidebar_row(1);
    assert_eq!(model.goal_sidebar.selected_row(), 1);

    model.handle_goal_run_update_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal Title".to_string(),
        thread_id: Some("thread-1".to_string()),
        child_task_ids: vec!["task-1".to_string(), "task-2".to_string()],
        status: Some(crate::wire::GoalRunStatus::Running),
        ..Default::default()
    });

    assert_eq!(
        model.goal_sidebar.selected_row(),
        1,
        "goal updates should preserve the selected goal-sidebar row when it stays valid"
    );

    model.handle_goal_run_detail_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal Title".to_string(),
        thread_id: Some("thread-1".to_string()),
        child_task_ids: vec!["task-1".to_string()],
        steps: vec![
            crate::wire::GoalRunStep {
                id: "step-1".to_string(),
                position: 0,
                title: "Plan".to_string(),
                ..Default::default()
            },
            crate::wire::GoalRunStep {
                id: "step-2".to_string(),
                position: 1,
                title: "Implement".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    assert_eq!(
        model.goal_sidebar.selected_row(),
        0,
        "goal detail refresh should clamp the goal-sidebar row when the previous selection no longer exists"
    );
}

#[test]
fn active_goal_run_checkpoint_task_file_tabs_clamp_after_refreshes() {
    let mut model = active_goal_run_sidebar_model();

    model.activate_goal_sidebar_tab(GoalSidebarTab::Checkpoints);
    model.select_goal_sidebar_row(1);
    model.handle_goal_run_checkpoints_event(
        "goal-1".to_string(),
        vec![crate::wire::CheckpointSummary {
            id: "checkpoint-1".to_string(),
            checkpoint_type: "plan".to_string(),
            step_index: Some(0),
            context_summary_preview: Some("Checkpoint for Plan".to_string()),
            ..Default::default()
        }],
    );
    assert_eq!(model.goal_sidebar.selected_row(), 0);

    model.activate_goal_sidebar_tab(GoalSidebarTab::Tasks);
    model.select_goal_sidebar_row(1);
    model.handle_task_list_event(vec![crate::wire::AgentTask {
        id: "task-1".to_string(),
        title: "Child Task One".to_string(),
        thread_id: Some("thread-1".to_string()),
        goal_run_id: Some("goal-1".to_string()),
        created_at: 10,
        ..Default::default()
    }]);
    assert_eq!(model.goal_sidebar.selected_row(), 0);

    model.activate_goal_sidebar_tab(GoalSidebarTab::Files);
    model.select_goal_sidebar_row(1);
    model.handle_work_context_event(crate::wire::ThreadWorkContext {
        thread_id: "thread-1".to_string(),
        entries: vec![crate::wire::WorkContextEntry {
            path: "/tmp/plan.md".to_string(),
            goal_run_id: Some("goal-1".to_string()),
            is_text: true,
            ..Default::default()
        }],
    });
    assert_eq!(model.goal_sidebar.selected_row(), 0);
}

#[test]
fn active_goal_run_selecting_step_in_main_pane_updates_goal_sidebar_highlight() {
    let mut model = active_goal_run_sidebar_model();
    model.activate_goal_sidebar_tab(GoalSidebarTab::Steps);
    model.select_goal_sidebar_row(0);

    assert!(model.select_goal_step_in_active_run("step-3".to_string()));

    assert_eq!(model.goal_sidebar.selected_row(), 2);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            ref goal_run_id,
            step_id: Some(ref step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-3"
    ));
}

#[test]
fn active_goal_run_task_tab_preserves_selected_task_across_reorder_refresh() {
    let mut model = active_goal_run_sidebar_model();
    model.activate_goal_sidebar_tab(GoalSidebarTab::Tasks);
    model.select_goal_sidebar_row(1);

    model.handle_goal_run_detail_event(crate::wire::GoalRun {
        id: "goal-1".to_string(),
        title: "Goal Title".to_string(),
        thread_id: Some("thread-1".to_string()),
        child_task_ids: vec!["task-2".to_string(), "task-1".to_string()],
        steps: vec![
            crate::wire::GoalRunStep {
                id: "step-1".to_string(),
                position: 0,
                title: "Plan".to_string(),
                ..Default::default()
            },
            crate::wire::GoalRunStep {
                id: "step-2".to_string(),
                position: 1,
                title: "Implement".to_string(),
                ..Default::default()
            },
            crate::wire::GoalRunStep {
                id: "step-3".to_string(),
                position: 2,
                title: "Verify".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    assert_eq!(
        model.goal_sidebar.selected_row(),
        0,
        "task-tab selection should follow the same task id when child task order changes"
    );
}
