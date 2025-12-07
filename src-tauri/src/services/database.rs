use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::fs;
use tauri::{AppHandle, Manager};

pub async fn initialize_db(app: &AppHandle) -> Result<SqlitePool, String> {
    // 1. 获取应用数据目录
    // 在 Tauri v2 中，可能需要通过 path 插件或 resolver 获取。
    // 这里假设已经在 tauri.conf.json 或 capabilities 中配置了 path 权限。
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // 2. 确保目录存在
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    }

    // 3. 数据库文件路径
    let db_path = app_data_dir.join("ace_translator.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    // 4. 如果数据库不存在则创建
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url)
            .await
            .map_err(|e| e.to_string())?;
    }

    // 5. 连接数据库
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| e.to_string())?;

    // 6. 创建表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS translation_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_text TEXT NOT NULL,
            translated_text TEXT NOT NULL,
            source_lang TEXT NOT NULL,
            target_lang TEXT NOT NULL,
            provider TEXT,
            model TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 6.1 数据库迁移：添加 provider 和 model 列（如果不存在）
    #[derive(sqlx::FromRow)]
    struct ColumnInfo {
        name: String,
    }

    let columns: Vec<ColumnInfo> = sqlx::query_as(
        "SELECT name FROM pragma_table_info('translation_history')"
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let column_names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();

    if !column_names.iter().any(|c| *c == "provider") {
        sqlx::query("ALTER TABLE translation_history ADD COLUMN provider TEXT")
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    if !column_names.iter().any(|c| *c == "model") {
        sqlx::query("ALTER TABLE translation_history ADD COLUMN model TEXT")
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 7. 创建服务商配置表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS provider_configs (
            provider_name TEXT PRIMARY KEY,
            enabled INTEGER NOT NULL DEFAULT 0,
            api_key TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            base_url TEXT,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 8. 初始化默认服务商配置（如果不存在）
    let providers = ["zhipu", "openai", "claude", "ollama"];
    for provider in providers {
        let enabled = if provider == "zhipu" { 1 } else { 0 };
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO provider_configs (provider_name, enabled, api_key, model, base_url)
            VALUES (?, ?, '', '', NULL)
            "#
        )
        .bind(provider)
        .bind(enabled)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(pool)
}