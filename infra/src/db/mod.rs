use std::{env, time::Duration};

use shared::types::DefaultError;
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};

pub mod models;

pub async fn create_pool() -> Result<PgPool, DefaultError> {
    let is_test_mode = cfg!(test);

    let var_name = if is_test_mode {
        "TEST_DATABASE_URL"
    } else {
        "DATABASE_URL"
    };
    let database_url =
        env::var(var_name).expect("DATABASE_URL and TEST_DATABASE_URL most be set in .env");

    let min_conn = if is_test_mode { 0 } else { 5 };
    let pool = async || -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(30)
            .min_connections(min_conn)
            .acquire_timeout(Duration::from_secs(2))
            .idle_timeout(Duration::from_secs(300))
            .test_before_acquire(true)
            .connect(&database_url)
            .await
    };

    match pool().await {
        Ok(p) => Ok(p),
        Err(e) => {
            if let Some(db_error) = e.as_database_error()
                && db_error.code().unwrap_or("NaN".into()) == "3D000"
            {
                Postgres::create_database(&database_url).await?;
                let pool = pool().await?;
                sqlx::migrate!("../migrations").run(&pool).await?;

                return Ok(pool);
            }

            Err(e.into())
        }
    }
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
