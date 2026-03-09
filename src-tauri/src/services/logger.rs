use chrono::Local;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const MAX_LOG_ENTRIES: usize = 500;

#[derive(Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct Logger {
    logs: Mutex<VecDeque<LogEntry>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)),
        }
    }

    pub fn log(&self, level: &str, message: &str) {
        let entry = LogEntry {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            level: level.to_string(),
            message: message.to_string(),
        };

        // 同时打印到控制台
        println!("[{}] [{}] {}", entry.timestamp, entry.level, entry.message);

        if let Ok(mut logs) = self.logs.lock() {
            if logs.len() >= MAX_LOG_ENTRIES {
                logs.pop_front();
            }
            logs.push_back(entry);
        }
    }

    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    pub fn debug(&self, message: &str) {
        self.log("DEBUG", message);
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        if let Ok(logs) = self.logs.lock() {
            logs.iter().cloned().collect()
        } else {
            vec![]
        }
    }

    pub fn clear(&self) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
        }
    }
}

// 全局日志实例
lazy_static::lazy_static! {
    pub static ref LOGGER: Arc<Logger> = Arc::new(Logger::new());
}

// 便捷宏
#[macro_export]
macro_rules! app_log {
    ($level:expr, $($arg:tt)*) => {
        $crate::services::logger::LOGGER.log($level, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! app_info {
    ($($arg:tt)*) => {
        $crate::services::logger::LOGGER.info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! app_error {
    ($($arg:tt)*) => {
        $crate::services::logger::LOGGER.error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! app_debug {
    ($($arg:tt)*) => {
        $crate::services::logger::LOGGER.debug(&format!($($arg)*))
    };
}
