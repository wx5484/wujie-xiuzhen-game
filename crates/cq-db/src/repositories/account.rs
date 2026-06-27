use cq_domain::account::AccountStatus;
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use time::{Duration, OffsetDateTime};

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AccountRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub status: String,
}

impl AccountRecord {
    pub fn status(&self) -> AccountStatus {
        match self.status.as_str() {
            "muted" => AccountStatus::Muted,
            "banned" => AccountStatus::Banned,
            _ => AccountStatus::Active,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SessionRecord {
    pub id: i64,
    pub account_id: i64,
    pub token: String,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct AccountRepository {
    pool: PgPool,
}

impl AccountRepository {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn create(
        &self,
        username: &str,
        password_hash: &str,
        email: Option<&str>,
    ) -> Result<AccountRecord, sqlx::Error> {
        sqlx::query_as::<_, AccountRecord>(
            r#"
            insert into accounts (username, password_hash, email, status)
            values ($1, $2, $3, 'active')
            returning id, username, password_hash, email, status
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_username(
        &self,
        username: &str,
    ) -> Result<Option<AccountRecord>, sqlx::Error> {
        sqlx::query_as::<_, AccountRecord>(
            "select id, username, password_hash, email, status from accounts where username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn create_session(
        &self,
        account_id: i64,
        token: &str,
        ttl_minutes: i64,
        device: Option<&str>,
        ip: Option<&str>,
    ) -> Result<SessionRecord, sqlx::Error> {
        let expires_at = OffsetDateTime::now_utc() + Duration::minutes(ttl_minutes);
        sqlx::query_as::<_, SessionRecord>(
            r#"
            insert into sessions (account_id, token, expires_at, device, ip)
            values ($1, $2, $3, $4, $5::inet)
            returning id, account_id, token, expires_at
            "#,
        )
        .bind(account_id)
        .bind(token)
        .bind(expires_at)
        .bind(device)
        .bind(ip)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_session(&self, token: &str) -> Result<Option<SessionRecord>, sqlx::Error> {
        sqlx::query_as::<_, SessionRecord>(
            r#"
            select id, account_id, token, expires_at
            from sessions
            where token = $1 and revoked_at is null and expires_at > now()
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
    }
}
