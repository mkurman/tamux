use super::*;
use crate::providers;
use crate::widgets;
impl TuiModel {
    pub(crate) fn stage_mission_control_assignment_modal_edit(
        &mut self,
        field: goal_mission_control::RuntimeAssignmentEditField,
    ) -> bool {
        if matches!(self.main_pane_view, MainPaneView::GoalComposer) {
            if self
                .goal_mission_control
                .display_role_assignments()
                .is_empty()
            {
                return false;
            }
        } else if !self.open_mission_control_runtime_editor() {
            return false;
        }
        let Some((row_index, _)) = self.selected_runtime_assignment_preview() else {
            return false;
        };
        self.goal_mission_control
            .stage_runtime_edit(row_index, field);
        match field {
            goal_mission_control::RuntimeAssignmentEditField::Provider => {
                self.settings_picker_target = Some(SettingsPickerTarget::Provider);
                self.modal
                    .reduce(modal::ModalAction::Push(modal::ModalKind::ProviderPicker));
                self.sync_provider_picker_item_count();
            }
            goal_mission_control::RuntimeAssignmentEditField::Model => {
                if !self.open_mission_control_assignment_model_picker() {
                    self.goal_mission_control.clear_runtime_edit();
                    return false;
                }
            }
            goal_mission_control::RuntimeAssignmentEditField::ReasoningEffort => {
                self.settings_picker_target = Some(SettingsPickerTarget::SubAgentReasoningEffort);
                self.modal
                    .reduce(modal::ModalAction::Push(modal::ModalKind::EffortPicker));
                self.modal.set_picker_item_count(6);
            }
            goal_mission_control::RuntimeAssignmentEditField::Role => {
                self.settings_picker_target = Some(SettingsPickerTarget::SubAgentRole);
                self.modal
                    .reduce(modal::ModalAction::Push(modal::ModalKind::RolePicker));
                self.modal
                    .set_picker_item_count(crate::state::subagents::role_picker_item_count());
            }
            goal_mission_control::RuntimeAssignmentEditField::Enabled
            | goal_mission_control::RuntimeAssignmentEditField::InheritFromMain => {}
        }
        true
    }

    pub(crate) fn update_selected_runtime_assignment(
        &mut self,
        update: impl FnOnce(&mut task::GoalAgentAssignment),
    ) -> bool {
        if matches!(self.main_pane_view, MainPaneView::GoalComposer)
            && !self.goal_mission_control.runtime_mode()
        {
            let updated = self
                .goal_mission_control
                .update_selected_preflight_assignment(update);
            if updated {
                self.status_line = "Mission Control preflight roster updated".to_string();
            }
            return updated;
        }
        let Some(goal_run_id) = self.selected_goal_run_id() else {
            return false;
        };
        if !self.open_mission_control_runtime_editor() {
            return false;
        }
        let Some((row_index, mut assignment)) = self.selected_runtime_assignment_preview() else {
            return false;
        };
        update(&mut assignment);
        let apply_mode = if self
            .goal_mission_control
            .selected_assignment_matches_active_step()
        {
            self.goal_mission_control.stage_runtime_change(
                goal_run_id,
                row_index,
                assignment,
                goal_mission_control::RuntimeAssignmentApplyMode::NextTurn,
            );
            self.modal.reduce(modal::ModalAction::Push(
                modal::ModalKind::GoalStepActionPicker,
            ));
            self.modal.set_picker_item_count(3);
            self.status_line =
                "Choose how the active step should adopt the pending roster change".to_string();
            return true;
        } else {
            goal_mission_control::RuntimeAssignmentApplyMode::NextTurn
        };
        self.goal_mission_control.stage_runtime_change(
            goal_run_id,
            row_index,
            assignment,
            apply_mode,
        );
        self.goal_mission_control
            .apply_runtime_assignment_change(row_index, apply_mode);
        self.status_line = "Mission Control roster updated for the next turn".to_string();
        true
    }

    pub(crate) fn cycle_selected_runtime_assignment(&mut self) -> bool {
        if !self.open_mission_control_runtime_editor() {
            return false;
        }
        if !self
            .goal_mission_control
            .cycle_selected_runtime_assignment(1)
        {
            return false;
        }
        let role_label = self
            .goal_mission_control
            .selected_runtime_row_label()
            .unwrap_or("runtime assignment");
        self.status_line = format!("Mission Control selected {role_label}");
        true
    }

