use super::super::*;
use super::markdown_renders_bold_to_apply_patch_tool_message_expanded_renders_diff::*;
use crate::state::chat::{AgentMessage, MessageRole, TranscriptMode};
use crate::theme::ThemeTokens;
#[test]
fn reasoning_renders_before_multiline_content() {
    let msg = AgentMessage {
        role: MessageRole::Assistant,
        content: "First line that wraps a bit for the test".into(),
        reasoning: Some("Let me think...".into()),
        ..Default::default()
    };
    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        20,
        &empty_expanded(),
        &empty_tools(),
    );
    let first_text: String = lines[0]
        .spans
        .iter()
        .map(|span| span.content.to_string())
        .collect();
    let second_text: String = lines[1]
        .spans
        .iter()
        .map(|span| span.content.to_string())
        .collect();
    assert!(
        first_text.contains("Reasoning"),
        "First line should be reasoning, got: {}",
        first_text
    );
    assert!(
        !second_text.contains("Reasoning"),
        "Content should start after reasoning, got: {}",
        second_text
    );
}

#[test]
fn reasoning_expandable() {
    let msg = AgentMessage {
        role: MessageRole::Assistant,
        content: "Answer".into(),
        reasoning: Some("Thinking step by step".into()),
        ..Default::default()
    };
    let collapsed = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    let mut exp = empty_expanded();
    exp.insert(0);
    let expanded = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &exp,
        &empty_tools(),
    );
    assert!(
        expanded.len() > collapsed.len(),
        "Expanded should have more lines"
    );
}

#[test]
fn meta_cognition_message_collapses_by_default() {
    let msg = AgentMessage {
        role: MessageRole::System,
        content: "Meta-cognitive intervention: warning before tool execution.\nPlanned tool: read_file\nDetected risks:\n- overconfidence".into(),
        ..Default::default()
    };

    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    let plain = plain_lines(&lines).join("\n");

    assert!(
        plain.contains("Meta-cognition"),
        "collapsed meta-cognition should show a disclosure header: {plain}"
    );
    assert!(
        !plain.contains("Planned tool: read_file") && !plain.contains("overconfidence"),
        "collapsed meta-cognition should hide details: {plain}"
    );
}

#[test]
fn meta_cognition_message_expands_with_reasoning_state() {
    let msg = AgentMessage {
        role: MessageRole::System,
        content: "Meta-cognitive intervention: warning before tool execution.\nPlanned tool: read_file\nDetected risks:\n- overconfidence".into(),
        ..Default::default()
    };
    let mut expanded = empty_expanded();
    expanded.insert(0);

    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &expanded,
        &empty_tools(),
    );
    let plain = plain_lines(&lines).join("\n");

    assert!(
        plain.contains("Planned tool: read_file") && plain.contains("overconfidence"),
        "expanded meta-cognition should show details: {plain}"
    );
}

#[test]
fn background_operation_finished_message_collapses_by_default_and_expands() {
    let msg = AgentMessage {
        role: MessageRole::System,
        content: "Background operation finished.\n\noperation_id: op-123\ntool: shell\nstate: succeeded\nregistered_at: 123\n\nOperation status:\n{\"state\":\"succeeded\"}".into(),
        ..Default::default()
    };

    let collapsed = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    let collapsed_plain = plain_lines(&collapsed).join("\n");

    assert!(
        collapsed_plain.contains("Background operation finished"),
        "collapsed background operation should show a disclosure header: {collapsed_plain}"
    );
    assert!(
        !collapsed_plain.contains("operation_id: op-123")
            && !collapsed_plain.contains("Operation status:"),
        "collapsed background operation should hide details: {collapsed_plain}"
    );

    let mut expanded = empty_expanded();
    expanded.insert(0);
    let expanded_lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &expanded,
        &empty_tools(),
    );
    let expanded_plain = plain_lines(&expanded_lines).join("\n");

    assert!(
        expanded_plain.contains("operation_id: op-123")
            && expanded_plain.contains("Operation status:"),
        "expanded background operation should show details: {expanded_plain}"
    );
}

