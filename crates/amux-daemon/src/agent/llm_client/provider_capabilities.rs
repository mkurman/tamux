//! Provider/model capability matrix.
//!
//! Single source of truth for what each LLM provider/model combination supports.
//! Used by the runtime to select the right API mode (e.g. strict json_schema vs
//! json_object fallback) without scattering substring checks across call sites.

use amux_shared::providers;

/// Capabilities reported for a specific provider+model combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelCapabilities {
    /// Native JSON-schema structured output (strict mode).
    pub structured_output: bool,
    /// Tool-call / function-calling API.
    pub tool_use: bool,
    /// Image / vision input.
    pub vision: bool,
    /// SSE streaming.
    pub streaming: bool,
}

impl ModelCapabilities {
    const fn all_false() -> Self {
        Self {
            structured_output: false,
            tool_use: false,
            vision: false,
            streaming: false,
        }
    }
}

/// Conservative fallback used for unknown providers or unknown models on known providers.
const FALLBACK: ModelCapabilities = ModelCapabilities {
    structured_output: false,
    tool_use: true,
    vision: false,
    streaming: true,
};

// ---------------------------------------------------------------------------
// Per-provider helpers
// ---------------------------------------------------------------------------

/// OpenAI model capabilities.
///
/// Sources:
/// - Structured output (strict json_schema): gpt-4o, gpt-4.1, gpt-5, o-series
///   <https://platform.openai.com/docs/guides/structured-outputs>
/// - Vision: gpt-4o, gpt-4.1, gpt-5 families
/// - Legacy gpt-3.5 / text-* models: no strict schema, json_object only
fn openai_capabilities(model: &str) -> ModelCapabilities {
    let is_modern = model.contains("gpt-4o")
        || model.contains("gpt-4.1")
        || model.contains("gpt-5")
        || model.starts_with('o');

    let is_legacy_no_tools = model.starts_with("text-");

    if is_legacy_no_tools {
        return ModelCapabilities::all_false();
    }

    let vision = model.contains("gpt-4o") || model.contains("gpt-4.1") || model.contains("gpt-5");

    ModelCapabilities {
        structured_output: is_modern,
        tool_use: true,
        vision,
        streaming: true,
    }
}

/// Anthropic model capabilities.
///
/// Sources:
/// - Structured output via tool-use API: claude-3+, claude-sonnet-4+, claude-opus-4+,
///   claude-haiku-4+  <https://docs.anthropic.com/en/docs/tool-use>
/// - Legacy claude-2 / claude-instant: no tool use, no structured output
fn anthropic_capabilities(model: &str) -> ModelCapabilities {
    let is_modern = model.contains("claude-3")
        || model.contains("claude-sonnet-4")
        || model.contains("claude-opus-4")
        || model.contains("claude-haiku-4");

    if is_modern {
        ModelCapabilities {
            structured_output: true,
            tool_use: true,
            vision: true,
            streaming: true,
        }
    } else {
        // claude-2*, claude-instant-* and any unrecognised claude variant
        ModelCapabilities::all_false()
    }
}

/// Google Gemini model capabilities.
///
/// Sources:
/// - Structured output + tools: gemini-1.5+, gemini-2+
///   <https://ai.google.dev/gemini-api/docs/structured-output>
/// - Older gemini-1.0: no structured output, no vision in JSON mode
fn google_capabilities(model: &str) -> ModelCapabilities {
    let is_modern = model.contains("gemini-1.5") || model.contains("gemini-2");

    if is_modern {
        ModelCapabilities {
            structured_output: true,
            tool_use: true,
            vision: true,
            streaming: true,
        }
    } else if model.contains("gemini-1.0") || model.contains("gemini-1.") {
        ModelCapabilities {
            structured_output: false,
            tool_use: true,
            vision: false,
            streaming: true,
        }
    } else {
        FALLBACK
    }
}

