use crate::commands::error_codes;
use crate::config::providers;
use crate::services::ai::claude::ClaudeProvider;
use crate::services::ai::ollama::OllamaProvider;
use crate::services::ai::openai::OpenAIProvider;
use crate::services::ai::zhipu::ZhipuProvider;
use crate::services::ai::AIProvider;
use crate::services::ai::{ApiTestResponse, ProviderConfig};
use crate::services::encryption::{decrypt_api_key, encrypt_api_key};
use crate::services::speech::{self, SpeechProviderConfig};
use crate::AppState;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub api_key: String,
    pub theme: String,
    /// 第一语言（默认翻译目标，如：中文）
    pub primary_target: String,
    /// 第二语言（当源语言是第一语言时使用，如：英文）
    pub secondary_target: String,
    /// 界面语言
    pub locale: String,
}

/// 服务商信息（包括可选模型）
#[derive(Debug, Serialize, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub config: ProviderConfig,
    pub available_models: Vec<String>,
    pub available_model_configs: Vec<providers::ModelConfig>,
    pub supports_base_url: bool,
    pub supports_custom_model: bool,
    pub is_preset: bool,
    pub api_key_optional: bool,
    pub default_base_url: Option<String>,
}

/// 语音服务商信息（独立于翻译服务商）
#[derive(Debug, Serialize, Clone)]
pub struct SpeechProviderInfo {
    pub name: String,
    pub display_name: String,
    pub config: SpeechProviderConfig,
    pub is_preset: bool,
    pub api_key_optional: bool,
    pub default_base_url: Option<String>,
    pub default_model: String,
    pub default_voice: String,
    pub default_audio_format: String,
}

// Helper to get value from DB
async fn get_val(pool: &sqlx::SqlitePool, key: &str) -> Option<String> {
    sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
}

fn is_localhost_base_url(base_url: &str) -> bool {
    let Ok(url) = Url::parse(base_url) else {
        return false;
    };
    let Some(host) = url.host_str() else {
        return false;
    };
    let normalized_host = host.trim_matches(&['[', ']'][..]);
    matches!(normalized_host, "localhost" | "127.0.0.1" | "::1")
}

fn validate_provider_enable_config(
    provider_name: &str,
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::INVALID_PROVIDER_CONFIG,
            "启用服务商前，请先填写模型",
        ));
    }

    let base_url = base_url
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .ok_or_else(|| {
            error_codes::with_code(
                error_codes::INVALID_PROVIDER_CONFIG,
                "启用服务商前，请先填写 API 地址",
            )
        })?;

    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        return Err(error_codes::with_code(
            error_codes::INVALID_PROVIDER_CONFIG,
            "API 地址格式无效，请使用 http(s):// 开头",
        ));
    }

    let api_key_optional = if let Some(preset) = providers::preset_by_id(provider_name) {
        preset.api_key_optional
    } else {
        false
    };

    if !api_key_optional && api_key.trim().is_empty() && !is_localhost_base_url(base_url) {
        return Err(error_codes::with_code(
            error_codes::INVALID_PROVIDER_CONFIG,
            "启用服务商前，请先填写 API Key",
        ));
    }

    Ok(())
}

fn normalize_audio_format(audio_format: &str) -> String {
    let normalized = audio_format.trim().to_lowercase();
    if normalized.is_empty() {
        providers::DEFAULT_TTS_AUDIO_FORMAT.to_string()
    } else {
        normalized
    }
}

fn validate_speech_provider_enable_config(
    provider_name: &str,
    api_key: &str,
    model: &str,
    base_url: Option<&str>,
    voice: &str,
    audio_format: &str,
) -> Result<(), String> {
    if model.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "启用语音服务商前，请先填写模型",
        ));
    }

    let base_url = base_url
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .ok_or_else(|| {
            error_codes::with_code(
                error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
                "启用语音服务商前，请先填写 API 地址",
            )
        })?;

    if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "语音 API 地址格式无效，请使用 http(s):// 开头",
        ));
    }

    if voice.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "启用语音服务商前，请先填写 voice",
        ));
    }

    let audio_format = normalize_audio_format(audio_format);
    if audio_format.trim().is_empty() {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "启用语音服务商前，请先填写 audio format",
        ));
    }

    let api_key_optional = if let Some(preset) = providers::speech_preset_by_id(provider_name) {
        preset.api_key_optional
    } else {
        false
    };

    if !api_key_optional && api_key.trim().is_empty() && !is_localhost_base_url(base_url) {
        return Err(error_codes::with_code(
            error_codes::INVALID_SPEECH_PROVIDER_CONFIG,
            "启用语音服务商前，请先填写 API Key",
        ));
    }

    Ok(())
}

