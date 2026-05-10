use super::task_list_received_replaces_tasks_to_goal_step_todos_use_latest_event::*;
use crate::state::task::*;
#[test]
fn goal_step_todos_use_goal_scoped_live_todos_when_event_snapshot_is_missing() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        thread_id: Some("thread-1".into()),
        steps: vec![GoalRunStep {
            id: "step-1".into(),
            title: "Plan".into(),
            order: 0,
            ..Default::default()
        }],
        ..Default::default()
    }));
    state.reduce(TaskAction::ThreadTodosReceived {
        thread_id: "thread-1".into(),
        goal_run_id: Some("goal-1".into()),
        step_index: Some(0),
        items: vec![TodoItem {
            id: "todo-1".into(),
            content: "step todo".into(),
            status: Some(TodoStatus::Pending),
            step_index: Some(0),
            ..Default::default()
        }],
    });

    let todos = state.goal_step_todos("goal-1", 0);

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, "todo-1");
    assert_eq!(todos[0].content, "step todo");
}

#[test]
fn goal_scoped_live_todo_thread_is_linked_to_goal_run() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        steps: vec![GoalRunStep {
            id: "step-1".into(),
            title: "Plan".into(),
            order: 0,
            ..Default::default()
        }],
        ..Default::default()
    }));
    state.reduce(TaskAction::ThreadTodosReceived {
        thread_id: "thread-live".into(),
        goal_run_id: Some("goal-1".into()),
        step_index: Some(0),
        items: vec![TodoItem {
            id: "todo-live".into(),
            content: "live worker todo".into(),
            status: Some(TodoStatus::InProgress),
            step_index: Some(0),
            ..Default::default()
        }],
    });

    assert!(state.thread_belongs_to_goal_run("goal-1", "thread-live"));
}

#[test]
fn goal_step_todos_ignore_thread_scoped_child_todos_without_goal_binding() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        steps: vec![GoalRunStep {
            id: "step-1".into(),
            title: "Plan".into(),
            order: 0,
            ..Default::default()
        }],
        ..Default::default()
    }));
    state.reduce(TaskAction::TaskListReceived(vec![AgentTask {
        id: "task-1".into(),
        title: "Child task".into(),
        thread_id: Some("thread-task".into()),
        goal_run_id: Some("goal-1".into()),
        ..Default::default()
    }]));
    state.reduce(TaskAction::ThreadTodosReceived {
        thread_id: "thread-task".into(),
        goal_run_id: None,
        step_index: None,
        items: vec![TodoItem {
            id: "todo-1".into(),
            content: "step todo from child thread".into(),
            status: Some(TodoStatus::InProgress),
            step_index: Some(0),
            ..Default::default()
        }],
    });

    let todos = state.goal_step_todos("goal-1", 0);

    assert!(todos.is_empty());
}

#[test]
fn goal_thread_ids_include_spawned_descendants_of_goal_threads() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        active_thread_id: Some("thread-goal-active".into()),
        ..Default::default()
    }));
    state.reduce(TaskAction::TaskListReceived(vec![AgentTask {
        id: "spawned-task".into(),
        title: "Spawned worker".into(),
        thread_id: Some("thread-spawned".into()),
        parent_thread_id: Some("thread-goal-active".into()),
        goal_run_id: None,
        ..Default::default()
    }]));

    assert_eq!(
        state.goal_thread_ids("goal-1"),
        vec![
            "thread-goal-active".to_string(),
            "thread-spawned".to_string()
        ]
    );
    assert!(state.thread_belongs_to_goal_run("goal-1", "thread-spawned"));
}

#[test]
fn goal_step_todos_use_event_step_index_when_snapshot_items_lack_step_index() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        steps: vec![GoalRunStep {
            id: "step-1".into(),
            title: "Plan".into(),
            order: 0,
            ..Default::default()
        }],
        events: vec![GoalRunEvent {
            id: "event-1".into(),
            timestamp: 10,
            step_index: Some(0),
            todo_snapshot: vec![TodoItem {
                id: "todo-1".into(),
                content: "snapshot todo".into(),
                status: Some(TodoStatus::InProgress),
                step_index: None,
                position: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }));

    let todos = state.goal_step_todos("goal-1", 0);

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, "todo-1");
    assert_eq!(todos[0].content, "snapshot todo");
    assert_eq!(todos[0].status, Some(TodoStatus::InProgress));
}

