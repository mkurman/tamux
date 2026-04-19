import { StatusChip } from "@/components/StatusChip";
import type { ReactNode } from "react";
import type { AgentRun } from "@/lib/agentRuns";
import {
  getRunStatusChip,
  getRunStatusReason,
  getRunStatusReasonChip,
} from "@/lib/agentRuns";
import type {
  SpawnedAgentTree,
  SpawnedAgentTreeNode,
} from "@/lib/spawnedAgentTree";
import { ActionButton, EmptyPanel } from "./shared";

type SpawnedAgentsPanelProps = {
  tree: SpawnedAgentTree<AgentRun> | null;
  selectedDaemonThreadId: string | null;
  canGoBackThread: boolean;
  threadNavigationDepth: number;
  backThreadTitle: string | null;
  canOpenSpawnedThread: (run: AgentRun) => boolean;
  openSpawnedThread: (run: AgentRun) => Promise<boolean>;
  goBackThread: () => void;
};

type SpawnedAgentNodeProps = {
  node: SpawnedAgentTreeNode<AgentRun>;
  depth: number;
  selectedDaemonThreadId: string | null;
  selectedTaskId?: string | null;
  canOpenSpawnedThread: (run: AgentRun) => boolean;
  openSpawnedThread: (run: AgentRun) => Promise<boolean>;
  renderActions?: (args: {
    node: SpawnedAgentTreeNode<AgentRun>;
    canOpen: boolean;
    openSpawnedThread: () => void;
  }) => ReactNode;
};

function sessionHint(run: AgentRun): string | null {
  if (!run.session_id) {
    return null;
  }
  return run.session_id;
}

function renderNodeMeta(run: AgentRun): string | null {
  const parts = [run.runtime ?? null, sessionHint(run)].filter(Boolean);
  return parts.length > 0 ? parts.join(" · ") : null;
}

