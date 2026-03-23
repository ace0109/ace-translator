// AI 服务商模块
pub mod claude;
pub mod key;
pub mod ollama;
pub mod openai;
pub mod provider;
pub mod zhipu;

pub use provider::{
    AIProvider, ApiTestResponse, ProviderConfig, StreamEvent, TranslationRequest,
    TranslationResponse,
};
