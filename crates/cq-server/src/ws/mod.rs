use std::time::{Duration, Instant};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use cq_db::repositories::{
    account::AccountRepository,
    activity::ActivityRepository,
    adventure::AdventureRepository,
    afk::AfkRepository,
    bot::BotRepository,
    character::{state_view, CharacterRecord, CharacterRepository, CharacterStateRecord, StaminaConsumption},
    guild::GuildRepository,
    inventory::{ConsumedPotion, EquipmentBonus, InventoryRepository},
    quest::QuestRepository,
    skill::{ActiveSkillRecord, SkillBonus, SkillRepository},
    systems::{SystemBonus, SystemsRepository},
};
use cq_domain::{
    character::{initial_stats, CharacterClass},
    combat::{magical_damage, physical_damage, roll_control, Combatant, DamageReport},
    map::{death_return_name, death_return_position, start_position, Position},
    mob::MobTemplate,
};
use cq_game::{
    command::GameCommand,
    drop::{roll_level_drop_for_source, MobDropKind},
};
use cq_protocol::{
    dto::{AfkStatusView, PlayerSkillList},
    events::{CombatLogEvent, RoomStateEvent, SystemNoticeEvent},
    ws::{AuthPayload, ClientEnvelope, CommandPayload, ServerEnvelope},
};
use futures_util::StreamExt;
use rand::{thread_rng, Rng};
use serde::Serialize;

use crate::state::AppState;

const FATIGUE_EXP_PCT: i64 = 5;
const FATIGUE_GOLD_PCT: i64 = 2;

#[derive(Debug, Clone)]
struct RuntimeMob {
    hp: i64,
    respawn_at: Option<Instant>,
}

#[derive(Debug, Clone, Copy, Default)]
struct DamageBonusOutcome {
    normal_mob_execute: bool,
    true_damage: i64,
    creation_strike: bool,
    full_restore: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct ActiveSkillEffect {
    target_current_hp_pct: i64,
    boss_cap_atk_multiplier: i64,
    execute_threshold_pct: i64,
    execute_bonus_pct: i64,
    flat_bonus_damage: i64,
}

fn apply_active_skill_effects_ws(
    report: &mut DamageReport,
    attacker: &Combatant,
    defender: &Combatant,
    target_is_boss: bool,
    effect: ActiveSkillEffect,
    lines: &mut Vec<String>,
) {
    if !report.hit || report.damage <= 0 {
        return;
    }
    let mut damage = report.damage;
    if effect.target_current_hp_pct > 0 {
        let mut extra = defender.hp.saturating_mul(effect.target_current_hp_pct.clamp(0, 20)) / 100;
        if target_is_boss {
            extra = extra.min(attacker.atk.max(1).saturating_mul(effect.boss_cap_atk_multiplier.max(1)));
        }
        if extra > 0 {
            damage = damage.saturating_add(extra);
            lines.push(format!("技能追加当前生命折算伤害 {} 点。", extra));
        }
    }
    if effect.flat_bonus_damage > 0 {
        damage = damage.saturating_add(effect.flat_bonus_damage);
        lines.push(format!("技能额外真元伤害 {} 点。", effect.flat_bonus_damage));
    }
    if effect.execute_threshold_pct > 0
        && effect.execute_bonus_pct > 0
        && defender.hp.saturating_mul(100) <= defender.max_hp.saturating_mul(effect.execute_threshold_pct.clamp(1, 99))
    {
        damage = damage.saturating_mul(100 + effect.execute_bonus_pct.clamp(0, 200)) / 100;
        lines.push(format!("斩杀机制触发，技能伤害提升 {}%。", effect.execute_bonus_pct.clamp(0, 200)));
    }
    report.damage = damage.max(1);
    report.remaining_hp = defender.hp.saturating_sub(report.damage).max(0);
}

pub async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let world = state.world.clone();
    let mobs = state.mobs.as_ref().clone();
    let mut authed_character_id = None;
    let mut active_character: Option<CharacterRecord> = None;
    let mut active_state: Option<CharacterStateRecord> = None;
    let mut position = start_position();
    let mut room_mobs = Vec::new();

