use super::task_list_received_replaces_tasks_to_goal_step_todos_use_latest_event::*;
use crate::state::spawned_tree::derive_spawned_agent_tree;
use crate::state::task::*;
#[test]
fn spawned_tree_uses_parent_thread_id_for_visible_roots() {
    let tasks = vec![
        make_task(
            "orphan-root",
            "Orphan Root",
            10,
            Some("thread-orphan-root"),
            None,
            None,
            Some(TaskStatus::InProgress),
        ),
        make_task(
            "spawned-root",
            "Spawned Root",
            20,
            Some("thread-spawned"),
            Some("missing-parent"),
            Some("thread-parent"),
            Some(TaskStatus::InProgress),
        ),
    ];

    let tree = derive_spawned_agent_tree(&tasks, Some("thread-parent")).expect("tree");
    assert!(tree.anchor.is_none());
    assert_eq!(
        tree.roots
            .iter()
            .map(|node| node.item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["spawned-root"]
    );
    assert!(tree.roots[0].openable);
}

#[test]
fn spawned_tree_keeps_partial_branches_when_parents_are_missing() {
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
            "orphan-child",
            "Orphan Child",
            20,
            Some("thread-orphan"),
            Some("missing-parent"),
            Some("thread-root"),
            Some(TaskStatus::InProgress),
        ),
        make_task(
            "grandchild",
            "Grandchild",
            30,
            Some("thread-grandchild"),
            Some("orphan-child"),
            Some("thread-orphan"),
            Some(TaskStatus::Completed),
        ),
    ];

    let tree = derive_spawned_agent_tree(&tasks, Some("thread-root")).expect("tree");
    assert_eq!(
        tree.roots
            .iter()
            .map(|node| node.item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["orphan-child"]
    );
    assert_eq!(
        tree.roots[0]
            .children
            .first()
            .map(|node| node.item.id.as_str()),
        Some("grandchild")
    );
    assert!(tree.roots[0].children[0].openable);
    assert!(!tree.roots[0].children[0].live);
}

#[test]
fn spawned_tree_blocks_self_referential_child_loops() {
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
            Some("thread-child"),
            Some(TaskStatus::InProgress),
        ),
    ];

    let tree = derive_spawned_agent_tree(&tasks, Some("thread-root")).expect("tree");
    let child = tree
        .anchor
        .as_ref()
        .and_then(|node| node.children.first())
        .expect("child");
    assert_eq!(child.item.id, "child-task");
    assert!(child.children.is_empty());
}

#[test]
fn spawned_tree_does_not_reintroduce_cycles_as_descendants() {
    let tasks = vec![
        make_task(
            "root-task",
            "Root",
            10,
            Some("thread-root"),
            Some("cycle-child"),
            Some("thread-root"),
            Some(TaskStatus::InProgress),
        ),
        make_task(
            "cycle-child",
            "Cycle Child",
            20,
            Some("thread-child"),
            Some("root-task"),
            Some("thread-root"),
            Some(TaskStatus::InProgress),
        ),
    ];

    let tree = derive_spawned_agent_tree(&tasks, Some("thread-root")).expect("tree");
    let root = tree.anchor.as_ref().expect("anchor");
    assert_eq!(root.item.id, "root-task");
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].item.id, "cycle-child");
    assert!(root.children[0].children.is_empty());
}

#[test]
fn heartbeat_digest_received_stores_and_replaces() {
    let mut state = TaskState::new();
    assert!(state.last_digest().is_none());
    state.reduce(TaskAction::HeartbeatDigestReceived(HeartbeatDigestVm {
        cycle_id: "c1".into(),
        actionable: true,
        digest: "2 items".into(),
        items: vec![HeartbeatDigestItemVm {
            priority: 1,
            check_type: "stale_todos".into(),
            title: "3 stale TODOs".into(),
            suggestion: "Review them".into(),
        }],
        checked_at: 1234567890,
        explanation: None,
    }));
    let d = state.last_digest().unwrap();
    assert_eq!(d.cycle_id, "c1");
    assert!(d.actionable);
    assert_eq!(d.items.len(), 1);
    assert_eq!(d.items[0].priority, 1);

    state.reduce(TaskAction::HeartbeatDigestReceived(HeartbeatDigestVm {
        cycle_id: "c2".into(),
        actionable: false,
        digest: "All clear".into(),
        items: vec![],
        checked_at: 1234567891,
        explanation: None,
    }));
    let d = state.last_digest().unwrap();
    assert_eq!(d.cycle_id, "c2");
    assert!(!d.actionable);
    assert!(d.items.is_empty());
}
