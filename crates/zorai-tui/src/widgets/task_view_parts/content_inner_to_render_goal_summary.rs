use crate::state::task::*;
use crate::theme::ThemeTokens;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::widgets::message::{render_markdown_pub, wrap_text};

pub(crate) fn content_inner(area: Rect) -> Rect {
    area
}

pub(crate) const SCROLLBAR_WIDTH: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskViewScrollbarLayout {
    pub content: Rect,
    pub scrollbar: Rect,
    pub thumb: Rect,
    pub scroll: usize,
    pub max_scroll: usize,
}

#[derive(Clone)]
pub(crate) struct RenderRow {
    pub(crate) line: Line<'static>,
    pub(crate) work_path: Option<String>,
    pub(crate) goal_step_id: Option<String>,
    pub(crate) close_preview: bool,
}

pub(crate) const ACTIVITY_SPINNER_FRAMES: [&str; 8] = ["⠁", "⠃", "⠇", "⠧", "⠷", "⠿", "⠷", "⠧"];

pub(crate) fn activity_spinner_frame(tick: u64) -> &'static str {
    ACTIVITY_SPINNER_FRAMES[((tick / 4) as usize) % ACTIVITY_SPINNER_FRAMES.len()]
}

pub(crate) fn is_goal_run_live(status: Option<GoalRunStatus>) -> bool {
    matches!(
        status,
        Some(GoalRunStatus::Planning)
            | Some(GoalRunStatus::Running)
            | Some(GoalRunStatus::AwaitingApproval)
    )
}

pub(crate) fn goal_status_badge(
    status: Option<GoalRunStatus>,
    theme: &ThemeTokens,
    _tick: Option<u64>,
) -> (&'static str, Style) {
    match status {
        Some(GoalRunStatus::Queued) => ("○ QUEUED", theme.fg_dim),
        Some(GoalRunStatus::Planning) => ("◌ PLANNING", theme.accent_secondary),
        Some(GoalRunStatus::Running) => ("◌ RUNNING", theme.accent_success),
        Some(GoalRunStatus::AwaitingApproval) => ("⏸ HOLD", theme.accent_secondary),
        Some(GoalRunStatus::Paused) => ("⏸ PAUSED", theme.accent_secondary),
        Some(GoalRunStatus::Completed) => ("✓ DONE", theme.accent_success),
        Some(GoalRunStatus::Failed) => ("! FAILED", theme.accent_danger),
        Some(GoalRunStatus::Cancelled) => ("■ STOPPED", theme.fg_dim),
        None => ("○ QUEUED", theme.fg_dim),
    }
}

pub(crate) fn goal_step_glyph(
    step_status: Option<GoalRunStatus>,
    active: bool,
    run_status: Option<GoalRunStatus>,
    theme: &ThemeTokens,
    tick: Option<u64>,
) -> (&'static str, Style) {
    let effective = if active {
        run_status.or(step_status)
    } else {
        step_status
    };
    match effective {
        Some(GoalRunStatus::Planning) | Some(GoalRunStatus::Running) => (
            if tick.is_some() {
                activity_spinner_frame(tick.unwrap_or(0))
            } else {
                "◌"
            },
            theme.accent_success,
        ),
        Some(GoalRunStatus::AwaitingApproval) | Some(GoalRunStatus::Paused) => {
            ("⏸", theme.accent_secondary)
        }
        Some(GoalRunStatus::Completed) => ("✓", theme.accent_success),
        Some(GoalRunStatus::Failed) => ("!", theme.accent_danger),
        Some(GoalRunStatus::Cancelled) => ("■", theme.fg_dim),
        _ => {
            if active {
                ("▶", theme.accent_primary)
            } else {
                ("○", theme.fg_dim)
            }
        }
    }
}

pub(crate) fn activity_phase_style(phase: &str, theme: &ThemeTokens) -> Style {
    match phase {
        "tool" | "tool_call" => theme.accent_primary,
        "todo" => theme.accent_secondary,
        "approval" => theme.accent_secondary,
        "failure" | "error" | "restart" => theme.accent_danger,
        _ => theme.fg_dim,
    }
}

