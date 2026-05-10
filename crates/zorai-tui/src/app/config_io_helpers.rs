use super::*;

pub(crate) fn normalize_provider_auth_source(provider_id: &str, auth_source: &str) -> String {
    if providers::supported_auth_sources_for(provider_id).contains(&auth_source) {
        auth_source.to_string()
    } else {
        providers::default_auth_source_for(provider_id).to_string()
    }
}

pub(crate) fn normalize_provider_transport(provider_id: &str, api_transport: &str) -> String {
    if providers::supported_transports_for(provider_id).contains(&api_transport) {
        api_transport.to_string()
    } else {
        providers::default_transport_for(provider_id).to_string()
    }
}

pub(crate) fn normalize_compliance_mode(mode: &str) -> String {
    match mode {
        "standard" | "soc2" | "hipaa" | "fedramp" => mode.to_string(),
        _ => "standard".to_string(),
    }
}

pub(super) fn split_openrouter_provider_list(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    for item in value.split(',') {
        let trimmed = item.trim();
        if trimmed.is_empty() || out.iter().any(|existing| existing == trimmed) {
            continue;
        }
        out.push(trimmed.to_string());
    }
    out
}

pub(super) fn openrouter_provider_list_value(
    provider_value: &serde_json::Value,
    key: &str,
) -> String {
    provider_value
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .or_else(|| {
            provider_value
                .get(key)
                .and_then(|value| value.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default()
}

fn escape_pointer_segment(segment: &str) -> String {
    segment.replace('~', "~0").replace('/', "~1")
}

pub(super) fn flatten_config_value(
    value: &serde_json::Value,
    pointer: &str,
    items: &mut Vec<(String, serde_json::Value)>,
) {
    match value {
        serde_json::Value::Object(map) if !map.is_empty() => {
            for (key, child) in map {
                let next = format!("{}/{}", pointer, escape_pointer_segment(key));
                flatten_config_value(child, &next, items);
            }
        }
        other => items.push((pointer.to_string(), other.clone())),
    }
}

#[path = "config_io_helpers_parts/provider_field_str_to_refresh_snapshot_stats.rs"]
mod provider_field_str_to_refresh_snapshot_stats;

#[path = "config_io_helpers_parts/build_config_patch_value.rs"]
mod build_config_patch_value;
