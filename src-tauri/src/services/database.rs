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
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

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

    Ok(pool)
}