pub(crate) fn activity_phase_label(phase: &str) -> String {
    if phase.trim().is_empty() {
        "event".to_string()
    } else {
        phase.replace('_', " ")
    }
}

pub(crate) struct SelectionSnapshot {
    pub(crate) rows: Vec<RenderRow>,
    pub(crate) scroll: usize,
    pub(crate) area: Rect,
}

pub enum TaskViewHitTarget {
    BackToGoal,
    WorkPath(String),
    GoalStep(String),
    ClosePreview,
}

pub(crate) const BACK_TO_GOAL_HIT_PATH: &str = "__zorai_task_view_back_to_goal__";

pub(crate) fn task_status_label(status: Option<TaskStatus>) -> &'static str {
    match status {
        Some(TaskStatus::InProgress) => "running",
        Some(TaskStatus::Completed) => "done",
        Some(TaskStatus::Failed)
        | Some(TaskStatus::FailedAnalyzing)
        | Some(TaskStatus::BudgetExceeded) => "budget exceeded",
        Some(TaskStatus::Blocked) => "blocked",
        Some(TaskStatus::AwaitingApproval) => "awaiting approval",
        Some(TaskStatus::Cancelled) => "cancelled",
        _ => "queued",
    }
}

pub(crate) fn task_status_chip(status: Option<TaskStatus>) -> &'static str {
    match status {
        Some(TaskStatus::InProgress) => "[~]",
        Some(TaskStatus::Completed) => "[x]",
        Some(TaskStatus::Blocked)
        | Some(TaskStatus::Failed)
        | Some(TaskStatus::FailedAnalyzing)
        | Some(TaskStatus::BudgetExceeded) => "[!]",
        _ => "[ ]",
    }
}

pub(crate) fn todo_status_chip(status: Option<TodoStatus>) -> &'static str {
    match status {
        Some(TodoStatus::InProgress) => "[~]",
        Some(TodoStatus::Completed) => "[x]",
        Some(TodoStatus::Blocked) => "[!]",
        _ => "[ ]",
    }
}

pub(crate) fn work_kind_label(kind: Option<WorkContextEntryKind>) -> &'static str {
    match kind {
        Some(WorkContextEntryKind::GeneratedSkill) => "skill",
        Some(WorkContextEntryKind::Artifact) => "file",
        _ => "diff",
    }
}

