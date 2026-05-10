use super::*;
use super::super::{build_model, rendered_chat_area};
#[test]
fn drag_selection_copies_expected_text_after_autoscroll() {
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
            content: (1..=80)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
    let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
        .chain(chat_area.y..preferred_row)
        .find(|row| {
            widgets::chat::selection_point_from_mouse(
                chat_area,
                &model.chat,
                &model.theme,
                model.tick_counter,
                Position::new(3, *row),
            )
            .is_some()
        })
        .expect("chat transcript should expose at least one selectable row");

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 3,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });
    for _ in 0..4 {
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
    }

    let anchor_point = model
        .chat_drag_anchor_point
        .expect("mouse down should capture a document anchor point");
    let current_point = model
        .chat_drag_current_point
        .expect("autoscroll should extend the current drag point");
    let expected = widgets::chat::selected_text(
        chat_area,
        &model.chat,
        &model.theme,
        model.tick_counter,
        anchor_point,
        current_point,
    )
    .expect("selection should resolve to copied text");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: 3,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        super::conversion::last_copied_text().as_deref(),
        Some(expected.as_str())
    );
    assert_eq!(model.status_line, "Copied selection to clipboard");
}

#[test]
fn work_context_drag_selection_copies_beyond_visible_window() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
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
            entries: vec![task::WorkContextEntry {
                path: "/tmp/demo.txt".to_string(),
                is_text: true,
                ..Default::default()
            }],
        },
    ));
    model
        .tasks
        .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
            path: "/tmp/demo.txt".to_string(),
            content: (1..=80)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            truncated: false,
            is_text: true,
        }));
    model.tasks.reduce(task::TaskAction::SelectWorkPath {
        thread_id: "thread-1".to_string(),
        path: Some("/tmp/demo.txt".to_string()),
    });
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Files));
    model.main_pane_view = MainPaneView::WorkContext;
    model.focus = FocusArea::Chat;

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let preferred_row = chat_area.y.saturating_add(chat_area.height / 2);
    let start_row = (preferred_row..chat_area.y.saturating_add(chat_area.height))
        .chain(chat_area.y..preferred_row)
        .find(|row| {
            widgets::work_context_view::selection_point_from_mouse(
                chat_area,
                &model.tasks,
                model.chat.active_thread_id(),
                model.sidebar.active_tab(),
                model.sidebar.selected_item(),
                &model.theme,
                model.task_view_scroll,
                Position::new(3, *row),
            )
            .is_some()
        })
        .expect("work-context preview should expose at least one selectable row");

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 3,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });
    for _ in 0..4 {
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 3,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
    }

    let anchor_point = model
        .work_context_drag_anchor_point
        .expect("mouse down should capture a preview anchor point");
    let current_point = model
        .work_context_drag_current_point
        .expect("scrolling should extend the preview selection");
    let expected = widgets::work_context_view::selected_text(
        chat_area,
        &model.tasks,
        model.chat.active_thread_id(),
        model.sidebar.active_tab(),
        model.sidebar.selected_item(),
        &model.theme,
        model.task_view_scroll,
        anchor_point,
        current_point,
    )
    .expect("selection should resolve to copied preview text");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: 3,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        super::conversion::last_copied_text().as_deref(),
        Some(expected.as_str())
    );
}

#[test]
fn goal_view_drag_selection_copies_beyond_visible_window() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.task_show_live_todos = false;
    model.task_show_timeline = false;
    model.task_show_files = false;
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Large Goal".to_string(),
            steps: (1..=80)
                .map(|idx| task::GoalRunStep {
                    id: format!("step-{idx}"),
                    title: format!("Step {idx}"),
                    order: idx - 1,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let chat_area = rendered_chat_area(&model);
    let start_row = chat_area.y.saturating_add(6);
    let start_col = chat_area.x.saturating_add(4);

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: start_col,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });
    for _ in 0..4 {
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: start_col,
            row: start_row,
            modifiers: KeyModifiers::NONE,
        });
    }
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: start_col,
        row: start_row,
        modifiers: KeyModifiers::NONE,
    });

    let copied = super::conversion::last_copied_text()
        .expect("dragging across goal view content should copy selected text");
    assert!(
        copied.contains("Step"),
        "expected goal selection to include goal text, got: {copied:?}"
    );
    assert_eq!(model.status_line, "Copied selection to clipboard");
}

#[test]
fn goal_view_drag_selection_copies_expanded_goal_prompt() {
    let mut model = build_model();
    model.show_sidebar_override = Some(false);
    model.focus = FocusArea::Chat;
    model.goal_workspace.set_prompt_expanded(true);
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Prompt Goal".to_string(),
            goal: "Copy this exact expanded goal prompt from the mission plan.".to_string(),
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let chat_area = rendered_chat_area(&model);
    let prompt_row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find(|row| {
            let start = widgets::goal_workspace::selection_point_from_mouse(
                chat_area,
                &model.tasks,
                "goal-1",
                &model.goal_workspace,
                Position::new(chat_area.x.saturating_add(5), *row),
            );
            let end = widgets::goal_workspace::selection_point_from_mouse(
                chat_area,
                &model.tasks,
                "goal-1",
                &model.goal_workspace,
                Position::new(chat_area.x.saturating_add(25), *row),
            );
            start.zip(end).is_some_and(|(start, end)| {
                start != end
                    && widgets::goal_workspace::selected_text(
                        chat_area,
                        &model.tasks,
                        "goal-1",
                        &model.goal_workspace,
                        start,
                        end,
                    )
                    .is_some_and(|text| text.contains("Copy"))
            })
        })
        .expect("expanded goal prompt should expose selectable text");
    let start = widgets::goal_workspace::selection_point_from_mouse(
        chat_area,
        &model.tasks,
        "goal-1",
        &model.goal_workspace,
        Position::new(chat_area.x.saturating_add(5), prompt_row),
    )
    .expect("expanded goal prompt should expose a selectable start point");
    let end = widgets::goal_workspace::selection_point_from_mouse(
        chat_area,
        &model.tasks,
        "goal-1",
        &model.goal_workspace,
        Position::new(chat_area.x.saturating_add(25), prompt_row),
    )
    .expect("expanded goal prompt should expose a selectable end point");
    assert_ne!(start, end, "prompt selection points should differ");

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chat_area.x.saturating_add(5),
        row: prompt_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: chat_area.x.saturating_add(25),
        row: prompt_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: chat_area.x.saturating_add(25),
        row: prompt_row,
        modifiers: KeyModifiers::NONE,
    });

    let copied = super::conversion::last_copied_text()
        .expect("dragging across expanded goal prompt should copy text");
    assert!(
        copied.contains("Copy this exact"),
        "expected selected text from goal prompt, got: {copied:?}"
    );
    assert_eq!(model.status_line, "Copied selection to clipboard");
}

