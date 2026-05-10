use serde_json::Value;

pub(crate) fn get_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub(crate) fn get_string_lossy(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(inner)) => inner.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}
