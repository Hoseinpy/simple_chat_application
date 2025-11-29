use std::sync::Arc;

use dotenvy::dotenv;
use redis::Client;
use sqlx::PgPool;
use tokio::sync::OnceCell;

use crate::{cache::get_redis_client, db::create_pool};

static TEST_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();
static TEST_CLIENT: OnceCell<Arc<Client>> = OnceCell::const_new();

pub async fn get_test_pool() -> Arc<PgPool> {
    dotenv().ok();
    TEST_POOL
        .get_or_init(|| async { Arc::new(create_pool(true).await.unwrap()) })
        .await
        .clone()
}

pub async fn get_test_client() -> Arc<Client> {
    dotenv().ok();
    TEST_CLIENT
        .get_or_init(|| async { Arc::new(get_redis_client().unwrap()) })
        .await
        .clone()
}
