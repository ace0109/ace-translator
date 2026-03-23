use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ModelConfig {
    pub id: &'static str,
    pub free: bool,
    pub rate_limit: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProviderPreset {
    pub id: &'static str,
    pub display_name: &'static str,
    pub default_model: &'static str,
    pub default_base_url: Option<&'static str>,
    pub model_configs: &'static [ModelConfig],
    pub supports_base_url: bool,
    pub supports_custom_model: bool,
    pub api_key_optional: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SpeechProviderPreset {
    pub id: &'static str,
    pub display_name: &'static str,
    pub default_model: &'static str,
    pub default_base_url: Option<&'static str>,
    pub default_voice: &'static str,
    pub default_audio_format: &'static str,
    pub api_key_optional: bool,
}

/// 默认的智谱模型
pub const DEFAULT_ZHIPU_MODEL: &str = "GLM-4-Flash";
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-4.1-mini";
pub const DEFAULT_DEEPSEEK_MODEL: &str = "deepseek-chat";
pub const DEFAULT_XIAOMI_MODEL: &str = "mimo-v2-pro";
pub const DEFAULT_MINIMAX_MODEL: &str = "MiniMax-M2.7";
pub const DEFAULT_MOONSHOT_MODEL: &str = "kimi-k2.5";
pub const DEFAULT_OLLAMA_MODEL: &str = "llama3.2";
pub const DEFAULT_CLAUDE_MODEL: &str = "claude-3-5-haiku-latest";
pub const DEFAULT_XIAOMI_TTS_MODEL: &str = "mimo-v2-tts";
pub const DEFAULT_XIAOMI_TTS_VOICE: &str = "mimo_default";
pub const DEFAULT_TTS_AUDIO_FORMAT: &str = "wav";

/// 智谱 AI 配置
pub const ZHIPU_INTERNAL_KEY: Option<&'static str> =
    Some("ad075d78e20a42a3b6cb4d27153a3908.BuJ0NgD1eNUfxLyC");

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
        id: DEFAULT_OPENAI_MODEL,
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "gpt-4.1",
        free: false,
        rate_limit: 0,
    },
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
];

/// DeepSeek（OpenAI 协议）配置
pub const DEEPSEEK_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: DEFAULT_DEEPSEEK_MODEL,
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "deepseek-reasoner",
        free: false,
        rate_limit: 0,
    },
];

/// 小米 MiMo（OpenAI 协议）配置
pub const XIAOMI_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: DEFAULT_XIAOMI_MODEL,
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "mimo-v2-omni",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "mimo-v2-flash",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "mimo-v2-tts",
        free: false,
        rate_limit: 0,
    },
];

/// MiniMax（OpenAI 协议）配置
pub const MINIMAX_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: DEFAULT_MINIMAX_MODEL,
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "MiniMax-M2.7-highspeed",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "MiniMax-M2.5",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "MiniMax-M2.5-highspeed",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "MiniMax-M2.1",
        free: false,
        rate_limit: 0,
    },
];

/// Moonshot Kimi（OpenAI 协议）配置
pub const MOONSHOT_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: DEFAULT_MOONSHOT_MODEL,
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "kimi-k2-turbo-preview",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "kimi-k2-thinking",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "moonshot-v1-32k",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "moonshot-v1-128k",
        free: false,
        rate_limit: 0,
    },
];