async fn disable_other_enabled<'e, E>(executor: E, provider_name: &str) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query(
        r#"
        UPDATE provider_configs
        SET enabled = 0, updated_at = CURRENT_TIMESTAMP
        WHERE provider_name != ? AND enabled = 1
        "#,
    )
    .bind(provider_name)
    .execute(executor)
    .await?;
    Ok(())
}

async fn disable_other_speech_enabled<'e, E>(
    executor: E,
    provider_name: &str,
) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    sqlx::query(
        r#"
        UPDATE speech_provider_configs
        SET enabled = 0, updated_at = CURRENT_TIMESTAMP
        WHERE provider_name != ? AND enabled = 1
        "#,
    )
    .bind(provider_name)
    .execute(executor)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    // Encrypt API Key
    let encrypted_key = encrypt_api_key(&settings.api_key)?;

    // Save to DB
    let queries = [
        ("api_key", encrypted_key),
        ("theme", settings.theme.clone()),
        ("primary_target", settings.primary_target.clone()),
        ("secondary_target", settings.secondary_target.clone()),
        ("locale", settings.locale.clone()),
    ];

    for (key, value) in queries {
        sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)")
            .bind(key)
            .bind(value)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Emit event to all windows
    app.emit("settings-changed", &settings)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let encrypted_key = get_val(&state.db, "api_key").await.unwrap_or_default();
    let theme = get_val(&state.db, "theme")
        .await
        .unwrap_or_else(|| "dark".to_string());
    let primary_target = get_val(&state.db, "primary_target")
        .await
        .unwrap_or_else(|| "zh-CN".to_string());
    let secondary_target = get_val(&state.db, "secondary_target")
        .await
        .unwrap_or_else(|| "en".to_string());
    let locale = get_val(&state.db, "locale")
        .await
        .unwrap_or_else(|| "zh-CN".to_string());

    let api_key = if !encrypted_key.is_empty() {
        decrypt_api_key(&encrypted_key).unwrap_or_default()
    } else {
        String::new()
    };

    Ok(AppSettings {
        api_key,
        theme,
        primary_target,
        secondary_target,
        locale,
    })
}

