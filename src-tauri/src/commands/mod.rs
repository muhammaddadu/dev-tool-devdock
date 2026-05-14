pub mod ai;
pub mod ports;
pub mod projects;
pub mod services;
pub mod settings;

use serde::Serialize;

/// Error type returned from Tauri commands. Serializes as a plain string for the frontend.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Other(err.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
