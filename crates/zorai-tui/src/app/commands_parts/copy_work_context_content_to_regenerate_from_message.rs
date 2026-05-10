use super::*;
impl TuiModel {
    pub(crate) fn copy_work_context_content(&mut self) {
        let Some(thread_id) = self.chat.active_thread_id().map(str::to_string) else {
            return;
        };

        let text = match self.sidebar.active_tab() {
            sidebar::SidebarTab::Files => {
                let Some(path) = self.selected_sidebar_file_path() else {
                    return;
                };
                let Some(entry) = self
                    .tasks
                    .work_context_for_thread(&thread_id)
                    .and_then(|context| context.entries.iter().find(|entry| entry.path == path))
                else {
                    return;
                };
                if let Some(repo_root) = entry.repo_root.as_deref() {
                    self.tasks
                        .diff_for_path(repo_root, &entry.path)
                        .map(str::to_string)
                        .filter(|value| !value.trim().is_empty())
                } else {
                    self.tasks
                        .preview_for_path(&entry.path)
                        .filter(|preview| preview.is_text)
                        .map(|preview| preview.content.clone())
                        .filter(|value| !value.trim().is_empty())
                }
            }
            sidebar::SidebarTab::Todos => self
                .tasks
                .todos_for_thread(&thread_id)
                .get(self.sidebar.selected_item())
                .map(|todo| todo.content.clone())
                .filter(|value| !value.trim().is_empty()),
            sidebar::SidebarTab::Spawned => None,
            sidebar::SidebarTab::Pinned => self
                .selected_sidebar_pinned_message()
                .map(|message| message.content)
                .filter(|value| !value.trim().is_empty()),
        };

        if let Some(text) = text {
            conversion::copy_to_clipboard(&text);
            self.status_line = "Copied to clipboard".to_string();
        }
    }

    pub(crate) fn resend_message(&mut self, index: usize) {
        let content = self
            .chat
            .active_thread()
            .and_then(|thread| thread.messages.get(index))
            .map(|message| message.content.clone());
        if let Some(content) = content.filter(|value| !value.trim().is_empty()) {
            self.submit_prompt(content);
        }
    }

    pub(crate) fn pin_message_for_compaction(&mut self, index: usize) {
        let (thread_id, message_id) = {
            let Some(thread) = self.chat.active_thread() else {
                return;
            };
            let Some(message) = thread.messages.get(index) else {
                return;
            };
            let Some(message_id) = message.id.clone().filter(|id| !id.is_empty()) else {
                self.status_line = "Cannot pin message without a daemon id".to_string();
                return;
            };
            (thread.id.clone(), message_id)
        };

        self.send_daemon_command(DaemonCommand::PinThreadMessageForCompaction {
            thread_id,
            message_id,
        });
    }

    pub(crate) fn unpin_message_for_compaction(&mut self, index: usize) {
        let (thread_id, message_id) = {
            let Some(thread) = self.chat.active_thread() else {
                return;
            };
            let Some(message) = thread.messages.get(index) else {
                return;
            };
            let Some(message_id) = message.id.clone().filter(|id| !id.is_empty()) else {
                self.status_line = "Cannot unpin message without a daemon id".to_string();
                return;
            };
            (thread.id.clone(), message_id)
        };

        let absolute_index = self
            .chat
            .active_thread()
            .map(|thread| thread.loaded_message_start.saturating_add(index));
        self.unpin_message_for_compaction_by_id(thread_id, message_id, absolute_index);
    }

    fn unpin_message_for_compaction_by_id(
        &mut self,
        thread_id: String,
        message_id: String,
        absolute_index: Option<usize>,
    ) {
        self.send_daemon_command(DaemonCommand::UnpinThreadMessageForCompaction {
            thread_id: thread_id.clone(),
            message_id: message_id.clone(),
        });
        self.chat
            .reduce(chat::ChatAction::UnpinMessageForCompaction {
                thread_id,
                message_id,
                absolute_index,
            });
        if self.sidebar.active_tab() == sidebar::SidebarTab::Pinned
            && !self.chat.active_thread_has_pinned_messages()
        {
            self.sidebar.reduce(sidebar::SidebarAction::SwitchTab(
                sidebar::SidebarTab::Todos,
            ));
        }
    }

    pub(crate) fn unpin_selected_sidebar_message(&mut self) {
        let Some(pinned_message) = self.selected_sidebar_pinned_message() else {
            return;
        };
        let Some(thread_id) = self.chat.active_thread_id().map(str::to_string) else {
            return;
        };
        self.unpin_message_for_compaction_by_id(
            thread_id,
            pinned_message.message_id,
            Some(pinned_message.absolute_index),
        );
    }

