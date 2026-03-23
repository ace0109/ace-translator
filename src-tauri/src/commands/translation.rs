use crate::commands::error_codes;
use crate::config::providers;
use crate::services::ai::{
    claude::ClaudeProvider, key::resolve_default_zhipu_api_key, ollama::OllamaProvider,
    openai::OpenAIProvider, zhipu::ZhipuProvider, AIProvider, ProviderConfig, StreamEvent,
    TranslationRequest, TranslationResponse,
};
use crate::services::encryption::decrypt_api_key;
use crate::services::speech::{self, SpeechProviderConfig};
use crate::AppState;
use futures::future::{AbortHandle, Abortable, Aborted};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc;
use whatlang::detect as detect_language_with_whatlang;

/// 单个服务商的翻译结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderTranslationResult {
    pub provider: String,
    pub model: String,
    pub detected_source_lang: String,
    pub target_lang: String,
    pub translation: String,
    pub success: bool,
    pub error: Option<String>,
}

/// 多服务商翻译结果
#[derive(Debug, Serialize, Deserialize)]
pub struct MultiProviderResult {
    pub results: Vec<ProviderTranslationResult>,
}

/// 旧版翻译结果结构（兼容）
#[derive(Debug, Serialize, Deserialize)]
pub struct TranslationResult {
    pub detected_source_lang: String,
    pub target_lang: String,
    pub translation: String,
}

/// 语音合成结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpeechSynthesisResult {
    pub provider: String,
    pub model: String,
    pub audio_format: String,
    pub audio_base64: String,
}

/// 取消标记：仅用于停止事件推送，不会真正中断上游 HTTP 请求。
fn mark_cancelled_request(state: &State<'_, AppState>, req_id: u64) {
    if let Ok(mut cancelled) = state.cancelled_requests.lock() {
        cancelled.insert(req_id);
    }
}

fn clear_cancelled_request(state: &State<'_, AppState>, req_id: u64) {
    if let Ok(mut cancelled) = state.cancelled_requests.lock() {
        cancelled.remove(&req_id);
    }
}

fn is_request_cancelled(app: &AppHandle, req_id: u64) -> bool {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(cancelled) = state.cancelled_requests.lock() {
            return cancelled.contains(&req_id);
        }
    }
    false
}

fn normalize_model(provider_name: &str, model: &str) -> String {
    let trimmed = model.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    providers::default_model_for_provider(provider_name)
        .unwrap_or("")
        .to_string()
}

fn normalize_base_url(provider_name: &str, base_url: Option<String>) -> Option<String> {
    if let Some(base) = base_url {
        let trimmed = base.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    providers::default_base_url_for_provider(provider_name).map(|v| v.to_string())
}

fn now_request_id(request_id: Option<u64>) -> u64 {
    request_id.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    })
}

fn log_translation_metric(
    mode: &str,
    request_id: u64,
    provider: &str,
    model: &str,
    status: &str,
    duration_ms: u128,
    detail: Option<&str>,
) {
    if let Some(message) = detail {
        crate::app_info!(
            "[TranslationMetric] mode={} request_id={} provider={} model={} status={} duration_ms={} detail={}",
            mode,
            request_id,
            provider,
            model,
            status,
            duration_ms,
            message,
        );
    } else {
        crate::app_info!(
            "[TranslationMetric] mode={} request_id={} provider={} model={} status={} duration_ms={}",
            mode,
            request_id,
            provider,
            model,
            status,
            duration_ms,
        );
    }
}

