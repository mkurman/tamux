use super::*;
use crate::widgets;
use ratatui::widgets::{Block, Borders};
impl TuiModel {
    pub(super) fn mission_control_navigation_state(&self) -> MissionControlNavigationState {
        self.mission_control_navigation.clone()
    }

    pub(super) fn update_mission_control_navigation_state(
        &mut self,
        update: impl FnOnce(&mut MissionControlNavigationState),
    ) {
        update(&mut self.mission_control_navigation);
    }

    pub(super) fn mission_control_source_goal_target(&self) -> Option<sidebar::SidebarItemTarget> {
        self.mission_control_navigation_state().source_goal_target
    }

    pub(super) fn set_mission_control_source_goal_target(
        &mut self,
        target: Option<sidebar::SidebarItemTarget>,
    ) {
        self.update_mission_control_navigation_state(|state| {
            state.source_goal_target = target;
        });
    }

    pub(crate) fn mission_control_return_to_goal_target(
        &self,
    ) -> Option<sidebar::SidebarItemTarget> {
        self.mission_control_navigation_state()
            .return_to_goal_target
    }

    pub(crate) fn set_mission_control_return_to_goal_target(
        &mut self,
        target: Option<sidebar::SidebarItemTarget>,
    ) {
        self.update_mission_control_navigation_state(|state| {
            state.return_to_goal_target = target;
        });
    }

    pub(crate) fn mission_control_return_to_thread_id(&self) -> Option<String> {
        self.mission_control_navigation_state().return_to_thread_id
    }

    pub(crate) fn set_mission_control_return_to_thread_id(&mut self, thread_id: Option<String>) {
        self.update_mission_control_navigation_state(|state| {
            state.return_to_thread_id = thread_id;
        });
    }

    pub(crate) fn mission_control_return_to_workspace(&self) -> bool {
        self.mission_control_navigation_state().return_to_workspace
    }

    pub(crate) fn set_mission_control_return_to_workspace(&mut self, return_to_workspace: bool) {
        self.update_mission_control_navigation_state(|state| {
            state.return_to_workspace = return_to_workspace;
        });
    }

    pub(crate) fn clear_mission_control_return_context(&mut self) {
        self.set_mission_control_return_to_goal_target(None);
        self.set_mission_control_return_to_thread_id(None);
        self.set_mission_control_return_to_workspace(false);
    }

    pub(crate) fn current_goal_return_target(&self) -> Option<sidebar::SidebarItemTarget> {
        self.mission_control_return_to_goal_target().or_else(|| {
            if matches!(self.main_pane_view, MainPaneView::GoalComposer) {
                self.mission_control_source_goal_target()
            } else {
                self.current_goal_target_for_mission_control()
            }
        })
    }

    pub(crate) fn set_mission_control_return_targets(
        &mut self,
        goal_target: Option<sidebar::SidebarItemTarget>,
        thread_id: Option<String>,
    ) {
        self.set_mission_control_return_to_goal_target(goal_target);
        self.set_mission_control_return_to_thread_id(thread_id);
        self.set_mission_control_return_to_workspace(false);
    }

    pub(crate) fn has_mission_control_return_target(&self) -> bool {
        self.mission_control_return_to_thread_id().is_some()
            || self.mission_control_return_to_goal_target().is_some()
            || self.mission_control_return_to_workspace()
    }

    pub(crate) fn open_work_context_for_thread(
        &mut self,
        thread_id: String,
        path: Option<String>,
        parent_thread_id: Option<String>,
        goal_target: Option<sidebar::SidebarItemTarget>,
        status_line: String,
    ) {
        self.set_mission_control_return_targets(goal_target, parent_thread_id);
        self.main_pane_view = MainPaneView::WorkContext;
        self.task_view_scroll = 0;
        self.focus = FocusArea::Chat;
        if let Some(path) = path {
            self.tasks.reduce(task::TaskAction::SelectWorkPath {
                thread_id: thread_id.clone(),
                path: Some(path),
            });
        }
        self.request_preview_for_selected_path(&thread_id);
        self.status_line = status_line;
    }

