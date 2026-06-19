use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{CreateSessionRequest, Session};
use crate::state::AppState;

pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let id = Uuid::new_v4().to_string();
    let is_hot = if req.is_hot { 1 } else { 0 };

    sqlx::query_as::<_, Session>(
        r#"
        INSERT INTO sessions (id, title, total_quota, available_quota, start_time, end_time, is_hot)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(&req.title)
    .bind(req.total_quota)
    .bind(req.total_quota)
    .bind(&req.start_time)
    .bind(&req.end_time)
    .bind(is_hot)
    .fetch_one(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "code": 201,
            "message": "场次创建成功",
            "data": { "id": id }
        })),
    ))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::SessionNotFound(id.clone()))?;

    Ok(Json(json!({
        "code": 200,
        "message": "success",
        "data": session
    })))
}