#[derive(sqlx::FromRow)]
struct DbProviderConfig {
    provider_name: String,
    enabled: i32,
    api_key: String,
    model: String,
    base_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DbSpeechProviderConfig {
    provider_name: String,
    enabled: i32,
    api_key: String,
    model: String,
    base_url: Option<String>,
    voice: String,
    audio_format: String,
}

async fn decode_provider_row(row: DbProviderConfig) -> ProviderConfig {
    let provider_name = row.provider_name;
    let api_key = if !row.api_key.is_empty() {
        decrypt_api_key(&row.api_key).unwrap_or_default()
    } else if provider_name == "zhipu" {
        resolve_default_zhipu_api_key().await
    } else {
        String::new()
    };

    ProviderConfig {
        provider_name: provider_name.clone(),
        enabled: row.enabled != 0,
        api_key,
        model: normalize_model(&provider_name, &row.model),
        base_url: normalize_base_url(&provider_name, row.base_url),
    }
}

fn normalize_speech_model(provider_name: &str, model: &str) -> String {
    let trimmed = model.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    providers::speech_default_model_for_provider(provider_name)
        .unwrap_or("")
        .to_string()
}

fn normalize_speech_base_url(provider_name: &str, base_url: Option<String>) -> Option<String> {
    if let Some(base) = base_url {
        let trimmed = base.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    providers::speech_default_base_url_for_provider(provider_name).map(|v| v.to_string())
}

fn normalize_speech_voice(provider_name: &str, voice: &str) -> String {
    let trimmed = voice.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    providers::speech_default_voice_for_provider(provider_name)
        .unwrap_or(providers::DEFAULT_XIAOMI_TTS_VOICE)
        .to_string()
}

fn normalize_speech_audio_format(provider_name: &str, audio_format: &str) -> String {
    let trimmed = audio_format.trim();
    if !trimmed.is_empty() {
        return trimmed.to_lowercase();
    }
    providers::speech_default_audio_format_for_provider(provider_name)
        .unwrap_or(providers::DEFAULT_TTS_AUDIO_FORMAT)
        .to_string()
}

async fn decode_speech_provider_row(row: DbSpeechProviderConfig) -> SpeechProviderConfig {
    let provider_name = row.provider_name;
    let api_key = if !row.api_key.is_empty() {
        decrypt_api_key(&row.api_key).unwrap_or_default()
    } else {
        String::new()
    };

    SpeechProviderConfig {
        provider_name: provider_name.clone(),
        enabled: row.enabled != 0,
        api_key,
        model: normalize_speech_model(&provider_name, &row.model),
        base_url: normalize_speech_base_url(&provider_name, row.base_url),
        voice: normalize_speech_voice(&provider_name, &row.voice),
        audio_format: normalize_speech_audio_format(&provider_name, &row.audio_format),
    }
}

/// 获取当前唯一启用的服务商配置（单服务商架构）
async fn get_active_provider(db: &sqlx::SqlitePool) -> Result<ProviderConfig, String> {
    let row: Option<DbProviderConfig> = sqlx::query_as(
        r#"
        SELECT provider_name, enabled, api_key, model, base_url
        FROM provider_configs
        WHERE enabled = 1
        ORDER BY datetime(updated_at) DESC, provider_name ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(db)
    .await
    .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    let Some(row) = row else {
        return Err(error_codes::with_code(
            error_codes::NO_ACTIVE_PROVIDER,
            "没有已启用的服务商。请在设置中配置并启用一个服务商。",
        ));
    };

    Ok(decode_provider_row(row).await)
}

/// 获取当前唯一启用的语音服务商配置（单服务商架构）
async fn get_active_speech_provider(db: &sqlx::SqlitePool) -> Result<SpeechProviderConfig, String> {
    let row: Option<DbSpeechProviderConfig> = sqlx::query_as(
        r#"
        SELECT provider_name, enabled, api_key, model, base_url, voice, audio_format
        FROM speech_provider_configs
        WHERE enabled = 1
        ORDER BY datetime(updated_at) DESC, provider_name ASC
        LIMIT 1
        "#,
    )
    .fetch_optional(db)
    .await
    .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    let Some(row) = row else {
        return Err(error_codes::with_code(
            error_codes::NO_ACTIVE_SPEECH_PROVIDER,
            "没有已启用的语音服务商。请在设置中配置并启用一个语音服务商。",
        ));
    };

    Ok(decode_speech_provider_row(row).await)
}

fn provider_can_translate(config: &ProviderConfig) -> bool {
    match config.provider_name.as_str() {
        "zhipu" => !config.api_key.trim().is_empty(),
        "ollama" => true,
        _ => !config.api_key.trim().is_empty(),
    }
}

fn normalize_whatlang_code_to_app_lang(code: &str) -> Option<&'static str> {
    match code {
        "cmn" | "zho" => Some("zh-CN"),
        "eng" => Some("en"),
        "jpn" => Some("ja"),
        "kor" => Some("ko"),
        "fra" | "fre" => Some("fr"),
        "deu" | "ger" => Some("de"),
        "spa" => Some("es"),
        "rus" => Some("ru"),
        "ara" => Some("ar"),
        "por" => Some("pt"),
        "ita" => Some("it"),
        "nld" | "dut" => Some("nl"),
        "swe" => Some("sv"),
        "nor" => Some("no"),
        "dan" => Some("da"),
        "fin" => Some("fi"),
        "pol" => Some("pl"),
        "ces" | "cze" => Some("cs"),
        "hun" => Some("hu"),
        "ron" | "rum" => Some("ro"),
        "ukr" => Some("uk"),
        "ell" | "gre" => Some("el"),
        "heb" => Some("he"),
        "hin" => Some("hi"),
        "tha" => Some("th"),
        "vie" => Some("vi"),
        "ind" => Some("id"),
        "msa" | "may" => Some("ms"),
        "tur" => Some("tr"),
        _ => None,
    }
}

fn detect_language_locally(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return "en".to_string();
    }

    if let Some(info) = detect_language_with_whatlang(trimmed) {
        let code = info.lang().code();
        if let Some(mapped) = normalize_whatlang_code_to_app_lang(code) {
            return mapped.to_string();
        }
    }

    // Final fallback for short/ambiguous texts where local detector is uncertain.
    if trimmed
        .chars()
        .any(|ch| (0x4E00..=0x9FFF).contains(&(ch as u32)))
    {
        return "zh-CN".to_string();
    }

    "en".to_string()
}

// Helper function: locally detect language and determine target language.
fn detect_and_plan(text: &str, primary_target: &str, secondary_target: &str) -> (String, String) {
    let detected_lang = detect_language_locally(text);

    // Simple normalization of detected_lang (take the first two characters, convert to lowercase)
    let normalized_detected = detected_lang.to_lowercase();
    let is_primary = normalized_detected
        .starts_with(&primary_target.to_lowercase()[..2.min(primary_target.len())]);

    let target_lang = if is_primary {
        secondary_target.to_string()
    } else {
        primary_target.to_string()
    };

    (detected_lang, target_lang)
}

/// 使用单个服务商进行翻译
async fn translate_with_provider(
    config: &ProviderConfig,
    request: &TranslationRequest,
) -> ProviderTranslationResult {
    let result: Result<TranslationResponse, String> = match config.provider_name.as_str() {
        "zhipu" => {
            let provider = ZhipuProvider::new(config);
            provider.translate(request).await.map_err(|e| e.to_string())
        }
        "ollama" => {
            let provider = OllamaProvider::new(config);
            provider.translate(request).await.map_err(|e| e.to_string())
        }
        "claude" => {
            let provider = ClaudeProvider::new(config);
            provider.translate(request).await.map_err(|e| e.to_string())
        }
        _ => {
            let provider = OpenAIProvider::new(config);
            provider.translate(request).await.map_err(|e| e.to_string())
        }
    };

    match result {
        Ok(resp) => ProviderTranslationResult {
            provider: config.provider_name.clone(),
            model: if resp.model.trim().is_empty() {
                config.model.clone()
            } else {
                resp.model
            },
            detected_source_lang: request.source_lang.clone(),
            target_lang: request.target_lang.clone(),
            translation: resp.translation,
            success: true,
            error: None,
        },
        Err(e) => ProviderTranslationResult {
            provider: config.provider_name.clone(),
            model: config.model.clone(),
            detected_source_lang: request.source_lang.clone(),
            target_lang: request.target_lang.clone(),
            translation: String::new(),
            success: false,
            error: Some(error_codes::ensure_code(e, error_codes::TRANSLATION_FAILED)),
        },
    }
}

/// 单服务商翻译（主命令）
#[tauri::command]
pub async fn translate_active_provider(
    _app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: Option<u64>,
) -> Result<ProviderTranslationResult, String> {
    if text.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::EMPTY_TEXT,
            "输入文本不能为空",
        ));
    }

    let req_id = now_request_id(requestId);
    clear_cancelled_request(&state, req_id);

    let started_at = Instant::now();
    let active_provider = get_active_provider(&state.db).await?;
    if !provider_can_translate(&active_provider) {
        return Err(error_codes::with_code(
            error_codes::INVALID_PROVIDER_CONFIG,
            "当前启用服务商缺少可用 API Key，无法进行翻译",
        ));
    }

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = true;
    }

    let (detected_lang, target_lang) = detect_and_plan(&text, &primaryTarget, &secondaryTarget);

    let request = TranslationRequest {
        text: text.clone(),
        source_lang: detected_lang.clone(),
        target_lang: target_lang.clone(),
    };

    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let active_provider_name = active_provider.provider_name.clone();
    let active_provider_model = active_provider.model.clone();

    {
        let mut handles_map = HashMap::new();
        handles_map.insert(active_provider_name.clone(), abort_handle);
        if let Ok(mut handles) = state.main_abort_handles.lock() {
            handles.insert(req_id, handles_map);
        }
    }

    let config_clone = active_provider.clone();
    let request_clone = request.clone();
    let abortable_result = Abortable::new(
        async move { translate_with_provider(&config_clone, &request_clone).await },
        abort_registration,
    )
    .await;

    {
        if let Ok(mut handles) = state.main_abort_handles.lock() {
            handles.remove(&req_id);
        }
    }

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = false;
    }

    let mut result = match abortable_result {
        Ok(mut provider_result) => {
            provider_result.detected_source_lang = detected_lang.clone();
            provider_result.target_lang = target_lang.clone();
            provider_result
        }
        Err(Aborted) => ProviderTranslationResult {
            provider: active_provider_name.clone(),
            model: active_provider_model.clone(),
            detected_source_lang: detected_lang.clone(),
            target_lang: target_lang.clone(),
            translation: String::new(),
            success: false,
            error: Some(error_codes::with_code(
                error_codes::TRANSLATION_CANCELLED,
                "请求已取消",
            )),
        },
    };

    if result.success {
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO translation_history (source_text, translated_text, source_lang, target_lang, provider, model) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&text)
        .bind(&result.translation)
        .bind(&result.detected_source_lang)
        .bind(&result.target_lang)
        .bind(&result.provider)
        .bind(&result.model)
        .execute(&state.db)
        .await;
    } else if let Some(err) = result.error.take() {
        result.error = Some(error_codes::ensure_code(
            err,
            error_codes::TRANSLATION_FAILED,
        ));
    }

    if let Ok(mut cancelled) = state.cancelled_requests.lock() {
        cancelled.remove(&req_id);
    }

    let duration_ms = started_at.elapsed().as_millis();
    let status = if result.success { "success" } else { "failed" };
    let detail = result.error.as_deref();
    log_translation_metric(
        "non_stream",
        req_id,
        &result.provider,
        &result.model,
        status,
        duration_ms,
        detail,
    );

    Ok(result)
}

