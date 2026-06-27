use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use cq_db::repositories::{
    account::AccountRepository,
    activity::ActivityRepository,
    adventure::{AdventureOfferView, AdventureRepository, AdventureResolveView},
    afk::AfkRepository,
    bot::{BotPvpTarget, BotRepository},
    character::{character_view, state_view, stats_view, CharacterRecord, CharacterRepository, CharacterStateRecord, StaminaConsumption},
    guild::{GuildActionError, GuildApplicationRecord, GuildJoinError, GuildRecord, GuildRepository},
    inventory::{ConsumedPotion, EquipmentBonus, InventoryActionError, InventoryRepository},
    mail::MailRepository,
    quest::{QuestError, QuestRepository},
    skill::{ActiveSkillRecord, SkillBonus, SkillRepository, SkillUpgradeError},
    systems::{SystemBonus, SystemsActionError, SystemsRepository},
};
use cq_domain::{
    character::{initial_stats, CharacterClass},
    combat::{magical_damage, physical_damage, roll_control, Combatant, DamageReport},
    map::{death_return_name, death_return_position, start_position, Position},
    mob::MobTemplate,
};
use cq_game::drop::{roll_level_drop_for_source, MobDropKind};
use cq_protocol::{
    dto::{
        AfkSettleResult, AfkStatusView, CharacterBundle, GameBootstrap, GameOverview,
        CreateGuildRequest, GuildApplyRequest, GuildBenefitRequest, GuildBenefitResult, GuildDonateRequest, GuildReviewApplicationRequest, GuildTaskRequest,
        InventoryActionResult, InventoryView, JoinGuildRequest, JoinGuildResult, LearnSkillRequest, LearnSkillResult,
        MailActionRequest, MailClaimResult, PkBotRequest, PkSettingsRequest, PlayerActivityView, PlayerCharacterStateView,
        PlayerCharacterView, PlayerGuildApplicationList, PlayerGuildApplicationView, PlayerGuildView,
        PlayerMailList, PlayerMailView, PlayerQuestList, PlayerSkillList, PlayerVipSettingsView,
        QuestActionRequest, QuestClaimResult, RechargeCardRequest, RechargeCardResult, ShopBuyRequest,
        SkillAutoToggleRequest, SystemActionResult, SystemUpgradeRequest, UseItemResult, VipPotionSettingsRequest,
        WanxiangExtractRequest, WanxiangExtractResult,
    },
    events::RoomStateEvent,
    rest::ApiOk,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, http::bearer_token, state::AppState};

const TELEPORT_COST_GOLD: i64 = 10_000;
const SAFE_AFK_MAX_MINUTES: i64 = 10_080;
const FATIGUE_EXP_PCT: i64 = 5;
const FATIGUE_GOLD_PCT: i64 = 2;

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterQuery {
    pub character_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AfkStartRequest {
    pub character_id: Option<i64>,
    pub skill_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TeleportRequest {
    pub character_id: i64,
    pub zone: String,
    pub room: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InventoryAction {
    pub character_id: i64,
    pub item_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillQuery {
    pub character_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoveRequest {
    pub character_id: i64,
    pub direction: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttackRequest {
    pub character_id: i64,
    pub target_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CastSkillRequest {
    pub character_id: i64,
    pub skill_id: String,
    pub target_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GuildTotemRequest {
    pub character_id: i64,
    pub totem: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GuildWarTechRequest {
    pub character_id: i64,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpecialSkillUpgradeRequest {
    pub character_id: i64,
    pub skill_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EquipmentDecomposeRequest {
    pub character_id: i64,
    #[serde(default)]
    pub rarities: Vec<String>,
    #[serde(default)]
    pub item_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MiscDecomposeRequest {
    pub character_id: i64,
    #[serde(default)]
    pub kinds: Vec<String>,
    #[serde(default)]
    pub item_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterialExchangeRequest {
    pub character_id: i64,
    pub material_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BattleInstinctUpgradeRequest {
    pub character_id: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GuildShopBuyRequest {
    pub character_id: i64,
    pub item_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealtimeActionResult {
    pub room: RoomStateEvent,
    pub log: Vec<String>,
    pub character: PlayerCharacterView,
    pub state: PlayerCharacterStateView,
    pub inventory: Option<InventoryView>,
    pub adventure: Option<AdventureOfferView>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdventureResolveRequest {
    pub character_id: i64,
    pub adventure_id: i64,
    pub option_id: String,
}

pub async fn bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<GameBootstrap>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, query.character_id).await?;
    let Some(character) = character else {
        return Ok(Json(ApiOk::new(GameBootstrap {
            character: None,
            position: cq_domain::map::start_position(),
            mobs: state.mobs.as_ref().clone(),
        })));
    };

    let bundle = character_bundle(&character_repo, &inventory_repo, character).await?;
    let position = Position { zone: bundle.state.zone.clone(), room: bundle.state.room.clone() };
    Ok(Json(ApiOk::new(GameBootstrap {
        character: Some(bundle),
        position,
        mobs: state.mobs.as_ref().clone(),
    })))
}

pub async fn inventory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<InventoryView>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, query.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(inventory_repo.view(character.id, character.level).await?)))
}

pub async fn overview(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<GameOverview>>, ApiError> {
    let character_id = if query.character_id.is_some() {
        resolve_character(&state, &headers, query.character_id)
            .await?
            .2
            .map(|character| character.id)
    } else {
        None
    };
    let activity_repo = ActivityRepository::new(state.db.pool());
    let activity_rows = match character_id {
        Some(character_id) => activity_repo.enabled_for_character(character_id).await?,
        None => activity_repo.enabled().await?,
    };
    let activities = activity_rows
        .into_iter()
        .map(|item| PlayerActivityView {
            id: item.id,
            code: item.code,
            name: item.name,
            enabled: item.enabled,
            config: item.config,
            points: item.points,
        })
        .collect();
    let guilds = GuildRepository::new(state.db.pool())
        .list_for_character(character_id)
        .await?
        .into_iter()
        .map(guild_view)
        .collect();
    let character_repo = CharacterRepository::new(state.db.pool());
    let systems = match character_id {
        Some(character_id) => SystemsRepository::new(state.db.pool()).overview(character_id).await?,
        None => Default::default(),
    };
    Ok(Json(ApiOk::new(GameOverview {
        activities,
        guilds,
        power_rankings: character_repo.top_by_power(10).await?,
        level_rankings: character_repo.top_by_level(10).await?,
        systems,
    })))
}

pub async fn room(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<RoomStateEvent>>, ApiError> {
    let (character_repo, _, character) = resolve_character(&state, &headers, query.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (_, position) = active_position(&state, &character_repo, &character).await?;
    Ok(Json(ApiOk::new(room_state_for_position(&state, &position, Some(character.id)).await?)))
}

pub async fn update_pk_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PkSettingsRequest>,
) -> Result<Json<ApiOk<PlayerCharacterStateView>>, ApiError> {
    let (character_repo, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let state_row = character_repo
        .update_pk_settings(
            character.id,
            input.pk_enabled,
            input.pk_enabled && input.sweep_attack_players,
        )
        .await?;
    Ok(Json(ApiOk::new(state_view(state_row))))
}

pub async fn move_character(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<MoveRequest>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (_, position) = active_position(&state, &character_repo, &character).await?;
    let (next_position, room) = state
        .world
        .move_to(&position, input.direction.trim())
        .map_err(|_| ApiError::BadRequest("这个方向走不通".into()))?;
    character_repo.save_position(character.id, &next_position).await?;
    let state_row = character_repo.state(character.id).await?;
    let mut log = vec![format!("你来到{}。", room.name)];
    let systems_repo = SystemsRepository::new(state.db.pool());
    if let Some(message) = systems_repo
        .unlock_for_position(character.id, &next_position.zone, &next_position.room)
        .await?
    {
        log.push(message);
    }
    if next_position.zone == "ancient_secret" && next_position.room == "stargazer_observatory" {
        if let Some(message) = systems_repo.record_stargazer_entry(character.id).await? {
            log.push(message);
        }
    }
    if let Some(result) =
        settle_afk_if_training_area_left(&state, &character_repo, &inventory_repo, &character, &next_position).await?
    {
        log.push(format!("离开修炼区域，{}", result.message));
    }
    let mut response = realtime_result(
        &state,
        character.clone(),
        state_row,
        &next_position,
        None,
        log,
    )
    .await?;
    response.adventure = AdventureRepository::new(state.db.pool())
        .maybe_trigger(&character, &next_position.zone, &next_position.room, "move", 30)
        .await?;
    Ok(Json(ApiOk::new(response)))
}

pub async fn teleport(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<TeleportRequest>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (_, position) = active_position(&state, &character_repo, &character).await?;
    let current_room = state
        .world
        .current_room(&position)
        .map_err(|_| ApiError::BadRequest("房间不存在".into()))?;
    if !current_room.safe {
        return Err(ApiError::BadRequest("传送员只在安全区提供服务。".into()));
    }

    let target = Position { zone: input.zone.trim().into(), room: input.room.trim().into() };
    if target == position {
        return Err(ApiError::BadRequest("你已经在这个安全区。".into()));
    }
    let target_room = state
        .world
        .current_room(&target)
        .map_err(|_| ApiError::BadRequest("目标安全区不存在。".into()))?;
    if !target_room.safe {
        return Err(ApiError::BadRequest("传送员只能传送到安全区。".into()));
    }
    let target_room_name = target_room.name.clone();

    let updated = character_repo
        .spend_gold_and_save_position(character.id, TELEPORT_COST_GOLD, &target)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => ApiError::BadRequest(format!("金币不足，传送需要 {} 金币。", TELEPORT_COST_GOLD)),
            err => ApiError::Database(err),
        })?;
    let state_row = character_repo.state(character.id).await?;
    let mut log = vec![format!("传送员收取 {} 金币，你抵达{}。", TELEPORT_COST_GOLD, target_room_name)];
    if let Some(result) =
        settle_afk_if_training_area_left(&state, &character_repo, &inventory_repo, &updated, &target).await?
    {
        log.push(format!("离开修炼区域，{}", result.message));
    }
    Ok(Json(ApiOk::new(realtime_result(
        &state,
        updated,
        state_row,
        &target,
        None,
        log,
    )
    .await?)))
}

pub async fn attack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AttackRequest>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    realtime_attack(state, headers, input.character_id, input.target_id, None).await
}

pub async fn explore(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CharacterQuery>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let character_id = input.character_id.ok_or_else(|| ApiError::BadRequest("请选择角色".into()))?;
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (state_row, position) = active_position(&state, &character_repo, &character).await?;
    let room = state
        .world
        .current_room(&position)
        .map_err(|_| ApiError::BadRequest("房间不存在".into()))?
        .clone();
    if !room.safe && state_row.pk_enabled && state_row.sweep_attack_players {
        if BotRepository::new(state.db.pool())
            .pvp_target_at(&position.zone, &position.room, character.id, 0)
            .await?
            .is_some()
        {
            let result = fight_bot_target(
                &state,
                &character_repo,
                &inventory_repo,
                character,
                state_row,
                position,
                0,
                true,
            )
            .await?;
            return Ok(Json(ApiOk::new(result)));
        }
    }
    if room.spawns.is_empty() {
        return Err(ApiError::BadRequest("这里很安全，暂时没有可探索的敌人".into()));
    }
    let mob = roll_area_encounter(state.mobs.as_ref().as_slice(), &room.spawns)?;
    let rounds = combat_round_limit(FightMode::AreaExplore, character.level, &mob);
    let outcome = fight_virtual_mob(
        &state,
        &character_repo,
        &inventory_repo,
        character,
        state_row,
        position,
        mob,
        rounds,
        FightMode::AreaExplore,
    )
    .await?;
    Ok(Json(ApiOk::new(outcome.result)))
}

pub async fn resolve_adventure(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AdventureResolveRequest>,
) -> Result<Json<ApiOk<AdventureResolveView>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let result = AdventureRepository::new(state.db.pool())
        .resolve(character.id, input.adventure_id, input.option_id.trim())
        .await?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn pk_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PkBotRequest>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (state_row, position) = active_position(&state, &character_repo, &character).await?;
    let result = fight_bot_target(
        &state,
        &character_repo,
        &inventory_repo,
        character,
        state_row,
        position,
        input.target_index,
        false,
    )
    .await?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn secret_realm_explore(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CharacterQuery>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let character_id = input.character_id.ok_or_else(|| ApiError::BadRequest("请选择角色".into()))?;
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (state_row, position) = active_position(&state, &character_repo, &character).await?;
    let room = state
        .world
        .current_room(&position)
        .map_err(|_| ApiError::BadRequest("当前房间不存在。".into()))?;
    if !room.safe {
        return Err(ApiError::BadRequest("探索秘境需要在各大城市或安全区开启。".into()));
    }
    let seconds = secret_realm_cooldown_seconds(state.db.pool(), character.id).await?;
    if seconds > 0 {
        return Err(ApiError::BadRequest(format!("秘境灵气尚未恢复，请 {} 秒后再探索", seconds)));
    }
    let mut current_character = character;
    let mut current_state = state_row;
    let mut current_position = position;
    let mut inventory_changed = false;
    let mut cleared = 0_i32;
    let mut reward_lines = Vec::new();
    for floor in 1..=40 {
        let mob = secret_realm_mob(floor);
        let mode = FightMode::SecretRealm(floor);
        let rounds = combat_round_limit(mode, current_character.level, &mob);
        let outcome = fight_virtual_mob(
            &state,
            &character_repo,
            &inventory_repo,
            current_character,
            current_state,
            current_position,
            mob,
            rounds,
            mode,
        )
        .await?;
        inventory_changed |= outcome.inventory_changed;
        append_challenge_reward_lines(&mut reward_lines, floor, &outcome.result.log);
        current_character = outcome.character;
        current_state = outcome.state_row;
        current_position = outcome.position;
        if outcome.victory {
            cleared += 1;
        } else {
            break;
        }
    }
    set_secret_realm_cooldown(state.db.pool(), character_id, 3600).await?;
    let mut lines = vec![format!("幻境挑战结束：通关 {} / 40 层。", cleared)];
    lines.extend(reward_lines);
    lines.push("幻境挑战冷却 1 小时。".into());
    let inventory = if inventory_changed {
        Some(inventory_repo.view(current_character.id, current_character.level).await?)
    } else {
        None
    };
    Ok(Json(ApiOk::new(
        realtime_result(
            &state,
            current_character,
            current_state,
            &current_position,
            inventory,
            lines,
        )
        .await?,
    )))
}

pub async fn tower_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CharacterQuery>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let character_id = input.character_id.ok_or_else(|| ApiError::BadRequest("请选择角色".into()))?;
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (state_row, position) = active_position(&state, &character_repo, &character).await?;
    if !is_tianshui_city(&position) {
        return Err(ApiError::BadRequest("无尽塔位于天水古城，请在天水古城内挑战。".into()));
    }
    let seconds = tower_cooldown_seconds(state.db.pool(), character.id).await?;
    if seconds > 0 {
        return Err(ApiError::BadRequest(format!("无尽塔每 1 小时可重置一次，请 {} 秒后再试。", seconds)));
    }
    let start_floor = 1;
    let mut floor = start_floor;
    let mut current_character = character;
    let mut current_state = state_row;
    let mut current_position = position;
    let mut cleared = 0_i32;
    let mut reached_cap = false;
    let mut inventory_changed = false;
    let mut reward_lines = Vec::new();

    loop {
        if floor >= 100 {
            reached_cap = true;
            reward_lines.push("你已到达当前版本边界。".into());
            break;
        }
        let mob = tower_mob(floor);
        let rounds = combat_round_limit(FightMode::Tower(floor), current_character.level, &mob);
        let outcome = fight_virtual_mob(
            &state,
            &character_repo,
            &inventory_repo,
            current_character,
            current_state,
            current_position,
            mob,
            rounds,
            FightMode::Tower(floor),
        )
        .await?;
        inventory_changed |= outcome.inventory_changed;
        append_challenge_reward_lines(&mut reward_lines, floor, &outcome.result.log);
        current_character = outcome.character;
        current_state = outcome.state_row;
        current_position = outcome.position;
        if outcome.victory {
            cleared += 1;
            if floor >= 99 {
                reached_cap = true;
                reward_lines.push("你已到达当前版本边界。".into());
                break;
            }
            floor += 1;
        } else {
            break;
        }
    }

    set_tower_cooldown(state.db.pool(), character_id, 3600).await?;
    let mut lines = vec![format!(
        "无尽塔扫荡结束：从第 {} 层开始，通关 {} 层，{}。",
        start_floor,
        cleared,
        if reached_cap {
            "已到达当前版本边界".to_owned()
        } else {
            format!("停在第 {} 层", floor)
        }
    )];
    lines.extend(reward_lines);
    lines.push("无尽塔挑战冷却 1 小时。".into());
    let inventory = if inventory_changed {
        Some(inventory_repo.view(current_character.id, current_character.level).await?)
    } else {
        None
    };
    Ok(Json(ApiOk::new(
        realtime_result(
            &state,
            current_character,
            current_state,
            &current_position,
            inventory,
            lines,
        )
        .await?,
    )))
}

fn append_challenge_reward_lines(lines: &mut Vec<String>, floor: i32, floor_log: &[String]) {
    for line in floor_log.iter().filter(|line| {
        line.contains("被击败，获得")
            || line.contains("奖励：")
            || line.contains("掉落：")
            || line.contains("境界突破")
    }) {
        lines.push(format!("第 {} 层：{}", floor, line));
    }
}

fn compact_combat_log(lines: Vec<String>, max_lines: usize) -> Vec<String> {
    if lines.len() <= max_lines || max_lines < 8 {
        return lines;
    }
    let head_count = 3usize;
    let tail_count = max_lines.saturating_sub(head_count + 1);
    let skipped = lines.len().saturating_sub(head_count + tail_count);
    let mut compacted = lines.iter().take(head_count).cloned().collect::<Vec<_>>();
    compacted.push(format!("中间 {} 条战斗记录已折叠。", skipped));
    compacted.extend(lines.iter().skip(lines.len().saturating_sub(tail_count)).cloned());
    compacted
}

pub async fn world_boss_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CharacterQuery>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let character_id = input.character_id.ok_or_else(|| ApiError::BadRequest("请选择角色".into()))?;
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (state_row, position) = active_position(&state, &character_repo, &character).await?;
    if !is_world_boss_room(&position) {
        return Err(ApiError::BadRequest("世界首领位于虚空要塞，请在虚空要塞内挑战。".into()));
    }
    let seconds = world_boss_cooldown_seconds(state.db.pool(), character.id).await?;
    if seconds > 0 {
        return Err(ApiError::BadRequest(format!("世界首领尚未刷新，请 {} 秒后再挑战", seconds)));
    }
    let mob = world_boss_mob();
    let rounds = combat_round_limit(FightMode::WorldBoss, character.level, &mob);
    let mut outcome = fight_virtual_mob(
        &state,
        &character_repo,
        &inventory_repo,
        character,
        state_row,
        position,
        mob,
        rounds,
        FightMode::WorldBoss,
    )
    .await?;
    let victory = outcome.victory;
    let mut lines = vec![if victory {
        "世界首领挑战成功。".into()
    } else {
        "世界首领挑战失败。".into()
    }];
    append_final_challenge_reward_lines(&mut lines, &outcome.result.log);
    if victory {
        let template_id = world_boss_equipment();
        if let Some(item) = inventory_repo.grant_item_direct(character_id, template_id, 1).await? {
            let location = if item.location == "warehouse" { "仓库" } else { "背包" };
            lines.push(format!("奖励：{} x1 已放入{}。", item.name, location));
            outcome.result.inventory = Some(inventory_repo.view(character_id, outcome.character.level).await?);
        } else {
            lines.push("奖励发放异常：未找到主宰装备模板，请检查物品模板配置。".into());
        }
        set_world_boss_cooldown(state.db.pool(), character_id, 14400).await?;
        lines.push("世界首领将在 4 小时后重新刷新。".into());
    }
    outcome.result.log = lines;
    Ok(Json(ApiOk::new(outcome.result)))
}

fn append_final_challenge_reward_lines(lines: &mut Vec<String>, combat_log: &[String]) {
    lines.extend(combat_log.iter().filter(|line| {
        line.contains("被击败，获得")
            || line.contains("奖励：")
            || line.contains("掉落：")
            || line.contains("境界突破")
    }).cloned());
}

pub async fn wild_afk(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CharacterQuery>,
) -> Result<Json<ApiOk<AfkStatusView>>, ApiError> {
    let character_id = input.character_id.ok_or_else(|| ApiError::BadRequest("请选择角色".into()))?;
    let (character_repo, _, character) = resolve_character(&state, &headers, Some(character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (_, position) = active_position(&state, &character_repo, &character).await?;
    let room = state
        .world
        .current_room(&position)
        .map_err(|_| ApiError::BadRequest("房间不存在".into()))?;
    if room.safe || room.spawns.is_empty() {
        return Err(ApiError::BadRequest("当前区域没有可挂机的野怪。".into()));
    }
    let mobs = room
        .spawns
        .iter()
        .filter_map(|id| select_mob(state.mobs.as_ref().as_slice(), id).ok())
        .collect::<Vec<_>>();
    if mobs.is_empty() {
        return Err(ApiError::BadRequest("当前区域没有可挂机的野怪。".into()));
    }
    let count = i64::try_from(mobs.len()).unwrap_or(1).max(1);
    let exp_per_minute = mobs.iter().map(|mob| mob.exp.max(1)).sum::<i64>() / count;
    let gold_per_minute = mobs.iter().map(|mob| mob.gold.max(0)).sum::<i64>() / count;
    Ok(Json(ApiOk::new(
        AfkRepository::new(state.db.pool())
            .start_wild(
                character.id,
                character.level,
                &position.zone,
                &position.room,
                exp_per_minute.max(1),
                gold_per_minute.max(0),
            )
            .await?,
    )))
}

pub async fn cast_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CastSkillRequest>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    realtime_attack(
        state,
        headers,
        input.character_id,
        input.target_id.unwrap_or(0),
        Some(input.skill_id),
    )
    .await
}

pub async fn join_guild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<JoinGuildRequest>,
) -> Result<Json<ApiOk<JoinGuildResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let guild = GuildRepository::new(state.db.pool())
        .join(character.id, input.guild_id)
        .await
        .map_err(guild_join_error)?;
    QuestRepository::new(state.db.pool())
        .add_progress(character.id, "join_guild", 1)
        .await?;
    let message = if guild.joined {
        format!("已加入行会：{}。", guild.name)
    } else {
        format!("行会状态已更新：{}。", guild.name)
    };
    Ok(Json(ApiOk::new(JoinGuildResult { guild: guild_view(guild), message })))
}

pub async fn create_guild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreateGuildRequest>,
) -> Result<Json<ApiOk<JoinGuildResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let guild = GuildRepository::new(state.db.pool())
        .create(character.id, &input.name)
        .await
        .map_err(guild_action_error)?;
    QuestRepository::new(state.db.pool())
        .add_progress(character.id, "join_guild", 1)
        .await?;
    Ok(Json(ApiOk::new(JoinGuildResult {
        message: format!("行会 {} 创建成功，你已成为会长。", guild.name),
        guild: guild_view(guild),
    })))
}

pub async fn apply_guild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildApplyRequest>,
) -> Result<Json<ApiOk<JoinGuildResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let guild = GuildRepository::new(state.db.pool())
        .apply(
            character.id,
            input.guild_id,
            input.message.as_deref().unwrap_or_default(),
        )
        .await
        .map_err(guild_action_error)?;
    Ok(Json(ApiOk::new(JoinGuildResult {
        message: format!("已向 {} 提交入会申请。", guild.name),
        guild: guild_view(guild),
    })))
}

pub async fn guild_applications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<PlayerGuildApplicationList>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, query.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let applications = GuildRepository::new(state.db.pool())
        .applications_for_reviewer(character.id)
        .await
        .map_err(guild_action_error)?
        .into_iter()
        .map(guild_application_view)
        .collect();
    Ok(Json(ApiOk::new(PlayerGuildApplicationList { applications })))
}

pub async fn review_guild_application(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildReviewApplicationRequest>,
) -> Result<Json<ApiOk<PlayerGuildApplicationList>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let applications = GuildRepository::new(state.db.pool())
        .review_application(character.id, input.application_id, input.accept)
        .await
        .map_err(guild_action_error)?
        .into_iter()
        .map(guild_application_view)
        .collect();
    Ok(Json(ApiOk::new(PlayerGuildApplicationList { applications })))
}

pub async fn donate_guild(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildDonateRequest>,
) -> Result<Json<ApiOk<JoinGuildResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let guild = GuildRepository::new(state.db.pool())
        .donate(character.id, input.gold)
        .await
        .map_err(guild_action_error)?;
    let points = input.gold / 10_000;
    QuestRepository::new(state.db.pool())
        .add_progress(character.id, "guild_donate", 1)
        .await?;
    Ok(Json(ApiOk::new(JoinGuildResult {
        message: format!("已向行会捐献 {} 金币，行会建设 +{}，个人贡献 +{}。", input.gold, points, points),
        guild: guild_view(guild),
    })))
}

pub async fn complete_guild_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildTaskRequest>,
) -> Result<Json<ApiOk<JoinGuildResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = GuildRepository::new(state.db.pool())
        .complete_task(character.id, &input.task_kind)
        .await
        .map_err(guild_action_error)?;
    QuestRepository::new(state.db.pool())
        .add_progress(character.id, "guild_task", 1)
        .await?;
    if input.task_kind.trim() == "supply" {
        QuestRepository::new(state.db.pool())
            .add_progress(character.id, "guild_donate", 1)
            .await?;
    }
    Ok(Json(ApiOk::new(JoinGuildResult {
        message: outcome.message,
        guild: guild_view(outcome.guild),
    })))
}

