use std::sync::Arc;

use axum::http::HeaderMap;
use redis::{AsyncCommands, Client};
use shared::types::DefaultError;

use crate::utils::extract_request_ip;

pub struct RateLimiter {
    key: String,
    limit: u16,
    seconds: u64,
    redis_client: Arc<Client>,
}

impl RateLimiter {
    pub fn new(key: String, limit: u16, seconds: u64, redis_client: Arc<Client>) -> Self {
        Self {
            key,
            limit,
            seconds,
            redis_client,
        }
    }
    async fn check_and_apply(&self) -> Result<bool, DefaultError> {
        let key = self.key.clone();

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        let get_cache: Option<String> = conn.get(&key).await?;
        match get_cache {
            Some(raw_value) => {
                let value: u16 = raw_value.parse()?;
                if value == 0 {
                    return Ok(false);
                }

                conn.decr(&key, 1).await?
            }
            None => {
                conn.set_ex(&key, self.limit.to_string(), self.seconds)
                    .await?
            }
        }

        Ok(true)
    }
    pub async fn run(
        headers: &HeaderMap,
        limit: u16,
        seconds: u64,
        redis_client: Arc<Client>,
    ) -> bool {
        let key = format!("rate_limiter:{}", extract_request_ip(headers));
        let rate_limiter = RateLimiter::new(key, limit, seconds, redis_client);

        rate_limiter.check_and_apply().await.unwrap_or(true)
    }
}
