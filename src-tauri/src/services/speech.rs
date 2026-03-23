use crate::config::providers;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const DEFAULT_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const REQUEST_TIMEOUT_SECONDS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechProviderConfig {
    pub provider_name: String,
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub base_url: Option<String>,
    pub voice: String,
    pub audio_format: String,
}

#[derive(Debug, Clone)]
pub struct SpeechRequestDebug {
    pub success: bool,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub request_payload: Value,
    pub raw_response: Option<Value>,
    pub audio_base64: Option<String>,
    pub error: Option<String>,
}

pub fn normalize_chat_completions_url(base_url: &str) -> String {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return DEFAULT_API_URL.to_string();
    }

    let normalized = trimmed.trim_end_matches('/');
    if normalized.contains("/chat/completions") {
        return normalized.to_string();
    }
    if normalized.ends_with("/v1") {
        return format!("{normalized}/chat/completions");
    }
    if let Ok(parsed) = reqwest::Url::parse(normalized) {
        let path = parsed.path().trim_end_matches('/');
        if path.is_empty() || path == "/" {
            return format!("{normalized}/v1/chat/completions");
        }
    }

    normalized.to_string()
}

pub async fn synthesize_openai_speech(
    config: &SpeechProviderConfig,
    text: &str,
) -> SpeechRequestDebug {
    let model = if config.model.trim().is_empty() {
        providers::speech_default_model_for_provider(&config.provider_name)
            .unwrap_or("")
            .to_string()
    } else {
        config.model.trim().to_string()
    };
    let voice = if config.voice.trim().is_empty() {
        providers::speech_default_voice_for_provider(&config.provider_name)
            .unwrap_or(providers::DEFAULT_XIAOMI_TTS_VOICE)
            .to_string()
    } else {
        config.voice.trim().to_string()
    };
    let audio_format = if config.audio_format.trim().is_empty() {
        providers::speech_default_audio_format_for_provider(&config.provider_name)
            .unwrap_or(providers::DEFAULT_TTS_AUDIO_FORMAT)
            .to_string()
    } else {
        config.audio_format.trim().to_string()
    };

    let resolved_base_url = normalize_chat_completions_url(
        config
            .base_url
            .as_deref()
            .or_else(|| providers::speech_default_base_url_for_provider(&config.provider_name))
            .unwrap_or(DEFAULT_API_URL),
    );

    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "assistant",
                "content": text
            }
        ],
        "audio": {
            "voice": voice,
            "format": audio_format
        },
        "stream": false
    });

    let client = match Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            return SpeechRequestDebug {
                success: false,
                status_code: 0,
                response_time_ms: 0,
                request_payload: payload,
                raw_response: None,
                audio_base64: None,
                error: Some(format!("构建 HTTP 客户端失败: {e}")),
            };
        }
    };

    let mut builder = client
        .post(&resolved_base_url)
        .header("Content-Type", "application/json");
    if !config.api_key.trim().is_empty() {
        builder = builder
            .header("Authorization", format!("Bearer {}", config.api_key.trim()))
            .header("api-key", config.api_key.trim());
    }

    let started_at = Instant::now();
    let response = builder.json(&payload).send().await;
    let response_time_ms = started_at.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let body_text = resp.text().await.unwrap_or_default();
            let parsed_json = serde_json::from_str::<Value>(&body_text).ok();
            let is_success_status = (200..300).contains(&status_code);

            if !is_success_status {
                return SpeechRequestDebug {
                    success: false,
                    status_code,
                    response_time_ms,
                    request_payload: payload,
                    raw_response: parsed_json,
                    audio_base64: None,
                    error: Some(body_text),
                };
            }

            let Some(raw_response) = parsed_json else {
                return SpeechRequestDebug {
                    success: false,
                    status_code,
                    response_time_ms,
                    request_payload: payload,
                    raw_response: None,
                    audio_base64: None,
                    error: Some("响应不是有效 JSON，无法提取音频".to_string()),
                };
            };

            let audio_base64 = extract_audio_base64(&raw_response);
            if audio_base64.is_none() {
                return SpeechRequestDebug {
                    success: false,
                    status_code,
                    response_time_ms,
                    request_payload: payload,
                    raw_response: Some(raw_response),
                    audio_base64: None,
                    error: Some(
                        "响应中未找到音频数据（choices[0].message.audio.data）".to_string(),
                    ),
                };
            }

            SpeechRequestDebug {
                success: true,
                status_code,
                response_time_ms,
                request_payload: payload,
                raw_response: Some(raw_response),
                audio_base64,
                error: None,
            }
        }
        Err(e) => SpeechRequestDebug {
            success: false,
            status_code: 0,
            response_time_ms,
            request_payload: payload,
            raw_response: None,
            audio_base64: None,
            error: Some(e.to_string()),
        },
    }
}

fn extract_audio_base64(raw_response: &Value) -> Option<String> {
    if let Some(data) = raw_response
        .pointer("/choices/0/message/audio/data")
        .and_then(|value| value.as_str())
    {
        return Some(data.to_string());
    }
    raw_response
        .pointer("/choices/0/delta/audio/data")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}
