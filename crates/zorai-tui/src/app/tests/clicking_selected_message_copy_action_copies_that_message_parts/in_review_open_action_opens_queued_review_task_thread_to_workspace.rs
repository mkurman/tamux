use super::*;
use super::super::unbounded_channel;
use std::sync::mpsc;
#[test]
fn in_review_open_action_opens_queued_review_task_thread() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model.workspace.set_tasks(
        "main".to_string(),
        vec![zorai_protocol::WorkspaceTask {
            id: "wtask-1".to_string(),
            workspace_id: "main".to_string(),
            title: "Review me".to_string(),
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            description: "Do it".to_string(),
            definition_of_done: None,
            priority: zorai_protocol::WorkspacePriority::Low,
            status: zorai_protocol::WorkspaceTaskStatus::InReview,
            sort_order: 1,
            reporter: zorai_protocol::WorkspaceActor::User,
            assignee: Some(zorai_protocol::WorkspaceActor::Agent("dola".to_string())),
            reviewer: Some(zorai_protocol::WorkspaceActor::Agent("swarog".to_string())),
            thread_id: Some("assignee-thread".to_string()),
            goal_run_id: None,
            runtime_history: Vec::new(),
            created_at: 1,
            updated_at: 1,
            started_at: None,
            completed_at: None,
            deleted_at: None,
            last_notice_id: None,
        }],
    );
    model
        .workspace
        .set_notices(vec![zorai_protocol::WorkspaceNotice {
        id: "notice-1".to_string(),
        workspace_id: "main".to_string(),
        task_id: "wtask-1".to_string(),
        notice_type: "review_requested".to_string(),
        message:
            "Workspace task review requested from agent:swarog; queued review task review-task-1"
                .to_string(),
        actor: Some(zorai_protocol::WorkspaceActor::Agent("swarog".to_string())),
        created_at: 2,
    }]);
    model
        .tasks
        .reduce(task::TaskAction::TaskUpdate(task::AgentTask {
            id: "review-task-1".to_string(),
            title: "Review".to_string(),
            thread_id: Some("review-thread-1".to_string()),
            ..Default::default()
        }));

    model.activate_workspace_task_action(
        "wtask-1".to_string(),
        zorai_protocol::WorkspaceTaskStatus::InReview,
        widgets::workspace_board::WorkspaceBoardAction::OpenRuntime,
    );

    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::RequestThread { thread_id, .. }) => {
                assert_eq!(thread_id, "review-thread-1");
                break;
            }
            other => panic!("expected review thread request, got {other:?}"),
        }
    }
}

#[test]
fn in_review_open_action_does_not_open_stale_internal_dm_review_thread() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model.workspace.set_tasks(
        "main".to_string(),
        vec![zorai_protocol::WorkspaceTask {
            id: "wtask-1".to_string(),
            workspace_id: "main".to_string(),
            title: "Review me".to_string(),
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            description: "Do it".to_string(),
            definition_of_done: None,
            priority: zorai_protocol::WorkspacePriority::Low,
            status: zorai_protocol::WorkspaceTaskStatus::InReview,
            sort_order: 1,
            reporter: zorai_protocol::WorkspaceActor::User,
            assignee: Some(zorai_protocol::WorkspaceActor::Agent("dola".to_string())),
            reviewer: Some(zorai_protocol::WorkspaceActor::Agent("weles".to_string())),
            thread_id: Some("assignee-thread".to_string()),
            goal_run_id: None,
            runtime_history: Vec::new(),
            created_at: 1,
            updated_at: 1,
            started_at: None,
            completed_at: None,
            deleted_at: None,
            last_notice_id: None,
        }],
    );
    model
        .workspace
        .set_notices(vec![zorai_protocol::WorkspaceNotice {
            id: "notice-1".to_string(),
            workspace_id: "main".to_string(),
            task_id: "wtask-1".to_string(),
            notice_type: "review_requested".to_string(),
            message:
                "Workspace task review requested from agent:weles; queued review task review-task-1"
                    .to_string(),
            actor: Some(zorai_protocol::WorkspaceActor::Agent("weles".to_string())),
            created_at: 2,
        }]);
    model
        .tasks
        .reduce(task::TaskAction::TaskUpdate(task::AgentTask {
            id: "review-task-1".to_string(),
            title: "Review".to_string(),
            thread_id: Some("dm:swarog:weles".to_string()),
            ..Default::default()
        }));

    model.activate_workspace_task_action(
        "wtask-1".to_string(),
        zorai_protocol::WorkspaceTaskStatus::InReview,
        widgets::workspace_board::WorkspaceBoardAction::OpenRuntime,
    );

    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::ListTasks) => break,
            Ok(DaemonCommand::RequestThread { thread_id, .. }) => {
                panic!("must not open stale internal review thread {thread_id}")
            }
            other => panic!("expected task refresh, got {other:?}"),
        }
    }
}

