use super::scan::normalize_package_name;
use super::*;

pub(super) fn resolve_target_package<'a>(
    graph: &'a SemanticGraph,
    target: Option<&str>,
) -> Result<&'a SemanticPackage> {
    let Some(target) = target
        .map(normalize_package_name)
        .filter(|value| !value.is_empty())
    else {
        anyhow::bail!("semantic dependency queries require a non-empty `target` package name");
    };

    if let Some(exact) = graph.packages.iter().find(|package| package.name == target) {
        return Ok(exact);
    }

    let partial_matches = graph
        .packages
        .iter()
        .filter(|package| package.name.contains(&target))
        .collect::<Vec<_>>();
    match partial_matches.as_slice() {
        [single] => Ok(*single),
        [] => Err(anyhow::anyhow!("no local package matched `{target}`")),
        _ => Err(anyhow::anyhow!(
            "multiple packages matched `{target}`; be more specific"
        )),
    }
}

pub(super) fn resolve_target_service<'a>(
    graph: &'a SemanticGraph,
    target: Option<&str>,
) -> Result<&'a SemanticService> {
    let Some(target) = target
        .map(normalize_package_name)
        .filter(|value| !value.is_empty())
    else {
        anyhow::bail!("service queries require a non-empty `target` service name");
    };

    if let Some(exact) = graph.services.iter().find(|service| service.name == target) {
        return Ok(exact);
    }

    let partial_matches = graph
        .services
        .iter()
        .filter(|service| service.name.contains(&target))
        .collect::<Vec<_>>();
    match partial_matches.as_slice() {
        [single] => Ok(*single),
        [] => Err(anyhow::anyhow!("no compose service matched `{target}`")),
        _ => Err(anyhow::anyhow!(
            "multiple compose services matched `{target}`; be more specific"
        )),
    }
}

#[allow(dead_code)]
pub(super) fn convention_entry_matches(
    entry: &crate::history::MemoryProvenanceReportEntry,
    target_tokens: &[String],
) -> bool {
    if target_tokens.is_empty() {
        return matches!(entry.target.as_str(), "MEMORY.md" | "USER.md");
    }

    let haystack = normalize_convention_text(&format!(
        "{} {} {} {}",
        entry.target,
        entry.source_kind,
        entry.content,
        entry.fact_keys.join(" ")
    ));
    target_tokens.iter().all(|token| haystack.contains(token))
        || target_tokens
            .iter()
            .any(|token| entry.fact_keys.iter().any(|key| key.contains(token)))
}

pub(super) fn tokenize_convention_query(target: &str) -> Vec<String> {
    normalize_convention_text(target)
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn normalize_convention_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
}

pub(super) fn target_matches_text(value: &str, normalized_target: &str) -> bool {
    normalize_convention_text(value).contains(normalized_target)
}

pub(super) fn path_is_under_root(root: &Path, candidate: &str) -> bool {
    let candidate_path = Path::new(candidate);
    if candidate_path.is_absolute() {
        candidate_path.starts_with(root)
    } else {
        false
    }
}

pub(super) fn collect_matching_skills(
    skills_root: &Path,
    target: Option<&str>,
    limit: usize,
) -> Vec<String> {
    let mut matches = Vec::new();
    let target_tokens = target.map(tokenize_convention_query).unwrap_or_default();
    for entry in WalkDir::new(skills_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            != "md"
        {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(skills_root)
            .unwrap_or(entry.path())
            .display()
            .to_string();
        let haystack = normalize_convention_text(&relative);
        if !target_tokens.is_empty() && !target_tokens.iter().all(|token| haystack.contains(token))
        {
            continue;
        }
        matches.push(relative);
        if matches.len() >= limit {
            break;
        }
    }
    matches
}

pub(super) fn summarize_text(content: &str, max_chars: usize) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let char_count = normalized.chars().count();
    if char_count <= max_chars {
        return normalized;
    }
    let truncated = normalized.chars().take(max_chars).collect::<String>();
    format!("{truncated}...")
}
