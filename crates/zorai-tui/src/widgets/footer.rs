#[path = "footer_parts/to_core_color_to_render_status_bar.rs"]
mod to_core_color_to_render_status_bar;

#[path = "footer_parts/status_bar_hit_test.rs"]
mod status_bar_hit_test;

pub use status_bar_hit_test::*;
pub use to_core_color_to_render_status_bar::*;

#[cfg(test)]
#[path = "footer_tests_parts"]
mod tests {
    

    mod footer_handles_empty_state_to_status_bar_shows_playing_indicator;
}