pub async fn claim_guild_benefit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildBenefitRequest>,
) -> Result<Json<ApiOk<GuildBenefitResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = GuildRepository::new(state.db.pool())
        .claim_benefit(character.id)
        .await
        .map_err(guild_action_error)?;
    let inventory = inventory_repo.view(character.id, character.level).await?;
    let character = character_repo.find(character.id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(GuildBenefitResult {
        guild: guild_view(outcome.guild),
        inventory,
        character: character_view(character),
        message: outcome.message,
    })))
}

pub async fn use_guild_merit_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildBenefitRequest>,
) -> Result<Json<ApiOk<GuildBenefitResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = GuildRepository::new(state.db.pool())
        .use_merit_token(character.id)
        .await
        .map_err(guild_action_error)?;
    let inventory = inventory_repo.view(character.id, character.level).await?;
    let character = character_repo.find(character.id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(GuildBenefitResult {
        guild: guild_view(outcome.guild),
        inventory,
        character: character_view(character),
        message: outcome.message,
    })))
}

pub async fn buy_guild_shop_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildShopBuyRequest>,
) -> Result<Json<ApiOk<GuildBenefitResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = GuildRepository::new(state.db.pool())
        .buy_shop_item(character.id, &input.item_id)
        .await
        .map_err(guild_action_error)?;
    let inventory = inventory_repo.view(character.id, character.level).await?;
    let character = character_repo.find(character.id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(GuildBenefitResult {
        guild: guild_view(outcome.guild),
        inventory,
        character: character_view(character),
        message: outcome.message,
    })))
}

pub async fn upgrade_guild_totem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildTotemRequest>,
) -> Result<Json<ApiOk<GuildBenefitResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = GuildRepository::new(state.db.pool())
        .upgrade_totem(character.id, &input.totem)
        .await
        .map_err(guild_action_error)?;
    let character = inventory_repo.refresh_character_power(character.id).await?;
    inventory_repo.clamp_character_resources(character.id).await?;
    let inventory = inventory_repo.view(character.id, character.level).await?;
    let character = character_repo.find(character.id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(GuildBenefitResult {
        guild: guild_view(outcome.guild),
        inventory,
        character: character_view(character),
        message: outcome.message,
    })))
}

pub async fn charge_guild_war_tech(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildWarTechRequest>,
) -> Result<Json<ApiOk<GuildBenefitResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = GuildRepository::new(state.db.pool())
        .charge_war_tech(character.id, &input.kind)
        .await
        .map_err(guild_action_error)?;
    let inventory = inventory_repo.view(character.id, character.level).await?;
    let character = character_repo.find(character.id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(GuildBenefitResult {
        guild: guild_view(outcome.guild),
        inventory,
        character: character_view(character),
        message: outcome.message,
    })))
}

pub async fn claim_sabak_tax(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<GuildBenefitRequest>,
) -> Result<Json<ApiOk<GuildBenefitResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = GuildRepository::new(state.db.pool())
        .claim_sabak_tax(character.id)
        .await
        .map_err(guild_action_error)?;
    let inventory = inventory_repo.view(character.id, character.level).await?;
    let character = character_repo.find(character.id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(GuildBenefitResult {
        guild: guild_view(outcome.guild),
        inventory,
        character: character_view(character),
        message: outcome.message,
    })))
}

pub async fn quest_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<PlayerQuestList>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, query.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let quests = QuestRepository::new(state.db.pool())
        .list_for_character(character.id)
        .await?;
    Ok(Json(ApiOk::new(PlayerQuestList { quests })))
}

pub async fn quest_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<QuestActionRequest>,
) -> Result<Json<ApiOk<QuestClaimResult>>, ApiError> {
    let (character_repo, inventory_repo, character) =
        resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let quest_repo = QuestRepository::new(state.db.pool());
    let reward = quest_repo
        .claim(character.id, input.quest_id.trim())
        .await
        .map_err(quest_error)?;
    let character = character_repo.find(character.id).await?.ok_or(ApiError::NotFound)?;
    let quests = PlayerQuestList {
        quests: quest_repo.list_for_character(character.id).await?,
    };
    let inventory = inventory_repo.view(character.id, character.level).await?;
    let mut parts = Vec::new();
    if reward.gold > 0 {
        parts.push(format!("金币 +{}", reward.gold));
    }
    if !reward.items.is_empty() {
        parts.push(reward.items.join("、"));
    }
    let message = if parts.is_empty() {
        format!("已领取任务奖励：{}。", reward.quest_name)
    } else {
        format!("已领取任务奖励：{}，{}。", reward.quest_name, parts.join("，"))
    };
    Ok(Json(ApiOk::new(QuestClaimResult {
        quests,
        inventory,
        character: character_view(character),
        message,
    })))
}

pub async fn equip(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryAction>,
) -> Result<Json<ApiOk<InventoryView>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .equip(character.id, input.item_id)
            .await
            .map_err(inventory_action_error)?,
    )))
}

pub async fn unequip(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryAction>,
) -> Result<Json<ApiOk<InventoryView>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .unequip(character.id, input.item_id)
            .await
            .map_err(inventory_action_error)?,
    )))
}

pub async fn use_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryAction>,
) -> Result<Json<ApiOk<UseItemResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (max_hp, max_mp) = character_max_resources(&state, &inventory_repo, &character).await;
    Ok(Json(ApiOk::new(
        inventory_repo
            .use_item(character.id, input.item_id, Some(max_hp), Some(max_mp))
            .await
            .map_err(inventory_action_error)?,
    )))
}

pub async fn enhance_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryAction>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let result = inventory_repo
        .enhance(character.id, input.item_id)
        .await
        .map_err(inventory_game_action_error)?;
    QuestRepository::new(state.db.pool())
        .add_progress(character.id, "enhance", 1)
        .await?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn recycle_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryAction>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .recycle(character.id, input.item_id)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn decompose_equipment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<EquipmentDecomposeRequest>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let rarities = input
        .rarities
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let result = inventory_repo
        .decompose(character.id, &rarities, &input.item_ids)
        .await
        .map_err(|err| {
            tracing::warn!(
                error = ?err,
                character_id = character.id,
                item_count = input.item_ids.len(),
                "equipment decompose failed"
            );
            match err {
                InventoryActionError::Database(db_err) => {
                    ApiError::BadRequest(format!("装备拆解数据库错误：{db_err}"))
                }
                err => inventory_game_action_error(err),
            }
        })?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn decompose_misc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<MiscDecomposeRequest>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let kinds = input
        .kinds
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| matches!(value.as_str(), "book" | "material" | "consumable"))
        .collect::<Vec<_>>();
    Ok(Json(ApiOk::new(
        inventory_repo
            .decompose_misc(character.id, &kinds, &input.item_ids)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn npc_material_exchange(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<MaterialExchangeRequest>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .exchange_insight_material(character.id, &input.material_id)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn npc_battle_instinct_upgrade(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<BattleInstinctUpgradeRequest>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (_, position) = active_position(&state, &character_repo, &character).await?;
    if !is_chaos_shelter(&position) {
        return Err(ApiError::BadRequest("战斗本能需要前往混沌庇护所寻找不动冥王提升。".into()));
    }
    Ok(Json(ApiOk::new(
        inventory_repo
            .upgrade_battle_instinct(character.id)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn npc_special_skill_upgrade(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SpecialSkillUpgradeRequest>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (_, position) = active_position(&state, &character_repo, &character).await?;
    if !is_chaos_shelter(&position) {
        return Err(ApiError::BadRequest("通用特殊技能需要前往混沌庇护所寻找不动冥王提升。".into()));
    }
    Ok(Json(ApiOk::new(
        inventory_repo
            .upgrade_special_passive(character.id, input.skill_id.trim())
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn store_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryAction>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .store(character.id, input.item_id)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn retrieve_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryAction>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .retrieve(character.id, input.item_id)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn shop_buy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ShopBuyRequest>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .buy_template(character.id, input.template_id.trim(), input.quantity)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn yuanbao_shop_buy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ShopBuyRequest>,
) -> Result<Json<ApiOk<InventoryActionResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        inventory_repo
            .buy_yuanbao_template(character.id, input.template_id.trim(), input.quantity)
            .await
            .map_err(inventory_game_action_error)?,
    )))
}

pub async fn update_vip_potion_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<VipPotionSettingsRequest>,
) -> Result<Json<ApiOk<PlayerVipSettingsView>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        SystemsRepository::new(state.db.pool())
            .update_vip_potion_settings(character.id, input)
            .await
            .map_err(systems_action_error)?,
    )))
}

pub async fn recharge_redeem(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RechargeCardRequest>,
) -> Result<Json<ApiOk<RechargeCardResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        SystemsRepository::new(state.db.pool())
            .redeem_card(character.id, &input.code)
            .await
            .map_err(systems_action_error)?,
    )))
}

pub async fn upgrade_pet(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SystemUpgradeRequest>,
) -> Result<Json<ApiOk<SystemActionResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let result = SystemsRepository::new(state.db.pool())
        .upgrade_pet(character.id, input.target_id)
        .await
        .map_err(systems_action_error)?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn upgrade_treasure(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SystemUpgradeRequest>,
) -> Result<Json<ApiOk<SystemActionResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let result = SystemsRepository::new(state.db.pool())
        .upgrade_treasure(character.id, input.target_id)
        .await
        .map_err(systems_action_error)?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn cultivation_breakthrough(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SystemUpgradeRequest>,
) -> Result<Json<ApiOk<SystemActionResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let result = SystemsRepository::new(state.db.pool())
        .cultivate(character.id)
        .await
        .map_err(systems_action_error)?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn wanxiang_upgrade(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SystemUpgradeRequest>,
) -> Result<Json<ApiOk<SystemActionResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let result = SystemsRepository::new(state.db.pool())
        .upgrade_wanxiang(character.id)
        .await
        .map_err(systems_action_error)?;
    Ok(Json(ApiOk::new(result)))
}

pub async fn wanxiang_extract(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<WanxiangExtractRequest>,
) -> Result<Json<ApiOk<WanxiangExtractResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let result = SystemsRepository::new(state.db.pool())
        .extract_wanxiang_essence(character.id, input.max_tier)
        .await
        .map_err(systems_action_error)?;
    let inventory = inventory_repo.view(character.id, result.character.level).await?;
    Ok(Json(ApiOk::new(WanxiangExtractResult {
        systems: result.systems,
        inventory,
        character: result.character,
        message: result.message,
    })))
}

pub async fn skill_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SkillQuery>,
) -> Result<Json<ApiOk<PlayerSkillList>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, query.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let skills = SkillRepository::new(state.db.pool()).list_for_character(character.id).await?;
    Ok(Json(ApiOk::new(PlayerSkillList { skills })))
}

async fn validate_skill_acquisition_location(
    state: &AppState,
    character_repo: &CharacterRepository,
    character: &CharacterRecord,
    config: &serde_json::Value,
) -> Result<(), ApiError> {
    let acquire = config.get("acquire").and_then(serde_json::Value::as_str).unwrap_or_default();
    if acquire.is_empty() || config.get("requires_book").and_then(serde_json::Value::as_bool).unwrap_or(true) {
        return Ok(());
    }
    let (_, position) = active_position(state, character_repo, character).await?;
    let valid = match acquire {
        "academy" => position.zone == "xiuzhen" && position.room == "tianshui_academy",
        "void_market" => position.zone == "feisheng" && position.room == "void_market",
        "chaos_master" => position.zone == "feisheng" && position.room == "chaos_shelter",
        _ => true,
    };
    if valid {
        return Ok(());
    }
    let message = match acquire {
        "academy" => "初期职业技能需要前往天水古城的天水书院购买。",
        "void_market" => "中期职业技能需要前往虚空要塞的虚空市集购买。",
        "chaos_master" => "通用技能需要前往混沌庇护所寻找不动冥王学习。",
        _ => "当前位置无法学习该技能。",
    };
    Err(ApiError::BadRequest(message.into()))
}

pub async fn learn_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<LearnSkillRequest>,
) -> Result<Json<ApiOk<LearnSkillResult>>, ApiError> {
    let (character_repo, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let skill_repo = SkillRepository::new(state.db.pool());
    let skill_id = input.skill_id.trim();
    let catalog_skill = skill_repo
        .list_for_character(character.id)
        .await?
        .into_iter()
        .find(|skill| skill.id == skill_id)
        .ok_or_else(|| ApiError::BadRequest("技能不存在或职业不符。".into()))?;
    validate_skill_acquisition_location(&state, &character_repo, &character, &catalog_skill.config).await?;
    let skill = skill_repo
        .learn(character.id, input.skill_id.trim())
        .await
        .map_err(skill_action_error)?;
    Ok(Json(ApiOk::new(LearnSkillResult {
        message: format!("已学会技能：{}。", skill.name),
        skill,
    })))
}

pub async fn upgrade_skill(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<LearnSkillRequest>,
) -> Result<Json<ApiOk<LearnSkillResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let outcome = SkillRepository::new(state.db.pool())
        .upgrade(character.id, input.skill_id.trim())
        .await
        .map_err(skill_upgrade_error)?;
    let message = format!("{} 已按累计技能经验校准到 {} 级。", outcome.skill.name, outcome.skill.level.unwrap_or(1));
    Ok(Json(ApiOk::new(LearnSkillResult { message, skill: outcome.skill })))
}

