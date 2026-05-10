use super::super::*;
use crate::state::chat::{AgentMessage, AgentThread, ChatAction, ChatState, MessageRole};
use crate::theme::ThemeTokens;
#[test]
fn compaction_artifact_lines_use_standard_message_left_padding() {
    let mut chat = chat_with_messages(vec![AgentMessage {
        role: MessageRole::Assistant,
        content: "Compacted summary line".into(),
        message_kind: "compaction_artifact".into(),
        ..Default::default()
    }]);
    chat.toggle_reasoning(0);

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 40, 0, false);
    let content_line = lines
        .iter()
        .find(|line| {
            line.message_index == Some(0)
                && matches!(line.kind, RenderedLineKind::ReasoningContent)
                && rendered_line_plain_text(line).contains("Compacted summary line")
        })
        .expect("compaction artifact content line should render");
    let plain = rendered_line_plain_text(content_line);

    assert!(
        plain.starts_with(&" ".repeat(MESSAGE_PADDING_X)),
        "compaction artifact content should be padded like regular messages, got: {:?}",
        plain
    );
}

#[test]
fn compaction_artifact_visible_header_renders_without_expand_toggle() {
    // Why this matters: the user complained "I don't see compaction info in the
    // chat anymore". The artifact previously collapsed to a single label line,
    // hiding the trigger/strategy summary unless the user manually expanded it.
    // The visible header (msg.content) must render as a banner by default so
    // compaction events are obvious as you scroll back through history.
    let chat = chat_with_messages(vec![AgentMessage {
        role: MessageRole::Assistant,
        content:
            "Pre-compaction context: ~92,000 / 200,000 tokens (threshold 160,000)\nTrigger: token-threshold\nStrategy: rule based"
                .into(),
        message_kind: "compaction_artifact".into(),
        ..Default::default()
    }]);

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let plain_lines = lines
        .iter()
        .map(rendered_line_plain_text)
        .collect::<Vec<_>>();

    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Pre-compaction context")),
        "compaction trigger summary must be visible without expand toggle: {plain_lines:?}"
    );
    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Trigger: token-threshold")),
        "compaction trigger must be visible without expand toggle: {plain_lines:?}"
    );
    assert!(
        plain_lines.iter().any(|line| line.contains("Auto compaction")),
        "automatic compaction must be labelled as 'Auto compaction': {plain_lines:?}"
    );
}

#[test]
fn manual_compaction_artifact_uses_manual_label() {
    // Why this matters: distinguishing /compact from automatic compactions
    // helps the user attribute context loss to their own action vs. a
    // background trigger when reviewing thread history.
    let chat = chat_with_messages(vec![AgentMessage {
        role: MessageRole::Assistant,
        content:
            "Pre-compaction context: ~92,000 / 200,000 tokens (threshold 160,000)\nTrigger: manual-request\nStrategy: rule based"
                .into(),
        message_kind: "compaction_artifact".into(),
        ..Default::default()
    }]);

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let plain_lines = lines
        .iter()
        .map(rendered_line_plain_text)
        .collect::<Vec<_>>();

    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Manual compaction")),
        "manual compaction must be labelled as 'Manual compaction', not 'Auto compaction': {plain_lines:?}"
    );
    assert!(
        plain_lines
            .iter()
            .all(|line| !line.contains("Auto compaction")),
        "manual compaction must not also surface the auto label: {plain_lines:?}"
    );
}

#[test]
fn compaction_artifact_renders_trigger_summary_above_payload_preview() {
    let mut chat = chat_with_messages(vec![AgentMessage {
        role: MessageRole::Assistant,
        content:
            "Pre-compaction context: ~182,400 / 200,000 tokens (threshold 160,000)\nTrigger: message-count\nStrategy: rule based\n\nContent:\n# Compact summary\n- preserved goals"
                .into(),
        message_kind: "compaction_artifact".into(),
        ..Default::default()
    }]);
    chat.toggle_reasoning(0);

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let plain_lines = lines
        .iter()
        .map(rendered_line_plain_text)
        .collect::<Vec<_>>();

    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Pre-compaction context: ~182,400 / 200,000 tokens")),
        "expected compaction trigger summary in rendered lines: {plain_lines:?}"
    );
    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Trigger: message-count")),
        "expected compaction trigger line in rendered lines: {plain_lines:?}"
    );
    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Strategy: rule based")),
        "expected compaction strategy line in rendered lines: {plain_lines:?}"
    );
    assert!(
        plain_lines.iter().any(|line| line.contains("Content:")),
        "expected compaction content heading in rendered lines: {plain_lines:?}"
    );
    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Compact summary") || line.contains("preserved goals")),
        "expected compaction payload preview in rendered lines: {plain_lines:?}"
    );
}

