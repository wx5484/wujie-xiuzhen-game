use axum::{extract::{Query, State}, http::HeaderMap, Json};
use cq_admin::operations;
use cq_db::repositories::{admin::AdminRepository, bot::BotRepository};
use cq_protocol::{
    dto::{
        AdminAccountList, AdminAuditLogList, AdminBotActionResult, AdminBotBatchRequest, AdminBotCreateRequest,
        AdminBotDeleteRequest, AdminBotList, AdminBotTickRequest, AdminCharacterDetail, AdminCharacterList,
        AdminGmActionRequest,
        AdminItemTemplateInput, AdminItemTemplateList, AdminMailOverview, AdminMobTemplateList, DashboardSummary,
    },
    rest::ApiOk,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{error::ApiError, http::require_admin, state::AppState};

const COFFEE_QR_KEY: &str = "coffee_qr_url";

#[derive(Debug, Clone, Deserialize)]
pub struct AdminLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminLoginResult {
    pub username: String,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminSettingsRequest {
    pub coffee_qr_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminSettingsView {
    pub coffee_qr_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminSendMailRequest {
    pub to_character_id: Option<i64>,
    pub title: String,
    pub body: String,
    pub gold: Option<i64>,
    pub yuanbao: Option<i64>,
    pub item_template_id: Option<String>,
    pub quantity: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminAdjustCharacterRequest {
    pub character_id: i64,
    pub exp_delta: Option<i64>,
    pub gold_delta: Option<i64>,
    pub yuanbao_delta: Option<i64>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminCharacterDetailQuery {
    pub character_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminUpdateCharacterRequest {
    pub character_id: i64,
    pub name: Option<String>,
    pub class: Option<String>,
    pub level: Option<i32>,
    pub exp: Option<i64>,
    pub gold: Option<i64>,
    pub yuanbao: Option<i64>,
    pub power: Option<i64>,
    pub zone: Option<String>,
    pub room: Option<String>,
    pub hp: Option<i64>,
    pub mp: Option<i64>,
    pub online: Option<bool>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminCharacterItemRequest {
    pub character_id: i64,
    pub item_id: Option<i64>,
    pub template_id: String,
    pub quantity: Option<i64>,
    pub location: String,
    pub slot: Option<String>,
    #[serde(default)]
    pub bind: bool,
    pub durability: Option<i32>,
    #[serde(default)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminDeleteCharacterItemRequest {
    pub character_id: i64,
    pub item_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminResetChallengeCooldownsRequest {
    pub character_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminActionResult {
    pub message: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<AdminLoginRequest>,
) -> Result<Json<ApiOk<AdminLoginResult>>, ApiError> {
    let expected_user = state.config.admin_bootstrap_user.as_str();
    let expected_password = state.config.admin_bootstrap_password.as_str();
    if input.username.trim() != expected_user || input.password != expected_password {
        return Err(ApiError::Unauthorized);
    }
    Ok(Json(ApiOk::new(AdminLoginResult {
        username: state.config.admin_bootstrap_user.clone(),
        token: state.config.admin_bootstrap_password.clone(),
    })))
}

pub async fn settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminSettingsView>>, ApiError> {
    require_admin(&headers, &state.config)?;
    Ok(Json(ApiOk::new(AdminSettingsView {
        coffee_qr_url: read_coffee_qr(state.db.pool()).await?,
    })))
}

pub async fn save_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminSettingsRequest>,
) -> Result<Json<ApiOk<AdminSettingsView>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let coffee_qr_url = input.coffee_qr_url.trim();
    if coffee_qr_url.chars().count() > 20000 {
        return Err(ApiError::BadRequest("二维码地址或 data URL 不能超过 20000 个字符。".into()));
    }
    sqlx::query(
        r#"
        insert into game_settings (key, value, version)
        values ($1, $2, 1)
        on conflict (key) do update
        set value = excluded.value,
            version = game_settings.version + 1,
            updated_at = now()
        "#,
    )
    .bind(COFFEE_QR_KEY)
    .bind(json!(coffee_qr_url))
    .execute(state.db.pool())
    .await
    .map_err(admin_write_error)?;
    AdminRepository::new(state.db.pool())
        .audit(None, "save_settings", "game_settings", json!({ "key": COFFEE_QR_KEY }))
        .await?;
    Ok(Json(ApiOk::new(AdminSettingsView {
        coffee_qr_url: coffee_qr_url.to_string(),
    })))
}

pub async fn coffee_qr_public(State(state): State<AppState>) -> Result<Json<ApiOk<AdminSettingsView>>, ApiError> {
    Ok(Json(ApiOk::new(AdminSettingsView {
        coffee_qr_url: read_coffee_qr(state.db.pool()).await?,
    })))
}

pub async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<DashboardSummary>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let summary = operations::dashboard(&repo)
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(summary)))
}

pub async fn accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminAccountList>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let accounts = operations::accounts(&repo)
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(accounts)))
}

pub async fn clear_all_accounts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let pool = state.db.pool();
    let (deleted_accounts,): (i64,) = sqlx::query_as(
        r#"
        with deleted as (
          delete from accounts
          returning id
        )
        select count(*)::bigint from deleted
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(admin_write_error)?;
    let (deleted_guilds,): (i64,) = sqlx::query_as(
        r#"
        with deleted as (
          delete from guilds
          returning id
        )
        select count(*)::bigint from deleted
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(admin_write_error)?;
    let (deleted_rankings,): (i64,) = sqlx::query_as(
        r#"
        with deleted as (
          delete from ranking_snapshots
          returning id
        )
        select count(*)::bigint from deleted
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(admin_write_error)?;
    let (deleted_sabak,): (i64,) = sqlx::query_as(
        r#"
        with deleted as (
          delete from sabak_campaigns
          returning id
        )
        select count(*)::bigint from deleted
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(admin_write_error)?;
    sqlx::query("delete from guild_sabak_state")
        .execute(pool)
        .await
        .map_err(admin_write_error)?;
    AdminRepository::new(pool)
        .audit(
            None,
            "clear_all_accounts",
            "accounts",
            json!({
                "deleted_accounts": deleted_accounts,
                "deleted_guilds": deleted_guilds,
                "deleted_rankings": deleted_rankings,
                "deleted_sabak": deleted_sabak,
            }),
        )
        .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: format!(
            "已清除 {} 个账号及其角色数据，并清理 {} 个行会、{} 个排行榜快照、{} 场沙巴克记录。",
            deleted_accounts, deleted_guilds, deleted_rankings, deleted_sabak
        ),
    })))
}

pub async fn characters(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminCharacterList>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let characters = operations::characters(&repo)
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(characters)))
}

