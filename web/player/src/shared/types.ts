export type CharacterClass = 'warrior' | 'mage' | 'taoist' | 'assassin'

export interface ApiOk<T> {
  ok: boolean
  data: T
}

export interface LoginResponse {
  account_id: number
  token: string
  expires_at: string
}

export interface CharacterRecord {
  id: number
  account_id: number
  name: string
  class: CharacterClass
  level: number
  exp: number
  gold: number
  yuanbao: number
  power: number
}

export interface Position {
  zone: string
  room: string
}

export interface MobTemplate {
  id: string
  name: string
  level: number
  max_hp: number
  atk: number
  def: number
  exp: number
  gold: number
  boss: boolean
  respawn_seconds: number
}

export interface CharacterStats {
  character_id: number
  attrs: {
    str: number
    dex: number
    int: number
    con: number
    spirit: number
  }
  max_hp: number
  max_mp: number
  atk: number
  def: number
  mag: number
  mdef: number
}

export interface CharacterState {
  character_id: number
  zone: string
  room: string
  hp: number
  mp: number
  stamina: number
  stamina_max: number
  online: boolean
  pk_enabled: boolean
  sweep_attack_players: boolean
  updated_at: string
}

export interface InventorySummary {
  bag_used: number
  bag_limit: number
  warehouse_used: number
  equipment: Record<string, number | null>
}

export interface InventoryItem {
  id: number
  character_id: number
  template_id: string
  name: string
  kind: string
  template_slot: string | null
  rarity: string
  price: number
  stackable: boolean
  stats: Record<string, unknown>
  quantity: number
  location: 'bag' | 'warehouse' | 'equipped' | string
  equipped_slot: string | null
  bind: boolean
  durability: number
}

export interface InventoryView {
  summary: InventorySummary
  items: InventoryItem[]
}

export interface UseItemResult {
  inventory: InventoryView
  message: string
}

export interface InventoryActionResult {
  inventory: InventoryView
  character: CharacterRecord
  message: string
}

export interface AfkStatus {
  active: boolean
  mode: 'safe' | 'wild' | string
  zone: string | null
  room: string | null
  training_skill_id: string | null
  training_skill_name: string | null
  started_at: string | null
  last_settled_at: string | null
  exp_per_minute: number
  gold_per_minute: number
}

export interface AfkSettleResult {
  status: AfkStatus
  exp: number
  gold: number
  minutes: number
  message: string
  adventure: AdventureOffer | null
}

export interface AdventureOption {
  id: string
  label: string
  cost_gold: number
}

export interface AdventureOffer {
  id: number
  script_id: string
  title: string
  body: string
  options: AdventureOption[]
}

export interface AdventureResolveResult {
  offer: AdventureOffer
  message: string
  character: CharacterRecord
  state: CharacterState
}

export interface MailAttachment {
  id: number
  item_template_id: string | null
  item_name: string | null
  quantity: number
  gold: number
  yuanbao: number
  claimed: boolean
}

export interface PlayerMail {
  id: number
  to_character_id: number
  from_name: string
  title: string
  body: string
  read: boolean
  claimed: boolean
  expires_at: string | null
  created_at: string
  attachments: MailAttachment[]
}

export interface PlayerMailList {
  mails: PlayerMail[]
}

export interface MailClaimResult {
  mail: PlayerMail
  inventory: InventoryView
  character: CharacterRecord
  message: string
}

export interface PlayerGuild {
  id: number
  name: string
  notice: string
  level: number
  funds: number
  sabak_owner: boolean
  member_count: number
  joined: boolean
  role: string | null
  contribution: number
  pending_application: boolean
  projects: PlayerGuildProject[]
  totems: PlayerGuildTotem[]
  war_techs: PlayerGuildWarTech[]
  sabak_tax_claimed_today: boolean
}

export interface PlayerGuildProject {
  kind: string
  name: string
  description: string
  progress: number
  required: number
  completed: boolean
  completed_today: boolean
  min_level: number
  available: boolean
}

export interface PlayerGuildTotem {
  kind: string
  name: string
  description: string
  level: number
  next_cost: number
  max_level: number
  unlocked: boolean
}

export interface PlayerGuildWarTech {
  kind: string
  name: string
  description: string
  level: number
  next_cost: number
  score_bonus: number
  unlocked: boolean
}

export interface JoinGuildResult {
  guild: PlayerGuild
  message: string
}

export interface GuildBenefitResult {
  guild: PlayerGuild
  inventory: InventoryView
  character: CharacterRecord
  message: string
}

export interface PlayerGuildApplication {
  id: number
  guild_id: number
  guild_name: string
  character_id: number
  character_name: string
  message: string
  status: string
  created_at: string
}

export interface PlayerGuildApplicationList {
  applications: PlayerGuildApplication[]
}

export interface PlayerActivity {
  id: number
  code: string
  name: string
  enabled: boolean
  config: Record<string, unknown>
  points: number
}

export interface PlayerQuest {
  id: string
  category: 'newbie' | 'main' | 'side' | 'daily' | string
  name: string
  description: string
  objectives: Record<string, unknown>
  rewards: Record<string, unknown>
  progress: number
  required: number
  status: 'available' | 'progress' | 'completed' | 'claimed' | string
  sort_order: number
}

export interface PlayerQuestList {
  quests: PlayerQuest[]
}

