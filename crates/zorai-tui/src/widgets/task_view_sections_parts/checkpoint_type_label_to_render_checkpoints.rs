use super::super::*;
use super::super::{push_section_title, RenderRow};
use crate::state::task::{GoalRun, TaskState};
use crate::theme::ThemeTokens;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

fn checkpoint_type_label(raw: &str) -> &'static str {
    match raw {
        "pre_step" => "pre-step",
        "post_step" => "post-step",
        "pre_recovery" => "recovery",
        "periodic" => "periodic",
        "manual" => "manual",
        _ => "checkpoint",
    }
}

fn short_checkpoint_id(id: &str) -> String {
    if id.chars().count() <= 18 {
        return id.to_string();
    }
    let tail: String = id
        .chars()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

fn projection_chip(state: &str) -> String {
    let label = if state.trim().is_empty() {
        "pending"
    } else {
        state
    };
    format!("[{}]", label.replace('_', " "))
}

pub(crate) fn render_live_activity(
    rows: &mut Vec<RenderRow>,
    tasks: &TaskState,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
    tick: Option<u64>,
) {
    let has_thread_context = run
        .thread_id
        .as_deref()
        .and_then(|thread_id| tasks.work_context_for_thread(thread_id))
        .is_some_and(|context| {
            context.entries.iter().any(|entry| {
                entry.goal_run_id.is_none() || entry.goal_run_id.as_deref() == Some(run.id.as_str())
            })
        });
    let has_thread_todos = run
        .thread_id
        .as_deref()
        .is_some_and(|thread_id| !tasks.todos_for_thread(thread_id).is_empty());
    if run.events.is_empty() && !has_thread_context && !has_thread_todos {
        return;
    }

    push_section_title(
        rows,
        "Live Activity",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );

    let mut rendered_any = false;
    let live_now = is_goal_run_live(run.status);
    for (index, event) in run.events.iter().rev().take(4).enumerate() {
        rendered_any = true;
        let marker = if index == 0 && live_now {
            tick.map(activity_spinner_frame).unwrap_or("◌")
        } else {
            "•"
        };
        let step_label = event
            .step_index
            .map(|step_index| format!("step {}", step_index + 1))
            .unwrap_or_else(|| "goal".to_string());
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::styled(
                    format!("{marker} "),
                    activity_phase_style(&event.phase, theme),
                ),
                Span::styled(
                    format!("[{}]", activity_phase_label(&event.phase)),
                    activity_phase_style(&event.phase, theme),
                ),
                Span::raw(" "),
                Span::styled(step_label, theme.fg_dim),
                Span::raw(" "),
                Span::styled(event.message.clone(), theme.fg_active),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        if let Some(details) = event.details.as_deref() {
            push_wrapped_text(rows, details, theme.fg_dim, width, 4);
        }
        if !event.todo_snapshot.is_empty() {
            push_todo_items(rows, &event.todo_snapshot, theme, width, 4);
        }
    }

    if let Some(thread_id) = run.thread_id.as_deref() {
        if let Some(context) = tasks.work_context_for_thread(thread_id) {
            for entry in context
                .entries
                .iter()
                .filter(|entry| {
                    entry.goal_run_id.is_none()
                        || entry.goal_run_id.as_deref() == Some(run.id.as_str())
                })
                .take(3)
            {
                rendered_any = true;
                let label = entry
                    .change_kind
                    .as_deref()
                    .unwrap_or_else(|| work_kind_label(entry.kind));
                rows.push(RenderRow {
                    line: Line::from(vec![
                        Span::raw("  "),
                        Span::styled("↳ ", theme.accent_primary),
                        Span::styled(entry.source.clone(), theme.accent_primary),
                        Span::raw(" "),
                        Span::styled(format!("[{label}]"), theme.fg_dim),
                        Span::raw(" "),
                        Span::styled(
                            truncate_tail(&entry.path, width.saturating_sub(18).max(8)),
                            theme.fg_active,
                        ),
                    ]),
                    work_path: None,
                    goal_step_id: None,
                    close_preview: false,
                });
            }
        }

        if rendered_any && !tasks.todos_for_thread(thread_id).is_empty() {
            rows.push(RenderRow {
                line: Line::from(Span::styled("  Thread todos", theme.fg_dim)),
                work_path: None,
                goal_step_id: None,
                close_preview: false,
            });
            push_todo_items(rows, tasks.todos_for_thread(thread_id), theme, width, 4);
        }
    }
}

pub(crate) fn render_dossier(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
) {
    let Some(dossier) = run.dossier.as_ref() else {
        return;
    };

    push_section_title(
        rows,
        "Execution Dossier",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    rows.push(RenderRow {
        line: Line::from(vec![
            Span::styled("Projection: ", theme.fg_dim),
            Span::styled(projection_chip(&dossier.projection_state), theme.fg_active),
        ]),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
    if let Some(summary) = dossier.summary.as_deref() {
        push_wrapped_text(rows, summary, theme.fg_active, width, 0);
    }
    if let Some(error) = dossier.projection_error.as_deref() {
        push_wrapped_text(rows, error, theme.accent_danger, width, 0);
    }
}

pub(crate) fn render_delivery_units(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
) {
    let Some(dossier) = run.dossier.as_ref() else {
        return;
    };

    push_section_title(
        rows,
        "Delivery Units",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    if dossier.units.is_empty() {
        rows.push(RenderRow {
            line: Line::from(Span::styled(
                "No delivery units recorded yet.",
                theme.fg_dim,
            )),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        return;
    }

    for unit in &dossier.units {
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::styled(projection_chip(&unit.status), theme.fg_dim),
                Span::raw(" "),
                Span::styled(unit.title.clone(), theme.fg_active),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        push_wrapped_text(
            rows,
            &format!(
                "execute via {}  verify via {}",
                unit.execution_binding, unit.verification_binding
            ),
            theme.fg_dim,
            width,
            2,
        );
        if let Some(summary) = unit.summary.as_deref() {
            push_wrapped_text(rows, summary, theme.fg_active, width, 2);
        }
    }
}

pub(crate) fn render_proof_coverage(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
) {
    let Some(dossier) = run.dossier.as_ref() else {
        return;
    };

    push_section_title(
        rows,
        "Proof Coverage",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    let mut rendered_any = false;
    for unit in &dossier.units {
        if unit.proof_checks.is_empty() && unit.evidence.is_empty() {
            continue;
        }
        rendered_any = true;
        rows.push(RenderRow {
            line: Line::from(Span::styled(unit.title.clone(), theme.fg_active)),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        for check in &unit.proof_checks {
            rows.push(RenderRow {
                line: Line::from(vec![
                    Span::raw("  "),
                    Span::styled(projection_chip(&check.state), theme.fg_dim),
                    Span::raw(" "),
                    Span::styled(check.title.clone(), theme.fg_active),
                ]),
                work_path: None,
                goal_step_id: None,
                close_preview: false,
            });
            if let Some(summary) = check.summary.as_deref() {
                push_wrapped_text(rows, summary, theme.fg_dim, width, 4);
            }
        }
        for evidence in &unit.evidence {
            let label = if evidence.title.is_empty() {
                evidence.id.as_str()
            } else {
                evidence.title.as_str()
            };
            push_wrapped_text(rows, &format!("evidence: {label}"), theme.fg_dim, width, 4);
            if let Some(summary) = evidence.summary.as_deref() {
                push_wrapped_text(rows, summary, theme.fg_dim, width, 6);
            }
        }
    }
    if !rendered_any {
        rows.push(RenderRow {
            line: Line::from(Span::styled(
                "No proof checks or evidence yet.",
                theme.fg_dim,
            )),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
}

pub(crate) fn render_reports(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
) {
    let Some(dossier) = run.dossier.as_ref() else {
        return;
    };

    push_section_title(
        rows,
        "Reports",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    let mut rendered_any = false;
    if let Some(report) = dossier.report.as_ref() {
        rendered_any = true;
        push_wrapped_text(
            rows,
            &format!("goal [{}] {}", report.state, report.summary),
            theme.fg_active,
            width,
            0,
        );
    }
    for unit in &dossier.units {
        let Some(report) = unit.report.as_ref() else {
            continue;
        };
        rendered_any = true;
        push_wrapped_text(
            rows,
            &format!("{} [{}] {}", unit.title, report.state, report.summary),
            theme.fg_active,
            width,
            0,
        );
        for note in &report.notes {
            push_wrapped_text(rows, note, theme.fg_dim, width, 2);
        }
    }
    if !rendered_any {
        rows.push(RenderRow {
            line: Line::from(Span::styled("No reports recorded yet.", theme.fg_dim)),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
}

pub(crate) fn render_resume_decision(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
) {
    let Some(decision) = run
        .dossier
        .as_ref()
        .and_then(|dossier| dossier.latest_resume_decision.as_ref())
    else {
        return;
    };

    push_section_title(
        rows,
        "Resume Decision",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    push_wrapped_text(
        rows,
        &format!(
            "{} via {} ({})",
            decision.action, decision.reason_code, decision.projection_state
        ),
        theme.fg_active,
        width,
        0,
    );
    if let Some(reason) = decision.reason.as_deref() {
        push_wrapped_text(rows, reason, theme.fg_dim, width, 0);
    }
    for detail in &decision.details {
        push_wrapped_text(rows, detail, theme.fg_dim, width, 2);
    }
}

pub(crate) fn render_checkpoints(
    rows: &mut Vec<RenderRow>,
    tasks: &TaskState,
    run: &GoalRun,
    theme: &ThemeTokens,
    width: usize,
) {
    push_section_title(
        rows,
        "Checkpoints",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    let checkpoints = tasks.checkpoints_for_goal_run(&run.id);
    if checkpoints.is_empty() {
        rows.push(RenderRow {
            line: Line::from(Span::styled("No checkpoints recorded yet.", theme.fg_dim)),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        return;
    }

    for checkpoint in checkpoints.iter().take(6) {
        let step_label = checkpoint
            .step_index
            .map(|step_index| format!("step {}", step_index + 1))
            .unwrap_or_else(|| "step ?".to_string());
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::styled("[", theme.fg_dim),
                Span::styled(
                    checkpoint_type_label(&checkpoint.checkpoint_type),
                    theme.fg_active,
                ),
                Span::styled("]", theme.fg_dim),
                Span::raw(" "),
                Span::styled(step_label, theme.fg_dim),
                Span::raw("  "),
                Span::styled(format!("{} task(s)", checkpoint.task_count), theme.fg_dim),
                Span::raw("  "),
                Span::styled(short_checkpoint_id(&checkpoint.id), theme.accent_primary),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
        if let Some(preview) = checkpoint.context_summary_preview.as_deref() {
            push_wrapped_text(rows, preview, theme.fg_dim, width, 2);
        }
    }
}
