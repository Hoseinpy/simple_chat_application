use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde_json::json;

use crate::handlers::common::{handle_health, handle_version};

mod common;

pub struct ApiResponse;

impl ApiResponse {
    pub fn build<T: serde::Serialize>(
        success: bool,
        data: T,
        status_code: StatusCode,
    ) -> impl IntoResponse {
        let body = Json(json!({ "success": success, "data": data }));
        (status_code, body)
    }
}

pub async fn init_app() -> Router {
    Router::new()
        .route("/version", get(handle_version))
        .route("/health", get(handle_health))
}