#[test]
fn in_review_run_action_uses_runtime_history_reviewer_task_when_notice_is_missing() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    let mut task = workspace_task_for_board(
        "wtask-1",
        zorai_protocol::WorkspaceTaskStatus::InReview,
        Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
    );
    task.reviewer = Some(zorai_protocol::WorkspaceActor::Subagent("qa".to_string()));
    task.runtime_history = vec![zorai_protocol::WorkspaceTaskRuntimeHistoryEntry {
        task_type: zorai_protocol::WorkspaceTaskType::Thread,
        thread_id: None,
        goal_run_id: None,
        agent_task_id: Some("review-task-runtime".to_string()),
        source: Some("workspace_review".to_string()),
        title: Some("Review workspace task".to_string()),
        review_path: None,
        review_feedback: None,
        archived_at: 9,
    }];
    model.workspace.set_tasks("main".to_string(), vec![task]);
    model
        .tasks
        .reduce(task::TaskAction::TaskUpdate(task::AgentTask {
            id: "review-task-runtime".to_string(),
            title: "Review".to_string(),
            thread_id: Some("review-thread-runtime".to_string()),
            ..Default::default()
        }));

    model.activate_workspace_task_action(
        "wtask-1".to_string(),
        zorai_protocol::WorkspaceTaskStatus::InReview,
        widgets::workspace_board::WorkspaceBoardAction::Run,
    );

    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::RequestThread { thread_id, .. }) => {
                assert_eq!(thread_id, "review-thread-runtime");
                break;
            }
            other => panic!("expected review thread request, got {other:?}"),
        }
    }
}

#[test]
fn todo_run_action_moves_task_to_in_progress_before_daemon_echo() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    model.workspace.set_tasks(
        "main".to_string(),
        vec![workspace_task_for_board(
            "wtask-run",
            zorai_protocol::WorkspaceTaskStatus::Todo,
            Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
        )],
    );

    model.activate_workspace_task_action(
        "wtask-run".to_string(),
        zorai_protocol::WorkspaceTaskStatus::Todo,
        widgets::workspace_board::WorkspaceBoardAction::Run,
    );

    match cmd_rx.try_recv() {
        Ok(DaemonCommand::RunWorkspaceTask(task_id)) => {
            assert_eq!(task_id, "wtask-run");
        }
        other => panic!("expected run command, got {other:?}"),
    }
    assert_eq!(
        model
            .workspace
            .task_by_id("wtask-run")
            .map(|task| &task.status),
        Some(&zorai_protocol::WorkspaceTaskStatus::InProgress)
    );
    let in_progress_column = model
        .workspace
        .projection()
        .columns
        .iter()
        .find(|column| column.status == zorai_protocol::WorkspaceTaskStatus::InProgress)
        .expect("in-progress column");
    assert!(in_progress_column
        .tasks
        .iter()
        .any(|task| task.id == "wtask-run"));
}

