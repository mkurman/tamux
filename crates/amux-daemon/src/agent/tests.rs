use super::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex as StdMutex};
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn sample_goal_run() -> GoalRun {
    GoalRun {
        id: "goal_test".to_string(),
        title: "Test goal".to_string(),
        goal: "Ship something".to_string(),
        client_request_id: None,
        status: GoalRunStatus::Failed,
        priority: TaskPriority::Normal,
        created_at: 10,
        updated_at: 30,
        started_at: Some(20),
        completed_at: Some(80),
        thread_id: None,
        session_id: Some("session-1".to_string()),
        current_step_index: 1,
        current_step_title: None,
        current_step_kind: None,
        replan_count: 1,
        max_replans: 2,
        plan_summary: Some("Plan".to_string()),
        reflection_summary: None,
        memory_updates: Vec::new(),
        generated_skill_path: Some("/tmp/skill.md".to_string()),
        last_error: Some("child task failed".to_string()),
        failure_cause: None,
        child_task_ids: vec!["task-a".to_string(), "task-b".to_string()],
        child_task_count: 0,
        approval_count: 0,
        awaiting_approval_id: None,
        policy_fingerprint: None,
        approval_expires_at: None,
        containment_scope: None,
        compensation_status: None,
        compensation_summary: None,
        active_task_id: None,
        duration_ms: None,
        steps: vec![
            GoalRunStep {
                id: "step-0".to_string(),
                position: 0,
                title: "Inspect".to_string(),
                instructions: "Inspect state".to_string(),
                kind: GoalRunStepKind::Research,
                success_criteria: "Know what failed".to_string(),
                session_id: None,
                status: GoalRunStepStatus::Completed,
                task_id: Some("task-a".to_string()),
                summary: Some("done".to_string()),
                error: None,
                started_at: Some(21),
                completed_at: Some(30),
            },
            GoalRunStep {
                id: "step-1".to_string(),
                position: 1,
                title: "Fix".to_string(),
                instructions: "Fix it".to_string(),
                kind: GoalRunStepKind::Command,
                success_criteria: "Green".to_string(),
                session_id: Some("session-1".to_string()),
                status: GoalRunStepStatus::Failed,
                task_id: Some("task-b".to_string()),
                summary: None,
                error: Some("step failure".to_string()),
                started_at: Some(31),
                completed_at: Some(50),
            },
        ],
        events: Vec::new(),
        total_prompt_tokens: 0,
        total_completion_tokens: 0,
        estimated_cost_usd: None,
        autonomy_level: Default::default(),
        authorship_tag: None,
    }
}

fn sample_task(id: &str, goal_run_id: &str) -> AgentTask {
    AgentTask {
        id: id.to_string(),
        title: id.to_string(),
        description: id.to_string(),
        status: TaskStatus::Failed,
        priority: TaskPriority::Normal,
        progress: 0,
        created_at: 1,
        started_at: Some(2),
        completed_at: Some(3),
        error: Some("task error".to_string()),
        result: None,
        thread_id: None,
        source: "goal_run".to_string(),
        notify_on_complete: false,
        notify_channels: Vec::new(),
        dependencies: Vec::new(),
        command: None,
        session_id: Some("session-1".to_string()),
        goal_run_id: Some(goal_run_id.to_string()),
        goal_run_title: Some("Test goal".to_string()),
        goal_step_id: Some("step-1".to_string()),
        goal_step_title: Some("Fix".to_string()),
        parent_task_id: None,
        parent_thread_id: None,
        runtime: "daemon".to_string(),
        retry_count: 0,
        max_retries: 0,
        next_retry_at: None,
        scheduled_at: None,
        blocked_reason: None,
        awaiting_approval_id: Some("apr-1".to_string()),
        policy_fingerprint: None,
        approval_expires_at: None,
        containment_scope: None,
        compensation_status: None,
        compensation_summary: None,
        lane_id: None,
        last_error: Some("task error".to_string()),
        logs: vec![AgentTaskLogEntry {
            id: format!("log-{id}"),
            timestamp: 4,
            level: TaskLogLevel::Warn,
            phase: "approval".to_string(),
            message: "managed command paused for operator approval".to_string(),
            details: None,
            attempt: 0,
        }],
        tool_whitelist: None,
        tool_blacklist: None,
        context_budget_tokens: None,
        context_overflow_action: None,
        termination_conditions: None,
        success_criteria: None,
        max_duration_secs: None,
        supervisor_config: None,
        override_provider: None,
        override_model: None,
        override_system_prompt: None,
        sub_agent_def_id: None,
    }
}

