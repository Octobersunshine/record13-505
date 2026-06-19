use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("场次不存在: {0}")]
    SessionNotFound(String),

    #[error("名额不足")]
    QuotaExhausted,

    #[error("重复预约: 用户 {user_id} 已预约场次 {session_id}")]
    DuplicateBooking { user_id: String, session_id: String },

    #[error("场次已结束，无法预约")]
    SessionEnded,

    #[error("预约不存在: {0}")]
    BookingNotFound(String),

    #[error("预约已取消，无法重复取消")]
    BookingAlreadyCancelled,

    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::SessionNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BookingNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::QuotaExhausted => (StatusCode::CONFLICT, self.to_string()),
            AppError::DuplicateBooking { .. } => (StatusCode::CONFLICT, self.to_string()),
            AppError::BookingAlreadyCancelled => (StatusCode::CONFLICT, self.to_string()),
            AppError::SessionEnded => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Database(e) => {
                tracing::error!("数据库错误: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误".to_string())
            }
            AppError::Internal(e) => {
                tracing::error!("内部错误: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误".to_string())
            }
        };

        let body = json!({
            "code": status.as_u16(),
            "message": message,
        });

        (status, axum::Json(body)).into_response()
    }
}