/// Azure OpenAI capabilities.
///
/// Azure hosts OpenAI models — mirror OpenAI capabilities by model name,
/// ignoring any deployment-prefix conventions.
fn azure_capabilities(model: &str) -> ModelCapabilities {
    openai_capabilities(model)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns the full capability set for the given provider/model pair.
pub fn model_capabilities(provider: &str, model: &str) -> ModelCapabilities {
    match provider {
        providers::PROVIDER_ID_OPENAI
        | providers::PROVIDER_ID_CHATGPT_SUBSCRIPTION
        | providers::PROVIDER_ID_GITHUB_COPILOT => openai_capabilities(model),

        providers::PROVIDER_ID_ANTHROPIC => anthropic_capabilities(model),

        providers::PROVIDER_ID_AZURE_OPENAI => azure_capabilities(model),

        // Google (Gemini) — no dedicated provider constant in amux-shared today;
        // match by string until one is added.
        "google" | "gemini" => google_capabilities(model),

        _ => FALLBACK,
    }
}

/// Returns `true` when the provider/model combination supports native
/// JSON-schema structured output (strict mode).
pub fn supports_structured_output(provider: &str, model: &str) -> bool {
    model_capabilities(provider, model).structured_output
}

/// Returns `true` when the provider/model combination supports tool-calling.
pub fn supports_tool_use(provider: &str, model: &str) -> bool {
    model_capabilities(provider, model).tool_use
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use amux_shared::providers;

    // -- OpenAI ---------------------------------------------------------------

    #[test]
    fn openai_gpt4o_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_OPENAI,
            "gpt-4o"
        ));
    }

    #[test]
    fn openai_gpt4o_mini_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_OPENAI,
            "gpt-4o-mini"
        ));
    }

    #[test]
    fn openai_gpt41_nano_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_OPENAI,
            "gpt-4.1-nano"
        ));
    }

    #[test]
    fn openai_gpt5_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_OPENAI,
            "gpt-5"
        ));
    }

    #[test]
    fn openai_o1_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_OPENAI,
            "o1"
        ));
    }

    #[test]
    fn openai_o3_mini_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_OPENAI,
            "o3-mini"
        ));
    }

    #[test]
    fn openai_gpt35_no_structured_output() {
        assert!(!supports_structured_output(
            providers::PROVIDER_ID_OPENAI,
            "gpt-3.5-turbo"
        ));
    }

    #[test]
    fn openai_gpt35_tool_use() {
        assert!(supports_tool_use(
            providers::PROVIDER_ID_OPENAI,
            "gpt-3.5-turbo"
        ));
    }

    #[test]
    fn openai_text_model_no_capabilities() {
        let caps = model_capabilities(providers::PROVIDER_ID_OPENAI, "text-davinci-003");
        assert!(!caps.structured_output);
        assert!(!caps.tool_use);
        assert!(!caps.vision);
        assert!(!caps.streaming);
    }

    // -- Anthropic ------------------------------------------------------------

    #[test]
    fn anthropic_claude_sonnet4_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_ANTHROPIC,
            "claude-sonnet-4-6"
        ));
    }

    #[test]
    fn anthropic_claude3_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_ANTHROPIC,
            "claude-3-opus-20240229"
        ));
    }

    #[test]
    fn anthropic_claude2_no_structured_output() {
        assert!(!supports_structured_output(
            providers::PROVIDER_ID_ANTHROPIC,
            "claude-2.1"
        ));
    }

    #[test]
    fn anthropic_claude_instant_no_structured_output() {
        assert!(!supports_structured_output(
            providers::PROVIDER_ID_ANTHROPIC,
            "claude-instant-1.2"
        ));
    }

    // -- Azure ----------------------------------------------------------------

    #[test]
    fn azure_gpt4o_structured_output() {
        assert!(supports_structured_output(
            providers::PROVIDER_ID_AZURE_OPENAI,
            "gpt-4o"
        ));
    }

    #[test]
    fn azure_gpt35_no_structured_output() {
        assert!(!supports_structured_output(
            providers::PROVIDER_ID_AZURE_OPENAI,
            "gpt-3.5-turbo"
        ));
    }

    // -- Google ---------------------------------------------------------------

    #[test]
    fn google_gemini15_structured_output() {
        let caps = model_capabilities("google", "gemini-1.5-pro");
        assert!(caps.structured_output);
        assert!(caps.tool_use);
        assert!(caps.vision);
    }

    #[test]
    fn google_gemini2_structured_output() {
        assert!(supports_structured_output("google", "gemini-2.0-flash"));
    }

    #[test]
    fn google_gemini10_no_structured_output() {
        let caps = model_capabilities("google", "gemini-1.0-pro");
        assert!(!caps.structured_output);
        assert!(caps.tool_use);
        assert!(!caps.vision);
    }

    // -- Unknown provider / model ---------------------------------------------

    #[test]
    fn unknown_provider_fallback() {
        let caps = model_capabilities("some-unknown-provider", "any-model");
        assert!(!caps.structured_output);
        assert!(caps.tool_use);
        assert!(!caps.vision);
        assert!(caps.streaming);
    }

    #[test]
    fn unknown_model_on_known_provider() {
        // An unrecognised model on OpenAI falls back conservatively.
        let caps = model_capabilities(providers::PROVIDER_ID_OPENAI, "some-future-model-xyz");
        // Does not match modern families → structured_output=false, tool_use=true
        assert!(!caps.structured_output);
        assert!(caps.tool_use);
    }
}
