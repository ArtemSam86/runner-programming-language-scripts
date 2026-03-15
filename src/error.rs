use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Script '{0}' not found")]
    ScriptNotFound(String),
    #[error("Script name invalid: {0}")]
    InvalidScriptName(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("MongoDB error: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("Script execution timed out")]
    Timeout,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AppError::ScriptNotFound(name) => (
                StatusCode::NOT_FOUND,
                format!("Script '{}' not found", name),
            ),
            AppError::InvalidScriptName(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Io(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO error: {}", e),
            ),
            AppError::Json(e) => (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)),
            AppError::Utf8(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("UTF-8 error: {}", e),
            ),
            AppError::Mongo(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            AppError::Timeout => (
                StatusCode::GATEWAY_TIMEOUT,
                "Script execution timed out".to_string(),
            ),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::UserAlreadyExists(msg) => (StatusCode::CONFLICT, msg),
        };
        (status, msg).into_response()
    }
}