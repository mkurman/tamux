use super::super::*;
use super::sidebar_handles_empty_state_to_spawned_sidebar_tabs_include_spawned::*;
use crate::state::chat::{ChatAction, ChatState};
use crate::state::sidebar::SidebarState;
use crate::state::task::{TaskAction, TaskState};
use crate::theme::ThemeTokens;
use ratatui::backend::TestBackend;
use ratatui::layout::{Position, Rect};
use ratatui::Terminal;

#[test]
fn spawned_sidebar_renders_nested_rows_under_active_thread() {
    let mut sidebar = SidebarState::new();
    sidebar.reduce(crate::state::sidebar::SidebarAction::SwitchTab(
        crate::state::sidebar::SidebarTab::Spawned,
    ));
    let mut tasks = TaskState::new();
    tasks.reduce(TaskAction::TaskListReceived(vec![
        spawned_sidebar_task(
            "root-task",
            "Root worker",
            10,
            Some("thread-root"),
            None,
            None,
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "child-task",
            "Child worker",
            20,
            Some("thread-child"),
            Some("root-task"),
            Some("thread-root"),
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "grandchild-task",
            "Grandchild worker",
            30,
            Some("thread-grandchild"),
            Some("child-task"),
            Some("thread-child"),
            Some(crate::state::task::TaskStatus::Completed),
        ),
    ]));
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-child".to_string(),
        title: "Child".to_string(),
    });
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-grandchild".to_string(),
        title: "Grandchild".to_string(),
    });
    chat.reduce(ChatAction::SelectThread("thread-root".to_string()));

    let area = Rect::new(0, 0, 44, 10);
    let backend = TestBackend::new(area.width, area.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| {
            render(
                frame,
                area,
                &chat,
                &sidebar,
                &tasks,
                Some("thread-root"),
                &ThemeTokens::default(),
                true,
                &[],
                &crate::state::tier::TierState::default(),
                None,
                None,
                &[],
            );
        })
        .expect("sidebar render should succeed");

    let buffer = terminal.backend().buffer();
    let plain = (area.y..area.y.saturating_add(area.height))
        .map(|y| {
            (area.x..area.x.saturating_add(area.width))
                .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        plain.contains("Root worker"),
        "expected anchor row in spawned sidebar, got: {plain}"
    );
    assert!(
        plain.contains("Child worker"),
        "expected child row in spawned sidebar, got: {plain}"
    );
    assert!(
        plain.contains("Grandchild worker"),
        "expected grandchild row in spawned sidebar, got: {plain}"
    );
}

#[test]
fn spawned_sidebar_hit_test_returns_spawned_target_for_child_row() {
    let mut sidebar = SidebarState::new();
    sidebar.reduce(crate::state::sidebar::SidebarAction::SwitchTab(
        crate::state::sidebar::SidebarTab::Spawned,
    ));
    let mut tasks = TaskState::new();
    tasks.reduce(TaskAction::TaskListReceived(vec![
        spawned_sidebar_task(
            "root-task",
            "Root worker",
            10,
            Some("thread-root"),
            None,
            None,
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "child-task",
            "Child worker",
            20,
            Some("thread-child"),
            Some("root-task"),
            Some("thread-root"),
            Some(crate::state::task::TaskStatus::InProgress),
        ),
    ]));
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-child".to_string(),
        title: "Child".to_string(),
    });
    chat.reduce(ChatAction::SelectThread("thread-root".to_string()));

    let area = Rect::new(0, 0, 44, 10);
    let target = hit_test(
        area,
        &chat,
        &sidebar,
        &tasks,
        Some("thread-root"),
        Position::new(2, 2),
    );

    assert_eq!(target, Some(SidebarHitTarget::Spawned(1)));
}

#[test]
fn spawned_sidebar_selects_first_thread_in_selected_branch() {
    let mut sidebar = SidebarState::new();
    sidebar.reduce(crate::state::sidebar::SidebarAction::SwitchTab(
        crate::state::sidebar::SidebarTab::Spawned,
    ));
    let mut tasks = TaskState::new();
    tasks.reduce(TaskAction::TaskListReceived(vec![
        spawned_sidebar_task(
            "root-task",
            "Root worker",
            40,
            Some("thread-root"),
            None,
            None,
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "other-task",
            "Other branch",
            30,
            Some("thread-other"),
            None,
            Some("thread-root"),
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "branch-task",
            "Spawn status row",
            20,
            None,
            Some("root-task"),
            Some("thread-root"),
            Some(crate::state::task::TaskStatus::Completed),
        ),
        spawned_sidebar_task(
            "child-task",
            "Nested child",
            10,
            Some("thread-child"),
            Some("branch-task"),
            None,
            Some(crate::state::task::TaskStatus::InProgress),
        ),
    ]));

    sidebar.select(2, 4);

    assert_eq!(
        selected_spawned_thread_id(&tasks, &sidebar, Some("thread-root")),
        Some("thread-child".to_string()),
        "non-openable rows should resolve to the first thread in their own subtree"
    );
}

