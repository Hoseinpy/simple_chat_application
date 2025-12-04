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
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use rand::distr::{Alphanumeric, SampleString};
use redis::AsyncTypedCommands;
use serde_json::{Value, json};
use shared::{helpers::generate_uuid_v4, models::AppState, types::DefaultError};
use tokio::sync::broadcast::{self, Sender};
use uuid::Uuid;

use crate::{
    models::{ApiResponse, RoomResponse},
    rate_limiter::RateLimiter,
};
use infra::db::models::{Message, Room};

pub async fn handle_create_room(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !RateLimiter::run(&headers, 10, 600, Arc::clone(&app_state.redis_client)).await {
        return ApiResponse::build(false, String::new(), StatusCode::TOO_MANY_REQUESTS);
    }

    let room_uuid = generate_uuid_v4().to_string();
    let mut conn = match app_state
        .redis_client
        .get_multiplexed_async_connection()
        .await
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("failed to open redis connection: {e}");
            return ApiResponse::build(
                false,
                "failed to create room".into(),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };
    let _ = conn
        .set_ex(
            format!("room:{}", room_uuid).as_str(),
            room_uuid.clone(),
            3600,
        )
        .await;
    ApiResponse::build(true, room_uuid, StatusCode::OK)
}

// TODO: write some test for this endpoint
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
    let standard_err_code = StatusCode::INTERNAL_SERVER_ERROR.into_response();

    let mut conn = match app_state
        .redis_client
        .get_multiplexed_async_connection()
        .await
    {
        Ok(v) => v,
        Err(e) => {
            log::error!("failed to open redis connection: {e}");
            return standard_err_code;
        }
    };
    let cache_exits = match conn.exists(&cache_key).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("failed to check room cache exists: {e}");
            return standard_err_code;
        }
    };
    let mut tx = match app_state.db_pool.begin().await {
        Ok(v) => v,
        Err(e) => {
            log::error!("failed to start db tx: {e}");
            return standard_err_code;
        }
    };

    if cache_exits {
        match Room::create(&mut tx, Some(parsed_uuid)).await {
            Ok(room) => {
                room_id = room.get_id();
            }
            Err(e) => {
                log::error!("failed to create room: {e}");
                return standard_err_code;
            }
        }
        let _ = conn.del(cache_key).await;
        let _ = tx.commit().await;
    } else if let Ok(r) = Room::read(
        None,
        Some(Arc::clone(&app_state.db_pool)),
        Some(parsed_uuid),
    )
    .await
    {
        if r.is_empty() {
            return StatusCode::NOT_FOUND.into_response();
        } else {
            room_id = r[0].get_id()
        }
    }

    ws.on_upgrade(move |socket| {
        ConnectRoomWebSocket::join(socket, (parsed_uuid, room_id), app_state, headers)
    })
}

struct ConnectRoomWebSocket {
    username: String,
    room_info: (Uuid, i32),
    channel_tx: Sender<Json<Value>>,
    app_state: AppState,
}

impl ConnectRoomWebSocket {
    async fn join(
        socket: WebSocket,
        room_info: (Uuid, i32),
        app_state: AppState,
        headers: HeaderMap,
    ) {
        let (mut socket_send, socket_recv) = socket.split();
        let username = format!(
            "anonymous_{}",
            Alphanumeric.sample_string(&mut rand::rng(), 10)
        );
        let channel_tx = {
            let mut map = app_state.channels.lock().await;
            map.entry(room_info.0)
                .or_insert_with(|| broadcast::channel(100).0)
                .clone()
        };

        let mut connect_room_web_socket = Self {
            username,
            room_info,
            channel_tx,
            app_state,
        };

        connect_room_web_socket
            .send_info(&mut socket_send)
            .await
            .unwrap_or(());
        connect_room_web_socket
            .load_previous_messages(&mut socket_send)
            .await
            .unwrap_or(());
        connect_room_web_socket
            .process(socket_send, socket_recv, headers)
            .await;
    }

    async fn send_info(
        &mut self,
        socket_send: &mut SplitSink<WebSocket, WsMessage>,
    ) -> Result<(), DefaultError> {
        socket_send
            .send(WsMessage::Text("Connected".into()))
            .await?;
        self.channel_tx.send(Json(Value::String(format!(
            "user {} joined to room",
            self.username
        ))))?;

        Ok(())
    }

    async fn load_previous_messages(
        &mut self,
        socket_send: &mut SplitSink<WebSocket, WsMessage>,
    ) -> Result<(), DefaultError> {
        for message in Message::read(
            None,
            Some(Arc::clone(&self.app_state.db_pool)),
            self.room_info.1,
            200,
        )
        .await
        .unwrap_or(Vec::new())
        {
            socket_send
                .send(WsMessage::Text(message.get_message().into()))
                .await?
        }
        Ok(())
    }