fn sample_subagent(id: &str, parent_task_id: &str, status: TaskStatus) -> AgentTask {
    AgentTask {
        id: id.to_string(),
        title: format!("Subagent {id}"),
        description: "Child work".to_string(),
        status,
        priority: TaskPriority::Normal,
        progress: 0,
        created_at: 1,
        started_at: None,
        completed_at: None,
        error: None,
        result: None,
        thread_id: None,
        source: "subagent".to_string(),
        notify_on_complete: false,
        notify_channels: Vec::new(),
        dependencies: Vec::new(),
        command: None,
        session_id: None,
        goal_run_id: None,
        goal_run_title: None,
        goal_step_id: None,
        goal_step_title: None,
        parent_task_id: Some(parent_task_id.to_string()),
        parent_thread_id: Some("thread-parent".to_string()),
        runtime: "daemon".to_string(),
        retry_count: 0,
        max_retries: 0,
        next_retry_at: None,
        scheduled_at: None,
        blocked_reason: None,
        awaiting_approval_id: None,
        policy_fingerprint: None,
        approval_expires_at: None,
        containment_scope: None,
        compensation_status: None,
        compensation_summary: None,
        lane_id: None,
        last_error: None,
        logs: Vec::new(),
        tool_whitelist: None,
        tool_blacklist: None,
        context_budget_tokens: None,
        context_overflow_action: None,
        termination_conditions: None,
        success_criteria: None,
        max_duration_secs: None,
        supervisor_config: None,
        override_provider: None,
        override_model: None,
        override_system_prompt: None,
        sub_agent_def_id: None,
    }
}

fn sample_session(session_id: &str, workspace_id: &str) -> amux_protocol::SessionInfo {
    amux_protocol::SessionInfo {
        id: uuid::Uuid::parse_str(session_id).expect("valid uuid"),
        title: Some("Agent lane".to_string()),
        cwd: Some("/tmp/repo".to_string()),
        cols: 120,
        rows: 40,
        created_at: 1,
        workspace_id: Some(workspace_id.to_string()),
        exit_code: None,
        is_alive: true,
        active_command: Some("cargo test".to_string()),
    }
}

pub(crate) async fn spawn_goal_recording_server(
    recorded_bodies: Arc<StdMutex<VecDeque<String>>>,
    assistant_content: String,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind goal recording server");
    let addr = listener.local_addr().expect("goal recording server addr");
    let response_json =
        serde_json::to_string(&assistant_content).expect("assistant content should serialize");

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let recorded_bodies = recorded_bodies.clone();
            let response_json = response_json.clone();
            tokio::spawn(async move {
                let mut buffer = vec![0u8; 65536];
                let read = socket
                    .read(&mut buffer)
                    .await
                    .expect("read goal recording request");
                let request = String::from_utf8_lossy(&buffer[..read]).to_string();
                let body = request
                    .split("\r\n\r\n")
                    .nth(1)
                    .unwrap_or_default()
                    .to_string();
                recorded_bodies
                    .lock()
                    .expect("lock goal request log")
                    .push_back(body);

                let response = format!(
                    concat!(
                        "HTTP/1.1 200 OK\r\n",
                        "content-type: text/event-stream\r\n",
                        "cache-control: no-cache\r\n",
                        "connection: close\r\n",
                        "\r\n",
                        "data: {{\"choices\":[{{\"delta\":{{\"content\":{}}}}}]}}\n\n",
                        "data: {{\"choices\":[{{\"delta\":{{}},\"finish_reason\":\"stop\"}}],\"usage\":{{\"prompt_tokens\":7,\"completion_tokens\":3}}}}\n\n",
                        "data: [DONE]\n\n"
                    ),
                    response_json
                );
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write goal recording response");
            });
        }
    });

    format!("http://{addr}/v1")
}

#[test]
fn collect_plan_issues_catches_empty_summary() {
    let plan = GoalPlanResponse {
        title: Some("Goal".to_string()),
        summary: String::new(),
        steps: vec![GoalPlanStepResponse {
            title: "Do it".to_string(),
            instructions: "Do it".to_string(),
            kind: GoalRunStepKind::Command,
            success_criteria: "Done".to_string(),
            session_id: None,
            llm_confidence: None,
            llm_confidence_rationale: None,
        }],
        rejected_alternatives: Vec::new(),
    };

    assert!(!collect_plan_issues(&plan).is_empty());
}

include!("tests/skill_mesh.rs");
include!("tests/skill_mesh_compiler.rs");

#[test]
fn retry_goal_run_step_resets_selected_step() {
    let mut goal_run = sample_goal_run();

    retry_goal_run_step(&mut goal_run, Some(1)).expect("retry should succeed");

    assert_eq!(goal_run.current_step_index, 1);
    assert_eq!(goal_run.status, GoalRunStatus::Running);
    assert!(goal_run.completed_at.is_none());
    assert!(goal_run.last_error.is_none());
    assert_eq!(goal_run.steps[1].status, GoalRunStepStatus::Pending);
    assert!(goal_run.steps[1].task_id.is_none());
    assert!(goal_run.generated_skill_path.is_some());
}

