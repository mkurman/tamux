use super::task_list_received_replaces_tasks_to_goal_step_todos_use_latest_event::*;
use crate::state::spawned_tree::derive_spawned_agent_tree;
use crate::state::task::*;
#[test]
fn goal_run_detail_received_stores_owner_profiles() {
    let mut state = TaskState::new();
    let converted = crate::app::conversion::convert_goal_run(crate::wire::GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        planner_owner_profile: Some(make_wire_owner_profile(
            "Planner",
            "openai",
            "gpt-4.1",
            Some("high"),
        )),
        current_step_owner_profile: Some(make_wire_owner_profile(
            "Verifier",
            "anthropic",
            "claude-sonnet-4",
            None,
        )),
        ..Default::default()
    });
    state.reduce(TaskAction::GoalRunDetailReceived(converted));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(
        goal.planner_owner_profile,
        Some(make_owner_profile(
            "Planner",
            "openai",
            "gpt-4.1",
            Some("high")
        ))
    );
    assert_eq!(
        goal.current_step_owner_profile,
        Some(make_owner_profile(
            "Verifier",
            "anthropic",
            "claude-sonnet-4",
            None
        ))
    );
}

#[test]
fn goal_run_detail_received_clears_owner_profiles_when_authoritative_payload_omits_them() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        planner_owner_profile: Some(make_owner_profile(
            "Planner",
            "openai",
            "gpt-4.1",
            Some("high"),
        )),
        current_step_owner_profile: Some(make_owner_profile(
            "Verifier",
            "anthropic",
            "claude-sonnet-4",
            None,
        )),
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Detailed".into(),
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Detailed");
    assert!(goal.planner_owner_profile.is_none());
    assert!(goal.current_step_owner_profile.is_none());
}

#[test]
fn goal_run_update_preserves_owner_profiles_when_incremental_payload_omits_them() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        planner_owner_profile: Some(make_owner_profile(
            "Planner",
            "openai",
            "gpt-4.1",
            Some("high"),
        )),
        current_step_owner_profile: Some(make_owner_profile(
            "Verifier",
            "anthropic",
            "claude-sonnet-4",
            None,
        )),
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunUpdate(GoalRun {
        id: "g1".into(),
        title: "Updated".into(),
        sparse_update: true,
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Original");
    assert_eq!(
        goal.planner_owner_profile,
        Some(make_owner_profile(
            "Planner",
            "openai",
            "gpt-4.1",
            Some("high")
        ))
    );
    assert_eq!(
        goal.current_step_owner_profile,
        Some(make_owner_profile(
            "Verifier",
            "anthropic",
            "claude-sonnet-4",
            None
        ))
    );
}

#[test]
fn task_update_drops_stale_awaiting_approval_id_when_status_advances() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::TaskListReceived(vec![AgentTask {
        id: "task-1".into(),
        title: "Hydrated approval".into(),
        thread_id: Some("thread-1".into()),
        status: Some(TaskStatus::AwaitingApproval),
        awaiting_approval_id: Some("approval-1".into()),
        ..Default::default()
    }]));

    state.reduce(TaskAction::TaskUpdate(AgentTask {
        id: "task-1".into(),
        title: "Hydrated approval".into(),
        thread_id: Some("thread-1".into()),
        status: Some(TaskStatus::Queued),
        awaiting_approval_id: None,
        ..Default::default()
    }));

    let task = state.task_by_id("task-1").expect("task should exist");
    assert_eq!(task.status, Some(TaskStatus::Queued));
    assert!(
        task.awaiting_approval_id.is_none(),
        "non-awaiting task updates should clear stale approval IDs instead of preserving them"
    );
}

