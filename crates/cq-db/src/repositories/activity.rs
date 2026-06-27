use serde::Serialize;
use sqlx::{FromRow, PgPool};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ActivityRecord {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub points: i64,
}

#[derive(Debug, Clone)]
pub struct ActivityRepository {
    pool: PgPool,
}

impl ActivityRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn enabled(&self) -> Result<Vec<ActivityRecord>, sqlx::Error> {
        sqlx::query_as::<_, ActivityRecord>(
            "select id, code, name, enabled, config, 0::bigint as points from activities where enabled = true order by id asc",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn enabled_for_character(&self, character_id: i64) -> Result<Vec<ActivityRecord>, sqlx::Error> {
        sqlx::query_as::<_, ActivityRecord>(
            r#"
            select
              a.id,
              a.code,
              a.name,
              a.enabled,
              a.config,
              coalesce(ap.points, 0) as points
            from activities a
            left join activity_points ap on ap.activity_code = a.code and ap.character_id = $1
            where a.enabled = true
            order by a.id asc
            "#,
        )
        .bind(character_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn add_points(&self, character_id: i64, activity_code: &str, points: i64) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as(
            r#"
            insert into activity_points (character_id, activity_code, points)
            values ($1, $2, greatest(0, $3))
            on conflict (character_id, activity_code) do update set
              points = activity_points.points + greatest(0, excluded.points),
              updated_at = now()
            returning points
            "#,
        )
        .bind(character_id)
        .bind(activity_code)
        .bind(points)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }
}
