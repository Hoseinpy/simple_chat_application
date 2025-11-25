use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post},
};
use serde_json::json;
use shared::models::AppState;

use crate::handlers::{
    common::{handle_health, handle_version},
    room::{handle_connect_room, handle_create_room},
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

pub async fn init_app(app_state: AppState) -> Router {
    Router::new()
        .route("/version", get(handle_version))
        .route("/health", get(handle_health))
        .route(
            "/room/create",
            post(handle_create_room).with_state(app_state.clone()),
        )
        .route(
            "/room/{uuid}",
            any(handle_connect_room).with_state(app_state),
        )
}
