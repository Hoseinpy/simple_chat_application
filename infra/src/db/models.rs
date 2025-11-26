use std::sync::Arc;

use chrono::NaiveDateTime;
use shared::{helpers::generate_uuid_v4, types::DefaultError};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

// TODO: remove allow attribute after use in some place
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Room {
    id: i32,
    uuid: Uuid,
    created_at: NaiveDateTime,
}

impl Room {
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
    pub async fn create(db_pool: Arc<PgPool>, uuid: Option<Uuid>) -> Result<Room, DefaultError> {
        let uuid = match uuid {
            Some(v) => v,
            None => generate_uuid_v4(),
        };
        let record = sqlx::query_as::<_, Room>("INSERT INTO room (uuid) VALUES ($1) RETURNING *")
            .bind(uuid)
            .fetch_one(&*db_pool)
            .await?;
        Ok(record)
    }
    pub async fn read(db_pool: Arc<PgPool>, uuid: Option<Uuid>) -> Result<Vec<Room>, DefaultError> {
        let records = match uuid {
            Some(v) => {
                sqlx::query_as::<_, Room>("SELECT * FROM room WHERE uuid = $1")
                    .bind(v)
                    .fetch_all(&*db_pool)
                    .await?
            }
            None => {
                sqlx::query_as::<_, Room>("SELECT * FROM room")
                    .fetch_all(&*db_pool)
                    .await?
            }
        };

        Ok(records)
    }
}

#[derive(FromRow)]
#[allow(dead_code)]
pub struct Message {
    id: i32,
    room_id: i32,
    message: String,
    created_at: NaiveDateTime,
}

impl Message {
    pub fn get_message(&self) -> String {
        self.message.clone()
    }
    pub async fn create(
        db_pool: Arc<PgPool>,
        message: String,
        room_id: i32,
    ) -> Result<Message, DefaultError> {
        let record = sqlx::query_as::<_, Message>(
            "INSERT INTO message (message, room_id) VALUES ($1, $2) RETURNING *",
        )
        .bind(message)
        .bind(room_id)
        .fetch_one(&*db_pool)
        .await?;
        Ok(record)
    }
    pub async fn read(
        db_pool: Arc<PgPool>,
        room_id: i32,
        limit: i32,
    ) -> Result<Vec<Message>, DefaultError> {
        let records =
            sqlx::query_as::<_, Message>("SELECT * FROM message WHERE room_id = $1 LIMIT $2")
                .bind(room_id)
                .bind(limit)
                .fetch_all(&*db_pool)
                .await?;

        Ok(records)
    }
}
