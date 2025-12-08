pub struct ModelConfig {
    pub id: &'static str,
    pub free: bool,
}

/// 默认的智谱模型（强制使用 glm-4-flash 以保证零配置可用）
pub const DEFAULT_ZHIPU_MODEL: &str = "glm-4-flash";

/// 智谱 AI 配置
pub const ZHIPU_INTERNAL_KEY: Option<&'static str> = Some("2583dd1bdd274cde9c7daa189f0d5be7.pvG5Y6xmopLS2k76");

pub const ZHIPU_MODELS: &[ModelConfig] = &[
    ModelConfig { id: DEFAULT_ZHIPU_MODEL, free: true },
    ModelConfig { id: "glm-4-flashx", free: false },
    ModelConfig { id: "glm-4-flash-250414", free: true },
];

/// OpenAI 配置
pub const OPENAI_MODELS: &[ModelConfig] = &[
    ModelConfig { id: "gpt-4o-mini", free: false },
    ModelConfig { id: "gpt-4o", free: false },
    ModelConfig { id: "gpt-4-turbo", free: false },
    ModelConfig { id: "gpt-3.5-turbo", free: false },
];

/// Claude 配置
pub const CLAUDE_MODELS: &[ModelConfig] = &[
    ModelConfig { id: "claude-3-5-haiku-latest", free: false },
    ModelConfig { id: "claude-3-5-sonnet-latest", free: false },
    ModelConfig { id: "claude-3-opus-latest", free: false },
];
