import { getBridge } from "./bridge";
import { formatTaskStatus, formatTaskTimestamp, isTaskActive, isTaskTerminal, taskStatusColor, type AgentTaskPriority, type AgentTaskStatus } from "./agentTaskQueue";
import type { StatusChipTone } from "./statusChips";

export type RunStatusTone = StatusChipTone;

export type RunStatusChip = {
    icon: string;
    label: string;
    tone: RunStatusTone;
};

export type RunStatusReasonChip = {
    icon: string;
    label: string;
    tone: RunStatusTone;
};

export type AgentRunKind = "task" | "subagent";
export type AgentRunClassification = "coding" | "research" | "ops" | "browser" | "messaging" | "mixed" | string;
export type AgentRunRuntimeStatusKind =
    | "queued"
    | "running"
    | "awaiting_approval"
    | "waiting_for_dependencies"
    | "waiting_for_subagents"
    | "scheduled"
    | "waiting_for_resources"
    | "blocked"
    | "retrying"
    | "failed_analyzing"
    | "budget_exceeded"
    | "completed"
    | "failed"
    | "cancelled";

export interface AgentRunRuntimeStatus {
    kind: AgentRunRuntimeStatusKind;
    reason?: string | null;
    awaiting_approval_id?: string | null;
    next_retry_at?: number | null;
    scheduled_at?: number | null;
}

export interface AgentRun {
    id: string;
    task_id: string;
    kind: AgentRunKind;
    classification: AgentRunClassification;
    title: string;
    description: string;
    status: AgentTaskStatus;
    runtime_status?: AgentRunRuntimeStatus | null;
    priority: AgentTaskPriority;
    progress: number;
    created_at: number;
    started_at?: number | null;
    completed_at?: number | null;
    thread_id?: string | null;
    session_id?: string | null;
    workspace_id?: string | null;
    source: string;
    runtime?: string | null;
    goal_run_id?: string | null;
    goal_run_title?: string | null;
    goal_step_id?: string | null;
    goal_step_title?: string | null;
    parent_run_id?: string | null;
    parent_task_id?: string | null;
    parent_thread_id?: string | null;
    parent_title?: string | null;
    blocked_reason?: string | null;
    error?: string | null;
    result?: string | null;
    last_error?: string | null;
}

export async function fetchAgentRuns(): Promise<AgentRun[]> {
    const amux = getBridge();
    if (!amux?.agentListRuns) {
        return [];
    }

    try {
        const result = await amux.agentListRuns();
        return Array.isArray(result) ? (result as AgentRun[]) : [];
    } catch {
        return [];
    }
}

export function isRunTerminal(run: AgentRun): boolean {
    const runtimeKind = run.runtime_status?.kind;
    if (runtimeKind) {
        return runtimeKind === "completed" || runtimeKind === "failed" || runtimeKind === "cancelled";
    }
    return isTaskTerminal(run);
}

export function isRunActive(run: AgentRun): boolean {
    const runtimeKind = run.runtime_status?.kind;
    if (runtimeKind) {
        return !(runtimeKind === "completed" || runtimeKind === "failed" || runtimeKind === "cancelled");
    }
    return isTaskActive(run);
}

export function isSubagentRun(run: AgentRun): boolean {
    return run.kind === "subagent" || Boolean(run.parent_run_id || run.parent_task_id || run.parent_thread_id);
}

export function formatRunStatus(run: AgentRun): string {
    switch (run.runtime_status?.kind) {
        case "running":
            return "Running";
        case "awaiting_approval":
            return "Awaiting approval";
        case "waiting_for_dependencies":
            return "Waiting for dependencies";
        case "waiting_for_subagents":
            return "Waiting for subagents";
        case "scheduled":
            return "Scheduled";
        case "waiting_for_resources":
            return "Waiting for resources";
        case "retrying":
            return "Retrying";
        case "failed_analyzing":
            return "Analyzing failure";
        case "budget_exceeded":
            return "Budget exceeded";
        case "completed":
            return "Completed";
        case "failed":
            return "Failed";
        case "cancelled":
            return "Cancelled";
        case "queued":
            return "Queued";
        case "blocked":
            return "Blocked";
        default:
            return formatTaskStatus(run);
    }
}

