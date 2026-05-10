use crate::state::spawned_tree::{derive_spawned_agent_tree, SpawnedAgentTreeNode};
use crate::state::task::{AgentTask, TaskState};

#[cfg(test)]
thread_local! {
    static FLATTENED_ITEMS_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[derive(Debug, Clone)]
pub(crate) struct SpawnedSidebarItem {
    pub(crate) depth: usize,
    pub(crate) title: String,
    pub(crate) target_thread_id: Option<String>,
    pub(crate) is_active: bool,
    pub(crate) openable: bool,
    pub(crate) live: bool,
}

fn branch_target_thread_id(
    node: &SpawnedAgentTreeNode<AgentTask>,
    active_thread_id: &str,
) -> Option<String> {
    if node.item.thread_id.as_deref() != Some(active_thread_id) {
        if let Some(thread_id) = node.item.thread_id.clone() {
            return Some(thread_id);
        }
    }

    node.children
        .iter()
        .find_map(|child| branch_target_thread_id(child, active_thread_id))
}

fn push_node_rows(
    rows: &mut Vec<SpawnedSidebarItem>,
    node: &SpawnedAgentTreeNode<AgentTask>,
    depth: usize,
    active_thread_id: &str,
) {
    rows.push(SpawnedSidebarItem {
        depth,
        title: node.item.title.clone(),
        target_thread_id: branch_target_thread_id(node, active_thread_id),
        is_active: node.item.thread_id.as_deref() == Some(active_thread_id),
        openable: node.openable && node.item.thread_id.as_deref() != Some(active_thread_id),
        live: node.live,
    });

    for child in &node.children {
        push_node_rows(rows, child, depth + 1, active_thread_id);
    }
}

pub(super) fn flattened_items(
    tasks: &TaskState,
    thread_id: Option<&str>,
) -> Vec<SpawnedSidebarItem> {
    #[cfg(test)]
    FLATTENED_ITEMS_CALLS.with(|calls| calls.set(calls.get() + 1));

    let Some(tree) = derive_spawned_agent_tree(tasks.spawned_tree_items(), thread_id) else {
        return Vec::new();
    };

    let mut rows = Vec::new();
    if let Some(anchor) = tree.anchor.as_ref() {
        push_node_rows(&mut rows, anchor, 0, tree.active_thread_id.as_str());
        for root in &tree.roots {
            push_node_rows(&mut rows, root, 1, tree.active_thread_id.as_str());
        }
    } else {
        for root in &tree.roots {
            push_node_rows(&mut rows, root, 0, tree.active_thread_id.as_str());
        }
    }
    rows
}

pub(super) fn has_content(tasks: &TaskState, thread_id: Option<&str>) -> bool {
    let Some(active_thread_id) = thread_id.filter(|thread_id| !thread_id.is_empty()) else {
        return false;
    };

    tasks.spawned_tree_items().iter().any(|task| {
        task.thread_id.as_deref() == Some(active_thread_id)
            || task.parent_thread_id.as_deref() == Some(active_thread_id)
    })
}

pub(super) fn selected_thread_id(
    tasks: &TaskState,
    selected_index: usize,
    thread_id: Option<&str>,
) -> Option<String> {
    flattened_items(tasks, thread_id)
        .get(selected_index)
        .and_then(|item| item.target_thread_id.clone())
}

pub(super) fn first_openable_index(tasks: &TaskState, thread_id: Option<&str>) -> Option<usize> {
    flattened_items(tasks, thread_id)
        .iter()
        .position(|item| item.openable)
}

#[cfg(test)]
pub(super) fn reset_flattened_items_call_count() {
    FLATTENED_ITEMS_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
pub(super) fn flattened_items_call_count() -> usize {
    FLATTENED_ITEMS_CALLS.with(std::cell::Cell::get)
}
