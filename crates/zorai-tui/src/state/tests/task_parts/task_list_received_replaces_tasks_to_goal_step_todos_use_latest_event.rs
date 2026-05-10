use crate::state::task::*;

pub(super) fn make_task(
    id: &str,
    title: &str,
    created_at: u64,
    thread_id: Option<&str>,
    parent_task_id: Option<&str>,
    parent_thread_id: Option<&str>,
    status: Option<TaskStatus>,
) -> AgentTask {
    AgentTask {
        id: id.into(),
        title: title.into(),
        created_at,
        thread_id: thread_id.map(str::to_string),
        parent_task_id: parent_task_id.map(str::to_string),
        parent_thread_id: parent_thread_id.map(str::to_string),
        status,
        ..Default::default()
    }
}

pub(super) fn make_goal_run(id: &str, title: &str) -> GoalRun {
    GoalRun {
        id: id.into(),
        title: title.into(),
        ..Default::default()
    }
}

pub(super) fn make_owner_profile(
    agent_label: &str,
    provider: &str,
    model: &str,
    reasoning_effort: Option<&str>,
) -> GoalRuntimeOwnerProfile {
    GoalRuntimeOwnerProfile {
        agent_label: agent_label.into(),
        provider: provider.into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.map(str::to_string),
    }
}

pub(super) fn make_wire_owner_profile(
    agent_label: &str,
    provider: &str,
    model: &str,
    reasoning_effort: Option<&str>,
) -> crate::wire::GoalRuntimeOwnerProfile {
    crate::wire::GoalRuntimeOwnerProfile {
        agent_label: agent_label.into(),
        provider: provider.into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.map(str::to_string),
    }
}

pub(super) fn make_assignment(
    role_id: &str,
    enabled: bool,
    provider: &str,
    model: &str,
    reasoning_effort: Option<&str>,
    inherit_from_main: bool,
) -> GoalAgentAssignment {
    GoalAgentAssignment {
        role_id: role_id.into(),
        enabled,
        provider: provider.into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.map(str::to_string),
        inherit_from_main,
    }
}

pub(super) fn make_wire_assignment(
    role_id: &str,
    enabled: bool,
    provider: &str,
    model: &str,
    reasoning_effort: Option<&str>,
    inherit_from_main: bool,
) -> crate::wire::GoalAgentAssignment {
    crate::wire::GoalAgentAssignment {
        role_id: role_id.into(),
        enabled,
        provider: provider.into(),
        model: model.into(),
        reasoning_effort: reasoning_effort.map(str::to_string),
        inherit_from_main,
    }
}

#[test]
fn task_list_received_replaces_tasks() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::TaskListReceived(vec![
        make_task("t1", "First", 1, None, None, None, None),
        make_task("t2", "Second", 2, None, None, None, None),
    ]));
    assert_eq!(state.tasks().len(), 2);

    state.reduce(TaskAction::TaskListReceived(vec![make_task(
        "t3", "Third", 3, None, None, None, None,
    )]));
    assert_eq!(state.tasks().len(), 1);
    assert_eq!(state.tasks()[0].id, "t3");
}

#[test]
fn task_update_upserts_by_id() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::TaskListReceived(vec![make_task(
        "t1", "Original", 1, None, None, None, None,
    )]));

    state.reduce(TaskAction::TaskUpdate(AgentTask {
        id: "t1".into(),
        title: "Updated".into(),
        status: Some(TaskStatus::InProgress),
        ..Default::default()
    }));
    assert_eq!(state.tasks().len(), 1);
    assert_eq!(state.tasks()[0].title, "Updated");
    assert_eq!(state.tasks()[0].status, Some(TaskStatus::InProgress));

    state.reduce(TaskAction::TaskUpdate(make_task(
        "t2",
        "New",
        2,
        None,
        None,
        None,
        Some(TaskStatus::Queued),
    )));
    assert_eq!(state.tasks().len(), 2);
}

#[test]
fn goal_run_list_received_replaces_goal_runs() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunListReceived(vec![
        make_goal_run("g1", "Goal One"),
        make_goal_run("g2", "Goal Two"),
    ]));
    assert_eq!(state.goal_runs().len(), 2);

    state.reduce(TaskAction::GoalRunListReceived(vec![]));
    assert_eq!(state.goal_runs().len(), 0);
}

#[test]
fn goal_run_detail_received_upserts() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunListReceived(vec![make_goal_run(
        "g1", "Original",
    )]));

    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "g1".into(),
        title: "Detailed".into(),
        ..Default::default()
    }));
    assert_eq!(state.goal_runs().len(), 1);
    assert_eq!(state.goal_runs()[0].title, "Detailed");

    state.reduce(TaskAction::GoalRunUpdate(make_goal_run("g2", "New Goal")));
    assert_eq!(state.goal_runs().len(), 2);
}

#[test]
fn active_goal_linked_task_update_revives_paused_goal_status() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        status: Some(GoalRunStatus::Paused),
        ..Default::default()
    }));

    state.reduce(TaskAction::TaskUpdate(AgentTask {
        id: "task-1".into(),
        title: "Child task".into(),
        goal_run_id: Some("goal-1".into()),
        status: Some(TaskStatus::InProgress),
        ..Default::default()
    }));

    assert_eq!(
        state.goal_run_by_id("goal-1").and_then(|goal| goal.status),
        Some(GoalRunStatus::Running)
    );
}

