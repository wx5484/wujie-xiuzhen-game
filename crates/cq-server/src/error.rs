use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use cq_protocol::rest::ApiErrorBody;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "请先登录".into()),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "资源不存在".into()),
            Self::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            Self::Database(err) => {
                tracing::error!(error = ?err, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "database_error", "数据库暂时不可用".into())
            }
            Self::Internal(message) => {
                tracing::error!(message, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "服务暂时不可用".into())
            }
        };
        (status, Json(ApiErrorBody::new(code, message))).into_response()
    }
}
