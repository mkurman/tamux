use super::spawned_agents;
use super::*;
use crate::state::chat::{ChatState, MessageRole};
use crate::state::sidebar::{SidebarState, SidebarTab};
use crate::state::task::TaskState;
use ratatui::prelude::*;
use ratatui::text::Line;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub(crate) struct SidebarRow {
    pub(crate) line: Line<'static>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarHitTarget {
    Tab(SidebarTab),
    File(String),
    Todo(usize),
    Spawned(usize),
    Pinned(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SidebarSnapshotKey {
    pub(crate) width: u16,
    pub(crate) active_tab: SidebarTab,
    pub(crate) thread_id: Option<String>,
    pub(crate) files_filter: String,
    pub(crate) show_spawned: bool,
    pub(crate) show_pinned: bool,
    pub(crate) body_hash: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct FileSidebarItem {
    pub(crate) path: String,
    pub(crate) label: String,
    pub(crate) display_path: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TodoSidebarItem {
    pub(crate) index: usize,
    pub(crate) marker: &'static str,
    pub(crate) text: String,
}

#[derive(Debug, Clone)]
pub(crate) struct PinnedSidebarItem {
    pub(crate) index: usize,
    pub(crate) metadata: String,
    pub(crate) snippet: String,
}

#[derive(Debug, Clone)]
pub(crate) enum SidebarBodySnapshot {
    Empty { message: String },
    Files(Vec<FileSidebarItem>),
    Todos(Vec<TodoSidebarItem>),
    Spawned(Vec<spawned_agents::SpawnedSidebarItem>),
    Pinned(Vec<PinnedSidebarItem>),
}

#[derive(Debug, Clone)]
pub struct CachedSidebarSnapshot {
    pub(crate) key: SidebarSnapshotKey,
    pub(crate) body: SidebarBodySnapshot,
}

impl CachedSidebarSnapshot {
    pub(crate) fn show_spawned(&self) -> bool {
        self.key.show_spawned
    }

    pub(crate) fn show_pinned(&self) -> bool {
        self.key.show_pinned
    }

    pub fn item_count(&self) -> usize {
        match &self.body {
            SidebarBodySnapshot::Empty { .. } => 1,
            SidebarBodySnapshot::Files(items) => items.len().max(1),
            SidebarBodySnapshot::Todos(items) => items.len().max(1),
            SidebarBodySnapshot::Spawned(items) => items.len().max(1),
            SidebarBodySnapshot::Pinned(items) => items.len().max(1),
        }
    }

    pub fn selected_file_path(&self, selected_index: usize) -> Option<String> {
        let SidebarBodySnapshot::Files(items) = &self.body else {
            return None;
        };
        let selected = selected_index.min(items.len().saturating_sub(1));
        items.get(selected).map(|item| item.path.clone())
    }

    pub fn filtered_file_index(&self, path: &str) -> Option<usize> {
        let SidebarBodySnapshot::Files(items) = &self.body else {
            return None;
        };
        items.iter().position(|item| item.path == path)
    }

    pub fn selected_spawned_thread_id(&self, selected_index: usize) -> Option<String> {
        let SidebarBodySnapshot::Spawned(items) = &self.body else {
            return None;
        };
        items
            .get(selected_index)
            .and_then(|item| item.target_thread_id.clone())
    }

    pub fn first_openable_spawned_index(&self) -> Option<usize> {
        let SidebarBodySnapshot::Spawned(items) = &self.body else {
            return None;
        };
        items.iter().position(|item| item.openable)
    }

    pub fn selected_pinned_message(
        &self,
        chat: &ChatState,
        selected_index: usize,
    ) -> Option<crate::state::chat::PinnedThreadMessage> {
        let SidebarBodySnapshot::Pinned(items) = &self.body else {
            return None;
        };
        let index = items.get(selected_index)?.index;
        chat.active_thread_pinned_messages().into_iter().nth(index)
    }

    pub(crate) fn row_target(&self, row_index: usize) -> Option<SidebarHitTarget> {
        match &self.body {
            SidebarBodySnapshot::Empty { .. } => None,
            SidebarBodySnapshot::Files(items) => items
                .get(row_index)
                .map(|item| SidebarHitTarget::File(item.path.clone())),
            SidebarBodySnapshot::Todos(items) => items
                .get(row_index)
                .map(|item| SidebarHitTarget::Todo(item.index)),
            SidebarBodySnapshot::Spawned(items) => items
                .get(row_index)
                .map(|_| SidebarHitTarget::Spawned(row_index)),
            SidebarBodySnapshot::Pinned(items) => items
                .get(row_index)
                .map(|item| SidebarHitTarget::Pinned(item.index)),
        }
    }
}

#[cfg(test)]
thread_local! {
    pub(super) static BUILD_CACHED_SNAPSHOT_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(crate) type PinnedSidebarRows = Vec<crate::state::chat::PinnedThreadMessage>;

pub(crate) fn file_entry_matches(
    entry: &crate::state::task::WorkContextEntry,
    filter: &str,
) -> bool {
    let query = filter.trim();
    if query.is_empty() {
        return true;
    }
    let query = query.to_ascii_lowercase();
    entry.path.to_ascii_lowercase().contains(&query)
        || entry
            .previous_path
            .as_deref()
            .is_some_and(|path| path.to_ascii_lowercase().contains(&query))
        || entry
            .change_kind
            .as_deref()
            .is_some_and(|kind| kind.to_ascii_lowercase().contains(&query))
}

pub(crate) fn filtered_file_entries<'a>(
    tasks: &'a TaskState,
    thread_id: Option<&str>,
    sidebar: &SidebarState,
) -> Vec<&'a crate::state::task::WorkContextEntry> {
    let Some(thread_id) = thread_id else {
        return Vec::new();
    };
    let Some(context) = tasks.work_context_for_thread(thread_id) else {
        return Vec::new();
    };
    context
        .entries
        .iter()
        .filter(|entry| file_entry_matches(entry, sidebar.files_filter()))
        .collect()
}

pub(crate) fn hash_sidebar_tab<H: Hasher>(hasher: &mut H, tab: SidebarTab) {
    match tab {
        SidebarTab::Files => 0u8.hash(hasher),
        SidebarTab::Todos => 1u8.hash(hasher),
        SidebarTab::Spawned => 2u8.hash(hasher),
        SidebarTab::Pinned => 3u8.hash(hasher),
    }
}

pub(crate) fn hash_message_role<H: Hasher>(hasher: &mut H, role: MessageRole) {
    match role {
        MessageRole::User => 0u8.hash(hasher),
        MessageRole::Assistant => 1u8.hash(hasher),
        MessageRole::System => 2u8.hash(hasher),
        MessageRole::Tool => 3u8.hash(hasher),
        MessageRole::Unknown => 4u8.hash(hasher),
    }
}

pub(crate) fn hash_task_status<H: Hasher>(
    hasher: &mut H,
    status: Option<crate::state::task::TaskStatus>,
) {
    match status {
        Some(crate::state::task::TaskStatus::Queued) => 0u8.hash(hasher),
        Some(crate::state::task::TaskStatus::InProgress) => 1u8.hash(hasher),
        Some(crate::state::task::TaskStatus::AwaitingApproval) => 2u8.hash(hasher),
        Some(crate::state::task::TaskStatus::Blocked) => 3u8.hash(hasher),
        Some(crate::state::task::TaskStatus::FailedAnalyzing) => 4u8.hash(hasher),
        Some(crate::state::task::TaskStatus::BudgetExceeded) => 5u8.hash(hasher),
        Some(crate::state::task::TaskStatus::Completed) => 6u8.hash(hasher),
        Some(crate::state::task::TaskStatus::Failed) => 7u8.hash(hasher),
        Some(crate::state::task::TaskStatus::Cancelled) => 8u8.hash(hasher),
        None => 9u8.hash(hasher),
    }
}

pub(crate) fn sidebar_snapshot_key(
    area: Rect,
    chat: &ChatState,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
) -> SidebarSnapshotKey {
    let show_spawned = has_spawned_tab(tasks, chat, thread_id);
    let pinned_rows = active_thread_pinned_rows(chat);
    let show_pinned = !pinned_rows.is_empty();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hash_sidebar_tab(&mut hasher, sidebar.active_tab());
    thread_id.hash(&mut hasher);
    sidebar.files_filter().hash(&mut hasher);
    show_spawned.hash(&mut hasher);
    show_pinned.hash(&mut hasher);

    match sidebar.active_tab() {
        SidebarTab::Files => {
            if let Some(thread_id) = thread_id {
                if let Some(context) = tasks.work_context_for_thread(thread_id) {
                    for entry in context
                        .entries
                        .iter()
                        .filter(|entry| file_entry_matches(entry, sidebar.files_filter()))
                    {
                        entry.path.hash(&mut hasher);
                        entry.previous_path.hash(&mut hasher);
                        entry.change_kind.hash(&mut hasher);
                        entry.is_text.hash(&mut hasher);
                    }
                }
            }
        }
        SidebarTab::Todos => {
            if let Some(thread_id) = thread_id {
                for todo in tasks.todos_for_thread(thread_id) {
                    todo.id.hash(&mut hasher);
                    todo.content.hash(&mut hasher);
                    todo.position.hash(&mut hasher);
                    hash_task_status(
                        &mut hasher,
                        todo.status.map(|status| match status {
                            crate::state::task::TodoStatus::Pending => {
                                crate::state::task::TaskStatus::Queued
                            }
                            crate::state::task::TodoStatus::InProgress => {
                                crate::state::task::TaskStatus::InProgress
                            }
                            crate::state::task::TodoStatus::Completed => {
                                crate::state::task::TaskStatus::Completed
                            }
                            crate::state::task::TodoStatus::Blocked => {
                                crate::state::task::TaskStatus::Blocked
                            }
                        }),
                    );
                }
            }
        }
        SidebarTab::Spawned => {
            tasks.tasks_revision().hash(&mut hasher);
            chat.can_go_back_thread().hash(&mut hasher);
        }
        SidebarTab::Pinned => {
            for message in &pinned_rows {
                message.message_id.hash(&mut hasher);
                message.absolute_index.hash(&mut hasher);
                hash_message_role(&mut hasher, message.role);
                message.content.hash(&mut hasher);
            }
        }
    }

    SidebarSnapshotKey {
        width: area.width,
        active_tab: sidebar.active_tab(),
        thread_id: thread_id.map(str::to_string),
        files_filter: sidebar.files_filter().to_string(),
        show_spawned,
        show_pinned,
        body_hash: hasher.finish(),
    }
}

pub(crate) fn truncate_tail(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    if max_len <= 1 {
        return "…".to_string();
    }
    let tail: String = text
        .chars()
        .rev()
        .take(max_len.saturating_sub(1))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("…{tail}")
}

pub(crate) fn build_body_snapshot(
    area: Rect,
    chat: &ChatState,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
) -> SidebarBodySnapshot {
    #[cfg(test)]
    BUILD_CACHED_SNAPSHOT_CALLS.with(|calls| calls.set(calls.get() + 1));

    let width = area.width as usize;

    match (sidebar.active_tab(), thread_id) {
        (SidebarTab::Files, None) => SidebarBodySnapshot::Empty {
            message: " No thread selected".to_string(),
        },
        (SidebarTab::Files, Some(thread_id)) => {
            let entries = filtered_file_entries(tasks, Some(thread_id), sidebar);
            if entries.is_empty() {
                return SidebarBodySnapshot::Empty {
                    message: if sidebar.files_filter().is_empty() {
                        " No files".to_string()
                    } else {
                        " No files match filter".to_string()
                    },
                };
            }

            SidebarBodySnapshot::Files(
                entries
                    .into_iter()
                    .map(|entry| {
                        let label = entry.change_kind.as_deref().unwrap_or_else(|| {
                            entry
                                .kind
                                .map(|kind| match kind {
                                    crate::state::task::WorkContextEntryKind::RepoChange => "diff",
                                    crate::state::task::WorkContextEntryKind::Artifact => "file",
                                    crate::state::task::WorkContextEntryKind::GeneratedSkill => {
                                        "skill"
                                    }
                                })
                                .unwrap_or("file")
                        });

                        FileSidebarItem {
                            path: entry.path.clone(),
                            label: label.to_string(),
                            display_path: truncate_tail(
                                &entry.path,
                                width.saturating_sub(12).max(8),
                            ),
                        }
                    })
                    .collect(),
            )
        }
        (SidebarTab::Todos, None) => SidebarBodySnapshot::Empty {
            message: " No thread selected".to_string(),
        },
        (SidebarTab::Todos, Some(thread_id)) => {
            let todos = tasks.todos_for_thread(thread_id);
            if todos.is_empty() {
                return SidebarBodySnapshot::Empty {
                    message: " No todos".to_string(),
                };
            }

            SidebarBodySnapshot::Todos(
                todos
                    .iter()
                    .enumerate()
                    .map(|(idx, todo)| TodoSidebarItem {
                        index: idx,
                        marker: match todo.status {
                            Some(crate::state::task::TodoStatus::Completed) => "[x]",
                            Some(crate::state::task::TodoStatus::InProgress) => "[~]",
                            Some(crate::state::task::TodoStatus::Blocked) => "[!]",
                            _ => "[ ]",
                        },
                        text: if todo.content.chars().count() > width.saturating_sub(8).max(8) {
                            format!(
                                "{}…",
                                todo.content
                                    .chars()
                                    .take(width.saturating_sub(9).max(7))
                                    .collect::<String>()
                            )
                        } else {
                            todo.content.clone()
                        },
                    })
                    .collect(),
            )
        }
        (SidebarTab::Spawned, _) => {
            let items = spawned_agents::flattened_items(tasks, thread_id);
            if items.is_empty() {
                SidebarBodySnapshot::Empty {
                    message: " No spawned agents".to_string(),
                }
            } else {
                SidebarBodySnapshot::Spawned(items)
            }
        }
        (SidebarTab::Pinned, _) => {
            let pinned_rows = active_thread_pinned_rows(chat);
            if pinned_rows.is_empty() {
                return SidebarBodySnapshot::Empty {
                    message: " No pinned messages".to_string(),
                };
            }

            SidebarBodySnapshot::Pinned(
                pinned_rows
                    .iter()
                    .enumerate()
                    .map(|(index, message)| PinnedSidebarItem {
                        index,
                        metadata: format!(
                            "[{} {}c]",
                            pinned_message_role_label(message.role),
                            pinned_message_chars(message)
                        ),
                        snippet: pinned_message_snippet(&message.content, width),
                    })
                    .collect(),
            )
        }
    }
}

pub(crate) fn build_cached_snapshot(
    area: Rect,
    chat: &ChatState,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
) -> CachedSidebarSnapshot {
    CachedSidebarSnapshot {
        key: sidebar_snapshot_key(area, chat, sidebar, tasks, thread_id),
        body: build_body_snapshot(area, chat, sidebar, tasks, thread_id),
    }
}

pub(crate) fn cached_snapshot_matches_render(
    snapshot: &CachedSidebarSnapshot,
    area: Rect,
    chat: &ChatState,
    sidebar: &SidebarState,
    tasks: &TaskState,
    thread_id: Option<&str>,
) -> bool {
    snapshot.key == sidebar_snapshot_key(area, chat, sidebar, tasks, thread_id)
}

pub(crate) fn selected_file_path(
    tasks: &TaskState,
    sidebar: &SidebarState,
    thread_id: Option<&str>,
) -> Option<String> {
    build_cached_snapshot(
        Rect::new(0, 0, 80, 0),
        &ChatState::new(),
        sidebar,
        tasks,
        thread_id,
    )
    .selected_file_path(sidebar.selected_item())
}
