use std::sync::Arc;

use sqlx::PgPool;

use crate::types::Channel;

// TODO: remove allow attribute after use in some place
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
    pub redis_client: Arc<redis::Client>,
    pub channels: Channel,
}

impl AppState {
    pub fn new(db_pool: Arc<PgPool>, redis_client: Arc<redis::Client>, channels: Channel) -> Self {
        Self {
            db_pool,
            redis_client,
            channels,
        }
    }
}