/// 获取所有服务商配置
#[tauri::command]
pub async fn get_provider_configs(state: State<'_, AppState>) -> Result<Vec<ProviderInfo>, String> {
    crate::app_info!("[Settings] get_provider_configs 被调用");

    #[derive(sqlx::FromRow)]
    struct DbProviderConfig {
        provider_name: String,
        enabled: i32,
        api_key: String,
        model: String,
        base_url: Option<String>,
    }

    crate::app_info!("[Settings] 正在查询 provider_configs 表...");
    let rows: Vec<DbProviderConfig> = sqlx::query_as(
        "SELECT provider_name, enabled, api_key, model, base_url FROM provider_configs ORDER BY provider_name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        crate::app_error!("[Settings] 查询 provider_configs 失败: {}", e);
        e.to_string()
    })?;

    crate::app_info!("[Settings] 查询到 {} 条记录", rows.len());
    for row in &rows {
        crate::app_info!(
            "[Settings] - provider: {}, enabled: {}",
            row.provider_name,
            row.enabled
        );
    }

    let mut provider_infos = Vec::new();
    for row in rows {
        let api_key = if !row.api_key.is_empty() {
            decrypt_api_key(&row.api_key).unwrap_or_default()
        } else {
            String::new()
        };

        let (
            display_name,
            available_models,
            available_model_configs,
            supports_base_url,
            supports_custom_model,
            is_preset,
            default_model,
            api_key_optional,
            default_base_url,
        ) = if let Some(preset) = providers::preset_by_id(&row.provider_name) {
            (
                preset.display_name.to_string(),
                preset
                    .model_configs
                    .iter()
                    .map(|m| m.id.to_string())
                    .collect::<Vec<String>>(),
                preset.model_configs.to_vec(),
                preset.supports_base_url,
                preset.supports_custom_model,
                true,
                preset.default_model.to_string(),
                preset.api_key_optional,
                preset.default_base_url.map(|v| v.to_string()),
            )
        } else if row.provider_name == "claude" {
            (
                "Claude".to_string(),
                providers::CLAUDE_MODELS
                    .iter()
                    .map(|m| m.id.to_string())
                    .collect::<Vec<String>>(),
                providers::CLAUDE_MODELS.to_vec(),
                false,
                true,
                false,
                providers::DEFAULT_CLAUDE_MODEL.to_string(),
                false,
                None,
            )
        } else {
            let mut custom_models = Vec::new();
            if !row.model.trim().is_empty() {
                custom_models.push(row.model.trim().to_string());
            }
            (
                row.provider_name.clone(),
                custom_models,
                Vec::new(),
                true,
                true,
                false,
                String::new(),
                false,
                None,
            )
        };

        // fallback to default model if DB value is empty (helps fresh installs)
        let model = if row.model.trim().is_empty() {
            default_model.clone()
        } else {
            row.model.trim().to_string()
        };

        let base_url = if let Some(base) = row.base_url {
            let trimmed = base.trim().to_string();
            if trimmed.is_empty() {
                default_base_url.clone()
            } else {
                Some(trimmed)
            }
        } else {
            default_base_url.clone()
        };

        let provider_name = row.provider_name;
        let enabled = row.enabled != 0;

        provider_infos.push(ProviderInfo {
            name: provider_name.clone(),
            display_name,
            config: ProviderConfig {
                provider_name,
                enabled,
                api_key,
                model,
                base_url,
            },
            available_models,
            available_model_configs,
            supports_base_url,
            supports_custom_model,
            is_preset,
            api_key_optional,
            default_base_url,
        });
    }

    provider_infos.sort_by(|a, b| {
        match (
            providers::preset_order(&a.name),
            providers::preset_order(&b.name),
        ) {
            (Some(ai), Some(bi)) => ai.cmp(&bi),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a
                .display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase()),
        }
    });

    crate::app_info!("[Settings] 返回 {} 个服务商配置", provider_infos.len());
    Ok(provider_infos)
}

/// 获取所有语音服务商配置
#[tauri::command]
pub async fn get_speech_provider_configs(
    state: State<'_, AppState>,
) -> Result<Vec<SpeechProviderInfo>, String> {
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

    let rows: Vec<DbSpeechProviderConfig> = sqlx::query_as(
        "SELECT provider_name, enabled, api_key, model, base_url, voice, audio_format FROM speech_provider_configs ORDER BY provider_name",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    let mut infos = Vec::new();
    for row in rows {
        let api_key = if !row.api_key.is_empty() {
            decrypt_api_key(&row.api_key).unwrap_or_default()
        } else {
            String::new()
        };

        let (
            display_name,
            is_preset,
            api_key_optional,
            default_base_url,
            default_model,
            default_voice,
            default_audio_format,
        ) = if let Some(preset) = providers::speech_preset_by_id(&row.provider_name) {
            (
                preset.display_name.to_string(),
                true,
                preset.api_key_optional,
                preset.default_base_url.map(|v| v.to_string()),
                preset.default_model.to_string(),
                preset.default_voice.to_string(),
                preset.default_audio_format.to_string(),
            )
        } else {
            (
                row.provider_name.clone(),
                false,
                false,
                None,
                String::new(),
                providers::DEFAULT_XIAOMI_TTS_VOICE.to_string(),
                providers::DEFAULT_TTS_AUDIO_FORMAT.to_string(),
            )
        };

        let model = if row.model.trim().is_empty() {
            default_model.clone()
        } else {
            row.model.trim().to_string()
        };

        let base_url = if let Some(base) = row.base_url {
            let trimmed = base.trim().to_string();
            if trimmed.is_empty() {
                default_base_url.clone()
            } else {
                Some(trimmed)
            }
        } else {
            default_base_url.clone()
        };

        let voice = if row.voice.trim().is_empty() {
            default_voice.clone()
        } else {
            row.voice.trim().to_string()
        };

        let audio_format = if row.audio_format.trim().is_empty() {
            default_audio_format.clone()
        } else {
            normalize_audio_format(&row.audio_format)
        };

        let provider_name = row.provider_name;
        let enabled = row.enabled != 0;

        infos.push(SpeechProviderInfo {
            name: provider_name.clone(),
            display_name,
            config: SpeechProviderConfig {
                provider_name,
                enabled,
                api_key,
                model,
                base_url,
                voice,
                audio_format,
            },
            is_preset,
            api_key_optional,
            default_base_url,
            default_model,
            default_voice,
            default_audio_format,
        });
    }

    infos.sort_by(|a, b| {
        match (
            providers::speech_preset_order(&a.name),
            providers::speech_preset_order(&b.name),
        ) {
            (Some(ai), Some(bi)) => ai.cmp(&bi),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a
                .display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase()),
        }
    });

    Ok(infos)
}

