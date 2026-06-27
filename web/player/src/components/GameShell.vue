<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { Backpack, Bot, BookOpen, Coffee, Gift, Mail, Map, PackageOpen, ScrollText, Shield, ShoppingBag, Skull, Sparkles, Sword, Users, Search, Grid, List, ChevronLeft, ChevronRight, ChevronDown, MoreHorizontal } from '@lucide/vue'
import type {
  AdventureOffer,
  AfkStatus,
  CharacterBundle,
  GameOverview,
  InventoryItem,
  InventoryView,
  MailAttachment,
  MobTemplate,
  PlayerGuild,
  PlayerGuildApplication,
  PlayerMail,
  PlayerQuest,
  PlayerSkill,
  PlayerVipSettings,
  RankingEntry,
  RoomStateEvent,
  TradeConsignment
} from '../shared/types'

const props = defineProps<{
  roomState: RoomStateEvent | null
  log: string[]
  mobs: MobTemplate[]
  overview: GameOverview | null
  inventory: InventoryView | null
  skills: PlayerSkill[]
  quests: PlayerQuest[]
  mails: PlayerMail[]
  guildApplications: PlayerGuildApplication[]
  consignments: TradeConsignment[]
  afk: AfkStatus | null
  zone: string
  authenticated: boolean
  lastEventAt: string
  bundle: CharacterBundle | null
  pendingAdventure: AdventureOffer | null
}>()

const emit = defineEmits<{
  explore: []
  exploreSecretRealm: []
  challengeTower: []
  challengeWorldBoss: []
  wildAfk: []
  attackBot: [targetIndex: number]
  move: [direction: string]
  teleport: [zone: string, room: string]
  refresh: []
  equip: [itemId: number]
  unequip: [itemId: number]
  useItem: [itemId: number]
  enhanceItem: [itemId: number]
  storeItem: [itemId: number]
  retrieveItem: [itemId: number]
  decomposeEquipment: [rarities: string[], itemIds: number[]]
  decomposeMisc: [kinds: string[], itemIds: number[]]
  buyShopItem: [templateId: string, quantity: number]
  buyYuanbaoItem: [templateId: string, quantity: number]
  updateVipPotionSettings: [settings: PlayerVipSettings]
  exchangeNpcMaterial: [materialId: string]
  upgradeBattleInstinct: []
  upgradeSpecialSkill: [skillId: string]
  redeemRecharge: []
  listTradeItem: [itemId: number, suggestedPrice: number]
  buyTradeItem: [consignmentId: number]
  cancelTradeItem: [consignmentId: number]
  learnSkill: [skillId: string]
  upgradeSkill: [skillId: string]
  toggleSkillAuto: [skillId: string, autoEnabled: boolean]
  claimQuest: [questId: string]
  readMail: [mailId: number]
  claimMail: [mailId: number]
  deleteMail: [mailId: number]
  joinGuild: [guildId: number]
  createGuild: [name: string]
  applyGuild: [guildId: number, message: string]
  donateGuild: [gold: number]
  completeGuildTask: [taskKind: string]
  claimGuildBenefit: []
  claimSabakTax: []
  useGuildMeritToken: []
  buyGuildShopItem: [itemId: string]
  upgradeGuildTotem: [totem: string]
  chargeGuildWarTech: [kind: string]
  reviewGuildApplication: [applicationId: number, accept: boolean]
  upgradePet: [petId?: number]
  upgradeTreasure: [treasureId?: number]
  cultivationBreakthrough: []
  upgradeWanxiang: []
  extractWanxiangEssence: [maxTier: number]
  updatePkSettings: [pkEnabled: boolean, sweepAttackPlayers: boolean]
  startAfk: [skillId: string]
  settleAfk: []
  stopAfk: []
  resolveAdventure: [adventureId: number, optionId: string]
}>()

type QuickPotionItem = InventoryItem & { stackIds: number[] }
type SetEffectDefinition = {
  pieces: number
  title: string
  text: string
}

const activePage = ref<'game' | 'inventory' | 'breakthrough' | 'guild' | 'yuanbao' | 'wiki' | 'about'>('game')
const inventorySubPage = ref<'equipment' | 'warehouse' | 'skills'>('equipment')
const inventoryTierFilter = ref('all')
const warehouseCategoryFilter = ref<'all' | 'equipment' | 'consumable' | 'material'>('all')
const selectedInventoryItemId = ref<number | null>(null)
const selectedSkillId = ref<string | null>(null)
const selectedTrainingSkillId = ref<string | null>(null)
const selectedQuickPotionId = ref<number | null>(null)
const vipHpEnabled = ref(true)
const vipHpThreshold = ref(35)
const vipHpTemplate = ref('potion_large')
const vipMpEnabled = ref(true)
const vipMpThreshold = ref(30)
const vipMpTemplate = ref('potion_mana_large')
const vipAutoDecomposeEnabled = ref(false)
const vipAutoDecomposeMaxTier = ref(3)
const vipAutoExtractEssenceEnabled = ref(false)
const vipAutoExtractEssenceMaxTier = ref(3)
const wanxiangExtractTier = ref(10)
const coffeeModalVisible = ref(false)
const coffeeQrUrl = ref('')
const coffeeQrLoading = ref(false)
const coffeeQrError = ref('')

const bagItems = computed(() => props.inventory?.items.filter((item) => item.location === 'bag') ?? [])
const quickPotionItems = computed<QuickPotionItem[]>(() => {
  const groups = new globalThis.Map<string, QuickPotionItem>()
  for (const item of bagItems.value.filter(isQuickPotion)) {
    const current = groups.get(item.template_id)
    if (!current) {
      groups.set(item.template_id, { ...item, stackIds: [item.id] })
      continue
    }
    current.quantity += item.quantity
    current.stackIds.push(item.id)
    if (item.id < current.id) current.id = item.id
  }
  return [...groups.values()].sort((left, right) => left.id - right.id)
})
const selectedQuickPotion = computed(() => {
  return quickPotionItems.value.find((item) => item.id === selectedQuickPotionId.value) ?? quickPotionItems.value[0] ?? null
})
const equippedItems = computed(() => props.inventory?.items.filter((item) => item.location === 'equipped') ?? [])
const warehouseItems = computed(() => props.inventory?.items.filter((item) => item.location === 'warehouse') ?? [])
const learnedSkills = computed(() => props.skills.filter((skill) => skill.learned))
const safeTrainingSkills = computed(() =>
  learnedSkills.value.filter((skill) => !(skill.config?.special_upgrade_only as boolean | undefined))
)
const selectedTrainingSkill = computed(() =>
  safeTrainingSkills.value.find((skill) => skill.id === selectedTrainingSkillId.value) ?? safeTrainingSkills.value[0] ?? null
)
const dominatorEquippedCount = computed(() => equippedItems.value.filter(isDominatorItem).length)
const hasDominatorSetPrivilege = computed(() => dominatorEquippedCount.value >= 8)
const characterClass = computed(() => props.bundle?.character.class ?? '')
const professionSkills = computed(() => props.skills.filter((skill) => skill.class === characterClass.value))
const commonSkills = computed(() => props.skills.filter((skill) => skill.class === 'all'))
const otherSkills = computed(() =>
  props.skills.filter((skill) => skill.class !== 'all' && skill.class !== characterClass.value)
)
const visibleQuests = computed(() => props.quests.filter((quest) => quest.status !== 'claimed').slice(0, 8))
const completedQuestCount = computed(() => props.quests.filter((quest) => quest.status === 'completed').length)
const activeGuild = computed(() => props.overview?.guilds.find((guild) => guild.joined) ?? null)
const canReviewGuildApplications = computed(() => ['leader', 'elder'].includes(activeGuild.value?.role ?? ''))
const primaryPet = computed(() => props.overview?.systems.pets[0] ?? null)
const primaryTreasure = computed(() => props.overview?.systems.treasures[0] ?? null)
const cultivation = computed(() => props.overview?.systems.cultivation ?? null)
const wanxiang = computed(() => props.overview?.systems.wanxiang ?? null)
const characterState = computed(() => props.bundle?.state ?? null)
const pkAllowedHere = computed(() => Boolean(props.roomState && !props.roomState.room.safe))
const isSafeRoom = computed(() => Boolean(props.roomState?.room.safe))
const isMaterialExchangeHub = computed(
  () => characterState.value?.zone === 'xiuzhen' && characterState.value?.room === 'tianshui_city'
)
const isMartialMasterHub = computed(
  () => characterState.value?.zone === 'feisheng' && characterState.value?.room === 'chaos_shelter'
)
const unreadMails = computed(() => props.mails.filter((mail) => !mail.read).length)
const visibleActivities = computed(() =>
  (props.overview?.activities ?? []).filter((activity) => activity.code !== 'boss_first_kill' && activity.name !== '触龙神首杀')
)
const marketItems = computed(() => props.consignments.filter((item) => !item.mine))
const myMarketItems = computed(() => props.consignments.filter((item) => item.mine))
const filteredBagItems = computed(() => {
  if (inventoryTierFilter.value === 'all') return bagItems.value
  return bagItems.value.filter((item) => tierFilterMatches(item, inventoryTierFilter.value))
})
const warehouseSearchQuery = ref('')
const warehouseQualityFilter = ref('all')
const warehouseTypeFilter = ref('all')
const warehouseSortOption = ref('default')
const warehouseViewMode = ref<'grid' | 'list'>('grid')
const warehouseCurrentPage = ref(1)
const warehousePageSize = ref(60)
const showMoreActions = ref(false)
const exploreCooldownMs = ref(0)
let exploreCooldownTimer: ReturnType<typeof window.setInterval> | null = null

const rarityWeights: Record<string, number> = {
  common: 1,
  uncommon: 2,
  rare: 3,
  epic: 4,
  legendary: 5,
  supreme: 5,
  ultimate: 5,
  mythic: 6
}

const filteredWarehouseItems = computed(() => {
  let items = [...warehouseItems.value]
  switch (warehouseCategoryFilter.value) {
    case 'equipment':
      items = items.filter((item) => Boolean(item.template_slot))
      break
    case 'consumable':
      items = items.filter((item) => item.kind === 'consumable')
      break
    case 'material':
      items = items.filter((item) => item.kind === 'material')
      break
  }

  if (warehouseSearchQuery.value.trim()) {
    const query = warehouseSearchQuery.value.toLowerCase().trim()
    items = items.filter((item) => item.name.toLowerCase().includes(query))
  }

  if (warehouseQualityFilter.value !== 'all') {
    items = items.filter((item) => tierFilterMatches(item, warehouseQualityFilter.value))
  }

  if (warehouseTypeFilter.value !== 'all') {
    const val = warehouseTypeFilter.value
    if (['weapon', 'chest', 'head', 'feet', 'waist', 'neck'].includes(val)) {
      items = items.filter((item) => item.template_slot === val)
    } else if (val === 'ring') {
      items = items.filter((item) => item.template_slot === 'ring_left' || item.template_slot === 'ring_right')
    } else if (val === 'bracelet') {
      items = items.filter((item) => item.template_slot === 'bracelet_left' || item.template_slot === 'bracelet_right')
    } else {
      items = items.filter((item) => item.kind === val)
    }
  }

  switch (warehouseSortOption.value) {
    case 'score_desc':
      items.sort((a, b) => {
        const scoreA = Number(a.stats.score ?? 0)
        const scoreB = Number(b.stats.score ?? 0)
        return scoreB - scoreA
      })
      break
    case 'score_asc':
      items.sort((a, b) => {
        const scoreA = Number(a.stats.score ?? 0)
        const scoreB = Number(b.stats.score ?? 0)
        return scoreA - scoreB
      })
      break
    case 'quantity_desc':
      items.sort((a, b) => b.quantity - a.quantity)
      break
    case 'quantity_asc':
      items.sort((a, b) => a.quantity - b.quantity)
      break
    case 'rarity_desc':
      items.sort((a, b) => {
        const weightA = rarityWeights[a.rarity] ?? 0
        const weightB = rarityWeights[b.rarity] ?? 0
        return weightB - weightA
      })
      break
  }

  return items
})

const paginatedWarehouseItems = computed(() => {
  const start = (warehouseCurrentPage.value - 1) * warehousePageSize.value
  const end = start + warehousePageSize.value
  return filteredWarehouseItems.value.slice(start, end)
})

const totalWarehousePages = computed(() => {
  return Math.max(1, Math.ceil(filteredWarehouseItems.value.length / warehousePageSize.value))
})

const warehousePages = computed(() => {
  const current = warehouseCurrentPage.value
  const total = totalWarehousePages.value
  const pages: (number | string)[] = []

  if (total <= 7) {
    for (let i = 1; i <= total; i++) pages.push(i)
  } else {
    if (current <= 4) {
      pages.push(1, 2, 3, 4, 5, '...', total)
    } else if (current >= total - 3) {
      pages.push(1, '...', total - 4, total - 3, total - 2, total - 1, total)
    } else {
      pages.push(1, '...', current - 1, current, current + 1, '...', total)
    }
  }
  return pages
})

function goToWarehousePage(p: number | string) {
  if (typeof p === 'number') {
    warehouseCurrentPage.value = p
  }
}

function prevWarehousePage() {
  if (warehouseCurrentPage.value > 1) {
    warehouseCurrentPage.value--
  }
}

function nextWarehousePage() {
  if (warehouseCurrentPage.value < totalWarehousePages.value) {
    warehouseCurrentPage.value++
  }
}

function formatNumber(num: number) {
  return new Intl.NumberFormat().format(num)
}

function toggleMoreActions() {
  showMoreActions.value = !showMoreActions.value
}

function handleExploreClick() {
  if (!canExploreArea.value) return
  emit('explore')
  if (!hasDominatorSetPrivilege.value) startExploreCooldown()
}

async function openCoffeeModal() {
  coffeeModalVisible.value = true
  coffeeQrLoading.value = true
  coffeeQrError.value = ''
  try {
    const response = await fetch('/api/game/coffee-qr')
    const body = await response.json() as { ok?: boolean; data?: { coffee_qr_url?: string }; message?: string }
    if (!response.ok || !body.ok) {
      throw new Error(body.message ?? '二维码读取失败')
    }
    coffeeQrUrl.value = body.data?.coffee_qr_url ?? ''
  } catch (error) {
    coffeeQrError.value = error instanceof Error ? error.message : '二维码读取失败'
  } finally {
    coffeeQrLoading.value = false
  }
}

function closeCoffeeModal() {
  coffeeModalVisible.value = false
}

function startExploreCooldown() {
  exploreCooldownMs.value = 2000
  const startedAt = Date.now()
  if (exploreCooldownTimer) window.clearInterval(exploreCooldownTimer)
  exploreCooldownTimer = window.setInterval(() => {
    exploreCooldownMs.value = Math.max(0, 2000 - (Date.now() - startedAt))
    if (exploreCooldownMs.value <= 0 && exploreCooldownTimer) {
      window.clearInterval(exploreCooldownTimer)
      exploreCooldownTimer = null
    }
  }, 100)
}

onBeforeUnmount(() => {
  if (exploreCooldownTimer) window.clearInterval(exploreCooldownTimer)
})

function itemDescriptionText(item: InventoryItem | null | undefined) {
  if (!item) return '请选择物品以查看其说明和相关信息。'
  if (item.stats.description) return String(item.stats.description)
  if (item.template_slot) {
    return `一件精良的${slotLabel(item.template_slot)}装备，装备后可大幅提升人物属性与评分，帮助探索更高难度的野外秘境。`
  }
  if (item.kind === 'consumable') {
    return `珍贵的消耗品药剂。使用后可以快速恢复人物生命值或魔法值，是探索打怪、保障生存的核心补给物资。`
  }
  if (item.kind === 'material') {
    return `贵重的进阶培养材料。主要产出自身日常打怪或秘境挑战，可用于行会建设、境界突破或装备强化，价值极高。`
  }
  if (item.kind === 'book') {
    return `技能修习秘笈。通过此技能书可以参悟并掌握相应的职业专属技能，是提升战斗流派玩法的必备道具。`
  }
  return `游戏内珍稀道具物资。可以通过存仓安全保管，或者取回背包在需要时使用、穿戴或进行其他杂项处置。`
}

watch(
  [warehouseCategoryFilter, warehouseSearchQuery, warehouseQualityFilter, warehouseTypeFilter, warehouseSortOption, warehousePageSize],
  () => {
    warehouseCurrentPage.value = 1
  }
)

const onDocClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.more-actions-wrapper')) {
    showMoreActions.value = false
  }
}
watch(inventorySubPage, (page) => {
  if (page === 'warehouse') {
    document.addEventListener('click', onDocClick)
  } else {
    document.removeEventListener('click', onDocClick)
    showMoreActions.value = false
  }
})
watch(selectedInventoryItemId, () => {
  showMoreActions.value = false
})
const selectedDecomposeItems = computed(() => filteredBagItems.value.filter(canDecompose))
const skillBookItems = computed(() =>
  [...bagItems.value, ...warehouseItems.value].filter((item) => item.kind === 'book')
)
const redundantSkillBookItems = computed(() => skillBookItems.value.filter(isRedundantSkillBook))
const staminaLabel = computed(() => {
  const state = characterState.value
  return `${state?.stamina ?? 0}/${state?.stamina_max ?? 5000}`
})
const fatigueActive = computed(() => (characterState.value?.stamina ?? 0) <= 0)
const canChallengeTower = computed(
  () => characterState.value?.zone === 'xiuzhen' && characterState.value?.room === 'tianshui_city'
)
const canExploreArea = computed(() => props.authenticated && (hasDominatorSetPrivilege.value || exploreCooldownMs.value <= 0))
const canExploreSecretRealm = computed(() => Boolean(props.roomState?.room.safe))
const canChallengeWorldBoss = computed(
  () => characterState.value?.zone === 'feisheng' && characterState.value?.room === 'void_fortress'
)
const isTrainingRoom = computed(() => characterState.value?.zone === 'xiuzhen' && characterState.value?.room === 'purgatory')
const isStudyRoom = computed(() => characterState.value?.zone === 'feisheng' && characterState.value?.room === 'void_realm')
const canStartPracticeAfk = computed(() => Boolean(props.authenticated && isTrainingRoom.value && !props.afk?.active))
const canStartStudyAfk = computed(() =>
  Boolean(props.authenticated && isStudyRoom.value && !props.afk?.active && selectedTrainingSkill.value)
)
const afkModeLabel = computed(() => {
  if (props.afk?.mode === 'skill_study') return '虚境研修'
  if (props.afk?.mode === 'wild') return '野外挂机'
  return '炼狱修炼'
})
const afkHint = computed(() =>
  props.afk?.mode === 'skill_study'
    ? `${props.afk?.training_skill_name ?? selectedTrainingSkill.value?.name ?? '选择技能'}技能经验 · 每 5 秒扣 1 万金币`
    : props.afk?.mode === 'wild'
      ? '当前区域挂机 · 每分钟消耗 1 点魔法 · 受每日体力影响'
      : '纯经验练功 · 每 5 秒扣 1 万金币 · 不产出装备金币材料'
)
const allInventoryItems = computed(() => [...equippedItems.value, ...bagItems.value, ...warehouseItems.value])
const selectedInventoryItem = computed(() => {
  return allInventoryItems.value.find((item) => item.id === selectedInventoryItemId.value) ?? allInventoryItems.value[0] ?? null
})
const selectedSkill = computed(
  () =>
    props.skills.find((skill) => skill.id === selectedSkillId.value) ??
    professionSkills.value[0] ??
    commonSkills.value[0] ??
    props.skills[0] ??
    null
)
const petUnlock = computed(() => props.overview?.systems.unlocks?.pet ?? null)
const treasureUnlock = computed(() => props.overview?.systems.unlocks?.treasure ?? null)
const cultivationUnlock = computed(() => props.overview?.systems.unlocks?.cultivation ?? null)

