import { StatusChip } from "@/components/StatusChip";
import { useCallback, useEffect, useMemo, useState } from "react";
import { getBridge } from "@/lib/bridge";
import { ActionButton, EmptyPanel, iconButtonStyle } from "../shared";
import {
  fetchFilePreview,
  fetchGitDiff,
  fetchThreadWorkContext,
  type ThreadWorkContext,
} from "../../../lib/agentWorkContext";
import {
  formatTaskStatus,
  formatTaskTimestamp,
  getTaskStatusChip,
  getTaskStatusReason,
  getTaskStatusReasonChip,
  isTaskActive,
  isTaskTerminal,
  taskStatusColor,
  type AgentQueueTask,
} from "../../../lib/agentTaskQueue";
import { type AgentRun } from "../../../lib/agentRuns";
import { type SpawnedAgentTree, type SpawnedAgentTreeNode } from "../../../lib/spawnedAgentTree";
import { shortenHomePath, useWorkspaceStore } from "../../../lib/workspaceStore";
import { SpawnedAgentNode } from "../SpawnedAgentsPanel";
import { detailLabelStyle } from "./styles";
import { findTaskWorkspaceLocation, taskLooksLikeCoding, workContextKindColor, workContextKindLabel } from "./helpers";
import type { TaskWorkspaceLocation, ThreadTarget } from "./types";

function compareSubagentRuns(left: AgentRun, right: AgentRun): number {
  if (left.created_at !== right.created_at) {
    return right.created_at - left.created_at;
  }

  if (left.task_id !== right.task_id) {
    return left.task_id.localeCompare(right.task_id);
  }

  return left.id.localeCompare(right.id);
}

function uniqueSubagentRuns(runs: readonly AgentRun[]): AgentRun[] {
  const seen = new Set<string>();
  const result: AgentRun[] = [];

  for (const run of runs) {
    if (seen.has(run.id)) {
      continue;
    }
    seen.add(run.id);
    result.push(run);
  }

  return result;
}

export function buildTaskSubagentTree(
  task: AgentQueueTask,
  subagents: AgentRun[],
): SpawnedAgentTree<AgentRun> | null {
  const canonicalSubagents = uniqueSubagentRuns(
    subagents
      .filter((run) => run.kind === "subagent")
      .slice()
      .sort(compareSubagentRuns),
  );

  if (canonicalSubagents.length === 0) {
    return null;
  }

  const byParentTaskId = new Map<string, AgentRun[]>();
  const byParentRunId = new Map<string, AgentRun[]>();
  const push = (map: Map<string, AgentRun[]>, key: string, run: AgentRun) => {
    const bucket = map.get(key);
    if (bucket) {
      bucket.push(run);
      return;
    }
    map.set(key, [run]);
  };

  for (const run of canonicalSubagents) {
    if (run.parent_task_id) {
      push(byParentTaskId, run.parent_task_id, run);
    }
    if (run.parent_run_id) {
      push(byParentRunId, run.parent_run_id, run);
    }
  }

  const buildChildren = (
    parentTaskId: string,
    parentRunId: string | null,
    ancestry: Set<string>,
  ): SpawnedAgentTreeNode<AgentRun>[] => {
    const childCandidates = uniqueSubagentRuns([
      ...(byParentTaskId.get(parentTaskId) ?? []),
      ...(parentRunId ? byParentRunId.get(parentRunId) ?? [] : []),
    ])
      .filter((candidate) => !ancestry.has(candidate.id))
      .sort(compareSubagentRuns);

    if (childCandidates.length === 0) {
      return [];
    }

    return childCandidates.map((candidate) => {
      const nextAncestry = new Set(ancestry);
      nextAncestry.add(candidate.id);
      return {
        item: candidate,
        children: buildChildren(candidate.task_id ?? candidate.id, candidate.id, nextAncestry),
        openable: Boolean(candidate.thread_id),
        live: !isTaskTerminal(candidate),
      };
    });
  };

  const roots = buildChildren(task.id, task.id, new Set());
  if (roots.length === 0) {
    return null;
  }

  return {
    activeThreadId: task.thread_id ?? task.id,
    anchor: null,
    roots,
  };
}

