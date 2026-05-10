use super::*;
use crate::state::task::*;
use crate::theme::ThemeTokens;
use crate::widgets::duration_format::format_duration_ms;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
pub(crate) fn render_goal_controls(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    step_id: Option<&str>,
    theme: &ThemeTokens,
    _width: usize,
) {
    let mut controls: Vec<(&str, &str, &str, Style)> = Vec::new();

    match run.status {
        Some(GoalRunStatus::Paused) => {
            controls.push(("▶", "Resume", "Ctrl+S", theme.accent_success))
        }
        Some(GoalRunStatus::Queued)
        | Some(GoalRunStatus::Planning)
        | Some(GoalRunStatus::Running)
        | Some(GoalRunStatus::AwaitingApproval) => {
            controls.push(("⏸", "Pause", "Ctrl+S", theme.accent_secondary))
        }
        _ => {}
    }

    let has_goal_actions = matches!(
        run.status,
        Some(GoalRunStatus::Paused)
            | Some(GoalRunStatus::Queued)
            | Some(GoalRunStatus::Planning)
            | Some(GoalRunStatus::Running)
            | Some(GoalRunStatus::AwaitingApproval)
    );
    if has_goal_actions || !run.steps.is_empty() {
        controls.push(("☰", "Actions", "A", theme.accent_primary));
    }

    let has_step_context = step_id.is_some()
        || !run.steps.is_empty()
            && (run
                .steps
                .iter()
                .any(|step| step.order as usize == run.current_step_index)
                || run.current_step_title.is_some()
                || !run.steps.is_empty());
    if has_step_context {
        controls.push(("↻", "Retry step", "R", theme.accent_secondary));
        controls.push(("⟲", "Rerun from here", "Ctrl+R", theme.accent_primary));
    }
    if has_goal_actions {
        controls.push(("⟳", "Refresh goal", "Shift+R", theme.accent_primary));
    }

    if has_goal_actions || has_step_context {
        controls.push(("✦", "Mission Control", "M", theme.accent_primary));
    }

    if controls.is_empty() {
        return;
    }

    push_section_title(
        rows,
        "Controls",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    for (icon, label, key, style) in controls {
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::styled(
                    format!("[{icon} {label}]"),
                    style.add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(key.to_string(), theme.fg_dim),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
}

pub(crate) fn format_count(value: u64) -> String {
    let raw = value.to_string();
    let mut grouped = String::new();
    for (index, ch) in raw.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    grouped.chars().rev().collect()
}

pub(crate) fn format_cost(cost: f64) -> String {
    if cost.abs() >= 1.0 {
        format!("${cost:.2}")
    } else {
        format!("${cost:.4}")
    }
}

pub(crate) fn has_goal_usage(run: &GoalRun) -> bool {
    run.total_prompt_tokens > 0
        || run.total_completion_tokens > 0
        || run.estimated_cost_usd.is_some()
        || !run.model_usage.is_empty()
}

pub(crate) fn render_goal_usage(
    rows: &mut Vec<RenderRow>,
    run: &GoalRun,
    theme: &ThemeTokens,
    _width: usize,
) {
    if !has_goal_usage(run) {
        return;
    }

    push_section_title(
        rows,
        "Usage",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );
    let mut aggregate = vec![
        Span::styled("prompt ", theme.fg_dim),
        Span::styled(format_count(run.total_prompt_tokens), theme.fg_active),
        Span::styled("  completion ", theme.fg_dim),
        Span::styled(format_count(run.total_completion_tokens), theme.fg_active),
    ];
    if let Some(cost) = run.estimated_cost_usd {
        aggregate.push(Span::styled("  cost ", theme.fg_dim));
        aggregate.push(Span::styled(format_cost(cost), theme.fg_active));
    }
    rows.push(RenderRow {
        line: Line::from(aggregate),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });

    for usage in &run.model_usage {
        render_goal_model_usage(rows, usage, theme);
    }
}

pub(crate) fn render_goal_model_usage(
    rows: &mut Vec<RenderRow>,
    usage: &GoalRunModelUsage,
    theme: &ThemeTokens,
) {
    let mut spans = vec![
        Span::styled(
            format!("{}/{}", usage.provider, usage.model),
            theme.fg_active,
        ),
        Span::styled("  ", theme.fg_dim),
        Span::styled(format!("{} req", usage.request_count), theme.fg_dim),
        Span::styled("  in ", theme.fg_dim),
        Span::styled(format_count(usage.prompt_tokens), theme.fg_dim),
        Span::styled("  out ", theme.fg_dim),
        Span::styled(format_count(usage.completion_tokens), theme.fg_dim),
    ];
    if let Some(cost) = usage.estimated_cost_usd {
        spans.push(Span::styled("  ", theme.fg_dim));
        spans.push(Span::styled(format_cost(cost), theme.fg_dim));
    }
    if let Some(duration_ms) = usage.duration_ms {
        spans.push(Span::styled("  ", theme.fg_dim));
        spans.push(Span::styled(format_duration_ms(duration_ms), theme.fg_dim));
    }
    rows.push(RenderRow {
        line: Line::from(spans),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
}

pub(crate) fn owner_profile_key(label: &str, provider: &str, model: &str) -> String {
    format!("{label}\n{provider}\n{model}")
}

pub(crate) fn render_owner_profile(
    rows: &mut Vec<RenderRow>,
    label: &str,
    profile: &GoalRuntimeOwnerProfile,
    theme: &ThemeTokens,
) {
    let mut spans = vec![
        Span::styled(format!("{label} "), theme.fg_dim),
        Span::styled(profile.agent_label.clone(), theme.fg_active),
        Span::styled("  ", theme.fg_dim),
        Span::styled(
            format!("{}/{}", profile.provider, profile.model),
            theme.fg_dim,
        ),
    ];
    if let Some(reasoning_effort) = &profile.reasoning_effort {
        spans.push(Span::styled("  ", theme.fg_dim));
        spans.push(Span::styled(reasoning_effort.clone(), theme.fg_dim));
    }
    rows.push(RenderRow {
        line: Line::from(spans),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
}

pub(crate) fn render_goal_assignment(
    rows: &mut Vec<RenderRow>,
    assignment: &GoalAgentAssignment,
    theme: &ThemeTokens,
) {
    let mut spans = vec![
        Span::styled("Role ", theme.fg_dim),
        Span::styled(assignment.role_id.clone(), theme.fg_active),
    ];
    if assignment.inherit_from_main {
        spans.push(Span::styled("  inherits main", theme.fg_dim));
    } else {
        spans.push(Span::styled("  ", theme.fg_dim));
        spans.push(Span::styled(
            format!("{}/{}", assignment.provider, assignment.model),
            theme.fg_dim,
        ));
    }
    if let Some(reasoning_effort) = &assignment.reasoning_effort {
        spans.push(Span::styled("  ", theme.fg_dim));
        spans.push(Span::styled(reasoning_effort.clone(), theme.fg_dim));
    }
    if !assignment.enabled {
        spans.push(Span::styled("  disabled", theme.fg_dim));
    }
    rows.push(RenderRow {
        line: Line::from(spans),
        work_path: None,
        goal_step_id: None,
        close_preview: false,
    });
}

pub(crate) fn render_goal_agents(
    rows: &mut Vec<RenderRow>,
    tasks: &TaskState,
    run: &GoalRun,
    theme: &ThemeTokens,
    _width: usize,
) {
    let related_tasks: Vec<&AgentTask> = tasks
        .tasks()
        .iter()
        .filter(|task| task.goal_run_id.as_deref() == Some(run.id.as_str()))
        .collect();
    if run.planner_owner_profile.is_none()
        && run.current_step_owner_profile.is_none()
        && run.runtime_assignment_list.is_empty()
        && run.launch_assignment_snapshot.is_empty()
        && related_tasks.is_empty()
    {
        return;
    }

    push_section_title(
        rows,
        "Agents",
        theme.accent_primary.add_modifier(Modifier::BOLD),
    );

    let mut owner_keys = Vec::new();
    if let Some(profile) = &run.planner_owner_profile {
        owner_keys.push(owner_profile_key(
            "Planner",
            &profile.provider,
            &profile.model,
        ));
        render_owner_profile(rows, "Planner", profile, theme);
    }
    if let Some(profile) = &run.current_step_owner_profile {
        let key = owner_profile_key("Current", &profile.provider, &profile.model);
        if !owner_keys.contains(&key) {
            owner_keys.push(key);
            render_owner_profile(rows, "Current", profile, theme);
        }
    }

    let assignments = if run.runtime_assignment_list.is_empty() {
        &run.launch_assignment_snapshot
    } else {
        &run.runtime_assignment_list
    };
    let mut assignment_keys = Vec::new();
    for assignment in assignments {
        let key = format!(
            "{}\n{}\n{}\n{}",
            assignment.role_id, assignment.provider, assignment.model, assignment.inherit_from_main
        );
        if assignment_keys.contains(&key) {
            continue;
        }
        assignment_keys.push(key);
        render_goal_assignment(rows, assignment, theme);
    }

    for task in related_tasks {
        let kind = if task.parent_task_id.is_some() || task.parent_thread_id.is_some() {
            "Subagent"
        } else {
            "Task"
        };
        rows.push(RenderRow {
            line: Line::from(vec![
                Span::styled(format!("{kind} "), theme.fg_dim),
                Span::styled(task.title.clone(), theme.fg_active),
                Span::styled("  ", theme.fg_dim),
                Span::styled(task_status_label(task.status), theme.fg_dim),
            ]),
            work_path: None,
            goal_step_id: None,
            close_preview: false,
        });
    }
}