#[test]
fn rerun_goal_run_from_step_resets_following_steps_and_skill_output() {
    let mut goal_run = sample_goal_run();

    rerun_goal_run_from_step(&mut goal_run, Some(0)).expect("rerun should succeed");

    assert_eq!(goal_run.current_step_index, 0);
    assert_eq!(goal_run.status, GoalRunStatus::Running);
    assert!(goal_run.completed_at.is_none());
    assert!(goal_run.generated_skill_path.is_none());
    assert!(goal_run.reflection_summary.is_none());
    assert_eq!(goal_run.steps[0].status, GoalRunStepStatus::Pending);
    assert_eq!(goal_run.steps[1].status, GoalRunStepStatus::Pending);
}

#[test]
fn project_goal_run_snapshot_derives_metrics() {
    let goal_run = sample_goal_run();
    let tasks = vec![sample_task("task-b", "goal_test")];

    let projected = project_goal_run_snapshot(goal_run, &tasks, 100);

    assert_eq!(projected.current_step_title.as_deref(), Some("Fix"));
    assert_eq!(projected.child_task_count, 2);
    assert_eq!(projected.approval_count, 1);
    assert_eq!(projected.awaiting_approval_id.as_deref(), Some("apr-1"));
    assert_eq!(
        projected.failure_cause.as_deref(),
        Some("child task failed")
    );
    assert_eq!(projected.duration_ms, Some(60));
}

#[test]
fn project_task_runs_exposes_parent_runtime_workspace_and_classification() {
    let mut parent = sample_task("parent-task", "goal_test");
    parent.title = "Implement rust file patching".to_string();
    parent.description = "Update repo files and tests".to_string();
    parent.status = TaskStatus::InProgress;
    parent.source = "user".to_string();
    parent.session_id = Some("11111111-1111-1111-1111-111111111111".to_string());

    let mut child = sample_subagent("child-task", "parent-task", TaskStatus::Queued);
    child.session_id = Some("22222222-2222-2222-2222-222222222222".to_string());
    child.runtime = "hermes".to_string();

    let runs = project_task_runs(
        &[parent.clone(), child.clone()],
        &[
            sample_session("11111111-1111-1111-1111-111111111111", "workspace-parent"),
            sample_session("22222222-2222-2222-2222-222222222222", "workspace-child"),
        ],
    );

    let parent_run = runs
        .iter()
        .find(|run| run.id == parent.id)
        .expect("parent run projected");
    assert_eq!(parent_run.kind, AgentRunKind::Task);
    assert_eq!(parent_run.classification, "coding");
    assert_eq!(parent_run.runtime_status.kind, AgentRunRuntimeStatusKind::Running);
    assert_eq!(parent_run.workspace_id.as_deref(), Some("workspace-parent"));

    let child_run = runs
        .iter()
        .find(|run| run.id == child.id)
        .expect("child run projected");
    assert_eq!(child_run.kind, AgentRunKind::Subagent);
    assert_eq!(child_run.runtime, "hermes");
    assert_eq!(child_run.runtime_status.kind, AgentRunRuntimeStatusKind::Queued);
    assert_eq!(child_run.parent_run_id.as_deref(), Some("parent-task"));
    assert_eq!(
        child_run.parent_title.as_deref(),
        Some("Implement rust file patching")
    );
    assert_eq!(child_run.workspace_id.as_deref(), Some("workspace-child"));
}