pub async fn character_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AdminCharacterDetailQuery>,
) -> Result<Json<ApiOk<AdminCharacterDetail>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let detail = repo.character_detail(query.character_id).await.map_err(admin_write_error)?;
    Ok(Json(ApiOk::new(detail)))
}

pub async fn mail_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminMailOverview>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let overview = operations::mail_overview(&repo)
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(overview)))
}

pub async fn item_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminItemTemplateList>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let templates = operations::item_templates(&repo)
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(templates)))
}

pub async fn mob_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminMobTemplateList>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let templates = operations::mob_templates(&repo)
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(templates)))
}

pub async fn audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminAuditLogList>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    let logs = operations::audit_logs(&repo)
        .await
        .map_err(|err| ApiError::Internal(err.to_string()))?;
    Ok(Json(ApiOk::new(logs)))
}

pub async fn bots(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ApiOk<AdminBotList>>, ApiError> {
    require_admin(&headers, &state.config)?;
    Ok(Json(ApiOk::new(BotRepository::new(state.db.pool()).list().await?)))
}

pub async fn bots_batch(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminBotBatchRequest>,
) -> Result<Json<ApiOk<AdminBotActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = BotRepository::new(state.db.pool());
    let bots = repo.batch_config(input.clone()).await.map_err(admin_write_error)?;
    AdminRepository::new(state.db.pool())
        .audit(
            None,
            "bot_batch_config",
            "bot",
            json!({
                "bot_ids": input.bot_ids,
                "mode": input.mode,
                "enabled": input.enabled,
                "zone": input.zone,
                "room": input.room,
                "team_code": input.team_code,
                "target_zone": input.target_zone,
                "target_room": input.target_room,
            }),
        )
        .await?;
    Ok(Json(ApiOk::new(AdminBotActionResult { bots, message: "Bot 批量配置已保存。".into() })))
}

pub async fn bots_create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminBotCreateRequest>,
) -> Result<Json<ApiOk<AdminBotActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let name_len = input.name.trim().chars().count();
    if !(2..=16).contains(&name_len) {
        return Err(ApiError::BadRequest("Bot 名称长度需要在 2-16 个字符之间。".into()));
    }
    let bots = BotRepository::new(state.db.pool())
        .create(input.clone())
        .await
        .map_err(admin_write_error)?;
    AdminRepository::new(state.db.pool())
        .audit(
            None,
            "bot_create",
            "bot",
            json!({ "name": input.name, "bot_class": input.bot_class }),
        )
        .await?;
    Ok(Json(ApiOk::new(AdminBotActionResult { bots, message: "Bot 已新增。".into() })))
}

