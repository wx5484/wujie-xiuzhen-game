<script setup lang="ts">
import { computed, onMounted, reactive, ref, type Component } from 'vue'
import {
  Activity,
  Bot,
  CircleAlert,
  Clock3,
  Coffee,
  Database,
  FlaskConical,
  HardDrive,
  KeyRound,
  LogIn,
  LogOut,
  Mail,
  NotebookTabs,
  Package,
  RefreshCw,
  ScrollText,
  Server,
  Settings,
  Shield,
  Swords,
  Trash2,
  UserRound,
  Users
} from '@lucide/vue'

type AdminTab =
  | 'dashboard'
  | 'accounts'
  | 'characters'
  | 'mail'
  | 'items'
  | 'mobs'
  | 'bots'
  | 'test'
  | 'backup'
  | 'audit'
  | 'settings'
  | 'system'

interface ApiOk<T> {
  ok: boolean
  data: T
  message?: string
}

interface DashboardSummary {
  online: number
  accounts: number
  characters: number
  mails: number
  guilds: number
  pending_backups: number
}

interface AdminAccount {
  id: number
  username: string
  email: string | null
  status: string
  created_at: string
  last_login_at: string | null
  character_count: number
}

interface AdminAccountList {
  total: number
  accounts: AdminAccount[]
}

interface AdminCharacter {
  id: number
  account_id: number
  account_username: string
  name: string
  class: string
  level: number
  exp: number
  gold: number
  yuanbao: number
  power: number
  zone: string | null
  room: string | null
  hp: number | null
  mp: number | null
  online: boolean
  created_at: string
  updated_at: string
}

interface AdminCharacterList {
  total: number
  characters: AdminCharacter[]
}

interface AdminCharacterStats {
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

interface AdminCharacterState {
  character_id: number
  zone: string
  room: string
  hp: number
  mp: number
  online: boolean
  updated_at: string
}

interface AdminInventoryItem {
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
  location: string
  equipped_slot: string | null
  bind: boolean
  durability: number
}

interface AdminInventoryView {
  summary: {
    bag_used: number
    bag_limit: number
    warehouse_used: number
    equipment: Record<string, number | null>
  }
  items: AdminInventoryItem[]
}

interface AdminCharacterDetail {
  character: AdminCharacter
  stats: AdminCharacterStats
  state: AdminCharacterState
  inventory: AdminInventoryView
}

interface AdminMail {
  id: number
  to_character_id: number
  to_character_name: string | null
  account_id: number | null
  account_username: string | null
  from_account_id: number | null
  from_name: string
  title: string
  read: boolean
  claimed: boolean
  expires_at: string | null
  created_at: string
  attachment_count: number
  attachment_gold: number
  attachment_yuanbao: number
}

interface AdminMailOverview {
  total: number
  unread: number
  unclaimed: number
  with_attachments: number
  recent: AdminMail[]
}

interface AdminItemTemplate {
  id: string
  name: string
  kind: string
  slot: string | null
  rarity: string
  price: number
  stackable: boolean
  stats: unknown
  flags: unknown
  version: number
  created_at: string
  updated_at: string
}

interface AdminItemTemplateList {
  total: number
  items: AdminItemTemplate[]
}

interface AdminMobTemplate {
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
  drops: unknown
  version: number
  created_at: string
  updated_at: string
}

interface AdminMobTemplateList {
  total: number
  mobs: AdminMobTemplate[]
}

interface AdminAuditLog {
  id: number
  admin_account_id: number | null
  admin_username: string | null
  action: string
  target: string
  detail: unknown
  ip: string | null
  created_at: string
}

interface AdminAuditLogList {
  total: number
  logs: AdminAuditLog[]
}

interface AdminBot {
  id: number
  name: string
  bot_class: string
  level: number
  exp: number
  gold: number
  power: number
  zone: string
  room: string
  hp: number
  mp: number
  mode: string
  team_code: string
  target_zone: string
  target_room: string
  enabled: boolean
  script: unknown
  last_action_at: string | null
  updated_at: string
}

interface AdminBotList {
  total: number
  bots: AdminBot[]
}

interface HealthResponse {
  status: string
  service: string
  version: string
}

interface AdminLoginResponse {
  username: string
  token: string
}

interface AdminSettingsView {
  coffee_qr_url: string
}

const tabs: Array<{ key: AdminTab; label: string; eyebrow: string; icon: Component }> = [
  { key: 'dashboard', label: '仪表盘', eyebrow: '运营状态', icon: Activity },
  { key: 'accounts', label: '账号注册', eyebrow: '注册人数列表', icon: Users },
  { key: 'characters', label: '角色概览', eyebrow: '角色资产', icon: UserRound },
  { key: 'mail', label: '邮件', eyebrow: '全服触达', icon: Mail },
  { key: 'items', label: '物品模板', eyebrow: '掉落与商城配置', icon: Package },
  { key: 'mobs', label: '怪物模板', eyebrow: '战斗模板', icon: Swords },
  { key: 'bots', label: '仿玩家', eyebrow: '脚本调度', icon: Bot },
  { key: 'test', label: '测试', eyebrow: '功能验证', icon: FlaskConical },
  { key: 'backup', label: '备份', eyebrow: '数据安全', icon: Database },
  { key: 'audit', label: '审计', eyebrow: 'GM 行为', icon: Shield },
  { key: 'settings', label: '设置', eyebrow: '展示与入口', icon: Settings },
  { key: 'system', label: '系统状态', eyebrow: '服务探活', icon: Server }
]

const token = ref(localStorage.getItem('cq-admin-token') ?? '')
const adminUser = ref(localStorage.getItem('cq-admin-user') ?? '')
const adminLogin = reactive({
  username: '',
  password: ''
})
const adminLoginLoading = ref(false)
const adminLoginError = ref('')
const activeTab = ref<AdminTab>('dashboard')
const summary = ref<DashboardSummary | null>(null)
const accounts = ref<AdminAccountList | null>(null)
const characters = ref<AdminCharacterList | null>(null)
const selectedCharacter = ref<AdminCharacterDetail | null>(null)
const mailOverview = ref<AdminMailOverview | null>(null)
const itemTemplates = ref<AdminItemTemplateList | null>(null)
const mobTemplates = ref<AdminMobTemplateList | null>(null)
const bots = ref<AdminBotList | null>(null)
const auditLogs = ref<AdminAuditLogList | null>(null)
const healthz = ref<HealthResponse | null>(null)
const readyz = ref<HealthResponse | null>(null)
const lastSyncedAt = ref('')
const actionMessage = ref('')

const adminSettings = reactive({
  coffee_qr_url: ''
})

const mailDraft = reactive({
  to_character_id: '',
  title: '测试补给',
  body: 'GM 发放的测试服补给，请在玩家端邮件中领取。',
  gold: 1000,
  yuanbao: 0,
  item_template_id: 'potion_small',
  quantity: 5
})

const characterPatch = reactive({
  character_id: '',
  exp_delta: 0,
  gold_delta: 0,
  yuanbao_delta: 0,
  reason: '测试服调试'
})

const activityToggle = reactive({
  code: 'daily_hunt',
  enabled: true
})

const testTools = reactive({
  character_id: ''
})

const accountStatusPatch = reactive({
  account_id: '',
  status: 'banned',
  reason: 'GM action'
})

const characterStatePatch = reactive({
  character_id: '',
  zone: '',
  room: '',
  hp: '',
  mp: '',
  online: false,
  force_offline: true,
  reason: 'GM kick'
})

const characterDetailPatch = reactive({
  character_id: '',
  name: '',
  class: 'warrior',
  level: 1,
  exp: 0,
  gold: 0,
  yuanbao: 0,
  power: 0,
  zone: '',
  room: '',
  hp: '',
  mp: '',
  online: false,
  reason: '角色详情调整'
})

const characterItemDraft = reactive({
  item_id: '',
  template_id: 'potion_small',
  quantity: 1,
  location: 'bag',
  slot: '',
  bind: false,
  durability: 100,
  extra: '{}'
})

const itemTemplateDraft = reactive({
  id: 'gm_test_item',
  name: 'GM Test Item',
  kind: 'consumable',
  slot: '',
  rarity: 'common',
  price: 0,
  stackable: true,
  stats: '{}',
  flags: '{}'
})

const botBatch = reactive({
  bot_id: 'all',
  mode: 'progression',
  enabled: true,
  zone: 'no_change',
  room: 'no_change',
  team_code: 'no_change',
  target_zone: 'no_change',
  target_room: 'no_change',
  tick_limit: 50
})

const botCreate = reactive({
  name: '',
  bot_class: 'warrior'
})

const botDelete = reactive({
  bot_id: ''
})

const loading = reactive<Record<AdminTab, boolean>>({
  dashboard: false,
  accounts: false,
  characters: false,
  mail: false,
  items: false,
  mobs: false,
  bots: false,
  test: false,
  backup: false,
  audit: false,
  settings: false,
  system: false
})

const errors = reactive<Record<AdminTab, string>>({
  dashboard: '',
  accounts: '',
  characters: '',
  mail: '',
  items: '',
  mobs: '',
  bots: '',
  test: '',
  backup: '',
  audit: '',
  settings: '',
  system: ''
})

const tokenValue = computed(() => token.value.trim())
const adminAuthenticated = computed(() => Boolean(tokenValue.value))
const currentTab = computed(() => tabs.find((tab) => tab.key === activeTab.value) ?? tabs[0])
const tokenStatus = computed(() => (adminAuthenticated.value ? '已登录' : '未登录'))
const accountRows = computed(() => accounts.value?.accounts ?? [])
const characterRows = computed(() => characters.value?.characters ?? [])
const selectedInventoryItems = computed(() => selectedCharacter.value?.inventory.items ?? [])
const selectedEquippedItems = computed(() => selectedInventoryItems.value.filter((item) => item.location === 'equipped'))
const selectedBagItems = computed(() => selectedInventoryItems.value.filter((item) => item.location === 'bag'))
const selectedWarehouseItems = computed(() => selectedInventoryItems.value.filter((item) => item.location === 'warehouse'))
const mailRows = computed(() => mailOverview.value?.recent ?? [])
const itemRows = computed(() => itemTemplates.value?.items ?? [])
const mobRows = computed(() => mobTemplates.value?.mobs ?? [])
const botRows = computed(() => bots.value?.bots ?? [])
const botTargetOptions = [
  { value: 'no_change', label: '不修改' },
  { value: '', label: '清空' },
  { value: 'newbie-a', label: '新手小队' },
  { value: 'mine-clear', label: '矿洞清场' },
  { value: 'stone-team', label: '石墓组队' },
  { value: 'boss-watch', label: '首领巡逻' }
]
const botTickOptions = [
  { value: 10, label: '推进 10 个' },
  { value: 50, label: '推进 50 个' },
  { value: 100, label: '推进 100 个' },
  { value: 200, label: '推进 200 个' }
]
const botZoneOptions = [
  { value: 'no_change', label: '不修改' },
  { value: 'bq_town', label: '比奇城' },
  { value: 'bq_plains', label: '比奇野外' },
  { value: 'snake_valley', label: '毒蛇山谷' },
  { value: 'mengzhong', label: '盟重' },
  { value: 'cangyue', label: '苍月岛' },
  { value: 'fengmo', label: '封魔谷' }
]
const botRoomOptions = [
  { value: 'no_change', label: '不修改' },
  { value: 'gate', label: '比奇城门' },
  { value: 'newbie_village', label: '新手村' },
  { value: 'plains', label: '比奇平原' },
  { value: 'forest', label: '森林小径' },
  { value: 'mine_entrance', label: '矿洞入口' },
  { value: 'valley_depth', label: '山谷深处' },
  { value: 'woma_temple', label: '沃玛寺庙' },
  { value: 'town', label: '盟重土城' },
  { value: 'stone_tomb', label: '石墓阵' },
  { value: 'zuma_temple', label: '祖玛神庙' },
  { value: 'safe_harbor', label: '苍月码头' },
  { value: 'bull_temple', label: '牛魔寺庙' },
  { value: 'camp', label: '封魔营地' },
  { value: 'mine_path', label: '封魔矿道' },
  { value: 'overlord_hall', label: '霸者大厅' },
  { value: 'altar', label: '封魔祭坛' },
  { value: 'bairimen', label: '白日门' },
  { value: 'valley_secret', label: '山谷密道' },
  { value: 'choice_land', label: '抉择之地' },
  { value: 'demon_altar', label: '恶魔祭坛' },
  { value: 'redmoon_canyon', label: '赤月峡谷' },
  { value: 'molong_city', label: '魔龙城' },
  { value: 'molong_east', label: '魔龙东郊' },
  { value: 'east_forest', label: '东林间胜地' },
  { value: 'old_molong_village', label: '魔龙旧寨' },
  { value: 'molong_swamp', label: '魔龙沼泽' },
  { value: 'molong_bloodland', label: '魔龙血域' }
]
const auditRows = computed(() => auditLogs.value?.logs ?? [])
const activeError = computed(() => {
  if (activeTab.value === 'backup') return errors.dashboard
  return errors[activeTab.value]
})
const isRefreshing = computed(() => {
  if (activeTab.value === 'backup') return loading.dashboard
  return loading[activeTab.value]
})

const dashboardMetrics = computed(() => [
  { label: '在线人数', value: summary.value?.online, hint: '实时在线角色' },
  { label: '注册账号', value: summary.value?.accounts, hint: '累计账号数' },
  { label: '角色总数', value: summary.value?.characters, hint: '未删除角色' },
  { label: '邮件总量', value: summary.value?.mails, hint: '站内邮件记录' },
  { label: '行会数量', value: summary.value?.guilds, hint: '已创建行会' },
  { label: '备份队列', value: summary.value?.pending_backups, hint: 'queued / running' }
])

const accountStatusRows = computed(() => {
  const counts = new Map<string, number>()
  for (const account of accountRows.value) {
    counts.set(account.status, (counts.get(account.status) ?? 0) + 1)
  }
  return Array.from(counts.entries()).map(([status, count]) => ({
    status,
    label: statusLabel(status),
    count
  }))
})

const characterAverage = computed(() => {
  if (!summary.value?.accounts) return '--'
  return (summary.value.characters / summary.value.accounts).toFixed(2)
})

const mailAverage = computed(() => {
  if (!summary.value?.characters) return '--'
  return (summary.value.mails / summary.value.characters).toFixed(2)
})

const onlineRate = computed(() => {
  if (!summary.value?.characters) return '--'
  return `${((summary.value.online / summary.value.characters) * 100).toFixed(1)}%`
})

const onlineCharacterCount = computed(() => characterRows.value.filter((character) => character.online).length)
const averageCharacterLevel = computed(() => averageValue(characterRows.value.map((character) => character.level)))
const totalCharacterPower = computed(() =>
  characterRows.value.reduce((total, character) => total + character.power, 0)
)
const stackableItemCount = computed(() => itemRows.value.filter((item) => item.stackable).length)
const itemKindCount = computed(() => new Set(itemRows.value.map((item) => item.kind)).size)
const bossMobCount = computed(() => mobRows.value.filter((mob) => mob.boss).length)
const averageMobLevel = computed(() => averageValue(mobRows.value.map((mob) => mob.level)))
const enabledBotCount = computed(() => botRows.value.filter((bot) => bot.enabled).length)
const averageBotLevel = computed(() => averageValue(botRows.value.map((bot) => bot.level)))

const systemRows = computed(() => [
  {
    name: '/api/healthz',
    label: 'HTTP 服务',
    status: healthz.value?.status ?? 'unknown',
    detail: healthz.value ? `${healthz.value.service} v${healthz.value.version}` : '未连接'
  },
  {
    name: '/api/readyz',
    label: '数据库就绪',
    status: readyz.value?.status ?? 'unknown',
    detail: readyz.value ? `${readyz.value.service} v${readyz.value.version}` : '未连接'
  },
  {
    name: '/admin/',
    label: 'GM 前端',
    status: 'ok',
    detail: 'Vite base 已部署到 /admin/'
  }
])

onMounted(() => {
  void loadSystem()
  if (tokenValue.value) void loadDashboard()
})

async function adminLoginAction() {
  adminLoginLoading.value = true
  adminLoginError.value = ''
  try {
    const response = await fetch('/api/admin/login', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        username: adminLogin.username.trim(),
        password: adminLogin.password
      })
    })
    const text = await response.text()
    const body = parseBody<AdminLoginResponse>(text)
    if (!response.ok || !body?.ok) {
      const fallback = `${response.status} ${response.statusText}`.trim() || '登录失败'
      throw new Error(body?.message ?? fallback)
    }
    token.value = body.data.token
    adminUser.value = body.data.username
    adminLogin.password = ''
    localStorage.setItem('cq-admin-token', token.value)
    localStorage.setItem('cq-admin-user', adminUser.value)
    await loadDashboard()
  } catch (error) {
    adminLoginError.value = errorMessage(error)
  } finally {
    adminLoginLoading.value = false
  }
}