    while let Some(next) = socket.next().await {
        let Ok(Message::Text(text)) = next else {
            continue;
        };
        let Ok(envelope) = serde_json::from_str::<ClientEnvelope>(text.as_str()) else {
            send(&mut socket, ServerEnvelope::error(0, "bad_json", "消息不是合法 JSON")).await;
            continue;
        };
        send(&mut socket, ServerEnvelope::ack(envelope.seq, true)).await;

        match envelope.msg_type.as_str() {
            "auth" => {
                let auth = serde_json::from_value::<AuthPayload>(envelope.payload);
                match auth {
                    Ok(auth) => {
                        let character_repo = CharacterRepository::new(state.db.pool());
                        let session = AccountRepository::new(state.db.pool())
                            .find_session(&auth.token)
                            .await
                            .ok()
                            .flatten();
                        let character = match session {
                            Some(session) => character_repo
                                .find_for_account(session.account_id, auth.character_id)
                                .await
                                .ok()
                                .flatten(),
                            None => None,
                        };

                        if let Some(character) = character {
                            authed_character_id = Some(character.id);
                            let _ = InventoryRepository::new(state.db.pool())
                                .clamp_character_resources(character.id)
                                .await;
                            let mut state_row = character_repo.state(character.id).await.ok();
                            position = state_row
                                .as_ref()
                                .map(|state| Position { zone: state.zone.clone(), room: state.room.clone() })
                                .unwrap_or_else(start_position);
                            if world.current_room(&position).is_err() {
                                position = start_position();
                                let (hp, mp) = default_resources(&character);
                                state_row = character_repo
                                    .save_state_snapshot(character.id, &position, hp, mp)
                                    .await
                                    .ok();
                            } else if state_row.as_ref().map(|state| state.hp <= 0).unwrap_or(false) {
                                let (hp, mp) = default_resources(&character);
                                position = death_return_position(&position);
                                state_row = character_repo
                                    .save_state_snapshot(character.id, &position, (hp / 2).max(1), (mp / 2).max(0))
                                    .await
                                    .ok();
                            }
                            let _ = character_repo.set_online(character.id, true).await;
                            active_character = Some(character);
                            active_state = state_row;
                            room_mobs = room_runtime(&world, &position, &mobs);
                            send(
                                &mut socket,
                                ServerEnvelope::event(
                                    envelope.seq,
                                    "auth_ok",
                                    SystemNoticeEvent {
                                        level: "info".into(),
                                        message: "WebSocket 已认证".into(),
                                    },
                                ),
                            )
                            .await;
                        } else {
                            send(
                                &mut socket,
                                ServerEnvelope::error(envelope.seq, "auth_error", "登录状态无效或角色不属于当前账号"),
                            )
                            .await;
                        }
                    }
                    Err(_) => send(&mut socket, ServerEnvelope::error(envelope.seq, "bad_auth", "认证参数错误")).await,
                }
            }
            "state_request" => {
                if authed_character_id.is_none() {
                    send(&mut socket, ServerEnvelope::error(envelope.seq, "unauthorized", "请先认证")).await;
                    continue;
                }
                if let Some(character_id) = authed_character_id {
                    sync_ws_position_from_db(
                        &world,
                        &mobs,
                        character_id,
                        &mut position,
                        &mut active_state,
                        &mut room_mobs,
                        state.db.pool(),
                    )
                    .await;
                }
                push_room_state_with_hp(
                    &mut socket,
                    envelope.seq,
                    &state,
                    &world,
                    &position,
                    &mobs,
                    &mut room_mobs,
                    authed_character_id,
                )
                .await;
            }
            "cmd" => {
                if authed_character_id.is_none() {
                    send(&mut socket, ServerEnvelope::error(envelope.seq, "unauthorized", "请先认证")).await;
                    continue;
                }
                let payload = serde_json::from_value::<CommandPayload>(envelope.payload);
                let Ok(payload) = payload else {
                    send(&mut socket, ServerEnvelope::error(envelope.seq, "bad_command", "命令参数错误")).await;
                    continue;
                };
                match GameCommand::parse(&payload.command, &payload.args) {
                    Ok(GameCommand::Move { direction }) => {
                        if let Some(character_id) = authed_character_id {
                            sync_ws_position_from_db(
                                &world,
                                &mobs,
                                character_id,
                                &mut position,
                                &mut active_state,
                                &mut room_mobs,
                                state.db.pool(),
                            )
                            .await;
                        }
                        match world.move_to(&position, &direction) {
                            Ok((next_position, room)) => {
                                position = next_position;
                                room_mobs = room.spawns.iter().map(|id| runtime_mob(&mobs, id)).collect();
                                if let Some(character_id) = authed_character_id {
                                    let character_repo = CharacterRepository::new(state.db.pool());
                                    let _ = character_repo.save_position(character_id, &position).await;
                                    let systems_repo = SystemsRepository::new(state.db.pool());
                                    let mut system_messages = Vec::new();
                                    if let Some(message) = systems_repo
                                        .unlock_for_position(character_id, &position.zone, &position.room)
                                        .await
                                        .ok()
                                        .flatten()
                                    {
                                        system_messages.push(message);
                                    }
                                    if position.zone == "ancient_secret" && position.room == "stargazer_observatory" {
                                        if let Some(message) = systems_repo
                                            .record_stargazer_entry(character_id)
                                            .await
                                            .ok()
                                            .flatten()
                                        {
                                            system_messages.push(message);
                                        }
                                    }
                                    let character = match active_character.clone() {
                                        Some(character) => Some(character),
                                        None => character_repo.find(character_id).await.ok().flatten(),
                                    };
                                    if let Some(character) = character {
                                        match settle_afk_if_training_area_left_ws(&state, &character_repo, character, &position).await {
                                            Ok(Some((message, afk_status))) => {
                                                send(
                                                    &mut socket,
                                                    ServerEnvelope::event(
                                                        envelope.seq,
                                                        "system_notice",
                                                        SystemNoticeEvent { level: "info".into(), message },
                                                    ),
                                                )
                                                .await;
                                                send(
                                                    &mut socket,
                                                    ServerEnvelope::event(envelope.seq, "afk_update", afk_status),
                                                )
                                                .await;
                                                if let Ok(Some(refreshed)) = character_repo.find(character_id).await {
                                                    active_character = Some(refreshed);
                                                }
                                                if let Ok(skills) = SkillRepository::new(state.db.pool()).list_for_character(character_id).await {
                                                    send(
                                                        &mut socket,
                                                        ServerEnvelope::event(
                                                            envelope.seq,
                                                            "skills_update",
                                                            PlayerSkillList { skills },
                                                        ),
                                                    )
                                                    .await;
                                                }
                                            }
                                            Ok(None) => {}
                                            Err(err) => tracing::warn!(error = ?err, "failed to stop afk after ws move"),
                                        }
                                    }
                                    for message in system_messages {
                                        send(
                                            &mut socket,
                                            ServerEnvelope::event(
                                                envelope.seq,
                                                "system_notice",
                                                SystemNoticeEvent { level: "info".into(), message },
                                            ),
                                        )
                                        .await;
                                    }
                                    if let Ok(saved_state) = character_repo.state(character_id).await {
                                        active_state = Some(saved_state);
                                    }
                                }
                                send(
                                    &mut socket,
                                    ServerEnvelope::event(
                                        envelope.seq,
                                        "system_notice",
                                        SystemNoticeEvent {
                                            level: "info".into(),
                                            message: format!("你来到 {}。", room.name),
                                        },
                                    ),
                                )
                                .await;
                                if let Some(character) = active_character.as_ref() {
                                    match AdventureRepository::new(state.db.pool())
                                        .maybe_trigger(character, &position.zone, &position.room, "move", 30)
                                        .await
                                    {
                                        Ok(Some(offer)) => {
                                            send(
                                                &mut socket,
                                                ServerEnvelope::event(envelope.seq, "adventure_offer", offer),
                                            )
                                            .await;
                                        }
                                        Ok(None) => {}
                                        Err(err) => tracing::warn!(error = ?err, "failed to trigger adventure after ws move"),
                                    }
                                }
                                push_room_state_with_hp(
                                    &mut socket,
                                    envelope.seq,
                                    &state,
                                    &world,
                                    &position,
                                    &mobs,
                                    &mut room_mobs,
                                    authed_character_id,
                                )
                                .await;
                            }
                            Err(_) => {
                                send(&mut socket, ServerEnvelope::error(envelope.seq, "bad_exit", "这个方向走不通")).await;
                            }
                        }
                    },
                    Ok(GameCommand::Attack { target_id }) => {
                        handle_attack(
                            &mut socket,
                            envelope.seq,
                            &state,
                            &world,
                            &mobs,
                            &mut position,
                            &mut room_mobs,
                            &mut active_character,
                            &mut active_state,
                            authed_character_id,
                            target_id,
                        )
                        .await;
                    }
                    Ok(GameCommand::CastSkill { skill_id, target_id }) => {
                        handle_cast_skill(
                            &mut socket,
                            envelope.seq,
                            &state,
                            &world,
                            &mobs,
                            &mut position,
                            &mut room_mobs,
                            &mut active_character,
                            &mut active_state,
                            authed_character_id,
                            &skill_id,
                            target_id,
                        )
                        .await;
                    }
                    Ok(GameCommand::UseItem { item_id }) => {
                        let character_id = authed_character_id.unwrap_or_default();
                        let inventory_repo = InventoryRepository::new(state.db.pool());
                        let (max_hp, max_mp) = match (active_character.as_ref(), active_state.as_ref()) {
                            (Some(character), Some(_state_row)) => {
                                let equipment_bonus = inventory_repo
                                    .equipment_bonus(character_id)
                                    .await
                                    .unwrap_or_default();
                                let skill_bonus = SkillRepository::new(state.db.pool())
                                    .bonus(character_id)
                                    .await
                                    .unwrap_or_default();
                                let system_bonus = SystemsRepository::new(state.db.pool())
                                    .combat_bonus(character_id)
                                    .await
                                    .unwrap_or_default();
                                character_resource_caps(character, &equipment_bonus, &skill_bonus, &system_bonus)
                            }
                            _ => (0, 0),
                        };
                        match inventory_repo
                            .use_item(character_id, item_id, Some(max_hp), Some(max_mp))
                            .await
                        {
                            Ok(result) => {
                                send(&mut socket, ServerEnvelope::event(envelope.seq, "inventory_update", result.inventory)).await;
                                send(
                                    &mut socket,
                                    ServerEnvelope::event(
                                        envelope.seq,
                                        "system_notice",
                                        SystemNoticeEvent { level: "info".into(), message: result.message },
                                    ),
                                )
                                .await;
                                if let Ok(state_row) = CharacterRepository::new(state.db.pool()).state(character_id).await {
                                    position = Position { zone: state_row.zone.clone(), room: state_row.room.clone() };
                                    active_state = Some(state_row);
                                    room_mobs = room_runtime(&world, &position, &mobs);
                                    if let Some(current_state) = active_state.clone() {
                                        send(&mut socket, ServerEnvelope::event(envelope.seq, "character_state_update", state_view(current_state))).await;
                                    }
                                    push_room_state_with_hp(
                                        &mut socket,
                                        envelope.seq,
                                        &state,
                                        &world,
                                        &position,
                                        &mobs,
                                        &mut room_mobs,
                                        authed_character_id,
                                    )
                                    .await;
                                }
                            }
                            Err(_) => {
                                send(&mut socket, ServerEnvelope::error(envelope.seq, "use_item_failed", "物品不存在或不可使用")).await;
                            }
                        }
                    }
                    Ok(GameCommand::LearnSkill { skill_id }) => {
                        let character_id = authed_character_id.unwrap_or_default();
                        match SkillRepository::new(state.db.pool()).learn(character_id, &skill_id).await {
                            Ok(skill) => {
                                send(
                                    &mut socket,
                                    ServerEnvelope::event(
                                        envelope.seq,
                                        "system_notice",
                                        SystemNoticeEvent {
                                            level: "info".into(),
                                            message: format!("已学会技能：{}。", skill.name),
                                        },
                                    ),
                                )
                                .await;
                            }
                            Err(_) => {
                                send(&mut socket, ServerEnvelope::error(envelope.seq, "learn_skill_failed", "技能不存在、职业不符或等级不足")).await;
                            }
                        }
                    }
                    Ok(GameCommand::StateRequest) => {
                        push_room_state_with_hp(
                            &mut socket,
                            envelope.seq,
                            &state,
                            &world,
                            &position,
                            &mobs,
                            &mut room_mobs,
                            authed_character_id,
                        )
                        .await
                    }
                    Err(err) => {
                        send(&mut socket, ServerEnvelope::error(envelope.seq, "bad_command", err.to_string())).await;
                    }
                }
            }
            _ => send(&mut socket, ServerEnvelope::error(envelope.seq, "unknown_type", "未知消息类型")).await,
        }
    }

