use crate::handlers::init_app;
use axum_test::TestServer;
use dotenvy::dotenv;
use infra::cache::get_redis_client;
use infra::db::create_pool;
use shared::models::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use redis::Client;

pub async fn setup_test_server() -> TestServer {
    dotenv().ok();

    let db_pool = Arc::new(create_pool().await.unwrap());
    let redis_client = Arc::new(get_redis_client().unwrap());

    let app_state = AppState::new(db_pool, redis_client, Arc::new(Mutex::new(HashMap::new())));
    let app = init_app(app_state).await;

    TestServer::new(app).unwrap()
}

pub fn setup_redis_client() -> Arc<Client> {
    dotenv().ok();
    Arc::new(get_redis_client().unwrap())
}
