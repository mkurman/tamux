use futures::StreamExt;

use super::*;

const DEFAULT_IMAGE_ANALYSIS_PROMPT: &str = "Analyze this image in detail.";
const DEFAULT_IMAGE_GENERATION_MODEL: &str = "gpt-image-1";
const DEFAULT_STT_MODEL: &str = "whisper-1";
const DEFAULT_TTS_MODEL: &str = "gpt-4o-mini-tts";
const DEFAULT_TTS_VOICE: &str = "alloy";
const DEFAULT_XAI_TTS_VOICE: &str = "eve";
const DEFAULT_XAI_TTS_LANGUAGE: &str = "auto";
const DEFAULT_IMAGE_OUTPUT_FORMAT: &str = "png";
const DEFAULT_TTS_OUTPUT_FORMAT: &str = "mp3";

const OPENROUTER_ATTRIBUTION_URL: &str = "https://tamux.app";
const OPENROUTER_ATTRIBUTION_TITLE: &str = "tamux";
const OPENROUTER_ATTRIBUTION_CATEGORIES: &str = "cli-agent";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaKind {
    Image,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AudioToolRoute {
    OpenAiCompatibleDirect,
    ProviderMultimodalCompletion,
    OpenRouterTts,
    XaiStt,
    XaiTts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OrderedMultipartField {
    Text { name: String, value: String },
    File {
        name: String,
        filename: String,
        mime_type: String,
    },
}

#[derive(Debug, Clone)]
enum MediaInputSource {
    RemoteUrl {
        url: String,
    },
    Inline {
        mime_type: String,
        base64_data: String,
        data_url: String,
    },
}

fn media_input_source_arg_count(args: &serde_json::Value) -> usize {
    ["path", "url", "base64", "data_url"]
        .iter()
        .filter(|key| {
            args.get(**key)
                .and_then(|value| value.as_str())
                .map(str::trim)
                .is_some_and(|value| !value.is_empty())
        })
        .count()
}

fn openai_like_endpoint(base_url: &str, endpoint: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let endpoint = endpoint.trim_start_matches('/');
    if base.ends_with("/v1") {
        format!("{base}/{endpoint}")
    } else {
        format!("{base}/v1/{endpoint}")
    }
}

fn infer_media_mime(path: &Path, kind: MediaKind) -> &'static str {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();

    match (kind, extension.as_str()) {
        (MediaKind::Image, "png") => "image/png",
        (MediaKind::Image, "jpg" | "jpeg") => "image/jpeg",
        (MediaKind::Image, "webp") => "image/webp",
        (MediaKind::Image, "gif") => "image/gif",
        (MediaKind::Image, "bmp") => "image/bmp",
        (MediaKind::Image, "svg") => "image/svg+xml",
        (MediaKind::Image, "tif" | "tiff") => "image/tiff",
        (MediaKind::Image, _) => "application/octet-stream",
        (MediaKind::Audio, "mp3") => "audio/mpeg",
        (MediaKind::Audio, "wav") => "audio/wav",
        (MediaKind::Audio, "ogg") => "audio/ogg",
        (MediaKind::Audio, "m4a") => "audio/mp4",
        (MediaKind::Audio, "mp4") => "audio/mp4",
        (MediaKind::Audio, "flac") => "audio/flac",
        (MediaKind::Audio, "webm") => "audio/webm",
        (MediaKind::Audio, _) => "application/octet-stream",
    }
}

fn file_extension_for_generated_mime(mime_type: &str, fallback: &str) -> String {
    match mime_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "audio/mpeg" => "mp3",
        "audio/L16" => "pcm",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/flac" => "flac",
        "audio/mp4" => "m4a",
        "audio/webm" => "webm",
        _ => fallback,
    }
    .to_string()
}

fn data_url_from_base64(mime_type: &str, base64_data: &str) -> String {
    format!("data:{mime_type};base64,{base64_data}")
}

fn audio_format_from_mime_type(mime_type: &str) -> Option<&'static str> {
    match mime_type.trim().to_ascii_lowercase().as_str() {
        "audio/wav" | "audio/x-wav" => Some("wav"),
        "audio/mpeg" | "audio/mp3" => Some("mp3"),
        "audio/ogg" => Some("ogg"),
        "audio/flac" => Some("flac"),
        "audio/mp4" | "audio/m4a" => Some("mp4"),
        "audio/webm" => Some("webm"),
        _ => None,
    }
}

fn parse_data_url(raw: &str) -> Result<(String, String)> {
    let trimmed = raw.trim();
    let header_end = trimmed
        .find(",")
        .ok_or_else(|| anyhow::anyhow!("invalid data URL: missing comma separator"))?;
    let (header, data) = trimmed.split_at(header_end);
    let data = data.trim_start_matches(',');
    if !header.starts_with("data:") {
        anyhow::bail!("invalid data URL: missing data: prefix");
    }
    let mime_type = header
        .trim_start_matches("data:")
        .split(';')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("application/octet-stream")
        .to_string();
    if !header.contains(";base64") {
        anyhow::bail!("invalid data URL: only base64 payloads are supported");
    }
    if data.trim().is_empty() {
        anyhow::bail!("invalid data URL: empty payload");
    }
    Ok((mime_type, data.trim().to_string()))
}