/// 保存单个服务商配置
#[tauri::command]
pub async fn save_provider_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ProviderConfig,
) -> Result<(), String> {
    let provider_name = config.provider_name.trim().to_string();
    if provider_name.is_empty() {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_NAME_EMPTY,
            "服务商名称不能为空",
        ));
    }

    let encrypted_key = if !config.api_key.is_empty() {
        encrypt_api_key(&config.api_key)?
    } else {
        String::new()
    };

    let model = if config.model.trim().is_empty() {
        providers::default_model_for_provider(&provider_name)
            .unwrap_or("")
            .to_string()
    } else {
        config.model.trim().to_string()
    };

    let base_url = if let Some(base) = config.base_url {
        let trimmed = base.trim().to_string();
        if trimmed.is_empty() {
            providers::default_base_url_for_provider(&provider_name).map(|v| v.to_string())
        } else {
            Some(trimmed)
        }
    } else {
        providers::default_base_url_for_provider(&provider_name).map(|v| v.to_string())
    };

    if config.enabled {
        validate_provider_enable_config(
            &provider_name,
            &config.api_key,
            &model,
            base_url.as_deref(),
        )?;
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    // 单选启用策略：如果当前服务商启用，则自动禁用其他服务商
    if config.enabled {
        disable_other_enabled(&mut *tx, &provider_name)
            .await
            .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;
    }

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO provider_configs (provider_name, enabled, api_key, model, base_url, updated_at)
        VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&provider_name)
    .bind(if config.enabled { 1 } else { 0 })
    .bind(&encrypted_key)
    .bind(&model)
    .bind(&base_url)
    .execute(&mut *tx)
    .await
    .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    // Emit event to notify about config change
    app.emit("provider-config-changed", &provider_name)
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    Ok(())
}

