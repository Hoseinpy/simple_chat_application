use std::sync::Arc;

use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;
use sqlx::PgPool;

use crate::handlers::{
    common::{handle_health, handle_version},
    room::handle_create_room,
};

mod common;
mod room;

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

// TODO: remove allow attribute after use in some place
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    db_pool: Arc<PgPool>,
    redis_client: redis::Client,
}

impl AppState {
    pub fn new(db_pool: Arc<PgPool>, redis_client: redis::Client) -> Self {
        Self {
            db_pool,
            redis_client,
        }
    }
}

pub async fn init_app(app_state: AppState) -> Router {
    Router::new()
        .route("/version", get(handle_version))
        .route("/health", get(handle_health))
        .route(
            "/room/create",
            post(handle_create_room).with_state(app_state),
        )
}