#[test]
fn compaction_artifact_uses_hidden_payload_when_visible_content_is_header_only() {
    let mut chat = chat_with_messages(vec![AgentMessage {
        role: MessageRole::Assistant,
        content:
            "Pre-compaction context: ~542,139 / 400,000 tokens (threshold 320,000)\nTrigger: token-threshold\nStrategy: custom model generated"
                .into(),
        compaction_payload: Some(
            "# 🤖 Agent Context: State Checkpoint\n\n## 🎯 Primary Objective\n> Preserve the coding task and next step."
                .into(),
        ),
        message_kind: "compaction_artifact".into(),
        ..Default::default()
    }]);
    chat.toggle_reasoning(0);

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let plain_lines = lines
        .iter()
        .map(rendered_line_plain_text)
        .collect::<Vec<_>>();

    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Strategy: custom model generated")),
        "expected compaction strategy line in rendered lines: {plain_lines:?}"
    );
    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Agent Context: State Checkpoint")),
        "expected hidden compaction payload heading in rendered lines: {plain_lines:?}"
    );
    assert!(
        plain_lines
            .iter()
            .any(|line| line.contains("Preserve the coding task and next step.")),
        "expected hidden compaction payload body in rendered lines: {plain_lines:?}"
    );
}

#[test]
fn long_retry_status_message_wraps_across_multiple_lines() {
    let mut chat = chat_with_messages(vec![AgentMessage {
        role: MessageRole::Assistant,
        content: "Earlier response".into(),
        ..Default::default()
    }]);
    chat.reduce(ChatAction::SetRetryStatus {
            thread_id: "t1".into(),
            phase: RetryPhase::Waiting,
            attempt: 1,
            max_retries: 3,
            delay_ms: 10_000,
            failure_class: "network_timeout".into(),
            message: "Connection to the provider timed out after the upstream gateway stopped responding during the retry window.".into(),
            received_at_tick: 0,
        });

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 36, 0, false);
    let retry_lines: Vec<String> = lines
        .iter()
        .filter(|line| matches!(line.kind, RenderedLineKind::RetryStatus))
        .map(rendered_line_plain_text)
        .collect();

    let wrapped_error_lines: Vec<&String> = retry_lines
        .iter()
        .filter(|line| {
            line.contains("Connection")
                || line.contains("upstream")
                || line.contains("retry window")
        })
        .collect();

    assert!(
        wrapped_error_lines.len() >= 2,
        "retry error message should wrap across multiple visible lines, got: {:?}",
        retry_lines
    );
}
#[test]
fn hit_test_selects_last_visible_message_instead_of_previous_padding_block() {
    let chat = chat_with_messages(vec![
        AgentMessage {
            role: MessageRole::Assistant,
            content: "first".into(),
            ..Default::default()
        },
        AgentMessage {
            role: MessageRole::Tool,
            tool_name: Some("read_file".into()),
            tool_status: Some("done".into()),
            content: "tool output".into(),
            ..Default::default()
        },
        AgentMessage {
            role: MessageRole::User,
            content: "continue, also write up your ideas into files".into(),
            ..Default::default()
        },
    ]);

    let area = Rect::new(0, 0, 80, 10);
    let (inner, visible) = visible_rendered_lines(area, &chat, &ThemeTokens::default(), 0, false)
        .expect("chat should produce visible lines");
    let last_message_row = visible
        .iter()
        .rposition(|line| {
            line.message_index == Some(2) && matches!(line.kind, RenderedLineKind::MessageBody)
        })
        .expect("last message should be visible");
    let hit_line = &visible[last_message_row];
    let (_, content_start, _) = rendered_line_content_bounds(hit_line);

    let hit = hit_test(
        area,
        &chat,
        &ThemeTokens::default(),
        0,
        Position::new(
            inner.x + content_start as u16 + 1,
            inner.y + last_message_row as u16,
        ),
    );

    assert_eq!(hit, Some(ChatHitTarget::Message(2)));
}

#[test]
fn selected_message_action_bar_stays_visible_at_bottom_edge() {
    let mut chat = chat_with_messages(vec![
        AgentMessage {
            role: MessageRole::Assistant,
            content: "older".into(),
            ..Default::default()
        },
        AgentMessage {
            role: MessageRole::User,
            content: "latest".into(),
            ..Default::default()
        },
    ]);
    chat.select_message(Some(1));

    let area = Rect::new(0, 0, 80, 4);
    let (_, visible) = visible_rendered_lines(area, &chat, &ThemeTokens::default(), 0, false)
        .expect("chat should produce visible lines");

    assert!(
            visible.iter().any(|line| {
                matches!(line.kind, RenderedLineKind::ActionBar) && line.message_index == Some(1)
            }),
            "selected message should keep its action row visible even when it is the last visible message"
        );
}

