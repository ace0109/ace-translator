// AI 服务商模块
pub mod claude;
pub mod ollama;
pub mod openai;
pub mod provider;
pub mod zhipu;

pub use provider::{
    AIError, AIProvider, ApiTestResponse, ProviderConfig, StreamEvent, TranslationRequest,
    TranslationResponse,
};
