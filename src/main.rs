mod db;
mod error;
mod handlers;
mod models;
mod state;

use axum::routing::{delete, get, post, put};
use axum::Router;
use tracing_subscriber::EnvFilter;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let pool = db::create_pool()
        .await
        .expect("无法连接数据库");

    tracing::info!("数据库初始化完成");

    let state = AppState { pool };

    let app = Router::new()
        .route("/sessions", post(handlers::session::create_session))
        .route("/sessions/{id}", get(handlers::session::get_session))
        .route("/bookings", post(handlers::booking::create_booking))
        .route("/bookings/{id}", delete(handlers::booking::cancel_booking))
        .route(
            "/bookings/{id}/no-show",
            put(handlers::booking::mark_no_show),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("无法绑定端口 3000");

    tracing::info!("服务启动在 http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("服务启动失败");
}
