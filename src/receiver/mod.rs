/// HTTP接收端 - 接收Chrome插件推送的实时数据
pub mod handler;
pub mod models;

use crate::receiver::handler::AppState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use axum::http::Method;

/// 启动HTTP接收服务
pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let conn = crate::db::open_db()?;
    // 启用WAL模式减少写锁冲突
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    let state = Arc::new(AppState {
        db: Mutex::new(conn),
        start_time: std::time::Instant::now(),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/v1/quote", post(handler::push_quote))
        .route("/api/v1/kline", post(handler::push_kline))
        .route("/api/v1/tick", post(handler::push_tick))
        .route("/api/v1/timeshare", post(handler::push_timeshare))
        .route("/api/v1/orderbook", post(handler::push_orderbook))
        .route("/api/v1/moneyflow", post(handler::push_moneyflow))
        .route("/api/v1/status", get(handler::get_status))
        .route("/api/v1/watchlist", get(handler::get_watchlist))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("astock 数据接收端已启动: http://127.0.0.1:{}", port);
    println!("等待Chrome插件推送数据...");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
