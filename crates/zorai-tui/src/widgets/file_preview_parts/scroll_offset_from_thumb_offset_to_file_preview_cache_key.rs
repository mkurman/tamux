use super::build_cached_lines_to_scrollbar_layout::*;
use super::syntax_highlighting::*;
use crate::app::ChatFilePreviewTarget;
use crate::state::task::TaskState;
use crate::theme::ThemeTokens;
use crate::widgets::image_preview;
use crate::widgets::message::{render_markdown_pub, wrap_text};
use crate::widgets::tool_diff::render_unified_diff;
use ratatui::prelude::*;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use std::sync::{Arc, Mutex, OnceLock};
use unicode_width::UnicodeWidthChar;

pub(super) const SCROLLBAR_WIDTH: u16 = 1;
pub(super) const FILE_PREVIEW_HEADER_LINES: u16 = 5;
pub(super) const TERMINAL_IMAGE_HEADER_LINES: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FilePreviewScrollbarLayout {
    pub(crate) content: Rect,
    pub(crate) scrollbar: Rect,
    pub(crate) thumb: Rect,
    pub(crate) scroll: usize,
    pub(crate) max_scroll: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilePreviewHitTarget {
    ClosePreview,
}

pub(super) fn scroll_offset_from_thumb_offset(
    thumb_offset: u16,
    track_span: u16,
    max_scroll: usize,
) -> usize {
    if max_scroll == 0 || track_span == 0 {
        return 0;
    }

    (((thumb_offset as usize) * max_scroll) + (track_span as usize / 2)) / track_span as usize
}

pub(super) fn scrollbar_layout_from_metrics(
    area: Rect,
    total_lines: usize,
    scroll: usize,
) -> Option<FilePreviewScrollbarLayout> {
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

    Some(FilePreviewScrollbarLayout {
        content,
        scrollbar,
        thumb,
        scroll,
        max_scroll,
    })
}

pub(super) fn is_markdown_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown") || lower.ends_with(".mdx")
}

pub(super) fn push_wrapped(lines: &mut Vec<Line<'static>>, text: &str, style: Style, width: usize) {
    for line in wrap_text(text, width.max(1)) {
        lines.push(Line::from(Span::styled(line, style)));
    }
}

pub(super) fn push_preview_content(
    lines: &mut Vec<Line<'static>>,
    path: &str,
    content: &str,
    width: usize,
    theme: &ThemeTokens,
) {
    if is_markdown_path(path) {
        lines.extend(render_markdown_pub(content, width.max(1)));
    } else if is_code_like_path(path) {
        push_syntax_highlighted(lines, content, width, theme);
    } else {
        push_wrapped(lines, content, theme.fg_dim, width);
    }
}

pub(super) struct SelectionSnapshot {
    pub(super) header_lines: Arc<Vec<Line<'static>>>,
    pub(super) body_lines: Arc<Vec<Line<'static>>>,
    pub(super) scroll: usize,
    pub(super) header_area: Rect,
    pub(super) body_area: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FilePreviewThemeKey {
    pub(super) fg_dim: Style,
    pub(super) fg_active: Style,
    pub(super) accent_primary: Style,
    pub(super) accent_secondary: Style,
    pub(super) accent_success: Style,
    pub(super) accent_danger: Style,
}

impl From<&ThemeTokens> for FilePreviewThemeKey {
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
pub(super) struct FilePreviewCacheKey {
    pub(super) area: Rect,
    pub(super) path: String,
    pub(super) repo_root: Option<String>,
    pub(super) repo_relative_path: Option<String>,
    pub(super) task_state_id: usize,
    pub(super) preview_revision: u64,
    pub(super) image_preview_revision: u64,
    pub(super) terminal_graphics: bool,
    pub(super) theme: FilePreviewThemeKey,
}

#[derive(Clone)]
pub(super) struct CachedFilePreviewLines {
    pub(super) key: FilePreviewCacheKey,
    pub(super) header_lines: Arc<Vec<Line<'static>>>,
    pub(super) body_lines: Arc<Vec<Line<'static>>>,
    pub(super) body_area: Rect,
    pub(super) max_scroll: usize,
}

#[derive(Default)]
pub(super) struct FilePreviewRenderCache {
    pub(super) lines: Vec<CachedFilePreviewLines>,
}

pub(super) struct FilePreviewSnapshot {
    pub(super) header_lines: Arc<Vec<Line<'static>>>,
    pub(super) body_lines: Arc<Vec<Line<'static>>>,
    pub(super) scroll: usize,
    pub(super) body_area: Rect,
    pub(super) layout: Option<FilePreviewScrollbarLayout>,
    pub(super) max_scroll: usize,
}

pub(super) fn global_file_preview_cache() -> &'static Mutex<FilePreviewRenderCache> {
    static CACHE: OnceLock<Mutex<FilePreviewRenderCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(FilePreviewRenderCache::default()))
}

