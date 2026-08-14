<script setup lang="ts">
import {computed, ref, watch} from 'vue'
import {useRouter} from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import OsBadge from '@/components/OsBadge.vue'
import PaginationBar from '@/components/PaginationBar.vue'
import {kickClient} from '@/api'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'

type StatusFilter = 'all' | 'online' | 'offline'
const FILTERS: StatusFilter[] = ['all', 'online', 'offline']

const store = useDashboardStore()
const router = useRouter()
const {t} = useLocale()

const page = ref(1)
const pageSize = ref(10)
const statusFilter = ref<StatusFilter>('all')
const kicking = ref<string | null>(null)
const search = ref('')

function isOnline(raw?: string) { return !raw || raw === 'online' }

const filtered = computed(() => {
  let list = store.clients
  if (statusFilter.value === 'online') list = list.filter(c => isOnline(c.status))
  else if (statusFilter.value === 'offline') list = list.filter(c => !isOnline(c.status))
  const q = search.value.trim().toLowerCase()
  if (q) list = list.filter(c =>
    c.runId.toLowerCase().includes(q) ||
    (c.hostname || '').toLowerCase().includes(q) ||
    (c.clientIP || '').toLowerCase().includes(q)
  )
  return list
})

const total = computed(() => filtered.value.length)
const pageItems = computed(() => {
  const start = (page.value - 1) * pageSize.value
  return filtered.value.slice(start, start + pageSize.value)
})

const statusCounts = computed(() => {
  let online = 0, offline = 0
  for (const c of store.clients) {
    if (isOnline(c.status)) online++; else offline++
  }
  return {all: store.clients.length, online, offline} as Record<StatusFilter, number>
})

watch([total, pageSize, statusFilter, search], () => {
  const maxPage = Math.max(1, Math.ceil(total.value / Math.max(pageSize.value, 1)))
  if (page.value > maxPage) page.value = maxPage
})
watch([statusFilter, search], () => { page.value = 1 })

function filterLabel(key: StatusFilter) {
  if (key === 'all') return t('clients.filterAll')
  if (key === 'online') return t('status.online')
  return t('status.offline')
}

function statusLabel(raw?: string) {
  if (isOnline(raw)) return t('status.online')
  if (raw === 'offline') return t('status.offline')
  return raw || t('status.offline')
}

function formatSeen(secs: number, online: boolean) {
  const n = Math.max(0, Math.floor(secs || 0))
  if (online) {
    if (n < 60) return t('clients.uptimeSecs', {n})
    if (n < 3600) return t('clients.uptimeMins', {n: Math.floor(n/60)})
    if (n < 86400) return t('clients.uptimeHours', {n: Math.floor(n/3600)})
    return t('clients.uptimeDays', {n: Math.floor(n/86400)})
  }
  if (n < 60) return t('clients.agoSecs', {n})
  if (n < 3600) return t('clients.agoMins', {n: Math.floor(n/60)})
  if (n < 86400) return t('clients.agoHours', {n: Math.floor(n/3600)})
  return t('clients.agoDays', {n: Math.floor(n/86400)})
}

function openDetail(runId: string) { router.push({name: 'client-detail', params: {runId}}) }
function onKeyOpen(evt: KeyboardEvent, runId: string) {
  if (evt.key === 'Enter' || evt.key === ' ') { evt.preventDefault(); openDetail(runId) }
}

async function onKick(runId: string, evt: Event) {
  evt.stopPropagation()
  if (kicking.value) return
  if (!window.confirm(t('clients.kickConfirm'))) return
  kicking.value = runId
  try {
    await kickClient(runId)
    await store.refresh()
  } catch {
    // Never expose raw error internals to the user — show a generic translated message.
    window.alert(t('clients.kickFailed'))
  } finally {
    kicking.value = null
  }
}
</script>

