use std::{env, time::Duration};

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::types::DefaultError;

pub mod models;

pub async fn create_pool() -> Result<PgPool, DefaultError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL most be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(300))
        .test_before_acquire(true)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
