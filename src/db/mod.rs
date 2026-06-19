use sqlx::SqlitePool;

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            total_quota INTEGER NOT NULL,
            available_quota INTEGER NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            is_hot INTEGER NOT NULL DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    let is_hot_exists: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM pragma_table_info('sessions') WHERE name = 'is_hot'",
    )
    .fetch_optional(pool)
    .await?;
    if is_hot_exists.is_none() {
        sqlx::query("ALTER TABLE sessions ADD COLUMN is_hot INTEGER NOT NULL DEFAULT 0")
            .execute(pool)
            .await?;
    }

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS bookings (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'confirmed',
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (session_id) REFERENCES sessions(id),
            UNIQUE(session_id, user_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_booking_stats (
            user_id TEXT PRIMARY KEY,
            no_show_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn create_pool() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect("sqlite:booking.db?mode=rwc").await?;
    init_db(&pool).await?;
    Ok(pool)
}