async fn resolve_media_input_source(
    args: &serde_json::Value,
    kind: MediaKind,
) -> Result<MediaInputSource> {
    let provided = media_input_source_arg_count(args);
    if provided == 0 {
        anyhow::bail!(
            "provide exactly one of 'path', 'url', 'base64', or 'data_url'"
        );
    }
    if provided > 1 {
        anyhow::bail!(
            "arguments 'path', 'url', 'base64', and 'data_url' are mutually exclusive"
        );
    }

    if let Some(url) = args
        .get("url")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let parsed = url::Url::parse(url)
            .map_err(|error| anyhow::anyhow!("invalid 'url' argument: {error}"))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            anyhow::bail!("'url' must use http or https");
        }
        return Ok(MediaInputSource::RemoteUrl {
            url: url.to_string(),
        });
    }

    if let Some(data_url) = args
        .get("data_url")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let (mime_type, base64_data) = parse_data_url(data_url)?;
        return Ok(MediaInputSource::Inline {
            mime_type,
            base64_data,
            data_url: data_url.to_string(),
        });
    }

    if let Some(base64_data) = args
        .get("base64")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let mime_type = args
            .get("mime_type")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("'mime_type' is required when using 'base64'"))?
            .to_string();
        return Ok(MediaInputSource::Inline {
            data_url: data_url_from_base64(&mime_type, base64_data),
            mime_type,
            base64_data: base64_data.to_string(),
        });
    }

    let path = args
        .get("path")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing media input path"))?;
    validate_read_path(path)?;
    let resolved = resolve_tool_path(path, None);
    let bytes = tokio::fs::read(&resolved).await?;
    let mime_type = args
        .get("mime_type")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| infer_media_mime(&resolved, kind).to_string());
    let base64_data = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(MediaInputSource::Inline {
        data_url: data_url_from_base64(&mime_type, &base64_data),
        mime_type,
        base64_data,
    })
}

fn select_media_provider_config(
    config: &AgentConfig,
    provider_id: &str,
    mut provider_config: crate::agent::types::ProviderConfig,
    explicit_model: Option<&str>,
    default_model: Option<&str>,
) -> (String, crate::agent::types::ProviderConfig) {
    let chosen_model = explicit_model
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| default_model.map(ToOwned::to_owned))
        .unwrap_or_else(|| provider_config.model.clone());
    provider_config.model = chosen_model;
    let provider_id = if provider_id.trim().is_empty() {
        config.provider.clone()
    } else {
        provider_id.to_string()
    };
    (provider_id, provider_config)
}

fn media_setting_string(
    config: &AgentConfig,
    nested_path: &[&str],
    legacy_flat_key: &str,
) -> Option<String> {
    config
        .extra
        .get("audio")
        .and_then(|value| {
            nested_path
                .iter()
                .try_fold(value, |acc, key| acc.get(*key))
                .and_then(|value| value.as_str())
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            config
                .extra
                .get(legacy_flat_key)
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn media_persisted_provider_model(
    config: &AgentConfig,
    endpoint: &str,
) -> (Option<String>, Option<String>) {
    let provider = media_setting_string(config, &[endpoint, "provider"], &format!("audio_{endpoint}_provider"));
    let model = media_setting_string(config, &[endpoint, "model"], &format!("audio_{endpoint}_model"));
    (provider, model)
}

async fn resolve_media_provider_config(
    agent: &AgentEngine,
    args: &serde_json::Value,
    default_model: Option<&str>,
) -> Result<(String, crate::agent::types::ProviderConfig)> {
    let config = agent.get_config().await;
    let explicit_provider = args
        .get("provider")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let explicit_model = args
        .get("model")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let endpoint = match default_model {
        Some(model) if model == DEFAULT_STT_MODEL => Some("stt"),
        Some(model) if model == DEFAULT_TTS_MODEL => Some("tts"),
        _ => None,
    };
    let (persisted_provider, persisted_model) = endpoint
        .map(|endpoint| media_persisted_provider_model(&config, endpoint))
        .unwrap_or((None, None));
    let selected_provider = explicit_provider.or(persisted_provider.as_deref());
    let selected_model = explicit_model.or(persisted_model.as_deref());

    if let Some(provider_id) = selected_provider {
        let provider_config = agent.resolve_sub_agent_provider_config(&config, provider_id)?;
        return Ok(select_media_provider_config(
            &config,
            provider_id,
            provider_config,
            selected_model,
            default_model,
        ));
    }

    let provider_config = agent.resolve_provider_config(&config)?;
    Ok(select_media_provider_config(
        &config,
        &config.provider,
        provider_config,
        selected_model,
        default_model,
    ))
}

fn effective_multimodal_transport(
    provider_id: &str,
    provider_config: &crate::agent::types::ProviderConfig,
) -> ApiTransport {
    let mut transport = if crate::agent::types::provider_supports_transport(
        provider_id,
        provider_config.api_transport,
    ) {
        provider_config.api_transport
    } else {
        crate::agent::types::default_api_transport_for_provider(provider_id)
    };
    if transport == ApiTransport::NativeAssistant {
        transport = if crate::agent::types::provider_supports_transport(
            provider_id,
            ApiTransport::Responses,
        ) {
            ApiTransport::Responses
        } else {
            ApiTransport::ChatCompletions
        };
    }
    transport
}

fn build_image_analysis_blocks(
    provider_id: &str,
    provider_config: &crate::agent::types::ProviderConfig,
    prompt: &str,
    source: &MediaInputSource,
) -> Result<(ApiTransport, Vec<serde_json::Value>)> {
    let api_type = crate::agent::types::get_provider_api_type(
        provider_id,
        &provider_config.model,
        &provider_config.base_url,
    );
    let transport = effective_multimodal_transport(provider_id, provider_config);

    if transport == ApiTransport::AnthropicMessages || api_type == ApiType::Anthropic {
        let MediaInputSource::Inline {
            mime_type,
            base64_data,
            ..
        } = source
        else {
            anyhow::bail!(
                "remote image URLs are not supported for Anthropic-compatible image analysis; use 'path', 'base64', or 'data_url'"
            );
        };
        return Ok((
            transport,
            vec![
                serde_json::json!({
                    "type": "text",
                    "text": prompt,
                }),
                serde_json::json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": mime_type,
                        "data": base64_data,
                    }
                }),
            ],
        ));
    }

    match transport {
        ApiTransport::ChatCompletions => {
            let image_payload = match source {
                MediaInputSource::RemoteUrl { url } => serde_json::json!({ "url": url }),
                MediaInputSource::Inline { data_url, .. } => serde_json::json!({ "url": data_url }),
            };
            Ok((
                transport,
                vec![
                    serde_json::json!({
                        "type": "text",
                        "text": prompt,
                    }),
                    serde_json::json!({
                        "type": "image_url",
                        "image_url": image_payload,
                    }),
                ],
            ))
        }
        _ => {
            let image_url = match source {
                MediaInputSource::RemoteUrl { url } => url.clone(),
                MediaInputSource::Inline { data_url, .. } => data_url.clone(),
            };
            Ok((
                transport,
                vec![
                    serde_json::json!({
                        "type": "input_text",
                        "text": prompt,
                    }),
                    serde_json::json!({
                        "type": "input_image",
                        "image_url": image_url,
                    }),
                ],
            ))
        }
    }
}

