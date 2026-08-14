<script setup lang="ts">
import {computed} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import OsBadge from '@/components/OsBadge.vue'
import SectionCard from '@/components/SectionCard.vue'
import TrafficChart from '@/components/TrafficChart.vue'
import TrafficSummary from '@/components/TrafficSummary.vue'
import {kickClient} from '@/api'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'
import {ref} from 'vue'

const route = useRoute()
const router = useRouter()
const store = useDashboardStore()
const {t} = useLocale()
const kicking = ref(false)

const runId = computed(() => route.params.runId as string)
const client = computed(() => store.clients.find(c => c.runId === runId.value))
const isOnline = computed(() => !client.value?.status || client.value.status === 'online')

const proxies = computed(() =>
  store.proxies.filter(p => p.runId === runId.value)
)

const TYPE_COLORS: Record<string, string> = {
  http:'#3b82f6', https:'#93c5fd', tcp:'#94a3b8',
  udp:'#2dd4bf', socks5:'#f97316', file:'#fb7185', stcp:'#a78bfa', xtcp:'#f472b6',
}
function typeColor(type: string) { return TYPE_COLORS[(type||'tcp').toLowerCase()] ?? '#60a5fa' }

function formatUptime(secs: number) {
  const n = Math.max(0, Math.floor(secs || 0))
  if (n < 60) return `${n}s`
  if (n < 3600) return `${Math.floor(n/60)}m ${n%60}s`
  if (n < 86400) { const h=Math.floor(n/3600); return `${h}h ${Math.floor((n%3600)/60)}m` }
  return `${Math.floor(n/86400)}d ${Math.floor((n%86400)/3600)}h`
}

async function onKick() {
  if (kicking.value) return
  if (!window.confirm(t('clients.kickConfirm'))) return
  kicking.value = true
  try {
    await kickClient(runId.value)
    await store.refresh()
    router.push({name: 'clients'})
  } catch (e) {
    window.alert(t('clients.kickFailed', {msg: e instanceof Error ? e.message : String(e)}))
  } finally {
    kicking.value = false
  }
}
</script>

<template>
  <div v-if="!client" class="not-found">
    <div class="not-found-inner">
      <svg viewBox="0 0 64 64" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <circle cx="32" cy="24" r="10"/>
        <path d="M12 54c0-11.046 8.954-20 20-20s20 8.954 20 20"/>
        <path d="M26 24h12M32 18v12" opacity="0.4"/>
      </svg>
      <p>{{ t('clients.notFound') }}</p>
      <button class="back-btn" @click="router.push({name:'clients'})">← {{ t('common.back') }}</button>
    </div>
  </div>

  <div v-else class="client-detail">
    <!-- ── Hero header ── -->
    <header class="detail-hero">
      <button class="back-icon" :aria-label="t('common.back')" @click="router.back()">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M10 3L5 8l5 5"/></svg>
      </button>

      <div class="hero-avatar" :class="{online: isOnline}">
        <OsBadge :os="client.os" :arch="client.arch" icon-only/>
        <span class="hero-dot" :class="{online: isOnline}"/>
      </div>

      <div class="hero-info">
        <div class="hero-title-row">
          <h1 class="hero-id">{{ client.runId }}</h1>
          <span class="status-pill" :class="{online: isOnline}">
            <span class="pill-dot"/>
            {{ isOnline ? t('status.online') : t('status.offline') }}
          </span>
        </div>
        <div class="hero-meta">
          <span v-if="client.hostname" class="meta-chip">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><rect x="2" y="3" width="12" height="9" rx="1.5"/><path d="M5 14h6M8 12v2"/></svg>
            {{ client.hostname }}
          </span>
          <span v-if="client.user" class="meta-chip">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><circle cx="8" cy="5.5" r="3"/><path d="M2 13.5c0-3.314 2.686-6 6-6s6 2.686 6 6"/></svg>
            {{ client.user }}
          </span>
          <span class="meta-chip">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><rect x="1" y="4" width="14" height="9" rx="1.5"/><path d="M4 4V2.5M12 4V2.5M1 8h14"/></svg>
            <span class="mono">{{ client.clientIP || '—' }}</span>
          </span>
          <span v-if="client.version" class="meta-chip accent">v{{ client.version }}</span>
          <OsBadge :os="client.os" :arch="client.arch" text-only/>
          <span v-if="isOnline" class="meta-chip uptime">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><circle cx="8" cy="8" r="6.5"/><path d="M8 4.5V8l2.5 2.5"/></svg>
            {{ formatUptime(client.connectedSecs) }}
          </span>
        </div>
      </div>

      <div class="hero-actions">
        <button
          v-if="isOnline"
          type="button" class="kick-btn"
          :disabled="kicking"
          @click="onKick"
        >
          <AppIcon name="kick"/>
          {{ t('clients.kick') }}
        </button>
      </div>
    </header>

    <!-- ── Traffic summary ── -->
    <div class="stat-row">
      <SectionCard :title="t('traffic.network')">
        <TrafficSummary
          :traffic-in="client.totalTrafficIn ?? 0"
          :traffic-out="client.totalTrafficOut ?? 0"
        />
      </SectionCard>

      <SectionCard :title="t('traffic.historyAll')">
        <TrafficChart variant="line" range="24h" :run-id="runId" :refresh-ms="6000"/>
      </SectionCard>
    </div>

    <!-- ── Proxy list ── -->
    <SectionCard :title="t('nav.proxies')">
      <template #extra>
        <span class="proxy-count">{{ proxies.length }}</span>
      </template>

      <div v-if="!proxies.length" class="empty-state">
        <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><rect x="8" y="14" width="32" height="22" rx="4"/><path d="M16 14V10M32 14V10M8 22h32"/></svg>
        <p>{{ t('overview.emptyProxies') }}</p>
      </div>

      <div v-else class="proxy-grid">
        <div
          v-for="p in proxies" :key="p.name"
          class="proxy-card"
          @click="$router.push({name:'proxy-detail', params:{name:p.name}})"
        >
          <div class="proxy-card-top">
            <span class="type-badge" :style="{color:typeColor(p.type), background:typeColor(p.type)+'1a', borderColor:typeColor(p.type)+'44'}">
              {{ (p.type||'tcp').toUpperCase() }}
            </span>
            <span class="conn-badge">{{ p.curConns ?? 0 }} conn</span>
          </div>
          <div class="proxy-name">{{ p.name }}</div>
          <div class="proxy-addr mono">{{ p.localAddr || '—' }}</div>
        </div>
      </div>
    </SectionCard>
  </div>
