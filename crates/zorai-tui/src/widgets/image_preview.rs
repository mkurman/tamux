use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
#[cfg(not(test))]
use std::thread;

use image::imageops::FilterType;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::theme::ThemeTokens;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff"];

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PreviewCacheKey {
    path: String,
    width: usize,
    max_height_lines: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PixelPair {
    top: [u8; 3],
    bottom: [u8; 3],
}

#[derive(Clone, Debug)]
struct CachedImagePreview {
    label: String,
    source_width: u32,
    source_height: u32,
    rows: Vec<Vec<PixelPair>>,
}

#[derive(Clone, Debug)]
enum PreviewCacheEntry {
    Pending,
    Ready(Arc<CachedImagePreview>),
    Failed(String),
}

#[derive(Clone, Debug)]
struct PreviewJob {
    key: PreviewCacheKey,
}

#[derive(Default)]
struct PreviewStore {
    entries: HashMap<PreviewCacheKey, PreviewCacheEntry>,
    jobs: VecDeque<PreviewJob>,
    revision: u64,
}

impl PreviewStore {
    fn lookup_or_enqueue(&mut self, job: PreviewJob) -> (PreviewCacheEntry, bool) {
        if let Some(entry) = self.entries.get(&job.key) {
            return (entry.clone(), false);
        }

        self.entries
            .insert(job.key.clone(), PreviewCacheEntry::Pending);
        self.jobs.push_back(job);
        self.revision = self.revision.wrapping_add(1);
        (PreviewCacheEntry::Pending, true)
    }

    fn pop_job(&mut self) -> Option<PreviewJob> {
        self.jobs.pop_front()
    }

    fn finish_job(&mut self, key: &PreviewCacheKey, result: Result<CachedImagePreview, String>) {
        let entry = match result {
            Ok(preview) => PreviewCacheEntry::Ready(Arc::new(preview)),
            Err(message) => PreviewCacheEntry::Failed(message),
        };
        self.entries.insert(key.clone(), entry);
        self.revision = self.revision.wrapping_add(1);
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    #[cfg(test)]
    fn queue_len(&self) -> usize {
        self.jobs.len()
    }
}

struct PreviewRuntimeInner {
    store: Mutex<PreviewStore>,
    condvar: Condvar,
}

#[derive(Clone)]
struct PreviewRuntime {
    inner: Arc<PreviewRuntimeInner>,
}

impl PreviewRuntime {
    fn new() -> Self {
        Self {
            inner: Arc::new(PreviewRuntimeInner {
                store: Mutex::new(PreviewStore::default()),
                condvar: Condvar::new(),
            }),
        }
    }

    #[cfg(not(test))]
    fn spawn_worker(&self) {
        let inner = self.inner.clone();
        let _ = thread::Builder::new()
            .name("zorai-image-preview".to_string())
            .spawn(move || loop {
                let job = {
                    let mut store = lock_store(&inner.store);
                    loop {
                        if let Some(job) = store.pop_job() {
                            break job;
                        }
                        store = inner
                            .condvar
                            .wait(store)
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                    }
                };

                let result =
                    build_cached_preview(&job.key.path, job.key.width, job.key.max_height_lines);
                let mut store = lock_store(&inner.store);
                store.finish_job(&job.key, result);
            });
    }

    fn lookup_or_enqueue(&self, job: PreviewJob) -> PreviewCacheEntry {
        let (entry, queued) = {
            let mut store = lock_store(&self.inner.store);
            store.lookup_or_enqueue(job)
        };
        if queued {
            self.inner.condvar.notify_one();
        }
        entry
    }

    #[cfg(test)]
    fn process_next_job_for_tests(&self) -> bool {
        let Some(job) = ({
            let mut store = lock_store(&self.inner.store);
            store.pop_job()
        }) else {
            return false;
        };

        let result = build_cached_preview(&job.key.path, job.key.width, job.key.max_height_lines);
        let mut store = lock_store(&self.inner.store);
        store.finish_job(&job.key, result);
        true
    }

    #[cfg(test)]
    fn queue_len_for_tests(&self) -> usize {
        lock_store(&self.inner.store).queue_len()
    }
}

fn lock_store(mutex: &Mutex<PreviewStore>) -> std::sync::MutexGuard<'_, PreviewStore> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn preview_runtime() -> &'static PreviewRuntime {
    static RUNTIME: OnceLock<PreviewRuntime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        let runtime = PreviewRuntime::new();
        #[cfg(not(test))]
        runtime.spawn_worker();
        runtime
    })
}

fn image_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
}

pub(crate) fn is_previewable_image_path(path: &str) -> bool {
    image_extension(path)
        .as_deref()
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

pub(crate) fn resolve_local_image_path(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(path) = trimmed.strip_prefix("file://") {
        return Some(path.to_string());
    }
    Path::new(trimmed)
        .is_absolute()
        .then(|| trimmed.to_string())
}

fn image_label(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn resize_dimensions(
    source_width: u32,
    source_height: u32,
    max_width_chars: usize,
    max_height_lines: usize,
) -> (u32, u32) {
    let max_width_chars = max_width_chars.max(1) as f32;
    let max_height_pixels = max_height_lines.max(1).saturating_mul(2) as f32;
    let aspect = source_height as f32 / source_width.max(1) as f32;

    let mut target_width = max_width_chars;
    let mut target_height = target_width * aspect;
    if target_height > max_height_pixels {
        target_height = max_height_pixels;
        target_width = (target_height / aspect).max(1.0);
    }

    (
        target_width.round().max(1.0) as u32,
        target_height.round().max(1.0) as u32,
    )
}

fn fallback_lines(path: &str, theme: &ThemeTokens, reason: &str) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Image: ", theme.fg_dim),
            Span::styled(image_label(path), theme.fg_active),
        ]),
        Line::from(Span::styled(reason.to_string(), theme.fg_dim)),
    ]
}

