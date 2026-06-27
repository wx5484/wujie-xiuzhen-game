<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { LogIn, UserPlus } from '@lucide/vue'
import { useSessionStore } from '../stores/session'

const session = useSessionStore()
const mode = ref<'login' | 'register'>('login')
const username = ref('')
const password = ref('')
const email = ref('')
const captchaText = ref(makeCaptcha())
const captchaInput = ref('')
const localError = ref('')
const visibleError = computed(() => localError.value || session.error)

function submit() {
  localError.value = ''
  if (mode.value === 'login') session.login(username.value, password.value)
  else {
    if (captchaInput.value.trim().toUpperCase() !== captchaText.value) {
      localError.value = '验证码不正确，请重新输入。'
      refreshCaptcha()
      return
    }
    session.register(username.value, password.value, email.value || undefined).finally(refreshCaptcha)
  }
}

function makeCaptcha() {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'
  return Array.from({ length: 4 }, () => chars[Math.floor(Math.random() * chars.length)]).join('')
}

function refreshCaptcha() {
  captchaText.value = makeCaptcha()
  captchaInput.value = ''
}

watch(mode, () => {
  localError.value = ''
  if (mode.value === 'register') refreshCaptcha()
})
</script>

<template>
  <main class="auth-screen">
    <section class="auth-showcase">
      <div class="brand-lockup">
        <span class="brand-mark">无</span>
        <div>
          <h1>无界修真：斩妖录</h1>
          <p>开放地图 · 挂机养成 · 斩妖问道</p>
        </div>
      </div>
      <div class="auth-game-ui">
        <div class="auth-map-card">
          <header>
            <span>当前界域</span>
            <strong>青牛城</strong>
          </header>
          <div class="auth-route">
            <i></i>
            <span>迷雾竹林</span>
            <span>废弃灵矿</span>
            <span>天水古道</span>
          </div>
        </div>
        <div class="auth-stat-grid">
          <article><span>体力</span><strong>5000</strong></article>
          <article><span>装备</span><strong>17 阶</strong></article>
          <article><span>技能</span><strong>100 级</strong></article>
          <article><span>万象</span><strong>Lv.1000</strong></article>
        </div>
        <div class="auth-log">
          <p>第 1 回合：你拔剑迎敌，斩落魔化藤蔓。</p>
          <p>获得经验、金币，并拾取凡尘系列装备。</p>
          <p>远方传来钟声，天水古城的路标已经亮起。</p>
        </div>
      </div>
    </section>

    <section class="auth-panel">
      <div class="brand-lockup">
        <span class="brand-mark">录</span>
        <div>
          <h1>无界修真：斩妖录</h1>
          <p>账号通行证</p>
        </div>
      </div>

      <div class="segmented">
        <button :class="{ active: mode === 'login' }" @click="mode = 'login'">登录</button>
        <button :class="{ active: mode === 'register' }" @click="mode = 'register'">注册</button>
      </div>

      <form class="form-stack" @submit.prevent="submit">
        <label>
          <span>账号</span>
          <input v-model="username" autocomplete="username" />
        </label>
        <label>
          <span>密码</span>
          <input v-model="password" type="password" autocomplete="current-password" />
        </label>
        <label v-if="mode === 'register'">
          <span>邮箱</span>
          <input v-model="email" type="email" autocomplete="email" />
        </label>
        <label v-if="mode === 'register'">
          <span>验证码</span>
          <div class="captcha-row">
            <button type="button" class="captcha-code" title="刷新验证码" @click="refreshCaptcha">{{ captchaText }}</button>
            <input v-model="captchaInput" autocomplete="off" maxlength="4" placeholder="输入验证码" />
          </div>
        </label>
        <p v-if="visibleError" class="error-line">{{ visibleError }}</p>
        <button class="primary-button" :disabled="session.loading">
          <LogIn v-if="mode === 'login'" :size="18" />
          <UserPlus v-else :size="18" />
          {{ session.loading ? '处理中' : mode === 'login' ? '进入游戏' : '创建账号' }}
        </button>
      </form>
    </section>
  </main>
</template>