pub async fn toggle_skill_auto(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SkillAutoToggleRequest>,
) -> Result<Json<ApiOk<LearnSkillResult>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let skill = SkillRepository::new(state.db.pool())
        .set_auto_enabled(character.id, input.skill_id.trim(), input.auto_enabled)
        .await
        .map_err(skill_action_error)?;
    let message = if skill.auto_enabled {
        format!("{} 已开启自动释放。", skill.name)
    } else {
        format!("{} 已关闭自动释放。", skill.name)
    };
    Ok(Json(ApiOk::new(LearnSkillResult { message, skill })))
}

pub async fn mail_list(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<PlayerMailList>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, query.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let mails = MailRepository::new(state.db.pool()).list_for_character(character.id).await?;
    Ok(Json(ApiOk::new(PlayerMailList { mails })))
}

pub async fn mail_read(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<MailActionRequest>,
) -> Result<Json<ApiOk<PlayerMailView>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        MailRepository::new(state.db.pool())
            .mark_read(character.id, input.mail_id)
            .await
            .map_err(mail_action_error)?,
    )))
}

pub async fn mail_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<MailActionRequest>,
) -> Result<Json<ApiOk<MailClaimResult>>, ApiError> {
    let (_, inventory_repo, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let claimed = MailRepository::new(state.db.pool())
        .claim(character.id, input.mail_id)
        .await
        .map_err(mail_action_error)?;
    let inventory = inventory_repo.view(character.id, claimed.character.level).await?;
    let message = claim_message(claimed.gold, claimed.yuanbao, claimed.item_quantity);
    Ok(Json(ApiOk::new(MailClaimResult {
        mail: claimed.mail,
        inventory,
        character: character_view(claimed.character),
        message,
    })))
}

pub async fn mail_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<MailActionRequest>,
) -> Result<Json<ApiOk<PlayerMailList>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, Some(input.character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let mails = MailRepository::new(state.db.pool())
        .delete_for_character(character.id, input.mail_id)
        .await
        .map_err(mail_action_error)?;
    Ok(Json(ApiOk::new(PlayerMailList { mails })))
}

pub async fn afk_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CharacterQuery>,
) -> Result<Json<ApiOk<AfkStatusView>>, ApiError> {
    let (_, _, character) = resolve_character(&state, &headers, query.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(AfkRepository::new(state.db.pool()).status(character.id).await?)))
}

pub async fn afk_start(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<AfkStartRequest>,
) -> Result<Json<ApiOk<AfkStatusView>>, ApiError> {
    let character_id = input.character_id.ok_or_else(|| ApiError::BadRequest("请选择角色".into()))?;
    let (character_repo, _, character) = resolve_character(&state, &headers, Some(character_id)).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    let (_, position) = active_position(&state, &character_repo, &character).await?;
    let _room = state
        .world
        .current_room(&position)
        .map_err(|_| ApiError::BadRequest("房间不存在".into()))?;
    if position.zone == "xiuzhen" && position.room == "purgatory" {
        return Ok(Json(ApiOk::new(
            AfkRepository::new(state.db.pool())
                .start_training(character.id, character.level)
                .await?,
        )));
    }
    if position.zone != "feisheng" || position.room != "void_realm" {
        return Err(ApiError::BadRequest("请前往破冰前哨站的炼狱，或混沌庇护所的虚境后再开始打坐。".into()));
    }
    let skill = SkillRepository::new(state.db.pool())
        .learned_for_character(character.id, input.skill_id.trim())
        .await
        .map_err(|_| ApiError::BadRequest("请选择一个已学习的技能进行虚境研修。".into()))?;
    if skill.config.get("special_upgrade_only").and_then(serde_json::Value::as_bool).unwrap_or(false) {
        return Err(ApiError::BadRequest("特殊被动只能在混沌庇护所的不动冥王处使用技能书残页提升。".into()));
    }
    Ok(Json(ApiOk::new(
        AfkRepository::new(state.db.pool())
            .start(character.id, character.level, &skill.id, &skill.name)
            .await?,
    )))
}

pub async fn afk_settle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CharacterQuery>,
) -> Result<Json<ApiOk<AfkSettleResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, input.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        settle_afk_for_character(&state, &character_repo, &inventory_repo, character, false).await?,
    )))
}

pub async fn afk_stop(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CharacterQuery>,
) -> Result<Json<ApiOk<AfkSettleResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, input.character_id).await?;
    let character = character.ok_or(ApiError::NotFound)?;
    Ok(Json(ApiOk::new(
        settle_afk_for_character(&state, &character_repo, &inventory_repo, character, true).await?,
    )))
}

async fn settle_afk_if_training_area_left(
    state: &AppState,
    character_repo: &CharacterRepository,
    inventory_repo: &InventoryRepository,
    character: &CharacterRecord,
    position: &Position,
) -> Result<Option<AfkSettleResult>, ApiError> {
    let afk_repo = AfkRepository::new(state.db.pool());
    let status = afk_repo.status(character.id).await?;
    if !status.active || afk_status_allowed_at(&status, position) {
        return Ok(None);
    }
    Ok(Some(
        settle_afk_for_character(state, character_repo, inventory_repo, character.clone(), true).await?,
    ))
}

async fn settle_afk_for_character(
    state: &AppState,
    character_repo: &CharacterRepository,
    inventory_repo: &InventoryRepository,
    character: CharacterRecord,
    force_stop: bool,
) -> Result<AfkSettleResult, ApiError> {
    let character_id = character.id;
    let afk_repo = AfkRepository::new(state.db.pool());
    let current_status = afk_repo.status(character_id).await?;
    if !current_status.active {
        return Ok(AfkSettleResult {
            status: current_status,
            exp: 0,
            gold: 0,
            minutes: 0,
            message: "当前没有进行中的打坐修炼。".into(),
            adventure: None,
        });
    }

    let equipment_bonus = inventory_repo.equipment_bonus(character_id).await.unwrap_or_default();
    let mut result = afk_repo
        .settle_with_cap(character_id, SAFE_AFK_MAX_MINUTES)
        .await
        .map_err(afk_action_error)?;
    if result.status.mode == "wild" {
        settle_wild_afk(
            state,
            character_repo,
            inventory_repo,
            character.clone(),
            &equipment_bonus,
            &mut result,
        )
        .await?;
    } else {
        if result.status.mode == "skill_study" && result.minutes > 0 {
            if let Some(skill_id) = result.status.training_skill_id.clone() {
                SkillRepository::new(state.db.pool())
                    .add_study_proficiency(character_id, &skill_id, result.minutes * 5)
                    .await
                    .map_err(|_| ApiError::BadRequest("技能研修失败，请重新选择已学习的非特殊技能。".into()))?;
            }
        }
        if result.exp > 0 || result.gold > 0 {
            character_repo.grant_reward(character_id, result.exp, result.gold).await?;
            inventory_repo.refresh_character_power(character_id).await?;
        }
        if result.exp == 0 && result.gold == 0 {
            result.message = if result.minutes > 0 {
                result.message
            } else {
                "打坐时间不足 5 秒，暂未产生收益。".into()
            };
        }
    }
    if result.minutes > 0 {
        QuestRepository::new(state.db.pool())
            .add_progress(character_id, "afk_settle", 1)
            .await?;
        let trigger_zone = result
            .status
            .zone
            .clone()
            .unwrap_or_else(|| "afk".into());
        let trigger_room = result
            .status
            .room
            .clone()
            .unwrap_or_else(|| result.status.mode.clone());
        result.adventure = AdventureRepository::new(state.db.pool())
            .maybe_trigger(&character, &trigger_zone, &trigger_room, "afk", 25)
            .await?;
    }
    if force_stop && result.status.active {
        result.status = afk_repo.stop(character_id).await.map_err(afk_action_error)?;
        if result.minutes == 0 && result.exp == 0 && result.gold == 0 {
            result.message = "打坐修炼已停止，时间不足 5 秒，暂未产生收益。".into();
        }
    }
    Ok(result)
}

fn afk_status_allowed_at(status: &AfkStatusView, position: &Position) -> bool {
    match status.mode.as_str() {
        "skill_study" => position.zone == "feisheng" && position.room == "void_realm",
        "practice" => position.zone == "xiuzhen" && position.room == "purgatory",
        "wild" => {
            status.zone.as_deref() == Some(position.zone.as_str())
                && status.room.as_deref() == Some(position.room.as_str())
        }
        _ => true,
    }
}

async fn settle_wild_afk(
    state: &AppState,
    character_repo: &CharacterRepository,
    inventory_repo: &InventoryRepository,
    mut character: CharacterRecord,
    equipment_bonus: &EquipmentBonus,
    result: &mut AfkSettleResult,
) -> Result<(), ApiError> {
    let potential_minutes = result.minutes.max(0);
    if potential_minutes <= 0 {
        result.exp = 0;
        result.gold = 0;
        result.message = "野怪挂机时间不足 1 分钟，暂未产生收益。".into();
        return Ok(());
    }

    let Some(zone) = result.status.zone.clone() else {
        result.exp = 0;
        result.gold = 0;
        result.minutes = 0;
        result.message = "野怪挂机位置丢失，无法结算收益，请重新开启。".into();
        return Ok(());
    };
    let Some(room_id) = result.status.room.clone() else {
        result.exp = 0;
        result.gold = 0;
        result.minutes = 0;
        result.message = "野怪挂机位置丢失，无法结算收益，请重新开启。".into();
        return Ok(());
    };

    let position = Position { zone, room: room_id };
    let (room_name, spawns) = {
        let room = state
            .world
            .current_room(&position)
            .map_err(|_| ApiError::BadRequest("野怪挂机区域不存在，请重新选择挂机点。".into()))?;
        if room.safe || room.spawns.is_empty() {
            result.exp = 0;
            result.gold = 0;
            result.minutes = 0;
            result.message = "当前挂机点已经不是野外区域，无法结算野怪收益。".into();
            return Ok(());
        }
        (room.name.clone(), room.spawns.clone())
    };

    let mut state_row = character_repo.state(character.id).await?;
    let (max_hp, max_mp) = character_max_resources(state, inventory_repo, &character).await;
    clamp_resource_values(&mut state_row, max_hp, max_mp);
    let capped_minutes = potential_minutes.min(120 + equipment_bonus.afk_extra_minutes.max(0));
    let effective_minutes = capped_minutes.min(state_row.mp.max(0));
    let mp_cost = effective_minutes;

    if effective_minutes <= 0 {
        result.exp = 0;
        result.gold = 0;
        result.minutes = 0;
        result.message = format!(
            "野怪挂机结算：魔法不足，每分钟固定消耗 1 点魔法，{} 分钟挂机未能完成探索判定，请补充魔法后继续。",
            potential_minutes
        );
        return Ok(());
    }

    let stamina_kills = apply_pct(effective_minutes, equipment_bonus.afk_kill_pct)
        .max(effective_minutes)
        .clamp(0, 10_000);
    let stamina = character_repo
        .consume_stamina_for_kills(character.id, stamina_kills)
        .await?;
    let stamina = apply_dominator_fatigue_privilege(stamina, equipment_bonus);

    result.minutes = effective_minutes;
    let base_exp = apply_pct(
        result.status.exp_per_minute.saturating_mul(effective_minutes),
        equipment_bonus.afk_base_reward_pct + equipment_bonus.afk_exp_pct + equipment_bonus.afk_kill_pct + equipment_bonus.afk_offline_reward_pct,
    );
    let base_gold = apply_pct(
        result.status.gold_per_minute.saturating_mul(effective_minutes),
        equipment_bonus.afk_base_reward_pct + equipment_bonus.afk_gold_pct + equipment_bonus.afk_kill_pct + equipment_bonus.afk_offline_reward_pct,
    );
    result.exp = fatigue_split_reward(base_exp, &stamina, FATIGUE_EXP_PCT);
    result.gold = fatigue_split_reward(base_gold, &stamina, FATIGUE_GOLD_PCT);
    state_row.mp = (state_row.mp - mp_cost).max(0);
    apply_stamina_to_state(&mut state_row, &stamina);

    let mut lines = vec![format!(
        "野怪挂机结算：{} 挂机 {} 分钟，每分钟固定消耗 1 点魔法，共消耗 {} 魔法，获得 {} 经验和 {} 金币。",
        room_name, effective_minutes, mp_cost, result.exp, result.gold
    )];
    append_stamina_batch_log(&mut lines, &stamina);
    if effective_minutes < potential_minutes {
        lines.push(format!("魔法不足，只结算了 {} / {} 分钟。", effective_minutes, potential_minutes));
    }

    if result.exp > 0 || result.gold > 0 {
        let before_level = character.level;
        character = character_repo.grant_reward(character.id, result.exp, result.gold).await?;
        character = inventory_repo.refresh_character_power(character.id).await?;
        if character.level > before_level {
            lines.push(format!("境界突破：{} 升到 {} 级。", character.name, character.level));
        }
    }

    let skill_bonus = SkillRepository::new(state.db.pool())
        .bonus(character.id)
        .await
        .unwrap_or_default();
    let system_bonus = SystemsRepository::new(state.db.pool())
        .combat_bonus(character.id)
        .await
        .unwrap_or_default();
    let (max_hp, max_mp) = character_resource_caps(&character, equipment_bonus, &skill_bonus, &system_bonus);
    clamp_resource_values(&mut state_row, max_hp, max_mp);
    let combatant = character_combatant(&character, &state_row, equipment_bonus, &skill_bonus, &system_bonus);
    let simulated_kills = if stamina.full_kills > 0 {
        apply_pct(effective_minutes.min(60), equipment_bonus.afk_kill_pct)
            .min(stamina.full_kills)
            .clamp(0, 120)
    } else {
        0
    };
    let simulated_mobs = (0..simulated_kills)
        .map(|_| roll_regular_area_encounter(state.mobs.as_ref().as_slice(), &spawns))
        .collect::<Result<Vec<_>, ApiError>>()?;
    let mut drop_count = 0_i64;
    let mut drop_names = Vec::new();
    for mob in simulated_mobs {
        for drop_roll in roll_level_drop_for_source(
            mob.level,
            MobDropKind::Normal,
            combatant.luck,
            Some(&mob.id),
            equipment_bonus.afk_drop_quality_pct,
        ) {
            if let Some(item) = inventory_repo
                .grant_item(character.id, &drop_roll.template_id, drop_roll.quantity)
                .await?
            {
                drop_count += drop_roll.quantity;
                if drop_names.len() < 5 {
                    drop_names.push(format!("{} x{}", item.name, drop_roll.quantity));
                }
            }
        }
    }
    if drop_count > 0 {
        lines.push(format!("挂机掉落：{}。", drop_names.join("、")));
    } else if stamina.fatigue_kills > 0 && stamina.full_kills == 0 {
        lines.push("疲劳状态：装备与材料掉率为 0%，本次野怪挂机没有掉落。".into());
    } else {
        lines.push("本次野怪挂机没有获得额外掉落。".into());
    }

    let average_level = average_spawn_level(state.mobs.as_ref().as_slice(), &spawns);
    if roll_wild_afk_death(effective_minutes, character.level, average_level) {
        if try_origin_revive(
            inventory_repo,
            character_repo,
            character.id,
            &mut state_row,
            max_hp,
            max_mp,
            &mut lines,
        )
        .await?
        {
            lines.push("野外挂机遭遇致命危险，但原地复活生效，挂机未中断。".into());
        } else {
            let return_name = death_return_name(&position);
            let return_position = death_return_position(&position);
            let (hp, mp) = default_resources(&character);
            character_repo
                .save_state_snapshot(character.id, &return_position, (hp / 2).max(1), (mp / 2).max(0))
                .await?;
            lines.push(format!("野外挂机途中被击倒，回到{}休整，挂机已中断。", return_name));
            apply_death_penalty(inventory_repo, character.id, &mut lines).await?;
            result.status = AfkRepository::new(state.db.pool())
                .stop(character.id)
                .await
                .map_err(afk_action_error)?;
        }
    } else {
        character_repo
            .save_resources(character.id, state_row.hp, state_row.mp)
            .await?;
    }

    result.message = lines.join(" ");
    Ok(())
}

async fn resolve_character(
    state: &AppState,
    headers: &HeaderMap,
    character_id: Option<i64>,
) -> Result<(CharacterRepository, InventoryRepository, Option<CharacterRecord>), ApiError> {
    let token = bearer_token(headers)?;
    let session = AccountRepository::new(state.db.pool())
        .find_session(token)
        .await?
        .ok_or(ApiError::Unauthorized)?;
    let character_repo = CharacterRepository::new(state.db.pool());
    let inventory_repo = InventoryRepository::new(state.db.pool());
    let character = match character_id {
        Some(id) => character_repo.find_for_account(session.account_id, id).await?,
        None => character_repo.first_for_account(session.account_id).await?,
    };
    if let Some(character) = character.as_ref() {
        let _ = character_repo.set_online(character.id, true).await;
    }
    Ok((character_repo, inventory_repo, character))
}

async fn character_bundle(
    character_repo: &CharacterRepository,
    inventory_repo: &InventoryRepository,
    character: CharacterRecord,
) -> Result<CharacterBundle, ApiError> {
    let stats = character_repo.stats(character.id).await?;
    inventory_repo.clamp_character_resources(character.id).await?;
    let state = character_repo.state(character.id).await?;
    let inventory = inventory_repo.summary(character.id, character.level).await?;
    Ok(CharacterBundle {
        character: character_view(character),
        stats: stats_view(stats),
        state: state_view(state),
        inventory,
    })
}

#[derive(Debug, Clone, Copy)]
enum FightMode {
    AreaExplore,
    WildAfk,
    SecretRealm(i32),
    Tower(i32),
    WorldBoss,
}

#[derive(Debug, Clone)]
struct FightOutcome {
    result: RealtimeActionResult,
    victory: bool,
    character: CharacterRecord,
    state_row: CharacterStateRecord,
    position: Position,
    inventory_changed: bool,
}

#[derive(Debug, Clone, Copy)]
enum AutoSkillOutcome {
    Attack {
        power: f64,
        magical: bool,
        target_current_hp_pct: i64,
        boss_cap_atk_multiplier: i64,
        execute_threshold_pct: i64,
        execute_bonus_pct: i64,
        flat_bonus_damage: i64,
    },
    Support,
}

#[derive(Debug, Clone, Copy, Default)]
struct DamageBonusOutcome {
    normal_mob_execute: bool,
    true_damage: i64,
    creation_strike: bool,
    full_restore: bool,
}