pub async fn bots_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminBotDeleteRequest>,
) -> Result<Json<ApiOk<AdminBotActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let bots = BotRepository::new(state.db.pool())
        .delete(input.bot_id)
        .await
        .map_err(admin_write_error)?;
    AdminRepository::new(state.db.pool())
        .audit(None, "bot_delete", "bot", json!({ "bot_id": input.bot_id }))
        .await?;
    Ok(Json(ApiOk::new(AdminBotActionResult { bots, message: "Bot 已删除。".into() })))
}

pub async fn bots_tick(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminBotTickRequest>,
) -> Result<Json<ApiOk<AdminBotActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let limit = input.limit.unwrap_or(50).clamp(1, 200);
    let (bots, changed) = BotRepository::new(state.db.pool())
        .tick(limit)
        .await
        .map_err(admin_write_error)?;
    AdminRepository::new(state.db.pool())
        .audit(None, "bot_tick", "bot", json!({ "limit": limit, "changed": changed }))
        .await?;
    Ok(Json(ApiOk::new(AdminBotActionResult {
        bots,
        message: format!("Bot 脚本推进完成：{} 个 bot 已行动。", changed),
    })))
}

pub async fn send_mail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminSendMailRequest>,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let title = input.title.trim();
    let body = input.body.trim();
    if title.is_empty() || body.is_empty() {
        return Err(ApiError::BadRequest("邮件标题和正文不能为空".into()));
    }
    let item_template_id = input
        .item_template_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let repo = AdminRepository::new(state.db.pool());
    let count = repo
        .send_system_mail(
            input.to_character_id,
            title,
            body,
            input.gold.unwrap_or_default(),
            input.yuanbao.unwrap_or_default(),
            item_template_id,
            input.quantity.unwrap_or(1),
        )
        .await
        .map_err(admin_write_error)?;
    repo.audit(
        None,
        "send_mail",
        "mail",
        json!({
            "to_character_id": input.to_character_id,
            "title": title,
            "count": count,
            "gold": input.gold.unwrap_or_default(),
            "yuanbao": input.yuanbao.unwrap_or_default(),
            "item_template_id": item_template_id,
        }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: format!("已发送 {} 封邮件。", count),
    })))
}

pub async fn adjust_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminAdjustCharacterRequest>,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let exp_delta = input.exp_delta.unwrap_or_default();
    let gold_delta = input.gold_delta.unwrap_or_default();
    let yuanbao_delta = input.yuanbao_delta.unwrap_or_default();
    if exp_delta == 0 && gold_delta == 0 && yuanbao_delta == 0 {
        return Err(ApiError::BadRequest("调整数值不能全为 0".into()));
    }
    let repo = AdminRepository::new(state.db.pool());
    repo.adjust_character_assets(input.character_id, exp_delta, gold_delta, yuanbao_delta)
        .await
        .map_err(admin_write_error)?;
    repo.audit(
        None,
        "adjust_character",
        "character",
        json!({
            "character_id": input.character_id,
            "exp_delta": exp_delta,
            "gold_delta": gold_delta,
            "yuanbao_delta": yuanbao_delta,
            "reason": input.reason.unwrap_or_default(),
        }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: "角色资产已调整。".into(),
    })))
}

pub async fn update_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminUpdateCharacterRequest>,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    if let Some(name) = input.name.as_deref() {
        let len = name.trim().chars().count();
        if len > 0 && !(2..=16).contains(&len) {
            return Err(ApiError::BadRequest("角色名需为 2-16 个字符".into()));
        }
    }
    if let Some(class) = input.class.as_deref() {
        if !matches!(class.trim(), "warrior" | "mage" | "taoist" | "assassin") {
            return Err(ApiError::BadRequest("职业必须是 warrior/mage/taoist/assassin".into()));
        }
    }
    let repo = AdminRepository::new(state.db.pool());
    repo.update_character_detail(
        input.character_id,
        input.name.as_deref(),
        input.class.as_deref(),
        input.level,
        input.exp,
        input.gold,
        input.yuanbao,
        input.power,
        input.zone.as_deref(),
        input.room.as_deref(),
        input.hp,
        input.mp,
        input.online,
    )
    .await
    .map_err(admin_write_error)?;
    repo.audit(
        None,
        "update_character_detail",
        "character",
        json!({
            "character_id": input.character_id,
            "name": input.name,
            "class": input.class,
            "level": input.level,
            "exp": input.exp,
            "gold": input.gold,
            "yuanbao": input.yuanbao,
            "power": input.power,
            "zone": input.zone,
            "room": input.room,
            "hp": input.hp,
            "mp": input.mp,
            "online": input.online,
            "reason": input.reason.unwrap_or_default(),
        }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: "角色详情已保存。".into(),
    })))
}

