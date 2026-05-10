use super::*;
use super::super::build_model;
fn spawned_thread_navigation_task(
    id: &str,
    title: &str,
    created_at: u64,
    thread_id: Option<&str>,
    parent_task_id: Option<&str>,
    parent_thread_id: Option<&str>,
    status: Option<task::TaskStatus>,
) -> task::AgentTask {
    task::AgentTask {
        id: id.to_string(),
        title: title.to_string(),
        created_at,
        thread_id: thread_id.map(str::to_string),
        parent_task_id: parent_task_id.map(str::to_string),
        parent_thread_id: parent_thread_id.map(str::to_string),
        status,
        ..Default::default()
    }
}

fn seed_spawned_thread_navigation_model_with_loaded_child(loaded_child: bool) -> TuiModel {
    let mut model = build_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    if loaded_child {
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: "thread-child".to_string(),
            title: "Child".to_string(),
        });
    }
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-other".to_string(),
        title: "Other".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-root".to_string()));
    model.tasks.reduce(task::TaskAction::TaskListReceived(vec![
        spawned_thread_navigation_task(
            "root-task",
            "Root worker",
            10,
            Some("thread-root"),
            None,
            None,
            Some(task::TaskStatus::InProgress),
        ),
        spawned_thread_navigation_task(
            "child-task",
            "Child worker",
            20,
            Some("thread-child"),
            Some("root-task"),
            Some("thread-root"),
            Some(task::TaskStatus::InProgress),
        ),
    ]));
    model
}

fn seed_spawned_thread_navigation_model() -> TuiModel {
    seed_spawned_thread_navigation_model_with_loaded_child(true)
}

fn seed_spawned_thread_navigation_model_with_disabled_row() -> TuiModel {
    let mut model = build_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-other".to_string(),
        title: "Other".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-root".to_string()));
    model.tasks.reduce(task::TaskAction::TaskListReceived(vec![
        spawned_thread_navigation_task(
            "root-task",
            "Root worker",
            30,
            Some("thread-root"),
            None,
            None,
            Some(task::TaskStatus::InProgress),
        ),
        spawned_thread_navigation_task(
            "other-task",
            "Other worker",
            20,
            Some("thread-other"),
            None,
            Some("thread-root"),
            Some(task::TaskStatus::InProgress),
        ),
        spawned_thread_navigation_task(
            "disabled-task",
            "Dormant worker",
            10,
            None,
            Some("root-task"),
            Some("thread-root"),
            Some(task::TaskStatus::Completed),
        ),
    ]));
    model
}

#[test]
fn spawned_thread_navigation_enter_switches_to_child_thread_and_pushes_history() {
    let mut model = seed_spawned_thread_navigation_model();
    model.focus = FocusArea::Sidebar;
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Spawned));
    model.sidebar.navigate(1, model.sidebar_item_count());

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()]
    );
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.focus, FocusArea::Chat);
}

#[test]
fn spawned_thread_navigation_keyboard_tab_switch_primes_first_openable_child() {
    let mut model = seed_spawned_thread_navigation_model();
    model.focus = FocusArea::Sidebar;

    let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.sidebar.active_tab(), SidebarTab::Files);

    let handled = model.handle_key(KeyCode::Right, KeyModifiers::NONE);
    assert!(!handled);
    assert_eq!(model.sidebar.active_tab(), SidebarTab::Spawned);

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()]
    );
}

#[test]
fn spawned_thread_navigation_enter_opens_unloaded_child_thread() {
    let mut model = seed_spawned_thread_navigation_model_with_loaded_child(false);
    model.focus = FocusArea::Sidebar;
    model.activate_sidebar_tab(SidebarTab::Spawned);

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()]
    );
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(model.thread_loading_id.as_deref(), Some("thread-child"));
}

#[test]
fn spawned_thread_navigation_preserves_pending_child_open_across_thread_list_refresh() {
    let mut model = seed_spawned_thread_navigation_model_with_loaded_child(false);
    model.focus = FocusArea::Sidebar;
    model.activate_sidebar_tab(SidebarTab::Spawned);

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
    assert_eq!(model.thread_loading_id.as_deref(), Some("thread-child"));

    model.handle_thread_list_event(vec![
        crate::wire::AgentThread {
            id: "thread-root".to_string(),
            title: "Root".to_string(),
            ..Default::default()
        },
        crate::wire::AgentThread {
            id: "thread-other".to_string(),
            title: "Other".to_string(),
            ..Default::default()
        },
    ]);

    assert_eq!(
        model.chat.active_thread_id(),
        Some("thread-child"),
        "thread-list refresh should not clear a child thread that is still loading"
    );
    assert_eq!(
        model.thread_loading_id.as_deref(),
        Some("thread-child"),
        "loading state should survive until the requested child thread arrives"
    );
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()]
    );
}

#[test]
fn spawned_thread_navigation_enter_on_anchor_opens_first_openable_child() {
    let mut model = seed_spawned_thread_navigation_model_with_loaded_child(false);
    model.focus = FocusArea::Sidebar;
    model.set_mission_control_return_targets(
        Some(SidebarItemTarget::GoalRun {
            goal_run_id: "goal-1".to_string(),
            step_id: Some("step-1".to_string()),
        }),
        None,
    );
    model.activate_sidebar_tab(SidebarTab::Spawned);
    model.sidebar.select(0, model.sidebar_item_count());

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()]
    );
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(model.thread_loading_id.as_deref(), Some("thread-child"));
    assert!(matches!(
        model.mission_control_return_to_goal_target(),
        Some(SidebarItemTarget::GoalRun {
            ref goal_run_id,
            step_id: Some(ref step_id),
        }) if goal_run_id == "goal-1" && step_id == "step-1"
    ));
}