#[test]
fn workspace_history_action_opens_previous_runtime_newest_first() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.main_pane_view = MainPaneView::Workspace;
    let mut task = workspace_task_for_board(
        "wtask-1",
        zorai_protocol::WorkspaceTaskStatus::InProgress,
        Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
    );
    task.thread_id = Some("workspace-thread:active".to_string());
    task.runtime_history = vec![
        zorai_protocol::WorkspaceTaskRuntimeHistoryEntry {
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            thread_id: Some("workspace-thread:old-2".to_string()),
            goal_run_id: None,
            agent_task_id: None,
            source: Some("workspace_runtime".to_string()),
            title: Some("Older run".to_string()),
            review_path: Some("task-wtask-1/failed-review.md".to_string()),
            review_feedback: Some("Second review failed".to_string()),
            archived_at: 20,
        },
        zorai_protocol::WorkspaceTaskRuntimeHistoryEntry {
            task_type: zorai_protocol::WorkspaceTaskType::Thread,
            thread_id: Some("workspace-thread:old-1".to_string()),
            goal_run_id: None,
            agent_task_id: None,
            source: Some("workspace_runtime".to_string()),
            title: Some("Oldest run".to_string()),
            review_path: Some("task-wtask-1/failed-review.md".to_string()),
            review_feedback: Some("First review failed".to_string()),
            archived_at: 10,
        },
    ];
    model.workspace.set_tasks("main".to_string(), vec![task]);

    model.activate_workspace_task_action(
        "wtask-1".to_string(),
        zorai_protocol::WorkspaceTaskStatus::InProgress,
        widgets::workspace_board::WorkspaceBoardAction::History,
    );

    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::WorkspaceTaskHistory)
    );
    let body = model.workspace_history_modal_body();
    assert!(body.find("active").unwrap() < body.find("old-2").unwrap());
    assert!(body.find("old-2").unwrap() < body.find("old-1").unwrap());

    model.modal.reduce(modal::ModalAction::Navigate(1));
    model.submit_workspace_history_modal();
    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::RequestThread { thread_id, .. }) => {
                assert_eq!(thread_id, "workspace-thread:old-2");
                break;
            }
            other => panic!("expected historical thread request, got {other:?}"),
        }
    }
}

#[test]
fn workspace_history_button_click_opens_modal() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, _cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    model.width = 140;
    model.height = 40;
    model.show_sidebar_override = Some(false);
    model.main_pane_view = MainPaneView::Workspace;
    model.focus = FocusArea::Chat;
    let mut task = workspace_task_for_board(
        "wtask-1",
        zorai_protocol::WorkspaceTaskStatus::InProgress,
        Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
    );
    task.runtime_history = vec![zorai_protocol::WorkspaceTaskRuntimeHistoryEntry {
        task_type: zorai_protocol::WorkspaceTaskType::Thread,
        thread_id: Some("workspace-thread:old".to_string()),
        goal_run_id: None,
        agent_task_id: None,
        source: Some("workspace_runtime".to_string()),
        title: Some("Old run".to_string()),
        review_path: Some("task-wtask-1/failed-review.md".to_string()),
        review_feedback: Some("Needs one more check".to_string()),
        archived_at: 10,
    }];
    model.workspace.set_tasks("main".to_string(), vec![task]);
    model
        .workspace_expanded_task_ids
        .insert("wtask-1".to_string());

    let click = workspace_hit_position(&model, |target| {
        matches!(
            target,
            widgets::workspace_board::WorkspaceBoardHitTarget::Action {
                task_id,
                action: widgets::workspace_board::WorkspaceBoardAction::History,
                ..
            } if task_id == "wtask-1"
        )
    });

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::WorkspaceTaskHistory)
    );
    assert!(model
        .workspace_history_modal_body()
        .contains("workspace-thread:old"));
}