</template>

<style scoped>
.client-detail {
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
  animation: page-in 0.35s ease both;
}

@keyframes page-in {
  from { opacity: 0; transform: translateY(6px); }
  to   { opacity: 1; transform: translateY(0); }
}

/* not found */
.not-found {
  display: grid;
  place-items: center;
  min-height: 20rem;
}
.not-found-inner {
  display: flex; flex-direction: column; align-items: center;
  gap: 1rem; color: var(--muted);
}
.not-found-inner svg { width: 4rem; height: 4rem; opacity: 0.4; }
.not-found-inner p { margin: 0; font-size: 0.9rem; }
.back-btn {
  padding: 0.4rem 1rem; border-radius: 8px;
  border: 1px solid var(--line); background: var(--panel);
  color: var(--text); font: inherit; font-size: 0.85rem;
  cursor: pointer; transition: background 0.15s;
}
.back-btn:hover { background: var(--panel-hover); }

/* hero */
.detail-hero {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  padding: 1.25rem 1.4rem;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: var(--shadow);
  position: relative;
}

.back-icon {
  width: 2rem; height: 2rem;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: transparent;
  color: var(--muted);
  display: grid; place-items: center;
  cursor: pointer; flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
  margin-top: 0.15rem;
}
.back-icon:hover { background: var(--panel-hover); color: var(--text); }
.back-icon svg { width: 1rem; height: 1rem; }

.hero-avatar {
  position: relative;
  width: 3.2rem; height: 3.2rem;
  border-radius: 14px;
  display: grid; place-items: center;
  flex-shrink: 0;
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 15%, transparent);
  transition: background 0.2s;
}
.hero-avatar.online {
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 25%, transparent);
}

.hero-dot {
  position: absolute;
  right: -0.2rem; bottom: -0.2rem;
  width: 0.65rem; height: 0.65rem;
  border-radius: 50%;
  background: var(--muted);
  border: 2.5px solid var(--panel);
  box-sizing: content-box;
}
.hero-dot.online {
  background: var(--status-ok);
  animation: pulse 2.4s ease-in-out infinite;
}
@keyframes pulse {
  0%,100% { box-shadow: 0 0 0 2px color-mix(in srgb, var(--status-ok) 20%, transparent); }
  50%      { box-shadow: 0 0 0 5px color-mix(in srgb, var(--status-ok) 7%, transparent); }
}

.hero-info { flex: 1; min-width: 0; }