    pub(crate) fn runtime_assignment_confirmation_items(&self) -> Vec<GoalActionPickerItem> {
        if self.goal_mission_control.pending_runtime_change.is_some() {
            vec![
                GoalActionPickerItem::ApplyRuntimeNextTurn,
                GoalActionPickerItem::ApplyRuntimeReassignActiveStep,
                GoalActionPickerItem::ApplyRuntimeRestartActiveStep,
            ]
        } else {
            Vec::new()
        }
    }

    pub(crate) fn mission_control_role_picker_value(&self) -> String {
        self.goal_mission_control
            .pending_runtime_edit
            .as_ref()
            .filter(|edit| edit.field == goal_mission_control::RuntimeAssignmentEditField::Role)
            .and_then(|edit| {
                self.goal_mission_control
                    .display_role_assignments()
                    .get(edit.row_index)
            })
            .map(|assignment| assignment.role_id.clone())
            .or_else(|| {
                self.subagents
                    .editor
                    .as_ref()
                    .map(|editor| editor.role.clone())
            })
            .unwrap_or_default()
    }

    pub(crate) fn mission_control_effort_picker_value(&self) -> Option<String> {
        self.goal_mission_control
            .pending_runtime_edit
            .as_ref()
            .filter(|edit| {
                edit.field == goal_mission_control::RuntimeAssignmentEditField::ReasoningEffort
            })
            .and_then(|edit| {
                self.goal_mission_control
                    .display_role_assignments()
                    .get(edit.row_index)
            })
            .and_then(|assignment| assignment.reasoning_effort.clone())
    }

    pub(crate) fn runtime_model_picker_current_selection(
        &self,
    ) -> Option<(String, Option<String>)> {
        self.goal_mission_control
            .pending_runtime_edit
            .as_ref()
            .filter(|edit| edit.field == goal_mission_control::RuntimeAssignmentEditField::Model)
            .and_then(|edit| {
                self.goal_mission_control
                    .display_role_assignments()
                    .get(edit.row_index)
            })
            .map(|assignment| (assignment.model.clone(), None))
    }

    pub(super) fn open_mission_control_assignment_model_picker(&mut self) -> bool {
        let Some((_, assignment)) = self.selected_runtime_assignment_preview() else {
            return false;
        };
        let provider_id = assignment.provider.clone();
        let (base_url, api_key, auth_source) = self.provider_auth_snapshot(&provider_id);
        let models = providers::known_models_for_provider_auth(&provider_id, &auth_source);
        self.config
            .reduce(config::ConfigAction::ModelsFetched(models));
        if self.should_fetch_remote_models(&provider_id, &auth_source) {
            self.send_daemon_command(DaemonCommand::FetchModels {
                provider_id,
                base_url,
                api_key,
                output_modalities: None,
            });
        }
        self.settings_picker_target = Some(SettingsPickerTarget::Model);
        self.modal
            .reduce(modal::ModalAction::Push(modal::ModalKind::ModelPicker));
        self.sync_model_picker_item_count();
        true
    }

    pub(crate) fn begin_mission_control_custom_model_edit(&mut self) {
        let Some((_, assignment)) = self.selected_runtime_assignment_preview() else {
            self.status_line = "Mission Control roster is unavailable".to_string();
            return;
        };
        if self.modal.top() != Some(modal::ModalKind::Settings) {
            self.modal
                .reduce(modal::ModalAction::Push(modal::ModalKind::Settings));
        }
        self.settings
            .start_editing("mission_control_assignment_model", &assignment.model);
        self.status_line = "Enter mission control model ID".to_string();
    }

    pub(crate) fn available_runtime_assignment_models(
        &self,
    ) -> Vec<crate::state::config::FetchedModel> {
        if let Some((current_model, custom_model_name)) =
            self.runtime_model_picker_current_selection()
        {
            widgets::model_picker::available_models_for(
                &self.config,
                &current_model,
                custom_model_name.as_deref(),
            )
        } else {
            Vec::new()
        }
    }

