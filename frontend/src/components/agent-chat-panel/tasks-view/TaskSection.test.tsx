import { Children, isValidElement, type ReactNode } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import type { AgentRun } from "@/lib/agentRuns";
import { buildTaskSubagentTree, TaskSubagentTree } from "./TaskSection";
import { collectSelectedTaskSubagents } from "../TasksView";

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
    thread_id: "thread-root",
    ...overrides,
  };
}

function resolveTree(node: ReactNode): ReactNode {
  if (node == null || typeof node === "boolean" || typeof node === "string" || typeof node === "number") {
    return node;
  }
  if (Array.isArray(node)) {
    return node.map((child) => resolveTree(child));
  }
  if (!isValidElement(node)) {
    return node;
  }
  if (typeof node.type === "function") {
    return resolveTree(node.type(node.props));
  }

  return {
    ...node,
    props: {
      ...node.props,
      children: Children.toArray(node.props.children).map((child) => resolveTree(child)),
    },
  };
}

function isResolvedElement(node: ReactNode): node is { type: unknown; props: Record<string, any> } {
  return Boolean(node) && typeof node === "object" && "type" in node && "props" in node;
}

function elementText(node: ReactNode): string {
  if (node == null || typeof node === "boolean") {
    return "";
  }
  if (typeof node === "string" || typeof node === "number") {
    return String(node);
  }
  if (Array.isArray(node)) {
    return node.map((child) => elementText(child)).join("");
  }
  if (!isValidElement(node) && !isResolvedElement(node)) {
    return "";
  }
  return elementText(node.props.children);
}

function findElementByDataAttr(node: ReactNode, attrName: string, attrValue: string): any {
  if (node == null || typeof node === "boolean" || typeof node === "string" || typeof node === "number") {
    return null;
  }
  if (Array.isArray(node)) {
    for (const child of node) {
      const found = findElementByDataAttr(child, attrName, attrValue);
      if (found) {
        return found;
      }
    }
    return null;
  }
  if (!isValidElement(node) && !isResolvedElement(node)) {
    return null;
  }

  if ((node.props as Record<string, unknown>)[attrName] === attrValue) {
    return node;
  }

  return findElementByDataAttr(node.props.children, attrName, attrValue);
}

function findButton(node: ReactNode, label: string): any {
  if (node == null || typeof node === "boolean" || typeof node === "string" || typeof node === "number") {
    return null;
  }
  if (Array.isArray(node)) {
    for (const child of node) {
      const found = findButton(child, label);
      if (found) {
        return found;
      }
    }
    return null;
  }
  if (!isValidElement(node) && !isResolvedElement(node)) {
    return null;
  }

  if (node.type === "button" && elementText(node.props.children).includes(label)) {
    return node;
  }

  return findButton(node.props.children, label);
}