#[test]
fn project_task_runs_normalizes_blocked_and_retry_runtime_states() {
    let mut awaiting_approval = sample_task("approval-task", "goal_test");
    awaiting_approval.status = TaskStatus::Blocked;
    awaiting_approval.blocked_reason = Some(
        "waiting for operator approval: orchestrator_policy_escalation".to_string(),
    );
    awaiting_approval.awaiting_approval_id = Some("approval-1".to_string());

    let mut waiting_for_dependencies = sample_task("deps-task", "goal_test");
    waiting_for_dependencies.status = TaskStatus::Blocked;
    waiting_for_dependencies.blocked_reason =
        Some("waiting for dependencies: task-a, task-b".to_string());
    waiting_for_dependencies.awaiting_approval_id = None;

    let mut waiting_for_subagents = sample_task("subagents-task", "goal_test");
    waiting_for_subagents.status = TaskStatus::Blocked;
    waiting_for_subagents.blocked_reason = Some("waiting for subagents: sub-1, sub-2".to_string());
    waiting_for_subagents.awaiting_approval_id = None;

    let mut scheduled = sample_task("scheduled-task", "goal_test");
    scheduled.status = TaskStatus::Blocked;
    scheduled.scheduled_at = Some(1_234_567);
    scheduled.blocked_reason = Some("scheduled for 1970-01-01T00:20:34Z".to_string());
    scheduled.awaiting_approval_id = None;

    let mut waiting_for_resources = sample_task("resource-task", "goal_test");
    waiting_for_resources.status = TaskStatus::Blocked;
    waiting_for_resources.blocked_reason = Some("waiting for workspace lock: repo-main".to_string());
    waiting_for_resources.awaiting_approval_id = None;

    let mut retrying = sample_task("retry-task", "goal_test");
    retrying.status = TaskStatus::FailedAnalyzing;
    retrying.blocked_reason = Some("temporary provider error".to_string());
    retrying.awaiting_approval_id = None;
    retrying.next_retry_at = Some(9_999);

    let runs = project_task_runs(
        &[
            awaiting_approval,
            waiting_for_dependencies,
            waiting_for_subagents,
            scheduled,
            waiting_for_resources,
            retrying,
        ],
        &[],
    );

    let by_id = runs
        .iter()
        .map(|run| (run.id.as_str(), &run.runtime_status))
        .collect::<std::collections::HashMap<_, _>>();

    let approval = by_id.get("approval-task").expect("approval runtime status");
    assert_eq!(approval.kind, AgentRunRuntimeStatusKind::AwaitingApproval);
    assert_eq!(approval.awaiting_approval_id.as_deref(), Some("approval-1"));
    assert_eq!(
        approval.reason.as_deref(),
        Some("waiting for operator approval: orchestrator_policy_escalation")
    );

    let deps = by_id.get("deps-task").expect("dependency runtime status");
    assert_eq!(deps.kind, AgentRunRuntimeStatusKind::WaitingForDependencies);
    assert_eq!(
        deps.reason.as_deref(),
        Some("waiting for dependencies: task-a, task-b")
    );

    let subagents = by_id.get("subagents-task").expect("subagent runtime status");
    assert_eq!(subagents.kind, AgentRunRuntimeStatusKind::WaitingForSubagents);

    let scheduled = by_id.get("scheduled-task").expect("scheduled runtime status");
    assert_eq!(scheduled.kind, AgentRunRuntimeStatusKind::Scheduled);
    assert_eq!(scheduled.scheduled_at, Some(1_234_567));

    let resources = by_id.get("resource-task").expect("resource runtime status");
    assert_eq!(resources.kind, AgentRunRuntimeStatusKind::WaitingForResources);

    let retrying = by_id.get("retry-task").expect("retry runtime status");
    assert_eq!(retrying.kind, AgentRunRuntimeStatusKind::Retrying);
    assert_eq!(retrying.next_retry_at, Some(9_999));
    assert_eq!(retrying.reason.as_deref(), Some("temporary provider error"));
}

#[test]
fn make_goal_run_event_with_todos_preserves_snapshot() {
    let event = make_goal_run_event_with_todos(
        "todo",
        "goal todo updated",
        None,
        Some(1),
        vec![TodoItem {
            id: "todo-1".to_string(),
            content: "Inspect failing test".to_string(),
            status: TodoStatus::InProgress,
            position: 0,
            step_index: Some(1),
            created_at: 10,
            updated_at: 20,
        }],
    );

    assert_eq!(event.phase, "todo");
    assert_eq!(event.step_index, Some(1));
    assert_eq!(event.todo_snapshot.len(), 1);
    assert_eq!(event.todo_snapshot[0].content, "Inspect failing test");
}

#[test]
fn planner_required_for_message_detects_multi_step_requests() {
    assert!(planner_required_for_message(
        "Investigate the failing tests, then update the parser, and finally rerun the suite."
    ));
    assert!(planner_required_for_message(
        "1. Inspect logs\n2. Find the bad config\n3. Patch it"
    ));
}

#[test]
fn planner_required_for_message_skips_simple_requests() {
    assert!(!planner_required_for_message(
        "What port is the daemon listening on?"
    ));
    assert!(!planner_required_for_message("Show me the last error."));
}

#[test]
fn refresh_task_queue_state_blocks_parent_while_subagents_are_active() {
    let mut tasks = VecDeque::from(vec![
        AgentTask {
            id: "parent".to_string(),
            title: "Parent".to_string(),
            description: "Parent".to_string(),
            status: TaskStatus::Queued,
            priority: TaskPriority::Normal,
            progress: 10,
            created_at: 1,
            started_at: None,
            completed_at: None,
            error: None,
            result: None,
            thread_id: None,
            source: "agent".to_string(),
            notify_on_complete: false,
            notify_channels: Vec::new(),
            dependencies: Vec::new(),
            command: None,
            session_id: None,
            goal_run_id: None,
            goal_run_title: None,
            goal_step_id: None,
            goal_step_title: None,
            parent_task_id: None,
            parent_thread_id: None,
            runtime: "daemon".to_string(),
            retry_count: 0,
            max_retries: 0,
            next_retry_at: None,
            scheduled_at: None,
            blocked_reason: None,
            awaiting_approval_id: None,
            policy_fingerprint: None,
            approval_expires_at: None,
            containment_scope: None,
            compensation_status: None,
            compensation_summary: None,
            lane_id: None,
            last_error: None,
            logs: Vec::new(),
            tool_whitelist: None,
            tool_blacklist: None,
            context_budget_tokens: None,
            context_overflow_action: None,
            termination_conditions: None,
            success_criteria: None,
            max_duration_secs: None,
            supervisor_config: None,
            override_provider: None,
            override_model: None,
            override_system_prompt: None,
            sub_agent_def_id: None,
        },
        sample_subagent("sub-1", "parent", TaskStatus::InProgress),
    ]);

    let changed = refresh_task_queue_state(&mut tasks, 100, &[], &AgentConfig::default());
    let parent = tasks.iter().find(|task| task.id == "parent").unwrap();

    assert_eq!(parent.status, TaskStatus::Blocked);
    assert!(parent
        .blocked_reason
        .as_deref()
        .unwrap_or_default()
        .contains("waiting for subagents"));
    assert_eq!(changed.len(), 1);
}

