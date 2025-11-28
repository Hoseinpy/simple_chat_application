use std::{env, time::Duration};

use shared::types::DefaultError;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub mod models;

pub async fn create_pool() -> Result<PgPool, DefaultError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL most be set in .env");
    let min_conn = if cfg!(test) {
        0
    } else if cfg!(debug_assertions) {
        1
    } else {
        5
    };

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(min_conn)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(300))
        .test_before_acquire(true)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
