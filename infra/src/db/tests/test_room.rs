use std::sync::Arc;

use shared::{helpers::generate_uuid_v4, types::DefaultError};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{db::models::Room, test_utils::get_test_pool};

async fn create_room(uuid: Option<Uuid>) -> Result<(Arc<PgPool>, Room), DefaultError> {
    let test_pool = get_test_pool().await;
    let room = Room::create(Arc::clone(&test_pool), uuid).await?;

    Ok((test_pool, room))
}

#[tokio::test]
async fn test_create_room_instance_without_provide_uuid() {
    let room = create_room(None).await;

    assert!(room.is_ok())
}
#[tokio::test]
async fn test_create_room_instance_provide_uuid() {
    let uuid = generate_uuid_v4();
    let room = create_room(Some(uuid)).await;

    assert!(room.is_ok());
    assert_eq!(room.unwrap().1.get_uuid(), uuid)
}
#[tokio::test]
async fn test_read_single_room() {
    let c_room = create_room(None).await.unwrap();

    let read_room = Room::read(c_room.0, Some(c_room.1.get_uuid())).await;

    assert!(read_room.is_ok());
    assert!(read_room.unwrap().len() == 1)
}
#[tokio::test]
async fn test_read_all_room() {
    let test_pool = get_test_pool().await;
    create_room(None).await.unwrap();
    create_room(None).await.unwrap();

    let read_room = Room::read(test_pool, None).await;

    assert!(read_room.is_ok());
    assert!(read_room.unwrap().len() > 1)
}
