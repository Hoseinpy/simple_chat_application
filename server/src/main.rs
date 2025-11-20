// TODO: remove unused crates in cargo.tom

use std::{process, sync::Arc};

use dotenvy::dotenv;
use infra::{cache::get_redis_client, db::create_pool, logging::init_logger};
use shared::models::AppState;
use tokio::net::TcpListener;

use crate::handlers::init_app;

mod handlers;
mod rate_limiter;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_logger();

    let db_pool = Arc::new(create_pool().await.unwrap_or_else(|error| {
        log::error!("failed to create db pool: {error}");
        process::exit(1)
    }));
    let redis_client = Arc::new(get_redis_client().unwrap_or_else(|error| {
        log::error!("failed to create redis client: {error}");
        process::exit(1)
    }));

    let app = init_app(AppState::new(db_pool, redis_client)).await;

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap_or_else(|error| {
            log::error!("failed to start listener: {error}");
            process::exit(1)
        });

    axum::serve(listener, app).await.unwrap_or_else(|error| {
        log::error!("failed to start server: {error}");
        std::process::exit(1)
    })
}
