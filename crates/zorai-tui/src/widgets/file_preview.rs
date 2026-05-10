#[path = "file_preview_parts/build_cached_lines_to_scrollbar_layout.rs"]
mod build_cached_lines_to_scrollbar_layout;
#[path = "file_preview_parts/scroll_offset_from_thumb_offset_to_file_preview_cache_key.rs"]
mod scroll_offset_from_thumb_offset_to_file_preview_cache_key;
#[path = "file_preview_parts/syntax_highlighting.rs"]
mod syntax_highlighting;

pub(crate) use build_cached_lines_to_scrollbar_layout::*;
pub(crate) use scroll_offset_from_thumb_offset_to_file_preview_cache_key::*;

#[cfg(test)]
#[path = "file_preview_tests_parts"]
mod tests {
    use super::*;

    mod cached_snapshot_reuses_built_lines_for_same_preview_input_to_git_diff;
}
