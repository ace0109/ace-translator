// AI Provider 接口定义
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::mpsc;

/// AI 服务商错误类型
#[derive(Debug)]
pub enum AIError {
    /// API 请求失败
    RequestFailed(String),
    /// API 响应解析失败
    ParseError(String),
    /// API Key 无效或未设置
    InvalidApiKey(String),
    /// 服务不可用
    ServiceUnavailable(String),
    /// 超时
    Timeout(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for AIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AIError::RequestFailed(msg) => write!(f, "请求失败: {}", msg),
            AIError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            AIError::InvalidApiKey(msg) => write!(f, "API Key 无效: {}", msg),
            AIError::ServiceUnavailable(msg) => write!(f, "服务不可用: {}", msg),
            AIError::Timeout(msg) => write!(f, "请求超时: {}", msg),
            AIError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for AIError {}

/// 流式翻译事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StreamEvent {
    /// 翻译开始
    Start {
        provider: String,
        model: String,
        request_id: u64,
    },
    /// 收到翻译片段
    Chunk {
        provider: String,
        content: String,
        request_id: u64,
    },
    /// 翻译完成
    Done {
        provider: String,
        model: String,
        detected_source_lang: String,
        target_lang: String,
        full_translation: String,
        request_id: u64,
    },
    /// 翻译出错
    Error {
        provider: String,
        error: String,
        request_id: u64,
    },
}

/// 流式发送器类型
pub type StreamSender = mpsc::Sender<StreamEvent>;

/// 翻译请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// 待翻译文本
    pub text: String,
    /// 源语言
    pub source_lang: String,
    /// 目标语言
    pub target_lang: String,
}

/// 翻译响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    /// 翻译结果
    pub translation: String,
    /// 服务商名称
    pub provider: String,
    /// 使用的模型
    pub model: String,
}

/// 服务商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// 服务商名称（zhipu/openai/claude/ollama）
    #[serde(default)]
    pub provider_name: String,
    /// 是否启用
    pub enabled: bool,
    /// API Key
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 自定义 API 地址（可选）
    pub base_url: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_name: String::new(),
            enabled: false,
            api_key: String::new(),
            model: String::new(),
            base_url: None,
        }
    }
}

/// API 测试响应（返回原始元数据，用于调试和验证）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestResponse {
    /// 是否成功
    pub success: bool,
    /// HTTP 状态码
    pub status_code: u16,
    /// 响应时间（毫秒）
    pub response_time_ms: u64,
    /// 原始响应体（JSON 格式）
    pub raw_response: Option<serde_json::Value>,
    /// 请求参数（JSON 格式）
    pub request_payload: Option<serde_json::Value>,
    /// 错误信息（如果失败）
    pub error: Option<String>,
    /// 服务商名称
    pub provider: String,
    /// 使用的模型
    pub model: String,
}

/// AI 服务商接口
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// 获取服务商名称
    fn name(&self) -> &'static str;

    /// 获取支持的模型列表
    #[allow(dead_code)]
    fn available_models(&self) -> Vec<&'static str>;

    /// 获取当前使用的模型
    fn current_model(&self) -> &str;

    /// 检测语种
    async fn detect_language(&self, text: &str) -> Result<String, AIError>;

    /// 执行翻译（非流式）
    async fn translate(&self, request: &TranslationRequest)
        -> Result<TranslationResponse, AIError>;

    /// 执行流式翻译
    /// 默认实现：调用非流式翻译，然后发送完整结果
    async fn translate_stream(
        &self,
        request: &TranslationRequest,
        sender: StreamSender,
        request_id: u64,
    ) -> Result<(), AIError> {
        // 发送开始事件
        let _ = sender
            .send(StreamEvent::Start {
                provider: self.name().to_string(),
                model: self.current_model().to_string(),
                request_id,
            })
            .await;

        // 调用非流式翻译
        match self.translate(request).await {
            Ok(response) => {
                // 发送完成事件（包含完整翻译）
                let _ = sender
                    .send(StreamEvent::Done {
                        provider: response.provider.clone(),
                        model: response.model.clone(),
                        detected_source_lang: request.source_lang.clone(),
                        target_lang: request.target_lang.clone(),
                        full_translation: response.translation.clone(),
                        request_id,
                    })
                    .await;
                Ok(())
            }
            Err(e) => {
                // 发送错误事件
                let _ = sender
                    .send(StreamEvent::Error {
                        provider: self.name().to_string(),
                        error: e.to_string(),
                        request_id,
                    })
                    .await;
                Err(e)
            }
        }
    }

    /// 测试 API 连接（返回原始响应元数据）
    async fn test_api(&self) -> ApiTestResponse;

    /// 测试连接（简单返回 bool）
    #[allow(dead_code)]
    async fn test_connection(&self) -> Result<bool, AIError> {
        let result = self.test_api().await;
        if result.success {
            Ok(true)
        } else {
            Err(AIError::RequestFailed(
                result.error.unwrap_or_else(|| "未知错误".to_string()),
            ))
        }
    }
}