fn provider_auth_method(provider_id: &str) -> crate::agent::types::AuthMethod {
    crate::agent::types::get_provider_definition(provider_id)
        .map(|definition| definition.auth_method)
        .unwrap_or(crate::agent::types::AuthMethod::Bearer)
}

fn apply_media_auth_headers(
    request: reqwest::RequestBuilder,
    provider_id: &str,
    provider_config: &crate::agent::types::ProviderConfig,
) -> Result<reqwest::RequestBuilder> {
    if provider_config.auth_source != crate::agent::types::AuthSource::ApiKey {
        anyhow::bail!(
            "provider '{}' currently requires api_key authentication for direct image/audio media endpoints",
            provider_id
        );
    }
    if provider_config.api_key.trim().is_empty() {
        anyhow::bail!("provider '{}' is missing an API key", provider_id);
    }
    let request = provider_auth_method(provider_id).apply(request, &provider_config.api_key);
    if provider_id == amux_shared::providers::PROVIDER_ID_OPENROUTER {
        return Ok(request
            .header("HTTP-Referer", OPENROUTER_ATTRIBUTION_URL)
            .header("X-OpenRouter-Title", OPENROUTER_ATTRIBUTION_TITLE)
            .header(
                "X-OpenRouter-Categories",
                OPENROUTER_ATTRIBUTION_CATEGORIES,
            ));
    }
    Ok(request)
}

fn ensure_openai_like_media_endpoint(
    provider_id: &str,
    provider_config: &crate::agent::types::ProviderConfig,
) -> Result<()> {
    let api_type = crate::agent::types::get_provider_api_type(
        provider_id,
        &provider_config.model,
        &provider_config.base_url,
    );
    if api_type != ApiType::OpenAI {
        anyhow::bail!(
            "provider '{}' does not expose an OpenAI-compatible media endpoint in the current configuration",
            provider_id
        );
    }
    Ok(())
}

fn audio_tool_route(
    provider_id: &str,
    audio_tool_kind: amux_shared::providers::AudioToolKind,
) -> AudioToolRoute {
    if provider_id == amux_shared::providers::PROVIDER_ID_XAI {
        return match audio_tool_kind {
            amux_shared::providers::AudioToolKind::SpeechToText => AudioToolRoute::XaiStt,
            amux_shared::providers::AudioToolKind::TextToSpeech => AudioToolRoute::XaiTts,
        };
    }

    if provider_id == amux_shared::providers::PROVIDER_ID_OPENROUTER {
        return match audio_tool_kind {
            amux_shared::providers::AudioToolKind::SpeechToText => {
                AudioToolRoute::ProviderMultimodalCompletion
            }
            amux_shared::providers::AudioToolKind::TextToSpeech => AudioToolRoute::OpenRouterTts,
        };
    }

    AudioToolRoute::OpenAiCompatibleDirect
}

fn resolve_audio_tool_route(
    provider_id: &str,
    provider_config: &crate::agent::types::ProviderConfig,
    audio_tool_kind: amux_shared::providers::AudioToolKind,
) -> Result<AudioToolRoute> {
    if !amux_shared::providers::provider_supports_audio_tool(provider_id, audio_tool_kind) {
        anyhow::bail!(
            "provider '{}' is not supported for {}",
            provider_id,
            match audio_tool_kind {
                amux_shared::providers::AudioToolKind::SpeechToText => "speech_to_text",
                amux_shared::providers::AudioToolKind::TextToSpeech => "text_to_speech",
            }
        );
    }

    let route = audio_tool_route(provider_id, audio_tool_kind);
    if route == AudioToolRoute::OpenAiCompatibleDirect {
        ensure_openai_like_media_endpoint(provider_id, provider_config)?;
    }
    Ok(route)
}

fn stt_endpoint_for_route(base_url: &str, route: AudioToolRoute) -> String {
    match route {
        AudioToolRoute::XaiStt => openai_like_endpoint(base_url, "stt"),
        _ => openai_like_endpoint(base_url, "audio/transcriptions"),
    }
}

fn tts_endpoint_for_route(base_url: &str, route: AudioToolRoute) -> String {
    match route {
        AudioToolRoute::OpenRouterTts | AudioToolRoute::XaiTts => {
            openai_like_endpoint(base_url, "tts")
        }
        _ => openai_like_endpoint(base_url, "audio/speech"),
    }
}

fn push_nonempty_text_field(
    fields: &mut Vec<OrderedMultipartField>,
    name: &str,
    value: Option<&str>,
) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        fields.push(OrderedMultipartField::Text {
            name: name.to_string(),
            value: value.to_string(),
        });
    }
}

