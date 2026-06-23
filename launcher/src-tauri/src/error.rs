//! Tipo de error unificado del backend.
//!
//! Implementa `Serialize` (a string) para poder devolverse directamente desde
//! comandos Tauri como `Result<T, AppError>`.

use serde::{Deserialize, Serialize, Serializer};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("E/S: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Red: {0}")]
    Http(#[from] reqwest::Error),

    #[error("ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("{0}")]
    Msg(String),

    #[error("structured error")]
    Structured(StructuredError),
}

/// Error estructurado para que la UI reaccione (p. ej. CF distribution blocked).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
}

impl AppError {
    pub fn msg(s: impl Into<String>) -> Self {
        AppError::Msg(s.into())
    }

    pub fn structured(err: StructuredError) -> Self {
        AppError::Structured(err)
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AppError::Structured(s) => s.serialize(serializer),
            other => serializer.serialize_str(&other.to_string()),
        }
    }
}