pub(super) fn lock_file_preview_cache() -> std::sync::MutexGuard<'static, FilePreviewRenderCache> {
    global_file_preview_cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
static BUILD_LINES_CALL_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
static BUILD_LINES_TRACKED_PATH: OnceLock<Mutex<Option<String>>> = OnceLock::new();

#[cfg(test)]
pub(super) fn tracked_build_lines_path_for_tests() -> &'static Mutex<Option<String>> {
    BUILD_LINES_TRACKED_PATH.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
pub(super) fn reset_build_lines_call_count_for_tests(path: &str) {
    BUILD_LINES_CALL_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
    *tracked_build_lines_path_for_tests()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(path.to_string());
    lock_file_preview_cache().lines.clear();
}

#[cfg(test)]
pub(super) fn build_lines_call_count_for_tests() -> usize {
    BUILD_LINES_CALL_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

pub(super) fn line_plain_text(line: &Line<'static>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

pub(super) fn line_display_width(line: &Line<'static>) -> usize {
    line_plain_text(line)
        .chars()
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(0))
        .sum()
}

pub(super) fn display_slice(text: &str, start_col: usize, end_col: usize) -> String {
    if start_col >= end_col {
        return String::new();
    }

    let mut result = String::new();
    let mut col = 0usize;
    for ch in text.chars() {
        let width = UnicodeWidthChar::width(ch).unwrap_or(0);
        let next = col + width;
        let overlaps = if width == 0 {
            col >= start_col && col < end_col
        } else {
            next > start_col && col < end_col
        };
        if overlaps {
            result.push(ch);
        }
        col = next;
        if col >= end_col {
            break;
        }
    }
    result
}

pub(super) fn highlight_line_range(
    line: &mut Line<'static>,
    start_col: usize,
    end_col: usize,
    highlight: Style,
) {
    if start_col >= end_col {
        return;
    }

    let original_spans = std::mem::take(&mut line.spans);
    let mut spans = Vec::new();
    let mut col = 0usize;

    for span in original_spans {
        let mut before = String::new();
        let mut selected = String::new();
        let mut after = String::new();

        for ch in span.content.chars() {
            let width = UnicodeWidthChar::width(ch).unwrap_or(0);
            let next = col + width;
            let overlaps = if width == 0 {
                col >= start_col && col < end_col
            } else {
                next > start_col && col < end_col
            };

            if overlaps {
                selected.push(ch);
            } else if col < start_col {
                before.push(ch);
            } else {
                after.push(ch);
            }

            col = next;
        }

        if !before.is_empty() {
            spans.push(Span::styled(before, span.style));
        }
        if !selected.is_empty() {
            spans.push(Span::styled(selected, span.style.patch(highlight)));
        }
        if !after.is_empty() {
            spans.push(Span::styled(after, span.style));
        }
    }

    line.spans = spans;
}

#[cfg(test)]
pub(super) fn build_lines(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
) -> Vec<Line<'static>> {
    let mut lines = build_header_lines(target, theme);
    lines.extend(build_body_lines(area, tasks, target, theme, scroll));
    lines
}