fn build_xai_stt_multipart_fields(
    args: &serde_json::Value,
    filename: &str,
    mime_type: &str,
) -> Vec<OrderedMultipartField> {
    let mut fields = Vec::new();
    push_nonempty_text_field(
        &mut fields,
        "language",
        args.get("language").and_then(|value| value.as_str()),
    );
    push_nonempty_text_field(
        &mut fields,
        "format",
        args.get("format").map(|value| match value {
            serde_json::Value::Bool(boolean) => boolean.to_string(),
            serde_json::Value::String(string) => string.trim().to_string(),
            _ => String::new(),
        }).as_deref(),
    );
    push_nonempty_text_field(
        &mut fields,
        "audio_format",
        args.get("audio_format").and_then(|value| value.as_str()),
    );
    push_nonempty_text_field(
        &mut fields,
        "sample_rate",
        args.get("sample_rate").map(|value| value.to_string()).as_deref(),
    );
    push_nonempty_text_field(
        &mut fields,
        "multichannel",
        args.get("multichannel").map(|value| value.to_string()).as_deref(),
    );
    push_nonempty_text_field(
        &mut fields,
        "channels",
        args.get("channels").map(|value| value.to_string()).as_deref(),
    );
    push_nonempty_text_field(
        &mut fields,
        "diarize",
        args.get("diarize").map(|value| value.to_string()).as_deref(),
    );
    fields.push(OrderedMultipartField::File {
        name: "file".to_string(),
        filename: filename.to_string(),
        mime_type: mime_type.to_string(),
    });
    fields
}

fn ordered_fields_to_multipart_form(
    fields: &[OrderedMultipartField],
    bytes: Vec<u8>,
) -> Result<reqwest::multipart::Form> {
    let mut form = reqwest::multipart::Form::new();
    let mut file_bytes = Some(bytes);
    for field in fields {
        match field {
            OrderedMultipartField::Text { name, value } => {
                form = form.text(name.clone(), value.clone());
            }
            OrderedMultipartField::File {
                name,
                filename,
                mime_type,
            } => {
                let part = reqwest::multipart::Part::bytes(
                    file_bytes
                        .take()
                        .ok_or_else(|| anyhow::anyhow!("multipart form is missing file bytes"))?,
                )
                    .file_name(filename.clone())
                    .mime_str(mime_type)?;
                form = form.part(name.clone(), part);
            }
        }
    }
    Ok(form)
}

fn xai_tts_voice(args: &serde_json::Value) -> &str {
    args.get("voice")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_XAI_TTS_VOICE)
}

fn build_xai_tts_body(
    args: &serde_json::Value,
    input: &str,
) -> Result<serde_json::Value> {
    let language = args
        .get("language")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_XAI_TTS_LANGUAGE);
    let mut body = serde_json::json!({
        "text": input,
        "voice_id": xai_tts_voice(args),
        "language": language,
    });

    let mut output_format = serde_json::Map::new();
    let codec = args
        .get("response_format")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_TTS_OUTPUT_FORMAT);
    output_format.insert(
        "codec".to_string(),
        serde_json::Value::String(codec.to_string()),
    );
    if let Some(sample_rate) = args.get("sample_rate") {
        output_format.insert("sample_rate".to_string(), sample_rate.clone());
    }
    if let Some(bit_rate) = args.get("bit_rate") {
        output_format.insert("bit_rate".to_string(), bit_rate.clone());
    }
    if !output_format.is_empty() {
        body["output_format"] = serde_json::Value::Object(output_format);
    }
    Ok(body)
}

fn format_speech_to_text_response(
    route: AudioToolRoute,
    response_format: &str,
    parsed: &serde_json::Value,
) -> Result<String> {
    if response_format == "text" {
        let transcript = parsed
            .get("text")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("speech_to_text returned no transcription text"))?;
        return Ok(transcript.to_string());
    }

    match route {
        AudioToolRoute::XaiStt => Ok(serde_json::to_string_pretty(parsed)?),
        _ => {
            if let Some(text) = parsed.get("text").and_then(|value| value.as_str()) {
                Ok(text.to_string())
            } else {
                Ok(serde_json::to_string_pretty(parsed)?)
            }
        }
    }
}

fn output_mime_type_for_codec(codec: &str) -> &'static str {
    match codec {
        "pcm" => "audio/L16",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "flac" => "audio/flac",
        "aac" | "m4a" => "audio/mp4",
        "mulaw" => "audio/basic",
        "alaw" => "audio/alaw",
        _ => "audio/mpeg",
    }
}

fn build_audio_transcription_prompt(language: Option<&str>, prompt: Option<&str>) -> String {
    let mut instruction =
        "Transcribe the provided audio. Return only the transcription text.".to_string();
    if let Some(language) = language.map(str::trim).filter(|value| !value.is_empty()) {
        instruction.push_str(" The spoken language is ");
        instruction.push_str(language);
        instruction.push('.');
    }
    if let Some(prompt) = prompt.map(str::trim).filter(|value| !value.is_empty()) {
        instruction.push_str(" Prefer these terms or context hints: ");
        instruction.push_str(prompt);
        instruction.push('.');
    }
    instruction
}

fn build_audio_transcription_blocks(
    transport: ApiTransport,
    prompt: &str,
    base64_data: &str,
    format: &str,
) -> Vec<serde_json::Value> {
    let text_type = if transport == ApiTransport::ChatCompletions {
        "text"
    } else {
        "input_text"
    };

    vec![
        serde_json::json!({
            "type": text_type,
            "text": prompt,
        }),
        serde_json::json!({
            "type": "input_audio",
            "input_audio": {
                "data": base64_data,
                "format": format,
            }
        }),
    ]
}

async fn execute_multimodal_speech_to_text(
    args: &serde_json::Value,
    http_client: &reqwest::Client,
    provider_id: &str,
    provider_config: &crate::agent::types::ProviderConfig,
    mime_type: &str,
    bytes: &[u8],
) -> Result<String> {
    let transport = effective_multimodal_transport(provider_id, provider_config);
    let audio_format = audio_format_from_mime_type(mime_type).unwrap_or("wav");
    let prompt = build_audio_transcription_prompt(
        args.get("language").and_then(|value| value.as_str()),
        args.get("prompt").and_then(|value| value.as_str()),
    );
    let base64_data = base64::engine::general_purpose::STANDARD.encode(bytes);
    let blocks = build_audio_transcription_blocks(transport, &prompt, &base64_data, audio_format);
    let messages = vec![ApiMessage {
        role: "user".to_string(),
        content: ApiContent::Blocks(blocks),
        tool_call_id: None,
        name: None,
        tool_calls: None,
    }];

    let stream = crate::agent::llm_client::send_completion_request(
        http_client,
        provider_id,
        provider_config,
        "",
        &messages,
        &[],
        transport,
        None,
        None,
        RetryStrategy::Bounded {
            max_retries: 1,
            retry_delay_ms: 1_000,
        },
    );
    let (content, _, _) = collect_completion_output(stream).await?;
    let transcript = content.trim();
    if transcript.is_empty() {
        anyhow::bail!("speech_to_text returned no transcription text");
    }

    let response_format = args
        .get("response_format")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("json");
    if response_format == "text" {
        Ok(transcript.to_string())
    } else {
        Ok(serde_json::to_string_pretty(&serde_json::json!({
            "text": transcript,
        }))?)
    }
}

