use sqlx::{postgres::PgPoolOptions, PgPool};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct Db {
    pool: PgPool,
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl Db {
    pub fn connect_lazy(config: &DbConfig) -> Result<Self, DbError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect_lazy(&config.database_url)?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn ready(&self) -> Result<(), DbError> {
        sqlx::query("select 1").execute(&self.pool).await?;
        Ok(())
    }
}
