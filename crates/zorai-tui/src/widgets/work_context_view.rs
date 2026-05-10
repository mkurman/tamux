#[path = "work_context_view_parts/scrollbar_layout_from_metrics_to_selection_point_from_snapshot.rs"]
mod scrollbar_layout_from_metrics_to_selection_point_from_snapshot;
#[path = "work_context_view_selection.rs"]
mod selection;
#[path = "work_context_view_parts/selection_points_from_mouse_to_terminal_image_overlay_spec.rs"]
mod selection_points_from_mouse_to_terminal_image_overlay_spec;
#[path = "work_context_view_parts/work_context_view.rs"]
mod work_context_view;

pub(crate) use scrollbar_layout_from_metrics_to_selection_point_from_snapshot::*;
pub(crate) use selection_points_from_mouse_to_terminal_image_overlay_spec::*;

#[cfg(test)]
#[path = "tests/work_context_view.rs"]
mod tests;