#[test]
fn goal_run_update_preserves_planner_fallback_when_current_step_owner_is_absent() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        planner_owner_profile: Some(make_owner_profile("Planner", "openai", "gpt-4.1", None)),
        current_step_owner_profile: None,
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunUpdate(GoalRun {
        id: "g1".into(),
        title: "Updated".into(),
        sparse_update: true,
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Original");
    assert_eq!(
        goal.planner_owner_profile,
        Some(make_owner_profile("Planner", "openai", "gpt-4.1", None))
    );
    assert!(goal.current_step_owner_profile.is_none());
}

#[test]
fn goal_run_update_preserves_existing_dossier_when_incremental_update_omits_it() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        dossier: Some(GoalRunDossier {
            projection_state: "in_progress".into(),
            summary: Some("planner seeded units".into()),
            units: vec![GoalDeliveryUnitRecord {
                id: "unit-1".into(),
                title: "Ship android logging".into(),
                status: "queued".into(),
                execution_binding: "builtin:android".into(),
                verification_binding: "subagent:validator".into(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunUpdate(GoalRun {
        id: "g1".into(),
        title: "Updated".into(),
        sparse_update: true,
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Original");
    let dossier = goal.dossier.as_ref().expect("dossier should be preserved");
    assert_eq!(dossier.projection_state, "in_progress");
    assert_eq!(dossier.summary.as_deref(), Some("planner seeded units"));
    assert_eq!(dossier.units.len(), 1);
    assert_eq!(dossier.units[0].execution_binding, "builtin:android");
}

#[test]
fn goal_run_detail_received_clears_dossier_when_authoritative_payload_omits_it() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        dossier: Some(GoalRunDossier {
            projection_state: "in_progress".into(),
            summary: Some("planner seeded units".into()),
            ..Default::default()
        }),
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Detailed".into(),
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Detailed");
    assert!(goal.dossier.is_none());
}

#[test]
fn goal_run_update_from_partial_wire_payload_preserves_existing_scalar_fields() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        goal: "Ship mission control".into(),
        current_step_index: 3,
        child_task_count: 5,
        approval_count: 2,
        created_at: 10,
        updated_at: 20,
        dossier: Some(GoalRunDossier {
            projection_state: "running".into(),
            ..Default::default()
        }),
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunUpdate(GoalRun {
        id: "g1".into(),
        title: "Updated".into(),
        sparse_update: true,
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Original");
    assert_eq!(goal.goal, "Ship mission control");
    assert_eq!(goal.current_step_index, 3);
    assert_eq!(goal.child_task_count, 5);
    assert_eq!(goal.approval_count, 2);
    assert_eq!(goal.created_at, 10);
    assert_eq!(goal.updated_at, 20);
    assert_eq!(
        goal.dossier
            .as_ref()
            .expect("incremental update should preserve existing dossier")
            .projection_state,
        "running"
    );
}

#[test]
fn goal_run_update_full_snapshot_can_clear_scalar_fields() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        goal: "Ship mission control".into(),
        child_task_count: 5,
        approval_count: 2,
        current_step_index: 3,
        created_at: 11,
        updated_at: 22,
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunUpdate(GoalRun {
        id: "g1".into(),
        title: "Updated".into(),
        goal: String::new(),
        child_task_count: 0,
        approval_count: 0,
        current_step_index: 0,
        created_at: 30,
        updated_at: 40,
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Updated");
    assert!(goal.goal.is_empty());
    assert_eq!(goal.child_task_count, 0);
    assert_eq!(goal.approval_count, 0);
    assert_eq!(goal.current_step_index, 0);
    assert_eq!(goal.created_at, 30);
    assert_eq!(goal.updated_at, 40);
}

#[test]
fn goal_run_checkpoints_received_replaces_goal_run_checkpoints() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunCheckpointsReceived {
        goal_run_id: "g1".into(),
        checkpoints: vec![GoalRunCheckpointSummary {
            id: "cp_1".into(),
            checkpoint_type: "manual".into(),
            task_count: 2,
            ..Default::default()
        }],
    });
    assert_eq!(state.checkpoints_for_goal_run("g1").len(), 1);
    assert_eq!(state.checkpoints_for_goal_run("g1")[0].id, "cp_1");

    state.reduce(TaskAction::GoalRunCheckpointsReceived {
        goal_run_id: "g1".into(),
        checkpoints: vec![],
    });
    assert!(state.checkpoints_for_goal_run("g1").is_empty());
}

#[test]
fn heartbeat_items_received_replaces() {
    let mut state = TaskState::new();
    let items = vec![
        HeartbeatItem {
            id: "h1".into(),
            label: "Service A".into(),
            ..Default::default()
        },
        HeartbeatItem {
            id: "h2".into(),
            label: "Service B".into(),
            ..Default::default()
        },
    ];
    state.reduce(TaskAction::HeartbeatItemsReceived(items));
    assert_eq!(state.heartbeat_items().len(), 2);

    state.reduce(TaskAction::HeartbeatItemsReceived(vec![]));
    assert_eq!(state.heartbeat_items().len(), 0);
}

#[test]
fn task_by_id_returns_correct_task() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::TaskListReceived(vec![
        make_task("t1", "Alpha", 1, None, None, None, None),
        make_task("t2", "Beta", 2, None, None, None, None),
    ]));
    assert_eq!(
        state.task_by_id("t2").map(|t| t.title.as_str()),
        Some("Beta")
    );
    assert!(state.task_by_id("unknown").is_none());
}

#[test]
fn task_update_preserves_hydrated_spawned_tree_metadata() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::TaskListReceived(vec![make_task(
        "task-1",
        "Original",
        11,
        Some("thread-1"),
        Some("parent-task"),
        Some("parent-thread"),
        Some(TaskStatus::InProgress),
    )]));

    state.reduce(TaskAction::TaskUpdate(AgentTask {
        id: "task-1".into(),
        title: "Updated".into(),
        status: Some(TaskStatus::Completed),
        ..Default::default()
    }));

    let task = state.task_by_id("task-1").expect("task");
    assert_eq!(task.title, "Updated");
    assert_eq!(task.status, Some(TaskStatus::Completed));
    assert_eq!(task.created_at, 11);
    assert_eq!(task.thread_id.as_deref(), Some("thread-1"));
    assert_eq!(task.parent_task_id.as_deref(), Some("parent-task"));
    assert_eq!(task.parent_thread_id.as_deref(), Some("parent-thread"));
}

#[test]
fn spawned_tree_nests_descendants_by_parent_task_id() {
    let tasks = vec![
        make_task(
            "root-task",
            "Root",
            10,
            Some("thread-root"),
            None,
            None,
            Some(TaskStatus::InProgress),
        ),
        make_task(
            "child-task",
            "Child",
            20,
            Some("thread-child"),
            Some("root-task"),
            Some("thread-root"),
            Some(TaskStatus::InProgress),
        ),
        make_task(
            "grandchild-task",
            "Grandchild",
            30,
            Some("thread-grandchild"),
            Some("child-task"),
            Some("thread-child"),
            Some(TaskStatus::Completed),
        ),
    ];

    let tree = derive_spawned_agent_tree(&tasks, Some("thread-root")).expect("tree");
    assert_eq!(tree.active_thread_id, "thread-root");
    assert_eq!(
        tree.anchor.as_ref().map(|node| node.item.id.as_str()),
        Some("root-task")
    );
    assert!(tree.roots.is_empty());
    assert_eq!(
        tree.anchor
            .as_ref()
            .and_then(|node| node.children.first())
            .map(|node| node.item.id.as_str()),
        Some("child-task")
    );
    assert_eq!(
        tree.anchor
            .as_ref()
            .and_then(|node| node.children.first())
            .and_then(|node| node.children.first())
            .map(|node| node.item.id.as_str()),
        Some("grandchild-task")
    );
    assert!(tree
        .anchor
        .as_ref()
        .and_then(|node| node.children.first())
        .is_some_and(|node| node.openable));
}
