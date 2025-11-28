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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_redis_client;
    use axum::http::{HeaderMap, HeaderValue};
    use std::sync::Arc;

    #[test]
    fn test_create_new_instatnce() {
        let redis_client = setup_redis_client();

        let key = "rate_limiter:127.0.0.1".to_string();
        let rate_limiter = RateLimiter::new(key.clone(), 10, 60, redis_client);

        assert_eq!(rate_limiter.key, key);
        assert_eq!(rate_limiter.limit, 10);
        assert_eq!(rate_limiter.seconds, 60);
    }
    #[tokio::test]
    async fn test_run_cehck_and_apply_with_exist_cache() {
        let redis_client = setup_redis_client();

        let rate_limiter = RateLimiter::new(
            "rate_limiter:127.0.0.2".to_string(),
            10,
            10,
            Arc::clone(&redis_client),
        );

        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        conn.del::<_, ()>(&rate_limiter.key).await.unwrap();

        conn.set_ex::<_, _, ()>(
            &rate_limiter.key,
            rate_limiter.limit.to_string(),
            rate_limiter.seconds,
        )
        .await
        .unwrap();

        let result = rate_limiter.check_and_apply().await.unwrap();

        let get_cache: Option<String> = conn.get(&rate_limiter.key).await.unwrap();
        assert_eq!(get_cache.unwrap(), "9");

        assert!(result);
    }
    #[tokio::test]
    async fn test_run_cehck_and_apply_with_not_exist_cache() {
        let redis_client = setup_redis_client();

        let rate_limiter = RateLimiter::new(
            "rate_limiter:127.0.0.3".to_string(),
            10,
            10,
            Arc::clone(&redis_client),
        );

        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        conn.del::<_, ()>(&rate_limiter.key).await.unwrap();

        let result = rate_limiter.check_and_apply().await.unwrap();

        let get_cache: Option<String> = conn.get(&rate_limiter.key).await.unwrap();
        assert_eq!(get_cache.unwrap(), "10");

        assert!(result);
    }
    #[tokio::test]
    async fn test_run_check_and_apply_value_eq_zero() {
        let redis_client = setup_redis_client();

        let rate_limiter = RateLimiter::new(
            "rate_limiter:127.0.0.4".to_string(),
            0,
            10,
            Arc::clone(&redis_client),
        );

        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        conn.del::<_, ()>(&rate_limiter.key).await.unwrap();

        conn.set_ex::<_, _, ()>(
            &rate_limiter.key,
            rate_limiter.limit.to_string(),
            rate_limiter.seconds,
        )
        .await
        .unwrap();

        let result = rate_limiter.check_and_apply().await.unwrap();

        assert!(!result);
    }
    #[tokio::test]
    async fn test_run_method() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_str("127.0.0.5").unwrap(),
        );

        let redis_client = setup_redis_client();
        let mut conn = redis_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();

        conn.del::<_, ()>("rate_limiter:127.0.0.5").await.unwrap();

        let result = RateLimiter::run(&headers, 10, 10, redis_client).await;

        assert!(result)
    }
}
