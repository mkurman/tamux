import { describe, expect, it } from "vitest";

import {
  getEffectiveContextWindow,
  getDefaultModelForProvider,
  getProviderApiType,
  getProviderDefinition,
  getProviderModels,
  getModelDefinition,
  getModelModalities,
  normalizeApiTransport,
  normalizeAgentProviderId,
  normalizeProviderConfig,
  providerSupportsAudioTool,
} from "./providers.ts";
import { buildDaemonAgentConfig } from "../agentDaemonConfig.ts";
import { normalizeAudioModelForProviderChange } from "../../components/settings-panel/agentTabHelpers.ts";
import { DEFAULT_AGENT_SETTINGS, normalizeAgentSettingsFromSource } from "./settings.ts";

describe("frontend NVIDIA provider catalog", () => {
  it("registers NVIDIA with hosted defaults and fetch support", () => {
    const nvidia = getProviderDefinition("nvidia");

    expect(nvidia).toBeDefined();
    expect(nvidia?.defaultBaseUrl).toBe("https://integrate.api.nvidia.com/v1");
    expect(nvidia?.defaultModel).toBe("minimaxai/minimax-m2.7");
    expect(nvidia?.supportsModelFetch).toBe(true);
  });

  it("recognizes NVIDIA as a valid provider id", () => {
    expect(normalizeAgentProviderId("nvidia")).toBe("nvidia");
    expect(getDefaultModelForProvider("nvidia")).toBe("minimaxai/minimax-m2.7");
  });
});

describe("frontend xAI provider catalog", () => {
  it("registers xAI with hosted defaults and responses transport", () => {
    const xai = getProviderDefinition("xai" as any);

    expect(xai).toBeDefined();
    expect(xai?.defaultBaseUrl).toBe("https://api.x.ai/v1");
    expect(xai?.defaultModel).toBe("grok-4");
    expect(xai?.supportsModelFetch).toBe(true);
    expect(xai?.defaultTransport).toBe("responses");
    expect(xai?.supportsResponseContinuity).toBe(false);
  });

  it("recognizes xAI as a valid provider id", () => {
    expect(normalizeAgentProviderId("xai")).toBe("xai");
    expect(getDefaultModelForProvider("xai" as any)).toBe("grok-4");
  });
});

describe("frontend xAI audio/provider settings coverage", () => {
  it("flags only supported frontend audio providers as audio-capable", () => {
    expect(providerSupportsAudioTool("openai", "stt")).toBe(true);
    expect(providerSupportsAudioTool("openrouter", "tts")).toBe(true);
    expect(providerSupportsAudioTool("xai", "stt")).toBe(true);
    expect(providerSupportsAudioTool("anthropic", "stt")).toBe(false);
    expect(providerSupportsAudioTool("together", "tts")).toBe(false);
  });

  it("normalizes stale audio models when switching providers to xAI", () => {
    expect(normalizeAudioModelForProviderChange("xai", "stt", "whisper-1")).toBe("grok-4");
    expect(normalizeAudioModelForProviderChange("xai", "tts", "gpt-4o-mini-tts")).toBe("grok-4");
    expect(normalizeAudioModelForProviderChange("xai", "stt", "grok-4")).toBe("grok-4");
  });

  it("covers xAI settings normalization and daemon serialization in a collected test file", () => {
    expect(DEFAULT_AGENT_SETTINGS.xai).toMatchObject({
      base_url: "https://api.x.ai/v1",
      model: "grok-4",
      api_transport: "responses",
    });

    const normalizedSettings = normalizeAgentSettingsFromSource({
      active_provider: "xai",
      xai: {
        base_url: "https://api.x.ai/v1",
        model: "grok-4-voice",
        api_key: "sk-xai",
        api_transport: "chat_completions",
      },
      audio_stt_provider: "xai",
      audio_stt_model: "grok-4-voice",
      audio_tts_provider: "xai",
      audio_tts_model: "grok-4-voice",
    } as any);

    expect(normalizedSettings.active_provider).toBe("xai");
    expect(normalizedSettings.audio_stt_provider).toBe("xai");
    expect(normalizedSettings.audio_tts_provider).toBe("xai");
    expect(normalizedSettings.xai).toMatchObject({
      base_url: "https://api.x.ai/v1",
      model: "grok-4-voice",
      api_key: "sk-xai",
      api_transport: "chat_completions",
    });

    const daemonConfig = buildDaemonAgentConfig({
      ...DEFAULT_AGENT_SETTINGS,
      active_provider: "xai",
      xai: {
        ...DEFAULT_AGENT_SETTINGS.xai,
        api_key: "sk-xai",
        model: "grok-4-voice",
      },
      audio_stt_provider: "xai",
      audio_stt_model: "grok-4-voice",
      audio_tts_provider: "xai",
      audio_tts_model: "grok-4-voice",
    });

    expect(daemonConfig.providers?.xai).toMatchObject({
      base_url: "https://api.x.ai/v1",
      model: "grok-4-voice",
    });
  });
});

