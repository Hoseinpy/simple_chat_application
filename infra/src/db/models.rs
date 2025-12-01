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
    pub async fn delete(db_pool: Arc<PgPool>, uuid: Option<Uuid>) -> Result<(), DefaultError> {
        match uuid {
            Some(v) => {
                sqlx::query("DELETE FROM room WHERE uuid = $1")
                    .bind(v)
                    .execute(&*db_pool)
                    .await?;
            }
            None => {
                sqlx::query("DELETE FROM room").execute(&*db_pool).await?;
            }
        }

        Ok(())
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
    pub fn get_room_id(&self) -> i32 {
        self.room_id
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

#[cfg(test)]
mod tests {
    use super::Room;
    use crate::test_utils::get_db_test_pool;
    use shared::helpers::generate_uuid_v4;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_room_not_provide_uuid() {
        let db_pool = get_db_test_pool().await;

        let record = Room::create(Arc::clone(&db_pool), None).await;
        assert!(record.is_ok());
    }
    #[tokio::test]
    async fn test_create_room_provide_uuid() {
        let db_pool = get_db_test_pool().await;
        let uuid = generate_uuid_v4();

        let record = Room::create(db_pool, Some(uuid.clone())).await;
        assert!(record.is_ok());

        assert_eq!(record.unwrap().get_uuid(), uuid)
    }
    #[tokio::test]
    async fn test_read_single_room() {
        let db_pool = get_db_test_pool().await;

        let record = Room::create(Arc::clone(&db_pool), None).await.unwrap();

        let result = Room::read(db_pool, Some(record.get_uuid())).await;
        assert!(result.is_ok());

        assert_eq!(result.unwrap().len(), 1)
    }
    #[tokio::test]
    async fn test_read_all_rooms() {
        let db_pool = get_db_test_pool().await;

        Room::create(Arc::clone(&db_pool), None).await.unwrap();
        Room::create(Arc::clone(&db_pool), None).await.unwrap();

        let results = Room::read(db_pool, None).await;
        assert!(results.is_ok());

        assert!(results.unwrap().len() > 1)
    }
    #[tokio::test]
    async fn test_delete_one_record() {
        let db_pool = get_db_test_pool().await;

        let record = Room::create(Arc::clone(&db_pool), None).await.unwrap();

        let result = Room::delete(db_pool, Some(record.get_uuid())).await;
        assert!(result.is_ok())
    }
    #[tokio::test]
    async fn test_delete_all_records() {
        let db_pool = get_db_test_pool().await;

        let result = Room::delete(db_pool, None).await;
        assert!(result.is_ok())
    }
}
