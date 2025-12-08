use crate::AppState;
use tauri::{AppHandle, State, Emitter};
use serde::{Deserialize, Serialize};
use crate::services::encryption::{encrypt_api_key, decrypt_api_key};
use crate::services::ai::{ProviderConfig, ApiTestResponse};
use crate::services::ai::zhipu::ZhipuProvider;
use crate::services::ai::openai::OpenAIProvider;
use crate::services::ai::claude::ClaudeProvider;
use crate::services::ai::ollama::{OllamaProvider, OLLAMA_MODELS};
use crate::services::ai::AIProvider;
use crate::config::providers;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub api_key: String,
    pub theme: String,
    /// 第一语言（默认翻译目标，如：中文）
    pub primary_target: String,
    /// 第二语言（当源语言是第一语言时使用，如：英文）
    pub secondary_target: String,
}

/// 服务商信息（包括可选模型）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub display_name: String,
    pub config: ProviderConfig,
    pub available_models: Vec<String>,
    pub supports_base_url: bool,
}

// Helper to get value from DB
async fn get_val(pool: &sqlx::SqlitePool, key: &str) -> Option<String> {
    sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, state: State<'_, AppState>, settings: AppSettings) -> Result<(), String> {
    // Encrypt API Key
    let encrypted_key = encrypt_api_key(&settings.api_key)?;

    // Save to DB
    let queries = [
        ("api_key", encrypted_key),
        ("theme", settings.theme.clone()),
        ("primary_target", settings.primary_target.clone()),
        ("secondary_target", settings.secondary_target.clone()),
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
    app.emit("settings-changed", &settings).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let encrypted_key = get_val(&state.db, "api_key").await.unwrap_or_default();
    let theme = get_val(&state.db, "theme").await.unwrap_or_else(|| "dark".to_string());
    let primary_target = get_val(&state.db, "primary_target").await.unwrap_or_else(|| "zh-CN".to_string());
    let secondary_target = get_val(&state.db, "secondary_target").await.unwrap_or_else(|| "en".to_string());

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
    })
}

/// 获取所有服务商配置
#[tauri::command]
pub async fn get_provider_configs(state: State<'_, AppState>) -> Result<Vec<ProviderInfo>, String> {
    #[derive(sqlx::FromRow)]
    struct DbProviderConfig {
        provider_name: String,
        enabled: i32,
        api_key: String,
        model: String,
        base_url: Option<String>,
    }

    let rows: Vec<DbProviderConfig> = sqlx::query_as(
        "SELECT provider_name, enabled, api_key, model, base_url FROM provider_configs ORDER BY provider_name"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let mut providers = Vec::new();
    for row in rows {
        let api_key = if !row.api_key.is_empty() {
            decrypt_api_key(&row.api_key).unwrap_or_default()
        } else {
            String::new()
        };

        let (display_name, available_models, supports_base_url, default_model) = match row.provider_name.as_str() {
            "zhipu" => (
                "智谱AI",
                providers::ZHIPU_MODELS.iter().map(|m| m.id.to_string()).collect(),
                false,
                providers::ZHIPU_MODELS.first().map(|m| m.id).unwrap_or("glm-4-flashx").to_string(),
            ),
            "openai" => (
                "OpenAI",
                providers::OPENAI_MODELS.iter().map(|m| m.id.to_string()).collect(),
                true,
                providers::OPENAI_MODELS.first().map(|m| m.id).unwrap_or("gpt-4o-mini").to_string(),
            ),
            "claude" => (
                "Claude",
                providers::CLAUDE_MODELS.iter().map(|m| m.id.to_string()).collect(),
                false,
                providers::CLAUDE_MODELS.first().map(|m| m.id).unwrap_or("claude-3-5-haiku-latest").to_string(),
            ),
            "ollama" => (
                "Ollama",
                OLLAMA_MODELS.iter().map(|s| s.to_string()).collect(),
                true,
                "llama3.2".to_string(),
            ),
            _ => continue,
        };

        // fallback to default model if DB value is empty (helps fresh installs)
        let model = if row.model.is_empty() { default_model.clone() } else { row.model.clone() };

        providers.push(ProviderInfo {
            name: row.provider_name.clone(),
            display_name: display_name.to_string(),
            config: ProviderConfig {
                provider_name: row.provider_name,
                enabled: row.enabled != 0,
                api_key,
                model,
                base_url: row.base_url,
            },
            available_models,
            supports_base_url,
        });
    }

    Ok(providers)
}

/// 保存单个服务商配置
#[tauri::command]
pub async fn save_provider_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: ProviderConfig
) -> Result<(), String> {
    let encrypted_key = if !config.api_key.is_empty() {
        encrypt_api_key(&config.api_key)?
    } else {
        String::new()
    };

    // Force enable Zhipu AI
    let enabled = if config.provider_name == "zhipu" {
        true
    } else {
        config.enabled
    };

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO provider_configs (provider_name, enabled, api_key, model, base_url, updated_at)
        VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
        "#
    )
    .bind(&config.provider_name)
    .bind(if enabled { 1 } else { 0 })
    .bind(&encrypted_key)
    .bind(&config.model)
    .bind(&config.base_url)
    .execute(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    // Emit event to notify about config change
    app.emit("provider-config-changed", &config.provider_name).map_err(|e| e.to_string())?;

    Ok(())
}

/// 测试服务商 API（返回原始响应元数据）
#[tauri::command]
pub async fn test_provider(config: ProviderConfig) -> Result<ApiTestResponse, String> {
    let provider_config = ProviderConfig {
        provider_name: config.provider_name.clone(),
        enabled: true,
        api_key: config.api_key.clone(),
        model: config.model.clone(),
        base_url: config.base_url.clone(),
    };

    let response = match config.provider_name.as_str() {
        "zhipu" => {
            let provider = ZhipuProvider::new(&provider_config);
            provider.test_api().await
        }
        "openai" => {
            let provider = OpenAIProvider::new(&provider_config);
            provider.test_api().await
        }
        "claude" => {
            let provider = ClaudeProvider::new(&provider_config);
            provider.test_api().await
        }
        "ollama" => {
            let provider = OllamaProvider::new(&provider_config);
            provider.test_api().await
        }
        _ => {
            return Err(format!("未知服务商: {}", config.provider_name));
        }
    };

    Ok(response)
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
    let double_copy = get_val(&state.db, "hotkey_double_copy").await
        .map(|v| v == "true")
        .unwrap_or(true);
    let alt_space = get_val(&state.db, "hotkey_alt_space").await
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
    config: HotkeyConfig
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
    app.emit("hotkey-config-changed", &config).map_err(|e| e.to_string())?;

    Ok(())
}