fn apply_auto_skill_effects(
    report: &mut DamageReport,
    attacker: &Combatant,
    defender: &Combatant,
    target_is_boss: bool,
    effect: AutoSkillOutcome,
    lines: &mut Vec<String>,
) {
    if !report.hit || report.damage <= 0 {
        return;
    }
    let AutoSkillOutcome::Attack {
        target_current_hp_pct,
        boss_cap_atk_multiplier,
        execute_threshold_pct,
        execute_bonus_pct,
        flat_bonus_damage,
        ..
    } = effect else {
        return;
    };
    let mut damage = report.damage;
    if target_current_hp_pct > 0 {
        let mut extra = defender.hp.saturating_mul(target_current_hp_pct.clamp(0, 20)) / 100;
        if target_is_boss {
            extra = extra.min(attacker.atk.max(1).saturating_mul(boss_cap_atk_multiplier.max(1)));
        }
        if extra > 0 {
            damage = damage.saturating_add(extra);
            lines.push(format!("技能追加当前生命折算伤害 {} 点。", extra));
        }
    }
    if flat_bonus_damage > 0 {
        damage = damage.saturating_add(flat_bonus_damage);
        lines.push(format!("技能额外真元伤害 {} 点。", flat_bonus_damage));
    }
    if execute_threshold_pct > 0
        && execute_bonus_pct > 0
        && defender.hp.saturating_mul(100) <= defender.max_hp.saturating_mul(execute_threshold_pct.clamp(1, 99))
    {
        damage = damage.saturating_mul(100 + execute_bonus_pct.clamp(0, 200)) / 100;
        lines.push(format!("斩杀机制触发，技能伤害提升 {}%。", execute_bonus_pct.clamp(0, 200)));
    }
    report.damage = damage.max(1);
    report.remaining_hp = defender.hp.saturating_sub(report.damage).max(0);
}

async fn fight_virtual_mob(
    state: &AppState,
    character_repo: &CharacterRepository,
    inventory_repo: &InventoryRepository,
    mut character: CharacterRecord,
    mut state_row: CharacterStateRecord,
    mut position: Position,
    mob: MobTemplate,
    rounds: usize,
    mode: FightMode,
) -> Result<FightOutcome, ApiError> {
    let mut lines = vec![format!("{}：你遭遇了 {}。", fight_mode_title(mode), mob.name)];
    let mut mob_hp = mob.max_hp.max(1);
    let mut inventory_changed = false;
    let mut leveled = false;
    let mut victory = false;
    let mut defeated = false;
    let auto_skills = if matches!(
        mode,
        FightMode::AreaExplore | FightMode::SecretRealm(_) | FightMode::Tower(_) | FightMode::WorldBoss
    ) {
        SkillRepository::new(state.db.pool())
            .active_skills_for_character(character.id)
            .await?
    } else {
        Vec::new()
    };

    if matches!(mode, FightMode::AreaExplore) {
        let mp_cost = 1_i64;
        if state_row.mp < mp_cost {
            return Err(ApiError::BadRequest(format!(
                "魔法不足，探索当前区域需要 {} 点魔法，当前只有 {} 点。",
                mp_cost, state_row.mp
            )));
        }
        state_row.mp -= mp_cost;
        lines.push(format!("你凝神探索当前区域，消耗 {} 点魔法。", mp_cost));
    }

    for round in 1..=rounds {
        let equipment_bonus = inventory_repo.equipment_bonus(character.id).await.unwrap_or_default();
        let skill_bonus = SkillRepository::new(state.db.pool())
            .bonus(character.id)
            .await
            .unwrap_or_default();
        let system_bonus = SystemsRepository::new(state.db.pool())
            .combat_bonus(character.id)
            .await
            .unwrap_or_default();
        let (max_hp, max_mp) = character_resource_caps(&character, &equipment_bonus, &skill_bonus, &system_bonus);
        clamp_resource_values(&mut state_row, max_hp, max_mp);
        if apply_vip_auto_potions(
            state,
            inventory_repo,
            character.id,
            &mut state_row,
            max_hp,
            max_mp,
            &mut lines,
        )
        .await?
        {
            inventory_changed = true;
        }
        let mut attacker = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
        if attacker.hp <= 0 {
            if try_origin_revive(
                inventory_repo,
                character_repo,
                character.id,
                &mut state_row,
                max_hp,
                max_mp,
                &mut lines,
            )
            .await?
            {
                continue;
            }
            let (hp, mp) = default_resources(&character);
            let return_name = death_return_name(&position);
            position = death_return_position(&position);
            state_row = character_repo
                .save_state_snapshot(character.id, &position, (hp / 2).max(1), (mp / 2).max(0))
                .await?;
            defeated = true;
            lines.push(format!("你已经倒下，回到{}休整。", return_name));
            if death_penalty_enabled(mode) {
                if apply_death_penalty(inventory_repo, character.id, &mut lines).await? {
                    inventory_changed = true;
                }
            } else {
                lines.push("挑战失败保护生效：本次死亡不会掉落背包或身上装备。".into());
            }
            break;
        }
        let auto_skill = try_auto_combat_skill(
            state,
            character.id,
            &auto_skills,
            &mut state_row,
            &attacker,
            &equipment_bonus,
            max_hp,
            &mut lines,
        )
        .await?;
        if matches!(auto_skill, Some(AutoSkillOutcome::Support)) {
            attacker = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
        }

        let defender = mob_combatant(&mob, mob_hp);
        let mut report = {
            let mut rng = thread_rng();
            match auto_skill {
                Some(AutoSkillOutcome::Attack { power, magical: true, .. }) => {
                    magical_damage(&mut rng, &attacker, &defender, power)
                }
                Some(AutoSkillOutcome::Attack { power, magical: false, .. }) => {
                    physical_damage(&mut rng, &attacker, &defender, power)
                }
                _ => physical_damage(&mut rng, &attacker, &defender, fight_mode_power(mode)),
            }
        };
        if let Some(effect @ AutoSkillOutcome::Attack { .. }) = auto_skill {
            apply_auto_skill_effects(
                &mut report,
                &attacker,
                &defender,
                mob.boss || matches!(mode, FightMode::WorldBoss),
                effect,
                &mut lines,
            );
        }
        let damage_bonus = apply_damage_bonuses(
            &mut report,
            &attacker,
            &defender,
            mob.boss || matches!(mode, FightMode::WorldBoss),
            matches!(mob_drop_kind(&mob, false), MobDropKind::Normal),
        );
        append_damage_bonus_lines(&mut lines, damage_bonus);
        apply_damage_bonus_restore(&mut state_row, max_hp, max_mp, damage_bonus, &mut lines);
        reset_skill_cooldowns_on_creation_strike(state, character.id, damage_bonus, &mut lines).await?;
        mob_hp = report.remaining_hp;
        lines.push(if report.hit {
            let crit = if report.crit { "暴击，" } else { "" };
            let heavy = if report.heavy { "重击，" } else { "" };
            format!("第 {} 回合：你攻击 {}，{}{}造成 {} 点伤害。", round, mob.name, crit, heavy, report.damage)
        } else {
            format!("第 {} 回合：{} 闪开了你的攻击。", round, mob.name)
        });
        if report.hit && report.damage > 0 && attacker.life_steal_pct > 0 {
            let heal = (report.damage * attacker.life_steal_pct.clamp(0, 80) / 100).max(1);
            state_row.hp = (state_row.hp + heal).min(attacker.max_hp).max(1);
            lines.push(format!("吸血生效，恢复 {} 点生命。", heal));
        }
        if report.hit && report.damage > 0 && attacker.mana_steal_pct > 0 {
            let restored = (report.damage * attacker.mana_steal_pct.clamp(0, 80) / 100).max(1);
            state_row.mp = (state_row.mp + restored).min(max_mp).max(0);
            lines.push(format!("吸蓝生效，恢复 {} 点魔法。", restored));
        }

        if mob_hp <= 0 {
            victory = true;
            let stamina = character_repo.consume_stamina_for_kill(character.id).await?;
            let stamina = apply_dominator_fatigue_privilege(stamina, &equipment_bonus);
            apply_stamina_to_state(&mut state_row, &stamina);
            append_stamina_single_log(&mut lines, &stamina);
            let fatigued = stamina.fatigue_kills > 0;
            let (base_exp, base_gold) = fight_mode_reward(mode, &mob);
            let (exp, gold) = fatigue_single_reward(base_exp, base_gold, fatigued);
            lines.push(format!("{} 被击败，获得 {} 经验和 {} 金币。", mob.name, exp, gold));
            let updated = character_repo.grant_reward(character.id, exp, gold).await?;
            leveled = updated.level > character.level;
            if leveled {
                lines.push(format!("境界突破：{} 升到 {} 级。", updated.name, updated.level));
            }
            character = inventory_repo.refresh_character_power(updated.id).await?;
            if let Some((hp, mp)) =
                apply_battle_end_restore(&mut state_row, max_hp, max_mp, equipment_bonus.battle_end_restore_pct)
            {
                lines.push(format!("沃玛套装续航生效，战斗结束恢复 {} 生命和 {} 魔法。", hp, mp));
            }

            if fatigued {
                lines.push("疲劳状态：装备与材料掉率为 0%。".into());
            } else {
                for (template_id, quantity) in fight_mode_drops(mode, attacker.luck) {
                    if let Some(item) = inventory_repo.grant_item(character.id, &template_id, quantity).await? {
                        inventory_changed = true;
                        lines.push(format!("奖励：{} x{} 已放入背包。", item.name, quantity));
                    }
                }
                if should_roll_regular_drops(mode) {
                    let source_id = drop_source_id(&position, &mob);
                    for drop in roll_level_drop_for_source(
                        mob.level,
                        mob_drop_kind(&mob, false),
                        attacker.luck,
                        Some(source_id.as_str()),
                        0,
                    ) {
                        if let Some(item) = inventory_repo.grant_item(character.id, &drop.template_id, drop.quantity).await? {
                            inventory_changed = true;
                            lines.push(format!("掉落：{} x{} 已放入背包。", item.name, drop.quantity));
                        }
                    }
                }
            }
            if let Ok(points) = ActivityRepository::new(state.db.pool())
                .add_points(character.id, "daily_hunt", 1)
                .await
            {
                lines.push(format!("每日猎魔进度 +1，当前 {}。", points));
            }
            QuestRepository::new(state.db.pool())
                .add_progress(character.id, "kill_any", 1)
                .await?;
            if let Some(message) = SystemsRepository::new(state.db.pool())
                .unlock_for_mob(character.id, &mob.id)
                .await?
            {
                lines.push(message);
            }
            if try_star_devourer_growth(state, inventory_repo, character.id, &mut lines).await? {
                character = inventory_repo.refresh_character_power(character.id).await?;
            }
            if leveled {
                state_row = character_repo.state(character.id).await?;
            }
            break;
        }

        let control = {
            let mut rng = thread_rng();
            report.hit.then(|| roll_control(&mut rng, &attacker, &defender)).flatten()
        };
        if let Some(effect) = control {
            lines.push(format!("{}生效，{} 本回合无法反击。", effect, mob.name));
            continue;
        }

        let mob_attacker = mob_combatant(&mob, mob_hp.max(1));
        let mut counter_target = attacker.clone();
        counter_target.hp = state_row.hp.clamp(0, counter_target.max_hp);
        counter_target.max_mp = max_mp;
        let counter = {
            let mut rng = thread_rng();
            physical_damage(&mut rng, &mob_attacker, &counter_target, fight_mode_counter_power(mode))
        };
        state_row.hp = counter.remaining_hp;
        if counter.hit {
            lines.push(format!("{} 反击，造成 {} 点伤害。", mob.name, counter.damage));
        } else {
            lines.push(format!("你闪开了 {} 的反击。", mob.name));
        }
        if state_row.hp <= 0 {
            if try_origin_revive(
                inventory_repo,
                character_repo,
                character.id,
                &mut state_row,
                max_hp,
                max_mp,
                &mut lines,
            )
            .await?
            {
                continue;
            }
            let (hp, mp) = default_resources(&character);
            let return_name = death_return_name(&position);
            position = death_return_position(&position);
            state_row = character_repo
                .save_state_snapshot(character.id, &position, (hp / 2).max(1), (mp / 2).max(0))
                .await?;
            defeated = true;
            lines.push(format!("你被击倒，回到{}休整。", return_name));
            if death_penalty_enabled(mode) {
                if apply_death_penalty(inventory_repo, character.id, &mut lines).await? {
                    inventory_changed = true;
                }
            } else {
                lines.push("挑战失败保护生效：本次死亡不会掉落背包或身上装备。".into());
            }
            break;
        }
    }

    if !victory && !defeated && state_row.hp > 0 {
        lines.push(format!(
            "战斗持续 {} 回合仍未分出胜负，{} 仍有 {} 点生命；你暂时撤出战斗。",
            rounds,
            mob.name,
            mob_hp.max(1)
        ));
    }
    if state_row.hp > 0 && !leveled {
        let (max_hp, max_mp) = character_max_resources(state, inventory_repo, &character).await;
        clamp_resource_values(&mut state_row, max_hp, max_mp);
        state_row = character_repo
            .save_resources(character.id, state_row.hp, state_row.mp)
            .await?;
    }
    let inventory = if inventory_changed {
        Some(inventory_repo.view(character.id, character.level).await?)
    } else {
        None
    };
    let mut result = realtime_result(
        state,
        character.clone(),
        state_row.clone(),
        &position,
        inventory,
        compact_combat_log(lines, combat_log_limit(mode)),
    )
    .await?;
    if victory && matches!(mode, FightMode::AreaExplore | FightMode::WildAfk) {
        result.adventure = AdventureRepository::new(state.db.pool())
            .maybe_trigger(&character, &position.zone, &position.room, "combat", 40)
            .await?;
    }
    Ok(FightOutcome {
        result,
        victory,
        character,
        state_row,
        position,
        inventory_changed,
    })
}

