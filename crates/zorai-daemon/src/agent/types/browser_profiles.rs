use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BrowserProfile {
    pub profile_id: String,
    pub label: String,
    pub profile_dir: String,
    pub browser_kind: Option<String>,
    pub workspace_id: Option<String>,
    pub health_state: BrowserProfileHealth,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_used_at: Option<u64>,
    pub last_auth_success_at: Option<u64>,
    pub last_auth_failure_at: Option<u64>,
    pub last_auth_failure_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BrowserProfileHealth {
    Healthy,
    Stale,
    Expired,
    Corrupted,
    RepairNeeded,
    RepairInProgress,
    Retired,
}

impl Default for BrowserProfileHealth {
    fn default() -> Self {
        Self::Healthy
    }
}

impl BrowserProfileHealth {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Stale => "stale",
            Self::Expired => "expired",
            Self::Corrupted => "corrupted",
            Self::RepairNeeded => "repair_needed",
            Self::RepairInProgress => "repair_in_progress",
            Self::Retired => "retired",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "healthy" => Some(Self::Healthy),
            "stale" => Some(Self::Stale),
            "expired" => Some(Self::Expired),
            "corrupted" => Some(Self::Corrupted),
            "repair_needed" => Some(Self::RepairNeeded),
            "repair_in_progress" => Some(Self::RepairInProgress),
            "retired" => Some(Self::Retired),
            _ => None,
        }
    }
}
