import type {
  ApiOk,
  AdventureResolveResult,
  AfkSettleResult,
  AfkStatus,
  CharacterClass,
  CharacterRecord,
  CharacterState,
  GameBootstrap,
  GameOverview,
  GuildBenefitResult,
  InventoryActionResult,
  InventoryView,
  JoinGuildResult,
  LearnSkillResult,
  LoginResponse,
  MailClaimResult,
  PlayerQuestList,
  PlayerMail,
  PlayerMailList,
  PlayerGuildApplicationList,
  QuestClaimResult,
  PlayerSkillList,
  PlayerVipSettings,
  RechargeCardResult,
  RealtimeActionResult,
  RoomStateEvent,
  SystemActionResult,
  TradeActionResult,
  TradeList,
  UseItemResult,
  WanxiangExtractResult
} from './types'

const base = ''

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const response = await fetch(`${base}${path}`, {
    ...options,
    headers: {
      'content-type': 'application/json',
      ...(options.headers ?? {})
    }
  })
  const body = (await response.json()) as ApiOk<T> | { ok: false; message: string }
  if (!response.ok || !body.ok) {
    throw new Error('message' in body ? body.message : '请求失败')
  }
  return body.data
}

export function register(username: string, password: string, email?: string) {
  return request<{ account_id: number; username: string }>('/api/auth/register', {
    method: 'POST',
    body: JSON.stringify({ username, password, email })
  })
}

export function login(username: string, password: string, device = 'web') {
  return request<LoginResponse>('/api/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password, device })
  })
}

export function listCharacters(token: string) {
  return request<{ characters: CharacterRecord[] }>('/api/characters', {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function createCharacter(token: string, name: string, characterClass: CharacterClass) {
  return request<{ character: CharacterRecord }>('/api/characters', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ name, class: characterClass })
  })
}

export function getGameBootstrap(token: string, characterId?: number) {
  const query = characterId ? `?character_id=${characterId}` : ''
  return request<GameBootstrap>(`/api/game/bootstrap${query}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function getGameOverview(token: string, characterId?: number) {
  const query = characterId ? `?character_id=${characterId}` : ''
  return request<GameOverview>(`/api/game/overview${query}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function getQuests(token: string, characterId: number) {
  return request<PlayerQuestList>(`/api/game/quests?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function claimQuest(token: string, characterId: number, questId: string) {
  return request<QuestClaimResult>('/api/game/quests/claim', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, quest_id: questId })
  })
}

export function getRoomState(token: string, characterId: number) {
  return request<RoomStateEvent>(`/api/game/room?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function updatePkSettings(token: string, characterId: number, pkEnabled: boolean, sweepAttackPlayers: boolean) {
  return request<CharacterState>('/api/game/pk-settings', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({
      character_id: characterId,
      pk_enabled: pkEnabled,
      sweep_attack_players: sweepAttackPlayers
    })
  })
}

export function moveCharacter(token: string, characterId: number, direction: string) {
  return request<RealtimeActionResult>('/api/game/move', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, direction })
  })
}

export function attackMob(token: string, characterId: number, targetId: number) {
  return request<RealtimeActionResult>('/api/game/attack', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, target_id: targetId })
  })
}

export function attackBot(token: string, characterId: number, targetIndex: number) {
  return request<RealtimeActionResult>('/api/game/pk/bot', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, target_index: targetIndex })
  })
}

export function exploreArea(token: string, characterId: number) {
  return request<RealtimeActionResult>('/api/game/explore', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function resolveAdventure(token: string, characterId: number, adventureId: number, optionId: string) {
  return request<AdventureResolveResult>('/api/game/adventure/resolve', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, adventure_id: adventureId, option_id: optionId })
  })
}

export function teleport(token: string, characterId: number, zone: string, room: string) {
  return request<RealtimeActionResult>('/api/game/teleport', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, zone, room })
  })
}

export function exploreSecretRealm(token: string, characterId: number) {
  return request<RealtimeActionResult>('/api/game/secret-realm/explore', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function challengeTower(token: string, characterId: number) {
  return request<RealtimeActionResult>('/api/game/tower/challenge', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function challengeWorldBoss(token: string, characterId: number) {
  return request<RealtimeActionResult>('/api/game/world-boss/challenge', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function wildAfk(token: string, characterId: number) {
  return request<AfkStatus>('/api/game/afk/wild', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function castSkillHttp(token: string, characterId: number, skillId: string, targetId = 0) {
  return request<RealtimeActionResult>('/api/game/cast-skill', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, skill_id: skillId, target_id: targetId })
  })
}

export function joinGuild(token: string, characterId: number, guildId: number) {
  return request<JoinGuildResult>('/api/game/guild/join', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, guild_id: guildId })
  })
}

export function createGuild(token: string, characterId: number, name: string) {
  return request<JoinGuildResult>('/api/game/guild/create', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, name })
  })
}

