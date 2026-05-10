use super::sections::*;
use super::selection::*;
use super::*;
use crate::state::sidebar::SidebarItemTarget;
use crate::state::task::*;
use crate::theme::ThemeTokens;
use crate::widgets::chat::SelectionPoint;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
pub(crate) fn build_rows(
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    width: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
    tick: Option<u64>,
) -> (String, Vec<RenderRow>) {
    let mut rows = Vec::new();
    let section_style = theme.accent_primary.add_modifier(Modifier::BOLD);
    let highlight_style = Style::default().bg(Color::Indexed(236));

    match target {
        SidebarItemTarget::GoalRun {
            goal_run_id,
            step_id,
        } => {
            let Some(run) = tasks.goal_run_by_id(goal_run_id) else {
                return (
                    " Goal ".to_string(),
                    vec![RenderRow {
                        line: Line::from(Span::styled("Goal run not found", theme.accent_danger)),
                        work_path: None,
                        goal_step_id: None,
                        close_preview: false,
                    }],
                );
            };
            render_goal_summary(&mut rows, run, theme, width, tick);
            render_goal_controls(&mut rows, run, step_id.as_deref(), theme, width);
            render_goal_usage(&mut rows, run, theme, width);
            render_goal_agents(&mut rows, tasks, run, theme, width);
            render_live_activity(&mut rows, tasks, run, theme, width, tick);
            render_dossier(&mut rows, run, theme, width);
            render_resume_decision(&mut rows, run, theme, width);
            render_delivery_units(&mut rows, run, theme, width);
            render_proof_coverage(&mut rows, run, theme, width);
            render_reports(&mut rows, run, theme, width);
            render_checkpoints(&mut rows, tasks, run, theme, width);
            if show_live_todos {
                render_live_todos(&mut rows, tasks, run.thread_id.as_deref(), theme, width);
            }
            if show_files {
                render_work_context(&mut rows, tasks, run.thread_id.as_deref(), theme, width);
            }
            render_steps(
                &mut rows,
                tasks,
                run,
                step_id.as_deref(),
                theme,
                width,
                tick,
            );

            let child_tasks: Vec<_> = tasks
                .tasks()
                .iter()
                .filter(|task| task.goal_run_id.as_deref() == Some(goal_run_id.as_str()))
                .collect();
            push_section_title(&mut rows, "Related Tasks", section_style);
            if child_tasks.is_empty() {
                rows.push(RenderRow {
                    line: Line::from(Span::styled("No tasks", theme.fg_dim)),
                    work_path: None,
                    goal_step_id: None,
                    close_preview: false,
                });
            } else {
                for task in child_tasks {
                    rows.push(RenderRow {
                        line: Line::from(vec![
                            Span::styled(
                                format!("{} ", task_status_chip(task.status)),
                                theme.fg_dim,
                            ),
                            Span::styled(task.title.clone(), theme.fg_active),
                        ]),
                        work_path: None,
                        goal_step_id: None,
                        close_preview: false,
                    });
                }
            }

            if let Some(summary) = &run.reflection_summary {
                push_section_title(&mut rows, "Reflection", section_style);
                push_wrapped_text(&mut rows, summary, theme.fg_dim, width, 0);
            }
            if !run.memory_updates.is_empty() {
                push_section_title(&mut rows, "Memory Updates", section_style);
                for update in &run.memory_updates {
                    push_wrapped_text(&mut rows, &format!("• {}", update), theme.fg_dim, width, 0);
                }
            }
            if show_timeline {
                render_step_timeline(&mut rows, run, theme, width);
            }

            (format!(" Goal: {} ", run.title), rows)
        }
        SidebarItemTarget::Task { task_id } => {
            let Some(task) = tasks.task_by_id(task_id) else {
                return (
                    " Task ".to_string(),
                    vec![RenderRow {
                        line: Line::from(Span::styled("Task not found", theme.accent_danger)),
                        work_path: None,
                        goal_step_id: None,
                        close_preview: false,
                    }],
                );
            };

            rows.push(RenderRow {
                line: Line::from(vec![
                    Span::styled("Status: ", theme.fg_dim),
                    Span::styled(task_status_label(task.status), theme.fg_active),
                ]),
                work_path: None,
                goal_step_id: None,
                close_preview: false,
            });
            rows.push(RenderRow {
                line: Line::from(vec![
                    Span::styled("Progress: ", theme.fg_dim),
                    Span::styled(format!("{}%", task.progress), theme.fg_active),
                ]),
                work_path: None,
                goal_step_id: None,
                close_preview: false,
            });
            if let Some(session_id) = &task.session_id {
                rows.push(RenderRow {
                    line: Line::from(vec![
                        Span::styled("Session: ", theme.fg_dim),
                        Span::styled(session_id.clone(), theme.fg_active),
                    ]),
                    work_path: None,
                    goal_step_id: None,
                    close_preview: false,
                });
            }

            let parent_goal = task
                .goal_run_id
                .as_deref()
                .and_then(|goal_run_id| tasks.goal_run_by_id(goal_run_id));
            if let Some(run) = parent_goal {
                push_section_title(&mut rows, "Navigation", section_style);
                rows.push(RenderRow {
                    line: Line::from(vec![
                        Span::styled("[← Back to goal]", theme.accent_primary),
                        Span::raw("  "),
                        Span::styled("Esc", theme.fg_dim),
                    ]),
                    work_path: Some(BACK_TO_GOAL_HIT_PATH.to_string()),
                    goal_step_id: None,
                    close_preview: false,
                });
                push_section_title(&mut rows, "Parent Goal", section_style);
                push_wrapped_text(&mut rows, &run.title, theme.fg_active, width, 0);
                if !run.goal.is_empty() {
                    push_wrapped_text(&mut rows, &run.goal, theme.fg_dim, width, 0);
                }
                if let Some(step) = run.steps.iter().find(|step| {
                    step.task_id.as_deref() == Some(task.id.as_str())
                        || task.goal_step_title.as_deref() == Some(step.title.as_str())
                }) {
                    rows.push(RenderRow {
                        line: Line::from(vec![
                            Span::styled("Step: ", theme.fg_dim),
                            Span::styled(step.title.clone(), theme.fg_active),
                        ])
                        .style(highlight_style),
                        work_path: None,
                        goal_step_id: None,
                        close_preview: false,
                    });
                    if !step.instructions.is_empty() {
                        push_wrapped_text(&mut rows, &step.instructions, theme.fg_dim, width, 2);
                    }
                    if let Some(summary) = &step.summary {
                        push_wrapped_text(&mut rows, summary, theme.fg_active, width, 2);
                    }
                }
                if show_timeline {
                    render_step_timeline(&mut rows, run, theme, width);
                }
            }

            if show_live_todos {
                render_live_todos(&mut rows, tasks, task.thread_id.as_deref(), theme, width);
            }
            if show_files {
                render_work_context(&mut rows, tasks, task.thread_id.as_deref(), theme, width);
            }
            if let Some(blocked_reason) = &task.blocked_reason {
                push_section_title(&mut rows, "Blocked Reason", section_style);
                push_wrapped_text(&mut rows, blocked_reason, theme.accent_danger, width, 0);
            }

            (format!(" Task: {} ", task.title), rows)
        }
    }
}