async fn collect_completion_output(
    mut stream: crate::agent::llm_client::CompletionStream,
) -> Result<(String, Option<String>, Option<CompletionProviderFinalResult>)> {
    let mut accumulated_content = String::new();
    let mut accumulated_reasoning = String::new();
    let mut provider_final_result = None;

    while let Some(chunk) = stream.next().await {
        match chunk? {
            CompletionChunk::Delta { content, reasoning } => {
                if !content.is_empty() {
                    accumulated_content.push_str(&content);
                }
                if let Some(reasoning) = reasoning.filter(|value| !value.is_empty()) {
                    if !accumulated_reasoning.is_empty() {
                        accumulated_reasoning.push('\n');
                    }
                    accumulated_reasoning.push_str(&reasoning);
                }
            }
            CompletionChunk::Done {
                content,
                reasoning,
                provider_final_result: final_result,
                ..
            } => {
                if !content.trim().is_empty() {
                    accumulated_content = content;
                }
                if let Some(reasoning) = reasoning.filter(|value| !value.trim().is_empty()) {
                    accumulated_reasoning = reasoning;
                }
                provider_final_result = final_result;
                break;
            }
            CompletionChunk::ToolCalls {
                tool_calls,
                content,
                reasoning,
                provider_final_result: final_result,
                ..
            } => {
                if !tool_calls.is_empty() {
                    anyhow::bail!(
                        "media analysis received unexpected tool calls from the provider"
                    );
                }
                if let Some(content) = content.filter(|value| !value.trim().is_empty()) {
                    accumulated_content = content;
                }
                if let Some(reasoning) = reasoning.filter(|value| !value.trim().is_empty()) {
                    accumulated_reasoning = reasoning;
                }
                provider_final_result = final_result;
                break;
            }
            CompletionChunk::TransportFallback { .. } | CompletionChunk::Retry { .. } => {}
            CompletionChunk::Error { message } => return Err(anyhow::anyhow!(message)),
        }
    }

    Ok((
        accumulated_content,
        (!accumulated_reasoning.trim().is_empty()).then_some(accumulated_reasoning),
        provider_final_result,
    ))
}

fn image_support_error(provider_id: &str, model: &str) -> anyhow::Error {
    let alternatives = crate::agent::types::models_supporting(
        provider_id,
        crate::agent::types::Modality::Image,
    );
    if alternatives.is_empty() {
        anyhow::anyhow!(
            "model '{}' for provider '{}' is not known to support image inputs",
            model,
            provider_id
        )
    } else {
        anyhow::anyhow!(
            "model '{}' for provider '{}' is not known to support image inputs. Try one of: {}",
            model,
            provider_id,
            alternatives.join(", ")
        )
    }
}

fn temp_output_path(prefix: &str, extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tamux-{}-{}.{}",
        prefix,
        uuid::Uuid::new_v4(),
        extension.trim_start_matches('.')
    ))
}

async fn execute_analyze_image(
    args: &serde_json::Value,
    agent: &AgentEngine,
    http_client: &reqwest::Client,
) -> Result<String> {
    let (provider_id, provider_config) = resolve_media_provider_config(agent, args, None).await?;
    if !crate::agent::types::model_supports(
        &provider_id,
        &provider_config.model,
        crate::agent::types::Modality::Image,
    ) {
        return Err(image_support_error(&provider_id, &provider_config.model));
    }

    let prompt = args
        .get("prompt")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_IMAGE_ANALYSIS_PROMPT);
    let source = resolve_media_input_source(args, MediaKind::Image).await?;
    let (transport, blocks) = build_image_analysis_blocks(&provider_id, &provider_config, prompt, &source)?;
    let messages = vec![ApiMessage {
        role: "user".to_string(),
        content: ApiContent::Blocks(blocks),
        tool_call_id: None,
        name: None,
        tool_calls: None,
    }];

    let stream = crate::agent::llm_client::send_completion_request(
        http_client,
        &provider_id,
        &provider_config,
        "",
        &messages,
        &[],
        transport,
        None,
        None,
        RetryStrategy::Bounded {
            max_retries: 1,
            retry_delay_ms: 1_000,
        },
    );
    let (content, reasoning, provider_final_result) = collect_completion_output(stream).await?;
    if content.trim().is_empty() {
        anyhow::bail!("image analysis returned no content");
    }

    let mut response = content;
    if args
        .get("include_reasoning")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        if let Some(reasoning) = reasoning.filter(|value| !value.trim().is_empty()) {
            response.push_str("\n\n[reasoning]\n");
            response.push_str(&reasoning);
        }
    }
    if args
        .get("include_provider_result")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        if let Some(provider_final_result) = provider_final_result {
            response.push_str("\n\n[provider_final_result]\n");
            response.push_str(&serde_json::to_string_pretty(&provider_final_result)?);
        }
    }

    Ok(response)
}

