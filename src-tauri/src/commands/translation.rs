use crate::services::ai::{
    ProviderConfig, TranslationRequest, TranslationResponse, AIProvider, StreamEvent,
    zhipu::ZhipuProvider,
    openai::OpenAIProvider,
    claude::ClaudeProvider,
    ollama::OllamaProvider,
    key::resolve_default_zhipu_api_key,
};
use crate::config::providers::{self, DEFAULT_ZHIPU_MODEL};
use crate::services::encryption::decrypt_api_key;
use crate::AppState;
use futures::future::{join_all, Abortable, AbortHandle, Aborted};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State, Manager};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use std::collections::HashMap;

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

/// 确保语言检测始终使用智谱，模型默认 glm-4-flash，API Key 远程拉取失败则使用内置。
async fn zhipu_detector_config(db: &sqlx::SqlitePool) -> Result<ProviderConfig, String> {
    #[derive(sqlx::FromRow)]
    struct ZhipuRow {
        api_key: String,
        model: String,
        base_url: Option<String>,
    }

    let row: Option<ZhipuRow> = sqlx::query_as(
        "SELECT api_key, model, base_url FROM provider_configs WHERE provider_name = 'zhipu' LIMIT 1",
    )
    .fetch_optional(db)
    .await
    .map_err(|e| e.to_string())?;

    let mut api_key = row
        .as_ref()
        .and_then(|r| if r.api_key.is_empty() { None } else { Some(decrypt_api_key(&r.api_key).unwrap_or_default()) })
        .unwrap_or_default();

    if api_key.is_empty() {
        api_key = resolve_default_zhipu_api_key().await;
    }

    let model = row
        .as_ref()
        .map(|r| if r.model.is_empty() { DEFAULT_ZHIPU_MODEL.to_string() } else { r.model.clone() })
        .unwrap_or_else(|| DEFAULT_ZHIPU_MODEL.to_string());

    Ok(ProviderConfig {
        provider_name: "zhipu".to_string(),
        enabled: true,
        api_key,
        model,
        base_url: row.and_then(|r| r.base_url),
    })
}

/// 从数据库获取已启用的服务商配置
async fn get_enabled_providers(db: &sqlx::SqlitePool) -> Result<Vec<ProviderConfig>, String> {
    #[derive(sqlx::FromRow)]
    struct DbProviderConfig {
        provider_name: String,
        enabled: i32,
        api_key: String,
        model: String,
        base_url: Option<String>,
    }

    let rows: Vec<DbProviderConfig> = sqlx::query_as(
        "SELECT provider_name, enabled, api_key, model, base_url FROM provider_configs WHERE enabled = 1 OR provider_name = 'zhipu'"
    )
    .fetch_all(db)
    .await
    .map_err(|e| e.to_string())?;

    let mut configs = Vec::new();
    for row in rows {
        let api_key = if !row.api_key.is_empty() {
            decrypt_api_key(&row.api_key).unwrap_or_default()
        } else if row.provider_name == "zhipu" {
            resolve_default_zhipu_api_key().await
        } else {
            String::new()
        };

        // Fallback to a sane default model if DB value is empty
        let model = if row.model.is_empty() {
            match row.provider_name.as_str() {
                "zhipu" => DEFAULT_ZHIPU_MODEL.to_string(),
                "openai" => providers::OPENAI_MODELS.first().map(|m| m.id).unwrap_or("gpt-4o-mini").to_string(),
                "claude" => providers::CLAUDE_MODELS.first().map(|m| m.id).unwrap_or("claude-3-5-haiku-latest").to_string(),
                "ollama" => "llama3.2".to_string(),
                _ => String::new(),
            }
        } else {
            row.model.clone()
        };

        configs.push(ProviderConfig {
            provider_name: row.provider_name,
            enabled: true, // 智谱强制启用，其余因查询条件已过滤
            api_key,
            model,
            base_url: row.base_url,
        });
    }

    Ok(configs)
}

