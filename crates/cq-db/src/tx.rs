use sqlx::{PgPool, Postgres, Transaction};

pub async fn begin(pool: &PgPool) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
    pool.begin().await
}
