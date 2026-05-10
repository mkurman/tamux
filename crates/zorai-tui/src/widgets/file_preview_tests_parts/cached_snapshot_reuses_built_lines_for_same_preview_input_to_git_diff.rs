use super::*;
use crate::app::ChatFilePreviewTarget;
use crate::state::task::TaskState;
use crate::terminal_graphics::TerminalImageProtocol;
use crate::theme::ThemeTokens;
use ratatui::backend::TestBackend;
use ratatui::layout::{Position, Rect};
use ratatui::Terminal;
use std::sync::{Mutex, OnceLock};

fn counter_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn render_preview_plain_text(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    scroll: usize,
) -> String {
    let backend = TestBackend::new(area.width, area.height);
    let mut terminal = Terminal::new(backend).expect("terminal should initialize");

    terminal
        .draw(|frame| {
            render(
                frame,
                area,
                tasks,
                target,
                &ThemeTokens::default(),
                scroll,
                None,
            );
        })
        .expect("file preview render should succeed");

    let buffer = terminal.backend().buffer();
    (area.y..area.y.saturating_add(area.height))
        .map(|y| {
            (area.x..area.x.saturating_add(area.width))
                .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn cached_snapshot_reuses_built_lines_for_same_preview_input() {
    let _lock = counter_test_lock();
    let mut tasks = TaskState::default();
    let path = "/tmp/cache-reuse-snapshot-preview.txt";
    tasks.reduce(crate::state::task::TaskAction::FilePreviewReceived(
        crate::state::task::FilePreview {
            path: path.to_string(),
            content: (1..=500)
                .map(|idx| format!("large preview line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            truncated: false,
            is_text: true,
        },
    ));
    let target = ChatFilePreviewTarget {
        path: path.to_string(),
        repo_root: None,
        repo_relative_path: None,
    };
    let area = Rect::new(0, 0, 80, 20);
    let theme = ThemeTokens::default();
    let mut cache = FilePreviewRenderCache::default();

    reset_build_lines_call_count_for_tests(path);
    let first = snapshot_with_cache(&mut cache, area, &tasks, &target, &theme, 0)
        .expect("first snapshot should build");
    let calls_after_first = build_lines_call_count_for_tests();
    let second = snapshot_with_cache(&mut cache, area, &tasks, &target, &theme, 10)
        .expect("second snapshot should be cached");

    assert_eq!(first.header_lines.len(), second.header_lines.len());
    assert_eq!(first.body_lines.len(), second.body_lines.len());
    assert_eq!(
        build_lines_call_count_for_tests(),
        calls_after_first,
        "same file preview input should reuse cached rendered lines"
    );
}

#[test]
fn repeated_render_reuses_cached_preview_lines() {
    let _lock = counter_test_lock();
    let mut tasks = TaskState::default();
    let path = "/tmp/cache-reuse-render-preview.txt";
    tasks.reduce(crate::state::task::TaskAction::FilePreviewReceived(
        crate::state::task::FilePreview {
            path: path.to_string(),
            content: (1..=500)
                .map(|idx| format!("large preview line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            truncated: false,
            is_text: true,
        },
    ));
    let target = ChatFilePreviewTarget {
        path: path.to_string(),
        repo_root: None,
        repo_relative_path: None,
    };
    let area = Rect::new(0, 0, 80, 20);

    reset_build_lines_call_count_for_tests(path);
    let _ = render_preview_plain_text(area, &tasks, &target, 10);
    let calls_after_first_render = build_lines_call_count_for_tests();
    let _ = render_preview_plain_text(area, &tasks, &target, 30);

    assert_eq!(
        build_lines_call_count_for_tests(),
        calls_after_first_render,
        "render should reuse cached preview lines across scroll positions"
    );
}

#[test]
fn scrolled_file_preview_keeps_close_control_and_path_visible() {
    let mut tasks = TaskState::default();
    tasks.reduce(crate::state::task::TaskAction::FilePreviewReceived(
        crate::state::task::FilePreview {
            path: "/tmp/large-preview.txt".to_string(),
            content: (1..=80)
                .map(|idx| format!("large preview line {idx}"))
                .collect::<Vec<_>>()
                .join("\n"),
            truncated: false,
            is_text: true,
        },
    ));
    let target = ChatFilePreviewTarget {
        path: "/tmp/large-preview.txt".to_string(),
        repo_root: None,
        repo_relative_path: None,
    };

    let rendered = render_preview_plain_text(Rect::new(0, 0, 80, 12), &tasks, &target, 20);

    assert!(
        rendered.contains("Close preview"),
        "close control should remain visible while scrolled"
    );
    assert!(
        rendered.contains("Path: /tmp/large-preview.txt"),
        "file path should remain visible while scrolled"
    );

    assert_eq!(
        hit_test(
            Rect::new(0, 0, 80, 12),
            &tasks,
            &target,
            20,
            Position::new(1, 0),
            &ThemeTokens::default(),
        ),
        Some(FilePreviewHitTarget::ClosePreview),
        "close control should remain clickable while scrolled"
    );
}

#[test]
fn terminal_image_overlay_spec_targets_file_preview_body() {
    crate::terminal_graphics::set_active_protocol_for_tests(TerminalImageProtocol::Kitty);

    let target = ChatFilePreviewTarget {
        path: "/tmp/demo.png".to_string(),
        repo_root: None,
        repo_relative_path: None,
    };
    let spec = terminal_image_overlay_spec(
        Rect::new(0, 0, 80, 30),
        &TaskState::default(),
        &target,
        &ThemeTokens::default(),
        0,
    )
    .expect("expected file preview image overlay spec");

    assert_eq!(spec.column, 0);
    assert_eq!(spec.row, 6);
    assert_eq!(spec.cols, 80);
    assert_eq!(spec.rows, 24);

    crate::terminal_graphics::set_active_protocol_for_tests(TerminalImageProtocol::None);
}

#[test]
fn git_diff_preview_colors_added_and_removed_lines() {
    let mut tasks = TaskState::default();
    tasks.reduce(crate::state::task::TaskAction::GitDiffReceived {
        repo_path: "/repo".to_string(),
        file_path: Some("src/demo.rs".to_string()),
        diff: [
            "diff --git a/src/demo.rs b/src/demo.rs",
            "index 1111111..2222222 100644",
            "--- a/src/demo.rs",
            "+++ b/src/demo.rs",
            "@@ -1,2 +1,2 @@",
            "-let before = 1;",
            "+let after = 2;",
        ]
        .join("\n"),
    });
    let target = ChatFilePreviewTarget {
        path: "/repo/src/demo.rs".to_string(),
        repo_root: Some("/repo".to_string()),
        repo_relative_path: Some("src/demo.rs".to_string()),
    };
    let theme = ThemeTokens::default();
    let lines = build_lines(Rect::new(0, 0, 80, 20), &tasks, &target, &theme, 0);

    let removed = lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .find(|span| span.content.as_ref() == "-let before = 1;")
        .expect("removed line should render as its own styled span");
    let added = lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .find(|span| span.content.as_ref() == "+let after = 2;")
        .expect("added line should render as its own styled span");

    assert_eq!(removed.style, theme.accent_danger);
    assert_eq!(added.style, theme.accent_success);
}

#[test]
fn selected_text_can_extract_header_path() {
    let tasks = TaskState::default();
    let target = ChatFilePreviewTarget {
        path: "/repo/src/demo.rs".to_string(),
        repo_root: None,
        repo_relative_path: None,
    };
    let text = selected_text(
        Rect::new(0, 0, 80, 20),
        &tasks,
        &target,
        &ThemeTokens::default(),
        0,
        crate::widgets::chat::SelectionPoint { row: 2, col: 6 },
        crate::widgets::chat::SelectionPoint { row: 2, col: 23 },
    )
    .expect("header path should be selectable");

    assert_eq!(text, "/repo/src/demo.rs");
}

#[test]
fn rust_preview_applies_cached_syntax_styles_without_diff_renderer() {
    let mut tasks = TaskState::default();
    tasks.reduce(crate::state::task::TaskAction::FilePreviewReceived(
        crate::state::task::FilePreview {
            path: "/repo/src/demo.rs".to_string(),
            content: [
                "fn main() {",
                "    let value = 42;",
                "    println!(\"{value}\");",
                "}",
            ]
            .join("\n"),
            truncated: false,
            is_text: true,
        },
    ));
    let target = ChatFilePreviewTarget {
        path: "/repo/src/demo.rs".to_string(),
        repo_root: None,
        repo_relative_path: None,
    };
    let theme = ThemeTokens::default();
    let lines = build_lines(Rect::new(0, 0, 80, 20), &tasks, &target, &theme, 0);
    let body_spans = lines
        .iter()
        .skip(FILE_PREVIEW_HEADER_LINES as usize)
        .flat_map(|line| line.spans.iter())
        .collect::<Vec<_>>();

    assert!(
        body_spans
            .iter()
            .any(|span| span.content.as_ref() == "fn" && span.style == theme.accent_primary),
        "expected Rust keyword to use syntax color"
    );
    assert!(
        body_spans
            .iter()
            .any(|span| span.content.as_ref() == "42" && span.style == theme.accent_secondary),
        "expected number literal to use syntax color"
    );
    assert!(
        body_spans.iter().any(|span| {
            span.content.as_ref() == "\"{value}\"" && span.style == theme.accent_success
        }),
        "expected string literal to use syntax color"
    );
}

#[test]
fn syntax_highlighted_preview_preserves_indentation() {
    let mut tasks = TaskState::default();
    tasks.reduce(crate::state::task::TaskAction::FilePreviewReceived(
        crate::state::task::FilePreview {
            path: "/repo/evaluate_convgpt.py".to_string(),
            content: [
                "def query_model():",
                "    payload = {",
                "        \"model\": \"convgpt\",",
                "    }",
            ]
            .join("\n"),
            truncated: false,
            is_text: true,
        },
    ));
    let target = ChatFilePreviewTarget {
        path: "/repo/evaluate_convgpt.py".to_string(),
        repo_root: None,
        repo_relative_path: None,
    };
    let lines = build_lines(
        Rect::new(0, 0, 80, 20),
        &tasks,
        &target,
        &ThemeTokens::default(),
        0,
    );
    let first_indented = line_plain_text(&lines[FILE_PREVIEW_HEADER_LINES as usize + 1]);
    let nested = line_plain_text(&lines[FILE_PREVIEW_HEADER_LINES as usize + 2]);

    assert_eq!(first_indented, "    payload = {");
    assert_eq!(nested, "        \"model\": \"convgpt\",");
}