#[test]
fn goal_step_details_filter_checkpoints_and_files_by_goal_and_step() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        thread_id: Some("thread-1".into()),
        steps: vec![
            GoalRunStep {
                id: "step-1".into(),
                title: "Plan".into(),
                order: 0,
                ..Default::default()
            },
            GoalRunStep {
                id: "step-2".into(),
                title: "Ship".into(),
                order: 1,
                ..Default::default()
            },
        ],
        ..Default::default()
    }));
    state.reduce(TaskAction::GoalRunCheckpointsReceived {
        goal_run_id: "goal-1".into(),
        checkpoints: vec![
            GoalRunCheckpointSummary {
                id: "checkpoint-1".into(),
                checkpoint_type: "plan".into(),
                step_index: Some(0),
                ..Default::default()
            },
            GoalRunCheckpointSummary {
                id: "checkpoint-2".into(),
                checkpoint_type: "ship".into(),
                step_index: Some(1),
                ..Default::default()
            },
        ],
    });
    state.reduce(TaskAction::WorkContextReceived(ThreadWorkContext {
        thread_id: "thread-1".into(),
        entries: vec![
            WorkContextEntry {
                path: "/tmp/plan.md".into(),
                goal_run_id: Some("goal-1".into()),
                step_index: Some(0),
                ..Default::default()
            },
            WorkContextEntry {
                path: "/tmp/ship.md".into(),
                goal_run_id: Some("goal-1".into()),
                step_index: Some(1),
                ..Default::default()
            },
            WorkContextEntry {
                path: "/tmp/other-goal.md".into(),
                goal_run_id: Some("goal-2".into()),
                step_index: Some(0),
                ..Default::default()
            },
        ],
    }));

    let checkpoint_ids = state
        .goal_step_checkpoints("goal-1", 0)
        .iter()
        .map(|checkpoint| checkpoint.id.as_str())
        .collect::<Vec<_>>();
    let file_paths = state
        .goal_step_files("goal-1", "thread-1", 0)
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>();

    assert_eq!(checkpoint_ids, vec!["checkpoint-1"]);
    assert_eq!(file_paths, vec!["/tmp/plan.md"]);
}

#[test]
fn goal_run_detail_received_parses_mission_control_metadata() {
    let wire_run = crate::wire::GoalRun {
        id: "g1".into(),
        title: "Mission Control".into(),
        launch_assignment_snapshot: vec![
            make_wire_assignment("planner", true, "openai", "gpt-4.1", Some("high"), true),
            make_wire_assignment(
                "validator",
                false,
                "anthropic",
                "claude-sonnet-4",
                None,
                false,
            ),
        ],
        runtime_assignment_list: vec![make_wire_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("medium"),
            true,
        )],
        root_thread_id: Some("thread-root".into()),
        active_thread_id: Some("thread-active".into()),
        execution_thread_ids: vec!["thread-root".into(), "thread-active".into()],
        ..Default::default()
    };

    let converted = crate::app::conversion::convert_goal_run(wire_run);

    assert_eq!(
        converted.launch_assignment_snapshot,
        vec![
            make_assignment("planner", true, "openai", "gpt-4.1", Some("high"), true),
            make_assignment(
                "validator",
                false,
                "anthropic",
                "claude-sonnet-4",
                None,
                false
            ),
        ]
    );
    assert_eq!(
        converted.runtime_assignment_list,
        vec![make_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("medium"),
            true,
        )]
    );
    assert_eq!(converted.root_thread_id.as_deref(), Some("thread-root"));
    assert_eq!(converted.active_thread_id.as_deref(), Some("thread-active"));
    assert_eq!(
        converted.execution_thread_ids,
        vec!["thread-root".to_string(), "thread-active".to_string()]
    );

    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(converted));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.launch_assignment_snapshot.len(), 2);
    assert_eq!(goal.runtime_assignment_list.len(), 1);
    assert_eq!(goal.root_thread_id.as_deref(), Some("thread-root"));
    assert_eq!(goal.active_thread_id.as_deref(), Some("thread-active"));
    assert_eq!(
        goal.execution_thread_ids,
        vec!["thread-root".to_string(), "thread-active".to_string()]
    );
}

#[test]
fn goal_run_detail_received_replaces_mission_control_metadata() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        launch_assignment_snapshot: vec![make_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("high"),
            true,
        )],
        runtime_assignment_list: vec![make_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("high"),
            true,
        )],
        root_thread_id: Some("thread-root".into()),
        active_thread_id: Some("thread-active".into()),
        execution_thread_ids: vec!["thread-root".into(), "thread-active".into()],
        ..Default::default()
    }));

    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Detailed".into(),
        launch_assignment_snapshot: Vec::new(),
        runtime_assignment_list: Vec::new(),
        root_thread_id: None,
        active_thread_id: None,
        execution_thread_ids: Vec::new(),
        ..Default::default()
    }));

    let goal = state.goal_run_by_id("g1").expect("goal should exist");
    assert_eq!(goal.title, "Detailed");
    assert!(goal.launch_assignment_snapshot.is_empty());
    assert!(goal.runtime_assignment_list.is_empty());
    assert!(goal.root_thread_id.is_none());
    assert!(goal.active_thread_id.is_none());
    assert!(goal.execution_thread_ids.is_empty());
}

#[test]
fn goal_run_update_preserves_mission_control_metadata_when_omitted() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Original".into(),
        launch_assignment_snapshot: vec![make_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("high"),
            true,
        )],
        runtime_assignment_list: vec![make_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("high"),
            true,
        )],
        root_thread_id: Some("thread-root".into()),
        active_thread_id: Some("thread-active".into()),
        execution_thread_ids: vec!["thread-root".into(), "thread-active".into()],
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
        goal.launch_assignment_snapshot,
        vec![make_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("high"),
            true,
        )]
    );
    assert_eq!(
        goal.runtime_assignment_list,
        vec![make_assignment(
            "planner",
            true,
            "openai",
            "gpt-4.1",
            Some("high"),
            true,
        )]
    );
    assert_eq!(goal.root_thread_id.as_deref(), Some("thread-root"));
    assert_eq!(goal.active_thread_id.as_deref(), Some("thread-active"));
    assert_eq!(
        goal.execution_thread_ids,
        vec!["thread-root".to_string(), "thread-active".to_string()]
    );
}
