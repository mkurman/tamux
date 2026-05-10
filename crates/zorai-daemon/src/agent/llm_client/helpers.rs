use super::*;
pub(crate) struct PendingToolCall {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) arguments: String,
}

pub(crate) fn synthesize_tool_call_id(seed: &str, index: usize, name: &str) -> String {
    let clean_name: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    format!("synthetic_tool_call_{seed}_{index}_{clean_name}")
}

pub(crate) fn drain_tool_calls(map: &mut HashMap<u32, PendingToolCall>) -> Vec<ToolCall> {
    let mut entries: Vec<(u32, PendingToolCall)> = map.drain().collect();
    entries.sort_by_key(|(idx, _)| *idx);
    entries
        .into_iter()
        .map(|(idx, tc)| {
            ToolCall::with_default_weles_review(
                if tc.id.trim().is_empty() {
                    synthesize_tool_call_id(
                        zorai_shared::providers::PROVIDER_ID_OPENAI,
                        idx as usize,
                        &tc.name,
                    )
                } else {
                    tc.id
                },
                ToolFunction {
                    name: tc.name,
                    arguments: tc.arguments,
                },
            )
        })
        .collect()
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FetchedModelPricing {
    pub prompt: Option<String>,
    pub completion: Option<String>,
    pub image: Option<String>,
    pub request: Option<String>,
    pub web_search: Option<String>,
    pub internal_reasoning: Option<String>,
    pub input_cache_read: Option<String>,
    pub input_cache_write: Option<String>,
    pub audio: Option<String>,
}

impl FetchedModelPricing {
    fn is_empty(&self) -> bool {
        self.prompt.is_none()
            && self.completion.is_none()
            && self.image.is_none()
            && self.request.is_none()
            && self.web_search.is_none()
            && self.internal_reasoning.is_none()
            && self.input_cache_read.is_none()
            && self.input_cache_write.is_none()
            && self.audio.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchedModel {
    pub id: String,
    pub name: Option<String>,
    pub context_window: Option<u32>,
    pub pricing: Option<FetchedModelPricing>,
    pub metadata: Option<serde_json::Value>,
}

#[allow(dead_code)]
pub fn fetched_model_feature_capabilities(
    provider_id: &str,
    model: &FetchedModel,
) -> zorai_shared::providers::ModelFeatureCapabilities {
    zorai_shared::providers::derive_model_feature_capabilities(
        provider_id,
        &model.id,
        model.metadata.as_ref(),
        model
            .pricing
            .as_ref()
            .and_then(|pricing| pricing.image.as_deref())
            .is_some(),
    )
}

fn json_string_field(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        serde_json::Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn parse_model_pricing(model: &serde_json::Value) -> Option<FetchedModelPricing> {
    let pricing = model.get("pricing")?.as_object()?;
    let pricing = FetchedModelPricing {
        prompt: json_string_field(pricing.get("prompt")),
        completion: json_string_field(pricing.get("completion")),
        image: json_string_field(pricing.get("image")),
        request: json_string_field(pricing.get("request")),
        web_search: json_string_field(pricing.get("web_search")),
        internal_reasoning: json_string_field(pricing.get("internal_reasoning")),
        input_cache_read: json_string_field(pricing.get("input_cache_read")),
        input_cache_write: json_string_field(pricing.get("input_cache_write")),
        audio: json_string_field(pricing.get("audio")),
    };
    (!pricing.is_empty()).then_some(pricing)
}

pub(crate) fn parse_fetched_models_response(json: &serde_json::Value) -> Vec<FetchedModel> {
    json.get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?.to_string();
                    let name = m
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string());

                    let context_window = m
                        .get("context_length")
                        .or_else(|| m.get("context_window"))
                        .and_then(|c| match c {
                            serde_json::Value::Number(number) => number.as_u64(),
                            serde_json::Value::String(text) => text.trim().parse::<u64>().ok(),
                            _ => None,
                        })
                        .and_then(|n| u32::try_from(n).ok());

                    Some(FetchedModel {
                        id,
                        name,
                        context_window,
                        pricing: parse_model_pricing(m),
                        metadata: Some(m.clone()),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn built_in_models_for_provider(provider_id: &str) -> Result<Vec<FetchedModel>> {
    get_provider_definition(provider_id)
        .map(|definition| {
            definition
                .models
                .iter()
                .map(|model| FetchedModel {
                    id: model.id.to_string(),
                    name: Some(model.name.to_string()),
                    context_window: Some(model.context_window),
                    pricing: None,
                    metadata: None,
                })
                .collect()
        })
        .ok_or_else(|| anyhow::anyhow!("Unknown provider '{}'", provider_id))
}

pub async fn fetch_models(
    provider_id: &str,
    base_url: &str,
    api_key: &str,
    output_modalities: Option<&str>,
) -> Result<Vec<FetchedModel>> {
    let _ = crate::agent::types::reload_custom_provider_catalog_from_default_path();

    if provider_id == zorai_shared::providers::PROVIDER_ID_GITHUB_COPILOT {
        return built_in_models_for_provider(provider_id);
    }

    let def = crate::agent::types::get_provider_definition(provider_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown provider '{}'", provider_id))?;

    if !def.supports_model_fetch {
        tracing::info!(
            provider_id,
            "provider does not support remote model fetching; returning built-in catalog"
        );
        return built_in_models_for_provider(provider_id);
    }

    let client = reqwest::Client::new();
    let mut url = format!("{}/models", base_url.trim_end_matches('/'));
    if provider_id == zorai_shared::providers::PROVIDER_ID_OPENROUTER {
        if let Some(output_modalities) = output_modalities
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let normalized = output_modalities.to_ascii_lowercase();
            if normalized == "embedding" || normalized == "embeddings" {
                url = format!("{}/embeddings/models", base_url.trim_end_matches('/'));
            } else {
                url.push_str("?output_modalities=");
                url.push_str(output_modalities);
            }
        }
    }
    let send_request = |include_auth: bool| {
        let mut req = client.get(&url).header("Content-Type", "application/json");
        if include_auth && !api_key.is_empty() {
            req = def.auth_method.apply(req, api_key);
        }
        apply_openrouter_attribution_headers(req, provider_id)
    };

    let mut response = send_request(true).send().await?;

    if provider_id == zorai_shared::providers::PROVIDER_ID_CHUTES
        && !api_key.is_empty()
        && matches!(
            response.status(),
            reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN
        )
    {
        tracing::info!(
            provider_id,
            status = %response.status(),
            "retrying model catalog fetch without auth after auth failure"
        );
        response = send_request(false).send().await?;
    }

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Failed to fetch models: {} - {}",
            status,
            text
        ));
    }

    let json: serde_json::Value = response.json().await?;
    Ok(parse_fetched_models_response(&json))
}

pub async fn validate_provider_connection(
    provider_id: &str,
    base_url: &str,
    api_key: &str,
    auth_source: AuthSource,
) -> Result<Option<Vec<FetchedModel>>> {
    let _ = reload_custom_provider_catalog_from_default_path();

    let def = get_provider_definition(provider_id)
        .with_context(|| format!("Unknown provider '{}'", provider_id))?;
    let resolved_base_url = if base_url.trim().is_empty() {
        def.default_base_url.to_string()
    } else {
        base_url.trim().to_string()
    };

    if provider_id == zorai_shared::providers::PROVIDER_ID_AZURE_OPENAI {
        let models = fetch_models(provider_id, &resolved_base_url, api_key, None).await?;
        return Ok(Some(models));
    }

    if provider_id == zorai_shared::providers::PROVIDER_ID_GITHUB_COPILOT {
        let models = super::super::copilot_auth::list_github_copilot_models(api_key, auth_source)?
            .into_iter()
            .map(|model| FetchedModel {
                id: model.id,
                name: model.name,
                context_window: model.context_window,
                pricing: None,
                metadata: None,
            })
            .collect::<Vec<_>>();
        if models.is_empty() {
            anyhow::bail!("GitHub Copilot auth is valid but no models are available");
        }
        return Ok(Some(models));
    }

    if provider_id == zorai_shared::providers::PROVIDER_ID_OPENAI
        && auth_source == AuthSource::ChatgptSubscription
    {
        let client = reqwest::Client::new();
        let config = ProviderConfig {
            base_url: resolved_base_url,
            model: def.default_model.to_string(),
            api_key: String::new(),
            assistant_id: String::new(),
            auth_source,
            api_transport: def.default_transport,
            reasoning_effort: "off".to_string(),
            context_window_tokens: def
                .models
                .iter()
                .find(|model| model.id == def.default_model)
                .map(|model| model.context_window)
                .unwrap_or(128_000),
            response_schema: None,
            stop_sequences: None,
            temperature: None,
            top_p: None,
            top_k: None,
            metadata: None,
            service_tier: None,
            container: None,
            inference_geo: None,
            cache_control: None,
            max_tokens: None,
            anthropic_tool_choice: None,
            output_effort: None,
            openrouter_provider_order: Vec::new(),
            openrouter_provider_ignore: Vec::new(),
            openrouter_allow_fallbacks: None,
            openrouter_response_cache_enabled: false,
        };
        let _ = resolve_openai_codex_request_auth(&client, provider_id, &config).await?;
        return Ok(None);
    }

    let client = reqwest::Client::new();
    let api_type = get_provider_api_type(provider_id, def.default_model, &resolved_base_url);
    let request = match api_type {
        ApiType::OpenAI => {
            let url = build_chat_completion_url(&resolved_base_url);
            let body = serde_json::json!({
                "model": def.default_model,
                "messages": [{ "role": "user", "content": "ok" }],
                "max_tokens": 1,
                "stream": false,
            });
            let req = client.post(url).header("Content-Type", "application/json");
            let req = if !api_key.is_empty() {
                def.auth_method.apply(req, api_key)
            } else {
                req
            };
            apply_dashscope_coding_plan_sdk_headers(req, provider_id, &resolved_base_url, api_type)
                .json(&body)
        }
        ApiType::Anthropic => {
            let base = resolved_base_url.trim_end_matches('/');
            let url = if base.ends_with("/v1") {
                format!("{}/messages", base)
            } else {
                format!("{}/v1/messages", base)
            };
            let body = serde_json::json!({
                "model": def.default_model,
                "max_tokens": 1,
                "messages": [{ "role": "user", "content": "ok" }],
            });
            let req = client.post(url).header("Content-Type", "application/json");
            let req = if !is_dashscope_coding_plan_anthropic_base_url(&resolved_base_url) {
                req.header("anthropic-version", "2023-06-01")
            } else {
                req
            };
            let req = if !api_key.is_empty() {
                def.auth_method.apply(req, api_key)
            } else {
                req
            };
            apply_dashscope_coding_plan_sdk_headers(req, provider_id, &resolved_base_url, api_type)
                .json(&body)
        }
    };

    let response = request.send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("Validation failed: {} - {}", status, text);
    }

    Ok(None)
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn parse_fetched_models_response_preserves_xai_metadata_and_pricing() {
        let json = serde_json::json!({
            "data": [{
                "id": "grok-4",
                "name": "Grok 4",
                "context_length": "262144",
                "pricing": {
                    "prompt": "3.00",
                    "completion": "15.00",
                    "input_cache_read": "0.75"
                },
                "modalities": ["text", "image"],
                "owned_by": "xai",
                "supports_reasoning": true
            }, {
                "id": "grok-code-fast-1",
                "context_window": 131072,
                "pricing": {
                    "request": 42
                },
                "mode": "code"
            }, {
                "id": "grok-overflow",
                "context_length": "4294967296"
            }]
        });

        let models = parse_fetched_models_response(&json);

        assert_eq!(models.len(), 3);
        assert_eq!(models[0].id, "grok-4");
        assert_eq!(models[0].name.as_deref(), Some("Grok 4"));
        assert_eq!(models[0].context_window, Some(262_144));
        assert_eq!(
            models[0].pricing.as_ref().and_then(|p| p.prompt.as_deref()),
            Some("3.00")
        );
        assert_eq!(
            models[0]
                .pricing
                .as_ref()
                .and_then(|p| p.completion.as_deref()),
            Some("15.00")
        );
        assert_eq!(
            models[0]
                .pricing
                .as_ref()
                .and_then(|p| p.input_cache_read.as_deref()),
            Some("0.75")
        );
        assert_eq!(
            models[0]
                .metadata
                .as_ref()
                .and_then(|m| m.get("owned_by"))
                .and_then(|v| v.as_str()),
            Some("xai")
        );
        assert_eq!(
            models[0]
                .metadata
                .as_ref()
                .and_then(|m| m.get("modalities"))
                .and_then(|v| v.as_array())
                .map(|items| items.len()),
            Some(2)
        );

        assert_eq!(models[1].id, "grok-code-fast-1");
        assert_eq!(models[1].context_window, Some(131_072));
        assert_eq!(
            models[1]
                .pricing
                .as_ref()
                .and_then(|p| p.request.as_deref()),
            Some("42")
        );
        assert_eq!(
            models[1]
                .metadata
                .as_ref()
                .and_then(|m| m.get("mode"))
                .and_then(|v| v.as_str()),
            Some("code")
        );

        assert_eq!(models[2].id, "grok-overflow");
        assert_eq!(models[2].context_window, None);
    }
}