pub(crate) fn truncate_tail(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    if max_len <= 1 {
        return "…".to_string();
    }
    let tail: String = text
        .chars()
        .rev()
        .take(max_len.saturating_sub(1))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

pub(crate) fn push_wrapped_text(
    rows: &mut Vec<RenderRow>,
    text: &str,
    style: Style,
    width: usize,
    indent: usize,
) {
    let available = width.saturating_sub(indent).max(1);
    for wrapped in wrap_text(text, available) {
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::raw(" ".repeat(indent)),
                Span::styled(wrapped, style),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
}

pub(crate) fn push_blank(rows: &mut Vec<RenderRow>) {
    rows.push(RenderRow {
        line: Line::raw(""),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
}

pub(crate) fn push_section_title(rows: &mut Vec<RenderRow>, title: &str, style: Style) {
    if !rows.is_empty() {
        push_blank(rows);
    }
    rows.push(RenderRow {
        line: Line::from(Span::styled(format!("╭─ {title}"), style)),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
}

pub(crate) fn is_markdown_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".mdx")
}

pub(crate) fn push_preview_text(
    rows: &mut Vec<RenderRow>,
    path: &str,
    content: &str,
    theme: &ThemeTokens,
    width: usize,
) {
    if is_markdown_path(path) {
        for line in render_markdown_pub(content, width.max(1)) {
            rows.push(RenderRow {
                line,
                work_path: None,
                goal_step_id: None,
                close_preview: false,
            });
        }
    } else {
        push_wrapped_text(rows, content, theme.fg_dim, width, 0);
    }
}

pub(crate) fn related_tasks_for_step<'a>(
    tasks: &'a TaskState,
    run: &GoalRun,
    step: &GoalRunStep,
) -> Vec<&'a AgentTask> {
    tasks
        .tasks()
        .iter()
        .filter(|task| {
            task.goal_run_id.as_deref() == Some(run.id.as_str())
                && (task.goal_step_title.as_deref() == Some(step.title.as_str())
                    || step
                        .task_id
                        .as_deref()
                        .is_some_and(|task_id| task.id == task_id))
        })
        .collect()
}

pub(crate) fn push_todo_items(
    rows: &mut Vec<RenderRow>,
    items: &[TodoItem],
    theme: &ThemeTokens,
    _width: usize,
    indent: usize,
) {
    if items.is_empty() {
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::raw(" ".repeat(indent)),
                Span::styled("No todos", theme.fg_dim),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        return;
    }

    let mut sorted = items.to_vec();
    sorted.sort_by_key(|item| item.position);
    for item in sorted {
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::raw(" ".repeat(indent)),
                Span::styled(todo_status_chip(item.status), theme.fg_dim),
                Span::raw(" "),
                Span::styled(item.content, theme.fg_active),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
}

pub(crate) fn render_goal_summary(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
    tick: Option<u64>,
) {
    let (status_badge, status_style) = goal_status_badge(run.status, theme, tick);
    let step_total = run
        .steps
        .len()
        .max(run.current_step_index.saturating_add(1));

    push_section_title(
        rows,
        "Run Status",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    rows.push(RenderRow {
        line: Line::from(vec![
            Span::styled(
                status_badge.to_string(),
                status_style.add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                run.title.clone(),
                theme.fg_active.add_modifier(Modifier::BOLD),
            ),
        ]),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
    rows.push(RenderRow {
        line: Line::from(vec![
            Span::styled("Tasks ", theme.fg_dim),
            Span::styled(run.child_task_count.to_string(), theme.fg_active),
            Span::styled("  Approvals ", theme.fg_dim),
            Span::styled(run.approval_count.to_string(), theme.fg_active),
            Span::styled("  Step ", theme.fg_dim),
            Span::styled(
                format!(
                    "{}/{}",
                    run.current_step_index.saturating_add(1),
                    step_total
                ),
                theme.fg_active,
            ),
        ]),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
    if let Some(current_step_title) = &run.current_step_title {
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::styled("Current focus ", theme.fg_dim),
                Span::styled(current_step_title.clone(), theme.fg_active),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
    if matches!(
        run.status,
        Some(GoalRunStatus::Paused) | Some(GoalRunStatus::AwaitingApproval)
    ) {
        let restart_paused = run
            .events
            .iter()
            .rev()
            .any(|event| event.phase.eq_ignore_ascii_case("restart"));
        let review_hint = if restart_paused {
            "Review here: check Live Activity and the dossier below, then use Ctrl+S to resume or A for actions."
        } else if run.awaiting_approval_id.is_some()
            || matches!(run.status, Some(GoalRunStatus::AwaitingApproval))
        {
            "Review here: inspect the current step and open approvals with Ctrl+A, then resume or stop from Controls."
        } else {
            "Review here: inspect the current step, recent activity, and dossier below before resuming."
        };
        push_wrapped_text(rows, review_hint, theme.accent_secondary, width, 0);
    }
    rows.push(RenderRow {
        line: Line::from(vec![
            Span::styled("ID ", theme.fg_dim),
            Span::styled(run.id.clone(), theme.fg_active),
            if let Some(thread_id) = run.thread_id.as_ref() {
                Span::styled(format!("  Thread {thread_id}"), theme.fg_dim)
            } else {
                Span::raw("")
            },
            if let Some(session_id) = run.session_id.as_ref() {
                Span::styled(format!("  Session {session_id}"), theme.fg_dim)
            } else {
                Span::raw("")
            },
        ]),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
    if !run.goal.is_empty() {
        push_section_title(
            rows,
            "Goal Definition",
            theme.accent_primary.add_modifier(Modifier::BOLD),
        );
        push_wrapped_text(rows, &run.goal, theme.fg_active, width, 0);
    }
    if let Some(last_error) = &run.last_error {
        push_section_title(
            rows,
            "Last Error",
            theme.accent_primary.add_modifier(Modifier::BOLD),
        );
        push_wrapped_text(rows, last_error, theme.accent_danger, width, 0);
    }
}
