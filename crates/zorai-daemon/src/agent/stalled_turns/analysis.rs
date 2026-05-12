use super::types::{StalledTurnClass, TurnEvidence};

const PROMISE_PREFIXES: &[&str] = &[
    "let me ",
    "i'll ",
    "i will ",
    "working",
    "give me a moment",
    "one moment",
    "excellent. let me ",
];

const PROMISE_PHRASES: &[&str] = &[
    "let me start",
    "let me draft",
    "let me produce",
    "let me do",
    "i'll draft",
    "i will draft",
    "i'll produce",
    "i will produce",
    "working. let me",
];

const COMPLETION_SIGNALS: &[&str] = &["here is", "completed", "complete.", "done.", "finished"];

pub(super) fn follow_through_observed(evidence: &TurnEvidence) -> bool {
    evidence.new_tool_call_followed
        || evidence.new_substantive_assistant_message_followed
        || evidence.task_or_goal_progressed
        || evidence.user_replied
}

pub(super) fn looks_like_promise_message(message: &str) -> bool {
    let normalized = normalize(message);
    if normalized.is_empty() {
        return false;
    }
    if COMPLETION_SIGNALS
        .iter()
        .any(|signal| normalized.contains(signal))
    {
        return false;
    }
    PROMISE_PREFIXES
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
        || PROMISE_PHRASES
            .iter()
            .any(|phrase| normalized.contains(phrase))
}

pub(super) fn classify_stalled_turn(evidence: &TurnEvidence) -> Option<StalledTurnClass> {
    if follow_through_observed(evidence) {
        return None;
    }

    if evidence.last_assistant_message.trim().is_empty() {
        return evidence
            .preceded_by_tool_result
            .then_some(StalledTurnClass::PostToolResultNoFollowThrough);
    }

    if !looks_like_promise_message(&evidence.last_assistant_message) {
        return None;
    }

    if evidence.preceded_by_tool_result {
        Some(StalledTurnClass::PostToolResultNoFollowThrough)
    } else {
        Some(StalledTurnClass::PromiseWithoutAction)
    }
}

fn normalize(message: &str) -> String {
    message
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