/// 保存单个语音服务商配置
#[tauri::command]
pub async fn save_speech_provider_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: SpeechProviderConfig,
) -> Result<(), String> {
    let provider_name = config.provider_name.trim().to_string();
    if provider_name.is_empty() {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_NAME_EMPTY,
            "语音服务商名称不能为空",
        ));
    }

    let encrypted_key = if !config.api_key.is_empty() {
        encrypt_api_key(&config.api_key)?
    } else {
        String::new()
    };

    let model = if config.model.trim().is_empty() {
        providers::speech_default_model_for_provider(&provider_name)
            .unwrap_or("")
            .to_string()
    } else {
        config.model.trim().to_string()
    };

    let base_url = if let Some(base) = config.base_url {
        let trimmed = base.trim().to_string();
        if trimmed.is_empty() {
            providers::speech_default_base_url_for_provider(&provider_name).map(|v| v.to_string())
        } else {
            Some(trimmed)
        }
    } else {
        providers::speech_default_base_url_for_provider(&provider_name).map(|v| v.to_string())
    };

    let voice = if config.voice.trim().is_empty() {
        providers::speech_default_voice_for_provider(&provider_name)
            .unwrap_or(providers::DEFAULT_XIAOMI_TTS_VOICE)
            .to_string()
    } else {
        config.voice.trim().to_string()
    };

    let audio_format = normalize_audio_format(if config.audio_format.trim().is_empty() {
        providers::speech_default_audio_format_for_provider(&provider_name)
            .unwrap_or(providers::DEFAULT_TTS_AUDIO_FORMAT)
    } else {
        config.audio_format.as_str()
    });

    if config.enabled {
        validate_speech_provider_enable_config(
            &provider_name,
            &config.api_key,
            &model,
            base_url.as_deref(),
            &voice,
            &audio_format,
        )?;
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    if config.enabled {
        disable_other_speech_enabled(&mut *tx, &provider_name)
            .await
            .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;
    }

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO speech_provider_configs
        (provider_name, enabled, api_key, model, base_url, voice, audio_format, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&provider_name)
    .bind(if config.enabled { 1 } else { 0 })
    .bind(&encrypted_key)
    .bind(&model)
    .bind(&base_url)
    .bind(&voice)
    .bind(&audio_format)
    .execute(&mut *tx)
    .await
    .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    app.emit("speech-provider-config-changed", &provider_name)
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    Ok(())
}

/// 测试服务商 API（返回原始响应元数据）
#[tauri::command]
pub async fn test_provider(config: ProviderConfig) -> Result<ApiTestResponse, String> {
    let provider_name = config.provider_name.trim().to_string();
    if provider_name.is_empty() {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_NAME_EMPTY,
            "服务商名称不能为空",
        ));
    }

    let model = if config.model.trim().is_empty() {
        providers::default_model_for_provider(&provider_name)
            .unwrap_or("")
            .to_string()
    } else {
        config.model.trim().to_string()
    };

    let base_url = if let Some(base) = config.base_url {
        let trimmed = base.trim().to_string();
        if trimmed.is_empty() {
            providers::default_base_url_for_provider(&provider_name).map(|v| v.to_string())
        } else {
            Some(trimmed)
        }
    } else {
        providers::default_base_url_for_provider(&provider_name).map(|v| v.to_string())
    };

    let provider_config = ProviderConfig {
        provider_name: provider_name.clone(),
        enabled: true,
        api_key: config.api_key.clone(),
        model,
        base_url,
    };

    let mut response = match provider_name.as_str() {
        "zhipu" => {
            let provider = ZhipuProvider::new(&provider_config);
            provider.test_api().await
        }
        "ollama" => {
            let provider = OllamaProvider::new(&provider_config);
            provider.test_api().await
        }
        "claude" => {
            let provider = ClaudeProvider::new(&provider_config);
            provider.test_api().await
        }
        _ => {
            let provider = OpenAIProvider::new(&provider_config);
            provider.test_api().await
        }
    };
    response.provider = provider_name;

    Ok(response)
}

