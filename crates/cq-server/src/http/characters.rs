use axum::{extract::State, http::HeaderMap, Json};
use cq_db::repositories::{
    account::AccountRepository, character::CharacterRepository, systems::SystemsRepository,
};
use cq_domain::character::CharacterClass;
use cq_protocol::rest::{ApiOk, CreateCharacterRequest};
use serde_json::json;

use crate::{error::ApiError, http::bearer_token, state::AppState};

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<serde_json::Value>>, ApiError> {
    let token = bearer_token(&headers)?;
    let session = AccountRepository::new(state.db.pool())
        .find_session(token)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    let characters = CharacterRepository::new(state.db.pool()).list_for_account(session.account_id).await?;
    Ok(Json(ApiOk::new(json!({ "characters": characters }))))
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateCharacterRequest>,
) -> Result<Json<ApiOk<serde_json::Value>>, ApiError> {
    let token = bearer_token(&headers)?;
    let session = AccountRepository::new(state.db.pool())
        .find_session(token)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    if input.name.trim().len() < 2 || input.name.trim().len() > 16 {
        return Err(ApiError::BadRequest("角色名需为 2-16 个字符".into()));
    }
    if matches!(input.class, CharacterClass::Assassin) {
        return Err(ApiError::BadRequest("职业体系仅开放剑修、法修、魂修创建。".into()));
    }
    let repo = CharacterRepository::new(state.db.pool());
    let character = repo
        .create(session.account_id, input.name.trim(), input.class)
        .await
        .map_err(create_character_error)?;
    if let Err(err) = SystemsRepository::new(state.db.pool()).ensure_starter(character.id).await {
        tracing::warn!(error = ?err, character_id = character.id, "failed to grant starter systems");
    }
    Ok(Json(ApiOk::new(json!({ "character": character }))))
}

fn create_character_error(err: sqlx::Error) -> ApiError {
    match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ApiError::Conflict("角色名已存在".into())
        }
        _ => ApiError::Database(err),
    }
}