/// 兼容旧命令：保留 `translate_multi` 返回结构
#[tauri::command]
pub async fn translate_multi(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: Option<u64>,
) -> Result<MultiProviderResult, String> {
    let result =
        translate_active_provider(app, state, text, primaryTarget, secondaryTarget, requestId)
            .await?;
    Ok(MultiProviderResult {
        results: vec![result],
    })
}

/// 旧版单服务商翻译（兼容，使用当前启用服务商）
#[tauri::command]
pub async fn translate_text(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: Option<u64>,
) -> Result<TranslationResult, String> {
    let result =
        translate_active_provider(app, state, text, primaryTarget, secondaryTarget, requestId)
            .await?;

    if result.success {
        Ok(TranslationResult {
            detected_source_lang: result.detected_source_lang,
            target_lang: result.target_lang,
            translation: result.translation,
        })
    } else {
        Err(result
            .error
            .unwrap_or_else(|| error_codes::with_code(error_codes::TRANSLATION_FAILED, "翻译失败")))
    }
}

/// 语音合成（基于当前启用的语音服务商）
#[tauri::command]
pub async fn synthesize_speech(
    state: State<'_, AppState>,
    text: String,
) -> Result<SpeechSynthesisResult, String> {
    if text.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::EMPTY_TEXT,
            "输入文本不能为空",
        ));
    }

    let active_provider = get_active_speech_provider(&state.db).await?;
    if active_provider.model.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "当前语音服务商缺少模型配置",
        ));
    }
    if active_provider
        .base_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_none()
    {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "当前语音服务商缺少 API 地址配置",
        ));
    }
    if active_provider.api_key.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "当前语音服务商缺少 API Key 配置",
        ));
    }
    if active_provider.voice.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "当前语音服务商缺少 voice 配置",
        ));
    }

    let response = speech::synthesize_openai_speech(&active_provider, text.trim()).await;
    if response.success {
        if let Some(audio_base64) = response.audio_base64 {
            return Ok(SpeechSynthesisResult {
                provider: active_provider.provider_name,
                model: active_provider.model,
                audio_format: active_provider.audio_format,
                audio_base64,
            });
        }
    }

    let error_message = response.error.unwrap_or_else(|| {
        if response.status_code == 0 {
            "语音合成请求失败".to_string()
        } else {
            format!("语音合成失败（HTTP {}）", response.status_code)
        }
    });

    Err(error_codes::with_code(
        error_codes::SPEECH_SYNTHESIS_FAILED,
        error_message,
    ))
}

