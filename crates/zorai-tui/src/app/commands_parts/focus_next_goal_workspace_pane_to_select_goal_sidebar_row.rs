use super::*;
use crate::widgets;
use ratatui::widgets::{Block, Borders};
impl TuiModel {
    pub(super) fn focus_next_goal_workspace_pane(&mut self) -> bool {
        if !matches!(
            self.main_pane_view,
            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
        ) || self.focus != FocusArea::Chat
        {
            return false;
        }

        match self.goal_workspace.focused_pane() {
            crate::state::goal_workspace::GoalWorkspacePane::CommandBar => false,
            crate::state::goal_workspace::GoalWorkspacePane::Plan => {
                self.goal_workspace
                    .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::Timeline);
                true
            }
            crate::state::goal_workspace::GoalWorkspacePane::Timeline => {
                self.goal_workspace
                    .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::Details);
                true
            }
            crate::state::goal_workspace::GoalWorkspacePane::Details => {
                self.goal_workspace
                    .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::CommandBar);
                true
            }
        }
    }

    pub(super) fn focus_prev_goal_workspace_pane(&mut self) -> bool {
        if !matches!(
            self.main_pane_view,
            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
        ) || self.focus != FocusArea::Chat
        {
            return false;
        }

        match self.goal_workspace.focused_pane() {
            crate::state::goal_workspace::GoalWorkspacePane::Plan => {
                self.goal_workspace
                    .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::CommandBar);
                true
            }
            crate::state::goal_workspace::GoalWorkspacePane::Timeline => {
                self.goal_workspace
                    .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::Plan);
                true
            }
            crate::state::goal_workspace::GoalWorkspacePane::Details => {
                self.goal_workspace
                    .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::Timeline);
                true
            }
            crate::state::goal_workspace::GoalWorkspacePane::CommandBar => {
                self.goal_workspace
                    .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::Details);
                true
            }
        }
    }

    pub(crate) fn set_goal_workspace_mode(
        &mut self,
        mode: crate::state::goal_workspace::GoalWorkspaceMode,
    ) -> bool {
        if !matches!(
            self.main_pane_view,
            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
        ) {
            return false;
        }
        let changed = self.goal_workspace.mode() != mode;
        self.goal_workspace.set_mode(mode);
        self.goal_workspace.set_selected_timeline_row(0);
        self.goal_workspace.set_selected_detail_row(0);
        self.goal_workspace.set_timeline_scroll(0);
        self.goal_workspace.set_detail_scroll(0);
        changed
    }

    pub(crate) fn cycle_goal_workspace_mode(&mut self, delta: i32) -> bool {
        let modes = [
            crate::state::goal_workspace::GoalWorkspaceMode::Goal,
            crate::state::goal_workspace::GoalWorkspaceMode::Files,
            crate::state::goal_workspace::GoalWorkspaceMode::Progress,
            crate::state::goal_workspace::GoalWorkspaceMode::Usage,
            crate::state::goal_workspace::GoalWorkspaceMode::ActiveAgent,
            crate::state::goal_workspace::GoalWorkspaceMode::Threads,
            crate::state::goal_workspace::GoalWorkspaceMode::NeedsAttention,
        ];
        let current = modes
            .iter()
            .position(|mode| *mode == self.goal_workspace.mode())
            .unwrap_or(0);
        let next = if delta >= 0 {
            (current + delta as usize).min(modes.len() - 1)
        } else {
            current.saturating_sub((-delta) as usize)
        };
        self.set_goal_workspace_mode(modes[next])
    }

    pub(crate) fn activate_goal_workspace_command_bar(&mut self) -> bool {
        if self.goal_workspace.focused_pane()
            != crate::state::goal_workspace::GoalWorkspacePane::CommandBar
        {
            return false;
        }
        self.goal_workspace
            .set_focused_pane(crate::state::goal_workspace::GoalWorkspacePane::Plan);
        true
    }

    pub(crate) fn step_goal_workspace_timeline_selection(&mut self, delta: i32) -> bool {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return false;
        };
        let row_count = widgets::goal_workspace::timeline_row_count(
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
        );
        if row_count == 0 {
            self.goal_workspace.set_selected_timeline_row(0);
            return false;
        }
        let current = self
            .goal_workspace
            .selected_timeline_row()
            .min(row_count - 1);
        let next = if delta >= 0 {
            (current + delta as usize).min(row_count - 1)
        } else {
            current.saturating_sub((-delta) as usize)
        };
        let changed = next != current;
        self.goal_workspace.set_selected_timeline_row(next);
        self.clamp_goal_workspace_timeline_scroll_to_selection();
        changed
    }

    pub(super) fn selected_goal_workspace_timeline_target(
        &self,
    ) -> Option<crate::widgets::goal_workspace::GoalWorkspaceHitTarget> {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return None;
        };
        crate::widgets::goal_workspace::timeline_targets(
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
        )
        .into_iter()
        .find_map(|(index, target)| {
            (index == self.goal_workspace.selected_timeline_row()).then_some(target)
        })
    }

    pub(crate) fn activate_goal_workspace_timeline_target(&mut self) -> bool {
        let Some(target) = self.selected_goal_workspace_timeline_target() else {
            return false;
        };
        match target {
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::DetailFile(path) => {
                self.open_file_preview_path(path.clone());
                self.status_line = path;
                true
            }
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::ThreadRow(thread_id) => {
                self.open_thread_conversation(thread_id);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn step_goal_workspace_detail_selection(&mut self, delta: i32) -> bool {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return false;
        };
        let row_count = widgets::goal_workspace::detail_target_count(
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
        );
        if row_count == 0 {
            self.goal_workspace.set_selected_detail_row(0);
            return false;
        }
        let current = self.goal_workspace.selected_detail_row().min(row_count - 1);
        let next = if delta >= 0 {
            (current + delta as usize).min(row_count - 1)
        } else {
            current.saturating_sub((-delta) as usize)
        };
        let changed = next != current;
        self.goal_workspace.set_selected_detail_row(next);
        self.clamp_goal_workspace_detail_scroll_to_selection();
        changed
    }

    pub(super) fn clamp_goal_workspace_timeline_scroll_to_selection(&mut self) {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return;
        };
        let area = self.pane_layout().chat;
        let viewport_height = widgets::goal_workspace::timeline_viewport_height(area);
        let viewport_width = {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(area);
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(32),
                    Constraint::Min(24),
                ])
                .split(layout[1]);
            Block::default()
                .borders(Borders::ALL)
                .inner(columns[1])
                .width as usize
        };
        if viewport_height == 0 {
            self.goal_workspace.set_timeline_scroll(0);
            return;
        }
        let max_scroll = widgets::goal_workspace::max_timeline_scroll(
            area,
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
        );
        let Some(selected_row) = widgets::goal_workspace::timeline_visual_row_for_selection(
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
            viewport_width,
        ) else {
            self.goal_workspace.set_timeline_scroll(0);
            return;
        };
        let current_scroll = self.goal_workspace.timeline_scroll().min(max_scroll);
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
            .set_timeline_scroll(next_scroll.min(max_scroll));
    }

    pub(super) fn clamp_goal_workspace_detail_scroll_to_selection(&mut self) {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return;
        };
        let area = self.pane_layout().chat;
        let viewport_height = widgets::goal_workspace::detail_viewport_height(area);
        let viewport_width = {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(area);
            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(32),
                    Constraint::Min(24),
                ])
                .split(layout[1]);
            Block::default()
                .borders(Borders::ALL)
                .inner(columns[2])
                .width as usize
        };
        if viewport_height == 0 {
            self.goal_workspace.set_detail_scroll(0);
            return;
        }
        let max_scroll = widgets::goal_workspace::max_detail_scroll(
            area,
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
        );
        let Some(selected_row) = widgets::goal_workspace::detail_visual_row_for_selection(
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
            viewport_width,
        ) else {
            self.goal_workspace.set_detail_scroll(0);
            return;
        };
        let current_scroll = self.goal_workspace.detail_scroll().min(max_scroll);
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
            .set_detail_scroll(next_scroll.min(max_scroll));
    }

    pub(super) fn selected_goal_workspace_detail_target(
        &self,
    ) -> Option<crate::widgets::goal_workspace::GoalWorkspaceHitTarget> {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return None;
        };
        crate::widgets::goal_workspace::detail_targets(
            &self.tasks,
            goal_run_id,
            &self.goal_workspace,
        )
        .into_iter()
        .find_map(|(index, target)| {
            (index == self.goal_workspace.selected_detail_row()).then_some(target)
        })
    }

    pub(crate) fn activate_goal_workspace_detail_target(&mut self) -> bool {
        let Some(target) = self.selected_goal_workspace_detail_target() else {
            return false;
        };
        match target {
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::DetailFile(path) => {
                let Some(run) = self.selected_goal_run() else {
                    return false;
                };
                let Some(thread_id) = run.thread_id.clone() else {
                    return false;
                };
                if self.chat.active_thread_id() != Some(thread_id.as_str()) {
                    self.chat
                        .reduce(chat::ChatAction::SelectThread(thread_id.clone()));
                }
                self.sidebar.reduce(sidebar::SidebarAction::SwitchTab(
                    sidebar::SidebarTab::Files,
                ));
                self.tasks.reduce(task::TaskAction::SelectWorkPath {
                    thread_id: thread_id.clone(),
                    path: Some(path.clone()),
                });
                if let Some(index) = self.filtered_sidebar_file_index(&path) {
                    let item_count = self.sidebar_item_count();
                    self.sidebar.select(index, item_count);
                }
                let status_line = path.clone();
                self.open_work_context_for_thread(
                    thread_id,
                    None,
                    None,
                    self.current_goal_return_target(),
                    status_line,
                );
                true
            }
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::DetailTask(task_id) => {
                self.open_sidebar_target(sidebar::SidebarItemTarget::Task { task_id });
                self.focus = FocusArea::Chat;
                true
            }
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::DetailThread(thread_id) => {
                self.open_thread_conversation(thread_id);
                true
            }
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::DetailAction(action) => {
                self.activate_goal_workspace_action(action)
            }
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::DetailTimelineDetails(_) => {
                self.status_line = "Timeline details are shown in the right pane".to_string();
                true
            }
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::FooterAction(_) => false,
            crate::widgets::goal_workspace::GoalWorkspaceHitTarget::DetailCheckpoint(_)
            | crate::widgets::goal_workspace::GoalWorkspaceHitTarget::ThreadRow(_)
            | crate::widgets::goal_workspace::GoalWorkspaceHitTarget::TimelineRow(_)
            | crate::widgets::goal_workspace::GoalWorkspaceHitTarget::PlanPromptToggle
            | crate::widgets::goal_workspace::GoalWorkspaceHitTarget::PlanMainThread(_)
            | crate::widgets::goal_workspace::GoalWorkspaceHitTarget::PlanStep(_)
            | crate::widgets::goal_workspace::GoalWorkspaceHitTarget::PlanTodo { .. }
            | crate::widgets::goal_workspace::GoalWorkspaceHitTarget::ModeTab(_) => false,
        }
    }

    pub(crate) fn activate_goal_workspace_action(
        &mut self,
        action: crate::widgets::goal_workspace::GoalWorkspaceAction,
    ) -> bool {
        match action {
            crate::widgets::goal_workspace::GoalWorkspaceAction::ToggleGoalRun => {
                self.request_selected_goal_run_toggle_confirmation()
            }
            crate::widgets::goal_workspace::GoalWorkspaceAction::OpenActions => {
                self.open_goal_step_action_picker()
            }
            crate::widgets::goal_workspace::GoalWorkspaceAction::RetryStep => {
                self.request_selected_goal_step_retry_confirmation()
            }
            crate::widgets::goal_workspace::GoalWorkspaceAction::RerunFromStep => {
                self.request_selected_goal_step_rerun_confirmation()
            }
            crate::widgets::goal_workspace::GoalWorkspaceAction::RefreshGoal => {
                let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
                    &self.main_pane_view
                else {
                    return false;
                };
                self.request_full_goal_view_refresh(goal_run_id.clone());
                self.status_line = "Refreshing goal, thread, and task metadata".to_string();
                true
            }
        }
    }

    pub(crate) fn step_goal_sidebar_tab(&mut self, delta: i32) {
        if delta < 0 {
            self.goal_sidebar.cycle_tab_left();
        } else if delta > 0 {
            self.goal_sidebar.cycle_tab_right();
        }
        self.reconcile_goal_sidebar_selection_for_active_goal_pane();
    }

    pub(crate) fn activate_goal_sidebar_tab(&mut self, tab: GoalSidebarTab) {
        while self.goal_sidebar.active_tab() != tab {
            match (self.goal_sidebar.active_tab(), tab) {
                (GoalSidebarTab::Steps, GoalSidebarTab::Checkpoints)
                | (GoalSidebarTab::Steps, GoalSidebarTab::Tasks)
                | (GoalSidebarTab::Steps, GoalSidebarTab::Files)
                | (GoalSidebarTab::Checkpoints, GoalSidebarTab::Tasks)
                | (GoalSidebarTab::Checkpoints, GoalSidebarTab::Files)
                | (GoalSidebarTab::Tasks, GoalSidebarTab::Files) => self.step_goal_sidebar_tab(1),
                _ => self.step_goal_sidebar_tab(-1),
            }
        }
    }

    pub(crate) fn navigate_goal_sidebar(&mut self, delta: i32) {
        self.goal_sidebar
            .navigate(delta, self.goal_sidebar_item_count());
        self.sync_goal_sidebar_selection_anchor();
    }

    pub(crate) fn select_goal_sidebar_row(&mut self, index: usize) {
        self.goal_sidebar
            .select_row(index, self.goal_sidebar_item_count());
        self.sync_goal_sidebar_selection_anchor();
    }
}
