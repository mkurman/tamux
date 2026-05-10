use super::*;
use super::super::{build_model, rendered_chat_area};
use ratatui::backend::TestBackend;
    use base64::Engine as _;
#[test]
fn goal_view_scroll_up_moves_off_bottom_after_mouse_overscroll() {
    fn render_task_view(model: &mut TuiModel) -> Vec<String> {
        let backend = TestBackend::new(model.width, model.height);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        terminal
            .draw(|frame| model.render(frame))
            .expect("task view render should succeed");

        let chat_area = rendered_chat_area(model);
        let buffer = terminal.backend().buffer();
        (chat_area.y..chat_area.y.saturating_add(chat_area.height))
            .map(|y| {
                (chat_area.x..chat_area.x.saturating_add(chat_area.width))
                    .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                    .collect::<String>()
            })
            .collect()
    }

    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model.task_show_live_todos = false;
    model.task_show_timeline = false;
    model.task_show_files = false;

    let steps = (1..=60)
        .map(|idx| task::GoalRunStep {
            id: format!("step-{idx}"),
            title: format!("Step {idx}"),
            instructions: format!(
                "Line {idx}A\nLine {idx}B\nLine {idx}C\nLine {idx}D\nLine {idx}E"
            ),
            order: idx,
            ..Default::default()
        })
        .collect();

    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Large Goal".to_string(),
            goal: (1..=40)
                .map(|idx| format!("Goal line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            steps,
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let chat_area = rendered_chat_area(&model);
    let mouse_column = chat_area.x.saturating_add(1);
    let mouse_row = chat_area.y.saturating_add(1);

    for _ in 0..400 {
        model.handle_mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: mouse_column,
            row: mouse_row,
            modifiers: KeyModifiers::NONE,
        });
    }

    let bottom_snapshot = render_task_view(&mut model);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: mouse_column,
        row: mouse_row,
        modifiers: KeyModifiers::NONE,
    });

    let after_one_up = render_task_view(&mut model);

    assert_ne!(
        after_one_up, bottom_snapshot,
        "one upward scroll should move away from the clamped bottom view"
    );
}

#[test]
fn task_view_renders_visible_scrollbar_when_content_overflows() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);

    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Large Goal".to_string(),
            goal: (1..=80)
                .map(|idx| format!("Goal line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            steps: (1..=40)
                .map(|idx| task::GoalRunStep {
                    id: format!("step-{idx}"),
                    title: format!("Step {idx}"),
                    instructions: format!("Line {idx}A\nLine {idx}B\nLine {idx}C"),
                    order: idx,
                    ..Default::default()
                })
                .collect(),
            events: (1..=40)
                .map(|idx| task::GoalRunEvent {
                    id: format!("event-{idx}"),
                    message: format!("Event {idx}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: None,
    });

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("task view render should succeed");

    let chat_area = rendered_chat_area(&model);
    let buffer = terminal.backend().buffer();
    let plain = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .map(|y| {
            buffer
                .cell((chat_area.x + chat_area.width - 1, y))
                .map(|cell| cell.symbol().to_string())
                .unwrap_or_default()
        })
        .collect::<String>();

    assert!(
        plain.contains("│") || plain.contains("█"),
        "expected visible scrollbar in task view, got: {plain:?}"
    );
}

#[test]
fn work_context_view_renders_visible_scrollbar_when_preview_overflows() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
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
            content: (1..=120)
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

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("work-context render should succeed");

    let chat_area = rendered_chat_area(&model);
    let buffer = terminal.backend().buffer();
    let plain = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .map(|y| {
            buffer
                .cell((chat_area.x + chat_area.width - 1, y))
                .map(|cell| cell.symbol().to_string())
                .unwrap_or_default()
        })
        .collect::<String>();

    assert!(
        plain.contains("│") || plain.contains("█"),
        "expected visible scrollbar in work-context view, got: {plain:?}"
    );
}

#[test]
fn file_preview_view_renders_visible_scrollbar_when_preview_overflows() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model
        .tasks
        .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
            path: "/tmp/demo.txt".to_string(),
            content: (1..=120)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            truncated: false,
            is_text: true,
        }));
    model.main_pane_view = MainPaneView::FilePreview(ChatFilePreviewTarget {
        path: "/tmp/demo.txt".to_string(),
        repo_root: None,
        repo_relative_path: None,
    });

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("file preview render should succeed");

    let chat_area = rendered_chat_area(&model);
    let buffer = terminal.backend().buffer();
    let plain = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .map(|y| {
            buffer
                .cell((chat_area.x + chat_area.width - 1, y))
                .map(|cell| cell.symbol().to_string())
                .unwrap_or_default()
        })
        .collect::<String>();

    assert!(
        plain.contains("│") || plain.contains("█"),
        "expected visible scrollbar in file preview view, got: {plain:?}"
    );
}

