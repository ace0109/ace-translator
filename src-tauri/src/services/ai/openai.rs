// OpenAI Provider
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

use super::provider::{
    AIError, AIProvider, ApiTestResponse, ProviderConfig, StreamEvent, StreamSender,
    TranslationRequest, TranslationResponse,
};
use crate::config::{prompts, providers};

const DEFAULT_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = providers::DEFAULT_OPENAI_MODEL;
const REQUEST_TIMEOUT: u64 = 30;

/// OpenAI Provider
pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT))
            .build()
            .unwrap_or_default();

        let model = if config.model.trim().is_empty() {
            DEFAULT_MODEL.to_string()
        } else {
            config.model.trim().to_string()
        };

        let raw_base_url = config.base_url.as_deref().unwrap_or(DEFAULT_API_URL);
        let base_url = normalize_chat_completions_url(raw_base_url);

        Self {
            client,
            api_key: config.api_key.clone(),
            model,
            base_url,
        }
    }

    fn build_prompt(&self, request: &TranslationRequest) -> String {
        prompts::build_translation_user_prompt(
            &request.source_lang,
            &request.target_lang,
            &request.text,
        )
    }

    fn request_builder(&self) -> reqwest::RequestBuilder {
        self.client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "OpenAI"
    }

    fn available_models(&self) -> Vec<&'static str> {
        providers::OPENAI_MODELS.iter().map(|m| m.id).collect()
    }

    fn current_model(&self) -> &str {
        &self.model
    }

    async fn detect_language(&self, text: &str) -> Result<String, AIError> {
        if self.api_key.is_empty() {
            return Err(AIError::InvalidApiKey("OpenAI API Key 未设置".to_string()));
        }

        let prompt = prompts::build_language_detection_prompt(text);
        let payload = json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": prompt }],
            "stream": false
        });

        let response = self
            .request_builder()
            .json(&payload)
            .send()
            .await
            .map_err(|e| AIError::RequestFailed(e.to_string()))?;

        let status = response.status();
        let resp_text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(AIError::RequestFailed(format!(
                "HTTP {}: {}",
                status, resp_text
            )));
        }

        let resp_json: Value =
            serde_json::from_str(&resp_text).map_err(|e| AIError::ParseError(e.to_string()))?;

        let raw_lang = extract_first_choice_text(&resp_json)
            .ok_or_else(|| AIError::ParseError("无法检测语言".to_string()))?;
        Ok(canonicalize_detected_language(&raw_lang))
    }

    async fn translate(
        &self,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, AIError> {
        if self.api_key.is_empty() {
            return Err(AIError::InvalidApiKey("OpenAI API Key 未设置".to_string()));
        }

        let payload = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": prompts::SYSTEM_PROMPT },
                { "role": "user", "content": self.build_prompt(request) }
            ],
            "stream": false
        });

        let response = self
            .request_builder()
            .json(&payload)
            .send()
            .await
            .map_err(|e| AIError::RequestFailed(e.to_string()))?;

        let status = response.status();
        let resp_text = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(AIError::RequestFailed(format!(
                "HTTP {}: {}",
                status, resp_text
            )));
        }

        let resp_json: Value =
            serde_json::from_str(&resp_text).map_err(|e| AIError::ParseError(e.to_string()))?;
        let raw_translation = extract_first_choice_text(&resp_json)
            .ok_or_else(|| AIError::ParseError("无翻译结果返回".to_string()))?;
        let translation = prompts::normalize_translation_output(&raw_translation);

        Ok(TranslationResponse {
            translation,
            provider: self.name().to_string(),
            model: self.model.clone(),
        })
    }

    async fn test_api(&self) -> ApiTestResponse {
        if self.api_key.is_empty() {
            return ApiTestResponse {
                success: false,
                status_code: 0,
                response_time_ms: 0,
                raw_response: None,
                request_payload: None,
                error: Some("API Key 未设置".to_string()),
                provider: self.name().to_string(),
                model: self.model.clone(),
            };
        }

        let payload = json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": "Hello" }],
            "stream": false
        });

        let start = Instant::now();
        let result = self.request_builder().json(&payload).send().await;
        let response_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => {
                let status_code = response.status().as_u16();
                let resp_text = response.text().await.unwrap_or_default();
                let raw_response = serde_json::from_str::<Value>(&resp_text).ok();
                let success = (200..300).contains(&status_code);

                ApiTestResponse {
                    success,
                    status_code,
                    response_time_ms,
                    raw_response,
                    request_payload: Some(payload),
                    error: if success { None } else { Some(resp_text) },
                    provider: self.name().to_string(),
                    model: self.model.clone(),
                }
            }
            Err(e) => ApiTestResponse {
                success: false,
                status_code: 0,
                response_time_ms,
                raw_response: None,
                request_payload: Some(payload),
                error: Some(e.to_string()),
                provider: self.name().to_string(),
                model: self.model.clone(),
            },
        }
    }

    async fn translate_stream(
        &self,
        request: &TranslationRequest,
        sender: StreamSender,
        request_id: u64,
    ) -> Result<(), AIError> {
        if self.api_key.is_empty() {
            let _ = sender
                .send(StreamEvent::Error {
                    provider: self.name().to_string(),
                    error: "OpenAI API Key 未设置".to_string(),
                    request_id,
                })
                .await;
            return Err(AIError::InvalidApiKey("OpenAI API Key 未设置".to_string()));
        }

        let _ = sender
            .send(StreamEvent::Start {
                provider: self.name().to_string(),
                model: self.model.clone(),
                request_id,
            })
            .await;

        let payload = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": prompts::SYSTEM_PROMPT },
                { "role": "user", "content": self.build_prompt(request) }
            ],
            "stream": true
        });

        let response = self
            .request_builder()
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                let _ = futures::executor::block_on(sender.send(StreamEvent::Error {
                    provider: self.name().to_string(),
                    error: e.to_string(),
                    request_id,
                }));
                AIError::RequestFailed(e.to_string())
            })?;

        let status = response.status();
        if !status.is_success() {
            let resp_text = response.text().await.unwrap_or_default();
            let error_msg = format!("HTTP {}: {}", status, resp_text);
            let _ = sender
                .send(StreamEvent::Error {
                    provider: self.name().to_string(),
                    error: error_msg.clone(),
                    request_id,
                })
                .await;
            return Err(AIError::RequestFailed(error_msg));
        }

        let mut full_content = String::new();
        let mut sse_buffer = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    let chunk_str = String::from_utf8_lossy(&bytes);
                    sse_buffer.push_str(&chunk_str.replace("\r\n", "\n"));

                    let events = drain_sse_events(&mut sse_buffer);
                    for event_raw in events {
                        if let Some(data) = parse_sse_data(&event_raw) {
                            if data.trim() == "[DONE]" {
                                continue;
                            }
                            if let Ok(json_val) = serde_json::from_str::<Value>(&data) {
                                if let Some(content) = extract_stream_choice_text(&json_val) {
                                    if !content.is_empty() {
                                        full_content.push_str(&content);
                                        let _ = sender
                                            .send(StreamEvent::Chunk {
                                                provider: self.name().to_string(),
                                                content,
                                                request_id,
                                            })
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = sender
                        .send(StreamEvent::Error {
                            provider: self.name().to_string(),
                            error: e.to_string(),
                            request_id,
                        })
                        .await;
                    return Err(AIError::RequestFailed(e.to_string()));
                }
            }
        }

        // 处理流结束前剩余缓冲，兼容最后一个 event 未以空行结尾的场景
        if !sse_buffer.trim().is_empty() {
            if let Some(data) = parse_sse_data(&sse_buffer) {
                if data.trim() != "[DONE]" {
                    if let Ok(json_val) = serde_json::from_str::<Value>(&data) {
                        if let Some(content) = extract_stream_choice_text(&json_val) {
                            if !content.is_empty() {
                                full_content.push_str(&content);
                            }
                        }
                    }
                }
            }
        }

        let _ = sender
            .send(StreamEvent::Done {
                provider: self.name().to_string(),
                model: self.model.clone(),
                detected_source_lang: request.source_lang.clone(),
                target_lang: request.target_lang.clone(),
                full_translation: prompts::normalize_translation_output(&full_content),
                request_id,
            })
            .await;

        Ok(())
    }
}