describe("TaskSubagentTree", () => {
  it("renders descendants even when the task has no thread id", () => {
    const task = {
      id: "task-root",
      title: "Root task",
      description: "Root task",
      status: "in_progress",
      priority: "normal",
      progress: 0,
      created_at: 1,
      source: "daemon",
      thread_id: null,
    } as never;
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      parent_task_id: "task-root",
      thread_id: "thread-child",
      created_at: 2,
    });
    const grandchild = makeRun({
      id: "run-grandchild",
      task_id: "task-grandchild",
      title: "Grandchild",
      thread_id: "thread-grandchild",
      parent_task_id: "task-child",
      created_at: 3,
    });
    const tree = buildTaskSubagentTree(task, [child, grandchild]);

    const html = renderToStaticMarkup(
      <TaskSubagentTree
        subagentCount={3}
        tree={tree}
        selectedTaskId={task.id}
        selectedDaemonThreadId={null}
        onSelectTask={vi.fn()}
        onOpenTaskThread={vi.fn()}
      />,
    );

    expect(html).toContain("Spawned Child");
    expect(html).toContain("Grandchild");
    expect(html).toContain('data-node-depth="1"');
  });

  it("renders root children linked only by parent_run_id", () => {
    const task = {
      id: "task-root",
      title: "Root task",
      description: "Root task",
      status: "in_progress",
      priority: "normal",
      progress: 0,
      created_at: 1,
      source: "daemon",
      thread_id: null,
    } as never;
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "thread-child",
      parent_run_id: "task-root",
      created_at: 2,
    });
    const tree = buildTaskSubagentTree(task, [child]);

    const html = renderToStaticMarkup(
      <TaskSubagentTree
        subagentCount={1}
        tree={tree}
        selectedTaskId={task.id}
        selectedDaemonThreadId={null}
        onSelectTask={vi.fn()}
        onOpenTaskThread={vi.fn()}
      />,
    );

    expect(html).toContain("Spawned Child");
  });

  it("opens a child thread through the shared navigation helper", () => {
    const task = {
      id: "task-root",
      title: "Root task",
      description: "Root task",
      status: "in_progress",
      priority: "normal",
      progress: 0,
      created_at: 1,
      source: "daemon",
      thread_id: "thread-root",
    } as never;
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "thread-child",
      parent_task_id: "task-root",
      created_at: 2,
    });
    const tree = buildTaskSubagentTree(task, [child]);
    const onOpenTaskThread = vi.fn();
    const resolved = resolveTree(
      <TaskSubagentTree
        subagentCount={1}
        tree={tree}
        selectedTaskId={task.id}
        selectedDaemonThreadId={task.thread_id}
        onSelectTask={vi.fn()}
        onOpenTaskThread={onOpenTaskThread}
      />,
    );

    const childNode = findElementByDataAttr(resolved, "data-node-title", "Spawned Child");
    const openChatButton = findButton(childNode?.props.children ?? null, "Open Chat");

    openChatButton.props.onClick();

    expect(onOpenTaskThread).toHaveBeenCalledTimes(1);
    expect(onOpenTaskThread).toHaveBeenCalledWith(child);
  });

  it("still supports selecting a child task for inspection", () => {
    const task = {
      id: "task-root",
      title: "Root task",
      description: "Root task",
      status: "in_progress",
      priority: "normal",
      progress: 0,
      created_at: 1,
      source: "daemon",
      thread_id: "thread-root",
    } as never;
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "thread-child",
      parent_task_id: "task-root",
      created_at: 2,
    });
    const tree = buildTaskSubagentTree(task, [child]);
    const onSelectTask = vi.fn();
    const resolved = resolveTree(
      <TaskSubagentTree
        subagentCount={1}
        tree={tree}
        selectedTaskId={task.id}
        selectedDaemonThreadId={task.thread_id}
        onSelectTask={onSelectTask}
        onOpenTaskThread={vi.fn()}
      />,
    );

    const childNode = findElementByDataAttr(resolved, "data-node-title", "Spawned Child");
    const inspectButton = findButton(childNode?.props.children ?? null, "Inspect");

    inspectButton.props.onClick();

    expect(onSelectTask).toHaveBeenCalledTimes(1);
    expect(onSelectTask).toHaveBeenCalledWith("task-child");
  });

});

describe("collectSelectedTaskSubagents", () => {
  it("keeps descendants linked by parent_task_id even when the task has no thread id", () => {
    const selectedTask = {
      id: "task-root",
      title: "Root task",
    } as any;
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "thread-child",
      parent_task_id: "task-root",
      created_at: 2,
    });

    expect(collectSelectedTaskSubagents(selectedTask, [child])).toEqual([child]);
  });

  it("keeps root descendants linked by parent_run_id", () => {
    const selectedTask = {
      id: "task-root",
      title: "Root task",
    } as any;
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "thread-child",
      parent_run_id: "task-root",
      created_at: 2,
    });

    expect(collectSelectedTaskSubagents(selectedTask, [child])).toEqual([child]);
  });

  it("does not include a same-thread branch with a different parent task", () => {
    const selectedTask = {
      id: "task-root",
      title: "Root task",
    } as any;
    const child = makeRun({
      id: "run-child",
      task_id: "task-child",
      title: "Spawned Child",
      thread_id: "thread-child",
      parent_task_id: "task-root",
      created_at: 2,
    });
    const unrelated = makeRun({
      id: "run-unrelated",
      task_id: "task-unrelated",
      title: "Unrelated",
      thread_id: "thread-unrelated",
      parent_task_id: "task-other",
      parent_thread_id: "thread-child",
      created_at: 3,
    });

    expect(collectSelectedTaskSubagents(selectedTask, [child, unrelated])).toEqual([child]);
  });
});
