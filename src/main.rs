use std::net::SocketAddr;

use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing::*;

mod handlers;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/", get(handlers::root));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8352));
    let listener = TcpListener::bind(addr).await.unwrap();

    info!("Listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
