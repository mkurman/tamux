use super::*;
use crate::widgets;
impl TuiModel {
    pub(crate) fn move_workspace_task_from_args(&mut self, args: &str) {
        let mut parts = args.split_whitespace();
        let Some(raw_task_id) = parts.next() else {
            self.status_line =
                "Usage: /workspace-move <task_id> <todo|in-progress|in-review|done>".to_string();
            return;
        };
        let Some(status) = parts.next().and_then(parse_workspace_status) else {
            self.status_line =
                "Usage: /workspace-move <task_id> <todo|in-progress|in-review|done>".to_string();
            return;
        };
        let Some(task_id) = self.resolve_workspace_task_id(raw_task_id) else {
            return;
        };
        self.send_daemon_command(DaemonCommand::MoveWorkspaceTask(
            zorai_protocol::WorkspaceTaskMove {
                task_id,
                status,
                sort_order: None,
            },
        ));
        self.main_pane_view = MainPaneView::Workspace;
        self.status_line = "Moving workspace task...".to_string();
    }

    pub(crate) fn assign_workspace_task_from_args(&mut self, args: &str) {
        let mut parts = args.split_whitespace();
        let Some(raw_task_id) = parts.next() else {
            self.status_line =
                "Usage: /workspace-assign <task_id> <svarog|agent:id|subagent:id|none>".to_string();
            return;
        };
        let Some(raw_actor) = parts.next() else {
            self.status_line =
                "Usage: /workspace-assign <task_id> <svarog|agent:id|subagent:id|none>".to_string();
            return;
        };
        let Some(assignee) = parse_workspace_actor_field(raw_actor) else {
            self.status_line = "Unknown workspace assignee".to_string();
            return;
        };
        if matches!(&assignee, Some(zorai_protocol::WorkspaceActor::User)) {
            self.status_line = "Workspace assignee must be an agent or subagent".to_string();
            return;
        }
        let Some(task_id) = self.resolve_workspace_task_id(raw_task_id) else {
            return;
        };
        self.send_daemon_command(DaemonCommand::UpdateWorkspaceTask {
            task_id,
            update: zorai_protocol::WorkspaceTaskUpdate {
                assignee: Some(assignee),
                ..Default::default()
            },
        });
        self.status_line = "Updating workspace assignee...".to_string();
    }

    pub(super) fn open_workspace_actor_picker(
        &mut self,
        task_id: String,
        mode: crate::app::workspace_actor_picker::WorkspaceActorPickerMode,
    ) {
        let count = crate::app::workspace_actor_picker::workspace_actor_picker_options(
            mode,
            &self.subagents,
        )
        .len();
        self.pending_workspace_actor_picker = Some(PendingWorkspaceActorPicker {
            target: PendingWorkspaceActorPickerTarget::Task {
                task_id: task_id.clone(),
            },
            task_id,
            mode,
        });
        self.modal.reduce(modal::ModalAction::Push(
            modal::ModalKind::WorkspaceActorPicker,
        ));
        self.modal.set_picker_item_count(count);
        self.status_line = format!("Select {}", mode.title().to_ascii_lowercase());
    }

    pub(crate) fn workspace_actor_picker_body(&self) -> String {
        let Some(pending) = self.pending_workspace_actor_picker.as_ref() else {
            return "No workspace task selected".to_string();
        };
        let options = crate::app::workspace_actor_picker::workspace_actor_picker_options(
            pending.mode,
            &self.subagents,
        );
        crate::app::workspace_actor_picker::workspace_actor_picker_body(
            &pending.task_id,
            pending.mode,
            &options,
            self.modal.picker_cursor(),
        )
    }

