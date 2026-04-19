import { describe, expect, it } from "vitest";
import { formatRunStatus, getRunStatusChip, getRunStatusReason, getRunStatusReasonChip, isRunActive, isRunTerminal, runStatusColor, type AgentRun } from "./agentRuns";

function makeRun(overrides: Partial<AgentRun> = {}): AgentRun {
  return {
    id: "run-1",
    task_id: "task-1",
    kind: "subagent",
    classification: "coding",
    title: "Run",
    description: "Inspect runtime status",
    status: "blocked",
    priority: "normal",
    progress: 10,
    created_at: 1,
    source: "daemon",
    ...overrides,
  };
}

describe("agentRuns runtime_status wiring", () => {
  it("formats normalized runtime statuses instead of coarse task status", () => {
    const run = makeRun({
      status: "blocked",
      runtime_status: {
        kind: "waiting_for_dependencies",
        reason: "waiting for dependencies: task-a",
      },
    });

    expect(formatRunStatus(run)).toBe("Waiting for dependencies");
    expect(runStatusColor(run)).toBe("var(--text-muted)");
    expect(isRunActive(run)).toBe(true);
    expect(isRunTerminal(run)).toBe(false);
  });

  it("treats retrying as active warning state", () => {
    const run = makeRun({
      status: "failed_analyzing",
      runtime_status: {
        kind: "retrying",
        next_retry_at: 123,
      },
    });

    expect(formatRunStatus(run)).toBe("Retrying");
    expect(runStatusColor(run)).toBe("var(--warning)");
    expect(isRunActive(run)).toBe(true);
  });

  it("treats normalized terminal states as terminal", () => {
    const run = makeRun({
      status: "in_progress",
      runtime_status: {
        kind: "completed",
      },
    });

    expect(formatRunStatus(run)).toBe("Completed");
    expect(runStatusColor(run)).toBe("var(--success)");
    expect(isRunTerminal(run)).toBe(true);
    expect(isRunActive(run)).toBe(false);
  });

  it("falls back to coarse task status when runtime_status is absent", () => {
    const run = makeRun({ status: "awaiting_approval", runtime_status: undefined });

    expect(formatRunStatus(run)).toBe("Awaiting approval");
    expect(runStatusColor(run)).toBe("var(--approval)");
  });

  it("surfaces runtime_status reason with blocked_reason fallback", () => {
    const runtimeReasonRun = makeRun({
      runtime_status: {
        kind: "waiting_for_resources",
        reason: "waiting for workspace lock: repo-main",
      },
      blocked_reason: "older blocked reason",
    });
    const fallbackRun = makeRun({
      runtime_status: undefined,
      blocked_reason: "waiting for dependencies: task-a",
    });
    const emptyRun = makeRun({
      runtime_status: {
        kind: "blocked",
        reason: "   ",
      },
      blocked_reason: "   ",
    });

    expect(getRunStatusReason(runtimeReasonRun)).toBe("Workspace: repo-main");
    expect(getRunStatusReason(fallbackRun)).toBe("Dependencies: task-a");
    expect(getRunStatusReason(emptyRun)).toBeNull();
  });

  it("returns specific status and reason chips for normalized runtime states", () => {
    const dependencyRun = makeRun({
      runtime_status: {
        kind: "waiting_for_dependencies",
        reason: "waiting for dependencies: task-a, task-b",
      },
    });
    const approvalRun = makeRun({
      runtime_status: {
        kind: "awaiting_approval",
        reason: "waiting for operator approval: review_command",
      },
    });

    expect(getRunStatusChip(dependencyRun)).toEqual({ icon: "⇢", label: "Deps", tone: "neutral" });
    expect(getRunStatusReasonChip(dependencyRun)).toEqual({ icon: "⇢", label: "Depends on" });
    expect(getRunStatusChip(approvalRun)).toEqual({ icon: "⚑", label: "Approval", tone: "approval" });
    expect(getRunStatusReasonChip(approvalRun)).toEqual({ icon: "⚑", label: "Approval" });
  });
});