/// Cache key for `rows_for_width`. Excludes `tick` deliberately — animation
/// updates would otherwise force a rebuild every frame, defeating the cache.
/// Stale-by-one-tick spinners are invisible to the eye and the cache
/// invalidates on any real state change via `tasks_revision`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RowsCacheKey {
    tasks_revision: u64,
    target: SidebarItemTarget,
    width: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
}

thread_local! {
    static ROWS_CACHE: std::cell::RefCell<Option<(RowsCacheKey, Vec<RenderRow>)>> =
        const { std::cell::RefCell::new(None) };
}

pub(crate) fn rows_for_width(
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    width: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
    tick: Option<u64>,
) -> Vec<RenderRow> {
    let key = RowsCacheKey {
        tasks_revision: tasks.tasks_revision(),
        target: target.clone(),
        width,
        show_live_todos,
        show_timeline,
        show_files,
    };
    let cached = ROWS_CACHE.with(|cell| {
        cell.borrow()
            .as_ref()
            .filter(|(cached_key, _)| cached_key == &key)
            .map(|(_, rows)| rows.clone())
    });
    if let Some(rows) = cached {
        return rows;
    }
    let (_, rows) = build_rows(
        tasks,
        target,
        theme,
        width,
        show_live_todos,
        show_timeline,
        show_files,
        tick,
    );
    ROWS_CACHE.with(|cell| {
        *cell.borrow_mut() = Some((key, rows.clone()));
    });
    rows
}

pub(crate) fn scrollbar_layout_from_metrics(
    area: Rect,
    total_rows: usize,
    scroll: usize,
) -> Option<TaskViewScrollbarLayout> {
    if area.width <= SCROLLBAR_WIDTH || area.height == 0 || total_rows <= area.height as usize {
        return None;
    }

    let viewport = area.height as usize;
    let max_scroll = total_rows.saturating_sub(viewport);
    let scroll = scroll.min(max_scroll);
    let content = Rect::new(
        area.x,
        area.y,
        area.width.saturating_sub(SCROLLBAR_WIDTH),
        area.height,
    );
    let scrollbar = Rect::new(
        area.x
            .saturating_add(area.width)
            .saturating_sub(SCROLLBAR_WIDTH),
        area.y,
        SCROLLBAR_WIDTH,
        area.height,
    );
    let thumb_height = ((viewport * viewport) / total_rows).max(1).min(viewport) as u16;
    let track_span = scrollbar.height.saturating_sub(thumb_height);
    let thumb_offset = if max_scroll == 0 || track_span == 0 {
        0
    } else {
        ((scroll * track_span as usize) + (max_scroll / 2)) / max_scroll
    } as u16;
    let thumb = Rect::new(
        scrollbar.x,
        scrollbar.y.saturating_add(thumb_offset),
        scrollbar.width,
        thumb_height,
    );

    Some(TaskViewScrollbarLayout {
        content,
        scrollbar,
        thumb,
        scroll,
        max_scroll,
    })
}