    if let Some(character_id) = authed_character_id {
        let _ = CharacterRepository::new(state.db.pool()).set_online(character_id, false).await;
    }
}

async fn handle_attack(
    socket: &mut WebSocket,
    seq: u64,
    state: &AppState,
    world: &cq_game::world::WorldService,
    mobs: &[MobTemplate],
    position: &mut Position,
    room_mobs: &mut Vec<RuntimeMob>,
    active_character: &mut Option<CharacterRecord>,
    active_state: &mut Option<CharacterStateRecord>,
    authed_character_id: Option<i64>,
    target_id: i64,
) {
    handle_player_attack(
        socket,
        seq,
        state,
        world,
        mobs,
        position,
        room_mobs,
        active_character,
        active_state,
        authed_character_id,
        target_id,
        None,
    )
    .await;
}

async fn handle_cast_skill(
    socket: &mut WebSocket,
    seq: u64,
    state: &AppState,
    world: &cq_game::world::WorldService,
    mobs: &[MobTemplate],
    position: &mut Position,
    room_mobs: &mut Vec<RuntimeMob>,
    active_character: &mut Option<CharacterRecord>,
    active_state: &mut Option<CharacterStateRecord>,
    authed_character_id: Option<i64>,
    skill_id: &str,
    target_id: i64,
) {
    let character_id = authed_character_id.unwrap_or_default();
    let skill = match SkillRepository::new(state.db.pool())
        .active_for_character(character_id, skill_id)
        .await
    {
        Ok(skill) => skill,
        Err(_) => {
            send(socket, ServerEnvelope::error(seq, "skill_not_learned", "技能尚未学习或不属于当前职业")).await;
            return;
        }
    };

    if skill_kind(&skill) == "passive" {
        send(socket, ServerEnvelope::error(seq, "passive_skill", "被动技能学习后会直接继承属性，无需释放")).await;
        return;
    }

    if skill_kind(&skill) == "heal" {
        handle_heal_skill(
            socket,
            seq,
            state,
            world,
            mobs,
            position,
            room_mobs,
            active_character,
            active_state,
            character_id,
            skill,
        )
        .await;
        return;
    }

    handle_player_attack(
        socket,
        seq,
        state,
        world,
        mobs,
        position,
        room_mobs,
        active_character,
        active_state,
        authed_character_id,
        target_id,
        Some(skill),
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn handle_player_attack(
    socket: &mut WebSocket,
    seq: u64,
    state: &AppState,
    world: &cq_game::world::WorldService,
    mobs: &[MobTemplate],
    position: &mut Position,
    room_mobs: &mut Vec<RuntimeMob>,
    active_character: &mut Option<CharacterRecord>,
    active_state: &mut Option<CharacterStateRecord>,
    authed_character_id: Option<i64>,
    target_id: i64,
    skill: Option<ActiveSkillRecord>,
) {
    let Ok(room) = world.current_room(position).cloned() else {
        send(socket, ServerEnvelope::error(seq, "room_not_found", "房间不存在")).await;
        return;
    };
    if room.spawns.is_empty() {
        send(socket, ServerEnvelope::error(seq, "no_target", "这里没有怪物")).await;
        return;
    }
    refresh_room_mobs(&room.spawns, room_mobs, mobs);

    let Ok(target_index) = usize::try_from(target_id) else {
        send(socket, ServerEnvelope::error(seq, "bad_target", "目标不存在")).await;
        return;
    };
    if target_index >= room.spawns.len() {
        send(socket, ServerEnvelope::error(seq, "bad_target", "目标不存在")).await;
        return;
    }

    let mob = select_mob_by_id(mobs, &room.spawns[target_index]);
    if room_mobs.get(target_index).and_then(|target| target.respawn_at).is_some() {
        send(socket, ServerEnvelope::error(seq, "target_respawning", "目标尚未刷新")).await;
        return;
    }

    let character_id = authed_character_id.unwrap_or_default();
    let character = match active_character.clone() {
        Some(character) => character,
        None => match CharacterRepository::new(state.db.pool()).find(character_id).await {
            Ok(Some(character)) => {
                *active_character = Some(character.clone());
                character
            }
            _ => {
                send(socket, ServerEnvelope::error(seq, "character_not_found", "角色不存在")).await;
                return;
            }
        },
    };
    let character_repo = CharacterRepository::new(state.db.pool());
    let mut state_row = match active_state.clone() {
        Some(state_row) => state_row,
        None => match character_repo.state(character_id).await {
            Ok(state_row) => state_row,
            Err(_) => {
                send(socket, ServerEnvelope::error(seq, "state_not_found", "角色状态不存在")).await;
                return;
            }
        },
    };

    let inventory_repo = InventoryRepository::new(state.db.pool());
    let mut inventory_changed = false;
    let equipment_bonus = inventory_repo
        .equipment_bonus(character_id)
        .await
        .unwrap_or_default();
    let skill_bonus = SkillRepository::new(state.db.pool())
        .bonus(character_id)
        .await
        .unwrap_or_default();
    let system_bonus = SystemsRepository::new(state.db.pool())
        .combat_bonus(character_id)
        .await
        .unwrap_or_default();
    let (max_hp, max_mp) = character_resource_caps(&character, &equipment_bonus, &skill_bonus, &system_bonus);
    clamp_resource_values(&mut state_row, max_hp, max_mp);
    let mut lines = Vec::new();
    let mut power = 1.0;
    let mut magical = false;
    let mut active_skill_effect = ActiveSkillEffect::default();
    if let Some(skill) = skill.as_ref() {
        let mp_cost = effective_skill_mp_cost(skill, &equipment_bonus);
        if state_row.mp < mp_cost {
            send(socket, ServerEnvelope::error(seq, "mp_not_enough", "魔法值不足")).await;
            return;
        }
        if SkillRepository::new(state.db.pool())
            .mark_used(character_id, &skill.id, skill.cooldown_ms)
            .await
            .is_err()
        {
            send(socket, ServerEnvelope::error(seq, "skill_cooldown", "技能冷却中")).await;
            return;
        }
        let current_mp = state_row.mp.max(0);
        let mp_damage_pct = skill.config.get("mp_to_damage_pct").and_then(serde_json::Value::as_i64).unwrap_or_default();
        let current_mp_cost_pct = skill.config.get("current_mp_cost_pct").and_then(serde_json::Value::as_i64).unwrap_or_default();
        active_skill_effect = ActiveSkillEffect {
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
        };
        state_row.mp = (state_row.mp - mp_cost).max(0);
        power = skill_power(skill, &equipment_bonus);
        magical = skill_kind(skill) == "magical";
        lines.push(format!("{} 释放 {}，消耗 {} 点魔法。", character.name, skill.name, mp_cost));
    }
    if let Ok(true) = apply_vip_auto_potions(
        state,
        &inventory_repo,
        character_id,
        &mut state_row,
        max_hp,
        max_mp,
        &mut lines,
    )
    .await
    {
        inventory_changed = true;
    }
    let mut attacker = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
    if attacker.hp <= 0 {
        if trigger_origin_revive_ws(
            state,
            &character_repo,
            character_id,
            &mut state_row,
            max_hp,
            max_mp,
            active_state,
            &mut lines,
        )
        .await
        {
            attacker = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
        } else {
            revive_character(socket, seq, state, world, mobs, position, room_mobs, &character, active_state).await;
            return;
        }
    }

    let current_hp = room_mobs
        .get(target_index)
        .map(|target| target.hp)
        .unwrap_or(mob.max_hp)
        .max(1);
    let defender = mob_combatant(mob, current_hp);
    let mut report = {
        let mut rng = thread_rng();
        if magical {
            magical_damage(&mut rng, &attacker, &defender, power)
        } else {
            physical_damage(&mut rng, &attacker, &defender, power)
        }
    };
    apply_active_skill_effects_ws(
        &mut report,
        &attacker,
        &defender,
        mob.boss,
        active_skill_effect,
        &mut lines,
    );
    let damage_bonus = apply_damage_bonuses(
        &mut report,
        &attacker,
        &defender,
        mob.boss,
        matches!(mob_drop_kind(mob, false), MobDropKind::Normal),
    );
    append_damage_bonus_lines(&mut lines, damage_bonus);
    apply_damage_bonus_restore(&mut state_row, max_hp, max_mp, damage_bonus, &mut lines);
    reset_skill_cooldowns_on_creation_strike(state, character_id, damage_bonus, &mut lines).await;
    let target_dead = report.remaining_hp == 0;
    if let Some(target) = room_mobs.get_mut(target_index) {
        target.hp = report.remaining_hp;
    }
    lines.push(if report.hit {
        let crit = if report.crit { "暴击，" } else { "" };
        let heavy = if report.heavy { "重击，" } else { "" };
        format!("{} 攻击 {}，{}{}造成 {} 点伤害。", attacker.name, mob.name, crit, heavy, report.damage)
    } else {
        format!("{} 的攻击被 {} 闪开。", attacker.name, mob.name)
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

    if target_dead {
        if let Some(target) = room_mobs.get_mut(target_index) {
            target.hp = 0;
            target.respawn_at = Some(Instant::now() + Duration::from_secs(respawn_seconds(mob)));
        }
        let stamina = match character_repo.consume_stamina_for_kill(attacker.id).await {
            Ok(stamina) => stamina,
            Err(err) => {
                tracing::warn!(error = ?err, "failed to consume stamina");
                StaminaConsumption {
                    before_stamina: state_row.stamina.max(1),
                    stamina: state_row.stamina,
                    stamina_max: state_row.stamina_max,
                    full_kills: 1,
                    fatigue_kills: 0,
                }
            }
        };
        let stamina = apply_dominator_fatigue_privilege(stamina, &equipment_bonus);
        apply_stamina_to_state(&mut state_row, &stamina);
        append_stamina_single_log(&mut lines, &stamina);
        let fatigued = stamina.fatigue_kills > 0;
        let (exp, gold) = fatigue_single_reward(mob.exp, mob.gold, fatigued);
        lines.push(format!("{} 被击败，获得 {} 经验与 {} 金币。", mob.name, exp, gold));
        match character_repo.grant_reward(attacker.id, exp, gold).await {
            Ok(updated) => {
                let leveled = updated.level > active_character.as_ref().map(|c| c.level).unwrap_or(updated.level);
                if leveled {
                    let (hp, mp) = default_resources(&updated);
                    state_row.hp = hp;
                    state_row.mp = mp;
                    lines.push(format!("境界突破！{} 升到 Lv.{}。", updated.name, updated.level));
                }
                match inventory_repo.refresh_character_power(updated.id).await {
                    Ok(refreshed) => *active_character = Some(refreshed),
                    Err(err) => {
                        tracing::warn!(error = ?err, "failed to refresh character power");
                        *active_character = Some(updated);
                    }
                }
            }
            Err(err) => tracing::warn!(error = ?err, "failed to grant battle reward"),
        }
        if let Some((hp, mp)) =
            apply_battle_end_restore(&mut state_row, max_hp, max_mp, equipment_bonus.battle_end_restore_pct)
        {
            lines.push(format!("沃玛套装续航生效，战斗结束恢复 {} 生命和 {} 魔法。", hp, mp));
        }

        if fatigued {
            lines.push("疲劳状态：装备与材料掉率为 0%。".into());
        } else {
            let inventory_repo = InventoryRepository::new(state.db.pool());
            let source_id = drop_source_id(position, mob);
            for drop in roll_level_drop_for_source(
                mob.level,
                mob_drop_kind(mob, false),
                attacker.luck,
                Some(source_id.as_str()),
                0,
            ) {
                match inventory_repo.grant_item(attacker.id, &drop.template_id, drop.quantity).await {
                    Ok(Some(item)) => {
                        inventory_changed = true;
                        lines.push(format!("掉落：{} x{} 已放入背包。", item.name, drop.quantity));
                    }
                    Ok(None) => {}
                    Err(err) => tracing::warn!(error = ?err, "failed to grant drop"),
                }
            }
        }
        match ActivityRepository::new(state.db.pool())
            .add_points(attacker.id, "daily_hunt", 1)
            .await
        {
            Ok(points) => lines.push(format!("每日猎魔进度 +1，当前 {}。", points)),
            Err(err) => tracing::warn!(error = ?err, "failed to add activity points"),
        }
        if let Err(err) = QuestRepository::new(state.db.pool())
            .add_progress(attacker.id, "kill_any", 1)
            .await
        {
            tracing::warn!(error = ?err, "failed to add quest progress");
        }
        match SystemsRepository::new(state.db.pool())
            .unlock_for_mob(attacker.id, &mob.id)
            .await
        {
            Ok(Some(message)) => lines.push(message),
            Ok(None) => {}
            Err(err) => tracing::warn!(error = ?err, "failed to unlock system"),
        }
        if let Some(character) = active_character.as_ref() {
            match AdventureRepository::new(state.db.pool())
                .maybe_trigger(character, &position.zone, &position.room, "combat", 40)
                .await
            {
                Ok(Some(offer)) => {
                    send(socket, ServerEnvelope::event(seq, "adventure_offer", offer)).await;
                }
                Ok(None) => {}
                Err(err) => tracing::warn!(error = ?err, "failed to trigger adventure after ws combat"),
            }
        }
    } else {
        lines.push(format!("{} 还剩 {} 点生命。", mob.name, report.remaining_hp));
        let control = {
            let mut rng = thread_rng();
            report.hit.then(|| roll_control(&mut rng, &attacker, &defender)).flatten()
        };
        if let Some(effect) = control {
            lines.push(format!("{}生效，{} 本回合无法反击。", effect, mob.name));
        } else {
            let mob_attacker = mob_combatant(mob, report.remaining_hp.max(1));
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
        }
    }

    if state_row.hp <= 0 {
        if !trigger_origin_revive_ws(
            state,
            &character_repo,
            character_id,
            &mut state_row,
            max_hp,
            max_mp,
            active_state,
            &mut lines,
        )
        .await
        {
            let return_name = death_return_name(position);
            *position = death_return_position(position);
            let (hp, mp) = default_resources(active_character.as_ref().unwrap_or(&character));
            match character_repo
                .save_state_snapshot(character_id, position, (hp / 2).max(1), (mp / 2).max(0))
                .await
            {
                Ok(saved) => {
                    state_row = saved;
                    *active_state = Some(state_row.clone());
                }
                Err(err) => tracing::warn!(error = ?err, "failed to save death state"),
            }
            *room_mobs = room_runtime(world, position, mobs);
            lines.push(format!("你被击倒，回到{}休整。", return_name));
            let drop_bag = {
                let mut rng = thread_rng();
                rng.gen_range(0..100) < 50
            };
            let drop_equipped = {
                let mut rng = thread_rng();
                rng.gen_range(0..100) < 10
            };
            let inventory_repo = InventoryRepository::new(state.db.pool());
            if drop_bag {
                match inventory_repo.drop_random_bag_items(character_id).await {
                    Ok(items) if !items.is_empty() => {
                        inventory_changed = true;
                        lines.push(format!("死亡惩罚：背包掉落并消失 {}。", items.join("、")));
                    }
                    Ok(_) => lines.push("死亡惩罚：背包判定触发，但背包没有可掉落物品。".into()),
                    Err(err) => tracing::warn!(error = ?err, "failed to apply bag death penalty"),
                }
            }
            if drop_equipped {
                match inventory_repo.drop_random_equipped_item(character_id).await {
                    Ok(Some(item_name)) => {
                        inventory_changed = true;
                        lines.push(format!("死亡惩罚：装备 {} 掉落并消失。", item_name));
                    }
                    Ok(None) => lines.push("死亡惩罚：装备判定触发，但身上没有可掉落装备。".into()),
                    Err(err) => tracing::warn!(error = ?err, "failed to apply equipped death penalty"),
                }
            }
            if !drop_bag && !drop_equipped {
                lines.push("死亡惩罚：本次运气不错，没有掉落物品。".into());
            }
        }
    } else {
        if let Some(current_character) = active_character.as_ref() {
            let equipment_bonus = InventoryRepository::new(state.db.pool())
                .equipment_bonus(character_id)
                .await
                .unwrap_or_default();
            let skill_bonus = SkillRepository::new(state.db.pool())
                .bonus(character_id)
                .await
                .unwrap_or_default();
            let system_bonus = SystemsRepository::new(state.db.pool())
                .combat_bonus(character_id)
                .await
                .unwrap_or_default();
            let (max_hp, max_mp) =
                character_resource_caps(current_character, &equipment_bonus, &skill_bonus, &system_bonus);
            clamp_resource_values(&mut state_row, max_hp, max_mp);
        }
        match character_repo.save_resources(character_id, state_row.hp, state_row.mp).await {
            Ok(saved) => {
                state_row = saved;
                *active_state = Some(state_row.clone());
            }
            Err(err) => tracing::warn!(error = ?err, "failed to save combat resources"),
        }
    }

    send(socket, ServerEnvelope::event(seq, "combat_log", CombatLogEvent { lines })).await;
    if let Some(character) = active_character.as_ref() {
        send(socket, ServerEnvelope::event(seq, "character_update", character)).await;
        if inventory_changed {
            if let Ok(view) = InventoryRepository::new(state.db.pool()).view(character.id, character.level).await {
                send(socket, ServerEnvelope::event(seq, "inventory_update", view)).await;
            }
        }
    }
    if let Ok(skills) = SkillRepository::new(state.db.pool()).list_for_character(character_id).await {
        send(socket, ServerEnvelope::event(seq, "skills_update", PlayerSkillList { skills })).await;
    }
    send(socket, ServerEnvelope::event(seq, "character_state_update", state_view(state_row))).await;
    push_room_state_with_hp(socket, seq, state, world, position, mobs, room_mobs, Some(character_id)).await;
}

#[allow(clippy::too_many_arguments)]
async fn handle_heal_skill(
    socket: &mut WebSocket,
    seq: u64,
    state: &AppState,
    world: &cq_game::world::WorldService,
    mobs: &[MobTemplate],
    position: &mut Position,
    room_mobs: &mut Vec<RuntimeMob>,
    active_character: &mut Option<CharacterRecord>,
    active_state: &mut Option<CharacterStateRecord>,
    character_id: i64,
    skill: ActiveSkillRecord,
) {
    let Some(character) = active_character.clone() else {
        send(socket, ServerEnvelope::error(seq, "character_not_found", "角色不存在")).await;
        return;
    };
    let character_repo = CharacterRepository::new(state.db.pool());
    let mut state_row = match active_state.clone() {
        Some(state_row) => state_row,
        None => match character_repo.state(character_id).await {
            Ok(state_row) => state_row,
            Err(_) => {
                send(socket, ServerEnvelope::error(seq, "state_not_found", "角色状态不存在")).await;
                return;
            }
        },
    };
    let equipment_bonus = InventoryRepository::new(state.db.pool())
        .equipment_bonus(character_id)
        .await
        .unwrap_or_default();
    let skill_bonus = SkillRepository::new(state.db.pool())
        .bonus(character_id)
        .await
        .unwrap_or_default();
    let system_bonus = SystemsRepository::new(state.db.pool())
        .combat_bonus(character_id)
        .await
        .unwrap_or_default();
    let (max_hp, max_mp) = character_resource_caps(&character, &equipment_bonus, &skill_bonus, &system_bonus);
    clamp_resource_values(&mut state_row, max_hp, max_mp);
    let mp_cost = effective_skill_mp_cost(&skill, &equipment_bonus);
    if state_row.mp < mp_cost {
        send(socket, ServerEnvelope::error(seq, "mp_not_enough", "魔法值不足")).await;
        return;
    }
    if SkillRepository::new(state.db.pool())
        .mark_used(character_id, &skill.id, skill.cooldown_ms)
        .await
        .is_err()
    {
        send(socket, ServerEnvelope::error(seq, "skill_cooldown", "技能冷却中")).await;
        return;
    }
    let combatant = character_combatant(&character, &state_row, &equipment_bonus, &skill_bonus, &system_bonus);
    let heal = skill_heal(&skill, &equipment_bonus).saturating_add(combatant.mag / 2);
    state_row.mp = (state_row.mp - mp_cost).max(0);
    state_row.hp = (state_row.hp + heal).min(combatant.max_hp).max(1);
    match character_repo.save_resources(character_id, state_row.hp, state_row.mp).await {
        Ok(saved) => {
            state_row = saved;
            *active_state = Some(state_row.clone());
        }
        Err(err) => tracing::warn!(error = ?err, "failed to save heal resources"),
    }

    send(
        socket,
        ServerEnvelope::event(
            seq,
            "combat_log",
            CombatLogEvent {
                lines: vec![format!(
                    "{} 施展 {}，消耗 {} 点魔法，恢复 {} 点生命。",
                    character.name,
                    skill.name,
                    mp_cost,
                    heal
                )],
            },
        ),
    )
    .await;
    if let Ok(skills) = SkillRepository::new(state.db.pool()).list_for_character(character_id).await {
        send(socket, ServerEnvelope::event(seq, "skills_update", PlayerSkillList { skills })).await;
    }
    send(socket, ServerEnvelope::event(seq, "character_state_update", state_view(state_row))).await;
    push_room_state_with_hp(socket, seq, state, world, position, mobs, room_mobs, Some(character_id)).await;
}

async fn sync_ws_position_from_db(
    world: &cq_game::world::WorldService,
    mobs: &[MobTemplate],
    character_id: i64,
    position: &mut Position,
    active_state: &mut Option<CharacterStateRecord>,
    room_mobs: &mut Vec<RuntimeMob>,
    pool: &sqlx::PgPool,
) {
    let character_repo = CharacterRepository::new(pool);
    let Ok(state_row) = character_repo.state(character_id).await else {
        return;
    };
    let db_position = Position { zone: state_row.zone.clone(), room: state_row.room.clone() };
    if world.current_room(&db_position).is_err() {
        return;
    }
    if *position != db_position {
        *position = db_position;
        *room_mobs = room_runtime(world, position, mobs);
    }
    *active_state = Some(state_row);
}

async fn settle_afk_if_training_area_left_ws(
    state: &AppState,
    character_repo: &CharacterRepository,
    character: CharacterRecord,
    position: &Position,
) -> Result<Option<(String, AfkStatusView)>, sqlx::Error> {
    let character_id = character.id;
    let afk_repo = AfkRepository::new(state.db.pool());
    let status = afk_repo.status(character_id).await?;
    if !status.active || afk_status_allowed_at_ws(&status.mode, status.zone.as_deref(), status.room.as_deref(), position) {
        return Ok(None);
    }

    let mut result = afk_repo.settle_with_cap(character_id, 10_080).await?;
    match result.status.mode.as_str() {
        "skill_study" => {
            if result.minutes > 0 {
                if let Some(skill_id) = result.status.training_skill_id.clone() {
                    SkillRepository::new(state.db.pool())
                        .add_study_proficiency(character_id, &skill_id, result.minutes * 5)
                        .await?;
                }
            }
        }
        "practice" => {
            if result.exp > 0 || result.gold > 0 {
                character_repo.grant_reward(character_id, result.exp, result.gold).await?;
                let _ = InventoryRepository::new(state.db.pool())
                    .refresh_character_power(character_id)
                    .await?;
            }
        }
        _ => {}
    }
    if result.minutes > 0 {
        QuestRepository::new(state.db.pool())
            .add_progress(character_id, "afk_settle", 1)
            .await?;
    }
    if result.status.active {
        result.status = afk_repo.stop(character_id).await?;
    }
    if result.exp == 0 && result.gold == 0 && result.minutes == 0 {
        result.message = "打坐修炼已停止，时间不足 5 秒，暂未产生收益。".into();
    }
    Ok(Some((format!("离开修炼区域，{}", result.message), result.status)))
}

fn afk_status_allowed_at_ws(mode: &str, zone: Option<&str>, room: Option<&str>, position: &Position) -> bool {
    match mode {
        "skill_study" => position.zone == "feisheng" && position.room == "void_realm",
        "practice" => position.zone == "xiuzhen" && position.room == "purgatory",
        "wild" => zone == Some(position.zone.as_str()) && room == Some(position.room.as_str()),
        _ => true,
    }
}

fn drop_source_id(position: &Position, mob: &MobTemplate) -> String {
    format!("{}:{}:{}", position.zone, position.room, mob.id)
}

async fn push_room_state_with_hp(
    socket: &mut WebSocket,
    seq: u64,
    state: &AppState,
    world: &cq_game::world::WorldService,
    position: &Position,
    mobs: &[MobTemplate],
    room_mobs: &mut Vec<RuntimeMob>,
    current_character_id: Option<i64>,
) {
    match world.current_room(position) {
        Ok(room) => {
            let mut room = room.clone();
            apply_sabak_room_description(state, position, &mut room).await;
            refresh_room_mobs(&room.spawns, room_mobs, mobs);
            let mob_lines = room
                .spawns
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    let mob = select_mob_by_id(mobs, id);
                    let Some(runtime) = room_mobs.get(index) else {
                        return format!("{} Lv.{} HP {}/{}", mob.name, mob.level, mob.max_hp, mob.max_hp);
                    };
                    match runtime.respawn_at {
                        Some(at) => {
                            let seconds = at.saturating_duration_since(Instant::now()).as_secs().max(1);
                            format!("{} Lv.{} 刷新中 {}s", mob.name, mob.level, seconds)
                        }
                        None => format!("{} Lv.{} HP {}/{}", mob.name, mob.level, runtime.hp, mob.max_hp),
                    }
                })
                .collect();
            let players = BotRepository::new(state.db.pool())
                .names_at(&position.zone, &position.room, current_character_id)
                .await
                .unwrap_or_default();
            send(
                socket,
                ServerEnvelope::event(
                    seq,
                    "room_state",
                    RoomStateEvent {
                        room,
                        players,
                        mobs: mob_lines,
                    },
                ),
            )
            .await;
        }
        Err(_) => send(socket, ServerEnvelope::error(seq, "room_not_found", "房间不存在")).await,
    }
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

fn select_mob_by_id<'a>(mobs: &'a [MobTemplate], id: &str) -> &'a MobTemplate {
    mobs.iter().find(|mob| mob.id == id).unwrap_or_else(|| &mobs[0])
}

fn room_runtime(
    world: &cq_game::world::WorldService,
    position: &Position,
    mobs: &[MobTemplate],
) -> Vec<RuntimeMob> {
    world
        .current_room(position)
        .map(|room| room.spawns.iter().map(|id| runtime_mob(mobs, id)).collect())
        .unwrap_or_default()
}

fn runtime_mob(mobs: &[MobTemplate], id: &str) -> RuntimeMob {
    RuntimeMob { hp: select_mob_by_id(mobs, id).max_hp, respawn_at: None }
}

fn refresh_room_mobs(spawns: &[String], room_mobs: &mut Vec<RuntimeMob>, mobs: &[MobTemplate]) {
    if room_mobs.len() != spawns.len() {
        *room_mobs = spawns.iter().map(|id| runtime_mob(mobs, id)).collect();
        return;
    }

    let now = Instant::now();
    for (index, id) in spawns.iter().enumerate() {
        let Some(runtime) = room_mobs.get_mut(index) else {
            continue;
        };
        if matches!(runtime.respawn_at, Some(at) if at <= now) {
            runtime.hp = select_mob_by_id(mobs, id).max_hp;
            runtime.respawn_at = None;
        }
    }
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

fn fatigue_single_reward(exp: i64, gold: i64, fatigued: bool) -> (i64, i64) {
    if fatigued {
        (scale_reward(exp, FATIGUE_EXP_PCT), scale_reward(gold, FATIGUE_GOLD_PCT))
    } else {
        (exp.max(0), gold.max(0))
    }
}

fn scale_reward(value: i64, pct: i64) -> i64 {
    value.max(0).saturating_mul(pct.clamp(0, 100)) / 100
}

fn clamp_resource_values(state_row: &mut CharacterStateRecord, max_hp: i64, max_mp: i64) {
    state_row.hp = state_row.hp.clamp(0, max_hp.max(1));
    state_row.mp = state_row.mp.clamp(0, max_mp.max(0));
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
) {
    if !outcome.creation_strike {
        return;
    }
    match SkillRepository::new(state.db.pool())
        .reset_active_cooldowns(character_id)
        .await
    {
        Ok(_) => lines.push("一念创世触发，所有主动技能冷却已清空。".into()),
        Err(_) => lines.push("一念创世触发，但技能冷却同步暂未完成。".into()),
    }
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

async fn apply_vip_auto_potions(
    state: &AppState,
    inventory_repo: &InventoryRepository,
    character_id: i64,
    state_row: &mut CharacterStateRecord,
    max_hp: i64,
    max_mp: i64,
    lines: &mut Vec<String>,
) -> Result<bool, sqlx::Error> {
    let systems_repo = SystemsRepository::new(state.db.pool());
    if !systems_repo.has_active_vip(character_id).await.unwrap_or(false) {
        return Ok(false);
    }
    let settings = systems_repo.vip_potion_settings(character_id).await.unwrap_or_default();
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
) -> Result<bool, sqlx::Error> {
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
        resource_name, threshold_pct, potion.name, state_row.hp, max_hp, state_row.mp, max_mp
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

fn should_auto_use(current: i64, max: i64, enabled: bool, threshold_pct: i32) -> bool {
    enabled && max > 0 && current.saturating_mul(100) <= max.saturating_mul(i64::from(threshold_pct.clamp(1, 99)))
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

fn respawn_seconds(mob: &MobTemplate) -> u64 {
    let fallback = if mob.boss { 180 } else { 20 };
    u64::try_from(mob.respawn_seconds.max(fallback)).unwrap_or(fallback as u64)
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

async fn trigger_origin_revive_ws(
    state: &AppState,
    character_repo: &CharacterRepository,
    character_id: i64,
    state_row: &mut CharacterStateRecord,
    max_hp: i64,
    max_mp: i64,
    active_state: &mut Option<CharacterStateRecord>,
    lines: &mut Vec<String>,
) -> bool {
    let inventory_repo = InventoryRepository::new(state.db.pool());
    match inventory_repo.trigger_origin_revive(character_id).await {
        Ok(true) => match character_repo.save_resources(character_id, max_hp.max(1), max_mp.max(0)).await {
            Ok(saved) => {
                *state_row = saved;
                *active_state = Some(state_row.clone());
                lines.push("涅槃复生触发【原地复活】：生命和魔法恢复至满值。".into());
                true
            }
            Err(err) => {
                tracing::warn!(error = ?err, "failed to save origin revive state");
                false
            }
        },
        Ok(false) => false,
        Err(err) => {
            tracing::warn!(error = ?err, "failed to trigger origin revive");
            false
        }
    }
}

async fn revive_character(
    socket: &mut WebSocket,
    seq: u64,
    state: &AppState,
    world: &cq_game::world::WorldService,
    mobs: &[MobTemplate],
    position: &mut Position,
    room_mobs: &mut Vec<RuntimeMob>,
    character: &CharacterRecord,
    active_state: &mut Option<CharacterStateRecord>,
) {
    let return_name = death_return_name(position);
    *position = death_return_position(position);
    let (hp, mp) = default_resources(character);
    let state_row = CharacterRepository::new(state.db.pool())
        .save_state_snapshot(character.id, position, (hp / 2).max(1), (mp / 2).max(0))
        .await
        .ok();
    if let Some(state_row) = state_row {
        *active_state = Some(state_row.clone());
        send(socket, ServerEnvelope::event(seq, "character_state_update", state_view(state_row))).await;
    }
    *room_mobs = room_runtime(world, position, mobs);
    send(
        socket,
        ServerEnvelope::event(
            seq,
            "combat_log",
            CombatLogEvent { lines: vec![format!("你已经倒下，回到{}休整。", return_name)] },
        ),
    )
    .await;
    push_room_state_with_hp(socket, seq, state, world, position, mobs, room_mobs, Some(character.id)).await;
}

async fn send(socket: &mut WebSocket, envelope: ServerEnvelope) {
    send_json(socket, &envelope).await;
}

async fn send_json<T: Serialize>(socket: &mut WebSocket, value: &T) {
    match serde_json::to_string(value) {
        Ok(text) => {
            if socket.send(Message::Text(text.into())).await.is_err() {
                tracing::debug!("websocket closed while sending");
            }
        }
        Err(err) => tracing::error!(error = ?err, "failed to encode websocket event"),
    }
}
