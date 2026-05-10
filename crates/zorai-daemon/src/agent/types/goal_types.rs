use super::*;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalRunStatus {
    Queued,
    Planning,
    Running,
    AwaitingApproval,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalRunStepKind {
    #[default]
    Reason,
    Command,
    Research,
    Memory,
    Skill,
    /// Route this step to a specialist subagent via the handoff broker.
    /// The String is the specialist role name (e.g., "backend-developer").
    Specialist(String),
    /// Spawn a divergent session with parallel framings for this step.
    /// The step instructions become the problem statement.
    Divergent,
    /// Start a structured debate session for this step.
    /// The step instructions become the debate topic.
    Debate,
    /// Fallback for unknown/empty kind values from LLM output.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GoalRunStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRunStep {
    pub id: String,
    pub position: usize,
    pub title: String,
    pub instructions: String,
    pub kind: GoalRunStepKind,
    pub success_criteria: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub status: GoalRunStepStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRunEvent {
    pub id: String,
    pub timestamp: u64,
    pub phase: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub todo_snapshot: Vec<TodoItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GoalAgentAssignment {
    pub role_id: String,
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    pub inherit_from_main: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct GoalRuntimeOwnerProfile {
    pub agent_label: String,
    pub provider: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GoalRunModelUsage {
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub request_count: u64,
    #[serde(default)]
    pub prompt_tokens: u64,
    #[serde(default)]
    pub completion_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalRun {
    pub id: String,
    pub title: String,
    pub goal: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_request_id: Option<String>,
    pub status: GoalRunStatus,
    pub priority: TaskPriority,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_thread_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub current_step_index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step_kind: Option<GoalRunStepKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub launch_assignment_snapshot: Vec<GoalAgentAssignment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtime_assignment_list: Vec<GoalAgentAssignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planner_owner_profile: Option<GoalRuntimeOwnerProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_step_owner_profile: Option<GoalRuntimeOwnerProfile>,
    pub replan_count: u32,
    pub max_replans: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reflection_summary: Option<String>,
    #[serde(default)]
    pub memory_updates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_skill_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_cause: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stopped_reason: Option<String>,
    #[serde(default)]
    pub child_task_ids: Vec<String>,
    #[serde(default)]
    pub child_task_count: u32,
    #[serde(default)]
    pub approval_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub awaiting_approval_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_expires_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containment_scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensation_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensation_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub steps: Vec<GoalRunStep>,
    #[serde(default)]
    pub events: Vec<GoalRunEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dossier: Option<GoalRunDossier>,
    /// Total prompt tokens consumed across all LLM calls in this goal run (COST-01).
    #[serde(default)]
    pub total_prompt_tokens: u64,
    /// Total completion tokens consumed across all LLM calls in this goal run (COST-01).
    #[serde(default)]
    pub total_completion_tokens: u64,
    /// Estimated cost in USD based on provider rate cards (COST-02).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
    /// Per-provider/model usage and timing statistics for this goal run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_usage: Vec<GoalRunModelUsage>,
    /// Per-goal autonomy dial: autonomous / aware / supervised (AUTO-01).
    #[serde(default)]
    pub autonomy_level: super::super::autonomy::AutonomyLevel,
    /// Attribution tag for goal-run output (AUTH-01).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authorship_tag: Option<super::super::authorship::AuthorshipTag>,
}

