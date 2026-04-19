import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import type { AgentRun } from "@/lib/agentRuns";
import type { SpawnedAgentTree } from "@/lib/spawnedAgentTree";
import { SpawnedAgentsPanel } from "./SpawnedAgentsPanel";

function makeRun(overrides: Partial<AgentRun> = {}): AgentRun {
  return {
    id: "run-root",
    task_id: "task-root",
    kind: "subagent",
    classification: "coding",
    title: "Root Agent",
    description: "Inspect the spawned agent",
    status: "in_progress",
    runtime_status: {
      kind: "running",
    },
    priority: "normal",
    progress: 50,
    created_at: 1,
    source: "daemon",
    runtime: "claude-code",
    ...overrides,
  };
}

describe("SpawnedAgentsPanel", () => {
  it("renders nested spawned-agent nodes, runtime/session hints, and disables unopened chat actions", () => {
    const anchor = makeRun({
      id: "run-anchor",
      task_id: "task-anchor",
      title: "Root Agent",
      thread_id: "daemon-root",
      session_id: "session-root",
    });
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "daemon-child",
      session_id: "session-child",
      parent_task_id: "task-anchor",
      parent_thread_id: "daemon-root",
      created_at: 2,
    });
    const waitingChild = makeRun({
      id: "run-waiting",
      task_id: "task-waiting",
      title: "Waiting Child",
      thread_id: null,
      session_id: "session-waiting",
      status: "queued",
      runtime_status: {
        kind: "waiting_for_dependencies",
        reason: "waiting for dependencies: task-anchor",
      },
      parent_task_id: "task-anchor",
      parent_thread_id: "daemon-root",
      created_at: 3,
    });
    const tree: SpawnedAgentTree<AgentRun> = {
      activeThreadId: "daemon-root",
      anchor: {
        item: anchor,
        openable: true,
        live: true,
        children: [
          {
            item: child,
            openable: true,
            live: true,
            children: [],
          },
          {
            item: waitingChild,
            openable: false,
            live: true,
            children: [],
          },
        ],
      },
      roots: [],
    };

    const html = renderToStaticMarkup(
      <SpawnedAgentsPanel
        tree={tree}
        selectedDaemonThreadId="daemon-root"
        canGoBackThread={true}
        threadNavigationDepth={2}
        backThreadTitle="Parent Thread"
        canOpenSpawnedThread={(run) => Boolean(run.thread_id)}
        openSpawnedThread={vi.fn(async () => true)}
        goBackThread={vi.fn()}
      />,
    );

    expect(html).toContain("Spawned Agents");
    expect(html).toContain("Back to Parent Thread");
    expect(html).toContain("2 hop history");
    expect(html).toContain("Root Agent");
    expect(html).toContain("Spawned Child");
    expect(html).toContain("Waiting Child");
    expect(html).toContain("Waiting for dependencies");
    expect(html).toContain("waiting for dependencies: task-anchor");
    expect(html).toContain("claude-code");
    expect(html).toContain("session-child");
    expect(html).toContain('aria-label="Open chat for Waiting Child"');
    expect(html).toContain('disabled=""');
  });

  it("renders an empty state when there is no spawned-agent tree for the active thread", () => {
    const html = renderToStaticMarkup(
      <SpawnedAgentsPanel
        tree={null}
        selectedDaemonThreadId={null}
        canGoBackThread={false}
        threadNavigationDepth={0}
        backThreadTitle={null}
        canOpenSpawnedThread={() => false}
        openSpawnedThread={vi.fn(async () => false)}
        goBackThread={vi.fn()}
      />,
    );

    expect(html).toContain("Spawned Agents");
    expect(html).toContain("No spawned agents for this thread yet.");
    expect(html).toContain('disabled=""');
  });
});
