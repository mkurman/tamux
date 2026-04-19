import type {
  AgentProviderConfig,
  AgentProviderId,
  ApiTransportMode,
  ApiType,
  AuthSource,
  ModelDefinition,
  Modality,
  ProviderDefinition,
} from "./types.ts";
import { AGENT_PROVIDER_IDS } from "./types.ts";

type AuthSourceSupportOptions = {
  daemonOwnedAuthAvailable?: boolean;
};

const API_KEY_ONLY_AUTH_SOURCES: AuthSource[] = ["api_key"];
const OPENAI_AUTH_SOURCES: AuthSource[] = ["chatgpt_subscription", "api_key"];
const GITHUB_COPILOT_AUTH_SOURCES: AuthSource[] = ["github_copilot", "api_key"];

const M_MULTI: Modality[] = ["text", "image", "video", "audio"];
const M_TI: Modality[] = ["text", "image"];

const OPENAI_API_MODELS: ModelDefinition[] = [
  { id: "gpt-5.4", name: "GPT-5.4", contextWindow: 1_000_000, modalities: M_MULTI },
  { id: "gpt-5.4-mini", name: "GPT-5.4 Mini", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.4-nano", name: "GPT-5.4 Nano", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.3-codex", name: "GPT-5.3 Codex", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.2-codex", name: "GPT-5.2 Codex", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.2", name: "GPT-5.2", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.1-codex-max", name: "GPT-5.1 Codex Max", contextWindow: 400_000 },
  { id: "gpt-5.1-codex", name: "GPT-5.1 Codex", contextWindow: 400_000 },
  { id: "gpt-5.1-codex-mini", name: "GPT-5.1 Codex Mini", contextWindow: 400_000 },
  { id: "gpt-5.1", name: "GPT-5.1", contextWindow: 400_000 },
  { id: "gpt-5-codex", name: "GPT-5 Codex", contextWindow: 400_000 },
  { id: "gpt-5-codex-mini", name: "GPT-5 Codex Mini", contextWindow: 200_000 },
  { id: "gpt-5", name: "GPT-5", contextWindow: 400_000 },
  { id: "codex-mini-latest", name: "Codex Mini Latest", contextWindow: 200_000 },
  { id: "o3", name: "o3", contextWindow: 200_000 },
  { id: "o4-mini", name: "o4 Mini", contextWindow: 200_000 },
];

const OPENAI_CHATGPT_SUBSCRIPTION_MODELS: ModelDefinition[] = [
  { id: "gpt-5.4", name: "GPT-5.4", contextWindow: 1_000_000 },
  { id: "gpt-5.4-mini", name: "GPT-5.4 Mini", contextWindow: 400_000 },
  { id: "gpt-5.3-codex", name: "GPT-5.3 Codex", contextWindow: 400_000 },
  { id: "gpt-5.2-codex", name: "GPT-5.2 Codex", contextWindow: 400_000 },
  { id: "gpt-5.2", name: "GPT-5.2", contextWindow: 400_000 },
  { id: "gpt-5.1-codex-max", name: "GPT-5.1 Codex Max", contextWindow: 400_000 },
  { id: "gpt-5.1-codex-mini", name: "GPT-5.1 Codex Mini", contextWindow: 400_000 },
];

const ANTHROPIC_MODELS: ModelDefinition[] = [
  { id: "claude-opus-4-7", name: "Claude Opus 4.7", contextWindow: 1_000_000, modalities: M_TI },
  { id: "claude-opus-4-6", name: "Claude Opus 4.6", contextWindow: 1_000_000, modalities: M_TI },
  { id: "claude-opus-4-5-20251101", name: "Claude Opus 4.5", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-opus-4-1-20250805", name: "Claude Opus 4.1", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-opus-4-20250514", name: "Claude Opus 4", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-sonnet-4-6", name: "Claude Sonnet 4.6", contextWindow: 1_000_000, modalities: M_TI },
  { id: "claude-sonnet-4-5-20250929", name: "Claude Sonnet 4.5", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-sonnet-4-20250514", name: "Claude Sonnet 4", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-3-7-sonnet-20250219", name: "Claude Sonnet 3.7", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-haiku-4-5-20251001", name: "Claude Haiku 4.5", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-3-5-haiku-20241022", name: "Claude Haiku 3.5", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-3-opus-20240229", name: "Claude Opus 3", contextWindow: 200_000, modalities: M_TI },
  { id: "claude-3-haiku-20240307", name: "Claude Haiku 3", contextWindow: 200_000, modalities: M_TI },
];

const GITHUB_COPILOT_MODELS: ModelDefinition[] = [
  { id: "claude-haiku-4.5", name: "Claude Haiku 4.5", contextWindow: 160_000, modalities: M_TI },
  { id: "claude-opus-4.5", name: "Claude Opus 4.5", contextWindow: 160_000, modalities: M_TI },
  { id: "claude-opus-4.6", name: "Claude Opus 4.6", contextWindow: 192_000, modalities: M_TI },
  { id: "claude-opus-4.6-fast", name: "Claude Opus 4.6 (fast mode) (Preview)", contextWindow: 192_000, modalities: M_TI },
  { id: "claude-sonnet-4", name: "Claude Sonnet 4", contextWindow: 144_000, modalities: M_TI },
  { id: "claude-sonnet-4.5", name: "Claude Sonnet 4.5", contextWindow: 160_000, modalities: M_TI },
  { id: "claude-sonnet-4.6", name: "Claude Sonnet 4.6", contextWindow: 160_000, modalities: M_TI },
  { id: "gemini-2.5-pro", name: "Gemini 2.5 Pro", contextWindow: 173_000, modalities: M_TI },
  { id: "gemini-3-flash-preview", name: "Gemini 3 Flash (Preview)", contextWindow: 173_000, modalities: M_TI },
  { id: "gemini-3.1-pro-preview", name: "Gemini 3.1 Pro (Preview)", contextWindow: 173_000, modalities: M_TI },
  { id: "gpt-4.1", name: "GPT-4.1", contextWindow: 128_000, modalities: M_TI },
  { id: "gpt-4o", name: "GPT-4o", contextWindow: 128_000, modalities: M_TI },
  { id: "gpt-5-mini", name: "GPT-5 mini", contextWindow: 192_000, modalities: M_TI },
  { id: "gpt-5.1", name: "GPT-5.1", contextWindow: 192_000, modalities: M_TI },
  { id: "gpt-5.1-codex", name: "GPT-5.1-Codex", contextWindow: 256_000, modalities: M_TI },
  { id: "gpt-5.1-codex-max", name: "GPT-5.1-Codex-Max", contextWindow: 256_000, modalities: M_TI },
  { id: "gpt-5.1-codex-mini", name: "GPT-5.1-Codex-Mini (Preview)", contextWindow: 256_000, modalities: M_TI },
  { id: "gpt-5.2", name: "GPT-5.2", contextWindow: 192_000, modalities: M_TI },
  { id: "gpt-5.2-codex", name: "GPT-5.2-Codex", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.3-codex", name: "GPT-5.3-Codex", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.4", name: "GPT-5.4", contextWindow: 400_000, modalities: M_TI },
  { id: "gpt-5.4-mini", name: "GPT-5.4 mini", contextWindow: 400_000, modalities: M_TI },
  { id: "grok-code-fast-1", name: "Grok Code Fast 1", contextWindow: 173_000 },
  { id: "raptor-mini", name: "Raptor mini (Preview)", contextWindow: 264_000, modalities: M_TI },
  { id: "goldeneye", name: "Goldeneye", contextWindow: 524_000, modalities: M_TI },
];

const ZAI_MODELS: ModelDefinition[] = [
  { id: "glm-5.1", name: "GLM-5.1", contextWindow: 204800 },
  { id: "glm-5", name: "GLM-5", contextWindow: 128000 },
  { id: "glm-4-plus", name: "GLM-4 Plus", contextWindow: 128000 },
  { id: "glm-4", name: "GLM-4", contextWindow: 128000 },
];

const ARCEE_MODELS: ModelDefinition[] = [
  { id: "trinity-large-thinking", name: "Trinity Large Thinking", contextWindow: 256_000 },
];

const NVIDIA_MODELS: ModelDefinition[] = [
  { id: "minimaxai/minimax-m2.7", name: "MiniMax M2.7", contextWindow: 205_000 },
];

const KIMI_MODELS: ModelDefinition[] = [
  { id: "moonshot-v1-8k", name: "Moonshot V1 8K", contextWindow: 8192 },
  { id: "moonshot-v1-32k", name: "Moonshot V1 32K", contextWindow: 32768 },
  { id: "moonshot-v1-128k", name: "Moonshot V1 128K", contextWindow: 131072 },
];

const KIMI_CODING_MODELS: ModelDefinition[] = [
  { id: "kimi-for-coding", name: "Kimi for Coding", contextWindow: 262144 },
  { id: "kimi-k2.5", name: "Kimi K2.5", contextWindow: 262144 },
];

const MINIMAX_MODELS: ModelDefinition[] = [
  { id: "MiniMax-M2.7", name: "MiniMax M2.7", contextWindow: 205000 },
  { id: "MiniMax-M2.5", name: "MiniMax M2.5", contextWindow: 205000 },
];

const ALIBABA_CODING_MODELS: ModelDefinition[] = [
  { id: "qwen3-coder-plus", name: "Qwen3 Coder Plus", contextWindow: 997952 },
  { id: "qwen3-coder-next", name: "Qwen3 Coder Next", contextWindow: 204800 },
  { id: "qwen3.6-plus", name: "Qwen3.6 Plus", contextWindow: 983616 },
  { id: "qwen3.5-plus", name: "Qwen3.5 Plus", contextWindow: 983616 },
  { id: "glm-5", name: "GLM-5", contextWindow: 202752 },
  { id: "kimi-k2.5", name: "Kimi K2.5", contextWindow: 262144 },
  { id: "MiniMax-M2.5", name: "MiniMax M2.5", contextWindow: 205000 },
];

const XIAOMI_MIMO_TOKEN_PLAN_MODELS: ModelDefinition[] = [
  { id: "mimo-v2-pro", name: "MiMo V2 Pro", contextWindow: 1_000_000 },
  { id: "mimo-v2-omni", name: "MiMo V2 Omni", contextWindow: 256_000, modalities: M_MULTI },
];

const NOUS_PORTAL_MODELS: ModelDefinition[] = [
  { id: "nousresearch/hermes-4-70b", name: "Nous: Hermes 4 70B", contextWindow: 131_072 },
  { id: "nousresearch/hermes-4-405b", name: "Nous: Hermes 4 405B", contextWindow: 131_072 },
  { id: "nousresearch/hermes-3-llama-3.1-70b", name: "Nous: Hermes 3 70B Instruct", contextWindow: 131_072 },
  { id: "nousresearch/hermes-3-llama-3.1-405b", name: "Nous: Hermes 3 405B Instruct", contextWindow: 131_072 },
];

const OPENCODE_ZEN_MODELS: ModelDefinition[] = [
  { id: "claude-opus-4-6", name: "Claude Opus 4.6", contextWindow: 200000, modalities: M_TI },
  { id: "claude-sonnet-4-5", name: "Claude Sonnet 4.5", contextWindow: 200000, modalities: M_TI },
  { id: "claude-sonnet-4", name: "Claude Sonnet 4", contextWindow: 200000, modalities: M_TI },
  { id: "gpt-5.4", name: "GPT-5.4", contextWindow: 128000, modalities: M_MULTI },
  { id: "gpt-5.3-codex", name: "GPT-5.3 Codex", contextWindow: 128000, modalities: M_TI },
  { id: "minimax-m2.5", name: "MiniMax M2.5", contextWindow: 205000 },
  { id: "glm-5", name: "GLM-5", contextWindow: 128000 },
  { id: "kimi-k2.5", name: "Kimi K2.5", contextWindow: 262144 },
];

const OPENCODE_GO_MODELS: ModelDefinition[] = [
  { id: "glm-5.1", name: "GLM-5.1", contextWindow: 204800 },
  { id: "glm-5", name: "GLM-5", contextWindow: 202752 },
  { id: "kimi-k2.5", name: "Kimi K2.5", contextWindow: 262144 },
  { id: "mimo-v2-pro", name: "MiMo V2 Pro", contextWindow: 1_000_000 },
  { id: "mimo-v2-omni", name: "MiMo V2 Omni", contextWindow: 256_000, modalities: M_MULTI },
  { id: "minimax-m2.5", name: "MiniMax M2.5", contextWindow: 205000 },
  { id: "minimax-m2.7", name: "MiniMax M2.7", contextWindow: 205000 },
  { id: "qwen3.5-plus", name: "Qwen3.5 Plus", contextWindow: 983616 },
  { id: "qwen3.6-plus", name: "Qwen3.6 Plus", contextWindow: 983616 },
];

const GROQ_MODELS: ModelDefinition[] = [
  { id: "llama-3.3-70b-versatile", name: "Llama 3.3 70B Versatile", contextWindow: 128000 },
  { id: "llama-3.1-8b-instant", name: "Llama 3.1 8B", contextWindow: 128000 },
];

const OLLAMA_MODELS: ModelDefinition[] = [
  { id: "llama3.1", name: "Llama 3.1", contextWindow: 128000 },
  { id: "llama3.2", name: "Llama 3.2", contextWindow: 128000 },
  { id: "qwen2.5", name: "Qwen 2.5", contextWindow: 128000 },
  { id: "codellama", name: "Code Llama", contextWindow: 16384 },
];

const EMPTY_MODELS: ModelDefinition[] = [];
const CHAT_ONLY_TRANSPORTS: ApiTransportMode[] = ["chat_completions"];
const RESPONSES_AND_CHAT_TRANSPORTS: ApiTransportMode[] = ["responses", "chat_completions"];
const NATIVE_AND_CHAT_TRANSPORTS: ApiTransportMode[] = ["native_assistant", "chat_completions"];
export const DEFAULT_PROVIDER_CONTEXT_WINDOW = 128_000;
export const DEFAULT_CUSTOM_MODEL_CONTEXT_WINDOW = 264_000;

function normalizeModelLookupValue(value: string | undefined): string {
  return (value ?? "").trim().toLowerCase();
}

export function normalizeAgentProviderId(value: unknown): AgentProviderId {
  if (typeof value !== "string") {
    return "openai";
  }
  const normalized = value.trim() as AgentProviderId;
  return AGENT_PROVIDER_IDS.includes(normalized) ? normalized : "openai";
}

export function normalizeApiTransport(
  providerId: AgentProviderId,
  value: unknown,
): ApiTransportMode {
  const normalized = value === "native_assistant"
    ? "native_assistant"
    : value === "chat_completions"
      ? "chat_completions"
      : "responses";
  return getSupportedApiTransports(providerId).includes(normalized)
    ? normalized
    : getDefaultApiTransport(providerId);
}

export function normalizeAuthSource(
  providerId: AgentProviderId,
  value: unknown,
  options?: AuthSourceSupportOptions,
): AuthSource {
  const normalized = value === "chatgpt_subscription"
    ? "chatgpt_subscription"
    : value === "github_copilot"
      ? "github_copilot"
      : "api_key";
  return getSupportedAuthSources(providerId, options).includes(normalized)
    ? normalized
    : getDefaultAuthSource(providerId, options);
}

export function normalizeProviderConfig(
  providerId: AgentProviderId,
  fallback: AgentProviderConfig,
  value: Partial<AgentProviderConfig> | undefined,
): AgentProviderConfig {
  const auth_source = normalizeAuthSource(providerId, value?.auth_source ?? fallback.auth_source);
  const requestedModel = typeof value?.model === "string" ? value.model.trim() : fallback.model.trim();
  const supportedModels = getProviderModels(providerId, auth_source);
  const matchesKnownModel = requestedModel && supportedModels.some((entry) => entry.id === requestedModel);
  const model = requestedModel
    ? requestedModel
    : getDefaultModelForProvider(providerId, auth_source);
  const custom_model_name = typeof value?.custom_model_name === "string"
    ? value.custom_model_name.trim()
    : "";
  const base_url = !providerUsesConfigurableBaseUrl(providerId)
    ? fallback.base_url
    : (value?.base_url ?? fallback.base_url);

  return {
    ...fallback,
    ...(value ?? {}),
    base_url,
    model,
    custom_model_name: matchesKnownModel ? "" : (custom_model_name || (requestedModel && !matchesKnownModel ? requestedModel : "")),
    assistant_id: typeof value?.assistant_id === "string" ? value.assistant_id : fallback.assistant_id,
    api_transport: normalizeApiTransport(providerId, value?.api_transport ?? fallback.api_transport),
    auth_source,
    context_window_tokens:
      typeof value?.context_window_tokens === "number" && Number.isFinite(value.context_window_tokens)
        ? Math.max(1000, Math.trunc(value.context_window_tokens))
        : null,
  };
}

export function getModelModalities(model: ModelDefinition | undefined): Modality[] {
  return model?.modalities ?? ["text"];
}

export function modelSupports(model: ModelDefinition | undefined, modality: Modality): boolean {
  return getModelModalities(model).includes(modality);
}

export const PROVIDER_DEFINITIONS: ProviderDefinition[] = [
  { id: "featherless", name: "Featherless", defaultBaseUrl: "https://api.featherless.ai/v1", defaultModel: "meta-llama/Llama-3.3-70B-Instruct", apiType: "openai", authMethod: "bearer", models: [], supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "anthropic", name: "Anthropic", defaultBaseUrl: "https://api.anthropic.com", defaultModel: "claude-opus-4-7", apiType: "anthropic", authMethod: "x-api-key", models: ANTHROPIC_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "openai", name: "OpenAI / ChatGPT", defaultBaseUrl: "https://api.openai.com/v1", defaultModel: "gpt-5.4", apiType: "openai", authMethod: "bearer", models: OPENAI_API_MODELS, supportsModelFetch: true, supportedTransports: RESPONSES_AND_CHAT_TRANSPORTS, defaultTransport: "responses", supportedAuthSources: OPENAI_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: true },
  { id: "azure-openai", name: "Azure OpenAI", defaultBaseUrl: "https://YOUR-RESOURCE-NAME.openai.azure.com/openai/v1", defaultModel: "", apiType: "openai", authMethod: "bearer", models: EMPTY_MODELS, supportsModelFetch: true, supportedTransports: RESPONSES_AND_CHAT_TRANSPORTS, defaultTransport: "responses", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: true },
  { id: "github-copilot", name: "GitHub Copilot", defaultBaseUrl: "https://api.githubcopilot.com", defaultModel: "gpt-4.1", apiType: "openai", authMethod: "bearer", models: GITHUB_COPILOT_MODELS, supportsModelFetch: true, supportedTransports: RESPONSES_AND_CHAT_TRANSPORTS, defaultTransport: "responses", supportedAuthSources: GITHUB_COPILOT_AUTH_SOURCES, defaultAuthSource: "github_copilot", supportsResponseContinuity: true },
  { id: "qwen", name: "Qwen", defaultBaseUrl: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1", defaultModel: "qwen-max", apiType: "openai", authMethod: "bearer", models: ALIBABA_CODING_MODELS, supportsModelFetch: true, supportedTransports: NATIVE_AND_CHAT_TRANSPORTS, defaultTransport: "native_assistant", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", nativeTransportKind: "alibaba_assistant_api", nativeBaseUrl: "https://dashscope-intl.aliyuncs.com/api/v1", supportsResponseContinuity: false },
  { id: "qwen-deepinfra", name: "Qwen (DeepInfra)", defaultBaseUrl: "https://api.deepinfra.com/v1/openai", defaultModel: "Qwen/Qwen2.5-72B-Instruct", apiType: "openai", authMethod: "bearer", models: [], supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "kimi", name: "Kimi (Moonshot)", defaultBaseUrl: "https://api.moonshot.ai/v1", defaultModel: "moonshot-v1-32k", apiType: "openai", authMethod: "bearer", models: KIMI_MODELS, supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "kimi-coding-plan", name: "Kimi Coding Plan", defaultBaseUrl: "https://api.kimi.com/coding/v1", defaultModel: "kimi-for-coding", apiType: "openai", authMethod: "bearer", models: KIMI_CODING_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "z.ai", name: "Z.AI (GLM)", defaultBaseUrl: "https://api.z.ai/api/paas/v4", defaultModel: "glm-4-plus", apiType: "openai", authMethod: "bearer", models: ZAI_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "z.ai-coding-plan", name: "Z.AI Coding Plan", defaultBaseUrl: "https://api.z.ai/api/coding/paas/v4", defaultModel: "glm-5", apiType: "openai", authMethod: "bearer", models: ZAI_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "arcee", name: "Arcee", defaultBaseUrl: "https://api.arcee.ai/api/v1", defaultModel: "trinity-large-thinking", apiType: "openai", authMethod: "bearer", models: ARCEE_MODELS, supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "nvidia", name: "NVIDIA", defaultBaseUrl: "https://integrate.api.nvidia.com/v1", defaultModel: "minimaxai/minimax-m2.7", apiType: "openai", authMethod: "bearer", models: NVIDIA_MODELS, supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "nous-portal", name: "Nous Portal", defaultBaseUrl: "https://inference-api.nousresearch.com/v1", defaultModel: "nousresearch/hermes-4-70b", apiType: "openai", authMethod: "bearer", models: NOUS_PORTAL_MODELS, supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "openrouter", name: "OpenRouter", defaultBaseUrl: "https://openrouter.ai/api/v1", defaultModel: "arcee-ai/trinity-large-thinking", apiType: "openai", authMethod: "bearer", models: [], supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "cerebras", name: "Cerebras", defaultBaseUrl: "https://api.cerebras.ai/v1", defaultModel: "llama-3.3-70b", apiType: "openai", authMethod: "bearer", models: [], supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "together", name: "Together", defaultBaseUrl: "https://api.together.xyz/v1", defaultModel: "meta-llama/Llama-3.3-70B-Instruct-Turbo", apiType: "openai", authMethod: "bearer", models: [], supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "groq", name: "Groq", defaultBaseUrl: "https://api.groq.com/openai/v1", defaultModel: "llama-3.3-70b-versatile", apiType: "openai", authMethod: "bearer", models: GROQ_MODELS, supportsModelFetch: true, supportedTransports: RESPONSES_AND_CHAT_TRANSPORTS, defaultTransport: "responses", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "ollama", name: "Ollama", defaultBaseUrl: "http://localhost:11434/v1", defaultModel: "llama3.1", apiType: "openai", authMethod: "bearer", models: OLLAMA_MODELS, supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "chutes", name: "Chutes", defaultBaseUrl: "https://llm.chutes.ai/v1", defaultModel: "deepseek-ai/DeepSeek-V3", apiType: "openai", authMethod: "bearer", models: [], supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "huggingface", name: "Hugging Face", defaultBaseUrl: "https://api-inference.huggingface.co/v1", defaultModel: "meta-llama/Llama-3.3-70B-Instruct", apiType: "openai", authMethod: "bearer", models: [], supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "minimax", name: "MiniMax", defaultBaseUrl: "https://api.minimax.io/anthropic", defaultModel: "MiniMax-M1-80k", apiType: "anthropic", authMethod: "x-api-key", models: MINIMAX_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "minimax-coding-plan", name: "MiniMax Coding Plan", defaultBaseUrl: "https://api.minimax.io/anthropic", defaultModel: "MiniMax-M2.7", apiType: "anthropic", authMethod: "x-api-key", models: MINIMAX_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "alibaba-coding-plan", name: "Alibaba Coding Plan", defaultBaseUrl: "https://coding-intl.dashscope.aliyuncs.com/v1", defaultModel: "qwen3.6-plus", apiType: "openai", authMethod: "bearer", models: ALIBABA_CODING_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "xiaomi-mimo-token-plan", name: "Xiaomi MiMo Token Plan", defaultBaseUrl: "https://api.xiaomimimo.com/v1", defaultModel: "mimo-v2-pro", apiType: "openai", authMethod: "bearer", models: XIAOMI_MIMO_TOKEN_PLAN_MODELS, supportsModelFetch: false, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "opencode-go", name: "OpenCode Go", defaultBaseUrl: "https://opencode.ai/zen/go/v1", defaultModel: "glm-5.1", apiType: "openai", authMethod: "bearer", models: OPENCODE_GO_MODELS, supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "opencode-zen", name: "OpenCode Zen", defaultBaseUrl: "https://opencode.ai/zen/v1", defaultModel: "claude-sonnet-4-5", apiType: "anthropic", authMethod: "bearer", models: OPENCODE_ZEN_MODELS, supportsModelFetch: true, supportedTransports: CHAT_ONLY_TRANSPORTS, defaultTransport: "chat_completions", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: false },
  { id: "custom", name: "Custom", defaultBaseUrl: "", defaultModel: "", apiType: "openai", authMethod: "bearer", models: EMPTY_MODELS, supportsModelFetch: false, supportedTransports: RESPONSES_AND_CHAT_TRANSPORTS, defaultTransport: "responses", supportedAuthSources: API_KEY_ONLY_AUTH_SOURCES, defaultAuthSource: "api_key", supportsResponseContinuity: true },
];

export function getProviderDefinition(id: AgentProviderId): ProviderDefinition | undefined {
  return PROVIDER_DEFINITIONS.find((provider) => provider.id === id);
}

export function getSupportedApiTransports(providerId: AgentProviderId): ApiTransportMode[] {
  return getProviderDefinition(providerId)?.supportedTransports ?? CHAT_ONLY_TRANSPORTS;
}

export function getDefaultApiTransport(providerId: AgentProviderId): ApiTransportMode {
  return getProviderDefinition(providerId)?.defaultTransport ?? "chat_completions";
}

export function getSupportedAuthSources(
  providerId: AgentProviderId,
  options?: AuthSourceSupportOptions,
): AuthSource[] {
  const supported = getProviderDefinition(providerId)?.supportedAuthSources ?? API_KEY_ONLY_AUTH_SOURCES;
  if (providerId === "openai" && options?.daemonOwnedAuthAvailable === false) {
    return supported.filter((source) => source !== "chatgpt_subscription");
  }
  return supported;
}

export function getDefaultAuthSource(
  providerId: AgentProviderId,
  options?: AuthSourceSupportOptions,
): AuthSource {
  const defaultSource = getProviderDefinition(providerId)?.defaultAuthSource ?? "api_key";
  return getSupportedAuthSources(providerId, options).includes(defaultSource)
    ? defaultSource
    : "api_key";
}

export function getProviderModels(
  providerId: AgentProviderId,
  auth_source?: AuthSource,
): ModelDefinition[] {
  if (providerId === "openai" && auth_source === "chatgpt_subscription") {
    return OPENAI_CHATGPT_SUBSCRIPTION_MODELS;
  }
  return getProviderDefinition(providerId)?.models ?? [];
}

export function getDefaultModelForProvider(
  providerId: AgentProviderId,
  auth_source?: AuthSource,
): string {
  const models = getProviderModels(providerId, auth_source);
  if (models.length > 0) {
    return models[0].id;
  }
  return getProviderDefinition(providerId)?.defaultModel ?? "";
}

export function getModelDefinition(
  providerId: AgentProviderId,
  modelId: string,
  auth_source?: AuthSource,
): ModelDefinition | undefined {
  const trimmed = modelId.trim();
  if (!trimmed) {
    return undefined;
  }
  return getProviderModels(providerId, auth_source).find((model) => model.id === trimmed);
}

export function resolveProviderModelDefinition(
  providerId: AgentProviderId,
  auth_source: AuthSource | undefined,
  ...candidates: Array<string | undefined>
): ModelDefinition | undefined {
  const normalizedCandidates = candidates
    .map((candidate) => normalizeModelLookupValue(candidate))
    .filter((candidate, index, all) => candidate.length > 0 && all.indexOf(candidate) === index);

  if (normalizedCandidates.length === 0) {
    return undefined;
  }

  return getProviderModels(providerId, auth_source).find((model) => {
    const normalizedId = normalizeModelLookupValue(model.id);
    const normalizedName = normalizeModelLookupValue(model.name);
    return normalizedCandidates.includes(normalizedId)
      || normalizedCandidates.includes(normalizedName);
  });
}

export function modelUsesContextWindowOverride(
  providerId: AgentProviderId,
  modelId: string,
  custom_model_name?: string,
  auth_source?: AuthSource,
): boolean {
  if (providerId === "custom") {
    return true;
  }

  return Boolean(
    (modelId.trim().length > 0 || (custom_model_name ?? "").trim().length > 0)
    && !resolveProviderModelDefinition(providerId, auth_source, modelId, custom_model_name),
  );
}

export function getEffectiveContextWindow(
  providerId: AgentProviderId,
  config: Pick<AgentProviderConfig, "model" | "custom_model_name" | "context_window_tokens" | "auth_source">,
): number {
  const resolvedModel = resolveProviderModelDefinition(
    providerId,
    config.auth_source,
    config.model,
    config.custom_model_name,
  );
  if (resolvedModel) {
    return resolvedModel.contextWindow;
  }

  if (typeof config.context_window_tokens === "number" && config.context_window_tokens > 0) {
    return Math.max(1000, Math.trunc(config.context_window_tokens));
  }

  return providerId === "custom"
    ? DEFAULT_CUSTOM_MODEL_CONTEXT_WINDOW
    : DEFAULT_PROVIDER_CONTEXT_WINDOW;
}

export function providerSupportsResponseContinuity(providerId: AgentProviderId): boolean {
  return Boolean(getProviderDefinition(providerId)?.supportsResponseContinuity);
}

export function providerUsesConfigurableBaseUrl(providerId: AgentProviderId): boolean {
  return providerId === "custom" || providerId === "azure-openai";
}

function isAlibabaCodingPlanAnthropicBaseUrl(baseUrl: string): boolean {
  const lower = (baseUrl || "").trim().toLowerCase();
  return lower.includes("dashscope.aliyuncs.com") && lower.includes("/apps/anthropic");
}

function opencodeGoUsesAnthropicApi(model: string): boolean {
  const normalized = (model || "").trim().toLowerCase();
  const unprefixed = normalized.startsWith("opencode-go/")
    ? normalized.slice("opencode-go/".length)
    : normalized;
  return unprefixed.startsWith("minimax-m2.");
}

export function getProviderApiType(
  providerId: AgentProviderId,
  model: string,
  configuredUrl: string = "",
): ApiType {
  const definition = getProviderDefinition(providerId);
  if (!definition) {
    return "openai";
  }
  if (providerId === "alibaba-coding-plan" && isAlibabaCodingPlanAnthropicBaseUrl(configuredUrl)) {
    return "anthropic";
  }
  if (definition.anthropicBaseUrl && model.startsWith("claude")) {
    return "anthropic";
  }
  if (providerId === "opencode-go" && opencodeGoUsesAnthropicApi(model)) {
    return "anthropic";
  }
  if (providerId === "opencode-zen" && !model.startsWith("claude")) {
    return "openai";
  }
  return definition.apiType;
}

export function getProviderBaseUrl(
  providerId: AgentProviderId,
  model: string,
  configuredUrl: string,
): string {
  if (configuredUrl) {
    return configuredUrl;
  }

  const definition = getProviderDefinition(providerId);
  if (!definition) {
    return configuredUrl;
  }
  if (definition.anthropicBaseUrl && model.startsWith("claude")) {
    return definition.anthropicBaseUrl;
  }
  return definition.defaultBaseUrl;
}