    pub(crate) fn confirm_runtime_assignment_change(
        &mut self,
        apply_mode: goal_mission_control::RuntimeAssignmentApplyMode,
    ) -> bool {
        let Some(change) = self.goal_mission_control.pending_runtime_change.clone() else {
            return false;
        };
        self.goal_mission_control
            .apply_runtime_assignment_change(change.row_index, apply_mode);
        self.status_line = format!(
            "Mission Control roster updated: {}",
            apply_mode.roster_status_label()
        );
        true
    }

    pub(super) fn selected_goal_run_toggle_action(&self) -> Option<PendingConfirmAction> {
        let run = self.selected_goal_run()?;
        let title = run.title.clone();
        match run.status {
            Some(task::GoalRunStatus::Paused) => Some(PendingConfirmAction::ResumeGoalRun {
                goal_run_id: run.id.clone(),
                title,
            }),
            Some(task::GoalRunStatus::Queued)
            | Some(task::GoalRunStatus::Planning)
            | Some(task::GoalRunStatus::Running)
            | Some(task::GoalRunStatus::AwaitingApproval) => {
                Some(PendingConfirmAction::PauseGoalRun {
                    goal_run_id: run.id.clone(),
                    title,
                })
            }
            _ => None,
        }
    }

    pub(crate) fn request_selected_goal_run_toggle_confirmation(&mut self) -> bool {
        let Some(action) = self.selected_goal_run_toggle_action() else {
            return false;
        };
        self.open_pending_action_confirm(action);
        true
    }

    pub(crate) fn request_selected_goal_run_stop_confirmation(&mut self) -> bool {
        let Some(run) = self.selected_goal_run() else {
            return false;
        };
        if matches!(
            run.status,
            Some(task::GoalRunStatus::Completed)
                | Some(task::GoalRunStatus::Failed)
                | Some(task::GoalRunStatus::Cancelled)
        ) {
            return false;
        }
        self.open_pending_action_confirm(PendingConfirmAction::StopGoalRun {
            goal_run_id: run.id.clone(),
            title: run.title.clone(),
        });
        true
    }

    pub(crate) fn request_preview_for_selected_path(&mut self, thread_id: &str) {
        let Some(context) = self.tasks.work_context_for_thread(thread_id) else {
            return;
        };
        let Some(selected_path) = self.tasks.selected_work_path(thread_id) else {
            return;
        };
        let Some(entry) = context
            .entries
            .iter()
            .find(|entry| entry.path == selected_path)
        else {
            return;
        };
        if let Some(repo_root) = entry.repo_root.as_deref() {
            self.send_daemon_command(DaemonCommand::RequestGitDiff {
                repo_path: repo_root.to_string(),
                file_path: Some(entry.path.clone()),
            });
        } else {
            self.send_daemon_command(DaemonCommand::RequestFilePreview {
                path: entry.path.clone(),
                max_bytes: Some(65_536),
            });
        }
    }

    pub(crate) fn ensure_task_view_preview(&mut self) {
        let MainPaneView::Task(target) = &self.main_pane_view else {
            return;
        };
        let Some(thread_id) = self.target_thread_id(target) else {
            return;
        };
        if self.tasks.selected_work_path(&thread_id).is_none() {
            if let Some(context) = self.tasks.work_context_for_thread(&thread_id) {
                if let Some(first) = context.entries.first() {
                    self.tasks.reduce(task::TaskAction::SelectWorkPath {
                        thread_id: thread_id.clone(),
                        path: Some(first.path.clone()),
                    });
                }
            }
        }
        self.request_preview_for_selected_path(&thread_id);
    }

    pub(crate) fn request_task_view_context(&mut self, target: &sidebar::SidebarItemTarget) {
        if let Some(thread_id) = self.target_thread_id(target) {
            self.send_daemon_command(DaemonCommand::RequestThreadTodos(thread_id.clone()));
            self.send_daemon_command(DaemonCommand::RequestThreadWorkContext(thread_id));
        }
    }