/// 取消指定的翻译请求
#[tauri::command]
pub async fn cancel_translation(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] requestId: u64,
) -> Result<bool, String> {
    let mut cancelled_any = false;

    mark_cancelled_request(&state, requestId);

    if let Ok(mut handles) = state.main_abort_handles.lock() {
        if let Some(inner_handles) = handles.remove(&requestId) {
            for (_, handle) in inner_handles {
                handle.abort();
                cancelled_any = true;
            }
        }
    }

    if cancelled_any {
        if let Ok(mut loading) = state.main_loading.lock() {
            *loading = false;
        }
    }

    Ok(cancelled_any)
}

/// 取消所有进行中的翻译请求
#[tauri::command]
pub async fn cancel_all_translations(state: State<'_, AppState>) -> Result<u32, String> {
    let mut count = 0u32;

    // 标记现有请求为已取消（即便无法真正中断上游 HTTP）
    if let Ok(handles) = state.main_abort_handles.lock() {
        if let Ok(mut cancelled) = state.cancelled_requests.lock() {
            for req_id in handles.keys() {
                cancelled.insert(*req_id);
            }
        }
    }

    if let Ok(mut handles) = state.main_abort_handles.lock() {
        for (_, inner_handles) in handles.drain() {
            for (_, handle) in inner_handles {
                handle.abort();
                count += 1;
            }
        }
    }

    if count > 0 {
        if let Ok(mut loading) = state.main_loading.lock() {
            *loading = false;
        }
    }

    Ok(count)
}

