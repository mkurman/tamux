use super::*;
use crate::widgets;
impl TuiModel {
    pub(crate) fn go_back_thread(&mut self) {
        if !self.chat.can_go_back_thread() {
            self.status_line = "No previous thread".to_string();
            return;
        }

        self.cleanup_concierge_on_navigate();
        self.clear_chat_drag_selection();
        self.clear_work_context_drag_selection();
        self.clear_task_view_drag_selection();
        self.pending_new_thread_target_agent = None;

        let Some(thread_id) = self.chat.go_back_thread() else {
            self.status_line = "No previous thread".to_string();
            return;
        };

        self.set_mission_control_return_to_thread_id(
            self.chat.thread_history_stack().last().cloned(),
        );
        self.request_latest_thread_page(thread_id.clone(), true);
        self.main_pane_view = MainPaneView::Conversation;
        self.task_view_scroll = 0;
        self.focus = FocusArea::Chat;
        self.status_line = format!("Returned to {thread_id}");
    }

    pub(crate) fn open_sidebar_target(&mut self, target: sidebar::SidebarItemTarget) {
        self.clear_mission_control_return_context();
        self.cleanup_concierge_on_navigate();
        self.clear_task_view_drag_selection();
        if let sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. } = &target {
            self.request_authoritative_goal_run_refresh(goal_run_id.clone());
            if self.tasks.goal_run_by_id(goal_run_id).is_some_and(|run| {
                matches!(
                    run.status,
                    Some(task::GoalRunStatus::Queued)
                        | Some(task::GoalRunStatus::Planning)
                        | Some(task::GoalRunStatus::Running)
                        | Some(task::GoalRunStatus::AwaitingApproval)
                )
            }) {
                self.schedule_goal_hydration_refresh(goal_run_id.clone());
            }
            self.goal_workspace.set_plan_scroll(0);
        }
        self.request_task_view_context(&target);
        self.main_pane_view = MainPaneView::Task(target);
        self.reconcile_goal_sidebar_selection_for_active_goal_pane();
        self.sync_goal_workspace_selection_for_active_goal_pane();
        self.task_view_scroll = 0;
        self.sync_contextual_approval_overlay();
    }

    pub(crate) fn sync_thread_picker_item_count(&mut self) {
        let thread_count = widgets::thread_picker::filtered_threads_for_workspace(
            &self.chat,
            &self.modal,
            &self.subagents,
            &self.tasks,
            &self.workspace,
        )
        .len();
        self.modal.set_picker_item_count(thread_count + 1);
    }

    pub(crate) fn sync_goal_picker_item_count(&mut self) {
        self.modal
            .set_picker_item_count(self.filtered_goal_runs().len() + 1);
    }

    pub(crate) fn selected_goal_step_context(
        &self,
    ) -> Option<(String, String, usize, crate::state::task::GoalRunStep)> {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
            goal_run_id,
            step_id,
        }) = &self.main_pane_view
        else {
            return None;
        };
        let run = self.tasks.goal_run_by_id(goal_run_id)?;
        let step = if let Some(step_id) = step_id {
            run.steps.iter().find(|step| step.id == *step_id)?.clone()
        } else {
            run.steps
                .iter()
                .find(|step| {
                    step.order as usize == run.current_step_index
                        || Some(step.title.as_str()) == run.current_step_title.as_deref()
                })
                .or_else(|| run.steps.iter().min_by_key(|step| step.order))
                .cloned()?
        };
        Some((run.id.clone(), run.title.clone(), step.order as usize, step))
    }

    pub(crate) fn select_goal_step_in_active_run(&mut self, step_id: String) -> bool {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return false;
        };
        let Some(run) = self.tasks.goal_run_by_id(goal_run_id) else {
            return false;
        };
        let Some(step) = run.steps.iter().find(|step| step.id == step_id) else {
            return false;
        };
        let step_title = step.title.clone();
        let step_order = step.order;

        self.main_pane_view = MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
            goal_run_id: goal_run_id.clone(),
            step_id: Some(step.id.clone()),
        });
        self.reconcile_goal_sidebar_selection_for_active_goal_pane();
        self.sync_goal_workspace_selection_for_active_goal_pane();
        self.status_line = format!("Selected step {}: {}", step_order + 1, step_title);
        true
    }

    pub(crate) fn step_goal_step_selection(&mut self, delta: i32) -> bool {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
            goal_run_id,
            step_id,
        }) = &self.main_pane_view
        else {
            return false;
        };
        let Some(run) = self.tasks.goal_run_by_id(goal_run_id) else {
            return false;
        };
        let mut steps = run.steps.clone();
        steps.sort_by_key(|step| step.order);
        if steps.is_empty() {
            return false;
        }

        let current_index = step_id
            .as_ref()
            .and_then(|selected| steps.iter().position(|step| step.id == *selected))
            .or_else(|| {
                steps.iter().position(|step| {
                    step.order as usize == run.current_step_index
                        || Some(step.title.as_str()) == run.current_step_title.as_deref()
                })
            })
            .unwrap_or(0);
        let next_index = if delta > 0 {
            current_index
                .saturating_add(delta as usize)
                .min(steps.len().saturating_sub(1))
        } else {
            current_index.saturating_sub((-delta) as usize)
        };
        let next_step = &steps[next_index];
        let step_title = next_step.title.clone();
        let step_order = next_step.order;
        self.main_pane_view = MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun {
            goal_run_id: goal_run_id.clone(),
            step_id: Some(next_step.id.clone()),
        });
        self.reconcile_goal_sidebar_selection_for_active_goal_pane();
        self.status_line = format!("Selected step {}: {}", step_order + 1, step_title);
        true
    }

    pub(crate) fn request_selected_goal_step_retry_confirmation(&mut self) -> bool {
        if let Some((goal_run_id, goal_title, step_index, step)) = self.selected_goal_step_context()
        {
            self.open_pending_action_confirm(PendingConfirmAction::RetryGoalStep {
                goal_run_id,
                goal_title,
                step_index,
                step_title: step.title,
            });
            return true;
        }

        let Some((goal_run_id, goal_title)) = self.selected_goal_prompt_context() else {
            return false;
        };
        self.open_pending_action_confirm(PendingConfirmAction::RetryGoalPrompt {
            goal_run_id,
            goal_title,
        });
        true
    }

    pub(crate) fn request_selected_goal_step_rerun_confirmation(&mut self) -> bool {
        if let Some((goal_run_id, goal_title, step_index, step)) = self.selected_goal_step_context()
        {
            self.open_pending_action_confirm(PendingConfirmAction::RerunGoalFromStep {
                goal_run_id,
                goal_title,
                step_index,
                step_title: step.title,
            });
            return true;
        }

        let Some((goal_run_id, goal_title)) = self.selected_goal_prompt_context() else {
            return false;
        };
        self.open_pending_action_confirm(PendingConfirmAction::RerunGoalPrompt {
            goal_run_id,
            goal_title,
        });
        true
    }

    pub(crate) fn goal_action_picker_items(&self) -> Vec<GoalActionPickerItem> {
        let confirmation_items = self.runtime_assignment_confirmation_items();
        if !confirmation_items.is_empty() {
            return confirmation_items;
        }

        let mut items = Vec::new();
        if let Some(run) = self.selected_goal_run() {
            match run.status {
                Some(task::GoalRunStatus::Paused) => items.push(GoalActionPickerItem::ResumeGoal),
                Some(task::GoalRunStatus::Queued)
                | Some(task::GoalRunStatus::Planning)
                | Some(task::GoalRunStatus::Running)
                | Some(task::GoalRunStatus::AwaitingApproval) => {
                    items.push(GoalActionPickerItem::PauseGoal);
                    items.push(GoalActionPickerItem::StopGoal);
                }
                _ => {}
            }
            if !run.runtime_assignment_list.is_empty() || !run.launch_assignment_snapshot.is_empty()
            {
                items.push(GoalActionPickerItem::CycleRuntimeAssignment);
                items.push(GoalActionPickerItem::EditRuntimeProvider);
                items.push(GoalActionPickerItem::EditRuntimeModel);
                items.push(GoalActionPickerItem::EditRuntimeReasoning);
                items.push(GoalActionPickerItem::EditRuntimeRole);
                items.push(GoalActionPickerItem::ToggleRuntimeEnabled);
                items.push(GoalActionPickerItem::ToggleRuntimeInherit);
            }
        }

        if self.selected_goal_step_context().is_some()
            || self.selected_goal_prompt_context().is_some()
        {
            items.push(GoalActionPickerItem::RetryStep);
            items.push(GoalActionPickerItem::RerunFromStep);
        }

        if self.selected_goal_run().is_some() {
            items.push(GoalActionPickerItem::DeleteGoal);
        }

        items
    }

    fn selected_goal_prompt_context(&self) -> Option<(String, String)> {
        let MainPaneView::Task(sidebar::SidebarItemTarget::GoalRun { goal_run_id, .. }) =
            &self.main_pane_view
        else {
            return None;
        };
        let run = self.tasks.goal_run_by_id(goal_run_id)?;
        run.steps
            .is_empty()
            .then(|| (run.id.clone(), run.title.clone()))
    }

    pub(crate) fn open_goal_step_action_picker(&mut self) -> bool {
        let items = self.goal_action_picker_items();
        if items.is_empty() {
            return false;
        }
        self.modal.reduce(modal::ModalAction::Push(
            modal::ModalKind::GoalStepActionPicker,
        ));
        self.modal.set_picker_item_count(items.len());
        true
    }

    pub(crate) fn open_queued_prompts_modal(&mut self) {
        if self.queued_prompts.is_empty() {
            self.status_line = "No queued messages".to_string();
            return;
        }
        if self.modal.top() != Some(modal::ModalKind::QueuedPrompts) {
            self.modal
                .reduce(modal::ModalAction::Push(modal::ModalKind::QueuedPrompts));
        }
        self.modal.set_picker_item_count(self.queued_prompts.len());
        self.queued_prompt_action = QueuedPromptAction::SendNow;
    }

    fn open_queued_prompt_viewer(&mut self, index: usize) {
        let Some(prompt) = self.queued_prompts.get(index) else {
            return;
        };
        let body = format_queued_prompt_viewer_body(prompt);

        self.prompt_modal_loading = false;
        self.prompt_modal_error = None;
        self.prompt_modal_scroll = 0;
        self.prompt_modal_title_override = Some("QUEUED MESSAGE".to_string());
        self.prompt_modal_body_override = Some(body);

        if self.modal.top() != Some(modal::ModalKind::PromptViewer) {
            self.modal
                .reduce(modal::ModalAction::Push(modal::ModalKind::PromptViewer));
        }
    }

    pub(crate) fn queue_prompt(&mut self, prompt: String) {
        self.queued_prompts.push(QueuedPrompt::new(prompt));
        self.status_line = format!("QUEUED ({})", self.queued_prompts.len());
        self.sync_queued_prompt_modal_state();
    }

    pub(crate) fn queue_participant_suggestion(
        &mut self,
        thread_id: String,
        suggestion_id: String,
        target_agent_id: String,
        target_agent_name: String,
        prompt: String,
        force_send: bool,
    ) {
        if let Some(existing) = self.queued_prompts.iter_mut().find(|queued| {
            queued.thread_id.as_deref() == Some(thread_id.as_str())
                && queued.suggestion_id.as_deref() == Some(suggestion_id.as_str())
        }) {
            existing.text = prompt;
            existing.participant_agent_id = Some(target_agent_id);
            existing.participant_agent_name = Some(target_agent_name);
            existing.force_send = force_send;
        } else {
            self.queued_prompts.push(QueuedPrompt::new_with_agent(
                prompt,
                thread_id,
                suggestion_id,
                target_agent_id,
                target_agent_name,
                force_send,
            ));
        }
        self.status_line = format!("QUEUED ({})", self.queued_prompts.len());
        self.sync_queued_prompt_modal_state();
    }

    fn remove_queued_prompt_at(&mut self, index: usize) -> Option<QueuedPrompt> {
        if index >= self.queued_prompts.len() {
            return None;
        }
        let prompt = self.queued_prompts.remove(index);
        self.sync_queued_prompt_modal_state();
        Some(prompt)
    }

    pub(crate) fn dispatch_next_queued_prompt_if_ready(&mut self) {
        if self.queue_barrier_active() {
            return;
        }
        let Some(index) = self
            .queued_prompts
            .iter()
            .position(|prompt| prompt.suggestion_id.is_none())
        else {
            return;
        };
        if let Some(prompt) = self.remove_queued_prompt_at(index) {
            self.submit_prompt(prompt.text);
        }
    }

    pub(crate) fn sync_participant_queued_prompts_for_thread(
        &mut self,
        thread_id: &str,
        live_suggestion_ids: &std::collections::HashSet<String>,
    ) {
        let before = self.queued_prompts.len();
        self.queued_prompts.retain(|prompt| {
            let Some(prompt_thread_id) = prompt.thread_id.as_deref() else {
                return true;
            };
            let Some(suggestion_id) = prompt.suggestion_id.as_deref() else {
                return true;
            };
            if prompt_thread_id != thread_id {
                return true;
            }
            live_suggestion_ids.contains(suggestion_id)
        });
        if self.queued_prompts.len() != before {
            self.sync_queued_prompt_modal_state();
        }
    }

    fn interrupt_current_stream(&mut self) {
        let Some(thread_id) = self.chat.active_thread_id().map(str::to_string) else {
            return;
        };
        self.cancelled_thread_id = Some(thread_id.clone());
        self.chat.reduce(chat::ChatAction::ForceStopStreaming);
        self.clear_active_thread_activity();
        self.pending_stop = false;
        self.send_daemon_command(DaemonCommand::StopStream { thread_id });
    }

    pub(crate) fn execute_selected_queued_prompt_action(&mut self) {
        let index = self.modal.picker_cursor();
        let action = self.queued_prompt_action;
        match action {
            QueuedPromptAction::Expand => self.open_queued_prompt_viewer(index),
            QueuedPromptAction::SendNow => {
                let Some(prompt) = self.remove_queued_prompt_at(index) else {
                    return;
                };
                let should_interrupt =
                    self.assistant_busy() && (prompt.suggestion_id.is_none() || prompt.force_send);
                if should_interrupt {
                    self.interrupt_current_stream();
                }
                if let (Some(thread_id), Some(suggestion_id)) =
                    (prompt.thread_id.clone(), prompt.suggestion_id.clone())
                {
                    self.send_daemon_command(DaemonCommand::SendParticipantSuggestion {
                        thread_id,
                        suggestion_id,
                    });
                } else {
                    self.submit_prompt(prompt.text);
                }
            }
            QueuedPromptAction::Copy => {
                let Some(prompt) = self.queued_prompts.get_mut(index) else {
                    return;
                };
                conversion::copy_to_clipboard(&prompt.text);
                prompt.mark_copied(self.tick_counter.saturating_add(100));
                self.status_line = "Copied queued message".to_string();
            }
            QueuedPromptAction::Delete => {
                if let Some(prompt) = self.remove_queued_prompt_at(index) {
                    if let (Some(thread_id), Some(suggestion_id)) =
                        (prompt.thread_id, prompt.suggestion_id)
                    {
                        self.send_daemon_command(DaemonCommand::DismissParticipantSuggestion {
                            thread_id,
                            suggestion_id,
                        });
                    }
                    self.status_line = "Removed queued message".to_string();
                }
            }
        }
    }
}
