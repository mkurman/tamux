use super::*;
use super::super::unbounded_channel;
use std::sync::mpsc;
#[test]
fn clicking_repo_backed_chat_file_chip_requests_git_diff() {
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

    let repo_root = "/home/mkurman/gitlab/it/cmux-next";
    let repo_path = format!("{repo_root}/README.md");
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Tool,
            tool_name: Some("write_file".to_string()),
            tool_arguments: Some(serde_json::json!({ "path": repo_path }).to_string()),
            tool_status: Some("done".to_string()),
            content: "updated".to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let chip_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::ToolFilePath { message_index: 0 })
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("tool row should expose a clickable repo-backed file chip");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RequestGitDiff {
            repo_path,
            file_path,
        }) => {
            assert_eq!(repo_path, repo_root);
            assert_eq!(file_path.as_deref(), Some("README.md"));
        }
        other => panic!("expected git diff request, got {:?}", other),
    }
    assert!(matches!(model.main_pane_view, MainPaneView::FilePreview(_)));
}

#[test]
fn clicking_apply_patch_file_chip_requests_git_diff() {
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

    let repo_path = "/home/mkurman/gitlab/it/cmux-next/crates/zorai-daemon/src/agent/gateway_loop/message_flow.rs";
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Tool,
            tool_name: Some("apply_patch".to_string()),
            tool_arguments: Some(
                serde_json::json!({
                    "input": format!(
                        "*** Begin Patch\n*** Update File: {repo_path}\n@@\n-old\n+new\n*** End Patch"
                    ),
                })
                .to_string(),
            ),
            tool_status: Some("done".to_string()),
            content: "diff --git a/message_flow.rs b/message_flow.rs\n-old\n+new".to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let chip_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::ToolFilePath { message_index: 0 })
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("apply_patch tool row should expose a clickable file chip");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RequestGitDiff {
            repo_path,
            file_path,
        }) => {
            assert_eq!(repo_path, "/home/mkurman/gitlab/it/cmux-next");
            assert_eq!(
                file_path.as_deref(),
                Some("crates/zorai-daemon/src/agent/gateway_loop/message_flow.rs")
            );
        }
        other => panic!("expected git diff request, got {:?}", other),
    }

    match &model.main_pane_view {
        MainPaneView::FilePreview(target) => {
            assert_eq!(target.path, repo_path);
            assert_eq!(
                target.repo_root.as_deref(),
                Some("/home/mkurman/gitlab/it/cmux-next")
            );
            assert_eq!(
                target.repo_relative_path.as_deref(),
                Some("crates/zorai-daemon/src/agent/gateway_loop/message_flow.rs")
            );
        }
        other => panic!("expected file preview pane, got {:?}", other),
    }
}

#[test]
fn clicking_repo_backed_read_file_chip_requests_plain_preview() {
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

    let repo_path = "/home/mkurman/gitlab/it/cmux-next/crates/zorai-daemon/src/agent/agent_loop/send_message/setup.rs";
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Tool,
            tool_name: Some("read_file".to_string()),
            tool_arguments: Some(serde_json::json!({ "path": repo_path }).to_string()),
            tool_status: Some("done".to_string()),
            content: "previewed".to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let chip_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::ToolFilePath { message_index: 0 })
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("tool row should expose a clickable repo-backed read_file chip");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RequestFilePreview { path, max_bytes }) => {
            assert_eq!(path, repo_path);
            assert_eq!(max_bytes, Some(65_536));
        }
        other => panic!("expected file preview request, got {:?}", other),
    }

    match &model.main_pane_view {
        MainPaneView::FilePreview(target) => {
            assert_eq!(target.path, repo_path);
            assert!(target.repo_root.is_none());
            assert!(target.repo_relative_path.is_none());
        }
        other => panic!("expected file preview pane, got {:?}", other),
    }
}

#[test]
fn clicking_read_skill_chip_requests_plain_preview() {
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

    let skills_root = "/home/mkurman/gitlab/it/cmux-next";
    let relative_path = "skills/development/superpowers/systematic-debugging/SKILL.md";
    let full_path = format!("{skills_root}/{relative_path}");
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Tool,
            tool_name: Some("read_skill".to_string()),
            tool_arguments: Some(
                serde_json::json!({ "skill": "systematic-debugging" }).to_string(),
            ),
            tool_status: Some("done".to_string()),
            content: serde_json::json!({
                "skills_root": skills_root,
                "path": relative_path,
                "content": "previewed",
                "truncated": false,
                "total_lines": 12
            })
            .to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let chip_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::ToolFilePath { message_index: 0 })
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("tool row should expose a clickable read_skill chip");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RequestFilePreview { path, max_bytes }) => {
            assert_eq!(path, full_path);
            assert_eq!(max_bytes, Some(65_536));
        }
        other => panic!("expected file preview request, got {:?}", other),
    }

    match &model.main_pane_view {
        MainPaneView::FilePreview(target) => {
            assert_eq!(target.path, full_path);
            assert!(target.repo_root.is_none());
            assert!(target.repo_relative_path.is_none());
        }
        other => panic!("expected file preview pane, got {:?}", other),
    }
}

#[test]
fn clicking_tool_file_path_metadata_requests_plain_preview() {
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

    let preview_path = "/home/mkurman/gitlab/it/cmux-next/.zorai/.cache/tools/thread-thread-1/bash_command-1700000123.txt";
    model.chat.reduce(chat::ChatAction::AppendMessage {
        thread_id: "thread-1".to_string(),
        message: chat::AgentMessage {
            role: chat::MessageRole::Tool,
            tool_name: Some("bash_command".to_string()),
            tool_status: Some("done".to_string()),
            tool_output_preview_path: Some(preview_path.to_string()),
            content: "Tool result saved to preview file\n- tool: bash_command".to_string(),
            ..Default::default()
        },
    });

    let input_start_row = model.height.saturating_sub(model.input_height() + 1);
    let chat_area = Rect::new(0, 3, model.width, input_start_row.saturating_sub(3));
    let chip_pos = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::chat::hit_test(
                    chat_area,
                    &model.chat,
                    &model.theme,
                    model.tick_counter,
                    pos,
                ) == Some(chat::ChatHitTarget::ToolFilePath { message_index: 0 })
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("preview-backed tool row should expose a clickable file chip");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: chip_pos.x,
        row: chip_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RequestFilePreview { path, max_bytes }) => {
            assert_eq!(path, preview_path);
            assert_eq!(max_bytes, Some(65_536));
        }
        other => panic!("expected plain file preview request, got {:?}", other),
    }

    match &model.main_pane_view {
        MainPaneView::FilePreview(target) => {
            assert_eq!(target.path, preview_path);
            assert!(target.repo_root.is_none());
            assert!(target.repo_relative_path.is_none());
        }
        other => panic!("expected file preview pane, got {:?}", other),
    }
}