export function SpawnedAgentNode({
  node,
  depth,
  selectedDaemonThreadId,
  selectedTaskId,
  canOpenSpawnedThread,
  openSpawnedThread,
  renderActions,
}: SpawnedAgentNodeProps) {
  const isSelected = Boolean(
    (node.item.thread_id && node.item.thread_id === selectedDaemonThreadId) ||
      (selectedTaskId &&
        (node.item.task_id === selectedTaskId || node.item.id === selectedTaskId)),
  );
  const canOpen = canOpenSpawnedThread(node.item);
  const meta = renderNodeMeta(node.item);
  const statusReason = getRunStatusReason(node.item);
  const statusChip = getRunStatusChip(node.item);
  const reasonChip = getRunStatusReasonChip(node.item);

  return (
    <div
      data-node-title={node.item.title}
      data-node-depth={depth}
      style={{
        display: "grid",
        gap: "var(--space-2)",
        marginLeft: depth > 0 ? 16 : 0,
        paddingLeft: depth > 0 ? "var(--space-3)" : 0,
        borderLeft: depth > 0 ? "1px solid var(--glass-border)" : "none",
      }}
    >
      <div
        style={{
          border: "1px solid",
          borderColor: isSelected ? "var(--accent)" : "var(--border)",
          borderRadius: "var(--radius-lg)",
          padding: "var(--space-3)",
          background: isSelected ? "rgba(94, 231, 223, 0.08)" : "var(--bg-secondary)",
          display: "grid",
          gap: "var(--space-2)",
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", gap: "var(--space-2)", alignItems: "flex-start" }}>
          <div style={{ display: "grid", gap: 4 }}>
            <div style={{ fontSize: "var(--text-sm)", fontWeight: 700, color: "var(--text-primary)" }}>
              {node.item.title}
            </div>
            {meta && (
              <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", wordBreak: "break-word" }}>
                {meta}
              </div>
            )}
          </div>
          <div style={{ display: "flex", gap: "var(--space-1)", flexWrap: "wrap", justifyContent: "flex-end" }}>
            <StatusChip
              icon={statusChip.icon}
              label={statusChip.label}
              tone={statusChip.tone}
            />
            {isSelected && (
              <StatusChip
                icon="•"
                label="Current"
                tone="accent"
                style={{ borderColor: "var(--accent-soft)" }}
              />
            )}
          </div>
        </div>

        {statusReason && (
          <div
            style={{
              fontSize: "var(--text-xs)",
              color: "var(--text-secondary)",
              background: "color-mix(in srgb, var(--bg-primary) 72%, transparent)",
              border: "1px solid var(--border)",
              borderRadius: "var(--radius-md)",
              padding: "6px 8px",
              wordBreak: "break-word",
              display: "flex",
              alignItems: "flex-start",
              gap: "var(--space-2)",
              flexWrap: "wrap",
            }}
          >
            {reasonChip ? (
              <StatusChip
                icon={reasonChip.icon}
                label={reasonChip.label}
                tone={reasonChip.tone}
                style={{ fontSize: 10, padding: "1px 6px" }}
              />
            ) : null}
            <span>{statusReason}</span>
          </div>
        )}

        <div style={{ display: "flex", justifyContent: "space-between", gap: "var(--space-2)", alignItems: "center", flexWrap: "wrap" }}>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)" }}>
            {node.item.thread_id ? "Chat ready" : "Waiting for thread"}
          </div>
          {renderActions ? (
            renderActions({
              node,
              canOpen,
              openSpawnedThread: () => void openSpawnedThread(node.item),
            })
          ) : (
            <ActionButton
              disabled={!canOpen}
              onClick={canOpen ? () => void openSpawnedThread(node.item) : undefined}
            >
              <span aria-label={`Open chat for ${node.item.title}`}>Open Chat</span>
            </ActionButton>
          )}
        </div>
      </div>

      {node.children.map((child) => (
        <SpawnedAgentNode
          key={child.item.id}
          node={child}
          depth={depth + 1}
          selectedDaemonThreadId={selectedDaemonThreadId}
          selectedTaskId={selectedTaskId}
          canOpenSpawnedThread={canOpenSpawnedThread}
          openSpawnedThread={openSpawnedThread}
          renderActions={renderActions}
        />
      ))}
    </div>
  );
}

export function SpawnedAgentsPanel({
  tree,
  selectedDaemonThreadId,
  canGoBackThread,
  threadNavigationDepth,
  backThreadTitle,
  canOpenSpawnedThread,
  openSpawnedThread,
  goBackThread,
}: SpawnedAgentsPanelProps) {
  const backLabel = backThreadTitle ? `Back to ${backThreadTitle}` : "Back";

  return (
    <aside
      style={{
        width: 300,
        minWidth: 260,
        maxWidth: 340,
        height: "100%",
        border: "1px solid var(--border)",
        borderRadius: "var(--radius-xl)",
        background: "var(--bg-primary)",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          padding: "var(--space-3)",
          borderBottom: "1px solid var(--border)",
          background: "var(--bg-secondary)",
          display: "grid",
          gap: "var(--space-2)",
        }}
      >
        <div>
          <div style={{ fontSize: "var(--text-sm)", fontWeight: 700 }}>
            Spawned Agents
          </div>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", marginTop: 2 }}>
            {threadNavigationDepth > 0
              ? `${threadNavigationDepth} hop history`
              : "Open child threads without leaving the transcript."}
          </div>
        </div>
        <ActionButton
          disabled={!canGoBackThread}
          onClick={canGoBackThread ? goBackThread : undefined}
        >
          {backLabel}
        </ActionButton>
      </div>

      <div
        style={{
          flex: 1,
          minHeight: 0,
          overflow: "auto",
          padding: "var(--space-3)",
          display: "grid",
          gap: "var(--space-3)",
        }}
      >
        {!tree && (
          <EmptyPanel message="No spawned agents for this thread yet." />
        )}

        {tree?.anchor && (
          <SpawnedAgentNode
            node={tree.anchor}
            depth={0}
            selectedDaemonThreadId={selectedDaemonThreadId}
            selectedTaskId={null}
            canOpenSpawnedThread={canOpenSpawnedThread}
            openSpawnedThread={openSpawnedThread}
          />
        )}

        {tree?.roots.map((root) => (
          <SpawnedAgentNode
            key={root.item.id}
            node={root}
            depth={0}
            selectedDaemonThreadId={selectedDaemonThreadId}
            selectedTaskId={null}
            canOpenSpawnedThread={canOpenSpawnedThread}
            openSpawnedThread={openSpawnedThread}
          />
        ))}
      </div>
    </aside>
  );
}
