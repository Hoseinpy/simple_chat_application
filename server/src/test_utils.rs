use std::{collections::HashMap, sync::Arc};

use axum_test::TestServer;
use dotenvy::dotenv;
use infra::{cache::get_redis_client, db::create_pool};
use shared::models::AppState;
use tokio::sync::{Mutex, OnceCell};

use crate::handlers::init_app;

static TEST_SERVER: OnceCell<Arc<TestServer>> = OnceCell::const_new();

pub async fn get_test_server() -> Arc<TestServer> {
    dotenv().ok();
    TEST_SERVER
        .get_or_init(|| async {
            let db_pool = Arc::new(create_pool(true).await.unwrap());
            let redis_client = Arc::new(get_redis_client().unwrap());
            let app_state =
                AppState::new(db_pool, redis_client, Arc::new(Mutex::new(HashMap::new())));
            let app = init_app(app_state).await;

            Arc::new(TestServer::new(app).unwrap())
        })
        .await
        .clone()
}
