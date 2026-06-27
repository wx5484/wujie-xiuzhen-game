use std::sync::Arc;

use axum::{
    http::HeaderMap,
    routing::{get, post},
    Router,
};

use crate::{config::Config, error::ApiError, state::AppState};

pub mod admin;
pub mod auth;
pub mod characters;
pub mod game;
pub mod health;
pub mod trade;

pub fn router(config: &Arc<Config>) -> Router<AppState> {
    let admin_dashboard = format!("/{}/dashboard", config.admin_path);
    let admin_login = format!("/{}/login", config.admin_path);
    let admin_accounts = format!("/{}/accounts", config.admin_path);
    let admin_accounts_clear_all = format!("/{}/accounts/clear-all", config.admin_path);
    let admin_characters = format!("/{}/characters", config.admin_path);
    let admin_character_detail = format!("/{}/character-detail", config.admin_path);
    let admin_update_character = format!("/{}/character-update", config.admin_path);
    let admin_character_item = format!("/{}/character-item", config.admin_path);
    let admin_character_item_delete = format!("/{}/character-item-delete", config.admin_path);
    let admin_mail = format!("/{}/mail", config.admin_path);
    let admin_mails = format!("/{}/mails", config.admin_path);
    let admin_items = format!("/{}/items", config.admin_path);
    let admin_item_templates = format!("/{}/item-templates", config.admin_path);
    let admin_mobs = format!("/{}/mobs", config.admin_path);
    let admin_mob_templates = format!("/{}/mob-templates", config.admin_path);
    let admin_bots = format!("/{}/bots", config.admin_path);
    let admin_bots_batch = format!("/{}/bots/batch", config.admin_path);
    let admin_bots_create = format!("/{}/bots/create", config.admin_path);
    let admin_bots_delete = format!("/{}/bots/delete", config.admin_path);
    let admin_bots_tick = format!("/{}/bots/tick", config.admin_path);
    let admin_audit = format!("/{}/audit", config.admin_path);
    let admin_audit_logs = format!("/{}/audit-logs", config.admin_path);
    let admin_settings = format!("/{}/settings", config.admin_path);
    let admin_send_mail = format!("/{}/send-mail", config.admin_path);
    let admin_adjust_character = format!("/{}/adjust-character", config.admin_path);
    let admin_toggle_activity = format!("/{}/toggle-activity", config.admin_path);
    let admin_reset_challenge_cooldowns = format!("/{}/test/reset-challenge-cooldowns", config.admin_path);
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/characters", get(characters::list).post(characters::create))
        .route("/game/bootstrap", get(game::bootstrap))
        .route("/game/coffee-qr", get(admin::coffee_qr_public))
        .route("/game/overview", get(game::overview))
        .route("/game/room", get(game::room))
        .route("/game/pk-settings", post(game::update_pk_settings))
        .route("/game/pk/bot", post(game::pk_bot))
        .route("/game/move", post(game::move_character))
        .route("/game/teleport", post(game::teleport))
        .route("/game/attack", post(game::attack))
        .route("/game/explore", post(game::explore))
        .route("/game/adventure/resolve", post(game::resolve_adventure))
        .route("/game/secret-realm/explore", post(game::secret_realm_explore))
        .route("/game/tower/challenge", post(game::tower_challenge))
        .route("/game/world-boss/challenge", post(game::world_boss_challenge))
        .route("/game/afk/wild", post(game::wild_afk))
        .route("/game/cast-skill", post(game::cast_skill))
        .route("/game/inventory", get(game::inventory))
        .route("/game/equip", post(game::equip))
        .route("/game/unequip", post(game::unequip))
        .route("/game/use-item", post(game::use_item))
        .route("/game/enhance", post(game::enhance_item))
        .route("/game/recycle", post(game::recycle_item))
        .route("/game/equipment/decompose", post(game::decompose_equipment))
        .route("/game/items/decompose", post(game::decompose_misc))
        .route("/game/npc/material-exchange", post(game::npc_material_exchange))
        .route("/game/npc/battle-instinct/upgrade", post(game::npc_battle_instinct_upgrade))
        .route("/game/npc/special-skill/upgrade", post(game::npc_special_skill_upgrade))
        .route("/game/store", post(game::store_item))
        .route("/game/retrieve", post(game::retrieve_item))
        .route("/game/shop/buy", post(game::shop_buy))
        .route("/game/yuanbao-shop/buy", post(game::yuanbao_shop_buy))
        .route("/game/vip-potion-settings", post(game::update_vip_potion_settings))
        .route("/game/recharge/redeem", post(game::recharge_redeem))
        .route("/game/systems/pet/upgrade", post(game::upgrade_pet))
        .route("/game/systems/treasure/upgrade", post(game::upgrade_treasure))
        .route("/game/systems/cultivation/breakthrough", post(game::cultivation_breakthrough))
        .route("/game/systems/wanxiang/upgrade", post(game::wanxiang_upgrade))
        .route("/game/systems/wanxiang/extract", post(game::wanxiang_extract))
        .route("/game/guild/join", post(game::join_guild))
        .route("/game/guild/create", post(game::create_guild))
        .route("/game/guild/apply", post(game::apply_guild))
        .route("/game/guild/applications", get(game::guild_applications))
        .route("/game/guild/applications/review", post(game::review_guild_application))
        .route("/game/guild/donate", post(game::donate_guild))
        .route("/game/guild/task/complete", post(game::complete_guild_task))
        .route("/game/guild/benefit/claim", post(game::claim_guild_benefit))
        .route("/game/guild/totem/upgrade", post(game::upgrade_guild_totem))
        .route("/game/guild/war-tech/charge", post(game::charge_guild_war_tech))
        .route("/game/guild/sabak-tax/claim", post(game::claim_sabak_tax))
        .route("/game/guild/merit/use", post(game::use_guild_merit_token))
        .route("/game/guild/shop/buy", post(game::buy_guild_shop_item))
        .route("/game/quests", get(game::quest_list))
        .route("/game/quests/claim", post(game::quest_claim))
        .route("/game/skills", get(game::skill_list))
        .route("/game/skills/learn", post(game::learn_skill))
        .route("/game/skills/upgrade", post(game::upgrade_skill))
        .route("/game/skills/auto-toggle", post(game::toggle_skill_auto))
        .route("/game/mail", get(game::mail_list))
        .route("/game/mail/read", post(game::mail_read))
        .route("/game/mail/claim", post(game::mail_claim))
        .route("/game/mail/delete", post(game::mail_delete))
        .route("/game/trade", get(trade::list))
        .route("/game/trade/list", post(trade::create))
        .route("/game/trade/buy", post(trade::buy))
        .route("/game/trade/cancel", post(trade::cancel))
        .route("/game/afk", get(game::afk_status))
        .route("/game/afk/start", post(game::afk_start))
        .route("/game/afk/settle", post(game::afk_settle))
        .route("/game/afk/stop", post(game::afk_stop))
        .route(&admin_login, post(admin::login))
        .route(&admin_dashboard, get(admin::dashboard))
        .route(&admin_accounts, get(admin::accounts))
        .route(&admin_accounts_clear_all, post(admin::clear_all_accounts))
        .route(&admin_characters, get(admin::characters))
        .route(&admin_character_detail, get(admin::character_detail))
        .route(&admin_update_character, post(admin::update_character))
        .route(&admin_character_item, post(admin::upsert_character_item))
        .route(&admin_character_item_delete, post(admin::delete_character_item))
        .route(&admin_mail, get(admin::mail_overview))
        .route(&admin_mails, get(admin::mail_overview))
        .route(&admin_items, get(admin::item_templates))
        .route(&admin_item_templates, get(admin::item_templates))
        .route(&admin_mobs, get(admin::mob_templates))
        .route(&admin_mob_templates, get(admin::mob_templates))
        .route(&admin_bots, get(admin::bots))
        .route(&admin_bots_batch, post(admin::bots_batch))
        .route(&admin_bots_create, post(admin::bots_create))
        .route(&admin_bots_delete, post(admin::bots_delete))
        .route(&admin_bots_tick, post(admin::bots_tick))
        .route(&admin_audit, get(admin::audit_logs))
        .route(&admin_audit_logs, get(admin::audit_logs))
        .route(&admin_settings, get(admin::settings).post(admin::save_settings))
        .route(&admin_send_mail, post(admin::send_mail))
        .route(&admin_adjust_character, post(admin::adjust_character))
        .route(&admin_toggle_activity, post(admin::toggle_activity))
        .route(&admin_reset_challenge_cooldowns, post(admin::reset_challenge_cooldowns))
}

pub fn bearer_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;
    value.strip_prefix("Bearer ").ok_or(ApiError::Unauthorized)
}

pub fn require_admin(headers: &HeaderMap, config: &Config) -> Result<(), ApiError> {
    let token = headers
        .get("x-admin-token")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;
    if token == config.admin_bootstrap_password {
        Ok(())
    } else {
        Err(ApiError::Unauthorized)
    }
}
