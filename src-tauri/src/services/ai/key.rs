use crate::config::providers::ZHIPU_INTERNAL_KEY;
use crate::{app_debug, app_error};

static DEFAULT_ZHIPU_KEY: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

/// 按优先级获取默认的智谱 API Key：
/// 1. 环境变量 `ZHIPU_API_KEY`
/// 2. 内置 Key（ZHIPU_INTERNAL_KEY，开源分支默认 None）
pub async fn resolve_default_zhipu_api_key() -> String {
    if let Some(cached) = DEFAULT_ZHIPU_KEY.get() {
        return cached.clone();
    }

    if let Ok(from_env) = std::env::var("ZHIPU_API_KEY") {
        let trimmed = from_env.trim();
        if !trimmed.is_empty() {
            app_debug!("使用环境变量 ZHIPU_API_KEY 作为默认智谱 Key");
            let key = trimmed.to_string();
            DEFAULT_ZHIPU_KEY
                .get_or_init(|| async { key.clone() })
                .await;
            return key;
        }
    }

    let fallback = ZHIPU_INTERNAL_KEY.unwrap_or("").to_string();
    if fallback.is_empty() {
        app_error!("未能获取到任何智谱 API Key（环境变量/内置均为空），后续请求将失败");
    } else {
        app_debug!("使用内置兜底的智谱 API Key");
    }

    DEFAULT_ZHIPU_KEY
        .get_or_init(|| async { fallback.clone() })
        .await
        .clone()
}
