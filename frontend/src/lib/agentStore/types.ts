import type { ToolCall, WelesReviewMeta } from "../agentTools";
import type { PaneId, SurfaceId, WorkspaceId } from "../types";

export interface ThreadParticipantState {
  agentId: string;
  agentName: string;
  instruction: string;
  status: "active" | "inactive";
  createdAt: number;
  updatedAt: number;
  deactivatedAt?: number | null;
  lastContributionAt?: number | null;
}

export interface ThreadParticipantSuggestion {
  id: string;
  targetAgentId: string;
  targetAgentName: string;
  instruction: string;
  forceSend?: boolean;
  status: "queued" | "failed";
  createdAt: number;
  updatedAt: number;
  error?: string | null;
}

export interface AgentThread {
  id: string;
  daemonThreadId?: string | null;
  workspaceId: WorkspaceId | null;
  surfaceId: SurfaceId | null;
  paneId: PaneId | null;
  agent_name: string;
  title: string;
  createdAt: number;
  updatedAt: number;
  messageCount: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalTokens: number;
  compactionCount: number;
  lastMessagePreview: string;
  upstreamThreadId?: string | null;
  upstreamTransport?: ApiTransportMode;
  upstreamProvider?: AgentProviderId | null;
  upstreamModel?: string | null;
  upstreamAssistantId?: string | null;
  threadParticipants?: ThreadParticipantState[];
  queuedParticipantSuggestions?: ThreadParticipantSuggestion[];
}

export type AgentRole = "user" | "assistant" | "system" | "tool";

export type AgentProviderId =
  | "featherless"
  | "anthropic"
  | "openai"
  | "azure-openai"
  | "github-copilot"
  | "qwen"
  | "qwen-deepinfra"
  | "kimi"
  | "kimi-coding-plan"
  | "z.ai"
  | "z.ai-coding-plan"
  | "arcee"
  | "nvidia"
  | "nous-portal"
  | "openrouter"
  | "cerebras"
  | "together"
  | "groq"
  | "ollama"
  | "chutes"
  | "huggingface"
  | "minimax"
  | "minimax-coding-plan"
  | "alibaba-coding-plan"
  | "xiaomi-mimo-token-plan"
  | "opencode-go"
  | "opencode-zen"
  | "custom";

export const AGENT_PROVIDER_IDS: AgentProviderId[] = [
  "featherless",
  "anthropic",
  "openai",
  "azure-openai",
  "github-copilot",
  "qwen",
  "qwen-deepinfra",
  "kimi",
  "kimi-coding-plan",
  "z.ai",
  "z.ai-coding-plan",
  "arcee",
  "nvidia",
  "nous-portal",
  "openrouter",
  "cerebras",
  "together",
  "groq",
  "ollama",
  "chutes",
  "huggingface",
  "minimax",
  "minimax-coding-plan",
  "alibaba-coding-plan",
  "xiaomi-mimo-token-plan",
  "opencode-go",
  "opencode-zen",
  "custom",
];

export interface AgentProviderConfig {
  base_url: string;
  model: string;
  custom_model_name: string;
  api_key: string;
  assistant_id: string;
  api_transport: ApiTransportMode;
  auth_source: AuthSource;
  context_window_tokens: number | null;
  custom_modalities?: Modality[];
}

export interface ProviderAuthState {
  provider_id: string;
  provider_name: string;
  authenticated: boolean;
  auth_source: AuthSource;
  model: string;
  base_url: string;
}

export interface SubAgentDefinition {
  id: string;
  name: string;
  provider: string;
  model: string;
  role?: string;
  system_prompt?: string;
  tool_whitelist?: string[];
  tool_blacklist?: string[];
  context_budget_tokens?: number;
  max_duration_secs?: number;
  supervisor_config?: {
    check_interval_secs?: number;
    stuck_timeout_secs?: number;
    max_retries?: number;
    intervention_level?: string;
  };
  enabled: boolean;
  builtin?: boolean;
  immutable_identity?: boolean;
  disable_allowed?: boolean;
  delete_allowed?: boolean;
  protected_reason?: string;
  reasoning_effort?: string;
  created_at: number;
}

export type ApiType = "openai" | "anthropic";
export type AuthMethod = "bearer" | "x-api-key";
export type AuthSource = "api_key" | "chatgpt_subscription" | "github_copilot";
export type ApiTransportMode = "native_assistant" | "responses" | "chat_completions";
export type NativeTransportKind = "alibaba_assistant_api";
export type Modality = "text" | "image" | "video" | "audio";
export type AgentBackend = "daemon" | "openclaw" | "hermes" | "legacy";

export interface ModelDefinition {
  id: string;
  name: string;
  contextWindow: number;
  modalities?: Modality[];
}

export interface ProviderDefinition {
  id: AgentProviderId;
  name: string;
  defaultBaseUrl: string;
  defaultModel: string;
  apiType: ApiType;
  authMethod: AuthMethod;
  models: ModelDefinition[];
  supportsModelFetch: boolean;
  anthropicBaseUrl?: string;
  supportedTransports: ApiTransportMode[];
  defaultTransport: ApiTransportMode;
  supportedAuthSources: AuthSource[];
  defaultAuthSource: AuthSource;
  nativeTransportKind?: NativeTransportKind;
  nativeBaseUrl?: string;
  supportsResponseContinuity: boolean;
}

export interface AgentMessage {
  id: string;
  threadId: string;
  createdAt: number;
  role: AgentRole;
  content: string;
  authorAgentId?: string;
  authorAgentName?: string;
  provider?: string;
  model?: string;
  api_transport?: ApiTransportMode;
  responseId?: string;
  providerFinalResult?: unknown;
  toolCalls?: ToolCall[];
  toolName?: string;
  toolCallId?: string;
  toolArguments?: string;
  toolStatus?: "requested" | "executing" | "done" | "error";
  welesReview?: WelesReviewMeta;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  reasoning?: string;
  reasoningTokens?: number;
  audioTokens?: number;
  videoTokens?: number;
  cost?: number;
  tps?: number;
  isCompactionSummary: boolean;
  messageKind?: "normal" | "compaction_artifact";
  compactionStrategy?: "heuristic" | "weles" | "custom_model";
  compactionPayload?: string;
  pinnedForCompaction?: boolean;
  isStreaming?: boolean;
}

export interface WelesHealthState {
  state: string;
  reason?: string;
  checkedAt: number;
}

export type AgentTodoStatus = "pending" | "in_progress" | "completed" | "blocked";

export interface AgentTodoItem {
  id: string;
  content: string;
  status: AgentTodoStatus;
  position: number;
  stepIndex?: number | null;
  createdAt?: number | null;
  updatedAt?: number | null;
}
