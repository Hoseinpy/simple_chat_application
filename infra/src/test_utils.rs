use crate::db::create_pool;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::OnceCell;

static DB_TEST_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

pub async fn get_db_test_pool() -> Arc<PgPool> {
    DB_TEST_POOL
        .get_or_init(|| async {
            dotenv().ok();
            Arc::new(create_pool().await.unwrap())
        })
        .await
        .clone()
}
