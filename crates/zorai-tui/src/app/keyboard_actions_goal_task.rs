use super::*;
use crossterm::event::{
    KeyCode, KeyModifiers,
};
impl TuiModel {
    pub(super) fn handle_goal_task_key_action(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        _ctrl: bool,
    ) -> Option<bool> {
        match code {
            KeyCode::Char('t')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::Task(_)) =>
            {
                {
                    self.task_show_live_todos = !self.task_show_live_todos;
                    self.clamp_detail_view_scroll();
                };
                Some(false)
            }
            KeyCode::Char('l')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::Task(_)) =>
            {
                {
                    self.task_show_timeline = !self.task_show_timeline;
                    self.clamp_detail_view_scroll();
                };
                Some(false)
            }
            KeyCode::Char('a')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::GoalComposer)
                    && !self.goal_mission_control.runtime_mode() =>
            {
                {
                    self.goal_mission_control.append_preflight_assignment();
                    let role_label = self
                        .goal_mission_control
                        .selected_runtime_row_label()
                        .unwrap_or("assignment");
                    self.status_line = format!("Mission Control added {role_label}");
                };
                Some(false)
            }
            KeyCode::Char('a')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::Task(_)) =>
            {
                {
                    if self.open_goal_step_action_picker() {
                        self.status_line = "Goal actions".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char('r')
                if self.focus == FocusArea::Chat
                    && modifiers.is_empty()
                    && matches!(self.main_pane_view, MainPaneView::Task(_)) =>
            {
                {
                    if self.request_selected_goal_step_retry_confirmation() {
                        self.status_line = "Retry selected goal step?".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char(ch)
                if self.focus == FocusArea::Chat
                    && Self::matches_shift_char(KeyCode::Char(ch), modifiers, 'r')
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    if let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
                        ref goal_run_id,
                        ..
                    }) = self.main_pane_view
                    {
                        self.request_full_goal_view_refresh(goal_run_id.clone());
                        self.status_line = "Refreshing goal, thread, and task metadata".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char('m')
                if self.focus == FocusArea::Chat
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    if self.open_mission_control_runtime_editor() {
                        self.status_line = "Opened Mission Control runtime editor".to_string();
                    } else {
                        self.status_line =
                            "Mission Control runtime editor is unavailable".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char('p')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::GoalComposer) =>
            {
                {
                    if !self.stage_mission_control_assignment_modal_edit(
                        goal_mission_control::RuntimeAssignmentEditField::Provider,
                    ) {
                        self.status_line = "Mission Control roster is unavailable".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char('m')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::GoalComposer) =>
            {
                {
                    if !self.stage_mission_control_assignment_modal_edit(
                        goal_mission_control::RuntimeAssignmentEditField::Model,
                    ) {
                        self.status_line = "Mission Control roster is unavailable".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char('e')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::GoalComposer) =>
            {
                {
                    if !self.stage_mission_control_assignment_modal_edit(
                        goal_mission_control::RuntimeAssignmentEditField::ReasoningEffort,
                    ) {
                        self.status_line = "Mission Control roster is unavailable".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char('r')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::GoalComposer) =>
            {
                {
                    if !self.stage_mission_control_assignment_modal_edit(
                        goal_mission_control::RuntimeAssignmentEditField::Role,
                    ) {
                        self.status_line = "Mission Control roster is unavailable".to_string();
                    }
                };
                Some(false)
            }
            KeyCode::Char('s')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::GoalComposer)
                    && !self.goal_mission_control.runtime_mode() =>
            {
                {
                    self.goal_mission_control.toggle_save_as_default_pending();
                    self.status_line = if self.goal_mission_control.save_as_default_pending {
                        "Mission Control preflight will be saved as the new default".to_string()
                    } else {
                        "Mission Control preflight will not overwrite defaults".to_string()
                    };
                };
                Some(false)
            }
            KeyCode::Char('[')
                if self.focus == FocusArea::Chat
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    self.step_goal_step_selection(-1);
                };
                Some(false)
            }
            KeyCode::Char(']')
                if self.focus == FocusArea::Chat
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    self.step_goal_step_selection(1);
                };
                Some(false)
            }
            KeyCode::Char('r') if self.focus == FocusArea::Chat => {
                {
                    if let Some(sel) = self.chat.selected_message() {
                        self.chat.toggle_reasoning(sel);
                    } else {
                        self.chat.toggle_last_reasoning();
                    }
                };
                Some(false)
            }
            KeyCode::Char('f')
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::Task(_)) =>
            {
                {
                    self.task_show_files = !self.task_show_files;
                    self.clamp_detail_view_scroll();
                };
                Some(false)
            }
            KeyCode::Char('e') if self.focus == FocusArea::Chat => {
                {
                    if let Some(sel) = self.chat.selected_message() {
                        let is_tool = self
                            .chat
                            .active_thread()
                            .and_then(|thread| thread.messages.get(sel))
                            .map(|msg| msg.role == chat::MessageRole::Tool)
                            .unwrap_or(false);
                        if is_tool {
                            self.chat.toggle_tool_expansion(sel);
                        }
                    }
                };
                Some(false)
            }
            _ => None,
        }
    }
}