fn normalize_chat_completions_url(base_url: &str) -> String {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return DEFAULT_API_URL.to_string();
    }

    let normalized = trimmed.trim_end_matches('/');
    if normalized.contains("/chat/completions") {
        return normalized.to_string();
    }
    if normalized.ends_with("/v1") {
        return format!("{}/chat/completions", normalized);
    }
    if let Ok(parsed) = reqwest::Url::parse(normalized) {
        let path = parsed.path().trim_end_matches('/');
        if path.is_empty() || path == "/" {
            return format!("{}/v1/chat/completions", normalized);
        }
    }

    normalized.to_string()
}

#[allow(dead_code)]
fn canonicalize_detected_language(raw: &str) -> String {
    let last = raw.split_whitespace().last().unwrap_or(raw);
    let cleaned = last
        .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .to_lowercase();

    match cleaned.as_str() {
        "zh" | "zh-cn" | "zh_hans" | "zh-hans" | "zh-hans-cn" => "zh-CN".to_string(),
        "zh-tw" | "zh_tw" | "zh-hant" | "zh_hant" | "zh-hant-tw" => "zh-TW".to_string(),
        "en-us" | "en-gb" => "en".to_string(),
        _ => cleaned,
    }
}

fn extract_first_choice_text(resp_json: &Value) -> Option<String> {
    let choice = resp_json.get("choices")?.get(0)?;
    if let Some(content) = choice.get("message").and_then(|m| m.get("content")) {
        return extract_text_from_content(content);
    }
    if let Some(text) = choice.get("text").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    None
}