/// 测试语音服务商 API（返回原始响应元数据）
#[tauri::command]
pub async fn test_speech_provider(config: SpeechProviderConfig) -> Result<ApiTestResponse, String> {
    let provider_name = config.provider_name.trim().to_string();
    if provider_name.is_empty() {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_NAME_EMPTY,
            "语音服务商名称不能为空",
        ));
    }

    let model = if config.model.trim().is_empty() {
        providers::speech_default_model_for_provider(&provider_name)
            .unwrap_or("")
            .to_string()
    } else {
        config.model.trim().to_string()
    };

    let base_url = if let Some(base) = config.base_url {
        let trimmed = base.trim().to_string();
        if trimmed.is_empty() {
            providers::speech_default_base_url_for_provider(&provider_name).map(|v| v.to_string())
        } else {
            Some(trimmed)
        }
    } else {
        providers::speech_default_base_url_for_provider(&provider_name).map(|v| v.to_string())
    };

    let voice = if config.voice.trim().is_empty() {
        providers::speech_default_voice_for_provider(&provider_name)
            .unwrap_or(providers::DEFAULT_XIAOMI_TTS_VOICE)
            .to_string()
    } else {
        config.voice.trim().to_string()
    };

    let audio_format = normalize_audio_format(if config.audio_format.trim().is_empty() {
        providers::speech_default_audio_format_for_provider(&provider_name)
            .unwrap_or(providers::DEFAULT_TTS_AUDIO_FORMAT)
    } else {
        config.audio_format.as_str()
    });

    let provider_config = SpeechProviderConfig {
        provider_name: provider_name.clone(),
        enabled: true,
        api_key: config.api_key.clone(),
        model: model.clone(),
        base_url,
        voice,
        audio_format,
    };

    let response =
        speech::synthesize_openai_speech(&provider_config, "你好，这是一条语音合成连通性测试。")
            .await;

    Ok(ApiTestResponse {
        success: response.success,
        status_code: response.status_code,
        response_time_ms: response.response_time_ms,
        raw_response: response.raw_response,
        request_payload: Some(response.request_payload),
        error: response.error,
        provider: provider_name,
        model,
    })
}

/// 创建自定义服务商（OpenAI 协议）
#[tauri::command]
pub async fn create_custom_provider(
    app: AppHandle,
    state: State<'_, AppState>,
    #[allow(non_snake_case)] providerName: String,
    model: Option<String>,
    #[allow(non_snake_case)] baseUrl: Option<String>,
) -> Result<(), String> {
    let provider_name = providerName.trim();
    if provider_name.is_empty() {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_NAME_EMPTY,
            "服务商名称不能为空",
        ));
    }
    if providers::is_preset_provider(provider_name) {
        return Err(error_codes::with_code(
            error_codes::PRESET_PROVIDER_IMMUTABLE,
            "该名称为系统预置服务商，请更换名称",
        ));
    }

    let exists: Option<(String,)> = sqlx::query_as(
        "SELECT provider_name FROM provider_configs WHERE provider_name = ? LIMIT 1",
    )
    .bind(provider_name)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;
    if exists.is_some() {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_ALREADY_EXISTS,
            "服务商名称已存在",
        ));
    }

    let model = model
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_default();

    let base_url = baseUrl
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    sqlx::query(
        r#"
        INSERT INTO provider_configs (provider_name, enabled, api_key, model, base_url, updated_at)
        VALUES (?, 0, '', ?, ?, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(provider_name)
    .bind(model)
    .bind(base_url)
    .execute(&state.db)
    .await
    .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    app.emit("provider-config-changed", provider_name)
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;
    Ok(())
}

/// 删除自定义服务商
#[tauri::command]
pub async fn delete_custom_provider(
    app: AppHandle,
    state: State<'_, AppState>,
    #[allow(non_snake_case)] providerName: String,
) -> Result<(), String> {
    let provider_name = providerName.trim();
    if provider_name.is_empty() {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_NAME_EMPTY,
            "服务商名称不能为空",
        ));
    }
    if providers::is_preset_provider(provider_name) {
        return Err(error_codes::with_code(
            error_codes::PRESET_PROVIDER_IMMUTABLE,
            "预置服务商不支持删除",
        ));
    }

    let result = sqlx::query("DELETE FROM provider_configs WHERE provider_name = ?")
        .bind(provider_name)
        .execute(&state.db)
        .await
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(error_codes::with_code(
            error_codes::PROVIDER_NOT_FOUND,
            "服务商不存在",
        ));
    }

    app.emit("provider-config-changed", provider_name)
        .map_err(|e| error_codes::with_code(error_codes::DB_OPERATION_FAILED, e.to_string()))?;
    Ok(())
}

/// 快捷键配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HotkeyConfig {
    /// 是否启用双击复制翻译
    pub double_copy_enabled: bool,
    /// 是否启用 Alt+Space 打开窗口
    pub alt_space_enabled: bool,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            double_copy_enabled: true,
            alt_space_enabled: true,
        }
    }
}