pub async fn upsert_character_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminCharacterItemRequest>,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let template_id = input.template_id.trim();
    if template_id.is_empty() {
        return Err(ApiError::BadRequest("物品模板 ID 不能为空".into()));
    }
    let location = input.location.trim();
    if !matches!(location, "bag" | "warehouse" | "equipped") {
        return Err(ApiError::BadRequest("位置必须是 bag/warehouse/equipped".into()));
    }
    if !input.extra.is_object() {
        return Err(ApiError::BadRequest("extra 必须是 JSON 对象".into()));
    }
    let slot = input.slot.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let repo = AdminRepository::new(state.db.pool());
    repo.upsert_character_item(
        input.character_id,
        input.item_id,
        template_id,
        input.quantity.unwrap_or(1),
        location,
        slot,
        input.bind,
        input.durability.unwrap_or(100),
        &input.extra,
    )
    .await
    .map_err(admin_write_error)?;
    repo.audit(
        None,
        "upsert_character_item",
        "inventory_item",
        json!({
            "character_id": input.character_id,
            "item_id": input.item_id,
            "template_id": template_id,
            "quantity": input.quantity.unwrap_or(1),
            "location": location,
            "slot": slot,
            "bind": input.bind,
        }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: if input.item_id.is_some() { "角色物品已修改。".into() } else { "角色物品已新增。".into() },
    })))
}

pub async fn delete_character_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminDeleteCharacterItemRequest>,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let repo = AdminRepository::new(state.db.pool());
    repo.delete_character_item(input.character_id, input.item_id)
        .await
        .map_err(admin_write_error)?;
    repo.audit(
        None,
        "delete_character_item",
        "inventory_item",
        json!({ "character_id": input.character_id, "item_id": input.item_id }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: "角色物品已删除。".into(),
    })))
}

pub async fn toggle_activity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminGmActionRequest>,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    if input.action.as_deref() == Some("set_account_status") {
        return set_account_status_action(state, input).await;
    }
    if input.action.as_deref() == Some("set_character_state") {
        return set_character_state_action(state, input).await;
    }
    if input.action.as_deref() == Some("upsert_item_template") {
        return upsert_item_template_action(state, input).await;
    }
    let code = input.code.as_deref().unwrap_or_default().trim();
    let enabled = input.enabled;
    if code.is_empty() {
        return Err(ApiError::BadRequest("活动 code 不能为空".into()));
    }
    let repo = AdminRepository::new(state.db.pool());
    repo.set_activity_enabled(code, enabled)
        .await
        .map_err(admin_write_error)?;
    repo.audit(
        None,
        "toggle_activity",
        "activity",
        json!({ "code": code, "enabled": enabled }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: if input.enabled { "活动已开启。".into() } else { "活动已关闭。".into() },
    })))
}

pub async fn reset_challenge_cooldowns(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdminResetChallengeCooldownsRequest>,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    require_admin(&headers, &state.config)?;
    let pool = state.db.pool();
    let affected = if let Some(character_id) = input.character_id {
        sqlx::query(
            r#"
            insert into character_challenge_state (character_id)
            values ($1)
            on conflict (character_id) do nothing
            "#,
        )
        .bind(character_id)
        .execute(pool)
        .await
        .map_err(admin_write_error)?;
        sqlx::query(
            r#"
            update character_challenge_state
            set
              secret_realm_available_at = now(),
              tower_available_at = now(),
              world_boss_available_at = now(),
              updated_at = now()
            where character_id = $1
            "#,
        )
        .bind(character_id)
        .execute(pool)
        .await
        .map_err(admin_write_error)?
        .rows_affected()
    } else {
        sqlx::query(
            r#"
            update character_challenge_state
            set
              secret_realm_available_at = now(),
              tower_available_at = now(),
              world_boss_available_at = now(),
              updated_at = now()
            "#,
        )
        .execute(pool)
        .await
        .map_err(admin_write_error)?
        .rows_affected()
    };
    let repo = AdminRepository::new(pool);
    repo.audit(
        None,
        "reset_challenge_cooldowns",
        "challenge_state",
        json!({ "character_id": input.character_id, "affected": affected }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: match input.character_id {
            Some(character_id) => format!("已重置角色 #{} 的秘境、无尽塔、世界首领挑战冷却。", character_id),
            None => format!("已重置所有挑战冷却，影响 {} 条挑战状态。", affected),
        },
    })))
}