async fn execute_generate_image(
    args: &serde_json::Value,
    agent: &AgentEngine,
    http_client: &reqwest::Client,
) -> Result<String> {
    let prompt = args
        .get("prompt")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing 'prompt' argument"))?;
    let (provider_id, provider_config) =
        resolve_media_provider_config(agent, args, Some(DEFAULT_IMAGE_GENERATION_MODEL)).await?;
    ensure_openai_like_media_endpoint(&provider_id, &provider_config)?;

    let output_format = args
        .get("output_format")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_IMAGE_OUTPUT_FORMAT);
    let url = openai_like_endpoint(&provider_config.base_url, "images/generations");
    let mut body = serde_json::json!({
        "model": provider_config.model,
        "prompt": prompt,
        "response_format": "b64_json",
    });
    if let Some(size) = args
        .get("size")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        body["size"] = serde_json::Value::String(size.to_string());
    }
    if let Some(quality) = args
        .get("quality")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        body["quality"] = serde_json::Value::String(quality.to_string());
    }
    if let Some(background) = args
        .get("background")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        body["background"] = serde_json::Value::String(background.to_string());
    }
    if let Some(style) = args
        .get("style")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        body["style"] = serde_json::Value::String(style.to_string());
    }
    body["output_format"] = serde_json::Value::String(output_format.to_string());

    let request = apply_media_auth_headers(http_client.post(&url), &provider_id, &provider_config)?;
    let response = request.json(&body).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("image generation failed: HTTP {status} {text}");
    }

    let payload: serde_json::Value = response.json().await?;
    let first = payload
        .get("data")
        .and_then(|value| value.as_array())
        .and_then(|items| items.first())
        .ok_or_else(|| anyhow::anyhow!("image generation returned no image payload"))?;

    let revised_prompt = first
        .get("revised_prompt")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);

    if let Some(b64_json) = first.get("b64_json").and_then(|value| value.as_str()) {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64_json)
            .map_err(|error| anyhow::anyhow!("invalid image base64 payload: {error}"))?;
        let mime_type = match output_format {
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            _ => "image/png",
        };
        let extension = file_extension_for_generated_mime(mime_type, output_format);
        let output_path = temp_output_path("image", &extension);
        tokio::fs::write(&output_path, &bytes).await?;
        return serde_json::to_string_pretty(&serde_json::json!({
            "provider": provider_id,
            "model": provider_config.model,
            "path": output_path,
            "mime_type": mime_type,
            "bytes": bytes.len(),
            "revised_prompt": revised_prompt,
        }))
        .map_err(Into::into);
    }

    if let Some(url) = first.get("url").and_then(|value| value.as_str()) {
        return serde_json::to_string_pretty(&serde_json::json!({
            "provider": provider_id,
            "model": provider_config.model,
            "url": url,
            "revised_prompt": revised_prompt,
        }))
        .map_err(Into::into);
    }

    anyhow::bail!("image generation returned neither 'b64_json' nor 'url'")
}

async fn execute_speech_to_text(
    args: &serde_json::Value,
    agent: &AgentEngine,
    http_client: &reqwest::Client,
) -> Result<String> {
    let path = args
        .get("path")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing 'path' argument"))?;
    validate_read_path(path)?;
    let resolved = resolve_tool_path(path, None);
    let bytes = tokio::fs::read(&resolved).await?;
    let filename = resolved
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("audio.bin")
        .to_string();
    let mime_type = args
        .get("mime_type")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| infer_media_mime(&resolved, MediaKind::Audio).to_string());
    let (provider_id, provider_config) =
        resolve_media_provider_config(agent, args, Some(DEFAULT_STT_MODEL)).await?;
    let route = resolve_audio_tool_route(
        &provider_id,
        &provider_config,
        amux_shared::providers::AudioToolKind::SpeechToText,
    )?;

    if route == AudioToolRoute::ProviderMultimodalCompletion {
        return execute_multimodal_speech_to_text(
            args,
            http_client,
            &provider_id,
            &provider_config,
            &mime_type,
            &bytes,
        )
        .await;
    }

    let url = stt_endpoint_for_route(&provider_config.base_url, route);
    let response_format = args
        .get("response_format")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("json");
    let form = if route == AudioToolRoute::XaiStt {
        let fields = build_xai_stt_multipart_fields(args, &filename, &mime_type);
        ordered_fields_to_multipart_form(&fields, bytes)?
    } else {
        let file_part = reqwest::multipart::Part::bytes(bytes)
            .file_name(filename)
            .mime_str(&mime_type)?;
        let mut form = reqwest::multipart::Form::new()
            .part("file", file_part)
            .text("model", provider_config.model.clone());
        if let Some(language) = args
            .get("language")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            form = form.text("language", language.to_string());
        }
        if let Some(prompt) = args
            .get("prompt")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            form = form.text("prompt", prompt.to_string());
        }
        form.text("response_format", response_format.to_string())
    };

    let request = apply_media_auth_headers(http_client.post(&url), &provider_id, &provider_config)?;
    let response = request.multipart(form).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("speech_to_text failed: HTTP {status} {text}");
    }

    let body = response.text().await?;
    if route != AudioToolRoute::XaiStt && response_format == "text" {
        return Ok(body);
    }

    let parsed: serde_json::Value = serde_json::from_str(&body)
        .map_err(|error| anyhow::anyhow!("speech_to_text returned invalid JSON: {error}"))?;
    format_speech_to_text_response(route, response_format, &parsed)
}