function adminLogout() {
  token.value = ''
  adminUser.value = ''
  actionMessage.value = ''
  adminLoginError.value = ''
  localStorage.removeItem('cq-admin-token')
  localStorage.removeItem('cq-admin-user')
}

function selectTab(tab: AdminTab) {
  activeTab.value = tab
  void ensureTabData(tab)
}

async function ensureTabData(tab: AdminTab) {
  if (tab === 'dashboard' && !summary.value && !loading.dashboard) await loadDashboard()
  if (tab === 'accounts' && !accounts.value && !loading.accounts) await loadAccounts()
  if (tab === 'characters' && !characters.value && !loading.characters) await loadCharacters()
  if (tab === 'mail' && !mailOverview.value && !loading.mail) await loadMailOverview()
  if (tab === 'items' && !itemTemplates.value && !loading.items) await loadItemTemplates()
  if (tab === 'mobs' && !mobTemplates.value && !loading.mobs) await loadMobTemplates()
  if (tab === 'bots' && !bots.value && !loading.bots) await loadBots()
  if (tab === 'audit' && !auditLogs.value && !loading.audit) await loadAuditLogs()
  if (tab === 'settings' && !loading.settings) await loadSettings()
  if (tab === 'backup' && !summary.value && !loading.dashboard) await loadDashboard()
  if (tab === 'system' && !loading.system) await loadSystem()
}

async function refreshActive() {
  if (activeTab.value === 'dashboard') await loadDashboard()
  if (activeTab.value === 'accounts') await loadAccounts()
  if (activeTab.value === 'characters') await loadCharacters()
  if (activeTab.value === 'mail') await loadMailOverview()
  if (activeTab.value === 'items') await loadItemTemplates()
  if (activeTab.value === 'mobs') await loadMobTemplates()
  if (activeTab.value === 'bots') await loadBots()
  if (activeTab.value === 'test') {
    errors.test = ''
    markSynced()
  }
  if (activeTab.value === 'audit') await loadAuditLogs()
  if (activeTab.value === 'settings') await loadSettings()
  if (activeTab.value === 'backup') await loadDashboard()
  if (activeTab.value === 'system') await loadSystem()
}

async function loadDashboard() {
  if (!requireToken('dashboard')) return
  loading.dashboard = true
  errors.dashboard = ''
  try {
    summary.value = await request<DashboardSummary>('/api/admin/dashboard', true)
    markSynced()
  } catch (error) {
    errors.dashboard = errorMessage(error)
  } finally {
    loading.dashboard = false
  }
}

async function loadAccounts() {
  if (!requireToken('accounts')) return
  loading.accounts = true
  errors.accounts = ''
  try {
    accounts.value = await request<AdminAccountList>('/api/admin/accounts', true)
    markSynced()
  } catch (error) {
    errors.accounts = errorMessage(error)
  } finally {
    loading.accounts = false
  }
}

async function loadCharacters() {
  if (!requireToken('characters')) return
  loading.characters = true
  errors.characters = ''
  try {
    characters.value = await request<AdminCharacterList>('/api/admin/characters', true)
    if (selectedCharacter.value) {
      await loadCharacterDetail(selectedCharacter.value.character.id, false)
    }
    markSynced()
  } catch (error) {
    errors.characters = errorMessage(error)
  } finally {
    loading.characters = false
  }
}

async function loadCharacterDetail(characterId: number, setBusy = true) {
  if (!requireToken('characters')) return
  if (setBusy) loading.characters = true
  errors.characters = ''
  try {
    selectedCharacter.value = await request<AdminCharacterDetail>(
      `/api/admin/character-detail?character_id=${characterId}`,
      true
    )
    fillCharacterDetailPatch(selectedCharacter.value)
    characterPatch.character_id = String(characterId)
    characterStatePatch.character_id = String(characterId)
    if (!itemTemplates.value && !loading.items) {
      await loadItemTemplates()
    }
    markSynced()
  } catch (error) {
    errors.characters = errorMessage(error)
  } finally {
    if (setBusy) loading.characters = false
  }
}