export function TaskCard({
  task,
  selected,
  onSelect,
  onCancel,
}: {
  task: AgentQueueTask;
  selected: boolean;
  onSelect: () => void;
  onCancel?: () => void;
}) {
  const statusColor = taskStatusColor(task.status);
  const statusChip = getTaskStatusChip(task);
  const statusReason = getTaskStatusReason(task);
  const reasonChip = getTaskStatusReasonChip(task);
  const isActive = isTaskActive(task);

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onSelect}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect();
        }
      }}
      style={{
        width: "100%",
        textAlign: "left",
        padding: "var(--space-3)",
        borderRadius: "var(--radius-md)",
        border: selected ? `1px solid ${statusColor}` : "1px solid var(--border)",
        background: selected ? "var(--bg-tertiary)" : "var(--bg-secondary)",
        marginBottom: "var(--space-2)",
        cursor: "pointer",
      }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--space-2)" }}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontSize: "var(--text-sm)", fontWeight: 500, color: "var(--text-primary)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
            {task.title}
          </div>
          {task.goal_run_title && (
            <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: 2, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              Goal: {task.goal_run_title}
            </div>
          )}
          {task.source === "subagent" && (
            <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: 2, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              Subagent · runtime {task.runtime ?? "daemon"}
            </div>
          )}
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", marginTop: 2, display: "flex", alignItems: "center", gap: "var(--space-2)", flexWrap: "wrap" }}>
            <StatusChip icon={statusChip.icon} label={statusChip.label} tone={statusChip.tone} />
            {task.status === "in_progress" && task.progress > 0 && <span>{task.progress}%</span>}
            <span>{formatTaskTimestamp(task.created_at)}</span>
            {typeof task.retry_count === "number" && typeof task.max_retries === "number" && (
              <span>
                retry {task.retry_count}/{task.max_retries === 0 ? "∞" : task.max_retries}
              </span>
            )}
          </div>
        </div>
        {isActive && onCancel && (
          <button type="button" onClick={(event) => { event.stopPropagation(); onCancel(); }} style={{ ...iconButtonStyle, fontSize: 11 }} title="Cancel task">
            Cancel
          </button>
        )}
      </div>
      {statusReason && (
        <div
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--text-secondary)",
            marginTop: "var(--space-2)",
            display: "flex",
            alignItems: "flex-start",
            gap: "var(--space-2)",
            flexWrap: "wrap",
          }}
        >
          {reasonChip ? <StatusChip icon={reasonChip.icon} label={reasonChip.label} tone={reasonChip.tone} style={{ fontSize: 10, padding: "1px 6px" }} /> : null}
          <span>{statusReason}</span>
        </div>
      )}
      {task.command && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: "var(--space-2)" }}>
          {task.command}
        </div>
      )}
    </div>
  );
}

