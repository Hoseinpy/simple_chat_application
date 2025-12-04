use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::json;

pub struct ApiResponse;

impl ApiResponse {
    pub fn build<T: Serialize>(
        success: bool,
        data: T,
        status_code: StatusCode,
    ) -> impl IntoResponse {
        let body = Json(json!({ "success": success, "data": data }));
        (status_code, body)
    }
}

#[derive(Serialize)]
pub struct RoomResponse {
    uuid: String,
    pub room_size: usize,
    connect_url: String,
}

impl RoomResponse {
    pub fn new(uuid: String, room_size: usize, connect_url: String) -> Self {
        Self {
            uuid,
            room_size,
            connect_url,
        }
    }
}
