use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub struct AppError(pub StatusCode, pub String);

impl AppError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self(StatusCode::UNPROCESSABLE_ENTITY, message.into())
    }

    pub fn unauthorized() -> Self {
        Self(
            StatusCode::UNAUTHORIZED,
            "Please log in to continue.".into(),
        )
    }

    pub fn not_found() -> Self {
        Self(StatusCode::NOT_FOUND, "That record was not found.".into())
    }

    pub fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "request failed");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong. Please try again.".into(),
        )
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::internal(error)
    }
}

impl From<tower_sessions::session::Error> for AppError {
    fn from(error: tower_sessions::session::Error) -> Self {
        Self::internal(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