    async fn process(
        &mut self,
        mut socket_send: SplitSink<WebSocket, WsMessage>,
        mut socket_recv: SplitStream<WebSocket>,
        headers: HeaderMap,
    ) {
        let mut channel_rx = self.channel_tx.subscribe();

        let app_state = self.app_state.clone();
        let username = self.username.clone();
        let room_info = self.room_info;
        let channel_tx = self.channel_tx.clone();

        let mut send_task = tokio::spawn(async move {
            while let Ok(message) = channel_rx.recv().await {
                if socket_send
                    .send(WsMessage::text(message.to_string()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(message)) = socket_recv.next().await {
                // TODO: notify user in limited
                if !RateLimiter::run(&headers, 10, 60, Arc::clone(&app_state.redis_client)).await {
                    continue;
                };

                match message {
                    WsMessage::Text(m) => {
                        let mut db_tx = match app_state.db_pool.begin().await {
                            Ok(v) => v,
                            Err(e) => {
                                log::error!("failed to start db tx: {e}");
                                continue;
                            }
                        };
                        let parse_message =
                            Json(json!({ "user": username, "message": m.to_string() }));
                        let _ = Message::create(&mut db_tx, parse_message.to_string(), room_info.1)
                            .await;

                        match channel_tx.send(parse_message) {
                            Ok(_) => {
                                let _ = db_tx.commit().await;
                            }
                            Err(_) => {
                                let _ = db_tx.rollback().await;
                            }
                        }
                    }
                    WsMessage::Close(_) => {
                        let _ = channel_tx.send(Json(Value::String(format!(
                            "user {} leave the room",
                            username
                        ))));
                        if channel_tx.receiver_count() == 1 {
                            app_state.channels.lock().await.remove(&room_info.0);
                        }
                        break;
                    }
                    _ => (),
                }
            }
        });

        tokio::select! {
            _ = &mut send_task => recv_task.abort(),
            _ = &mut recv_task => send_task.abort(),
        };
    }
}

// TODO: replace domain with real one
pub async fn handle_rooms_list(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !RateLimiter::run(&headers, 10, 60, app_state.redis_client).await {
        return ApiResponse::build(false, Vec::new(), StatusCode::TOO_MANY_REQUESTS);
    }

    let mut rooms = Vec::new();
    {
        let channels = app_state.channels.lock().await;

        rooms.reserve(channels.len());

        for (uuid, tx) in channels.iter() {
            let room_size = tx.receiver_count();

            rooms.push(RoomResponse::new(
                uuid.to_string(),
                room_size,
                format!("ws://domain.com/room/{}", uuid),
            ));
        }
    }

    rooms.sort_unstable_by(|a, b| b.room_size.cmp(&a.room_size));

    ApiResponse::build(true, rooms, StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{get_redis_test_client, get_test_server};
    use redis::AsyncCommands;
    use serde_json::Value;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_handle_create_room() {
        let server = get_test_server().await;

        let response = server
            .post("/room/create")
            .add_header("x-forwarded-for", "127.0.0.6")
            .await;
        assert_eq!(response.status_code(), 200);

        let response_uuid =
            Uuid::parse_str(response.json::<Value>()["data"].as_str().unwrap()).unwrap();

        let redis_client = get_redis_test_client().await;
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();
        let cache_key = format!("room:{}", response_uuid);

        assert!(
            conn.get::<_, Option<String>>(&cache_key)
                .await
                .unwrap()
                .is_some()
        );

        conn.del::<_, ()>(cache_key).await.unwrap();
        conn.del::<_, ()>("rate_limiter:127.0.0.6").await.unwrap();
    }
    #[tokio::test]
    async fn test_handle_create_room_return_429() {
        let server = get_test_server().await;

        let redis_client = get_redis_test_client().await;
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        conn.set_ex::<_, _, ()>("rate_limiter:127.0.0.7", "0", 10)
            .await
            .unwrap();

        let response = server
            .post("/room/create")
            .add_header("x-forwarded-for", "127.0.0.7")
            .await;
        assert_eq!(response.status_code(), 429);
    }
    #[tokio::test]
    async fn test_handle_rooms_list() {
        let server = get_test_server().await;

        let response = server
            .get("/room/list")
            .add_header("x-forwarded-for", "127.0.0.8")
            .await;
        assert_eq!(response.status_code(), 200);

        assert_eq!(
            response.json::<Value>()["data"].as_array().unwrap().len(),
            0
        )
    }
    #[tokio::test]
    async fn test_handle_rooms_list_return_429() {
        let server = get_test_server().await;

        let redis_client = get_redis_test_client().await;
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        conn.set_ex::<_, _, ()>("rate_limiter:127.0.0.9", "0", 10)
            .await
            .unwrap();

        let response = server
            .get("/room/list")
            .add_header("x-forwarded-for", "127.0.0.9")
            .await;
        assert_eq!(response.status_code(), 429);
    }
}
