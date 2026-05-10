use super::*;
use crate::test_support::{env_var_lock, EnvVarGuard, ZORAI_DATA_DIR_ENV};
use crate::app::tests::goal_sidebar_tab_cycling_stays_to_collaboration_mouse_clicks_select_rows::goal_sidebar_tab_cycling_stays_mod::*;
use super::super::rendered_chat_area;
use ratatui::backend::TestBackend;
#[test]
fn goal_sidebar_row_selection_clamps_per_active_tab() {
    let mut state = GoalSidebarState::new();

    state.select_row(4, 3);
    assert_eq!(state.selected_row(), 2);

    state.cycle_tab_right();
    state.select_row(1, 0);
    assert_eq!(state.selected_row(), 0);

    state.cycle_tab_right();
    state.select_row(3, 2);
    assert_eq!(state.selected_row(), 1);

    state.cycle_tab_right();
    state.select_row(7, 1);
    assert_eq!(state.selected_row(), 0);
}

#[test]
fn goal_run_render_uses_full_workspace_without_legacy_goal_sidebar_tabs() {
    let mut model = goal_sidebar_model();

    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("goal run render should succeed");

    let buffer = terminal.backend().buffer();
    let chat_area = rendered_chat_area(&model);

    let chat_plain = (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .map(|y| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width))
                .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        chat_plain.contains("Plan")
            && chat_plain.contains("Run timeline")
            && chat_plain.contains("Dossier")
            && chat_plain.contains("Files")
            && chat_plain.contains("Prompt"),
        "expected full goal workspace content in main pane, got: {chat_plain}"
    );
    assert!(
        !chat_plain.contains("Steps  Checkpoints  Tasks"),
        "expected legacy goal sidebar labels to be absent, got: {chat_plain}"
    );
    assert!(
        model.pane_layout().sidebar.is_none(),
        "goal workspace should take the full content width"
    );
}

#[test]
fn goal_workspace_keyboard_navigation_uses_plan_state() {
    let mut model = goal_sidebar_model();
    model.focus = FocusArea::Chat;

    let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);
    assert!(!handled);
    assert!(model.goal_workspace.is_step_expanded("step-1"));

    let handled = model.handle_key(KeyCode::Down, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.goal_workspace.selected_plan_row(), 3);
    assert_eq!(
        model.goal_workspace.selected_plan_item(),
        Some(&goal_workspace::GoalPlanSelection::Todo {
            step_id: "step-1".to_string(),
            todo_id: "todo-1".to_string(),
        })
    );

    let handled = model.handle_key(KeyCode::Left, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.goal_workspace.selected_plan_row(), 2);
    assert_eq!(
        model.goal_workspace.selected_plan_item(),
        Some(&goal_workspace::GoalPlanSelection::Step {
            step_id: "step-1".to_string(),
        })
    );
}

#[test]
fn goal_workspace_tab_cycles_between_internal_panes_before_input() {
    let mut model = goal_sidebar_model();
    model.focus = FocusArea::Chat;

    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::Plan
    );

    let handled = model.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::Timeline
    );

    let handled = model.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::Details
    );

    let handled = model.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::CommandBar
    );

    let handled = model.handle_key(KeyCode::Tab, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Input);

    let handled = model.handle_key(KeyCode::BackTab, KeyModifiers::SHIFT);
    assert!(!handled);
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::CommandBar
    );
}

#[test]
fn selected_goal_step_workspace_click_syncs_main_goal_detail_selection() {
    let mut model = goal_sidebar_model();
    let click = find_goal_workspace_hit_position(
        &model,
        widgets::goal_workspace::GoalWorkspaceHitTarget::PlanStep("step-2".to_string()),
    );
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(model.goal_workspace.selected_plan_row(), 3);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun {
            goal_run_id,
            step_id: Some(step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-2"
    ));
}

#[test]
fn goal_workspace_mouse_clicks_focus_timeline_and_details_panes() {
    let mut model = goal_sidebar_model();
    model.focus = FocusArea::Input;
    if let Some(run) = model.tasks.goal_run_by_id_mut("goal-1") {
        run.events = vec![
            task::GoalRunEvent {
                id: "event-1".to_string(),
                message: "first".to_string(),
                timestamp: 10,
                ..Default::default()
            },
            task::GoalRunEvent {
                id: "event-2".to_string(),
                message: "second".to_string(),
                timestamp: 20,
                ..Default::default()
            },
        ];
    }

    let chat_area = rendered_chat_area(&model);
    let (_, timeline, details) = goal_workspace_click_targets(chat_area);

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: timeline.x,
        row: timeline.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: timeline.x,
        row: timeline.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::Timeline
    );

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: details.x,
        row: details.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: details.x,
        row: details.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::Details
    );
}