const resourceCaps = computed(() => {
  const stats = props.bundle?.stats
  const equipment = aggregateEquipmentStats()
  const skills = aggregateSkillStats()
  const systems = aggregateSystemStats()
  const hp = (stats?.max_hp ?? 0) + equipment.hp + skills.hp + systems.hp
  const mp = (stats?.max_mp ?? 0) + equipment.mp + skills.mp + systems.mp
  return {
    hp: applyPct(hp, equipment.hpPct + skills.hpPct + systems.hpPct),
    mp: applyPct(mp, equipment.mpPct + skills.mpPct + systems.mpPct),
    equipment,
    skills,
    systems
  }
})

const idleRecoveryLabel = computed(() => {
  const hp = Math.min(120, Math.max(4, Math.floor(resourceCaps.value.hp / 60)))
  const mp = Math.min(80, Math.max(3, Math.floor(resourceCaps.value.mp / 60)))
  return `${hp} 生命 / ${mp} 魔法`
})

watch(
  () => props.overview?.systems.vip_settings,
  (settings) => {
    if (!settings) return
    vipHpEnabled.value = settings.hp_enabled
    vipHpThreshold.value = settings.hp_threshold_pct
    vipHpTemplate.value = settings.hp_template_id
    vipMpEnabled.value = settings.mp_enabled
    vipMpThreshold.value = settings.mp_threshold_pct
    vipMpTemplate.value = settings.mp_template_id
    vipAutoDecomposeEnabled.value = settings.auto_decompose_enabled
    vipAutoDecomposeMaxTier.value = settings.auto_decompose_max_tier || 3
    vipAutoExtractEssenceEnabled.value = settings.auto_extract_essence_enabled
    vipAutoExtractEssenceMaxTier.value = settings.auto_extract_essence_max_tier || 3
  },
  { immediate: true }
)

watch(vipAutoDecomposeEnabled, (enabled) => {
  if (enabled) vipAutoExtractEssenceEnabled.value = false
})

watch(vipAutoExtractEssenceEnabled, (enabled) => {
  if (enabled) vipAutoDecomposeEnabled.value = false
})

watch(
  quickPotionItems,
  (items) => {
    if (items.some((item) => item.id === selectedQuickPotionId.value)) return
    selectedQuickPotionId.value = items[0]?.id ?? null
  },
  { immediate: true }
)

watch(
  [safeTrainingSkills, () => props.afk?.training_skill_id],
  ([skills, trainingSkillId]) => {
    if (trainingSkillId && skills.some((skill) => skill.id === trainingSkillId)) {
      selectedTrainingSkillId.value = trainingSkillId
      return
    }
    if (skills.some((skill) => skill.id === selectedTrainingSkillId.value)) return
    selectedTrainingSkillId.value = skills[0]?.id ?? null
  },
  { immediate: true }
)

watch(
  [filteredWarehouseItems, inventorySubPage],
  ([items, page]) => {
    if (page !== 'warehouse') return
    if (items.some((item) => item.id === selectedInventoryItemId.value)) return
    selectedInventoryItemId.value = items[0]?.id ?? null
  },
  { immediate: true }
)

const tierOptions = [
  { value: 'all', label: '全部阶级' },
  { value: '0', label: '无阶级' },
  ...Array.from({ length: 17 }, (_, index) => ({
    value: String(index + 1),
    label: tierLabel(index + 1)
  })),
  { value: '18', label: '主宰·神话' }
]
const wanxiangTierOptions = Array.from({ length: 17 }, (_, index) => index + 1)

const petMaxLevel = 200
const treasureMaxLevel = 200
const treasureMaxStage = 20
const cultivationMaxLayer = 81
const growthMaterialTotal = 10000

const detailStats = computed(() => {
  const stats = props.bundle?.stats
  const state = props.bundle?.state
  const { equipment, skills, systems } = resourceCaps.value
  const maxHp = resourceCaps.value.hp
  const maxMp = resourceCaps.value.mp
  const atk = (stats?.atk ?? 0) + equipment.atk + skills.atk + systems.atk
  const mag = (stats?.mag ?? 0) + equipment.mag + skills.mag + systems.mag
  const def = (stats?.def ?? 0) + equipment.def + skills.def + systems.def
  const mdef = (stats?.mdef ?? 0) + equipment.mdef + skills.mdef + systems.mdef
  const currentHp = Math.min(Math.max(state?.hp ?? 0, 0), maxHp)
  const currentMp = Math.min(Math.max(state?.mp ?? 0, 0), maxMp)
  return [
    { label: '生命', value: `${currentHp}/${maxHp}` },
    { label: '魔法', value: `${currentMp}/${maxMp}` },
    { label: '攻击', value: applyPct(atk, equipment.atkPct + skills.atkPct + systems.atkPct) },
    { label: '魔法攻击', value: applyPct(mag, equipment.atkPct + skills.magPct + systems.magPct) },
    { label: '物理防御', value: applyPct(def, equipment.defPct + skills.defPct + systems.defPct) },
    { label: '魔法防御', value: applyPct(mdef, equipment.defPct + skills.defPct + systems.defPct) },
    { label: '幸运', value: (stats?.attrs.dex ?? 0) + equipment.dex + skills.dex + systems.dex },
    { label: '暴击', value: `${equipment.crit + skills.crit + systems.crit}%` },
    { label: '暴伤', value: `${equipment.critDamage + skills.critDamage + systems.critDamage}%` },
    { label: '异常抗性', value: `${skills.controlResist + systems.controlResist}%` },
    { label: '吸血', value: `${equipment.lifeSteal + skills.lifeSteal + systems.lifeSteal}%` },
    { label: '吸蓝', value: `${equipment.manaSteal + skills.manaSteal + systems.manaSteal}%` },
    { label: '重击', value: `${equipment.heavyHit}%` },
    { label: '麻痹', value: `${equipment.paralyze}%` },
    { label: '石化', value: `${equipment.petrify}%` },
    { label: '技能伤害', value: `${equipment.skillDamage}%` },
    { label: '首领增伤', value: `${equipment.bossDamage}%` },
    { label: '伤害减免', value: `${equipment.damageReduce + skills.damageReduce + systems.damageReduce}%` },
    { label: '无视防御', value: `${equipment.ignoreDef + skills.ignoreDef + systems.ignoreDef}%` },
    { label: '真实伤害', value: `${equipment.targetMaxHpTrueDamage}%` },
    { label: '创世一击', value: `${equipment.creationStrike}%` }
  ]
})

const wikiSections = [
  {
    title: '地图推进',
    rows: [
      '凡尘界承接 1-40 级，从青牛城直连迷雾竹林、废弃灵矿、苍茫荒野和平原商道，分支首领会掉落当前阶段极品装备。',
      '修真界承接 41-160 级，天水古城、万妖谷、镇魔残塔和极寒冰原逐步抬高血防压力，破冰前哨站是中转安全区。',
      '飞升界承接 161-300 级，九幽黄泉、狐月神殿、混沌深渊开始考验技能真伤、吸血吸蓝和高阶散件面板。',
      '上古探索区承接 301-500 级，太初远征营地连通太初古矿、洪荒大泽、造化仙门、涅槃火域和鸿蒙星海。',
      '鸿蒙星海末端的星际观测台为无怪物节点，第一次进入即可开启万象铸体。',
      '移动、探索和挂机均有极低概率触发奇遇；角色达到 500 级后奇遇关闭。',
      '虚空要塞可挑战万古渊魔，按增强后的 Lv.600 世界 BOSS 模板生成，拥有 50% 穿透，主要追求主宰套装。'
    ]
  },
  {
    title: '战斗与补给',
    rows: [
      '探索当前区域固定消耗 1 点魔法，再遭遇普通怪、精英怪或区域首领；默认只攻击怪物。',
      '每日体力上限 5000 点；体力大于 0 时每次击杀怪物扣 1 点并正常获得奖励，体力为 0 后经验降至 5%、金币降至 2%、装备和材料掉率为 0%。',
      '角色无动作时按分钟恢复生命和魔法，恢复量随当前最大血蓝增长，最多累计 60 分钟，适合安全区休整但不能替代药品。',
      '安全区显示补给商店，可用金币购买生命药剂、魔法药剂、生命精华、魔力精华、小还丹、大还丹和回城卷。',
      '野外挂机会消耗每日体力；体力耗尽后仍可继续挂机，但经验降至 5%、金币降至 2%，装备和材料掉率为 0%。',
      '普通野外和 PK 死亡惩罚分两次独立判定：50% 掉落背包随机 1-3 格物品，10% 掉落身上最多 1 件装备；无尽塔和幻境挑战失败不会掉落装备。'
    ]
  },
  {
    title: '任务与奖励',
    rows: [
      '任务分为新手、主线、支线、日常四类，可以理解为新手引导、阶段目标、可选成长和每日循环。',
      '新手任务奖励以绑定药水、少量金币和少量培养材料为主，优先帮你站稳前 20 级。',
      '主线任务主要推动等级目标，支线任务引导挂机、强化、行会等玩法，日常任务提供稳定小补给。',
      '日常和行会任务是稳定材料来源，卡图时先检查任务、装备强化、宠物、法宝和境界是否落后。'
    ]
  },
  {
    title: '装备与背包',
    rows: [
      '装备按 1-17 阶推进，每阶只保留一套装备；T3、T6、T9、T12、T15 拥有套装效果，其余等阶为高面板散件。',
      '野外掉落统一三步判定：50% 触发装备掉落，再按 N-2/N-1/N/N+1 的 50%/30%/19%/1% 权重抽阶，最后按防具 50%、首饰 49%、武器 1% 抽部位。',
      '背包和仓库的阶级筛选包含“主宰·神话”，可筛出主宰套、血色幽影、噬星镯和小太子奶。',
      '装备与技能页的装备子页只展示当前穿戴装备；没有穿戴装备时显示为“无”，具体属性在角色信息和物品详情中查看。',
      '背包筛选阶级后点击一键拆解，会拆解当前筛选范围内的未绑定装备；拆装备主要获得炼器石，少量高阶装备可能额外获得鸿蒙石。',
      '技能页的“拆多余书”会拆解已经学会、非本职业或无法对应技能的技能书，返还技能书残页、少量金币，高级书额外返还悟性丹。',
      '强化 +1 到 +10 失败不碎；+11 到 +15 失败降级；+16 到 +20 失败碎装，武器 +11 起每级额外提供少量吸血。',
      '寄售市场是全服公开列表：你上架的未绑定物品别人刷新寄售即可看到，你刷新也能看到别人上架的物品；自己的上架会标记为“我的寄售”。',
      '寄售可选择金币或元宝标价，上架前先确认物品未绑定且仍在背包。'
    ]
  },
  {
    title: '技能与界限突破',
    rows: [
      '技能按主动和被动区分：主动技能在扫荡和 PK 中自动释放并消耗魔法，被动技能学习后常驻加成。',
      '主动技能详情里可以关闭“自动释放”，用于节省魔法；关闭后扫荡和 PK 不会自动使用该技能，手动施法仍可按需使用。',
      '技能不是只靠等级学习：技能 1-4 可在天水古城的天水书院购买，技能 5-6 可在虚空要塞的虚空市集购买；技能 7-8 来自塔顶封印·怨魂聚合体，技能 9-10 来自判官殿·阎罗判官。',
      '三职业 9-10 技能书为终极稀有掉落，阎罗判官击杀后整体掉率为 0.01%。',
      '技能上限 100 级；1-99 级每级提升 5% 主动伤害/治疗，100 级获得满级质变，主动技能魔法消耗统一降为 1。',
      '技能研修在混沌庇护所的虚境进行，每 5 秒扣 1 万金币，指定已学习技能获得 5 点技能经验；主动释放也会提升熟练度。',
      '界限突破包含法宝、境界、宠物和万象铸体：击杀狂暴猪王开启法宝，击杀尸傀监工开启宠物，击杀镇界石魔开启境界。',
      '万象铸体通过装备提取灵韵升级，首次进入星际观测台开启；Lv.500 后开始出现失败率，Lv.1000 获得完整肉身属性。'
    ]
  },
  {
    title: '挑战副本',
    rows: [
      '探索秘境在各安全区都可以挑战，最高 40 层，奖励按现有秘境奖励翻倍，失败不掉装备。',
      '无尽塔位于天水古城，每 1 小时重置一次：每次都从第 1 层重新挑战，最高 100 层；第 100 层为当前版本边界，失败不掉装备。',
      '世界首领位于虚空要塞，胜利后 4 小时刷新，击杀必定获得 1 件主宰套装随机部件，主宰武器掉率最低。',
      '挑战打不过时先回主线地图补等级、强化装备、升级技能，再检查药品和主动技能自动开关。'
    ]
  },
  {
    title: '元宝与会员',
    rows: [
      '元宝商城投放造化丹和 30 天会员资格；造化丹可恢复 100% 生命和魔法。',
      '元宝只通过指定装备分解自然获得：镇界石魔 1% 掉荣耀，冰原主宰 1% 掉传承，二者分解分别得 1/2 元宝；主宰部件/武器分解额外获得 10/100 元宝。',
      '会员拥有自动用药、低阶装备自动拆解或自动提取灵韵功能，不提供额外战斗属性。',
      '会员可设置生命和魔法低于指定百分比时自动使用哪种药品，战斗中按阈值消耗背包药品。',
      '自动拆解和自动提取灵韵只能任选一个；提取会把达到阈值阶级以内的掉落装备直接转为灵韵。'
    ]
  },
  {
    title: 'PK 与安全',
    rows: [
      'PK 默认关闭；开启 PK 模式后，可以手动攻击同屏玩家，但“探索当前区域”只会攻击怪物，不会自动攻击玩家。',
      '安全区和禁 PK 地图永远不能攻击玩家，未开启 PK 时普通探索只打怪。',
      'PK 会消耗生命、魔法和药品，主动技能自动释放越多，魔法消耗越快。',
      '打不过同屏目标时可以先关闭 PK，或关闭高耗蓝主动技能，回到主线地图继续发育。'
    ]
  },
  {
    title: '行会玩法',
    rows: [
      '玩家可以花费 1000 万金币自行创建行会，也可以申请加入其他玩家行会。',
      '加入行会后每天可完成巡猎、补给建设和首领演武，任务会增加个人贡献、行会建设值和当日行会目标进度。',
      '行会建设值会推动行会等级成长，等级上限 20 级，人数上限 10 人。',
      '行会福利每天按当前行会等级领取一次，所有有效无阶材料从 1 级起发放，行会每升一级数量 +1。'
    ]
  }
]

const aboutSections = [
  {
    title: '技术栈',
    rows: [
      '后端：Rust workspace、Axum HTTP、标准 WebSocket JSON、PostgreSQL、sqlx、Tokio。',
      '前端：Vue 3、TypeScript、Pinia、Vite、PWA、lucide 图标。',
      '部署：Docker、Docker Compose、Caddy 反代、数据库迁移服务、备份/恢复脚本。',
      '移动端：Android WebView 外壳骨架。'
    ]
  },
  {
    title: '免责声明',
    rows: [
      '本项目为文字游戏与技术演示用途，当前数值、玩法、掉落和运营配置仍可能继续调整。',
      '请勿将测试环境账号、虚拟道具、元宝或游戏内数据视为现实资产或投资凭证。',
      '部署到公网前应自行完成账号安全、数据备份、隐私合规、日志审计和压力测试。',
      '第三方框架、图标、运行时和数据库组件遵循各自开源许可证。'
    ]
  }
]

const featureCards = [
  { icon: Sword, title: '区域探索', text: '随机遭遇当前区域敌人，多回合自动结算经验、金币和状态。' },
  { icon: Backpack, title: '掉落入包', text: '探索胜利后有概率获得装备和材料，补给药剂主要由商店购买。' },
  { icon: Shield, title: '装备变强', text: '装备等阶、强化和特殊属性会共同影响战斗与掉落倾向。' },
  { icon: BookOpen, title: '技能成长', text: '主动技能可开关自动释放，被动技能常驻加成。' },
  { icon: Sparkles, title: '世界目标', text: '活动、行会、排行榜和世界首领挑战由数据库内容驱动。' }
]

const shopItems = [
  { templateId: 'potion_small', name: '生命药剂', quantity: 10, unitPrice: 1000, desc: '生命 +1000' },
  { templateId: 'potion_mana', name: '魔法药剂', quantity: 10, unitPrice: 1000, desc: '魔法 +1000' },
  { templateId: 'potion_large', name: '生命精华', quantity: 10, unitPrice: 10000, desc: '生命 +5000' },
  { templateId: 'potion_mana_large', name: '魔力精华', quantity: 10, unitPrice: 10000, desc: '魔法 +1000' },
  { templateId: 'potion_sun', name: '小还丹', quantity: 10, unitPrice: 100000, desc: '生命 +10% / 魔法 +10%' },
  { templateId: 'potion_dahuan', name: '大还丹', quantity: 10, unitPrice: 300000, desc: '生命 +10% / 魔法 +10%' },
  { templateId: 'scroll_return', name: '回城卷', quantity: 1, unitPrice: 200, desc: '返回所在界域主城' }
]

const shopQuantities = ref<Record<string, number>>(
  Object.fromEntries(shopItems.map((item) => [item.templateId, item.quantity]))
)

const teleportDestinations = [
  { zone: 'fanchen', room: 'qingniu_city', name: '青牛城' },
  { zone: 'xiuzhen', room: 'tianshui_city', name: '天水古城' },
  { zone: 'xiuzhen', room: 'ice_outpost', name: '破冰前哨站' },
  { zone: 'feisheng', room: 'void_fortress', name: '虚空要塞' },
  { zone: 'feisheng', room: 'chaos_shelter', name: '混沌庇护所' },
  { zone: 'ancient_secret', room: 'taichu_camp', name: '太初远征营地' }
]

const yuanbaoItems = [
  { templateId: 'potion_full', name: '造化丹', quantity: 1, price: 10, desc: '恢复 100% 生命和魔法' },
  { templateId: 'vip_month_card', name: '会员资格', quantity: 1, price: 300, desc: '购买后立即激活 30 天会员' },
  { templateId: 'belt_taizi_small', name: '小太子奶', quantity: 1, price: 1500, desc: '手镯：双攻 +1200，生命/魔法 +1400，30 秒原地复活' }
]

