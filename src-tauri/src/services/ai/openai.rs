// OpenAI Provider
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

use super::provider::{
    AIError, AIProvider, ApiTestResponse, ProviderConfig, StreamEvent, StreamSender,
    TranslationRequest, TranslationResponse,
};
use crate::config::{prompts, providers};

const DEFAULT_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const REQUEST_TIMEOUT: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    content: String,
}

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

        Self {
            client,
            api_key: config.api_key.clone(),
            model: if config.model.is_empty() {
                DEFAULT_MODEL.to_string()
            } else {
                config.model.clone()
            },
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| DEFAULT_API_URL.to_string()),
        }
    }

    fn build_prompt(&self, request: &TranslationRequest) -> String {
        format!(
            r#"Translate the following text from {source} to {target}.

    Do not output any explanations or JSON, just the translated text.

    Text:

    {text}"#,
            source = request.source_lang,
            target = request.target_lang,
            text = request.text,
        )
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

        let prompt = format!(

                "Detect the language of the following text. Return ONLY the ISO 639-1 language code (e.g., 'en', 'zh', 'ja'). Do not explain.\n\nText: {}",

                text

            );

        let payload = json!({

            "model": self.model,

            "messages": [

                { "role": "user", "content": prompt }

            ],

            "stream": false

        });

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AIError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AIError::RequestFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let resp_text = response.text().await.unwrap_or_default();

        let chat_response: ChatResponse =
            serde_json::from_str(&resp_text).map_err(|e| AIError::ParseError(e.to_string()))?;

        if let Some(choice) = chat_response.choices.first() {
            let lang = choice.message.content.trim().to_string();

            let lang = lang.split_whitespace().last().unwrap_or(&lang).to_string();

            let lang = lang
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string();

            Ok(lang)
        } else {
            Err(AIError::ParseError("无法检测语言".to_string()))
        }
    }

    async fn translate(
        &self,
        request: &TranslationRequest,
    ) -> Result<TranslationResponse, AIError> {
        if self.api_key.is_empty() {
            return Err(AIError::InvalidApiKey("OpenAI API Key 未设置".to_string()));
        }

        let system_prompt = prompts::SYSTEM_PROMPT;

        let user_prompt = self.build_prompt(request);

        let payload = json!({

            "model": self.model,

            "messages": [

                { "role": "system", "content": system_prompt },

                { "role": "user", "content": user_prompt }

            ],

            "stream": false

        });

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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

        let chat_response: ChatResponse =
            serde_json::from_str(&resp_text).map_err(|e| AIError::ParseError(e.to_string()))?;

        if let Some(choice) = chat_response.choices.first() {
            Ok(TranslationResponse {
                translation: choice.message.content.clone(),

                provider: self.name().to_string(),

                model: self.model.clone(),
            })
        } else {
            Err(AIError::ParseError("无翻译结果返回".to_string()))
        }
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

            "messages": [

                { "role": "user", "content": "Hello" }

            ],

            "stream": false

        });

        let start = Instant::now();

        let result = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        let response_time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(response) => {
                let status_code = response.status().as_u16();

                let resp_text = response.text().await.unwrap_or_default();

                let raw_response = serde_json::from_str::<Value>(&resp_text).ok();

                let success = status_code >= 200 && status_code < 300;

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

        // 发送开始事件

        let _ = sender
            .send(StreamEvent::Start {
                provider: self.name().to_string(),

                model: self.model.clone(),

                request_id,
            })
            .await;

        let system_prompt = prompts::SYSTEM_PROMPT;

        let user_prompt = self.build_prompt(request);

        let payload = json!({



            "model": self.model,



            "messages": [



                { "role": "system", "content": system_prompt },



                { "role": "user", "content": user_prompt }



            ],



            "stream": true



        });

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
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

        // 处理 SSE 流

        let mut full_content = String::new();

        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    let chunk_str = String::from_utf8_lossy(&bytes);

                    // SSE 格式: data: {...}\n\n

                    for line in chunk_str.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data.trim() == "[DONE]" {
                                continue;
                            }

                            if let Ok(json_val) = serde_json::from_str::<Value>(data) {
                                if let Some(content) = json_val
                                    .get("choices")
                                    .and_then(|c| c.get(0))
                                    .and_then(|c| c.get("delta"))
                                    .and_then(|d| d.get("content"))
                                    .and_then(|c| c.as_str())
                                {
                                    full_content.push_str(content);

                                    let _ = sender
                                        .send(StreamEvent::Chunk {
                                            provider: self.name().to_string(),

                                            content: content.to_string(),

                                            request_id,
                                        })
                                        .await;
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

        // 发送完成事件

        let _ = sender
            .send(StreamEvent::Done {
                provider: self.name().to_string(),

                model: self.model.clone(),

                detected_source_lang: request.source_lang.clone(),

                target_lang: request.target_lang.clone(),

                full_translation: full_content,

                request_id,
            })
            .await;

        Ok(())
    }
}

#[allow(dead_code)]
fn try_parse_json(content: &str) -> Option<Value> {
    if let Ok(val) = serde_json::from_str::<Value>(content) {
        return Some(val);
    }

    let fence_variants = ["```json", "```"];

    for fence in fence_variants {
        if let Some(start) = content.find(fence) {
            let rest = &content[start + fence.len()..];

            if let Some(end) = rest.find("```") {
                let block = &rest[..end];

                if let Ok(val) = serde_json::from_str::<Value>(block) {
                    return Some(val);
                }
            }
        }
    }

    let stripped = content.trim_matches('`').trim();

    if let Ok(val) = serde_json::from_str::<Value>(stripped) {
        return Some(val);
    }

    None
}