#[test]
fn goal_workspace_mode_tabs_are_clickable_and_keyboard_focusable() {
    let mut model = goal_sidebar_model();
    model.focus = FocusArea::Chat;
    model
        .goal_workspace
        .set_focused_pane(goal_workspace::GoalWorkspacePane::CommandBar);

    let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.goal_workspace.mode(),
        goal_workspace::GoalWorkspaceMode::Files
    );

    let handled = model.handle_key(KeyCode::Char('4'), KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.goal_workspace.mode(),
        goal_workspace::GoalWorkspaceMode::Usage
    );

    let handled = model.handle_key(KeyCode::Char('5'), KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.goal_workspace.mode(),
        goal_workspace::GoalWorkspaceMode::ActiveAgent
    );

    let handled = model.handle_key(KeyCode::Char('6'), KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.goal_workspace.mode(),
        goal_workspace::GoalWorkspaceMode::Threads
    );

    let handled = model.handle_key(KeyCode::Char('7'), KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.goal_workspace.mode(),
        goal_workspace::GoalWorkspaceMode::NeedsAttention
    );

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::Plan
    );

    let mut model = goal_sidebar_model();
    let click = find_goal_workspace_hit_position(
        &model,
        widgets::goal_workspace::GoalWorkspaceHitTarget::ModeTab(
            goal_workspace::GoalWorkspaceMode::ActiveAgent,
        ),
    );
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(
        model.goal_workspace.focused_pane(),
        goal_workspace::GoalWorkspacePane::CommandBar
    );
    assert_eq!(
        model.goal_workspace.mode(),
        goal_workspace::GoalWorkspaceMode::ActiveAgent
    );
}

#[test]
fn goal_workspace_files_mode_click_opens_file_preview() {
    let _lock = env_var_lock();
    let temp_home = tempfile::tempdir().expect("temp home should exist");
    let _data_dir = EnvVarGuard::set(ZORAI_DATA_DIR_ENV, temp_home.path());

    let goal_root = zorai_protocol::ensure_zorai_data_dir()
        .expect("zorai data dir")
        .join("goals")
        .join("goal-1");
    std::fs::create_dir_all(goal_root.join("inventory/specs"))
        .expect("goal inventory tree should exist");
    let file_path = goal_root.join("goal.md");
    std::fs::write(&file_path, "# Goal\n").expect("goal.md should be written");

    let mut model = goal_sidebar_model();
    model
        .goal_workspace
        .set_mode(goal_workspace::GoalWorkspaceMode::Files);
    let click = find_goal_workspace_hit_position(
        &model,
        widgets::goal_workspace::GoalWorkspaceHitTarget::DetailFile(
            file_path.display().to_string(),
        ),
    );

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });

    assert!(matches!(model.main_pane_view, MainPaneView::FilePreview(_)));
    assert!(model.mission_control_return_to_goal_target().is_some());
    match &model.main_pane_view {
        MainPaneView::FilePreview(target) => {
            assert_eq!(target.path, file_path.display().to_string());
            assert!(target.repo_root.is_none());
        }
        _ => panic!("expected file preview"),
    }
}

#[test]
fn goal_workspace_escape_from_files_mode_preview_restores_goal_run() {
    let _lock = env_var_lock();
    let temp_home = tempfile::tempdir().expect("temp home should exist");
    let _data_dir = EnvVarGuard::set(ZORAI_DATA_DIR_ENV, temp_home.path());

    let goal_root = zorai_protocol::ensure_zorai_data_dir()
        .expect("zorai data dir")
        .join("goals")
        .join("goal-1");
    std::fs::create_dir_all(goal_root.join("inventory/specs"))
        .expect("goal inventory tree should exist");
    let file_path = goal_root.join("goal.md");
    std::fs::write(&file_path, "# Goal\n").expect("goal.md should be written");

    let mut model = goal_sidebar_model();
    model
        .goal_workspace
        .set_mode(goal_workspace::GoalWorkspaceMode::Files);
    let click = find_goal_workspace_hit_position(
        &model,
        widgets::goal_workspace::GoalWorkspaceHitTarget::DetailFile(
            file_path.display().to_string(),
        ),
    );

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });
    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column: click.x,
        row: click.y,
        modifiers: KeyModifiers::NONE,
    });

    let handled = model.handle_key(KeyCode::Esc, KeyModifiers::NONE);

    assert!(!handled);
    assert!(matches!(
        model.main_pane_view,
        MainPaneView::Task(SidebarItemTarget::GoalRun { ref goal_run_id, .. })
            if goal_run_id == "goal-1"
    ));
}

#[test]
fn goal_workspace_goal_mode_restores_old_goal_sections() {
    let mut model = goal_sidebar_model();

    let plain = render_chat_plain(&mut model);

    assert!(plain.contains("Step Actions"), "{plain}");
    assert!(!plain.contains("Controls"), "{plain}");
    assert!(plain.contains("Related Tasks"), "{plain}");
    assert!(plain.contains("Execution Dossier"), "{plain}");
    assert!(plain.contains("Goal Prompt"), "{plain}");
    assert!(plain.contains("Main agent"), "{plain}");
    assert!(plain.contains("Ground the user's background"), "{plain}");
    assert!(!plain.contains("/tmp/plan.md"), "{plain}");
}

#[test]
fn goal_workspace_footer_omits_refresh_button_and_keeps_ctrl_r_rerun() {
    let mut model = goal_sidebar_model();

    let plain = render_chat_plain(&mut model);

    assert!(plain.contains("[Rerun from here] Ctrl+R"), "{plain}");
    assert!(!plain.contains("[Refresh]"), "{plain}");
    assert!(!plain.contains("[Rerun from here] Shift+R"), "{plain}");
}
