use super::super::*;
use super::super::{push_section_title, RenderRow};
use crate::state::task::{GoalRun, TaskState};
use crate::theme::ThemeTokens;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub(crate) fn render_steps(
    rows: &mut Vec<RenderRow>,
    tasks: &TaskState,
    run: &GoalRun,
    selected_step_id: Option<&str>,
    theme: &ThemeTokens,
    width: usize,
    tick: Option<u64>,
) {
    push_section_title(
        rows,
        "Execution Plan",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );

    let mut steps = run.steps.clone();
    steps.sort_by_key(|step| step.order);

    if steps.is_empty() {
        rows.push(RenderRow {
            line: Line::from(Span::styled("No steps", theme.fg_dim)),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        return;
    }

    for step in &steps {
        let active = run.current_step_index == step.order as usize
            || run.current_step_title.as_deref() == Some(step.title.as_str());
        let (chip, chip_style) = goal_step_glyph(step.status, active, run.status, theme, tick);
        let mut line = Line::from(vec![
            Span::styled(chip.to_string(), chip_style),
            Span::raw(" "),
            Span::styled(
                format!("{:02}.", step.order.saturating_add(1)),
                theme.fg_dim,
            ),
            Span::raw(" "),
            Span::styled(
                step.title.clone(),
                if active {
                    theme.accent_primary
                } else {
                    theme.fg_active
                },
            ),
        ]);
        if selected_step_id == Some(step.id.as_str()) {
            line = line.style(Style::default().bg(Color::Indexed(236)));
        }
        rows.push(RenderRow {
            line,
            work_path: None,
            goal_step_id: Some(step.id.clone()),
            close_preview: false,
        });

        if !step.instructions.is_empty() {
            push_wrapped_text(rows, &step.instructions, theme.fg_dim, width, 2);
        }
        if let Some(summary) = &step.summary {
            push_wrapped_text(rows, summary, theme.fg_active, width, 2);
        }
        if let Some(error) = &step.error {
            push_wrapped_text(rows, error, theme.accent_danger, width, 2);
        }

        for task in related_tasks_for_step(tasks, run, step) {
            rows.push(RenderRow {
                line: Line::from(vec![
                    Span::raw("  "),
                    Span::styled("• ", theme.fg_dim),
                    Span::styled(task.title.clone(), theme.fg_active),
                    Span::raw(" "),
                    Span::styled(task_status_label(task.status), theme.fg_dim),
                ]),
                work_path: None,
                goal_step_id: None,
                close_preview: false,
            });
        }
    }
}

pub(crate) fn render_step_timeline(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
) {
    if run.events.is_empty() {
        return;
    }

    push_section_title(
        rows,
        "Step Timeline",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    for event in run.events.iter().rev().take(18).rev() {
        let mut prefix = format!("[{}] {}", event.phase, event.message);
        if let Some(step_index) = event.step_index {
            prefix = format!("step {} • {}", step_index + 1, prefix);
        }
        push_wrapped_text(rows, &prefix, theme.fg_active, width, 0);
        if let Some(details) = &event.details {
            push_wrapped_text(rows, details, theme.fg_dim, width, 2);
        }
        if !event.todo_snapshot.is_empty() {
            push_todo_items(rows, &event.todo_snapshot, theme, width, 4);
        }
    }
}

pub(crate) fn render_live_todos(
    rows: &mut Vec<RenderRow>,
    tasks: &TaskState,
    thread_id: Option<&str>,
    theme: &ThemeTokens,
    width: usize,
) {
    let Some(thread_id) = thread_id else {
        return;
    };
    push_section_title(
        rows,
        "Live Todos",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    push_todo_items(rows, tasks.todos_for_thread(thread_id), theme, width, 0);
}

pub(crate) fn render_work_context(
    rows: &mut Vec<RenderRow>,
    tasks: &TaskState,
    thread_id: Option<&str>,
    theme: &ThemeTokens,
    width: usize,
) {
    let Some(thread_id) = thread_id else {
        return;
    };
    let Some(context) = tasks.work_context_for_thread(thread_id) else {
        return;
    };

    push_section_title(
        rows,
        "Files",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    if context.entries.is_empty() {
        rows.push(RenderRow {
            line: Line::from(Span::styled(
                "No file or artifact activity yet.",
                theme.fg_dim,
            )),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        return;
    }

    let selected_path = tasks.selected_work_path(thread_id);
    for entry in &context.entries {
        let label = entry
            .change_kind
            .as_deref()
            .unwrap_or_else(|| work_kind_label(entry.kind));
        let marker = if selected_path == Some(entry.path.as_str()) {
            ">"
        } else {
            " "
        };
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::styled(marker, theme.accent_primary),
                Span::raw(" "),
                Span::styled(format!("[{}]", label), theme.fg_dim),
                Span::raw(" "),
                Span::styled(entry.path.clone(), theme.fg_active),
                if !entry.source.trim().is_empty() {
                    Span::styled(format!("  via {}", entry.source), theme.fg_dim)
                } else {
                    Span::raw("")
                },
            ]),
            work_path: Some(entry.path.clone()),
            goal_step_id: None,
            close_preview: false,
        });
        if let Some(previous_path) = &entry.previous_path {
            push_wrapped_text(
                rows,
                &format!("from {}", previous_path),
                theme.fg_dim,
                width,
                4,
            );
        }
    }

    let Some(selected_path) = selected_path else {
        return;
    };
    let Some(selected_entry) = context
        .entries
        .iter()
        .find(|entry| entry.path == selected_path)
    else {
        return;
    };

    push_section_title(
        rows,
        "Preview",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    rows.push(RenderRow {
        line: Line::from(vec![
            Span::styled("[x]", theme.accent_danger),
            Span::raw(" "),
            Span::styled("Close preview", theme.fg_dim),
        ]),
        work_path: None,
        goal_step_id: None,
        close_preview: true,
    });
    if let Some(repo_root) = selected_entry.repo_root.as_deref() {
        if let Some(diff) = tasks.diff_for_path(repo_root, &selected_entry.path) {
            if diff.trim().is_empty() {
                rows.push(RenderRow {
                    line: Line::from(Span::styled(
                        "No diff preview available for the selected file.",
                        theme.fg_dim,
                    )),
                    work_path: None,
                    goal_step_id: None,
                    close_preview: false,
                });
            } else {
                push_wrapped_text(rows, diff, theme.fg_dim, width, 0);
            }
            return;
        }
    }

    let preview_key = if selected_entry.repo_root.is_some() {
        selected_entry
            .repo_root
            .as_deref()
            .map(|repo_root| format!("{repo_root}/{}", selected_entry.path))
            .unwrap_or_else(|| selected_entry.path.clone())
    } else {
        selected_entry.path.clone()
    };
    if let Some(preview) = tasks.preview_for_path(&preview_key) {
        if preview.is_text {
            push_preview_text(rows, &selected_entry.path, &preview.content, theme, width);
        } else {
            rows.push(RenderRow {
                line: Line::from(Span::styled(
                    "Binary file preview is not available.",
                    theme.fg_dim,
                )),
                work_path: None,
                goal_step_id: None,
                close_preview: false,
            });
        }
    } else {
        rows.push(RenderRow {
            line: Line::from(Span::styled("Loading preview...", theme.fg_dim)),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
}