describe("frontend Anthropic provider catalog", () => {
  it("registers Anthropic with static defaults", () => {
    const anthropic = getProviderDefinition("anthropic" as any);

    expect(anthropic).toBeDefined();
    expect(anthropic?.defaultBaseUrl).toBe("https://api.anthropic.com");
    expect(anthropic?.defaultModel).toBe("claude-opus-4-7");
    expect(anthropic?.supportsModelFetch).toBe(false);
    expect(anthropic?.models.map((model) => [model.id, model.contextWindow])).toEqual([
      ["claude-opus-4-7", 1_000_000],
      ["claude-opus-4-6", 1_000_000],
      ["claude-opus-4-5-20251101", 200_000],
      ["claude-opus-4-1-20250805", 200_000],
      ["claude-opus-4-20250514", 200_000],
      ["claude-sonnet-4-6", 1_000_000],
      ["claude-sonnet-4-5-20250929", 200_000],
      ["claude-sonnet-4-20250514", 200_000],
      ["claude-3-7-sonnet-20250219", 200_000],
      ["claude-haiku-4-5-20251001", 200_000],
      ["claude-3-5-haiku-20241022", 200_000],
      ["claude-3-opus-20240229", 200_000],
      ["claude-3-haiku-20240307", 200_000],
    ]);
  });

  it("recognizes Anthropic as a valid provider id", () => {
    expect(normalizeAgentProviderId("anthropic")).toBe("anthropic");
    expect(getDefaultModelForProvider("anthropic" as any)).toBe("claude-opus-4-7");
  });
});

describe("frontend Xiaomi MiMo token plan provider catalog", () => {
  it("registers Xiaomi MiMo token plan with static defaults", () => {
    const mimo = getProviderDefinition("xiaomi-mimo-token-plan" as any);

    expect(mimo).toBeDefined();
    expect(mimo?.defaultBaseUrl).toBe("https://api.xiaomimimo.com/v1");
    expect(mimo?.defaultModel).toBe("mimo-v2-pro");
    expect(mimo?.supportsModelFetch).toBe(false);
    expect(mimo?.models.map((model) => [model.id, model.contextWindow])).toEqual([
      ["mimo-v2-pro", 1_000_000],
      ["mimo-v2-omni", 256_000],
    ]);
  });

  it("recognizes Xiaomi MiMo token plan as a valid provider id", () => {
    expect(normalizeAgentProviderId("xiaomi-mimo-token-plan")).toBe("xiaomi-mimo-token-plan");
    expect(getDefaultModelForProvider("xiaomi-mimo-token-plan" as any)).toBe("mimo-v2-pro");
  });
});

describe("frontend Nous Portal provider catalog", () => {
  it("registers Nous Portal with fetchable defaults", () => {
    const nous = getProviderDefinition("nous-portal" as any);

    expect(nous).toBeDefined();
    expect(nous?.defaultBaseUrl).toBe("https://inference-api.nousresearch.com/v1");
    expect(nous?.defaultModel).toBe("nousresearch/hermes-4-70b");
    expect(nous?.supportsModelFetch).toBe(true);
    expect(nous?.models.map((model) => [model.id, model.contextWindow])).toEqual([
      ["nousresearch/hermes-4-70b", 131_072],
      ["nousresearch/hermes-4-405b", 131_072],
      ["nousresearch/hermes-3-llama-3.1-70b", 131_072],
      ["nousresearch/hermes-3-llama-3.1-405b", 131_072],
    ]);
  });

  it("recognizes Nous Portal as a valid provider id", () => {
    expect(normalizeAgentProviderId("nous-portal")).toBe("nous-portal");
    expect(getDefaultModelForProvider("nous-portal" as any)).toBe("nousresearch/hermes-4-70b");
  });
});

describe("frontend Azure OpenAI provider catalog", () => {
  it("registers Azure OpenAI with OpenAI-compatible defaults", () => {
    const azure = getProviderDefinition("azure-openai" as any);

    expect(azure).toBeDefined();
    expect(azure?.defaultBaseUrl).toBe(
      "https://YOUR-RESOURCE-NAME.openai.azure.com/openai/v1",
    );
    expect(azure?.defaultModel).toBe("");
    expect(azure?.supportsModelFetch).toBe(true);
    expect(azure?.defaultTransport).toBe("responses");
    expect(azure?.supportedAuthSources).toEqual(["api_key"]);
    expect(azure?.supportsResponseContinuity).toBe(true);
  });

  it("recognizes Azure OpenAI as a valid provider id", () => {
    expect(normalizeAgentProviderId("azure-openai")).toBe("azure-openai");
    expect(getDefaultModelForProvider("azure-openai" as any)).toBe("");
  });

  it("preserves the configured resource-specific base URL", () => {
    const normalized = normalizeProviderConfig(
      "azure-openai" as any,
      {
        base_url: "https://YOUR-RESOURCE-NAME.openai.azure.com/openai/v1",
        model: "",
        custom_model_name: "",
        api_key: "",
        assistant_id: "",
        api_transport: "responses",
        auth_source: "api_key",
        context_window_tokens: null,
      },
      {
        base_url: "https://my-real-resource.openai.azure.com/openai/v1",
        model: "deployment-name",
      },
    );

    expect(normalized.base_url).toBe("https://my-real-resource.openai.azure.com/openai/v1");
    expect(normalized.model).toBe("deployment-name");
  });
});