    pub(super) fn current_goal_target_for_mission_control(
        &self,
    ) -> Option<sidebar::SidebarItemTarget> {
        match &self.main_pane_view {
            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                goal_run_id,
                step_id,
            }) => Some(sidebar::SidebarItemTarget::GoalRun {
                goal_run_id: goal_run_id.clone(),
                step_id: step_id.clone(),
            }),
            MainPaneView::Task(sidebar::SidebarItemTarget::Task { task_id }) => {
                self.parent_goal_target_for_task(task_id)
            }
            _ => None,
        }
    }

    pub(crate) fn mission_control_goal_run(&self) -> Option<&task::GoalRun> {
        let target = self.mission_control_source_goal_target()?;
        let goal_run_id = target_goal_run_id(self, &target)?;
        self.tasks.goal_run_by_id(&goal_run_id)
    }

    pub(crate) fn mission_control_thread_target(&self) -> Option<(String, bool)> {
        let run = self.mission_control_goal_run()?;
        run.active_thread_id
            .clone()
            .map(|thread_id| (thread_id, false))
            .or_else(|| {
                run.root_thread_id
                    .clone()
                    .map(|thread_id| (thread_id, true))
            })
    }

    pub(crate) fn goal_prompt_thread_target(&self) -> Option<(sidebar::SidebarItemTarget, String)> {
        let target = self.current_goal_target_for_mission_control()?;
        let goal_run_id = target_goal_run_id(self, &target)?;
        let run = self.tasks.goal_run_by_id(&goal_run_id)?;
        let thread_id = run
            .thread_id
            .clone()
            .or_else(|| run.root_thread_id.clone())
            .or_else(|| run.active_thread_id.clone())
            .or_else(|| run.execution_thread_ids.first().cloned())
            .or_else(|| {
                self.tasks
                    .tasks()
                    .iter()
                    .find(|task| task.goal_run_id.as_deref() == Some(run.id.as_str()))
                    .and_then(|task| task.thread_id.clone())
            })?;
        Some((target, thread_id))
    }

    pub(crate) fn mission_control_has_thread_target(&self) -> bool {
        self.mission_control_thread_target().is_some()
    }

    fn runtime_assignments_for_goal_run(
        &self,
        run: &task::GoalRun,
    ) -> (Vec<task::GoalAgentAssignment>, bool) {
        if !run.runtime_assignment_list.is_empty() {
            (run.runtime_assignment_list.clone(), false)
        } else if !run.launch_assignment_snapshot.is_empty() {
            (run.launch_assignment_snapshot.clone(), true)
        } else {
            let fallback_profile = self.current_conversation_agent_profile();
            (
                vec![task::GoalAgentAssignment {
                    role_id: zorai_protocol::AGENT_ID_SWAROG.to_string(),
                    enabled: true,
                    provider: fallback_profile.provider,
                    model: fallback_profile.model,
                    reasoning_effort: fallback_profile.reasoning_effort,
                    inherit_from_main: false,
                }],
                true,
            )
        }
    }

    fn runtime_assignment_matches_owner(
        assignment: &task::GoalAgentAssignment,
        owner: &task::GoalRuntimeOwnerProfile,
    ) -> bool {
        assignment.provider == owner.provider
            && assignment.model == owner.model
            && assignment.reasoning_effort == owner.reasoning_effort
    }

    fn active_runtime_assignment_index_for_run(
        &self,
        run: &task::GoalRun,
        assignments: &[task::GoalAgentAssignment],
    ) -> Option<usize> {
        run.current_step_owner_profile
            .as_ref()
            .or(run.planner_owner_profile.as_ref())
            .and_then(|owner| {
                assignments.iter().position(|assignment| {
                    Self::runtime_assignment_matches_owner(assignment, owner)
                })
            })
    }

    pub(crate) fn sync_goal_mission_control_from_run(
        &mut self,
        run: &task::GoalRun,
        preserve_pending: bool,
    ) {
        let (assignments, uses_fallback) = self.runtime_assignments_for_goal_run(run);
        let active_index = self.active_runtime_assignment_index_for_run(run, &assignments);
        let preserved_pending = preserve_pending
            && self.goal_mission_control.runtime_goal_run_id.as_deref() == Some(run.id.as_str())
            && self.goal_mission_control.pending_role_assignments.is_some();
        let preserved_overlay = preserved_pending
            .then(|| self.goal_mission_control.pending_role_assignments.clone())
            .flatten();
        let preserved_modes = preserved_pending
            .then(|| {
                self.goal_mission_control
                    .pending_runtime_apply_modes
                    .clone()
            })
            .unwrap_or_default();
        let preserved_selection = preserved_pending
            .then_some(self.goal_mission_control.selected_runtime_assignment_index);

        self.goal_mission_control.configure_runtime_assignments(
            run.id.clone(),
            assignments,
            active_index,
            uses_fallback,
        );
        if let Some(overlay) = preserved_overlay {
            self.goal_mission_control.pending_role_assignments = Some(overlay);
            self.goal_mission_control.pending_runtime_apply_modes = preserved_modes;
        }
        if let Some(selection) = preserved_selection {
            self.goal_mission_control
                .set_selected_runtime_assignment_index(selection);
        }
    }

    pub(crate) fn sync_goal_mission_control_from_selected_goal_run(&mut self) -> bool {
        let Some(run) = self.selected_goal_run().cloned() else {
            return false;
        };
        self.set_mission_control_source_goal_target(self.current_goal_target_for_mission_control());
        self.sync_goal_mission_control_from_run(&run, true);
        true
    }

    pub(crate) fn sidebar_uses_goal_sidebar(&self) -> bool {
        matches!(
            self.main_pane_view,
            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
        ) && self.sidebar_visible()
    }

    pub(crate) fn active_goal_sidebar_goal_run(&self) -> Option<&str> {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return None;
        };
        Some(goal_run_id.as_str())
    }

    pub(super) fn goal_sidebar_item_count(&self) -> usize {
        self.active_goal_sidebar_goal_run()
            .map(|goal_run_id| {
                self.goal_sidebar_item_count_for_tab(goal_run_id, self.goal_sidebar.active_tab())
            })
            .unwrap_or(0)
    }

    pub(super) fn goal_workspace_plan_items(
        &self,
    ) -> Vec<crate::state::goal_workspace::GoalPlanSelection> {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return Vec::new();
        };
        widgets::goal_workspace::plan_selection_rows(&self.tasks, goal_run_id, &self.goal_workspace)
            .into_iter()
            .map(|(_, selection)| selection)
            .collect()
    }

    pub(crate) fn sync_goal_workspace_selection_for_active_goal_pane(&mut self) {
        let items = self.goal_workspace_plan_items();
        if items.is_empty() {
            self.goal_workspace.set_selected_plan_row(0);
            self.goal_workspace.set_selected_plan_item(None);
            self.goal_workspace.set_plan_scroll(0);
            return;
        }

        let current_step_selection = match &self.main_pane_view {
            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                step_id: Some(step_id),
                ..
            }) => Some(crate::state::goal_workspace::GoalPlanSelection::Step {
                step_id: step_id.clone(),
            }),
            _ => None,
        };

        let target = self
            .goal_workspace
            .selected_plan_item()
            .cloned()
            .filter(|item| items.contains(item))
            .or_else(|| current_step_selection.filter(|item| items.contains(item)))
            .unwrap_or_else(|| items[0].clone());

        let row = items.iter().position(|item| *item == target).unwrap_or(0);
        self.goal_workspace.set_selected_plan_row(row);
        self.goal_workspace.set_selected_plan_item(Some(target));
        self.clamp_goal_workspace_plan_scroll_to_selection();
    }

    pub(crate) fn select_goal_workspace_plan_item(
        &mut self,
        item: crate::state::goal_workspace::GoalPlanSelection,
    ) -> bool {
        let changed = match &item {
            crate::state::goal_workspace::GoalPlanSelection::Step { step_id }
            | crate::state::goal_workspace::GoalPlanSelection::Todo { step_id, .. } => {
                self.select_goal_step_in_active_run(step_id.clone())
            }
            crate::state::goal_workspace::GoalPlanSelection::PromptToggle
            | crate::state::goal_workspace::GoalPlanSelection::MainThread { .. } => false,
        };
        let items = self.goal_workspace_plan_items();
        let row = items
            .iter()
            .position(|candidate| candidate == &item)
            .unwrap_or(0);
        self.goal_workspace.set_selected_plan_row(row);
        self.goal_workspace.set_selected_plan_item(Some(item));
        self.clamp_goal_workspace_plan_scroll_to_selection();
        changed
    }

    pub(super) fn clamp_goal_workspace_plan_scroll_to_selection(&mut self) {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return;
        };

        let area = self.pane_layout().chat;
        let (viewport_height, viewport_width) = {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(4), Constraint::Min(1)])
                .split(area);
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(32),
                    Constraint::Min(24),
                ])
                .split(layout[1]);
            let inner = Block::default().borders(Borders::ALL).inner(columns[0]);
            (inner.height as usize, inner.width as usize)
        };
        if viewport_height == 0 {
            self.goal_workspace.set_plan_scroll(0);
            return;
        }

        let max_scroll = widgets::goal_workspace::max_plan_scroll(
            area,
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
        );
        let selected_row = widgets::goal_workspace::plan_visual_row_for_selection(
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
            viewport_width,
        )
        .unwrap_or(0);
        let current_scroll = self.goal_workspace.plan_scroll().min(max_scroll);
        let next_scroll = if selected_row < current_scroll {
            selected_row
        } else if selected_row >= current_scroll.saturating_add(viewport_height) {
            selected_row
                .saturating_add(1)
                .saturating_sub(viewport_height)
        } else {
            current_scroll
        };
        self.goal_workspace
            .set_plan_scroll(next_scroll.min(max_scroll));
    }

    pub(crate) fn step_goal_workspace_plan_selection(&mut self, delta: i32) -> bool {
        self.sync_goal_workspace_selection_for_active_goal_pane();
        let items = self.goal_workspace_plan_items();
        if items.is_empty() {
            return false;
        }
        let current = self.goal_workspace.selected_plan_row().min(items.len() - 1);
        let next = if delta >= 0 {
            (current + delta as usize).min(items.len() - 1)
        } else {
            current.saturating_sub((-delta) as usize)
        };
        self.select_goal_workspace_plan_item(items[next].clone())
    }

    pub(crate) fn activate_goal_workspace_plan_target(&mut self) -> bool {
        let Some(selection) = self.goal_workspace.selected_plan_item().cloned() else {
            return false;
        };
        match selection {
            crate::state::goal_workspace::GoalPlanSelection::PromptToggle => {
                self.goal_workspace.toggle_prompt_expanded();
                self.sync_goal_workspace_selection_for_active_goal_pane();
                self.clamp_goal_workspace_plan_scroll_to_selection();
                true
            }
            crate::state::goal_workspace::GoalPlanSelection::MainThread { thread_id } => {
                self.open_thread_conversation(thread_id);
                true
            }
            crate::state::goal_workspace::GoalPlanSelection::Step { step_id } => {
                self.goal_workspace.toggle_step_expanded(step_id);
                self.sync_goal_workspace_selection_for_active_goal_pane();
                self.clamp_goal_workspace_plan_scroll_to_selection();
                true
            }
            crate::state::goal_workspace::GoalPlanSelection::Todo { .. } => false,
        }
    }

    pub(crate) fn expand_selected_goal_workspace_step(&mut self) -> bool {
        self.sync_goal_workspace_selection_for_active_goal_pane();
        let Some(selection) = self.goal_workspace.selected_plan_item().cloned() else {
            return false;
        };
        let crate::state::goal_workspace::GoalPlanSelection::Step { step_id } = selection else {
            return false;
        };
        self.goal_workspace.set_step_expanded(step_id, true);
        self.sync_goal_workspace_selection_for_active_goal_pane();
        true
    }

    pub(crate) fn collapse_goal_workspace_selection(&mut self) -> bool {
        self.sync_goal_workspace_selection_for_active_goal_pane();
        let Some(selection) = self.goal_workspace.selected_plan_item().cloned() else {
            return false;
        };
        match selection {
            crate::state::goal_workspace::GoalPlanSelection::PromptToggle
            | crate::state::goal_workspace::GoalPlanSelection::MainThread { .. } => false,
            crate::state::goal_workspace::GoalPlanSelection::Step { step_id } => {
                if self.goal_workspace.is_step_expanded(&step_id) {
                    self.goal_workspace.set_step_expanded(step_id, false);
                    self.sync_goal_workspace_selection_for_active_goal_pane();
                    true
                } else {
                    false
                }
            }
            crate::state::goal_workspace::GoalPlanSelection::Todo { step_id, .. } => self
                .select_goal_workspace_plan_item(
                    crate::state::goal_workspace::GoalPlanSelection::Step { step_id },
                ),
        }
    }
}
