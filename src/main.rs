use std::{net::ToSocketAddrs, process};

use axum::{routing::get, Router};
use serde::Deserialize;
use tokio::net::TcpListener;
use tracing::*;

mod handlers;

#[tokio::main]
async fn main() {
    // 应用配置的结构体
    #[derive(Deserialize)]
    struct Settings {
        server: Server,
        postgres: Postgres,
    }
    #[derive(Deserialize)]
    struct Server {
        listen_addr: String,
        listen_port: u16,
    }
    #[derive(Deserialize)]
    struct Postgres {
        uri: String,
    }

    // 初始化应用配置
    let settings = match config::Config::builder()
        .add_source(config::File::new("config.toml", config::FileFormat::Toml))
        .build()
    {
        Ok(settings) => settings.try_deserialize::<Settings>().unwrap(),
        Err(e) => {
            error!("Failed to read config: {}", e);
            process::exit(1);
        }
    };

    // 初始化 Tracing 收集器
    tracing_subscriber::fmt::init();

    // 连接数据库
    let db_pool = match sqlx::postgres::PgPoolOptions::new()
        .connect(&settings.postgres.uri)
        .await
    {
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

    // 解析 Socket 地址
    let socket_addr = match format!(
        "{}:{}",
        settings.server.listen_addr, settings.server.listen_port
    )
    .to_socket_addrs()
    {
        Ok(mut addr) => addr.next().unwrap(),
        Err(e) => {
            error!("Failed to parse listen address: {}", e);
            process::exit(1);
        }
    };
    // 创建监听器
    let listener = TcpListener::bind(socket_addr).await.unwrap();

    // 启动服务
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
