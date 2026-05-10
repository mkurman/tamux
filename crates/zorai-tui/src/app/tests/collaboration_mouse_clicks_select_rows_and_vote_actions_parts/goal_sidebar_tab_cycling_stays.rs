use super::super::{build_model, rendered_chat_area};
use super::*;
use ratatui::backend::TestBackend;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn make_temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("zorai-tui-tab-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("temporary directory should be creatable");
    dir
}

pub(super) struct CurrentDirGuard(PathBuf);

impl CurrentDirGuard {
    pub(super) fn enter(dir: &Path) -> Self {
        let previous = std::env::current_dir().expect("current dir should be readable");
        std::env::set_current_dir(dir).expect("temporary dir should be settable");
        Self(previous)
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

pub fn goal_sidebar_model() -> TuiModel {
    let mut model = build_model();
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-1".to_string(),
        title: "Goal Thread".to_string(),
    });
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-2".to_string(),
        title: "Child Task Thread".to_string(),
    });
    model.chat.reduce(chat::ChatAction::ThreadCreated {
        thread_id: "thread-exec".to_string(),
        title: "Execution Thread".to_string(),
    });
    model
        .chat
        .reduce(chat::ChatAction::SelectThread("thread-1".to_string()));
    model.tasks.reduce(task::TaskAction::TaskListReceived(vec![
        task::AgentTask {
            id: "task-1".to_string(),
            title: "Child Task One".to_string(),
            thread_id: Some("thread-1".to_string()),
            goal_run_id: Some("goal-1".to_string()),
            created_at: 10,
            ..Default::default()
        },
        task::AgentTask {
            id: "task-2".to_string(),
            title: "Child Task Two".to_string(),
            thread_id: Some("thread-2".to_string()),
            goal_run_id: Some("goal-1".to_string()),
            created_at: 20,
            ..Default::default()
        },
    ]));
    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal Title".to_string(),
            thread_id: Some("thread-1".to_string()),
            root_thread_id: Some("thread-1".to_string()),
            active_thread_id: Some("thread-exec".to_string()),
            execution_thread_ids: vec!["thread-exec".to_string()],
            goal: "goal definition body".to_string(),
            status: Some(task::GoalRunStatus::Running),
            current_step_title: Some("Implement".to_string()),
            runtime_assignment_list: vec![task::GoalAgentAssignment {
                role_id: "implementer".to_string(),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                reasoning_effort: Some("high".to_string()),
                enabled: true,
                inherit_from_main: false,
            }],
            planner_owner_profile: Some(task::GoalRuntimeOwnerProfile {
                agent_label: "Planner".to_string(),
                provider: "openai".to_string(),
                model: "gpt-5.4-mini".to_string(),
                reasoning_effort: Some("medium".to_string()),
            }),
            current_step_owner_profile: Some(task::GoalRuntimeOwnerProfile {
                agent_label: "Executor".to_string(),
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                reasoning_effort: Some("high".to_string()),
            }),
            child_task_ids: vec!["task-1".to_string(), "task-2".to_string()],
            last_error: Some("upstream returned an empty error body".to_string()),
            steps: vec![
                task::GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    order: 0,
                    instructions: "Ground the user's background before planning".to_string(),
                    summary: Some("Gather current experience and constraints first.".to_string()),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-2".to_string(),
                    title: "Implement".to_string(),
                    order: 1,
                    instructions: "Collect and index sources into a reusable inventory".to_string(),
                    summary: Some("Build the source-backed inventory and checkpoints.".to_string()),
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-3".to_string(),
                    title: "Verify".to_string(),
                    order: 2,
                    instructions: "Check proof coverage and package the final entry plan"
                        .to_string(),
                    ..Default::default()
                },
            ],
            dossier: Some(task::GoalRunDossier {
                projection_state: "in_progress".to_string(),
                summary: Some(
                    "Execution is split into source collection and onboarding package delivery."
                        .to_string(),
                ),
                latest_resume_decision: Some(task::GoalResumeDecisionRecord {
                    action: "advance".to_string(),
                    reason_code: "step_completed".to_string(),
                    reason: Some("goal step completed successfully".to_string()),
                    details: vec!["advance via step_completed".to_string()],
                    projection_state: "in_progress".to_string(),
                    ..Default::default()
                }),
                units: vec![task::GoalDeliveryUnitRecord {
                    id: "unit-1".to_string(),
                    title: "Collect and index ingenix.ai sources".to_string(),
                    status: "in_progress".to_string(),
                    execution_binding: "builtin:swarog".to_string(),
                    verification_binding: "builtin:swarog".to_string(),
                    summary: Some(
                        "Build a source-backed inventory in markdown and csv.".to_string(),
                    ),
                    proof_checks: vec![task::GoalProofCheckRecord {
                        id: "pc-1".to_string(),
                        title: "Verify sources cover company info".to_string(),
                        state: "pending".to_string(),
                        summary: Some("Need official pages plus supporting evidence.".to_string()),
                        ..Default::default()
                    }],
                    report: Some(task::GoalRunReportRecord {
                        summary: "Source inventory is in progress".to_string(),
                        state: "in_progress".to_string(),
                        notes: vec!["Capture official pages first.".to_string()],
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                report: Some(task::GoalRunReportRecord {
                    summary: "Overall goal remains in progress".to_string(),
                    state: "in_progress".to_string(),
                    notes: vec!["Approval still pending for final package.".to_string()],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            events: vec![task::GoalRunEvent {
                id: "event-1".to_string(),
                phase: "execution".to_string(),
                message: "queued child task for goal step".to_string(),
                details: Some("Collect and index ingenix.ai sources -> task_123".to_string()),
                step_index: Some(1),
                todo_snapshot: vec![task::TodoItem {
                    id: "todo-impl-1".to_string(),
                    content: "Verify sources cover company info".to_string(),
                    status: Some(task::TodoStatus::Pending),
                    step_index: Some(1),
                    position: 0,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }));
    model
        .tasks
        .reduce(task::TaskAction::GoalRunCheckpointsReceived {
            goal_run_id: "goal-1".to_string(),
            checkpoints: vec![
                task::GoalRunCheckpointSummary {
                    id: "checkpoint-1".to_string(),
                    checkpoint_type: "plan".to_string(),
                    step_index: Some(1),
                    context_summary_preview: Some("Checkpoint for Implement".to_string()),
                    ..Default::default()
                },
                task::GoalRunCheckpointSummary {
                    id: "checkpoint-2".to_string(),
                    checkpoint_type: "note".to_string(),
                    context_summary_preview: Some("Loose checkpoint".to_string()),
                    ..Default::default()
                },
            ],
        });
    model.tasks.reduce(task::TaskAction::ThreadTodosReceived {
        thread_id: "thread-1".to_string(),
        goal_run_id: Some("goal-1".to_string()),
        step_index: Some(0),
        items: vec![
            task::TodoItem {
                id: "todo-1".to_string(),
                content: "Draft outline".to_string(),
                status: Some(task::TodoStatus::InProgress),
                step_index: Some(0),
                position: 0,
                ..Default::default()
            },
            task::TodoItem {
                id: "todo-2".to_string(),
                content: "Verify sources".to_string(),
                status: Some(task::TodoStatus::Pending),
                step_index: Some(0),
                position: 1,
                ..Default::default()
            },
        ],
    });
    model.tasks.reduce(task::TaskAction::WorkContextReceived(
        task::ThreadWorkContext {
            thread_id: "thread-1".to_string(),
            entries: vec![
                task::WorkContextEntry {
                    path: "/tmp/plan.md".to_string(),
                    goal_run_id: Some("goal-1".to_string()),
                    is_text: true,
                    ..Default::default()
                },
                task::WorkContextEntry {
                    path: "/tmp/report.md".to_string(),
                    goal_run_id: Some("goal-1".to_string()),
                    is_text: true,
                    ..Default::default()
                },
            ],
        },
    ));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-1".to_string()),
    });
    model.focus = FocusArea::Sidebar;
    model
}

pub(super) fn open_goal_execution_thread(model: &mut TuiModel) {
    model.focus = FocusArea::Chat;
    model.goal_workspace.set_selected_plan_row(1);
    model.goal_workspace.set_selected_plan_item(Some(
        goal_workspace::GoalPlanSelection::MainThread {
            thread_id: "thread-exec".to_string(),
        },
    ));
    let handled = model.handle_key(KeyCode::Enter, KeyModifiers::NONE);
    assert!(!handled);
    assert!(matches!(model.main_pane_view, MainPaneView::Conversation));
    assert_eq!(model.chat.active_thread_id(), Some("thread-exec"));
}

pub(super) fn mission_control_thread_router_model(
    active_thread_id: Option<&str>,
    root_thread_id: Option<&str>,
) -> TuiModel {
    let mut model = build_model();
    model.connected = true;
    model.agent_config_loaded = true;
    let thread_ids = [active_thread_id, root_thread_id];
    for thread_id in thread_ids.into_iter().flatten() {
        model.chat.reduce(chat::ChatAction::ThreadCreated {
            thread_id: thread_id.to_string(),
            title: format!("Thread {thread_id}"),
        });
        model
            .chat
            .reduce(chat::ChatAction::ThreadDetailReceived(chat::AgentThread {
                id: thread_id.to_string(),
                title: format!("Thread {thread_id}"),
                messages: vec![chat::AgentMessage {
                    id: Some(format!("message-{thread_id}")),
                    role: chat::MessageRole::Assistant,
                    content: format!("Conversation for {thread_id}"),
                    ..Default::default()
                }],
                loaded_message_end: 1,
                total_message_count: 1,
                ..Default::default()
            }));
    }

    model
        .tasks
        .reduce(task::TaskAction::GoalRunDetailReceived(task::GoalRun {
            id: "goal-1".to_string(),
            title: "Goal Title".to_string(),
            thread_id: root_thread_id.map(str::to_string),
            root_thread_id: root_thread_id.map(str::to_string),
            active_thread_id: active_thread_id.map(str::to_string),
            goal: "goal definition body".to_string(),
            current_step_title: Some("Implement".to_string()),
            steps: vec![
                task::GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    order: 0,
                    ..Default::default()
                },
                task::GoalRunStep {
                    id: "step-2".to_string(),
                    title: "Implement".to_string(),
                    order: 1,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }));
    model.main_pane_view = MainPaneView::Task(SidebarItemTarget::GoalRun {
        goal_run_id: "goal-1".to_string(),
        step_id: Some("step-2".to_string()),
    });
    model.focus = FocusArea::Chat;
    model.open_new_goal_view();
    model.status_line.clear();
    model
}

pub fn render_chat_plain(model: &mut TuiModel) -> String {
    let backend = TestBackend::new(model.width, model.height);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    terminal
        .draw(|frame| model.render(frame))
        .expect("render should succeed");

    let chat_area = rendered_chat_area(model);
    let buffer = terminal.backend().buffer();
    (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .map(|y| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width))
                .filter_map(|x| buffer.cell((x, y)).map(|cell| cell.symbol()))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn goal_workspace_click_targets(chat_area: Rect) -> (Position, Position, Position) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(1)])
        .split(chat_area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(32),
            Constraint::Min(24),
        ])
        .split(layout[1]);
    let plan = Block::default().borders(Borders::ALL).inner(columns[0]);
    let timeline = Block::default().borders(Borders::ALL).inner(columns[1]);
    let details = Block::default().borders(Borders::ALL).inner(columns[2]);

    (
        Position::new(plan.x.saturating_add(1), plan.y),
        Position::new(timeline.x.saturating_add(1), timeline.y),
        Position::new(details.x.saturating_add(1), details.y),
    )
}

pub(super) fn find_goal_workspace_hit_position(
    model: &TuiModel,
    expected: widgets::goal_workspace::GoalWorkspaceHitTarget,
) -> Position {
    let chat_area = rendered_chat_area(model);
    (chat_area.y..chat_area.y.saturating_add(chat_area.height))
        .find_map(|row| {
            (chat_area.x..chat_area.x.saturating_add(chat_area.width)).find_map(|column| {
                let pos = Position::new(column, row);
                let hit = match &model.main_pane_view {
                    MainPaneView::Task(SidebarItemTarget::GoalRun { goal_run_id, .. }) => {
                        widgets::goal_workspace::hit_test(
                            chat_area,
                            &model.tasks,
                            goal_run_id,
                            &model.goal_workspace,
                            pos,
                        )
                    }
                    _ => None,
                };
                (hit == Some(expected.clone())).then_some(pos)
            })
        })
        .expect("expected goal workspace hit target should be clickable")
}

#[test]
fn goal_sidebar_defaults_to_steps_on_model_init() {
    let model = build_model();
    assert_eq!(model.goal_sidebar.active_tab(), GoalSidebarTab::Steps);
}

#[test]
fn goal_sidebar_tab_cycling_stays_within_goal_tabs() {
    let mut state = GoalSidebarState::new();
    assert_eq!(state.active_tab(), GoalSidebarTab::Steps);

    state.cycle_tab_left();
    assert_eq!(state.active_tab(), GoalSidebarTab::Steps);

    state.cycle_tab_right();
    assert_eq!(state.active_tab(), GoalSidebarTab::Checkpoints);

    state.cycle_tab_right();
    assert_eq!(state.active_tab(), GoalSidebarTab::Tasks);

    state.cycle_tab_right();
    assert_eq!(state.active_tab(), GoalSidebarTab::Files);

    state.cycle_tab_right();
    assert_eq!(state.active_tab(), GoalSidebarTab::Files);

    state.cycle_tab_left();
    assert_eq!(state.active_tab(), GoalSidebarTab::Tasks);
}