export function applyGuild(token: string, characterId: number, guildId: number, message = '') {
  return request<JoinGuildResult>('/api/game/guild/apply', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, guild_id: guildId, message })
  })
}

export function donateGuild(token: string, characterId: number, gold: number) {
  return request<JoinGuildResult>('/api/game/guild/donate', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, gold })
  })
}

export function completeGuildTask(token: string, characterId: number, taskKind: string) {
  return request<JoinGuildResult>('/api/game/guild/task/complete', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, task_kind: taskKind })
  })
}

export function claimGuildBenefit(token: string, characterId: number) {
  return request<GuildBenefitResult>('/api/game/guild/benefit/claim', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function useGuildMeritToken(token: string, characterId: number) {
  return request<GuildBenefitResult>('/api/game/guild/merit/use', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function buyGuildShopItem(token: string, characterId: number, itemId: string) {
  return request<GuildBenefitResult>('/api/game/guild/shop/buy', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function upgradeGuildTotem(token: string, characterId: number, totem: string) {
  return request<GuildBenefitResult>('/api/game/guild/totem/upgrade', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, totem })
  })
}

export function chargeGuildWarTech(token: string, characterId: number, kind: string) {
  return request<GuildBenefitResult>('/api/game/guild/war-tech/charge', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, kind })
  })
}

export function claimSabakTax(token: string, characterId: number) {
  return request<GuildBenefitResult>('/api/game/guild/sabak-tax/claim', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function upgradeSpecialSkill(token: string, characterId: number, skillId: string) {
  return request<InventoryActionResult>('/api/game/npc/special-skill/upgrade', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, skill_id: skillId })
  })
}

export function getGuildApplications(token: string, characterId: number) {
  return request<PlayerGuildApplicationList>(`/api/game/guild/applications?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function reviewGuildApplication(token: string, characterId: number, applicationId: number, accept: boolean) {
  return request<PlayerGuildApplicationList>('/api/game/guild/applications/review', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, application_id: applicationId, accept })
  })
}

export function getInventory(token: string, characterId: number) {
  return request<InventoryView>(`/api/game/inventory?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function equipItem(token: string, characterId: number, itemId: number) {
  return request<InventoryView>('/api/game/equip', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function unequipItem(token: string, characterId: number, itemId: number) {
  return request<InventoryView>('/api/game/unequip', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function useItem(token: string, characterId: number, itemId: number) {
  return request<UseItemResult>('/api/game/use-item', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function enhanceItem(token: string, characterId: number, itemId: number) {
  return request<InventoryActionResult>('/api/game/enhance', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function recycleItem(token: string, characterId: number, itemId: number) {
  return request<InventoryActionResult>('/api/game/recycle', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function decomposeEquipment(token: string, characterId: number, rarities: string[] = [], itemIds: number[] = []) {
  return request<InventoryActionResult>('/api/game/equipment/decompose', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, rarities, item_ids: itemIds })
  })
}

export function decomposeMisc(token: string, characterId: number, kinds: string[] = ['book'], itemIds: number[] = []) {
  return request<InventoryActionResult>('/api/game/items/decompose', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, kinds, item_ids: itemIds })
  })
}

export function storeItem(token: string, characterId: number, itemId: number) {
  return request<InventoryActionResult>('/api/game/store', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function retrieveItem(token: string, characterId: number, itemId: number) {
  return request<InventoryActionResult>('/api/game/retrieve', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId })
  })
}

export function buyShopItem(token: string, characterId: number, templateId: string, quantity = 1) {
  return request<InventoryActionResult>('/api/game/shop/buy', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, template_id: templateId, quantity })
  })
}

export function buyYuanbaoItem(token: string, characterId: number, templateId: string, quantity = 1) {
  return request<InventoryActionResult>('/api/game/yuanbao-shop/buy', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, template_id: templateId, quantity })
  })
}

