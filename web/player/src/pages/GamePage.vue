<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { LogOut, Plus } from '@lucide/vue'
import GameShell from '../components/GameShell.vue'
import StatusBar from '../components/StatusBar.vue'
import { useGameStore } from '../stores/game'
import { useSessionStore } from '../stores/session'
import type { CharacterClass } from '../shared/types'

const session = useSessionStore()
const game = useGameStore()
const name = ref('')
const characterClass = ref<CharacterClass>('warrior')
const canPlay = computed(() => Boolean(session.token && game.activeCharacterId))

onMounted(async () => {
  await bootGame()
})

async function bootGame() {
  try {
    if (!session.token) return
    game.connected = false
    game.authenticated = false
    await loadGameData()
    if (game.activeCharacterId) game.connect(session.token)
  } catch (error) {
    handleBootError(error)
  }
}

async function loadGameData() {
  await game.loadCharacters(session.token)
  await game.loadOverview(session.token)
  await game.loadBootstrap(session.token)
  await game.loadRoom(session.token)
  await game.loadInventory(session.token)
  await game.loadSkills(session.token)
  await game.loadQuests(session.token)
  await game.loadGuildApplications(session.token)
  await game.loadAfk(session.token)
  await game.loadMail(session.token)
  await game.loadTrade(session.token)
}

function handleBootError(error: unknown) {
  const message = error instanceof Error ? error.message : '游戏数据加载失败'
  if (message.includes('请先登录') || message.includes('unauthorized')) {
    session.logout()
    return
  }
  game.log.unshift(message)
}

async function create() {
  await game.createCharacter(session.token, name.value.trim(), characterClass.value)
  await loadGameData()
  game.connect(session.token)
}

async function refreshAll() {
  await game.refreshState(session.token)
  await game.loadOverview(session.token)
  await game.loadInventory(session.token)
  await game.loadSkills(session.token)
  await game.loadQuests(session.token)
  await game.loadGuildApplications(session.token)
  await game.loadAfk(session.token)
  await game.loadMail(session.token)
  await game.loadTrade(session.token)
}

async function listTradeItem(itemId: number, suggestedPrice: number) {
  const rawCurrency = window.prompt('请输入寄售币种：yuanbao 或 gold', 'yuanbao')
  const currency = rawCurrency?.trim() === 'gold' ? 'gold' : rawCurrency?.trim() === 'yuanbao' ? 'yuanbao' : ''
  if (!currency) {
    game.log.unshift('寄售币种只能选择 yuanbao 或 gold')
    return
  }
  const label = currency === 'gold' ? '金币' : '元宝'
  const raw = window.prompt(`请输入寄售价格（${label}）`, String(suggestedPrice))
  if (!raw) return
  const price = Number(raw)
  if (!Number.isFinite(price) || price <= 0) {
    game.log.unshift('寄售价格必须大于 0')
    return
  }
  await game.listTradeItem(session.token, itemId, Math.floor(price), currency)
}

async function redeemRecharge() {
  const code = window.prompt('请输入充值卡兑换码')
  if (!code?.trim()) return
  await game.redeemRechargeCard(session.token, code.trim())
}
</script>