const setEffectDefinitions: Record<string, SetEffectDefinition[]> = {
  qingyun: [
    { pieces: 3, title: '青云根基', text: '最大生命值与最大魔法值提升 15%。' },
    { pieces: 5, title: '基础发育', text: '挂机经验获取 +20%，金币获取 +20%。' }
  ],
  pureyang: [
    { pieces: 3, title: '通用续航', text: '每次击杀怪物后恢复自身 20% 生命值与魔法值。' },
    { pieces: 5, title: '高效闭关', text: '挂机经验加成提升至 +40%。' }
  ],
  jiuxiao: [
    { pieces: 3, title: '技能真伤', text: '所有主动技能伤害倍率提升 20%。' },
    { pieces: 6, title: '九霄雷引', text: '所有主动技能等级 +2，造成伤害时附带自身最大魔法值 2% 的真实伤害。' }
  ],
  chaos: [
    { pieces: 3, title: '混沌壁垒', text: '物理与魔法抗性全局提升 30%。' },
    { pieces: 5, title: '混沌虚影', text: '受到的最终伤害降低 30%，并获得 30% 负面状态免疫。' }
  ],
  zaohua: [
    { pieces: 3, title: '造化破防', text: '所有攻击与技能无视目标 50% 防御力。' },
    { pieces: 5, title: '造化夺天', text: '每次造成伤害时，基于伤害吸取 10% 的生命和法力。' }
  ],
  dominator: [
    { pieces: 2, title: '主宰之域', text: '最大生命与魔法上限翻倍，获得 20% 全局伤害吸血和吸蓝。' },
    { pieces: 4, title: '主宰之威', text: '所有伤害转化为主宰级真实伤害，所有技能最终伤害翻倍。' },
    { pieces: 6, title: '主宰之躯', text: '免疫一切负面状态，受到暴击伤害减少 100%。' },
    { pieces: 8, title: '言出法随', text: '挂机无视疲劳限制，探索无 2 秒间隔，战斗有 15% 概率触发天罚。' }
  ]
}

const hpPotionOptions = [
  { value: 'potion_small', label: '生命药剂' },
  { value: 'potion_large', label: '生命精华' },
  { value: 'potion_sun', label: '小还丹' },
  { value: 'potion_dahuan', label: '大还丹' },
  { value: 'potion_full', label: '造化丹' }
]

const mpPotionOptions = [
  { value: 'potion_mana', label: '魔法药剂' },
  { value: 'potion_mana_large', label: '魔力精华' },
  { value: 'potion_sun', label: '小还丹' },
  { value: 'potion_dahuan', label: '大还丹' },
  { value: 'potion_full', label: '造化丹' }
]

const guildLevelThresholds = [
  0, 1000, 2500, 4500, 7000, 10000, 14000, 19000, 26000, 36500, 50000, 68000, 90000, 118000,
  152000, 193000, 242000, 300000, 368000, 446000
]

const equipmentSlots = [
  { key: 'weapon', label: '武器' },
  { key: 'chest', label: '衣服' },
  { key: 'head', label: '头盔' },
  { key: 'neck', label: '项链' },
  { key: 'bracelet_left', label: '左手镯' },
  { key: 'bracelet_right', label: '右手镯' },
  { key: 'ring_left', label: '左戒指' },
  { key: 'ring_right', label: '右戒指' },
  { key: 'waist', label: '腰带' },
  { key: 'feet', label: '鞋子' }
]

const equipmentSlotRows = computed(() =>
  equipmentSlots.map((slot) => ({
    ...slot,
    item: equippedItems.value.find((item) => (item.equipped_slot ?? item.template_slot) === slot.key) ?? null
  }))
)

function questCategoryLabel(category: string) {
  if (category === 'newbie') return '新手'
  if (category === 'main') return '主线'
  if (category === 'side') return '支线'
  if (category === 'daily') return '日常'
  return category
}

function questStatusLabel(status: string) {
  if (status === 'claimed') return '已领'
  if (status === 'completed') return '可领'
  if (status === 'progress') return '进行中'
  return '未完成'
}

function questRewardLabel(quest: PlayerQuest) {
  const parts: string[] = []
  const gold = numericStat(quest.rewards.gold)
  if (gold > 0) parts.push(`${gold} 金币`)
  const items = Array.isArray(quest.rewards.items) ? quest.rewards.items : []
  for (const item of items.slice(0, 2)) {
    if (!item || typeof item !== 'object') continue
    const templateId = 'template_id' in item ? String(item.template_id) : ''
    const quantity = 'quantity' in item ? Number(item.quantity) : 1
    if (templateId) parts.push(`${rewardItemName(templateId)} x${Number.isFinite(quantity) ? quantity : 1}`)
  }
  return parts.length ? parts.join(' · ') : '少量补给'
}

function rewardItemName(templateId: string) {
  if (templateId === 'potion_small') return '生命药剂'
  if (templateId === 'potion_mana') return '魔法药剂'
  if (templateId === 'potion_large') return '生命精华'
  if (templateId === 'potion_mana_large') return '魔力精华'
  if (templateId === 'potion_sun') return '小还丹'
  if (templateId === 'potion_dahuan') return '大还丹'
  if (templateId === 'potion_big_taizi') return '护脉丹'
  if (templateId === 'potion_jiuzhuan') return '九转还魂丹'
  if (templateId === 'potion_full') return '造化丹'
  if (templateId === 'scroll_return') return '回城卷'
  if (templateId === 'pet_food') return '灵兽粮'
  if (templateId === 'treasure_shard') return '法宝碎片'
  if (templateId === 'cultivation_pill') return '修炼丹'
  if (templateId === 'pill_insight') return '悟性丹'
  if (templateId === 'skill_page') return '技能书残页'
  if (templateId === 'guild_merit_token') return '行会功勋令'
  if (templateId === 'stone_refine') return '炼器石'
  if (templateId === 'stone_hongmeng') return '鸿蒙石'
  return templateId
}

function saveVipPotionSettings() {
  emit('updateVipPotionSettings', {
    hp_enabled: vipHpEnabled.value,
    hp_threshold_pct: clampPercent(vipHpThreshold.value),
    hp_template_id: vipHpTemplate.value,
    mp_enabled: vipMpEnabled.value,
    mp_threshold_pct: clampPercent(vipMpThreshold.value),
    mp_template_id: vipMpTemplate.value,
    auto_decompose_enabled: vipAutoDecomposeEnabled.value,
    auto_decompose_max_tier: Math.min(17, Math.max(0, Math.floor(Number(vipAutoDecomposeMaxTier.value) || 0))),
    auto_extract_essence_enabled: vipAutoExtractEssenceEnabled.value,
    auto_extract_essence_max_tier: Math.min(17, Math.max(0, Math.floor(Number(vipAutoExtractEssenceMaxTier.value) || 0)))
  })
}

function clampPercent(value: number) {
  return Math.min(99, Math.max(1, Math.floor(Number(value) || 1)))
}

function petBonusLabel() {
  if (!systemUnlocked('pet')) return unlockRequirement('pet')
  const pet = primaryPet.value
  if (!pet) return '未获得宠物'
  const level = Math.min(petMaxLevel, Math.max(1, pet.level))
  const atk = scaledGrowth(level, 2600, petMaxLevel)
  const mag = scaledGrowth(level, 2300, petMaxLevel)
  const hp = scaledGrowth(level, 16000, petMaxLevel)
  const def = scaledGrowth(level, 600, petMaxLevel)
  const mdef = scaledGrowth(level, 600, petMaxLevel)
  return `攻击 +${atk} · 魔法攻击 +${mag} · 生命 +${hp} · 防御 +${def} · 魔防 +${mdef}`
}

function treasureBonusLabel() {
  if (!systemUnlocked('treasure')) return unlockRequirement('treasure')
  const treasure = primaryTreasure.value
  if (!treasure) return '未获得法宝'
  const level = Math.min(treasureMaxLevel, Math.max(1, treasure.level))
  const stage = Math.min(treasureMaxStage, Math.max(1, treasure.stage))
  return `攻击 +${level * 10 + stage * 40} · 魔法攻击 +${level * 10 + stage * 40} · 生命 +${level * 60 + stage * 200} · 暴击 +${Math.floor(level / 20) + stage * 2}%`
}

function cultivationBonusLabel() {
  if (!systemUnlocked('cultivation')) return unlockRequirement('cultivation')
  const layer = cultivationTotalLayer()
  if (!layer) return '未开启修炼'
  return `攻击 +${scaledGrowth(layer, 3200, cultivationMaxLayer)} · 魔法攻击 +${scaledGrowth(layer, 3200, cultivationMaxLayer)} · 生命 +${scaledGrowth(layer, 32000, cultivationMaxLayer)} · 魔法 +${scaledGrowth(layer, 32000, cultivationMaxLayer)} · 双防 +${scaledGrowth(layer, 2000, cultivationMaxLayer)}`
}

function wanxiangBonusLabel() {
  const body = wanxiang.value
  if (!body) return '万化神炉未点燃'
  return `攻击 +${body.atk} · 魔法攻击 +${body.mag} · 生命 +${body.hp} · 魔法 +${body.mp} · 双防 +${body.def} · 吸血/吸魔 +${body.life_steal_pct}% · 减伤 +${body.damage_reduce_pct}%`
}

function wanxiangCostLabel() {
  const body = wanxiang.value
  if (!body) return '灵韵 0/0'
  if (body.level >= 1000) return `已满级 · 灵韵 ${body.essence}`
  return `灵韵 ${body.essence}/${body.next_cost} · 失败率 ${body.fail_pct}%`
}

function handleWanxiangExtract() {
  if (!systemUnlocked('wanxiang')) return
  const tier = Math.min(17, Math.max(1, Math.floor(Number(wanxiangExtractTier.value) || 1)))
  wanxiangExtractTier.value = tier
  emit('extractWanxiangEssence', tier)
}

function promptCreateGuild() {
  const name = window.prompt('请输入行会名称（2-16 个字符，创建消耗 1000 万金币）')
  if (!name?.trim()) return
  emit('createGuild', name.trim())
}

function promptApplyGuild(guildId: number) {
  const message = window.prompt('请输入申请留言（可留空）', '')
  if (message === null) return
  emit('applyGuild', guildId, message.trim())
}

function promptDonateGuild() {
  const raw = window.prompt('请输入捐献金币数量（1 万金币 = 1 点建设和贡献）', '10000')
  if (!raw) return
  const gold = Math.floor(Number(raw))
  if (!Number.isFinite(gold) || gold <= 0) return
  emit('donateGuild', gold)
}

function shopBuyQuantity(item: { templateId: string; quantity: number }) {
  return Math.min(999, Math.max(1, Math.floor(Number(shopQuantities.value[item.templateId]) || item.quantity)))
}

function shopItemTotal(item: { templateId: string; quantity: number; unitPrice: number }) {
  return shopBuyQuantity(item) * item.unitPrice
}

function buyShopItem(item: { templateId: string; quantity: number }) {
  emit('buyShopItem', item.templateId, shopBuyQuantity(item))
}

function guildRoleLabel(role: string | null | undefined) {
  if (role === 'leader') return '会长'
  if (role === 'elder') return '长老'
  if (role === 'member') return '成员'
  return '未加入'
}

function guildNextLevelLabel(guild: PlayerGuild | null) {
  if (!guild) return '未加入行会'
  if (guild.level >= 20) return '已满级'
  const next = guildLevelThresholds[guild.level] ?? guildLevelThresholds[guildLevelThresholds.length - 1]
  const remaining = Math.max(0, next - guild.funds)
  return `${guild.funds}/${next} · 距 ${guild.level + 1} 级还需 ${remaining}`
}

function guildProgressPct(guild: PlayerGuild | null) {
  if (!guild) return 0
  if (guild.level >= 20) return 100
  const current = guildLevelThresholds[guild.level - 1] ?? 0
  const next = guildLevelThresholds[guild.level] ?? guildLevelThresholds[guildLevelThresholds.length - 1]
  return Math.min(100, Math.max(0, Math.round(((guild.funds - current) / Math.max(1, next - current)) * 100)))
}

function guildBenefitPreview(level: number | undefined) {
  const value = Math.min(10, Math.max(1, Math.floor(Number(level) || 1)))
  return `灵兽粮、法宝碎片、修炼丹、悟性丹、技能书残页、行会功勋令、炼器石、鸿蒙石各 x${value}`
}

function guildProjectButtonLabel(project: { completed_today: boolean; completed: boolean; available: boolean }) {
  if (!project.available) return '未开放'
  if (project.completed_today) return '今日已做'
  if (project.completed) return '协助'
  return '完成'
}

function guildProjectDisabled(project: { completed_today: boolean; available: boolean }) {
  return !props.authenticated || !project.available || project.completed_today
}

function changePkEnabled(event: Event) {
  const checked = (event.target as HTMLInputElement).checked
  emit('updatePkSettings', checked, checked)
}

function canEquip(item: InventoryItem) {
  return item.location === 'bag' && Boolean(item.template_slot)
}

function canUse(item: InventoryItem) {
  return item.location === 'bag' && item.kind === 'consumable'
}

function isQuickPotion(item: InventoryItem) {
  if (!canUse(item)) return false
  return (
    Boolean(item.stats.full_restore) ||
    numericStat(item.stats.hp) > 0 ||
    numericStat(item.stats.mp) > 0 ||
    numericStat(item.stats.hp_pct) > 0 ||
    numericStat(item.stats.mp_pct) > 0
  )
}

function quickPotionEffect(item: InventoryItem) {
  if (item.stats.full_restore) return '生命/魔法 100%'
  const parts: string[] = []
  const hp = numericStat(item.stats.hp)
  const mp = numericStat(item.stats.mp)
  const hpPct = numericStat(item.stats.hp_pct)
  const mpPct = numericStat(item.stats.mp_pct)
  if (hp > 0) parts.push(`生命 +${hp}`)
  if (mp > 0) parts.push(`魔法 +${mp}`)
  if (hpPct > 0) parts.push(`生命 +${hpPct}%`)
  if (mpPct > 0) parts.push(`魔法 +${mpPct}%`)
  return parts.join(' / ') || '可使用'
}

function selectQuickPotion(itemId: number) {
  selectedQuickPotionId.value = itemId
}

function useSelectedQuickPotion() {
  if (!selectedQuickPotion.value) return
  emit('useItem', selectedQuickPotion.value.id)
}

function selectEquipmentItem(item: InventoryItem | null) {
  if (!item) return
  selectedInventoryItemId.value = item.id
}

function canDecompose(item: InventoryItem) {
  return ['bag', 'warehouse'].includes(item.location) && (Boolean(item.template_slot) || item.stats.decompose_only === true) && !item.bind
}

function canDecomposeMisc(item: InventoryItem) {
  return ['bag', 'warehouse'].includes(item.location) && !item.template_slot && ['book', 'material', 'consumable'].includes(item.kind)
}

function suggestedTradePrice(item: InventoryItem) {
  return Math.max(1, item.price || 1)
}

function decomposableIds() {
  return selectedDecomposeItems.value.map((item) => item.id)
}

function redundantSkillBookIds() {
  return redundantSkillBookItems.value.map((item) => item.id)
}

function skillBookTarget(item: InventoryItem) {
  const skill = item.stats.skill
  return typeof skill === 'string' ? skill : ''
}

function isRedundantSkillBook(item: InventoryItem) {
  if (item.kind !== 'book') return false
  const target = skillBookTarget(item)
  if (!target) return true
  const skill = props.skills.find((entry) => entry.id === target)
  if (!skill) return true
  if (skill.class !== 'all' && skill.class !== characterClass.value) return true
  return skill.learned
}

function itemTier(item: InventoryItem) {
  if (isMythicDominatorTierItem(item)) return 18
  const tier = Number(item.stats.tier ?? 0)
  return Number.isFinite(tier) ? Math.max(0, tier) : 0
}

function tierFilterMatches(item: InventoryItem, filter: string) {
  const targetTier = Number(filter)
  if (!Number.isFinite(targetTier)) return false
  return itemTier(item) === targetTier
}

function isMythicDominatorTierItem(item: InventoryItem) {
  return isDominatorItem(item)
    || ['ring_blood_shadow', 'bracelet_star_devourer', 'belt_taizi_small'].includes(item.template_id)
}

function aggregateEquipmentStats() {
  const totals = {
    atk: 0,
    def: 0,
    mag: 0,
    mdef: 0,
    dex: 0,
    hp: 0,
    mp: 0,
    crit: 0,
    lifeSteal: 0,
    manaSteal: 0,
    heavyHit: 0,
    paralyze: 0,
    petrify: 0,
    atkPct: 0,
    defPct: 0,
    hpPct: 0,
    mpPct: 0,
    skillDamage: 0,
    bossDamage: 0,
    critDamage: 0,
    battleEndRestore: 0,
    afkBaseReward: 0,
    afkExp: 0,
    afkGold: 0,
    afkDropQuality: 0,
    afkKill: 0,
    afkExtraMinutes: 0,
    afkOfflineReward: 0,
    allSkillBonus: 0,
    normalMobExecute: 0,
    damageReduce: 0,
    paralyzeResist: 0,
    petrifyResist: 0,
    ignoreDef: 0,
    guaranteedHit: 0,
    targetMaxHpTrueDamage: 0,
    creationStrike: 0,
    creationStrikeDamage: 0,
    creationStrikeFullRestore: 0,
    controlImmune: 0
  }
  const sets: Record<string, number> = {}
  for (const item of equippedItems.value) {
    totals.atk += numericStat(item.stats.atk)
    totals.def += numericStat(item.stats.def)
    totals.mag += numericStat(item.stats.mag)
    totals.mdef += numericStat(item.stats.mdef)
    totals.dex += numericStat(item.stats.luck) + numericStat(item.stats.dex)
    totals.hp += numericStat(item.stats.hp)
    totals.mp += numericStat(item.stats.mp)
    totals.crit += numericStat(item.stats.crit_pct) + numericStat(item.stats.crit)
    totals.lifeSteal += numericStat(item.stats.life_steal_pct)
    totals.manaSteal += numericStat(item.stats.mana_steal_pct)
    totals.heavyHit += numericStat(item.stats.heavy_hit_pct)
    totals.paralyze += numericStat(item.stats.paralyze_pct)
    totals.petrify += numericStat(item.stats.petrify_pct)
    totals.atkPct += numericStat(item.stats.atk_pct)
    totals.defPct += numericStat(item.stats.def_pct)
    totals.hpPct += numericStat(item.stats.hp_pct)
    totals.mpPct += numericStat(item.stats.mp_pct)
    totals.skillDamage += numericStat(item.stats.skill_damage_pct)
    totals.bossDamage += numericStat(item.stats.boss_damage_pct)
    totals.critDamage += numericStat(item.stats.crit_damage_pct)
    totals.battleEndRestore += numericStat(item.stats.battle_end_restore_pct)
    totals.afkBaseReward += numericStat(item.stats.afk_base_reward_pct)
    totals.afkExp += numericStat(item.stats.afk_exp_pct)
    totals.afkGold += numericStat(item.stats.afk_gold_pct)
    totals.afkDropQuality += numericStat(item.stats.afk_drop_quality_pct)
    totals.afkKill += numericStat(item.stats.afk_kill_pct)
    totals.afkExtraMinutes += numericStat(item.stats.afk_extra_minutes)
    totals.afkOfflineReward += numericStat(item.stats.afk_offline_reward_pct)
    totals.allSkillBonus += numericStat(item.stats.all_skill_bonus)
    totals.normalMobExecute += numericStat(item.stats.normal_mob_execute_pct)
    totals.damageReduce += numericStat(item.stats.damage_reduce_pct)
    totals.paralyzeResist += numericStat(item.stats.paralyze_resist_pct)
    totals.petrifyResist += numericStat(item.stats.petrify_resist_pct)
    totals.ignoreDef += numericStat(item.stats.ignore_def_pct)
    totals.guaranteedHit += numericStat(item.stats.guaranteed_hit_pct)
    totals.targetMaxHpTrueDamage += numericStat(item.stats.target_max_hp_true_damage_pct)
    totals.creationStrike += numericStat(item.stats.creation_strike_pct)
    totals.creationStrikeDamage += numericStat(item.stats.creation_strike_damage_pct)
    if (item.stats.creation_strike_full_restore === true) totals.creationStrikeFullRestore = 1
    if (item.stats.control_immune === true) totals.controlImmune = 1
    const set = stringStat(item.stats.set)
    if (set) sets[set] = (sets[set] ?? 0) + 1
  }
  if ((sets.qingyun ?? 0) >= 3) {
    totals.hpPct += 15
    totals.mpPct += 15
  }
  if ((sets.qingyun ?? 0) >= 5) {
    totals.afkExp += 20
    totals.afkGold += 20
  }
  if ((sets.pureyang ?? 0) >= 3) totals.battleEndRestore += 20
  if ((sets.pureyang ?? 0) >= 5) totals.afkExp += 40
  if ((sets.jiuxiao ?? 0) >= 3) totals.skillDamage += 20
  if ((sets.jiuxiao ?? 0) >= 6) {
    totals.allSkillBonus += 2
    totals.targetMaxHpTrueDamage += 2
  }
  if ((sets.chaos ?? 0) >= 3) totals.defPct += 30
  if ((sets.chaos ?? 0) >= 5) {
    totals.damageReduce += 30
    totals.controlImmune = 1
  }
  if ((sets.zaohua ?? 0) >= 3) totals.ignoreDef += 50
  if ((sets.zaohua ?? 0) >= 5) {
    totals.lifeSteal += 10
    totals.manaSteal += 10
  }
  if ((sets.dominator ?? 0) >= 2) {
    totals.hpPct += 100
    totals.mpPct += 100
    totals.lifeSteal += 20
    totals.manaSteal += 20
  }
  if ((sets.dominator ?? 0) >= 4) {
    totals.skillDamage += 100
    totals.ignoreDef += 95
    totals.guaranteedHit += 100
  }
  if ((sets.dominator ?? 0) >= 6) totals.controlImmune = 1
  if ((sets.dominator ?? 0) >= 8) {
    totals.creationStrike += 15
    totals.creationStrikeDamage += 1000
    totals.creationStrikeFullRestore = 1
  }
  return totals
}