#[test]
fn refresh_task_queue_state_requeues_parent_after_subagents_finish() {
    let mut tasks = VecDeque::from(vec![
        AgentTask {
            id: "parent".to_string(),
            title: "Parent".to_string(),
            description: "Parent".to_string(),
            status: TaskStatus::Blocked,
            priority: TaskPriority::Normal,
            progress: 90,
            created_at: 1,
            started_at: None,
            completed_at: None,
            error: None,
            result: None,
            thread_id: None,
            source: "agent".to_string(),
            notify_on_complete: false,
            notify_channels: Vec::new(),
            dependencies: Vec::new(),
            command: None,
            session_id: None,
            goal_run_id: None,
            goal_run_title: None,
            goal_step_id: None,
            goal_step_title: None,
            parent_task_id: None,
            parent_thread_id: None,
            runtime: "daemon".to_string(),
            retry_count: 0,
            max_retries: 0,
            next_retry_at: None,
            scheduled_at: None,
            blocked_reason: Some("waiting for subagents: sub-1".to_string()),
            awaiting_approval_id: None,
            policy_fingerprint: None,
            approval_expires_at: None,
            containment_scope: None,
            compensation_status: None,
            compensation_summary: None,
            lane_id: None,
            last_error: None,
            logs: Vec::new(),
            tool_whitelist: None,
            tool_blacklist: None,
            context_budget_tokens: None,
            context_overflow_action: None,
            termination_conditions: None,
            success_criteria: None,
            max_duration_secs: None,
            supervisor_config: None,
            override_provider: None,
            override_model: None,
            override_system_prompt: None,
            sub_agent_def_id: None,
        },
        sample_subagent("sub-1", "parent", TaskStatus::Completed),
    ]);

    let changed = refresh_task_queue_state(&mut tasks, 100, &[], &AgentConfig::default());
    let parent = tasks.iter().find(|task| task.id == "parent").unwrap();

    assert_eq!(parent.status, TaskStatus::Queued);
    assert!(parent.blocked_reason.is_none());
    assert_eq!(changed.len(), 1);
}

#[tokio::test]
async fn request_goal_replan_includes_recovery_guidance_when_present() {
    let recorded_bodies = Arc::new(StdMutex::new(VecDeque::new()));
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.provider = "openai".to_string();
    config.base_url = spawn_goal_recording_server(
        recorded_bodies.clone(),
        serde_json::json!({
            "title": "Revised plan",
            "summary": "Retry with a narrower fix path",
            "steps": [
                {
                    "title": "Retry step",
                    "instructions": "Apply the smaller repair.",
                    "kind": "command",
                    "success_criteria": "command succeeds",
                    "session_id": null,
                    "llm_confidence": "likely",
                    "llm_confidence_rationale": "Similar fixes recovered recently"
                }
            ],
            "rejected_alternatives": ["Repeat the same failed command unchanged"]
        })
        .to_string(),
    )
    .await;
    config.model = "gpt-4o-mini".to_string();
    config.api_key = "test-key".to_string();
    config.api_transport = ApiTransport::ChatCompletions;
    config.auto_retry = false;
    config.max_retries = 0;
    config.max_tool_loops = 1;
    let engine = AgentEngine::new_test(manager, config, root.path()).await;

    engine
        .history
        .insert_causal_trace(
            "causal_replan_guidance",
            Some("thread-replan-guidance"),
            None,
            None,
            "recovery",
            &serde_json::json!({
                "option_type": "replan_after_failure",
                "reasoning": "Recovered from a previous failed step.",
                "rejection_reason": null,
                "estimated_success_prob": 0.71,
                "arguments_hash": "ctx_hash"
            })
            .to_string(),
            "[]",
            "ctx_hash",
            "[]",
            &serde_json::to_string(
                &crate::agent::learning::traces::CausalTraceOutcome::NearMiss {
                    what_went_wrong: "step failed due to over-broad command".to_string(),
                    how_recovered: "replanned into smaller scoped steps".to_string(),
                },
            )
            .expect("serialize outcome"),
            Some("gpt-4o-mini"),
            now_millis(),
        )
        .await
        .expect("insert replan guidance trace");

    let mut goal_run = sample_goal_run();
    goal_run.thread_id = Some("thread-replan-guidance".to_string());

    let _ = engine
        .request_goal_replan(&goal_run, "managed command failed permanently")
        .await
        .expect("replan should succeed");

    let recorded = recorded_bodies.lock().expect("lock recorded bodies");
    let body = recorded.back().expect("expected one recorded request body");
    assert!(
        body.contains("Recent Causal Guidance"),
        "expected replan prompt to include the recent causal guidance block"
    );
    assert!(
        body.contains("replanned into smaller scoped steps"),
        "expected recovery guidance text in the replan prompt"
    );
}