<template>
  <section class="client-list">
    <!-- toolbar -->
    <div class="toolbar">
      <div class="filter-group" role="group" :aria-label="t('clients.filter')">
        <button
          v-for="key in FILTERS" :key="key" type="button"
          class="filter-chip" :class="{active: statusFilter===key}"
          @click="statusFilter=key"
        >
          <span class="chip-dot" :class="key"/>
          {{ filterLabel(key) }}
          <em>{{ statusCounts[key] }}</em>
        </button>
      </div>
      <label class="search-wrap" :aria-label="t('clients.search')">
        <svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="6.5" cy="6.5" r="4.5"/><path d="M10.5 10.5L14 14"/></svg>
        <input
          v-model="search" type="search"
          class="search-input"
          :placeholder="t('clients.search')"
          autocomplete="off"
        />
      </label>
    </div>

    <!-- empty states -->
    <div v-if="!store.clients.length" class="empty-state">
      <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="24" cy="16" r="8"/><path d="M8 40c0-8.837 7.163-16 16-16s16 7.163 16 16"/></svg>
      <p>{{ t('clients.empty') }}</p>
    </div>
    <div v-else-if="!filtered.length" class="empty-state">
      <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><circle cx="20" cy="20" r="12"/><path d="M32 32L42 42"/><path d="M15 20h10M20 15v10"/></svg>
      <p>{{ t('clients.filterEmpty') }}</p>
    </div>

    <!-- client cards -->
    <transition-group name="list" tag="div" class="cards">
      <article
        v-for="c in pageItems" :key="c.runId"
        class="client-card" :class="{offline: !isOnline(c.status)}"
        role="button" tabindex="0" :aria-label="t('clients.detail')"
        @click="openDetail(c.runId)" @keydown="onKeyOpen($event, c.runId)"
      >
        <!-- left: avatar + info -->
        <div class="card-left">
          <div class="avatar" :class="{online: isOnline(c.status)}">
            <OsBadge :os="c.os" :arch="c.arch" icon-only/>
            <span class="pulse-dot" :class="{online: isOnline(c.status)}"/>
          </div>
          <div class="card-body">
            <div class="card-title">
              <span class="run-id">{{ c.runId }}</span>
              <span v-if="c.hostname" class="tag">{{ c.hostname }}</span>
              <span v-if="c.user" class="tag">{{ c.user }}</span>
              <span v-if="c.version" class="tag accent">v{{ c.version }}</span>
              <span class="tag soft">{{ t('clients.proxies') }} {{ c.proxyCount ?? 0 }}</span>
            </div>
            <div class="card-meta">
              <span class="meta-item">
                <svg viewBox="0 0 16 16" aria-hidden="true"><rect x="1" y="3" width="14" height="11" rx="2"/><path d="M5 3V1.5M11 3V1.5M1 7h14"/></svg>
                <span class="mono">{{ c.clientIP || '—' }}</span>
              </span>
              <OsBadge :os="c.os" :arch="c.arch" text-only/>
              <span class="meta-item seen">
                <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M2 12V8.5M5.5 12V5M9 12V7M12.5 12V3.5"/></svg>
                {{ formatSeen(c.connectedSecs, isOnline(c.status)) }}
              </span>
            </div>
          </div>
        </div>

        <!-- right: actions + status -->
        <div class="card-right">
          <button
            v-if="isOnline(c.status)" type="button"
            class="kick-btn" :disabled="kicking===c.runId"
            :title="t('clients.kick')" :aria-label="t('clients.kick')"
            @click="onKick(c.runId,$event)"
          >
            <AppIcon name="kick"/>
          </button>
          <span class="status-pill" :class="{online: isOnline(c.status)}">
            <span class="pill-dot"/>
            {{ statusLabel(c.status) }}
          </span>
        </div>
      </article>
    </transition-group>

    <PaginationBar v-model:page="page" v-model:page-size="pageSize" :total="total"/>
  </section>
</template>

<style scoped>
.client-list {
  display: flex;
  flex-direction: column;
  gap: 0.8rem;
  animation: page-in 0.35s ease both;
}

@keyframes page-in {
  from { opacity: 0; transform: translateY(6px); }
  to   { opacity: 1; transform: translateY(0); }
}

/* toolbar */
.toolbar {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  flex: 1;
  min-width: 0;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border: 1px solid var(--line);
  background: var(--panel);
  color: var(--muted);
  font: inherit;
  font-size: 0.78rem;
  font-weight: 600;
  padding: 0.36rem 0.7rem;
  border-radius: 999px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}

.chip-dot {
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 50%;
  background: var(--muted);
  flex-shrink: 0;
}

.chip-dot.online  { background: var(--status-ok); }
.chip-dot.offline { background: var(--danger); }

.filter-chip em {
  font-style: normal;
  font-variant-numeric: tabular-nums;
  font-size: 0.7rem;
  font-weight: 600;
  padding: 0.03rem 0.38rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--muted) 12%, transparent);
}

.filter-chip:hover:not(.active) { color: var(--text); border-color: var(--line-strong); }

.filter-chip.active {
  color: var(--accent-text);
  border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  background: var(--accent-soft);
}
.filter-chip.active em {
  color: var(--accent-text);
  background: color-mix(in srgb, var(--accent) 18%, transparent);
}

/* search */
.search-wrap {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  padding: 0.38rem 0.75rem;
  border: 1px solid var(--line);
  border-radius: 999px;
  background: var(--panel);
  transition: border-color 0.15s, box-shadow 0.15s;
  cursor: text;
}

.search-wrap:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 18%, transparent);
}

.search-wrap svg {
  width: 0.9rem;
  height: 0.9rem;
  fill: none;
  stroke: var(--muted);
  stroke-width: 1.8;
  stroke-linecap: round;
  flex-shrink: 0;
}

