use std::{env, time::Duration};

use shared::types::DefaultError;
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};

pub mod models;

#[cfg(test)]
mod tests;

pub async fn create_pool(test_mode: bool) -> Result<PgPool, DefaultError> {
    let var_name = if test_mode {
        "TEST_DATABASE_URL"
    } else {
        "DATABASE_URL"
    };
    let database_url = env::var(var_name).expect("DATABASE_URL most be set in .env");
    let db_exists = Postgres::database_exists(&database_url).await?;

    if !db_exists {
        Postgres::create_database(&database_url).await?;
    }

    let min_conn = if test_mode { 0 } else { 5 };
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(min_conn)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(300))
        .test_before_acquire(true)
        .connect(&database_url)
        .await?;

    if !db_exists {
        let _ = sqlx::migrate!("../migrations").run(&pool).await;
    }

    Ok(pool)
}
