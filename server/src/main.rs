// TODO: remove unused crates in cargo.tom

use std::{process, sync::Arc};

use dotenvy::dotenv;
use infra::{cache::get_redis_client, db::create_pool, logging::init_logger};
use tokio::net::TcpListener;

use crate::{
    constants::{SERVER_RUNNING_ADDRESS, SERVER_RUNNING_PORT},
    handlers::{AppState, init_app},
};

mod constants;
mod handlers;

#[tokio::main]
async fn main() {
    dotenv().ok();

    init_logger();

    let db_pool = Arc::new(create_pool().await.unwrap_or_else(|error| {
        log::error!("failed to create db pool: {error}");
        process::exit(1)
    }));
    let redis_client = get_redis_client().unwrap_or_else(|error| {
        log::error!("failed to create redis client: {error}");
        process::exit(1)
    });

    let app_state = AppState::new(db_pool, redis_client);
    let app = init_app(app_state).await;

    let addr = format!("{}:{}", SERVER_RUNNING_ADDRESS, SERVER_RUNNING_PORT);
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|error| {
        log::error!("failed to start listener: {error}");
        process::exit(1)
    });

    axum::serve(listener, app).await.unwrap_or_else(|error| {
        log::error!("failed to start server: {error}");
        std::process::exit(1)
    })
}
