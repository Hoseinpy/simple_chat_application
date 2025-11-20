use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use redis::AsyncTypedCommands;
use shared::helpers::generate_uuid_v4;

use crate::{
    handlers::{ApiResponse, AppState},
    rate_limiter::RateLimiter,
};

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
