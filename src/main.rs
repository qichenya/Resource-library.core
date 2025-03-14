use std::{net::SocketAddr, process};

use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing::*;

mod handlers;

#[tokio::main]
async fn main() {
    // 初始化 Tracing 收集器
    tracing_subscriber::fmt::init();

    // 连接数据库
    let db_url = "";
    let db_pool = match sqlx::postgres::PgPoolOptions::new().connect(db_url).await {
        Ok(pool) => {
            info!("Database connection successful");
            pool
        }
        Err(e) => {
            error!("Database connection failed: {}", e);
            process::exit(1);
        }
    };

    // 定义 HTTP 服务
    let app = Router::new()
        .route("/", get(handlers::root))
        .route(
            "/get_all_resources",
            get(handlers::get_all_resources::handler),
        )
        .with_state(db_pool);

    // 创建监听器
    let addr = SocketAddr::from(([0, 0, 0, 0], 8352));
    let listener = TcpListener::bind(addr).await.unwrap();

    // 启动服务
    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
