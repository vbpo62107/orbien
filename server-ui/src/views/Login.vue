<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useLocale } from '@/composables/useLocale'
import { useWebAuthn } from '@/composables/useWebAuthn'
import logoUrl from '@/assets/images/logo.png'

const { t } = useLocale()
const router = useRouter()
const auth = useAuthStore()
const { supported, registering, authenticating, register, authenticate } = useWebAuthn()

const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)
const mode = ref<'password' | 'webauthn'>('password')

// Fetch server capabilities on mount so we know if WebAuthn is configured.
onMounted(() => auth.loadCapabilities())

/**
 * Show the Passkey tab only when BOTH:
 *   1. The browser supports WebAuthn (window.PublicKeyCredential)
 *   2. The server has WebAuthn enabled (auth.capabilities.webauthn)
 */
const canWebAuthn = computed(
  () => supported.value && auth.capabilities.webauthn,
)

async function loginPassword() {
  if (!username.value || !password.value) {
    error.value = t('login.errorEmpty')
    return
  }
  loading.value = true
  error.value = ''
  try {
    await auth.loginWithPassword(username.value, password.value)
    router.push('/')
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : t('login.errorFailed')
  } finally {
    loading.value = false
  }
}

async function loginFingerprint() {
  error.value = ''
  try {
    const ok = await authenticate()
    if (ok) {
      auth.setAuthenticated(true)
      router.push('/')
    } else {
      error.value = t('login.errorWebAuthn')
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : t('login.errorWebAuthn')
  }
}

async function registerFingerprint() {
  if (!username.value) {
    error.value = t('login.errorEmptyUser')
    return
  }
  error.value = ''
  try {
    await register(username.value)
    error.value = ''
    mode.value = 'webauthn'
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : t('login.errorRegister')
  }
}
</script>

<template>
  <div class="login-bg">
    <div class="login-card">
      <!-- Logo -->
      <div class="login-logo">
        <img :src="logoUrl" alt="Orbien" class="logo-img" />
        <span class="brand-orb">Orb</span><span class="brand-rest">ien</span>
      </div>

      <h1 class="login-title">{{ t('login.title') }}</h1>
      <p class="login-sub">{{ t('login.subtitle') }}</p>

      <!-- Mode tabs: only render the Passkey tab when the server says WebAuthn is on -->
      <div class="login-tabs" role="tablist">
        <button
          role="tab"
          class="login-tab"
          :class="{ active: mode === 'password' }"
          @click="mode = 'password'"
        >{{ t('login.tabPassword') }}</button>
        <button
          v-if="canWebAuthn"
          role="tab"
          class="login-tab"
          :class="{ active: mode === 'webauthn' }"
          @click="mode = 'webauthn'"
        >{{ t('login.tabFingerprint') }}</button>
      </div>

      <!-- Password form -->
      <form v-if="mode === 'password'" class="login-form" @submit.prevent="loginPassword">
        <div class="field">
          <label for="login-user">{{ t('login.username') }}</label>
          <input
            id="login-user"
            v-model="username"
            type="text"
            autocomplete="username"
            :placeholder="t('login.usernamePh')"
          />
        </div>
        <div class="field">
          <label for="login-pass">{{ t('login.password') }}</label>
          <input
            id="login-pass"
            v-model="password"
            type="password"
            autocomplete="current-password"
            :placeholder="t('login.passwordPh')"
          />
        </div>
        <p v-if="error" class="login-error" role="alert">{{ error }}</p>
        <button type="submit" class="btn-primary" :disabled="loading">
          <span v-if="loading" class="spin" aria-hidden="true" />
          {{ loading ? t('login.loading') : t('login.submit') }}
        </button>

        <!-- Register passkey button — only shown when WebAuthn is available -->
        <button
          v-if="canWebAuthn"
          type="button"
          class="btn-ghost"
          :disabled="registering"
          @click="registerFingerprint"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true" class="btn-icon">
            <path d="M12 2C8 2 5 5.5 5 9c0 2.4.9 4.5 2.4 6a8.6 8.6 0 0 0 1.6 1.4v2a1 1 0 0 0 1 1h4a1 1 0 0 0 1-1v-2A7 7 0 0 0 19 9c0-3.9-3.1-7-7-7z"/>
            <path d="M10 9a2 2 0 1 1 4 0 2 2 0 0 1-4 0"/>
          </svg>
          {{ registering ? t('login.registering') : t('login.registerFingerprint') }}
        </button>
      </form>

      <!-- WebAuthn / Passkey form -->
      <div v-else class="login-form">
        <div class="fingerprint-area">
          <button
            class="fingerprint-btn"
            :class="{ scanning: authenticating }"
            :disabled="authenticating"
            :aria-label="t('login.scanFingerprint')"
            @click="loginFingerprint"
          >
            <svg viewBox="0 0 64 64" fill="none" class="fp-icon" aria-hidden="true">
              <circle cx="32" cy="32" r="28" stroke="currentColor" stroke-width="2" opacity="0.15"/>
              <circle cx="32" cy="32" r="20" stroke="currentColor" stroke-width="2" opacity="0.25"/>
              <circle cx="32" cy="32" r="12" stroke="currentColor" stroke-width="2" opacity="0.4"/>
              <path d="M22 32c0-5.5 4.5-10 10-10" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <path d="M19 32c0-7.2 5.8-13 13-13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <path d="M25 32c0-3.9 3.1-7 7-7" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <path d="M28 32a4 4 0 0 1 8 0c0 3-1.8 5.5-4 7" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
              <path d="M32 22v-2M22 32h-2M32 42v2M42 32h2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" opacity="0.5"/>
            </svg>
            <span class="fp-label">
              {{ authenticating ? t('login.scanning') : t('login.scanFingerprint') }}
            </span>
          </button>
        </div>
        <p v-if="error" class="login-error" role="alert">{{ error }}</p>
        <p class="login-hint">{{ t('login.webAuthnHint') }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-bg {
  min-height: 100dvh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
  padding: 1rem;
}

.login-card {
  width: 100%;
  max-width: 420px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 1rem;
  padding: 2.5rem 2rem;
  box-shadow: 0 8px 32px oklch(0 0 0 / 0.15);
  display: flex;
  flex-direction: column;
  gap: 0;
}

.login-logo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
}
.login-logo .logo-img { width: 32px; height: 32px; }
.brand-orb { font-weight: 700; font-size: 1.4rem; color: var(--primary); }
.brand-rest { font-weight: 700; font-size: 1.4rem; color: var(--text); }

.login-title {
  font-size: 1.35rem;
  font-weight: 700;
  color: var(--text);
  margin: 0 0 0.25rem;
}
.login-sub {
  font-size: 0.85rem;
  color: var(--text-muted);
  margin: 0 0 1.5rem;
}

.login-tabs {
  display: flex;
  gap: 0.25rem;
  background: var(--bg);
  border-radius: 0.5rem;
  padding: 0.25rem;
  margin-bottom: 1.5rem;
}
.login-tab {
  flex: 1;
  padding: 0.45rem 0.75rem;
  border-radius: 0.35rem;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.85rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 150ms, color 150ms;
}
.login-tab.active {
  background: var(--surface);
  color: var(--text);
  box-shadow: 0 1px 4px oklch(0 0 0 / 0.1);
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
.field label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.field input {
  padding: 0.6rem 0.85rem;
  border-radius: 0.5rem;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
  font-size: 0.9rem;
  transition: border-color 150ms, box-shadow 150ms;
}
.field input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px oklch(from var(--primary) l c h / 0.15);
}

.login-error {
  font-size: 0.82rem;
  color: var(--error, #e53e3e);
  margin: 0;
  padding: 0.5rem 0.75rem;
  background: oklch(from var(--error, #e53e3e) l c h / 0.08);
  border-radius: 0.4rem;
}

.btn-primary {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.65rem 1rem;
  border-radius: 0.5rem;
  border: none;
  background: var(--primary);
  color: #fff;
  font-size: 0.9rem;
  font-weight: 600;
  cursor: pointer;
  transition: background 150ms, opacity 150ms;
}
.btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
.btn-primary:disabled { opacity: 0.55; cursor: not-allowed; }

.btn-ghost {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.4rem;
  padding: 0.6rem 1rem;
  border-radius: 0.5rem;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-muted);
  font-size: 0.85rem;
  cursor: pointer;
  transition: border-color 150ms, color 150ms;
}
.btn-ghost:hover:not(:disabled) { border-color: var(--primary); color: var(--primary); }
.btn-icon { width: 1rem; height: 1rem; stroke: currentColor; fill: none; stroke-width: 2; }

.fingerprint-area {
  display: flex;
  justify-content: center;
  padding: 1rem 0;
}
.fingerprint-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  padding: 1.5rem 2rem;
  border-radius: 1rem;
  border: 2px solid var(--border);
  background: var(--bg);
  color: var(--primary);
  cursor: pointer;
  transition: border-color 200ms, box-shadow 200ms, color 200ms;
  width: 100%;
  max-width: 240px;
}
.fingerprint-btn:hover:not(:disabled) {
  border-color: var(--primary);
  box-shadow: 0 0 0 4px oklch(from var(--primary) l c h / 0.12);
}
.fingerprint-btn.scanning {
  border-color: var(--primary);
  animation: pulse-border 1.2s ease-in-out infinite;
}
.fp-icon { width: 64px; height: 64px; }
.fp-label { font-size: 0.85rem; font-weight: 600; color: var(--text); }

.login-hint {
  font-size: 0.78rem;
  color: var(--text-muted);
  text-align: center;
  margin: 0;
}

.spin {
  width: 0.9rem;
  height: 0.9rem;
  border: 2px solid rgba(255,255,255,0.35);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

@keyframes spin { to { transform: rotate(360deg); } }
@keyframes pulse-border {
  0%, 100% { box-shadow: 0 0 0 0 oklch(from var(--primary) l c h / 0.2); }
  50% { box-shadow: 0 0 0 8px oklch(from var(--primary) l c h / 0); }
}
</style>
