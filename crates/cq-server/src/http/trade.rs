use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use cq_db::repositories::{
    account::AccountRepository,
    character::{character_view, CharacterRecord, CharacterRepository},
    inventory::InventoryRepository,
    trade::{TradeError, TradeRepository},
};
use cq_protocol::{
    dto::{
        CreateConsignmentRequest, TradeActionRequest, TradeActionResult, TradeList,
    },
    rest::ApiOk,
};
use serde::Deserialize;

use crate::{error::ApiError, http::bearer_token, state::AppState};

#[derive(Debug, Clone, Deserialize)]
pub struct TradeQuery {
    pub character_id: i64,
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TradeQuery>,
) -> Result<Json<ApiOk<TradeList>>, ApiError> {
    let character = resolve_character(&state, &headers, query.character_id).await?;
    let consignments = TradeRepository::new(state.db.pool()).list(character.id).await?;
    Ok(Json(ApiOk::new(TradeList { consignments })))
}

pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateConsignmentRequest>,
) -> Result<Json<ApiOk<TradeActionResult>>, ApiError> {
    let character = resolve_character(&state, &headers, input.character_id).await?;
    let consignments = TradeRepository::new(state.db.pool())
        .create(
            character.id,
            input.item_id,
            input.price,
            input.price_currency.as_deref().unwrap_or("yuanbao"),
        )
        .await
        .map_err(trade_error)?;
    action_result(&state, character.id, consignments, "上架成功").await
}

pub async fn buy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<TradeActionRequest>,
) -> Result<Json<ApiOk<TradeActionResult>>, ApiError> {
    let character = resolve_character(&state, &headers, input.character_id).await?;
    let consignments = TradeRepository::new(state.db.pool())
        .buy(character.id, input.consignment_id)
        .await
        .map_err(trade_error)?;
    action_result(&state, character.id, consignments, "购买成功").await
}

pub async fn cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<TradeActionRequest>,
) -> Result<Json<ApiOk<TradeActionResult>>, ApiError> {
    let character = resolve_character(&state, &headers, input.character_id).await?;
    let consignments = TradeRepository::new(state.db.pool())
        .cancel(character.id, input.consignment_id)
        .await
        .map_err(trade_error)?;
    action_result(&state, character.id, consignments, "下架成功").await
}

async fn resolve_character(
    state: &AppState,
    headers: &HeaderMap,
    character_id: i64,
) -> Result<CharacterRecord, ApiError> {
    let token = bearer_token(headers)?;
    let session = AccountRepository::new(state.db.pool())
        .find_session(token)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    CharacterRepository::new(state.db.pool())
        .find_for_account(session.account_id, character_id)
        .await?
        .ok_or(ApiError::NotFound)
}

async fn action_result(
    state: &AppState,
    character_id: i64,
    consignments: Vec<cq_protocol::dto::TradeConsignmentView>,
    message: &str,
) -> Result<Json<ApiOk<TradeActionResult>>, ApiError> {
    let character_repo = CharacterRepository::new(state.db.pool());
    let character = character_repo.find(character_id).await?.ok_or(ApiError::NotFound)?;
    let inventory = InventoryRepository::new(state.db.pool())
        .view(character.id, character.level)
        .await?;
    Ok(Json(ApiOk::new(TradeActionResult {
        consignments,
        inventory,
        character: character_view(character),
        message: message.into(),
    })))
}

fn trade_error(err: TradeError) -> ApiError {
    match err {
        TradeError::NotFound => ApiError::NotFound,
        TradeError::InvalidPrice => ApiError::BadRequest("价格必须大于 0".into()),
        TradeError::ItemUnavailable => ApiError::BadRequest("物品不存在、已绑定或不在背包中".into()),
        TradeError::OwnConsignment => ApiError::BadRequest("不能购买自己的寄售".into()),
        TradeError::NotEnoughYuanbao => ApiError::BadRequest("元宝不足".into()),
        TradeError::NotEnoughGold => ApiError::BadRequest("金币不足".into()),
        TradeError::BagFull => ApiError::BadRequest("背包已满".into()),
        TradeError::Database(err) => ApiError::Database(err),
    }
}
