use crate::handlers::init_app;
use axum_test::TestServer;
use dotenvy::dotenv;
use infra::cache::get_redis_client;
use infra::db::create_pool;
use redis::Client;
use shared::models::AppState;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, OnceCell};

static REDIS_TEST_CLIENT: OnceCell<Arc<Client>> = OnceCell::const_new();
static DB_TEST_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();
static TEST_SERVER: OnceCell<Arc<TestServer>> = OnceCell::const_new();

pub async fn get_test_server() -> Arc<TestServer> {
    TEST_SERVER
        .get_or_init(|| async {
            let redis_client = get_redis_test_client().await;
            let db_pool = get_db_test_pool().await;
            let channels = Arc::new(Mutex::new(HashMap::new()));

            let app_state = AppState::new(db_pool, redis_client, channels);
            let app = init_app(app_state).await;

            Arc::new(TestServer::new(app).unwrap())
        })
        .await
        .clone()
}

pub async fn get_redis_test_client() -> Arc<Client> {
    REDIS_TEST_CLIENT
        .get_or_init(|| async {
            dotenv().ok();
            Arc::new(get_redis_client().unwrap())
        })
        .await
        .clone()
}

pub async fn get_db_test_pool() -> Arc<PgPool> {
    DB_TEST_POOL
        .get_or_init(|| async {
            dotenv().ok();
            Arc::new(create_pool().await.unwrap())
        })
        .await
        .clone()
}
