import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchAuthStatus, type AuthStatus } from '@/api/client'

export const useAuthStore = defineStore('auth', () => {
  // Authenticated flag — survives SPA navigation but resets on hard reload.
  // The server uses an HTTP-only session cookie as the real auth gate.
  const authenticated = ref(false)
  const username = ref('')

  // Server-reported capabilities — loaded once on app start / login page mount.
  const capabilities = ref<AuthStatus>({ webauthn: false, password: true })
  const capabilitiesLoaded = ref(false)

  /** Call once when the Login page is shown. */
  async function loadCapabilities(): Promise<void> {
    if (capabilitiesLoaded.value) return
    capabilities.value = await fetchAuthStatus()
    capabilitiesLoaded.value = true
  }

  function setAuthenticated(val: boolean, user = '') {
    authenticated.value = val
    username.value = user
  }

  async function loginWithPassword(user: string, pass: string): Promise<void> {
    const res = await fetch('/api/v1/auth/login', {
      method: 'POST',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username: user, password: pass }),
    })
    if (res.status === 401) throw new Error('用户名或密码错误')
    if (!res.ok) throw new Error(`登录失败 (${res.status})`)
    authenticated.value = true
    username.value = user
  }

  async function logout(): Promise<void> {
    await fetch('/api/v1/auth/logout', { method: 'POST', credentials: 'include' }).catch(() => {})
    authenticated.value = false
    username.value = ''
    // Reset so the next login page load re-fetches.
    capabilitiesLoaded.value = false
  }

  return {
    authenticated,
    username,
    capabilities,
    capabilitiesLoaded,
    loadCapabilities,
    setAuthenticated,
    loginWithPassword,
    logout,
  }
})