async function loadMailOverview() {
  if (!requireToken('mail')) return
  loading.mail = true
  errors.mail = ''
  try {
    mailOverview.value = await request<AdminMailOverview>('/api/admin/mail', true)
    markSynced()
  } catch (error) {
    errors.mail = errorMessage(error)
  } finally {
    loading.mail = false
  }
}

async function loadItemTemplates() {
  if (!requireToken('items')) return
  loading.items = true
  errors.items = ''
  try {
    itemTemplates.value = await request<AdminItemTemplateList>('/api/admin/items', true)
    markSynced()
  } catch (error) {
    errors.items = errorMessage(error)
  } finally {
    loading.items = false
  }
}

async function loadMobTemplates() {
  if (!requireToken('mobs')) return
  loading.mobs = true
  errors.mobs = ''
  try {
    mobTemplates.value = await request<AdminMobTemplateList>('/api/admin/mobs', true)
    markSynced()
  } catch (error) {
    errors.mobs = errorMessage(error)
  } finally {
    loading.mobs = false
  }
}

async function loadBots() {
  if (!requireToken('bots')) return
  loading.bots = true
  errors.bots = ''
  try {
    bots.value = await request<AdminBotList>('/api/admin/bots', true)
    markSynced()
  } catch (error) {
    errors.bots = errorMessage(error)
  } finally {
    loading.bots = false
  }
}

async function loadAuditLogs() {
  if (!requireToken('audit')) return
  loading.audit = true
  errors.audit = ''
  try {
    auditLogs.value = await request<AdminAuditLogList>('/api/admin/audit', true)
    markSynced()
  } catch (error) {
    errors.audit = errorMessage(error)
  } finally {
    loading.audit = false
  }
}

async function loadSettings() {
  if (!requireToken('settings')) return
  loading.settings = true
  errors.settings = ''
  try {
    const data = await request<AdminSettingsView>('/api/admin/settings', true)
    adminSettings.coffee_qr_url = data.coffee_qr_url
    markSynced()
  } catch (error) {
    errors.settings = errorMessage(error)
  } finally {
    loading.settings = false
  }
}

async function loadSystem() {
  loading.system = true
  errors.system = ''
  const [health, ready] = await Promise.allSettled([
    request<HealthResponse>('/api/healthz', false),
    request<HealthResponse>('/api/readyz', false)
  ])

  if (health.status === 'fulfilled') {
    healthz.value = health.value
  } else {
    healthz.value = null
    errors.system = errorMessage(health.reason)
  }

  if (ready.status === 'fulfilled') {
    readyz.value = ready.value
  } else {
    readyz.value = null
    errors.system = [errors.system, errorMessage(ready.reason)].filter(Boolean).join('；')
  }

  if (!errors.system) markSynced()
  loading.system = false
}

async function saveSettingsAction() {
  if (!requireToken('settings')) return
  loading.settings = true
  errors.settings = ''
  actionMessage.value = ''
  try {
    const data = await requestPost<AdminSettingsView>('/api/admin/settings', {
      coffee_qr_url: adminSettings.coffee_qr_url
    })
    adminSettings.coffee_qr_url = data.coffee_qr_url
    actionMessage.value = '设置已保存。'
    await loadAuditLogs()
    markSynced()
  } catch (error) {
    errors.settings = errorMessage(error)
  } finally {
    loading.settings = false
  }
}

async function sendMailAction() {
  if (!requireToken('mail')) return
  loading.mail = true
  errors.mail = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/send-mail', {
      to_character_id: optionalNumber(mailDraft.to_character_id),
      title: mailDraft.title,
      body: mailDraft.body,
      gold: Number(mailDraft.gold) || 0,
      yuanbao: Number(mailDraft.yuanbao) || 0,
      item_template_id: mailDraft.item_template_id.trim() || null,
      quantity: Number(mailDraft.quantity) || 1
    })
    actionMessage.value = result.message
    await loadMailOverview()
    await loadAuditLogs()
  } catch (error) {
    errors.mail = errorMessage(error)
  } finally {
    loading.mail = false
  }
}

async function adjustCharacterAction() {
  if (!requireToken('characters')) return
  loading.characters = true
  errors.characters = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/adjust-character', {
      character_id: Number(characterPatch.character_id),
      exp_delta: Number(characterPatch.exp_delta) || 0,
      gold_delta: Number(characterPatch.gold_delta) || 0,
      yuanbao_delta: Number(characterPatch.yuanbao_delta) || 0,
      reason: characterPatch.reason
    })
    actionMessage.value = result.message
    await loadCharacters()
    await loadDashboard()
    await loadAuditLogs()
  } catch (error) {
    errors.characters = errorMessage(error)
  } finally {
    loading.characters = false
  }
}

async function setAccountStatusAction() {
  if (!requireToken('accounts')) return
  loading.accounts = true
  errors.accounts = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/toggle-activity', {
      action: 'set_account_status',
      account_id: Number(accountStatusPatch.account_id),
      status: accountStatusPatch.status,
      reason: accountStatusPatch.reason
    })
    actionMessage.value = result.message
    await loadAccounts()
    await loadAuditLogs()
  } catch (error) {
    errors.accounts = errorMessage(error)
  } finally {
    loading.accounts = false
  }
}

async function clearAllAccountsAction() {
  if (!requireToken('accounts')) return
  const confirmed = window.confirm('确认清除所有账号、角色与关联玩家数据？此操作不可恢复。')
  if (!confirmed) return
  loading.accounts = true
  errors.accounts = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/accounts/clear-all', {})
    actionMessage.value = result.message
    accounts.value = null
    characters.value = null
    selectedCharacter.value = null
    mailOverview.value = null
    await loadDashboard()
    await loadAccounts()
    await loadAuditLogs()
  } catch (error) {
    errors.accounts = errorMessage(error)
  } finally {
    loading.accounts = false
  }
}

async function setCharacterStateAction() {
  if (!requireToken('characters')) return
  loading.characters = true
  errors.characters = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/toggle-activity', {
      action: 'set_character_state',
      character_id: Number(characterStatePatch.character_id),
      zone: characterStatePatch.zone.trim() || null,
      room: characterStatePatch.room.trim() || null,
      hp: optionalNumber(characterStatePatch.hp),
      mp: optionalNumber(characterStatePatch.mp),
      online: characterStatePatch.online,
      force_offline: characterStatePatch.force_offline,
      reason: characterStatePatch.reason
    })
    actionMessage.value = result.message
    await loadCharacters()
    await loadDashboard()
    await loadAuditLogs()
  } catch (error) {
    errors.characters = errorMessage(error)
  } finally {
    loading.characters = false
  }
}

async function saveCharacterDetailAction() {
  if (!requireToken('characters')) return
  loading.characters = true
  errors.characters = ''
  actionMessage.value = ''
  try {
    const characterId = Number(characterDetailPatch.character_id)
    const result = await requestPost<{ message: string }>('/api/admin/character-update', {
      character_id: characterId,
      name: characterDetailPatch.name.trim(),
      class: characterDetailPatch.class,
      level: Number(characterDetailPatch.level) || 1,
      exp: Number(characterDetailPatch.exp) || 0,
      gold: Number(characterDetailPatch.gold) || 0,
      yuanbao: Number(characterDetailPatch.yuanbao) || 0,
      power: Number(characterDetailPatch.power) || 0,
      zone: characterDetailPatch.zone.trim() || null,
      room: characterDetailPatch.room.trim() || null,
      hp: optionalNumber(characterDetailPatch.hp),
      mp: optionalNumber(characterDetailPatch.mp),
      online: characterDetailPatch.online,
      reason: characterDetailPatch.reason
    })
    actionMessage.value = result.message
    await loadCharacters()
    await loadCharacterDetail(characterId, false)
    await loadDashboard()
    await loadAuditLogs()
  } catch (error) {
    errors.characters = errorMessage(error)
  } finally {
    loading.characters = false
  }
}

async function saveCharacterItemAction() {
  if (!requireToken('characters')) return
  loading.characters = true
  errors.characters = ''
  actionMessage.value = ''
  try {
    const characterId = Number(characterDetailPatch.character_id)
    const result = await requestPost<{ message: string }>('/api/admin/character-item', {
      character_id: characterId,
      item_id: optionalNumber(characterItemDraft.item_id),
      template_id: characterItemDraft.template_id.trim(),
      quantity: Number(characterItemDraft.quantity) || 1,
      location: characterItemDraft.location,
      slot: characterItemDraft.slot.trim() || null,
      bind: characterItemDraft.bind,
      durability: Number(characterItemDraft.durability) || 100,
      extra: parseJsonObject(characterItemDraft.extra, 'extra')
    })
    actionMessage.value = result.message
    await loadCharacterDetail(characterId, false)
    await loadAuditLogs()
  } catch (error) {
    errors.characters = errorMessage(error)
  } finally {
    loading.characters = false
  }
}

async function deleteCharacterItemAction(item: AdminInventoryItem) {
  if (!requireToken('characters')) return
  loading.characters = true
  errors.characters = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/character-item-delete', {
      character_id: item.character_id,
      item_id: item.id
    })
    actionMessage.value = result.message
    await loadCharacterDetail(item.character_id, false)
    await loadAuditLogs()
  } catch (error) {
    errors.characters = errorMessage(error)
  } finally {
    loading.characters = false
  }
}

async function saveItemTemplateAction() {
  if (!requireToken('items')) return
  loading.items = true
  errors.items = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/toggle-activity', {
      action: 'upsert_item_template',
      item: {
        id: itemTemplateDraft.id,
        name: itemTemplateDraft.name,
        kind: itemTemplateDraft.kind,
        slot: itemTemplateDraft.slot.trim() || null,
        rarity: itemTemplateDraft.rarity,
        price: Number(itemTemplateDraft.price) || 0,
        stackable: itemTemplateDraft.stackable,
        stats: parseJsonObject(itemTemplateDraft.stats, 'stats'),
        flags: parseJsonObject(itemTemplateDraft.flags, 'flags')
      }
    })
    actionMessage.value = result.message
    await loadItemTemplates()
    await loadAuditLogs()
  } catch (error) {
    errors.items = errorMessage(error)
  } finally {
    loading.items = false
  }
}

async function toggleActivityAction() {
  if (!requireToken('system')) return
  loading.system = true
  errors.system = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/toggle-activity', {
      code: activityToggle.code,
      enabled: activityToggle.enabled
    })
    actionMessage.value = result.message
    await loadAuditLogs()
  } catch (error) {
    errors.system = errorMessage(error)
  } finally {
    loading.system = false
  }
}

