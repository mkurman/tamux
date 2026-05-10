#[path = "sidebar/spawned_agents.rs"]
mod spawned_agents;
#[path = "sidebar/tab_layout.rs"]
mod tab_layout;

#[path = "sidebar_parts/filtered_file_index_to_render.rs"]
mod filtered_file_index_to_render;
#[path = "sidebar_parts/render_cached_to_spawned_sidebar_flatten_call_count.rs"]
mod render_cached_to_spawned_sidebar_flatten_call_count;
#[path = "sidebar_parts/show_spawned_to_selected_file_path.rs"]
mod show_spawned_to_selected_file_path;
#[path = "sidebar_parts/sidebar.rs"]
mod sidebar;

pub(crate) use filtered_file_index_to_render::*;
pub(crate) use render_cached_to_spawned_sidebar_flatten_call_count::*;
pub(crate) use show_spawned_to_selected_file_path::*;

#[cfg(test)]
pub(crate) use tab_layout::{tab_cells, tab_hit_test};

#[cfg(test)]
#[path = "tests/sidebar.rs"]
mod tests;
