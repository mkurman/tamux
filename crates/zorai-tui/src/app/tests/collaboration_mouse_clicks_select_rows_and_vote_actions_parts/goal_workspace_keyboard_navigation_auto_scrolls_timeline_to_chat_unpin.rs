use super::*;
use crate::app::tests::goal_sidebar_tab_cycling_stays_to_collaboration_mouse_clicks_select_rows::goal_sidebar_tab_cycling_stays_mod::*;
use super::super::{build_model, unbounded_channel};
use std::sync::mpsc;
#[test]
fn goal_workspace_keyboard_navigation_auto_scrolls_timeline() {
    let mut model = goal_sidebar_model();
    if let Some(run) = model.tasks.goal_run_by_id_mut("goal-1") {
        run.events = (0..40)
            .map(|idx| task::GoalRunEvent {
                id: format!("event-{idx}"),
                phase: "execution".to_string(),
                message: format!("event {idx}"),
                details: Some(format!("details {idx}")),
                step_index: Some(1),
                ..Default::default()
            })
            .collect();
    }
    model.tasks.reduce(task::TaskAction::WorkContextReceived(
        task::ThreadWorkContext {
            thread_id: "thread-1".to_string(),
            entries: (0..40)
                .map(|idx| task::WorkContextEntry {
                    path: format!("/tmp/detail-{idx}.md"),
                    goal_run_id: Some("goal-1".to_string()),
                    is_text: true,
                    ..Default::default()
                })
                .collect(),
        },
    ));
    model.focus = FocusArea::Chat;
    model
        .goal_workspace
        .set_focused_pane(goal_workspace::GoalWorkspacePane::Timeline);

    for _ in 0..20 {
        let handled = model.handle_key(KeyCode::Down, KeyModifiers::NONE);
        assert!(!handled);
    }

    assert!(model.goal_workspace.selected_timeline_row() >= 20);
    assert!(model.goal_workspace.timeline_scroll() > 0);
}

#[test]
fn goal_sidebar_blocks_hidden_pinned_shortcuts() {
    let mut model = goal_sidebar_model();
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-1".to_string(),
            title: "Goal Thread".to_string(),
            messages: vec![chat::AgentMessage {
                id: Some("message-1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Pinned content".to_string(),
                pinned_for_compaction: true,
                ..Default::default()
            }],
            loaded_message_end: 1,
            total_message_count: 1,
            ..Default::default()
        },
    ));
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Pinned));

    let handled = model.handle_key(KeyCode::Char('k'), KeyModifiers::CONTROL);
    assert!(!handled);

    let handled = model.handle_key(KeyCode::Char('u'), KeyModifiers::NONE);
    assert!(!handled);

    assert!(model.chat.active_thread_has_pinned_messages());
    assert_eq!(model.focus, FocusArea::Input);
    assert_eq!(model.goal_sidebar.active_tab(), GoalSidebarTab::Steps);
}

#[test]
fn sidebar_arrow_keys_follow_todos_first_tab_order() {
    let mut model = build_model();
    model.focus = FocusArea::Sidebar;

    assert_eq!(model.sidebar.active_tab(), SidebarTab::Todos);

    let handled = model.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.sidebar.active_tab(), SidebarTab::Todos);

    let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.sidebar.active_tab(), SidebarTab::Files);
}

#[test]
fn typing_in_files_sidebar_filters_entries_and_escape_clears_query() {
    let mut model = build_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.tasks.reduce(task::TaskAction::WorkContextReceived(
        task::ThreadWorkContext {
            thread_id: "thread-1".to_string(),
            entries: vec![
                task::WorkContextEntry {
                    path: "/tmp/readme.md".to_string(),
                    is_text: true,
                    ..Default::default()
                },
                task::WorkContextEntry {
                    path: "/tmp/runtime.rs".to_string(),
                    is_text: true,
                    ..Default::default()
                },
                task::WorkContextEntry {
                    path: "/tmp/schema.sql".to_string(),
                    is_text: true,
                    ..Default::default()
                },
            ],
        },
    ));
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Files));
    model.focus = FocusArea::Sidebar;

    for ch in "runtime".chars() {
        let handled = model.handle_key(KeyCode::Char(ch), KeyModifiers::NONE);
        assert!(!handled);
    }

    assert_eq!(model.focus, FocusArea::Sidebar);
    assert_eq!(model.sidebar_item_count(), 1);
    assert_eq!(model.sidebar.files_filter(), "runtime");

    let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Sidebar);
    assert_eq!(model.sidebar.files_filter(), "");
    assert_eq!(model.sidebar_item_count(), 3);

    for ch in "runtime".chars() {
        let handled = model.handle_key(KeyCode::Char(ch), KeyModifiers::NONE);
        assert!(!handled);
    }

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::WorkContext));
    assert_eq!(
        model.tasks.selected_work_path("thread-1"),
        Some("/tmp/runtime.rs")
    );
}

#[test]
fn mouse_wheel_over_sidebar_moves_file_selection() {
    let mut model = build_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.tasks.reduce(task::TaskAction::WorkContextReceived(
        task::ThreadWorkContext {
            thread_id: "thread-1".to_string(),
            entries: (0..12)
                .map(|idx| task::WorkContextEntry {
                    path: format!("/tmp/file-{idx}.rs"),
                    is_text: true,
                    ..Default::default()
                })
                .collect(),
        },
    ));
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Files));
    model.focus = FocusArea::Sidebar;

    let sidebar_area = model
        .pane_layout()
        .sidebar
        .expect("default layout should include a sidebar");
    let mouse = MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: sidebar_area.x.saturating_add(2),
        row: sidebar_area.y.saturating_add(3),
        modifiers: KeyModifiers::NONE,
    };

    model.handle_mouse(mouse);

    assert_eq!(model.focus, FocusArea::Sidebar);
    assert_eq!(model.sidebar.selected_item(), 3);

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.tasks.selected_work_path("thread-1"),
        Some("/tmp/file-3.rs")
    );
}