/// Claude 配置
pub const CLAUDE_MODELS: &[ModelConfig] = &[
    ModelConfig {
        id: DEFAULT_CLAUDE_MODEL,
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

/// Ollama 常用模型（可自定义输入其他模型）
pub const OLLAMA_MODEL_CONFIGS: &[ModelConfig] = &[
    ModelConfig {
        id: DEFAULT_OLLAMA_MODEL,
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "llama3.1",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "qwen2.5",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "mistral",
        free: false,
        rate_limit: 0,
    },
    ModelConfig {
        id: "gemma2",
        free: false,
        rate_limit: 0,
    },
];

/// 预置服务商（预置名称/模型/地址，API Key 由用户自行填写）
pub const PROVIDER_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "zhipu",
        display_name: "智谱AI",
        default_model: DEFAULT_ZHIPU_MODEL,
        default_base_url: Some("https://open.bigmodel.cn/api/paas/v4/chat/completions"),
        model_configs: ZHIPU_MODELS,
        supports_base_url: true,
        supports_custom_model: true,
        api_key_optional: true,
    },
    ProviderPreset {
        id: "openai",
        display_name: "OpenAI",
        default_model: DEFAULT_OPENAI_MODEL,
        default_base_url: Some("https://api.openai.com/v1/chat/completions"),
        model_configs: OPENAI_MODELS,
        supports_base_url: true,
        supports_custom_model: true,
        api_key_optional: false,
    },
    ProviderPreset {
        id: "deepseek",
        display_name: "DeepSeek",
        default_model: DEFAULT_DEEPSEEK_MODEL,
        default_base_url: Some("https://api.deepseek.com/chat/completions"),
        model_configs: DEEPSEEK_MODELS,
        supports_base_url: true,
        supports_custom_model: true,
        api_key_optional: false,
    },
    ProviderPreset {
        id: "xiaomi",
        display_name: "小米 MiMo",
        default_model: DEFAULT_XIAOMI_MODEL,
        default_base_url: Some("https://api.xiaomimimo.com/v1/chat/completions"),
        model_configs: XIAOMI_MODELS,
        supports_base_url: true,
        supports_custom_model: true,
        api_key_optional: false,
    },
    ProviderPreset {
        id: "minimax",
        display_name: "MiniMax",
        default_model: DEFAULT_MINIMAX_MODEL,
        default_base_url: Some("https://api.minimax.io/v1/chat/completions"),
        model_configs: MINIMAX_MODELS,
        supports_base_url: true,
        supports_custom_model: true,
        api_key_optional: false,
    },
    ProviderPreset {
        id: "moonshot",
        display_name: "Moonshot Kimi",
        default_model: DEFAULT_MOONSHOT_MODEL,
        default_base_url: Some("https://api.moonshot.ai/v1/chat/completions"),
        model_configs: MOONSHOT_MODELS,
        supports_base_url: true,
        supports_custom_model: true,
        api_key_optional: false,
    },
    ProviderPreset {
        id: "ollama",
        display_name: "Ollama",
        default_model: DEFAULT_OLLAMA_MODEL,
        default_base_url: Some("http://localhost:11434/api/chat"),
        model_configs: OLLAMA_MODEL_CONFIGS,
        supports_base_url: true,
        supports_custom_model: true,
        api_key_optional: true,
    },
];

/// 预置语音服务商（与翻译服务商解耦）
pub const SPEECH_PROVIDER_PRESETS: &[SpeechProviderPreset] = &[SpeechProviderPreset {
    id: "xiaomi",
    display_name: "小米 MiMo TTS",
    default_model: DEFAULT_XIAOMI_TTS_MODEL,
    default_base_url: Some("https://api.xiaomimimo.com/v1/chat/completions"),
    default_voice: DEFAULT_XIAOMI_TTS_VOICE,
    default_audio_format: DEFAULT_TTS_AUDIO_FORMAT,
    api_key_optional: false,
}];

pub fn preset_by_id(provider_name: &str) -> Option<&'static ProviderPreset> {
    PROVIDER_PRESETS
        .iter()
        .find(|preset| preset.id == provider_name)
}

pub fn preset_order(provider_name: &str) -> Option<usize> {
    PROVIDER_PRESETS
        .iter()
        .position(|preset| preset.id == provider_name)
}

pub fn speech_preset_by_id(provider_name: &str) -> Option<&'static SpeechProviderPreset> {
    SPEECH_PROVIDER_PRESETS
        .iter()
        .find(|preset| preset.id == provider_name)
}

pub fn speech_preset_order(provider_name: &str) -> Option<usize> {
    SPEECH_PROVIDER_PRESETS
        .iter()
        .position(|preset| preset.id == provider_name)
}

pub fn is_preset_provider(provider_name: &str) -> bool {
    preset_by_id(provider_name).is_some()
}

pub fn default_model_for_provider(provider_name: &str) -> Option<&'static str> {
    if provider_name == "claude" {
        return Some(DEFAULT_CLAUDE_MODEL);
    }
    preset_by_id(provider_name).map(|preset| preset.default_model)
}

pub fn default_base_url_for_provider(provider_name: &str) -> Option<&'static str> {
    preset_by_id(provider_name).and_then(|preset| preset.default_base_url)
}

pub fn speech_default_model_for_provider(provider_name: &str) -> Option<&'static str> {
    speech_preset_by_id(provider_name).map(|preset| preset.default_model)
}

pub fn speech_default_base_url_for_provider(provider_name: &str) -> Option<&'static str> {
    speech_preset_by_id(provider_name).and_then(|preset| preset.default_base_url)
}

pub fn speech_default_voice_for_provider(provider_name: &str) -> Option<&'static str> {
    speech_preset_by_id(provider_name).map(|preset| preset.default_voice)
}

pub fn speech_default_audio_format_for_provider(provider_name: &str) -> Option<&'static str> {
    speech_preset_by_id(provider_name).map(|preset| preset.default_audio_format)
}