/// 使用单个服务商进行流式翻译
async fn translate_stream_with_provider(
    app: &AppHandle,
    config: &ProviderConfig,
    request: &TranslationRequest,
    req_id: u64,
) -> ProviderTranslationResult {
    let started_at = Instant::now();
    let (tx, mut rx) = mpsc::channel::<StreamEvent>(100);

    let provider_name = config.provider_name.clone();
    let model = config.model.clone();
    let request_clone = request.clone();
    let config_clone = config.clone();

    // Start translation task
    let translate_task = tokio::spawn(async move {
        match config_clone.provider_name.as_str() {
            "zhipu" => {
                let provider = ZhipuProvider::new(&config_clone);
                provider.translate_stream(&request_clone, tx, req_id).await
            }
            "ollama" => {
                let provider = OllamaProvider::new(&config_clone);
                provider.translate_stream(&request_clone, tx, req_id).await
            }
            "claude" => {
                let provider = ClaudeProvider::new(&config_clone);
                provider.translate_stream(&request_clone, tx, req_id).await
            }
            _ => {
                let provider = OpenAIProvider::new(&config_clone);
                provider.translate_stream(&request_clone, tx, req_id).await
            }
        }
    });

    // Forward events to the frontend
    let mut final_result: Option<ProviderTranslationResult> = None;

    while let Some(mut event) = rx.recv().await {
        if is_request_cancelled(app, req_id) {
            // 如果前端已取消/关闭，跳过后续事件，等待任务自然结束
            continue;
        }

        // Normalize provider/model to backend ID to keep frontend keys consistent
        match &mut event {
            StreamEvent::Start {
                provider,
                model: evt_model,
                ..
            } => {
                *provider = provider_name.clone();
                if evt_model.is_empty() {
                    *evt_model = model.clone();
                }
            }
            StreamEvent::Chunk { provider, .. } => {
                *provider = provider_name.clone();
            }
            StreamEvent::Done {
                provider,
                model: evt_model,
                ..
            } => {
                *provider = provider_name.clone();
                if evt_model.is_empty() {
                    *evt_model = model.clone();
                }
            }
            StreamEvent::Error { provider, .. } => {
                *provider = provider_name.clone();
            }
        }

        if let StreamEvent::Error { error, .. } = &mut event {
            *error = error_codes::ensure_code(error.clone(), error_codes::TRANSLATION_FAILED);
        }

        // Emit normalized event to frontend
        let _ = app.emit("translation-stream", &event);

        // Process completion and error events
        match &event {
            StreamEvent::Done {
                provider,
                model,
                detected_source_lang,
                target_lang,
                full_translation,
                request_id: event_req_id,
            } => {
                // Only process if the event belongs to the current request (req_id)
                if *event_req_id == req_id {
                    final_result = Some(ProviderTranslationResult {
                        provider: provider.clone(),
                        model: model.clone(),
                        detected_source_lang: detected_source_lang.clone(),
                        target_lang: target_lang.clone(),
                        translation: full_translation.clone(),
                        success: true,
                        error: None,
                    });
                }
            }
            StreamEvent::Error {
                provider,
                error,
                request_id: event_req_id,
            } => {
                // Only process if the event belongs to the current request (req_id)
                if *event_req_id == req_id {
                    final_result = Some(ProviderTranslationResult {
                        provider: provider.clone(),
                        model: model.clone(),
                        detected_source_lang: request.source_lang.clone(),
                        target_lang: request.target_lang.clone(),
                        translation: String::new(),
                        success: false,
                        error: Some(error_codes::ensure_code(
                            error.clone(),
                            error_codes::TRANSLATION_FAILED,
                        )),
                    });
                }
            }
            _ => {}
        }
    }

    // Wait for the translation task to complete
    let _ = translate_task.await;

    // Write to cache (only if successful and it's the current request)
    if let Some(res) = &final_result {
        if res.success {
            let db = app.state::<AppState>().db.clone();
            let text_clone = request.text.clone();
            let res_clone = res.clone();
            tokio::spawn(async move {
                let _ = sqlx::query(
                    "INSERT OR REPLACE INTO translation_history (source_text, translated_text, source_lang, target_lang, provider, model) VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(&text_clone)
                .bind(&res_clone.translation)
                .bind(&res_clone.detected_source_lang)
                .bind(&res_clone.target_lang)
                .bind(&res_clone.provider)
                .bind(&res_clone.model)
                .execute(&db)
                .await;
            });
        }
    }

    let result = final_result.unwrap_or_else(|| ProviderTranslationResult {
        provider: provider_name,
        model,
        detected_source_lang: request.source_lang.clone(),
        target_lang: request.target_lang.clone(),
        translation: String::new(),
        success: false,
        error: Some(error_codes::with_code(
            error_codes::TRANSLATION_FAILED,
            "翻译任务未完成",
        )),
    });

    let status = if result.success { "success" } else { "failed" };
    let detail = result.error.as_deref();
    log_translation_metric(
        "stream",
        req_id,
        &result.provider,
        &result.model,
        status,
        started_at.elapsed().as_millis(),
        detail,
    );

    result
}

/// 语言检测结果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanguageDetectionResult {
    pub detected_lang: String,
    pub target_lang: String,
}

