use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::fs;
use tauri::{AppHandle, Manager};

pub async fn initialize_db(app: &AppHandle) -> Result<SqlitePool, String> {
    crate::app_info!("[DB] 开始初始化数据库...");

    // 1. 获取应用数据目录
    let app_data_dir = app.path().app_data_dir().map_err(|e| {
        crate::app_error!("[DB] 获取应用数据目录失败: {}", e);
        e.to_string()
    })?;
    crate::app_info!("[DB] 应用数据目录: {:?}", app_data_dir);

    // 2. 确保目录存在
    if !app_data_dir.exists() {
        crate::app_info!("[DB] 目录不存在，正在创建...");
        fs::create_dir_all(&app_data_dir).map_err(|e| {
            crate::app_error!("[DB] 创建目录失败: {}", e);
            e.to_string()
        })?;
        crate::app_info!("[DB] 目录创建成功");
    } else {
        crate::app_info!("[DB] 目录已存在");
    }

    // 3. 数据库文件路径
    let db_path = app_data_dir.join("ace_translator.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());
    crate::app_info!("[DB] 数据库路径: {:?}", db_path);
    crate::app_info!("[DB] 数据库URL: {}", db_url);

    // 4. 如果数据库不存在则创建
    let db_exists = Sqlite::database_exists(&db_url).await.unwrap_or(false);
    crate::app_info!("[DB] 数据库是否存在: {}", db_exists);

    if !db_exists {
        crate::app_info!("[DB] 正在创建数据库...");
        Sqlite::create_database(&db_url)
            .await
            .map_err(|e| {
                crate::app_error!("[DB] 创建数据库失败: {}", e);
                e.to_string()
            })?;
        crate::app_info!("[DB] 数据库创建成功");
    }

    // 5. 连接数据库
    crate::app_info!("[DB] 正在连接数据库...");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| {
            crate::app_error!("[DB] 连接数据库失败: {}", e);
            e.to_string()
        })?;
    crate::app_info!("[DB] 数据库连接成功");

    // 6. 创建表
    crate::app_info!("[DB] 正在创建 translation_history 表...");
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
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        crate::app_error!("[DB] 创建 translation_history 表失败: {}", e);
        e.to_string()
    })?;
    crate::app_info!("[DB] translation_history 表创建成功");

    // 6.1 数据库迁移：添加 provider 和 model 列（如果不存在）
    #[derive(sqlx::FromRow)]
    struct ColumnInfo {
        name: String,
    }

    let columns: Vec<ColumnInfo> =
        sqlx::query_as("SELECT name FROM pragma_table_info('translation_history')")
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
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // 7. 创建服务商配置表
    crate::app_info!("[DB] 正在创建 provider_configs 表...");
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
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        crate::app_error!("[DB] 创建 provider_configs 表失败: {}", e);
        e.to_string()
    })?;
    crate::app_info!("[DB] provider_configs 表创建成功");

    // 8. 初始化默认服务商配置（如果不存在）
    crate::app_info!("[DB] 正在初始化服务商配置...");
    let providers = ["zhipu", "openai", "claude", "ollama"];
    for provider in providers {
        let result = sqlx::query(
            r#"
            INSERT OR IGNORE INTO provider_configs (provider_name, enabled, api_key, model, base_url)
            VALUES (?, ?, '', '', NULL)
            "#
        )
        .bind(provider)
        .bind(0)
        .execute(&pool)
        .await
        .map_err(|e| {
            crate::app_error!("[DB] 插入服务商 {} 配置失败: {}", provider, e);
            e.to_string()
        })?;
        crate::app_info!("[DB] 服务商 {} 初始化完成, rows_affected: {}", provider, result.rows_affected());
    }

    // 验证数据是否插入成功
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM provider_configs")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            crate::app_error!("[DB] 查询 provider_configs 数量失败: {}", e);
            e.to_string()
        })?;
    crate::app_info!("[DB] provider_configs 表中共有 {} 条记录", count.0);

    // 查询 zhipu 是否存在
    let zhipu_exists: Option<(String,)> = sqlx::query_as(
        "SELECT provider_name FROM provider_configs WHERE provider_name = 'zhipu'"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        crate::app_error!("[DB] 查询 zhipu 配置失败: {}", e);
        e.to_string()
    })?;
    crate::app_info!("[DB] zhipu 配置是否存在: {}", zhipu_exists.is_some());

    crate::app_info!("[DB] 数据库初始化完成!");
    Ok(pool)
}
