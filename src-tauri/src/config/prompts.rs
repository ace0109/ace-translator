use serde_json::Value;

// AI 提示词配置，便于集中管理和随时调整
// SYSTEM_PROMPT：模型角色/行为约束
pub const SYSTEM_PROMPT: &str = r#"
You are a production translation engine.
Requirements:
1. Return ONLY the translated text.
2. Do not include explanations, notes, language labels, or confidence statements.
3. Do not output Markdown code fences or JSON.
4. Keep placeholders and non-translatable tokens unchanged, including patterns like {name}, {{name}}, %s, %d, URLs, emails, and code snippets.
5. Preserve original line breaks, punctuation, and formatting.
"#;

// 语言代码列表（需与前端 languageOptions 同步）
#[allow(dead_code)]
pub const LANGUAGE_CODES: &str = r#"["zh-CN","zh-TW","en","ja","ko","fr","de","es","ru","ar","pt","it","nl","sv","no","da","fi","pl","cs","hu","ro"]"#;

pub fn build_translation_user_prompt(source_lang: &str, target_lang: &str, text: &str) -> String {
    format!(
        r#"Translate the text inside <text></text> from "{source}" to "{target}".
Rules:
- Output translated text only.
- Do not add explanations or metadata.
- Keep placeholders/tokens unchanged.

<text>
{text}
</text>"#,
        source = source_lang,
        target = target_lang,
        text = text
    )
}

#[allow(dead_code)]
pub fn build_language_detection_prompt(text: &str) -> String {
    format!(
        r#"Detect the language of the text inside <text></text>.
Return ONLY one language code from this exact list: {codes}
Rules:
- For Simplified Chinese return zh-CN
- For Traditional Chinese return zh-TW
- For all other languages return the best-matching code from the list
- Output the code only

<text>
{text}
</text>"#,
        codes = LANGUAGE_CODES,
        text = text
    )
}

pub fn normalize_translation_output(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let unfenced = strip_markdown_fences(trimmed);
    if let Some(from_json) = extract_translation_field(unfenced) {
        return from_json.trim().to_string();
    }

    unfenced.trim().to_string()
}

fn strip_markdown_fences(raw: &str) -> &str {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") || !trimmed.ends_with("```") {
        return trimmed;
    }

    if let Some(first_newline) = trimmed.find('\n') {
        let inner = &trimmed[first_newline + 1..];
        if let Some(last_fence) = inner.rfind("```") {
            return inner[..last_fence].trim();
        }
    }

    trimmed.trim_matches('`').trim()
}

fn extract_translation_field(raw: &str) -> Option<String> {
    let val: Value = serde_json::from_str(raw).ok()?;
    match val {
        Value::String(s) => Some(s),
        Value::Object(map) => {
            for key in [
                "translation",
                "translated_text",
                "translatedText",
                "text",
                "result",
                "content",
                "output",
            ] {
                if let Some(v) = map.get(key).and_then(|x| x.as_str()) {
                    return Some(v.to_string());
                }
            }
            None
        }
        _ => None,
    }
}
