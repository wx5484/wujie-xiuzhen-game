<script setup lang="ts">
import { Activity, Coins, Gem, Package, Signal } from '@lucide/vue'
import type { CharacterBundle, CharacterRecord, InventoryView, PlayerSkill } from '../shared/types'

const props = defineProps<{
  character: CharacterRecord
  bundle: CharacterBundle | null
  inventory: InventoryView | null
  skills: PlayerSkill[]
  connected: boolean
  authenticated: boolean
  playable: boolean
}>()

const classLabels: Record<CharacterRecord['class'], string> = {
  warrior: '剑修',
  mage: '法修',
  taoist: '魂修',
  assassin: '刺客'
}
</script>

<template>
  <section class="statusbar">
    <div class="status-identity">
      <strong>{{ character.name }}</strong>
      <span>{{ classLabels[character.class] ?? character.class }} {{ character.level }} 级</span>
    </div>
    <div title="经验">经验 {{ character.exp }}</div>
    <div title="战力"><Activity :size="16" /> {{ character.power }}</div>
    <div title="金币"><Coins :size="16" /> {{ character.gold }}</div>
    <div title="元宝"><Gem :size="16" /> {{ character.yuanbao }}</div>
    <div title="体力">体力 {{ bundle?.state.stamina ?? 0 }}/{{ bundle?.state.stamina_max ?? 5000 }}</div>
    <div title="背包">
      <Package :size="16" />
      {{ inventory?.summary.bag_used ?? bundle?.inventory.bag_used ?? 0 }}/{{ inventory?.summary.bag_limit ?? bundle?.inventory.bag_limit ?? 0 }}
    </div>
    <div :class="['net-dot', { online: connected || playable, authed: authenticated }]" title="连接状态">
      <Signal :size="16" />
      {{ authenticated ? '已认证' : (playable ? 'HTTP模式' : (connected ? '连接中' : '离线')) }}
    </div>
  </section>
</template>
