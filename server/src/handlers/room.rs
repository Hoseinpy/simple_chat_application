use std::sync::Arc;

use axum::{
    Json,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message as WsMessage, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use rand::distr::{Alphanumeric, SampleString};
use redis::AsyncTypedCommands;
use serde_json::{Value, json};
use shared::helpers::generate_uuid_v4;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    handlers::{ApiResponse, AppState},
    rate_limiter::RateLimiter,
};
use infra::db::models::{Message, Room};

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

// TODO: if possible write some test for this endpoint
pub async fn handle_connect_room(
    ws: WebSocketUpgrade,
    Path(uuid): Path<String>,
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    // validate uuid
    let parsed_uuid = match Uuid::parse_str(&uuid) {
        Ok(v) => v,
        Err(_) => {
            return StatusCode::NOT_FOUND.into_response();
        }
    };
    let mut room_id = 0;
    let cache_key = format!("room:{}", uuid);

    match app_state
        .redis_client
        .get_multiplexed_async_connection()
        .await
    {
        Ok(mut conn) => match conn.exists(&cache_key).await {
            Ok(true) => {
                let c_room =
                    match Room::create(Arc::clone(&app_state.db_pool), Some(parsed_uuid)).await {
                        Ok(r) => r,
                        Err(e) => {
                            log::error!("failed to check room exists: {e}");
                            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                        }
                    };
                room_id = c_room.get_id();
                let _ = conn.del(cache_key).await;
            }
            Ok(false) => {
                if let Ok(r) = Room::read(Arc::clone(&app_state.db_pool), Some(parsed_uuid)).await {
                    if r.is_empty() {
                        return StatusCode::NOT_FOUND.into_response();
                    } else {
                        room_id = r[0].get_id()
                    }
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

    ws.on_upgrade(move |socket| connect_room(socket, parsed_uuid, app_state, room_id, headers))
}

// TODO: refactor it
async fn connect_room(
    socket: WebSocket,
    uuid: Uuid,
    app_state: AppState,
    room_id: i32,
    headers: HeaderMap,
) {
    let (mut socket_send, mut socket_recv) = socket.split();
    let tx = {
        let mut map = app_state.channels.lock().await;
        map.entry(uuid)
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    };
    let mut rx = tx.subscribe();

    let random_name = format!(
        "anonymous_{}",
        Alphanumeric.sample_string(&mut rand::rng(), 10)
    );
    let join_message = format!("user {} joined to room", &random_name);
    let leave_message = format!("user {} leave the room", &random_name);

    // setup
    let _ = tx.send(Json(Value::String(join_message)));
    for message in Message::read(Arc::clone(&app_state.db_pool), room_id, 100)
        .await
        .unwrap_or(Vec::new())
    {
        let _ = socket_send
            .send(WsMessage::Text(message.get_message().into()))
            .await;
    }

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if socket_send
                .send(WsMessage::text(msg.to_string()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = socket_recv.next().await {
            if !RateLimiter::run(&headers, 10, 60, Arc::clone(&app_state.redis_client)).await {
                // let _ = socket_send.send(WsMessage::text(String::from(
                //     "you are limited. please try after 60 seconds",
                // )));
                continue;
            };
            match msg {
                WsMessage::Text(m) => {
                    let message = Json(json!({ "user": random_name, "message": m.to_string() }));
                    let _ = Message::create(
                        Arc::clone(&app_state.db_pool),
                        message.to_string(),
                        room_id,
                    )
                    .await;
                    let _ = tx.send(message);
                }
                WsMessage::Close(_) => {
                    let _ = tx.send(Json(Value::String(leave_message.clone())));
                }
                _ => (),
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::get_test_server;
    use infra::cache::get_redis_client;
    use redis::AsyncCommands;
    use serde_json::Value;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_handle_create_room_successfully() {
        let server = get_test_server().await;

        let response = server.post("/room/create").await;
        assert_eq!(response.status_code(), 200);

        let created_uuid =
            Uuid::parse_str(response.json::<Value>()["data"].as_str().unwrap()).unwrap();
        let redis_client = get_redis_client().unwrap();
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        let cache_key = format!("room:{}", created_uuid);
        let cache: Option<String> = conn.get(&cache_key).await.unwrap();
        assert!(cache.is_some());

        conn.del::<_, ()>(cache_key).await.unwrap();
    }
    #[tokio::test]
    async fn test_handle_create_room_to_many_requests_error() {
        let server = get_test_server().await;

        // fake rate limit
        let redis_client = get_redis_client().unwrap();
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();
        conn.set_ex::<_, _, ()>("rate_limiter:127.0.0.10", "0", 10)
            .await
            .unwrap();

        let response = server
            .post("/room/create")
            .add_header("x-forwarded-for", "127.0.0.10")
            .await;
        assert_eq!(response.status_code(), 429)
    }
}