pub(crate) fn hit_test(
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    scroll: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
    position: Position,
) -> Option<TaskViewHitTarget> {
    let inner = content_inner(area);
    let layout = scrollbar_layout(
        area,
        tasks,
        target,
        theme,
        scroll,
        show_live_todos,
        show_timeline,
        show_files,
    );
    let content = layout.map(|layout| layout.content).unwrap_or(inner);
    if !content.contains(position) {
        return None;
    }

    let rows = rows_for_width(
        tasks,
        target,
        theme,
        content.width as usize,
        show_live_todos,
        show_timeline,
        show_files,
        None,
    );
    let resolved_scroll = layout.map(|layout| layout.scroll).unwrap_or(scroll);
    let row_index = resolved_scroll + position.y.saturating_sub(content.y) as usize;
    rows.get(row_index).and_then(|row| {
        if row.close_preview {
            Some(TaskViewHitTarget::ClosePreview)
        } else if let Some(step_id) = row.goal_step_id.clone() {
            Some(TaskViewHitTarget::GoalStep(step_id))
        } else {
            row.work_path.clone().map(|path| {
                if path == BACK_TO_GOAL_HIT_PATH {
                    TaskViewHitTarget::BackToGoal
                } else {
                    TaskViewHitTarget::WorkPath(path)
                }
            })
        }
    })
}

pub(crate) fn selection_snapshot(
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    scroll: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
) -> Option<SelectionSnapshot> {
    let layout = scrollbar_layout(
        area,
        tasks,
        target,
        theme,
        scroll,
        show_live_todos,
        show_timeline,
        show_files,
    );
    let content = layout
        .map(|layout| layout.content)
        .unwrap_or(content_inner(area));
    let resolved_scroll = layout.map(|layout| layout.scroll).unwrap_or(scroll);
    let rows = rows_for_width(
        tasks,
        target,
        theme,
        content.width as usize,
        show_live_todos,
        show_timeline,
        show_files,
        None,
    );
    if rows.is_empty() || content.width == 0 || content.height == 0 {
        return None;
    }
    Some(SelectionSnapshot {
        rows,
        scroll: resolved_scroll,
        area: content,
    })
}

pub(crate) fn selection_point_from_snapshot(
    snapshot: &SelectionSnapshot,
    mouse: Position,
) -> Option<SelectionPoint> {
    let area = snapshot.area;
    let clamped_x = mouse
        .x
        .clamp(area.x, area.x.saturating_add(area.width).saturating_sub(1));
    let clamped_y = mouse
        .y
        .clamp(area.y, area.y.saturating_add(area.height).saturating_sub(1));
    let row = snapshot
        .scroll
        .saturating_add(clamped_y.saturating_sub(area.y) as usize)
        .min(snapshot.rows.len().saturating_sub(1));
    let col = clamped_x.saturating_sub(area.x) as usize;
    let width = line_display_width(&snapshot.rows.get(row)?.line);
    Some(SelectionPoint {
        row,
        col: col.min(width),
    })
}

pub(crate) fn selection_points_from_mouse(
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    scroll: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
    start: Position,
    end: Position,
) -> Option<(SelectionPoint, SelectionPoint)> {
    let snapshot = selection_snapshot(
        area,
        tasks,
        target,
        theme,
        scroll,
        show_live_todos,
        show_timeline,
        show_files,
    )?;
    Some((
        selection_point_from_snapshot(&snapshot, start)?,
        selection_point_from_snapshot(&snapshot, end)?,
    ))
}

pub(crate) fn selection_point_from_mouse(
    area: Rect,
    tasks: &TaskState,
    target: &SidebarItemTarget,
    theme: &ThemeTokens,
    scroll: usize,
    show_live_todos: bool,
    show_timeline: bool,
    show_files: bool,
    mouse: Position,
) -> Option<SelectionPoint> {
    let snapshot = selection_snapshot(
        area,
        tasks,
        target,
        theme,
        scroll,
        show_live_todos,
        show_timeline,
        show_files,
    )?;
    selection_point_from_snapshot(&snapshot, mouse)
}