async fn realtime_attack(
    state: AppState,
    headers: HeaderMap,
    character_id: i64,
    target_id: i64,
    skill_id: Option<String>,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let (character_repo, inventory_repo, character) = resolve_character(&state, &headers, Some(character_id)).await?;
    let mut character = character.ok_or(ApiError::NotFound)?;
    let (mut state_row, mut position) = active_position(&state, &character_repo, &character).await?;
    let room = state
        .world
        .current_room(&position)
        .map_err(|_| ApiError::BadRequest("房间不存在".into()))?
        .clone();
    if room.spawns.is_empty() {
        return Err(ApiError::BadRequest("这里没有怪物".into()));
    }
    let target_index = usize::try_from(target_id).map_err(|_| ApiError::BadRequest("目标不存在".into()))?;
    let Some(mob_id) = room.spawns.get(target_index) else {
        return Err(ApiError::BadRequest("目标不存在".into()));
    };
    let mob = select_mob(state.mobs.as_ref().as_slice(), mob_id)?.clone();

    let mut active_skill = None;
    if let Some(skill_id) = skill_id.as_deref() {
        let skill = SkillRepository::new(state.db.pool())
            .active_for_character(character.id, skill_id.trim())
            .await
            .map_err(skill_action_error)?;
        if skill_kind(&skill) == "passive" {
            return Err(ApiError::BadRequest("被动技能学习后会直接继承属性，无需释放。".into()));
        }
        if skill_kind(&skill) == "heal" {
            return heal_with_http(state, character_repo, character, state_row, position, skill).await;
        }
        active_skill = Some(skill);
    }

    let mut lines = Vec::new();
    let mut mob_hp = mob.max_hp.max(1);
    let mut inventory_changed = false;
    let mut leveled = false;
    let mut skill_used = false;
    let mut victory = false;
    let mut defeated = false;
    let round_limit = combat_round_limit(FightMode::AreaExplore, character.level, &mob);

    for round in 1..=round_limit {
        let equipment_bonus = inventory_repo.equipment_bonus(character.id).await.unwrap_or_default();
        let skill_bonus = SkillRepository::new(state.db.pool())
            .bonus(character.id)
            .await
            .unwrap_or_default();
        let system_bonus = SystemsRepository::new(state.db.pool())
            .combat_bonus(character.id)
            .await
            .unwrap_or_default();
        let (max_hp, max_mp) = character_resource_caps(&character, &equipment_bonus, &skill_bonus, &system_bonus);
        clamp_resource_values(&mut state_row, max_hp, max_mp);
        let mut power = 1.0;
        let mut magical = false;
        let mut active_effect = None;
        if let Some(skill) = active_skill.as_ref().filter(|_| !skill_used) {
            let mp_cost = effective_skill_mp_cost(skill, &equipment_bonus);
            if state_row.mp < mp_cost {
                return Err(ApiError::BadRequest("魔法值不足".into()));
            }
            SkillRepository::new(state.db.pool())
                .mark_used(character.id, &skill.id, skill.cooldown_ms)
                .await
                .map_err(|_| ApiError::BadRequest("技能冷却中".into()))?;
            let current_mp = state_row.mp.max(0);
            let mp_damage_pct = skill.config.get("mp_to_damage_pct").and_then(serde_json::Value::as_i64).unwrap_or_default();
            let current_mp_cost_pct = skill.config.get("current_mp_cost_pct").and_then(serde_json::Value::as_i64).unwrap_or_default();
            state_row.mp = (state_row.mp - mp_cost).max(0);
            power = skill_power(skill, &equipment_bonus);
            magical = skill_kind(skill) == "magical";
            active_effect = Some(AutoSkillOutcome::Attack {
                power,
                magical,
                target_current_hp_pct: skill
                    .config
                    .get("target_current_hp_pct")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or_default(),
                boss_cap_atk_multiplier: skill
                    .config
                    .get("boss_extra_damage_cap_atk")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(5),
                execute_threshold_pct: skill
                    .config
                    .get("execute_threshold_pct")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or_default(),
                execute_bonus_pct: skill
                    .config
                    .get("execute_bonus_pct")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or_default(),
                flat_bonus_damage: current_mp
                    .saturating_mul(current_mp_cost_pct.clamp(0, 50))
                    .saturating_mul(mp_damage_pct.clamp(0, 200))
                    / 10_000,
            });
            skill_used = true;
            lines.push(format!(
                "{} 释放 {}，消耗 {} 点魔法。",
                character.name,
                skill.name,
                mp_cost
            ));
        }
        if apply_vip_auto_potions(
            &state,
            &inventory_repo,
            character.id,
            &mut state_row,
            max_hp,
            max_mp,
            &mut lines,
        )
        .await?
        {
            inventory_changed = true;
        }
        let attacker = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
        if attacker.hp <= 0 {
            if try_origin_revive(
                &inventory_repo,
                &character_repo,
                character.id,
                &mut state_row,
                max_hp,
                max_mp,
                &mut lines,
            )
            .await?
            {
                continue;
            }
            let (hp, mp) = default_resources(&character);
            let return_name = death_return_name(&position);
            position = death_return_position(&position);
            state_row = character_repo
                .save_state_snapshot(character.id, &position, (hp / 2).max(1), (mp / 2).max(0))
                .await?;
            defeated = true;
            lines.push(format!("你已经倒下，回到{}休整。", return_name));
            if apply_death_penalty(&inventory_repo, character.id, &mut lines).await? {
                inventory_changed = true;
            }
            break;
        }

        let defender = mob_combatant(&mob, mob_hp);
        let mut report = {
            let mut rng = thread_rng();
            if magical {
                magical_damage(&mut rng, &attacker, &defender, power)
            } else {
                physical_damage(&mut rng, &attacker, &defender, power)
            }
        };
        if let Some(effect) = active_effect {
            apply_auto_skill_effects(&mut report, &attacker, &defender, mob.boss, effect, &mut lines);
        }
        let damage_bonus = apply_damage_bonuses(
            &mut report,
            &attacker,
            &defender,
            mob.boss,
            matches!(mob_drop_kind(&mob, false), MobDropKind::Normal),
        );
        append_damage_bonus_lines(&mut lines, damage_bonus);
        apply_damage_bonus_restore(&mut state_row, max_hp, max_mp, damage_bonus, &mut lines);
        reset_skill_cooldowns_on_creation_strike(&state, character.id, damage_bonus, &mut lines).await?;
        mob_hp = report.remaining_hp;
        lines.push(if report.hit {
            let crit = if report.crit { "暴击，" } else { "" };
            let heavy = if report.heavy { "重击，" } else { "" };
            format!("第 {} 回合：{} 攻击 {}，{}{}造成 {} 点伤害。", round, attacker.name, mob.name, crit, heavy, report.damage)
        } else {
            format!("第 {} 回合：{} 的攻击被 {} 闪开。", round, attacker.name, mob.name)
        });
        if report.hit && report.damage > 0 && attacker.life_steal_pct > 0 {
            let heal = (report.damage * attacker.life_steal_pct.clamp(0, 80) / 100).max(1);
            state_row.hp = (state_row.hp + heal).min(attacker.max_hp).max(1);
            lines.push(format!("吸血生效，恢复 {} 点生命。", heal));
        }
        if report.hit && report.damage > 0 && attacker.mana_steal_pct > 0 {
            let restored = (report.damage * attacker.mana_steal_pct.clamp(0, 80) / 100).max(1);
            state_row.mp = (state_row.mp + restored).min(max_mp).max(0);
            lines.push(format!("吸蓝生效，恢复 {} 点魔法。", restored));
        }

        if mob_hp <= 0 {
            victory = true;
            let stamina = character_repo.consume_stamina_for_kill(character.id).await?;
            let stamina = apply_dominator_fatigue_privilege(stamina, &equipment_bonus);
            apply_stamina_to_state(&mut state_row, &stamina);
            append_stamina_single_log(&mut lines, &stamina);
            let fatigued = stamina.fatigue_kills > 0;
            let (exp, gold) = fatigue_single_reward(mob.exp, mob.gold, fatigued);
            lines.push(format!("{} 被击败，获得 {} 经验和 {} 金币。", mob.name, exp, gold));
            let updated = character_repo.grant_reward(character.id, exp, gold).await?;
            leveled = updated.level > character.level;
            if leveled {
                lines.push(format!("境界突破：{} 升到 {} 级。", updated.name, updated.level));
            }
            character = inventory_repo.refresh_character_power(updated.id).await?;
            if let Some((hp, mp)) =
                apply_battle_end_restore(&mut state_row, max_hp, max_mp, equipment_bonus.battle_end_restore_pct)
            {
                lines.push(format!("沃玛套装续航生效，战斗结束恢复 {} 生命和 {} 魔法。", hp, mp));
            }

            if fatigued {
                lines.push("疲劳状态：装备与材料掉率为 0%。".into());
            } else {
                let source_id = drop_source_id(&position, &mob);
                for drop in roll_level_drop_for_source(
                    mob.level,
                    mob_drop_kind(&mob, false),
                    attacker.luck,
                    Some(source_id.as_str()),
                    0,
                ) {
                    if let Some(item) = inventory_repo.grant_item(character.id, &drop.template_id, drop.quantity).await? {
                        inventory_changed = true;
                        lines.push(format!("掉落：{} x{} 已放入背包。", item.name, drop.quantity));
                    }
                }
            }
            if let Ok(points) = ActivityRepository::new(state.db.pool())
                .add_points(character.id, "daily_hunt", 1)
                .await
            {
                lines.push(format!("每日猎魔进度 +1，当前 {}。", points));
            }
            QuestRepository::new(state.db.pool())
                .add_progress(character.id, "kill_any", 1)
                .await?;
            if let Some(message) = SystemsRepository::new(state.db.pool())
                .unlock_for_mob(character.id, &mob.id)
                .await?
            {
                lines.push(message);
            }
            if leveled {
                state_row = character_repo.state(character.id).await?;
            }
            break;
        }

        let control = {
            let mut rng = thread_rng();
            report.hit.then(|| roll_control(&mut rng, &attacker, &defender)).flatten()
        };
        if let Some(effect) = control {
            lines.push(format!("{}生效，{} 本回合无法反击。", effect, mob.name));
            continue;
        }

        let mob_attacker = mob_combatant(&mob, mob_hp.max(1));
        let mut counter_target = attacker.clone();
        counter_target.hp = state_row.hp.clamp(0, counter_target.max_hp);
        counter_target.max_mp = max_mp;
        let counter = {
            let mut rng = thread_rng();
            physical_damage(&mut rng, &mob_attacker, &counter_target, if mob.boss { 1.2 } else { 1.0 })
        };
        state_row.hp = counter.remaining_hp;
        if counter.hit {
            lines.push(format!("{} 反击，造成 {} 点伤害。", mob.name, counter.damage));
        } else {
            lines.push(format!("你闪开了 {} 的反击。", mob.name));
        }
        if state_row.hp <= 0 {
            if try_origin_revive(
                &inventory_repo,
                &character_repo,
                character.id,
                &mut state_row,
                max_hp,
                max_mp,
                &mut lines,
            )
            .await?
            {
                continue;
            }
            let (hp, mp) = default_resources(&character);
            let return_name = death_return_name(&position);
            position = death_return_position(&position);
            state_row = character_repo
                .save_state_snapshot(character.id, &position, (hp / 2).max(1), (mp / 2).max(0))
                .await?;
            defeated = true;
            lines.push(format!("你被击倒，回到{}休整。", return_name));
            if apply_death_penalty(&inventory_repo, character.id, &mut lines).await? {
                inventory_changed = true;
            }
            break;
        }
    }

    if !victory && !defeated && state_row.hp > 0 {
        lines.push(format!(
            "战斗持续 {} 回合仍未分出胜负，{} 仍有 {} 点生命；你暂时撤出战斗。",
            round_limit,
            mob.name,
            mob_hp.max(1)
        ));
    }

    if state_row.hp > 0 && !leveled {
        let (max_hp, max_mp) = character_max_resources(&state, &inventory_repo, &character).await;
        clamp_resource_values(&mut state_row, max_hp, max_mp);
        state_row = character_repo
            .save_resources(character.id, state_row.hp, state_row.mp)
            .await?;
    }

    let inventory = if inventory_changed {
        Some(inventory_repo.view(character.id, character.level).await?)
    } else {
        None
    };
    let mut response = realtime_result(
        &state,
        character.clone(),
        state_row,
        &position,
        inventory,
        compact_combat_log(lines, 120),
    )
    .await?;
    if victory {
        response.adventure = AdventureRepository::new(state.db.pool())
            .maybe_trigger(&character, &position.zone, &position.room, "combat", 40)
            .await?;
    }
    Ok(Json(ApiOk::new(response)))
}

async fn fight_bot_target(
    state: &AppState,
    character_repo: &CharacterRepository,
    inventory_repo: &InventoryRepository,
    character: CharacterRecord,
    mut state_row: CharacterStateRecord,
    mut position: Position,
    target_index: i64,
    automatic: bool,
) -> Result<RealtimeActionResult, ApiError> {
    let room = state
        .world
        .current_room(&position)
        .map_err(|_| ApiError::BadRequest("房间不存在".into()))?;
    if room.safe {
        return Err(ApiError::BadRequest("安全区不能 PK。".into()));
    }
    if !state_row.pk_enabled {
        return Err(ApiError::BadRequest("请先开启 PK 模式。".into()));
    }
    if automatic && !state_row.sweep_attack_players {
        return Err(ApiError::BadRequest("当前扫荡不会攻击玩家。".into()));
    }

    let bot_repo = BotRepository::new(state.db.pool());
    let target = bot_repo
        .pvp_target_at(&position.zone, &position.room, character.id, target_index)
        .await?
        .ok_or_else(|| ApiError::BadRequest("当前地图没有可攻击的玩家。".into()))?;
    let (bot_max_hp, bot_max_mp) = bot_resource_caps(target.level);
    let mut bot_hp = target.hp.clamp(1, bot_max_hp.max(1));
    let bot_mp = target.mp.clamp(0, bot_max_mp.max(0));
    let mut lines = if automatic {
        vec![format!("PK 扫荡：你锁定了同屏玩家 {}。", target.name)]
    } else {
        vec![format!("你向同屏玩家 {} 发起 PK。", target.name)]
    };
    let mut inventory_changed = false;
    let mut defeated_bot = false;
    let auto_skills = SkillRepository::new(state.db.pool())
        .active_skills_for_character(character.id)
        .await?;

    for round in 1..=10 {
        let equipment_bonus = inventory_repo.equipment_bonus(character.id).await.unwrap_or_default();
        let skill_bonus = SkillRepository::new(state.db.pool())
            .bonus(character.id)
            .await
            .unwrap_or_default();
        let system_bonus = SystemsRepository::new(state.db.pool())
            .combat_bonus(character.id)
            .await
            .unwrap_or_default();
        let (max_hp, max_mp) = character_resource_caps(&character, &equipment_bonus, &skill_bonus, &system_bonus);
        clamp_resource_values(&mut state_row, max_hp, max_mp);
        if apply_vip_auto_potions(
            state,
            inventory_repo,
            character.id,
            &mut state_row,
            max_hp,
            max_mp,
            &mut lines,
        )
        .await?
        {
            inventory_changed = true;
        }

        let mut attacker = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
        if attacker.hp <= 0 {
            if try_origin_revive(
                inventory_repo,
                character_repo,
                character.id,
                &mut state_row,
                max_hp,
                max_mp,
                &mut lines,
            )
            .await?
            {
                continue;
            }
            break;
        }
        let auto_skill = try_auto_combat_skill(
            state,
            character.id,
            &auto_skills,
            &mut state_row,
            &attacker,
            &equipment_bonus,
            max_hp,
            &mut lines,
        )
        .await?;
        if matches!(auto_skill, Some(AutoSkillOutcome::Support)) {
            attacker = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
        }
        let defender = bot_combatant(&target, bot_hp);
        let mut report = {
            let mut rng = thread_rng();
            match auto_skill {
                Some(AutoSkillOutcome::Attack { power, magical: true, .. }) => {
                    magical_damage(&mut rng, &attacker, &defender, power)
                }
                Some(AutoSkillOutcome::Attack { power, magical: false, .. }) => {
                    physical_damage(&mut rng, &attacker, &defender, power)
                }
                _ => physical_damage(&mut rng, &attacker, &defender, 1.0),
            }
        };
        if let Some(effect @ AutoSkillOutcome::Attack { .. }) = auto_skill {
            apply_auto_skill_effects(&mut report, &attacker, &defender, false, effect, &mut lines);
        }
        bot_hp = report.remaining_hp;
        lines.push(if report.hit {
            let crit = if report.crit { "暴击，" } else { "" };
            format!("第 {} 回合：你攻击 {}，{}造成 {} 点伤害。", round, target.name, crit, report.damage)
        } else {
            format!("第 {} 回合：{} 闪开了你的攻击。", round, target.name)
        });
        if report.hit && report.damage > 0 && attacker.life_steal_pct > 0 {
            let heal = (report.damage * attacker.life_steal_pct.clamp(0, 80) / 100).max(1);
            state_row.hp = (state_row.hp + heal).min(attacker.max_hp).max(1);
            lines.push(format!("吸血生效，恢复 {} 点生命。", heal));
        }
        if report.hit && report.damage > 0 && attacker.mana_steal_pct > 0 {
            let restored = (report.damage * attacker.mana_steal_pct.clamp(0, 80) / 100).max(1);
            state_row.mp = (state_row.mp + restored).min(max_mp).max(0);
            lines.push(format!("吸蓝生效，恢复 {} 点魔法。", restored));
        }
        if bot_hp <= 0 {
            defeated_bot = true;
            lines.push(format!("{} 被击退，回到青牛城休整。", target.name));
            break;
        }

        let defender = bot_combatant(&target, bot_hp);
        let control = {
            let mut rng = thread_rng();
            report.hit.then(|| roll_control(&mut rng, &attacker, &defender)).flatten()
        };
        if let Some(effect) = control {
            lines.push(format!("{}生效，{} 本回合无法反击。", effect, target.name));
            continue;
        }

        let mut counter_target = attacker.clone();
        counter_target.hp = state_row.hp.clamp(0, counter_target.max_hp);
        counter_target.max_mp = max_mp;
        let mut counter = {
            let mut rng = thread_rng();
            physical_damage(
                &mut rng,
                &defender,
                &counter_target,
                bot_pvp_counter_power(target.level, character.level),
            )
        };
        if counter.hit {
            let floor = bot_pvp_damage_floor(target.level, character.level, counter_target.max_hp);
            if counter.damage < floor {
                counter.damage = floor.min(counter_target.hp.max(1));
                counter.remaining_hp = (counter_target.hp - counter.damage).max(0);
            }
        }
        state_row.hp = counter.remaining_hp;
        if counter.hit {
            lines.push(format!("{} 反击，造成 {} 点伤害。", target.name, counter.damage));
        } else {
            lines.push(format!("你闪开了 {} 的反击。", target.name));
        }
        if state_row.hp <= 0 {
            if try_origin_revive(
                inventory_repo,
                character_repo,
                character.id,
                &mut state_row,
                max_hp,
                max_mp,
                &mut lines,
            )
            .await?
            {
                continue;
            }
            let (hp, mp) = default_resources(&character);
            let return_name = death_return_name(&position);
            position = death_return_position(&position);
            state_row = character_repo
                .save_state_snapshot(character.id, &position, (hp / 2).max(1), (mp / 2).max(0))
                .await?;
            lines.push(format!("你被 {} 击倒，回到{}休整。", target.name, return_name));
            if apply_death_penalty(inventory_repo, character.id, &mut lines).await? {
                inventory_changed = true;
            }
            break;
        }
    }

    bot_repo
        .save_pvp_state(&target, bot_hp, bot_mp, defeated_bot)
        .await?;
    if state_row.hp > 0 {
        let (max_hp, max_mp) = character_max_resources(state, inventory_repo, &character).await;
        clamp_resource_values(&mut state_row, max_hp, max_mp);
        state_row = character_repo
            .save_resources(character.id, state_row.hp, state_row.mp)
            .await?;
    }
    let inventory = if inventory_changed {
        Some(inventory_repo.view(character.id, character.level).await?)
    } else {
        None
    };
    realtime_result(state, character, state_row, &position, inventory, lines).await
}

async fn heal_with_http(
    state: AppState,
    character_repo: CharacterRepository,
    character: CharacterRecord,
    mut state_row: CharacterStateRecord,
    position: Position,
    skill: ActiveSkillRecord,
) -> Result<Json<ApiOk<RealtimeActionResult>>, ApiError> {
    let equipment_bonus = InventoryRepository::new(state.db.pool())
        .equipment_bonus(character.id)
        .await
        .unwrap_or_default();
    let skill_bonus = SkillRepository::new(state.db.pool())
        .bonus(character.id)
        .await
        .unwrap_or_default();
    let system_bonus = SystemsRepository::new(state.db.pool())
        .combat_bonus(character.id)
        .await
        .unwrap_or_default();
    let (max_hp, max_mp) = character_resource_caps(&character, &equipment_bonus, &skill_bonus, &system_bonus);
    clamp_resource_values(&mut state_row, max_hp, max_mp);
    let mp_cost = effective_skill_mp_cost(&skill, &equipment_bonus);
    if state_row.mp < mp_cost {
        return Err(ApiError::BadRequest("魔法值不足".into()));
    }
    SkillRepository::new(state.db.pool())
        .mark_used(character.id, &skill.id, skill.cooldown_ms)
        .await
        .map_err(|_| ApiError::BadRequest("技能冷却中".into()))?;
    let combatant = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
    let heal = skill_heal(&skill, &equipment_bonus).saturating_add(combatant.mag / 2);
    state_row.mp = (state_row.mp - mp_cost).max(0);
    state_row.hp = (state_row.hp + heal).min(combatant.max_hp).max(1);
    state_row = character_repo
        .save_resources(character.id, state_row.hp, state_row.mp)
        .await?;
    let log = vec![format!(
        "{} 施展 {}，消耗 {} 点魔法，恢复 {} 点生命。",
        character.name,
        skill.name,
        mp_cost,
        heal
    )];
    Ok(Json(ApiOk::new(realtime_result(
        &state,
        character,
        state_row,
        &position,
        None,
        log,
    )
    .await?)))
}

async fn try_auto_combat_skill(
    state: &AppState,
    character_id: i64,
    skills: &[ActiveSkillRecord],
    state_row: &mut CharacterStateRecord,
    attacker: &Combatant,
    equipment: &EquipmentBonus,
    max_hp: i64,
    lines: &mut Vec<String>,
) -> Result<Option<AutoSkillOutcome>, ApiError> {
    if skills.is_empty() {
        return Ok(None);
    }

    let needs_heal = max_hp > 0 && state_row.hp.saturating_mul(100) <= max_hp.saturating_mul(55);
    if needs_heal {
        let mut heal_candidates = skills
            .iter()
            .filter(|skill| skill_kind(skill) == "heal" && state_row.mp >= effective_skill_mp_cost(skill, equipment))
            .collect::<Vec<_>>();
        {
            let mut rng = thread_rng();
            heal_candidates.shuffle(&mut rng);
        }
        for skill in heal_candidates {
            let mp_cost = effective_skill_mp_cost(skill, equipment);
            match SkillRepository::new(state.db.pool())
                .mark_used(character_id, &skill.id, 0)
                .await
            {
                Ok(()) => {
                    let heal = skill_heal(skill, equipment).saturating_add(attacker.mag / 2);
                    state_row.mp = (state_row.mp - mp_cost).max(0);
                    state_row.hp = (state_row.hp + heal).min(max_hp).max(1);
                    lines.push(format!(
                        "{} 自动施展 {}，消耗 {} 点魔法，恢复 {} 点生命。",
                        attacker.name, skill.name, mp_cost, heal
                    ));
                    return Ok(Some(AutoSkillOutcome::Support));
                }
                Err(sqlx::Error::RowNotFound) => continue,
                Err(err) => return Err(ApiError::Database(err)),
            }
        }
    }

    let mut attack_candidates = skills
        .iter()
        .filter(|skill| skill_kind(skill) != "heal" && state_row.mp >= effective_skill_mp_cost(skill, equipment))
        .collect::<Vec<_>>();
    {
        let mut rng = thread_rng();
        attack_candidates.shuffle(&mut rng);
    }
    for skill in attack_candidates {
        let mp_cost = effective_skill_mp_cost(skill, equipment);
        match SkillRepository::new(state.db.pool())
            .mark_used(character_id, &skill.id, 0)
            .await
        {
            Ok(()) => {
                let current_mp = state_row.mp.max(0);
                let mp_damage_pct = skill.config.get("mp_to_damage_pct").and_then(serde_json::Value::as_i64).unwrap_or_default();
                let current_mp_cost_pct = skill.config.get("current_mp_cost_pct").and_then(serde_json::Value::as_i64).unwrap_or_default();
                let flat_bonus_damage = current_mp
                    .saturating_mul(current_mp_cost_pct.clamp(0, 50))
                    .saturating_mul(mp_damage_pct.clamp(0, 200))
                    / 10_000;
                state_row.mp = (state_row.mp - mp_cost).max(0);
                lines.push(format!(
                    "{} 自动释放 {}，消耗 {} 点魔法。",
                    attacker.name, skill.name, mp_cost
                ));
                return Ok(Some(AutoSkillOutcome::Attack {
                    power: skill_power(skill, equipment),
                    magical: skill_kind(skill) == "magical",
                    target_current_hp_pct: skill
                        .config
                        .get("target_current_hp_pct")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or_default(),
                    boss_cap_atk_multiplier: skill
                        .config
                        .get("boss_extra_damage_cap_atk")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or(5),
                    execute_threshold_pct: skill
                        .config
                        .get("execute_threshold_pct")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or_default(),
                    execute_bonus_pct: skill
                        .config
                        .get("execute_bonus_pct")
                        .and_then(serde_json::Value::as_i64)
                        .unwrap_or_default(),
                    flat_bonus_damage,
                }));
            }
            Err(sqlx::Error::RowNotFound) => continue,
            Err(err) => return Err(ApiError::Database(err)),
        }
    }

    Ok(None)
}