/// 单服务商流式翻译（通过事件流式更新前端）
#[tauri::command]
pub async fn translate_active_provider_stream(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: u64,
) -> Result<LanguageDetectionResult, String> {
    if text.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::EMPTY_TEXT,
            "输入文本不能为空",
        ));
    }
    clear_cancelled_request(&state, requestId);

    let active_provider = get_active_provider(&state.db).await?;
    if !provider_can_translate(&active_provider) {
        return Err(error_codes::with_code(
            error_codes::INVALID_PROVIDER_CONFIG,
            "当前启用服务商缺少可用 API Key，无法进行翻译",
        ));
    }

    let (detected_lang, target_lang) = detect_and_plan(&text, &primaryTarget, &secondaryTarget);

    log_translation_metric(
        "stream",
        requestId,
        &active_provider.provider_name,
        &active_provider.model,
        "accepted",
        0,
        None,
    );

    let base_request = TranslationRequest {
        text: text.clone(),
        source_lang: detected_lang.clone(),
        target_lang: target_lang.clone(),
    };

    let mut abort_handles_for_request = HashMap::new();

    let app_clone = app.clone();
    let config_clone = active_provider.clone();
    let request_clone = base_request.clone();
    let req_id_clone = requestId;

    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    abort_handles_for_request.insert(config_clone.provider_name.clone(), abort_handle);

    let join_handle = tokio::spawn(async move {
        let _ = Abortable::new(
            translate_stream_with_provider(&app_clone, &config_clone, &request_clone, req_id_clone),
            abort_registration,
        )
        .await;
    });

    // Store the individual abort handles under the main requestId
    {
        if let Ok(mut handles) = state.main_abort_handles.lock() {
            // Remove any previous handles for this request ID
            handles.remove(&requestId);
            // Store new handles
            handles.insert(requestId, abort_handles_for_request.into());
        }
    }

    // Cleanup abort handles once all streaming tasks complete
    {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = join_handle.await;
            if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(mut handles) = state.main_abort_handles.lock() {
                    handles.remove(&requestId);
                }
                if let Ok(mut cancelled) = state.cancelled_requests.lock() {
                    cancelled.remove(&requestId);
                }
            }
        });
    }

    Ok(LanguageDetectionResult {
        detected_lang,
        target_lang,
    })
}

