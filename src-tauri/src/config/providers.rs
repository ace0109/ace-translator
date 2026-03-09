use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModelConfig {
    pub id: &'static str,
    pub free: bool,
    pub rate_limit: u32,
}

/// 默认的智谱模型（强制使用 GLM-4-Flash 以保证零配置可用）
pub const DEFAULT_ZHIPU_MODEL: &str = "GLM-4-Flash";

pub const ZHIPU_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: DEFAULT_ZHIPU_MODEL,
        free: true,
        rate_limit: 200,
    },
    ModelConfig {
        id: "GLM-4-Flash-250414",
        free: true,
        rate_limit: 200,
    },
    ModelConfig {
        id: "GLM-4.7-Flash",
        free: true,
        rate_limit: 1,
    },
    ModelConfig {
        id: "GLM-4-FlashX",
        free: false,
        rate_limit: 50,
    },
    ModelConfig {
        id: "GLM-4-FlashX-250414",
        free: false,
        rate_limit: 100,
    },
    ModelConfig {
        id: "GLM-4.7-FlashX",
        free: false,
        rate_limit: 3,
    },
];

/// OpenAI 配置
pub const OPENAI_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: "gpt-4o-mini",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "gpt-4o",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "gpt-4-turbo",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "gpt-3.5-turbo",
        free: false,
        rate_limit: 0,
    },
];

/// Claude 配置
pub const CLAUDE_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: "claude-3-5-haiku-latest",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "claude-3-5-sonnet-latest",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "claude-3-opus-latest",
        free: false,
        rate_limit: 0,
    },
];