#[tokio::test]
async fn request_goal_plan_adapts_prompt_and_truncates_output_when_satisfaction_is_strained() {
    let recorded_bodies = Arc::new(StdMutex::new(VecDeque::new()));
    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.provider = "openai".to_string();
    config.base_url = spawn_goal_recording_server(
        recorded_bodies.clone(),
        serde_json::json!({
            "title": "Compressed plan",
            "summary": "Take the direct path",
            "steps": [
                {
                    "title": "Step 1",
                    "instructions": "Do first thing.",
                    "kind": "command",
                    "success_criteria": "first done",
                    "session_id": null,
                    "llm_confidence": "likely",
                    "llm_confidence_rationale": "known-good path"
                },
                {
                    "title": "Step 2",
                    "instructions": "Do second thing.",
                    "kind": "command",
                    "success_criteria": "second done",
                    "session_id": null,
                    "llm_confidence": "likely",
                    "llm_confidence_rationale": "known-good path"
                },
                {
                    "title": "Step 3",
                    "instructions": "Do third thing.",
                    "kind": "command",
                    "success_criteria": "third done",
                    "session_id": null,
                    "llm_confidence": "likely",
                    "llm_confidence_rationale": "known-good path"
                },
                {
                    "title": "Step 4",
                    "instructions": "Do fourth thing.",
                    "kind": "command",
                    "success_criteria": "fourth done",
                    "session_id": null,
                    "llm_confidence": "likely",
                    "llm_confidence_rationale": "known-good path"
                },
                {
                    "title": "Step 5",
                    "instructions": "Do fifth thing.",
                    "kind": "command",
                    "success_criteria": "fifth done",
                    "session_id": null,
                    "llm_confidence": "likely",
                    "llm_confidence_rationale": "known-good path"
                },
                {
                    "title": "Step 6",
                    "instructions": "Do sixth thing.",
                    "kind": "command",
                    "success_criteria": "sixth done",
                    "session_id": null,
                    "llm_confidence": "likely",
                    "llm_confidence_rationale": "known-good path"
                }
            ],
            "rejected_alternatives": [
                "Alternative A: too broad",
                "Alternative B: too slow",
                "Alternative C: too risky"
            ]
        })
        .to_string(),
    )
    .await;
    config.model = "gpt-4o-mini".to_string();
    config.api_key = "test-key".to_string();
    config.api_transport = ApiTransport::ChatCompletions;
    config.auto_retry = false;
    config.max_retries = 0;
    config.max_tool_loops = 1;
    let engine = AgentEngine::new_test(manager, config, root.path()).await;

    {
        let mut model = engine.operator_model.write().await;
        model.cognitive_style.message_count = 1;
        model.operator_satisfaction.score = 0.18;
        model.operator_satisfaction.label = "strained".to_string();
        model.implicit_feedback.top_tool_fallbacks = vec![
            "read_file -> search_files".to_string(),
            "bash_command -> read_file".to_string(),
            "search_files -> search_files".to_string(),
        ];
    }

    let mut goal_run = sample_goal_run();
    goal_run.status = GoalRunStatus::Running;
    goal_run.goal = "Finish the implementation with minimal churn".to_string();

    let plan = engine
        .request_goal_plan(&goal_run)
        .await
        .expect("goal plan should succeed");

    assert_eq!(plan.steps.len(), 4);
    assert_eq!(plan.rejected_alternatives.len(), 1);

    let recorded = recorded_bodies.lock().expect("lock recorded bodies");
    let body = recorded.back().expect("expected one recorded request body");
    assert!(
        body.contains("Operator satisfaction is strained"),
        "expected strained satisfaction guidance in the plan prompt"
    );
    assert!(
        body.contains("Prefer the shortest viable plan"),
        "expected direct-path guidance in the plan prompt"
    );
    assert!(
        body.contains("Prefer them earlier when they fit"),
        "expected fallback guidance in the plan prompt"
    );
    assert!(
        body.contains("search_files, read_file"),
        "expected deduplicated fallback targets in the plan prompt"
    );
    assert!(
        body.contains("Use kind=debate when a step needs structured resolution of tradeoffs"),
        "expected debate guidance in the plan prompt"
    );
    assert!(
        body.contains("Reserve kind=divergent for broader multi-perspective exploration"),
        "expected divergent guidance in the plan prompt"
    );
}

