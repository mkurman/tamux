use super::super::{build_model, unbounded_channel};
use super::*;
use std::sync::mpsc;
#[test]
fn builtin_goal_commands_support_goal_picker_and_new_goal_composer() {
    let mut model = build_model();

    assert!(model.is_builtin_command("new-goal"));
    assert!(model.is_builtin_command("goal"));
    assert!(!model.is_builtin_command("goals"));

    model.execute_command("goal");

    assert_eq!(model.modal.top(), Some(modal::ModalKind::GoalPicker));

    model.execute_command("new-goal");

    assert!(matches!(model.main_pane_view, MainPaneView::GoalComposer));
}

#[test]
fn retry_wait_keyboard_can_trigger_from_input_focus_when_input_is_empty() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.focus = FocusArea::Input;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_retry_status_event(
        "thread-1".to_string(),
        "waiting".to_string(),
        1,
        0,
        30_000,
        "transport".to_string(),
        "upstream transport error".to_string(),
    );

    let handled = model.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert!(!handled);

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RetryStreamNow { thread_id }) => assert_eq!(thread_id, "thread-1"),
        other => panic!("expected retry-now command, got {:?}", other),
    }
}

#[test]
fn retry_wait_keyboard_can_trigger_from_input_focus_with_pending_text() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.focus = FocusArea::Input;
    model.input.reduce(input::InputAction::InsertChar('c'));
    model.input.reduce(input::InputAction::InsertChar('o'));
    model.input.reduce(input::InputAction::InsertChar('n'));
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_retry_status_event(
        "thread-1".to_string(),
        "waiting".to_string(),
        1,
        0,
        30_000,
        "transport".to_string(),
        "upstream transport error".to_string(),
    );

    let handled = model.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert!(!handled);

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RetryStreamNow { thread_id }) => assert_eq!(thread_id, "thread-1"),
        other => panic!("expected retry-now command, got {:?}", other),
    }
}

#[test]
fn retry_wait_mouse_click_triggers_immediate_retry() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.width = 100;
    model.height = 40;
    model.focus = FocusArea::Input;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_retry_status_event(
        "thread-1".to_string(),
        "waiting".to_string(),
        1,
        0,
        30_000,
        "transport".to_string(),
        "upstream transport error".to_string(),
    );

    let chat_area = model.pane_layout().chat;
    let retry_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::RetryStartNow)
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("retry action should expose a clickable yes target");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: retry_pos.x,
        row: retry_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: retry_pos.x,
        row: retry_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    assert!(
        model.chat.retry_status().is_none(),
        "retry prompt should clear locally after mouse retry-now"
    );

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RetryStreamNow { thread_id }) => assert_eq!(thread_id, "thread-1"),
        other => panic!("expected retry-now command, got {:?}", other),
    }
}

#[test]
fn retry_wait_mouse_down_triggers_immediate_retry() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.width = 100;
    model.height = 40;
    model.focus = FocusArea::Input;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.handle_retry_status_event(
        "thread-1".to_string(),
        "waiting".to_string(),
        1,
        0,
        30_000,
        "transport".to_string(),
        "upstream transport error".to_string(),
    );

    let chat_area = model.pane_layout().chat;
    let retry_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::RetryStartNow)
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("retry action should expose a clickable yes target");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: retry_pos.x,
        row: retry_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    assert!(
        model.chat.retry_status().is_none(),
        "retry prompt should clear locally after mouse-down retry-now"
    );

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RetryStreamNow { thread_id }) => assert_eq!(thread_id, "thread-1"),
        other => panic!("expected retry-now command on mouse down, got {:?}", other),
    }
}