async fn active_position(
    state: &AppState,
    character_repo: &CharacterRepository,
    character: &CharacterRecord,
) -> Result<(CharacterStateRecord, Position), ApiError> {
    let mut state_row = character_repo.state(character.id).await?;
    let mut position = Position { zone: state_row.zone.clone(), room: state_row.room.clone() };
    if state.world.current_room(&position).is_err() || state_row.hp <= 0 {
        position = if state.world.current_room(&position).is_err() {
            start_position()
        } else {
            death_return_position(&position)
        };
        let (hp, mp) = default_resources(character);
        let saved = character_repo
            .save_state_snapshot(character.id, &position, (hp / 2).max(1), (mp / 2).max(0))
            .await?;
        return Ok((saved, position));
    }
    let inventory_repo = InventoryRepository::new(state.db.pool());
    let (max_hp, max_mp) = character_max_resources(state, &inventory_repo, character).await;
    let (hp_per_minute, mp_per_minute) = idle_recovery_rates(max_hp, max_mp);
    let recovered = character_repo
        .recover_idle_resources(character.id, max_hp, max_mp, hp_per_minute, mp_per_minute)
        .await?;
    let _ = (recovered.minutes, recovered.hp, recovered.mp);
    state_row = recovered.state;
    let _ = SystemsRepository::new(state.db.pool())
        .unlock_for_position(character.id, &position.zone, &position.room)
        .await?;
    Ok((state_row, position))
}

async fn realtime_result(
    state: &AppState,
    character: CharacterRecord,
    state_row: CharacterStateRecord,
    position: &Position,
    inventory: Option<InventoryView>,
    log: Vec<String>,
) -> Result<RealtimeActionResult, ApiError> {
    Ok(RealtimeActionResult {
        room: room_state_for_position(state, position, Some(character.id)).await?,
        log,
        character: character_view(character),
        state: state_view(state_row),
        inventory,
        adventure: None,
    })
}

async fn room_state_for_position(
    state: &AppState,
    position: &Position,
    current_character_id: Option<i64>,
) -> Result<RoomStateEvent, ApiError> {
    let mut room = state
        .world
        .current_room(position)
        .map_err(|_| ApiError::BadRequest("房间不存在".into()))?
        .clone();
    apply_sabak_room_description(state, position, &mut room).await;
    let mobs = room
        .spawns
        .iter()
        .map(|id| {
            let mob = select_mob(state.mobs.as_ref().as_slice(), id)?;
            Ok(format!("{} {} 级 生命 {}/{}", mob.name, mob.level, mob.max_hp, mob.max_hp))
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    let players = BotRepository::new(state.db.pool())
        .names_at(&position.zone, &position.room, current_character_id)
        .await
        .unwrap_or_default();
    Ok(RoomStateEvent { room, players, mobs })
}

async fn apply_sabak_room_description(state: &AppState, position: &Position, room: &mut cq_domain::map::Room) {
    if position.zone != "mengzhong" || position.room != "town" {
        return;
    }
    let owner = GuildRepository::new(state.db.pool())
        .sabak_owner_name()
        .await
        .unwrap_or_else(|_| "比奇远征队".into());
    room.desc = format!(
        "【沙巴克霸主】：当前由行会 [{}] 统治！盟重的黄沙将为其加冕！\n\n{}",
        owner, room.desc
    );
}

fn select_mob<'a>(mobs: &'a [MobTemplate], id: &str) -> Result<&'a MobTemplate, ApiError> {
    mobs.iter()
        .find(|mob| mob.id == id)
        .or_else(|| mobs.first())
        .ok_or_else(|| ApiError::Internal("empty mob table".into()))
}

fn drop_source_id(position: &Position, mob: &MobTemplate) -> String {
    format!("{}:{}:{}", position.zone, position.room, mob.id)
}

fn mob_combatant(mob: &MobTemplate, hp: i64) -> Combatant {
    Combatant {
        id: 0,
        name: mob.name.clone(),
        level: mob.level,
        hp: hp.max(0),
        max_hp: mob.max_hp,
        max_mp: 0,
        atk: mob.atk,
        def: mob.def,
        mag: mob.atk / 2,
        mdef: mob.def / 2,
        dex: i64::from(mob.level) + 5,
        crit_pct: if mob.boss { 8 } else { 2 },
        luck: 0,
        heavy_hit_pct: 0,
        life_steal_pct: 0,
        mana_steal_pct: 0,
        paralyze_pct: 0,
        petrify_pct: 0,
        paralyze_resist_pct: 0,
        petrify_resist_pct: 0,
        crit_damage_pct: 0,
        boss_damage_pct: 0,
        damage_deepen_pct: 0,
        normal_mob_execute_pct: 0,
        damage_reduce_pct: high_level_mob_damage_reduce(mob),
        ignore_def_pct: if mob.id == "world_boss_eternal_abyss_demon" { 50 } else { 0 },
        guaranteed_hit_pct: 0,
        target_max_hp_true_damage_pct: 0,
        self_max_mp_true_damage_pct: 0,
        creation_strike_pct: 0,
        creation_strike_damage_pct: 0,
        creation_strike_full_restore: false,
        control_immune: false,
    }
}

fn high_level_mob_damage_reduce(mob: &MobTemplate) -> i64 {
    if mob.boss {
        return ((max_drop_tier_for_mob_level(mob.level) - 5) * 15 / 10).clamp(0, 90) as i64;
    }
    0
}

fn max_drop_tier_for_mob_level(level: i32) -> i32 {
    match level.max(1) {
        1..=20 => 1,
        21..=40 => 2,
        41..=80 => 3,
        81..=120 => 4,
        121..=160 => 5,
        161..=200 => 6,
        201..=220 => 7,
        221..=240 => 8,
        241..=280 => 9,
        281..=300 => 10,
        301..=340 => 11,
        341..=380 => 12,
        381..=420 => 13,
        421..=460 => 14,
        461..=480 => 15,
        481..=499 => 16,
        _ => 17,
    }
}

fn bot_combatant(bot: &BotPvpTarget, hp: i64) -> Combatant {
    let level_bonus = i64::from(bot.level.saturating_sub(1));
    let (max_hp, _) = bot_resource_caps(bot.level);
    let class_mag_bonus = if bot.bot_class == "mage" || bot.bot_class == "taoist" { level_bonus * 3 } else { 0 };
    let class_atk_bonus = if bot.bot_class == "warrior" || bot.bot_class == "assassin" {
        level_bonus * 4
    } else {
        level_bonus * 2
    };
    Combatant {
        id: bot.id,
        name: bot.name.clone(),
        level: bot.level,
        hp: hp.clamp(0, max_hp),
        max_hp,
        max_mp: 0,
        atk: 34 + level_bonus * 5 + bot.power / 55 + class_atk_bonus,
        def: 10 + level_bonus * 2 + bot.power / 220,
        mag: 8 + level_bonus * 2 + bot.power / 95 + class_mag_bonus,
        mdef: 8 + level_bonus * 2 + bot.power / 260,
        dex: 12 + level_bonus * 2,
        crit_pct: 5 + level_bonus / 12,
        luck: 10 + level_bonus * 2,
        heavy_hit_pct: if bot.bot_class == "warrior" { 4 + level_bonus / 20 } else { 0 },
        life_steal_pct: 0,
        mana_steal_pct: 0,
        paralyze_pct: 0,
        petrify_pct: 0,
        paralyze_resist_pct: 0,
        petrify_resist_pct: 0,
        crit_damage_pct: 0,
        boss_damage_pct: 0,
        damage_deepen_pct: 0,
        normal_mob_execute_pct: 0,
        damage_reduce_pct: 0,
        ignore_def_pct: 0,
        guaranteed_hit_pct: 0,
        target_max_hp_true_damage_pct: 0,
        self_max_mp_true_damage_pct: 0,
        creation_strike_pct: 0,
        creation_strike_damage_pct: 0,
        creation_strike_full_restore: false,
        control_immune: false,
    }
}

fn bot_resource_caps(level: i32) -> (i64, i64) {
    let level_bonus = i64::from(level.saturating_sub(1));
    (160 + level_bonus * 24, 60 + level_bonus * 8)
}

fn bot_pvp_counter_power(bot_level: i32, character_level: i32) -> f64 {
    let delta = bot_level - character_level;
    if delta >= 0 {
        (1.0 + f64::from(delta.min(20)) * 0.035).clamp(1.0, 1.7)
    } else {
        (1.0 + f64::from(delta.max(-30)) * 0.015).clamp(0.75, 1.0)
    }
}

fn bot_pvp_damage_floor(bot_level: i32, character_level: i32, character_max_hp: i64) -> i64 {
    let delta = i64::from(bot_level - character_level);
    let base = if delta > 0 {
        6 + delta.saturating_mul(3) + i64::from(bot_level) / 2
    } else {
        3 + i64::from(bot_level.max(1)) / 5
    };
    let cap = (character_max_hp / 5).clamp(8, 160);
    base.clamp(3, cap)
}

async fn character_max_resources(
    state: &AppState,
    inventory_repo: &InventoryRepository,
    character: &CharacterRecord,
) -> (i64, i64) {
    let equipment_bonus = inventory_repo.equipment_bonus(character.id).await.unwrap_or_default();
    let skill_bonus = SkillRepository::new(state.db.pool())
        .bonus(character.id)
        .await
        .unwrap_or_default();
    let system_bonus = SystemsRepository::new(state.db.pool())
        .combat_bonus(character.id)
        .await
        .unwrap_or_default();
    character_resource_caps(character, &equipment_bonus, &skill_bonus, &system_bonus)
}

fn character_resource_caps(
    character: &CharacterRecord,
    equipment: &EquipmentBonus,
    skills: &SkillBonus,
    systems: &SystemBonus,
) -> (i64, i64) {
    let base = initial_stats(class_from_str(&character.class), character.level);
    let hp = base.max_hp + equipment.hp + skills.hp + systems.hp;
    let mp = base.max_mp + equipment.mp + skills.mp + systems.mp;
    (
        apply_pct(hp, equipment.hp_pct + skills.hp_pct + systems.hp_pct),
        apply_pct(mp, equipment.mp_pct + skills.mp_pct + systems.mp_pct),
    )
}

fn idle_recovery_rates(max_hp: i64, max_mp: i64) -> (i64, i64) {
    ((max_hp / 60).clamp(4, 120), (max_mp / 60).clamp(3, 80))
}

fn clamp_resource_values(state_row: &mut CharacterStateRecord, max_hp: i64, max_mp: i64) {
    state_row.hp = state_row.hp.clamp(0, max_hp.max(1));
    state_row.mp = state_row.mp.clamp(0, max_mp.max(0));
}

async fn apply_vip_auto_potions(
    state: &AppState,
    inventory_repo: &InventoryRepository,
    character_id: i64,
    state_row: &mut CharacterStateRecord,
    max_hp: i64,
    max_mp: i64,
    lines: &mut Vec<String>,
) -> Result<bool, ApiError> {
    let systems_repo = SystemsRepository::new(state.db.pool());
    if !systems_repo.has_active_vip(character_id).await.unwrap_or(false) {
        return Ok(false);
    }
    let settings = systems_repo
        .vip_potion_settings(character_id)
        .await
        .unwrap_or_default();
    let mut consumed = false;
    if should_auto_use(state_row.hp, max_hp, settings.hp_enabled, settings.hp_threshold_pct) {
        consumed |= consume_auto_potion(
            inventory_repo,
            character_id,
            &settings.hp_template_id,
            state_row,
            max_hp,
            max_mp,
            "生命",
            settings.hp_threshold_pct,
            lines,
        )
        .await?;
    }
    if should_auto_use(state_row.mp, max_mp, settings.mp_enabled, settings.mp_threshold_pct) {
        consumed |= consume_auto_potion(
            inventory_repo,
            character_id,
            &settings.mp_template_id,
            state_row,
            max_hp,
            max_mp,
            "魔法",
            settings.mp_threshold_pct,
            lines,
        )
        .await?;
    }
    Ok(consumed)
}

async fn consume_auto_potion(
    inventory_repo: &InventoryRepository,
    character_id: i64,
    template_id: &str,
    state_row: &mut CharacterStateRecord,
    max_hp: i64,
    max_mp: i64,
    resource_name: &str,
    threshold_pct: i32,
    lines: &mut Vec<String>,
) -> Result<bool, ApiError> {
    let Some(potion) = inventory_repo.consume_auto_potion(character_id, template_id.trim()).await? else {
        return Ok(false);
    };
    let before_hp = state_row.hp;
    let before_mp = state_row.mp;
    apply_consumed_potion(state_row, &potion, max_hp, max_mp);
    if state_row.hp == before_hp && state_row.mp == before_mp {
        return Ok(false);
    }
    lines.push(format!(
        "会员自动用药：{}低于 {}%，使用 {}，生命 {}/{}，魔法 {}/{}。",
        resource_name,
        threshold_pct,
        potion.name,
        state_row.hp,
        max_hp,
        state_row.mp,
        max_mp
    ));
    Ok(true)
}

fn apply_consumed_potion(
    state_row: &mut CharacterStateRecord,
    potion: &ConsumedPotion,
    max_hp: i64,
    max_mp: i64,
) {
    if potion.full_restore {
        state_row.hp = max_hp.max(1);
        state_row.mp = max_mp.max(0);
        return;
    }
    let hp_restore = potion.hp + max_hp.saturating_mul(potion.hp_pct) / 100;
    let mp_restore = potion.mp + max_mp.saturating_mul(potion.mp_pct) / 100;
    if hp_restore > 0 {
        state_row.hp = (state_row.hp + hp_restore).clamp(0, max_hp.max(1));
    }
    if mp_restore > 0 {
        state_row.mp = (state_row.mp + mp_restore).clamp(0, max_mp.max(0));
    }
}

fn should_auto_use(current: i64, max_value: i64, enabled: bool, threshold_pct: i32) -> bool {
    enabled && max_value > 0 && current.saturating_mul(100) <= max_value.saturating_mul(i64::from(threshold_pct.clamp(1, 99)))
}

fn character_combatant(
    character: &CharacterRecord,
    state: &CharacterStateRecord,
    equipment: &EquipmentBonus,
    skills: &SkillBonus,
    systems: &SystemBonus,
) -> Combatant {
    let level_bonus = i64::from(character.level.saturating_sub(1));
    let (max_hp, max_mp) = character_resource_caps(character, equipment, skills, systems);
    let luck = 10 + level_bonus + equipment.dex + skills.dex + systems.dex;
    let atk = 22 + level_bonus * 3 + character.power / 80 + equipment.atk + skills.atk + systems.atk;
    let def = 5 + level_bonus + equipment.def + skills.def + systems.def;
    let mag = 5 + level_bonus + equipment.mag + skills.mag + systems.mag;
    let mdef = 4 + level_bonus + equipment.mdef + skills.mdef + systems.mdef;
    Combatant {
        id: character.id,
        name: character.name.clone(),
        level: character.level,
        hp: state.hp.clamp(0, max_hp),
        max_hp,
        max_mp,
        atk: apply_pct(atk, equipment.atk_pct + skills.atk_pct + systems.atk_pct),
        def: apply_pct(def, equipment.def_pct + skills.def_pct + systems.def_pct),
        mag: apply_pct(mag, equipment.atk_pct + skills.mag_pct + systems.atk_pct),
        mdef: apply_pct(mdef, equipment.def_pct + skills.def_pct + systems.def_pct),
        dex: luck,
        crit_pct: 6 + skills.crit_pct + equipment.crit_pct + systems.crit_pct + luck / 20,
        luck,
        heavy_hit_pct: equipment.heavy_hit_pct,
        life_steal_pct: equipment.life_steal_pct + skills.life_steal_pct + systems.life_steal_pct,
        mana_steal_pct: equipment.mana_steal_pct + skills.mana_steal_pct + systems.mana_steal_pct,
        paralyze_pct: equipment.paralyze_pct,
        petrify_pct: equipment.petrify_pct,
        paralyze_resist_pct: equipment.paralyze_resist_pct + skills.control_resist_pct + systems.control_resist_pct,
        petrify_resist_pct: equipment.petrify_resist_pct + skills.control_resist_pct + systems.control_resist_pct,
        crit_damage_pct: equipment.crit_damage_pct + skills.crit_damage_pct + systems.crit_damage_pct,
        boss_damage_pct: equipment.boss_damage_pct,
        damage_deepen_pct: equipment.damage_deepen_pct + skills.damage_deepen_pct,
        normal_mob_execute_pct: equipment.normal_mob_execute_pct,
        damage_reduce_pct: equipment.damage_reduce_pct + skills.damage_reduce_pct + systems.damage_reduce_pct,
        ignore_def_pct: equipment.ignore_def_pct + skills.ignore_def_pct,
        guaranteed_hit_pct: equipment.guaranteed_hit_pct + skills.guaranteed_hit_pct,
        target_max_hp_true_damage_pct: equipment.target_max_hp_true_damage_pct,
        self_max_mp_true_damage_pct: equipment.self_max_mp_true_damage_pct,
        creation_strike_pct: equipment.creation_strike_pct,
        creation_strike_damage_pct: equipment.creation_strike_damage_pct,
        creation_strike_full_restore: equipment.creation_strike_full_restore,
        control_immune: equipment.control_immune,
    }
}

fn apply_pct(value: i64, pct: i64) -> i64 {
    value.saturating_mul(100 + pct.clamp(-90, 500)) / 100
}

fn apply_stamina_to_state(state_row: &mut CharacterStateRecord, stamina: &StaminaConsumption) {
    state_row.stamina = stamina.stamina;
    state_row.stamina_max = stamina.stamina_max;
}

fn apply_dominator_fatigue_privilege(
    mut stamina: StaminaConsumption,
    equipment: &EquipmentBonus,
) -> StaminaConsumption {
    if equipment.fatigue_immune && stamina.fatigue_kills > 0 {
        stamina.full_kills = stamina.full_kills.saturating_add(stamina.fatigue_kills);
        stamina.fatigue_kills = 0;
    }
    stamina
}

fn append_stamina_single_log(lines: &mut Vec<String>, stamina: &StaminaConsumption) {
    if stamina.full_kills > 0 {
        lines.push(format!("体力 -1，剩余 {}/{}。", stamina.stamina, stamina.stamina_max));
    } else {
        lines.push("体力为 0，进入疲劳状态：经验降至 5%，金币降至 2%。".into());
    }
}

fn append_stamina_batch_log(lines: &mut Vec<String>, stamina: &StaminaConsumption) {
    if stamina.full_kills > 0 && stamina.fatigue_kills == 0 {
        lines.push(format!(
            "体力消耗 {} 点，剩余 {}/{}。",
            stamina.full_kills, stamina.stamina, stamina.stamina_max
        ));
    } else if stamina.full_kills > 0 {
        lines.push(format!(
            "体力不足：{} 次击杀正常结算，{} 次击杀进入疲劳结算；剩余 {}/{}。",
            stamina.full_kills, stamina.fatigue_kills, stamina.stamina, stamina.stamina_max
        ));
    } else {
        lines.push(format!(
            "体力为 0，{} 次击杀全部进入疲劳结算：经验降至 5%，金币降至 2%，装备与材料掉率为 0%。",
            stamina.fatigue_kills
        ));
    }
}

