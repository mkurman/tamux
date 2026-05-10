use super::super::{build_model, rendered_chat_area, unbounded_channel};
use super::*;
use ratatui::backend::TestBackend;
use std::sync::mpsc;

#[test]
fn sidebar_mouse_click_reuses_cached_snapshot_for_spawned_history() {
    let mut model = build_model();
    model.show_sidebar_override = Some(true);
    model.focus = FocusArea::Sidebar;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-child".to_string(),
        title: "Child".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-root".to_string()));
    model.tasks.reduce(task::TaskAction::TaskListReceived(vec![
        task::AgentTask {
            id: "root-task".to_string(),
            title: "Root worker".to_string(),
            created_at: 20,
            thread_id: Some("thread-root".to_string()),
            status: Some(task::TaskStatus::InProgress),
            ..Default::default()
        },
        task::AgentTask {
            id: "child-task".to_string(),
            title: "Child worker".to_string(),
            created_at: 10,
            thread_id: Some("thread-child".to_string()),
            parent_task_id: Some("root-task".to_string()),
            parent_thread_id: Some("thread-root".to_string()),
            status: Some(task::TaskStatus::InProgress),
            ..Default::default()
        },
    ]));
    model.activate_sidebar_tab(SidebarTab::Spawned);

    widgets::sidebar::reset_build_cached_snapshot_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("initial render should succeed");

    let sidebar_area = model
        .pane_layout()
        .sidebar
        .expect("sidebar should be visible");
    let buffer = terminal.backend().buffer();
    let child_row = (sidebar_area.y..sidebar_area.y.saturating_add(sidebar_area.height))
        .find(|row| {
            (sidebar_area.x..sidebar_area.x.saturating_add(sidebar_area.width))
                .filter_map(|x| buffer.cell((x, *row)).map(|cell| cell.symbol()))
                .collect::<String>()
                .contains("Child worker")
        })
        .expect("rendered sidebar should include the child spawned row");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: sidebar_area.x.saturating_add(2),
        row: child_row,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        widgets::sidebar::build_cached_snapshot_call_count(),
        1,
        "sidebar clicks should reuse the cached sidebar snapshot after the initial render"
    );
    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
}

#[test]
fn repeated_spawned_sidebar_renders_do_not_reflatten_unchanged_tree() {
    let mut model = build_model();
    model.show_sidebar_override = Some(true);
    model.focus = FocusArea::Sidebar;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-child".to_string(),
        title: "Child".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-root".to_string()));
    model.tasks.reduce(task::TaskAction::TaskListReceived(vec![
        task::AgentTask {
            id: "root-task".to_string(),
            title: "Root worker".to_string(),
            created_at: 20,
            thread_id: Some("thread-root".to_string()),
            status: Some(task::TaskStatus::InProgress),
            ..Default::default()
        },
        task::AgentTask {
            id: "child-task".to_string(),
            title: "Child worker".to_string(),
            created_at: 10,
            thread_id: Some("thread-child".to_string()),
            parent_task_id: Some("root-task".to_string()),
            parent_thread_id: Some("thread-root".to_string()),
            status: Some(task::TaskStatus::InProgress),
            ..Default::default()
        },
    ]));
    model.activate_sidebar_tab(SidebarTab::Spawned);

    widgets::sidebar::reset_spawned_sidebar_flatten_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");
    terminal
        .draw(|frame| model.render(frame))
        .expect("second render should succeed");

    assert_eq!(
        widgets::sidebar::spawned_sidebar_flatten_call_count(),
        1,
        "unchanged spawned sidebar renders should not recompute the flattened tree"
    );
}

#[test]
fn anticipatory_events_no_longer_shrink_chat_area() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.show_sidebar_override = Some(false);
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    let original = rendered_chat_area(&model);

    model.handle_client_event(crate::client::ClientEvent::AnticipatoryItems(vec![
        crate::wire::AnticipatoryItem {
            id: "digest-1".to_string(),
            title: "Task May Be Stuck".to_string(),
            summary: "The task has been blocked for 20 minutes.".to_string(),
            confidence: 0.78,
            ..Default::default()
        },
    ]));

    assert_eq!(rendered_chat_area(&model), original);
}

