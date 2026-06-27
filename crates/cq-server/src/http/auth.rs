use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use cq_db::repositories::account::AccountRepository;
use cq_protocol::rest::{ApiOk, LoginRequest, LoginResponse, RegisterRequest};
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

pub async fn register(
    State(state): State<AppState>,
    Json(input): Json<RegisterRequest>,
) -> Result<Json<ApiOk<serde_json::Value>>, ApiError> {
    validate_credentials(&input.username, &input.password)?;
    let repo = AccountRepository::new(state.db.pool());
    if repo.find_by_username(&input.username).await?.is_some() {
        return Err(ApiError::Conflict("用户名已存在".into()));
    }
    let password_hash = hash_password(&input.password)?;
    let account = repo.create(&input.username, &password_hash, input.email.as_deref()).await?;
    Ok(Json(ApiOk::new(json!({
        "account_id": account.id,
        "username": account.username,
        "status": account.status
    }))))
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<ApiOk<LoginResponse>>, ApiError> {
    let repo = AccountRepository::new(state.db.pool());
    let account = repo.find_by_username(&input.username).await?.ok_or(ApiError::Unauthorized)?;
    if account.status().as_str() == "banned" {
        return Err(ApiError::Unauthorized);
    }
    verify_password(&input.password, &account.password_hash)?;
    let token = Uuid::new_v4().to_string();
    let session = repo
        .create_session(
            account.id,
            &token,
            state.config.session_ttl_min,
            input.device.as_deref(),
            None,
        )
        .await?;
    Ok(Json(ApiOk::new(LoginResponse {
        account_id: account.id,
        token: session.token,
        expires_at: session
            .expires_at
            .format(&Rfc3339)
            .unwrap_or_else(|_| session.expires_at.to_string()),
    })))
}

fn validate_credentials(username: &str, password: &str) -> Result<(), ApiError> {
    let username_ok = username.len() >= 3
        && username.len() <= 24
        && username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !username_ok {
        return Err(ApiError::BadRequest("用户名需为 3-24 位字母、数字、下划线或横线".into()));
    }
    if password.len() < 8 {
        return Err(ApiError::BadRequest("密码至少 8 位".into()));
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|err| ApiError::Internal(err.to_string()))
}

fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    let parsed = PasswordHash::new(hash).map_err(|_| ApiError::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| ApiError::Unauthorized)
}
