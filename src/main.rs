use tokio::net::TcpListener;

use crate::{
    constants::{SERVER_RUNNING_ADDRESS, SERVER_RUNNING_PORT},
    handlers::init_app,
    utils::init_logger,
};

mod constants;
mod handlers;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// TODO: fix axum default logs

#[tokio::main]
async fn main() {
    init_logger();

    let app = init_app().await;

    let addr = format!("{}:{}", SERVER_RUNNING_ADDRESS, SERVER_RUNNING_PORT);
    let listener = TcpListener::bind(addr).await.unwrap_or_else(|error| {
        log::error!("failed to start listener: {error}");
        std::process::exit(1)
    });

    axum::serve(listener, app).await.unwrap_or_else(|error| {
        log::error!("failed to start server: {error}");
        std::process::exit(1)
    })
}