    pub(crate) fn current_sidebar_snapshot(
        &self,
    ) -> Option<&widgets::sidebar::CachedSidebarSnapshot> {
        let area = self.pane_layout().sidebar?;
        self.sidebar_snapshot.as_ref().filter(|snapshot| {
            widgets::sidebar::cached_snapshot_matches_render(
                snapshot,
                area,
                &self.chat,
                &self.sidebar,
                &self.tasks,
                self.chat.active_thread_id(),
            )
        })
    }

    pub(crate) fn selected_sidebar_file_path(&self) -> Option<String> {
        self.current_sidebar_snapshot()
            .and_then(|snapshot| snapshot.selected_file_path(self.sidebar.selected_item()))
            .or_else(|| {
                widgets::sidebar::selected_file_path(
                    &self.tasks,
                    &self.sidebar,
                    self.chat.active_thread_id(),
                )
            })
    }

    pub(crate) fn filtered_sidebar_file_index(&self, path: &str) -> Option<usize> {
        self.current_sidebar_snapshot()
            .and_then(|snapshot| snapshot.filtered_file_index(path))
            .or_else(|| {
                widgets::sidebar::filtered_file_index(
                    &self.tasks,
                    &self.sidebar,
                    self.chat.active_thread_id(),
                    path,
                )
            })
    }

    pub(super) fn selected_sidebar_spawned_thread_id(&self) -> Option<String> {
        self.current_sidebar_snapshot()
            .and_then(|snapshot| snapshot.selected_spawned_thread_id(self.sidebar.selected_item()))
            .or_else(|| {
                widgets::sidebar::selected_spawned_thread_id(
                    &self.tasks,
                    &self.sidebar,
                    self.chat.active_thread_id(),
                )
            })
    }

    pub(super) fn first_openable_sidebar_spawned_index(&self) -> Option<usize> {
        self.current_sidebar_snapshot()
            .and_then(widgets::sidebar::CachedSidebarSnapshot::first_openable_spawned_index)
            .or_else(|| {
                widgets::sidebar::first_openable_spawned_index(
                    &self.tasks,
                    self.chat.active_thread_id(),
                )
            })
    }

    pub(super) fn selected_sidebar_pinned_message(
        &self,
    ) -> Option<crate::state::chat::PinnedThreadMessage> {
        self.current_sidebar_snapshot()
            .and_then(|snapshot| {
                snapshot.selected_pinned_message(&self.chat, self.sidebar.selected_item())
            })
            .or_else(|| widgets::sidebar::selected_pinned_message(&self.chat, &self.sidebar))
    }

    pub(crate) fn sidebar_item_count(&self) -> usize {
        self.current_sidebar_snapshot()
            .map(widgets::sidebar::CachedSidebarSnapshot::item_count)
            .unwrap_or_else(|| {
                widgets::sidebar::body_item_count(
                    &self.tasks,
                    &self.chat,
                    &self.sidebar,
                    self.chat.active_thread_id(),
                )
            })
    }

    pub(crate) fn activate_sidebar_tab(&mut self, tab: sidebar::SidebarTab) {
        self.sidebar.reduce(sidebar::SidebarAction::SwitchTab(tab));
        if tab == sidebar::SidebarTab::Spawned {
            if let Some(index) = self.first_openable_sidebar_spawned_index() {
                self.sidebar.select(index, self.sidebar_item_count());
            }
        }
    }

    pub(super) fn open_selected_spawned_thread(&mut self) {
        let Some(from_thread_id) = self.chat.active_thread_id().map(str::to_string) else {
            return;
        };
        let Some(to_thread_id) = self.selected_sidebar_spawned_thread_id() else {
            return;
        };

        self.cleanup_concierge_on_navigate();
        self.clear_chat_drag_selection();
        self.clear_work_context_drag_selection();
        self.clear_task_view_drag_selection();
        self.pending_new_thread_target_agent = None;

        if !self
            .chat
            .open_spawned_thread(&from_thread_id, &to_thread_id)
        {
            return;
        }

        self.set_mission_control_return_targets(
            self.current_goal_return_target(),
            Some(from_thread_id),
        );
        self.request_latest_thread_page(to_thread_id.clone(), true);
        self.main_pane_view = MainPaneView::Conversation;
        self.task_view_scroll = 0;
        self.focus = FocusArea::Chat;
        self.status_line = format!("Opened spawned thread {to_thread_id}");
    }
}
