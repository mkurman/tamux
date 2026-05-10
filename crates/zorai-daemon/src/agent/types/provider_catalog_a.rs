use super::*;

pub const OPENAI_MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "gpt-5.5",
        name: "GPT-5.5",
        context_window: 1_000_000,
        modalities: MULTIMODAL,
    },
    ModelDefinition {
        id: "gpt-5.4",
        name: "GPT-5.4",
        context_window: 1_000_000,
        modalities: MULTIMODAL,
    },
    ModelDefinition {
        id: "gpt-5.4-mini",
        name: "GPT-5.4 Mini",
        context_window: 400000,
        modalities: TEXT_IMAGE,
    },
    ModelDefinition {
        id: "gpt-5.4-nano",
        name: "GPT-5.4 Nano",
        context_window: 400000,
        modalities: TEXT_IMAGE,
    },
    ModelDefinition {
        id: "gpt-5.3-codex",
        name: "GPT-5.3 Codex",
        context_window: 400000,
        modalities: TEXT_IMAGE,
    },
    ModelDefinition {
        id: "gpt-5.2-codex",
        name: "GPT-5.2 Codex",
        context_window: 400000,
        modalities: TEXT_IMAGE,
    },
    ModelDefinition {
        id: "gpt-5.2",
        name: "GPT-5.2",
        context_window: 400000,
        modalities: TEXT_IMAGE,
    },
    ModelDefinition {
        id: "gpt-5.1-codex-max",
        name: "GPT-5.1 Codex Max",
        context_window: 400000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "gpt-5.1-codex",
        name: "GPT-5.1 Codex",
        context_window: 400000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "gpt-5.1-codex-mini",
        name: "GPT-5.1 Codex Mini",
        context_window: 400000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "gpt-5.1",
        name: "GPT-5.1",
        context_window: 400000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "gpt-5-codex",
        name: "GPT-5 Codex",
        context_window: 400000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "gpt-5-codex-mini",
        name: "GPT-5 Codex Mini",
        context_window: 200000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "gpt-5",
        name: "GPT-5",
        context_window: 400000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "codex-mini-latest",
        name: "Codex Mini Latest",
        context_window: 200000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "o3",
        name: "o3",
        context_window: 200000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "o4-mini",
        name: "o4 Mini",
        context_window: 200000,
        modalities: TEXT_ONLY,
    },
];

pub const QWEN_MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "qwen-max",
        name: "Qwen Max",
        context_window: 32768,
        modalities: TEXT_IMAGE,
    },
    ModelDefinition {
        id: "qwen-plus",
        name: "Qwen Plus",
        context_window: 32768,
        modalities: TEXT_IMAGE,
    },
    ModelDefinition {
        id: "qwen-turbo",
        name: "Qwen Turbo",
        context_window: 8192,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "qwen-long",
        name: "Qwen Long",
        context_window: 1000000,
        modalities: TEXT_ONLY,
    },
];

pub const ZAI_MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "glm-5.1",
        name: "GLM-5.1",
        context_window: 204800,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-5",
        name: "GLM-5",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4-plus",
        name: "GLM-4 Plus",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4",
        name: "GLM-4",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4-air",
        name: "GLM-4 Air",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4-flash",
        name: "GLM-4 Flash",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
];

pub const ZAI_CODING_MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "glm-5",
        name: "GLM-5",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-5.1",
        name: "GLM-5.1",
        context_window: 204800,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4-plus",
        name: "GLM-4 Plus",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4",
        name: "GLM-4",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4-air",
        name: "GLM-4 Air",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "glm-4-flash",
        name: "GLM-4 Flash",
        context_window: 128000,
        modalities: TEXT_ONLY,
    },
];

pub const KIMI_MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "moonshot-v1-8k",
        name: "Moonshot V1 8K",
        context_window: 8192,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "moonshot-v1-32k",
        name: "Moonshot V1 32K",
        context_window: 32768,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "moonshot-v1-128k",
        name: "Moonshot V1 128K",
        context_window: 131072,
        modalities: TEXT_ONLY,
    },
];

pub const KIMI_CODING_MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "kimi-for-coding",
        name: "Kimi for Coding",
        context_window: 262144,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "kimi-k2.6",
        name: "Kimi K2.6",
        context_window: 262144,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "kimi-k2.5",
        name: "Kimi K2.5",
        context_window: 262144,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "kimi-k2-turbo-preview",
        name: "Kimi K2 Turbo Preview",
        context_window: 262144,
        modalities: TEXT_ONLY,
    },
];

pub const MINIMAX_MODELS: &[ModelDefinition] = &[
    ModelDefinition {
        id: "MiniMax-M2.7",
        name: "MiniMax M2.7",
        context_window: 205000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "MiniMax-M2.5",
        name: "MiniMax M2.5",
        context_window: 205000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "MiniMax-M2.5-highspeed",
        name: "MiniMax M2.5 High Speed",
        context_window: 205000,
        modalities: TEXT_ONLY,
    },
    ModelDefinition {
        id: "MiniMax-M1-80k",
        name: "MiniMax M1 80K",
        context_window: 80000,
        modalities: TEXT_ONLY,
    },
];

pub const NVIDIA_MODELS: &[ModelDefinition] = &[ModelDefinition {
    id: "minimaxai/minimax-m2.7",
    name: "MiniMax M2.7",
    context_window: 205000,
    modalities: TEXT_ONLY,
}];
