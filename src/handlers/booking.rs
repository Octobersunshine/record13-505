use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    Booking, CreateBookingRequest, Session, BOOKING_STATUS_CANCELLED,
    BOOKING_STATUS_CONFIRMED,
};
use crate::state::AppState;

pub async fn create_booking(
    State(state): State<AppState>,
    Json(req): Json<CreateBookingRequest>,
) -> Result<(StatusCode, Json<Value>), AppError> {
    let session = sqlx::query_as::<_, Session>(
        "SELECT * FROM sessions WHERE id = ?",
    )
    .bind(&req.session_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::SessionNotFound(req.session_id.clone()))?;

    if session.available_quota <= 0 {
        return Err(AppError::QuotaExhausted);
    }

    let existing = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE session_id = ? AND user_id = ?",
    )
    .bind(&req.session_id)
    .bind(&req.user_id)
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::DuplicateBooking {
            user_id: req.user_id.clone(),
            session_id: req.session_id.clone(),
        });
    }

    let booking_id = Uuid::new_v4().to_string();

    let mut tx = state.pool.begin().await?;

    let updated = sqlx::query(
        r#"
        UPDATE sessions
        SET available_quota = available_quota - 1,
            updated_at = datetime('now')
        WHERE id = ? AND available_quota > 0
        "#,
    )
    .bind(&req.session_id)
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::QuotaExhausted);
    }

    let booking = sqlx::query_as::<_, Booking>(
        r#"
        INSERT INTO bookings (id, session_id, user_id, status)
        VALUES (?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&booking_id)
    .bind(&req.session_id)
    .bind(&req.user_id)
    .bind(BOOKING_STATUS_CONFIRMED)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "code": 201,
            "message": "预约成功",
            "data": {
                "id": booking.id,
                "session_id": booking.session_id,
                "user_id": booking.user_id,
                "status": booking.status,
            }
        })),
    ))
}

pub async fn cancel_booking(
    State(state): State<AppState>,
    Path(booking_id): Path<String>,
) -> Result<Json<Value>, AppError> {
    let booking = sqlx::query_as::<_, Booking>(
        "SELECT * FROM bookings WHERE id = ?",
    )
    .bind(&booking_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::BookingNotFound(booking_id.clone()))?;

    if booking.status == BOOKING_STATUS_CANCELLED {
        return Err(AppError::BookingAlreadyCancelled);
    }

    let mut tx = state.pool.begin().await?;

    let booking_updated = sqlx::query(
        r#"
        UPDATE bookings
        SET status = ?,
            updated_at = datetime('now')
        WHERE id = ? AND status = ?
        "#,
    )
    .bind(BOOKING_STATUS_CANCELLED)
    .bind(&booking_id)
    .bind(BOOKING_STATUS_CONFIRMED)
    .execute(&mut *tx)
    .await?;

    if booking_updated.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::BookingAlreadyCancelled);
    }

    let quota_updated = sqlx::query(
        r#"
        UPDATE sessions
        SET available_quota = available_quota + 1,
            updated_at = datetime('now')
        WHERE id = ? AND available_quota < total_quota
        "#,
    )
    .bind(&booking.session_id)
    .execute(&mut *tx)
    .await?;

    if quota_updated.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(AppError::Internal("名额返还失败，名额已达上限".into()));
    }

    tx.commit().await?;

    Ok(Json(json!({
        "code": 200,
        "message": "预约已取消，名额已返还",
        "data": {
            "id": booking.id,
            "session_id": booking.session_id,
            "status": BOOKING_STATUS_CANCELLED
        }
    })))
}
