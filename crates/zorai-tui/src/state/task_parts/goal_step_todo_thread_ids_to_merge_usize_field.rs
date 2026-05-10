use super::task_status_to_task_state::*;
pub(super) fn goal_step_todo_thread_ids(state: &TaskState, run: &GoalRun) -> Vec<String> {
    let mut thread_ids = Vec::new();
    let mut task_ids = Vec::new();

    for thread_id in run
        .active_thread_id
        .iter()
        .chain(run.root_thread_id.iter())
        .chain(run.thread_id.iter())
    {
        push_unique_id(&mut thread_ids, thread_id);
    }
    for thread_id in &run.execution_thread_ids {
        push_unique_id(&mut thread_ids, thread_id);
    }
    for task in state
        .tasks()
        .iter()
        .filter(|task| task.goal_run_id.as_deref() == Some(run.id.as_str()))
    {
        push_unique_id(&mut task_ids, &task.id);
        if let Some(thread_id) = task.thread_id.as_deref() {
            push_unique_id(&mut thread_ids, thread_id);
        }
    }
    if let Some(goal_threads) = state.goal_thread_ids.get(&run.id) {
        for thread_id in goal_threads {
            push_unique_id(&mut thread_ids, thread_id);
        }
    }
    loop {
        let mut changed = false;
        for task in state.tasks() {
            let belongs_to_goal = task.goal_run_id.as_deref() == Some(run.id.as_str())
                || task
                    .parent_task_id
                    .as_deref()
                    .is_some_and(|parent_task_id| task_ids.iter().any(|id| id == parent_task_id))
                || task
                    .parent_thread_id
                    .as_deref()
                    .is_some_and(|parent_thread_id| {
                        thread_ids.iter().any(|id| id == parent_thread_id)
                    });
            if !belongs_to_goal {
                continue;
            }

            changed |= push_unique_id(&mut task_ids, &task.id);

            if let Some(thread_id) = task.thread_id.as_deref() {
                changed |= push_unique_id(&mut thread_ids, thread_id);
            }
        }
        if !changed {
            break;
        }
    }

    thread_ids
}

pub(super) fn push_unique_id(ids: &mut Vec<String>, id: &str) -> bool {
    if id.is_empty() || ids.iter().any(|existing| existing == id) {
        return false;
    }
    ids.push(id.to_string());
    true
}

pub(super) fn remember_goal_thread(
    goal_thread_ids: &mut std::collections::HashMap<String, Vec<String>>,
    goal_run_id: &str,
    thread_id: &str,
) {
    if goal_run_id.is_empty() || thread_id.is_empty() {
        return;
    }
    let threads = goal_thread_ids.entry(goal_run_id.to_string()).or_default();
    if !threads.iter().any(|existing| existing == thread_id) {
        threads.push(thread_id.to_string());
    }
}

pub(super) fn goal_step_live_todo_key(goal_run_id: &str, step_index: usize) -> String {
    format!("{goal_run_id}::{step_index}")
}

pub(super) fn reconcile_goal_run_status_from_tasks(tasks: &[AgentTask], goal_runs: &mut [GoalRun]) {
    for goal_run in goal_runs {
        if matches!(
            goal_run.status,
            Some(GoalRunStatus::Completed | GoalRunStatus::Failed | GoalRunStatus::Cancelled)
                | Some(GoalRunStatus::Planning)
        ) {
            continue;
        }

        let mut next_status = None;
        for task in tasks
            .iter()
            .filter(|task| task.goal_run_id.as_deref() == Some(goal_run.id.as_str()))
        {
            match task.status {
                Some(TaskStatus::AwaitingApproval) => {
                    next_status = Some(GoalRunStatus::AwaitingApproval);
                    break;
                }
                Some(
                    TaskStatus::Queued
                    | TaskStatus::InProgress
                    | TaskStatus::Blocked
                    | TaskStatus::FailedAnalyzing,
                ) => {
                    next_status = Some(GoalRunStatus::Running);
                }
                _ => {}
            }
        }

        if let Some(next_status) = next_status {
            goal_run.status = Some(next_status);
        }
    }
}

pub(super) fn merge_task_update(existing: &AgentTask, mut updated: AgentTask) -> AgentTask {
    if updated.description.is_empty() {
        updated.description = existing.description.clone();
    }
    if updated.thread_id.is_none() {
        updated.thread_id = existing.thread_id.clone();
    }
    if updated.parent_task_id.is_none() {
        updated.parent_task_id = existing.parent_task_id.clone();
    }
    if updated.parent_thread_id.is_none() {
        updated.parent_thread_id = existing.parent_thread_id.clone();
    }
    if updated.created_at == 0 {
        updated.created_at = existing.created_at;
    }
    if updated.session_id.is_none() {
        updated.session_id = existing.session_id.clone();
    }
    if updated.goal_run_id.is_none() {
        updated.goal_run_id = existing.goal_run_id.clone();
    }
    if updated.goal_step_title.is_none() {
        updated.goal_step_title = existing.goal_step_title.clone();
    }
    if updated.command.is_none() {
        updated.command = existing.command.clone();
    }
    let effective_status = updated.status.or(existing.status);
    if updated.awaiting_approval_id.is_none()
        && matches!(effective_status, Some(TaskStatus::AwaitingApproval))
    {
        updated.awaiting_approval_id = existing.awaiting_approval_id.clone();
    }
    if updated.blocked_reason.is_none() {
        updated.blocked_reason = existing.blocked_reason.clone();
    }

    updated
}

