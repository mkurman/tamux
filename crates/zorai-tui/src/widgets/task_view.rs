#[path = "task_view_sections.rs"]
mod sections;
#[path = "task_view_selection.rs"]
mod selection;

#[path = "task_view_parts/build_rows_to_selection_point_from_mouse.rs"]
mod build_rows_to_selection_point_from_mouse;
#[path = "task_view_parts/content_inner_to_render_goal_summary.rs"]
mod content_inner_to_render_goal_summary;
#[path = "task_view_parts/render_goal_controls_to_render_goal_agents.rs"]
mod render_goal_controls_to_render_goal_agents;
#[path = "task_view_parts/selected_text_to_scrollbar_layout.rs"]
mod selected_text_to_scrollbar_layout;
#[path = "task_view_parts/task_view.rs"]
mod task_view;

pub(crate) use build_rows_to_selection_point_from_mouse::*;
pub(crate) use content_inner_to_render_goal_summary::*;
pub(crate) use render_goal_controls_to_render_goal_agents::*;
pub(crate) use selected_text_to_scrollbar_layout::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::sidebar::SidebarItemTarget;
    use crate::state::task::{AgentTask, GoalRun, GoalRunStep, TaskState, TaskStatus};
    use crate::theme::ThemeTokens;
    use ratatui::layout::{Position, Rect};

    #[test]
    fn hit_test_returns_goal_step_for_step_rows() {
        let mut tasks = TaskState::new();
        tasks.reduce(crate::state::task::TaskAction::GoalRunDetailReceived(
            GoalRun {
                id: "goal-1".to_string(),
                title: "Goal One".to_string(),
                steps: vec![
                    GoalRunStep {
                        id: "step-1".to_string(),
                        title: "Plan".to_string(),
                        order: 0,
                        ..Default::default()
                    },
                    GoalRunStep {
                        id: "step-2".to_string(),
                        title: "Execute".to_string(),
                        order: 1,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        ));

        let area = Rect::new(0, 0, 80, 20);
        let target = SidebarItemTarget::GoalRun {
            goal_run_id: "goal-1".to_string(),
            step_id: None,
        };

        let found = (area.y..area.y.saturating_add(area.height)).find_map(|row| {
            match hit_test(
                area,
                &tasks,
                &target,
                &ThemeTokens::default(),
                0,
                true,
                true,
                true,
                Position::new(area.x.saturating_add(2), row),
            ) {
                Some(TaskViewHitTarget::GoalStep(step_id)) if step_id == "step-2" => Some(step_id),
                _ => None,
            }
        });

        assert_eq!(found.as_deref(), Some("step-2"));
    }

    #[test]
    fn hit_test_returns_back_to_goal_for_task_navigation_row() {
        let mut tasks = TaskState::new();
        tasks.reduce(crate::state::task::TaskAction::TaskListReceived(vec![
            AgentTask {
                id: "task-1".to_string(),
                title: "Child Task".to_string(),
                goal_run_id: Some("goal-1".to_string()),
                ..Default::default()
            },
        ]));
        tasks.reduce(crate::state::task::TaskAction::GoalRunDetailReceived(
            GoalRun {
                id: "goal-1".to_string(),
                title: "Goal One".to_string(),
                goal: "Goal body".to_string(),
                steps: vec![GoalRunStep {
                    id: "step-1".to_string(),
                    title: "Plan".to_string(),
                    task_id: Some("task-1".to_string()),
                    order: 0,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ));

        let area = Rect::new(0, 0, 80, 20);
        let target = SidebarItemTarget::Task {
            task_id: "task-1".to_string(),
        };

        let found = (area.y..area.y.saturating_add(area.height)).find_map(|row| {
            match hit_test(
                area,
                &tasks,
                &target,
                &ThemeTokens::default(),
                0,
                true,
                true,
                true,
                Position::new(area.x.saturating_add(2), row),
            ) {
                Some(TaskViewHitTarget::BackToGoal) => Some(row),
                _ => None,
            }
        });

        assert!(
            found.is_some(),
            "expected task view navigation row to be clickable"
        );
    }

    #[test]
    fn goal_run_rows_include_usage_and_agents() {
        let mut tasks = TaskState::new();
        tasks.reduce(crate::state::task::TaskAction::TaskListReceived(vec![
            AgentTask {
                id: "task-root".to_string(),
                title: "Root implementation".to_string(),
                goal_run_id: Some("goal-usage".to_string()),
                status: Some(TaskStatus::Completed),
                ..Default::default()
            },
            AgentTask {
                id: "task-review".to_string(),
                title: "Verifier subagent".to_string(),
                goal_run_id: Some("goal-usage".to_string()),
                parent_task_id: Some("task-root".to_string()),
                status: Some(TaskStatus::Completed),
                ..Default::default()
            },
        ]));
        tasks.reduce(crate::state::task::TaskAction::GoalRunDetailReceived(
            GoalRun {
                id: "goal-usage".to_string(),
                title: "Token accounting".to_string(),
                goal: "Show model usage".to_string(),
                total_prompt_tokens: 1234,
                total_completion_tokens: 567,
                estimated_cost_usd: Some(0.0425),
                planner_owner_profile: Some(crate::state::task::GoalRuntimeOwnerProfile {
                    agent_label: "Svarog".to_string(),
                    provider: "openai".to_string(),
                    model: "gpt-5.4".to_string(),
                    reasoning_effort: None,
                }),
                runtime_assignment_list: vec![crate::state::task::GoalAgentAssignment {
                    role_id: "weles".to_string(),
                    enabled: true,
                    provider: "openrouter".to_string(),
                    model: "anthropic/claude-sonnet-4".to_string(),
                    reasoning_effort: Some("high".to_string()),
                    inherit_from_main: false,
                }],
                model_usage: vec![crate::state::task::GoalRunModelUsage {
                    provider: "openrouter".to_string(),
                    model: "anthropic/claude-sonnet-4".to_string(),
                    request_count: 2,
                    prompt_tokens: 1000,
                    completion_tokens: 500,
                    estimated_cost_usd: Some(0.04),
                    duration_ms: Some(90_000),
                }],
                ..Default::default()
            },
        ));

        let target = SidebarItemTarget::GoalRun {
            goal_run_id: "goal-usage".to_string(),
            step_id: None,
        };
        let text = rows_for_width(
            &tasks,
            &target,
            &ThemeTokens::default(),
            120,
            true,
            true,
            true,
            None,
        )
        .into_iter()
        .map(|row| selection::line_plain_text(&row.line))
        .collect::<Vec<_>>()
        .join("\n");

        assert!(text.contains("Usage"), "{text}");
        assert!(text.contains("prompt 1,234"), "{text}");
        assert!(text.contains("completion 567"), "{text}");
        assert!(text.contains("$0.0425"), "{text}");
        assert!(
            text.contains("openrouter/anthropic/claude-sonnet-4"),
            "{text}"
        );
        assert!(text.contains("2 req"), "{text}");
        assert!(text.contains("Agents"), "{text}");
        assert!(text.contains("Planner Svarog"), "{text}");
        assert!(text.contains("Role weles"), "{text}");
        assert!(text.contains("Task Root implementation"), "{text}");
        assert!(text.contains("Subagent Verifier subagent"), "{text}");
    }
}