/// 使用指定的源语言和目标语言进行流式翻译（跳过语言检测）
#[tauri::command]
pub async fn translate_active_provider_with_specified_langs_stream(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] sourceLang: String,
    #[allow(non_snake_case)] targetLang: String,
    #[allow(non_snake_case)] requestId: u64,
) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::EMPTY_TEXT,
            "输入文本不能为空",
        ));
    }
    clear_cancelled_request(&state, requestId);

    let active_provider = get_active_provider(&state.db).await?;
    if !provider_can_translate(&active_provider) {
        return Err(error_codes::with_code(
            error_codes::INVALID_PROVIDER_CONFIG,
            "当前启用服务商缺少可用 API Key，无法进行翻译",
        ));
    }

    log_translation_metric(
        "stream_with_langs",
        requestId,
        &active_provider.provider_name,
        &active_provider.model,
        "accepted",
        0,
        None,
    );

    let base_request = TranslationRequest {
        text: text.clone(),
        source_lang: sourceLang.clone(),
        target_lang: targetLang.clone(),
    };

    let mut abort_handles_for_request = HashMap::new();

    let app_clone = app.clone();
    let config_clone = active_provider.clone();
    let request_clone = base_request.clone();
    let req_id_clone = requestId;

    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    abort_handles_for_request.insert(config_clone.provider_name.clone(), abort_handle);

    let join_handle = tokio::spawn(async move {
        let _ = Abortable::new(
            translate_stream_with_provider(&app_clone, &config_clone, &request_clone, req_id_clone),
            abort_registration,
        )
        .await;
    });

    // Store the individual abort handles under the main requestId
    {
        if let Ok(mut handles) = state.main_abort_handles.lock() {
            handles.remove(&requestId);
            handles.insert(requestId, abort_handles_for_request.into());
        }
    }

    // Cleanup abort handles once all streaming tasks complete
    {
        let app_handle = app.clone();
        tokio::spawn(async move {
            let _ = join_handle.await;
            if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(mut handles) = state.main_abort_handles.lock() {
                    handles.remove(&requestId);
                }
                if let Ok(mut cancelled) = state.cancelled_requests.lock() {
                    cancelled.remove(&requestId);
                }
            }
        });
    }

    Ok(())
}