/// 获取快捷键配置
#[tauri::command]
pub async fn get_hotkey_config(state: State<'_, AppState>) -> Result<HotkeyConfig, String> {
    let double_copy = get_val(&state.db, "hotkey_double_copy")
        .await
        .map(|v| v == "true")
        .unwrap_or(true);
    let alt_space = get_val(&state.db, "hotkey_alt_space")
        .await
        .map(|v| v == "true")
        .unwrap_or(true);

    Ok(HotkeyConfig {
        double_copy_enabled: double_copy,
        alt_space_enabled: alt_space,
    })
}

/// 保存快捷键配置
#[tauri::command]
pub async fn save_hotkey_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: HotkeyConfig,
) -> Result<(), String> {
    let queries = [
        ("hotkey_double_copy", config.double_copy_enabled.to_string()),
        ("hotkey_alt_space", config.alt_space_enabled.to_string()),
    ];

    for (key, value) in queries {
        sqlx::query("INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)")
            .bind(key)
            .bind(value)
            .execute(&state.db)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 更新 AppState 中的热键配置
    if let Ok(mut guard) = state.hotkey_double_copy_enabled.lock() {
        *guard = config.double_copy_enabled;
    }
    if let Ok(mut guard) = state.hotkey_alt_space_enabled.lock() {
        *guard = config.alt_space_enabled;
    }

    // 发送事件通知
    app.emit("hotkey-config-changed", &config)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localhost_base_url_detection_works() {
        assert!(is_localhost_base_url("http://localhost:11434/api/chat"));
        assert!(is_localhost_base_url(
            "http://127.0.0.1:8080/v1/chat/completions"
        ));
        assert!(is_localhost_base_url(
            "http://[::1]:8080/v1/chat/completions"
        ));
        assert!(!is_localhost_base_url(
            "https://api.openai.com/v1/chat/completions"
        ));
    }

    #[test]
    fn validate_provider_enable_config_returns_coded_error_for_missing_model() {
        let err = validate_provider_enable_config(
            "openai",
            "sk-test",
            "",
            Some("https://api.openai.com/v1/chat/completions"),
        )
        .expect_err("expected validation error");

        assert!(err.starts_with("[INVALID_PROVIDER_CONFIG]"));
    }

    #[test]
    fn validate_provider_enable_config_allows_empty_api_key_for_localhost() {
        let result = validate_provider_enable_config(
            "openai",
            "",
            "gpt-4.1-mini",
            Some("http://localhost:11434/v1/chat/completions"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn validate_provider_enable_config_rejects_empty_api_key_for_remote() {
        let err = validate_provider_enable_config(
            "openai",
            "",
            "gpt-4.1-mini",
            Some("https://api.openai.com/v1/chat/completions"),
        )
        .expect_err("expected validation error");

        assert!(err.starts_with("[INVALID_PROVIDER_CONFIG]"));
    }

    #[tokio::test]
    async fn disable_other_enabled_keeps_single_provider_enabled() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect sqlite memory");

        sqlx::query(
            r#"
            CREATE TABLE provider_configs (
                provider_name TEXT PRIMARY KEY,
                enabled INTEGER NOT NULL DEFAULT 0,
                api_key TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL DEFAULT '',
                base_url TEXT,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create provider_configs");

        sqlx::query("INSERT INTO provider_configs (provider_name, enabled) VALUES ('zhipu', 1)")
            .execute(&pool)
            .await
            .expect("insert zhipu");
        sqlx::query("INSERT INTO provider_configs (provider_name, enabled) VALUES ('openai', 1)")
            .execute(&pool)
            .await
            .expect("insert openai");

        disable_other_enabled(&pool, "openai")
            .await
            .expect("disable others");

        let openai_enabled: i64 =
            sqlx::query_scalar("SELECT enabled FROM provider_configs WHERE provider_name='openai'")
                .fetch_one(&pool)
                .await
                .expect("query openai enabled");
        let zhipu_enabled: i64 =
            sqlx::query_scalar("SELECT enabled FROM provider_configs WHERE provider_name='zhipu'")
                .fetch_one(&pool)
                .await
                .expect("query zhipu enabled");

        assert_eq!(openai_enabled, 1);
        assert_eq!(zhipu_enabled, 0);
    }
}
