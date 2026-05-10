use super::*;
use crossterm::event::{
    KeyCode, KeyModifiers,
};
impl TuiModel {
    pub(super) fn handle_navigation_key_action(
        &mut self,
        code: KeyCode,
        _modifiers: KeyModifiers,
        _ctrl: bool,
    ) -> Option<bool> {
        match code {
            KeyCode::Down if self.focus != FocusArea::Input => {
                match self.focus {
                    FocusArea::Chat => {
                        if matches!(self.main_pane_view, MainPaneView::Collaboration)
                            && self.collaboration.focus() == CollaborationPaneFocus::Navigator
                        {
                            self.collaboration.reduce(CollaborationAction::SelectRow(
                                self.collaboration.selected_row_index().saturating_add(1),
                            ));
                        } else if matches!(
                            self.main_pane_view,
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                        ) {
                            match self.goal_workspace.focused_pane() {
                                crate::state::goal_workspace::GoalWorkspacePane::Plan => {
                                    self.step_goal_workspace_plan_selection(1);
                                }
                                crate::state::goal_workspace::GoalWorkspacePane::Timeline => {
                                    self.step_goal_workspace_timeline_selection(1);
                                }
                                crate::state::goal_workspace::GoalWorkspacePane::Details => {
                                    self.step_goal_workspace_detail_selection(1);
                                }
                                crate::state::goal_workspace::GoalWorkspacePane::CommandBar => {}
                            }
                        } else if matches!(self.main_pane_view, MainPaneView::GoalComposer) {
                            if self
                                .goal_mission_control
                                .cycle_selected_runtime_assignment(1)
                            {
                                let role_label = self
                                    .goal_mission_control
                                    .selected_runtime_row_label()
                                    .unwrap_or("assignment");
                                self.status_line = format!("Mission Control selected {role_label}");
                            }
                        } else if matches!(
                            self.main_pane_view,
                            MainPaneView::Task(_) | MainPaneView::WorkContext
                        ) {
                            self.step_detail_view_scroll(1);
                        } else {
                            self.chat.select_next_message()
                        }
                    }
                    FocusArea::Sidebar => {
                        if self.sidebar_uses_goal_sidebar() {
                            self.navigate_goal_sidebar(1);
                        } else {
                            self.sidebar.navigate(1, self.sidebar_item_count());
                        }
                    }
                    _ => {}
                };
                Some(false)
            }
            KeyCode::Up if self.focus != FocusArea::Input => {
                match self.focus {
                    FocusArea::Chat => {
                        if matches!(self.main_pane_view, MainPaneView::Collaboration)
                            && self.collaboration.focus() == CollaborationPaneFocus::Navigator
                        {
                            self.collaboration.reduce(CollaborationAction::SelectRow(
                                self.collaboration.selected_row_index().saturating_sub(1),
                            ));
                        } else if matches!(
                            self.main_pane_view,
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                        ) {
                            match self.goal_workspace.focused_pane() {
                                crate::state::goal_workspace::GoalWorkspacePane::Plan => {
                                    self.step_goal_workspace_plan_selection(-1);
                                }
                                crate::state::goal_workspace::GoalWorkspacePane::Timeline => {
                                    self.step_goal_workspace_timeline_selection(-1);
                                }
                                crate::state::goal_workspace::GoalWorkspacePane::Details => {
                                    self.step_goal_workspace_detail_selection(-1);
                                }
                                crate::state::goal_workspace::GoalWorkspacePane::CommandBar => {}
                            }
                        } else if matches!(self.main_pane_view, MainPaneView::GoalComposer) {
                            if self
                                .goal_mission_control
                                .cycle_selected_runtime_assignment(-1)
                            {
                                let role_label = self
                                    .goal_mission_control
                                    .selected_runtime_row_label()
                                    .unwrap_or("assignment");
                                self.status_line = format!("Mission Control selected {role_label}");
                            }
                        } else if matches!(
                            self.main_pane_view,
                            MainPaneView::Task(_) | MainPaneView::WorkContext
                        ) {
                            self.step_detail_view_scroll(-1);
                        } else {
                            self.chat.select_prev_message()
                        }
                    }
                    FocusArea::Sidebar => {
                        if self.sidebar_uses_goal_sidebar() {
                            self.navigate_goal_sidebar(-1);
                        } else {
                            self.sidebar.navigate(-1, self.sidebar_item_count());
                        }
                    }
                    _ => {}
                };
                Some(false)
            }
            KeyCode::Left
                if self.focus == FocusArea::Chat
                    && self.goal_workspace.focused_pane()
                        == crate::state::goal_workspace::GoalWorkspacePane::CommandBar
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    self.cycle_goal_workspace_mode(-1);
                };
                Some(false)
            }
            KeyCode::Right
                if self.focus == FocusArea::Chat
                    && self.goal_workspace.focused_pane()
                        == crate::state::goal_workspace::GoalWorkspacePane::CommandBar
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    self.cycle_goal_workspace_mode(1);
                };
                Some(false)
            }
            KeyCode::Left
                if self.focus == FocusArea::Chat
                    && self.goal_workspace.focused_pane()
                        == crate::state::goal_workspace::GoalWorkspacePane::Plan
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    self.collapse_goal_workspace_selection();
                };
                Some(false)
            }
            KeyCode::Right
                if self.focus == FocusArea::Chat
                    && self.goal_workspace.focused_pane()
                        == crate::state::goal_workspace::GoalWorkspacePane::Plan
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                    ) =>
            {
                {
                    self.expand_selected_goal_workspace_step();
                };
                Some(false)
            }
            KeyCode::Left
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::Collaboration) =>
            {
                {
                    if self.collaboration.focus() == CollaborationPaneFocus::Detail {
                        if self.collaboration.selected_detail_action_index() > 0 {
                            self.collaboration
                                .reduce(CollaborationAction::StepDetailAction(-1));
                        } else {
                            self.collaboration.reduce(CollaborationAction::SetFocus(
                                CollaborationPaneFocus::Navigator,
                            ));
                        }
                    }
                };
                Some(false)
            }
            KeyCode::Right
                if self.focus == FocusArea::Chat
                    && matches!(self.main_pane_view, MainPaneView::Collaboration) =>
            {
                {
                    if self.collaboration.focus() == CollaborationPaneFocus::Navigator {
                        self.collaboration.reduce(CollaborationAction::SetFocus(
                            CollaborationPaneFocus::Detail,
                        ));
                    } else {
                        self.collaboration
                            .reduce(CollaborationAction::StepDetailAction(1));
                    }
                };
                Some(false)
            }
            KeyCode::Left if self.focus == FocusArea::Sidebar => {
                {
                    self.step_sidebar_tab(-1);
                };
                Some(false)
            }
            KeyCode::Right if self.focus == FocusArea::Sidebar => {
                {
                    self.step_sidebar_tab(1);
                };
                Some(false)
            }
            KeyCode::Char('[')
                if self.sidebar_visible()
                    && self.focus != FocusArea::Input
                    && !(self.focus == FocusArea::Chat
                        && matches!(
                            self.main_pane_view,
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                        )) =>
            {
                {
                    self.step_sidebar_tab(-1);
                };
                Some(false)
            }
            KeyCode::Char(']')
                if self.sidebar_visible()
                    && self.focus != FocusArea::Input
                    && !(self.focus == FocusArea::Chat
                        && matches!(
                            self.main_pane_view,
                            MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { .. })
                        )) =>
            {
                {
                    self.step_sidebar_tab(1);
                };
                Some(false)
            }
            KeyCode::Char('u')
                if self.focus == FocusArea::Sidebar
                    && !self.sidebar_uses_goal_sidebar()
                    && self.sidebar.active_tab() == sidebar::SidebarTab::Pinned =>
            {
                {
                    self.unpin_selected_sidebar_message();
                };
                Some(false)
            }
            KeyCode::Char('b')
                if self.focus == FocusArea::Chat
                    && matches!(
                        self.main_pane_view,
                        MainPaneView::Conversation | MainPaneView::Task(_)
                    )
                    && self.has_mission_control_return_target() =>
            {
                {
                    let _ = self.return_from_mission_control_navigation();
                }
                Some(false)
            }
            KeyCode::Char('d')
                if self.focus == FocusArea::Chat || self.focus == FocusArea::Sidebar =>
            {
                {
                    if let Some(entry_id) = self.audit.selected_entry_id().map(String::from) {
                        self.audit
                            .reduce(crate::state::audit::AuditAction::DismissEntry(
                                entry_id.clone(),
                            ));
                        self.send_daemon_command(DaemonCommand::AuditDismiss { entry_id });
                        self.show_input_notice(
                            "Audit entry dismissed",
                            InputNoticeKind::Success,
                            40,
                            true,
                        );
                    }
                };
                Some(false)
            }
            _ => None,
        }
    }
}