#[test]
fn new_goal_run_updates_are_inserted_first() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunListReceived(vec![
        make_goal_run("g1", "Goal One"),
        make_goal_run("g2", "Goal Two"),
    ]));

    state.reduce(TaskAction::GoalRunUpdate(make_goal_run(
        "g3",
        "Newest Goal",
    )));

    assert_eq!(
        state
            .goal_runs()
            .iter()
            .map(|run| run.id.as_str())
            .collect::<Vec<_>>(),
        vec!["g3", "g1", "g2"]
    );
}

#[test]
fn goal_steps_in_display_order_sorts_by_step_order() {
    let mut state = TaskState::new();
    state.reduce(TaskAction::GoalRunDetailReceived(GoalRun {
        id: "goal-1".into(),
        title: "Goal".into(),
        steps: vec![
            GoalRunStep {
                id: "step-3".into(),
                title: "Third".into(),
                order: 2,
                ..Default::default()
            },
            GoalRunStep {
                id: "step-1".into(),
                title: "First".into(),
                order: 0,
                ..Default::default()
            },
            GoalRunStep {
                id: "step-2".into(),
                title: "Second".into(),
                order: 1,
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let step_ids = state
        .goal_steps_in_display_order("goal-1")
        .iter()
        .map(|step| step.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(step_ids, vec!["step-1", "step-2", "step-3"]);
}

#[test]
fn goal_step_todos_use_live_goal_step_replacement_over_stale_event_snapshot() {
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
        events: vec![
            GoalRunEvent {
                id: "event-old".into(),
                timestamp: 10,
                step_index: Some(0),
                todo_snapshot: vec![
                    TodoItem {
                        id: "todo-1".into(),
                        content: "old snapshot item".into(),
                        status: Some(TodoStatus::Pending),
                        step_index: Some(0),
                        position: 0,
                        ..Default::default()
                    },
                    TodoItem {
                        id: "todo-2".into(),
                        content: "current todo two".into(),
                        status: Some(TodoStatus::Pending),
                        step_index: Some(0),
                        position: 1,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            GoalRunEvent {
                id: "event-new".into(),
                timestamp: 20,
                step_index: Some(0),
                todo_snapshot: vec![TodoItem {
                    id: "todo-2".into(),
                    content: "current todo two".into(),
                    status: Some(TodoStatus::Pending),
                    step_index: Some(0),
                    position: 1,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    }));
    state.reduce(TaskAction::ThreadTodosReceived {
        thread_id: "thread-1".into(),
        goal_run_id: Some("goal-1".into()),
        step_index: Some(0),
        items: vec![TodoItem {
            id: "todo-2".into(),
            content: "current todo two".into(),
            status: Some(TodoStatus::InProgress),
            step_index: Some(0),
            position: 1,
            ..Default::default()
        }],
    });

    let todos = state.goal_step_todos("goal-1", 0);
    let todo_ids = todos
        .iter()
        .map(|todo| todo.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(todo_ids, vec!["todo-2"]);
    assert_eq!(todos[0].content, "current todo two");
    assert_eq!(todos[0].status, Some(TodoStatus::InProgress));
}

#[test]
fn goal_step_todos_use_latest_status_when_same_content_has_new_id() {
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
        events: vec![GoalRunEvent {
            id: "event-old".into(),
            timestamp: 10,
            step_index: Some(0),
            todo_snapshot: vec![TodoItem {
                id: "todo-old".into(),
                content: "Run acceptance checklist for coverage".into(),
                status: Some(TodoStatus::Pending),
                step_index: Some(0),
                position: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }));
    state.reduce(TaskAction::ThreadTodosReceived {
        thread_id: "thread-1".into(),
        goal_run_id: Some("goal-1".into()),
        step_index: Some(0),
        items: vec![TodoItem {
            id: "todo-new".into(),
            content: "Run acceptance checklist for coverage".into(),
            status: Some(TodoStatus::InProgress),
            step_index: Some(0),
            position: 0,
            ..Default::default()
        }],
    });

    let todos = state.goal_step_todos("goal-1", 0);

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].content, "Run acceptance checklist for coverage");
    assert_eq!(todos[0].status, Some(TodoStatus::InProgress));
    assert_eq!(todos[0].id, "todo-new");
}

#[test]
fn goal_step_todos_use_latest_event_snapshot_without_resurrecting_removed_items() {
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
        events: vec![
            GoalRunEvent {
                id: "event-old".into(),
                timestamp: 10,
                step_index: Some(0),
                todo_snapshot: vec![
                    TodoItem {
                        id: "todo-removed".into(),
                        content: "removed stale item".into(),
                        status: Some(TodoStatus::Pending),
                        step_index: Some(0),
                        position: 0,
                        ..Default::default()
                    },
                    TodoItem {
                        id: "todo-current".into(),
                        content: "current item".into(),
                        status: Some(TodoStatus::Pending),
                        step_index: Some(0),
                        position: 1,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            GoalRunEvent {
                id: "event-new".into(),
                timestamp: 20,
                step_index: Some(0),
                todo_snapshot: vec![TodoItem {
                    id: "todo-current".into(),
                    content: "current item".into(),
                    status: Some(TodoStatus::InProgress),
                    step_index: Some(0),
                    position: 0,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let todos = state.goal_step_todos("goal-1", 0);

    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, "todo-current");
    assert_eq!(todos[0].content, "current item");
    assert_eq!(todos[0].status, Some(TodoStatus::InProgress));
}
