// Ollama Provider (本地模型)
use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

use super::provider::{AIError, AIProvider, ApiTestResponse, ProviderConfig, TranslationRequest, TranslationResponse, StreamSender, StreamEvent};
use crate::config::prompts;

const DEFAULT_API_URL: &str = "http://localhost:11434/api/chat";
const DEFAULT_MODEL: &str = "llama3.2";
const REQUEST_TIMEOUT: u64 = 60; // 本地模型可能需要更长时间

/// Ollama 常用模型（用户可自行输入）
pub const OLLAMA_MODELS: &[&str] = &[
    "llama3.2",
    "llama3.1",
    "qwen2.5",
    "mistral",
    "gemma2",
];

#[derive(Debug, Serialize, Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OllamaMessage {
    content: String,
}

/// Ollama Provider
pub struct OllamaProvider {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT))
            .build()
            .unwrap_or_default();

        Self {
            client,
            model: if config.model.is_empty() {
                DEFAULT_MODEL.to_string()
            } else {
                config.model.clone()
            },
            base_url: config.base_url.clone().unwrap_or_else(|| DEFAULT_API_URL.to_string()),
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

    impl AIProvider for OllamaProvider {

        fn name(&self) -> &'static str {

            "Ollama"

        }

    

        fn available_models(&self) -> Vec<&'static str> {

            OLLAMA_MODELS.to_vec()

        }

    

        fn current_model(&self) -> &str {

            &self.model

        }

    

        async fn detect_language(&self, text: &str) -> Result<String, AIError> {

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

    

            let response = self.client

                .post(&self.base_url)

                .header("Content-Type", "application/json")

                .json(&payload)

                .send()

                .await

                .map_err(|e| {

                    if e.is_connect() {

                        AIError::ServiceUnavailable("无法连接到 Ollama 服务".to_string())

                    } else {

                        AIError::RequestFailed(e.to_string())

                    }

                })?;

    

            if !response.status().is_success() {

                return Err(AIError::RequestFailed(format!("HTTP {}", response.status())));

            }

    

            let ollama_response: OllamaResponse = serde_json::from_str(&response.text().await.unwrap_or_default())

                .map_err(|e| AIError::ParseError(e.to_string()))?;

    

            let lang = ollama_response.message.content.trim().to_string();

            // 简单清理

            let lang = lang.split_whitespace().last().unwrap_or(&lang).to_string();

            let lang = lang.trim_matches(|c: char| !c.is_alphanumeric() && c != '-').to_string();

            Ok(lang)

        }

    

        async fn translate(&self, request: &TranslationRequest) -> Result<TranslationResponse, AIError> {

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

    

            let response = self.client

                .post(&self.base_url)

                .header("Content-Type", "application/json")

                .json(&payload)

                .send()

                .await

                .map_err(|e| {

                    if e.is_connect() {

                        AIError::ServiceUnavailable("无法连接到 Ollama 服务，请确保 Ollama 正在运行".to_string())

                    } else if e.is_timeout() {

                        AIError::Timeout("Ollama 请求超时".to_string())

                    } else {

                        AIError::RequestFailed(e.to_string())

                    }

                })?;

    

            let status = response.status();

            let resp_text = response.text().await.unwrap_or_default();

    

            if !status.is_success() {

                return Err(AIError::RequestFailed(format!("HTTP {}: {}", status, resp_text)));

            }

    

            let ollama_response: OllamaResponse = serde_json::from_str(&resp_text)

                .map_err(|e| AIError::ParseError(e.to_string()))?;

    

                    Ok(TranslationResponse {

    

                        translation: ollama_response.message.content.clone(),

    

                        provider: self.name().to_string(),

    

                        model: self.model.clone(),

    

                    })

    

                }

    

        async fn test_connection(&self) -> Result<bool, AIError> {

            // Ollama 特殊处理：先检查服务是否可用

            let tags_url = self.base_url.replace("/api/chat", "/api/tags");

    

            self.client

                .get(&tags_url)

                .send()

                .await

                .map_err(|e| {

                    if e.is_connect() {

                        AIError::ServiceUnavailable("无法连接到 Ollama 服务".to_string())

                    } else {

                        AIError::RequestFailed(e.to_string())

                    }

                })?;

    

            Ok(true)

        }

    

            async fn test_api(&self) -> ApiTestResponse {
            let tags_url = self.base_url.replace("/api/chat", "/api/tags");
            let payload = json!({ "url": tags_url }); // Dummy payload for display

            let start = Instant::now();
            let tags_result = self.client.get(&tags_url).send().await;
            let response_time_ms = start.elapsed().as_millis() as u64;

            match tags_result {
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
                Err(e) => {
                    let error_msg = if e.is_connect() {
                        "无法连接到 Ollama 服务，请确保 Ollama 正在运行".to_string()
                    } else if e.is_timeout() {
                        "Ollama 请求超时".to_string()
                    } else {
                        e.to_string()
                    };

                    ApiTestResponse {
                        success: false,
                        status_code: 0,
                        response_time_ms,
                        raw_response: None,
                        request_payload: Some(payload),
                        error: Some(error_msg),
                        provider: self.name().to_string(),
                        model: self.model.clone(),
                    }
                }
            }
        }

        async fn translate_stream(

    

                &self,

    

                request: &TranslationRequest,

    

                sender: StreamSender,

    

                request_id: u64,

    

            ) -> Result<(), AIError> {

    

                // 发送开始事件

    

                let _ = sender.send(StreamEvent::Start {

    

                    provider: self.name().to_string(),

    

                    model: self.model.clone(),

    

                    request_id,

    

                }).await;

    

        

    

                let system_prompt = prompts::SYSTEM_PROMPT;

    

                let user_prompt = self.build_prompt(request);

    

        

    

                let payload = json!({

    

                    "model": self.model,

    

                    "messages": [

    

                        { "role": "system", "content": system_prompt },

    

                        { "role": "user", "content": user_prompt }

    

                    ],

    

                    "stream": true, // Enable streaming

    

                });

    

        

    

                let response = self.client

    

                    .post(&self.base_url)

    

                    .header("Content-Type", "application/json")

    

                    .json(&payload)

    

                    .send()

    

                    .await

    

                    .map_err(|e| {

    

                        let error_msg = if e.is_connect() {

    

                            "无法连接到 Ollama 服务，请确保 Ollama 正在运行".to_string()

    

                        } else if e.is_timeout() {

    

                            "Ollama 请求超时".to_string()

    

                        } else {

    

                            e.to_string()

    

                        };

    

                        let _ = futures::executor::block_on(sender.send(StreamEvent::Error {

    

                            provider: self.name().to_string(),

    

                            error: error_msg.clone(),

    

                            request_id,

    

                        }));

    

                        AIError::RequestFailed(error_msg)

    

                    })?;

    

        

    

                let status = response.status();

    

                if !status.is_success() {

    

                    let resp_text = response.text().await.unwrap_or_default();

    

                    let error_msg = format!("HTTP {}: {}", status, resp_text);

    

                    let _ = sender.send(StreamEvent::Error {

    

                        provider: self.name().to_string(),

    

                        error: error_msg.clone(),

    

                        request_id,

    

                    }).await;

    

                    return Err(AIError::RequestFailed(error_msg));

    

                }

    

        

    

                // 处理流式响应

    

                let mut full_content = String::new();

    

                let mut stream = response.bytes_stream();

    

        

    

                while let Some(chunk_result) = stream.next().await {

    

                    match chunk_result {

    

                        Ok(bytes) => {

    

                            let chunk_str = String::from_utf8_lossy(&bytes);

    

                            // Ollama streaming returns newline-delimited JSON objects

    

                            for line in chunk_str.lines() {

    

                                if line.trim().is_empty() {

    

                                    continue;

    

                                }

    

                                if let Ok(json_val) = serde_json::from_str::<Value>(line) {

    

                                    if let Some(content_chunk) = json_val

    

                                        .get("message")

    

                                        .and_then(|m| m.get("content"))

    

                                        .and_then(|c| c.as_str())

    

                                    {

    

                                        full_content.push_str(content_chunk);

    

                                        let _ = sender.send(StreamEvent::Chunk {

    

                                            provider: self.name().to_string(),

    

                                            content: content_chunk.to_string(),

    

                                            request_id,

    

                                        }).await;

    

                                    }

    

                                }

    

                            }

    

                        }

    

                        Err(e) => {

    

                            let _ = sender.send(StreamEvent::Error {

    

                                provider: self.name().to_string(),

    

                                error: e.to_string(),

    

                                request_id,

    

                            }).await;

    

                            return Err(AIError::RequestFailed(e.to_string()));

    

                        }

    

                    }

    

                }

    

        

    

                // 发送完成事件

    

                let _ = sender.send(StreamEvent::Done {

    

                    provider: self.name().to_string(),

    

                    model: self.model.clone(),

    

                    detected_source_lang: request.source_lang.clone(),

    

                    target_lang: request.target_lang.clone(),

    

                    full_translation: full_content,

    

                    request_id,

    

                }).await;

    

        

    

                Ok(())

    

            }

    

        }

    

        

    
