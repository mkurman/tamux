use super::whatsapp_modal_esc_sends_stop_and_closes_to_clicking_rendered_settings::*;
use crate::app::*;
#[test]
fn clicking_footer_queue_indicator_opens_queued_prompts_modal() {
    let (mut model, _daemon_rx) = make_model();
    model.queued_prompts.push(QueuedPrompt::new("preview me"));

    let status_area = Rect::new(0, model.height.saturating_sub(1), model.width, 1);
    let queue_pos = (status_area.x..status_area.x.saturating_add(status_area.width))
        .find_map(|column| {
            let pos = Position::new(column, status_area.y);
            if widgets::footer::status_bar_hit_test(
                status_area,
                model.connected,
                model.last_error.is_some(),
                model.voice_recording,
                model.voice_player.is_some(),
                model.queued_prompts.len(),
                pos,
            ) == Some(widgets::footer::StatusBarHitTarget::QueuedPrompts)
            {
                Some(pos)
            } else {
                None
            }
        })
        .expect("queue indicator should be clickable");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: queue_pos.x,
        row: queue_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.modal.top(), Some(modal::ModalKind::QueuedPrompts));
}

#[test]
fn clicking_participant_summary_opens_thread_participants_modal() {
    let (mut model, _daemon_rx) = make_model();
    model.width = 120;
    model.height = 40;
    model.connected = true;
    model.agent_config_loaded = true;
    model.handle_client_event(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
        id: "thread-1".to_string(),
        title: "Participant Thread".to_string(),
        agent_name: Some("Svarog".to_string()),
        thread_participants: vec![crate::wire::ThreadParticipantState {
            agent_id: "weles".to_string(),
            agent_name: "Weles".to_string(),
            instruction: "verify claims".to_string(),
            status: "active".to_string(),
            created_at: 1,
            updated_at: 2,
            deactivated_at: None,
            last_contribution_at: Some(3),
            always_auto_response: false,
        }],
        ..Default::default()
    })));

    let chat_area = model.pane_layout().chat;
    let click = Position::new(chat_area.x.saturating_add(2), chat_area.y.saturating_add(1));

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        model.modal.top(),
        Some(modal::ModalKind::ThreadParticipants)
    );
}

#[test]
fn subagent_inline_edit_does_not_sync_main_config() {
    let (mut model, mut daemon_rx) = make_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.config.agent_config_raw = Some(serde_json::json!({}));

    let mut editor = crate::state::subagents::SubAgentEditorState::new(
        None,
        1,
        "openai".to_string(),
        "gpt-5.4".to_string(),
    );
    editor.name = "Draft".to_string();
    model.subagents.editor = Some(editor);
    model.settings.start_editing("subagent_name", "Draft");
    model.settings.reduce(SettingsAction::InsertChar('X'));

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );

    assert!(!quit);
    assert_eq!(
        model
            .subagents
            .editor
            .as_ref()
            .map(|editor| editor.name.as_str()),
        Some("DraftX")
    );
    assert!(
        daemon_rx.try_recv().is_err(),
        "sub-agent field edits should stay local until Save"
    );
}

#[test]
fn settings_textarea_ctrl_j_confirms_whatsapp_allowlist_edit() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .settings
        .reduce(SettingsAction::SwitchTab(SettingsTab::Gateway));
    model
        .settings
        .start_editing("whatsapp_allowed_contacts", "123123123123");
    model.settings.reduce(SettingsAction::InsertChar('\n'));
    model.settings.reduce(SettingsAction::InsertChar('4'));

    let quit = model.handle_key_modal(
        KeyCode::Char('j'),
        KeyModifiers::CONTROL,
        modal::ModalKind::Settings,
    );

    assert!(!quit);
    assert_eq!(model.settings.editing_field(), None);
    assert_eq!(model.config.whatsapp_allowed_contacts, "123123123123\n4");
}

#[test]
fn settings_textarea_ctrl_m_confirms_whatsapp_allowlist_edit() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .settings
        .reduce(SettingsAction::SwitchTab(SettingsTab::Gateway));
    model
        .settings
        .start_editing("whatsapp_allowed_contacts", "123123123123");
    model.settings.reduce(SettingsAction::InsertChar('\n'));
    model.settings.reduce(SettingsAction::InsertChar('5'));

    let quit = model.handle_key_modal(
        KeyCode::Char('m'),
        KeyModifiers::CONTROL,
        modal::ModalKind::Settings,
    );

    assert!(!quit);
    assert_eq!(model.settings.editing_field(), None);
    assert_eq!(model.config.whatsapp_allowed_contacts, "123123123123\n5");
}

