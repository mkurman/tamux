use super::super::{build_model, rendered_chat_area};
use super::*;
use ratatui::backend::TestBackend;

#[test]
fn drag_selection_does_not_rebuild_full_transcript_for_every_mouse_event() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
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
            content: "alpha beta gamma".to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find(|candidate| {
            *candidate > chat_area.y.saturating_add(1)
                && *candidate
                    < chat_area
                        .y
                        .saturating_add(chat_area.height)
                        .saturating_sub(2)
                && widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(3, *candidate),
                )
                .is_some()
        })
        .expect("chat transcript should expose a selectable row");

    let (anchor_col, drag_col) = (chat_area.x..chat_area.x.saturating_add(chat_area.width))
        .find_map(|start_col| {
            let start_point = widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(start_col, row),
            )?;
            ((start_col + 1)..chat_area.x.saturating_add(chat_area.width)).find_map(|end_col| {
                let end_point = widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(end_col, row),
                )?;
                (end_point != start_point).then_some((start_col, end_col))
            })
        })
        .expect("chat transcript should expose two distinct selectable columns");

    widgets::chat::reset_build_rendered_lines_call_count();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: anchor_col,
        row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: drag_col,
        row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: drag_col,
        row,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        0,
        "dragging a static selection should not rebuild the full transcript"
    );
}

#[test]
fn render_during_active_drag_reuses_cached_snapshot_and_shows_highlight() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
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
            content: (1..=80)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });
    model.chat.reduce(chat::ChatAction::ScrollChat(8));

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find(|candidate| {
            *candidate > chat_area.y.saturating_add(1)
                && *candidate
                    < chat_area
                        .y
                        .saturating_add(chat_area.height)
                        .saturating_sub(2)
                && widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(3, *candidate),
                )
                .is_some()
        })
        .expect("chat transcript should expose at least one selectable row");

    let (anchor_col, drag_col) = (chat_area.x..chat_area.x.saturating_add(chat_area.width))
        .find_map(|start_col| {
            let start_point = widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(start_col, row),
            )?;
            ((start_col + 1)..chat_area.x.saturating_add(chat_area.width)).find_map(|end_col| {
                let end_point = widgets::chat::selection_point_from_mouse(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    Position::new(end_col, row),
                )?;
                (end_point != start_point).then_some((start_col, end_col))
            })
        })
        .expect("chat transcript should expose two distinct selectable columns");

    widgets::chat::reset_build_rendered_lines_call_count();
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: anchor_col,
        row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: drag_col,
        row,
        modifiers: KeyModifiers::NONE,
    });

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("model render should succeed");

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        0,
        "active drag rendering should not rebuild the full transcript"
    );

    let buffer = terminal.backend().buffer();
    let highlighted = (0..model.height)
        .flat_map(|y| (0..model.width).filter_map(move |x| buffer.cell((x, y))))
        .filter(|cell| cell.bg == Color::Indexed(31))
        .count();
    assert!(
        highlighted > 0,
        "active drag should paint a visible selection highlight even while scrolled"
    );
}

#[test]
fn repeated_chat_renders_reuse_cached_snapshot_when_transcript_is_unchanged() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
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
            content: (1..=200)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });

    widgets::chat::reset_build_rendered_lines_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");
    terminal
        .draw(|frame| model.render(frame))
        .expect("second render should succeed");

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        0,
        "unchanged chat renders should not build a full transcript snapshot"
    );
}

#[test]
fn older_history_snapshot_renders_only_visible_markdown_window() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    for index in 0..240 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                id: Some(format!("msg-{index}")),
                content: format!("**message {index}**\n\n- detail {index}"),
                ..Default::default()
            },
        });
    }

    let chat_area = Rect::new(
        0,
        3,
        model.width,
        model
            .height
            .saturating_sub(model.input_height())
            .saturating_sub(4),
    );

    crate::widgets::message::reset_markdown_render_call_count();
    let snapshot = widgets::chat::build_selection_snapshot(
        chat_area,
        &model.chat,
        &model.theme,
        model.tick_counter,
        model.retry_wait_start_selected,
    );

    assert!(
        snapshot.is_some(),
        "large loaded transcript should produce a chat snapshot"
    );
    assert!(
        crate::widgets::message::markdown_render_call_count() < 80,
        "chat snapshot should render only the visible markdown window plus overscan, not every loaded message"
    );
}

#[test]
fn scrolling_large_markdown_history_keeps_visible_rows() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    for index in 0..40 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                id: Some(format!("msg-{index}")),
                content: (0..18)
                    .map(|line| format!("message {index} paragraph segment {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });
    }
    model.chat.reduce(chat::ChatAction::ScrollChat(120));

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("render after scrolling large markdown history should succeed");

    let visible = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(
        visible.contains("message "),
        "scrolled markdown history should keep message content visible instead of blanking the viewport"
    );
}