export function getRunStatusChip(run: AgentRun): RunStatusChip {
    switch (run.runtime_status?.kind) {
        case "running":
            return { icon: "●", label: "Running", tone: "accent" };
        case "awaiting_approval":
            return { icon: "⚑", label: "Approval", tone: "approval" };
        case "waiting_for_dependencies":
            return { icon: "⇢", label: "Deps", tone: "neutral" };
        case "waiting_for_subagents":
            return { icon: "◌", label: "Children", tone: "neutral" };
        case "scheduled":
            return { icon: "◷", label: "Scheduled", tone: "neutral" };
        case "waiting_for_resources":
            return { icon: "⌛", label: "Resources", tone: "neutral" };
        case "retrying":
            return { icon: "↻", label: "Retrying", tone: "warning" };
        case "failed_analyzing":
            return { icon: "△", label: "Analyzing", tone: "warning" };
        case "budget_exceeded":
            return { icon: "⚠", label: "Budget", tone: "warning" };
        case "completed":
            return { icon: "✓", label: "Done", tone: "success" };
        case "failed":
            return { icon: "✕", label: "Failed", tone: "danger" };
        case "cancelled":
            return { icon: "—", label: "Cancelled", tone: "neutral" };
        case "queued":
            return { icon: "○", label: "Queued", tone: "neutral" };
        case "blocked":
            return { icon: "⏸", label: "Blocked", tone: "neutral" };
        default:
            return {
                icon: "○",
                label: formatRunStatus(run),
                tone: "neutral",
            };
    }
}

export function runStatusColor(run: AgentRun): string {
    switch (run.runtime_status?.kind) {
        case "running":
            return "var(--accent)";
        case "awaiting_approval":
            return "var(--approval)";
        case "waiting_for_dependencies":
        case "waiting_for_subagents":
        case "scheduled":
        case "waiting_for_resources":
        case "blocked":
            return "var(--text-muted)";
        case "retrying":
        case "failed_analyzing":
        case "budget_exceeded":
            return "var(--warning)";
        case "completed":
            return "var(--success)";
        case "failed":
            return "var(--danger)";
        case "cancelled":
            return "var(--text-muted)";
        case "queued":
            return "var(--text-secondary)";
        default:
            return taskStatusColor(run.status);
    }
}

export function getRunStatusReason(run: AgentRun): string | null {
    const reason = run.runtime_status?.reason ?? run.blocked_reason ?? null;
    if (typeof reason !== "string") {
        return null;
    }
    const trimmed = reason.trim();
    if (trimmed.length === 0) {
        return null;
    }

    return shortenRunStatusReason(trimmed);
}

export function getRunStatusReasonChip(run: AgentRun): RunStatusReasonChip | null {
    switch (run.runtime_status?.kind) {
        case "awaiting_approval":
            return { icon: "⚑", label: "Approval", tone: "approval" };
        case "waiting_for_dependencies":
            return { icon: "⇢", label: "Depends on", tone: "neutral" };
        case "waiting_for_subagents":
            return { icon: "◌", label: "Waiting on", tone: "neutral" };
        case "scheduled":
            return { icon: "◷", label: "At", tone: "neutral" };
        case "waiting_for_resources":
            return { icon: "⌛", label: "Resource", tone: "neutral" };
        case "retrying":
            return { icon: "↻", label: "Retry", tone: "warning" };
        case "failed_analyzing":
            return { icon: "△", label: "Analysis", tone: "warning" };
        case "budget_exceeded":
            return { icon: "⚠", label: "Budget", tone: "warning" };
        case "blocked":
            return { icon: "⏸", label: "Blocked", tone: "neutral" };
        default:
            return null;
    }
}

function shortenRunStatusReason(reason: string): string {
    const prefixes: Array<[string, string]> = [
        ["waiting for dependencies:", "Dependencies: "],
        ["waiting for subagents:", "Subagents: "],
        ["waiting for workspace lock:", "Workspace: "],
        ["waiting for lane availability:", "Lane: "],
        ["waiting for subagent slot:", "Slot: "],
        ["waiting for operator approval:", "Approval: "],
        ["scheduled for ", "At "],
    ];

    const lower = reason.toLowerCase();
    for (const [prefix, replacement] of prefixes) {
        if (lower.startsWith(prefix)) {
            return `${replacement}${reason.slice(prefix.length).trim()}`;
        }
    }

    return reason;
}

export function formatRunTimestamp(timestamp?: number | null): string {
    return formatTaskTimestamp(timestamp);
}