#[test]
fn file_preview_drag_selection_copies_preview_text() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model
        .tasks
        .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
            path: "/tmp/demo.txt".to_string(),
            content: "alpha preview line\nbeta preview line\ngamma preview line".to_string(),
            truncated: false,
            is_text: true,
        }));
    model.main_pane_view = MainPaneView::FilePreview(ChatFilePreviewTarget {
        path: "/tmp/demo.txt".to_string(),
        repo_root: None,
        repo_relative_path: None,
    });

    let chat_area = rendered_chat_area(&model);
    let start_col = chat_area.x;
    let end_col = chat_area.x.saturating_add(13);
    let content_row = chat_area.y.saturating_add(5);

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: start_col,
        row: content_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: end_col,
        row: content_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: end_col,
        row: content_row,
        modifiers: KeyModifiers::NONE,
    });

    let copied = super::conversion::last_copied_text()
        .expect("dragging across file preview should copy selected text");
    assert!(
        copied.contains("alpha preview"),
        "expected selected preview text, got: {copied:?}"
    );
    assert_eq!(model.status_line, "Copied selection to clipboard");
}

#[test]
fn file_preview_drag_selection_copies_header_path() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::FilePreview(ChatFilePreviewTarget {
        path: "/tmp/demo.txt".to_string(),
        repo_root: None,
        repo_relative_path: None,
    });

    let chat_area = rendered_chat_area(&model);
    let path_row = chat_area.y.saturating_add(2);
    let start_col = chat_area.x.saturating_add(6);
    let end_col = start_col.saturating_add("/tmp/demo.txt".len() as u16);

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: start_col,
        row: path_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: end_col,
        row: path_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: end_col,
        row: path_row,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        super::conversion::last_copied_text().as_deref(),
        Some("/tmp/demo.txt")
    );
    assert_eq!(model.status_line, "Copied selection to clipboard");
}

#[test]
fn ctrl_c_copies_active_file_preview_path_selection() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::FilePreview(ChatFilePreviewTarget {
        path: "/tmp/demo.txt".to_string(),
        repo_root: None,
        repo_relative_path: None,
    });

    let chat_area = rendered_chat_area(&model);
    let path_row = chat_area.y.saturating_add(2);
    let start_col = chat_area.x.saturating_add(6);
    let end_col = start_col.saturating_add("/tmp/demo.txt".len() as u16);

    super::conversion::reset_last_copied_text();

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: start_col,
        row: path_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: end_col,
        row: path_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_key(KeyCode::Char('c'), KeyModifiers::CONTROL);

    assert_eq!(
        super::conversion::last_copied_text().as_deref(),
        Some("/tmp/demo.txt")
    );
    assert_eq!(model.status_line, "Copied selection to clipboard");
}

#[test]
fn esc_closes_work_context_even_from_input_focus() {
    let mut model = build_model();
    model.focus = FocusArea::Input;
    model.main_pane_view = MainPaneView::WorkContext;

    let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.focus, FocusArea::Chat);
}

#[test]
fn status_modal_mouse_wheel_scrolls_body() {
    let mut model = build_model();
    model.status_modal_snapshot = Some(crate::client::AgentStatusSnapshotVm {
        tier: "mission_control".to_string(),
        activity: "waiting_for_operator".to_string(),
        active_thread_id: Some("thread-1".to_string()),
        active_goal_run_id: None,
        active_goal_run_title: Some("Close release gap".to_string()),
        provider_health_json: r#"{"openai":{"can_execute":true,"trip_count":0}}"#.to_string(),
        gateway_statuses_json: r#"{"slack":{"status":"connected"}}"#.to_string(),
        recent_actions_json: serde_json::to_string(
            &(0..40)
                .map(|idx| {
                    serde_json::json!({
                        "action_type": format!("tool_{idx}"),
                        "summary": format!("summary {idx}"),
                        "timestamp": 1712345678_u64 + idx,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    });
    model.status_modal_diagnostics_json = Some(
        serde_json::json!({
            "aline": {
                "available": true,
                "watcher_state": "running",
                "imported_count": 1,
                "generated_count": 1,
            }
        })
        .to_string(),
    );
    model
        .modal
        .reduce(modal::ModalAction::Push(modal::ModalKind::Status));

    let (_, overlay_area) = model
        .current_modal_area()
        .expect("status modal should expose an overlay area");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: overlay_area.x.saturating_add(2),
        row: overlay_area.y.saturating_add(2),
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.status_modal_scroll, 3);
}
