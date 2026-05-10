#[path = "approval_center_parts/render_to_header_hit_test.rs"]
mod render_to_header_hit_test;

#[path = "approval_center_parts/queue_hit_test_to_rule_detail_hit_test.rs"]
mod queue_hit_test_to_rule_detail_hit_test;

pub use render_to_header_hit_test::*;
#[cfg(test)]
#[path = "approval_center_tests_parts"]
mod tests {
    

    mod approval_center_renders_without_panicking_to_approval_center_wraps_long;
}