<template>
  <main class="game-screen">
    <header class="topbar">
      <div class="brand-row">
        <span class="brand-mark small">无</span>
        <div>
          <strong>无界修真：斩妖录</strong>
          <span>开放地图 / 放置修真</span>
        </div>
      </div>
      <button class="icon-button" title="退出" @click="session.logout()">
        <LogOut :size="18" />
      </button>
    </header>

    <section v-if="!game.activeCharacter" class="create-character">
      <h2>创建角色</h2>
      <div class="form-grid">
        <input v-model="name" placeholder="角色名" />
        <select v-model="characterClass">
          <option value="warrior">剑修</option>
          <option value="mage">法修</option>
          <option value="taoist">魂修</option>
        </select>
        <button class="primary-button" @click="create">
          <Plus :size="18" />
          创建
        </button>
      </div>
    </section>

    <template v-else>
      <StatusBar
        :character="game.activeCharacter"
        :bundle="game.activeBundle"
        :inventory="game.inventory"
        :skills="game.skills"
        :connected="game.connected"
        :authenticated="game.authenticated"
        :playable="canPlay"
      />
      <GameShell
        :room-state="game.room"
        :log="game.log"
        :mobs="game.mobs"
        :overview="game.overview"
        :inventory="game.inventory"
        :skills="game.skills"
        :quests="game.quests"
        :mails="game.mails"
        :guild-applications="game.guildApplications"
        :consignments="game.consignments"
        :afk="game.afk"
        :zone="game.activeBundle?.state.zone ?? ''"
        :authenticated="canPlay"
        :last-event-at="game.lastEventAt"
        :bundle="game.activeBundle"
        :pending-adventure="game.pendingAdventure"
        @explore="game.explore(session.token)"
        @explore-secret-realm="game.exploreSecretRealm(session.token)"
        @challenge-tower="game.challengeTower(session.token)"
        @challenge-world-boss="game.challengeWorldBoss(session.token)"
        @wild-afk="game.wildAfk(session.token)"
        @attack-bot="(targetIndex) => game.attackBot(session.token, targetIndex)"
        @move="(direction) => game.move(session.token, direction)"
        @teleport="(zone, room) => game.teleport(session.token, zone, room)"
        @refresh="refreshAll"
        @equip="(itemId) => game.equip(session.token, itemId)"
        @unequip="(itemId) => game.unequip(session.token, itemId)"
        @use-item="(itemId) => game.useItem(session.token, itemId)"
        @enhance-item="(itemId) => game.enhanceItem(session.token, itemId)"
        @store-item="(itemId) => game.storeItem(session.token, itemId)"
        @retrieve-item="(itemId) => game.retrieveItem(session.token, itemId)"
        @decompose-equipment="(rarities, itemIds) => game.decomposeEquipment(session.token, rarities, itemIds)"
        @decompose-misc="(kinds, itemIds) => game.decomposeMisc(session.token, kinds, itemIds)"
        @buy-shop-item="(templateId, quantity) => game.buyShopItem(session.token, templateId, quantity)"
        @buy-yuanbao-item="(templateId, quantity) => game.buyYuanbaoItem(session.token, templateId, quantity)"
        @update-vip-potion-settings="(settings) => game.updateVipPotionSettings(session.token, settings)"
        @exchange-npc-material="(materialId) => game.exchangeNpcMaterial(session.token, materialId)"
        @upgrade-battle-instinct="game.upgradeBattleInstinct(session.token)"
        @upgrade-special-skill="(skillId) => game.upgradeSpecialSkill(session.token, skillId)"
        @redeem-recharge="redeemRecharge"
        @list-trade-item="listTradeItem"
        @buy-trade-item="(consignmentId) => game.buyTradeItem(session.token, consignmentId)"
        @cancel-trade-item="(consignmentId) => game.cancelTradeItem(session.token, consignmentId)"
        @learn-skill="(skillId) => game.learnSkill(session.token, skillId)"
        @upgrade-skill="(skillId) => game.upgradeSkill(session.token, skillId)"
        @toggle-skill-auto="(skillId, autoEnabled) => game.toggleSkillAuto(session.token, skillId, autoEnabled)"
        @claim-quest="(questId) => game.claimQuest(session.token, questId)"
        @create-guild="(name) => game.createGuild(session.token, name)"
        @apply-guild="(guildId, message) => game.applyGuild(session.token, guildId, message)"
        @donate-guild="(gold) => game.donateGuild(session.token, gold)"
        @complete-guild-task="(taskKind) => game.completeGuildTask(session.token, taskKind)"
        @claim-guild-benefit="game.claimGuildBenefit(session.token)"
        @claim-sabak-tax="game.claimSabakTax(session.token)"
        @use-guild-merit-token="game.useGuildMeritToken(session.token)"
        @buy-guild-shop-item="(itemId) => game.buyGuildShopItem(session.token, itemId)"
        @upgrade-guild-totem="(totem) => game.upgradeGuildTotem(session.token, totem)"
        @charge-guild-war-tech="(kind) => game.chargeGuildWarTech(session.token, kind)"
        @review-guild-application="(applicationId, accept) => game.reviewGuildApplication(session.token, applicationId, accept)"
        @upgrade-pet="(petId) => game.upgradePet(session.token, petId)"
        @upgrade-treasure="(treasureId) => game.upgradeTreasure(session.token, treasureId)"
        @cultivation-breakthrough="game.cultivationBreakthrough(session.token)"
        @upgrade-wanxiang="game.upgradeWanxiang(session.token)"
        @extract-wanxiang-essence="(maxTier) => game.extractWanxiangEssence(session.token, maxTier)"
        @update-pk-settings="(pkEnabled, sweepAttackPlayers) => game.updatePkSettings(session.token, pkEnabled, sweepAttackPlayers)"
        @read-mail="(mailId) => game.readMail(session.token, mailId)"
        @claim-mail="(mailId) => game.claimMail(session.token, mailId)"
        @delete-mail="(mailId) => game.deleteMail(session.token, mailId)"
        @join-guild="(guildId) => game.joinGuild(session.token, guildId)"
        @start-afk="(skillId) => game.startAfk(session.token, skillId)"
        @settle-afk="game.settleAfk(session.token)"
        @stop-afk="game.stopAfk(session.token)"
        @resolve-adventure="(adventureId, optionId) => game.resolveAdventure(session.token, adventureId, optionId)"
      />
    </template>
  </main>
</template>
