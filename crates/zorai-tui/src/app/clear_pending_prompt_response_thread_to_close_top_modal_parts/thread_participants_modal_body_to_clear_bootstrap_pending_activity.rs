use super::*;
use crate::widgets;
impl TuiModel {
    pub(crate) fn thread_participants_modal_body(&self) -> String {
        let Some(thread) = self.chat.active_thread() else {
            return "No active thread selected.".to_string();
        };

        let active: Vec<_> = thread
            .thread_participants
            .iter()
            .filter(|participant| participant.status.eq_ignore_ascii_case("active"))
            .collect();
        let inactive: Vec<_> = thread
            .thread_participants
            .iter()
            .filter(|participant| !participant.status.eq_ignore_ascii_case("active"))
            .collect();

        let mut body = String::new();
        body.push_str(&format!("Thread: {}\n", thread.title));
        body.push_str("==============================\n\n");

        body.push_str("Active Participants\n");
        body.push_str("-------------------\n");
        if active.is_empty() {
            body.push_str("- none\n");
        } else {
            for participant in active {
                body.push_str(&format!(
                    "- {} ({})\n  instruction: {}\n",
                    participant.agent_name,
                    participant.agent_id,
                    participant.instruction.trim()
                ));
            }
        }
        body.push('\n');

        body.push_str("Inactive Participants\n");
        body.push_str("---------------------\n");
        if inactive.is_empty() {
            body.push_str("- none\n");
        } else {
            for participant in inactive {
                body.push_str(&format!(
                    "- {} ({})\n  instruction: {}\n",
                    participant.agent_name,
                    participant.agent_id,
                    participant.instruction.trim()
                ));
            }
        }
        body.push('\n');

        body.push_str("Queued Suggestions\n");
        body.push_str("------------------\n");
        if thread.queued_participant_suggestions.is_empty() {
            body.push_str("- none\n");
        } else {
            for suggestion in &thread.queued_participant_suggestions {
                let mut badges = vec![suggestion.status.clone()];
                if suggestion.force_send {
                    badges.push("force_send".to_string());
                }
                body.push_str(&format!(
                    "- {} [{}]\n  message: {}\n",
                    suggestion.target_agent_name,
                    badges.join(", "),
                    suggestion.instruction.trim()
                ));
                if let Some(error) = suggestion.error.as_deref() {
                    if !error.trim().is_empty() {
                        body.push_str(&format!("  error: {}\n", error.trim()));
                    }
                }
            }
        }

        body
    }