#[test]
fn deleting_message_keeps_sidebar_visible_when_thread_still_has_pins() {
    let mut model = build_model();
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-1".to_string(),
            title: "Pinned".to_string(),
            messages: vec![
                chat::AgentMessage {
                    id: Some("message-1".to_string()),
                    role: chat::MessageRole::Assistant,
                    content: "Pinned one".to_string(),
                    pinned_for_compaction: true,
                    ..Default::default()
                },
                chat::AgentMessage {
                    id: Some("message-2".to_string()),
                    role: chat::MessageRole::Assistant,
                    content: "Pinned two".to_string(),
                    pinned_for_compaction: true,
                    ..Default::default()
                },
            ],
            loaded_message_end: 2,
            total_message_count: 2,
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Pinned));

    assert!(
        model.pane_layout().sidebar.is_some(),
        "pinned threads should keep the sidebar visible"
    );

    model.delete_message(0);

    assert!(
        model.pane_layout().sidebar.is_some(),
        "remaining pins should keep the sidebar visible after deletion"
    );
    assert_eq!(model.sidebar.active_tab(), SidebarTab::Pinned);
}

#[test]
fn pinned_summary_only_thread_keeps_sidebar_visible() {
    let mut model = build_model();
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-1".to_string(),
            title: "Pinned".to_string(),
            messages: vec![chat::AgentMessage {
                id: Some("message-2".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Latest visible".to_string(),
                ..Default::default()
            }],
            pinned_messages: vec![chat::PinnedThreadMessage {
                message_id: "message-1".to_string(),
                absolute_index: 0,
                role: chat::MessageRole::User,
                content: "Pinned offscreen".to_string(),
            }],
            loaded_message_start: 1,
            loaded_message_end: 2,
            total_message_count: 2,
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    assert!(
        model.pane_layout().sidebar.is_some(),
        "summary pins should keep the sidebar visible even when the loaded page has no pinned rows"
    );
}

#[test]
fn ctrl_k_then_j_jumps_to_selected_pinned_message_from_input_focus() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, _cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-1".to_string(),
            title: "Pinned".to_string(),
            messages: vec![
                chat::AgentMessage {
                    id: Some("message-1".to_string()),
                    role: chat::MessageRole::User,
                    content: "Pinned user message".to_string(),
                    pinned_for_compaction: true,
                    ..Default::default()
                },
                chat::AgentMessage {
                    id: Some("message-2".to_string()),
                    role: chat::MessageRole::Assistant,
                    content: "Later reply".to_string(),
                    ..Default::default()
                },
            ],
            loaded_message_end: 2,
            total_message_count: 2,
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Pinned));
    model.focus = FocusArea::Input;
    model.input.set_text("draft");

    let handled = model.handle_key(KeyCode::Char('k'), KeyModifiers::CONTROL);
    assert!(!handled);

    let handled = model.handle_key(KeyCode::Char('j'), KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.chat.selected_message(), Some(0));
    assert_eq!(model.input.buffer(), "draft");
}

#[test]
fn ctrl_k_then_u_unpins_selected_pinned_message_from_input_focus() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-1".to_string(),
            title: "Pinned".to_string(),
            messages: vec![chat::AgentMessage {
                id: Some("message-1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Pinned content".to_string(),
                pinned_for_compaction: true,
                ..Default::default()
            }],
            loaded_message_end: 1,
            total_message_count: 1,
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Pinned));
    model.focus = FocusArea::Input;
    model.input.set_text("draft");

    let handled = model.handle_key(KeyCode::Char('k'), KeyModifiers::CONTROL);
    assert!(!handled);

    let handled = model.handle_key(KeyCode::Char('u'), KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.input.buffer(), "draft");
    assert!(
        !model.chat.active_thread_has_pinned_messages(),
        "selected pin should disappear immediately from the sidebar state"
    );
    assert_eq!(model.sidebar.active_tab(), SidebarTab::Todos);
    let command = cmd_rx
        .try_recv()
        .expect("Ctrl+K then U should unpin the selected message");
    assert!(matches!(
        command,
        DaemonCommand::UnpinThreadMessageForCompaction {
            thread_id,
            message_id
        } if thread_id == "thread-1" && message_id == "message-1"
    ));
}

#[test]
fn chat_unpin_updates_pinned_sidebar_without_waiting_for_thread_refresh() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.chat.reduce(chat::ChatAction::ThreadDetailReceived(
        crate::state::chat::AgentThread {
            id: "thread-1".to_string(),
            title: "Pinned".to_string(),
            messages: vec![chat::AgentMessage {
                id: Some("message-1".to_string()),
                role: chat::MessageRole::Assistant,
                content: "Pinned content".to_string(),
                pinned_for_compaction: true,
                ..Default::default()
            }],
            loaded_message_end: 1,
            total_message_count: 1,
            ..Default::default()
        },
    ));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Pinned));

    model.unpin_message_for_compaction(0);

    assert!(
        !model.chat.active_thread_has_pinned_messages(),
        "chat-side unpin should clear the pinned sidebar immediately"
    );
    assert_eq!(model.sidebar.active_tab(), SidebarTab::Todos);
    let command = cmd_rx
        .try_recv()
        .expect("chat-side unpin should still notify the daemon");
    assert!(matches!(
        command,
        DaemonCommand::UnpinThreadMessageForCompaction {
            thread_id,
            message_id
        } if thread_id == "thread-1" && message_id == "message-1"
    ));
}
