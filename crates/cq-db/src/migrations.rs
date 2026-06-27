use std::{
    fs,
    path::{Path, PathBuf},
};

use sqlx::PgPool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("migration io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

pub async fn run(pool: &PgPool, migrations_dir: impl AsRef<Path>) -> Result<(), MigrationError> {
    sqlx::query(
        "create table if not exists schema_migrations (version text primary key, applied_at timestamptz not null default now())",
    )
    .execute(pool)
    .await?;

    for file in migration_files(migrations_dir.as_ref())? {
        let version = file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let applied: Option<(i32,)> =
            sqlx::query_as("select 1 from schema_migrations where version = $1")
                .bind(&version)
                .fetch_optional(pool)
                .await?;
        if applied.is_some() {
            continue;
        }

        let sql = fs::read_to_string(&file)?;
        let mut tx = pool.begin().await?;
        sqlx::raw_sql(&sql).execute(&mut *tx).await?;
        sqlx::query("insert into schema_migrations (version) values ($1)")
            .bind(&version)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }

    Ok(())
}

fn migration_files(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("sql"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}