#[test]
fn spawned_thread_navigation_enter_on_disabled_row_does_nothing() {
    let mut model = seed_spawned_thread_navigation_model_with_disabled_row();
    model.focus = FocusArea::Sidebar;
    model.activate_sidebar_tab(SidebarTab::Spawned);
    let disabled_index = (0..model.sidebar_item_count())
        .find(|index| {
            model.sidebar.select(*index, model.sidebar_item_count());
            widgets::sidebar::selected_spawned_thread_id(
                &model.tasks,
                &model.sidebar,
                model.chat.active_thread_id(),
            )
            .is_none()
        })
        .expect("disabled row should map to a spawned sidebar index");
    model
        .sidebar
        .select(disabled_index, model.sidebar_item_count());

    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-root"));
    assert!(
        model.chat.thread_history_stack().is_empty(),
        "disabled rows should not mutate spawned-thread history"
    );
    assert_eq!(model.thread_loading_id, None);
}

#[test]
fn spawned_thread_navigation_mouse_click_opens_unloaded_child_thread() {
    let mut model = seed_spawned_thread_navigation_model_with_loaded_child(false);
    let sidebar_area = model
        .pane_layout()
        .sidebar
        .expect("default layout should include a sidebar");
    model.activate_sidebar_tab(SidebarTab::Spawned);

    let child_pos = (sidebar_area.y..sidebar_area.y.saturating_add(sidebar_area.height))
        .find_map(|row| {
            (sidebar_area.x..sidebar_area.x.saturating_add(sidebar_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::sidebar::hit_test(
                    sidebar_area,
                    &model.chat,
                    &model.sidebar,
                    &model.tasks,
                    model.chat.active_thread_id(),
                    pos,
                ) == Some(widgets::sidebar::SidebarHitTarget::Spawned(1))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("spawned sidebar should expose a clickable child row");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: child_pos.x,
        row: child_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()]
    );
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(model.thread_loading_id.as_deref(), Some("thread-child"));
}

#[test]
fn spawned_thread_navigation_mouse_click_anchor_opens_first_openable_child() {
    let mut model = seed_spawned_thread_navigation_model_with_loaded_child(false);
    let sidebar_area = model
        .pane_layout()
        .sidebar
        .expect("default layout should include a sidebar");
    model.activate_sidebar_tab(SidebarTab::Spawned);

    let anchor_pos = (sidebar_area.y..sidebar_area.y.saturating_add(sidebar_area.height))
        .find_map(|row| {
            (sidebar_area.x..sidebar_area.x.saturating_add(sidebar_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                if widgets::sidebar::hit_test(
                    sidebar_area,
                    &model.chat,
                    &model.sidebar,
                    &model.tasks,
                    model.chat.active_thread_id(),
                    pos,
                ) == Some(widgets::sidebar::SidebarHitTarget::Spawned(0))
                {
                    Some(pos)
                } else {
                    None
                }
            })
        })
        .expect("spawned sidebar should expose a clickable anchor row");

    model.handle_mouse(MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: anchor_pos.x,
        row: anchor_pos.y,
        modifiers: KeyModifiers::NONE,
    });

    assert_eq!(model.chat.active_thread_id(), Some("thread-child"));
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()]
    );
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.focus, FocusArea::Chat);
    assert_eq!(model.thread_loading_id.as_deref(), Some("thread-child"));
}

#[test]
fn spawned_thread_navigation_back_action_pops_to_previous_thread() {
    let mut model = seed_spawned_thread_navigation_model();
    model.focus = FocusArea::Sidebar;
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Spawned));
    model.sidebar.navigate(1, model.sidebar_item_count());
    model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    model.focus = FocusArea::Sidebar;

    let handled = model.handle_key(KeyCode::Backspace, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-root"));
    assert!(
        model.chat.thread_history_stack().is_empty(),
        "back navigation should pop the spawned thread stack"
    );
}

#[test]
fn spawned_thread_navigation_back_is_disabled_when_stack_is_empty() {
    let mut model = seed_spawned_thread_navigation_model();
    model.focus = FocusArea::Sidebar;
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Spawned));

    let handled = model.handle_key(KeyCode::Backspace, KeyModifiers::NONE);

    assert!(!handled);
    assert_eq!(model.chat.active_thread_id(), Some("thread-root"));
    assert!(
        model.chat.thread_history_stack().is_empty(),
        "empty stack should stay unchanged"
    );
}

#[test]
fn spawned_thread_navigation_ordinary_thread_switches_do_not_mutate_stack() {
    let mut model = seed_spawned_thread_navigation_model();
    model.focus = FocusArea::Sidebar;
    model
        .sidebar
        .reduce(SidebarAction::SwitchTab(SidebarTab::Spawned));
    model.sidebar.navigate(1, model.sidebar_item_count());
    model.handle_key(KeyCode::Enter, KeyModifiers::NONE);

    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-other".to_string()));

    assert_eq!(model.chat.active_thread_id(), Some("thread-other"));
    assert_eq!(
        model.chat.thread_history_stack(),
        &["thread-root".to_string()],
        "direct thread selection should preserve existing spawned thread history"
    );
}