async fn set_account_status_action(
    state: AppState,
    input: AdminGmActionRequest,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    let account_id = input
        .account_id
        .ok_or_else(|| ApiError::BadRequest("account_id is required".into()))?;
    let status = input.status.as_deref().unwrap_or_default().trim();
    if !matches!(status, "active" | "muted" | "banned") {
        return Err(ApiError::BadRequest("status must be active, muted, or banned".into()));
    }
    let reason = input.reason.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let repo = AdminRepository::new(state.db.pool());
    repo.set_account_status(account_id, status, reason)
        .await
        .map_err(admin_write_error)?;
    repo.audit(
        None,
        "set_account_status",
        "account",
        json!({ "account_id": account_id, "status": status, "reason": reason }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: format!("account #{} status set to {}", account_id, status),
    })))
}

async fn set_character_state_action(
    state: AppState,
    input: AdminGmActionRequest,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    let character_id = input
        .character_id
        .ok_or_else(|| ApiError::BadRequest("character_id is required".into()))?;
    let zone = input.zone.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let room = input.room.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let force_offline = input.force_offline.unwrap_or(false);
    if zone.is_none()
        && room.is_none()
        && input.hp.is_none()
        && input.mp.is_none()
        && input.online.is_none()
        && !force_offline
    {
        return Err(ApiError::BadRequest("no character state changes provided".into()));
    }
    let reason = input.reason.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let repo = AdminRepository::new(state.db.pool());
    repo.set_character_state(
        character_id,
        zone,
        room,
        input.hp,
        input.mp,
        input.online,
        force_offline,
        reason,
    )
    .await
    .map_err(admin_write_error)?;
    repo.audit(
        None,
        "set_character_state",
        "character",
        json!({
            "character_id": character_id,
            "zone": zone,
            "room": room,
            "hp": input.hp,
            "mp": input.mp,
            "online": input.online,
            "force_offline": force_offline,
            "reason": reason,
        }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: format!("character #{} state updated", character_id),
    })))
}

async fn upsert_item_template_action(
    state: AppState,
    input: AdminGmActionRequest,
) -> Result<Json<ApiOk<AdminActionResult>>, ApiError> {
    let item = input
        .item
        .ok_or_else(|| ApiError::BadRequest("item is required".into()))?;
    validate_item_template(&item)?;
    let repo = AdminRepository::new(state.db.pool());
    repo.upsert_item_template(&item)
        .await
        .map_err(admin_write_error)?;
    repo.audit(
        None,
        "upsert_item_template",
        "item_template",
        json!({
            "id": item.id.trim(),
            "name": item.name.trim(),
            "kind": item.kind.trim(),
            "slot": item.slot.as_deref().map(str::trim),
            "rarity": item.rarity.trim(),
            "price": item.price.max(0),
            "stackable": item.stackable,
            "stats": item.stats.clone(),
            "flags": item.flags.clone(),
        }),
    )
    .await?;
    Ok(Json(ApiOk::new(AdminActionResult {
        message: format!("item template {} saved", item.id.trim()),
    })))
}

fn validate_item_template(item: &AdminItemTemplateInput) -> Result<(), ApiError> {
    if item.id.trim().is_empty() || item.name.trim().is_empty() || item.kind.trim().is_empty() {
        return Err(ApiError::BadRequest("item id, name, and kind are required".into()));
    }
    if item.rarity.trim().is_empty() {
        return Err(ApiError::BadRequest("item rarity is required".into()));
    }
    if !item.stats.is_object() || !item.flags.is_object() {
        return Err(ApiError::BadRequest("item stats and flags must be JSON objects".into()));
    }
    Ok(())
}

async fn read_coffee_qr(pool: &sqlx::PgPool) -> Result<String, ApiError> {
    let value: Option<serde_json::Value> = sqlx::query_scalar("select value from game_settings where key = $1")
        .bind(COFFEE_QR_KEY)
        .fetch_optional(pool)
        .await
        .map_err(admin_write_error)?;
    Ok(value
        .and_then(|entry| entry.as_str().map(ToOwned::to_owned))
        .unwrap_or_default())
}

fn admin_write_error(err: sqlx::Error) -> ApiError {
    match err {
        sqlx::Error::RowNotFound => ApiError::NotFound,
        err => ApiError::Database(err),
    }
}