#[test]
fn behavior_adaptation_profile_requests_clarification_when_strained_and_corrections_are_present() {
    let mut model = OperatorModel::default();
    model.cognitive_style.message_count = 1;
    model.operator_satisfaction.label = "strained".to_string();
    model.operator_satisfaction.score = 0.18;
    model.implicit_feedback.correction_message_count = 1;
    model.implicit_feedback.top_tool_fallbacks = vec![
        "read_file -> search_files".to_string(),
        "bash_command -> read_file".to_string(),
    ];

    let profile = BehaviorAdaptationProfile::from_model(&model);
    assert_eq!(profile.mode, SatisfactionAdaptationMode::Minimal);
    assert!(profile.prompt_for_clarification);
    assert!(profile.compact_response);
    assert_eq!(
        profile.preferred_tool_fallbacks,
        vec!["search_files".to_string(), "read_file".to_string()]
    );
}

#[test]
fn behavior_adaptation_profile_tightens_clarification_and_compactness_when_fragile_feedback_appears(
) {
    let mut model = OperatorModel::default();
    model.cognitive_style.message_count = 1;
    model.operator_satisfaction.label = "fragile".to_string();
    model.operator_satisfaction.score = 0.54;
    model.implicit_feedback.correction_message_count = 1;

    let profile = BehaviorAdaptationProfile::from_model(&model);
    assert_eq!(profile.mode, SatisfactionAdaptationMode::Tightened);
    assert!(profile.prompt_for_clarification);
    assert!(profile.compact_response);
}

#[test]
fn behavior_adaptation_profile_requests_clarification_when_fragile_feedback_includes_rapid_revert()
{
    let mut model = OperatorModel::default();
    model.cognitive_style.message_count = 1;
    model.operator_satisfaction.label = "fragile".to_string();
    model.operator_satisfaction.score = 0.49;
    model.implicit_feedback.rapid_revert_count = 1;

    let profile = BehaviorAdaptationProfile::from_model(&model);
    assert_eq!(profile.mode, SatisfactionAdaptationMode::Tightened);
    assert!(
        profile.prompt_for_clarification,
        "fragile rapid-revert feedback should trigger targeted clarification"
    );
    assert!(profile.compact_response);
}

#[tokio::test]
async fn request_goal_plan_injects_targeted_clarification_guidance_when_behavior_adapter_requires_it(
) {
    let recorded_bodies = Arc::new(StdMutex::new(VecDeque::new()));
    let plan_payload = serde_json::json!({
        "title": "Execution plan",
        "summary": "Use the normal execution mode.",
        "steps": [
            {
                "title": "Step 1",
                "instructions": "Inspect the current state.",
                "kind": "research",
                "success_criteria": "current state understood",
                "session_id": null,
                "llm_confidence": "likely",
                "llm_confidence_rationale": "small inspection"
            },
            {
                "title": "Step 2",
                "instructions": "Apply the smallest viable fix.",
                "kind": "command",
                "success_criteria": "fix applied",
                "session_id": null,
                "llm_confidence": "likely",
                "llm_confidence_rationale": "direct change"
            }
        ],
        "rejected_alternatives": ["Alternative A: broader but slower"]
    })
    .to_string();

    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.provider = "openai".to_string();
    config.base_url = spawn_goal_recording_server(recorded_bodies.clone(), plan_payload).await;
    config.model = "gpt-4o-mini".to_string();
    config.api_key = "test-key".to_string();
    config.api_transport = ApiTransport::ChatCompletions;
    config.auto_retry = false;
    config.max_retries = 0;
    config.max_tool_loops = 1;
    let engine = AgentEngine::new_test(manager, config, root.path()).await;

    {
        let mut model = engine.operator_model.write().await;
        model.cognitive_style.message_count = 1;
        model.operator_satisfaction.score = 0.18;
        model.operator_satisfaction.label = "strained".to_string();
        model.implicit_feedback.correction_message_count = 1;
        model.implicit_feedback.top_tool_fallbacks = vec![
            "read_file -> search_files".to_string(),
            "bash_command -> read_file".to_string(),
        ];
    }

    let _plan = engine
        .request_goal_plan(&sample_goal_run())
        .await
        .expect("goal plan should succeed");

    let recorded = recorded_bodies.lock().expect("lock recorded bodies");
    let body = recorded.back().expect("expected one recorded request body");
    assert!(
        body.contains("targeted clarification checkpoint"),
        "expected explicit clarification guidance in the plan prompt"
    );
}

