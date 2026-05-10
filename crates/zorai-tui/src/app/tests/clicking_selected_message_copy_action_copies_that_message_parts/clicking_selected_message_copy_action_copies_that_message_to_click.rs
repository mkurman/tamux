use super::*;
use super::super::{build_model, unbounded_channel};
use std::sync::mpsc;
#[test]
fn clicking_selected_message_copy_action_copies_that_message() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::User,
            content: "first".to_string(),
            ..Default::default()
        },
    });
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "second".to_string(),
            ..Default::default()
        },
    });
    model.chat.select_message(Some(1));

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let copy_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::CopyMessage(1))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("selected message should expose a clickable copy action");

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: copy_pos.x,
        row: copy_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: copy_pos.x,
        row: copy_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        super::conversion::last_copied_text().as_deref(),
        Some("second")
    );
}

#[test]
fn clicking_selected_message_action_with_small_pointer_shift_still_copies_message() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "second".to_string(),
            ..Default::default()
        },
    });
    model.chat.select_message(Some(0));

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let copy_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::CopyMessage(0))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("selected message should expose a clickable copy action");

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: copy_pos.x,
        row: copy_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: copy_pos.x.saturating_add(1),
        row: copy_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        super::conversion::last_copied_text().as_deref(),
        Some("second")
    );
}

#[test]
fn clicking_reasoning_action_with_small_pointer_shift_still_toggles_reasoning() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "Answer".to_string(),
            reasoning: Some("Think".to_string()),
            ..Default::default()
        },
    });
    model.chat.select_message(Some(0));

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let expand_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::ReasoningToggle(0))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("selected reasoning message should expose a clickable expand action");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: expand_pos.x,
        row: expand_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: expand_pos.x.saturating_add(1),
        row: expand_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert!(
        model.chat.expanded_reasoning().contains(&0),
        "reasoning should toggle even when the pointer shifts within the action button"
    );
}

#[test]
fn clicking_tool_action_with_small_pointer_shift_still_toggles_tool_details() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Tool,
            content: "result".to_string(),
            tool_name: Some("read_file".to_string()),
            tool_status: Some("done".to_string()),
            ..Default::default()
        },
    });
    model.chat.select_message(Some(0));

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let expand_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::ToolToggle(0))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("selected tool message should expose a clickable expand action");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: expand_pos.x,
        row: expand_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: expand_pos.x.saturating_add(1),
        row: expand_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert!(
        model.chat.expanded_tools().contains(&0),
        "tool details should toggle even when the pointer shifts within the action button"
    );
}

#[test]
fn pressing_enter_executes_selected_inline_message_action() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "second".to_string(),
            ..Default::default()
        },
    });
    model.chat.select_message(Some(0));
    model.chat.select_message_action(0);
    super::conversion::reset_last_copied_text();

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(
        super::conversion::last_copied_text().as_deref(),
        Some("second")
    );
}

#[test]
fn operator_question_event_uses_keyboard_submission_from_inline_actions() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_client_event(ClientEvent::OperatorQuestion {
        question_id: "oq-1".to_string(),
        content: "Approve this slice?\nA - proceed\nB - revise".to_string(),
        options: vec!["A".to_string(), "B".to_string()],
        session_id: None,
        thread_id: Some("thread-1".to_string()),
    });
    assert_eq!(model.modal.top(), None);
    let thread = model.chat.active_thread().expect("thread should exist");
    let message = thread
        .messages
        .last()
        .expect("question message should exist");
    assert!(message.is_operator_question);
    assert_eq!(message.operator_question_id.as_deref(), Some("oq-1"));
    assert_eq!(message.actions[0].label, "A");
    assert_eq!(message.actions[1].label, "B");
    model.chat.select_message(Some(0));

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    let sent = cmd_rx
        .try_recv()
        .expect("pressing Enter should answer the operator question");
    match sent {
        DaemonCommand::AnswerOperatorQuestion {
            question_id,
            answer,
        } => {
            assert_eq!(question_id, "oq-1");
            assert_eq!(answer, "A");
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn click_without_drag_uses_press_location_for_message_selection() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::User,
            content: "first".to_string(),
            ..Default::default()
        },
    });
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "second".to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let message1_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::Message(1))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("second message should expose a clickable body position");
    let message0_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::Message(0))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("first message should expose a clickable body position");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: message1_pos.x,
        row: message1_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: message0_pos.x,
        row: message0_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.chat.selected_message(), Some(1));
}
