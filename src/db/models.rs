use chrono::NaiveDateTime;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct Room {
    id: i32,
    uuid: Uuid,
    created_at: NaiveDateTime,
}