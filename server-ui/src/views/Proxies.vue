<script setup lang="ts">
import {computed, ref, watch} from 'vue'
import {useRouter} from 'vue-router'
import PaginationBar from '@/components/PaginationBar.vue'
import SectionCard from '@/components/SectionCard.vue'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'

const store = useDashboardStore()
const router = useRouter()
const {t} = useLocale()

const page = ref(1)
const pageSize = ref(15)
const typeFilter = ref('all')
const search = ref('')

const allTypes = computed(() => {
  const types = new Set<string>()
  for (const p of store.proxies) types.add((p.type || 'tcp').toLowerCase())
  return ['all', ...Array.from(types).sort()]
})

const filtered = computed(() => {
  let list = store.proxies
  if (typeFilter.value !== 'all') list = list.filter(p => (p.type||'tcp').toLowerCase() === typeFilter.value)
  const q = search.value.trim().toLowerCase()
  if (q) list = list.filter(p => (p.name||'').toLowerCase().includes(q))
  return list
})

const total = computed(() => filtered.value.length)
const pageItems = computed(() => {
  const start = (page.value - 1) * pageSize.value
  return filtered.value.slice(start, start + pageSize.value)
})

watch([total, pageSize, typeFilter, search], () => {
  const max = Math.max(1, Math.ceil(total.value / Math.max(pageSize.value, 1)))
  if (page.value > max) page.value = max
})
watch([typeFilter, search], () => { page.value = 1 })

const TYPE_COLORS: Record<string, string> = {
  http:   '#3b82f6',
  https:  '#93c5fd',
  tcp:    '#94a3b8',
  udp:    '#2dd4bf',
  socks5: '#f97316',
  file:   '#fb7185',
  stcp:   '#a78bfa',
  xtcp:   '#f472b6',
}

function typeColor(type: string) {
  return TYPE_COLORS[(type||'tcp').toLowerCase()] ?? '#60a5fa'
}

function openDetail(name: string) { router.push({name: 'proxy-detail', params: {name}}) }
function onKeyOpen(evt: KeyboardEvent, name: string) {
  if (evt.key === 'Enter' || evt.key === ' ') { evt.preventDefault(); openDetail(name) }
}

const maxConns = computed(() => {
  return Math.max(...pageItems.value.map(p => p.curConns ?? 0), 1)
})
</script>