function isDominatorItem(item: InventoryItem) {
  return stringStat(item.stats.set) === 'dominator' || item.template_id.startsWith('dominator_')
}

function aggregateSkillStats() {
  const totals = {
    atk: 0,
    def: 0,
    mag: 0,
    mdef: 0,
    dex: 0,
    hp: 0,
    mp: 0,
    crit: 0,
    critDamage: 0,
    controlResist: 0,
    atkPct: 0,
    magPct: 0,
    defPct: 0,
    hpPct: 0,
    mpPct: 0,
    lifeSteal: 0,
    manaSteal: 0,
    damageReduce: 0,
    ignoreDef: 0,
    guaranteedHit: 0,
    damageDeepen: 0
  }
  for (const skill of learnedSkills.value) {
    const level = skill.level ?? 1
    totals.atk += numericStat(skill.config.atk_bonus) * level
    totals.def += numericStat(skill.config.def_bonus) * level
    totals.mag += numericStat(skill.config.mag_bonus) * level
    totals.mdef += numericStat(skill.config.mdef_bonus) * level
    totals.dex += (numericStat(skill.config.dex_bonus) + numericStat(skill.config.luck_bonus)) * level
    totals.dex += Math.floor((numericStat(skill.config.luck_bonus_per_100) * level) / 100)
    totals.hp += numericStat(skill.config.hp_bonus) * level
    totals.mp += numericStat(skill.config.mp_bonus) * level
    totals.crit += (numericStat(skill.config.crit_bonus) + numericStat(skill.config.crit_pct_bonus)) * level
    totals.critDamage += numericStat(skill.config.crit_damage_pct_bonus) * level
    totals.controlResist += Math.floor((numericStat(skill.config.control_resist_pct_per_100) * level) / 100)
    totals.atkPct += Math.floor((numericStat(skill.config.atk_pct_per_100) * level) / 100)
    totals.magPct += Math.floor((numericStat(skill.config.mag_pct_per_100) * level) / 100)
    totals.defPct += Math.floor((numericStat(skill.config.def_pct_per_100) * level) / 100)
    totals.hpPct += Math.floor((numericStat(skill.config.hp_pct_per_100) * level) / 100)
    totals.mpPct += Math.floor((numericStat(skill.config.mp_pct_per_100) * level) / 100)
    totals.lifeSteal += numericStat(skill.config.life_steal_base_pct)
      + Math.floor((numericStat(skill.config.life_steal_pct_per_100) * level) / 100)
    totals.manaSteal += numericStat(skill.config.mana_steal_base_pct)
      + Math.floor((numericStat(skill.config.mana_steal_pct_per_100) * level) / 100)
    totals.damageReduce += Math.floor((numericStat(skill.config.damage_reduce_pct_per_100) * level) / 100)
    totals.ignoreDef += Math.floor((numericStat(skill.config.ignore_def_pct_per_100) * level) / 100)
    totals.guaranteedHit += Math.floor((numericStat(skill.config.guaranteed_hit_pct_per_100) * level) / 100)
    totals.damageDeepen += Math.floor((numericStat(skill.config.damage_deepen_pct_per_100) * level) / 100)
  }
  return totals
}

function aggregateSystemStats() {
  const totals = {
    atk: 0,
    def: 0,
    mag: 0,
    mdef: 0,
    dex: 0,
    hp: 0,
    mp: 0,
    crit: 0,
    critDamage: 0,
    controlResist: 0,
    lifeSteal: 0,
    manaSteal: 0,
    damageReduce: 0,
    atkPct: 0,
    magPct: 0,
    defPct: 0,
    hpPct: 0,
    mpPct: 0,
    ignoreDef: 0
  }
  const pet = primaryPet.value
  if (pet?.fighting && systemUnlocked('pet')) {
    const level = Math.min(petMaxLevel, Math.max(1, pet.level))
    totals.atk += scaledGrowth(level, 2600, petMaxLevel)
    totals.mag += scaledGrowth(level, 2300, petMaxLevel)
    totals.hp += scaledGrowth(level, 16000, petMaxLevel)
    totals.def += scaledGrowth(level, 600, petMaxLevel)
    totals.mdef += scaledGrowth(level, 600, petMaxLevel)
  }
  for (const treasure of props.overview?.systems.treasures ?? []) {
    if (!systemUnlocked('treasure')) continue
    if (!treasure.equipped) continue
    const level = Math.min(treasureMaxLevel, Math.max(1, treasure.level))
    const stage = Math.min(treasureMaxStage, Math.max(1, treasure.stage))
    totals.atk += level * 10 + stage * 40
    totals.mag += level * 10 + stage * 40
    totals.hp += level * 60 + stage * 200
    totals.crit += Math.floor(level / 20) + stage * 2
  }
  const layer = systemUnlocked('cultivation') ? cultivationTotalLayer() : 0
  if (layer > 0) {
    totals.atk += scaledGrowth(layer, 3200, cultivationMaxLayer)
    totals.mag += scaledGrowth(layer, 3200, cultivationMaxLayer)
    totals.hp += scaledGrowth(layer, 32000, cultivationMaxLayer)
    totals.mp += scaledGrowth(layer, 32000, cultivationMaxLayer)
    totals.def += scaledGrowth(layer, 2000, cultivationMaxLayer)
    totals.mdef += scaledGrowth(layer, 2000, cultivationMaxLayer)
  }
  if (wanxiang.value) {
    totals.atk += wanxiang.value.atk
    totals.mag += wanxiang.value.mag
    totals.hp += wanxiang.value.hp
    totals.mp += wanxiang.value.mp
    totals.def += wanxiang.value.def
    totals.mdef += wanxiang.value.mdef
    totals.lifeSteal += wanxiang.value.life_steal_pct
    totals.manaSteal += wanxiang.value.mana_steal_pct
    totals.damageReduce += wanxiang.value.damage_reduce_pct
  }
  for (const totem of activeGuild.value?.totems ?? []) {
    const level = totem.level ?? 0
    if (totem.kind === 'qiongqi') {
      totals.atk += level * 100
      totals.mag += level * 100
    } else if (totem.kind === 'bifang') {
      totals.crit += Math.floor(level / 5)
      totals.critDamage += level
    } else if (totem.kind === 'chenghuang') {
      totals.hp += level * 500
      totals.mp += level * 500
    } else if (totem.kind === 'xuangui') {
      totals.def += level * 50
      totals.mdef += level * 50
    }
  }
  return totals
}

function systemUnlocked(system: 'pet' | 'treasure' | 'cultivation' | 'wanxiang') {
  const unlocks = props.overview?.systems.unlocks
  if (system === 'pet') return Boolean(unlocks?.pet?.unlocked)
  if (system === 'treasure') return Boolean(unlocks?.treasure?.unlocked)
  if (system === 'cultivation') return Boolean(unlocks?.cultivation?.unlocked)
  return Boolean(unlocks?.wanxiang?.unlocked)
}

function unlockRequirement(system: 'pet' | 'treasure' | 'cultivation' | 'wanxiang') {
  const unlocks = props.overview?.systems.unlocks
  const info = system === 'pet'
    ? unlocks?.pet
    : system === 'treasure'
      ? unlocks?.treasure
      : system === 'cultivation'
        ? unlocks?.cultivation
        : unlocks?.wanxiang
  return info?.unlocked ? '已开启' : `未开启 · 需要${info?.requirement ?? '完成指定挑战'}`
}

function unlockBadge(system: 'pet' | 'treasure' | 'cultivation' | 'wanxiang') {
  return systemUnlocked(system) ? '已开启' : '未开启'
}

function materialQuantity(templateId: string) {
  return bagItems.value
    .filter((item) => item.template_id === templateId)
    .reduce((sum, item) => sum + item.quantity, 0)
}

function petCostLabel() {
  const level = primaryPet.value?.level ?? 1
  const food = growthMaterialCost(level, petMaxLevel)
  const gold = growthGoldCost(level, 15000, 100)
  return `灵兽粮 ${materialQuantity('pet_food')}/${food} · 金币 ${gold}`
}

function treasureCostLabel() {
  const treasure = primaryTreasure.value
  const shards = growthMaterialCost(treasure?.level ?? 1, treasureMaxLevel)
  const gold = growthGoldCost(treasure?.level ?? 1, 10000, 90)
  return `法宝碎片 ${materialQuantity('treasure_shard')}/${shards} · 金币 ${gold}`
}

function cultivationCostLabel() {
  const layer = cultivationTotalLayer()
  const pills = growthMaterialCost(layer || 1, cultivationMaxLayer)
  const gold = growthGoldCost(layer || 1, 100000, 12000)
  return `修炼丹 ${materialQuantity('cultivation_pill')}/${pills} · 金币 ${gold}`
}

function cultivationTotalLayer() {
  const state = cultivation.value
  if (!state) return 0
  const base: Record<string, number> = {
    '淬体': 0,
    '凝神': 9,
    '练气': 18,
    '元婴': 27,
    '登仙': 36,
    '化神': 45,
    '太初': 54,
    '创世': 63,
    '主宰': 72
  }
  return (base[state.realm] ?? 0) + state.layer
}

function growthMaterialCost(currentRank: number, maxRank: number) {
  const max = Math.max(2, Math.floor(maxRank))
  const steps = max - 1
  const current = Math.min(max - 1, Math.max(1, Math.floor(currentRank || 1)))
  const completed = current - 1
  const variable = Math.max(0, growthMaterialTotal - steps)
  const denominator = Math.max(1, steps * steps)
  const previous = Math.floor((variable * completed * completed) / denominator)
  const next = Math.floor((variable * current * current) / denominator)
  return 1 + Math.max(0, next - previous)
}

function growthGoldCost(currentRank: number, base: number, cubeMultiplier: number) {
  const rank = Math.max(1, Math.floor(currentRank || 1))
  return base + rank * rank * rank * cubeMultiplier
}

function scaledGrowth(current: number, total: number, max: number) {
  return Math.floor((total * Math.min(max, Math.max(1, current))) / Math.max(1, max))
}

function numericStat(value: unknown) {
  return typeof value === 'number' && Number.isFinite(value) ? value : 0
}

function stringStat(value: unknown) {
  return typeof value === 'string' ? value : ''
}

function applyPct(value: number, pct: number) {
  return Math.floor((value * (100 + Math.max(-90, Math.min(500, pct)))) / 100)
}

function directionLabel(value: string | number) {
  const labels: Record<string, string> = {
    north: '北',
    south: '南',
    east: '东',
    west: '西'
  }
  return labels[String(value)] ?? String(value)
}

function startSafeAfk() {
  if (isTrainingRoom.value) {
    emit('startAfk', '')
    return
  }
  if (!selectedTrainingSkill.value) return
  emit('startAfk', selectedTrainingSkill.value.id)
}

function teleportDisabled(destination: { zone: string; room: string }) {
  return (
    !props.authenticated ||
    !isSafeRoom.value ||
    (characterState.value?.zone === destination.zone && characterState.value?.room === destination.room)
  )
}

function itemStatParts(item: InventoryItem | null | undefined) {
  return item ? statParts(item.stats) : []
}

function itemSetId(item: InventoryItem | null | undefined) {
  return item ? stringStat(item.stats.set).trim() : ''
}

function equippedSetPieceCount(setId: string) {
  if (!setId) return 0
  return equippedItems.value.filter((item) => itemSetId(item) === setId).length
}

function setProgressLabel(item: InventoryItem | null | undefined) {
  const setId = itemSetId(item)
  if (!setId) return ''
  return `当前穿戴 ${equippedSetPieceCount(setId)} 件`
}

function itemSetEffectRows(item: InventoryItem | null | undefined) {
  const setId = itemSetId(item)
  const pieces = equippedSetPieceCount(setId)
  return (setEffectDefinitions[setId] ?? []).map((effect) => ({
    ...effect,
    active: pieces >= effect.pieces
  }))
}

function equipmentRank(item: InventoryItem) {
  if (!item.template_slot) return rarityLabel(item.rarity)
  const tier = itemTier(item)
  const score = Number(item.stats.score ?? 0)
  const rank = tier > 0 ? tierLabel(tier) : rarityLabel(item.rarity)
  return score > 0 ? `${rank} · 评分 ${score}` : rank
}

function tradeStats(item: TradeConsignment) {
  const parts = statParts(item.stats)
  return parts.join(' / ') || '无属性'
}

function statParts(stats: Record<string, unknown>) {
  return Object.entries(stats)
    .filter(([key, value]) => {
      if (typeof value === 'number') return value !== 0
      if (typeof value === 'boolean') return value
      return ['set', 'special_mechanism', 'special_mechanism_extra'].includes(key) && typeof value === 'string' && value.trim().length > 0
    })
    .map(([key, value]) => {
      if (typeof value === 'boolean') {
        return `${statLabel(key)} ${value ? '是' : '否'}`
      }
      if (typeof value === 'string') {
        return `${statLabel(key)} ${key === 'set' ? setLabel(value) : value}`
      }
      const suffix = statSuffix(key)
      return `${statLabel(key)} +${value}${suffix}`
    })
}

function statSuffix(key: string) {
  if (key.endsWith('_pct') || key.endsWith('_base_pct') || key.endsWith('_pct_per_100')) return '%'
  if (key === 'max_percent_affix_cap') return '%'
  if (key.endsWith('_seconds')) return '秒'
  return ''
}

function tradeCurrencyLabel(item: TradeConsignment) {
  return item.price_currency === 'gold' ? '金币' : '元宝'
}

function tradeTaxLabel(item: TradeConsignment) {
  const tax = item.price_currency === 'gold' ? item.trade_tax_gold : item.trade_tax_yuanbao
  return `${tax} ${tradeCurrencyLabel(item)}`
}

function tradeSellerReceives(item: TradeConsignment) {
  const receives = item.price_currency === 'gold' ? item.seller_receives_gold : item.seller_receives_yuanbao
  return `${receives} ${tradeCurrencyLabel(item)}`
}

function skillStats(skill: PlayerSkill) {
  const parts = skillEffectParts(skill)
  const type = typeof skill.config.type === 'string' ? skill.config.type : '被动'
  const useMode = skillKindValue(skill) === 'passive'
    ? '被动常驻'
    : `${autoEnabled(skill) ? '自动释放' : '自动关闭'} · 消耗 ${skill.mp_cost} 魔法`
  return `${type} · ${useMode} · ${parts.join(' / ') || `等级要求 Lv.${skill.min_level}`}`
}

function skillDescription(skill: PlayerSkill) {
  const description = skill.config.desc ?? skill.config.description
  return typeof description === 'string' && description.trim() ? description : skillStats(skill)
}

function skillSourceLabel(skill: PlayerSkill) {
  const source = skill.config.source
  return typeof source === 'string' && source.trim() ? source : '通过技能书、任务或首领掉落获取'
}

function skillLevelLabel(skill: PlayerSkill) {
  if (skill.learned) {
    const level = Math.max(1, skill.level ?? 1)
    const effectiveLevel = effectiveSkillLevel(skill)
    return effectiveLevel !== level ? `Lv.${level}->${effectiveLevel}` : `Lv.${level}`
  }
  return '未学习'
}

function skillKindValue(skill: PlayerSkill) {
  const kind = skill.config.kind
  if (typeof kind === 'string') return kind
  const type = skill.config.type
  if (typeof type === 'string' && type.includes('主动')) return 'physical'
  return 'passive'
}

function skillTypeLabel(skill: PlayerSkill) {
  const kind = skillKindValue(skill)
  if (kind === 'magical') return '主动·魔法'
  if (kind === 'heal') return '主动·辅助'
  if (kind === 'physical') return '主动·物理'
  return '被动'
}

function isActiveSkill(skill: PlayerSkill) {
  return skillKindValue(skill) !== 'passive'
}

function isSpecialPassive(skill: PlayerSkill) {
  return skill.config.special_upgrade_only === true
}

function autoEnabled(skill: PlayerSkill) {
  return skill.auto_enabled !== false
}

function skillAutoStatus(skill: PlayerSkill) {
  if (!skill.learned) return '未学习'
  if (!isActiveSkill(skill)) return '被动常驻'
  return autoEnabled(skill) ? '自动释放开' : '自动释放关'
}

function skillConfigType(skill: PlayerSkill, fallback: string) {
  const type = skill.config.type
  return typeof type === 'string' && type.trim() ? type : fallback
}