#[test]
fn batched_background_operations_message_collapses_by_default_and_expands() {
    let msg = AgentMessage {
        role: MessageRole::System,
        content: "Background operations finished.\n\nOperation results:\n[{\"operation_id\":\"op-123\",\"state\":\"completed\"},{\"operation_id\":\"op-456\",\"state\":\"failed\"}]".into(),
        ..Default::default()
    };

    let collapsed = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    let collapsed_plain = plain_lines(&collapsed).join("\n");

    assert!(
        collapsed_plain.contains("Background operations finished"),
        "collapsed background operations should show a disclosure header: {collapsed_plain}"
    );
    assert!(
        !collapsed_plain.contains("op-123") && !collapsed_plain.contains("Operation results:"),
        "collapsed background operations should hide details: {collapsed_plain}"
    );

    let mut expanded = empty_expanded();
    expanded.insert(0);
    let expanded_lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &expanded,
        &empty_tools(),
    );
    let expanded_plain = plain_lines(&expanded_lines).join("\n");

    assert!(
        expanded_plain.contains("op-123") && expanded_plain.contains("op-456"),
        "expanded background operations should show batched details: {expanded_plain}"
    );
}

#[test]
fn compaction_artifact_shows_header_inline_and_hides_payload_until_expand() {
    // Why this matters: previously collapsed artifacts showed only a one-line
    // disclosure label, hiding the trigger/strategy summary unless the user
    // expanded — the user reported "I don't see compaction info in the chat
    // anymore". The visible header (msg.content) must always render so users
    // can see at a glance that a compaction happened and why; the bulkier
    // hidden payload still needs an explicit expand.
    let msg = AgentMessage {
        role: MessageRole::Assistant,
        content:
            "Pre-compaction context: ~182,400 / 200,000 tokens (threshold 160,000)\nTrigger: token-threshold\nStrategy: custom model generated"
                .into(),
        compaction_payload: Some("# Compact summary\n- preserved goals".into()),
        message_kind: "compaction_artifact".into(),
        ..Default::default()
    };

    let collapsed = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    let collapsed_plain = plain_lines(&collapsed).join("\n");

    assert!(
        collapsed_plain.contains("Auto compaction"),
        "collapsed compaction artifact should show a disclosure header: {collapsed_plain}"
    );
    assert!(
        collapsed_plain.contains("Pre-compaction context")
            && collapsed_plain.contains("Trigger: token-threshold")
            && collapsed_plain.contains("Strategy: custom model generated"),
        "collapsed compaction artifact must keep its trigger/strategy banner visible: {collapsed_plain}"
    );
    assert!(
        !collapsed_plain.contains("Compact summary")
            && !collapsed_plain.contains("preserved goals"),
        "collapsed compaction artifact should still hide the bulky payload until expand: {collapsed_plain}"
    );

    let mut expanded = empty_expanded();
    expanded.insert(0);
    let expanded_lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Compact,
        &ThemeTokens::default(),
        80,
        &expanded,
        &empty_tools(),
    );
    let expanded_plain = plain_lines(&expanded_lines).join("\n");

    assert!(
        expanded_plain.contains("Pre-compaction context")
            && expanded_plain.contains("Compact summary")
            && expanded_plain.contains("preserved goals"),
        "expanded compaction artifact should show header and payload details: {expanded_plain}"
    );
}

#[test]
fn tools_mode_skips_non_tool_messages() {
    let msg = AgentMessage {
        role: MessageRole::User,
        content: "Hello".into(),
        ..Default::default()
    };
    let lines = message_to_lines(
        &msg,
        0,
        TranscriptMode::Tools,
        &ThemeTokens::default(),
        80,
        &empty_expanded(),
        &empty_tools(),
    );
    assert!(lines.is_empty());
}

#[test]
fn wrap_text_empty_string() {
    let lines = wrap_text("", 80);
    assert_eq!(lines, vec![""]);
}

#[test]
fn wrap_text_zero_width() {
    let lines = wrap_text("hello", 0);
    assert_eq!(lines, vec!["hello"]);
}
