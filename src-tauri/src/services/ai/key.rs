use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::config::providers::ZHIPU_INTERNAL_KEY;
use crate::{app_debug, app_error};

const ZHIPU_KEY_ENDPOINT: &str = "https://nest.wangcaiyuan.com/ai/translator/api-key";
static DEFAULT_ZHIPU_KEY: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

#[derive(Deserialize)]
struct ApiKeyResponse {
    code: String,
    data: Option<ApiKeyData>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct ApiKeyData {
    #[serde(rename = "apiKey")]
    api_key: String,
}

/// 按优先级获取默认的智谱 API Key：
/// 1. 远程接口（成功且 code == 00000）
/// 2. 内置 Key（ZHIPU_INTERNAL_KEY）
pub async fn resolve_default_zhipu_api_key() -> String {
    if let Some(cached) = DEFAULT_ZHIPU_KEY.get() {
        return cached.clone();
    }

    // 远程拉取
    let client = Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .unwrap_or_default();

    match client.get(ZHIPU_KEY_ENDPOINT).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<ApiKeyResponse>().await {
            Ok(parsed) if parsed.code == "00000" => {
                if let Some(data) = parsed.data {
                    if !data.api_key.is_empty() {
                        app_debug!("使用远程下发的智谱 API Key");
                        let key = data.api_key;
                        DEFAULT_ZHIPU_KEY
                            .get_or_init(|| async { key.clone() })
                            .await;
                        return key;
                    }
                }
                app_error!(
                    "远程智谱 API Key 响应无数据或为空 (code={}, msg={:?})",
                    parsed.code,
                    parsed.message
                );
            }
            Ok(parsed) => {
                app_error!(
                    "远程智谱 API Key 响应 code 非预期: {} msg={:?}",
                    parsed.code,
                    parsed.message
                );
            }
            Err(e) => {
                app_error!("解析远程智谱 API Key 失败: {}", e);
            }
        },
        Ok(resp) => {
            app_error!("远程智谱 API Key 请求失败，HTTP {}", resp.status());
        }
        Err(e) => {
            app_error!("远程智谱 API Key 请求异常: {}", e);
        }
    }

    // 兜底使用内置 Key
    let fallback = ZHIPU_INTERNAL_KEY.unwrap_or("").to_string();
    if fallback.is_empty() {
        app_error!("未能获取到任何智谱 API Key（远程/内置均为空），后续请求将失败");
    } else {
        app_debug!("使用内置兜底的智谱 API Key");
    }
    DEFAULT_ZHIPU_KEY
        .get_or_init(|| async { fallback.clone() })
        .await
        .clone()
}