fn build_cached_preview(
    path: &str,
    width: usize,
    max_height_lines: usize,
) -> Result<CachedImagePreview, String> {
    let reader =
        image::ImageReader::open(path).map_err(|_| "Failed to open image preview.".to_string())?;
    let decoded = reader
        .decode()
        .map_err(|_| "Failed to decode image preview.".to_string())?;
    let rgba = decoded.to_rgba8();
    let (target_width, target_height) = resize_dimensions(
        rgba.width(),
        rgba.height(),
        width.saturating_sub(2).max(1),
        max_height_lines.max(1),
    );
    let resized = image::imageops::resize(&rgba, target_width, target_height, FilterType::Triangle);

    let mut rows = Vec::with_capacity(resized.height().div_ceil(2) as usize);
    for y in (0..resized.height()).step_by(2) {
        let mut row = Vec::with_capacity(resized.width() as usize);
        for x in 0..resized.width() {
            let top = resized.get_pixel(x, y).0;
            let bottom = if y + 1 < resized.height() {
                resized.get_pixel(x, y + 1).0
            } else {
                [0, 0, 0, 255]
            };
            row.push(PixelPair {
                top: [top[0], top[1], top[2]],
                bottom: [bottom[0], bottom[1], bottom[2]],
            });
        }
        rows.push(row);
    }

    Ok(CachedImagePreview {
        label: image_label(path),
        source_width: rgba.width(),
        source_height: rgba.height(),
        rows,
    })
}

fn render_cached_preview_lines(
    preview: &CachedImagePreview,
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(vec![
        Span::styled("Image: ", theme.fg_dim),
        Span::styled(
            format!(
                "{} ({}x{})",
                preview.label, preview.source_width, preview.source_height
            ),
            theme.fg_active,
        ),
    ])];

    for row in &preview.rows {
        let spans = row
            .iter()
            .map(|pair| {
                Span::styled(
                    "▀",
                    Style::default()
                        .fg(Color::Rgb(pair.top[0], pair.top[1], pair.top[2]))
                        .bg(Color::Rgb(pair.bottom[0], pair.bottom[1], pair.bottom[2])),
                )
            })
            .collect::<Vec<_>>();
        lines.push(Line::from(spans));
    }

    lines
}

fn render_image_preview_lines_with_runtime(
    runtime: &PreviewRuntime,
    raw_path: &str,
    width: usize,
    max_height_lines: usize,
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    let Some(path) = resolve_local_image_path(raw_path) else {
        return fallback_lines(
            raw_path,
            theme,
            "Preview is unavailable for non-local images.",
        );
    };
    if !is_previewable_image_path(&path) {
        return fallback_lines(
            &path,
            theme,
            "Preview is unavailable for this image format.",
        );
    }

    let key = PreviewCacheKey {
        path: path.clone(),
        width,
        max_height_lines,
    };
    let entry = runtime.lookup_or_enqueue(PreviewJob { key });
    match entry {
        PreviewCacheEntry::Pending => fallback_lines(&path, theme, "Loading image preview..."),
        PreviewCacheEntry::Ready(preview) => render_cached_preview_lines(&preview, theme),
        PreviewCacheEntry::Failed(reason) => fallback_lines(&path, theme, &reason),
    }
}

pub(crate) fn render_image_preview_lines(
    raw_path: &str,
    width: usize,
    max_height_lines: usize,
    theme: &ThemeTokens,
) -> Vec<Line<'static>> {
    render_image_preview_lines_with_runtime(
        preview_runtime(),
        raw_path,
        width,
        max_height_lines,
        theme,
    )
}

pub(crate) fn preview_cache_revision() -> u64 {
    lock_store(&preview_runtime().inner.store).revision()
}

#[cfg(test)]
pub(crate) fn process_preview_jobs_for_path_until_stable_for_tests(raw_path: &str) -> bool {
    let Some(path) = resolve_local_image_path(raw_path) else {
        return false;
    };
    let runtime = preview_runtime();

    loop {
        let (saw_entry, has_pending, has_ready) = {
            let store = lock_store(&runtime.inner.store);
            let mut saw_entry = false;
            let mut has_pending = false;
            let mut has_ready = false;
            for (key, entry) in &store.entries {
                if key.path != path {
                    continue;
                }
                saw_entry = true;
                match entry {
                    PreviewCacheEntry::Pending => has_pending = true,
                    PreviewCacheEntry::Ready(_) => has_ready = true,
                    PreviewCacheEntry::Failed(_) => {}
                }
            }
            (saw_entry, has_pending, has_ready)
        };

        if !saw_entry {
            return false;
        }
        if !has_pending {
            return has_ready;
        }
        if !runtime.process_next_job_for_tests() {
            return false;
        }
    }
}

#[cfg(test)]
#[path = "image_preview_tests_parts"]
mod tests {
    

    mod render_image_preview_lines_queues_work_and_reuses_cached_result;
}
