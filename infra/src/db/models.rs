use std::{future, sync::Arc};

use chrono::NaiveDateTime;
use shared::{helpers::generate_uuid_v4, types::DefaultError};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub trait CRUD {
    type Model;

    fn create(
        db_pool: Arc<PgPool>,
    ) -> impl future::Future<Output = Result<Self::Model, DefaultError>> + Send;
    fn read(&self, db_pool: Arc<PgPool>) -> impl future::Future<Output = ()> + Send;
    fn update(&mut self, db_pool: Arc<PgPool>) -> impl future::Future<Output = ()> + Send;
    fn delete(self, db_pool: Arc<PgPool>) -> impl future::Future<Output = ()> + Send;
}

// TODO: remove allow attribute after use in some place
#[derive(FromRow)]
#[allow(dead_code)]
pub struct Room {
    id: i32,
    uuid: Uuid,
    created_at: NaiveDateTime,
}

impl Room {
    pub fn get_uuid(&self) -> Uuid {
        self.uuid
    }
}

// TODO: remove _ prefix after use
impl CRUD for Room {
    type Model = Room;

    async fn create(db_pool: Arc<PgPool>) -> Result<Self::Model, DefaultError> {
        let uuid = generate_uuid_v4();
        let record = sqlx::query_as::<_, Room>("INSERT INTO room (uuid) VALUES ($1) RETURNING *")
            .bind(uuid)
            .fetch_one(&*db_pool)
            .await?;
        Ok(record)
    }
    async fn read(&self, _db_pool: Arc<PgPool>) {}
    async fn update(&mut self, _db_pool: Arc<PgPool>) {}
    async fn delete(self, _db_pool: Arc<PgPool>) {}
}