    pub(crate) fn workspace_actor_picker_scroll(&self) -> usize {
        let Some(pending) = self.pending_workspace_actor_picker.as_ref() else {
            return 0;
        };
        let options = crate::app::workspace_actor_picker::workspace_actor_picker_options(
            pending.mode,
            &self.subagents,
        );
        let viewport_lines = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::WorkspaceActorPicker)
            .map(|(_, area)| area.height.saturating_sub(3) as usize)
            .unwrap_or(1)
            .max(1);
        let total_lines = 3usize.saturating_add(options.len()).saturating_add(2);
        let selected_line = 3usize.saturating_add(self.modal.picker_cursor());
        selected_line
            .saturating_sub(viewport_lines.saturating_sub(1))
            .min(total_lines.saturating_sub(viewport_lines))
    }

    pub(crate) fn submit_workspace_actor_picker(&mut self) {
        let Some(pending) = self.pending_workspace_actor_picker.clone() else {
            self.close_top_modal();
            return;
        };
        let options = crate::app::workspace_actor_picker::workspace_actor_picker_options(
            pending.mode,
            &self.subagents,
        );
        let Some(selected) = options.get(self.modal.picker_cursor()).cloned() else {
            self.status_line = "No workspace actor selected".to_string();
            return;
        };
        self.close_top_modal();
        if let Some(agent_alias) = self.workspace_actor_setup_alias(selected.actor.as_ref()) {
            self.open_builtin_persona_workspace_actor_setup_flow(
                &agent_alias,
                pending,
                selected
                    .actor
                    .expect("setup alias should only exist for concrete actors"),
            );
            return;
        }
        self.apply_workspace_actor_selection(pending, selected.actor);
    }

    fn workspace_actor_setup_alias(
        &self,
        actor: Option<&zorai_protocol::WorkspaceActor>,
    ) -> Option<String> {
        let Some(zorai_protocol::WorkspaceActor::Subagent(agent_id)) = actor else {
            return None;
        };
        (!self.builtin_persona_configured(agent_id)).then(|| agent_id.clone())
    }

    pub(crate) fn apply_workspace_actor_selection(
        &mut self,
        pending: PendingWorkspaceActorPicker,
        actor: Option<zorai_protocol::WorkspaceActor>,
    ) {
        if pending.target == PendingWorkspaceActorPickerTarget::CreateForm {
            if let Some(form) = self.pending_workspace_create_form.as_mut() {
                match pending.mode {
                    crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Assignee => {
                        form.assignee = actor;
                        self.status_line = "Workspace assignee selected".to_string();
                    }
                    crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Reviewer => {
                        form.reviewer = actor;
                        self.status_line = "Workspace reviewer selected".to_string();
                    }
                }
            }
            return;
        }
        if pending.target == PendingWorkspaceActorPickerTarget::EditForm {
            if let Some(form) = self.pending_workspace_edit_form.as_mut() {
                match pending.mode {
                    crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Assignee => {
                        form.assignee = actor;
                        self.status_line = "Workspace assignee selected".to_string();
                    }
                    crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Reviewer => {
                        form.reviewer = actor;
                        self.status_line = "Workspace reviewer selected".to_string();
                    }
                }
            }
            return;
        }
        let update = match pending.mode {
            crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Assignee => {
                zorai_protocol::WorkspaceTaskUpdate {
                    assignee: Some(actor),
                    ..Default::default()
                }
            }
            crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Reviewer => {
                zorai_protocol::WorkspaceTaskUpdate {
                    reviewer: Some(actor),
                    ..Default::default()
                }
            }
        };
        self.send_daemon_command(DaemonCommand::UpdateWorkspaceTask {
            task_id: match pending.target {
                PendingWorkspaceActorPickerTarget::Task { task_id } => task_id,
                PendingWorkspaceActorPickerTarget::CreateForm => pending.task_id,
                PendingWorkspaceActorPickerTarget::EditForm => pending.task_id,
            },
            update,
        });
        self.main_pane_view = MainPaneView::Workspace;
        self.status_line = match pending.mode {
            crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Assignee => {
                "Updating workspace assignee...".to_string()
            }
            crate::app::workspace_actor_picker::WorkspaceActorPickerMode::Reviewer => {
                "Updating workspace reviewer...".to_string()
            }
        };
    }

    pub(crate) fn update_workspace_task_from_args(&mut self, args: &str) {
        let draft = match crate::app::workspace_update::parse_workspace_update_args(args) {
            Ok(draft) => draft,
            Err(err) => {
                self.status_line = err.to_string();
                return;
            }
        };
        let Some(task_id) = self.resolve_workspace_task_id(&draft.task_id) else {
            return;
        };
        self.send_daemon_command(DaemonCommand::UpdateWorkspaceTask {
            task_id,
            update: draft.update,
        });
        self.main_pane_view = MainPaneView::Workspace;
        self.status_line = "Updating workspace task...".to_string();
    }

    pub(crate) fn set_workspace_reviewer_from_args(&mut self, args: &str) {
        let mut parts = args.split_whitespace();
        let Some(raw_task_id) = parts.next() else {
            self.status_line =
                "Usage: /workspace-reviewer <task_id> <user|svarog|agent:id|subagent:id|none>"
                    .to_string();
            return;
        };
        let Some(raw_actor) = parts.next() else {
            self.status_line =
                "Usage: /workspace-reviewer <task_id> <user|svarog|agent:id|subagent:id|none>"
                    .to_string();
            return;
        };
        let Some(reviewer) = parse_workspace_actor_field(raw_actor) else {
            self.status_line = "Unknown workspace reviewer".to_string();
            return;
        };
        let Some(task_id) = self.resolve_workspace_task_id(raw_task_id) else {
            return;
        };
        self.send_daemon_command(DaemonCommand::UpdateWorkspaceTask {
            task_id,
            update: zorai_protocol::WorkspaceTaskUpdate {
                reviewer: Some(reviewer),
                ..Default::default()
            },
        });
        self.status_line = "Updating workspace reviewer...".to_string();
    }

    pub(crate) fn review_workspace_task_from_args(&mut self, args: &str) {
        let mut parts = args.trim().splitn(3, char::is_whitespace);
        let Some(raw_task_id) = parts.next().filter(|value| !value.is_empty()) else {
            self.status_line =
                "Usage: /workspace-review <task_id> <pass|fail> [message]".to_string();
            return;
        };
        let verdict = match parts
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "pass" | "passed" | "approve" | "approved" => {
                zorai_protocol::WorkspaceReviewVerdict::Pass
            }
            "fail" | "failed" | "reject" | "rejected" => {
                zorai_protocol::WorkspaceReviewVerdict::Fail
            }
            _ => {
                self.status_line =
                    "Usage: /workspace-review <task_id> <pass|fail> [message]".to_string();
                return;
            }
        };
        let Some(task_id) = self.resolve_workspace_task_id(raw_task_id) else {
            return;
        };
        let message = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        self.send_daemon_command(DaemonCommand::SubmitWorkspaceReview(
            zorai_protocol::WorkspaceReviewSubmission {
                task_id,
                verdict,
                message,
            },
        ));
        self.main_pane_view = MainPaneView::Workspace;
        self.status_line = "Submitting workspace review...".to_string();
    }

    pub(crate) fn open_workspace_task_runtime(&mut self, task_id: String) {
        let Some(task) = self.workspace.task_by_id(&task_id).cloned() else {
            self.status_line = "Workspace task not found".to_string();
            return;
        };
        if let Some(goal_run_id) = task.goal_run_id {
            self.open_sidebar_target(sidebar::SidebarItemTarget::GoalRun {
                goal_run_id: goal_run_id.clone(),
                step_id: None,
            });
            self.set_mission_control_return_to_workspace(true);
            self.status_line = format!("Opened workspace goal {goal_run_id}");
            return;
        }
        if let Some(thread_id) = task.thread_id.as_deref() {
            self.apply_workspace_task_thread_identity_hint(&thread_id, &task);
            self.open_thread_conversation(thread_id.to_string());
            self.set_mission_control_return_to_workspace(true);
            self.status_line = format!("Opened workspace thread {thread_id}");
            return;
        }
        self.status_line = "Workspace task has not been run yet".to_string();
    }

    fn apply_workspace_task_thread_identity_hint(
        &mut self,
        thread_id: &str,
        task: &zorai_protocol::WorkspaceTask,
    ) {
        let Some(agent_name) = task
            .assignee
            .as_ref()
            .and_then(|assignee| self.workspace_actor_display_name(assignee))
        else {
            return;
        };
        let existing_thread = self
            .chat
            .threads()
            .iter()
            .find(|thread| thread.id == thread_id);
        if existing_thread.is_some_and(|thread| {
            thread
                .agent_name
                .as_deref()
                .is_some_and(|name| !name.trim().is_empty())
        }) {
            return;
        }
        let title = existing_thread
            .map(|thread| thread.title.as_str())
            .filter(|title| !title.trim().is_empty())
            .unwrap_or(task.title.as_str())
            .to_string();
        self.chat
            .reduce(chat::ChatAction::ThreadDetailReceived(chat::AgentThread {
                id: thread_id.to_string(),
                agent_name: Some(agent_name),
                title,
                ..Default::default()
            }));
    }

    fn workspace_actor_display_name(
        &self,
        actor: &zorai_protocol::WorkspaceActor,
    ) -> Option<String> {
        match actor {
            zorai_protocol::WorkspaceActor::User => None,
            zorai_protocol::WorkspaceActor::Agent(agent_id)
            | zorai_protocol::WorkspaceActor::Subagent(agent_id) => {
                let agent_id = agent_id.trim();
                (!agent_id.is_empty()).then(|| self.participant_display_name(agent_id))
            }
        }
    }

    pub(crate) fn step_workspace_board_selection(&mut self, delta: i32) {
        self.workspace_board_selection = widgets::workspace_board::step_selection(
            &self.workspace,
            &self.workspace_expanded_task_ids,
            self.workspace_board_selection.as_ref(),
            delta,
        );
        self.sync_workspace_board_scroll_to_selection();
    }

    pub(super) fn sync_workspace_board_scroll_to_selection(&mut self) {
        let Some(target) = self.workspace_board_selection.as_ref() else {
            return;
        };
        let chat_area = self.pane_layout().chat;
        self.workspace_board_scroll = widgets::workspace_board::scroll_for_target(
            chat_area,
            &self.workspace,
            &self.workspace_expanded_task_ids,
            &self.workspace_board_scroll,
            target,
        );
    }

    pub(crate) fn step_workspace_board_scroll_at(&mut self, position: Position, delta: i32) {
        let chat_area = self.pane_layout().chat;
        let Some(status) = widgets::workspace_board::column_status_at_position(
            chat_area,
            &self.workspace,
            position,
        ) else {
            return;
        };
        self.workspace_board_scroll = widgets::workspace_board::stepped_scroll_for_status(
            &self.workspace,
            &self.workspace_board_scroll,
            &status,
            delta,
        );
    }

    pub(crate) fn activate_workspace_board_selection(&mut self) {
        let target = self.workspace_board_selection.clone().or_else(|| {
            widgets::workspace_board::selectable_targets(
                &self.workspace,
                &self.workspace_expanded_task_ids,
            )
            .into_iter()
            .next()
        });
        let Some(target) = target else {
            self.status_line = "No workspace action selected".to_string();
            return;
        };
        self.workspace_board_selection = Some(target.clone());
        self.activate_workspace_board_target(target);
    }

    pub(super) fn activate_workspace_board_target(
        &mut self,
        target: widgets::workspace_board::WorkspaceBoardHitTarget,
    ) {
        match target {
            widgets::workspace_board::WorkspaceBoardHitTarget::Toolbar(action) => {
                self.activate_workspace_toolbar_action(action);
            }
            widgets::workspace_board::WorkspaceBoardHitTarget::Task { task_id, .. } => {
                self.open_workspace_detail_modal(task_id);
            }
            widgets::workspace_board::WorkspaceBoardHitTarget::Action {
                task_id,
                status,
                action,
            } => {
                self.activate_workspace_task_action(task_id, status, action);
            }
            widgets::workspace_board::WorkspaceBoardHitTarget::Column { .. } => {}
        }
    }
}
