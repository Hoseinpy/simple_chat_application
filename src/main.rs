use std::process;

use dotenvy::dotenv;
use tokio::net::TcpListener;

use crate::{
    constants::{SERVER_RUNNING_ADDRESS, SERVER_RUNNING_PORT},
    db::create_pool,
    handlers::init_app,
    utils::init_logger,
};

mod constants;
mod db;
mod handlers;
mod types;
mod utils;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() {
    dotenv().ok();

    init_logger();

    create_pool().await.unwrap_or_else(|error| {
        log::error!("failed to create db pool: {error}");
        process::exit(1)
    });

    let app = init_app().await;

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