function skillDetailEffect(skill: PlayerSkill | null) {
  if (!skill) return '暂无技能'
  const kind = skillKindValue(skill)
  const mpCost = skillMpCost(skill)
  const levelBoost = `等级倍率 ${skillLevelMultiplierLabel(skill)}`
  const configuredEffect = typeof skill.config.effect === 'string' && skill.config.effect.trim()
    ? `${skill.config.effect} · `
    : ''
  if (kind === 'heal') {
    const heal = numericStat(skill.config.heal_power)
    return `${configuredEffect}${autoEnabled(skill) ? '自动治愈' : '自动关闭'} · 消耗 ${mpCost} 魔法 · ${levelBoost}${heal > 0 ? ` · 基础恢复 ${heal}` : ''}`
  }
  const power = numericStat(skill.config.power)
  if (kind !== 'passive' && power > 0) {
    return `${configuredEffect}${autoEnabled(skill) ? '自动释放' : '自动关闭'} · 消耗 ${mpCost} 魔法 · 基础倍率 ${Math.round(power * 100)}% · ${levelBoost}`
  }
  const parts = Object.entries(skill.config)
    .flatMap(([key, value]) => skillEffectPart(skill, key, value))
  if (parts.length) return `${configuredEffect}${parts.join(' / ')}`
  return skillStats(skill)
}

function skillEffectParts(skill: PlayerSkill) {
  return Object.entries(skill.config).flatMap(([key, value]) => skillEffectPart(skill, key, value))
}

function skillEffectPart(skill: PlayerSkill, key: string, value: unknown) {
  if (typeof value !== 'number' || !Number.isFinite(value) || value === 0) return []
  const level = Math.max(1, skill.level ?? 1)
  if (key.endsWith('_bonus')) return [`${statLabel(key)} +${Math.floor(value * level)}`]
  if (key.endsWith('_base_pct')) return [`${statLabel(key)} +${Math.floor(value)}%`]
  if (key === 'luck_bonus_per_100') {
    const amount = Math.floor((value * level) / 100)
    return amount > 0 ? [`${statLabel(key)} +${amount}`] : []
  }
  if (key.endsWith('_pct_per_100') || key === 'control_resist_pct_per_100') {
    const amount = Math.floor((value * level) / 100)
    return amount > 0 ? [`${statLabel(key)} +${amount}%`] : []
  }
  return []
}

function canUpgradeSkill(_skill: PlayerSkill) {
  return false
}

function skillLevelMultiplierLabel(skill: PlayerSkill) {
  const level = effectiveSkillLevel(skill)
  const normalGrowth = (Math.min(level, 99) - 1) * 5
  const totalGrowth = level >= 100 ? normalGrowth * 3 : normalGrowth
  return `${100 + totalGrowth}%`
}

function skillMpCost(skill: PlayerSkill) {
  return effectiveSkillLevel(skill) >= 100 && skillKindValue(skill) !== 'passive' ? 1 : Math.max(0, skill.mp_cost)
}

function effectiveSkillLevel(skill: PlayerSkill) {
  const level = Math.max(1, skill.level ?? 1)
  const bonus = skillKindValue(skill) === 'passive' ? 0 : Math.max(0, resourceCaps.value.equipment.allSkillBonus)
  return Math.min(100, level + bonus)
}

function canClaimMail(mail: PlayerMail) {
  return mail.attachments.some(
    (attachment) =>
      !attachment.claimed &&
      (Boolean(attachment.item_template_id) || attachment.gold > 0 || attachment.yuanbao > 0)
  )
}

function attachmentLabel(attachment: MailAttachment) {
  const parts: string[] = []
  if (attachment.item_template_id) {
    parts.push(`${attachment.item_name ?? attachment.item_template_id} x${attachment.quantity}`)
  }
  if (attachment.gold > 0) parts.push(`金币 ${attachment.gold}`)
  if (attachment.yuanbao > 0) parts.push(`元宝 ${attachment.yuanbao}`)
  return parts.join(' / ') || '附件'
}

function formatMailTime(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleDateString()
}

function rarityLabel(value: string) {
  const labels: Record<string, string> = {
    common: '普通',
    uncommon: '魔法',
    rare: '稀有',
    epic: '史诗',
    legendary: '传奇',
    mythic: '神话',
    supreme: '传奇',
    ultimate: '传奇'
  }
  return labels[value] ?? value
}

function itemRarityLabel(item: InventoryItem | null | undefined) {
  if (!item) return ''
  return isMythicDominatorTierItem(item) ? '神话' : rarityLabel(item.rarity)
}

function rarityClass(value: string) {
  if (['common', 'uncommon', 'rare', 'epic', 'legendary', 'mythic', 'supreme', 'ultimate'].includes(value)) {
    return `rarity-${value}`
  }
  return 'rarity-common'
}

function tierLabel(value: number) {
  const labels: Record<number, string> = {
    1: '一阶',
    2: '二阶',
    3: '三阶',
    4: '四阶',
    5: '五阶',
    6: '六阶',
    7: '七阶',
    8: '八阶',
    9: '九阶',
    10: '十阶',
    11: '十一阶',
    12: '十二阶',
    13: '十三阶',
    14: '十四阶',
    15: '十五阶',
    16: '十六阶',
    17: '十七阶',
    18: '主宰·神话'
  }
  return labels[value] ?? `${value} 阶`
}

function slotLabel(value: string | null) {
  const labels: Record<string, string> = {
    weapon: '武器',
    chest: '衣服',
    head: '头盔',
    feet: '鞋子',
    waist: '腰带',
    neck: '项链',
    ring_left: '戒指',
    ring_right: '戒指',
    bracelet_left: '手镯',
    bracelet_right: '手镯'
  }
  return value ? labels[value] ?? value : '物品'
}

function setLabel(value: string) {
  const labels: Record<string, string> = {
    qingyun: '青云套装',
    pureyang: '纯阳套装',
    jiuxiao: '九霄套装',
    chaos: '混沌套装',
    zaohua: '造化套装',
    dominator: '主宰套装'
  }
  return labels[value] ?? value
}

function statLabel(value: string) {
  const labels: Record<string, string> = {
    atk: '攻击',
    def: '防御',
    mag: '魔法',
    mdef: '魔防',
    dex: '幸运',
    luck: '幸运',
    spirit: '精神',
    atk_bonus: '攻击',
    def_bonus: '防御',
    mag_bonus: '魔法',
    mdef_bonus: '魔防',
    dex_bonus: '幸运',
    luck_bonus: '幸运',
    luck_bonus_per_100: '幸运',
    hp_bonus: '生命',
    mp_bonus: '魔法值',
    crit_bonus: '暴击',
    crit_pct_bonus: '暴击',
    crit_pct: '暴击',
    crit: '暴击',
    crit_damage_pct_bonus: '暴击伤害',
    crit_damage_pct: '暴击伤害',
    control_resist_pct_per_100: '异常抗性',
    control_resist_pct: '异常抗性',
    life_steal_pct: '吸血',
    mana_steal_pct: '吸蓝',
    heavy_hit_pct: '重击',
    paralyze_pct: '麻痹',
    petrify_pct: '石化',
    atk_pct: '攻击加成',
    atk_pct_per_100: '攻击加成',
    mag_pct: '魔法攻击加成',
    mag_pct_per_100: '魔法攻击加成',
    def_pct: '防御加成',
    def_pct_per_100: '防御加成',
    hp_pct: '生命加成',
    hp_pct_per_100: '生命加成',
    mp_pct: '魔法加成',
    mp_pct_per_100: '魔法加成',
    life_steal_base_pct: '吸血',
    life_steal_pct_per_100: '吸血',
    mana_steal_base_pct: '吸蓝',
    mana_steal_pct_per_100: '吸蓝',
    skill_damage_pct: '技能伤害',
    boss_damage_pct: '首领增伤',
    boss_drop_pct: '首领掉落率',
    damage_deepen_pct: '伤害加深',
    battle_end_restore_pct: '战后恢复',
    afk_base_reward_pct: '挂机基础收益',
    afk_exp_pct: '挂机经验',
    afk_gold_pct: '挂机金币',
    afk_drop_quality_pct: '稀有掉落加成',
    afk_kill_pct: '挂机击杀效率',
    afk_extra_minutes: '离线挂机上限',
    afk_offline_reward_pct: '离线收益',
    all_skill_bonus: '全技能等级',
    normal_mob_execute_pct: '小怪秒杀',
    damage_reduce_pct: '伤害减免',
    damage_reduce_pct_per_100: '伤害减免',
    damage_deepen_pct_per_100: '伤害加深',
    paralyze_resist_pct: '麻痹抗性',
    petrify_resist_pct: '石化抗性',
    ignore_def_pct: '无视防御',
    ignore_def_pct_per_100: '无视防御',
    guaranteed_hit_pct: '必定命中',
    guaranteed_hit_pct_per_100: '命中提升',
    target_max_hp_true_damage_pct: '目标生命真伤',
    self_max_mp_true_damage_pct: '自身魔法真伤',
    creation_strike_pct: '创世一击',
    creation_strike_damage_pct: '创世伤害',
    creation_strike_full_restore: '创世回满',
    control_immune: '控制免疫',
    death_drop_immune: '死亡防掉',
    origin_revive_cd_seconds: '原地复活冷却',
    yuanbao_decompose: '分解元宝',
    decompose_only: '分解专用',
    affix_count: '词条数',
    max_percent_affix_cap: '百分比词条上限',
    star_devourer_kill_growth_pct: '噬星成长概率',
    special_mechanism: '特殊机制',
    special_mechanism_extra: '特殊机制',
    series: '系列',
    set: '套装',
    active_crit_pct: '主动暴击',
    boss_extra_damage_cap_atk: '首领额外伤害上限',
    burn_mag_pct: '灼烧魔攻系数',
    burn_turns: '灼烧回合',
    crit_damage_reduce_pct: '暴击伤害减免',
    current_mp_cost_pct: '当前魔法消耗',
    def_to_mag_base_pct: '防御转道术',
    def_to_mag_pct_per_100: '防御转道术',
    execute_bonus_pct: '斩杀伤害',
    execute_threshold_pct: '斩杀阈值',
    free_recast_base_pct: '免费再施法',
    free_recast_pct_per_100: '免费再施法',
    hit_count_max: '最大段数',
    hit_count_min: '最小段数',
    ignore_mdef_pct: '无视魔防',
    lethal_guard_once: '致命保护',
    max_drop_tier: '最高掉落阶级',
    drop_rate: '掉落率',
    drop_rate_multiplier: '掉落倍率',
    contribution: '贡献',
    gold: '金币',
    yuanbao: '元宝',
    quantity: '数量',
    vip_days: '会员天数',
    realm_index: '境界序号',
    skill_proficiency: '技能熟练度',
    mp_to_damage_pct: '魔法转伤害',
    poison_bonus_tick: '毒素追加结算',
    poison_mag_multiplier: '毒素道术倍率',
    poison_turns: '中毒回合',
    regen_reduce_pct: '回血压制',
    slow_next_attack_pct: '下一击减速',
    target_current_hp_pct: '当前生命伤害',
    vulnerability_pct: '易伤',
    vulnerability_turns: '易伤回合',
    weakness_next_damage_reduce_pct: '虚弱减伤',
    skill_flame_blade_bonus: '烈火等级',
    skill_halfmoon_blade_bonus: '半月等级',
    skill_ice_roar_bonus: '冰咆哮等级',
    skill_poison_lore_bonus: '施毒等级',
    hp: '生命',
    mp: '魔法值',
    enhance: '强化',
    tier: '等阶',
    score: '评分'
  }
  return labels[value] ?? value
}

function classLabel(value: string) {
  const labels: Record<string, string> = {
    warrior: '剑修',
    mage: '法修',
    taoist: '魂修',
    assassin: '刺客'
  }
  return labels[value] ?? value
}

function rankingLabel(entry: RankingEntry, mode: 'power' | 'level') {
  return mode === 'power'
    ? `${entry.name} · ${classLabel(entry.class)} · 战力 ${entry.power}`
    : `${entry.name} · ${entry.level} 级`
}

function vipLabel(value: string | undefined) {
  const labels: Record<string, string> = {
    vip: 'VIP',
    svip: 'SVIP',
    permanent_svip: '永久 SVIP'
  }
  return value ? labels[value] ?? value : '未激活'
}

function vipExpiryLabel(value: string | null | undefined, active: boolean) {
  if (!active) return '未开通'
  if (!value) return '永久'
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString()
}

function petSkillLabel(value: unknown) {
  if (Array.isArray(value)) return value.join(' / ')
  if (typeof value === 'string') return value
  return '基础协战'
}

function sabakTime(value: string | undefined) {
  if (!value) return '--'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}
</script>