export function updateVipPotionSettings(
  token: string,
  characterId: number,
  settings: PlayerVipSettings
) {
  return request<PlayerVipSettings>('/api/game/vip-potion-settings', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, ...settings })
  })
}

export function exchangeNpcMaterial(token: string, characterId: number, materialId: string) {
  return request<InventoryActionResult>('/api/game/npc/material-exchange', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, material_id: materialId })
  })
}

export function upgradeBattleInstinct(token: string, characterId: number) {
  return request<InventoryActionResult>('/api/game/npc/battle-instinct/upgrade', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function redeemRechargeCard(token: string, characterId: number, code: string) {
  return request<RechargeCardResult>('/api/game/recharge/redeem', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, code })
  })
}

export function upgradePet(token: string, characterId: number, targetId?: number) {
  return request<SystemActionResult>('/api/game/systems/pet/upgrade', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, target_id: targetId ?? null })
  })
}

export function upgradeTreasure(token: string, characterId: number, targetId?: number) {
  return request<SystemActionResult>('/api/game/systems/treasure/upgrade', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, target_id: targetId ?? null })
  })
}

export function cultivationBreakthrough(token: string, characterId: number) {
  return request<SystemActionResult>('/api/game/systems/cultivation/breakthrough', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function upgradeWanxiang(token: string, characterId: number) {
  return request<SystemActionResult>('/api/game/systems/wanxiang/upgrade', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function extractWanxiangEssence(token: string, characterId: number, maxTier: number) {
  return request<WanxiangExtractResult>('/api/game/systems/wanxiang/extract', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, max_tier: maxTier })
  })
}

export function getSkills(token: string, characterId: number) {
  return request<PlayerSkillList>(`/api/game/skills?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function learnSkill(token: string, characterId: number, skillId: string) {
  return request<LearnSkillResult>('/api/game/skills/learn', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, skill_id: skillId })
  })
}

export function upgradeSkill(token: string, characterId: number, skillId: string) {
  return request<LearnSkillResult>('/api/game/skills/upgrade', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, skill_id: skillId })
  })
}

export function toggleSkillAuto(token: string, characterId: number, skillId: string, autoEnabled: boolean) {
  return request<LearnSkillResult>('/api/game/skills/auto-toggle', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, skill_id: skillId, auto_enabled: autoEnabled })
  })
}

export function getMail(token: string, characterId: number) {
  return request<PlayerMailList>(`/api/game/mail?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function markMailRead(token: string, characterId: number, mailId: number) {
  return request<PlayerMail>('/api/game/mail/read', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, mail_id: mailId })
  })
}

export function claimMail(token: string, characterId: number, mailId: number) {
  return request<MailClaimResult>('/api/game/mail/claim', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, mail_id: mailId })
  })
}

export function deleteMail(token: string, characterId: number, mailId: number) {
  return request<PlayerMailList>('/api/game/mail/delete', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, mail_id: mailId })
  })
}

export function getAfkStatus(token: string, characterId: number) {
  return request<AfkStatus>(`/api/game/afk?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function startAfk(token: string, characterId: number, skillId: string) {
  return request<AfkStatus>('/api/game/afk/start', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, skill_id: skillId })
  })
}

export function settleAfk(token: string, characterId: number) {
  return request<AfkSettleResult>('/api/game/afk/settle', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function stopAfk(token: string, characterId: number) {
  return request<AfkSettleResult>('/api/game/afk/stop', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId })
  })
}

export function getTradeList(token: string, characterId: number) {
  return request<TradeList>(`/api/game/trade?character_id=${characterId}`, {
    headers: { authorization: `Bearer ${token}` }
  })
}

export function listTradeItem(
  token: string,
  characterId: number,
  itemId: number,
  price: number,
  priceCurrency: 'gold' | 'yuanbao'
) {
  return request<TradeActionResult>('/api/game/trade/list', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, item_id: itemId, price, price_currency: priceCurrency })
  })
}

export function buyTradeItem(token: string, characterId: number, consignmentId: number) {
  return request<TradeActionResult>('/api/game/trade/buy', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, consignment_id: consignmentId })
  })
}

export function cancelTradeItem(token: string, characterId: number, consignmentId: number) {
  return request<TradeActionResult>('/api/game/trade/cancel', {
    method: 'POST',
    headers: { authorization: `Bearer ${token}` },
    body: JSON.stringify({ character_id: characterId, consignment_id: consignmentId })
  })
}