#[test]
fn mouse_drag_snapshot_uses_rendered_chat_area_with_sidebar() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.show_sidebar_override = Some(true);
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model
        .anticipatory
        .reduce(crate::state::AnticipatoryAction::Replace(vec![
            crate::wire::AnticipatoryItem {
                id: "digest-1".to_string(),
                ..Default::default()
            },
        ]));
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Assistant,
            content: "alpha\nbeta\ngamma\ndelta".to_string(),
            ..Default::default()
        },
    });

    let chat_area = rendered_chat_area(&model);
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chat_area.x.saturating_add(3),
        row: chat_area
            .y
            .saturating_add(chat_area.height.saturating_sub(2)),
        modifiers: KeyModifiers::NONE,
    });

    let snapshot = model
        .chat_selection_snapshot
        .as_ref()
        .expect("mouse down should create a chat selection snapshot");
    assert!(
        widgets::chat::cached_snapshot_matches_area(snapshot, chat_area),
        "sidebar drag snapshots must use the exact rendered chat area"
    );
}

#[test]
fn thread_detail_refresh_clears_active_chat_drag_snapshot() {
    let (daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, _cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.show_sidebar_override = Some(true);
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
            content: "short content".to_string(),
            ..Default::default()
        },
    });

    let chat_area = rendered_chat_area(&model);
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chat_area.x.saturating_add(3),
        row: chat_area
            .y
            .saturating_add(chat_area.height.saturating_sub(2)),
        modifiers: KeyModifiers::NONE,
    });
    assert!(model.chat_selection_snapshot.is_some());

    daemon_tx
        .send(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
            id: "thread-1".to_string(),
            title: "Thread".to_string(),
            messages: vec![crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Assistant,
                content: (1..=120)
                    .map(|idx| format!("line {idx}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            }],
            ..Default::default()
        })))
        .expect("thread detail event should send");
    model.pump_daemon_events();

    assert!(
        model.chat_selection_snapshot.is_none(),
        "thread-detail refresh should invalidate stale drag snapshots"
    );
    assert!(model.chat_drag_anchor.is_none());
    assert!(model.chat_drag_current.is_none());
    assert!(model.chat_drag_anchor_point.is_none());
    assert!(model.chat_drag_current_point.is_none());
}

#[test]
fn clicking_chat_scrollbar_track_scrolls_transcript() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.show_sidebar_override = Some(false);
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    for idx in 0..40 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: format!("message {idx}"),
                ..Default::default()
            },
        });
    }

    let chat_area = rendered_chat_area(&model);
    let before = model.chat.scroll_offset();
    let column = chat_area
        .x
        .saturating_add(chat_area.width)
        .saturating_sub(1);
    let row = chat_area.y.saturating_add(1);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    });

    assert!(
        model.chat.scroll_offset() > before,
        "clicking the scrollbar track should scroll toward older transcript content"
    );
}

#[test]
fn dragging_chat_scrollbar_thumb_updates_scroll_offset() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.show_sidebar_override = Some(false);
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    for idx in 0..50 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                content: format!("message {idx}"),
                ..Default::default()
            },
        });
    }

    let chat_area = rendered_chat_area(&model);
    let column = chat_area
        .x
        .saturating_add(chat_area.width)
        .saturating_sub(1);
    let start_row = chat_area.y.saturating_add(chat_area.height / 2);
    let end_row = chat_area
        .y
        .saturating_add(chat_area.height)
        .saturating_sub(2);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });

    let after_press = model.chat.scroll_offset();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column,
        row: end_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column,
        row: end_row,
        modifiers: KeyModifiers::NONE,
    });

    assert!(
        model.chat.scroll_offset() != after_press,
        "dragging the scrollbar thumb should update transcript scroll position"
    );
}

#[test]
fn thread_detail_conversion_preserves_weles_review_metadata() {
    let (daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, _cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);

    daemon_tx
        .send(ClientEvent::ThreadDetail(Some(crate::wire::AgentThread {
            id: "thread-1".to_string(),
            title: "Thread".to_string(),
            messages: vec![crate::wire::AgentMessage {
                role: crate::wire::MessageRole::Tool,
                content: "done".to_string(),
                tool_name: Some("bash_command".to_string()),
                tool_call_id: Some("call-1".to_string()),
                tool_status: Some("done".to_string()),
                weles_review: Some(crate::wire::WelesReviewMetaVm {
                    weles_reviewed: false,
                    verdict: "allow".to_string(),
                    reasons: vec!["governance_not_run".to_string()],
                    audit_id: Some("audit-wire-1".to_string()),
                    security_override_mode: None,
                }),
                ..Default::default()
            }],
            ..Default::default()
        })))
        .expect("thread detail event should send");
    model.pump_daemon_events();

    let thread = model.chat.active_thread().expect("thread should exist");
    let message = thread.messages.last().expect("message should exist");
    let stored = message
        .weles_review
        .as_ref()
        .expect("weles review should survive conversion");
    assert!(!stored.weles_reviewed);
    assert_eq!(stored.verdict, "allow");
    assert_eq!(stored.audit_id.as_deref(), Some("audit-wire-1"));
}