#[test]
fn settings_textarea_ctrl_s_confirms_whatsapp_allowlist_edit() {
    let (mut model, _daemon_rx) = make_model();
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .settings
        .reduce(SettingsAction::SwitchTab(SettingsTab::Gateway));
    model
        .settings
        .start_editing("whatsapp_allowed_contacts", "123123123123");
    model.settings.reduce(SettingsAction::InsertChar('\n'));
    model.settings.reduce(SettingsAction::InsertChar('6'));

    let quit = model.handle_key_modal(
        KeyCode::Char('s'),
        KeyModifiers::CONTROL,
        modal::ModalKind::Settings,
    );

    assert!(!quit);
    assert_eq!(model.settings.editing_field(), None);
    assert_eq!(model.config.whatsapp_allowed_contacts, "123123123123\n6");
}

#[test]
fn subagent_system_prompt_textarea_supports_arrow_keys() {
    let (mut model, _daemon_rx) = make_model();
    let editor = crate::state::subagents::SubAgentEditorState::new(
        None,
        1,
        "openai".to_string(),
        "gpt-5.4".to_string(),
    );
    model.subagents.editor = Some(editor);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .settings
        .reduce(SettingsAction::SwitchTab(SettingsTab::SubAgents));
    model
        .settings
        .start_editing("subagent_system_prompt", "abc\ndef");

    let quit = model.handle_key_modal(
        KeyCode::Left,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    assert_eq!(model.settings.edit_cursor_line_col(), (1, 2));

    let quit = model.handle_key_modal(KeyCode::Up, KeyModifiers::NONE, modal::ModalKind::Settings);
    assert!(!quit);
    assert_eq!(model.settings.edit_cursor_line_col(), (0, 2));

    let quit = model.handle_key_modal(
        KeyCode::Right,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    assert_eq!(model.settings.edit_cursor_line_col(), (0, 3));

    let quit = model.handle_key_modal(
        KeyCode::Down,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    assert_eq!(model.settings.edit_cursor_line_col(), (1, 3));
}

#[test]
fn subagent_editor_navigation_wraps_between_first_and_last_fields() {
    let (mut model, _daemon_rx) = make_model();
    let editor = crate::state::subagents::SubAgentEditorState::new(
        None,
        1,
        "openai".to_string(),
        "gpt-5.4".to_string(),
    );
    model.subagents.editor = Some(editor);
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
    model
        .settings
        .reduce(SettingsAction::SwitchTab(SettingsTab::SubAgents));

    let quit = model.handle_key_modal(KeyCode::Up, KeyModifiers::NONE, modal::ModalKind::Settings);
    assert!(!quit);
    assert_eq!(
        model.subagents.editor.as_ref().map(|editor| editor.field),
        Some(crate::state::subagents::SubAgentEditorField::Cancel)
    );

    let quit = model.handle_key_modal(
        KeyCode::Down,
        KeyModifiers::NONE,
        modal::ModalKind::Settings,
    );
    assert!(!quit);
    assert_eq!(
        model.subagents.editor.as_ref().map(|editor| editor.field),
        Some(crate::state::subagents::SubAgentEditorField::Name)
    );
}

pub(super) fn sample_notification(read_at: Option<i64>) -> zorai_protocol::InboxNotification {
    zorai_protocol::InboxNotification {
        id: "n1".to_string(),
        source: "plugin_auth".to_string(),
        kind: "plugin_auth_warning".to_string(),
        title: "Refresh needed".to_string(),
        body: "Reconnect plugin auth before it expires.".to_string(),
        subtitle: Some("gmail".to_string()),
        severity: "warning".to_string(),
        created_at: 1,
        updated_at: 1,
        read_at,
        archived_at: None,
        deleted_at: None,
        actions: Vec::new(),
        metadata_json: None,
    }
}

fn drain_upsert_notifications(
    daemon_rx: &mut tokio::sync::mpsc::UnboundedReceiver<DaemonCommand>,
) -> Vec<zorai_protocol::InboxNotification> {
    let mut notifications = Vec::new();
    while let Ok(command) = daemon_rx.try_recv() {
        if let DaemonCommand::UpsertNotification(notification) = command {
            notifications.push(notification);
        }
    }
    notifications
}

#[test]
fn notifications_modal_shift_r_lowercase_marks_all_read() {
    let (mut model, mut daemon_rx) = make_model();
    let mut unread = sample_notification(None);
    unread.id = "n-unread".to_string();
    let mut still_read = sample_notification(Some(5));
    still_read.id = "n-read".to_string();
    model
        .notifications
        .reduce(crate::state::NotificationsAction::Replace(vec![
            unread, still_read,
        ]));
    model.toggle_notifications_modal();

    let handled = model.handle_key_modal(
        KeyCode::Char('r'),
        KeyModifiers::SHIFT,
        modal::ModalKind::Notifications,
    );

    assert!(!handled);
    assert_eq!(model.notifications.unread_count(), 0);
    let persisted = drain_upsert_notifications(&mut daemon_rx);
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].id, "n-unread");
    assert!(persisted[0].read_at.is_some());
    assert_eq!(persisted[0].archived_at, None);
}

#[test]
fn notifications_modal_shift_a_lowercase_archives_read() {
    let (mut model, mut daemon_rx) = make_model();
    let mut unread = sample_notification(None);
    unread.id = "n-unread".to_string();
    unread.updated_at = 10;
    let mut read = sample_notification(Some(5));
    read.id = "n-read".to_string();
    read.updated_at = 5;
    model
        .notifications
        .reduce(crate::state::NotificationsAction::Replace(vec![
            unread, read,
        ]));
    model.toggle_notifications_modal();

    let handled = model.handle_key_modal(
        KeyCode::Char('a'),
        KeyModifiers::SHIFT,
        modal::ModalKind::Notifications,
    );

    assert!(!handled);
    let active_ids = model
        .notifications
        .active_items()
        .into_iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(active_ids, vec!["n-unread".to_string()]);
    let persisted = drain_upsert_notifications(&mut daemon_rx);
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].id, "n-read");
    assert_eq!(persisted[0].read_at, Some(5));
    assert!(persisted[0].archived_at.is_some());
}

