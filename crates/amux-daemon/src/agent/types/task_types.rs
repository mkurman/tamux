// ---------------------------------------------------------------------------
// Task queue
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Sub-agent management
// ---------------------------------------------------------------------------

/// Configuration for sub-agent supervision — how often to check, when to
/// consider a sub-agent stuck, and what intervention level to apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    /// How often to check sub-agent health (seconds). Default: 30.
    #[serde(default = "default_supervisor_check_interval")]
    pub check_interval_secs: u64,
    /// Seconds of no progress before flagging as stuck. Default: 300 (5 min).
    #[serde(default = "default_stuck_timeout")]
    pub stuck_timeout_secs: u64,
    /// Maximum retries before escalating. Default: 2.
    #[serde(default = "default_supervisor_max_retries")]
    pub max_retries: u32,
    /// How aggressively to intervene. Default: Normal.
    #[serde(default)]
    pub intervention_level: InterventionLevel,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: default_supervisor_check_interval(),
            stuck_timeout_secs: default_stuck_timeout(),
            max_retries: default_supervisor_max_retries(),
            intervention_level: InterventionLevel::default(),
        }
    }
}

fn default_supervisor_check_interval() -> u64 {
    30
}
fn default_stuck_timeout() -> u64 {
    300
}
fn default_supervisor_max_retries() -> u32 {
    2
}

/// How aggressively the supervisor should intervene when issues are detected.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum InterventionLevel {
    /// Only log, never intervene automatically.
    Passive,
    /// Self-correct where safe (compress context, inject reflection).
    #[default]
    Normal,
    /// Aggressively intervene (terminate stuck agents, retry from checkpoint).
    Aggressive,
}

/// Overall health state of a sub-agent as determined by the supervisor.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentHealthState {
    #[default]
    Healthy,
    Degraded,
    Stuck,
    Crashed,
}

/// Why a sub-agent is considered stuck.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StuckReason {
    /// No tool calls or progress for configured timeout.
    NoProgress,
    /// Same error repeated 3+ times in a row.
    ErrorLoop,
    /// Cycling tool calls (A→B→A→B pattern).
    ToolCallLoop,
    /// Context budget > 90% consumed.
    ResourceExhaustion,
    /// Exceeded max_duration_secs.
    Timeout,
}

/// What the supervisor should do when a problem is detected.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterventionAction {
    /// Inject a self-assessment prompt asking the agent to reflect.
    SelfAssess,
    /// Compress context to free up budget.
    CompressContext,
    /// Retry from the last successful checkpoint.
    RetryFromCheckpoint,
    /// Escalate to the parent task/agent.
    EscalateToParent,
    /// Escalate to the user for manual intervention.
    EscalateToUser,
}

/// What to do when a context budget is exceeded.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContextOverflowAction {
    /// Compress older context to free space.
    #[default]
    Compress,
    /// Truncate oldest messages.
    Truncate,
    /// Return an error and stop execution.
    Error,
}

// ---------------------------------------------------------------------------
// Task queue
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Queued,
    #[serde(alias = "running")]
    InProgress,
    AwaitingApproval,
    Blocked,
    FailedAnalyzing,
    BudgetExceeded,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskLogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskLogEntry {
    pub id: String,
    pub timestamp: u64,
    pub level: TaskLogLevel,
    pub phase: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default)]
    pub attempt: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    #[serde(default)]
    pub priority: TaskPriority,
    #[serde(default)]
    pub progress: u8,
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub notify_on_complete: bool,
    #[serde(default)]
    pub notify_channels: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_run_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_step_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_step_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_thread_id: Option<String>,
    #[serde(default = "default_task_runtime")]
    pub runtime: String,
    #[serde(default)]
    pub retry_count: u32,
    #[serde(default = "default_max_task_retries")]
    pub max_retries: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
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
    pub lane_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default)]
    pub logs: Vec<AgentTaskLogEntry>,

    // -- Sub-agent management extensions (Phase 1) --
    /// Restrict which tools this sub-agent may call. `None` = all tools allowed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_whitelist: Option<Vec<String>>,
    /// Tools this sub-agent must NOT call. Applied after whitelist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_blacklist: Option<Vec<String>>,
    /// Maximum tokens this sub-agent may consume for its context window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_budget_tokens: Option<u32>,
    /// What to do when the context budget is exceeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_overflow_action: Option<ContextOverflowAction>,
    /// DSL expression for automatic termination (e.g. "timeout(300) OR error_count(3)").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination_conditions: Option<String>,
    /// Criteria the sub-agent must satisfy for the step to be considered successful.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success_criteria: Option<String>,
    /// Hard time limit in seconds (fallback: 1800 = 30 min).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_duration_secs: Option<u64>,
    /// Supervision configuration for this sub-agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supervisor_config: Option<SupervisorConfig>,

    // -- Provider/model override for sub-agent dispatch --
    /// Override provider for this task (from SubAgentDefinition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_provider: Option<String>,
    /// Override model for this task (from SubAgentDefinition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_model: Option<String>,
    /// Override system prompt for this task (from SubAgentDefinition).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_system_prompt: Option<String>,
    /// The SubAgentDefinition ID this task was spawned from, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_agent_def_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunKind {
    Task,
    Subagent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunRuntimeStatusKind {
    Queued,
    Running,
    AwaitingApproval,
    WaitingForDependencies,
    WaitingForSubagents,
    Scheduled,
    WaitingForResources,
    Blocked,
    Retrying,
    FailedAnalyzing,
    BudgetExceeded,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentRunRuntimeStatus {
    pub kind: AgentRunRuntimeStatusKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub awaiting_approval_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled_at: Option<u64>,
}

impl Default for AgentRunRuntimeStatus {
    fn default() -> Self {
        Self {
            kind: AgentRunRuntimeStatusKind::Queued,
            reason: None,
            awaiting_approval_id: None,
            next_retry_at: None,
            scheduled_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: String,
    pub task_id: String,
    pub kind: AgentRunKind,
    pub classification: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    #[serde(default)]
    pub runtime_status: AgentRunRuntimeStatus,
    #[serde(default)]
    pub priority: TaskPriority,
    #[serde(default)]
    pub progress: u8,
    pub created_at: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_task_runtime")]
    pub runtime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_run_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_step_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_step_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_task_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

fn default_source() -> String {
    "user".into()
}

fn default_max_task_retries() -> u32 {
    3
}

fn default_task_runtime() -> String {
    "daemon".into()
}