async function resetChallengeCooldownsAction() {
  if (!requireToken('test')) return
  loading.test = true
  errors.test = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ message: string }>('/api/admin/test/reset-challenge-cooldowns', {
      character_id: optionalNumber(testTools.character_id)
    })
    actionMessage.value = result.message
    await loadAuditLogs()
    markSynced()
  } catch (error) {
    errors.test = errorMessage(error)
  } finally {
    loading.test = false
  }
}

async function saveBotBatchAction() {
  if (!requireToken('bots')) return
  loading.bots = true
  errors.bots = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ bots: AdminBotList; message: string }>('/api/admin/bots/batch', {
      bot_ids: botBatch.bot_id === 'all' ? [] : [Number(botBatch.bot_id)],
      mode: botBatch.mode,
      enabled: botBatch.enabled,
      zone: optionValue(botBatch.zone),
      room: optionValue(botBatch.room),
      team_code: optionValue(botBatch.team_code),
      target_zone: optionValue(botBatch.target_zone),
      target_room: optionValue(botBatch.target_room),
      script: {}
    })
    bots.value = result.bots
    actionMessage.value = result.message
    await loadAuditLogs()
  } catch (error) {
    errors.bots = errorMessage(error)
  } finally {
    loading.bots = false
  }
}

async function createBotAction() {
  if (!requireToken('bots')) return
  loading.bots = true
  errors.bots = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ bots: AdminBotList; message: string }>('/api/admin/bots/create', {
      name: botCreate.name.trim(),
      bot_class: botCreate.bot_class
    })
    bots.value = result.bots
    botCreate.name = ''
    actionMessage.value = result.message
    await loadAuditLogs()
  } catch (error) {
    errors.bots = errorMessage(error)
  } finally {
    loading.bots = false
  }
}

async function deleteBotAction() {
  if (!requireToken('bots') || !botDelete.bot_id) return
  loading.bots = true
  errors.bots = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ bots: AdminBotList; message: string }>('/api/admin/bots/delete', {
      bot_id: Number(botDelete.bot_id)
    })
    bots.value = result.bots
    botDelete.bot_id = ''
    actionMessage.value = result.message
    await loadAuditLogs()
  } catch (error) {
    errors.bots = errorMessage(error)
  } finally {
    loading.bots = false
  }
}

async function tickBotsAction() {
  if (!requireToken('bots')) return
  loading.bots = true
  errors.bots = ''
  actionMessage.value = ''
  try {
    const result = await requestPost<{ bots: AdminBotList; message: string }>('/api/admin/bots/tick', {
      limit: Number(botBatch.tick_limit) || 50
    })
    bots.value = result.bots
    actionMessage.value = result.message
    await loadAuditLogs()
  } catch (error) {
    errors.bots = errorMessage(error)
  } finally {
    loading.bots = false
  }
}

function optionValue(value: string) {
  return value === 'no_change' ? null : value
}

async function request<T>(path: string, withToken: boolean): Promise<T> {
  const headers: Record<string, string> = {}
  if (withToken) headers['x-admin-token'] = tokenValue.value
  const response = await fetch(path, { headers })
  const text = await response.text()
  const body = parseBody<T>(text)
  if (!response.ok || !body?.ok) {
    const fallback = `${response.status} ${response.statusText}`.trim() || '请求失败'
    throw new Error(body?.message ?? fallback)
  }
  return body.data
}

async function requestPost<T>(path: string, payload: unknown): Promise<T> {
  const response = await fetch(path, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      'x-admin-token': tokenValue.value
    },
    body: JSON.stringify(payload)
  })
  const text = await response.text()
  const body = parseBody<T>(text)
  if (!response.ok || !body?.ok) {
    const fallback = `${response.status} ${response.statusText}`.trim() || '请求失败'
    throw new Error(body?.message ?? fallback)
  }
  return body.data
}

function parseBody<T>(text: string) {
  if (!text) return null
  try {
    return JSON.parse(text) as ApiOk<T>
  } catch {
    return null
  }
}

function requireToken(target: AdminTab) {
  if (tokenValue.value) {
    localStorage.setItem('cq-admin-token', tokenValue.value)
    return true
  }
  errors[target] = '请先登录后台'
  return false
}

function optionalNumber(value: string) {
  const trimmed = value.trim()
  if (!trimmed) return null
  const parsed = Number(trimmed)
  return Number.isFinite(parsed) ? parsed : null
}

function parseJsonObject(value: string, label: string) {
  try {
    const parsed = JSON.parse(value || '{}')
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) return parsed
  } catch {
    // handled below
  }
  throw new Error(`${label} must be a JSON object`)
}

function editItemTemplate(item: AdminItemTemplate) {
  itemTemplateDraft.id = item.id
  itemTemplateDraft.name = item.name
  itemTemplateDraft.kind = item.kind
  itemTemplateDraft.slot = item.slot ?? ''
  itemTemplateDraft.rarity = item.rarity
  itemTemplateDraft.price = item.price
  itemTemplateDraft.stackable = item.stackable
  itemTemplateDraft.stats = JSON.stringify(item.stats, null, 2)
  itemTemplateDraft.flags = JSON.stringify(item.flags, null, 2)
}

function fillCharacterDetailPatch(detail: AdminCharacterDetail) {
  characterDetailPatch.character_id = String(detail.character.id)
  characterDetailPatch.name = detail.character.name
  characterDetailPatch.class = detail.character.class
  characterDetailPatch.level = detail.character.level
  characterDetailPatch.exp = detail.character.exp
  characterDetailPatch.gold = detail.character.gold
  characterDetailPatch.yuanbao = detail.character.yuanbao
  characterDetailPatch.power = detail.character.power
  characterDetailPatch.zone = detail.state.zone
  characterDetailPatch.room = detail.state.room
  characterDetailPatch.hp = String(detail.state.hp)
  characterDetailPatch.mp = String(detail.state.mp)
  characterDetailPatch.online = detail.state.online
  resetCharacterItemDraft()
}

function resetCharacterItemDraft() {
  characterItemDraft.item_id = ''
  characterItemDraft.template_id = itemRows.value[0]?.id ?? 'potion_small'
  characterItemDraft.quantity = 1
  characterItemDraft.location = 'bag'
  characterItemDraft.slot = ''
  characterItemDraft.bind = false
  characterItemDraft.durability = 100
  characterItemDraft.extra = '{}'
}

function editCharacterItem(item: AdminInventoryItem) {
  characterItemDraft.item_id = String(item.id)
  characterItemDraft.template_id = item.template_id
  characterItemDraft.quantity = item.quantity
  characterItemDraft.location = item.location
  characterItemDraft.slot = item.equipped_slot ?? ''
  characterItemDraft.bind = item.bind
  characterItemDraft.durability = item.durability
  const extra: Record<string, unknown> = {}
  if (typeof item.stats.enhance === 'number') extra.enhance = item.stats.enhance
  characterItemDraft.extra = JSON.stringify(extra, null, 2)
}

function inventoryLocationLabel(value: string) {
  const labels: Record<string, string> = {
    bag: '背包',
    warehouse: '仓库',
    equipped: '已装备'
  }
  return labels[value] ?? value
}

function itemLine(item: AdminInventoryItem) {
  const slot = item.equipped_slot ?? item.template_slot ?? '物品'
  const enhance = typeof item.stats.enhance === 'number' ? ` +${item.stats.enhance}` : ''
  return `${inventoryLocationLabel(item.location)} / ${slot}${enhance} / ${item.rarity}`
}

function markSynced() {
  lastSyncedAt.value = new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  }).format(new Date())
}

function formatNumber(value: number | null | undefined) {
  return typeof value === 'number' ? new Intl.NumberFormat('zh-CN').format(value) : '--'
}

function formatDate(value: string | null | undefined) {
  if (!value) return '暂无'
  const date = parseDate(value)
  if (!date) return value
  return new Intl.DateTimeFormat('zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false
  }).format(date)
}

function parseDate(value: string) {
  const trimmed = value.trim()
  const direct = new Date(trimmed)
  if (!Number.isNaN(direct.getTime())) return direct
  const normalized = trimmed
    .replace(/^(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2})(\.\d+)? \+00:00:00$/, (_match, day, time, fraction = '') => {
      const millis = fraction ? fraction.slice(0, 4).padEnd(4, '0') : ''
      return `${day}T${time}${millis}Z`
    })
    .replace(' UTC', 'Z')
    .replace(' ', 'T')
  const date = new Date(normalized)
  return Number.isNaN(date.getTime()) ? null : date
}

function statusLabel(status: string) {
  const labels: Record<string, string> = {
    active: '正常',
    locked: '锁定',
    banned: '封禁',
    disabled: '停用'
  }
  return labels[status] ?? status
}

function boolLabel(value: boolean) {
  return value ? '是' : '否'
}

function formatPosition(character: AdminCharacter) {
  const parts = [character.zone, character.room].filter(Boolean)
  return parts.length ? parts.join(' / ') : '--'
}

function formatAttachment(mail: AdminMail) {
  const parts = [`${mail.attachment_count} 件`]
  if (mail.attachment_gold) parts.push(`${formatNumber(mail.attachment_gold)} 金币`)
  if (mail.attachment_yuanbao) parts.push(`${formatNumber(mail.attachment_yuanbao)} 元宝`)
  return parts.join(' / ')
}

function formatJsonPreview(value: unknown) {
  const text = typeof value === 'string' ? value : JSON.stringify(value)
  if (!text) return '--'
  return text.length > 100 ? `${text.slice(0, 100)}...` : text
}

function averageValue(values: number[]) {
  if (!values.length) return '--'
  const total = values.reduce((sum, value) => sum + value, 0)
  return (total / values.length).toFixed(1)
}

function systemTone(status: string) {
  return status === 'ok' || status === 'ready' ? 'ok' : 'warn'
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : '请求失败'
}
</script>