fn fatigue_single_reward(exp: i64, gold: i64, fatigued: bool) -> (i64, i64) {
    if fatigued {
        (scale_reward(exp, FATIGUE_EXP_PCT), scale_reward(gold, FATIGUE_GOLD_PCT))
    } else {
        (exp.max(0), gold.max(0))
    }
}

fn fatigue_split_reward(value: i64, stamina: &StaminaConsumption, fatigue_pct: i64) -> i64 {
    let total_kills = stamina.full_kills.saturating_add(stamina.fatigue_kills).max(1);
    let normal = value.max(0).saturating_mul(stamina.full_kills.max(0)) / total_kills;
    let fatigue_base = value.max(0).saturating_sub(normal);
    normal.saturating_add(scale_reward(fatigue_base, fatigue_pct))
}

fn scale_reward(value: i64, pct: i64) -> i64 {
    value.max(0).saturating_mul(pct.clamp(0, 100)) / 100
}

fn apply_battle_end_restore(
    state_row: &mut CharacterStateRecord,
    max_hp: i64,
    max_mp: i64,
    restore_pct: i64,
) -> Option<(i64, i64)> {
    let pct = restore_pct.clamp(0, 100);
    if pct <= 0 {
        return None;
    }
    let before_hp = state_row.hp;
    let before_mp = state_row.mp;
    state_row.hp = (state_row.hp + max_hp.saturating_mul(pct) / 100).clamp(0, max_hp.max(1));
    state_row.mp = (state_row.mp + max_mp.saturating_mul(pct) / 100).clamp(0, max_mp.max(0));
    let restored_hp = state_row.hp.saturating_sub(before_hp);
    let restored_mp = state_row.mp.saturating_sub(before_mp);
    if restored_hp > 0 || restored_mp > 0 {
        Some((restored_hp, restored_mp))
    } else {
        None
    }
}

fn apply_damage_bonuses(
    report: &mut DamageReport,
    attacker: &Combatant,
    defender: &Combatant,
    target_is_boss: bool,
    target_is_regular_mob: bool,
) -> DamageBonusOutcome {
    if !report.hit || report.damage <= 0 {
        return DamageBonusOutcome::default();
    }
    let mut outcome = DamageBonusOutcome::default();
    let mut damage = report.damage;
    if attacker.damage_deepen_pct > 0 {
        damage = apply_pct(damage, attacker.damage_deepen_pct);
    }
    if target_is_boss && attacker.boss_damage_pct > 0 {
        damage = apply_pct(damage, attacker.boss_damage_pct);
    }
    if attacker.target_max_hp_true_damage_pct > 0 {
        let mut true_damage =
            defender.max_hp.saturating_mul(attacker.target_max_hp_true_damage_pct.clamp(0, 50)) / 100;
        if target_is_boss {
            true_damage = true_damage.min(attacker.atk.max(1).saturating_mul(5));
        }
        if true_damage > 0 {
            outcome.true_damage = true_damage;
            damage = damage.saturating_add(true_damage);
        }
    }
    if attacker.self_max_mp_true_damage_pct > 0 {
        let true_damage =
            attacker.max_mp.saturating_mul(attacker.self_max_mp_true_damage_pct.clamp(0, 20)) / 100;
        if true_damage > 0 {
            outcome.true_damage = outcome.true_damage.saturating_add(true_damage);
            damage = damage.saturating_add(true_damage);
        }
    }
    if attacker.creation_strike_pct > 0
        && thread_rng().gen_range(0..100) < attacker.creation_strike_pct.clamp(0, 80)
    {
        damage = damage
            .saturating_mul(attacker.creation_strike_damage_pct.clamp(100, 1000))
            / 100;
        outcome.creation_strike = true;
        outcome.full_restore = attacker.creation_strike_full_restore;
    }
    let execute = target_is_regular_mob
        && attacker.normal_mob_execute_pct > 0
        && thread_rng().gen_range(0..100) < attacker.normal_mob_execute_pct.clamp(0, 80);
    if execute {
        damage = defender.hp.max(1);
        outcome.normal_mob_execute = true;
    }
    report.damage = damage.max(1);
    report.remaining_hp = defender.hp.saturating_sub(report.damage).max(0);
    outcome
}

fn append_damage_bonus_lines(lines: &mut Vec<String>, outcome: DamageBonusOutcome) {
    if outcome.true_damage > 0 {
        lines.push(format!("真实伤害生效，追加 {} 点伤害。", outcome.true_damage));
    }
    if outcome.normal_mob_execute {
        lines.push("魔龙降临触发，真实伤害瞬间击溃普通怪。".into());
    }
    if outcome.creation_strike {
        lines.push("天罚触发，本次伤害放大并清空技能节奏。".into());
    }
}

fn apply_damage_bonus_restore(
    state_row: &mut CharacterStateRecord,
    max_hp: i64,
    max_mp: i64,
    outcome: DamageBonusOutcome,
    lines: &mut Vec<String>,
) {
    if !outcome.full_restore {
        return;
    }
    state_row.hp = max_hp.max(1);
    state_row.mp = max_mp.max(0);
    lines.push("主宰之躯回满自身生命和魔法。".into());
}

async fn reset_skill_cooldowns_on_creation_strike(
    state: &AppState,
    character_id: i64,
    outcome: DamageBonusOutcome,
    lines: &mut Vec<String>,
) -> Result<(), ApiError> {
    if !outcome.creation_strike {
        return Ok(());
    }
    SkillRepository::new(state.db.pool())
        .reset_active_cooldowns(character_id)
        .await?;
    lines.push("一念创世触发，所有主动技能冷却已清空。".into());
    Ok(())
}

fn mob_drop_kind(mob: &MobTemplate, force_boss: bool) -> MobDropKind {
    if force_boss || mob.boss {
        MobDropKind::Boss
    } else if mob.id.contains("elite") || mob.id.ends_with("_elite") || mob.name.starts_with("精英") {
        MobDropKind::Elite
    } else {
        MobDropKind::Normal
    }
}

fn default_resources(character: &CharacterRecord) -> (i64, i64) {
    let base = initial_stats(class_from_str(&character.class), character.level);
    (base.max_hp, base.max_mp)
}

fn class_from_str(value: &str) -> CharacterClass {
    match value {
        "mage" => CharacterClass::Mage,
        "taoist" => CharacterClass::Taoist,
        "assassin" => CharacterClass::Assassin,
        _ => CharacterClass::Warrior,
    }
}

fn combat_round_limit(mode: FightMode, character_level: i32, mob: &MobTemplate) -> usize {
    let level = character_level.clamp(1, 500) as usize;
    match mode {
        FightMode::WorldBoss => 3000,
        FightMode::SecretRealm(_) => {
            if mob.boss {
                1600
            } else {
                900
            }
        }
        FightMode::Tower(floor) => {
            if floor % 5 == 0 {
                700
            } else {
                420
            }
        }
        FightMode::AreaExplore | FightMode::WildAfk => {
            if mob.boss {
                (720 + level * 8).min(2400)
            } else {
                (360 + level * 4).min(1400)
            }
        }
    }
}

fn combat_log_limit(mode: FightMode) -> usize {
    match mode {
        FightMode::Tower(_) => 90,
        FightMode::WorldBoss => 140,
        _ => 120,
    }
}

fn death_penalty_enabled(mode: FightMode) -> bool {
    !matches!(mode, FightMode::SecretRealm(_) | FightMode::Tower(_))
}

fn skill_kind(skill: &ActiveSkillRecord) -> &str {
    skill.config.get("kind").and_then(serde_json::Value::as_str).unwrap_or("physical")
}

fn effective_skill_level(skill: &ActiveSkillRecord, equipment: &EquipmentBonus) -> i32 {
    skill
        .level
        .saturating_add(equipment.all_skill_bonus.max(0) as i32)
        .clamp(1, 100)
}

fn skill_level_multiplier(level: i32) -> f64 {
    let level = level.clamp(1, 100);
    let normal_growth = f64::from(level.min(99).saturating_sub(1)) * 0.05;
    if level >= 100 {
        1.0 + normal_growth + normal_growth * 2.0
    } else {
        1.0 + normal_growth
    }
}

fn effective_skill_mp_cost(skill: &ActiveSkillRecord, equipment: &EquipmentBonus) -> i64 {
    if effective_skill_level(skill, equipment) >= 100 && skill_kind(skill) != "passive" {
        1
    } else {
        skill.mp_cost.max(0)
    }
}

fn skill_power(skill: &ActiveSkillRecord, equipment: &EquipmentBonus) -> f64 {
    let base = skill.config.get("power").and_then(serde_json::Value::as_f64).unwrap_or(1.2);
    let power = base * skill_level_multiplier(effective_skill_level(skill, equipment));
    power * (1.0 + (equipment.skill_damage_pct.clamp(0, 300) as f64 / 100.0))
}

fn skill_heal(skill: &ActiveSkillRecord, equipment: &EquipmentBonus) -> i64 {
    let base = skill.config.get("heal_power").and_then(serde_json::Value::as_i64).unwrap_or(35);
    (base as f64 * skill_level_multiplier(effective_skill_level(skill, equipment))).round() as i64
}

async fn try_origin_revive(
    inventory_repo: &InventoryRepository,
    character_repo: &CharacterRepository,
    character_id: i64,
    state_row: &mut CharacterStateRecord,
    max_hp: i64,
    max_mp: i64,
    lines: &mut Vec<String>,
) -> Result<bool, ApiError> {
    if !inventory_repo.trigger_origin_revive(character_id).await? {
        return Ok(false);
    }
    *state_row = character_repo
        .save_resources(character_id, max_hp.max(1), max_mp.max(0))
        .await?;
    lines.push("涅槃复生触发【原地复活】：生命和魔法恢复至满值。".into());
    Ok(true)
}

async fn try_star_devourer_growth(
    state: &AppState,
    inventory_repo: &InventoryRepository,
    character_id: i64,
    lines: &mut Vec<String>,
) -> Result<bool, ApiError> {
    let equipped = sqlx::query_as::<_, (bool,)>(
        r#"
        select exists(
          select 1
          from inventory_items
          where character_id = $1
            and location = 'equipped'
            and template_id = 'bracelet_star_devourer'
        )
        "#,
    )
    .bind(character_id)
    .fetch_one(state.db.pool())
    .await?
    .0;
    if !equipped {
        return Ok(false);
    }
    if thread_rng().gen_range(0..2_000) != 0 {
        return Ok(false);
    }
    let (column, label) = match thread_rng().gen_range(0..6) {
        0 => ("max_hp", "最大生命"),
        1 => ("max_mp", "最大魔法"),
        2 => ("atk", "物理攻击"),
        3 => ("mag", "魔法攻击"),
        4 => ("def", "物理防御"),
        _ => ("mdef", "魔法防御"),
    };
    let sql = format!(
        "update character_stats set {column} = {column} + 1 where character_id = $1",
        column = column
    );
    sqlx::query(&sql)
        .bind(character_id)
        .execute(state.db.pool())
        .await?;
    inventory_repo.clamp_character_resources(character_id).await?;
    lines.push(format!("噬星镯汲取星辉，永久提升 {} +1。", label));
    Ok(true)
}

async fn apply_death_penalty(
    inventory_repo: &InventoryRepository,
    character_id: i64,
    lines: &mut Vec<String>,
) -> Result<bool, ApiError> {
    let drop_bag = {
        let mut rng = thread_rng();
        rng.gen_range(0..100) < 50
    };
    let drop_equipped = {
        let mut rng = thread_rng();
        rng.gen_range(0..100) < 10
    };
    let mut changed = false;
    if drop_bag {
        let items = inventory_repo.drop_random_bag_items(character_id).await?;
        if items.is_empty() {
            lines.push("死亡惩罚：背包判定触发，但背包没有可掉落物品。".into());
        } else {
            changed = true;
            lines.push(format!("死亡惩罚：背包掉落并消失 {}。", items.join("、")));
        }
    }
    if drop_equipped {
        if let Some(item_name) = inventory_repo.drop_random_equipped_item(character_id).await? {
            changed = true;
            lines.push(format!("死亡惩罚：装备 {} 掉落并消失。", item_name));
        } else {
            lines.push("死亡惩罚：装备判定触发，但身上没有可掉落装备。".into());
        }
    }
    if !drop_bag && !drop_equipped {
        lines.push("死亡惩罚：本次运气不错，没有掉落物品。".into());
    }
    Ok(changed)
}

fn wild_afk_rates(mobs: &[MobTemplate], spawns: &[String]) -> Result<(i64, i64), ApiError> {
    let candidates = regular_spawn_mobs(mobs, spawns);
    if candidates.is_empty() {
        return Err(ApiError::BadRequest("这里没有可挂机的野怪".into()));
    }
    let total_exp = candidates.iter().map(|mob| mob.exp.max(1)).sum::<i64>();
    let total_gold = candidates.iter().map(|mob| mob.gold.max(0)).sum::<i64>();
    let count = i64::try_from(candidates.len()).unwrap_or(1).max(1);
    Ok(((total_exp / count / 8).max(1), (total_gold / count / 8).max(0)))
}

fn average_spawn_level(mobs: &[MobTemplate], spawns: &[String]) -> i32 {
    let candidates = regular_spawn_mobs(mobs, spawns);
    if candidates.is_empty() {
        return 1;
    }
    let total = candidates.iter().map(|mob| mob.level.max(1)).sum::<i32>();
    total / i32::try_from(candidates.len()).unwrap_or(1).max(1)
}

fn regular_spawn_mobs<'a>(mobs: &'a [MobTemplate], spawns: &[String]) -> Vec<&'a MobTemplate> {
    let mut candidates = spawns
        .iter()
        .filter_map(|id| select_mob(mobs, id).ok())
        .filter(|mob| !mob.boss)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        candidates = spawns.iter().filter_map(|id| select_mob(mobs, id).ok()).collect();
    }
    candidates
}

fn roll_wild_afk_death(minutes: i64, character_level: i32, area_level: i32) -> bool {
    if minutes <= 0 {
        return false;
    }
    let level_gap = area_level.saturating_sub(character_level).max(0);
    let chance = (2 + level_gap * 2).clamp(2, 45);
    let rolls = minutes.min(60);
    let mut rng = thread_rng();
    (0..rolls).any(|_| rng.gen_range(0..100) < chance)
}

fn roll_area_encounter(mobs: &[MobTemplate], spawns: &[String]) -> Result<MobTemplate, ApiError> {
    let mut rng = thread_rng();
    let target_id = rng.gen_range(0..spawns.len());
    let mut mob = select_mob(mobs, &spawns[target_id])?.clone();
    if mob.boss {
        return Ok(mob);
    }
    let roll = rng.gen::<f64>();
    if roll < 0.05 {
        mob.id = format!("{}_area_boss", mob.id);
        mob.name = format!("{}首领", mob.name);
        mob.level += 3;
        mob.max_hp = (mob.max_hp * 22 / 10).max(mob.max_hp + 1);
        mob.atk = (mob.atk * 14 / 10).max(mob.atk + 1);
        mob.def = (mob.def * 13 / 10).max(mob.def + 1);
        mob.exp *= 3;
        mob.gold *= 3;
        mob.boss = true;
        mob.respawn_seconds = 0;
    } else if roll < 0.18 {
        mob.id = format!("{}_elite", mob.id);
        mob.name = format!("精英{}", mob.name);
        mob.level += 1;
        mob.max_hp = (mob.max_hp * 14 / 10).max(mob.max_hp + 1);
        mob.atk = (mob.atk * 12 / 10).max(mob.atk + 1);
        mob.def = (mob.def * 11 / 10).max(mob.def + 1);
        mob.exp = mob.exp * 3 / 2;
        mob.gold = mob.gold * 3 / 2;
    }
    Ok(mob)
}

fn roll_regular_area_encounter(mobs: &[MobTemplate], spawns: &[String]) -> Result<MobTemplate, ApiError> {
    let mut rng = thread_rng();
    let mut candidates = spawns
        .iter()
        .filter_map(|id| select_mob(mobs, id).ok())
        .filter(|mob| !mob.boss)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        candidates = spawns.iter().filter_map(|id| select_mob(mobs, id).ok()).collect();
    }
    let Some(mob) = candidates.get(rng.gen_range(0..candidates.len().max(1))) else {
        return Err(ApiError::BadRequest("这里没有可挂机的野怪".into()));
    };
    Ok((*mob).clone())
}

fn balanced_monster_stats(level: i32, boss: bool) -> (i64, i64, i64) {
    let level = level.clamp(1, 600);
    let l = f64::from(level);
    let normal_hp = if level <= 40 {
        30.0 + 18.0 * l + 8.0 * l.powf(1.35)
    } else if level <= 120 {
        1800.0 + 95.0 * f64::from(level - 40).powf(1.45)
    } else if level <= 200 {
        52_000.0 + 520.0 * f64::from(level - 120).powf(1.55)
    } else if level <= 300 {
        520_000.0 + 1250.0 * f64::from(level - 200).powf(1.55)
    } else if level <= 380 {
        2_100_000.0 + 3000.0 * f64::from(level - 300).powf(1.60)
    } else if level <= 460 {
        5_500_000.0 + 6000.0 * f64::from(level - 380).powf(1.62)
    } else {
        13_000_000.0 + 18_000.0 * f64::from(level - 460).powf(1.65)
    };
    let normal_atk = if level <= 40 {
        4.0 + 2.6 * l + 0.28 * l.powf(1.25)
    } else if level <= 120 {
        110.0 + 7.2 * f64::from(level - 40).powf(1.08)
    } else if level <= 200 {
        950.0 + 12.5 * f64::from(level - 120).powf(1.08)
    } else if level <= 300 {
        2500.0 + 22.0 * f64::from(level - 200).powf(1.08)
    } else if level <= 380 {
        6000.0 + 45.0 * f64::from(level - 300).powf(1.08)
    } else if level <= 460 {
        11_000.0 + 78.0 * f64::from(level - 380).powf(1.08)
    } else {
        18_000.0 + 135.0 * f64::from(level - 460).powf(1.08)
    };
    let normal_def = if level <= 40 {
        1.0 + 0.55 * l + 0.08 * l.powf(1.20)
    } else if level <= 120 {
        40.0 + 2.4 * f64::from(level - 40).powf(1.05)
    } else if level <= 200 {
        380.0 + 4.2 * f64::from(level - 120).powf(1.08)
    } else if level <= 300 {
        900.0 + 8.5 * f64::from(level - 200).powf(1.08)
    } else if level <= 380 {
        2100.0 + 16.0 * f64::from(level - 300).powf(1.08)
    } else if level <= 460 {
        3900.0 + 28.0 * f64::from(level - 380).powf(1.08)
    } else {
        6500.0 + 48.0 * f64::from(level - 460).powf(1.08)
    };
    let hp = normal_hp.round().max(25.0) as i64;
    let atk = normal_atk.round().max(3.0) as i64;
    let def = normal_def.round().max(0.0) as i64;
    if boss {
        (hp * 6, atk * 155 / 100, def * 125 / 100)
    } else {
        (hp, atk, def)
    }
}

