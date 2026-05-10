use super::*;
use crate::app::tests::goal_sidebar_tab_cycling_stays_to_collaboration_mouse_clicks_select_rows::submit_operator_profile_answer_allows_empty_input_when_question_mod::sample_collaboration_sessions;
use std::fs;
use crate::app::tests::goal_sidebar_tab_cycling_stays_to_collaboration_mouse_clicks_select_rows::goal_sidebar_tab_cycling_stays_mod::*;
use super::super::{rendered_chat_area, unbounded_channel};
use std::sync::mpsc;
#[test]
fn collaboration_mouse_clicks_select_rows_and_vote_actions() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Collaboration;
    model.focus = FocusArea::Chat;
    model
        .collaboration
        .reduce(crate::state::CollaborationAction::SessionsLoaded(
            sample_collaboration_sessions(),
        ));

    let chat_area = rendered_chat_area(&model);
    let left_x = chat_area.x + 3;
    let top_y = chat_area.y + 2;

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: left_x,
        row: top_y,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(model.collaboration.selected_row_index(), 0);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: left_x,
        row: top_y + 1,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(model.collaboration.selected_row_index(), 1);

    let right_x = chat_area.x + (chat_area.width / 2);
    let action_y = chat_area.y + 6;
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: right_x,
        row: action_y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx
        .try_recv()
        .expect("expected collaboration vote command from mouse action")
    {
        DaemonCommand::VoteOnCollaborationDisagreement { position, .. } => {
            assert_eq!(position, "roll forward");
        }
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn done_event_stores_provider_final_result_on_final_message() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, _cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.chat.reduce(chat::ChatAction::Delta {
        thread_id: "thread-1".to_string(),
        content: "Answer".to_string(),
    });

    model.handle_client_event(ClientEvent::Done {
        thread_id: "thread-1".to_string(),
        input_tokens: 10,
        output_tokens: 20,
        cost: None,
        provider: Some(zorai_shared::providers::PROVIDER_ID_GITHUB_COPILOT.to_string()),
        model: Some("gpt-5.4".to_string()),
        tps: None,
        generation_ms: None,
        reasoning: None,
        provider_final_result_json: Some(
            r#"{"provider":"open_ai_responses","id":"resp_tui_done"}"#.to_string(),
        ),
    });

    let thread = model.chat.active_thread().expect("thread should exist");
    let last = thread
        .messages
        .last()
        .expect("assistant message should exist");
    let json = last
        .provider_final_result_json
        .as_deref()
        .expect("provider final result should be stored");
    let value: serde_json::Value =
        serde_json::from_str(json).expect("parse provider final result json");
    assert_eq!(
        value.get("provider").and_then(|v| v.as_str()),
        Some("open_ai_responses")
    );
    assert_eq!(
        value.get("id").and_then(|v| v.as_str()),
        Some("resp_tui_done")
    );
}

#[test]
fn submit_prompt_appends_referenced_files_footer() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    let cwd = make_temp_dir();
    let source_dir = cwd.join("src");
    fs::create_dir_all(&source_dir).expect("source directory should be creatable");
    let main_rs = source_dir.join("main.rs");
    fs::write(&main_rs, "fn main() {}\n").expect("fixture file should be writable");

    model.submit_prompt(format!(
        "Please inspect @{} before editing",
        main_rs.display()
    ));

    let expected = format!(
        "Please inspect @{} before editing\n\nReferenced files: {}\nInspect these with read_file before making assumptions.",
        main_rs.display(),
        main_rs.display()
    );

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::SendMessage { content, .. }) => {
            assert_eq!(content, expected);
        }
        other => panic!("expected send-message command, got {:?}", other),
    }

    assert_eq!(
        model
            .chat
            .active_thread()
            .and_then(|thread| thread.messages.last())
            .map(|message| message.content.as_str()),
        Some(expected.as_str())
    );
}

#[test]
fn submit_prompt_does_not_inline_referenced_file_contents() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;

    let cwd = make_temp_dir();
    let notes = cwd.join("notes.txt");
    fs::write(&notes, "top secret contents\n").expect("fixture file should be writable");

    model.submit_prompt(format!("Review @{}", notes.display()));

    let sent_content = match cmd_rx.try_recv() {
        Ok(DaemonCommand::SendMessage { content, .. }) => content,
        other => panic!("expected send-message command, got {:?}", other),
    };

    assert!(
        sent_content.contains("Referenced files:"),
        "resolved references should append footer metadata"
    );
    assert!(
        sent_content.contains(&notes.display().to_string()),
        "footer should include the normalized absolute path"
    );
    assert!(
        !sent_content.contains("top secret contents"),
        "submit_prompt should not inline referenced file contents"
    );
    assert!(
        !sent_content.contains("<attached_file"),
        "file references should not create synthetic attachments"
    );
}

#[test]
fn submit_prompt_deduplicates_referenced_files() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;

    let cwd = make_temp_dir();
    let lib_rs = cwd.join("lib.rs");
    fs::write(&lib_rs, "pub fn demo() {}\n").expect("fixture file should be writable");

    model.submit_prompt(format!("Check @{0} and again @{0}", lib_rs.display()));

    let sent_content = match cmd_rx.try_recv() {
        Ok(DaemonCommand::SendMessage { content, .. }) => content,
        other => panic!("expected send-message command, got {:?}", other),
    };

    let footer = format!("Referenced files: {}", lib_rs.display());
    assert_eq!(sent_content.matches(&footer).count(), 1);
}

#[test]
fn submit_prompt_ignores_nonexistent_referenced_files() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.connected = true;
    model.concierge.auto_cleanup_on_navigate = false;

    let missing = make_temp_dir().join("missing.txt");
    let prompt = format!("Investigate @{} later", missing.display());

    model.submit_prompt(prompt.clone());

    let sent_content = match cmd_rx.try_recv() {
        Ok(DaemonCommand::SendMessage { content, .. }) => content,
        other => panic!("expected send-message command, got {:?}", other),
    };

    assert_eq!(sent_content, prompt);
    assert!(
        !sent_content.contains("Referenced files:"),
        "nonexistent references should remain plain text without footer metadata"
    );
}
