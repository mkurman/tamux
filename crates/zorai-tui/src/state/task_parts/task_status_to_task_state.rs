pub const GOAL_RUN_HISTORY_FETCH_DEBOUNCE_TICKS: u64 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    InProgress,
    AwaitingApproval,
    Blocked,
    FailedAnalyzing,
    BudgetExceeded,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Default)]
pub struct AgentTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub thread_id: Option<String>,
    pub parent_task_id: Option<String>,
    pub parent_thread_id: Option<String>,
    pub created_at: u64,
    pub status: Option<TaskStatus>,
    pub progress: u8,
    pub session_id: Option<String>,
    pub goal_run_id: Option<String>,
    pub goal_step_title: Option<String>,
    pub command: Option<String>,
    pub awaiting_approval_id: Option<String>,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Default)]
pub struct GoalRunStep {
    pub id: String,
    pub title: String,
    pub status: Option<GoalRunStatus>,
    pub order: u32,
    pub instructions: String,
    pub kind: String,
    pub task_id: Option<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GoalRunEvent {
    pub id: String,
    pub timestamp: u64,
    pub phase: String,
    pub message: String,
    pub details: Option<String>,
    pub step_index: Option<usize>,
    pub todo_snapshot: Vec<TodoItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GoalRuntimeOwnerProfile {
    pub agent_label: String,
    pub provider: String,
    pub model: String,
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GoalAgentAssignment {
    pub role_id: String,
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    pub reasoning_effort: Option<String>,
    pub inherit_from_main: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GoalRunModelUsage {
    pub provider: String,
    pub model: String,
    pub request_count: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub estimated_cost_usd: Option<f64>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct GoalRun {
    pub id: String,
    pub title: String,
    pub thread_id: Option<String>,
    pub root_thread_id: Option<String>,
    pub active_thread_id: Option<String>,
    pub execution_thread_ids: Vec<String>,
    pub session_id: Option<String>,
    pub status: Option<GoalRunStatus>,
    pub current_step_title: Option<String>,
    pub launch_assignment_snapshot: Vec<GoalAgentAssignment>,
    pub runtime_assignment_list: Vec<GoalAgentAssignment>,
    pub planner_owner_profile: Option<GoalRuntimeOwnerProfile>,
    pub current_step_owner_profile: Option<GoalRuntimeOwnerProfile>,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub estimated_cost_usd: Option<f64>,
    pub model_usage: Vec<GoalRunModelUsage>,
    pub child_task_count: u32,
    pub approval_count: u32,
    pub awaiting_approval_id: Option<String>,
    pub last_error: Option<String>,
    pub goal: String,
    pub current_step_index: usize,
    pub reflection_summary: Option<String>,
    pub memory_updates: Vec<String>,
    pub generated_skill_path: Option<String>,
    pub child_task_ids: Vec<String>,
    pub loaded_step_start: usize,
    pub loaded_step_end: usize,
    pub total_step_count: usize,
    pub loaded_event_start: usize,
    pub loaded_event_end: usize,
    pub total_event_count: usize,
    pub older_page_pending: bool,
    pub older_page_request_cooldown_until_tick: Option<u64>,
    pub sparse_update: bool,
    pub steps: Vec<GoalRunStep>,
    pub events: Vec<GoalRunEvent>,
    pub dossier: Option<GoalRunDossier>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Default)]
pub struct GoalEvidenceRecord {
    pub id: String,
    pub title: String,
    pub source: Option<String>,
    pub uri: Option<String>,
    pub summary: Option<String>,
    pub captured_at: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct GoalProofCheckRecord {
    pub id: String,
    pub title: String,
    pub state: String,
    pub summary: Option<String>,
    pub evidence_ids: Vec<String>,
    pub resolved_at: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct GoalRunReportRecord {
    pub summary: String,
    pub state: String,
    pub notes: Vec<String>,
    pub evidence: Vec<GoalEvidenceRecord>,
    pub proof_checks: Vec<GoalProofCheckRecord>,
    pub generated_at: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct GoalResumeDecisionRecord {
    pub action: String,
    pub reason_code: String,
    pub reason: Option<String>,
    pub details: Vec<String>,
    pub decided_at: Option<u64>,
    pub projection_state: String,
}

#[derive(Debug, Clone, Default)]
pub struct GoalDeliveryUnitRecord {
    pub id: String,
    pub title: String,
    pub status: String,
    pub execution_binding: String,
    pub verification_binding: String,
    pub summary: Option<String>,
    pub proof_checks: Vec<GoalProofCheckRecord>,
    pub evidence: Vec<GoalEvidenceRecord>,
    pub report: Option<GoalRunReportRecord>,
}

#[derive(Debug, Clone, Default)]
pub struct GoalRunDossier {
    pub units: Vec<GoalDeliveryUnitRecord>,
    pub projection_state: String,
    pub latest_resume_decision: Option<GoalResumeDecisionRecord>,
    pub report: Option<GoalRunReportRecord>,
    pub summary: Option<String>,
    pub projection_error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GoalRunCheckpointSummary {
    pub id: String,
    pub checkpoint_type: String,
    pub step_index: Option<usize>,
    pub task_count: usize,
    pub context_summary_preview: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartbeatOutcome {
    Ok,
    Warn,
    Error,
}

#[derive(Debug, Clone, Default)]
pub struct HeartbeatItem {
    pub id: String,
    pub label: String,
    pub outcome: Option<HeartbeatOutcome>,
    pub message: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Default)]
pub struct HeartbeatDigestVm {
    pub cycle_id: String,
    pub actionable: bool,
    pub digest: String,
    pub items: Vec<HeartbeatDigestItemVm>,
    pub checked_at: u64,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HeartbeatDigestItemVm {
    pub priority: u8,
    pub check_type: String,
    pub title: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

#[derive(Debug, Clone, Default)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: Option<TodoStatus>,
    pub position: usize,
    pub step_index: Option<usize>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkContextEntryKind {
    RepoChange,
    Artifact,
    GeneratedSkill,
}

#[derive(Debug, Clone, Default)]
pub struct WorkContextEntry {
    pub path: String,
    pub previous_path: Option<String>,
    pub kind: Option<WorkContextEntryKind>,
    pub source: String,
    pub change_kind: Option<String>,
    pub repo_root: Option<String>,
    pub goal_run_id: Option<String>,
    pub step_index: Option<usize>,
    pub session_id: Option<String>,
    pub is_text: bool,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ThreadWorkContext {
    pub thread_id: String,
    pub entries: Vec<WorkContextEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct FilePreview {
    pub path: String,
    pub content: String,
    pub truncated: bool,
    pub is_text: bool,
}


#[derive(Debug, Clone)]
pub enum TaskAction {
    TaskListReceived(Vec<AgentTask>),
    TaskUpdate(AgentTask),
    GoalRunListReceived(Vec<GoalRun>),
    GoalRunDetailReceived(GoalRun),
    GoalRunUpdate(GoalRun),
    GoalRunCheckpointsReceived {
        goal_run_id: String,
        checkpoints: Vec<GoalRunCheckpointSummary>,
    },
    GoalRunDeleted {
        goal_run_id: String,
    },
    ThreadTodosReceived {
        thread_id: String,
        goal_run_id: Option<String>,
        step_index: Option<usize>,
        items: Vec<TodoItem>,
    },
    WorkContextReceived(ThreadWorkContext),
    GitDiffReceived {
        repo_path: String,
        file_path: Option<String>,
        diff: String,
    },
    FilePreviewReceived(FilePreview),
    SelectWorkPath {
        thread_id: String,
        path: Option<String>,
    },
    HeartbeatItemsReceived(Vec<HeartbeatItem>),
    HeartbeatDigestReceived(HeartbeatDigestVm),
}


pub struct TaskState {
    pub(crate) tasks: Vec<AgentTask>,
    pub(crate) tasks_revision: u64,
    pub(crate) preview_revision: u64,
    pub(crate) goal_runs: Vec<GoalRun>,
    pub(crate) goal_run_checkpoints:
        std::collections::HashMap<String, Vec<GoalRunCheckpointSummary>>,
    pub(crate) thread_todos: std::collections::HashMap<String, Vec<TodoItem>>,
    pub(crate) goal_step_live_todos: std::collections::HashMap<String, Vec<TodoItem>>,
    pub(crate) goal_thread_ids: std::collections::HashMap<String, Vec<String>>,
    pub(crate) work_contexts: std::collections::HashMap<String, ThreadWorkContext>,
    pub(crate) selected_work_paths: std::collections::HashMap<String, String>,
    pub(crate) git_diffs: std::collections::HashMap<String, String>,
    pub(crate) file_previews: std::collections::HashMap<String, FilePreview>,
    pub(crate) heartbeat_items: Vec<HeartbeatItem>,
    pub(crate) last_digest: Option<HeartbeatDigestVm>,
    pub(crate) goal_thread_ids_cache:
        std::cell::RefCell<Option<(u64, Vec<String>)>>,
}
