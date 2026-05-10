use serde::{Deserialize, Serialize};
use serde_json::Value;

fn default_external_runtime_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalRuntimeConflictPolicy {
    Skip,
    Merge,
    Replace,
    StageForReview,
}

impl Default for ExternalRuntimeConflictPolicy {
    fn default() -> Self {
        Self::StageForReview
    }
}

impl ExternalRuntimeConflictPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Merge => "merge",
            Self::Replace => "replace",
            Self::StageForReview => "stage_for_review",
        }
    }
}

impl std::str::FromStr for ExternalRuntimeConflictPolicy {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "skip" => Ok(Self::Skip),
            "merge" => Ok(Self::Merge),
            "replace" => Ok(Self::Replace),
            "stage_for_review" | "stage-for-review" | "stage" => Ok(Self::StageForReview),
            other => Err(format!("unsupported conflict policy '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalRuntimeAssetBucket {
    Imported,
    Mapped,
    Unsupported,
    Missing,
    ManualActionRequired,
}

impl ExternalRuntimeAssetBucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Imported => "imported",
            Self::Mapped => "mapped",
            Self::Unsupported => "unsupported",
            Self::Missing => "missing",
            Self::ManualActionRequired => "manual_action_required",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalRuntimeReportSeverity {
    Safe,
    Informational,
    Warning,
    Blocking,
}

impl ExternalRuntimeReportSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Informational => "informational",
            Self::Warning => "warning",
            Self::Blocking => "blocking",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalRuntimeProfile {
    pub runtime: String,
    pub source_config_path: String,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub cwd: Option<String>,
    pub has_zorai_mcp: bool,
    pub imported_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalRuntimeImportSession {
    pub session_id: String,
    pub runtime: String,
    pub source_config_path: String,
    pub source_fingerprint: String,
    pub dry_run: bool,
    pub conflict_policy: ExternalRuntimeConflictPolicy,
    pub source_surface: String,
    pub imported_at_ms: u64,
    #[serde(default = "default_external_runtime_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub asset_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportedRuntimeAsset {
    pub asset_id: String,
    pub session_id: String,
    pub runtime: String,
    pub asset_kind: String,
    pub bucket: ExternalRuntimeAssetBucket,
    pub severity: ExternalRuntimeReportSeverity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_fingerprint: Option<String>,
    pub conflict_policy: ExternalRuntimeConflictPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_query_hint: Option<String>,
    pub payload: Value,
    pub created_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalRuntimeShadowRunOutcome {
    pub run_id: String,
    pub runtime: String,
    pub session_id: String,
    pub workflow: String,
    pub readiness_score: u8,
    pub blocker_count: u32,
    pub summary: String,
    pub payload: Value,
    pub created_at_ms: u64,
}
