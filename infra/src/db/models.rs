use chrono::NaiveDateTime;
use shared::{helpers::generate_uuid_v4, types::DefaultError};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;

use crate::db::queries::{Binds, delete, fetch, insert};

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
    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        uuid: Option<Uuid>,
    ) -> Result<Room, DefaultError> {
        let uuid = match uuid {
            Some(v) => v,
            None => generate_uuid_v4(),
        };
        let record = insert(
            "INSERT INTO room (uuid) VALUES ($1) RETURNING *",
            vec![Binds::Uuid(uuid)],
            tx,
        )
        .await?;

        Ok(record)
    }
    pub async fn read(
        tx: Option<&mut Transaction<'_, Postgres>>,
        db_pool: Option<Arc<PgPool>>,
        uuid: Option<Uuid>,
    ) -> Result<Vec<Room>, DefaultError> {
        match uuid {
            Some(v) => {
                let records = fetch(
                    "SELECT * FROM room WHERE uuid = $1",
                    vec![Binds::Uuid(v)],
                    tx,
                    db_pool,
                )
                .await?;
                Ok(records)
            }
            None => {
                let records = fetch("SELECT * FROM room", vec![], tx, db_pool).await?;
                Ok(records)
            }
        }
    }
    pub async fn delete(tx: &mut Transaction<'_, Postgres>, id: i32) -> Result<(), DefaultError> {
        delete::<Room>("DELETE FROM room WHERE id = $1", vec![Binds::I32(id)], tx).await?;

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
    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_message(&self) -> String {
        self.message.clone()
    }
    pub async fn create(
        tx: &mut Transaction<'_, Postgres>,
        message: String,
        room_id: i32,
    ) -> Result<Message, DefaultError> {
        let record = insert(
            "INSERT INTO message (message, room_id) VALUES ($1, $2) RETURNING *",
            vec![Binds::String(message), Binds::I32(room_id)],
            tx,
        )
        .await?;
        Ok(record)
    }
    pub async fn read(
        tx: Option<&mut Transaction<'_, Postgres>>,
        db_pool: Option<Arc<PgPool>>,
        room_id: i32,
        limit: i32,
    ) -> Result<Vec<Message>, DefaultError> {
        let records = fetch(
            "SELECT * FROM message WHERE room_id = $1 LIMIT $2",
            vec![Binds::I32(room_id), Binds::I32(limit)],
            tx,
            db_pool,
        )
        .await?;
        Ok(records)
    }
    pub async fn delete(tx: &mut Transaction<'_, Postgres>, id: i32) -> Result<(), DefaultError> {
        delete::<Message>(
            "DELETE FROM message WHERE id = $1",
            vec![Binds::I32(id)],
            tx,
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use shared::helpers::generate_uuid_v4;

    use super::{Message, Room};
    use crate::test_utils::get_db_test_pool;

    #[tokio::test]
    async fn test_create_room_not_provide_uuid() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        let record = Room::create(&mut tx, None).await;
        assert!(record.is_ok());

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_create_room_provide_uuid() {
        let db_pool = get_db_test_pool().await;
        let uuid = generate_uuid_v4();
        let mut tx = db_pool.begin().await.unwrap();

        let record = Room::create(&mut tx, Some(uuid.clone())).await;
        assert!(record.is_ok());

        assert_eq!(record.unwrap().get_uuid(), uuid);

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_read_single_room() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        let record = Room::create(&mut tx, None).await.unwrap();

        let result = Room::read(Some(&mut tx), None, Some(record.get_uuid())).await;
        assert!(result.is_ok());

        assert_eq!(result.unwrap().len(), 1);

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_read_all_rooms() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        for _ in 0..=2 {
            Room::create(&mut tx, None).await.unwrap();
        }
        let results = Room::read(Some(&mut tx), None, None).await;
        assert!(results.is_ok());

        assert!(results.unwrap().len() > 1);

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_delete_one_room() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        let record = Room::create(&mut tx, None).await.unwrap();

        let result = Room::delete(&mut tx, record.get_id()).await;
        assert!(result.is_ok());

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_create_message() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        let some_room = Room::create(&mut tx, None).await.unwrap();
        let record = Message::create(&mut tx, "hello-rust".to_string(), some_room.get_id()).await;
        assert!(record.is_ok());
        assert_eq!(record.unwrap().get_message(), "hello-rust");

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_read_single_message() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        let some_room_id = Room::create(&mut tx, None).await.unwrap().get_id();
        Message::create(&mut tx, "hello-rust-2".to_string(), some_room_id)
            .await
            .unwrap();

        let record = Message::read(Some(&mut tx), None, some_room_id, 10).await;
        assert!(record.is_ok());
        assert_eq!(record.unwrap().len(), 1);

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_read_all_messages() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        let some_room_id = Room::create(&mut tx, None).await.unwrap().get_id();

        for _ in 0..=2 {
            Message::create(&mut tx, "hello-rust-3".to_string(), some_room_id)
                .await
                .unwrap();
        }

        let record = Message::read(Some(&mut tx), None, some_room_id, 10).await;
        assert!(record.is_ok());
        assert!(record.unwrap().len() > 1);

        tx.rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_delete_one_message() {
        let db_pool = get_db_test_pool().await;
        let mut tx = db_pool.begin().await.unwrap();

        let some_room = Room::create(&mut tx, None).await.unwrap();
        let message = Message::create(&mut tx, "hello-rust-love".to_string(), some_room.get_id())
            .await
            .unwrap();

        let result = Message::delete(&mut tx, message.get_id()).await;
        assert!(result.is_ok());

        tx.rollback().await.unwrap();
    }
}