#[test]
fn file_preview_scrollbar_thumb_drags_detail_scroll() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);
    model
        .tasks
        .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
            path: "/tmp/demo.txt".to_string(),
            content: (1..=240)
                .map(|idx| format!("line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            truncated: false,
            is_text: true,
        }));
    model.main_pane_view = MainPaneView::FilePreview(ChatFilePreviewTarget {
        path: "/tmp/demo.txt".to_string(),
        repo_root: None,
        repo_relative_path: None,
    });
    model.task_view_scroll = 40;

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("file preview render should succeed");

    let chat_area = rendered_chat_area(&model);
    let buffer = terminal.backend().buffer();
    let thumb_row = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find(|y| {
            buffer
                .cell((chat_area.x + chat_area.width - 1, *y))
                .is_some_and(|cell| cell.symbol() == "█")
        })
        .expect("expected draggable scrollbar thumb to be visible");
    let drag_row = thumb_row.saturating_add(6).min(
        chat_area
            .y
            .saturating_add(chat_area.height)
            .saturating_sub(1),
    );

    let initial_scroll = model.task_view_scroll;

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chat_area.x + chat_area.width - 1,
        row: thumb_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Drag(MouseButton::Left),
        column: chat_area.x + chat_area.width - 1,
        row: drag_row,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: chat_area.x + chat_area.width - 1,
        row: drag_row,
        modifiers: KeyModifiers::NONE,
    });

    assert!(
        model.task_view_scroll > initial_scroll,
        "dragging the visible preview scrollbar thumb should advance preview scroll"
    );
}

#[test]
fn file_preview_view_renders_image_preview_instead_of_binary_placeholder() {
    let mut model = build_model();
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);

    let image_path =
        std::env::temp_dir().join(format!("zorai-preview-image-{}.png", uuid::Uuid::new_v4()));
    std::fs::write(
        &image_path,
        base64::engine::general_purpose::STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO0pGfcAAAAASUVORK5CYII=")
            .expect("fixture PNG should decode"),
    )
    .expect("fixture PNG should write");

    model
        .tasks
        .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
            path: image_path.display().to_string(),
            content: String::new(),
            truncated: false,
            is_text: false,
        }));
    model.main_pane_view = MainPaneView::FilePreview(ChatFilePreviewTarget {
        path: image_path.display().to_string(),
        repo_root: None,
        repo_relative_path: None,
    });

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("file preview render should succeed");

    let chat_area = rendered_chat_area(&model);
    let buffer = terminal.backend().buffer();
    let plain = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .flat_map(|y| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width))
                .filter_map(move |x| buffer.cell((x, y)).map(|cell| cell.symbol().to_string()))
        })
        .collect::<String>();

    assert!(
        plain.contains("Image:"),
        "expected image preview header, got: {plain:?}"
    );
    assert!(
        !plain.contains("Binary file preview is not available."),
        "expected image preview instead of binary placeholder, got: {plain:?}"
    );
}

#[test]
fn file_preview_view_uses_available_height_for_image_preview() {
    let mut model = build_model();
    model.width = 100;
    model.height = 40;
    model.focus = FocusArea::Chat;
    model.show_sidebar_override = Some(false);

    let image_path =
        std::env::temp_dir().join(format!("zorai-preview-image-{}.png", uuid::Uuid::new_v4()));
    image::RgbaImage::from_fn(1024, 1024, |x, y| {
        image::Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255])
    })
    .save(&image_path)
    .expect("fixture PNG should write");

    model
        .tasks
        .reduce(task::TaskAction::FilePreviewReceived(task::FilePreview {
            path: image_path.display().to_string(),
            content: String::new(),
            truncated: false,
            is_text: false,
        }));
    model.main_pane_view = MainPaneView::FilePreview(ChatFilePreviewTarget {
        path: image_path.display().to_string(),
        repo_root: None,
        repo_relative_path: None,
    });

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("file preview render should succeed");
    assert!(
        crate::widgets::image_preview::process_preview_jobs_for_path_until_stable_for_tests(
            &image_path.display().to_string(),
        ),
        "expected initial file preview render to queue and complete image preview work",
    );
    terminal
        .draw(|frame| model.render(frame))
        .expect("file preview rerender should use cached image preview");
    assert!(
        crate::widgets::image_preview::process_preview_jobs_for_path_until_stable_for_tests(
            &image_path.display().to_string(),
        ),
        "expected rerendered file preview to settle any resized image preview work",
    );
    terminal
        .draw(|frame| model.render(frame))
        .expect("file preview final render should use settled image preview");

    let chat_area = rendered_chat_area(&model);
    let buffer = terminal.backend().buffer();
    let image_rows = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .filter(|&y| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).any(|x| {
                buffer
                    .cell((x, y))
                    .map(|cell| cell.symbol() == "▀")
                    .unwrap_or(false)
            })
        })
        .count();

    assert!(
        image_rows > 20,
        "expected image preview to use more than the old 20-row cap, got {image_rows} rows"
    );
}
