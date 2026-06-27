import { defineStore } from 'pinia'
import { login, register } from '../shared/api'

const key = 'cq-session'

export const useSessionStore = defineStore('session', {
  state: () => ({
    accountId: 0,
    token: '',
    expiresAt: '',
    error: '',
    loading: false
  }),
  actions: {
    restore() {
      const raw = localStorage.getItem(key)
      if (!raw) return
      let parsed: { accountId?: number; token?: string; expiresAt?: string }
      try {
        parsed = JSON.parse(raw)
      } catch {
        this.logout()
        return
      }
      if (!parsed.token || !parsed.expiresAt || isExpired(parsed.expiresAt)) {
        this.logout()
        return
      }
      this.accountId = parsed.accountId ?? 0
      this.token = parsed.token
      this.expiresAt = parsed.expiresAt
    },
    persist() {
      localStorage.setItem(
        key,
        JSON.stringify({ accountId: this.accountId, token: this.token, expiresAt: this.expiresAt })
      )
    },
    async login(username: string, password: string) {
      this.loading = true
      this.error = ''
      try {
        const data = await login(username, password)
        this.accountId = data.account_id
        this.token = data.token
        this.expiresAt = data.expires_at
        this.persist()
      } catch (error) {
        this.error = error instanceof Error ? error.message : '登录失败'
      } finally {
        this.loading = false
      }
    },
    async register(username: string, password: string, email?: string) {
      this.loading = true
      this.error = ''
      try {
        await register(username, password, email)
        await this.login(username, password)
      } catch (error) {
        this.error = error instanceof Error ? error.message : '注册失败'
      } finally {
        this.loading = false
      }
    },
    logout() {
      this.accountId = 0
      this.token = ''
      this.expiresAt = ''
      this.error = ''
      localStorage.removeItem(key)
    }
  }
})

function isExpired(value: string) {
  const timestamp = Date.parse(value)
  if (Number.isNaN(timestamp)) return true
  return timestamp <= Date.now()
}
