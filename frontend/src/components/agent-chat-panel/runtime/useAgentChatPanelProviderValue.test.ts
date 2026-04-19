import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentRun } from "@/lib/agentRuns";
import { useAgentStore } from "@/lib/agentStore";
import type { AgentThread, AgentTodoItem } from "@/lib/agentStore";
import type { Workspace } from "@/lib/types";
import {
  deriveSpawnedAgentNavigationState,
  openSpawnedAgentThreadFromRun,
  resetPendingSpawnedThreadHydrationsForTest,
} from "./useAgentChatPanelProviderValue";

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

function makeThread(id: string, title: string, daemonThreadId: string | null): AgentThread {
  return {
    id,
    daemonThreadId,
    workspaceId: null,
    surfaceId: null,
    paneId: null,
    agent_name: "tamux",
    title,
    createdAt: 1,
    updatedAt: 1,
    messageCount: 0,
    totalInputTokens: 0,
    totalOutputTokens: 0,
    totalTokens: 0,
    compactionCount: 0,
    lastMessagePreview: "",
    upstreamThreadId: null,
    upstreamTransport: undefined,
    upstreamProvider: null,
    upstreamModel: null,
    upstreamAssistantId: null,
  };
}

function makeWorkspace(sessionId: string): Workspace {
  return {
    id: "workspace-1",
    name: "Workspace",
    icon: "terminal",
    accentColor: "#000",
    cwd: "/repo",
    gitBranch: null,
    gitDirty: false,
    listeningPorts: [],
    unreadCount: 0,
    activeSurfaceId: "surface-1",
    createdAt: 1,
    surfaces: [
      {
        id: "surface-1",
        workspaceId: "workspace-1",
        name: "Surface",
        icon: "terminal",
        layoutMode: "bsp",
        layout: {
          type: "leaf",
          id: "pane-1",
          sessionId,
        },
        paneNames: {},
        paneIcons: {},
        activePaneId: "pane-1",
        canvasState: {
          panX: 0,
          panY: 0,
          zoomLevel: 1,
          previousView: null,
        },
        canvasPanels: [
          {
            id: "panel-1",
            paneId: "pane-1",
            panelType: "terminal",
            title: "Terminal",
            icon: "terminal",
            x: 0,
            y: 0,
            width: 100,
            height: 100,
            status: "running",
            sessionId,
            url: null,
            cwd: "/repo",
            userRenamed: false,
            lastActivityAt: 1,
          },
        ],
        createdAt: 1,
      },
    ],
  };
}

describe("deriveSpawnedAgentNavigationState", () => {
  it("derives the visible tree and back-thread hint from active thread context", () => {
    const activeThread = makeThread("local-root", "Root Thread", "daemon-root");
    const previousThread = makeThread("local-parent", "Parent Thread", "daemon-parent");
    const state = deriveSpawnedAgentNavigationState({
      activeThread,
      threads: [activeThread, previousThread],
      threadHistoryStack: ["local-parent"],
      runs: [
        makeRun({
          id: "run-root",
          task_id: "task-root",
          thread_id: "daemon-root",
        }),
        makeRun({
          id: "run-child",
          task_id: "task-child",
          title: "Child Thread",
          thread_id: "daemon-child",
          parent_task_id: "task-root",
          parent_thread_id: "daemon-root",
          created_at: 2,
        }),
      ],
    });

    expect(state.tree?.anchor?.item.id).toBe("run-root");
    expect(state.tree?.anchor?.children[0]?.item.id).toBe("run-child");
    expect(state.canGoBackThread).toBe(true);
    expect(state.threadNavigationDepth).toBe(1);
    expect(state.backThreadTitle).toBe("Parent Thread");
  });
});

