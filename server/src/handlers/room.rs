use std::sync::Arc;

use axum::{
    extract::{Path, State, WebSocketUpgrade, ws::WebSocket},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use redis::AsyncTypedCommands;
use shared::helpers::generate_uuid_v4;
use uuid::Uuid;

use crate::{
    handlers::{ApiResponse, AppState},
    rate_limiter::RateLimiter,
};
use infra::db::models::{CRUD, Room};

pub async fn handle_create_room(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !RateLimiter::run(&headers, 10, 600, Arc::clone(&app_state.redis_client)).await {
        return ApiResponse::build(
            false,
            "To Many Requests".to_string(),
            StatusCode::TOO_MANY_REQUESTS,
        );
    }

    let room_uuid = generate_uuid_v4().to_string();

    match app_state
        .redis_client
        .get_multiplexed_async_connection()
        .await
    {
        Ok(mut conn) => {
            let _ = conn
                .set_ex(
                    format!("room:{}", room_uuid).as_str(),
                    room_uuid.clone(),
                    3600,
                )
                .await;
            ApiResponse::build(true, room_uuid, StatusCode::OK)
        }
        Err(e) => {
            log::error!("failed to open redis connection: {e}");
            ApiResponse::build(
                false,
                "failed to create room".into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

pub async fn handle_connect_room(
    ws: WebSocketUpgrade,
    Path(uuid): Path<String>,
    State(app_state): State<AppState>,
) -> Response {
    // validate uuid
    let parsed_uuid = match Uuid::parse_str(&uuid) {
        Ok(v) => v,
        Err(_) => {
            return StatusCode::NOT_FOUND.into_response();
        }
    };

    match app_state
        .redis_client
        .get_multiplexed_async_connection()
        .await
    {
        Ok(mut conn) => match conn.exists(format!("room:{}", uuid)).await {
            Ok(exist) => {
                if !exist
                    && let Ok(v) =
                        Room::read(Arc::clone(&app_state.db_pool), Some(parsed_uuid)).await
                    && v.is_empty()
                {
                    return StatusCode::NOT_FOUND.into_response();
                }
            }
            Err(e) => {
                log::error!("failed to check room exists: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
        Err(e) => {
            log::error!("failed to open redis connection: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    ws.on_upgrade(move |socket| connect_room(socket, parsed_uuid, app_state))
}

async fn connect_room(mut socket: WebSocket, uuid: Uuid, app_state: AppState) {}
