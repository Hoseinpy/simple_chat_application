use axum::{
    routing::{any, get, post},
    Router,
};
use shared::models::AppState;

use crate::handlers::{
    common::{handle_health, handle_version},
    room::{handle_connect_room, handle_create_room, handle_rooms_list},
};

mod common;
mod room;

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
            any(handle_connect_room).with_state(app_state.clone()),
        )
        .route("/room/list", get(handle_rooms_list).with_state(app_state))
}
