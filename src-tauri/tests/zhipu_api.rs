use ace_translator_lib::services::zhipu;
use serde_json::Value;

// 环境变量：ZHIPU_API_KEY，用于手动验证真实接口。未设置则跳过。
#[tokio::test]
async fn test_zhipu_multi_translation() {
    let api_key = match std::env::var("ZHIPU_API_KEY") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            println!("ZHIPU_API_KEY not set, skip real API test");
            return;
        }
    };

    let text = "Hello, world!";
    let targets = vec!["zh-CN".to_string(), "ja".to_string()];

    let res: Value = zhipu::call_zhipu_api(&api_key, text, "auto", &targets)
        .await
        .expect("API call failed");

    println!("API response: {res}");

    let detected = res
        .get("detected_source_lang")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let translations = res
        .get("translations")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    assert!(!detected.is_empty(), "detected_source_lang missing");
    for t in &targets {
        assert!(
            translations.contains_key(t),
            "translations missing target lang {t}"
        );
    }
}
