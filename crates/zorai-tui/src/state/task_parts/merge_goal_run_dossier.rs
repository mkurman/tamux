use super::goal_step_todo_thread_ids_to_merge_usize_field::*;
use super::task_status_to_task_state::*;
pub(super) fn merge_goal_run(
    existing: &mut GoalRun,
    incoming: GoalRun,
    preserve_owner_metadata: bool,
) {
    let preserve_sparse_fields = preserve_owner_metadata && incoming.sparse_update;
    let older_page_request_cooldown_until_tick = existing
        .older_page_request_cooldown_until_tick
        .max(incoming.older_page_request_cooldown_until_tick);

    if preserve_sparse_fields {
        if existing.title.is_empty() {
            existing.title = incoming.title;
        }
    } else {
        existing.title = incoming.title;
    }
    merge_optional_field(
        &mut existing.thread_id,
        incoming.thread_id,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.session_id,
        incoming.session_id,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.status,
        incoming.status,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.current_step_title,
        incoming.current_step_title,
        preserve_sparse_fields,
    );
    merge_vec_field(
        &mut existing.launch_assignment_snapshot,
        incoming.launch_assignment_snapshot,
        preserve_sparse_fields,
    );
    merge_vec_field(
        &mut existing.runtime_assignment_list,
        incoming.runtime_assignment_list,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.root_thread_id,
        incoming.root_thread_id,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.active_thread_id,
        incoming.active_thread_id,
        preserve_sparse_fields,
    );
    merge_vec_field(
        &mut existing.execution_thread_ids,
        incoming.execution_thread_ids,
        preserve_sparse_fields,
    );
    if preserve_sparse_fields {
        existing.planner_owner_profile = incoming
            .planner_owner_profile
            .or(existing.planner_owner_profile.take());
        existing.current_step_owner_profile = incoming
            .current_step_owner_profile
            .or(existing.current_step_owner_profile.take());
    } else {
        existing.planner_owner_profile = incoming.planner_owner_profile;
        existing.current_step_owner_profile = incoming.current_step_owner_profile;
    }
    merge_u64_field(
        &mut existing.total_prompt_tokens,
        incoming.total_prompt_tokens,
        preserve_sparse_fields,
    );
    merge_u64_field(
        &mut existing.total_completion_tokens,
        incoming.total_completion_tokens,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.estimated_cost_usd,
        incoming.estimated_cost_usd,
        preserve_sparse_fields,
    );
    merge_vec_field(
        &mut existing.model_usage,
        incoming.model_usage,
        preserve_sparse_fields,
    );
    merge_u32_field(
        &mut existing.child_task_count,
        incoming.child_task_count,
        preserve_sparse_fields,
    );
    merge_u32_field(
        &mut existing.approval_count,
        incoming.approval_count,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.awaiting_approval_id,
        incoming.awaiting_approval_id,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.last_error,
        incoming.last_error,
        preserve_sparse_fields,
    );
    merge_string_field(&mut existing.goal, incoming.goal, preserve_sparse_fields);
    merge_usize_field(
        &mut existing.current_step_index,
        incoming.current_step_index,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.reflection_summary,
        incoming.reflection_summary,
        preserve_sparse_fields,
    );
    merge_vec_field(
        &mut existing.memory_updates,
        incoming.memory_updates,
        preserve_sparse_fields,
    );
    merge_optional_field(
        &mut existing.generated_skill_path,
        incoming.generated_skill_path,
        preserve_sparse_fields,
    );
    merge_vec_field(
        &mut existing.child_task_ids,
        incoming.child_task_ids,
        preserve_sparse_fields,
    );
    existing.dossier = merge_goal_run_dossier(
        existing.dossier.take(),
        incoming.dossier,
        preserve_sparse_fields,
    );
    merge_u64_field(
        &mut existing.created_at,
        incoming.created_at,
        preserve_sparse_fields,
    );
    merge_u64_field(
        &mut existing.updated_at,
        incoming.updated_at,
        preserve_sparse_fields,
    );
    if preserve_sparse_fields {
        existing.total_step_count = existing.total_step_count.max(incoming.total_step_count);
        existing.total_event_count = existing.total_event_count.max(incoming.total_event_count);
    } else {
        existing.total_step_count = incoming.total_step_count;
        existing.total_event_count = incoming.total_event_count;
    }

    let (loaded_step_start, loaded_step_end, steps) = merge_range_vec(
        existing.loaded_step_start,
        existing.loaded_step_end,
        &existing.steps,
        incoming.loaded_step_start,
        incoming.loaded_step_end,
        &incoming.steps,
    );
    existing.loaded_step_start = loaded_step_start;
    existing.loaded_step_end = loaded_step_end;
    existing.steps = steps;

    let (loaded_event_start, loaded_event_end, events) = merge_range_vec(
        existing.loaded_event_start,
        existing.loaded_event_end,
        &existing.events,
        incoming.loaded_event_start,
        incoming.loaded_event_end,
        &incoming.events,
    );
    existing.loaded_event_start = loaded_event_start;
    existing.loaded_event_end = loaded_event_end;
    existing.events = events;

    existing.older_page_pending = false;
    existing.older_page_request_cooldown_until_tick = older_page_request_cooldown_until_tick;
    existing.sparse_update = false;
}

pub(super) fn merge_goal_run_dossier(
    existing: Option<GoalRunDossier>,
    incoming: Option<GoalRunDossier>,
    preserve_existing_when_missing: bool,
) -> Option<GoalRunDossier> {
    if !preserve_existing_when_missing {
        return incoming;
    }
    match (existing, incoming) {
        (None, dossier) | (dossier, None) => dossier,
        (Some(existing), Some(mut incoming)) => {
            if incoming.units.is_empty() {
                incoming.units = existing.units;
            }
            if incoming.projection_state.is_empty() {
                incoming.projection_state = existing.projection_state;
            }
            if incoming.latest_resume_decision.is_none() {
                incoming.latest_resume_decision = existing.latest_resume_decision;
            }
            if incoming.report.is_none() {
                incoming.report = existing.report;
            }
            if incoming.summary.is_none() {
                incoming.summary = existing.summary;
            }
            if incoming.projection_error.is_none() {
                incoming.projection_error = existing.projection_error;
            }
            Some(incoming)
        }
    }
}