async fn execute_text_to_speech(
    args: &serde_json::Value,
    agent: &AgentEngine,
    http_client: &reqwest::Client,
) -> Result<String> {
    let input = args
        .get("input")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing 'input' argument"))?;
    let (provider_id, provider_config) =
        resolve_media_provider_config(agent, args, Some(DEFAULT_TTS_MODEL)).await?;
    let route = resolve_audio_tool_route(
        &provider_id,
        &provider_config,
        amux_shared::providers::AudioToolKind::TextToSpeech,
    )?;

    let voice = if route == AudioToolRoute::XaiTts {
        xai_tts_voice(args)
    } else {
        args.get("voice")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_TTS_VOICE)
    };
    let output_format = args
        .get("response_format")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_TTS_OUTPUT_FORMAT);
    let url = tts_endpoint_for_route(&provider_config.base_url, route);
    let body = if route == AudioToolRoute::XaiTts {
        build_xai_tts_body(args, input)?
    } else {
        serde_json::json!({
            "model": provider_config.model,
            "input": input,
            "voice": voice,
            "response_format": output_format,
        })
    };

    let request = apply_media_auth_headers(http_client.post(&url), &provider_id, &provider_config)?;
    let response = request.json(&body).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        anyhow::bail!("text_to_speech failed: HTTP {status} {text}");
    }

    let bytes = response.bytes().await?;
    let mime_type = output_mime_type_for_codec(output_format);
    let extension = file_extension_for_generated_mime(mime_type, output_format);
    let output_path = temp_output_path("speech", &extension);
    tokio::fs::write(&output_path, &bytes).await?;

    serde_json::to_string_pretty(&serde_json::json!({
        "provider": provider_id,
        "model": provider_config.model,
        "voice": voice,
        "path": output_path,
        "mime_type": mime_type,
        "bytes": bytes.len(),
    }))
    .map_err(Into::into)
}

#[cfg(test)]
mod media_tools_tests {
    use super::*;

    #[test]
    fn infer_image_mime_from_extension() {
        assert_eq!(
            infer_media_mime(Path::new("/tmp/cat.png"), MediaKind::Image),
            "image/png"
        );
        assert_eq!(
            infer_media_mime(Path::new("/tmp/cat.jpeg"), MediaKind::Image),
            "image/jpeg"
        );
    }

    #[test]
    fn parse_data_url_extracts_mime_and_payload() {
        let (mime_type, data) =
            parse_data_url("data:image/png;base64,Zm9v").expect("data url should parse");
        assert_eq!(mime_type, "image/png");
        assert_eq!(data, "Zm9v");
    }

    #[test]
    fn openai_like_endpoint_respects_existing_v1_suffix() {
        assert_eq!(
            openai_like_endpoint("https://api.example.com/v1", "audio/speech"),
            "https://api.example.com/v1/audio/speech"
        );
        assert_eq!(
            openai_like_endpoint("https://api.example.com", "/audio/speech"),
            "https://api.example.com/v1/audio/speech"
        );
    }

    #[test]
    fn openrouter_speech_to_text_uses_multimodal_completion_route() {
        assert_eq!(
            audio_tool_route(
                amux_shared::providers::PROVIDER_ID_OPENROUTER,
                amux_shared::providers::AudioToolKind::SpeechToText,
            ),
            AudioToolRoute::ProviderMultimodalCompletion
        );
    }

    #[test]
    fn openrouter_text_to_speech_uses_tts_endpoint() {
        assert_eq!(
            audio_tool_route(
                amux_shared::providers::PROVIDER_ID_OPENROUTER,
                amux_shared::providers::AudioToolKind::TextToSpeech,
            ),
            AudioToolRoute::OpenRouterTts
        );
    }

