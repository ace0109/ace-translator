// AI 提示词配置，便于集中管理和随时调整
// SYSTEM_PROMPT：模型角色/行为约束

pub const SYSTEM_PROMPT: &str = r#"
You are a professional translation engine.
Requirements:
1. Output ONLY the translated text.
2. Do not include original text, explanations, notes, or definitions.
3. Do not use Markdown or code blocks.
4. Do not output JSON.
5. Preserve original formatting and punctuation.
"#;

// 语言代码列表（需与前端 languageOptions 同步）
#[allow(dead_code)]
pub const LANGUAGE_CODES: &str = r#"["zh-CN","zh-TW","en","ja","ko","fr","de","es","ru","ar","pt","it","nl","sv","no","da","fi","pl","cs","hu","ro"]"#;