describe("frontend curated media provider catalog", () => {
  it("keeps representative modalities aligned with the curated matrix", () => {
    expect(getModelModalities(getModelDefinition("openai", "gpt-5.4"))).toEqual([
      "text",
      "image",
      "video",
      "audio",
    ]);
    expect(
      getModelModalities(getModelDefinition("anthropic", "claude-opus-4-7")),
    ).toEqual(["text", "image"]);
    expect(
      getModelModalities(getModelDefinition("xiaomi-mimo-token-plan", "mimo-v2-omni")),
    ).toEqual(["text", "image", "video", "audio"]);
    expect(getModelModalities(getModelDefinition("arcee", "trinity-large-thinking"))).toEqual([
      "text",
    ]);
  });

  it("keeps first-party default model resolution aligned with the provider defaults", () => {
    expect(getDefaultModelForProvider("qwen")).toBe("qwen-max");
    expect(getDefaultModelForProvider("kimi")).toBe("moonshot-v1-32k");
    expect(getDefaultModelForProvider("z.ai")).toBe("glm-4-plus");
    expect(getDefaultModelForProvider("z.ai-coding-plan")).toBe("glm-5");
    expect(getDefaultModelForProvider("alibaba-coding-plan")).toBe("qwen3.6-plus");
  });

  it("removes stale frontend catalog drift for z.ai, kimi coding, and alibaba coding", () => {
    expect(getProviderModels("qwen").map((model) => [model.id, getModelModalities(model)])).toEqual([
      ["qwen-max", ["text", "image"]],
      ["qwen-plus", ["text", "image"]],
      ["qwen-turbo", ["text"]],
      ["qwen-long", ["text"]],
    ]);
    expect(getProviderModels("z.ai").map((model) => model.id)).toEqual([
      "glm-4-plus",
      "glm-5.1",
      "glm-5",
      "glm-4",
      "glm-4-air",
      "glm-4-flash",
    ]);
    expect(getProviderModels("z.ai-coding-plan").map((model) => model.id)).toEqual([
      "glm-5",
      "glm-5.1",
      "glm-4-plus",
      "glm-4",
      "glm-4-air",
      "glm-4-flash",
    ]);
    expect(getProviderModels("kimi-coding-plan").map((model) => model.id)).toEqual([
      "kimi-for-coding",
      "kimi-k2.5",
      "kimi-k2-turbo-preview",
    ]);
    expect(getProviderModels("alibaba-coding-plan").map((model) => model.id)).toEqual([
      "qwen3.6-plus",
      "qwen3-coder-plus",
      "qwen3-coder-next",
      "glm-5",
      "kimi-k2.5",
      "MiniMax-M2.5",
    ]);
  });

  it("keeps legacy alibaba coding-plan models on the curated context-window path", () => {
    const normalized = normalizeProviderConfig(
      "alibaba-coding-plan" as any,
      {
        base_url: "https://coding-intl.dashscope.aliyuncs.com/v1",
        model: "qwen3.6-plus",
        custom_model_name: "",
        api_key: "",
        assistant_id: "",
        api_transport: "chat_completions",
        auth_source: "api_key",
        context_window_tokens: null,
      },
      {
        model: "qwen3.5-plus",
      },
    );

    expect(normalized.custom_model_name).toBe("");
    expect(getEffectiveContextWindow("alibaba-coding-plan", normalized)).toBe(983_616);
  });
});

describe("frontend GitHub Copilot provider routing", () => {
  it("keeps Claude models on the OpenAI provider api type", () => {
    expect(
      getProviderApiType(
        "github-copilot",
        "claude-sonnet-4.6",
        "https://api.githubcopilot.com",
      ),
    ).toBe("openai");
  });

  it("exposes anthropic_messages as an explicit selectable transport", () => {
    const copilot = getProviderDefinition("github-copilot");

    expect(copilot?.supportedTransports).toContain("anthropic_messages");
    expect(normalizeApiTransport("github-copilot", "anthropic_messages")).toBe(
      "anthropic_messages",
    );
  });
});