<template>
  <SectionCard :title="t('nav.proxies')">
    <template #extra>
      <span class="proxy-count-badge">{{ total }}</span>
    </template>

    <!-- toolbar -->
    <div class="toolbar">
      <!-- type filter tabs -->
      <div class="type-tabs" role="tablist">
        <button
          v-for="type in allTypes" :key="type"
          type="button" role="tab"
          class="type-tab" :class="{active: typeFilter===type}"
          :style="typeFilter===type && type!=='all' ? {color: typeColor(type), borderColor: typeColor(type)+'55'} : {}"
          @click="typeFilter=type"
        >
          <span v-if="type !== 'all'" class="type-dot" :style="{background: typeColor(type)}"/>
          {{ type === 'all' ? t('clients.filterAll') : type.toUpperCase() }}
        </button>
      </div>

      <label class="search-wrap">
        <svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="6.5" cy="6.5" r="4.5"/><path d="M10.5 10.5L14 14"/></svg>
        <input v-model="search" type="search" class="search-input" :placeholder="t('clients.search')" autocomplete="off"/>
      </label>
    </div>

    <!-- empty -->
    <div v-if="!store.proxies.length || !filtered.length" class="empty-state">
      <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><rect x="8" y="14" width="32" height="22" rx="4"/><path d="M16 14V10M32 14V10M8 22h32"/></svg>
      <p>{{ !store.proxies.length ? t('overview.emptyProxies') : t('clients.filterEmpty') }}</p>
    </div>

    <!-- proxy table -->
    <div v-else class="proxy-table-wrap">
      <table class="proxy-table" role="grid">
        <thead>
          <tr>
            <th>{{ t('proxies.name') }}</th>
            <th>{{ t('proxies.type') }}</th>
            <th>{{ t('proxies.localAddr') }}</th>
            <th class="num-col">{{ t('proxies.connections') }}</th>
            <th class="num-col">{{ t('proxies.trafficIn') }}</th>
            <th class="num-col">{{ t('proxies.trafficOut') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="p in pageItems" :key="p.name"
            class="proxy-row"
            role="button" tabindex="0" :aria-label="p.name"
            @click="openDetail(p.name)" @keydown="onKeyOpen($event, p.name)"
          >
            <td class="name-cell">
              <span class="proxy-name">{{ p.name }}</span>
            </td>
            <td>
              <span class="type-badge" :style="{color: typeColor(p.type), background: typeColor(p.type)+'1a', borderColor: typeColor(p.type)+'44'}">
                {{ (p.type||'tcp').toUpperCase() }}
              </span>
            </td>
            <td class="addr-cell mono">{{ p.localAddr || '—' }}</td>
            <td class="num-col">
              <div class="conn-cell">
                <div class="conn-bar-wrap">
                  <div class="conn-bar" :style="{width: ((p.curConns??0)/maxConns*100).toFixed(1)+'%', background: typeColor(p.type)}"/>
                </div>
                <span class="conn-num">{{ p.curConns ?? 0 }}</span>
              </div>
            </td>
            <td class="num-col mono">{{ p.todayTrafficIn ?? '—' }}</td>
            <td class="num-col mono">{{ p.todayTrafficOut ?? '—' }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <PaginationBar v-model:page="page" v-model:page-size="pageSize" :total="total"/>
  </SectionCard>
</template>

<style scoped>
.proxy-count-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.14rem 0.5rem;
  border-radius: 999px;
  font-size: 0.74rem;
  font-weight: 700;
  color: var(--accent-text);
  background: var(--accent-soft);
  border: 1px solid color-mix(in srgb, var(--accent) 28%, transparent);
  font-variant-numeric: tabular-nums;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
  margin-bottom: 1rem;
}

.type-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.type-tab {
  display: inline-flex;
  align-items: center;
  gap: 0.32rem;
  padding: 0.3rem 0.65rem;
  border-radius: 999px;
  border: 1px solid var(--line);
  background: var(--panel);
  color: var(--muted);
  font: inherit;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
}

.type-tab:hover:not(.active) { color: var(--text); border-color: var(--line-strong); }

.type-tab.active {
  background: var(--accent-soft);
  border-color: color-mix(in srgb, var(--accent) 40%, transparent);
  color: var(--accent-text);
}

.type-dot {
  width: 0.4rem;
  height: 0.4rem;
  border-radius: 50%;
  flex-shrink: 0;
}

/* search */
.search-wrap {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.34rem 0.7rem;
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
  width: 0.85rem; height: 0.85rem;
  fill: none; stroke: var(--muted); stroke-width: 1.8; stroke-linecap: round;
}
.search-input {
  border: 0; background: transparent;
  color: var(--text); font: inherit; font-size: 0.78rem;
  width: 10rem; outline: none;
}
.search-input::placeholder { color: var(--muted); }

/* empty */
.empty-state {
  display: flex; flex-direction: column; align-items: center;
  gap: 0.75rem; padding: 3rem 1rem;
  color: var(--muted);
}
.empty-state svg { width: 2.8rem; height: 2.8rem; opacity: 0.4; }
.empty-state p { margin: 0; font-size: 0.88rem; }

/* table */
.proxy-table-wrap {
  overflow-x: auto;
  border-radius: 10px;
  border: 1px solid var(--line);
}

.proxy-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.84rem;
}

.proxy-table thead tr {
  background: color-mix(in srgb, var(--muted) 6%, transparent);
}

.proxy-table th {
  padding: 0.6rem 0.85rem;
  color: var(--muted);
  font-weight: 600;
  font-size: 0.76rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border-bottom: 1px solid var(--line);
  white-space: nowrap;
}

.proxy-row {
  cursor: pointer;
  transition: background 0.14s;
}

.proxy-row:hover td { background: color-mix(in srgb, var(--accent) 5%, transparent); }
.proxy-row:focus-visible { outline: none; }
.proxy-row:focus-visible td:first-child { box-shadow: inset 2px 0 0 var(--accent); }

.proxy-table td {
  padding: 0.62rem 0.85rem;
  border-bottom: 1px solid var(--line);
  color: var(--text);
  vertical-align: middle;
}

.proxy-table tbody tr:last-child td { border-bottom: none; }

.proxy-name {
  font-weight: 600;
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.82rem;
}

.type-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.12rem 0.5rem;
  border-radius: 6px;
  font-size: 0.68rem;
  font-weight: 700;
  border: 1px solid;
  letter-spacing: 0.04em;
}

.addr-cell {
  font-size: 0.78rem;
  color: var(--muted);
}

.num-col { text-align: right; }

.conn-cell {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  justify-content: flex-end;
}

.conn-bar-wrap {
  width: 4rem;
  height: 0.28rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--muted) 15%, transparent);
  overflow: hidden;
}

.conn-bar {
  height: 100%;
  border-radius: 999px;
  min-width: 2px;
  transition: width 0.4s ease;
  opacity: 0.8;
}

.conn-num {
  font-variant-numeric: tabular-nums;
  font-size: 0.8rem;
  color: var(--text);
  min-width: 1.5rem;
  text-align: right;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.78rem;
  color: var(--muted);
}
</style>
