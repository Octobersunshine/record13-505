use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub total_quota: i64,
    pub available_quota: i64,
    pub start_time: String,
    pub end_time: String,
    pub is_hot: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub title: String,
    pub total_quota: i64,
    pub start_time: String,
    pub end_time: String,
    #[serde(default)]
    pub is_hot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Booking {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserBookingStats {
    pub user_id: String,
    pub no_show_count: i64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookingRequest {
    pub session_id: String,
    pub user_id: String,
}

pub const BOOKING_STATUS_CONFIRMED: &str = "confirmed";
pub const BOOKING_STATUS_CANCELLED: &str = "cancelled";
pub const BOOKING_STATUS_NO_SHOW: &str = "no_show";

pub const MAX_NO_SHOW_FOR_HOT_SESSION: i64 = 2;