impl crate::state::spawned_tree::SpawnedAgentTreeSource for AgentTask {
    fn spawned_tree_identity(&self) -> &str {
        &self.id
    }

    fn spawned_tree_created_at(&self) -> u64 {
        self.created_at
    }

    fn spawned_tree_thread_id(&self) -> Option<&str> {
        self.thread_id.as_deref()
    }

    fn spawned_tree_parent_task_id(&self) -> Option<&str> {
        self.parent_task_id.as_deref()
    }

    fn spawned_tree_parent_thread_id(&self) -> Option<&str> {
        self.parent_thread_id.as_deref()
    }

    fn spawned_tree_status(&self) -> Option<TaskStatus> {
        self.status
    }
}

impl Default for TaskState {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn normalize_goal_run_ranges(mut run: GoalRun) -> GoalRun {
    if run.total_step_count == 0 {
        run.total_step_count = run.steps.len();
    }
    if run.loaded_step_end == 0 && !run.steps.is_empty() {
        run.loaded_step_end = run.loaded_step_start.saturating_add(run.steps.len());
    }
    if run.loaded_step_end < run.loaded_step_start {
        run.loaded_step_end = run.loaded_step_start;
    }
    if run.loaded_step_start == 0
        && run.loaded_step_end == 0
        && run.total_step_count == run.steps.len()
    {
        run.loaded_step_end = run.steps.len();
    }
    run.total_step_count = run.total_step_count.max(run.loaded_step_end);

    if run.total_event_count == 0 {
        run.total_event_count = run.events.len();
    }
    if run.loaded_event_end == 0 && !run.events.is_empty() {
        run.loaded_event_end = run.loaded_event_start.saturating_add(run.events.len());
    }
    if run.loaded_event_end < run.loaded_event_start {
        run.loaded_event_end = run.loaded_event_start;
    }
    if run.loaded_event_start == 0
        && run.loaded_event_end == 0
        && run.total_event_count == run.events.len()
    {
        run.loaded_event_end = run.events.len();
    }
    run.total_event_count = run.total_event_count.max(run.loaded_event_end);
    run
}

pub(super) fn merge_range_vec<T: Clone>(
    existing_start: usize,
    existing_end: usize,
    existing_items: &[T],
    incoming_start: usize,
    incoming_end: usize,
    incoming_items: &[T],
) -> (usize, usize, Vec<T>) {
    if existing_items.is_empty() || existing_start == existing_end {
        return (incoming_start, incoming_end, incoming_items.to_vec());
    }
    if incoming_items.is_empty() || incoming_start == incoming_end {
        return (existing_start, existing_end, existing_items.to_vec());
    }

    if incoming_end <= existing_start {
        let mut merged = incoming_items.to_vec();
        merged.extend_from_slice(existing_items);
        return (incoming_start, existing_end, merged);
    }
    if existing_end <= incoming_start {
        let mut merged = existing_items.to_vec();
        merged.extend_from_slice(incoming_items);
        return (existing_start, incoming_end, merged);
    }

    let union_start = existing_start.min(incoming_start);
    let union_end = existing_end.max(incoming_end);
    let mut merged = Vec::with_capacity(union_end.saturating_sub(union_start));
    for absolute_idx in union_start..union_end {
        if absolute_idx >= incoming_start && absolute_idx < incoming_end {
            merged.push(incoming_items[absolute_idx - incoming_start].clone());
        } else if absolute_idx >= existing_start && absolute_idx < existing_end {
            merged.push(existing_items[absolute_idx - existing_start].clone());
        }
    }
    (union_start, union_end, merged)
}

pub(super) fn merge_optional_field<T>(
    existing: &mut Option<T>,
    incoming: Option<T>,
    preserve_existing: bool,
) {
    if preserve_existing {
        if incoming.is_some() {
            *existing = incoming;
        }
    } else {
        *existing = incoming;
    }
}

pub(super) fn merge_vec_field<T>(
    existing: &mut Vec<T>,
    incoming: Vec<T>,
    preserve_existing_when_empty: bool,
) {
    if preserve_existing_when_empty && incoming.is_empty() {
        return;
    }
    *existing = incoming;
}

pub(super) fn merge_string_field(
    existing: &mut String,
    incoming: String,
    preserve_existing_when_empty: bool,
) {
    if preserve_existing_when_empty && incoming.is_empty() {
        return;
    }
    *existing = incoming;
}

pub(super) fn merge_u32_field(
    existing: &mut u32,
    incoming: u32,
    preserve_existing_when_zero: bool,
) {
    if preserve_existing_when_zero && incoming == 0 && *existing != 0 {
        return;
    }
    *existing = incoming;
}

pub(super) fn merge_u64_field(
    existing: &mut u64,
    incoming: u64,
    preserve_existing_when_zero: bool,
) {
    if preserve_existing_when_zero && incoming == 0 && *existing != 0 {
        return;
    }
    *existing = incoming;
}

pub(super) fn merge_usize_field(
    existing: &mut usize,
    incoming: usize,
    preserve_existing_when_zero: bool,
) {
    if preserve_existing_when_zero && incoming == 0 && *existing != 0 {
        return;
    }
    *existing = incoming;
}