#[test]
fn selecting_message_does_not_shift_visible_window() {
    let mut chat = chat_with_messages(
        (0..8)
            .map(|idx| AgentMessage {
                role: MessageRole::Assistant,
                content: format!("message {idx}"),
                ..Default::default()
            })
            .collect(),
    );
    chat.reduce(ChatAction::ScrollChat(4));

    let area = Rect::new(0, 0, 80, 6);
    let (_, before_visible) =
        visible_rendered_lines(area, &chat, &ThemeTokens::default(), 0, false)
            .expect("chat should produce visible lines before selection");
    let before_last_visible_message = before_visible
        .iter()
        .filter_map(|line| line.message_index)
        .max()
        .expect("a message should be visible before selection");

    chat.select_message(Some(0));

    let (_, after_visible) = visible_rendered_lines(area, &chat, &ThemeTokens::default(), 0, false)
        .expect("chat should produce visible lines after selection");
    let after_last_visible_message = after_visible
        .iter()
        .filter_map(|line| line.message_index)
        .max()
        .expect("a message should be visible after selection");

    assert_eq!(
        after_last_visible_message, before_last_visible_message,
        "message selection should not auto-scroll the transcript window"
    );
}

#[test]
fn assistant_messages_show_responder_labels_across_thread_handoffs() {
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    chat.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        title: "Test".into(),
        messages: vec![
            AgentMessage {
                role: MessageRole::Assistant,
                content: "Main reply".into(),
                ..Default::default()
            },
            AgentMessage {
                role: MessageRole::System,
                content:
                    "[[handoff_event]]{\"from_agent_name\":\"Svarog\",\"to_agent_name\":\"Weles\"}"
                        .into(),
                ..Default::default()
            },
            AgentMessage {
                role: MessageRole::Assistant,
                content: "Governance reply".into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let first_message_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.message_index == Some(0))
        .map(rendered_line_plain_text)
        .collect();
    let handoff_message_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.message_index == Some(2))
        .map(rendered_line_plain_text)
        .collect();

    assert!(
        first_message_lines
            .iter()
            .any(|line| line.contains("Responder: Svarog")),
        "expected main responder label, got: {first_message_lines:?}"
    );
    assert!(
        handoff_message_lines
            .iter()
            .any(|line| line.contains("Responder: Weles")),
        "expected handoff responder label, got: {handoff_message_lines:?}"
    );
}

#[test]
fn assistant_messages_fall_back_to_thread_agent_name_when_handoff_marker_is_missing() {
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    chat.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        agent_name: Some("Weles".into()),
        title: "Test".into(),
        messages: vec![AgentMessage {
            role: MessageRole::Assistant,
            content: "Governance reply".into(),
            ..Default::default()
        }],
        ..Default::default()
    }));

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let message_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.message_index == Some(0))
        .map(rendered_line_plain_text)
        .collect();

    assert!(
        message_lines
            .iter()
            .any(|line| line.contains("Responder: Weles")),
        "expected responder fallback from thread agent_name, got: {message_lines:?}"
    );
}

#[test]
fn historical_handoff_from_agent_does_not_override_thread_responder_fallback() {
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    chat.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        agent_name: Some("Dola".into()),
        title: "Test".into(),
        messages: vec![
            AgentMessage {
                role: MessageRole::System,
                content: "[[handoff_event]]{\"from_agent_name\":\"Svarog\"}".into(),
                ..Default::default()
            },
            AgentMessage {
                role: MessageRole::Assistant,
                content: "Current responder reply".into(),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let message_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.message_index == Some(1))
        .map(rendered_line_plain_text)
        .collect();

    assert!(
        message_lines
            .iter()
            .any(|line| line.contains("Responder: Dola")),
        "explicit thread responder fallback should survive historical markers: {message_lines:?}"
    );
    assert!(
        !message_lines
            .iter()
            .any(|line| line.contains("Responder: Svarog")),
        "historical from_agent marker should not reseed the current responder: {message_lines:?}"
    );
}

#[test]
fn concierge_thread_messages_render_rarog_as_the_responder() {
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "concierge".into(),
        title: "Concierge".into(),
    });
    chat.reduce(ChatAction::AppendMessage {
        thread_id: "concierge".into(),
        message: AgentMessage {
            role: MessageRole::Assistant,
            content: "Welcome back".into(),
            ..Default::default()
        },
    });

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let message_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.message_index == Some(0))
        .map(rendered_line_plain_text)
        .collect();

    assert!(
        message_lines
            .iter()
            .any(|line| line.contains("Responder: Rarog")),
        "expected concierge responder label to use Rarog, got: {message_lines:?}"
    );
}