.search-input {
  border: 0;
  background: transparent;
  color: var(--text);
  font: inherit;
  font-size: 0.8rem;
  width: 12rem;
  outline: none;
}

.search-input::placeholder { color: var(--muted); }

/* empty states */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.85rem;
  padding: 3.5rem 1rem;
  background: var(--panel);
  border: 1px dashed var(--line-strong);
  border-radius: 16px;
  color: var(--muted);
}

.empty-state svg {
  width: 3rem;
  height: 3rem;
  opacity: 0.45;
}

.empty-state p {
  margin: 0;
  font-size: 0.9rem;
}

/* card list */
.cards {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
}

/* list transition */
.list-enter-active, .list-leave-active { transition: all 0.22s ease; }
.list-enter-from { opacity: 0; transform: translateY(-8px); }
.list-leave-to   { opacity: 0; transform: translateY(8px); }

.client-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.9rem 1.1rem;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 14px;
  box-shadow: var(--shadow);
  cursor: pointer;
  transition: border-color 0.18s, box-shadow 0.18s, background 0.18s, transform 0.12s;
}

.client-card:hover {
  border-color: var(--line-strong);
  background: var(--panel-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 16px rgba(0,0,0,0.22);
}

.client-card:active { transform: translateY(0); }

.client-card:focus-visible {
  outline: none;
  box-shadow: var(--focus-ring);
}

.client-card.offline { opacity: 0.88; }

/* card left */
.card-left {
  display: flex;
  align-items: flex-start;
  gap: 0.8rem;
  min-width: 0;
  flex: 1;
}

.avatar {
  position: relative;
  width: 2.4rem;
  height: 2.4rem;
  border-radius: 10px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 14%, transparent);
  transition: background 0.2s, border-color 0.2s;
}

.avatar.online {
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 25%, transparent);
}

.pulse-dot {
  position: absolute;
  right: -0.14rem;
  bottom: -0.14rem;
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 50%;
  background: var(--muted);
  border: 2px solid var(--panel);
  box-sizing: content-box;
  transition: background 0.2s;
}

.pulse-dot.online {
  background: var(--status-ok);
  animation: pulse 2.4s ease-in-out infinite;
}

@keyframes pulse {
  0%,100% { box-shadow: 0 0 0 2px color-mix(in srgb, var(--status-ok) 20%, transparent); }
  50%      { box-shadow: 0 0 0 4px color-mix(in srgb, var(--status-ok) 8%, transparent); }
}

/* card body */
.card-body { min-width: 0; display: flex; flex-direction: column; gap: 0.38rem; }

.card-title {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.38rem;
}

.run-id {
  font-size: 0.95rem;
  font-weight: 700;
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  letter-spacing: -0.01em;
  color: var(--text);
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: 0.1rem 0.48rem;
  border-radius: 999px;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 15%, transparent);
}

.tag.accent {
  color: var(--accent-text);
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 22%, transparent);
}

.tag.soft { font-weight: 500; }

/* meta */
.card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.6rem 1rem;
  font-size: 0.78rem;
  color: var(--muted);
}

.meta-item {
  display: inline-flex;
  align-items: center;
  gap: 0.28rem;
}

.meta-item svg {
  width: 0.82rem;
  height: 0.82rem;
  fill: none;
  stroke: currentColor;
  stroke-width: 1.6;
  stroke-linecap: round;
  opacity: 0.75;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.76rem;
  color: var(--text-secondary);
}

/* card right */
.card-right { display: flex; align-items: center; gap: 0.6rem; flex-shrink: 0; }

.status-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.38rem;
  min-width: 4rem;
  padding: 0.26rem 0.7rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 650;
  color: var(--muted);
  background: color-mix(in srgb, var(--muted) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--muted) 18%, transparent);
  justify-content: center;
}

.status-pill.online {
  color: var(--status-ok);
  background: var(--status-ok-soft);
  border-color: color-mix(in srgb, var(--status-ok) 25%, transparent);
}

.pill-dot {
  width: 0.36rem;
  height: 0.36rem;
  border-radius: 50%;
  background: currentColor;
  flex-shrink: 0;
}

.kick-btn {
  width: 1.85rem;
  height: 1.85rem;
  padding: 0;
  border-radius: 8px;
  border: 1px solid var(--danger-border);
  background: var(--danger-bg);
  color: var(--danger);
  display: inline-grid;
  place-items: center;
  cursor: pointer;
  font-size: 1rem;
  transition: background 0.15s, border-color 0.15s, transform 0.1s;
}

.kick-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--danger) 18%, transparent);
  border-color: var(--danger);
  transform: scale(1.08);
}

.kick-btn:disabled { opacity: 0.4; cursor: wait; }

@media (max-width: 720px) {
  .client-card { flex-direction: column; align-items: stretch; }
  .card-right { align-self: flex-end; }
  .search-input { width: 8rem; }
}
</style>
