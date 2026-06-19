use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Booking, CreateBookingRequest, Session};
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
        VALUES (?, ?, ?, 'confirmed')
        RETURNING *
        "#,
    )
    .bind(&booking_id)
    .bind(&req.session_id)
    .bind(&req.user_id)
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
