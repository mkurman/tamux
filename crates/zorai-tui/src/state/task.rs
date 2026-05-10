#![allow(dead_code)]

#[path = "task_parts/goal_step_todo_thread_ids_to_merge_usize_field.rs"]
mod goal_step_todo_thread_ids_to_merge_usize_field;
#[path = "task_parts/merge_goal_run_dossier.rs"]
mod merge_goal_run_dossier;
#[path = "task_parts/new_to_reduce.rs"]
mod new_to_reduce;
#[path = "task_parts/task_status_to_task_state.rs"]
mod task_status_to_task_state;

pub use task_status_to_task_state::*;

#[cfg(test)]
#[path = "tests/task.rs"]
mod tests;
