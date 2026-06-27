import { defineStore } from 'pinia'
import {
  attackMob,
  claimMail as claimMailRequest,
  claimGuildBenefit as claimGuildBenefitRequest,
  claimSabakTax as claimSabakTaxRequest,
  chargeGuildWarTech as chargeGuildWarTechRequest,
  buyGuildShopItem as buyGuildShopItemRequest,
  buyTradeItem,
  cancelTradeItem,
  castSkillHttp,
  challengeTower as challengeTowerRequest,
  challengeWorldBoss as challengeWorldBossRequest,
  claimQuest as claimQuestRequest,
  completeGuildTask as completeGuildTaskRequest,
  cultivationBreakthrough as cultivationBreakthroughRequest,
  applyGuild as applyGuildRequest,
  attackBot as attackBotRequest,
  createGuild as createGuildRequest,
  createCharacter,
  buyShopItem as buyShopItemRequest,
  buyYuanbaoItem as buyYuanbaoItemRequest,
  deleteMail as deleteMailRequest,
  decomposeEquipment as decomposeEquipmentRequest,
  decomposeMisc as decomposeMiscRequest,
  donateGuild as donateGuildRequest,
  exchangeNpcMaterial as exchangeNpcMaterialRequest,
  extractWanxiangEssence as extractWanxiangEssenceRequest,
  equipItem,
  exploreArea,
  exploreSecretRealm as exploreSecretRealmRequest,
  enhanceItem as enhanceItemRequest,
  getAfkStatus,
  getGameBootstrap,
  getGameOverview,
  getInventory,
  getGuildApplications,
  getRoomState,
  getSkills,
  joinGuild as joinGuildRequest,
  learnSkill as learnSkillRequest,
  toggleSkillAuto as toggleSkillAutoRequest,
  upgradeSkill as upgradeSkillRequest,
  getMail,
  getQuests,
  getTradeList,
  listTradeItem,
  listCharacters,
  markMailRead as markMailReadRequest,
  moveCharacter,
  reviewGuildApplication as reviewGuildApplicationRequest,
  settleAfk,
  startAfk,
  stopAfk,
  teleport as teleportRequest,
  upgradePet as upgradePetRequest,
  upgradeWanxiang as upgradeWanxiangRequest,
  updateVipPotionSettings as updateVipPotionSettingsRequest,
  updatePkSettings as updatePkSettingsRequest,
  upgradeBattleInstinct as upgradeBattleInstinctRequest,
  upgradeGuildTotem as upgradeGuildTotemRequest,
  upgradeSpecialSkill as upgradeSpecialSkillRequest,
  useGuildMeritToken as useGuildMeritTokenRequest,
  recycleItem as recycleItemRequest,
  redeemRechargeCard as redeemRechargeCardRequest,
  retrieveItem as retrieveItemRequest,
  resolveAdventure as resolveAdventureRequest,
  storeItem as storeItemRequest,
  unequipItem,
  upgradeTreasure as upgradeTreasureRequest,
  useItem,
  wildAfk as wildAfkRequest
} from '../shared/api'
import type {
  AdventureOffer,
  AfkSettleResult,
  AfkStatus,
  CharacterClass,
  CharacterRecord,
  CharacterState,
  CombatLogEvent,
  GameBootstrap,
  GameOverview,
  InventoryView,
  MobTemplate,
  PlayerGuildApplication,
  PlayerMail,
  PlayerQuest,
  PlayerSkill,
  PlayerSkillList,
  PlayerVipSettings,
  RealtimeActionResult,
  RoomStateEvent,
  SystemActionResult,
  SystemNoticeEvent,
  TradeConsignment,
  TradeActionResult,
  WsEnvelope
} from '../shared/types'
import { GameSocket } from '../shared/ws'

const MAX_LOG_LINES = 120

