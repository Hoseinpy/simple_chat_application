use std::{env, time::Duration};

use shared::types::DefaultError;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub mod models;

pub async fn create_pool() -> Result<PgPool, DefaultError> {
    let is_test_mode = if cfg!(test) { true } else { false };

    let var_name = if is_test_mode {
        "TEST_DATABASE_URL"
    } else {
        "DATABASE_URL"
    };
    let database_url =
        env::var(var_name).expect("DATABASE_URL and TEST_DATABASE_URL most be set in .env");

    let min_conn = if is_test_mode { 0 } else { 5 };
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(min_conn)
        .acquire_timeout(Duration::from_secs(2))
        .idle_timeout(Duration::from_secs(300))
        .test_before_acquire(true)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::create_pool;
    use dotenvy::dotenv;

    #[tokio::test]
    async fn test_create_pool() {
        dotenv().ok();
        assert!(create_pool().await.is_ok())
    }
}
