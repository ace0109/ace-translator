use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::config::prompts;

const API_URL: &str = "https://open.bigmodel.cn/api/paas/v4/chat/completions";
const DEFAULT_MODEL: &str = "GLM-4.6";

#[derive(Debug, Serialize, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    content: String,
}

pub async fn call_zhipu_api(
    api_key: &str,
    text: &str,
    source_lang: &str,
    target_langs: &[String],
    request_id: Option<u64>,
) -> Result<Value, String> {
    println!(
        "[zhipu] request | req_id={:?} model={} source_lang={} target_langs={:?} text_len={}",
        request_id,
        DEFAULT_MODEL,
        source_lang,
        target_langs,
        text.len()
    );

    let client = Client::new();

    let system_prompt = prompts::SYSTEM_PROMPT;

    let targets_str = target_langs.join(", ");

    let user_prompt = format!(
        "源语言：{}\n目标语言列表：[{targets}]\n\n文本：{text}\n\n语言代码列表：{codes}\n输出要求：{output}\n仅返回 JSON，不要额外解释。",
        if source_lang == "auto" { "auto(请自行检测)" } else { source_lang },
        targets = targets_str,
        text = text,
        codes = prompts::LANGUAGE_CODES,
        output = prompts::OUTPUT_FORMAT,
    );

    let payload = json!({
        "model": DEFAULT_MODEL,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": user_prompt }
        ],
        "stream": false
    });

    let response = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    let resp_text = response.text().await.unwrap_or_default();
    println!("[zhipu] response status={} body_preview=\"{}\"", status, resp_text.chars().take(120).collect::<String>());

    if !status.is_success() {
        return Err(format!("API Error: {}", resp_text));
    }

    let chat_response: ChatResponse = serde_json::from_str(&resp_text)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(choice) = chat_response.choices.first() {
        let content = choice.message.content.clone();

        // 优先解析 JSON，若失败尝试从代码块中提取
        if let Some(val) = try_parse_json(&content) {
            return Ok(val);
        }

        // 兜底：返回原始字符串
        return Ok(json!({ "detected_source_lang": source_lang, "translations": {}, "raw": content }));
    }

    Err("No translation result returned".to_string())
}

fn try_parse_json(content: &str) -> Option<Value> {
    // 1) 直接尝试
    if let Ok(val) = serde_json::from_str::<Value>(content) {
        return Some(val);
    }

    // 2) 提取 ```json ... ``` 或 ``` ... ```
    let fence_variants = ["```json", "```"];
    for fence in fence_variants {
        if let Some(start) = content.find(fence) {
            let rest = &content[start + fence.len()..];
            if let Some(end) = rest.find("```") {
                let block = &rest[..end];
                if let Ok(val) = serde_json::from_str::<Value>(block) {
                    return Some(val);
                }
            }
        }
    }

    // 3) 去除反引号后再试
    let stripped = content.trim_matches('`').trim();
    if let Ok(val) = serde_json::from_str::<Value>(stripped) {
        return Some(val);
    }

    None
}