export const useGameStore = defineStore('game', {
  state: () => ({
    characters: [] as CharacterRecord[],
    activeCharacterId: 0,
    bootstrap: null as GameBootstrap | null,
    overview: null as GameOverview | null,
    inventory: null as InventoryView | null,
    skills: [] as PlayerSkill[],
    afk: null as AfkStatus | null,
    mails: [] as PlayerMail[],
    guildApplications: [] as PlayerGuildApplication[],
    quests: [] as PlayerQuest[],
    consignments: [] as TradeConsignment[],
    mobs: [] as MobTemplate[],
    room: null as RoomStateEvent | null,
    log: [] as string[],
    connected: false,
    authenticated: false,
    lastEventAt: '',
    rawState: null as Record<string, unknown> | null,
    featureUpdates: {} as Record<string, unknown>,
    pendingAdventure: null as AdventureOffer | null,
    socketGeneration: 0,
    wsFallbackNotified: false,
    socket: null as GameSocket | null
  }),
  getters: {
    activeCharacter(state) {
      return state.characters.find((character) => character.id === state.activeCharacterId) ?? state.characters[0]
    },
    activeBundle(state) {
      return state.bootstrap?.character ?? null
    },
    mobById(state) {
      return (id: string) => state.mobs.find((mob) => mob.id === id)
    },
    activeGuild(state) {
      return state.overview?.guilds.find((guild) => guild.joined) ?? null
    }
  },
  actions: {
    async loadCharacters(token: string) {
      const previousId = this.activeCharacterId
      const data = await listCharacters(token)
      this.characters = data.characters
      this.activeCharacterId = this.characters.some((character) => character.id === previousId)
        ? previousId
        : (this.characters[0]?.id ?? 0)
    },
    async loadBootstrap(token: string) {
      try {
        const data = await getGameBootstrap(token, this.activeCharacterId || undefined)
        this.bootstrap = data
        this.mobs = data.mobs ?? []
        if (data.character) {
          this.upsertCharacter(data.character.character)
        }
      } catch (error) {
        this.log.unshift(error instanceof Error ? `启动数据读取失败：${error.message}` : '启动数据读取失败')
      }
    },
    async loadInventory(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.setInventory(await getInventory(token, this.activeCharacterId))
      } catch (error) {
        this.log.unshift(error instanceof Error ? `背包读取失败：${error.message}` : '背包读取失败')
      }
    },
    async loadOverview(token: string) {
      try {
        this.overview = await getGameOverview(token, this.activeCharacterId || undefined)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `世界看板读取失败：${error.message}` : '世界看板读取失败')
      }
    },
    async loadRoom(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.room = await getRoomState(token, this.activeCharacterId)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `地图状态读取失败：${error.message}` : '地图状态读取失败')
      }
    },
    async loadAfk(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.afk = await getAfkStatus(token, this.activeCharacterId)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `挂机状态读取失败：${error.message}` : '挂机状态读取失败')
      }
    },
    async loadSkills(token: string) {
      if (!this.activeCharacterId) return
      try {
        const data = await getSkills(token, this.activeCharacterId)
        this.skills = data.skills
      } catch (error) {
        this.log.unshift(error instanceof Error ? `技能读取失败：${error.message}` : '技能读取失败')
      }
    },
    async loadMail(token: string) {
      if (!this.activeCharacterId) return
      try {
        const data = await getMail(token, this.activeCharacterId)
        this.mails = data.mails
      } catch (error) {
        this.log.unshift(error instanceof Error ? `邮件读取失败：${error.message}` : '邮件读取失败')
      }
    },
    async loadQuests(token: string) {
      if (!this.activeCharacterId) return
      try {
        const data = await getQuests(token, this.activeCharacterId)
        this.quests = data.quests
      } catch (error) {
        this.log.unshift(error instanceof Error ? `任务读取失败：${error.message}` : '任务读取失败')
      }
    },
    async loadGuildApplications(token: string) {
      if (!this.activeCharacterId) return
      const role = this.activeGuild?.role
      if (!['leader', 'elder'].includes(role ?? '')) {
        this.guildApplications = []
        return
      }
      try {
        const data = await getGuildApplications(token, this.activeCharacterId)
        this.guildApplications = data.applications
      } catch (error) {
        this.log.unshift(error instanceof Error ? `入会申请读取失败：${error.message}` : '入会申请读取失败')
      }
    },
    async loadTrade(token: string) {
      if (!this.activeCharacterId) return
      try {
        const data = await getTradeList(token, this.activeCharacterId)
        this.consignments = data.consignments
      } catch (error) {
        this.log.unshift(error instanceof Error ? `寄售读取失败：${error.message}` : '寄售读取失败')
      }
    },
    async createCharacter(token: string, name: string, characterClass: CharacterClass) {
      const data = await createCharacter(token, name, characterClass)
      this.upsertCharacter(data.character)
      this.activeCharacterId = data.character.id
    },
    async equip(token: string, itemId: number) {
      if (!this.activeCharacterId) return
      try {
        this.setInventory(await equipItem(token, this.activeCharacterId, itemId))
        await this.loadBootstrap(token)
        this.log.unshift('装备已穿戴。')
      } catch (error) {
        this.log.unshift(error instanceof Error ? `穿戴失败：${error.message}` : '穿戴失败')
      }
    },
    async unequip(token: string, itemId: number) {
      if (!this.activeCharacterId) return
      try {
        this.setInventory(await unequipItem(token, this.activeCharacterId, itemId))
        await this.loadBootstrap(token)
        this.log.unshift('装备已卸下。')
      } catch (error) {
        this.log.unshift(error instanceof Error ? `卸下失败：${error.message}` : '卸下失败')
      }
    },
    async useItem(token: string, itemId: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await useItem(token, this.activeCharacterId, itemId)
        this.setInventory(result.inventory)
        this.log.unshift(result.message)
        await this.loadBootstrap(token)
        await this.loadRoom(token)
        if (this.activeCharacterId) this.connect(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `使用失败：${error.message}` : '使用失败')
      }
    },
    async enhanceItem(token: string, itemId: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await enhanceItemRequest(token, this.activeCharacterId, itemId)
        this.applyInventoryActionResult(result)
        await this.loadQuests(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `强化失败：${error.message}` : '强化失败')
      }
    },
    async recycleItem(token: string, itemId: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await recycleItemRequest(token, this.activeCharacterId, itemId)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `回收失败：${error.message}` : '回收失败')
      }
    },
    async sellBagItems(token: string, itemIds: number[]) {
      if (!this.activeCharacterId) return
      const uniqueIds = [...new Set(itemIds)].filter((id) => Number.isFinite(id) && id > 0)
      if (!uniqueIds.length) {
        this.log.unshift('没有可售卖的背包物品')
        return
      }
      const beforeGold = this.activeCharacter?.gold ?? 0
      let sold = 0
      let failed = 0
      for (const itemId of uniqueIds) {
        try {
          const result = await recycleItemRequest(token, this.activeCharacterId, itemId)
          this.setInventory(result.inventory)
          this.upsertCharacter(result.character)
          sold += 1
        } catch {
          failed += 1
        }
      }
      const gained = Math.max(0, (this.activeCharacter?.gold ?? beforeGold) - beforeGold)
      if (sold > 0) {
        this.log.unshift(`一键售卖完成：${sold} 件物品，金币 +${gained}${failed ? `，${failed} 件失败` : ''}`)
      } else {
        this.log.unshift('一键售卖失败：没有物品被售卖')
      }
    },
    async storeItem(token: string, itemId: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await storeItemRequest(token, this.activeCharacterId, itemId)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `存仓失败：${error.message}` : '存仓失败')
      }
    },
    async retrieveItem(token: string, itemId: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await retrieveItemRequest(token, this.activeCharacterId, itemId)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `取回失败：${error.message}` : '取回失败')
      }
    },
    async buyShopItem(token: string, templateId: string, quantity = 1) {
      if (!this.activeCharacterId) return
      try {
        const result = await buyShopItemRequest(token, this.activeCharacterId, templateId, quantity)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `购买失败：${error.message}` : '购买失败')
      }
    },
    async buyYuanbaoItem(token: string, templateId: string, quantity = 1) {
      if (!this.activeCharacterId) return
      try {
        const result = await buyYuanbaoItemRequest(token, this.activeCharacterId, templateId, quantity)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `元宝购买失败：${error.message}` : '元宝购买失败')
      }
    },
    async updateVipPotionSettings(token: string, settings: PlayerVipSettings) {
      if (!this.activeCharacterId) return
      try {
        const vipSettings = await updateVipPotionSettingsRequest(token, this.activeCharacterId, settings)
        if (this.overview) {
          this.overview.systems.vip_settings = vipSettings
        }
        this.log.unshift('会员设置已保存。')
      } catch (error) {
        this.log.unshift(error instanceof Error ? `会员设置失败：${error.message}` : '会员设置失败')
      }
    },
    async exchangeNpcMaterial(token: string, materialId: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await exchangeNpcMaterialRequest(token, this.activeCharacterId, materialId)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `材料兑换失败：${error.message}` : '材料兑换失败')
      }
    },
    async upgradeBattleInstinct(token: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await upgradeBattleInstinctRequest(token, this.activeCharacterId)
        this.applyInventoryActionResult(result)
        await this.loadSkills(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `战斗本能升级失败：${error.message}` : '战斗本能升级失败')
      }
    },
    async upgradeSpecialSkill(token: string, skillId: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await upgradeSpecialSkillRequest(token, this.activeCharacterId, skillId)
        this.applyInventoryActionResult(result)
        await this.loadSkills(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `特殊被动升级失败：${error.message}` : '特殊被动升级失败')
      }
    },
    async redeemRechargeCard(token: string, code: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await redeemRechargeCardRequest(token, this.activeCharacterId, code)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
        await this.loadOverview(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `兑换失败：${error.message}` : '兑换失败')
      }
    },
    async upgradePet(token: string, petId?: number) {
      if (!this.activeCharacterId) return
      try {
        this.applySystemActionResult(await upgradePetRequest(token, this.activeCharacterId, petId))
        await this.loadInventory(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `宠物升级失败：${error.message}` : '宠物升级失败')
      }
    },
    async upgradeTreasure(token: string, treasureId?: number) {
      if (!this.activeCharacterId) return
      try {
        this.applySystemActionResult(await upgradeTreasureRequest(token, this.activeCharacterId, treasureId))
        await this.loadInventory(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `法宝升级失败：${error.message}` : '法宝升级失败')
      }
    },
    async cultivationBreakthrough(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.applySystemActionResult(await cultivationBreakthroughRequest(token, this.activeCharacterId))
        await this.loadInventory(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `修炼突破失败：${error.message}` : '修炼突破失败')
      }
    },
    async upgradeWanxiang(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.applySystemActionResult(await upgradeWanxiangRequest(token, this.activeCharacterId))
      } catch (error) {
        this.log.unshift(error instanceof Error ? `万象铸体失败：${error.message}` : '万象铸体失败')
      }
    },
    async extractWanxiangEssence(token: string, maxTier: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await extractWanxiangEssenceRequest(token, this.activeCharacterId, maxTier)
        if (this.overview) {
          this.overview.systems = result.systems
        }
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `灵韵提取失败：${error.message}` : '灵韵提取失败')
      }
    },
    async updatePkSettings(token: string, pkEnabled: boolean, sweepAttackPlayers: boolean) {
      if (!this.activeCharacterId) return
      try {
        const state = await updatePkSettingsRequest(token, this.activeCharacterId, pkEnabled, sweepAttackPlayers)
        this.updateCharacterState(state)
        this.log.unshift(
          state.pk_enabled
            ? 'PK 模式已开启，探索会优先扫荡同屏玩家。'
            : 'PK 已关闭，探索只攻击怪物。'
        )
      } catch (error) {
        this.log.unshift(error instanceof Error ? `PK 设置失败：${error.message}` : 'PK 设置失败')
      }
    },
    async learnSkill(token: string, skillId: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await learnSkillRequest(token, this.activeCharacterId, skillId)
        this.upsertSkill(result.skill)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `学习失败：${error.message}` : '学习失败')
      }
    },
    async upgradeSkill(token: string, skillId: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await upgradeSkillRequest(token, this.activeCharacterId, skillId)
        this.upsertSkill(result.skill)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `技能校准失败：${error.message}` : '技能校准失败')
      }
    },
    async toggleSkillAuto(token: string, skillId: string, autoEnabled: boolean) {
      if (!this.activeCharacterId) return
      try {
        const result = await toggleSkillAutoRequest(token, this.activeCharacterId, skillId, autoEnabled)
        this.upsertSkill(result.skill)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `技能设置失败：${error.message}` : '技能设置失败')
      }
    },
    async joinGuild(token: string, guildId: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await joinGuildRequest(token, this.activeCharacterId, guildId)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : { ...guild, joined: false }
          )
        }
        this.log.unshift(result.message)
        await this.loadOverview(token)
        await this.loadQuests(token)
        await this.loadGuildApplications(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `加入行会失败：${error.message}` : '加入行会失败')
      }
    },
    async createGuild(token: string, name: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await createGuildRequest(token, this.activeCharacterId, name)
        this.log.unshift(result.message)
        await this.loadOverview(token)
        await this.loadQuests(token)
        await this.loadGuildApplications(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `创建行会失败：${error.message}` : '创建行会失败')
      }
    },
    async applyGuild(token: string, guildId: number, message = '') {
      if (!this.activeCharacterId) return
      try {
        const result = await applyGuildRequest(token, this.activeCharacterId, guildId, message)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `申请入会失败：${error.message}` : '申请入会失败')
      }
    },
    async donateGuild(token: string, gold: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await donateGuildRequest(token, this.activeCharacterId, gold)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.log.unshift(result.message)
        await this.loadBootstrap(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `行会捐献失败：${error.message}` : '行会捐献失败')
      }
    },
    async completeGuildTask(token: string, taskKind: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await completeGuildTaskRequest(token, this.activeCharacterId, taskKind)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.log.unshift(result.message)
        await this.loadOverview(token)
        await this.loadQuests(token)
        await this.loadBootstrap(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `行会任务失败：${error.message}` : '行会任务失败')
      }
    },
    async claimGuildBenefit(token: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await claimGuildBenefitRequest(token, this.activeCharacterId)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `行会福利领取失败：${error.message}` : '行会福利领取失败')
      }
    },
    async useGuildMeritToken(token: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await useGuildMeritTokenRequest(token, this.activeCharacterId)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `功勋令使用失败：${error.message}` : '功勋令使用失败')
      }
    },
    async buyGuildShopItem(token: string, itemId: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await buyGuildShopItemRequest(token, this.activeCharacterId, itemId)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `行会商城购买失败：${error.message}` : '行会商城购买失败')
      }
    },
    async upgradeGuildTotem(token: string, totem: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await upgradeGuildTotemRequest(token, this.activeCharacterId, totem)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
        await this.loadBootstrap(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `图腾升级失败：${error.message}` : '图腾升级失败')
      }
    },
    async chargeGuildWarTech(token: string, kind: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await chargeGuildWarTechRequest(token, this.activeCharacterId, kind)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `攻城科技充能失败：${error.message}` : '攻城科技充能失败')
      }
    },
    async claimSabakTax(token: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await claimSabakTaxRequest(token, this.activeCharacterId)
        if (this.overview) {
          this.overview.guilds = this.overview.guilds.map((guild) =>
            guild.id === result.guild.id ? result.guild : guild
          )
        }
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `沙城税收领取失败：${error.message}` : '沙城税收领取失败')
      }
    },
    async reviewGuildApplication(token: string, applicationId: number, accept: boolean) {
      if (!this.activeCharacterId) return
      try {
        const data = await reviewGuildApplicationRequest(token, this.activeCharacterId, applicationId, accept)
        this.guildApplications = data.applications
        this.log.unshift(accept ? '已通过入会申请。' : '已拒绝入会申请。')
        await this.loadOverview(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `审批失败：${error.message}` : '审批失败')
      }
    },
    async claimQuest(token: string, questId: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await claimQuestRequest(token, this.activeCharacterId, questId)
        this.quests = result.quests.quests
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `任务领取失败：${error.message}` : '任务领取失败')
      }
    },
    async readMail(token: string, mailId: number) {
      if (!this.activeCharacterId) return
      try {
        this.upsertMail(await markMailReadRequest(token, this.activeCharacterId, mailId))
      } catch (error) {
        this.log.unshift(error instanceof Error ? `邮件标记失败：${error.message}` : '邮件标记失败')
      }
    },
    async claimMail(token: string, mailId: number) {
      if (!this.activeCharacterId) return
      try {
        const result = await claimMailRequest(token, this.activeCharacterId, mailId)
        this.upsertMail(result.mail)
        this.setInventory(result.inventory)
        this.upsertCharacter(result.character)
        this.log.unshift(result.message)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `领取失败：${error.message}` : '领取失败')
      }
    },
    async deleteMail(token: string, mailId: number) {
      if (!this.activeCharacterId) return
      try {
        const data = await deleteMailRequest(token, this.activeCharacterId, mailId)
        this.mails = data.mails
        this.log.unshift('邮件已删除。')
      } catch (error) {
        this.log.unshift(error instanceof Error ? `邮件删除失败：${error.message}` : '邮件删除失败')
      }
    },
    async listTradeItem(token: string, itemId: number, price: number, priceCurrency: 'gold' | 'yuanbao') {
      if (!this.activeCharacterId) return
      try {
        this.applyTradeResult(await listTradeItem(token, this.activeCharacterId, itemId, price, priceCurrency))
        await this.loadInventory(token)
        await this.loadTrade(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `上架失败：${error.message}` : '上架失败')
      }
    },
    async buyTradeItem(token: string, consignmentId: number) {
      if (!this.activeCharacterId) return
      try {
        this.applyTradeResult(await buyTradeItem(token, this.activeCharacterId, consignmentId))
      } catch (error) {
        this.log.unshift(error instanceof Error ? `购买失败：${error.message}` : '购买失败')
      }
    },
    async cancelTradeItem(token: string, consignmentId: number) {
      if (!this.activeCharacterId) return
      try {
        this.applyTradeResult(await cancelTradeItem(token, this.activeCharacterId, consignmentId))
      } catch (error) {
        this.log.unshift(error instanceof Error ? `下架失败：${error.message}` : '下架失败')
      }
    },
    async startAfk(token: string, skillId: string) {
      if (!this.activeCharacterId) return
      try {
        this.afk = await startAfk(token, this.activeCharacterId, skillId)
        this.log.unshift(
          this.afk.mode === 'skill_study'
            ? `虚境研修已开始：正在研修 ${this.afk.training_skill_name ?? '已选技能'}。`
            : '炼狱修炼已开始。'
        )
        await this.loadRoom(token)
        await this.loadAfk(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `挂机启动失败：${error.message}` : '挂机启动失败')
      }
    },
    async settleAfk(token: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await settleAfk(token, this.activeCharacterId)
        await this.applyAfkSettleResult(token, result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `挂机结算失败：${error.message}` : '挂机结算失败')
      }
    },
    async stopAfk(token: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await stopAfk(token, this.activeCharacterId)
        await this.applyAfkSettleResult(token, result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `挂机停止失败：${error.message}` : '挂机停止失败')
      }
    },
    async applyAfkSettleResult(token: string, result: AfkSettleResult) {
      this.afk = result.status
      this.log.unshift(result.message)
      if (result.adventure) this.pendingAdventure = result.adventure
      await this.loadBootstrap(token)
      await this.loadRoom(token)
      await this.loadInventory(token)
      await this.loadSkills(token)
      await this.loadQuests(token)
      await this.loadAfk(token)
    },
    async resolveAdventure(token: string, adventureId: number, optionId: string) {
      if (!this.activeCharacterId) return
      try {
        const result = await resolveAdventureRequest(token, this.activeCharacterId, adventureId, optionId)
        this.pendingAdventure = null
        this.upsertCharacter(result.character)
        this.updateCharacterState(result.state)
        this.log.unshift(result.message)
        this.log = this.log.slice(0, MAX_LOG_LINES)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `奇遇结算失败：${error.message}` : '奇遇结算失败')
      }
    },
    connect(token: string) {
      this.connected = false
      this.authenticated = false
      this.socket?.close()
      const generation = this.socketGeneration + 1
      this.socketGeneration = generation
      const socket = new GameSocket()
      socket.on((event) => {
        if (this.socket === socket && this.socketGeneration === generation) {
          this.handleEvent(event)
        }
      })
      socket.connect(
        () => {
          if (this.socket !== socket || this.socketGeneration !== generation) return
          if (!this.activeCharacterId) {
            socket.close()
            return
          }
          this.connected = true
          this.wsFallbackNotified = false
          socket.send('auth', { token, character_id: this.activeCharacterId, device: 'web' })
        },
        () => {
          if (this.socket !== socket || this.socketGeneration !== generation) return
          this.connected = false
          this.authenticated = false
          if (this.activeCharacterId && !this.wsFallbackNotified) {
            this.wsFallbackNotified = true
            this.log.unshift('WebSocket 暂不可用，已切换 HTTP 模式。')
          }
        }
      )
      this.socket = socket
    },
    handleEvent(event: WsEnvelope) {
      this.lastEventAt = new Date().toLocaleTimeString()
      if (event.type === 'auth_ok') {
        this.authenticated = true
        const notice = event.payload as Partial<SystemNoticeEvent>
        this.log.unshift(notice.message ?? 'WebSocket 已认证')
        this.socket?.send('state_request')
      } else if (event.type === 'room_state') {
        this.room = event.payload as RoomStateEvent
      } else if (event.type === 'combat_log') {
        this.log.unshift(...(event.payload as CombatLogEvent).lines)
      } else if (event.type === 'adventure_offer') {
        this.pendingAdventure = event.payload as AdventureOffer
        this.log.unshift(`触发奇遇：${this.pendingAdventure.title}`)
      } else if (event.type === 'character_update') {
        this.upsertCharacter(event.payload as CharacterRecord)
      } else if (event.type === 'character_state_update') {
        this.updateCharacterState(event.payload as CharacterState)
      } else if (event.type === 'inventory_update') {
        this.setInventory(event.payload as InventoryView)
      } else if (event.type === 'skills_update') {
        this.skills = (event.payload as PlayerSkillList).skills
      } else if (event.type === 'afk_update') {
        this.afk = event.payload as AfkStatus
      } else if (event.type === 'system_notice') {
        this.log.unshift((event.payload as SystemNoticeEvent).message)
      } else if (event.type === 'state') {
        this.rawState = event.payload as Record<string, unknown>
      } else if (event.type.endsWith('_update')) {
        this.featureUpdates[event.type] = event.payload
      } else if (event.type === 'force_logout') {
        this.log.unshift('账号在其他设备登录，当前连接已下线')
        this.socket?.close()
      } else if (event.type === 'error') {
        this.log.unshift(String((event.payload as { message: string }).message))
      }
      this.log = this.log.slice(0, MAX_LOG_LINES)
    },
    async refreshState(token?: string) {
      if (!this.authenticated) {
        if (token && this.activeCharacterId) {
          await this.loadRoom(token)
          return
        }
        this.log.unshift('正在载入角色，请稍候')
        return
      }
      this.socket?.send('state_request')
    },
    async move(token: string, direction: string) {
      if (!this.authenticated) {
        if (!this.activeCharacterId) return
        try {
          this.applyRealtimeActionResult(await moveCharacter(token, this.activeCharacterId, direction))
          await this.loadBootstrap(token)
          await this.loadSkills(token)
          await this.loadAfk(token)
        } catch (error) {
          this.log.unshift(error instanceof Error ? `移动失败：${error.message}` : '移动失败')
        }
        return
      }
      this.socket?.send('cmd', { command: 'move', args: { direction } })
    },
    async teleport(token: string, zone: string, room: string) {
      if (!this.activeCharacterId) return
      try {
        this.applyRealtimeActionResult(await teleportRequest(token, this.activeCharacterId, zone, room))
        await this.loadBootstrap(token)
        await this.loadSkills(token)
        await this.loadAfk(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `传送失败：${error.message}` : '传送失败')
      }
    },
    async attack(token: string, targetId: number) {
      if (!this.authenticated) {
        if (!this.activeCharacterId) return
        try {
          this.applyRealtimeActionResult(await attackMob(token, this.activeCharacterId, targetId))
          await this.loadBootstrap(token)
          await this.loadOverview(token)
          await this.loadQuests(token)
          await this.loadSkills(token)
        } catch (error) {
          this.log.unshift(error instanceof Error ? `攻击失败：${error.message}` : '攻击失败')
        }
        return
      }
      this.socket?.send('cmd', { command: 'attack', args: { target_id: targetId } })
    },
    async attackBot(token: string, targetIndex: number) {
      if (!this.activeCharacterId) return
      try {
        this.applyRealtimeActionResult(await attackBotRequest(token, this.activeCharacterId, targetIndex))
        await this.loadBootstrap(token)
        await this.loadOverview(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `PK 失败：${error.message}` : 'PK 失败')
      }
    },
    async explore(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.applyRealtimeActionResult(await exploreArea(token, this.activeCharacterId))
        await this.loadBootstrap(token)
        await this.loadOverview(token)
        await this.loadQuests(token)
        await this.loadSkills(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `探索失败：${error.message}` : '探索失败')
      }
    },
    async exploreSecretRealm(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.applyRealtimeActionResult(await exploreSecretRealmRequest(token, this.activeCharacterId))
        await this.loadBootstrap(token)
        await this.loadOverview(token)
        await this.loadQuests(token)
        await this.loadSkills(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `秘境探索失败：${error.message}` : '秘境探索失败')
      }
    },
    async challengeTower(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.applyRealtimeActionResult(await challengeTowerRequest(token, this.activeCharacterId))
        await this.loadBootstrap(token)
        await this.loadOverview(token)
        await this.loadQuests(token)
        await this.loadSkills(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `无尽塔挑战失败：${error.message}` : '无尽塔挑战失败')
      }
    },
    async challengeWorldBoss(token: string) {
      if (!this.activeCharacterId) return
      try {
        this.applyRealtimeActionResult(await challengeWorldBossRequest(token, this.activeCharacterId))
        await this.loadBootstrap(token)
        await this.loadOverview(token)
        await this.loadQuests(token)
        await this.loadSkills(token)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `世界首领挑战失败：${error.message}` : '世界首领挑战失败')
      }
    },
    async wildAfk(token: string) {
      if (!this.activeCharacterId) return
      try {
        const status = await wildAfkRequest(token, this.activeCharacterId)
        this.afk = status
        this.log.unshift(`炼狱修炼已开始：预估经验 ${status.exp_per_minute}/分，每 5 秒扣除 10000 金币。`)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `炼狱修炼失败：${error.message}` : '炼狱修炼失败')
      }
    },
    async decomposeEquipment(token: string, rarities: string[], itemIds: number[] = []) {
      if (!this.activeCharacterId) return
      try {
        const result = await decomposeEquipmentRequest(token, this.activeCharacterId, rarities, itemIds)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `装备拆解失败：${error.message}` : '装备拆解失败')
      }
    },
    async decomposeMisc(token: string, kinds: string[] = ['book'], itemIds: number[] = []) {
      if (!this.activeCharacterId) return
      try {
        const result = await decomposeMiscRequest(token, this.activeCharacterId, kinds, itemIds)
        this.applyInventoryActionResult(result)
      } catch (error) {
        this.log.unshift(error instanceof Error ? `杂项拆解失败：${error.message}` : '杂项拆解失败')
      }
    },
    async castSkill(token: string, skillId: string, targetId = 0) {
      if (!this.authenticated) {
        if (!this.activeCharacterId) return
        try {
          this.applyRealtimeActionResult(await castSkillHttp(token, this.activeCharacterId, skillId, targetId))
          await this.loadBootstrap(token)
          await this.loadSkills(token)
          await this.loadOverview(token)
        } catch (error) {
          this.log.unshift(error instanceof Error ? `施法失败：${error.message}` : '施法失败')
        }
        return
      }
      this.socket?.send('cmd', { command: 'cast_skill', args: { skill_id: skillId, target_id: targetId } })
    },
    upsertCharacter(character: CharacterRecord) {
      const index = this.characters.findIndex((item) => item.id === character.id)
      if (index >= 0) this.characters.splice(index, 1, character)
      else this.characters.push(character)
      if (this.bootstrap?.character?.character.id === character.id) {
        this.bootstrap.character.character = character
      }
      if (!this.activeCharacterId) this.activeCharacterId = character.id
    },
    setInventory(view: InventoryView) {
      this.inventory = view
      if (this.bootstrap?.character) {
        this.bootstrap.character.inventory = view.summary
      }
    },
    upsertMail(mail: PlayerMail) {
      const index = this.mails.findIndex((item) => item.id === mail.id)
      if (index >= 0) this.mails.splice(index, 1, mail)
      else this.mails.unshift(mail)
    },
    upsertSkill(skill: PlayerSkill) {
      const index = this.skills.findIndex((item) => item.id === skill.id)
      if (index >= 0) this.skills.splice(index, 1, skill)
      else this.skills.push(skill)
    },
    updateCharacterState(state: CharacterState) {
      if (this.bootstrap?.character?.character.id === state.character_id) {
        this.bootstrap.character.state = state
      }
    },
    applyTradeResult(result: TradeActionResult) {
      this.consignments = result.consignments
      this.setInventory(result.inventory)
      this.upsertCharacter(result.character)
      this.log.unshift(result.message)
    },
    applyInventoryActionResult(result: { inventory: InventoryView; character: CharacterRecord; message: string }) {
      this.setInventory(result.inventory)
      this.upsertCharacter(result.character)
      this.log.unshift(result.message)
    },
    applySystemActionResult(result: SystemActionResult) {
      if (this.overview) {
        this.overview.systems = result.systems
      }
      this.upsertCharacter(result.character)
      this.log.unshift(result.message)
    },
    applyRealtimeActionResult(result: RealtimeActionResult) {
      this.room = result.room
      this.upsertCharacter(result.character)
      this.updateCharacterState(result.state)
      if (result.inventory) this.setInventory(result.inventory)
      if (result.adventure) this.pendingAdventure = result.adventure
      if (result.log.length) this.log.unshift(...result.log)
      this.log = this.log.slice(0, MAX_LOG_LINES)
    }
  }
})
