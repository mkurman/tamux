use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Persisted daemon state (saved between restarts).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DaemonState {
    /// Sessions that were running when the daemon last shut down.
    /// We store metadata only — the actual PTY processes are gone after a
    /// daemon restart, but we record them so the UI can show "stale" sessions
    /// and offer to re-create them.
    pub previous_sessions: Vec<SavedSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSession {
    pub id: String,
    pub shell: Option<String>,
    pub cwd: Option<String>,
    pub workspace_id: Option<String>,
    pub cols: u16,
    pub rows: u16,
}

fn legacy_state_path() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("amux").join("daemon-state.json")
}

fn canonical_state_path() -> PathBuf {
    let base = amux_protocol::ensure_amux_data_dir().unwrap_or_else(|_| PathBuf::from("."));
    base.join("daemon-state.json")
}

fn migrate_legacy_state_file(target: &Path) {
    let legacy = legacy_state_path();
    if target.exists() || !legacy.exists() {
        return;
    }

    if let Some(parent) = target.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::rename(&legacy, target);
}

/// Default path for the state file.
pub fn default_state_path() -> PathBuf {
    let path = canonical_state_path();
    migrate_legacy_state_file(&path);
    path
}

/// Load state from disk.
#[allow(dead_code)]
pub fn load_state(path: &std::path::Path) -> Result<DaemonState> {
    migrate_legacy_state_file(path);
    if path.exists() {
        let data = std::fs::read_to_string(path)?;
        let state: DaemonState = serde_json::from_str(&data)?;
        Ok(state)
    } else {
        Ok(DaemonState::default())
    }
}

/// Save state to disk.
#[allow(dead_code)]
pub fn save_state(path: &std::path::Path, state: &DaemonState) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(state)?;
    std::fs::write(path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_state_creates_parent_directories() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let path = tempdir.path().join("nested").join("daemon-state.json");

        save_state(&path, &DaemonState::default()).expect("save state");

        assert!(path.exists(), "state file should be written");
    }

    #[test]
    fn load_state_reads_existing_file() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let path = tempdir.path().join("daemon-state.json");
        let state = DaemonState {
            previous_sessions: vec![SavedSession {
                id: "session-1".to_string(),
                shell: Some("/bin/zsh".to_string()),
                cwd: Some("/tmp".to_string()),
                workspace_id: Some("workspace-a".to_string()),
                cols: 120,
                rows: 40,
            }],
        };

        save_state(&path, &state).expect("save state");

        let loaded = load_state(&path).expect("load state");
        assert_eq!(loaded.previous_sessions.len(), 1);
        assert_eq!(loaded.previous_sessions[0].id, "session-1");
    }
}