pub(super) fn build_header_lines(
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("[x]", theme.accent_danger),
            Span::raw(" "),
            Span::styled("Close preview", theme.fg_dim),
        ]),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Path: ", theme.fg_dim),
            Span::styled(target.path.clone(), theme.fg_active),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Preview",
            theme.accent_primary.add_modifier(Modifier::BOLD),
        )),
    ]
}

pub(super) fn build_body_lines(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    scroll: usize,
) -> Vec<Line<'static>> {
    #[cfg(test)]
    {
        let should_count = tracked_build_lines_path_for_tests()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_deref()
            == Some(target.path.as_str());
        if should_count {
            BUILD_LINES_CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    let width = area.width as usize;
    let mut lines = Vec::new();
    let image_preview_height = area.height.max(1) as usize;
    let use_terminal_graphics = uses_terminal_graphics(target, scroll);

    if let Some(repo_root) = target.repo_root.as_deref() {
        let diff_key = target
            .repo_relative_path
            .as_deref()
            .unwrap_or(target.path.as_str());
        if let Some(diff) = tasks.diff_for_path(repo_root, diff_key) {
            if diff.trim().is_empty() {
                push_wrapped(
                    &mut lines,
                    "No diff preview available for this file.",
                    theme.fg_dim,
                    width,
                );
            } else {
                lines.extend(render_unified_diff(diff, theme, width));
            }
            return lines;
        }
        if let Some(preview) = tasks.preview_for_path(&target.path) {
            if preview.is_text {
                push_preview_content(&mut lines, &target.path, &preview.content, width, theme);
            } else {
                push_wrapped(
                    &mut lines,
                    "Binary file preview is not available.",
                    theme.fg_dim,
                    width,
                );
            }
        } else {
            push_wrapped(&mut lines, "Loading diff...", theme.fg_dim, width);
        }
        return lines;
    }

    if let Some(preview) = tasks.preview_for_path(&target.path) {
        if preview.is_text {
            push_preview_content(&mut lines, &target.path, &preview.content, width, theme);
        } else if use_terminal_graphics {
            push_terminal_graphics_placeholder(&mut lines, image_preview_height, theme);
        } else if image_preview::is_previewable_image_path(&target.path) {
            lines.extend(image_preview::render_image_preview_lines(
                &target.path,
                width,
                image_preview_height,
                theme,
            ));
        } else {
            push_wrapped(
                &mut lines,
                "Binary file preview is not available.",
                theme.fg_dim,
                width,
            );
        }
    } else {
        if image_preview::is_previewable_image_path(&target.path) {
            if use_terminal_graphics {
                push_terminal_graphics_placeholder(&mut lines, image_preview_height, theme);
            } else {
                lines.extend(image_preview::render_image_preview_lines(
                    &target.path,
                    width,
                    image_preview_height,
                    theme,
                ));
            }
        } else {
            push_wrapped(&mut lines, "Loading preview...", theme.fg_dim, width);
        }
    }

    lines
}

pub(super) fn file_preview_body_area(area: Rect) -> Rect {
    let header_height = FILE_PREVIEW_HEADER_LINES.min(area.height);
    Rect::new(
        area.x,
        area.y.saturating_add(header_height),
        area.width,
        area.height.saturating_sub(header_height),
    )
}

pub(super) fn file_preview_cache_key(
    area: Rect,
    tasks: &TaskState,
    target: &ChatFilePreviewTarget,
    theme: &ThemeTokens,
    terminal_graphics: bool,
) -> FilePreviewCacheKey {
    let image_preview_revision = if terminal_graphics {
        0
    } else {
        image_preview::resolve_local_image_path(&target.path)
            .as_deref()
            .filter(|path| image_preview::is_previewable_image_path(path))
            .map(|_| image_preview::preview_cache_revision())
            .unwrap_or(0)
    };

    FilePreviewCacheKey {
        area,
        path: target.path.clone(),
        repo_root: target.repo_root.clone(),
        repo_relative_path: target.repo_relative_path.clone(),
        task_state_id: tasks as *const TaskState as usize,
        preview_revision: tasks.preview_revision(),
        image_preview_revision,
        terminal_graphics,
        theme: FilePreviewThemeKey::from(theme),
    }
}
