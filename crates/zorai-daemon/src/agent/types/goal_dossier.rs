use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalProjectionState {
    #[default]
    Pending,
    InProgress,
    Blocked,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalResumeAction {
    #[default]
    Advance,
    Pause,
    Stop,
    Replan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalRoleBinding {
    Builtin(String),
    Subagent(String),
}

impl Default for GoalRoleBinding {
    fn default() -> Self {
        Self::Builtin(String::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoalEvidenceRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoalProofCheck {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub state: GoalProjectionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoalRunReport {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub state: GoalProjectionState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<GoalEvidenceRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_checks: Vec<GoalProofCheck>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoalResumeDecision {
    #[serde(default)]
    pub action: GoalResumeAction,
    #[serde(default)]
    pub reason_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decided_at: Option<u64>,
    #[serde(default)]
    pub projection_state: GoalProjectionState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoalDeliveryUnit {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: GoalProjectionState,
    #[serde(default)]
    pub execution_binding: GoalRoleBinding,
    #[serde(default)]
    pub verification_binding: GoalRoleBinding,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_checks: Vec<GoalProofCheck>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<GoalEvidenceRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<GoalRunReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoalRunDossier {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub units: Vec<GoalDeliveryUnit>,
    #[serde(default)]
    pub projection_state: GoalProjectionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_resume_decision: Option<GoalResumeDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<GoalRunReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_error: Option<String>,
}
