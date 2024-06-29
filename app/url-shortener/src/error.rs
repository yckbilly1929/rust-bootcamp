use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("bind json error: {0}")]
    BindJsonError(#[from] JsonRejection),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("url already exists: {0}")]
    UrlDuplicated(String),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("unknown error: {0}")]
    UnknownError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    // pub code: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match &self {
            Self::BindJsonError(_) => StatusCode::BAD_REQUEST,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::UrlDuplicated(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnknownError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(ErrorResponse::new(self.to_string()))).into_response()
    }
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            message: error.into(), // TODO: trim crlf and space
        }
    }
}
