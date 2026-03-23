pub const EMPTY_TEXT: &str = "EMPTY_TEXT";
pub const NO_ACTIVE_PROVIDER: &str = "NO_ACTIVE_PROVIDER";
pub const INVALID_PROVIDER_CONFIG: &str = "INVALID_PROVIDER_CONFIG";
#[allow(dead_code)]
pub const LANGUAGE_DETECTION_FAILED: &str = "LANGUAGE_DETECTION_FAILED";
pub const TRANSLATION_FAILED: &str = "TRANSLATION_FAILED";
pub const TRANSLATION_CANCELLED: &str = "TRANSLATION_CANCELLED";
pub const NO_ACTIVE_SPEECH_PROVIDER: &str = "NO_ACTIVE_SPEECH_PROVIDER";
pub const INVALID_SPEECH_PROVIDER_CONFIG: &str = "INVALID_SPEECH_PROVIDER_CONFIG";
pub const SPEECH_SYNTHESIS_FAILED: &str = "SPEECH_SYNTHESIS_FAILED";
pub const PROVIDER_NAME_EMPTY: &str = "PROVIDER_NAME_EMPTY";
pub const PROVIDER_ALREADY_EXISTS: &str = "PROVIDER_ALREADY_EXISTS";
pub const PROVIDER_NOT_FOUND: &str = "PROVIDER_NOT_FOUND";
pub const PRESET_PROVIDER_IMMUTABLE: &str = "PRESET_PROVIDER_IMMUTABLE";
pub const DB_OPERATION_FAILED: &str = "DB_OPERATION_FAILED";

pub fn with_code(code: &str, message: impl AsRef<str>) -> String {
    format!("[{}] {}", code, message.as_ref())
}

pub fn ensure_code(message: impl Into<String>, fallback_code: &str) -> String {
    let msg = message.into();
    if has_code_prefix(&msg) {
        msg
    } else {
        with_code(fallback_code, msg)
    }
}

fn has_code_prefix(message: &str) -> bool {
    let bytes = message.as_bytes();
    if bytes.len() < 4 || bytes.first() != Some(&b'[') {
        return false;
    }
    let Some(end) = message.find(']') else {
        return false;
    };
    if end <= 1 {
        return false;
    }
    message[1..end]
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_code_adds_code_when_missing() {
        let got = ensure_code("plain error", INVALID_PROVIDER_CONFIG);
        assert_eq!(got, "[INVALID_PROVIDER_CONFIG] plain error");
    }

    #[test]
    fn ensure_code_keeps_existing_code() {
        let got = ensure_code(
            "[NO_ACTIVE_PROVIDER] 没有已启用服务商",
            INVALID_PROVIDER_CONFIG,
        );
        assert_eq!(got, "[NO_ACTIVE_PROVIDER] 没有已启用服务商");
    }
}
