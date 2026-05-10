use super::selection::*;
use super::*;
use crate::state::sidebar::SidebarTab;
use crate::state::task::{TaskState, TodoStatus, WorkContextEntryKind};
use crate::terminal_graphics::{active_protocol, TerminalImageProtocol};
use crate::theme::ThemeTokens;
use crate::widgets::chat::SelectionPoint;
use crate::widgets::image_preview;
use crate::widgets::message::{render_markdown_pub, wrap_text};
use crate::widgets::tool_diff::render_unified_diff;
use ratatui::prelude::*;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

pub(crate) const SCROLLBAR_WIDTH: u16 = 1;
pub(crate) const TERMINAL_IMAGE_HEADER_LINES: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkContextScrollbarLayout {
    pub(crate) content: Rect,
    pub(crate) scrollbar: Rect,
    pub(crate) thumb: Rect,
    pub(crate) scroll: usize,
    pub(crate) max_scroll: usize,
}

#[derive(Clone)]
pub(crate) struct RenderedWorkLine {
    pub(crate) line: Line<'static>,
    pub(crate) close_preview: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkContextHitTarget {
    ClosePreview,
}

pub(crate) struct SelectionSnapshot {
    pub(crate) all_lines: Vec<RenderedWorkLine>,
    pub(crate) scroll: usize,
    pub(crate) area: Rect,
    pub(crate) header_area: Option<Rect>,
    pub(crate) body_area: Option<Rect>,
    pub(crate) header_len: usize,
}

pub(crate) struct StickyFilesSnapshot {
    pub(crate) header_lines: Vec<RenderedWorkLine>,
    pub(crate) body_lines: Vec<RenderedWorkLine>,
    pub(crate) scroll: usize,
    pub(crate) header_area: Rect,
    pub(crate) body_area: Rect,
    pub(crate) layout: Option<WorkContextScrollbarLayout>,
    pub(crate) max_scroll: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorkContextThemeKey {
    pub(crate) fg_dim: Style,
    pub(crate) fg_active: Style,
    pub(crate) accent_primary: Style,
    pub(crate) accent_secondary: Style,
    pub(crate) accent_success: Style,
    pub(crate) accent_danger: Style,
}

impl From<&ThemeTokens> for WorkContextThemeKey {
    fn from(theme: &ThemeTokens) -> Self {
        Self {
            fg_dim: theme.fg_dim,
            fg_active: theme.fg_active,
            accent_primary: theme.accent_primary,
            accent_secondary: theme.accent_secondary,
            accent_success: theme.accent_success,
            accent_danger: theme.accent_danger,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FileBodyCacheKey {
    pub(crate) area: Rect,
    pub(crate) path: String,
    pub(crate) repo_root: Option<String>,
    pub(crate) task_state_id: usize,
    pub(crate) preview_revision: u64,
    pub(crate) image_preview_revision: u64,
    pub(crate) terminal_image: bool,
    pub(crate) theme: WorkContextThemeKey,
}

#[derive(Clone)]
pub(crate) struct CachedFileBodyLines {
    pub(crate) key: FileBodyCacheKey,
    pub(crate) lines: Vec<RenderedWorkLine>,
}

#[derive(Default)]
pub(crate) struct WorkContextRenderCache {
    pub(crate) file_body_lines: Vec<CachedFileBodyLines>,
}

pub(crate) const WORK_CONTEXT_CACHE_CAPACITY: usize = 16;

pub(crate) fn global_work_context_cache() -> &'static std::sync::Mutex<WorkContextRenderCache> {
    static CACHE: std::sync::OnceLock<std::sync::Mutex<WorkContextRenderCache>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(WorkContextRenderCache::default()))
}

pub(crate) fn lock_work_context_cache() -> std::sync::MutexGuard<'static, WorkContextRenderCache> {
    global_work_context_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
static FILE_BODY_LINES_CALL_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
static FILE_BODY_LINES_TRACKED_PATH: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
    std::sync::OnceLock::new();

#[cfg(test)]
pub(crate) fn tracked_file_body_lines_path_for_tests() -> &'static std::sync::Mutex<Option<String>>
{
    FILE_BODY_LINES_TRACKED_PATH.get_or_init(|| std::sync::Mutex::new(None))
}

#[cfg(test)]
pub(crate) fn reset_file_body_lines_call_count_for_tests(path: &str) {
    FILE_BODY_LINES_CALL_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
    *tracked_file_body_lines_path_for_tests()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(path.to_string());
    lock_work_context_cache().file_body_lines.clear();
}

#[cfg(test)]
pub(crate) fn file_body_lines_call_count_for_tests() -> usize {
    FILE_BODY_LINES_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

pub(crate) fn scrollbar_layout_from_metrics(
    area: Rect,
    total_lines: usize,
    scroll: usize,
) -> Option<WorkContextScrollbarLayout> {
    if area.width <= SCROLLBAR_WIDTH || area.height == 0 || total_lines <= area.height as usize {
        return None;
    }

    let viewport = area.height as usize;
    let max_scroll = total_lines.saturating_sub(viewport);
    let scroll = scroll.min(max_scroll);
    let content = Rect::new(
        area.x,
        area.y,
        area.width.saturating_sub(SCROLLBAR_WIDTH),
        area.height,
    );
    let scrollbar = Rect::new(
        area.x
            .saturating_add(area.width)
            .saturating_sub(SCROLLBAR_WIDTH),
        area.y,
        SCROLLBAR_WIDTH,
        area.height,
    );
    let thumb_height = ((viewport * viewport) / total_lines).max(1).min(viewport) as u16;
    let track_span = scrollbar.height.saturating_sub(thumb_height);
    let thumb_offset = if max_scroll == 0 || track_span == 0 {
        0
    } else {
        ((scroll * track_span as usize) + (max_scroll / 2)) / max_scroll
    } as u16;
    let thumb = Rect::new(
        scrollbar.x,
        scrollbar.y.saturating_add(thumb_offset),
        scrollbar.width,
        thumb_height,
    );

    Some(WorkContextScrollbarLayout {
        content,
        scrollbar,
        thumb,
        scroll,
        max_scroll,
    })
}

pub(crate) fn push_wrapped(
    lines: &mut Vec<Line<'static>>,
    text: &str,
    style: Style,
    width: usize,
    indent: usize,
) {
    let available = width.saturating_sub(indent).max(1);
    for line in wrap_text(text, available) {
        lines.push(Line::from(vec![
            Span::raw(" ".repeat(indent)),
            Span::styled(line, style),
        ]));
    }
}

pub(crate) fn section(lines: &mut Vec<Line<'static>>, title: &str, theme: &ThemeTokens) {
    if !lines.is_empty() {
        lines.push(Line::raw(""));
    }
    lines.push(Line::from(Span::styled(
        title.to_string(),
        theme.accent_primary.add_modifier(Modifier::BOLD),
    )));
}

pub(crate) fn is_markdown_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".mdx")
}

pub(crate) fn push_preview_content(
    lines: &mut Vec<Line<'static>>,
    path: &str,
    content: &str,
    width: usize,
    theme: &ThemeTokens,
) {
    if is_markdown_path(path) {
        lines.extend(render_markdown_pub(content, width.max(1)));
    } else {
        push_wrapped(lines, content, theme.fg_dim, width, 0);
    }
}

pub(crate) fn file_header_lines(
    entry: &crate::state::task::WorkContextEntry,
    theme: &ThemeTokens,
) -> Vec<RenderedWorkLine> {
    let mut lines = Vec::new();
    section(&mut lines, "File", theme);
    lines.push(Line::from(vec![
        Span::styled("[x]", theme.accent_danger),
        Span::raw(" "),
        Span::styled("Close preview", theme.fg_dim),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Path: ", theme.fg_dim),
        Span::styled(entry.path.clone(), theme.fg_active),
    ]));
    let kind = match entry.kind {
        Some(WorkContextEntryKind::RepoChange) => "repo change",
        Some(WorkContextEntryKind::Artifact) => "artifact",
        Some(WorkContextEntryKind::GeneratedSkill) => "generated skill",
        None => "file",
    };
    lines.push(Line::from(vec![
        Span::styled("Type: ", theme.fg_dim),
        Span::styled(kind, theme.fg_active),
    ]));
    if let Some(change_kind) = &entry.change_kind {
        lines.push(Line::from(vec![
            Span::styled("Change: ", theme.fg_dim),
            Span::styled(change_kind.clone(), theme.fg_active),
        ]));
    }
    if let Some(previous_path) = &entry.previous_path {
        lines.push(Line::from(vec![
            Span::styled("From: ", theme.fg_dim),
            Span::styled(previous_path.clone(), theme.fg_active),
        ]));
    }
    section(&mut lines, "Preview", theme);

    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| RenderedWorkLine {
            line,
            close_preview: index == 1,
        })
        .collect()
}

pub(crate) fn file_body_cache_key(
    area: Rect,
    tasks: &TaskState,
    entry: &crate::state::task::WorkContextEntry,
    active_tab: SidebarTab,
    theme: &ThemeTokens,
    scroll: usize,
) -> FileBodyCacheKey {
    let terminal_image =
        uses_terminal_graphics(&entry.path, entry.repo_root.as_deref(), active_tab, scroll);
    let image_preview_revision = if terminal_image {
        0
    } else {
        image_preview::resolve_local_image_path(&entry.path)
            .as_deref()
            .filter(|path| image_preview::is_previewable_image_path(path))
            .map(|_| image_preview::preview_cache_revision())
            .unwrap_or(0)
    };

    FileBodyCacheKey {
        area,
        path: entry.path.clone(),
        repo_root: entry.repo_root.clone(),
        task_state_id: tasks as *const TaskState as usize,
        preview_revision: tasks.preview_revision(),
        image_preview_revision,
        terminal_image,
        theme: WorkContextThemeKey::from(theme),
    }
}

pub(crate) fn file_body_lines(
    area: Rect,
    tasks: &TaskState,
    entry: &crate::state::task::WorkContextEntry,
    active_tab: SidebarTab,
    theme: &ThemeTokens,
    scroll: usize,
) -> Vec<RenderedWorkLine> {
    let key = file_body_cache_key(area, tasks, entry, active_tab, theme, scroll);
    {
        let cache = lock_work_context_cache();
        if let Some(cached) = cache
            .file_body_lines
            .iter()
            .find(|cached| cached.key == key)
            .cloned()
        {
            return cached.lines;
        }
    }

    let lines = build_file_body_lines(area, tasks, entry, active_tab, theme, scroll);
    let mut cache = lock_work_context_cache();
    if let Some(cached) = cache
        .file_body_lines
        .iter()
        .find(|cached| cached.key == key)
        .cloned()
    {
        return cached.lines;
    }
    cache.file_body_lines.push(CachedFileBodyLines {
        key,
        lines: lines.clone(),
    });
    if cache.file_body_lines.len() > WORK_CONTEXT_CACHE_CAPACITY {
        cache.file_body_lines.remove(0);
    }
    lines
}

pub(crate) fn build_file_body_lines(
    area: Rect,
    tasks: &TaskState,
    entry: &crate::state::task::WorkContextEntry,
    active_tab: SidebarTab,
    theme: &ThemeTokens,
    scroll: usize,
) -> Vec<RenderedWorkLine> {
    #[cfg(test)]
    {
        let should_count = tracked_file_body_lines_path_for_tests()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_deref()
            == Some(entry.path.as_str());
        if should_count {
            FILE_BODY_LINES_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let width = area.width as usize;
    let mut lines = Vec::new();
    let image_preview_height = area.height.max(1) as usize;
    let use_terminal_image =
        uses_terminal_graphics(&entry.path, entry.repo_root.as_deref(), active_tab, scroll);

    if let Some(repo_root) = entry.repo_root.as_deref() {
        if let Some(diff) = tasks.diff_for_path(repo_root, &entry.path) {
            if diff.trim().is_empty() {
                push_wrapped(
                    &mut lines,
                    "No diff preview available for this file.",
                    theme.fg_dim,
                    width,
                    0,
                );
            } else {
                lines.extend(render_unified_diff(diff, theme, width));
            }
        } else {
            push_wrapped(&mut lines, "Loading diff...", theme.fg_dim, width, 0);
        }
    } else if let Some(preview) = tasks.preview_for_path(&entry.path) {
        if preview.is_text {
            push_preview_content(&mut lines, &entry.path, &preview.content, width, theme);
        } else if use_terminal_image {
            push_terminal_graphics_placeholder(&mut lines, image_preview_height, theme);
        } else if image_preview::is_previewable_image_path(&entry.path) {
            lines.extend(image_preview::render_image_preview_lines(
                &entry.path,
                width,
                14,
                theme,
            ));
        } else {
            push_wrapped(
                &mut lines,
                "Binary file preview is not available.",
                theme.fg_dim,
                width,
                0,
            );
        }
    } else if image_preview::is_previewable_image_path(&entry.path) {
        if use_terminal_image {
            push_terminal_graphics_placeholder(&mut lines, image_preview_height, theme);
        } else {
            lines.extend(image_preview::render_image_preview_lines(
                &entry.path,
                width,
                14,
                theme,
            ));
        }
    } else {
        push_wrapped(&mut lines, "Loading preview...", theme.fg_dim, width, 0);
    }

    lines
        .into_iter()
        .map(|line| RenderedWorkLine {
            line,
            close_preview: false,
        })
        .collect()
}

pub(crate) fn sticky_files_snapshot(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<StickyFilesSnapshot> {
    let thread_id = thread_id?;
    let context = tasks.work_context_for_thread(thread_id)?;
    let entry = context.entries.get(selected_index)?;
    let header_lines = file_header_lines(entry, theme);
    let header_height = (header_lines.len() as u16).min(area.height);
    let header_area = Rect::new(area.x, area.y, area.width, header_height);
    let body_track_area = Rect::new(
        area.x,
        area.y.saturating_add(header_height),
        area.width,
        area.height.saturating_sub(header_height),
    );
    let body_lines = file_body_lines(
        body_track_area,
        tasks,
        entry,
        SidebarTab::Files,
        theme,
        scroll,
    );
    let first_layout = scrollbar_layout_from_metrics(body_track_area, body_lines.len(), scroll);
    let mut body_area = first_layout
        .map(|layout| layout.content)
        .unwrap_or(body_track_area);
    let body_lines = if body_area.width != body_track_area.width {
        file_body_lines(body_area, tasks, entry, SidebarTab::Files, theme, scroll)
    } else {
        body_lines
    };
    let layout = scrollbar_layout_from_metrics(body_track_area, body_lines.len(), scroll);
    if let Some(layout) = layout {
        body_area = layout.content;
    }
    let resolved_scroll = layout
        .map(|layout| layout.scroll)
        .unwrap_or_else(|| scroll.min(body_lines.len().saturating_sub(body_area.height as usize)));
    let max_scroll = body_lines.len().saturating_sub(body_area.height as usize);

    Some(StickyFilesSnapshot {
        header_lines,
        body_lines,
        scroll: resolved_scroll,
        header_area,
        body_area,
        layout,
        max_scroll,
    })
}

pub(crate) fn uses_terminal_graphics(
    entry_path: &str,
    repo_root: Option<&str>,
    active_tab: SidebarTab,
    scroll: usize,
) -> bool {
    active_tab == SidebarTab::Files
        && scroll == 0
        && repo_root.is_none()
        && active_protocol() != TerminalImageProtocol::None
        && image_preview::resolve_local_image_path(entry_path)
            .as_deref()
            .is_some_and(image_preview::is_previewable_image_path)
}

pub(crate) fn push_terminal_graphics_placeholder(
    lines: &mut Vec<Line<'static>>,
    image_preview_height: usize,
    theme: &ThemeTokens,
) {
    lines.push(Line::from(vec![
        Span::styled("Image: ", theme.fg_dim),
        Span::styled("high-quality terminal preview", theme.fg_active),
    ]));
    for _ in 1..image_preview_height {
        lines.push(Line::raw(""));
    }
}

pub(crate) fn build_lines(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
) -> Vec<RenderedWorkLine> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let width = area.width as usize;
    let mut lines = Vec::new();

    match active_tab {
        SidebarTab::Files => {
            if let Some(snapshot) =
                sticky_files_snapshot(area, tasks, thread_id, selected_index, theme, scroll)
            {
                return snapshot
                    .header_lines
                    .into_iter()
                    .chain(snapshot.body_lines)
                    .collect();
            }

            section(&mut lines, "Files", theme);
            let Some(thread_id) = thread_id else {
                push_wrapped(&mut lines, "No thread selected.", theme.fg_dim, width, 0);
                return lines
                    .into_iter()
                    .map(|line| RenderedWorkLine {
                        line,
                        close_preview: false,
                    })
                    .collect();
            };
            lines.push(Line::from(vec![
                Span::styled("[x]", theme.accent_danger),
                Span::raw(" "),
                Span::styled("Close preview", theme.fg_dim),
            ]));
            let Some(context) = tasks.work_context_for_thread(thread_id) else {
                push_wrapped(
                    &mut lines,
                    "No files recorded for this thread.",
                    theme.fg_dim,
                    width,
                    0,
                );
                return lines
                    .into_iter()
                    .map(|line| RenderedWorkLine {
                        line,
                        close_preview: false,
                    })
                    .collect();
            };
            let Some(entry) = context.entries.get(selected_index) else {
                push_wrapped(
                    &mut lines,
                    "Select a file from the sidebar.",
                    theme.fg_dim,
                    width,
                    0,
                );
                return lines
                    .into_iter()
                    .map(|line| RenderedWorkLine {
                        line,
                        close_preview: false,
                    })
                    .collect();
            };

            lines.push(Line::from(vec![
                Span::styled("Path: ", theme.fg_dim),
                Span::styled(entry.path.clone(), theme.fg_active),
            ]));
            let kind = match entry.kind {
                Some(WorkContextEntryKind::RepoChange) => "repo change",
                Some(WorkContextEntryKind::Artifact) => "artifact",
                Some(WorkContextEntryKind::GeneratedSkill) => "generated skill",
                None => "file",
            };
            lines.push(Line::from(vec![
                Span::styled("Type: ", theme.fg_dim),
                Span::styled(kind, theme.fg_active),
            ]));
            if let Some(change_kind) = &entry.change_kind {
                lines.push(Line::from(vec![
                    Span::styled("Change: ", theme.fg_dim),
                    Span::styled(change_kind.clone(), theme.fg_active),
                ]));
            }
            if let Some(previous_path) = &entry.previous_path {
                lines.push(Line::from(vec![
                    Span::styled("From: ", theme.fg_dim),
                    Span::styled(previous_path.clone(), theme.fg_active),
                ]));
            }

            section(&mut lines, "Preview", theme);
            let image_preview_height =
                area.height.saturating_sub(lines.len() as u16).max(1) as usize;
            let use_terminal_image =
                uses_terminal_graphics(&entry.path, entry.repo_root.as_deref(), active_tab, scroll);
            if let Some(repo_root) = entry.repo_root.as_deref() {
                if let Some(diff) = tasks.diff_for_path(repo_root, &entry.path) {
                    if diff.trim().is_empty() {
                        push_wrapped(
                            &mut lines,
                            "No diff preview available for this file.",
                            theme.fg_dim,
                            width,
                            0,
                        );
                    } else {
                        push_wrapped(&mut lines, diff, theme.fg_dim, width, 0);
                    }
                } else {
                    push_wrapped(&mut lines, "Loading diff...", theme.fg_dim, width, 0);
                }
            } else if let Some(preview) = tasks.preview_for_path(&entry.path) {
                if preview.is_text {
                    push_preview_content(&mut lines, &entry.path, &preview.content, width, theme);
                } else if use_terminal_image {
                    push_terminal_graphics_placeholder(&mut lines, image_preview_height, theme);
                } else if image_preview::is_previewable_image_path(&entry.path) {
                    lines.extend(image_preview::render_image_preview_lines(
                        &entry.path,
                        width,
                        14,
                        theme,
                    ));
                } else {
                    push_wrapped(
                        &mut lines,
                        "Binary file preview is not available.",
                        theme.fg_dim,
                        width,
                        0,
                    );
                }
            } else {
                if image_preview::is_previewable_image_path(&entry.path) {
                    if use_terminal_image {
                        push_terminal_graphics_placeholder(&mut lines, image_preview_height, theme);
                    } else {
                        lines.extend(image_preview::render_image_preview_lines(
                            &entry.path,
                            width,
                            14,
                            theme,
                        ));
                    }
                } else {
                    push_wrapped(&mut lines, "Loading preview...", theme.fg_dim, width, 0);
                }
            }
        }
        SidebarTab::Todos => {
            section(&mut lines, "Todos", theme);
            let Some(thread_id) = thread_id else {
                push_wrapped(&mut lines, "No thread selected.", theme.fg_dim, width, 0);
                return lines
                    .into_iter()
                    .map(|line| RenderedWorkLine {
                        line,
                        close_preview: false,
                    })
                    .collect();
            };
            lines.push(Line::from(vec![
                Span::styled("[x]", theme.accent_danger),
                Span::raw(" "),
                Span::styled("Close preview", theme.fg_dim),
            ]));
            let todos = tasks.todos_for_thread(thread_id);
            if todos.is_empty() {
                push_wrapped(
                    &mut lines,
                    "No todos recorded for this thread.",
                    theme.fg_dim,
                    width,
                    0,
                );
                return lines
                    .into_iter()
                    .map(|line| RenderedWorkLine {
                        line,
                        close_preview: false,
                    })
                    .collect();
            }

            let index = selected_index.min(todos.len().saturating_sub(1));
            let todo = &todos[index];
            let status = match todo.status {
                Some(TodoStatus::Completed) => "completed",
                Some(TodoStatus::InProgress) => "in progress",
                Some(TodoStatus::Blocked) => "blocked",
                _ => "pending",
            };
            lines.push(Line::from(vec![
                Span::styled("Status: ", theme.fg_dim),
                Span::styled(status, theme.fg_active),
            ]));
            if let Some(step_index) = todo.step_index {
                lines.push(Line::from(vec![
                    Span::styled("Step: ", theme.fg_dim),
                    Span::styled((step_index + 1).to_string(), theme.fg_active),
                ]));
            }
            section(&mut lines, "Selected Todo", theme);
            push_wrapped(&mut lines, &todo.content, theme.fg_active, width, 0);

            section(&mut lines, "All Todos", theme);
            for (idx, item) in todos.iter().enumerate() {
                let marker = if idx == index { ">" } else { " " };
                let chip = match item.status {
                    Some(TodoStatus::Completed) => "[x]",
                    Some(TodoStatus::InProgress) => "[~]",
                    Some(TodoStatus::Blocked) => "[!]",
                    _ => "[ ]",
                };
                push_wrapped(
                    &mut lines,
                    &format!("{marker} {chip} {}", item.content),
                    if idx == index {
                        theme.fg_active
                    } else {
                        theme.fg_dim
                    },
                    width,
                    0,
                );
            }
        }
        SidebarTab::Spawned => {
            section(&mut lines, "Spawned", theme);
            lines.push(Line::from(vec![
                Span::styled("[x]", theme.accent_danger),
                Span::raw(" "),
                Span::styled("Close preview", theme.fg_dim),
            ]));
            push_wrapped(
                &mut lines,
                "Spawned agent navigation stays in the conversation view.",
                theme.fg_dim,
                width,
                0,
            );
        }
        SidebarTab::Pinned => {
            section(&mut lines, "Pinned", theme);
            lines.push(Line::from(vec![
                Span::styled("[x]", theme.accent_danger),
                Span::raw(" "),
                Span::styled("Close preview", theme.fg_dim),
            ]));
            push_wrapped(
                &mut lines,
                "Pinned messages jump back into the conversation view.",
                theme.fg_dim,
                width,
                0,
            );
        }
    }

    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| RenderedWorkLine {
            line,
            close_preview: index == 1,
        })
        .collect()
}

pub(crate) fn selection_snapshot(
    area: Rect,
    tasks: &TaskState,
    thread_id: Option<&str>,
    active_tab: SidebarTab,
    selected_index: usize,
    theme: &ThemeTokens,
    scroll: usize,
) -> Option<SelectionSnapshot> {
    if active_tab == SidebarTab::Files {
        let snapshot =
            sticky_files_snapshot(area, tasks, thread_id, selected_index, theme, scroll)?;
        let header_len = snapshot.header_lines.len();
        let all_lines = snapshot
            .header_lines
            .into_iter()
            .chain(snapshot.body_lines)
            .collect();
        return Some(SelectionSnapshot {
            all_lines,
            scroll: snapshot.scroll,
            area,
            header_area: Some(snapshot.header_area),
            body_area: Some(snapshot.body_area),
            header_len,
        });
    }

    let layout = scrollbar_layout(
        area,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        scroll,
    );
    let content = layout.map(|layout| layout.content).unwrap_or(area);
    let resolved_scroll = layout.map(|layout| layout.scroll).unwrap_or(scroll);
    let all_lines = build_lines(
        content,
        tasks,
        thread_id,
        active_tab,
        selected_index,
        theme,
        resolved_scroll,
    );
    if all_lines.is_empty() || content.width == 0 || content.height == 0 {
        return None;
    }
    Some(SelectionSnapshot {
        all_lines,
        scroll: resolved_scroll,
        area: content,
        header_area: None,
        body_area: None,
        header_len: 0,
    })
}

pub(crate) fn selection_point_from_snapshot(
    snapshot: &SelectionSnapshot,
    mouse: Position,
) -> Option<SelectionPoint> {
    if let (Some(header_area), Some(body_area)) = (snapshot.header_area, snapshot.body_area) {
        if header_area.contains(mouse) {
            let clamped_x = mouse.x.clamp(
                header_area.x,
                header_area
                    .x
                    .saturating_add(header_area.width)
                    .saturating_sub(1),
            );
            let row = mouse
                .y
                .saturating_sub(header_area.y)
                .min(header_area.height.saturating_sub(1)) as usize;
            let col = clamped_x.saturating_sub(header_area.x) as usize;
            let width = line_display_width(&snapshot.all_lines.get(row)?.line);
            return Some(SelectionPoint {
                row,
                col: col.min(width),
            });
        }

        if !body_area.contains(mouse) || body_area.width == 0 || body_area.height == 0 {
            return None;
        }
        let clamped_x = mouse.x.clamp(
            body_area.x,
            body_area
                .x
                .saturating_add(body_area.width)
                .saturating_sub(1),
        );
        let clamped_y = mouse.y.clamp(
            body_area.y,
            body_area
                .y
                .saturating_add(body_area.height)
                .saturating_sub(1),
        );
        let row = snapshot
            .header_len
            .saturating_add(snapshot.scroll)
            .saturating_add(clamped_y.saturating_sub(body_area.y) as usize)
            .min(snapshot.all_lines.len().saturating_sub(1));
        let col = clamped_x.saturating_sub(body_area.x) as usize;
        let width = line_display_width(&snapshot.all_lines.get(row)?.line);
        return Some(SelectionPoint {
            row,
            col: col.min(width),
        });
    }

    let area = snapshot.area;
    let clamped_x = mouse
        .x
        .clamp(area.x, area.x.saturating_add(area.width).saturating_sub(1));
    let clamped_y = mouse
        .y
        .clamp(area.y, area.y.saturating_add(area.height).saturating_sub(1));
    let row = snapshot
        .scroll
        .saturating_add(clamped_y.saturating_sub(area.y) as usize)
        .min(snapshot.all_lines.len().saturating_sub(1));
    let col = clamped_x.saturating_sub(area.x) as usize;
    let width = line_display_width(&snapshot.all_lines.get(row)?.line);
    Some(SelectionPoint {
        row,
        col: col.min(width),
    })
}