export interface QuestClaimResult {
  quests: PlayerQuestList
  inventory: InventoryView
  character: CharacterRecord
  message: string
}

export interface RankingEntry {
  rank: number
  character_id: number
  name: string
  class: CharacterClass | string
  level: number
  exp: number
  gold: number
  yuanbao: number
  power: number
}

export interface GameOverview {
  activities: PlayerActivity[]
  guilds: PlayerGuild[]
  power_rankings: RankingEntry[]
  level_rankings: RankingEntry[]
  systems: PlayerSystems
}

export interface PlayerSystems {
  pets: PlayerPet[]
  treasures: PlayerTreasure[]
  vip: PlayerVip | null
  vip_settings: PlayerVipSettings
  cultivation: PlayerCultivation | null
  wanxiang: PlayerWanxiangBody | null
  unlocks: PlayerSystemUnlocks
  sabak: PlayerSabak | null
}

export interface PlayerSystemUnlocks {
  pet: PlayerSystemUnlock
  treasure: PlayerSystemUnlock
  cultivation: PlayerSystemUnlock
  wanxiang: PlayerSystemUnlock
}

export interface PlayerSystemUnlock {
  unlocked: boolean
  requirement: string
  source: string
  unlocked_at: string | null
}

export interface PlayerPet {
  id: number
  template_id: string
  name: string
  rarity: string
  level: number
  exp: number
  fighting: boolean
  base_hp: number
  base_atk: number
  skills: unknown
}

export interface PlayerTreasure {
  id: number
  template_id: string
  name: string
  family: string
  passive: string
  level: number
  stage: number
  equipped: boolean
  config: Record<string, unknown>
}

export interface PlayerVip {
  tier: string
  starts_at: string
  ends_at: string | null
}

export interface PlayerVipSettings {
  hp_enabled: boolean
  hp_threshold_pct: number
  hp_template_id: string
  mp_enabled: boolean
  mp_threshold_pct: number
  mp_template_id: string
  auto_decompose_enabled: boolean
  auto_decompose_max_tier: number
  auto_extract_essence_enabled: boolean
  auto_extract_essence_max_tier: number
}

export interface PlayerCultivation {
  realm: string
  layer: number
  next_level_exp: number
  progress_pct: number
}

export interface PlayerWanxiangBody {
  level: number
  essence: number
  next_cost: number
  fail_pct: number
  progress_pct: number
  atk: number
  mag: number
  hp: number
  mp: number
  def: number
  mdef: number
  life_steal_pct: number
  mana_steal_pct: number
  damage_reduce_pct: number
}

export interface SystemActionResult {
  systems: PlayerSystems
  character: CharacterRecord
  message: string
}

export interface WanxiangExtractResult {
  systems: PlayerSystems
  inventory: InventoryView
  character: CharacterRecord
  message: string
}

export interface PlayerSabak {
  campaign_id: number
  status: string
  signup_starts_at: string
  battle_starts_at: string
  battle_ends_at: string
  defending_guild: string | null
  winner_guild: string | null
  signup_count: number
  tax_rate_pct: number
}

export interface RechargeCardResult {
  character: CharacterRecord
  vip: PlayerVip | null
  message: string
}

export interface PlayerSkill {
  id: string
  name: string
  class: CharacterClass | 'all' | string
  min_level: number
  mp_cost: number
  cooldown_ms: number
  config: Record<string, unknown>
  learned: boolean
  auto_enabled: boolean
  level: number | null
  proficiency: number | null
}

export interface PlayerSkillList {
  skills: PlayerSkill[]
}

export interface LearnSkillResult {
  skill: PlayerSkill
  message: string
}

export interface TradeConsignment {
  id: number
  seller_character_id: number
  seller_name: string
  item_id: number
  template_id: string
  name: string
  kind: string
  template_slot: string | null
  rarity: string
  quantity: number
  price: number
  price_currency: 'gold' | 'yuanbao' | string
  listing_fee_gold: number
  trade_tax_yuanbao: number
  trade_tax_gold: number
  seller_receives_yuanbao: number
  seller_receives_gold: number
  stats: Record<string, unknown>
  bind: boolean
  durability: number
  expires_at: string
  created_at: string
  mine: boolean
}

export interface TradeList {
  consignments: TradeConsignment[]
}

export interface TradeActionResult {
  consignments: TradeConsignment[]
  inventory: InventoryView
  character: CharacterRecord
  message: string
}

export interface CharacterBundle {
  character: CharacterRecord
  stats: CharacterStats
  state: CharacterState
  inventory: InventorySummary
}

export interface GameBootstrap {
  character: CharacterBundle | null
  position: Position
  mobs: MobTemplate[]
}

export interface Room {
  id: string
  name: string
  desc: string
  exits: Record<string, string>
  spawns: string[]
  safe: boolean
}

export interface WsEnvelope<T = unknown> {
  type: string
  seq: number
  payload: T
}

export interface RoomStateEvent {
  room: Room
  players: string[]
  mobs: string[]
}

export interface RealtimeActionResult {
  room: RoomStateEvent
  log: string[]
  character: CharacterRecord
  state: CharacterState
  inventory: InventoryView | null
  adventure: AdventureOffer | null
}

export interface CombatLogEvent {
  lines: string[]
}

export interface SystemNoticeEvent {
  level: string
  message: string
}
