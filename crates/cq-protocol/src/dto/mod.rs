use cq_domain::{
    character::{Character, CharacterState, CharacterStats},
    guild::Guild,
    mail::Mail,
    map::Position,
    mob::MobTemplate,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBundle {
    pub character: PlayerCharacterView,
    pub stats: PlayerCharacterStatsView,
    pub state: PlayerCharacterStateView,
    pub inventory: PlayerInventorySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameBootstrap {
    pub character: Option<CharacterBundle>,
    pub position: Position,
    pub mobs: Vec<MobTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCharacterView {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub class: String,
    pub level: i32,
    pub exp: i64,
    pub gold: i64,
    pub yuanbao: i64,
    pub power: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAttributesView {
    #[serde(rename = "str")]
    pub str_: i64,
    pub dex: i64,
    #[serde(rename = "int")]
    pub int_: i64,
    pub con: i64,
    pub spirit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCharacterStatsView {
    pub character_id: i64,
    pub attrs: PlayerAttributesView,
    pub max_hp: i64,
    pub max_mp: i64,
    pub atk: i64,
    pub def: i64,
    pub mag: i64,
    pub mdef: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCharacterStateView {
    pub character_id: i64,
    pub zone: String,
    pub room: String,
    pub hp: i64,
    pub mp: i64,
    pub stamina: i32,
    pub stamina_max: i32,
    pub online: bool,
    pub pk_enabled: bool,
    pub sweep_attack_players: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAdventureOptionView {
    pub id: String,
    pub label: String,
    pub cost_gold: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAdventureOfferView {
    pub id: i64,
    pub script_id: String,
    pub title: String,
    pub body: String,
    pub options: Vec<PlayerAdventureOptionView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAdventureResolveView {
    pub offer: PlayerAdventureOfferView,
    pub message: String,
    pub character: PlayerCharacterView,
    pub state: PlayerCharacterStateView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkSettingsRequest {
    pub character_id: i64,
    pub pk_enabled: bool,
    pub sweep_attack_players: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkBotRequest {
    pub character_id: i64,
    pub target_index: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerEquipmentSlots {
    pub weapon: Option<i64>,
    pub chest: Option<i64>,
    pub head: Option<i64>,
    pub feet: Option<i64>,
    pub waist: Option<i64>,
    pub neck: Option<i64>,
    pub ring_left: Option<i64>,
    pub ring_right: Option<i64>,
    pub bracelet_left: Option<i64>,
    pub bracelet_right: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInventorySummary {
    pub bag_used: usize,
    pub bag_limit: usize,
    pub warehouse_used: usize,
    pub equipment: PlayerEquipmentSlots,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInventoryItemView {
    pub id: i64,
    pub character_id: i64,
    pub template_id: String,
    pub name: String,
    pub kind: String,
    pub template_slot: Option<String>,
    pub rarity: String,
    pub price: i64,
    pub stackable: bool,
    pub stats: serde_json::Value,
    pub quantity: i64,
    pub location: String,
    pub equipped_slot: Option<String>,
    pub bind: bool,
    pub durability: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMailAttachmentView {
    pub id: i64,
    pub item_template_id: Option<String>,
    pub item_name: Option<String>,
    pub quantity: i64,
    pub gold: i64,
    pub yuanbao: i64,
    pub claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMailView {
    pub id: i64,
    pub to_character_id: i64,
    pub from_name: String,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub claimed: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub attachments: Vec<PlayerMailAttachmentView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMailList {
    pub mails: Vec<PlayerMailView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailActionRequest {
    pub character_id: i64,
    pub mail_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailClaimResult {
    pub mail: PlayerMailView,
    pub inventory: InventoryView,
    pub character: PlayerCharacterView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildView {
    pub id: i64,
    pub name: String,
    pub notice: String,
    pub level: i32,
    pub funds: i64,
    pub sabak_owner: bool,
    pub member_count: i64,
    pub joined: bool,
    pub role: Option<String>,
    pub contribution: i64,
    pub pending_application: bool,
    pub projects: Vec<PlayerGuildProjectView>,
    pub totems: Vec<PlayerGuildTotemView>,
    pub war_techs: Vec<PlayerGuildWarTechView>,
    pub sabak_tax_claimed_today: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildProjectView {
    pub kind: String,
    pub name: String,
    pub description: String,
    pub progress: i64,
    pub required: i64,
    pub completed: bool,
    pub completed_today: bool,
    pub min_level: i32,
    pub available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildTotemView {
    pub kind: String,
    pub name: String,
    pub description: String,
    pub level: i32,
    pub next_cost: i64,
    pub max_level: i32,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildWarTechView {
    pub kind: String,
    pub name: String,
    pub description: String,
    pub level: i32,
    pub next_cost: i64,
    pub score_bonus: i64,
    pub unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinGuildRequest {
    pub character_id: i64,
    pub guild_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGuildRequest {
    pub character_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildApplyRequest {
    pub character_id: i64,
    pub guild_id: i64,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildDonateRequest {
    pub character_id: i64,
    pub gold: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildTaskRequest {
    pub character_id: i64,
    pub task_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildBenefitRequest {
    pub character_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildReviewApplicationRequest {
    pub character_id: i64,
    pub application_id: i64,
    pub accept: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildApplicationView {
    pub id: i64,
    pub guild_id: i64,
    pub guild_name: String,
    pub character_id: i64,
    pub character_name: String,
    pub message: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGuildApplicationList {
    pub applications: Vec<PlayerGuildApplicationView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinGuildResult {
    pub guild: PlayerGuildView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildBenefitResult {
    pub guild: PlayerGuildView,
    pub inventory: InventoryView,
    pub character: PlayerCharacterView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerActivityView {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub points: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerQuestView {
    pub id: String,
    pub category: String,
    pub name: String,
    pub description: String,
    pub objectives: serde_json::Value,
    pub rewards: serde_json::Value,
    pub progress: i64,
    pub required: i64,
    pub status: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerQuestList {
    pub quests: Vec<PlayerQuestView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestActionRequest {
    pub character_id: i64,
    pub quest_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestClaimResult {
    pub quests: PlayerQuestList,
    pub inventory: InventoryView,
    pub character: PlayerCharacterView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRankingEntry {
    pub rank: i64,
    pub character_id: i64,
    pub name: String,
    pub class: String,
    pub level: i32,
    pub exp: i64,
    pub gold: i64,
    pub yuanbao: i64,
    pub power: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOverview {
    pub activities: Vec<PlayerActivityView>,
    pub guilds: Vec<PlayerGuildView>,
    pub power_rankings: Vec<PlayerRankingEntry>,
    pub level_rankings: Vec<PlayerRankingEntry>,
    pub systems: PlayerSystemsView,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerSystemsView {
    pub pets: Vec<PlayerPetView>,
    pub treasures: Vec<PlayerTreasureView>,
    pub vip: Option<PlayerVipView>,
    pub vip_settings: PlayerVipSettingsView,
    pub cultivation: Option<PlayerCultivationView>,
    pub wanxiang: Option<PlayerWanxiangBodyView>,
    pub unlocks: PlayerSystemUnlocksView,
    pub sabak: Option<PlayerSabakView>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerSystemUnlocksView {
    pub pet: PlayerSystemUnlockView,
    pub treasure: PlayerSystemUnlockView,
    pub cultivation: PlayerSystemUnlockView,
    pub wanxiang: PlayerSystemUnlockView,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerSystemUnlockView {
    pub unlocked: bool,
    pub requirement: String,
    pub source: String,
    pub unlocked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUpgradeRequest {
    pub character_id: i64,
    pub target_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemActionResult {
    pub systems: PlayerSystemsView,
    pub character: PlayerCharacterView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WanxiangExtractRequest {
    pub character_id: i64,
    pub max_tier: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WanxiangExtractResult {
    pub systems: PlayerSystemsView,
    pub inventory: InventoryView,
    pub character: PlayerCharacterView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPetView {
    pub id: i64,
    pub template_id: String,
    pub name: String,
    pub rarity: String,
    pub level: i32,
    pub exp: i64,
    pub fighting: bool,
    pub base_hp: i64,
    pub base_atk: i64,
    pub skills: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTreasureView {
    pub id: i64,
    pub template_id: String,
    pub name: String,
    pub family: String,
    pub passive: String,
    pub level: i32,
    pub stage: i32,
    pub equipped: bool,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerVipView {
    pub tier: String,
    pub starts_at: String,
    pub ends_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerVipSettingsView {
    pub hp_enabled: bool,
    pub hp_threshold_pct: i32,
    pub hp_template_id: String,
    pub mp_enabled: bool,
    pub mp_threshold_pct: i32,
    pub mp_template_id: String,
    pub auto_decompose_enabled: bool,
    pub auto_decompose_max_tier: i32,
    pub auto_extract_essence_enabled: bool,
    pub auto_extract_essence_max_tier: i32,
}

impl Default for PlayerVipSettingsView {
    fn default() -> Self {
        Self {
            hp_enabled: true,
            hp_threshold_pct: 35,
            hp_template_id: "potion_large".into(),
            mp_enabled: true,
            mp_threshold_pct: 30,
            mp_template_id: "potion_mana_large".into(),
            auto_decompose_enabled: false,
            auto_decompose_max_tier: 0,
            auto_extract_essence_enabled: false,
            auto_extract_essence_max_tier: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VipPotionSettingsRequest {
    pub character_id: i64,
    pub hp_enabled: bool,
    pub hp_threshold_pct: i32,
    pub hp_template_id: String,
    pub mp_enabled: bool,
    pub mp_threshold_pct: i32,
    pub mp_template_id: String,
    #[serde(default)]
    pub auto_decompose_enabled: bool,
    #[serde(default)]
    pub auto_decompose_max_tier: i32,
    #[serde(default)]
    pub auto_extract_essence_enabled: bool,
    #[serde(default)]
    pub auto_extract_essence_max_tier: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCultivationView {
    pub realm: String,
    pub layer: i32,
    pub next_level_exp: i64,
    pub progress_pct: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerWanxiangBodyView {
    pub level: i32,
    pub essence: i64,
    pub next_cost: i64,
    pub fail_pct: i32,
    pub progress_pct: i32,
    pub atk: i64,
    pub mag: i64,
    pub hp: i64,
    pub mp: i64,
    pub def: i64,
    pub mdef: i64,
    pub life_steal_pct: i64,
    pub mana_steal_pct: i64,
    pub damage_reduce_pct: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSabakView {
    pub campaign_id: i64,
    pub status: String,
    pub signup_starts_at: String,
    pub battle_starts_at: String,
    pub battle_ends_at: String,
    pub defending_guild: Option<String>,
    pub winner_guild: Option<String>,
    pub signup_count: i64,
    pub tax_rate_pct: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSkillView {
    pub id: String,
    pub name: String,
    pub class: String,
    pub min_level: i32,
    pub mp_cost: i64,
    pub cooldown_ms: i32,
    pub config: serde_json::Value,
    pub learned: bool,
    pub auto_enabled: bool,
    pub level: Option<i32>,
    pub proficiency: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSkillList {
    pub skills: Vec<PlayerSkillView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnSkillRequest {
    pub character_id: i64,
    pub skill_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAutoToggleRequest {
    pub character_id: i64,
    pub skill_id: String,
    pub auto_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnSkillResult {
    pub skill: PlayerSkillView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub online: i64,
    pub accounts: i64,
    pub characters: i64,
    pub mails: i64,
    pub guilds: i64,
    pub pending_backups: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccountView {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub status: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
    pub character_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAccountList {
    pub total: i64,
    pub accounts: Vec<AdminAccountView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCharacterListItem {
    pub id: i64,
    pub account_id: i64,
    pub account_username: String,
    pub name: String,
    pub class: String,
    pub level: i32,
    pub exp: i64,
    pub gold: i64,
    pub yuanbao: i64,
    pub power: i64,
    pub zone: Option<String>,
    pub room: Option<String>,
    pub hp: Option<i64>,
    pub mp: Option<i64>,
    pub online: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCharacterList {
    pub total: i64,
    pub characters: Vec<AdminCharacterListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCharacterDetail {
    pub character: AdminCharacterListItem,
    pub stats: PlayerCharacterStatsView,
    pub state: PlayerCharacterStateView,
    pub inventory: InventoryView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminMailView {
    pub id: i64,
    pub to_character_id: i64,
    pub to_character_name: Option<String>,
    pub account_id: Option<i64>,
    pub account_username: Option<String>,
    pub from_account_id: Option<i64>,
    pub from_name: String,
    pub title: String,
    pub read: bool,
    pub claimed: bool,
    pub expires_at: Option<String>,
    pub created_at: String,
    pub attachment_count: i64,
    pub attachment_gold: i64,
    pub attachment_yuanbao: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminMailOverview {
    pub total: i64,
    pub unread: i64,
    pub unclaimed: i64,
    pub with_attachments: i64,
    pub recent: Vec<AdminMailView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminItemTemplateView {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub slot: Option<String>,
    pub rarity: String,
    pub price: i64,
    pub stackable: bool,
    pub stats: serde_json::Value,
    pub flags: serde_json::Value,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminItemTemplateList {
    pub total: i64,
    pub items: Vec<AdminItemTemplateView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminItemTemplateInput {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub slot: Option<String>,
    pub rarity: String,
    pub price: i64,
    pub stackable: bool,
    pub stats: serde_json::Value,
    pub flags: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminGmActionRequest {
    pub action: Option<String>,
    pub code: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    pub account_id: Option<i64>,
    pub status: Option<String>,
    pub reason: Option<String>,
    pub character_id: Option<i64>,
    pub zone: Option<String>,
    pub room: Option<String>,
    pub hp: Option<i64>,
    pub mp: Option<i64>,
    pub online: Option<bool>,
    pub force_offline: Option<bool>,
    pub item: Option<AdminItemTemplateInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminMobTemplateView {
    pub id: String,
    pub name: String,
    pub level: i32,
    pub max_hp: i64,
    pub atk: i64,
    pub def: i64,
    pub exp: i64,
    pub gold: i64,
    pub boss: bool,
    pub respawn_seconds: i32,
    pub drops: serde_json::Value,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminMobTemplateList {
    pub total: i64,
    pub mobs: Vec<AdminMobTemplateView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditLogView {
    pub id: i64,
    pub admin_account_id: Option<i64>,
    pub admin_username: Option<String>,
    pub action: String,
    pub target: String,
    pub detail: serde_json::Value,
    pub ip: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAuditLogList {
    pub total: i64,
    pub logs: Vec<AdminAuditLogView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminCharacterView {
    pub character: Character,
    pub stats: CharacterStats,
    pub state: CharacterState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailBox {
    pub mails: Vec<Mail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuildList {
    pub guilds: Vec<Guild>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryView {
    pub summary: PlayerInventorySummary,
    pub items: Vec<PlayerInventoryItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryActionResult {
    pub inventory: InventoryView,
    pub character: PlayerCharacterView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopBuyRequest {
    pub character_id: i64,
    pub template_id: String,
    pub quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBotView {
    pub id: i64,
    pub name: String,
    pub bot_class: String,
    pub level: i32,
    pub exp: i64,
    pub gold: i64,
    pub power: i64,
    pub zone: String,
    pub room: String,
    pub hp: i64,
    pub mp: i64,
    pub mode: String,
    pub team_code: String,
    pub target_zone: String,
    pub target_room: String,
    pub enabled: bool,
    pub script: serde_json::Value,
    pub last_action_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBotList {
    pub total: i64,
    pub bots: Vec<AdminBotView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBotBatchRequest {
    #[serde(default)]
    pub bot_ids: Vec<i64>,
    pub mode: Option<String>,
    pub enabled: Option<bool>,
    pub zone: Option<String>,
    pub room: Option<String>,
    pub team_code: Option<String>,
    pub target_zone: Option<String>,
    pub target_room: Option<String>,
    #[serde(default)]
    pub script: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBotCreateRequest {
    pub name: String,
    pub bot_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBotDeleteRequest {
    pub bot_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBotTickRequest {
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminBotActionResult {
    pub bots: AdminBotList,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeConsignmentView {
    pub id: i64,
    pub seller_character_id: i64,
    pub seller_name: String,
    pub item_id: i64,
    pub template_id: String,
    pub name: String,
    pub kind: String,
    pub template_slot: Option<String>,
    pub rarity: String,
    pub quantity: i64,
    pub price: i64,
    pub price_currency: String,
    pub listing_fee_gold: i64,
    pub trade_tax_yuanbao: i64,
    pub trade_tax_gold: i64,
    pub seller_receives_yuanbao: i64,
    pub seller_receives_gold: i64,
    pub stats: serde_json::Value,
    pub bind: bool,
    pub durability: i32,
    pub expires_at: String,
    pub created_at: String,
    pub mine: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeList {
    pub consignments: Vec<TradeConsignmentView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConsignmentRequest {
    pub character_id: i64,
    pub item_id: i64,
    pub price: i64,
    pub price_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeActionRequest {
    pub character_id: i64,
    pub consignment_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeActionResult {
    pub consignments: Vec<TradeConsignmentView>,
    pub inventory: InventoryView,
    pub character: PlayerCharacterView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseItemResult {
    pub inventory: InventoryView,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RechargeCardRequest {
    pub character_id: i64,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RechargeCardResult {
    pub character: PlayerCharacterView,
    pub vip: Option<PlayerVipView>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfkStatusView {
    pub active: bool,
    pub mode: String,
    pub zone: Option<String>,
    pub room: Option<String>,
    pub training_skill_id: Option<String>,
    pub training_skill_name: Option<String>,
    pub started_at: Option<String>,
    pub last_settled_at: Option<String>,
    pub exp_per_minute: i64,
    pub gold_per_minute: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AfkSettleResult {
    pub status: AfkStatusView,
    pub exp: i64,
    pub gold: i64,
    pub minutes: i64,
    pub message: String,
    pub adventure: Option<PlayerAdventureOfferView>,
}
