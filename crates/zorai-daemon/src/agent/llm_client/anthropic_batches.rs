#![allow(dead_code)]

use super::*;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageBatchRequestCounts {
    pub canceled: u64,
    pub errored: u64,
    pub expired: u64,
    pub processing: u64,
    pub succeeded: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageBatch {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel_initiated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub processing_status: String,
    pub request_counts: AnthropicMessageBatchRequestCounts,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results_url: Option<String>,
    #[serde(rename = "type")]
    pub batch_type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageBatchList {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub data: Vec<AnthropicMessageBatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    pub has_more: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicDeletedMessageBatch {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(rename = "type")]
    pub batch_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<AnthropicMessageContainer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<AnthropicContentBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub message_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<AnthropicMessageUsage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicMessageContainer {
    pub id: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation: Option<AnthropicCacheCreationUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<AnthropicServerToolUsage>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicCacheCreationUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral_1h_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ephemeral_5m_input_tokens: Option<u64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicServerToolUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_fetch_requests: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_search_requests: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicContentSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl PartialEq<serde_json::Value> for AnthropicContentSource {
    fn eq(&self, other: &serde_json::Value) -> bool {
        serde_json::to_value(self).ok().as_ref() == Some(other)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicCitation {
    #[serde(rename = "type")]
    pub citation_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_char_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_char_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_page_number: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_page_number: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_block_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_block_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cited_text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_title: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnthropicCitationList {
    pub items: Vec<AnthropicCitation>,
}

impl PartialEq<serde_json::Value> for AnthropicCitationList {
    fn eq(&self, other: &serde_json::Value) -> bool {
        serde_json::to_value(self).ok().as_ref() == Some(other)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicToolReference {
    #[serde(rename = "type")]
    pub reference_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnthropicToolReferenceList {
    pub items: Vec<AnthropicToolReference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnthropicBlockContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
    Json(serde_json::Value),
}

impl AnthropicBlockContent {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn as_blocks(&self) -> Option<&[AnthropicContentBlock]> {
        match self {
            Self::Blocks(blocks) => Some(blocks.as_slice()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<AnthropicBlockContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<AnthropicContentSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub citations: Option<AnthropicCitationList>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_age: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_references: Option<AnthropicToolReferenceList>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_code: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_file_update: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caller: Option<AnthropicContentBlockCaller>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicContentBlockCaller {
    #[serde(rename = "type")]
    pub caller_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl AnthropicContentBlock {
    pub fn block_type(&self) -> &str {
        &self.block_type
    }

    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicBatchCreateRequest {
    pub custom_id: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct AnthropicMessageBatchListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicMessageBatchResult {
    Succeeded { message: AnthropicMessage },
    Errored { error: AnthropicErrorDetail },
    Canceled {},
    Expired {},
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessageBatchIndividualResponse {
    pub custom_id: String,
    pub result: AnthropicMessageBatchResult,
}

fn anthropic_batches_url(base_url: &str) -> String {
    format!("{}/batches", anthropic_messages_url(base_url))
}

fn anthropic_batch_url(base_url: &str, batch_id: &str) -> String {
    format!("{}/{}", anthropic_batches_url(base_url), batch_id)
}

fn anthropic_batch_results_url(base_url: &str, batch_id: &str) -> String {
    format!("{}/{}/results", anthropic_batches_url(base_url), batch_id)
}

fn build_anthropic_api_request(
    client: &reqwest::Client,
    provider: &str,
    config: &ProviderConfig,
    method: reqwest::Method,
    url: &str,
    body: Option<serde_json::Value>,
) -> Result<reqwest::Request> {
    let auth_method = get_provider_definition(provider)
        .map(|d| d.auth_method)
        .unwrap_or(AuthMethod::XApiKey);
    let mut request = auth_method.apply(client.request(method, url), &config.api_key);
    if body.is_some() {
        request = request.header("Content-Type", "application/json");
    }
    if !is_dashscope_coding_plan_anthropic_base_url(&config.base_url) {
        request = request.header("anthropic-version", "2023-06-01");
    }
    if needs_coding_plan_sdk_headers(provider) {
        request = request.header(
            "anthropic-beta",
            "fine-grained-tool-streaming-2025-05-14,interleaved-thinking-2025-05-14",
        );
    }
    request = apply_dashscope_coding_plan_sdk_headers(
        request,
        provider,
        &config.base_url,
        ApiType::Anthropic,
    );
    if let Some(body) = body {
        request = request.body(body.to_string());
    }
    request.build().map_err(Into::into)
}

async fn parse_anthropic_json_response<T: DeserializeOwned>(
    response: reqwest::Response,
    label: &str,
) -> Result<T> {
    if !response.status().is_success() {
        let status = response.status();
        let retry_after_ms = extract_retry_after_ms(Some(response.headers()), "");
        let text = response.text().await.unwrap_or_default();
        return Err(classify_http_failure_with_retry_after(
            status,
            "Anthropic",
            &text,
            retry_after_ms.or_else(|| extract_retry_after_ms(None, &text)),
        ));
    }

    let request_id = response
        .headers()
        .get("request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let mut value: serde_json::Value = response
        .json()
        .await
        .with_context(|| format!("parse Anthropic {label} response"))?;
    if let (Some(request_id), Some(object)) = (request_id, value.as_object_mut()) {
        object.insert("request_id".to_string(), serde_json::json!(request_id));
    }
    serde_json::from_value(value).with_context(|| format!("decode Anthropic {label} response"))
}

fn ensure_anthropic_batch_provider(provider: &str, config: &ProviderConfig) -> Result<()> {
    if get_provider_api_type(provider, &config.model, &config.base_url) == ApiType::Anthropic {
        Ok(())
    } else {
        Err(transport_incompatibility_error(
            provider,
            "message batches are only implemented for Anthropic Messages providers",
        ))
    }
}

pub fn build_anthropic_message_batch_params(
    provider: &str,
    config: &ProviderConfig,
    system_prompt: &str,
    messages: &[ApiMessage],
    tools: &[ToolDefinition],
    max_tokens: u32,
) -> serde_json::Value {
    let mut body = build_anthropic_base_body(provider, config, system_prompt, messages, tools);
    body["max_tokens"] = serde_json::json!(max_tokens);
    body
}

pub async fn create_message_batch(
    client: &reqwest::Client,
    provider: &str,
    config: &ProviderConfig,
    requests: &[AnthropicBatchCreateRequest],
) -> Result<AnthropicMessageBatch> {
    ensure_anthropic_batch_provider(provider, config)?;
    let request = build_anthropic_api_request(
        client,
        provider,
        config,
        reqwest::Method::POST,
        &anthropic_batches_url(&config.base_url),
        Some(serde_json::json!({ "requests": requests })),
    )?;
    let response = client.execute(request).await?;
    parse_anthropic_json_response(response, "create batch").await
}

pub async fn retrieve_message_batch(
    client: &reqwest::Client,
    provider: &str,
    config: &ProviderConfig,
    batch_id: &str,
) -> Result<AnthropicMessageBatch> {
    ensure_anthropic_batch_provider(provider, config)?;
    let request = build_anthropic_api_request(
        client,
        provider,
        config,
        reqwest::Method::GET,
        &anthropic_batch_url(&config.base_url, batch_id),
        None,
    )?;
    let response = client.execute(request).await?;
    parse_anthropic_json_response(response, "retrieve batch").await
}

pub async fn list_message_batches(
    client: &reqwest::Client,
    provider: &str,
    config: &ProviderConfig,
    params: &AnthropicMessageBatchListParams,
) -> Result<AnthropicMessageBatchList> {
    ensure_anthropic_batch_provider(provider, config)?;
    let mut request = build_anthropic_api_request(
        client,
        provider,
        config,
        reqwest::Method::GET,
        &anthropic_batches_url(&config.base_url),
        None,
    )?;
    if params != &AnthropicMessageBatchListParams::default() {
        *request.url_mut() = request
            .url()
            .clone()
            .join("")
            .expect("url join should succeed");
    }
    let response = client
        .execute(if params == &AnthropicMessageBatchListParams::default() {
            request
        } else {
            let mut builder = client.get(request.url().clone());
            for (name, value) in request.headers().iter() {
                builder = builder.header(name, value.clone());
            }
            builder.query(params).build().map_err(anyhow::Error::from)?
        })
        .await?;
    parse_anthropic_json_response(response, "list batches").await
}

pub async fn cancel_message_batch(
    client: &reqwest::Client,
    provider: &str,
    config: &ProviderConfig,
    batch_id: &str,
) -> Result<AnthropicMessageBatch> {
    ensure_anthropic_batch_provider(provider, config)?;
    let request = build_anthropic_api_request(
        client,
        provider,
        config,
        reqwest::Method::POST,
        &format!("{}/cancel", anthropic_batch_url(&config.base_url, batch_id)),
        None,
    )?;
    let response = client.execute(request).await?;
    parse_anthropic_json_response(response, "cancel batch").await
}

pub async fn delete_message_batch(
    client: &reqwest::Client,
    provider: &str,
    config: &ProviderConfig,
    batch_id: &str,
) -> Result<AnthropicDeletedMessageBatch> {
    ensure_anthropic_batch_provider(provider, config)?;
    let request = build_anthropic_api_request(
        client,
        provider,
        config,
        reqwest::Method::DELETE,
        &anthropic_batch_url(&config.base_url, batch_id),
        None,
    )?;
    let response = client.execute(request).await?;
    parse_anthropic_json_response(response, "delete batch").await
}

pub async fn retrieve_message_batch_results(
    client: &reqwest::Client,
    provider: &str,
    config: &ProviderConfig,
    batch_id: &str,
) -> Result<Vec<AnthropicMessageBatchIndividualResponse>> {
    ensure_anthropic_batch_provider(provider, config)?;
    let request = build_anthropic_api_request(
        client,
        provider,
        config,
        reqwest::Method::GET,
        &anthropic_batch_results_url(&config.base_url, batch_id),
        None,
    )?;
    let response = client.execute(request).await?;

    if !response.status().is_success() {
        let status = response.status();
        let retry_after_ms = extract_retry_after_ms(Some(response.headers()), "");
        let text = response.text().await.unwrap_or_default();
        return Err(classify_http_failure_with_retry_after(
            status,
            "Anthropic",
            &text,
            retry_after_ms.or_else(|| extract_retry_after_ms(None, &text)),
        ));
    }

    let text = response
        .text()
        .await
        .context("read Anthropic batch results body")?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str::<AnthropicMessageBatchIndividualResponse>(line)
                .with_context(|| "parse Anthropic batch result line")
        })
        .collect()
}