    #[test]
    fn openrouter_transcription_blocks_include_audio_input() {
        let blocks = build_audio_transcription_blocks(
            ApiTransport::ChatCompletions,
            "Please transcribe this audio.",
            "UklGRg==",
            "wav",
        );

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0]["type"], "text");
        assert_eq!(blocks[1]["type"], "input_audio");
        assert_eq!(blocks[1]["input_audio"]["format"], "wav");
        assert_eq!(blocks[1]["input_audio"]["data"], "UklGRg==");
    }

    #[test]
    fn xai_speech_to_text_uses_xai_stt_route_and_endpoint() {
        let route = audio_tool_route(
            amux_shared::providers::PROVIDER_ID_XAI,
            amux_shared::providers::AudioToolKind::SpeechToText,
        );
        assert_eq!(route, AudioToolRoute::XaiStt);
        assert_eq!(
            stt_endpoint_for_route("https://api.x.ai/v1", route),
            "https://api.x.ai/v1/stt"
        );
    }

    #[test]
    fn xai_text_to_speech_uses_xai_tts_route_and_endpoint() {
        let route = audio_tool_route(
            amux_shared::providers::PROVIDER_ID_XAI,
            amux_shared::providers::AudioToolKind::TextToSpeech,
        );
        assert_eq!(route, AudioToolRoute::XaiTts);
        assert_eq!(
            tts_endpoint_for_route("https://api.x.ai/v1", route),
            "https://api.x.ai/v1/tts"
        );
    }

    #[test]
    fn xai_stt_multipart_fields_append_file_last() {
        let fields = build_xai_stt_multipart_fields(
            &serde_json::json!({
                "language": "en",
                "format": true,
                "audio_format": "wav",
                "sample_rate": 16000,
                "multichannel": false,
                "channels": 1,
                "diarize": true
            }),
            "clip.wav",
            "audio/wav",
        );

        assert_eq!(
            fields,
            vec![
                OrderedMultipartField::Text {
                    name: "language".to_string(),
                    value: "en".to_string()
                },
                OrderedMultipartField::Text {
                    name: "format".to_string(),
                    value: "true".to_string()
                },
                OrderedMultipartField::Text {
                    name: "audio_format".to_string(),
                    value: "wav".to_string()
                },
                OrderedMultipartField::Text {
                    name: "sample_rate".to_string(),
                    value: "16000".to_string()
                },
                OrderedMultipartField::Text {
                    name: "multichannel".to_string(),
                    value: "false".to_string()
                },
                OrderedMultipartField::Text {
                    name: "channels".to_string(),
                    value: "1".to_string()
                },
                OrderedMultipartField::Text {
                    name: "diarize".to_string(),
                    value: "true".to_string()
                },
                OrderedMultipartField::File {
                    name: "file".to_string(),
                    filename: "clip.wav".to_string(),
                    mime_type: "audio/wav".to_string()
                }
            ]
        );
    }

    #[test]
    fn xai_tts_body_maps_input_voice_language_and_output_format() {
        let body = build_xai_tts_body(
            &serde_json::json!({
                "input": "Hello from tamux",
                "voice": "nova",
                "language": "pl",
                "response_format": "wav",
                "sample_rate": 24000,
                "bit_rate": 128000
            }),
            "Hello from tamux",
        )
        .expect("xai body should build");

        assert_eq!(body["text"], "Hello from tamux");
        assert_eq!(body["voice_id"], "nova");
        assert_eq!(body["language"], "pl");
        assert_eq!(body["output_format"]["codec"], "wav");
        assert_eq!(body["output_format"]["sample_rate"], 24000);
        assert_eq!(body["output_format"]["bit_rate"], 128000);
        assert!(body.get("model").is_none());
    }

    #[test]
    fn xai_tts_body_includes_default_mp3_codec_when_response_format_is_omitted() {
        let body = build_xai_tts_body(
            &serde_json::json!({
                "input": "Hello from tamux",
                "language": "en"
            }),
            "Hello from tamux",
        )
        .expect("xai body should build");

        assert_eq!(body["output_format"]["codec"], "mp3");
    }

    #[test]
    fn xai_tts_body_defaults_language_to_auto() {
        let body = build_xai_tts_body(
            &serde_json::json!({
                "input": "Hello from tamux"
            }),
            "Hello from tamux",
        )
        .expect("xai body should default missing language");

        assert_eq!(body["language"], "auto");
    }

    #[test]
    fn xai_stt_formats_json_response_according_to_daemon_contract() {
        let payload = serde_json::json!({
            "text": "hello world",
            "duration": 1.23
        });

        assert_eq!(
            format_speech_to_text_response(AudioToolRoute::XaiStt, "text", &payload)
                .expect("xai text contract should parse"),
            "hello world"
        );
        assert_eq!(
            format_speech_to_text_response(AudioToolRoute::XaiStt, "json", &payload)
                .expect("xai json contract should pretty-print transcript payload"),
            serde_json::to_string_pretty(&payload).expect("payload should serialize")
        );
    }

    #[test]
    fn direct_provider_stt_text_contract_still_returns_plain_text_body() {
        let payload = serde_json::json!({
            "text": "hello world"
        });

        assert_eq!(
            format_speech_to_text_response(
                AudioToolRoute::OpenAiCompatibleDirect,
                "text",
                &payload
            )
            .expect("text contract should extract transcript"),
            "hello world"
        );
    }

    #[test]
    fn telephony_codecs_map_to_correct_tts_mime_types() {
        assert_eq!(output_mime_type_for_codec("mulaw"), "audio/basic");
        assert_eq!(output_mime_type_for_codec("alaw"), "audio/alaw");
    }

    #[tokio::test]
    async fn resolve_media_provider_config_uses_nested_audio_tts_settings() {
        let root = tempfile::tempdir().expect("tempdir should succeed");
        let manager = crate::session_manager::SessionManager::new_test(root.path()).await;
        let mut config = AgentConfig::default();
        config.provider = "openai".to_string();
        config.model = "gpt-5.4".to_string();
        config.api_key = "sk-active".to_string();
        config.extra.insert(
            "audio".to_string(),
            serde_json::json!({
                "tts": {
                    "provider": "custom",
                    "model": "sonic-voice"
                }
            }),
        );
        config.providers.insert(
            "custom".to_string(),
            crate::agent::types::ProviderConfig {
                base_url: "https://audio.example/v1".to_string(),
                model: "fallback-model".to_string(),
                api_key: "sk-audio".to_string(),
                assistant_id: String::new(),
                auth_source: crate::agent::types::AuthSource::ApiKey,
                api_transport: crate::agent::types::ApiTransport::Responses,
                reasoning_effort: "medium".to_string(),
                context_window_tokens: 128_000,
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
            },
        );

        let engine = AgentEngine::new_test(manager, config, root.path()).await;
        let (provider_id, provider_config) =
            resolve_media_provider_config(&engine, &serde_json::json!({}), Some(DEFAULT_TTS_MODEL))
                .await
                .expect("audio TTS settings should resolve");

        assert_eq!(provider_id, "custom");
        assert_eq!(provider_config.model, "sonic-voice");
        assert_eq!(provider_config.base_url, "https://audio.example/v1");
    }

    #[tokio::test]
    async fn resolve_media_provider_config_falls_back_to_flattened_legacy_audio_keys() {
        let root = tempfile::tempdir().expect("tempdir should succeed");
        let manager = crate::session_manager::SessionManager::new_test(root.path()).await;
        let mut config = AgentConfig::default();
        config.provider = "openai".to_string();
        config.model = "gpt-5.4".to_string();
        config.api_key = "sk-active".to_string();
        config.extra.insert(
            "audio_stt_provider".to_string(),
            serde_json::Value::String("custom".to_string()),
        );
        config.extra.insert(
            "audio_stt_model".to_string(),
            serde_json::Value::String("whisper-legacy".to_string()),
        );
        config.providers.insert(
            "custom".to_string(),
            crate::agent::types::ProviderConfig {
                base_url: "https://audio.example/v1".to_string(),
                model: "fallback-model".to_string(),
                api_key: "sk-audio".to_string(),
                assistant_id: String::new(),
                auth_source: crate::agent::types::AuthSource::ApiKey,
                api_transport: crate::agent::types::ApiTransport::Responses,
                reasoning_effort: "medium".to_string(),
                context_window_tokens: 128_000,
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
            },
        );

        let engine = AgentEngine::new_test(manager, config, root.path()).await;
        let (provider_id, provider_config) =
            resolve_media_provider_config(&engine, &serde_json::json!({}), Some(DEFAULT_STT_MODEL))
                .await
                .expect("legacy STT settings should resolve");

        assert_eq!(provider_id, "custom");
        assert_eq!(provider_config.model, "whisper-legacy");
        assert_eq!(provider_config.base_url, "https://audio.example/v1");
    }
}
