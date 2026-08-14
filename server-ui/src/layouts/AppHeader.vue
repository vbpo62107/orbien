<script setup lang="ts">
import ThemeToggle from '@/components/ThemeToggle.vue'
import LocaleSwitcher from '@/components/LocaleSwitcher.vue'
import { useLocale } from '@/composables/useLocale'
import { useSidebar } from '@/composables/useSidebar'
import { useAuthStore } from '@/stores/auth'
import { useRouter } from 'vue-router'
import logoUrl from '@/assets/images/logo.png'
import githubIcon from '@/assets/icon/github.svg?raw'

const GITHUB_URL = 'https://github.com/orbien-org/orbien'

const { t } = useLocale()
const { isMobile, mobileOpen, toggleCollapsed } = useSidebar()
const auth = useAuthStore()
const router = useRouter()

async function logout() {
  await auth.logout()
  router.push('/login')
}
</script>

<template>
  <header class="top">
    <div class="top-left">
      <button
        v-if="isMobile"
        type="button"
        class="icon-btn menu-btn"
        :aria-label="mobileOpen ? t('actions.closeMenu') : t('actions.openMenu')"
        :aria-expanded="mobileOpen"
        @click="toggleCollapsed"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path v-if="mobileOpen" d="M6 6l12 12M18 6L6 18" />
          <path v-else d="M4 7h16M4 12h16M4 17h16" />
        </svg>
      </button>

      <div class="brand-block">
        <img class="logo-img" :src="logoUrl" alt="Orbien" />
        <div class="brand-title" aria-label="Orbien">
          <span class="brand-orb">Orb</span><span class="brand-rest">ien</span>
        </div>
      </div>
    </div>

    <div class="actions">
      <a
        class="icon-btn github-link"
        :href="GITHUB_URL"
        target="_blank"
        rel="noopener noreferrer"
        :aria-label="t('actions.github')"
        :title="t('actions.github')"
      >
        <span class="github-icon" aria-hidden="true" v-html="githubIcon" />
      </a>
      <LocaleSwitcher />
      <ThemeToggle />

      <!-- User badge + logout -->
      <div v-if="auth.authenticated" class="user-badge">
        <span class="user-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="8" r="4" />
            <path d="M4 20c1.8-3.5 4.7-5.5 8-5.5s6.2 2 8 5.5" />
          </svg>
        </span>
        <span class="user-name" v-if="auth.username">{{ auth.username }}</span>
        <button
          type="button"
          class="logout-btn"
          :title="t('actions.logout')"
          :aria-label="t('actions.logout')"
          @click="logout"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <path d="M17 16l4-4m0 0l-4-4m4 4H7" />
            <path d="M9 20H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h4" />
          </svg>
        </button>
      </div>
    </div>
  </header>
</template>

<style scoped>
.github-link { text-decoration: none; color: var(--text); }
.github-icon {
  display: inline-grid;
  place-items: center;
  width: 1.15rem;
  height: 1.15rem;
  line-height: 0;
}
.github-icon :deep(svg) {
  width: 100%; height: 100%; display: block;
  fill: currentColor; stroke: none;
}

.user-badge {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  padding: 0.25rem 0.5rem;
  border-radius: 2rem;
  background: var(--surface-offset, var(--surface));
  border: 1px solid var(--border);
  font-size: 0.8rem;
  color: var(--text-muted);
}
.user-icon {
  display: inline-grid;
  place-items: center;
  width: 1rem;
  height: 1rem;
  color: var(--primary);
}
.user-icon svg { width: 100%; height: 100%; }
.user-name { font-weight: 500; color: var(--text); max-width: 80px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

.logout-btn {
  display: inline-grid;
  place-items: center;
  width: 1.2rem;
  height: 1.2rem;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 0.25rem;
  transition: color 150ms;
}
.logout-btn:hover { color: var(--error, #e53e3e); }
.logout-btn svg { width: 100%; height: 100%; }
</style>
