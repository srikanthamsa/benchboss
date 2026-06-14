use serde::Serialize;
use std::fmt;

/// A serialisable error type for Tauri commands.
#[derive(Debug, Serialize)]
pub struct AppError(String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError(e.to_string())
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError(s.to_owned())
    }
}

pub type AppResult<T> = Result<T, AppError>;