    pub(crate) fn prompt_modal_max_scroll(&self) -> usize {
        let body = self.prompt_modal_body();
        let (viewport_lines, inner_width) = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::PromptViewer)
            .map(|(_, area)| {
                (
                    area.height.saturating_sub(3) as usize,
                    area.width.saturating_sub(2) as usize,
                )
            })
            .unwrap_or((1, 1));
        let total_lines = crate::widgets::message::wrap_text(&body, inner_width.max(1))
            .len()
            .max(1);
        let viewport_lines = viewport_lines.max(1);
        total_lines.saturating_sub(viewport_lines)
    }

    pub(crate) fn status_modal_max_scroll(&self) -> usize {
        let body = self.status_modal_body();
        let (viewport_lines, inner_width) = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Status)
            .map(|(_, area)| {
                (
                    area.height.saturating_sub(3) as usize,
                    area.width.saturating_sub(2) as usize,
                )
            })
            .unwrap_or((1, 1));
        let total_lines = crate::widgets::message::wrap_text(&body, inner_width.max(1))
            .len()
            .max(1);
        let viewport_lines = viewport_lines.max(1);
        total_lines.saturating_sub(viewport_lines)
    }

    pub(crate) fn statistics_modal_max_scroll(&self) -> usize {
        let body = self.statistics_modal_body();
        let (viewport_lines, inner_width) = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Statistics)
            .map(|(_, area)| {
                (
                    area.height.saturating_sub(5) as usize,
                    area.width.saturating_sub(2) as usize,
                )
            })
            .unwrap_or((1, 1));
        let total_lines = crate::widgets::message::wrap_text(&body, inner_width.max(1))
            .len()
            .max(1);
        let viewport_lines = viewport_lines.max(1);
        total_lines.saturating_sub(viewport_lines)
    }

    pub(crate) fn thread_participants_modal_max_scroll(&self) -> usize {
        let body = self.thread_participants_modal_body();
        let (viewport_lines, inner_width) = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::ThreadParticipants)
            .map(|(_, area)| {
                (
                    area.height.saturating_sub(3) as usize,
                    area.width.saturating_sub(2) as usize,
                )
            })
            .unwrap_or((1, 1));
        let total_lines = crate::widgets::message::wrap_text(&body, inner_width.max(1))
            .len()
            .max(1);
        let viewport_lines = viewport_lines.max(1);
        total_lines.saturating_sub(viewport_lines)
    }

    pub(crate) fn help_modal_max_scroll(&self) -> usize {
        let body = render_helpers::help_modal_text();
        let (viewport_lines, inner_width) = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Help)
            .map(|(_, area)| {
                (
                    area.height.saturating_sub(3) as usize,
                    area.width.saturating_sub(2) as usize,
                )
            })
            .unwrap_or((1, 1));
        let total_lines = crate::widgets::message::wrap_text(&body, inner_width.max(1))
            .len()
            .max(1);
        let viewport_lines = viewport_lines.max(1);
        total_lines.saturating_sub(viewport_lines)
    }

    pub(crate) fn set_status_modal_scroll(&mut self, scroll: usize) {
        self.status_modal_scroll = scroll.min(self.status_modal_max_scroll());
    }

    pub(crate) fn set_statistics_modal_scroll(&mut self, scroll: usize) {
        self.statistics_modal_scroll = scroll.min(self.statistics_modal_max_scroll());
    }

    pub(crate) fn step_status_modal_scroll(&mut self, delta: i32) {
        let current = self.status_modal_scroll as i32;
        let next = (current + delta).max(0) as usize;
        self.set_status_modal_scroll(next);
    }

    pub(crate) fn step_statistics_modal_scroll(&mut self, delta: i32) {
        let current = self.statistics_modal_scroll as i32;
        let next = (current + delta).max(0) as usize;
        self.set_statistics_modal_scroll(next);
    }

    pub(crate) fn page_status_modal_scroll(&mut self, direction: i32) {
        let page = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Status)
            .map(|(_, area)| area.height.saturating_sub(4) as i32)
            .unwrap_or(10)
            .max(1);
        self.step_status_modal_scroll(page * direction);
    }

    pub(crate) fn page_statistics_modal_scroll(&mut self, direction: i32) {
        let page = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Statistics)
            .map(|(_, area)| area.height.saturating_sub(6) as i32)
            .unwrap_or(10)
            .max(1);
        self.step_statistics_modal_scroll(page * direction);
    }

    pub(crate) fn set_prompt_modal_scroll(&mut self, scroll: usize) {
        self.prompt_modal_scroll = scroll.min(self.prompt_modal_max_scroll());
    }

    pub(crate) fn settings_modal_max_scroll(&self) -> usize {
        self.current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Settings)
            .map(|(_, area)| {
                widgets::settings::max_scroll(
                    area,
                    &self.settings,
                    &self.config,
                    &self.modal,
                    &self.auth,
                    &self.subagents,
                    &self.concierge,
                    &self.tier,
                    &self.plugin_settings,
                    &self.theme,
                )
            })
            .unwrap_or(0)
    }

    pub(crate) fn set_settings_modal_scroll(&mut self, scroll: usize) {
        self.settings_modal_scroll = scroll.min(self.settings_modal_max_scroll());
    }

    pub(crate) fn step_settings_modal_scroll(&mut self, delta: i32) {
        let current = self.settings_modal_scroll as i32;
        let next = (current + delta).max(0) as usize;
        self.set_settings_modal_scroll(next);
    }

    pub(crate) fn page_settings_modal_scroll(&mut self, direction: i32) {
        let page = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Settings)
            .map(|(_, area)| area.height.saturating_sub(4) as i32)
            .unwrap_or(10)
            .max(1);
        self.step_settings_modal_scroll(page * direction);
    }

    pub(crate) fn sync_settings_modal_scroll_to_selection(&mut self) {
        let Some((kind, area)) = self.current_modal_area() else {
            return;
        };
        if kind != modal::ModalKind::Settings {
            return;
        }

        self.settings_modal_scroll = widgets::settings::scroll_for_selected_field(
            area,
            &self.settings,
            &self.config,
            &self.modal,
            &self.auth,
            &self.subagents,
            &self.concierge,
            &self.tier,
            &self.plugin_settings,
            self.settings_modal_scroll,
            &self.theme,
        );
    }

    pub(crate) fn set_thread_participants_modal_scroll(&mut self, scroll: usize) {
        self.thread_participants_modal_scroll =
            scroll.min(self.thread_participants_modal_max_scroll());
    }

    pub(crate) fn set_help_modal_scroll(&mut self, scroll: usize) {
        self.help_modal_scroll = scroll.min(self.help_modal_max_scroll());
    }

    pub(crate) fn step_prompt_modal_scroll(&mut self, delta: i32) {
        let current = self.prompt_modal_scroll as i32;
        let next = (current + delta).max(0) as usize;
        self.set_prompt_modal_scroll(next);
    }

    pub(crate) fn page_prompt_modal_scroll(&mut self, direction: i32) {
        let page = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::PromptViewer)
            .map(|(_, area)| area.height.saturating_sub(4) as i32)
            .unwrap_or(10)
            .max(1);
        self.step_prompt_modal_scroll(page * direction);
    }

    pub(crate) fn step_thread_participants_modal_scroll(&mut self, delta: i32) {
        let current = self.thread_participants_modal_scroll as i32;
        let next = (current + delta).max(0) as usize;
        self.set_thread_participants_modal_scroll(next);
    }

    pub(crate) fn step_help_modal_scroll(&mut self, delta: i32) {
        let current = self.help_modal_scroll as i32;
        let next = (current + delta).max(0) as usize;
        self.set_help_modal_scroll(next);
    }

    pub(crate) fn page_thread_participants_modal_scroll(&mut self, direction: i32) {
        let page = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::ThreadParticipants)
            .map(|(_, area)| area.height.saturating_sub(4) as i32)
            .unwrap_or(10)
            .max(1);
        self.step_thread_participants_modal_scroll(page * direction);
    }

    pub(crate) fn page_help_modal_scroll(&mut self, direction: i32) {
        let page = self
            .current_modal_area()
            .filter(|(kind, _)| *kind == modal::ModalKind::Help)
            .map(|(_, area)| area.height.saturating_sub(4) as i32)
            .unwrap_or(10)
            .max(1);
        self.step_help_modal_scroll(page * direction);
    }

    pub(crate) fn open_thread_participants_modal(&mut self) {
        if self.chat.active_thread().is_none() {
            self.status_line = "Open a thread first, then run /participants".to_string();
            return;
        }
        self.thread_participants_modal_scroll = 0;
        if self.modal.top() != Some(modal::ModalKind::ThreadParticipants) {
            self.modal.reduce(modal::ModalAction::Push(
                modal::ModalKind::ThreadParticipants,
            ));
        }
    }

    pub(crate) fn request_prompt_inspection(&mut self, agent_id: Option<String>) {
        self.open_prompt_modal_loading();
        self.send_daemon_command(DaemonCommand::RequestPromptInspection { agent_id });
        self.status_line = "Requesting assembled agent prompt...".to_string();
    }

    pub(crate) fn request_statistics_window(
        &mut self,
        window: zorai_protocol::AgentStatisticsWindow,
    ) {
        self.statistics_modal_window = window;
        self.open_statistics_modal_loading();
        self.send_daemon_command(DaemonCommand::RequestAgentStatistics { window });
        self.status_line = format!("Loading statistics for {}...", window.as_str());
    }

    pub(crate) fn select_statistics_tab(&mut self, tab: crate::state::statistics::StatisticsTab) {
        self.statistics_modal_tab = tab;
        self.statistics_modal_scroll = 0;
    }

    pub(crate) fn cycle_statistics_tab(&mut self, direction: i32) {
        let next = if direction >= 0 {
            self.statistics_modal_tab.next()
        } else {
            self.statistics_modal_tab.prev()
        };
        self.select_statistics_tab(next);
    }

    pub(crate) fn cycle_statistics_window(&mut self, direction: i32) {
        let windows = [
            zorai_protocol::AgentStatisticsWindow::Today,
            zorai_protocol::AgentStatisticsWindow::Last7Days,
            zorai_protocol::AgentStatisticsWindow::Last30Days,
            zorai_protocol::AgentStatisticsWindow::All,
        ];
        let current_index = windows
            .iter()
            .position(|window| *window == self.statistics_modal_window)
            .unwrap_or(3) as i32;
        let len = windows.len() as i32;
        let next_index = (current_index + direction).rem_euclid(len) as usize;
        self.request_statistics_window(windows[next_index]);
    }

    pub(crate) fn show_input_notice(
        &mut self,
        text: impl Into<String>,
        kind: InputNoticeKind,
        duration_ticks: u64,
        dismiss_on_interaction: bool,
    ) {
        self.input_notice = Some(InputNotice {
            text: text.into(),
            kind,
            expires_at_tick: self.tick_counter.saturating_add(duration_ticks),
            dismiss_on_interaction,
        });
    }

    pub(crate) fn clear_dismissable_input_notice(&mut self) {
        if self
            .input_notice
            .as_ref()
            .is_some_and(|notice| notice.dismiss_on_interaction)
        {
            self.input_notice = None;
        }
    }

    pub(crate) fn begin_thread_loading(&mut self, thread_id: impl Into<String>) {
        let thread_id = thread_id.into();
        self.thread_loading_id = Some(thread_id.clone());
        self.status_line = match self.chat.active_thread() {
            Some(thread) if !thread.title.trim().is_empty() => {
                format!("Loading thread: {}", thread.title.trim())
            }
            _ => format!("Loading thread: {thread_id}"),
        };
    }

    pub(crate) fn finish_thread_loading(&mut self, thread_id: &str) {
        if self.thread_loading_id.as_deref() == Some(thread_id) {
            self.thread_loading_id = None;
        }
    }

    pub(crate) fn should_show_thread_loading(&self) -> bool {
        self.thread_loading_id
            .as_deref()
            .is_some_and(|thread_id| self.chat.active_thread_id() == Some(thread_id))
            && self
                .chat
                .active_thread()
                .is_some_and(|thread| thread.messages.is_empty())
            && !self.chat.is_streaming()
    }

    pub(crate) fn clear_pending_stop(&mut self) {
        self.pending_stop = false;
        self.clear_dismissable_input_notice();
    }

    pub(crate) fn pending_stop_active(&self) -> bool {
        self.pending_stop && self.tick_counter.saturating_sub(self.pending_stop_tick) < 100
    }

    pub(crate) fn current_thread_agent_activity(&self) -> Option<&str> {
        self.chat
            .active_thread_id()
            .and_then(|thread_id| self.thread_agent_activity.get(thread_id))
            .map(String::as_str)
            .or(self.agent_activity.as_deref())
    }

    pub(crate) fn set_agent_activity_for(
        &mut self,
        thread_id: Option<String>,
        activity: impl Into<String>,
    ) {
        let activity = activity.into();
        if let Some(thread_id) = thread_id {
            if activity != "thinking" {
                self.clear_pending_prompt_response_thread(thread_id.as_str());
            }
            self.thread_agent_activity.insert(thread_id, activity);
        } else {
            self.agent_activity = Some(activity);
        }
    }

    pub(crate) fn set_active_thread_activity(&mut self, activity: impl Into<String>) {
        self.set_agent_activity_for(self.chat.active_thread_id().map(str::to_string), activity);
    }

    pub(crate) fn mark_bootstrap_pending_activity_thread(&mut self, thread_id: impl Into<String>) {
        self.bootstrap_pending_activity_threads
            .insert(thread_id.into());
    }

    pub(crate) fn mark_pending_prompt_response_thread(&mut self, thread_id: impl Into<String>) {
        self.pending_prompt_response_threads
            .insert(thread_id.into());
    }

    pub(crate) fn clear_bootstrap_pending_activity_thread(&mut self, thread_id: &str) {
        self.bootstrap_pending_activity_threads.remove(thread_id);
    }
}
