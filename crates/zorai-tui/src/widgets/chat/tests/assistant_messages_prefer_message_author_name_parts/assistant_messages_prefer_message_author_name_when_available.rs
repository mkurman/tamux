use super::super::*;
use crate::state::chat::{AgentMessage, AgentThread, ChatAction, ChatState, MessageRole};
use crate::theme::ThemeTokens;
#[test]
fn assistant_messages_prefer_message_author_name_when_available() {
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    chat.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        agent_name: Some("Svarog".into()),
        title: "Test".into(),
        messages: vec![AgentMessage {
            role: MessageRole::Assistant,
            content: "Active responder reply".into(),
            author_agent_id: Some("swarozyc".into()),
            author_agent_name: Some("Swarozyc".into()),
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
            .any(|line| line.contains("Responder: Swarozyc")),
        "expected responder label from message author, got: {message_lines:?}"
    );
}

#[test]
fn participant_authored_messages_render_with_at_prefixed_responder_label() {
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    chat.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        agent_name: Some("Swarozyc".into()),
        title: "Test".into(),
        thread_participants: vec![crate::state::chat::ThreadParticipantState {
            agent_id: "weles".into(),
            agent_name: "Weles".into(),
            instruction: "verify claims".into(),
            status: "active".into(),
            created_at: 1,
            updated_at: 1,
            deactivated_at: None,
            last_contribution_at: None,
            always_auto_response: false,
        }],
        messages: vec![AgentMessage {
            role: MessageRole::Assistant,
            content: "Participant note".into(),
            author_agent_id: Some("weles".into()),
            author_agent_name: Some("Weles".into()),
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
            .any(|line| line.contains("Responder: @Weles")),
        "expected participant responder label, got: {message_lines:?}"
    );
}

#[test]
fn different_responders_get_distinct_label_colors() {
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "t1".into(),
        title: "Test".into(),
    });
    chat.reduce(ChatAction::ThreadDetailReceived(AgentThread {
        id: "t1".into(),
        agent_name: Some("Swarozyc".into()),
        title: "Test".into(),
        thread_participants: vec![crate::state::chat::ThreadParticipantState {
            agent_id: "weles".into(),
            agent_name: "Weles".into(),
            instruction: "verify claims".into(),
            status: "active".into(),
            created_at: 1,
            updated_at: 1,
            deactivated_at: None,
            last_contribution_at: None,
            always_auto_response: false,
        }],
        messages: vec![
            AgentMessage {
                role: MessageRole::Assistant,
                content: "Main reply".into(),
                author_agent_id: Some("swarozyc".into()),
                author_agent_name: Some("Swarozyc".into()),
                ..Default::default()
            },
            AgentMessage {
                role: MessageRole::Assistant,
                content: "Participant note".into(),
                author_agent_id: Some("weles".into()),
                author_agent_name: Some("Weles".into()),
                ..Default::default()
            },
        ],
        ..Default::default()
    }));

    let (lines, _) = build_rendered_lines(&chat, &ThemeTokens::default(), 80, 0, false);
    let responder_lines: Vec<_> = lines
        .iter()
        .filter(|line| {
            matches!(line.message_index, Some(0) | Some(1))
                && rendered_line_plain_text(line).contains("Responder: ")
        })
        .collect();
    assert_eq!(responder_lines.len(), 2, "expected two responder lines");
    let first_message_first_line = lines
        .iter()
        .find(|line| {
            line.message_index == Some(0) && !matches!(line.kind, RenderedLineKind::Padding)
        })
        .expect("first message should render a non-padding line");
    assert!(
        rendered_line_plain_text(first_message_first_line).contains("Responder: Swarozyc"),
        "assistant responder badge should render before the message body, got: {:?}",
        rendered_line_plain_text(first_message_first_line)
    );

    let main_label_style = responder_lines[0]
        .line
        .spans
        .iter()
        .find(|span| span.content.contains("Swarozyc"))
        .expect("main responder label span")
        .style;
    let participant_label_style = responder_lines[1]
        .line
        .spans
        .iter()
        .find(|span| span.content.contains("@Weles"))
        .expect("participant responder label span")
        .style;
    let theme = ThemeTokens::default();
    assert_ne!(
        main_label_style.fg, participant_label_style.fg,
        "different responders should render with distinct label colors"
    );
    assert_eq!(
        main_label_style.fg, theme.accent_assistant.fg,
        "main responder label should use the assistant violet accent"
    );
    assert_ne!(
        main_label_style.fg, theme.accent_success.fg,
        "main responder label should stay visually distinct from done/success green"
    );
}

#[test]
fn chat_scrollbar_geometry_reserves_right_gutter_when_transcript_overflows() {
    let chat = chat_with_messages(
        (0..12)
            .map(|idx| AgentMessage {
                role: MessageRole::Assistant,
                content: format!("message {idx}"),
                ..Default::default()
            })
            .collect(),
    );

    let layout = scrollbar_layout(
        Rect::new(0, 0, 40, 6),
        &chat,
        &ThemeTokens::default(),
        0,
        false,
    )
    .expect("overflowing transcript should allocate a scrollbar");

    assert_eq!(layout.content.width, 39);
    assert_eq!(layout.scrollbar.x, 39);
    assert_eq!(layout.scrollbar.width, 1);
    assert!(layout.thumb.height >= 1);
}

#[test]
fn chat_scrollbar_geometry_omits_gutter_when_content_fits() {
    let chat = chat_with_messages(vec![AgentMessage {
        role: MessageRole::Assistant,
        content: "short".into(),
        ..Default::default()
    }]);

    assert!(
        scrollbar_layout(
            Rect::new(0, 0, 40, 6),
            &chat,
            &ThemeTokens::default(),
            0,
            false
        )
        .is_none(),
        "short transcripts should not render a scrollbar gutter"
    );
}

#[test]
fn assistant_messages_ignore_non_system_handoff_markers() {
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
                content:
                    "[[handoff_event]]{\"from_agent_name\":\"Svarog\",\"to_agent_name\":\"Weles\"}"
                        .into(),
                ..Default::default()
            },
            AgentMessage {
                role: MessageRole::Assistant,
                content: "Still the main responder".into(),
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
    let second_message_lines: Vec<String> = lines
        .iter()
        .filter(|line| line.message_index == Some(1))
        .map(rendered_line_plain_text)
        .collect();

    assert!(
        first_message_lines
            .iter()
            .any(|line| line.contains("Responder: Svarog")),
        "expected default responder label, got: {first_message_lines:?}"
    );
    assert!(
            second_message_lines
                .iter()
                .any(|line| line.contains("Responder: Svarog")),
            "non-system handoff markers should not relabel later assistant messages: {second_message_lines:?}"
        );
}