#[tokio::test]
async fn request_goal_plan_adapts_execution_recommendation_when_satisfaction_is_strained() {
    let plan_payload = serde_json::json!({
        "title": "Execution plan",
        "summary": "Use the normal execution mode.",
        "steps": [
            {
                "title": "Step 1",
                "instructions": "Inspect the current state.",
                "kind": "research",
                "success_criteria": "current state understood",
                "session_id": null,
                "llm_confidence": "likely",
                "llm_confidence_rationale": "small inspection"
            },
            {
                "title": "Step 2",
                "instructions": "Apply the smallest viable fix.",
                "kind": "command",
                "success_criteria": "fix applied",
                "session_id": null,
                "llm_confidence": "likely",
                "llm_confidence_rationale": "direct change"
            }
        ],
        "rejected_alternatives": ["Alternative A: broader but slower"]
    })
    .to_string();

    let root_normal = tempdir().expect("tempdir");
    let manager_normal = SessionManager::new_test(root_normal.path()).await;
    let mut config_normal = AgentConfig::default();
    config_normal.provider = "openai".to_string();
    config_normal.base_url = spawn_goal_recording_server(
        Arc::new(StdMutex::new(VecDeque::new())),
        plan_payload.clone(),
    )
    .await;
    config_normal.model = "gpt-4o-mini".to_string();
    config_normal.api_key = "test-key".to_string();
    config_normal.api_transport = ApiTransport::ChatCompletions;
    config_normal.auto_retry = false;
    config_normal.max_retries = 0;
    config_normal.max_tool_loops = 1;
    let engine_normal =
        AgentEngine::new_test(manager_normal, config_normal, root_normal.path()).await;

    {
        let mut model = engine_normal.operator_model.write().await;
        model.cognitive_style.message_count = 1;
        model.operator_satisfaction.score = 0.72;
        model.operator_satisfaction.label = "healthy".to_string();
    }

    let normal_plan = engine_normal
        .request_goal_plan(&sample_goal_run())
        .await
        .expect("normal goal plan should succeed");

    let root_strained = tempdir().expect("tempdir");
    let manager_strained = SessionManager::new_test(root_strained.path()).await;
    let mut config_strained = AgentConfig::default();
    config_strained.provider = "openai".to_string();
    config_strained.base_url =
        spawn_goal_recording_server(Arc::new(StdMutex::new(VecDeque::new())), plan_payload).await;
    config_strained.model = "gpt-4o-mini".to_string();
    config_strained.api_key = "test-key".to_string();
    config_strained.api_transport = ApiTransport::ChatCompletions;
    config_strained.auto_retry = false;
    config_strained.max_retries = 0;
    config_strained.max_tool_loops = 1;
    let engine_strained =
        AgentEngine::new_test(manager_strained, config_strained, root_strained.path()).await;

    {
        let mut model = engine_strained.operator_model.write().await;
        model.cognitive_style.message_count = 1;
        model.operator_satisfaction.score = 0.18;
        model.operator_satisfaction.label = "strained".to_string();
    }

    let strained_plan = engine_strained
        .request_goal_plan(&sample_goal_run())
        .await
        .expect("strained goal plan should succeed");

    assert_eq!(normal_plan.summary, "Use the normal execution mode.");
    assert_ne!(strained_plan.summary, normal_plan.summary);
    assert!(strained_plan
        .summary
        .contains("Conservative execution mode:"));
    assert!(strained_plan.summary.contains("prefer proven tools"));
    assert!(strained_plan
        .summary
        .contains("keep iteration bounds short"));
}

#[tokio::test]
async fn request_goal_plan_injects_compact_delivery_guidance_when_behavior_adapter_requires_it() {
    let recorded_bodies = Arc::new(StdMutex::new(VecDeque::new()));
    let plan_payload = serde_json::json!({
        "title": "Execution plan",
        "summary": "Use the normal execution mode.",
        "steps": [
            {
                "title": "Step 1",
                "instructions": "Inspect the current state.",
                "kind": "research",
                "success_criteria": "current state understood",
                "session_id": null,
                "llm_confidence": "likely",
                "llm_confidence_rationale": "small inspection"
            },
            {
                "title": "Step 2",
                "instructions": "Apply the smallest viable fix.",
                "kind": "command",
                "success_criteria": "fix applied",
                "session_id": null,
                "llm_confidence": "likely",
                "llm_confidence_rationale": "direct change"
            }
        ],
        "rejected_alternatives": ["Alternative A: broader but slower"]
    })
    .to_string();

    let root = tempdir().expect("tempdir");
    let manager = SessionManager::new_test(root.path()).await;
    let mut config = AgentConfig::default();
    config.provider = "openai".to_string();
    config.base_url = spawn_goal_recording_server(recorded_bodies.clone(), plan_payload).await;
    config.model = "gpt-4o-mini".to_string();
    config.api_key = "test-key".to_string();
    config.api_transport = ApiTransport::ChatCompletions;
    config.auto_retry = false;
    config.max_retries = 0;
    config.max_tool_loops = 1;
    let engine = AgentEngine::new_test(manager, config, root.path()).await;

    {
        let mut model = engine.operator_model.write().await;
        model.cognitive_style.message_count = 1;
        model.operator_satisfaction.score = 0.54;
        model.operator_satisfaction.label = "fragile".to_string();
        model.implicit_feedback.correction_message_count = 1;
    }

    let _plan = engine
        .request_goal_plan(&sample_goal_run())
        .await
        .expect("goal plan should succeed");

    let recorded = recorded_bodies.lock().expect("lock recorded bodies");
    let body = recorded.back().expect("expected one recorded request body");
    assert!(
        body.contains("Keep the plan summary and step instructions compact"),
        "expected compact delivery guidance in the plan prompt"
    );
    assert!(
        body.contains("targeted clarification checkpoint"),
        "expected fragile feedback to raise clarification guidance before broad guessing"
    );
}