#[test]
fn workspace_history_button_opens_empty_state_for_legacy_tasks() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, _cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    let mut task = workspace_task_for_board(
        "wtask-legacy",
        zorai_protocol::WorkspaceTaskStatus::InProgress,
        Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
    );
    task.thread_id = None;
    task.goal_run_id = None;
    model.workspace.set_tasks("main".to_string(), vec![task]);

    model.activate_workspace_task_action(
        "wtask-legacy".to_string(),
        zorai_protocol::WorkspaceTaskStatus::InProgress,
        widgets::workspace_board::WorkspaceBoardAction::History,
    );

    assert_eq!(
        model.modal.top(),
        Some(crate::state::modal::ModalKind::WorkspaceTaskHistory)
    );
    assert!(model
        .workspace_history_modal_body()
        .contains("No previous thread or goal runs"));
}

#[test]
fn workspace_history_modal_uses_active_runtime_when_history_missing() {
    let (_daemon_tx, daemon_rx) = mpsc::channel();
    let (cmd_tx, mut cmd_rx) = unbounded_channel();
    let mut model = TuiModel::new(daemon_rx, cmd_tx);
    let mut task = workspace_task_for_board(
        "wtask-legacy",
        zorai_protocol::WorkspaceTaskStatus::InReview,
        Some(zorai_protocol::WorkspaceActor::Agent("svarog".to_string())),
    );
    task.thread_id = Some("workspace-thread:active-legacy".to_string());
    task.runtime_history.clear();
    model.workspace.set_tasks("main".to_string(), vec![task]);

    model.open_workspace_history_modal("wtask-legacy".to_string());

    let body = model.workspace_history_modal_body();
    assert!(body.contains("workspace-thread:active-legacy"), "{body}");
    model.submit_workspace_history_modal();
    loop {
        match cmd_rx.try_recv() {
            Ok(DaemonCommand::DismissConciergeWelcome) => continue,
            Ok(DaemonCommand::RequestThread { thread_id, .. }) => {
                assert_eq!(thread_id, "workspace-thread:active-legacy");
                break;
            }
            other => panic!("expected active runtime thread request, got {other:?}"),
        }
    }
}

pub(super) fn workspace_settings_for_operator(
    operator: zorai_protocol::WorkspaceOperator,
) -> zorai_protocol::WorkspaceSettings {
    zorai_protocol::WorkspaceSettings {
        workspace_id: "main".to_string(),
        workspace_root: None,
        operator,
        repo_monitor_enabled: false,
        repo_monitor_include_dirs: Vec::new(),
        repo_monitor_exclude_dirs: Vec::new(),
        created_at: 1,
        updated_at: 1,
    }
}

pub(super) fn workspace_task_for_board(
    id: &str,
    status: zorai_protocol::WorkspaceTaskStatus,
    assignee: Option<zorai_protocol::WorkspaceActor>,
) -> zorai_protocol::WorkspaceTask {
    zorai_protocol::WorkspaceTask {
        id: id.to_string(),
        workspace_id: "main".to_string(),
        title: id.to_string(),
        task_type: zorai_protocol::WorkspaceTaskType::Thread,
        description: "Description".to_string(),
        definition_of_done: None,
        priority: zorai_protocol::WorkspacePriority::Low,
        status,
        sort_order: 1,
        reporter: zorai_protocol::WorkspaceActor::User,
        assignee,
        reviewer: Some(zorai_protocol::WorkspaceActor::User),
        thread_id: Some(format!("workspace-thread:{id}")),
        goal_run_id: None,
        runtime_history: Vec::new(),
        created_at: 1,
        updated_at: 1,
        started_at: None,
        completed_at: None,
        deleted_at: None,
        last_notice_id: None,
    }
}

pub(super) fn workspace_hit_position(
    model: &TuiModel,
    matches_target: impl Fn(widgets::workspace_board::WorkspaceBoardHitTarget) -> bool,
) -> Position {
    let chat_area = model.pane_layout().chat;
    (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let position = Position::new(column, row);
                widgets::workspace_board::hit_test_with_scroll(
                    chat_area,
                    &model.workspace,
                    &model.workspace_expanded_task_ids,
                    &model.workspace_board_scroll,
                    position,
                )
                .filter(|target| matches_target(target.clone()))
                .map(|_| position)
            })
        })
        .expect("workspace board target should be visible")
}