/// 兼容旧命令：保留 `translate_multi_stream_individual` 命名
#[tauri::command]
pub async fn translate_multi_stream_individual(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: u64,
) -> Result<LanguageDetectionResult, String> {
    translate_active_provider_stream(app, state, text, primaryTarget, secondaryTarget, requestId)
        .await
}

/// 兼容旧命令：保留 `translate_with_specified_langs` 命名
#[tauri::command]
pub async fn translate_with_specified_langs(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] sourceLang: String,
    #[allow(non_snake_case)] targetLang: String,
    #[allow(non_snake_case)] requestId: u64,
) -> Result<(), String> {
    translate_active_provider_with_specified_langs_stream(
        app, state, text, sourceLang, targetLang, requestId,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_request_id_prefers_given_value() {
        assert_eq!(now_request_id(Some(42)), 42);
    }

    #[test]
    fn normalize_model_uses_fallback_for_empty_model() {
        let got = normalize_model("openai", "");
        assert_eq!(got, providers::DEFAULT_OPENAI_MODEL);
    }

    #[test]
    fn normalize_base_url_uses_preset_default() {
        let got = normalize_base_url("moonshot", None);
        assert_eq!(
            got.as_deref(),
            Some("https://api.moonshot.ai/v1/chat/completions")
        );
    }

    #[test]
    fn provider_can_translate_matches_expected_rules() {
        let ollama = ProviderConfig {
            provider_name: "ollama".to_string(),
            enabled: true,
            api_key: String::new(),
            model: "llama3.2".to_string(),
            base_url: Some("http://localhost:11434/api/chat".to_string()),
        };
        assert!(provider_can_translate(&ollama));

        let openai_without_key = ProviderConfig {
            provider_name: "openai".to_string(),
            enabled: true,
            api_key: String::new(),
            model: "gpt-4.1-mini".to_string(),
            base_url: Some("https://api.openai.com/v1/chat/completions".to_string()),
        };
        assert!(!provider_can_translate(&openai_without_key));
    }

    #[test]
    fn normalize_whatlang_code_maps_common_languages() {
        assert_eq!(normalize_whatlang_code_to_app_lang("eng"), Some("en"));
        assert_eq!(normalize_whatlang_code_to_app_lang("cmn"), Some("zh-CN"));
        assert_eq!(normalize_whatlang_code_to_app_lang("jpn"), Some("ja"));
        assert_eq!(normalize_whatlang_code_to_app_lang("kor"), Some("ko"));
        assert_eq!(normalize_whatlang_code_to_app_lang("spa"), Some("es"));
    }

    #[test]
    fn detect_language_locally_has_reasonable_fallback() {
        assert_eq!(detect_language_locally(""), "en");
        assert_eq!(detect_language_locally("你好"), "zh-CN");
    }
}