.hero-title-row {
  display: flex; align-items: center; gap: 0.65rem;
  flex-wrap: wrap; margin-bottom: 0.55rem;
}

.hero-id {
  margin: 0;
  font-size: 1.2rem;
  font-weight: 700;
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  letter-spacing: -0.02em;
  color: var(--text);
  word-break: break-all;
}

.status-pill {
  display: inline-flex; align-items: center; gap: 0.35rem;
  padding: 0.22rem 0.65rem;
  border-radius: 999px; font-size: 0.73rem; font-weight: 650;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 18%, transparent);
}
.status-pill.online {
  color: var(--status-ok);
  background: var(--status-ok-soft);
  border-color: color-mix(in srgb, var(--status-ok) 25%, transparent);
}
.pill-dot {
  width: 0.34rem; height: 0.34rem;
  border-radius: 50%; background: currentColor; flex-shrink: 0;
}

.hero-meta {
  display: flex; flex-wrap: wrap; gap: 0.4rem 0.6rem;
  align-items: center;
}

.meta-chip {
  display: inline-flex; align-items: center; gap: 0.3rem;
  padding: 0.18rem 0.55rem;
  border-radius: 8px;
  font-size: 0.76rem; font-weight: 500;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 8%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 14%, transparent);
}
.meta-chip svg { width: 0.78rem; height: 0.78rem; opacity: 0.75; flex-shrink: 0; }
.meta-chip.accent { color: var(--accent-text); background: var(--accent-soft); border-color: color-mix(in srgb, var(--accent) 22%, transparent); }
.meta-chip.uptime { color: var(--status-ok); background: var(--status-ok-soft); border-color: color-mix(in srgb, var(--status-ok) 22%, transparent); }
.mono { font-family: 'IBM Plex Mono', ui-monospace, monospace; font-size: 0.75rem; }

.hero-actions { flex-shrink: 0; }

.kick-btn {
  display: inline-flex; align-items: center; gap: 0.45rem;
  padding: 0.48rem 1rem;
  border-radius: 10px;
  border: 1px solid var(--danger-border);
  background: var(--danger-bg);
  color: var(--danger);
  font: inherit; font-size: 0.82rem; font-weight: 600;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, transform 0.1s;
}
.kick-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--danger) 18%, transparent);
  border-color: var(--danger);
}
.kick-btn:disabled { opacity: 0.4; cursor: wait; }

/* stat row */
.stat-row {
  display: grid;
  grid-template-columns: 1fr 2fr;
  gap: 1rem;
  align-items: stretch;
}

/* proxy grid */
.proxy-count {
  font-size: 0.75rem; font-weight: 700;
  color: var(--accent-text);
  background: var(--accent-soft);
  border: 1px solid color-mix(in srgb, var(--accent) 28%, transparent);
  padding: 0.12rem 0.48rem; border-radius: 999px;
  font-variant-numeric: tabular-nums;
}

.empty-state {
  display: flex; flex-direction: column; align-items: center;
  gap: 0.75rem; padding: 2.5rem 1rem; color: var(--muted);
}
.empty-state svg { width: 2.5rem; height: 2.5rem; opacity: 0.4; }
.empty-state p { margin: 0; font-size: 0.88rem; }

.proxy-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 0.65rem;
}

.proxy-card {
  padding: 0.85rem 1rem;
  border-radius: 12px;
  border: 1px solid var(--line);
  background: color-mix(in srgb, var(--muted) 4%, transparent);
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, transform 0.12s;
  display: flex; flex-direction: column; gap: 0.38rem;
}
.proxy-card:hover {
  background: var(--panel-hover);
  border-color: var(--line-strong);
  transform: translateY(-1px);
}

.proxy-card-top {
  display: flex; align-items: center; justify-content: space-between;
}

.type-badge {
  display: inline-flex; align-items: center;
  padding: 0.1rem 0.45rem;
  border-radius: 6px; font-size: 0.66rem; font-weight: 700;
  border: 1px solid; letter-spacing: 0.04em;
}

.conn-badge {
  font-size: 0.68rem; color: var(--muted);
  font-variant-numeric: tabular-nums;
}

.proxy-name {
  font-weight: 600;
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.82rem; color: var(--text);
  word-break: break-all;
}

.proxy-addr {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.74rem; color: var(--muted);
}

@media (max-width: 860px) {
  .detail-hero { flex-wrap: wrap; }
  .hero-actions { width: 100%; }
  .stat-row { grid-template-columns: 1fr; }
}
</style>
