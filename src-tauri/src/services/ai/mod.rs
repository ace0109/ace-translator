// AI 服务商模块
pub mod provider;
pub mod zhipu;
pub mod openai;
pub mod claude;
pub mod ollama;
pub mod key;

pub use provider::{AIProvider, AIError, TranslationRequest, TranslationResponse, ProviderConfig, ApiTestResponse, StreamEvent};