fn extract_stream_choice_text(resp_json: &Value) -> Option<String> {
    let choice = resp_json.get("choices")?.get(0)?;

    if let Some(content) = choice.get("delta").and_then(|d| d.get("content")) {
        return extract_text_from_content(content);
    }
    if let Some(content) = choice.get("message").and_then(|m| m.get("content")) {
        return extract_text_from_content(content);
    }
    None
}

fn extract_text_from_content(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.to_string()),
        Value::Array(arr) => {
            let mut out = String::new();
            for item in arr {
                match item {
                    Value::String(s) => out.push_str(s),
                    Value::Object(map) => {
                        if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                            out.push_str(text);
                        } else if let Some(text) = map.get("content").and_then(|v| v.as_str()) {
                            out.push_str(text);
                        }
                    }
                    _ => {}
                }
            }
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        }
        Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                Some(text.to_string())
            } else {
                map.get("content")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }
        }
        _ => None,
    }
}

fn drain_sse_events(buffer: &mut String) -> Vec<String> {
    let mut events = Vec::new();
    while let Some(idx) = buffer.find("\n\n") {
        let event = buffer[..idx].to_string();
        buffer.drain(..idx + 2);
        if !event.trim().is_empty() {
            events.push(event);
        }
    }
    events
}

fn parse_sse_data(event_raw: &str) -> Option<String> {
    let mut data_lines = Vec::new();
    for line in event_raw.lines() {
        let normalized_line = line.trim_end_matches('\r');
        if let Some(data) = normalized_line.strip_prefix("data:") {
            data_lines.push(data.trim_start().to_string());
        }
    }

    if data_lines.is_empty() {
        None
    } else {
        Some(data_lines.join("\n"))
    }
}
