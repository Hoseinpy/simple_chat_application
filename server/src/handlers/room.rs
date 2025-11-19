use axum::{extract::State, http::StatusCode, response::IntoResponse};

use redis::AsyncTypedCommands;
use shared::helpers::generate_uuid_v4;

use crate::handlers::{ApiResponse, AppState};

pub async fn handle_create_room(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state
        .redis_client
        .get_multiplexed_async_connection()
        .await
    {
        Ok(mut conn) => {
            let room_uuid = generate_uuid_v4().to_string();
            let _ = conn
                .set_ex(format!("room_{}", room_uuid), &room_uuid, 3600)
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