<template>
  <section class="shell-stack">
    <div class="view-tabs">
      <button :class="{ active: activePage === 'game' }" type="button" @click="activePage = 'game'">角色与探索</button>
      <button :class="{ active: activePage === 'inventory' }" type="button" @click="activePage = 'inventory'">装备与技能</button>
      <button :class="{ active: activePage === 'breakthrough' }" type="button" @click="activePage = 'breakthrough'">界限突破</button>
      <button :class="{ active: activePage === 'guild' }" type="button" @click="activePage = 'guild'">行会</button>
      <button :class="{ active: activePage === 'yuanbao' }" type="button" @click="activePage = 'yuanbao'">元宝玩法</button>
      <button :class="{ active: activePage === 'wiki' }" type="button" @click="activePage = 'wiki'">玩法 Wiki</button>
      <button :class="{ active: activePage === 'about' }" type="button" @click="activePage = 'about'">关于</button>
      <button class="coffee-nav-button" type="button" @click="openCoffeeModal"><Coffee :size="16" /> 请作者喝咖啡</button>
    </div>

    <div v-if="coffeeModalVisible" class="coffee-modal-backdrop" @click="closeCoffeeModal">
      <article class="coffee-modal">
        <Coffee :size="22" />
        <strong>请作者喝咖啡</strong>
        <p v-if="coffeeQrLoading">二维码读取中...</p>
        <img v-else-if="coffeeQrUrl" :src="coffeeQrUrl" alt="请作者喝咖啡二维码" />
        <p v-else>{{ coffeeQrError || '作者暂未设置二维码。' }}</p>
      </article>
    </div>

    <div v-if="pendingAdventure" class="adventure-modal-backdrop">
      <article class="adventure-modal">
        <header>
          <Sparkles :size="20" />
          <strong>{{ pendingAdventure.title }}</strong>
        </header>
        <p>{{ pendingAdventure.body }}</p>
        <div class="adventure-options">
          <button
            v-for="option in pendingAdventure.options"
            :key="option.id"
            type="button"
            @click="emit('resolveAdventure', pendingAdventure.id, option.id)"
          >
            <span>{{ option.label }}</span>
            <small v-if="option.cost_gold > 0">金币 {{ formatNumber(option.cost_gold) }}</small>
          </button>
        </div>
      </article>
    </div>

    <section v-if="activePage === 'wiki'" class="wiki-page">
      <article v-for="section in wikiSections" :key="section.title" class="panel wiki-panel">
        <header><BookOpen :size="18" /> <strong>{{ section.title }}</strong></header>
        <ul>
          <li v-for="row in section.rows" :key="row">{{ row }}</li>
        </ul>
      </article>
    </section>

    <section v-else-if="activePage === 'about'" class="wiki-page">
      <article v-for="section in aboutSections" :key="section.title" class="panel wiki-panel">
        <header><BookOpen :size="18" /> <strong>{{ section.title }}</strong></header>
        <ul>
          <li v-for="row in section.rows" :key="row">{{ row }}</li>
        </ul>
      </article>
    </section>

    <section v-else-if="activePage === 'inventory'" class="inventory-page">
      <div class="inventory-tabs">
        <button :class="{ active: inventorySubPage === 'equipment' }" type="button" @click="inventorySubPage = 'equipment'">
          <Sword :size="17" /> 装备
        </button>
        <button :class="{ active: inventorySubPage === 'warehouse' }" type="button" @click="inventorySubPage = 'warehouse'">
          <PackageOpen :size="17" /> 仓库
        </button>
        <button :class="{ active: inventorySubPage === 'skills' }" type="button" @click="inventorySubPage = 'skills'">
          <BookOpen :size="17" /> 技能
        </button>
      </div>

      <section v-if="inventorySubPage === 'equipment'" class="equipment-page">
        <article class="panel inventory-character-panel equipment-character-panel">
          <header><Sparkles :size="18" /> <strong>角色详情</strong></header>
          <div class="equipment-character-body">
            <div class="character-detail-grid">
              <div v-for="item in detailStats" :key="`equipment-detail-${item.label}`" class="detail-stat">
                <span>{{ item.label }}</span>
                <strong>{{ item.value }}</strong>
              </div>
            </div>
          </div>
        </article>

        <article class="panel inventory-workbench equipment-center-panel">
          <header>
            <Sword :size="18" />
            <strong>装备栏</strong>
            <small>{{ equippedItems.length }}/{{ equipmentSlotRows.length }} 件</small>
          </header>
          <div class="equipment-summary-grid equipment-only-grid">
            <div class="equipment-slot-list">
              <div class="section-title tight">
                <strong>装备</strong>
                <small>{{ equippedItems.length }}/{{ equipmentSlotRows.length }} 件</small>
              </div>
              <button
                v-for="slot in equipmentSlotRows"
                :key="slot.key"
                :class="{ selected: slot.item && selectedInventoryItem?.id === slot.item.id, empty: !slot.item }"
                type="button"
                :disabled="!slot.item"
                @click="selectEquipmentItem(slot.item)"
              >
                <span>{{ slot.label }}</span>
                <strong>{{ slot.item?.name ?? '无装备' }}</strong>
                <small>{{ slot.item ? equipmentRank(slot.item) : '空槽位' }}</small>
              </button>
            </div>
          </div>

          <section class="inventory-section bag-grid-section">
            <div class="section-title">
              <span>背包</span>
              <small>{{ inventory?.summary.bag_used ?? 0 }}/{{ inventory?.summary.bag_limit ?? 0 }}</small>
              <select v-model="inventoryTierFilter" class="mini-select">
                <option v-for="option in tierOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
              <button
                class="mini-action"
                title="拆解当前筛选阶级中的未绑定背包装备"
                :disabled="!authenticated || !selectedDecomposeItems.length"
                @click="emit('decomposeEquipment', [], decomposableIds())"
              >
                一键拆解
              </button>
            </div>
            <div class="item-card-grid">
              <article
                v-for="item in filteredBagItems"
                :key="item.id"
                :class="['item-card', rarityClass(item.rarity), { selected: selectedInventoryItem?.id === item.id }]"
                @click="selectedInventoryItemId = item.id"
              >
                <strong>{{ item.name }}</strong>
                <span>{{ item.quantity }}</span>
                <small>{{ itemRarityLabel(item) }} · {{ slotLabel(item.template_slot) }}</small>
              </article>
              <p v-if="!filteredBagItems.length" class="empty-line">背包为空，先去野外打几只怪。</p>
            </div>
          </section>
        </article>

        <article class="panel item-detail-panel">
          <header><PackageOpen :size="18" /> <strong>{{ selectedInventoryItem?.name ?? '物品详情' }}</strong></header>
          <div v-if="selectedInventoryItem" class="item-detail-body">
            <small>{{ itemRarityLabel(selectedInventoryItem) }} · {{ slotLabel(selectedInventoryItem.template_slot ?? selectedInventoryItem.equipped_slot) }}</small>
            <strong class="item-score">{{ numericStat(selectedInventoryItem.stats.score) || selectedInventoryItem.price }}</strong>
            <span>{{ equipmentRank(selectedInventoryItem) }}</span>
            <div class="item-stat-list">
              <span v-for="part in itemStatParts(selectedInventoryItem)" :key="part">{{ part }}</span>
              <span v-if="!itemStatParts(selectedInventoryItem).length">无属性</span>
            </div>
            <section v-if="itemSetEffectRows(selectedInventoryItem).length" class="set-effect-box">
              <div class="set-effect-heading">
                <strong>{{ setLabel(itemSetId(selectedInventoryItem)) }}</strong>
                <small>{{ setProgressLabel(selectedInventoryItem) }}</small>
              </div>
              <div
                v-for="effect in itemSetEffectRows(selectedInventoryItem)"
                :key="`${itemSetId(selectedInventoryItem)}-${effect.pieces}`"
                :class="['set-effect-row', { active: effect.active }]"
              >
                <span>{{ effect.pieces }}件</span>
                <div>
                  <strong>{{ effect.title }}</strong>
                  <small>{{ effect.text }}</small>
                </div>
              </div>
            </section>
            <div class="inventory-actions detail-actions">
              <button v-if="canUse(selectedInventoryItem)" class="mini-action" @click="emit('useItem', selectedInventoryItem.id)">使用</button>
              <button v-if="canEquip(selectedInventoryItem)" class="mini-action" @click="emit('equip', selectedInventoryItem.id)">穿戴</button>
              <button v-if="selectedInventoryItem.location === 'equipped'" class="mini-action" @click="emit('unequip', selectedInventoryItem.id)">卸下</button>
              <button v-if="selectedInventoryItem.template_slot" class="mini-action" @click="emit('enhanceItem', selectedInventoryItem.id)">强化</button>
              <button v-if="selectedInventoryItem.location === 'bag'" class="mini-action" @click="emit('storeItem', selectedInventoryItem.id)">存仓</button>
              <button v-if="selectedInventoryItem.location === 'warehouse'" class="mini-action" @click="emit('retrieveItem', selectedInventoryItem.id)">取回</button>
              <button v-if="canDecomposeMisc(selectedInventoryItem)" class="mini-action danger" :disabled="!authenticated" @click="emit('decomposeMisc', [selectedInventoryItem.kind], [selectedInventoryItem.id])">杂项拆解</button>
              <button v-if="!selectedInventoryItem.bind && selectedInventoryItem.location === 'bag'" class="mini-action" @click="emit('listTradeItem', selectedInventoryItem.id, suggestedTradePrice(selectedInventoryItem))">寄售</button>
            </div>
          </div>
          <p v-else class="empty-line">暂无可查看物品</p>
        </article>
      </section>

      <section v-else-if="inventorySubPage === 'warehouse'" class="warehouse-page">
        <!-- 物品过滤 Sidebar Panel -->
        <article class="panel warehouse-sidebar">
          <header>
            <strong>物品过滤</strong>
          </header>
          <div class="warehouse-category-list">
            <button :class="{ active: warehouseCategoryFilter === 'all' }" type="button" @click="warehouseCategoryFilter = 'all'">
              <span class="category-btn-content">
                <PackageOpen :size="16" /> 全部物品
              </span>
              <span class="category-btn-count">{{ formatNumber(warehouseItems.length) }}</span>
            </button>
            <button :class="{ active: warehouseCategoryFilter === 'equipment' }" type="button" @click="warehouseCategoryFilter = 'equipment'">
              <span class="category-btn-content">
                <Sword :size="16" /> 装备
              </span>
              <span class="category-btn-count">{{ formatNumber(warehouseItems.filter((item) => item.template_slot).length) }}</span>
            </button>
            <button :class="{ active: warehouseCategoryFilter === 'consumable' }" type="button" @click="warehouseCategoryFilter = 'consumable'">
              <span class="category-btn-content">
                <ShoppingBag :size="16" /> 消耗品
              </span>
              <span class="category-btn-count">{{ formatNumber(warehouseItems.filter((item) => item.kind === 'consumable').length) }}</span>
            </button>
            <button :class="{ active: warehouseCategoryFilter === 'material' }" type="button" @click="warehouseCategoryFilter = 'material'">
              <span class="category-btn-content">
                <Sparkles :size="16" /> 材料
              </span>
              <span class="category-btn-count">{{ formatNumber(warehouseItems.filter((item) => item.kind === 'material').length) }}</span>
            </button>
          </div>
        </article>

        <!-- 仓库物品 Middle Panel -->
        <article class="panel warehouse-grid-panel">
          <header>
            <strong>仓库物品</strong>
          </header>

          <div class="warehouse-filter-bar">
            <!-- Search Row -->
            <div class="warehouse-search-wrapper">
              <Search :size="16" class="search-icon" />
              <input v-model="warehouseSearchQuery" type="text" placeholder="搜索物品名称..." class="warehouse-search-input" />
            </div>

            <!-- Filters Row -->
            <div class="warehouse-filter-actions">
              <div class="select-wrapper">
                <select v-model="warehouseQualityFilter" class="warehouse-select">
                  <option v-for="option in tierOptions" :key="`warehouse-${option.value}`" :value="option.value">
                    {{ option.label }}
                  </option>
                </select>
                <ChevronDown :size="14" class="select-chevron" />
              </div>

              <div class="select-wrapper">
                <select v-model="warehouseTypeFilter" class="warehouse-select">
                  <option value="all">全部类型</option>
                  <option value="weapon">武器</option>
                  <option value="chest">衣服</option>
                  <option value="head">头盔</option>
                  <option value="feet">鞋子</option>
                  <option value="waist">腰带</option>
                  <option value="neck">项链</option>
                  <option value="ring">戒指</option>
                  <option value="bracelet">手镯</option>
                  <option value="consumable">消耗品</option>
                  <option value="material">材料</option>
                  <option value="book">技能书</option>
                </select>
                <ChevronDown :size="14" class="select-chevron" />
              </div>

              <div class="select-wrapper">
                <select v-model="warehouseSortOption" class="warehouse-select">
                  <option value="default">默认排序</option>
                  <option value="score_desc">评分最高</option>
                  <option value="score_asc">评分最低</option>
                  <option value="quantity_desc">数量最多</option>
                  <option value="quantity_asc">数量最少</option>
                  <option value="rarity_desc">稀有度最高</option>
                </select>
                <ChevronDown :size="14" class="select-chevron" />
              </div>

              <div class="warehouse-view-toggle">
                <button :class="{ active: warehouseViewMode === 'grid' }" type="button" @click="warehouseViewMode = 'grid'" title="网格视图">
                  <Grid :size="16" />
                </button>
                <button :class="{ active: warehouseViewMode === 'list' }" type="button" @click="warehouseViewMode = 'list'" title="列表视图">
                  <List :size="16" />
                </button>
              </div>
            </div>
          </div>

          <!-- Items View Area -->
          <div class="warehouse-view-scroll-area">
            <template v-if="filteredWarehouseItems.length > 0">
              <!-- Grid Layout -->
              <div v-if="warehouseViewMode === 'grid'" class="item-card-grid warehouse-card-grid">
                <article
                  v-for="item in paginatedWarehouseItems"
                  :key="item.id"
                  :class="['item-card', rarityClass(item.rarity), { selected: selectedInventoryItem?.id === item.id && selectedInventoryItem?.location === 'warehouse' }]"
                  @click="selectedInventoryItemId = item.id"
                >
                  <strong>{{ item.name }}</strong>
                  <span>{{ item.quantity }}</span>
                  <small>{{ itemRarityLabel(item) }} · {{ slotLabel(item.template_slot) }}</small>
                </article>
              </div>

              <!-- List Layout -->
              <div v-else class="warehouse-list-container">
                <div
                  v-for="item in paginatedWarehouseItems"
                  :key="item.id"
                  :class="['warehouse-list-row', rarityClass(item.rarity), { selected: selectedInventoryItem?.id === item.id && selectedInventoryItem?.location === 'warehouse' }]"
                  @click="selectedInventoryItemId = item.id"
                >
                  <div class="row-icon">
                    <Sword v-if="item.template_slot" :size="16" />
                    <ShoppingBag v-else-if="item.kind === 'consumable'" :size="16" />
                    <Sparkles v-else :size="16" />
                  </div>
                  <div class="row-name">
                    <strong>{{ item.name }}</strong>
                    <small>{{ itemRarityLabel(item) }} · {{ slotLabel(item.template_slot) }}</small>
                  </div>
                  <div class="row-score" v-if="item.stats.score">
                    评分 {{ item.stats.score }}
                  </div>
                  <div class="row-quantity">
                    x{{ item.quantity }}
                  </div>
                </div>
              </div>
            </template>

            <!-- Centered Empty State when Category is selected but no item selected/exists -->
            <div v-else class="warehouse-empty-state">
              <p class="empty-title">当前分类没有物品</p>
              <p class="empty-subtitle">请更换筛选条件或在左侧选择其他分类</p>
            </div>
          </div>

          <!-- Bottom Pagination Controls -->
          <div class="warehouse-pagination-bar" v-if="filteredWarehouseItems.length > 0">
            <button class="page-arrow" :disabled="warehouseCurrentPage === 1" @click="prevWarehousePage">
              <ChevronLeft :size="16" />
            </button>

            <div class="page-numbers">
              <button v-for="(p, index) in warehousePages" :key="index" :class="['page-num', { active: warehouseCurrentPage === p, dot: p === '...' }]" @click="goToWarehousePage(p)" :disabled="p === '...'">
                {{ p }}
              </button>
            </div>

            <button class="page-arrow" :disabled="warehouseCurrentPage === totalWarehousePages" @click="nextWarehousePage">
              <ChevronRight :size="16" />
            </button>

            <div class="page-size-selector select-wrapper">
              <select v-model="warehousePageSize" class="warehouse-select page-size-select">
                <option :value="20">20 条/页</option>
                <option :value="40">40 条/页</option>
                <option :value="60">60 条/页</option>
                <option :value="100">100 条/页</option>
              </select>
              <ChevronDown :size="12" class="select-chevron" />
            </div>
          </div>
        </article>

        <!-- 物品详情 Right Panel -->
        <article class="panel item-detail-panel">
          <header>
            <strong>物品详情</strong>
          </header>

          <div class="warehouse-detail-content">
            <template v-if="selectedInventoryItem && selectedInventoryItem.location === 'warehouse'">
              <div class="item-detail-body">
                <small>{{ itemRarityLabel(selectedInventoryItem) }} · {{ slotLabel(selectedInventoryItem.template_slot ?? selectedInventoryItem.equipped_slot) }}</small>
                <div class="item-stat-list">
                  <span v-for="part in itemStatParts(selectedInventoryItem)" :key="part">{{ part }}</span>
                  <span v-if="!itemStatParts(selectedInventoryItem).length">无属性</span>
                </div>
                <section v-if="itemSetEffectRows(selectedInventoryItem).length" class="set-effect-box">
                  <div class="set-effect-heading">
                    <strong>{{ setLabel(itemSetId(selectedInventoryItem)) }}</strong>
                    <small>{{ setProgressLabel(selectedInventoryItem) }}</small>
                  </div>
                  <div
                    v-for="effect in itemSetEffectRows(selectedInventoryItem)"
                    :key="`warehouse-${itemSetId(selectedInventoryItem)}-${effect.pieces}`"
                    :class="['set-effect-row', { active: effect.active }]"
                  >
                    <span>{{ effect.pieces }}件</span>
                    <div>
                      <strong>{{ effect.title }}</strong>
                      <small>{{ effect.text }}</small>
                    </div>
                  </div>
                </section>
              </div>

              <!-- Divider line -->
              <div class="detail-divider"></div>

              <!-- 物品说明 Section -->
              <div class="item-desc-panel">
                <div class="section-title">
                  <strong>物品说明</strong>
                </div>
                <p class="desc-text">{{ itemDescriptionText(selectedInventoryItem) }}</p>
              </div>
            </template>

            <template v-else>
              <!-- Center empty placeholder -->
              <div class="item-detail-empty">
                <p class="empty-title">请在中间选择物品</p>
                <p class="empty-subtitle">查看详细信息</p>
              </div>

              <div class="detail-divider"></div>

              <!-- 物品说明 placeholder -->
              <div class="item-desc-panel">
                <div class="section-title">
                  <strong>物品说明</strong>
                </div>
                <p class="desc-text text-muted">请选择物品以查看其说明和相关信息。</p>
              </div>
            </template>
          </div>

          <!-- Actions Footer Row -->
          <div class="warehouse-detail-footer">
            <button
              class="btn-retrieve-all"
              :disabled="!authenticated || !selectedInventoryItem || selectedInventoryItem.location !== 'warehouse'"
              @click="emit('retrieveItem', selectedInventoryItem.id)"
            >
              取回背包
            </button>

            <div class="more-actions-wrapper">
              <button
                class="btn-more-actions"
                :disabled="!selectedInventoryItem || selectedInventoryItem.location !== 'warehouse'"
                @click="toggleMoreActions"
                title="更多操作"
              >
                <MoreHorizontal :size="18" />
              </button>

              <div class="more-actions-dropdown" v-if="showMoreActions">
                <button v-if="canDecompose(selectedInventoryItem)" class="dropdown-item danger" @click="emit('decomposeEquipment', [], [selectedInventoryItem.id]); showMoreActions = false">
                  拆解装备
                </button>
                <button v-if="canDecomposeMisc(selectedInventoryItem)" class="dropdown-item danger" @click="emit('decomposeMisc', [selectedInventoryItem.kind], [selectedInventoryItem.id]); showMoreActions = false">
                  杂项拆解
                </button>
                <div v-if="!canDecompose(selectedInventoryItem) && !canDecomposeMisc(selectedInventoryItem)" class="dropdown-item disabled">
                  暂无其他操作
                </div>
              </div>
            </div>
          </div>
        </article>
      </section>

      <section v-else class="skill-page">
        <article class="panel skill-library-panel">
          <header>
            <BookOpen :size="18" />
            <strong>本职业技能</strong>
            <button
              class="mini-action danger"
              title="拆解已学会或非本职业的技能书"
              :disabled="!authenticated || !redundantSkillBookItems.length"
              @click="emit('decomposeMisc', ['book'], redundantSkillBookIds())"
            >
              拆多余书
            </button>
          </header>
          <div class="skill-card-grid">
            <article
              v-for="skill in professionSkills"
              :key="skill.id"
              :class="['skill-card', { selected: selectedSkill?.id === skill.id, learned: skill.learned, 'auto-disabled': skill.learned && isActiveSkill(skill) && !autoEnabled(skill) }]"
              @click="selectedSkillId = skill.id"
            >
              <strong>{{ skill.name }}</strong>
              <span>{{ skillLevelLabel(skill) }}</span>
              <small>{{ classLabel(skill.class) }} · {{ skillTypeLabel(skill) }} · {{ skillAutoStatus(skill) }}</small>
            </article>
            <p v-if="!professionSkills.length" class="empty-line">暂无本职业技能。</p>
          </div>
          <template v-if="commonSkills.length">
            <header><Shield :size="18" /> <strong>通用技能</strong></header>
            <div class="skill-card-grid compact">
              <article
                v-for="skill in commonSkills"
                :key="skill.id"
                :class="['skill-card', { selected: selectedSkill?.id === skill.id, learned: skill.learned, 'auto-disabled': skill.learned && isActiveSkill(skill) && !autoEnabled(skill) }]"
                @click="selectedSkillId = skill.id"
              >
                <strong>{{ skill.name }}</strong>
                <span>{{ skillLevelLabel(skill) }}</span>
                <small>{{ classLabel(skill.class) }} · {{ skillConfigType(skill, '通用成长') }} · {{ skillAutoStatus(skill) }}</small>
                <p>{{ skillDescription(skill) }}</p>
              </article>
            </div>
          </template>
          <template v-if="otherSkills.length">
            <header><Sparkles :size="18" /> <strong>其他职业技能</strong></header>
            <div class="skill-card-grid compact">
              <article
                v-for="skill in otherSkills"
                :key="skill.id"
                :class="['skill-card', { selected: selectedSkill?.id === skill.id, learned: skill.learned, 'auto-disabled': skill.learned && isActiveSkill(skill) && !autoEnabled(skill) }]"
                @click="selectedSkillId = skill.id"
              >
                <strong>{{ skill.name }}</strong>
                <span>{{ skillLevelLabel(skill) }}</span>
                <small>{{ classLabel(skill.class) }} · {{ skillConfigType(skill, '职业成长') }} · {{ skillAutoStatus(skill) }}</small>
                <p>{{ skillDescription(skill) }}</p>
              </article>
            </div>
          </template>
        </article>

        <article class="panel skill-detail-panel">
          <header><BookOpen :size="18" /> <strong>{{ selectedSkill?.name ?? '技能详情' }}</strong></header>
          <div v-if="selectedSkill" class="skill-detail-body">
            <span class="skill-badge">{{ skillTypeLabel(selectedSkill) }}</span>
            <strong>{{ skillLevelLabel(selectedSkill) }} / 100</strong>
            <dl class="system-detail-list">
              <div><dt>技能描述</dt><dd>{{ skillDescription(selectedSkill) }}</dd></div>
              <div><dt>技能效果</dt><dd>{{ skillDetailEffect(selectedSkill) }}</dd></div>
              <div><dt>释放方式</dt><dd>{{ skillKindValue(selectedSkill) === 'passive' ? '被动常驻' : (autoEnabled(selectedSkill) ? '扫荡与 PK 自动使用' : '自动释放已关闭') }}</dd></div>
              <div v-if="selectedSkill.learned && isActiveSkill(selectedSkill)"><dt>自动开关</dt><dd>{{ autoEnabled(selectedSkill) ? '已开启' : '已关闭' }}</dd></div>
              <div v-if="skillKindValue(selectedSkill) !== 'passive'"><dt>魔法消耗</dt><dd>{{ skillMpCost(selectedSkill) }} 点</dd></div>
              <div><dt>技能出处</dt><dd>{{ skillSourceLabel(selectedSkill) }}</dd></div>
              <div><dt>学习要求</dt><dd>角色等级 Lv.{{ selectedSkill.min_level }}</dd></div>
            </dl>
            <div class="inventory-actions detail-actions">
              <button v-if="!selectedSkill.learned && !isSpecialPassive(selectedSkill)" class="mini-action" :disabled="!authenticated" @click="emit('learnSkill', selectedSkill.id)">学习</button>
              <button v-if="selectedSkill.learned && canUpgradeSkill(selectedSkill)" class="mini-action" :disabled="!authenticated" @click="emit('upgradeSkill', selectedSkill.id)">突破</button>
              <button v-if="selectedSkill.learned && isActiveSkill(selectedSkill)" class="mini-action" :disabled="!authenticated" @click="emit('toggleSkillAuto', selectedSkill.id, !autoEnabled(selectedSkill))">
                {{ autoEnabled(selectedSkill) ? '关闭自动释放' : '开启自动释放' }}
              </button>
              <span v-if="!selectedSkill.learned && isSpecialPassive(selectedSkill)" class="mini-pill">不动冥王参悟</span>
              <span v-if="selectedSkill.learned && !canUpgradeSkill(selectedSkill) && !isActiveSkill(selectedSkill)" class="mini-pill">已学习</span>
            </div>
          </div>
        </article>
      </section>
    </section>

    <section v-else-if="activePage === 'breakthrough'" class="breakthrough-page">
      <article :class="['panel system-panel', { locked: !systemUnlocked('treasure') }]">
        <header>
          <Sparkles :size="18" />
          <strong>法宝</strong>
          <span :class="['scene-badge', systemUnlocked('treasure') ? 'safe' : 'wild']">{{ unlockBadge('treasure') }}</span>
        </header>
        <div class="system-focus">
          <strong>{{ primaryTreasure?.name ?? '龙纹印' }}</strong>
          <span>{{ systemUnlocked('treasure') && primaryTreasure ? `${primaryTreasure.level} 级 · ${primaryTreasure.stage} 阶 · ${primaryTreasure.equipped ? '已装备' : '未装备'}` : unlockRequirement('treasure') }}</span>
          <small>{{ primaryTreasure?.passive ?? '主攻法宝提升攻击、魔法、生命和暴击，是中期推进首领战的核心成长线。' }}</small>
        </div>
        <dl class="system-detail-list">
          <div><dt>当前加成</dt><dd>{{ treasureBonusLabel() }}</dd></div>
          <div><dt>升级消耗</dt><dd>{{ treasureCostLabel() }}</dd></div>
          <div><dt>开启条件</dt><dd>击杀狂暴猪王后开启。</dd></div>
          <div><dt>定位</dt><dd>中期配合装备突破地图门槛，后期用生命和暴击拉长 Boss 战容错。</dd></div>
        </dl>
        <button class="mini-action" :disabled="!authenticated || !primaryTreasure || !systemUnlocked('treasure')" @click="emit('upgradeTreasure', primaryTreasure?.id)">
          {{ systemUnlocked('treasure') ? '升级法宝' : '未开启' }}
        </button>
      </article>

      <article :class="['panel system-panel', { locked: !systemUnlocked('cultivation') }]">
        <header>
          <Shield :size="18" />
          <strong>境界</strong>
          <span :class="['scene-badge', systemUnlocked('cultivation') ? 'safe' : 'wild']">{{ unlockBadge('cultivation') }}</span>
        </header>
        <div class="system-focus">
          <strong>{{ cultivation ? `${cultivation.realm} ${cultivation.layer} 层` : '凡境 1 层' }}</strong>
          <span>{{ systemUnlocked('cultivation') && cultivation ? `进度 ${cultivation.progress_pct}% · 下级经验 ${cultivation.next_level_exp}` : unlockRequirement('cultivation') }}</span>
          <small>境界修炼是长期底盘，稳定提供攻击、魔法、生命、魔法值和双防，越到中期越影响续航。</small>
        </div>
        <dl class="system-detail-list">
          <div><dt>当前加成</dt><dd>{{ cultivationBonusLabel() }}</dd></div>
          <div><dt>突破消耗</dt><dd>{{ cultivationCostLabel() }}</dd></div>
          <div><dt>开启条件</dt><dd>击杀镇界石魔后开启。</dd></div>
          <div><dt>定位</dt><dd>不爆发但最稳，适合填平怪物强度曲线，避免只靠装备造成战力断层。</dd></div>
        </dl>
        <button class="mini-action" :disabled="!authenticated || !cultivation || !systemUnlocked('cultivation')" @click="emit('cultivationBreakthrough')">
          {{ systemUnlocked('cultivation') ? '境界突破' : '未开启' }}
        </button>
      </article>

      <article :class="['panel system-panel', { locked: !systemUnlocked('wanxiang') }]">
        <header>
          <Sparkles :size="18" />
          <strong>万象铸体</strong>
          <span :class="['scene-badge', systemUnlocked('wanxiang') ? 'safe' : 'wild']">{{ unlockBadge('wanxiang') }}</span>
        </header>
        <div class="system-focus">
          <strong>{{ wanxiang ? `Lv.${wanxiang.level} / 1000` : 'Lv.1 / 1000' }}</strong>
          <span>{{ systemUnlocked('wanxiang') && wanxiang ? `进度 ${wanxiang.progress_pct}% · 灵韵 ${wanxiang.essence}` : unlockRequirement('wanxiang') }}</span>
          <small>天地为炉，造化为工；废弃装备可提取灵韵淬炼肉身。</small>
        </div>
        <dl class="system-detail-list">
          <div><dt>当前加成</dt><dd>{{ wanxiangBonusLabel() }}</dd></div>
          <div><dt>突破消耗</dt><dd>{{ wanxiangCostLabel() }}</dd></div>
          <div><dt>开启条件</dt><dd>第一次进入星际观测台后开启。</dd></div>
          <div><dt>提取规则</dt><dd>选择阶级后批量提取背包中 1 至所选阶级的装备。</dd></div>
          <div><dt>定位</dt><dd>启动【万化神炉】，将废弃装备投入其中进行“灵韵提取”，可剥离出最纯粹的天地灵韵来淬炼肉身，重塑五脏六腑。当你将万千凡铁尽数化为己用时，你的凡胎肉体必将超越界限，成就万象神体！</dd></div>
        </dl>
        <div class="inventory-actions detail-actions">
          <select v-model.number="wanxiangExtractTier" class="compact-select">
            <option v-for="tier in wanxiangTierOptions" :key="tier" :value="tier">{{ tierLabel(tier) }}</option>
          </select>
          <button class="mini-action" :disabled="!authenticated || !systemUnlocked('wanxiang')" @click="handleWanxiangExtract">批量提取</button>
          <button class="mini-action" :disabled="!authenticated || !systemUnlocked('wanxiang') || !wanxiang || wanxiang.level >= 1000" @click="emit('upgradeWanxiang')">
            铸体突破
          </button>
        </div>
      </article>

      <article :class="['panel system-panel', { locked: !systemUnlocked('pet') }]">
        <header>
          <Bot :size="18" />
          <strong>宠物</strong>
          <span :class="['scene-badge', systemUnlocked('pet') ? 'safe' : 'wild']">{{ unlockBadge('pet') }}</span>
        </header>
        <div class="system-focus">
          <strong>{{ primaryPet?.name ?? '白虎幼崽' }}</strong>
          <span>{{ systemUnlocked('pet') && primaryPet ? `${primaryPet.level} 级 · ${primaryPet.fighting ? '出战中' : '休息中'} · ${petSkillLabel(primaryPet.skills)}` : unlockRequirement('pet') }}</span>
          <small>宠物提供协战攻击、生命和少量防御，帮助新手期稳定打怪，中期承担补刀和护主价值。</small>
        </div>
        <dl class="system-detail-list">
          <div><dt>当前加成</dt><dd>{{ petBonusLabel() }}</dd></div>
          <div><dt>升级消耗</dt><dd>{{ petCostLabel() }}</dd></div>
          <div><dt>开启条件</dt><dd>击杀尸傀监工后开启。</dd></div>
          <div><dt>定位</dt><dd>成长成本较低，适合最先补；后续高阶地图仍需要和法宝、修炼一起搭配。</dd></div>
        </dl>
        <button class="mini-action" :disabled="!authenticated || !primaryPet || !systemUnlocked('pet')" @click="emit('upgradePet', primaryPet?.id)">
          {{ systemUnlocked('pet') ? '升级宠物' : '未开启' }}
        </button>
      </article>
    </section>

    <section v-else-if="activePage === 'guild'" class="guild-page">
      <article class="panel guild-status-panel">
        <header>
          <Users :size="18" />
          <strong>{{ activeGuild?.name ?? '行会' }}</strong>
          <button v-if="!activeGuild" class="mini-action" :disabled="!authenticated" @click="promptCreateGuild">
            创建
          </button>
          <button v-else class="mini-action" :disabled="!authenticated" @click="promptDonateGuild">
            捐献
          </button>
        </header>
        <div v-if="activeGuild" class="guild-focus">
          <strong>{{ activeGuild.sabak_owner ? '沙城 · ' : '' }}{{ activeGuild.name }} {{ activeGuild.level }} 级</strong>
          <span>{{ guildRoleLabel(activeGuild.role) }} · 个人贡献 {{ activeGuild.contribution }} · 成员 {{ activeGuild.member_count }}</span>
          <small>{{ activeGuild.notice || '暂无公告' }}</small>
        </div>
        <div v-else class="guild-focus">
          <strong>尚未加入行会</strong>
          <span>可申请已有行会，或花费 1000 万金币创建新行会。</span>
          <small>加入或创建成功后，未处理的入会申请会自动取消。</small>
        </div>
        <div class="guild-progress-box">
          <div>
            <span>行会建设</span>
            <strong>{{ guildNextLevelLabel(activeGuild) }}</strong>
          </div>
          <div class="guild-progress-track">
            <span :style="{ width: `${guildProgressPct(activeGuild)}%` }"></span>
          </div>
        </div>
      </article>

      <article class="panel guild-benefit-panel">
        <header><Gift :size="18" /> <strong>行会福利</strong></header>
        <div class="guild-focus">
          <strong>{{ activeGuild ? `${activeGuild.level} 级福利` : '未加入行会' }}</strong>
          <span>{{ activeGuild ? guildBenefitPreview(activeGuild.level) : '加入行会后可领取' }}</span>
          <small>每日一次，奖励均为绑定升级材料。</small>
        </div>
        <button class="mini-action" :disabled="!authenticated || !activeGuild" @click="emit('claimGuildBenefit')">
          领取福利
        </button>
        <button class="mini-action" :disabled="!authenticated || !activeGuild" @click="emit('useGuildMeritToken')">
          使用功勋令
        </button>
        <button
          class="mini-action"
          :disabled="!authenticated || !activeGuild?.sabak_owner || activeGuild?.sabak_tax_claimed_today"
          @click="emit('claimSabakTax')"
        >
          {{ activeGuild?.sabak_tax_claimed_today ? '税收已领' : '领取沙城税收' }}
        </button>
      </article>

      <article class="panel guild-totem-panel">
        <header><Sparkles :size="18" /> <strong>行会图腾</strong></header>
        <div class="trade-list">
          <article v-for="totem in activeGuild?.totems ?? []" :key="totem.kind" class="trade-row">
            <Shield :size="16" />
            <div>
              <strong>{{ totem.name }} Lv.{{ totem.level }}/{{ totem.max_level }}</strong>
              <small>{{ totem.description }}</small>
              <small>{{ totem.unlocked ? `下级消耗贡献 ${totem.next_cost}` : '行会 10 级解锁' }}</small>
            </div>
            <button
              class="mini-action"
              :disabled="!authenticated || !activeGuild || !totem.unlocked || totem.level >= totem.max_level"
              @click="emit('upgradeGuildTotem', totem.kind)"
            >
              升级
            </button>
          </article>
          <p v-if="!activeGuild" class="empty-line">加入 10 级行会后开放个人图腾。</p>
        </div>
      </article>

      <article class="panel guild-war-panel">
        <header><Sword :size="18" /> <strong>沙城争霸</strong></header>
        <div class="guild-focus">
          <strong>{{ activeGuild?.sabak_owner ? '当前沙巴克霸主' : '20 级行会自动参战' }}</strong>
          <span>攻城战车与守城结界会计入每周自动演算积分。</span>
          <small>虚拟守城方为比奇远征队，不生成假人账号。</small>
        </div>
        <div class="trade-list">
          <article v-for="tech in activeGuild?.war_techs ?? []" :key="tech.kind" class="trade-row">
            <Shield :size="16" />
            <div>
              <strong>{{ tech.name }} Lv.{{ tech.level }}</strong>
              <small>{{ tech.description }}</small>
              <small>{{ tech.unlocked ? `积分 +${tech.score_bonus} · 下级贡献 ${tech.next_cost}` : '行会 20 级解锁' }}</small>
            </div>
            <button
              class="mini-action"
              :disabled="!authenticated || !activeGuild || !tech.unlocked"
              @click="emit('chargeGuildWarTech', tech.kind)"
            >
              充能
            </button>
          </article>
        </div>
      </article>

      <article class="panel guild-shop-panel">
        <header><ShoppingBag :size="18" /> <strong>行会商城</strong></header>
        <div class="trade-list">
          <article class="trade-row">
            <Shield :size="16" />
            <div>
              <strong>血色幽影</strong>
              <small>主宰神话戒指 · 双攻 +2000 · 生命/魔法 +2000 · 50% 暴击 · 150% 爆伤</small>
              <small>1000000 贡献 · 不朽防爆</small>
            </div>
            <button class="mini-action" :disabled="!authenticated || !activeGuild" @click="emit('buyGuildShopItem', 'blood_shadow')">购买</button>
          </article>
          <article class="trade-row">
            <Gift :size="16" />
            <div>
              <strong>护脉丹</strong>
              <small>使用恢复 40% 生命和魔法</small>
              <small>80 贡献</small>
            </div>
            <button class="mini-action" :disabled="!authenticated || !activeGuild" @click="emit('buyGuildShopItem', 'big_taizi')">购买</button>
          </article>
          <article class="trade-row">
            <Gift :size="16" />
            <div>
              <strong>九转还魂丹</strong>
              <small>使用恢复 60% 生命和魔法</small>
              <small>200 贡献</small>
            </div>
            <button class="mini-action" :disabled="!authenticated || !activeGuild" @click="emit('buyGuildShopItem', 'jiuzhuan')">购买</button>
          </article>
        </div>
      </article>

      <article class="panel guild-task-panel">
        <header>
          <ScrollText :size="18" />
          <strong>行会任务</strong>
          <span class="panel-count">{{ activeGuild?.projects.length ?? 0 }}</span>
        </header>
        <div v-if="activeGuild?.projects.length" class="guild-project-list expanded">
          <article v-for="project in activeGuild.projects" :key="project.kind" class="guild-project-row">
            <div>
              <strong>{{ project.name }}</strong>
              <small>{{ project.progress }}/{{ project.required }} · {{ project.description }}</small>
              <small>最低角色 {{ project.min_level }} 级 · {{ project.available ? '可完成' : '行会等级不足' }}</small>
            </div>
            <button
              class="mini-action"
              :disabled="guildProjectDisabled(project)"
              @click="emit('completeGuildTask', project.kind)"
            >
              {{ guildProjectButtonLabel(project) }}
            </button>
          </article>
        </div>
        <p v-else class="empty-line">加入行会后开放每日任务。</p>
      </article>

      <article class="panel guild-list-panel">
        <header>
          <Users :size="18" />
          <strong>行会列表</strong>
          <span class="panel-count">{{ overview?.guilds.length ?? 0 }}</span>
        </header>
        <div class="guild-list">
          <article v-for="guild in overview?.guilds ?? []" :key="guild.id" :class="['guild-row', { joined: guild.joined }]">
            <div>
              <strong>{{ guild.sabak_owner ? '沙城 · ' : '' }}{{ guild.name }} {{ guild.level }} 级</strong>
              <small>{{ guild.notice || '暂无公告' }}</small>
              <small>
                成员 {{ guild.member_count }} · 建设 {{ guild.funds }}
                <template v-if="guild.joined"> · {{ guildRoleLabel(guild.role) }} · 贡献 {{ guild.contribution }}</template>
              </small>
            </div>
            <button
              class="mini-action"
              :disabled="!authenticated || Boolean(activeGuild) || guild.pending_application || guild.joined"
              @click="promptApplyGuild(guild.id)"
            >
              {{ guild.joined ? '已加入' : guild.pending_application ? '待审批' : '申请' }}
            </button>
          </article>
          <p v-if="!(overview?.guilds.length)" class="empty-line">暂未创建行会</p>
        </div>
      </article>

      <article v-if="canReviewGuildApplications" class="panel guild-review-panel">
        <header>
          <Mail :size="18" />
          <strong>入会审批</strong>
          <span class="panel-count">{{ guildApplications.length }}</span>
        </header>
        <div class="guild-application-list">
          <article v-for="application in guildApplications" :key="application.id" class="guild-application-row">
            <div>
              <strong>{{ application.character_name }}</strong>
              <small>{{ application.message || '未填写留言' }}</small>
            </div>
            <button class="mini-action" :disabled="!authenticated" @click="emit('reviewGuildApplication', application.id, true)">
              通过
            </button>
            <button class="mini-action danger" :disabled="!authenticated" @click="emit('reviewGuildApplication', application.id, false)">
              拒绝
            </button>
          </article>
          <p v-if="!guildApplications.length" class="empty-line">暂无待审批申请</p>
        </div>
      </article>
    </section>

    <section v-else-if="activePage === 'yuanbao'" class="yuanbao-page">
      <article class="panel trade-panel">
        <header>
          <ShoppingBag :size="18" />
          <strong>寄售</strong>
          <span class="panel-count">{{ consignments.length }}</span>
          <button class="mini-action" @click="emit('refresh')">刷新</button>
        </header>
        <div class="trade-list">
          <article v-for="item in marketItems" :key="item.id" class="trade-row">
            <ShoppingBag :size="16" />
            <div>
              <strong>{{ item.name }} x{{ item.quantity }}</strong>
              <small>{{ item.seller_name }} / {{ slotLabel(item.template_slot) }} / {{ rarityLabel(item.rarity) }}</small>
              <small>标价 {{ item.price }} {{ tradeCurrencyLabel(item) }} · 成交税由卖家承担</small>
              <small>{{ tradeStats(item) }}</small>
            </div>
            <button class="mini-action" :disabled="!authenticated" @click="emit('buyTradeItem', item.id)">
              {{ item.price }} {{ tradeCurrencyLabel(item) }}
            </button>
          </article>
          <article v-for="item in myMarketItems" :key="`mine-${item.id}`" class="trade-row mine">
            <ShoppingBag :size="16" />
            <div>
              <strong>{{ item.name }} x{{ item.quantity }}</strong>
              <small>
                我的寄售 / 标价 {{ item.price }} {{ tradeCurrencyLabel(item) }} / 税后 {{ tradeSellerReceives(item) }}
              </small>
              <small>上架费 {{ item.listing_fee_gold }} 金币 / 成交税 {{ tradeTaxLabel(item) }}</small>
              <small>{{ tradeStats(item) }}</small>
            </div>
            <button class="mini-action" :disabled="!authenticated" @click="emit('cancelTradeItem', item.id)">下架</button>
          </article>
          <p v-if="!consignments.length" class="empty-line">暂无寄售</p>
        </div>
      </article>

      <article class="panel vip-panel">
        <header><Gift :size="18" /> <strong>会员功能</strong></header>
        <div class="vip-status-line">
          <span>{{ vipLabel(overview?.systems.vip?.tier) }}</span>
          <small>自动用药、低阶装备自动拆解或灵韵提取，不增加战斗属性。</small>
          <small>会员到期时间：{{ vipExpiryLabel(overview?.systems.vip?.ends_at, Boolean(overview?.systems.vip)) }}</small>
        </div>
        <div class="vip-settings-grid">
          <label class="toggle-line">
            <input v-model="vipHpEnabled" type="checkbox" />
            <span>生命自动用药</span>
          </label>
          <label>
            <span>生命低于百分比自动用药</span>
            <input v-model.number="vipHpThreshold" min="1" max="99" type="number" />
          </label>
          <label>
            <span>生命药品</span>
            <select v-model="vipHpTemplate">
              <option v-for="option in hpPotionOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
          </label>
          <label class="toggle-line">
            <input v-model="vipMpEnabled" type="checkbox" />
            <span>魔法自动用药</span>
          </label>
          <label>
            <span>魔法低于百分比自动用药</span>
            <input v-model.number="vipMpThreshold" min="1" max="99" type="number" />
          </label>
          <label>
            <span>魔法药品</span>
            <select v-model="vipMpTemplate">
              <option v-for="option in mpPotionOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
          </label>
          <label class="toggle-line">
            <input v-model="vipAutoDecomposeEnabled" type="checkbox" />
            <span>自动拆解低阶装备</span>
          </label>
          <label class="toggle-line">
            <input v-model="vipAutoExtractEssenceEnabled" type="checkbox" />
            <span>自动提取灵韵</span>
          </label>
          <label>
            <span>拆解阶级上限</span>
            <select v-model.number="vipAutoDecomposeMaxTier">
              <option :value="0">关闭</option>
              <option v-for="tier in 17" :key="tier" :value="tier">{{ tier }} 阶及以下</option>
            </select>
          </label>
          <label>
            <span>提取阶级上限</span>
            <select v-model.number="vipAutoExtractEssenceMaxTier">
              <option :value="0">关闭</option>
              <option v-for="tier in 17" :key="`extract-${tier}`" :value="tier">{{ tier }} 阶及以下</option>
            </select>
          </label>
        </div>
        <button class="mini-action" :disabled="!authenticated" @click="saveVipPotionSettings">保存设置</button>
      </article>

      <article class="panel yuanbao-shop-panel">
        <header><ShoppingBag :size="18" /> <strong>元宝商城</strong></header>
        <div class="trade-list">
          <article v-for="item in yuanbaoItems" :key="item.templateId" class="trade-row">
            <Sparkles :size="16" />
            <div>
              <strong>{{ item.name }} x{{ item.quantity }}</strong>
              <small>{{ item.desc }}</small>
              <small>{{ item.price }} 元宝</small>
            </div>
            <button class="mini-action" :disabled="!authenticated" @click="emit('buyYuanbaoItem', item.templateId, item.quantity)">购买</button>
          </article>
        </div>
      </article>
    </section>

  <section v-else class="game-grid">
    <article class="panel playbook-panel">
      <header><Sparkles :size="18" /> <strong>角色详情</strong></header>
      <div class="character-detail-grid">
        <div v-for="item in detailStats" :key="item.label" class="detail-stat">
          <span>{{ item.label }}</span>
          <strong>{{ item.value }}</strong>
        </div>
      </div>
    </article>

    <div class="quest-skill-stack">
      <article class="panel quest-panel">
        <header>
          <Gift :size="18" />
          <strong>任务</strong>
          <span class="panel-count">{{ completedQuestCount }}</span>
        </header>
        <div class="quest-list">
          <article v-for="quest in visibleQuests" :key="quest.id" :class="['quest-row', quest.status]">
            <div>
              <div class="quest-heading">
                <strong>{{ quest.name }}</strong>
                <span>{{ questCategoryLabel(quest.category) }}</span>
              </div>
              <small>{{ quest.description }}</small>
              <small>进度 {{ Math.min(quest.progress, quest.required) }}/{{ quest.required }} · {{ questRewardLabel(quest) }}</small>
            </div>
            <button
              class="mini-action"
              :disabled="!authenticated || quest.status !== 'completed'"
              @click="emit('claimQuest', quest.id)"
            >
              {{ questStatusLabel(quest.status) }}
            </button>
          </article>
          <p v-if="!visibleQuests.length" class="empty-line">当前任务都已领取，日常会在新周期刷新。</p>
        </div>
      </article>

      <article class="panel mail-panel">
        <header>
          <Mail :size="18" />
          <strong>邮件</strong>
          <span class="panel-count">{{ unreadMails }}</span>
        </header>
        <div class="mail-list">
          <article v-for="mail in mails" :key="mail.id" :class="['mail-row', { unread: !mail.read }]">
            <div class="mail-heading">
              <strong>{{ mail.title }}</strong>
              <small>{{ mail.from_name }} · {{ formatMailTime(mail.created_at) }}</small>
            </div>
            <p>{{ mail.body }}</p>
            <div v-if="mail.attachments.length" class="attachment-list">
              <span v-for="attachment in mail.attachments" :key="attachment.id" :class="{ claimed: attachment.claimed }">
                <Gift :size="13" />
                {{ attachmentLabel(attachment) }}
              </span>
            </div>
            <div class="mail-actions">
              <button class="mini-action" :disabled="!authenticated || mail.read" @click="emit('readMail', mail.id)">
                {{ mail.read ? '已读' : '标记已读' }}
              </button>
              <button class="mini-action" :disabled="!authenticated || mail.claimed || !canClaimMail(mail)" @click="emit('claimMail', mail.id)">
                {{ mail.claimed ? '已领取' : '领取' }}
              </button>
              <button class="mini-action danger" :disabled="!authenticated" @click="emit('deleteMail', mail.id)">
                删除
              </button>
            </div>
          </article>
          <p v-if="!mails.length" class="empty-line">暂无邮件</p>
        </div>
      </article>
    </div>

    <div class="middle-stack">
      <article class="panel room-panel">
        <header>
          <Map :size="18" />
          <strong>{{ roomState?.room.name ?? '载入中' }}</strong>
          <span v-if="roomState" :class="['scene-badge', roomState.room.safe ? 'safe' : 'wild']">
            {{ roomState.room.safe ? '安全区' : '野外' }}
          </span>
        </header>
        <p class="room-desc">{{ roomState?.room.desc ?? '正在进入比奇地区。' }}</p>
        <div class="map-meta">
          <span>位置 {{ roomState?.room.name ?? '--' }}</span>
          <span>出没 {{ roomState?.mobs.length ?? 0 }}</span>
          <span :class="{ danger: fatigueActive }">体力 {{ staminaLabel }}</span>
          <span>空闲恢复 {{ idleRecoveryLabel }}/分</span>
        </div>
        <div v-if="isStudyRoom" class="afk-box room-afk-box">
          <div>
            <strong>虚境研修</strong>
            <small>
              {{ afk?.active ? `${afkModeLabel}运行中` : '未开始' }}
              <template v-if="afk"> · 预估经验 {{ afk.exp_per_minute }}/分</template>
              · {{ afkHint }}
            </small>
          </div>
          <label class="afk-skill-select">
            <span>研修技能</span>
            <select v-model="selectedTrainingSkillId" :disabled="!authenticated || afk?.active || !safeTrainingSkills.length">
              <option v-for="skill in safeTrainingSkills" :key="skill.id" :value="skill.id">
                {{ skill.name }} · Lv.{{ skill.level ?? 1 }}
              </option>
            </select>
          </label>
          <div class="afk-actions">
            <button class="mini-action" :disabled="!canStartStudyAfk" @click="startSafeAfk">开始修炼</button>
            <button class="mini-action" :disabled="!authenticated || !afk?.active" @click="emit('stopAfk')">停止</button>
          </div>
        </div>
        <div v-else-if="isTrainingRoom" class="afk-box room-afk-box">
          <div>
            <strong>炼狱修炼</strong>
            <small>
              {{ afk?.active ? `${afkModeLabel}运行中` : '未开始' }}
              <template v-if="afk"> · 预估经验 {{ afk.exp_per_minute }}/分</template>
              · {{ afkHint }}
            </small>
          </div>
          <div class="afk-actions">
            <button class="mini-action" :disabled="!canStartPracticeAfk" @click="startSafeAfk">开始修炼</button>
            <button class="mini-action" :disabled="!authenticated || !afk?.active" @click="emit('stopAfk')">停止</button>
          </div>
        </div>
        <div class="exit-list">
          <button
            v-for="(_, direction) in roomState?.room.exits"
            :key="direction"
            class="exit-button"
            :disabled="!authenticated"
            @click="emit('move', String(direction))"
          >
            {{ directionLabel(direction) }}
          </button>
        </div>
        <div v-if="isSafeRoom" class="teleport-box">
          <div class="section-title tight">
            <strong>传送员</strong>
            <small>每次 10000 金币</small>
          </div>
          <div class="teleport-list">
            <button
              v-for="destination in teleportDestinations"
              :key="`${destination.zone}:${destination.room}`"
              class="mini-action"
              :disabled="teleportDisabled(destination)"
              @click="emit('teleport', destination.zone, destination.room)"
            >
              {{ destination.name }}
            </button>
          </div>
        </div>
        <div v-if="isMaterialExchangeHub" class="teleport-box npc-action-box">
          <div class="section-title tight">
            <strong>商人协会</strong>
            <small>悟性丹 x10 兑换</small>
          </div>
          <div class="teleport-list">
            <button class="mini-action" :disabled="!authenticated" @click="emit('exchangeNpcMaterial', 'treasure_shard')">法宝碎片</button>
            <button class="mini-action" :disabled="!authenticated" @click="emit('exchangeNpcMaterial', 'cultivation_pill')">修炼丹</button>
            <button class="mini-action" :disabled="!authenticated" @click="emit('exchangeNpcMaterial', 'pet_food')">灵兽粮</button>
          </div>
        </div>
        <div v-if="isMartialMasterHub" class="teleport-box npc-action-box">
          <div class="section-title tight">
            <strong>不动冥王</strong>
            <small>特殊被动突破</small>
          </div>
          <div class="teleport-list">
            <button class="mini-action" :disabled="!authenticated" @click="emit('upgradeBattleInstinct')">升级战斗本能</button>
            <button class="mini-action" :disabled="!authenticated" @click="emit('upgradeSpecialSkill', 'talent_immovable_king')">升级不动冥王身</button>
          </div>
        </div>
      </article>

      <article class="panel potion-panel">
        <header>
          <ShoppingBag :size="18" />
          <strong>药品栏</strong>
          <span class="panel-count">{{ quickPotionItems.length }}</span>
        </header>
        <div v-if="quickPotionItems.length" class="quick-potion-layout">
          <div class="quick-potion-strip" role="listbox" aria-label="药品栏">
            <button
              v-for="item in quickPotionItems"
              :key="item.id"
              type="button"
              :class="['quick-potion-card', `rarity-${item.rarity}`, { selected: selectedQuickPotion?.id === item.id }]"
              :aria-pressed="selectedQuickPotion?.id === item.id"
              :title="`${item.name} · ${quickPotionEffect(item)}`"
              @click="selectQuickPotion(item.id)"
            >
              <strong>{{ item.name }}</strong>
              <span>x{{ item.quantity }}</span>
              <small>{{ quickPotionEffect(item) }}</small>
            </button>
          </div>
          <div class="potion-action-row">
            <span>{{ selectedQuickPotion ? `${selectedQuickPotion.name} · ${quickPotionEffect(selectedQuickPotion)}` : '未选择药品' }}</span>
            <button class="mini-action" type="button" :disabled="!authenticated || !selectedQuickPotion" @click="useSelectedQuickPotion">
              使用
            </button>
          </div>
        </div>
        <p v-else class="empty-line">背包暂无可用药剂</p>
      </article>

      <article class="panel players-panel">
        <header><Users :size="18" /> <strong>同屏玩家</strong></header>
        <p v-if="!(roomState?.players.length)" class="empty-line">暂无其他玩家</p>
        <div v-for="(player, index) in roomState?.players" :key="player" class="player-line action-line">
          <span>{{ player }}</span>
          <button
            class="mini-action"
            :disabled="!authenticated || !characterState?.pk_enabled || !pkAllowedHere"
            @click="emit('attackBot', index)"
          >
            PK
          </button>
        </div>
      </article>

      <article class="panel supply-panel">
        <header>
          <ShoppingBag :size="18" />
          <strong>补给商店</strong>
        </header>
        <p v-if="!isSafeRoom" class="empty-line">补给商店只在安全区营业。</p>
        <div v-else class="trade-list">
          <article v-for="item in shopItems" :key="item.templateId" class="trade-row">
            <ShoppingBag :size="16" />
            <div>
              <strong>{{ item.name }} x{{ shopBuyQuantity(item) }}</strong>
              <small>{{ item.desc }} · 单价 {{ item.unitPrice }} 金币 · 合计 {{ shopItemTotal(item) }} 金币</small>
            </div>
            <div class="shop-buy-controls">
              <input
                v-model.number="shopQuantities[item.templateId]"
                class="mini-number"
                min="1"
                max="999"
                type="number"
              />
              <button class="mini-action" :disabled="!authenticated" @click="buyShopItem(item)">购买</button>
            </div>
          </article>
        </div>
      </article>
    </div>

    <article class="panel target-panel">
      <header>
        <Skull :size="18" />
        <strong>探索与挑战</strong>
        <span class="panel-count">{{ roomState?.mobs.length ?? 0 }}</span>
      </header>
      <div class="pk-settings">
        <label class="toggle-line">
          <input
            type="checkbox"
            :checked="Boolean(characterState?.pk_enabled)"
            :disabled="!authenticated"
            @change="changePkEnabled"
          />
          <span>PK 模式</span>
        </label>
        <small>{{ pkAllowedHere ? '开启后可手动攻击同屏玩家，探索会优先扫荡玩家。' : '安全区禁 PK' }}</small>
      </div>
      <div class="target-list">
        <button
          class="target-row"
          :disabled="!canExploreArea"
          @click="handleExploreClick"
        >
          <span>
            <strong>探索当前区域</strong>
            <small>{{ exploreCooldownMs > 0 && !hasDominatorSetPrivilege ? `${Math.ceil(exploreCooldownMs / 1000)} 秒后可再次探索。` : '随机遭遇当前房间出没的敌人，自动完成数回合战斗并结算奖励。' }}</small>
          </span>
          <Sword :size="18" />
        </button>
        <button class="target-row" :disabled="!authenticated || !canExploreSecretRealm" @click="emit('exploreSecretRealm')">
          <span>
            <strong>探索秘境</strong>
            <small>{{ canExploreSecretRealm ? '幻境每 1 小时挑战一次，只产出核心养成材料。' : '进入任意城市或安全区后开启。' }}</small>
          </span>
          <Sparkles :size="18" />
        </button>
        <button class="target-row" :disabled="!authenticated || !canChallengeTower" @click="emit('challengeTower')">
          <span>
            <strong>挑战无尽塔</strong>
            <small>{{ canChallengeTower ? '每 1 小时从 1 层连续扫荡到失败或当前版本边界。' : '前往天水古城后开启。' }}</small>
          </span>
          <Shield :size="18" />
        </button>
        <button class="target-row" :disabled="!authenticated || !canChallengeWorldBoss" @click="emit('challengeWorldBoss')">
          <span>
            <strong>世界首领</strong>
            <small>{{ canChallengeWorldBoss ? '在虚空要塞挑战 Lv.600 万古渊魔，胜利后 4 小时刷新并必掉主宰套装。' : '前往虚空要塞后开启。' }}</small>
          </span>
          <Skull :size="18" />
        </button>
        <div class="current-spawn-box target-spawn-box">
          <strong>当前出没</strong>
          <div class="current-spawn-list">
            <span v-for="mob in roomState?.mobs ?? []" :key="mob">{{ mob }}</span>
            <span v-if="!(roomState?.mobs.length)">安全区暂无随机敌人</span>
          </div>
        </div>
      </div>
      <p v-if="!(roomState?.mobs.length)" class="empty-line">安全区没有随机敌人，可以去秘境、无尽塔或世界首领。</p>
    </article>

    <article class="panel log-panel">
      <header>
        <ScrollText :size="18" />
        <strong>战斗记录</strong>
        <span class="scene-badge">{{ lastEventAt || '--' }}</span>
      </header>
      <ol class="log-list">
        <li v-for="(line, index) in log" :key="`${line}-${index}`">{{ line }}</li>
      </ol>
    </article>

    <article class="panel feature-roadmap compact-roadmap">
      <header><Shield :size="18" /> <strong>世界看板</strong></header>
      <div class="overview-grid">
        <section>
          <strong>战力榜</strong>
          <ol class="ranking-list">
            <li v-for="entry in overview?.power_rankings ?? []" :key="`power-${entry.character_id}`">
              <span>{{ entry.rank }}</span>
              {{ rankingLabel(entry, 'power') }}
            </li>
          </ol>
          <p v-if="!(overview?.power_rankings.length)" class="empty-line">暂无排行数据</p>
        </section>
        <section>
          <strong>等级榜</strong>
          <ol class="ranking-list">
            <li v-for="entry in overview?.level_rankings ?? []" :key="`level-${entry.character_id}`">
              <span>{{ entry.rank }}</span>
              {{ rankingLabel(entry, 'level') }}
            </li>
          </ol>
          <p v-if="!(overview?.level_rankings.length)" class="empty-line">暂无排行数据</p>
        </section>
        <section>
          <strong>活动</strong>
          <div class="overview-tags">
            <span v-for="activity in visibleActivities" :key="activity.id">
              {{ activity.name }} · {{ activity.points }}
            </span>
          </div>
          <p v-if="!visibleActivities.length" class="empty-line">暂无开启活动</p>
        </section>
      </div>
      <div class="feature-grid compact">
        <div class="feature-card">
          <Bot :size="18" />
          <strong>宠物</strong>
          <span v-if="!systemUnlocked('pet')">{{ unlockRequirement('pet') }}</span>
          <span v-else-if="primaryPet">
            {{ primaryPet.name }} {{ primaryPet.level }} 级
            · {{ primaryPet.fighting ? '出战中' : '休息中' }}
            · {{ petSkillLabel(primaryPet.skills) }}
          </span>
          <span v-else>暂无宠物，建角后会发放白虎幼崽。</span>
        </div>
        <div class="feature-card">
          <Sparkles :size="18" />
          <strong>法宝</strong>
          <span v-if="!systemUnlocked('treasure')">{{ unlockRequirement('treasure') }}</span>
          <span v-else-if="primaryTreasure">
            {{ primaryTreasure.name }} {{ primaryTreasure.level }} 级
            · {{ primaryTreasure.equipped ? '已装备' : '未装备' }}
            · {{ primaryTreasure.passive }}
          </span>
          <span v-else>暂无法宝，建角后会发放龙纹印。</span>
        </div>
        <div class="feature-card">
          <Shield :size="18" />
          <strong>境界</strong>
          <span v-if="!systemUnlocked('cultivation')">{{ unlockRequirement('cultivation') }}</span>
          <span v-else-if="cultivation">
            {{ cultivation.realm }} {{ cultivation.layer }} 层
            · 下级经验 {{ cultivation.next_level_exp }}
            · {{ cultivation.progress_pct }}%
          </span>
          <span v-else>暂无境界数据。</span>
        </div>
        <div class="feature-card">
          <Gift :size="18" />
          <strong>会员</strong>
          <span>{{ vipLabel(overview?.systems.vip?.tier) }}</span>
        </div>
        <div class="feature-card">
          <Users :size="18" />
          <strong>沙巴克</strong>
          <span v-if="overview?.systems.sabak">
            霸主 {{ overview.systems.sabak.winner_guild || '比奇远征队' }}
            · 20级行会 {{ overview.systems.sabak.signup_count }}
            · 下次结算 {{ sabakTime(overview.systems.sabak.battle_starts_at) }}
          </span>
          <span v-else>暂无攻城计划。</span>
        </div>
        <div v-for="feature in featureCards" :key="feature.title" class="feature-card">
          <component :is="feature.icon" :size="18" />
          <strong>{{ feature.title }}</strong>
          <span>{{ feature.text }}</span>
        </div>
      </div>
    </article>
  </section>
  </section>
</template>
