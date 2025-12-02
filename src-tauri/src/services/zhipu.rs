use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

const API_URL: &str = "https://open.bigmodel.cn/api/paas/v4/chat/completions";
const DEFAULT_MODEL: &str = "glm-4-flash"; 

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
    target_lang: &str,
) -> Result<String, String> {
    println!(
        "[zhipu] request | model={} source_lang={} target_lang={} text_len={}",
        DEFAULT_MODEL,
        source_lang,
        target_lang,
        text.len()
    );

    let client = Client::new();

    let system_prompt = "你是一个专业的翻译助手。请直接将用户提供的文本翻译成目标语言，不要解释，不要添加任何额外内容。如果源语言未指定(auto)，请自动检测。";
    
    let user_prompt = format!(
        "请将以下文本从 {} 翻译成 {}:\n\n{}",
        if source_lang == "auto" { "自动检测语言" } else { source_lang },
        target_lang,
        text
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
        Ok(choice.message.content.clone())
    } else {
        Err("No translation result returned".to_string())
    }
}