#[test]
fn scrolling_reuses_cached_snapshot_and_updates_visible_window() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
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
            content: (1..=200)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });

    widgets::chat::reset_build_rendered_lines_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");
    let before = terminal.backend().buffer().clone();

    model.chat.reduce(chat::ChatAction::ScrollChat(8));

    terminal
        .draw(|frame| model.render(frame))
        .expect("second render after scroll should succeed");
    let after = terminal.backend().buffer().clone();

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        0,
        "scrolling should update the visible window without rebuilding all lines"
    );
    assert_ne!(
        before, after,
        "scrolling should still change the visible transcript window"
    );
}

#[test]
fn scrolling_beyond_cached_overscan_keeps_chat_content_visible() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
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
            content: (1..=220)
                .map(|idx| format!("history row {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });

    widgets::chat::reset_build_rendered_lines_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");

    model.chat.reduce(chat::ChatAction::ScrollChat(120));

    terminal
        .draw(|frame| model.render(frame))
        .expect("second render after large scroll should succeed");

    let visible = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    assert!(
        visible.contains("history row "),
        "scrolling past cached overscan should render message content instead of a blank chat view"
    );
    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        0,
        "large scrolls should rebuild only the virtual window, not the full transcript"
    );
}

#[test]
fn render_clamps_stale_scroll_after_bottom_messages_are_deleted() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model
        .chat
        .reduce(chat::ChatAction::ThreadDetailReceived(chat::AgentThread {
            id: "thread-1".to_string(),
            title: "Thread".to_string(),
            total_message_count: 30,
            loaded_message_start: 0,
            loaded_message_end: 30,
            messages: (0..30)
                .map(|index| chat::AgentMessage {
                    id: Some(format!("m{index}")),
                    role: chat::MessageRole::Assistant,
                    content: format!("message {index}\nbody {index}\nmore {index}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }));
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("initial render should succeed");

    model
        .chat
        .reduce(chat::ChatAction::ScrollChat(i32::MAX / 2));
    terminal
        .draw(|frame| model.render(frame))
        .expect("scrolled render should succeed");
    assert!(
        model.chat.scroll_offset() > 0,
        "test setup should leave the transcript scrolled away from bottom"
    );

    for _ in 0..26 {
        let last = model
            .chat
            .active_thread()
            .map(|thread| thread.messages.len().saturating_sub(1))
            .expect("thread should remain");
        model.chat.delete_active_message(last);
    }
    model.chat.reduce(chat::ChatAction::ScrollChat(-3));

    terminal
        .draw(|frame| model.render(frame))
        .expect("render after delete and scroll should succeed");

    assert_eq!(
        model.chat.scroll_offset(),
        0,
        "render should clamp a stale scroll offset after deleted rows shrink the transcript"
    );
    let visible = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(
        visible.contains("message "),
        "remaining messages should render instead of an empty cached window"
    );
}

#[test]
fn scrolling_beyond_cached_overscan_reuses_transcript_metrics() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    for index in 0..240 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                id: Some(format!("msg-{index}")),
                content: (0..20)
                    .map(|line| format!("message {index} paragraph {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });
    }

    widgets::chat::reset_build_transcript_metrics_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");

    model.chat.reduce(chat::ChatAction::ScrollChat(160));

    terminal
        .draw(|frame| model.render(frame))
        .expect("second render after large scroll should succeed");

    assert_eq!(
        widgets::chat::build_transcript_metrics_call_count(),
        1,
        "scrolling the loaded history window should reuse cached transcript metrics"
    );
}

#[test]
fn streaming_delta_reuses_cached_history_metrics() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    for index in 0..240 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                id: Some(format!("msg-{index}")),
                content: (0..20)
                    .map(|line| format!("message {index} paragraph {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });
    }

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("initial render should build a cached transcript snapshot");

    widgets::chat::reset_build_transcript_metrics_call_count();
    model.chat.reduce(chat::ChatAction::Delta {
        thread_id: "thread-1".to_string(),
        content: "streaming tail".to_string(),
    });

    terminal
        .draw(|frame| model.render(frame))
        .expect("streaming render should succeed");

    assert_eq!(
        widgets::chat::build_transcript_metrics_call_count(),
        0,
        "streaming deltas should reuse cached stored-message metrics instead of rescanning loaded history"
    );
}

#[test]
fn message_highlight_toggle_reuses_cached_history_metrics() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    for index in 0..240 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                id: Some(format!("msg-{index}")),
                content: (0..20)
                    .map(|line| format!("message {index} paragraph {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });
    }

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("initial render should build a cached transcript snapshot");

    widgets::chat::reset_build_transcript_metrics_call_count();
    model.chat.select_message(Some(239));
    terminal
        .draw(|frame| model.render(frame))
        .expect("highlight render should succeed");
    model.chat.select_message(None);
    terminal
        .draw(|frame| model.render(frame))
        .expect("dehighlight render should succeed");

    assert_eq!(
        widgets::chat::build_transcript_metrics_call_count(),
        0,
        "highlighting or dehighlighting a message should reuse cached stored-message metrics"
    );
}

#[test]
fn mouse_click_highlight_reuses_rendered_chat_snapshot() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));

    for index in 0..240 {
        model.chat.reduce(chat::ChatAction::AppendMessage {
            thread_id: "thread-1".to_string(),
            message: chat::AgentMessage {
                role: chat::MessageRole::Assistant,
                id: Some(format!("msg-{index}")),
                content: (0..20)
                    .map(|line| format!("message {index} paragraph {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                ..Default::default()
            },
        });
    }

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("initial render should build a cached transcript snapshot");

    let chat_area = rendered_chat_area(&model);
    let click_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            let pos = Position::new(chat_area.x.saturating_add(4), row);
            matches!(
                model.cached_chat_hit_test(chat_area, pos),
                Some(chat::ChatHitTarget::Message(_))
            )
            .then_some(pos)
        })
        .expect("visible chat should expose a clickable message row");

    widgets::chat::reset_build_transcript_metrics_call_count();
    widgets::chat::reset_build_rendered_lines_call_count();
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click_pos.x,
        row: click_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: click_pos.x,
        row: click_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert!(
        model.chat.selected_message().is_some(),
        "click should still select a visible message"
    );
    assert_eq!(
        widgets::chat::build_transcript_metrics_call_count(),
        0,
        "click highlight should reuse the snapshot from the preceding render instead of rebuilding metrics"
    );
    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        0,
        "click highlight should not rebuild the full transcript"
    );
}

#[test]
fn stale_cached_snapshot_is_ignored_after_sidebar_layout_change() {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
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
            content: "hello world".to_string(),
            ..Default::default()
        },
    });

    let full_width_area = Rect::new(
        0,
        3,
        model.width,
        model.height.saturating_sub(model.input_height() + 4),
    );
    model.chat_selection_snapshot = widgets::chat::build_selection_snapshot(
        full_width_area,
        &model.chat,
        &model.theme,
        model.tick_counter,
        model.retry_wait_start_selected,
    );
    model.chat_drag_anchor = None;
    model.chat_drag_current = None;
    model.chat_drag_anchor_point = None;
    model.chat_drag_current_point = None;

    model.tasks.reduce(task::TaskAction::WorkContextReceived(
        task::ThreadWorkContext {
            thread_id: "thread-1".to_string(),
            entries: vec![task::WorkContextEntry {
                path: "/tmp/demo.txt".to_string(),
                is_text: true,
                ..Default::default()
            }],
        },
    ));
    model.show_sidebar_override = Some(true);

    widgets::chat::reset_build_rendered_lines_call_count();
    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("render should fall back to fresh layout instead of using stale snapshot");

    assert_eq!(
        widgets::chat::build_rendered_lines_call_count(),
        0,
        "layout changes should rebuild the visible chat rows without building the full transcript"
    );
}

#[test]
fn mouse_drag_snapshot_uses_rendered_chat_area_without_sidebar() {
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
        "drag snapshots must use the exact rendered chat area"
    );
}

#[test]
fn repeated_sidebar_renders_reuse_cached_snapshot_when_history_is_unchanged() {
    let mut model = build_model();
    model.show_sidebar_override = Some(true);
    model.focus = FocusArea::Sidebar;
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
            entries: (0..200)
                .map(|idx| task::WorkContextEntry {
                    path: format!("/tmp/file-{idx:03}.rs"),
                    change_kind: Some("modified".to_string()),
                    is_text: true,
                    ..Default::default()
                })
                .collect(),
        },
    ));
    model.activate_sidebar_tab(SidebarTab::Files);

    widgets::sidebar::reset_build_cached_snapshot_call_count();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("first render should succeed");
    terminal
        .draw(|frame| model.render(frame))
        .expect("second render should succeed");

    assert_eq!(
        widgets::sidebar::build_cached_snapshot_call_count(),
        1,
        "unchanged sidebar renders should reuse the cached sidebar snapshot"
    );
}