function TaskCodePreview({
  task,
  location,
}: {
  task: AgentQueueTask;
  location: TaskWorkspaceLocation | null;
}) {
  const [context, setContext] = useState<ThreadWorkContext>({ threadId: "", entries: [] });
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [previewText, setPreviewText] = useState("");
  const [loadingEntries, setLoadingEntries] = useState(false);
  const [loadingDiff, setLoadingDiff] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const codingTask = taskLooksLikeCoding(task);
  const threadId = task.thread_id || null;
  const bridge = getBridge();
  const selectedEntry = useMemo(
    () => context.entries.find((entry) => entry.path === selectedPath) ?? null,
    [context.entries, selectedPath],
  );

  useEffect(() => {
    if (!threadId) {
      setContext({ threadId: "", entries: [] });
      setSelectedPath(null);
      setPreviewText("");
      setLoadingEntries(false);
      setLoadingDiff(false);
      setError(null);
      return;
    }

    let cancelled = false;
    setLoadingEntries(true);
    setError(null);

    void fetchThreadWorkContext(threadId)
      .then((nextContext) => {
        if (cancelled) return;
        setContext(nextContext);
        setSelectedPath((current) => current && nextContext.entries.some((entry) => entry.path === current) ? current : nextContext.entries[0]?.path ?? null);
      })
      .catch((reason: unknown) => {
        if (cancelled) return;
        setContext({ threadId, entries: [] });
        setSelectedPath(null);
        setPreviewText("");
        setError(reason instanceof Error ? reason.message : String(reason));
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingEntries(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [task.id, threadId]);

  useEffect(() => {
    if (!threadId || !bridge?.onAgentEvent) {
      return;
    }
    return bridge.onAgentEvent((event: any) => {
      if (event?.type !== "work_context_update" || event?.thread_id !== threadId) {
        return;
      }
      void fetchThreadWorkContext(threadId).then((nextContext) => {
        setContext(nextContext);
        setSelectedPath((current) => current && nextContext.entries.some((entry) => entry.path === current) ? current : nextContext.entries[0]?.path ?? null);
      });
    });
  }, [bridge, threadId]);

  useEffect(() => {
    if (!selectedEntry) {
      setPreviewText("");
      setLoadingDiff(false);
      return;
    }

    let cancelled = false;
    setLoadingDiff(true);
    setError(null);

    const previewPromise = selectedEntry.repoRoot
      ? fetchGitDiff(selectedEntry.repoRoot, selectedEntry.path)
      : fetchFilePreview(selectedEntry.path).then((preview) => preview?.content ?? "");

    void previewPromise
      .then((output) => {
        if (!cancelled) {
          setPreviewText(output);
        }
      })
      .catch((reason: unknown) => {
        if (!cancelled) {
          setPreviewText("");
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoadingDiff(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [selectedEntry]);

  if (!threadId || (!codingTask && context.entries.length === 0 && !loadingEntries && !error)) {
    return null;
  }

  return (
    <div style={{ marginTop: "var(--space-3)", display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
      <div style={detailLabelStyle}>Work Context</div>
      <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
        Scope: {location?.cwd ? shortenHomePath(location.cwd) : "thread workspace"}
        <span style={{ marginLeft: "var(--space-2)" }}>
          {loadingEntries ? "Refreshing..." : `${context.entries.length} file${context.entries.length === 1 ? "" : "s"} / artifact${context.entries.length === 1 ? "" : "s"}`}
        </span>
      </div>
      {error && <div style={{ fontSize: "var(--text-xs)", color: "var(--danger)" }}>{error}</div>}
      {context.entries.length > 0 ? (
        <div style={{ display: "grid", gridTemplateColumns: "minmax(220px, 280px) minmax(0, 1fr)", gap: "var(--space-2)" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)", maxHeight: 320, overflow: "auto" }}>
            {context.entries.map((entry) => {
              const selected = entry.path === selectedPath;
              return (
                <button
                  key={`${entry.source}:${entry.path}`}
                  type="button"
                  onClick={() => setSelectedPath(entry.path)}
                  style={{
                    textAlign: "left",
                    padding: "var(--space-2)",
                    borderRadius: "var(--radius-sm)",
                    border: selected ? `1px solid ${workContextKindColor(entry)}` : "1px solid var(--border)",
                    background: selected ? "var(--bg-tertiary)" : "var(--bg-secondary)",
                    cursor: "pointer",
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)", marginBottom: 4, flexWrap: "wrap" }}>
                    <span style={{ fontSize: "var(--text-xs)", color: workContextKindColor(entry), fontWeight: 600 }}>
                      {workContextKindLabel(entry)}
                    </span>
                    <span style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}>
                      {entry.source}
                    </span>
                  </div>
                  <div style={{ fontSize: "var(--text-xs)", color: "var(--text-primary)", fontFamily: "var(--font-mono)", wordBreak: "break-word" }}>
                    {entry.path}
                  </div>
                  {entry.previousPath && (
                    <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", marginTop: 4, wordBreak: "break-word" }}>
                      from {entry.previousPath}
                    </div>
                  )}
                </button>
              );
            })}
          </div>
          <div style={{ minHeight: 220, maxHeight: 320, overflow: "auto", padding: "var(--space-2)", borderRadius: "var(--radius-sm)", background: "var(--bg-tertiary)", border: "1px solid var(--border)" }}>
            {loadingDiff ? (
              <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>Loading preview...</div>
            ) : previewText.trim() ? (
              <pre
                style={{
                  margin: 0,
                  fontSize: "var(--text-xs)",
                  lineHeight: 1.5,
                  color: "var(--text-primary)",
                  fontFamily: "var(--font-mono)",
                  whiteSpace: "pre-wrap",
                  wordBreak: "break-word",
                }}
              >
                {previewText}
              </pre>
            ) : (
              <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
                No preview available for the selected item.
              </div>
            )}
          </div>
        </div>
      ) : (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", padding: "var(--space-2)", borderRadius: "var(--radius-sm)", background: "var(--bg-tertiary)" }}>
          {loadingEntries ? "Refreshing work context..." : "No file or artifact activity detected for this task yet."}
        </div>
      )}
    </div>
  );
}

type TaskSubagentTreeProps = {
  subagentCount: number;
  tree: SpawnedAgentTree<AgentRun> | null;
  selectedTaskId: string | null;
  selectedDaemonThreadId: string | null;
  onSelectTask: (taskId: string) => void;
  onOpenTaskThread: (task: ThreadTarget) => void;
};

export function TaskSubagentTree({
  subagentCount,
  tree,
  selectedTaskId,
  selectedDaemonThreadId,
  onSelectTask,
  onOpenTaskThread,
}: TaskSubagentTreeProps) {
  const renderNodeActions = ({
    node,
    canOpen,
    openSpawnedThread,
  }: {
    node: SpawnedAgentTreeNode<AgentRun>;
    canOpen: boolean;
    openSpawnedThread: () => void;
  }) => (
    <>
      {node.item.thread_id && (
        <ActionButton disabled={!canOpen} onClick={canOpen ? openSpawnedThread : undefined}>
          <span aria-label={`Open chat for ${node.item.title}`}>Open Chat</span>
        </ActionButton>
      )}
      <ActionButton onClick={() => onSelectTask(node.item.task_id ?? node.item.id)}>
        Inspect
      </ActionButton>
    </>
  );

  return (
    <div style={{ marginTop: "var(--space-3)" }}>
      <div
        style={{
          fontSize: "var(--text-xs)",
          color: "var(--text-muted)",
          textTransform: "uppercase",
          letterSpacing: "0.06em",
          marginBottom: "var(--space-2)",
        }}
      >
        Subagents ({subagentCount})
      </div>
      {tree ? (
        <div style={{ display: "grid", gap: "var(--space-2)" }}>
          {tree.roots.map((root) => (
            <SpawnedAgentNode
              key={root.item.id}
              node={root}
              depth={0}
              selectedDaemonThreadId={selectedDaemonThreadId}
              selectedTaskId={selectedTaskId}
              canOpenSpawnedThread={(run) => Boolean(run.thread_id)}
              openSpawnedThread={async (run) => {
                onOpenTaskThread(run);
                return true;
              }}
              renderActions={renderNodeActions}
            />
          ))}
        </div>
      ) : (
        <EmptyPanel message="No child subagents have been spawned for this task." />
      )}
    </div>
  );
}

export function TaskPostMortem({
  task,
  subagents,
  onSelectTask,
  onOpenTaskThread,
}: {
  task: AgentQueueTask;
  subagents: AgentRun[];
  onSelectTask: (taskId: string) => void;
  onOpenTaskThread: (task: ThreadTarget) => void;
}) {
  const workspaces = useWorkspaceStore((state) => state.workspaces);
  const setActiveWorkspace = useWorkspaceStore((state) => state.setActiveWorkspace);
  const setActiveSurface = useWorkspaceStore((state) => state.setActiveSurface);
  const setActivePaneId = useWorkspaceStore((state) => state.setActivePaneId);
  const focusCanvasPanel = useWorkspaceStore((state) => state.focusCanvasPanel);
  const statusChip = getTaskStatusChip(task);
  const statusReason = getTaskStatusReason(task);
  const reasonChip = getTaskStatusReasonChip(task);
  const logs = [...(task.logs ?? [])].slice(-8).reverse();
  const location = useMemo(
    () => findTaskWorkspaceLocation(workspaces, task.session_id),
    [task.session_id, workspaces],
  );
  const tree = useMemo(
    () => buildTaskSubagentTree(task, subagents),
    [subagents, task],
  );
  const openTaskSession = useCallback(() => {
    if (!location) {
      return;
    }
    setActiveWorkspace(location.workspaceId);
    setActiveSurface(location.surfaceId);
    focusCanvasPanel(location.paneId, { storePreviousView: true });
    setActivePaneId(location.paneId);
  }, [focusCanvasPanel, location, setActivePaneId, setActiveSurface, setActiveWorkspace]);

  return (
    <div style={{ padding: "var(--space-3)", borderRadius: "var(--radius-md)", border: "1px solid var(--border)", background: "var(--bg-secondary)" }}>
      <div style={{ fontSize: "var(--text-sm)", color: "var(--text-primary)", fontWeight: 600 }}>{task.title}</div>
      {task.goal_run_title && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: 4 }}>
          Goal: {task.goal_run_title}
          {task.goal_step_title && task.goal_step_title !== task.title ? ` · Step: ${task.goal_step_title}` : ""}
        </div>
      )}
      <div style={{ fontSize: "var(--text-xs)", marginTop: 4, display: "flex", alignItems: "center", gap: "var(--space-2)", flexWrap: "wrap" }}>
        <StatusChip icon={statusChip.icon} label={statusChip.label} tone={statusChip.tone} />
        <span style={{ color: "var(--text-muted)" }}>{formatTaskStatus(task)}</span>
      </div>
      {task.command && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: "var(--space-2)" }}>
          Command: {task.command}
        </div>
      )}
      {task.session_id && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: 4 }}>
          Session: {task.session_id}
        </div>
      )}
      <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: 4 }}>
        Runtime: {task.runtime ?? "daemon"}
      </div>
      {location && (
        <div style={{ marginTop: "var(--space-2)", display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--space-2)", flexWrap: "wrap" }}>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
            Workspace: {location.workspaceName} · Surface: {location.surfaceName}
            {location.cwd ? ` · ${shortenHomePath(location.cwd)}` : ""}
          </div>
          <div style={{ display: "flex", gap: "var(--space-2)", flexWrap: "wrap" }}>
            {task.thread_id && <ActionButton onClick={() => onOpenTaskThread(task)}>Open Chat</ActionButton>}
            <ActionButton onClick={openTaskSession}>Open Session</ActionButton>
          </div>
        </div>
      )}
      {!location && task.thread_id && (
        <div style={{ marginTop: "var(--space-2)", display: "flex", justifyContent: "flex-end" }}>
          <ActionButton onClick={() => onOpenTaskThread(task)}>Open Chat</ActionButton>
        </div>
      )}
      {task.dependencies && task.dependencies.length > 0 && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: 4 }}>
          Depends on: {task.dependencies.join(", ")}
        </div>
      )}
      {task.parent_task_id && (
        <div style={{ marginTop: 4, display: "flex", alignItems: "center", justifyContent: "space-between", gap: "var(--space-2)", flexWrap: "wrap" }}>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)" }}>
            Parent task: {task.parent_task_id}
          </div>
          <ActionButton onClick={() => onSelectTask(task.parent_task_id!)}>Back To Parent</ActionButton>
        </div>
      )}
      {statusReason && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: "var(--space-2)", display: "flex", alignItems: "flex-start", gap: "var(--space-2)", flexWrap: "wrap" }}>
          {reasonChip ? <StatusChip icon={reasonChip.icon} label={reasonChip.label} tone={reasonChip.tone} style={{ fontSize: 10, padding: "1px 6px" }} /> : null}
          <span>{statusReason}</span>
        </div>
      )}
      {task.last_error && task.last_error !== statusReason && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--danger)", marginTop: "var(--space-2)" }}>
          {task.last_error}
        </div>
      )}
      <TaskCodePreview task={task} location={location} />
      <TaskSubagentTree
        subagentCount={subagents.length}
        tree={tree}
        selectedTaskId={task.id}
        selectedDaemonThreadId={task.thread_id ?? null}
        onSelectTask={onSelectTask}
        onOpenTaskThread={onOpenTaskThread}
      />
      <div style={{ marginTop: "var(--space-3)", display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
        {logs.length > 0 ? logs.map((log) => (
          <div key={log.id} style={{ padding: "var(--space-2)", borderRadius: "var(--radius-sm)", background: "var(--bg-tertiary)" }}>
            <div style={{ fontSize: "var(--text-xs)", color: log.level === "error" ? "var(--danger)" : log.level === "warn" ? "var(--warning)" : "var(--text-muted)" }}>
              {log.phase} · attempt {log.attempt || 0} · {formatTaskTimestamp(log.timestamp)}
            </div>
            <div style={{ fontSize: "var(--text-sm)", color: "var(--text-primary)", marginTop: 2 }}>{log.message}</div>
            {log.details && (
              <div style={{ fontSize: "var(--text-xs)", color: "var(--text-secondary)", marginTop: 4 }}>{log.details}</div>
            )}
          </div>
        )) : (
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>No task logs recorded yet.</div>
        )}
      </div>
    </div>
  );
}