// Helper function: Detect language and determine target language
async fn detect_and_plan(
    config: &ProviderConfig,
    text: &str,
    primary_target: &str,
    secondary_target: &str,
) -> Result<(String, String), String> {
    // 语言检测强制只使用智谱
    let detected_lang = match config.provider_name.as_str() {
        "zhipu" => ZhipuProvider::new(config).detect_language(text).await,
        _ => Err(crate::services::ai::AIError::Other(
            "语言检测仅支持智谱 AI".to_string(),
        )),
    }
    .map_err(|e| e.to_string())?;

    // Simple normalization of detected_lang (take the first two characters, convert to lowercase)
    let normalized_detected = detected_lang.to_lowercase();
    let is_primary = normalized_detected.starts_with(&primary_target.to_lowercase()[..2.min(primary_target.len())]);

    let target_lang = if is_primary {
        secondary_target.to_string()
    } else {
        primary_target.to_string()
    };

    Ok((detected_lang, target_lang))
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
        "openai" => {
            let provider = OpenAIProvider::new(config);
            provider.translate(request).await.map_err(|e| e.to_string())
        }
        "claude" => {
            let provider = ClaudeProvider::new(config);
            provider.translate(request).await.map_err(|e| e.to_string())
        }
        "ollama" => {
            let provider = OllamaProvider::new(config);
            provider.translate(request).await.map_err(|e| e.to_string())
        }
        _ => Err(format!("未知服务商: {}", config.provider_name)),
    };

    match result {
        Ok(resp) => ProviderTranslationResult {
            provider: resp.provider,
            model: resp.model,
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
            error: Some(e),
        },
    }
}

/// 多服务商并行翻译
#[tauri::command]
pub async fn translate_multi(
    _app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: Option<u64>,
) -> Result<MultiProviderResult, String> {
    if text.trim().is_empty() {
        return Err("Text is empty".to_string());
    }

    // Generate request ID
    let req_id = requestId.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    });
    clear_cancelled_request(&state, req_id);

    // Get enabled providers
    let providers = get_enabled_providers(&state.db).await?;

    if providers.is_empty() {
        return Err("没有已启用的服务商。请在设置中配置并启用至少一个服务商。".to_string());
    }

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = true;
    }

    // 1. Language detection (only Zhipu)
    let detector_config = zhipu_detector_config(&state.db).await?;
    let (detected_lang, target_lang) = match detect_and_plan(&detector_config, &text, &primaryTarget, &secondaryTarget).await {
        Ok(res) => res,
        Err(e) => {
            if let Ok(mut loading) = state.main_loading.lock() {
                *loading = false;
            }
            return Err(format!("语言检测失败: {}", e));
        }
    };

    let request = TranslationRequest {
        text: text.clone(),
        source_lang: detected_lang.clone(),
        target_lang: target_lang.clone(),
    };

    // Create abortable futures
    let mut abort_handles = Vec::new();
    let mut abortable_futures = Vec::new();

    for config in providers.iter() {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        abort_handles.push(abort_handle);

        let config_clone = config.clone();
        let request_clone = request.clone();
        let future = async move {
            translate_with_provider(&config_clone, &request_clone).await
        };

        abortable_futures.push(Abortable::new(future, abort_registration));
    }

    // 存储 abort handles 以便取消
    {
        let mut handles_map = HashMap::new();
        // Note: providers and abort_handles are aligned by index
        for (i, handle) in abort_handles.into_iter().enumerate() {
            if let Some(provider) = providers.get(i) {
                handles_map.insert(provider.provider_name.clone(), handle);
            }
        }

        if let Ok(mut handles) = state.main_abort_handles.lock() {
            handles.insert(req_id, handles_map);
        }
    }

    // Execute all translation requests in parallel
    let abortable_results = join_all(abortable_futures).await;

    // Clear abort handles
    {
        if let Ok(mut handles) = state.main_abort_handles.lock() {
            handles.remove(&req_id);
        }
    }

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = false;
    }

    // Process results
    let mut results = Vec::new();
    for (i, result) in abortable_results.into_iter().enumerate() {
        match result {
            Ok(mut provider_result) => {
                provider_result.detected_source_lang = detected_lang.clone();
                provider_result.target_lang = target_lang.clone();
                results.push(provider_result);
            },
            Err(Aborted) => {
                // Request was cancelled
                results.push(ProviderTranslationResult {
                    provider: providers[i].provider_name.clone(),
                    model: providers[i].model.clone(),
                    detected_source_lang: detected_lang.clone(),
                    target_lang: target_lang.clone(),
                    translation: String::new(),
                    success: false,
                    error: Some("请求已取消".to_string()),
                });
            }
        }
    }

    // Write to cache (using the first successful result)
    if let Some(first_success) = results.iter().find(|r| r.success) {
        let _ = sqlx::query(
            "INSERT OR REPLACE INTO translation_history (source_text, translated_text, source_lang, target_lang, provider, model) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&text)
        .bind(&first_success.translation)
        .bind(&first_success.detected_source_lang)
        .bind(&first_success.target_lang)
        .bind(&first_success.provider)
        .bind(&first_success.model)
        .execute(&state.db)
        .await;
    }

    if let Ok(mut cancelled) = state.cancelled_requests.lock() {
        cancelled.remove(&req_id);
    }

    Ok(MultiProviderResult { results })
}