describe("openSpawnedAgentThreadFromRun", () => {
  function createLiveStoreWrappers() {
    const createThread = useAgentStore.getState().createThread;
    const addMessage = useAgentStore.getState().addMessage;
    const setThreadDaemonId = useAgentStore.getState().setThreadDaemonId;
    const setThreadTodos = useAgentStore.getState().setThreadTodos;
    const openSpawnedThread = useAgentStore.getState().openSpawnedThread;

    return {
      createThread: vi.fn((...args: Parameters<typeof createThread>) =>
        createThread(...args)),
      addMessage: vi.fn((...args: Parameters<typeof addMessage>) =>
        addMessage(...args)),
      setThreadDaemonId: vi.fn((...args: Parameters<typeof setThreadDaemonId>) =>
        setThreadDaemonId(...args)),
      setThreadTodos: vi.fn((...args: Parameters<typeof setThreadTodos>) =>
        setThreadTodos(...args)),
      openSpawnedThread: vi.fn((...args: Parameters<typeof openSpawnedThread>) =>
        openSpawnedThread(...args)),
    };
  }

  function findHydratedThread(daemonThreadId: string): AgentThread {
    const thread = useAgentStore.getState().threads.find((entry) => entry.daemonThreadId === daemonThreadId);
    expect(thread).toBeDefined();
    return thread!;
  }

  beforeEach(() => {
    resetPendingSpawnedThreadHydrationsForTest();
    const rootThread = makeThread("local-root", "Root Thread", "daemon-root");
    const parentThread = makeThread("local-parent", "Parent Thread", "daemon-parent");
    const otherThread = makeThread("local-other", "Other Thread", "daemon-other");
    useAgentStore.setState({
      threads: [rootThread, parentThread, otherThread],
      messages: {
        [rootThread.id]: [],
        [parentThread.id]: [],
        [otherThread.id]: [],
      },
      todos: {
        [rootThread.id]: [],
        [parentThread.id]: [],
        [otherThread.id]: [],
      },
      activeThreadId: "local-root",
      threadHistoryStack: ["local-parent"],
    } as any);
  });

  it("hydrates a missing daemon thread into local state before navigating", async () => {
    const liveStore = createLiveStoreWrappers();
    const getRemoteThread = vi.fn(async () => ({
      id: "daemon-child",
      title: "Hydrated Child",
      messages: [
        {
          id: "message-1",
          role: "assistant",
          content: "ready",
          timestamp: 10,
          input_tokens: 0,
          output_tokens: 0,
        },
      ],
    }));
    const fetchThreadTodos = vi.fn(async () => [{ id: "todo-1" }]);

    const result = await openSpawnedAgentThreadFromRun({
      activeThreadId: "local-root",
      threads: useAgentStore.getState().threads,
      workspaces: [makeWorkspace("session-child")],
      run: makeRun({
        id: "run-child",
        task_id: "task-child",
        title: "Spawned Child",
        thread_id: "daemon-child",
        session_id: "session-child",
        parent_task_id: "task-root",
        parent_thread_id: "daemon-root",
      }),
      messageLimit: 75,
      getRemoteThread,
      fetchThreadTodos,
      ...liveStore,
    });

    const hydratedThread = findHydratedThread("daemon-child");
    expect(result).toBe(true);
    expect(getRemoteThread).toHaveBeenCalledWith("daemon-child", { messageLimit: 75 });
    expect(liveStore.createThread).toHaveBeenCalledWith({
      workspaceId: "workspace-1",
      surfaceId: "surface-1",
      paneId: "pane-1",
      title: "Hydrated Child",
    });
    expect(liveStore.setThreadDaemonId).toHaveBeenCalledWith(hydratedThread.id, "daemon-child");
    expect(liveStore.addMessage).toHaveBeenCalledTimes(1);
    expect(liveStore.setThreadTodos).toHaveBeenCalledWith(hydratedThread.id, [{ id: "todo-1" }]);
    expect(liveStore.openSpawnedThread).toHaveBeenCalledWith("local-root", hydratedThread.id);
    expect(useAgentStore.getState().activeThreadId).toBe(hydratedThread.id);
    expect(useAgentStore.getState().threadHistoryStack).toEqual(["local-parent", "local-root"]);
    expect(useAgentStore.getState().messages[hydratedThread.id]).toHaveLength(1);
    expect(useAgentStore.getState().todos[hydratedThread.id]).toEqual([{ id: "todo-1" }]);
  });

  it("dedupes concurrent hydrations and only replays navigation for the active caller", async () => {
    let releaseTodos: ((value: AgentTodoItem[]) => void) | null = null;
    const liveStore = createLiveStoreWrappers();
    const getRemoteThread = vi.fn(async () => ({
      id: "daemon-child",
      title: "Hydrated Child",
      messages: [],
    }));
    const fetchThreadTodos = vi.fn(
      () =>
        new Promise<AgentTodoItem[]>((resolve) => {
          releaseTodos = resolve;
        }),
    );

    const params = {
      activeThreadId: "local-root",
      threads: useAgentStore.getState().threads,
      workspaces: [makeWorkspace("session-child")],
      run: makeRun({
        id: "run-child",
        task_id: "task-child",
        title: "Spawned Child",
        thread_id: "daemon-child",
        session_id: "session-child",
      }),
      messageLimit: 75,
      getRemoteThread,
      fetchThreadTodos,
      ...liveStore,
    };

    const firstOpen = openSpawnedAgentThreadFromRun(params);
    const secondOpen = openSpawnedAgentThreadFromRun(params);

    await Promise.resolve();
    await Promise.resolve();

    expect(liveStore.createThread).toHaveBeenCalledTimes(1);
    expect(getRemoteThread).toHaveBeenCalledTimes(1);
    expect(liveStore.openSpawnedThread).not.toHaveBeenCalled();

    releaseTodos?.([{ id: "todo-1" }]);

    const hydratedThread = findHydratedThread("daemon-child");
    await expect(firstOpen).resolves.toBe(true);
    await expect(secondOpen).resolves.toBe(true);
    expect(liveStore.openSpawnedThread).toHaveBeenCalledTimes(1);
    expect(liveStore.openSpawnedThread).toHaveBeenCalledWith("local-root", hydratedThread.id);
    expect(liveStore.setThreadTodos).toHaveBeenCalledWith(hydratedThread.id, [{ id: "todo-1" }]);
    expect(useAgentStore.getState().activeThreadId).toBe(hydratedThread.id);
    expect(useAgentStore.getState().threadHistoryStack).toEqual(["local-parent", "local-root"]);
  });

  it("replays navigation for a later caller from a different source thread", async () => {
    let releaseTodos: ((value: AgentTodoItem[]) => void) | null = null;
    const liveStore = createLiveStoreWrappers();
    const getRemoteThread = vi.fn(async () => ({
      id: "daemon-child",
      title: "Hydrated Child",
      messages: [],
    }));
    const fetchThreadTodos = vi.fn(
      () =>
        new Promise<AgentTodoItem[]>((resolve) => {
          releaseTodos = resolve;
        }),
    );

    const sharedRun = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "daemon-child",
      session_id: "session-child",
    });

    const firstOpen = openSpawnedAgentThreadFromRun({
      activeThreadId: "local-root",
      threads: [
        makeThread("local-root", "Root Thread", "daemon-root"),
        makeThread("local-other", "Other Thread", "daemon-other"),
      ],
      workspaces: [makeWorkspace("session-child")],
      run: sharedRun,
      messageLimit: 75,
      getRemoteThread,
      fetchThreadTodos,
      ...liveStore,
    });

    useAgentStore.setState({
      activeThreadId: "local-other",
      threadHistoryStack: ["local-upstream"],
    } as any);

    const secondOpen = openSpawnedAgentThreadFromRun({
      activeThreadId: "local-other",
      threads: useAgentStore.getState().threads,
      workspaces: [makeWorkspace("session-child")],
      run: sharedRun,
      messageLimit: 75,
      getRemoteThread,
      fetchThreadTodos,
      ...liveStore,
    });

    await Promise.resolve();
    await Promise.resolve();
    releaseTodos?.([{ id: "todo-1" }]);

    const hydratedThread = findHydratedThread("daemon-child");
    await expect(firstOpen).resolves.toBe(false);
    await expect(secondOpen).resolves.toBe(true);
    expect(liveStore.createThread).toHaveBeenCalledTimes(1);
    expect(liveStore.openSpawnedThread).toHaveBeenCalledTimes(1);
    expect(liveStore.openSpawnedThread).toHaveBeenCalledWith("local-other", hydratedThread.id);
    expect(useAgentStore.getState().activeThreadId).toBe(hydratedThread.id);
    expect(useAgentStore.getState().threadHistoryStack).toEqual(["local-upstream", "local-other"]);
  });

  it("clears pending hydration state after a failed daemon fetch", async () => {
    const error = new Error("boom");
    const getRemoteThread = vi.fn()
      .mockRejectedValueOnce(error)
      .mockResolvedValueOnce(null);

    const firstOpen = openSpawnedAgentThreadFromRun({
      activeThreadId: "local-root",
      threads: useAgentStore.getState().threads,
      workspaces: [makeWorkspace("session-child")],
      run: makeRun({
        id: "run-child",
        task_id: "task-child",
        title: "Spawned Child",
        thread_id: "daemon-child",
        session_id: "session-child",
      }),
      messageLimit: 75,
      getRemoteThread,
      fetchThreadTodos: vi.fn(),
      createThread: vi.fn(),
      addMessage: vi.fn(),
      setThreadDaemonId: vi.fn(),
      setThreadTodos: vi.fn(),
      openSpawnedThread: vi.fn(),
    });

    await expect(firstOpen).rejects.toThrow("boom");

    const secondOpen = openSpawnedAgentThreadFromRun({
      activeThreadId: "local-root",
      threads: useAgentStore.getState().threads,
      workspaces: [makeWorkspace("session-child")],
      run: makeRun({
        id: "run-child",
        task_id: "task-child",
        title: "Spawned Child",
        thread_id: "daemon-child",
        session_id: "session-child",
      }),
      messageLimit: 75,
      getRemoteThread,
      fetchThreadTodos: vi.fn(),
      createThread: vi.fn(),
      addMessage: vi.fn(),
      setThreadDaemonId: vi.fn(),
      setThreadTodos: vi.fn(),
      openSpawnedThread: vi.fn(),
    });

    await expect(secondOpen).resolves.toBe(false);
    expect(getRemoteThread).toHaveBeenCalledTimes(2);
  });
});
