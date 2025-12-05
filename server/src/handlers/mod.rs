use axum::{
    Router,
    http::Method,
    routing::{any, get, post},
};
use axum_helmet::{Helmet, HelmetLayer};
use shared::models::AppState;
use tower_http::cors::{Any, CorsLayer};

use crate::handlers::{
    common::{handle_health, handle_version},
    room::{handle_connect_room, handle_create_room, handle_rooms_list},
};

mod common;
mod room;

pub async fn init_app(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

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
        .layer(HelmetLayer::new(
            Helmet::new()
                .add(helmet_core::XContentTypeOptions::nosniff())
                .add(helmet_core::XFrameOptions::deny())
                .add(helmet_core::XXSSProtection::on().mode_block())
                .add(
                    helmet_core::StrictTransportSecurity::default()
                        .max_age(31536000)
                        .include_sub_domains()
                        .preload(),
                )
                .add(helmet_core::ReferrerPolicy::same_origin())
                .add(helmet_core::CrossOriginOpenerPolicy::same_origin()),
        ))
        .layer(cors)
}