    pub(crate) fn delete_message(&mut self, index: usize) {
        let (thread_id, msg_id, has_persistent_id) = {
            let Some(thread) = self.chat.active_thread() else {
                return;
            };
            if index >= thread.messages.len() {
                return;
            }
            let persistent_id = thread.messages[index].id.clone();
            let mid = persistent_id
                .clone()
                .unwrap_or_else(|| format!("{}:{}", thread.id, index));
            (thread.id.clone(), mid, persistent_id.is_some())
        };

        self.send_daemon_command(DaemonCommand::DeleteMessages {
            thread_id: thread_id.clone(),
            message_ids: vec![msg_id],
        });

        let viewport_anchor = self.capture_locked_chat_viewport(Some(thread_id.as_str()));
        self.chat.delete_active_message(index);
        self.chat_selection_snapshot = None;
        self.restore_locked_chat_viewport(viewport_anchor);
        if has_persistent_id {
            *self
                .pending_local_message_delete_reload_suppression
                .entry(thread_id.clone())
                .or_insert(0) += 1;
        }
        self.queue_older_messages_after_delete(&thread_id);
        self.status_line = format!("Deleted message {}", index + 1);
    }

    fn queue_older_messages_after_delete(&mut self, thread_id: &str) {
        let target_size = self.chat_history_delete_backfill_target_size();
        let Some((window, loaded_count)) = self.chat.active_thread().and_then(|thread| {
            if thread.id != thread_id
                || thread.loaded_message_start == 0
                || thread.messages.len() >= target_size
            {
                return None;
            }

            Some((
                chat::chat_window::MessageWindow::from_thread(thread),
                thread.messages.len(),
            ))
        }) else {
            tracing::info!(
                thread_id,
                target_loaded_count = target_size,
                "delete older-message backfill not queued"
            );
            return;
        };

        let pending = self
            .pending_local_message_delete_backfills
            .entry(thread_id.to_string())
            .or_insert(0);
        *pending = pending.saturating_add(1);
        let pending_backfills = *pending;
        let local_deleted_count = self.chat.local_deleted_message_count_for_thread(thread_id);
        tracing::info!(
            thread_id,
            pending_backfills,
            threshold = MESSAGE_DELETE_BACKFILL_THRESHOLD,
            target_loaded_count = target_size,
            loaded_count,
            local_deleted_count,
            loaded_start = window.start,
            loaded_end = window.end,
            total_messages = window.total,
            "queued delete older-message backfill"
        );
        if pending_backfills < MESSAGE_DELETE_BACKFILL_THRESHOLD {
            return;
        }

        let outstanding_rows = self
            .pending_local_message_delete_fetches
            .get(thread_id)
            .map(|fetch| fetch.outstanding_rows)
            .unwrap_or(0);
        let fetch_start = window.start.saturating_sub(outstanding_rows);
        let message_limit = pending_backfills.min(fetch_start);
        let message_offset = window.total.saturating_sub(fetch_start);
        let Some(request) = (message_limit > 0).then_some(chat::chat_window::ThreadPageRequest {
            message_limit,
            message_offset,
        }) else {
            tracing::info!(
                thread_id,
                pending_backfills,
                outstanding_rows,
                local_deleted_count,
                loaded_start = window.start,
                loaded_end = window.end,
                total_messages = window.total,
                "delete older-message backfill threshold reached but no older rows are available"
            );
            return;
        };

        if self.chat.active_thread_older_page_pending() {
            self.pending_local_message_delete_backfills
                .remove(thread_id);
            tracing::info!(
                thread_id,
                pending_backfills,
                message_limit = request.message_limit,
                message_offset = request.message_offset,
                outstanding_rows,
                local_deleted_count,
                loaded_start = window.start,
                loaded_end = window.end,
                total_messages = window.total,
                "delete older-message backfill coalesced with pending older fetch"
            );
            return;
        }

        self.pending_local_message_delete_backfills
            .remove(thread_id);

        tracing::info!(
            thread_id,
            pending_backfills,
            message_limit = request.message_limit,
            message_offset = request.message_offset,
            outstanding_rows,
            local_deleted_count,
            loaded_start = window.start,
            loaded_end = window.end,
            total_messages = window.total,
            "requesting older messages after delete threshold"
        );
        self.pending_local_message_delete_fetches
            .entry(thread_id.to_string())
            .and_modify(|fetch| {
                fetch.message_limit = request.message_limit;
                fetch.message_offset = request.message_offset;
                fetch.outstanding_rows =
                    fetch.outstanding_rows.saturating_add(request.message_limit);
                fetch.requested_at_tick = self.tick_counter;
            })
            .or_insert(PendingDeleteBackfillFetch {
                message_limit: request.message_limit,
                message_offset: request.message_offset,
                outstanding_rows: request.message_limit,
                requested_at_tick: self.tick_counter,
            });
        self.request_thread_page(
            thread_id.to_string(),
            request.message_limit,
            request.message_offset,
            false,
        );
    }

    pub(crate) fn regenerate_from_message(&mut self, index: usize) {
        let prompt = self.chat.active_thread().and_then(|thread| {
            thread
                .messages
                .iter()
                .take(index)
                .rev()
                .find(|message| {
                    message.role == chat::MessageRole::User && !message.content.trim().is_empty()
                })
                .map(|message| message.content.clone())
        });
        if let Some(prompt) = prompt {
            self.submit_prompt(prompt);
        }
    }
}