/// 旧版单服务商翻译（兼容，使用第一个已启用的服务商）
#[tauri::command]
pub async fn translate_text(
    _app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: Option<u64>,
) -> Result<TranslationResult, String> {
    let req_id = requestId.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    });
    clear_cancelled_request(&state, req_id);

    println!(
        "[translate_text] start | req_id={:?} primary={} secondary={} text_len={}",
        req_id,
        primaryTarget,
        secondaryTarget,
        text.len()
    );

    if text.trim().is_empty() {
        return Err("Text is empty".to_string());
    }

    // 获取已启用的服务商
    let providers = get_enabled_providers(&state.db).await?;

    if providers.is_empty() {
        return Err("没有已启用的服务商。请在设置中配置并启用至少一个服务商。".to_string());
    }

    // 使用第一个已启用的服务商
    let config = &providers[0];

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = true;
    }

    // 1. 语言检测（智谱）
    let detector_config = zhipu_detector_config(&state.db).await?;
    let (detected_lang, target_lang) = match detect_and_plan(&detector_config, &text, &primaryTarget, &secondaryTarget).await {
        Ok(res) => res,
        Err(e) => {
            if let Ok(mut loading) = state.main_loading.lock() {
                *loading = false;
            }
            return Err(format!("语言检测失败: {}", e));
        }
    };

    let request = TranslationRequest {
        text: text.clone(),
        source_lang: detected_lang.clone(),
        target_lang: target_lang.clone(),
    };

    let mut result = translate_with_provider(config, &request).await;

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = false;
    }

    if result.success {
        result.detected_source_lang = detected_lang.clone();
        result.target_lang = target_lang.clone();

        // 写入缓存
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

        if let Ok(mut cancelled) = state.cancelled_requests.lock() {
            cancelled.remove(&req_id);
        }

        Ok(TranslationResult {
            detected_source_lang: result.detected_source_lang,
            target_lang: result.target_lang,
            translation: result.translation,
        })
    } else {
        if let Ok(mut cancelled) = state.cancelled_requests.lock() {
            cancelled.remove(&req_id);
        }
        Err(result.error.unwrap_or_else(|| "翻译失败".to_string()))
    }
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
pub async fn cancel_all_translations(
    state: State<'_, AppState>,
) -> Result<u32, String> {
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
            "openai" => {
                let provider = OpenAIProvider::new(&config_clone);
                provider.translate_stream(&request_clone, tx, req_id).await
            }
            "claude" => {
                let provider = ClaudeProvider::new(&config_clone);
                provider.translate_stream(&request_clone, tx, req_id).await
            }
            "ollama" => {
                let provider = OllamaProvider::new(&config_clone);
                provider.translate_stream(&request_clone, tx, req_id).await
            }
            _ => {
                let _ = tx.send(StreamEvent::Error {
                    provider: config_clone.provider_name.clone(),
                    error: format!("未知服务商: {}", config_clone.provider_name),
                    request_id: req_id, // Pass req_id here
                }).await;
                Err(crate::services::ai::AIError::Other(format!("未知服务商: {}", config_clone.provider_name)))
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
            StreamEvent::Start { provider, model: evt_model, .. } => {
                *provider = provider_name.clone();
                if evt_model.is_empty() {
                    *evt_model = model.clone();
                }
            }
            StreamEvent::Chunk { provider, .. } => {
                *provider = provider_name.clone();
            }
            StreamEvent::Done { provider, model: evt_model, .. } => {
                *provider = provider_name.clone();
                if evt_model.is_empty() {
                    *evt_model = model.clone();
                }
            }
            StreamEvent::Error { provider, .. } => {
                *provider = provider_name.clone();
            }
        }

        // Emit normalized event to frontend
        let _ = app.emit("translation-stream", &event);

        // Process completion and error events
        match &event {
            StreamEvent::Done { provider, model, detected_source_lang, target_lang, full_translation, request_id: event_req_id } => {
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
            StreamEvent::Error { provider, error, request_id: event_req_id } => {
                // Only process if the event belongs to the current request (req_id)
                if *event_req_id == req_id {
                    final_result = Some(ProviderTranslationResult {
                        provider: provider.clone(),
                        model: model.clone(),
                        detected_source_lang: request.source_lang.clone(),
                        target_lang: request.target_lang.clone(),
                        translation: String::new(),
                        success: false,
                        error: Some(error.clone()),
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

    final_result.unwrap_or_else(|| ProviderTranslationResult {
        provider: provider_name,
        model,
        detected_source_lang: request.source_lang.clone(),
        target_lang: request.target_lang.clone(),
        translation: String::new(),
        success: false,
        error: Some("翻译任务未完成".to_string()),
    })
}

/// 多服务商并行流式翻译 (Old approach: waits for all results then returns)
async fn _translate_multi_stream_parallel(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    primary_target: String,
    secondary_target: String,
    request_id: Option<u64>,
) -> Result<MultiProviderResult, String> {
    if text.trim().is_empty() {
        return Err("Text is empty".to_string());
    }

    let req_id = request_id.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    });

    let providers = get_enabled_providers(&state.db).await?;

    if providers.is_empty() {
        return Err("没有已启用的服务商。请在设置中配置并启用至少一个服务商。".to_string());
    }

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = true;
    }

    let detector_config = providers.iter()
        .find(|p| p.provider_name == "zhipu")
        .or_else(|| providers.first())
        .ok_or("无可用服务商")?;

    let (detected_lang, target_lang) = match detect_and_plan(detector_config, &text, &primary_target, &secondary_target).await {
        Ok(res) => res,
        Err(e) => {
            if let Ok(mut loading) = state.main_loading.lock() {
                *loading = false;
            }
            return Err(format!("语言检测失败: {}", e));
        }
    };

    let request = TranslationRequest {
        text: text.clone(),
        source_lang: detected_lang.clone(),
        target_lang: target_lang.clone(),
    };

    let mut futures = Vec::new();
    for config in providers.iter() {
        let app_clone = app.clone();
        let config_clone = config.clone();
        let request_clone = request.clone();
        let req_id_clone = req_id;

        futures.push(async move {
            translate_stream_with_provider(&app_clone, &config_clone, &request_clone, req_id_clone).await
        });
    }

    let results = join_all(futures).await;

    if let Ok(mut loading) = state.main_loading.lock() {
        *loading = false;
    }

    let mut final_results = Vec::new();
    for mut res in results {
        res.detected_source_lang = detected_lang.clone();
        res.target_lang = target_lang.clone();
        final_results.push(res);
    }
    
    Ok(MultiProviderResult { results: final_results })
}


/// 多服务商并行流式翻译（通过事件流式更新前端）
#[tauri::command]
pub async fn translate_multi_stream_individual(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
    #[allow(non_snake_case)] primaryTarget: String,
    #[allow(non_snake_case)] secondaryTarget: String,
    #[allow(non_snake_case)] requestId: u64,
    providers: Vec<String>, // List of provider names to use
) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err("Text is empty".to_string());
    }
    clear_cancelled_request(&state, requestId);

    // Get all enabled providers
    let all_enabled_providers = get_enabled_providers(&state.db).await?;
    if all_enabled_providers.is_empty() {
        return Err("没有已启用的服务商。请在设置中配置并启用至少一个服务商。".to_string());
    }

    // Filter providers based on the requested names
    let selected_providers: Vec<ProviderConfig> = all_enabled_providers.into_iter()
        .filter(|p| providers.contains(&p.provider_name))
        .collect();

    if selected_providers.is_empty() {
        return Err("没有找到指定的已启用服务商。".to_string());
    }

    // 1. Language detection (Zhipu only)
    let detector_config = zhipu_detector_config(&state.db).await?;

    let (detected_lang, target_lang) = match detect_and_plan(&detector_config, &text, &primaryTarget, &secondaryTarget).await {
        Ok(res) => res,
        Err(e) => {
            return Err(format!("语言检测失败: {}", e));
        }
    };

    let base_request = TranslationRequest {
        text: text.clone(),
        source_lang: detected_lang.clone(),
        target_lang: target_lang.clone(),
    };

    // Store abort handles for cancellation
    let mut abort_handles_for_request = HashMap::new();

    let mut join_handles = Vec::new();

    for config in selected_providers.into_iter() {
        let app_clone = app.clone();
        let config_clone = config.clone();
        let request_clone = base_request.clone();
        let req_id_clone = requestId;

        // Create an AbortHandle for each individual streaming task
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        abort_handles_for_request.insert(config_clone.provider_name.clone(), abort_handle);

        let join_handle = tokio::spawn(async move {
            let _ = Abortable::new(
                translate_stream_with_provider(&app_clone, &config_clone, &request_clone, req_id_clone),
                abort_registration
            ).await;
        });

        join_handles.push(join_handle);
    }

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
            let _ = futures::future::join_all(join_handles).await;
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
