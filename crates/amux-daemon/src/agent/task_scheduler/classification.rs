use super::*;

const TASK_APPROVAL_REASON_PREFIX: &str = "waiting for operator approval:";

pub(in crate::agent) fn project_task_runs(
    tasks: &[AgentTask],
    sessions: &[amux_protocol::SessionInfo],
) -> Vec<AgentRun> {
    let task_titles = tasks
        .iter()
        .map(|task| (task.id.as_str(), task.title.as_str()))
        .collect::<HashMap<_, _>>();
    let session_workspaces = sessions
        .iter()
        .map(|session| (session.id.to_string(), session.workspace_id.clone()))
        .collect::<HashMap<_, _>>();

    tasks
        .iter()
        .map(|task| {
            let session_id = task
                .session_id
                .clone()
                .filter(|value| !value.trim().is_empty());
            let workspace_id = session_id
                .as_deref()
                .and_then(|value| session_workspaces.get(value))
                .cloned()
                .flatten();
            let kind = if task.source == "subagent"
                || task
                    .parent_task_id
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
                || task
                    .parent_thread_id
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            {
                AgentRunKind::Subagent
            } else {
                AgentRunKind::Task
            };

            AgentRun {
                id: task.id.clone(),
                task_id: task.id.clone(),
                kind,
                classification: classify_task(task).to_string(),
                title: task.title.clone(),
                description: task.description.clone(),
                status: task.status,
                runtime_status: project_run_runtime_status(task),
                priority: task.priority,
                progress: task.progress,
                created_at: task.created_at,
                started_at: task.started_at,
                completed_at: task.completed_at,
                thread_id: task.thread_id.clone(),
                session_id,
                workspace_id,
                source: task.source.clone(),
                runtime: task.runtime.clone(),
                goal_run_id: task.goal_run_id.clone(),
                goal_run_title: task.goal_run_title.clone(),
                goal_step_id: task.goal_step_id.clone(),
                goal_step_title: task.goal_step_title.clone(),
                parent_run_id: task.parent_task_id.clone(),
                parent_task_id: task.parent_task_id.clone(),
                parent_thread_id: task.parent_thread_id.clone(),
                parent_title: task
                    .parent_task_id
                    .as_deref()
                    .and_then(|value| task_titles.get(value))
                    .map(|value| (*value).to_string()),
                blocked_reason: task.blocked_reason.clone(),
                error: task.error.clone(),
                result: task.result.clone(),
                last_error: task.last_error.clone(),
            }
        })
        .collect()
}

fn project_run_runtime_status(task: &AgentTask) -> AgentRunRuntimeStatus {
    let reason = normalized_reason(task.blocked_reason.as_deref());
    let awaiting_approval_id = task.awaiting_approval_id.clone();
    let next_retry_at = task.next_retry_at;
    let scheduled_at = task.scheduled_at;

    let kind = match task.status {
        TaskStatus::Queued => AgentRunRuntimeStatusKind::Queued,
        TaskStatus::InProgress => AgentRunRuntimeStatusKind::Running,
        TaskStatus::AwaitingApproval => AgentRunRuntimeStatusKind::AwaitingApproval,
        TaskStatus::Blocked => blocked_runtime_status_kind(task),
        TaskStatus::FailedAnalyzing => {
            if next_retry_at.is_some() {
                AgentRunRuntimeStatusKind::Retrying
            } else {
                AgentRunRuntimeStatusKind::FailedAnalyzing
            }
        }
        TaskStatus::BudgetExceeded => AgentRunRuntimeStatusKind::BudgetExceeded,
        TaskStatus::Completed => AgentRunRuntimeStatusKind::Completed,
        TaskStatus::Failed => AgentRunRuntimeStatusKind::Failed,
        TaskStatus::Cancelled => AgentRunRuntimeStatusKind::Cancelled,
    };

    AgentRunRuntimeStatus {
        kind,
        reason,
        awaiting_approval_id,
        next_retry_at,
        scheduled_at,
    }
}

fn blocked_runtime_status_kind(task: &AgentTask) -> AgentRunRuntimeStatusKind {
    let Some(reason) = task.blocked_reason.as_deref().map(str::trim) else {
        return AgentRunRuntimeStatusKind::Blocked;
    };
    let lower = reason.to_ascii_lowercase();

    if task.awaiting_approval_id.is_some()
        || lower.starts_with(TASK_APPROVAL_REASON_PREFIX)
        || lower.contains("awaiting approval")
        || lower.contains("needs user approval")
        || lower.contains("supervised acknowledgment")
    {
        AgentRunRuntimeStatusKind::AwaitingApproval
    } else if lower.starts_with("waiting for dependencies:") {
        AgentRunRuntimeStatusKind::WaitingForDependencies
    } else if lower.starts_with("waiting for subagents:") {
        AgentRunRuntimeStatusKind::WaitingForSubagents
    } else if lower.starts_with("scheduled for ") {
        AgentRunRuntimeStatusKind::Scheduled
    } else if lower.starts_with("waiting for lane availability:")
        || lower.starts_with("waiting for subagent slot:")
        || lower.starts_with("waiting for workspace lock:")
    {
        AgentRunRuntimeStatusKind::WaitingForResources
    } else {
        AgentRunRuntimeStatusKind::Blocked
    }
}

fn normalized_reason(reason: Option<&str>) -> Option<String> {
    reason
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub(in crate::agent) fn classify_task(task: &AgentTask) -> &'static str {
    let haystack = format!(
        "{} {} {} {}",
        task.title,
        task.description,
        task.command.as_deref().unwrap_or_default(),
        task.source
    )
    .to_ascii_lowercase();

    if contains_any(
        &haystack,
        &[
            "code",
            "coding",
            "repo",
            "git",
            "diff",
            "patch",
            "file",
            "test",
            "build",
            "compile",
            "rust",
            "typescript",
            "frontend",
            "backend",
            "refactor",
            "implement",
        ],
    ) {
        "coding"
    } else if contains_any(
        &haystack,
        &[
            "browser", "browse", "web", "page", "url", "search", "navigate",
        ],
    ) {
        "browser"
    } else if contains_any(
        &haystack,
        &[
            "slack", "discord", "telegram", "whatsapp", "message", "reply", "channel",
        ],
    ) {
        "messaging"
    } else if contains_any(
        &haystack,
        &[
            "terminal", "shell", "daemon", "deploy", "restart", "service", "ops", "infra",
        ],
    ) {
        "ops"
    } else if contains_any(
        &haystack,
        &[
            "research",
            "investigate",
            "analyze",
            "analyse",
            "explain",
            "read",
            "audit",
        ],
    ) {
        "research"
    } else {
        "mixed"
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