<template>
  <main v-if="!adminAuthenticated" class="admin-login-screen">
    <section class="admin-login-preview">
      <div class="brand">
        <span class="brand-mark">无</span>
        <div>
          <strong>无界修真：斩妖录</strong>
          <small>GM 工作台</small>
        </div>
      </div>
      <div class="login-dashboard-grid">
        <article>
          <span>在线巡检</span>
          <strong>{{ healthz?.status ?? 'unknown' }}</strong>
          <small>/api/healthz</small>
        </article>
        <article>
          <span>数据库</span>
          <strong>{{ readyz?.status ?? 'unknown' }}</strong>
          <small>/api/readyz</small>
        </article>
        <article>
          <span>后台入口</span>
          <strong>Admin</strong>
          <small>账号密码登录</small>
        </article>
      </div>
      <div class="login-terminal">
        <p>系统记录</p>
        <code>ADMIN_BOOTSTRAP_USER</code>
        <code>ADMIN_BOOTSTRAP_PASSWORD</code>
        <span>登录后可进行账号、角色、模板、挑战测试和展示设置。</span>
      </div>
    </section>

    <section class="admin-login-panel">
      <div class="brand compact">
        <span class="brand-mark">GM</span>
        <div>
          <strong>后台登录</strong>
          <small>使用环境变量配置的管理员账号</small>
        </div>
      </div>
      <form class="admin-login-form" @submit.prevent="adminLoginAction">
        <label>
          <span>管理员账号</span>
          <input v-model="adminLogin.username" autocomplete="username" placeholder="ADMIN_BOOTSTRAP_USER" />
        </label>
        <label>
          <span>管理员密码</span>
          <input
            v-model="adminLogin.password"
            autocomplete="current-password"
            placeholder="ADMIN_BOOTSTRAP_PASSWORD"
            type="password"
          />
        </label>
        <p v-if="adminLoginError" class="alert">
          <CircleAlert :size="18" />
          <span>{{ adminLoginError }}</span>
        </p>
        <button
          class="refresh-button"
          type="submit"
          :disabled="adminLoginLoading || !adminLogin.username.trim() || !adminLogin.password"
        >
          <LogIn :size="18" />
          <span>{{ adminLoginLoading ? '登录中' : '进入后台' }}</span>
        </button>
      </form>
    </section>
  </main>

  <main v-else class="admin-shell">
    <aside class="sidebar">
      <div class="brand">
        <span class="brand-mark">无</span>
        <div>
          <strong>无界修真 GM</strong>
          <small>/admin/</small>
        </div>
      </div>

      <nav class="nav-list" aria-label="GM 管理模块">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          :class="{ active: activeTab === tab.key }"
          type="button"
          @click="selectTab(tab.key)"
        >
          <component :is="tab.icon" :size="18" />
          <span>{{ tab.label }}</span>
        </button>
      </nav>

      <div class="sidebar-status">
        <span class="status-dot" :class="{ ok: adminAuthenticated }"></span>
        <div>
          <span>{{ tokenStatus }}</span>
          <small>最近同步 {{ lastSyncedAt || '--' }}</small>
        </div>
      </div>
    </aside>

    <section class="workspace">
      <header class="topbar">
        <div>
          <p class="eyebrow">{{ currentTab.eyebrow }}</p>
          <h1>{{ currentTab.label }}</h1>
        </div>
        <div class="token-actions">
          <div class="admin-user-chip">
            <KeyRound :size="18" />
            <span>{{ adminUser || '管理员' }}</span>
          </div>
          <button class="refresh-button" type="button" :disabled="isRefreshing" @click="refreshActive">
            <RefreshCw :class="{ spin: isRefreshing }" :size="18" />
            <span>刷新</span>
          </button>
          <button class="refresh-button ghost" type="button" @click="adminLogout">
            <LogOut :size="18" />
            <span>退出</span>
          </button>
        </div>
      </header>

      <p v-if="activeError" class="alert">
        <CircleAlert :size="18" />
        <span>{{ activeError }}</span>
      </p>
      <p v-if="actionMessage" class="alert ok-alert">
        <Shield :size="18" />
        <span>{{ actionMessage }}</span>
      </p>

      <section v-if="activeTab === 'dashboard'" class="page-stack">
        <div class="metric-grid">
          <article v-for="metric in dashboardMetrics" :key="metric.label" class="metric-card">
            <span>{{ metric.label }}</span>
            <strong>{{ formatNumber(metric.value) }}</strong>
            <small>{{ metric.hint }}</small>
          </article>
        </div>

        <div class="content-grid">
          <article class="panel">
            <div class="panel-title">
              <div>
                <p class="eyebrow">运营快照</p>
                <h2>核心比例</h2>
              </div>
              <Activity :size="20" />
            </div>
            <dl class="detail-list">
              <div>
                <dt>人均角色</dt>
                <dd>{{ characterAverage }}</dd>
              </div>
              <div>
                <dt>角色邮件比</dt>
                <dd>{{ mailAverage }}</dd>
              </div>
              <div>
                <dt>在线率</dt>
                <dd>{{ onlineRate }}</dd>
              </div>
            </dl>
          </article>

          <article class="panel">
            <div class="panel-title">
              <div>
                <p class="eyebrow">接口覆盖</p>
                <h2>GM 数据面板</h2>
              </div>
              <NotebookTabs :size="20" />
            </div>
            <dl class="detail-list">
              <div>
                <dt>账号</dt>
                <dd>/api/admin/accounts</dd>
              </div>
              <div>
                <dt>角色 / 邮件</dt>
                <dd>/api/admin/characters / mail</dd>
              </div>
              <div>
                <dt>模板 / 审计</dt>
                <dd>items / mobs / audit</dd>
              </div>
            </dl>
          </article>
        </div>
      </section>

      <section v-else-if="activeTab === 'accounts'" class="page-stack">
        <div class="metric-grid compact">
          <article class="metric-card">
            <span>注册总数</span>
            <strong>{{ formatNumber(accounts?.total ?? summary?.accounts) }}</strong>
            <small>/api/admin/accounts</small>
          </article>
          <article class="metric-card">
            <span>列表样本</span>
            <strong>{{ formatNumber(accounts?.accounts.length) }}</strong>
            <small>最新 100 个账号</small>
          </article>
          <article class="metric-card">
            <span>角色总数</span>
            <strong>{{ formatNumber(summary?.characters) }}</strong>
            <small>来自仪表盘</small>
          </article>
        </div>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">GM Action</p>
              <h2>Account ban / unban</h2>
            </div>
            <Shield :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>Account ID</span>
              <input v-model="accountStatusPatch.account_id" inputmode="numeric" placeholder="1" />
            </label>
            <label>
              <span>Status</span>
              <select v-model="accountStatusPatch.status">
                <option value="banned">banned</option>
                <option value="active">active</option>
                <option value="muted">muted</option>
              </select>
            </label>
            <label class="wide-field">
              <span>Reason</span>
              <input v-model="accountStatusPatch.reason" />
            </label>
            <button class="refresh-button" type="button" :disabled="loading.accounts" @click="setAccountStatusAction">
              Save status
            </button>
          </div>
        </article>

        <article class="panel danger-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">测试清档</p>
              <h2>一键清除所有账号</h2>
            </div>
            <Trash2 :size="20" />
          </div>
          <p class="danger-copy">用于测试服快速删除所有账号、角色、会话、背包、邮件、行会和角色关联数据。操作前会再次确认。</p>
          <button class="refresh-button danger" type="button" :disabled="loading.accounts" @click="clearAllAccountsAction">
            <Trash2 :size="18" />
            <span>一键清除所有账号</span>
          </button>
        </article>

        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">账号列表</p>
              <h2>注册与角色数</h2>
            </div>
            <Users :size="20" />
          </div>
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>账号</th>
                  <th>邮箱</th>
                  <th>状态</th>
                  <th>角色</th>
                  <th>注册时间</th>
                  <th>最近登录</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="account in accountRows" :key="account.id">
                  <td>#{{ account.id }}</td>
                  <td>{{ account.username }}</td>
                  <td>{{ account.email || '未填写' }}</td>
                  <td><span class="pill">{{ statusLabel(account.status) }}</span></td>
                  <td>{{ account.character_count }}</td>
                  <td>{{ formatDate(account.created_at) }}</td>
                  <td>{{ formatDate(account.last_login_at) }}</td>
                </tr>
                <tr v-if="!accountRows.length">
                  <td colspan="7" class="empty-cell">暂无账号数据</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>

        <article class="panel admin-guide-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">后台说明</p>
              <h2>运营与排查</h2>
            </div>
            <NotebookTabs :size="20" />
          </div>
          <div class="guide-grid">
            <section>
              <strong>角色与资产</strong>
              <p>角色页可以调整等级、货币、位置、生命魔法，并查看装备、背包和仓库；物品维护支持新增、修改、删除角色物品。</p>
            </section>
            <section>
              <strong>模板配置</strong>
              <p>物品模板负责装备、药剂、技能书、材料的基础数据；怪物模板负责等级、战斗数值和区域出没。</p>
            </section>
            <section>
              <strong>验证路径</strong>
              <p>推送后先运行迁移和 smoke，再检查玩家端注册登录、探索、强化、挂机、寄售、邮件和 GM 角色详情。</p>
            </section>
            <section>
              <strong>风险操作</strong>
              <p>强化高段会消耗鸿蒙石并可能碎装；死亡惩罚会删除随机装备，GM 调整前建议确认角色 ID 和审计记录。</p>
            </section>
          </div>
        </article>

        <article class="panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">账号状态</p>
              <h2>当前样本分布</h2>
            </div>
            <Shield :size="20" />
          </div>
          <div class="status-list">
            <div v-for="row in accountStatusRows" :key="row.status" class="status-row">
              <span>{{ row.label }}</span>
              <strong>{{ row.count }}</strong>
            </div>
            <div v-if="!accountStatusRows.length" class="empty-note">暂无状态样本</div>
          </div>
        </article>
      </section>

      <section v-else-if="activeTab === 'characters'" class="page-stack">
        <div class="metric-grid compact">
          <article class="metric-card">
            <span>角色总数</span>
            <strong>{{ formatNumber(characters?.total ?? summary?.characters) }}</strong>
            <small>/api/admin/characters</small>
          </article>
          <article class="metric-card">
            <span>在线角色</span>
            <strong>{{ formatNumber(onlineCharacterCount || summary?.online) }}</strong>
            <small>character_state.online</small>
          </article>
          <article class="metric-card">
            <span>平均等级</span>
            <strong>{{ averageCharacterLevel }}</strong>
            <small>当前列表样本</small>
          </article>
          <article class="metric-card">
            <span>样本战力</span>
            <strong>{{ formatNumber(totalCharacterPower) }}</strong>
            <small>列表战力合计</small>
          </article>
        </div>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">GM 操作</p>
              <h2>角色资产调整</h2>
            </div>
            <UserRound :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>角色 ID</span>
              <input v-model="characterPatch.character_id" inputmode="numeric" placeholder="例如 1" />
            </label>
            <label>
              <span>经验增减</span>
              <input v-model.number="characterPatch.exp_delta" type="number" />
            </label>
            <label>
              <span>金币增减</span>
              <input v-model.number="characterPatch.gold_delta" type="number" />
            </label>
            <label>
              <span>元宝增减</span>
              <input v-model.number="characterPatch.yuanbao_delta" type="number" />
            </label>
            <label class="wide-field">
              <span>原因</span>
              <input v-model="characterPatch.reason" />
            </label>
            <button class="refresh-button" type="button" :disabled="loading.characters" @click="adjustCharacterAction">
              执行调整
            </button>
          </div>
        </article>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">GM 操作</p>
              <h2>角色位置与踢下线</h2>
            </div>
            <UserRound :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>角色 ID</span>
              <input v-model="characterStatePatch.character_id" inputmode="numeric" placeholder="1" />
            </label>
            <label>
              <span>区域</span>
              <input v-model="characterStatePatch.zone" placeholder="可选" />
            </label>
            <label>
              <span>房间</span>
              <input v-model="characterStatePatch.room" placeholder="可选" />
            </label>
            <label>
              <span>生命</span>
              <input v-model="characterStatePatch.hp" inputmode="numeric" placeholder="可选" />
            </label>
            <label>
              <span>魔法</span>
              <input v-model="characterStatePatch.mp" inputmode="numeric" placeholder="可选" />
            </label>
            <label>
              <span>在线</span>
              <select v-model="characterStatePatch.online">
                <option :value="true">在线</option>
                <option :value="false">离线</option>
              </select>
            </label>
            <label>
              <span>踢下线</span>
              <select v-model="characterStatePatch.force_offline">
                <option :value="true">标记踢下线</option>
                <option :value="false">不踢下线</option>
              </select>
            </label>
            <label class="wide-field">
              <span>原因</span>
              <input v-model="characterStatePatch.reason" />
            </label>
            <button class="refresh-button" type="button" :disabled="loading.characters" @click="setCharacterStateAction">
              保存状态
            </button>
          </div>
        </article>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">角色详情</p>
              <h2>属性、装备、背包与仓库</h2>
            </div>
            <Package :size="20" />
          </div>
          <div v-if="selectedCharacter" class="operation-grid">
            <label>
              <span>角色 ID</span>
              <input v-model="characterDetailPatch.character_id" disabled />
            </label>
            <label>
              <span>角色名</span>
              <input v-model="characterDetailPatch.name" />
            </label>
            <label>
              <span>职业</span>
              <select v-model="characterDetailPatch.class">
                <option value="warrior">剑修</option>
                <option value="mage">法修</option>
                <option value="taoist">魂修</option>
                <option value="assassin">刺客</option>
              </select>
            </label>
            <label>
              <span>等级</span>
              <input v-model.number="characterDetailPatch.level" type="number" min="1" />
            </label>
            <label>
              <span>经验</span>
              <input v-model.number="characterDetailPatch.exp" type="number" min="0" />
            </label>
            <label>
              <span>金币</span>
              <input v-model.number="characterDetailPatch.gold" type="number" min="0" />
            </label>
            <label>
              <span>元宝</span>
              <input v-model.number="characterDetailPatch.yuanbao" type="number" min="0" />
            </label>
            <label>
              <span>战力</span>
              <input v-model.number="characterDetailPatch.power" type="number" min="0" />
            </label>
            <label>
              <span>区域</span>
              <input v-model="characterDetailPatch.zone" />
            </label>
            <label>
              <span>房间</span>
              <input v-model="characterDetailPatch.room" />
            </label>
            <label>
              <span>生命</span>
              <input v-model="characterDetailPatch.hp" inputmode="numeric" />
            </label>
            <label>
              <span>魔法</span>
              <input v-model="characterDetailPatch.mp" inputmode="numeric" />
            </label>
            <label>
              <span>在线</span>
              <select v-model="characterDetailPatch.online">
                <option :value="true">在线</option>
                <option :value="false">离线</option>
              </select>
            </label>
            <label class="wide-field">
              <span>原因</span>
              <input v-model="characterDetailPatch.reason" />
            </label>
            <button class="refresh-button" type="button" :disabled="loading.characters" @click="saveCharacterDetailAction">
              保存角色详情
            </button>
          </div>
          <p v-else class="empty-cell">从下方角色列表选择一个角色后，可查看并修改装备、背包和仓库。</p>
          <div v-if="selectedCharacter" class="detail-list">
            <div>
              <dt>基础属性</dt>
              <dd>
                攻 {{ selectedCharacter.stats.atk }} / 防 {{ selectedCharacter.stats.def }} / 魔 {{ selectedCharacter.stats.mag }} / 魔防 {{ selectedCharacter.stats.mdef }}
              </dd>
            </div>
            <div>
              <dt>背包容量</dt>
              <dd>{{ selectedCharacter.inventory.summary.bag_used }}/{{ selectedCharacter.inventory.summary.bag_limit }}，仓库 {{ selectedCharacter.inventory.summary.warehouse_used }}</dd>
            </div>
            <div>
              <dt>物品分布</dt>
              <dd>装备 {{ selectedEquippedItems.length }} / 背包 {{ selectedBagItems.length }} / 仓库 {{ selectedWarehouseItems.length }}</dd>
            </div>
          </div>
        </article>

        <article v-if="selectedCharacter" class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">物品维护</p>
              <h2>新增、修改、删除角色物品</h2>
            </div>
            <Package :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>物品 ID</span>
              <input v-model="characterItemDraft.item_id" placeholder="留空新增" />
            </label>
            <label>
              <span>模板</span>
              <select v-model="characterItemDraft.template_id">
                <option v-for="item in itemRows" :key="item.id" :value="item.id">{{ item.name }} / {{ item.id }}</option>
              </select>
            </label>
            <label>
              <span>数量</span>
              <input v-model.number="characterItemDraft.quantity" type="number" min="1" />
            </label>
            <label>
              <span>位置</span>
              <select v-model="characterItemDraft.location">
                <option value="bag">背包</option>
                <option value="warehouse">仓库</option>
                <option value="equipped">已装备</option>
              </select>
            </label>
            <label>
              <span>装备槽</span>
              <input v-model="characterItemDraft.slot" placeholder="weapon/chest/ring_left" />
            </label>
            <label>
              <span>绑定</span>
              <select v-model="characterItemDraft.bind">
                <option :value="false">未绑定</option>
                <option :value="true">绑定</option>
              </select>
            </label>
            <label>
              <span>耐久</span>
              <input v-model.number="characterItemDraft.durability" type="number" min="0" max="100" />
            </label>
            <label class="wide-field">
              <span>扩展 JSON</span>
              <textarea v-model="characterItemDraft.extra" rows="3"></textarea>
            </label>
            <button class="refresh-button" type="button" :disabled="loading.characters" @click="saveCharacterItemAction">
              保存物品
            </button>
            <button class="refresh-button ghost" type="button" @click="resetCharacterItemDraft">清空表单</button>
          </div>
          <div class="table-wrap xl-table">
            <table>
              <thead>
                <tr>
                  <th>物品</th>
                  <th>位置</th>
                  <th>数量</th>
                  <th>绑定</th>
                  <th>耐久</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in selectedInventoryItems" :key="item.id">
                  <td>
                    <span class="cell-primary">
                      <strong>{{ item.name }}</strong>
                      <small>#{{ item.id }} / {{ item.template_id }}</small>
                    </span>
                  </td>
                  <td>{{ itemLine(item) }}</td>
                  <td>{{ item.quantity }}</td>
                  <td>{{ item.bind ? '绑定' : '未绑定' }}</td>
                  <td>{{ item.durability }}</td>
                  <td>
                    <button class="mini-button" type="button" @click="editCharacterItem(item)">编辑</button>
                    <button class="mini-button danger" type="button" @click="deleteCharacterItemAction(item)">删除</button>
                  </td>
                </tr>
                <tr v-if="!selectedInventoryItems.length">
                  <td colspan="6" class="empty-cell">暂无装备、背包或仓库物品</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>

        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">角色列表</p>
              <h2>账号、资产与位置</h2>
            </div>
            <UserRound :size="20" />
          </div>
          <div class="table-wrap xl-table">
            <table>
              <thead>
                <tr>
                  <th>角色</th>
                  <th>账号</th>
                  <th>职业</th>
                  <th>等级</th>
                  <th>经验</th>
                  <th>金币</th>
                  <th>元宝</th>
                  <th>战力</th>
                  <th>位置</th>
                  <th>在线</th>
                  <th>更新时间</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="character in characterRows" :key="character.id">
                  <td>
                    <span class="cell-primary">
                      <strong>{{ character.name }}</strong>
                      <small>#{{ character.id }}</small>
                    </span>
                  </td>
                  <td>
                    <span class="cell-primary">
                      <strong>{{ character.account_username }}</strong>
                      <small>#{{ character.account_id }}</small>
                    </span>
                  </td>
                  <td>{{ character.class }}</td>
                  <td>{{ character.level }}</td>
                  <td>{{ formatNumber(character.exp) }}</td>
                  <td>{{ formatNumber(character.gold) }}</td>
                  <td>{{ formatNumber(character.yuanbao) }}</td>
                  <td>{{ formatNumber(character.power) }}</td>
                  <td>{{ formatPosition(character) }}</td>
                  <td>
                    <span class="pill" :class="character.online ? 'ok' : 'neutral'">
                      {{ character.online ? '在线' : '离线' }}
                    </span>
                  </td>
                  <td>{{ formatDate(character.updated_at) }}</td>
                  <td>
                    <button class="mini-button" type="button" @click="loadCharacterDetail(character.id)">详情</button>
                  </td>
                </tr>
                <tr v-if="!characterRows.length">
                  <td colspan="12" class="empty-cell">暂无角色数据</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </section>

      <section v-else-if="activeTab === 'mail'" class="page-stack">
        <div class="metric-grid four">
          <article class="metric-card">
            <span>total</span>
            <strong>{{ formatNumber(mailOverview?.total ?? summary?.mails) }}</strong>
            <small>邮件总量</small>
          </article>
          <article class="metric-card">
            <span>unread</span>
            <strong>{{ formatNumber(mailOverview?.unread) }}</strong>
            <small>未读邮件</small>
          </article>
          <article class="metric-card">
            <span>unclaimed</span>
            <strong>{{ formatNumber(mailOverview?.unclaimed) }}</strong>
            <small>未领取附件</small>
          </article>
          <article class="metric-card">
            <span>with_attachments</span>
            <strong>{{ formatNumber(mailOverview?.with_attachments) }}</strong>
            <small>含附件邮件</small>
          </article>
        </div>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">GM 操作</p>
              <h2>发送系统邮件</h2>
            </div>
            <Mail :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>角色 ID</span>
              <input v-model="mailDraft.to_character_id" inputmode="numeric" placeholder="留空为全服" />
            </label>
            <label>
              <span>金币</span>
              <input v-model.number="mailDraft.gold" type="number" min="0" />
            </label>
            <label>
              <span>元宝</span>
              <input v-model.number="mailDraft.yuanbao" type="number" min="0" />
            </label>
            <label>
              <span>物品模板</span>
              <input v-model="mailDraft.item_template_id" placeholder="potion_small" />
            </label>
            <label>
              <span>数量</span>
              <input v-model.number="mailDraft.quantity" type="number" min="1" />
            </label>
            <label>
              <span>标题</span>
              <input v-model="mailDraft.title" />
            </label>
            <label class="wide-field">
              <span>正文</span>
              <textarea v-model="mailDraft.body" rows="3"></textarea>
            </label>
            <button class="refresh-button" type="button" :disabled="loading.mail" @click="sendMailAction">
              发送邮件
            </button>
          </div>
        </article>

        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">recent</p>
              <h2>最近邮件</h2>
            </div>
            <Mail :size="20" />
          </div>
          <div class="table-wrap xl-table">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>收件角色</th>
                  <th>账号</th>
                  <th>发件人</th>
                  <th>标题</th>
                  <th>已读</th>
                  <th>已领取</th>
                  <th>附件</th>
                  <th>到期</th>
                  <th>创建时间</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="mail in mailRows" :key="mail.id">
                  <td>#{{ mail.id }}</td>
                  <td>
                    <span class="cell-primary">
                      <strong>{{ mail.to_character_name || '未知角色' }}</strong>
                      <small>#{{ mail.to_character_id }}</small>
                    </span>
                  </td>
                  <td>{{ mail.account_username || '--' }}</td>
                  <td>{{ mail.from_name }}</td>
                  <td>{{ mail.title }}</td>
                  <td><span class="pill" :class="mail.read ? 'ok' : 'warn'">{{ boolLabel(mail.read) }}</span></td>
                  <td><span class="pill" :class="mail.claimed ? 'ok' : 'warn'">{{ boolLabel(mail.claimed) }}</span></td>
                  <td>{{ formatAttachment(mail) }}</td>
                  <td>{{ formatDate(mail.expires_at) }}</td>
                  <td>{{ formatDate(mail.created_at) }}</td>
                </tr>
                <tr v-if="!mailRows.length">
                  <td colspan="10" class="empty-cell">暂无邮件数据</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </section>

      <section v-else-if="activeTab === 'items'" class="page-stack">
        <div class="metric-grid compact">
          <article class="metric-card">
            <span>模板总数</span>
            <strong>{{ formatNumber(itemTemplates?.total) }}</strong>
            <small>/api/admin/items</small>
          </article>
          <article class="metric-card">
            <span>可堆叠</span>
            <strong>{{ formatNumber(stackableItemCount) }}</strong>
            <small>stackable = true</small>
          </article>
          <article class="metric-card">
            <span>物品类型</span>
            <strong>{{ formatNumber(itemKindCount) }}</strong>
            <small>kind 去重</small>
          </article>
        </div>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">CRUD</p>
              <h2>Item template</h2>
            </div>
            <Package :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>ID</span>
              <input v-model="itemTemplateDraft.id" />
            </label>
            <label>
              <span>Name</span>
              <input v-model="itemTemplateDraft.name" />
            </label>
            <label>
              <span>Kind</span>
              <input v-model="itemTemplateDraft.kind" />
            </label>
            <label>
              <span>Slot</span>
              <input v-model="itemTemplateDraft.slot" placeholder="optional" />
            </label>
            <label>
              <span>Rarity</span>
              <input v-model="itemTemplateDraft.rarity" />
            </label>
            <label>
              <span>Price</span>
              <input v-model.number="itemTemplateDraft.price" type="number" min="0" />
            </label>
            <label>
              <span>Stackable</span>
              <select v-model="itemTemplateDraft.stackable">
                <option :value="true">true</option>
                <option :value="false">false</option>
              </select>
            </label>
            <label class="wide-field">
              <span>Stats JSON</span>
              <textarea v-model="itemTemplateDraft.stats" rows="4"></textarea>
            </label>
            <label class="wide-field">
              <span>Flags JSON</span>
              <textarea v-model="itemTemplateDraft.flags" rows="4"></textarea>
            </label>
            <button class="refresh-button" type="button" :disabled="loading.items" @click="saveItemTemplateAction">
              Save item
            </button>
          </div>
        </article>

        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">模板数据</p>
              <h2>物品配置</h2>
            </div>
            <Package :size="20" />
          </div>
          <div class="table-wrap xl-table">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>名称</th>
                  <th>类型</th>
                  <th>部位</th>
                  <th>稀有度</th>
                  <th>价格</th>
                  <th>堆叠</th>
                  <th>版本</th>
                  <th>属性</th>
                  <th>标记</th>
                  <th>更新时间</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="item in itemRows" :key="item.id">
                  <td>{{ item.id }}</td>
                  <td>{{ item.name }}</td>
                  <td>{{ item.kind }}</td>
                  <td>{{ item.slot || '--' }}</td>
                  <td><span class="pill">{{ item.rarity }}</span></td>
                  <td>{{ formatNumber(item.price) }}</td>
                  <td>{{ boolLabel(item.stackable) }}</td>
                  <td>v{{ item.version }}</td>
                  <td class="json-cell">{{ formatJsonPreview(item.stats) }}</td>
                  <td class="json-cell">{{ formatJsonPreview(item.flags) }}</td>
                  <td>{{ formatDate(item.updated_at) }}</td>
                  <td>
                    <button class="table-action" type="button" @click="editItemTemplate(item)">Edit</button>
                  </td>
                </tr>
                <tr v-if="!itemRows.length">
                  <td colspan="12" class="empty-cell">暂无物品模板数据</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </section>

      <section v-else-if="activeTab === 'mobs'" class="page-stack">
        <div class="metric-grid compact">
          <article class="metric-card">
            <span>模板总数</span>
            <strong>{{ formatNumber(mobTemplates?.total) }}</strong>
            <small>/api/admin/mobs</small>
          </article>
          <article class="metric-card">
            <span>Boss 模板</span>
            <strong>{{ formatNumber(bossMobCount) }}</strong>
            <small>boss = true</small>
          </article>
          <article class="metric-card">
            <span>平均等级</span>
            <strong>{{ averageMobLevel }}</strong>
            <small>当前模板样本</small>
          </article>
        </div>

        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">模板数据</p>
              <h2>怪物配置</h2>
            </div>
            <Swords :size="20" />
          </div>
          <div class="table-wrap xl-table">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>名称</th>
                  <th>等级</th>
                  <th>HP</th>
                  <th>攻击</th>
                  <th>防御</th>
                  <th>经验</th>
                  <th>金币</th>
                  <th>Boss</th>
                  <th>刷新秒</th>
                  <th>掉落</th>
                  <th>版本</th>
                  <th>更新时间</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="mob in mobRows" :key="mob.id">
                  <td>{{ mob.id }}</td>
                  <td>{{ mob.name }}</td>
                  <td>{{ mob.level }}</td>
                  <td>{{ formatNumber(mob.max_hp) }}</td>
                  <td>{{ formatNumber(mob.atk) }}</td>
                  <td>{{ formatNumber(mob.def) }}</td>
                  <td>{{ formatNumber(mob.exp) }}</td>
                  <td>{{ formatNumber(mob.gold) }}</td>
                  <td><span class="pill" :class="mob.boss ? 'warn' : 'neutral'">{{ boolLabel(mob.boss) }}</span></td>
                  <td>{{ formatNumber(mob.respawn_seconds) }}</td>
                  <td class="json-cell">{{ formatJsonPreview(mob.drops) }}</td>
                  <td>v{{ mob.version }}</td>
                  <td>{{ formatDate(mob.updated_at) }}</td>
                </tr>
                <tr v-if="!mobRows.length">
                  <td colspan="13" class="empty-cell">暂无怪物模板数据</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </section>

      <section v-else-if="activeTab === 'bots'" class="page-stack">
        <div class="metric-grid compact">
          <article class="metric-card">
            <span>Bot 总数</span>
            <strong>{{ formatNumber(bots?.total) }}</strong>
            <small>/api/admin/bots</small>
          </article>
          <article class="metric-card">
            <span>启用中</span>
            <strong>{{ formatNumber(enabledBotCount) }}</strong>
            <small>enabled = true</small>
          </article>
          <article class="metric-card">
            <span>平均等级</span>
            <strong>{{ averageBotLevel }}</strong>
            <small>当前 bot 样本</small>
          </article>
        </div>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">批量调度</p>
              <h2>Bot 脚本设定</h2>
            </div>
            <Bot :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>Bot 范围</span>
              <select v-model="botBatch.bot_id">
                <option value="all">全部 Bot</option>
                <option v-for="bot in botRows" :key="bot.id" :value="String(bot.id)">
                  #{{ bot.id }} · {{ bot.name }}
                </option>
              </select>
            </label>
            <label>
              <span>模式</span>
              <select v-model="botBatch.mode">
                <option value="progression">自然成长</option>
                <option value="dispatch">统一调度</option>
                <option value="team_farm">同图组队</option>
                <option value="fixed_clear">固定清场</option>
              </select>
            </label>
            <label>
              <span>启用</span>
              <select v-model="botBatch.enabled">
                <option :value="true">启用</option>
                <option :value="false">停用</option>
              </select>
            </label>
            <label>
              <span>当前位置 Zone</span>
              <select v-model="botBatch.zone">
                <option v-for="option in botZoneOptions" :key="`zone-${option.value}`" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <label>
              <span>当前位置 Room</span>
              <select v-model="botBatch.room">
                <option v-for="option in botRoomOptions" :key="`room-${option.value}`" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <label>
              <span>队伍编号</span>
              <select v-model="botBatch.team_code">
                <option v-for="option in botTargetOptions" :key="`team-${option.value}`" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <label>
              <span>目标 Zone</span>
              <select v-model="botBatch.target_zone">
                <option v-for="option in botZoneOptions" :key="`target-zone-${option.value}`" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <label>
              <span>目标 Room</span>
              <select v-model="botBatch.target_room">
                <option v-for="option in botRoomOptions" :key="`target-room-${option.value}`" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <button class="refresh-button" type="button" :disabled="loading.bots" @click="saveBotBatchAction">
              保存调度
            </button>
          </div>
        </article>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">Bot 增删</p>
              <h2>角色脚本入口</h2>
            </div>
            <Bot :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>新增名称</span>
              <input v-model="botCreate.name" maxlength="16" placeholder="自定义角色名" />
            </label>
            <label>
              <span>职业</span>
              <select v-model="botCreate.bot_class">
                <option value="warrior">剑修</option>
                <option value="mage">法修</option>
                <option value="taoist">魂修</option>
                <option value="assassin">刺客</option>
              </select>
            </label>
            <button class="refresh-button" type="button" :disabled="loading.bots || !botCreate.name.trim()" @click="createBotAction">
              增加 Bot
            </button>
            <label>
              <span>删除对象</span>
              <select v-model="botDelete.bot_id">
                <option value="">选择 Bot</option>
                <option v-for="bot in botRows" :key="`delete-${bot.id}`" :value="String(bot.id)">
                  #{{ bot.id }} · {{ bot.name }}
                </option>
              </select>
            </label>
            <button class="refresh-button danger" type="button" :disabled="loading.bots || !botDelete.bot_id" @click="deleteBotAction">
              删除 Bot
            </button>
          </div>
        </article>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">脚本推进</p>
              <h2>手动 tick</h2>
            </div>
            <Activity :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>推进数量</span>
              <select v-model.number="botBatch.tick_limit">
                <option v-for="option in botTickOptions" :key="`tick-${option.value}`" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </label>
            <button class="refresh-button" type="button" :disabled="loading.bots" @click="tickBotsAction">
              推进脚本
            </button>
            <button class="refresh-button ghost" type="button" :disabled="loading.bots" @click="loadBots">
              刷新列表
            </button>
          </div>
        </article>

        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">Bot 列表</p>
              <h2>成长、位置与调度状态</h2>
            </div>
            <Bot :size="20" />
          </div>
          <div class="table-wrap xl-table">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>名称</th>
                  <th>职业</th>
                  <th>等级</th>
                  <th>金币</th>
                  <th>战力</th>
                  <th>位置</th>
                  <th>模式</th>
                  <th>队伍</th>
                  <th>目标</th>
                  <th>启用</th>
                  <th>最近行动</th>
                  <th>脚本</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="bot in botRows" :key="bot.id">
                  <td>#{{ bot.id }}</td>
                  <td>{{ bot.name }}</td>
                  <td>{{ bot.bot_class }}</td>
                  <td>{{ bot.level }}</td>
                  <td>{{ formatNumber(bot.gold) }}</td>
                  <td>{{ formatNumber(bot.power) }}</td>
                  <td>{{ bot.zone }} / {{ bot.room }}</td>
                  <td><span class="pill">{{ bot.mode }}</span></td>
                  <td>{{ bot.team_code || '--' }}</td>
                  <td>{{ bot.target_zone || '--' }} / {{ bot.target_room || '--' }}</td>
                  <td><span class="pill" :class="bot.enabled ? 'ok' : 'neutral'">{{ boolLabel(bot.enabled) }}</span></td>
                  <td>{{ formatDate(bot.last_action_at) }}</td>
                  <td class="json-cell">{{ formatJsonPreview(bot.script) }}</td>
                </tr>
                <tr v-if="!botRows.length">
                  <td colspan="13" class="empty-cell">暂无 bot 数据</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </section>

      <section v-else-if="activeTab === 'test'" class="page-stack">
        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">挑战冷却</p>
              <h2>重置挑战延时</h2>
            </div>
            <FlaskConical :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>角色 ID</span>
              <input v-model="testTools.character_id" placeholder="留空则重置全部" />
            </label>
            <button
              class="refresh-button"
              type="button"
              :disabled="loading.test"
              @click="resetChallengeCooldownsAction"
            >
              重置秘境/无尽塔/世界首领
            </button>
          </div>
        </article>

        <article class="panel admin-guide-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">用途</p>
              <h2>测试说明</h2>
            </div>
            <NotebookTabs :size="20" />
          </div>
          <ul class="guide-list">
            <li>留空角色 ID 会重置所有已有挑战状态记录。</li>
            <li>填写角色 ID 会先创建该角色挑战状态，再清空三个挑战冷却。</li>
            <li>此操作写入 GM 审计，用于平衡测试和功能回归。</li>
          </ul>
        </article>
      </section>

      <section v-else-if="activeTab === 'backup'" class="page-stack">
        <div class="metric-grid compact">
          <article class="metric-card">
            <span>备份队列</span>
            <strong>{{ formatNumber(summary?.pending_backups) }}</strong>
            <small>queued / running</small>
          </article>
          <article class="metric-card">
            <span>数据来源</span>
            <strong>dashboard</strong>
            <small>/api/admin/dashboard</small>
          </article>
          <article class="metric-card">
            <span>最近同步</span>
            <strong>{{ lastSyncedAt || '--' }}</strong>
            <small>手动刷新当前页</small>
          </article>
        </div>
        <article class="panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">备份监控</p>
              <h2>队列状态</h2>
            </div>
            <HardDrive :size="20" />
          </div>
          <dl class="detail-list">
            <div>
              <dt>排队或执行中任务</dt>
              <dd>{{ formatNumber(summary?.pending_backups) }}</dd>
            </div>
            <div>
              <dt>登录状态</dt>
              <dd>{{ tokenStatus }}</dd>
            </div>
            <div>
              <dt>页面模式</dt>
              <dd>队列监控</dd>
            </div>
          </dl>
        </article>
      </section>

      <section v-else-if="activeTab === 'audit'" class="page-stack">
        <div class="metric-grid compact">
          <article class="metric-card">
            <span>日志总数</span>
            <strong>{{ formatNumber(auditLogs?.total) }}</strong>
            <small>/api/admin/audit</small>
          </article>
          <article class="metric-card">
            <span>列表样本</span>
            <strong>{{ formatNumber(auditRows.length) }}</strong>
            <small>最新 100 条</small>
          </article>
          <article class="metric-card">
            <span>登录状态</span>
            <strong>{{ tokenStatus }}</strong>
            <small>管理员会话</small>
          </article>
        </div>

        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">审计日志</p>
              <h2>GM 操作记录</h2>
            </div>
            <ScrollText :size="20" />
          </div>
          <div class="table-wrap xl-table">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>时间</th>
                  <th>管理员</th>
                  <th>action</th>
                  <th>target</th>
                  <th>IP</th>
                  <th>detail</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="log in auditRows" :key="log.id">
                  <td>#{{ log.id }}</td>
                  <td>{{ formatDate(log.created_at) }}</td>
                  <td>{{ log.admin_username || log.admin_account_id || '--' }}</td>
                  <td><span class="pill">{{ log.action }}</span></td>
                  <td>{{ log.target }}</td>
                  <td>{{ log.ip || '--' }}</td>
                  <td class="json-cell">{{ formatJsonPreview(log.detail) }}</td>
                </tr>
                <tr v-if="!auditRows.length">
                  <td colspan="7" class="empty-cell">暂无审计日志</td>
                </tr>
              </tbody>
            </table>
          </div>
        </article>
      </section>

      <section v-else-if="activeTab === 'settings'" class="page-stack">
        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">展示设置</p>
              <h2>请作者喝咖啡二维码</h2>
            </div>
            <Coffee :size="20" />
          </div>
          <div class="operation-grid settings-grid">
            <label class="wide-field">
              <span>二维码图片地址或 data URL</span>
              <textarea
                v-model="adminSettings.coffee_qr_url"
                placeholder="https://... 或 data:image/png;base64,..."
                rows="5"
              ></textarea>
            </label>
            <div class="qr-preview">
              <img v-if="adminSettings.coffee_qr_url.trim()" :src="adminSettings.coffee_qr_url.trim()" alt="请作者喝咖啡二维码预览" />
              <span v-else>暂无二维码</span>
            </div>
            <button class="refresh-button" type="button" :disabled="loading.settings" @click="saveSettingsAction">
              保存设置
            </button>
          </div>
        </article>

        <article class="panel admin-guide-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">前端展示</p>
              <h2>玩家端入口</h2>
            </div>
            <NotebookTabs :size="20" />
          </div>
          <div class="guide-grid">
            <section>
              <strong>入口位置</strong>
              <p>玩家端“关于”按钮后方展示“请作者喝咖啡”，点击后读取此处保存的二维码。</p>
            </section>
            <section>
              <strong>关闭方式</strong>
              <p>二维码弹窗点击任意位置即可关闭；未配置时会展示暂未设置提示。</p>
            </section>
          </div>
        </article>
      </section>

      <section v-else class="page-stack">
        <article class="panel table-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">服务探活</p>
              <h2>HTTP / DB / Admin</h2>
            </div>
            <Server :size="20" />
          </div>
          <div class="system-grid">
            <div v-for="item in systemRows" :key="item.name" class="system-row">
              <span class="status-dot" :class="systemTone(item.status)"></span>
              <div>
                <strong>{{ item.label }}</strong>
                <small>{{ item.name }}</small>
              </div>
              <span class="pill" :class="systemTone(item.status)">{{ item.status }}</span>
              <p>{{ item.detail }}</p>
            </div>
          </div>
        </article>

        <article class="panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">刷新时间</p>
              <h2>同步状态</h2>
            </div>
            <Clock3 :size="20" />
          </div>
          <dl class="detail-list">
            <div>
              <dt>最近同步</dt>
              <dd>{{ lastSyncedAt || '--' }}</dd>
            </div>
            <div>
              <dt>管理路径</dt>
              <dd>/admin/</dd>
            </div>
          </dl>
        </article>

        <article class="panel operation-panel">
          <div class="panel-title">
            <div>
              <p class="eyebrow">GM 操作</p>
              <h2>活动开关</h2>
            </div>
            <Activity :size="20" />
          </div>
          <div class="operation-grid">
            <label>
              <span>活动 code</span>
              <input v-model="activityToggle.code" placeholder="daily_hunt" />
            </label>
            <label>
              <span>状态</span>
              <select v-model="activityToggle.enabled">
                <option :value="true">开启</option>
                <option :value="false">关闭</option>
              </select>
            </label>
            <button class="refresh-button" type="button" :disabled="loading.system" @click="toggleActivityAction">
              保存活动
            </button>
          </div>
        </article>
      </section>
    </section>
  </main>
</template>
