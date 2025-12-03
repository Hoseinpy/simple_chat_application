use std::{env, time::Duration};

use shared::types::DefaultError;
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};

pub mod models;

pub async fn create_pool() -> Result<PgPool, DefaultError> {
    let is_test_mode = cfg!(test);

    let (var_name, expect_err) = if is_test_mode {
        ("TEST_DATABASE_URL", "TEST_DATABASE_URL most be set in .env")
    } else {
        ("DATABASE_URL", "DATABASE_URL most be set in .env")
    };
    let database_url = env::var(var_name).expect(expect_err);

    let (min_conn, max_conn) = if is_test_mode { (0, 1) } else { (5, 30) };
    let pool = async || -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(max_conn)
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
