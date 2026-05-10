#[path = "thread_picker_parts/from_tasks_to_is_weles_thread.rs"]
mod from_tasks_to_is_weles_thread;
#[path = "thread_picker_parts/hit_test_for_workspace_to_now_millis.rs"]
mod hit_test_for_workspace_to_now_millis;
#[path = "thread_picker_parts/is_svarog_agent_name_to_hit_test.rs"]
mod is_svarog_agent_name_to_hit_test;

pub(crate) use from_tasks_to_is_weles_thread::*;
pub(crate) use hit_test_for_workspace_to_now_millis::*;
pub(crate) use is_svarog_agent_name_to_hit_test::*;

#[cfg(test)]
#[path = "thread_picker_tests_parts"]
mod tests {
    

    mod filtered_threads_swarog_tab_excludes_unattributed_threads_to_thread;
    mod format_time_ago_zero_returns_empty_to_filtered_threads_swarog_tab;
}
