// src-tauri/src/utils/error.rs
#[allow(dead_code)] // Allow unused type alias
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[allow(dead_code)] // Allow unused struct
#[derive(Debug)]
pub struct AppError(pub String);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}