fn secret_realm_mob(floor: i32) -> MobTemplate {
    let floor = floor.clamp(1, 40);
    let progress = f64::from(floor) / 40.0;
    let level = (progress * 500.0).ceil().clamp(10.0, 500.0) as i32;
    let cap = heavenly_dao_phantom_mob();
    let scale = 0.04 + 0.96 * progress.powf(1.55);
    MobTemplate {
        id: format!("secret_realm_floor_{}", floor),
        name: format!("幻境第 {} 层镇守者", floor),
        level,
        max_hp: scale_i64(cap.max_hp, scale).max(120),
        atk: scale_i64(cap.atk, scale).max(8),
        def: scale_i64(cap.def, scale).max(4),
        exp: i64::from(level) * 900,
        gold: 0,
        boss: true,
        respawn_seconds: 0,
    }
}

fn tower_mob(floor: i32) -> MobTemplate {
    let floor = floor.clamp(1, 99);
    let world_boss = world_boss_mob();
    let scale = if floor <= 69 {
        let progress = f64::from(floor) / 70.0;
        0.04 + 0.96 * progress.powf(1.55)
    } else if floor <= 89 {
        1.0 + f64::from(floor - 70) * 0.05
    } else {
        3.0 + f64::from(floor - 90)
    };
    let level = if floor <= 70 {
        (f64::from(floor) / 70.0 * 600.0).ceil().clamp(10.0, 600.0) as i32
    } else {
        600
    };
    MobTemplate {
        id: format!("tower_floor_{}", floor),
        name: if floor % 10 == 0 || floor >= 70 {
            format!("无尽塔第 {} 层镇守者", floor)
        } else {
            format!("无尽塔第 {} 层守卫", floor)
        },
        level,
        max_hp: scale_i64(world_boss.max_hp, scale).max(120),
        atk: scale_i64(world_boss.atk, scale).max(8),
        def: scale_i64(world_boss.def, scale).max(4),
        exp: i64::from(level) * if floor % 10 == 0 { 920 } else { 520 },
        gold: 0,
        boss: floor % 10 == 0 || floor >= 70,
        respawn_seconds: 0,
    }
}

fn world_boss_mob() -> MobTemplate {
    MobTemplate {
        id: "world_boss_eternal_abyss_demon".into(),
        name: "万古渊魔".into(),
        level: 600,
        max_hp: 1_240_000_000,
        atk: 152_000,
        def: 64_000,
        exp: 25_000_000,
        gold: 5_000_000,
        boss: true,
        respawn_seconds: 14_400,
    }
}

fn heavenly_dao_phantom_mob() -> MobTemplate {
    let (max_hp, atk, def) = balanced_monster_stats(500, true);
    MobTemplate {
        id: "boss_heavenly_dao_phantom".into(),
        name: "鸿蒙天道幻影".into(),
        level: 500,
        max_hp: max_hp * 170 / 100,
        atk: atk * 170 / 100,
        def: def * 170 / 100,
        exp: 8_400 * 500,
        gold: 0,
        boss: true,
        respawn_seconds: 0,
    }
}

fn scale_i64(value: i64, scale: f64) -> i64 {
    (value as f64 * scale).round().max(1.0) as i64
}

fn fight_mode_title(mode: FightMode) -> &'static str {
    match mode {
        FightMode::AreaExplore => "探索当前区域",
        FightMode::WildAfk => "野怪挂机",
        FightMode::SecretRealm(_) => "幻境挑战",
        FightMode::Tower(_) => "挑战无尽塔",
        FightMode::WorldBoss => "挑战世界首领",
    }
}

fn fight_mode_power(mode: FightMode) -> f64 {
    match mode {
        FightMode::WorldBoss => 1.08,
        FightMode::Tower(floor) if floor % 5 == 0 => 1.05,
        FightMode::WildAfk => 0.96,
        _ => 1.0,
    }
}

fn fight_mode_counter_power(mode: FightMode) -> f64 {
    match mode {
        FightMode::WorldBoss => 1.3,
        FightMode::Tower(floor) if floor % 5 == 0 => 1.16,
        FightMode::WildAfk => 1.08,
        _ => 1.0,
    }
}

fn fight_mode_reward(mode: FightMode, mob: &MobTemplate) -> (i64, i64) {
    match mode {
        FightMode::AreaExplore => (mob.exp, mob.gold),
        FightMode::WildAfk => (mob.exp, mob.gold),
        FightMode::SecretRealm(_) => ((mob.exp + 120) * 2, (mob.gold + 60) * 2),
        FightMode::Tower(floor) => {
            let bonus = if floor % 5 == 0 { 2 } else { 1 };
            (mob.exp * bonus, mob.gold * bonus)
        }
        FightMode::WorldBoss => (mob.exp, mob.gold),
    }
}

fn fight_mode_drops(mode: FightMode, luck: i64) -> Vec<(String, i64)> {
    match mode {
        FightMode::AreaExplore => vec![],
        FightMode::WildAfk => vec![],
        FightMode::SecretRealm(floor) => secret_realm_materials(floor, luck),
        FightMode::Tower(floor) => tower_challenge_drops(floor, luck),
        FightMode::WorldBoss => vec![],
    }
}

fn should_roll_regular_drops(mode: FightMode) -> bool {
    matches!(mode, FightMode::AreaExplore | FightMode::WildAfk)
}

fn secret_realm_materials(floor: i32, luck: i64) -> Vec<(String, i64)> {
    let mut rng = thread_rng();
    let floor = floor.clamp(1, 40);
    let total = match floor {
        1..=5 => 2,
        6..=10 => 3,
        11..=20 => 5,
        21..=30 => 8,
        _ => 10,
    } * 2;
    let lucky_extra = if rng.gen_range(0..1000) < (luck.max(0) / 10).min(80) as i32 { 1 } else { 0 };
    let pool = ["treasure_shard", "cultivation_pill", "pet_food", "skill_page", "stone_hongmeng"];
    let mut drops = Vec::new();
    for _ in 0..(total + lucky_extra) {
        let item = pool[rng.gen_range(0..pool.len())].to_owned();
        if let Some((_, quantity)) = drops.iter_mut().find(|(template_id, _)| template_id == &item) {
            *quantity += 1;
        } else {
            drops.push((item, 1));
        }
    }
    drops
}

fn tower_challenge_drops(floor: i32, _luck: i64) -> Vec<(String, i64)> {
    let floor = floor.clamp(1, 99);
    let tier = (10 + floor / 10).clamp(10, 17);
    vec![(tower_weighted_equipment(tier), 1)]
}

fn tower_weighted_equipment(tier: i32) -> String {
    let mut rng = thread_rng();
    let roll = rng.gen_range(0..100);
    let slot = if roll < 50 {
        random_item(&["chest", "head", "feet"])
    } else if roll < 99 {
        random_item(&["neck", "bracelet", "ring", "waist"])
    } else {
        "weapon"
    };
    format!("t{tier:02}_{slot}", tier = tier.clamp(1, 17), slot = slot)
}

fn world_boss_equipment() -> &'static str {
    let roll = thread_rng().gen_range(0..100);
    if roll < 50 {
        random_item(&["dominator_armor", "dominator_helm", "dominator_boots"])
    } else if roll < 99 {
        random_item(&["dominator_necklace", "dominator_bracelet", "dominator_ring", "dominator_belt"])
    } else {
        "dominator_blade"
    }
}

fn random_item(items: &[&'static str]) -> &'static str {
    items[thread_rng().gen_range(0..items.len())]
}

fn is_tianshui_city(position: &Position) -> bool {
    position.zone == "xiuzhen" && position.room == "tianshui_city"
}

fn is_world_boss_room(position: &Position) -> bool {
    position.zone == "feisheng" && position.room == "void_fortress"
}

fn is_chaos_shelter(position: &Position) -> bool {
    position.zone == "feisheng" && position.room == "chaos_shelter"
}

async fn ensure_challenge_state(pool: &sqlx::PgPool, character_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        insert into character_challenge_state (character_id)
        values ($1)
        on conflict (character_id) do nothing
        "#,
    )
    .bind(character_id)
    .execute(pool)
    .await?;
    Ok(())
}

async fn tower_floor(pool: &sqlx::PgPool, character_id: i64) -> Result<i32, sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    let (floor,): (i32,) = sqlx::query_as("select tower_floor from character_challenge_state where character_id = $1")
        .bind(character_id)
        .fetch_one(pool)
        .await?;
    Ok(floor.clamp(1, 200))
}

async fn advance_tower_floor(pool: &sqlx::PgPool, character_id: i64, floor: i32) -> Result<(), sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    sqlx::query(
        r#"
        update character_challenge_state
        set tower_floor = greatest(tower_floor, $2)
        where character_id = $1
        "#,
    )
    .bind(character_id)
    .bind(floor.clamp(1, 200))
    .execute(pool)
    .await?;
    Ok(())
}

async fn tower_cooldown_seconds(pool: &sqlx::PgPool, character_id: i64) -> Result<i64, sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    let (seconds,): (i64,) = sqlx::query_as(
        r#"
        select greatest(extract(epoch from (tower_available_at - now()))::bigint, 0)
        from character_challenge_state
        where character_id = $1
        "#,
    )
    .bind(character_id)
    .fetch_one(pool)
    .await?;
    Ok(seconds)
}

async fn set_tower_cooldown(pool: &sqlx::PgPool, character_id: i64, seconds: i32) -> Result<(), sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    sqlx::query(
        r#"
        update character_challenge_state
        set tower_available_at = now() + ($2::int * interval '1 second')
        where character_id = $1
        "#,
    )
    .bind(character_id)
    .bind(seconds.max(0))
    .execute(pool)
    .await?;
    Ok(())
}

async fn secret_realm_cooldown_seconds(pool: &sqlx::PgPool, character_id: i64) -> Result<i64, sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    let (seconds,): (i64,) = sqlx::query_as(
        r#"
        select greatest(extract(epoch from (secret_realm_available_at - now()))::bigint, 0)
        from character_challenge_state
        where character_id = $1
        "#,
    )
    .bind(character_id)
    .fetch_one(pool)
    .await?;
    Ok(seconds)
}

async fn set_secret_realm_cooldown(pool: &sqlx::PgPool, character_id: i64, seconds: i32) -> Result<(), sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    sqlx::query(
        r#"
        update character_challenge_state
        set secret_realm_available_at = now() + ($2::int * interval '1 second')
        where character_id = $1
        "#,
    )
    .bind(character_id)
    .bind(seconds.max(0))
    .execute(pool)
    .await?;
    Ok(())
}

async fn world_boss_cooldown_seconds(pool: &sqlx::PgPool, character_id: i64) -> Result<i64, sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    let (seconds,): (i64,) = sqlx::query_as(
        r#"
        select greatest(extract(epoch from (world_boss_available_at - now()))::bigint, 0)
        from character_challenge_state
        where character_id = $1
        "#,
    )
    .bind(character_id)
    .fetch_one(pool)
    .await?;
    Ok(seconds)
}

async fn set_world_boss_cooldown(pool: &sqlx::PgPool, character_id: i64, seconds: i32) -> Result<(), sqlx::Error> {
    ensure_challenge_state(pool, character_id).await?;
    sqlx::query(
        r#"
        update character_challenge_state
        set world_boss_available_at = now() + ($2::int * interval '1 second')
        where character_id = $1
        "#,
    )
    .bind(character_id)
    .bind(seconds.max(0))
    .execute(pool)
    .await?;
    Ok(())
}

fn inventory_action_error(err: sqlx::Error) -> ApiError {
    match err {
        sqlx::Error::RowNotFound => ApiError::BadRequest("物品不存在、不可使用或不可装备".into()),
        err => ApiError::Database(err),
    }
}

fn inventory_game_action_error(err: InventoryActionError) -> ApiError {
    match err {
        InventoryActionError::NotFound => ApiError::BadRequest("物品不存在或当前状态不可操作".into()),
        InventoryActionError::NotEnoughGold => ApiError::BadRequest("金币不足".into()),
        InventoryActionError::NotEnoughYuanbao => ApiError::BadRequest("元宝不足".into()),
        InventoryActionError::BagFull => ApiError::BadRequest("背包已满".into()),
        InventoryActionError::MaxEnhance => ApiError::BadRequest("已达到当前等级上限".into()),
        InventoryActionError::NotEnoughMaterial(message) => ApiError::BadRequest(message),
        InventoryActionError::Database(err) => ApiError::Database(err),
    }
}

fn systems_action_error(err: SystemsActionError) -> ApiError {
    match err {
        SystemsActionError::NotFound => ApiError::BadRequest("充值卡不存在、已使用或角色不存在".into()),
        SystemsActionError::NotEnoughGold => ApiError::BadRequest("金币不足".into()),
        SystemsActionError::NotEnoughMaterial => ApiError::BadRequest("培养材料不足".into()),
        SystemsActionError::MaxLevel => ApiError::BadRequest("已达到当前培养上限".into()),
        SystemsActionError::Locked(message) => ApiError::BadRequest(message),
        SystemsActionError::Database(err) => ApiError::Database(err),
    }
}

fn afk_action_error(err: sqlx::Error) -> ApiError {
    match err {
        sqlx::Error::RowNotFound => ApiError::BadRequest("挂机尚未开始".into()),
        err => ApiError::Database(err),
    }
}

fn mail_action_error(err: sqlx::Error) -> ApiError {
    match err {
        sqlx::Error::RowNotFound => ApiError::BadRequest("邮件不存在、已过期或不属于当前角色".into()),
        err => ApiError::Database(err),
    }
}

fn skill_action_error(err: sqlx::Error) -> ApiError {
    match err {
        sqlx::Error::RowNotFound => ApiError::BadRequest("技能不存在、职业不符、等级不足或缺少技能书".into()),
        err => ApiError::Database(err),
    }
}

fn skill_upgrade_error(err: SkillUpgradeError) -> ApiError {
    match err {
        SkillUpgradeError::NotFound => ApiError::BadRequest("技能尚未学习或不属于当前职业".into()),
        SkillUpgradeError::MaxLevel => ApiError::BadRequest("技能已达到 100 级满级".into()),
        SkillUpgradeError::NotEnoughProficiency { required, current } => {
            ApiError::BadRequest(format!("技能经验不足，需要 {}，当前 {}", required, current))
        }
        SkillUpgradeError::Database(err) => ApiError::Database(err),
    }
}

fn quest_error(err: QuestError) -> ApiError {
    match err {
        QuestError::NotFound => ApiError::BadRequest("任务不存在或尚未开放".into()),
        QuestError::NotComplete => ApiError::BadRequest("任务尚未完成".into()),
        QuestError::AlreadyClaimed => ApiError::BadRequest("任务奖励已经领取".into()),
        QuestError::Database(err) => ApiError::Database(err),
    }
}

fn guild_join_error(err: GuildJoinError) -> ApiError {
    match err {
        GuildJoinError::NotFound => ApiError::BadRequest("行会不存在".into()),
        GuildJoinError::Full => ApiError::Conflict("行会人数已满，最多 10 人".into()),
        GuildJoinError::AlreadyInGuild => ApiError::Conflict("角色已经加入其他行会".into()),
        GuildJoinError::Database(err) => ApiError::Database(err),
    }
}

fn guild_action_error(err: GuildActionError) -> ApiError {
    match err {
        GuildActionError::NotFound => ApiError::BadRequest("行会、角色或申请不存在".into()),
        GuildActionError::InvalidName => ApiError::BadRequest("行会名称需为 2-16 个字符".into()),
        GuildActionError::InvalidAmount => ApiError::BadRequest("捐献金币必须为 1 万的整数倍".into()),
        GuildActionError::InvalidTask => ApiError::BadRequest("行会任务不存在".into()),
        GuildActionError::AlreadyCompleted => ApiError::Conflict("今日已经完成过这个行会任务".into()),
        GuildActionError::AlreadyClaimed => ApiError::Conflict("今日奖励已经领取".into()),
        GuildActionError::TaskUnavailable => ApiError::BadRequest("行会等级或角色等级不足，暂不能完成该任务".into()),
        GuildActionError::FeatureLocked => ApiError::BadRequest("行会等级不足，暂未解锁该功能".into()),
        GuildActionError::MaxLevel => ApiError::BadRequest("已达到等级上限".into()),
        GuildActionError::GuildFull => ApiError::Conflict("行会人数已满，最多 10 人".into()),
        GuildActionError::BagFull => ApiError::BadRequest("背包已满，请先整理或拆解装备".into()),
        GuildActionError::NotEnoughGold => ApiError::BadRequest("金币不足".into()),
        GuildActionError::NotEnoughContribution => ApiError::BadRequest("行会贡献不足".into()),
        GuildActionError::NotEnoughMaterial => ApiError::BadRequest("缺少行会功勋令".into()),
        GuildActionError::AlreadyInGuild => ApiError::Conflict("角色已经加入其他行会".into()),
        GuildActionError::AlreadyPending => ApiError::Conflict("已提交过入会申请，请等待审批".into()),
        GuildActionError::PermissionDenied => ApiError::BadRequest("只有会长或长老可以审批申请".into()),
        GuildActionError::Database(err) => ApiError::Database(err),
    }
}

fn guild_view(item: GuildRecord) -> PlayerGuildView {
    PlayerGuildView {
        id: item.id,
        name: item.name,
        notice: item.notice,
        level: item.level,
        funds: item.funds,
        sabak_owner: item.sabak_owner,
        member_count: item.member_count,
        joined: item.joined,
        role: item.role,
        contribution: item.contribution,
        pending_application: item.pending_application,
        projects: item
            .projects
            .into_iter()
            .map(|project| cq_protocol::dto::PlayerGuildProjectView {
                kind: project.kind,
                name: project.name,
                description: project.description,
                progress: project.progress,
                required: project.required,
                completed: project.completed,
                completed_today: project.completed_today,
                min_level: project.min_level,
                available: project.available,
            })
            .collect(),
        totems: item
            .totems
            .into_iter()
            .map(|totem| cq_protocol::dto::PlayerGuildTotemView {
                kind: totem.kind,
                name: totem.name,
                description: totem.description,
                level: totem.level,
                next_cost: totem.next_cost,
                max_level: totem.max_level,
                unlocked: totem.unlocked,
            })
            .collect(),
        war_techs: item
            .war_techs
            .into_iter()
            .map(|tech| cq_protocol::dto::PlayerGuildWarTechView {
                kind: tech.kind,
                name: tech.name,
                description: tech.description,
                level: tech.level,
                next_cost: tech.next_cost,
                score_bonus: tech.score_bonus,
                unlocked: tech.unlocked,
            })
            .collect(),
        sabak_tax_claimed_today: item.sabak_tax_claimed_today,
    }
}

fn guild_application_view(item: GuildApplicationRecord) -> PlayerGuildApplicationView {
    PlayerGuildApplicationView {
        id: item.id,
        guild_id: item.guild_id,
        guild_name: item.guild_name,
        character_id: item.character_id,
        character_name: item.character_name,
        message: item.message,
        status: item.status,
        created_at: item.created_at,
    }
}

fn claim_message(gold: i64, yuanbao: i64, item_quantity: i64) -> String {
    let mut parts = Vec::new();
    if gold > 0 {
        parts.push(format!("金币 +{}", gold));
    }
    if yuanbao > 0 {
        parts.push(format!("元宝 +{}", yuanbao));
    }
    if item_quantity > 0 {
        parts.push(format!("物品 +{}", item_quantity));
    }
    if parts.is_empty() {
        "邮件已标记为已领取。".into()
    } else {
        format!("领取成功：{}。", parts.join("，"))
    }
}