#[test]
fn spawned_sidebar_disabled_row_does_not_resolve_to_other_branch() {
    let mut sidebar = SidebarState::new();
    sidebar.reduce(crate::state::sidebar::SidebarAction::SwitchTab(
        crate::state::sidebar::SidebarTab::Spawned,
    ));
    let mut tasks = TaskState::new();
    tasks.reduce(TaskAction::TaskListReceived(vec![
        spawned_sidebar_task(
            "root-task",
            "Root worker",
            30,
            Some("thread-root"),
            None,
            None,
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "other-task",
            "Other branch",
            20,
            Some("thread-other"),
            None,
            Some("thread-root"),
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "disabled-task",
            "Dormant worker",
            10,
            None,
            Some("root-task"),
            Some("thread-root"),
            Some(crate::state::task::TaskStatus::Completed),
        ),
    ]));
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-other".to_string(),
        title: "Other".to_string(),
    });
    chat.reduce(ChatAction::SelectThread("thread-root".to_string()));

    let area = Rect::new(0, 0, 44, 10);
    let theme = ThemeTokens::default();
    let backend = TestBackend::new(area.width, area.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| {
            render(
                frame,
                area,
                &chat,
                &sidebar,
                &tasks,
                Some("thread-root"),
                &theme,
                true,
                &[],
                &crate::state::tier::TierState::default(),
                None,
                None,
                &[],
            );
        })
        .expect("sidebar render should succeed");

    let buffer = terminal.backend().buffer();
    let disabled_index = (area.y..area.y.saturating_add(area.height))
        .find_map(|row| {
            let rendered = (area.x..area.x.saturating_add(area.width))
                .filter_map(|x| buffer.cell((x, row)).map(|cell| cell.symbol()))
                .collect::<String>();
            if !rendered.contains("Dormant worker") {
                return None;
            }
            (area.x..area.x.saturating_add(area.width)).find_map(|column| {
                match hit_test(
                    area,
                    &chat,
                    &sidebar,
                    &tasks,
                    Some("thread-root"),
                    Position::new(column, row),
                ) {
                    Some(SidebarHitTarget::Spawned(index)) => Some(index),
                    _ => None,
                }
            })
        })
        .expect("disabled row should map to a spawned sidebar index");

    sidebar.select(disabled_index, 3);

    assert_eq!(
        selected_spawned_thread_id(&tasks, &sidebar, Some("thread-root")),
        None,
        "rows without a descendant thread should stay disabled instead of falling through to another branch"
    );
}

#[test]
fn spawned_sidebar_renders_disabled_rows_dimmed() {
    let mut sidebar = SidebarState::new();
    sidebar.reduce(crate::state::sidebar::SidebarAction::SwitchTab(
        crate::state::sidebar::SidebarTab::Spawned,
    ));
    let mut tasks = TaskState::new();
    tasks.reduce(TaskAction::TaskListReceived(vec![
        spawned_sidebar_task(
            "root-task",
            "Root worker",
            20,
            Some("thread-root"),
            None,
            None,
            Some(crate::state::task::TaskStatus::InProgress),
        ),
        spawned_sidebar_task(
            "disabled-task",
            "Dormant worker",
            10,
            None,
            Some("root-task"),
            Some("thread-root"),
            Some(crate::state::task::TaskStatus::Completed),
        ),
    ]));
    let mut chat = ChatState::new();
    chat.reduce(ChatAction::ThreadCreated {
        thread_id: "thread-root".to_string(),
        title: "Root".to_string(),
    });
    chat.reduce(ChatAction::SelectThread("thread-root".to_string()));

    let area = Rect::new(0, 0, 44, 10);
    let theme = ThemeTokens::default();
    let backend = TestBackend::new(area.width, area.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| {
            render(
                frame,
                area,
                &chat,
                &sidebar,
                &tasks,
                Some("thread-root"),
                &theme,
                true,
                &[],
                &crate::state::tier::TierState::default(),
                None,
                None,
                &[],
            );
        })
        .expect("sidebar render should succeed");

    let buffer = terminal.backend().buffer();
    let dim_fg = theme.fg_dim.fg.expect("dim fg");
    let disabled_row_dimmed = (area.y..area.y.saturating_add(area.height))
        .find_map(|y| {
            let row = (area.x..area.x.saturating_add(area.width))
                .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                .collect::<String>();
            let start = row.find("Dormant worker")?;
            Some((0..("Dormant worker".len())).all(|offset| {
                buffer
                    .cell((area.x + (start as u16) + (offset as u16), y))
                    .map(|cell| cell.fg == dim_fg)
                    .unwrap_or(false)
            }))
        })
        .expect("disabled row should be visible");

    assert!(
        disabled_row_dimmed,
        "disabled spawned rows should render with the dim foreground"
    );
}
