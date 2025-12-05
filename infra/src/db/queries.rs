use std::sync::Arc;

use sqlx::{
    FromRow, PgPool, Postgres, Transaction,
    postgres::{PgArguments, PgRow},
    query::QueryAs,
};
use uuid::Uuid;

pub enum Binds {
    String(String),
    I32(i32),
    I64(i64),
    Bool(bool),
    Uuid(Uuid),
}

pub async fn insert<M>(
    sql: &str,
    binds: Vec<Binds>,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<M, sqlx::Error>
where
    M: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let query = sqlx::query_as::<_, M>(sql);
    let query = apply_bind_to_query(query, binds);
    let query = query.fetch_one(&mut **tx).await?;

    Ok(query)
}

pub async fn fetch<M>(
    sql: &str,
    binds: Vec<Binds>,
    tx: Option<&mut Transaction<'_, Postgres>>,
    db_pool: Option<Arc<PgPool>>,
) -> Result<Vec<M>, sqlx::Error>
where
    M: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let query = sqlx::query_as::<_, M>(sql);
    let query = apply_bind_to_query(query, binds);

    if let Some(t) = tx {
        let query = query.fetch_all(&mut **t).await?;
        Ok(query)
    } else if let Some(d) = db_pool {
        let query = query.fetch_all(&*d).await?;
        Ok(query)
    } else {
        Ok(Vec::new())
    }
}

pub async fn delete<M>(
    sql: &str,
    binds: Vec<Binds>,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error>
where
    M: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let query = sqlx::query_as::<_, M>(sql);
    let query = apply_bind_to_query(query, binds);
    query.fetch_optional(&mut **tx).await?;

    Ok(())
}

fn apply_bind_to_query<M>(
    mut query: QueryAs<'_, Postgres, M, PgArguments>,
    binds: Vec<Binds>,
) -> QueryAs<'_, Postgres, M, PgArguments>
where
    M: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    for bind in binds {
        match bind {
            Binds::String(v) => {
                query = query.bind(v);
            }
            Binds::I32(v) => {
                query = query.bind(v);
            }
            Binds::I64(v) => {
                query = query.bind(v);
            }
            Binds::Bool(v) => {
                query = query.bind(v);
            }
            Binds::Uuid(v) => {
                query = query.bind(v);
            }
        };
    }

    query
}