#[test]
fn notifications_modal_uses_wider_overlay_width() {
    let (mut model, _daemon_rx) = make_model();
    model.width = 100;
    model.height = 40;
    model.toggle_notifications_modal();

    let (_, overlay_area) = model
        .current_modal_area()
        .expect("notifications modal should be visible");

    assert_eq!(overlay_area.width, 78);
}

#[test]
fn notifications_modal_left_right_changes_header_focus_and_enter_uses_it() {
    let (mut model, _daemon_rx) = make_model();
    model
        .notifications
        .reduce(crate::state::NotificationsAction::Replace(vec![
            sample_notification(None),
        ]));
    model.toggle_notifications_modal();

    assert_eq!(
        model.notifications.selected_header_action(),
        Some(crate::state::NotificationsHeaderAction::MarkAllRead)
    );

    let quit = model.handle_key_modal(
        KeyCode::Right,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert_eq!(
        model.notifications.selected_header_action(),
        Some(crate::state::NotificationsHeaderAction::Close)
    );

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert!(model.modal.top().is_none());
}

#[test]
fn notifications_modal_down_clears_header_focus_and_enter_expands_row() {
    let (mut model, _daemon_rx) = make_model();
    model
        .notifications
        .reduce(crate::state::NotificationsAction::Replace(vec![
            sample_notification(None),
        ]));
    model.toggle_notifications_modal();

    let quit = model.handle_key_modal(
        KeyCode::Down,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert_eq!(model.notifications.selected_header_action(), None);

    let quit = model.handle_key_modal(
        KeyCode::Enter,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert_eq!(model.notifications.expanded_id(), Some("n1"));
}

#[test]
fn notifications_modal_tab_switches_between_header_and_row_actions() {
    let (mut model, _daemon_rx) = make_model();
    model
        .notifications
        .reduce(crate::state::NotificationsAction::Replace(vec![
            sample_notification(None),
        ]));
    model.toggle_notifications_modal();

    assert_eq!(
        model.notifications.selected_header_action(),
        Some(crate::state::NotificationsHeaderAction::MarkAllRead)
    );
    assert_eq!(model.notifications.selected_row_action_index(), None);

    let quit = model.handle_key_modal(
        KeyCode::Tab,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert_eq!(model.notifications.selected_header_action(), None);
    assert_eq!(model.notifications.selected_row_action_index(), Some(0));

    let quit = model.handle_key_modal(
        KeyCode::Tab,
        KeyModifiers::NONE,
        modal::ModalKind::Notifications,
    );
    assert!(!quit);
    assert_eq!(
        model.notifications.selected_header_action(),
        Some(crate::state::NotificationsHeaderAction::MarkAllRead)
    );
    assert_eq!(model.notifications.selected_row_action_index(), None);
}
