#[path = "tool_diff_parts/render_tool_edit_diff_to_summarize_array_value.rs"]
mod render_tool_edit_diff_to_summarize_array_value;

#[path = "tool_diff_parts/empty_key_to_wrap_preserving_whitespace.rs"]
mod empty_key_to_wrap_preserving_whitespace;

pub(crate) use render_tool_edit_diff_to_summarize_array_value::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn summarize_array_value_returns_empty_brackets_for_empty_array() {
        assert_eq!(summarize_array_value(&[]), "[]");
    }

    #[test]
    fn summarize_array_value_lists_primitives_inline() {
        let items = vec![
            Value::Number(1.into()),
            Value::Number(2.into()),
            Value::Bool(true),
        ];
        assert_eq!(summarize_array_value(&items), "[1, 2, true]");
    }

    #[test]
    fn summarize_array_value_truncates_after_five_primitives() {
        let items: Vec<Value> = (1..=8).map(|n| Value::Number(n.into())).collect();
        assert_eq!(summarize_array_value(&items), "[1, 2, 3, 4, 5, +3 more]");
    }

    #[test]
    fn summarize_array_value_falls_back_to_count_for_complex_items() {
        let items = vec![Value::Bool(true), json!([1, 2, 3])];
        assert_eq!(summarize_array_value(&items), "2 items");
    }

    #[test]
    fn summarize_array_value_falls_back_to_count_for_nested_objects() {
        let items = vec![json!({"k": 1}), json!({"k": 2})];
        assert_eq!(summarize_array_value(&items), "2 items");
    }
}
