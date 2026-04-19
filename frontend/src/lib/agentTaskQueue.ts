import { getBridge } from "./bridge";
import type { StatusChipTone } from "./statusChips";

export type AgentTaskStatus =
    | "queued"
    | "in_progress"
    | "awaiting_approval"
    | "blocked"
    | "failed_analyzing"
    | "budget_exceeded"
    | "completed"
    | "failed"
    | "cancelled";

export type AgentTaskPriority = "low" | "normal" | "high" | "urgent";

export type TaskStatusTone = StatusChipTone;

export type TaskStatusChip = {
    icon: string;
    label: string;
    tone: TaskStatusTone;
};

export type TaskStatusReasonChip = {
    icon: string;
    label: string;
    tone: TaskStatusTone;
};

export type AgentTaskLogLevel = "info" | "warn" | "error";

export interface AgentTaskLogEntry {
    id: string;
    timestamp: number;
    level: AgentTaskLogLevel;
    phase: string;
    message: string;
    details?: string | null;
    attempt: number;
}

export interface AgentQueueTask {
    id: string;
    title: string;
    description: string;
    status: AgentTaskStatus;
    priority: AgentTaskPriority;
    progress: number;
    created_at: number;
    started_at?: number | null;
    completed_at?: number | null;
    error?: string | null;
    result?: string | null;
    thread_id?: string | null;
    source: string;
    notify_on_complete?: boolean;
    notify_channels?: string[];
    dependencies?: string[];
    command?: string | null;
    session_id?: string | null;
    goal_run_title?: string | null;
    goal_step_id?: string | null;
    goal_step_title?: string | null;
    parent_task_id?: string | null;
    parent_thread_id?: string | null;
    runtime?: string | null;
    retry_count?: number;
    max_retries?: number;
    next_retry_at?: number | null;
    scheduled_at?: number | null;
    blocked_reason?: string | null;
    awaiting_approval_id?: string | null;
    lane_id?: string | null;
    last_error?: string | null;
    logs?: AgentTaskLogEntry[];
}

export async function fetchAgentTasks(): Promise<AgentQueueTask[]> {
    const amux = getBridge();
    if (!amux?.agentListTasks) {
        return [];
    }

    try {
        const result = await amux.agentListTasks();
        return Array.isArray(result) ? (result as AgentQueueTask[]) : [];
    } catch {
        return [];
    }
}

export function isTaskTerminal(task: AgentQueueTask): boolean {
    return task.status === "completed" || task.status === "failed" || task.status === "cancelled";
}

export function isTaskActive(task: AgentQueueTask): boolean {
    return !isTaskTerminal(task);
}

export function isSubagentTask(task: AgentQueueTask): boolean {
    return Boolean(task.parent_task_id || task.parent_thread_id || task.source === "subagent");
}

export function formatTaskStatus(task: AgentQueueTask): string {
    switch (task.status) {
        case "in_progress":
            return "In progress";
        case "awaiting_approval":
            return "Awaiting approval";
        case "failed_analyzing":
            return "Analyzing failure";
        case "budget_exceeded":
            return "Budget exceeded";
        default:
            return task.status.replace(/_/g, " ");
    }
}

export function taskStatusColor(status: AgentTaskStatus): string {
    switch (status) {
        case "in_progress":
            return "var(--accent)";
        case "awaiting_approval":
            return "var(--approval)";
        case "blocked":
            return "var(--text-muted)";
        case "failed_analyzing":
            return "var(--warning)";
        case "budget_exceeded":
            return "var(--warning)";
        case "completed":
            return "var(--success)";
        case "failed":
            return "var(--danger)";
        case "cancelled":
            return "var(--text-muted)";
        default:
            return "var(--text-secondary)";
    }
}

export function getTaskStatusChip(task: AgentQueueTask): TaskStatusChip {
    switch (task.status) {
        case "in_progress":
            return { icon: "●", label: "Running", tone: "accent" };
        case "awaiting_approval":
            return { icon: "⚑", label: "Approval", tone: "approval" };
        case "blocked":
            return { icon: "⏸", label: "Blocked", tone: "neutral" };
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
        default:
            return { icon: "○", label: "Queued", tone: "neutral" };
    }
}

export function getTaskStatusReason(task: AgentQueueTask): string | null {
    const reason = task.blocked_reason ?? task.error ?? null;
    if (typeof reason !== "string") {
        return null;
    }

    const trimmed = reason.trim();
    if (trimmed.length === 0) {
        return null;
    }

    return shortenTaskStatusReason(trimmed);
}

export function getTaskStatusReasonChip(task: AgentQueueTask): TaskStatusReasonChip | null {
    if (!getTaskStatusReason(task)) {
        return null;
    }

    switch (task.status) {
        case "awaiting_approval":
            return { icon: "⚑", label: "Approval", tone: "approval" };
        case "blocked":
            return { icon: "⏸", label: "Blocked", tone: "neutral" };
        case "failed_analyzing":
            return { icon: "△", label: "Analysis", tone: "warning" };
        case "budget_exceeded":
            return { icon: "⚠", label: "Budget", tone: "warning" };
        case "failed":
            return { icon: "✕", label: "Error", tone: "danger" };
        default:
            return task.blocked_reason
                ? { icon: "⏸", label: "Gate", tone: "neutral" }
                : { icon: "✕", label: "Error", tone: "danger" };
    }
}

function shortenTaskStatusReason(reason: string): string {
    const prefixes: Array<[string, string]> = [
        ["waiting for dependencies:", "Dependencies: "],
        ["waiting for subagents:", "Subagents: "],
        ["waiting for workspace lock:", "Workspace: "],
        ["waiting for lane availability:", "Lane: "],
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

export function formatTaskTimestamp(timestamp?: number | null): string {
    if (!timestamp) {
        return "-";
    }
    return new Date(timestamp).toLocaleTimeString();
}
