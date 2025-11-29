use axum::{http::StatusCode, response::IntoResponse};

use crate::handlers::ApiResponse;

pub async fn handle_version() -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    ApiResponse::build(true, version, StatusCode::OK)
}

pub async fn handle_health() -> StatusCode {
    StatusCode::OK
}
