<script setup lang="ts">
import {computed, ref} from 'vue'
import {useRoute, useRouter} from 'vue-router'
import SectionCard from '@/components/SectionCard.vue'
import TrafficChart from '@/components/TrafficChart.vue'
import TrafficSummary from '@/components/TrafficSummary.vue'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'

const route = useRoute()
const router = useRouter()
const store = useDashboardStore()
const {t} = useLocale()

const trafficRange = ref<'24h'|'7d'>('24h')
const proxyName = computed(() => route.params.name as string)
const proxy = computed(() => store.proxies.find(p => p.name === proxyName.value))

const TYPE_COLORS: Record<string, string> = {
  http:'#3b82f6', https:'#93c5fd', tcp:'#94a3b8',
  udp:'#2dd4bf', socks5:'#f97316', file:'#fb7185', stcp:'#a78bfa', xtcp:'#f472b6',
}

function typeColor(type: string) { return TYPE_COLORS[(type||'tcp').toLowerCase()] ?? '#60a5fa' }

interface FieldRow { label: string; value: string | number | null | undefined; mono?: boolean; highlight?: string }

const fields = computed<FieldRow[]>(() => {
  const p = proxy.value
  if (!p) return []
  return [
    {label: t('proxies.name'), value: p.name, mono: true},
    {label: t('proxies.type'), value: (p.type||'tcp').toUpperCase(), highlight: typeColor(p.type)},
    {label: t('proxies.localAddr'), value: p.localAddr || '—', mono: true},
    {label: t('proxies.remoteAddr'), value: p.remoteAddr || '—', mono: true},
    {label: t('proxies.connections'), value: p.curConns ?? 0},
    {label: t('proxies.trafficIn'), value: p.todayTrafficIn ?? '—', mono: true},
    {label: t('proxies.trafficOut'), value: p.todayTrafficOut ?? '—', mono: true},
    {label: t('proxies.historyConns'), value: p.historyConns ?? 0},
  ]
})
</script>

<template>
  <div v-if="!proxy" class="not-found">
    <div class="not-found-inner">
      <svg viewBox="0 0 64 64" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <rect x="12" y="20" width="40" height="28" rx="5"/>
        <path d="M22 20V14M42 20V14M12 32h40"/>
        <path d="M26 42h12" opacity="0.4"/>
      </svg>
      <p>{{ t('proxies.notFound') }}</p>
      <button class="back-btn" @click="router.push({name:'proxies'})">← {{ t('common.back') }}</button>
    </div>
  </div>

  <div v-else class="proxy-detail">
    <!-- ── Hero ── -->
    <header class="detail-hero">
      <button class="back-icon" :aria-label="t('common.back')" @click="router.back()">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M10 3L5 8l5 5"/></svg>
      </button>

      <div class="hero-icon" :style="{background: typeColor(proxy.type)+'1a', borderColor: typeColor(proxy.type)+'44'}">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <rect x="2" y="7" width="20" height="15" rx="2"/>
          <path d="M16 7V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2"/>
          <path d="M12 12v4M10 14h4"/>
        </svg>
      </div>

      <div class="hero-info">
        <div class="hero-title-row">
          <h1 class="hero-name">{{ proxy.name }}</h1>
          <span class="type-pill" :style="{color:typeColor(proxy.type), background:typeColor(proxy.type)+'1a', borderColor:typeColor(proxy.type)+'44'}">
            {{ (proxy.type||'tcp').toUpperCase() }}
          </span>
          <span class="conn-chip">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"><path d="M2 12V8.5M5.5 12V5M9 12V7M12.5 12V3.5"/></svg>
            {{ proxy.curConns ?? 0 }} {{ t('proxies.connections') }}
          </span>
        </div>
        <div class="hero-addrs">
          <span class="addr-item">
            <span class="addr-label">local</span>
            <span class="mono">{{ proxy.localAddr || '—' }}</span>
          </span>
          <span class="addr-sep">→</span>
          <span class="addr-item">
            <span class="addr-label">remote</span>
            <span class="mono">{{ proxy.remoteAddr || '—' }}</span>
          </span>
        </div>
      </div>
    </header>

    <!-- ── Traffic + Stats ── -->
    <div class="mid-row">
      <SectionCard :title="t('traffic.network')">
        <TrafficSummary
          :traffic-in="proxy.todayTrafficIn ?? 0"
          :traffic-out="proxy.todayTrafficOut ?? 0"
        />
      </SectionCard>

      <SectionCard :title="t('monitor.serverConfig')">
        <div class="fields-grid">
          <div v-for="f in fields" :key="f.label" class="field-row">
            <span class="field-label">{{ f.label }}</span>
            <span
              class="field-val"
              :class="{mono: f.mono}"
              :style="f.highlight ? {color: f.highlight} : {}"
            >{{ f.value }}</span>
          </div>
        </div>
      </SectionCard>
    </div>

    <!-- ── History chart ── -->
    <SectionCard :title="t('traffic.historyAll')">
      <template #extra>
        <div class="range-seg" role="group">
          <button type="button" class="seg-btn" :class="{active: trafficRange==='24h'}" @click="trafficRange='24h'">{{ t('traffic.range24h') }}</button>
          <button type="button" class="seg-btn" :class="{active: trafficRange==='7d'}" @click="trafficRange='7d'">{{ t('traffic.range7d') }}</button>
        </div>
      </template>
      <TrafficChart variant="line" :range="trafficRange" :proxy-name="proxyName" :refresh-ms="6000"/>
    </SectionCard>
  </div>
</template>

<style scoped>
.proxy-detail {
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
.not-found { display: grid; place-items: center; min-height: 20rem; }
.not-found-inner { display: flex; flex-direction: column; align-items: center; gap: 1rem; color: var(--muted); }
.not-found-inner svg { width: 4rem; height: 4rem; opacity: 0.4; }
.not-found-inner p { margin: 0; font-size: 0.9rem; }
.back-btn {
  padding: 0.4rem 1rem; border-radius: 8px;
  border: 1px solid var(--line); background: var(--panel);
  color: var(--text); font: inherit; font-size: 0.85rem; cursor: pointer;
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
}

.back-icon {
  width: 2rem; height: 2rem; border-radius: 8px;
  border: 1px solid var(--line); background: transparent;
  color: var(--muted); display: grid; place-items: center;
  cursor: pointer; flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
  margin-top: 0.18rem;
}
.back-icon:hover { background: var(--panel-hover); color: var(--text); }
.back-icon svg { width: 1rem; height: 1rem; }

.hero-icon {
  width: 3.2rem; height: 3.2rem;
  border-radius: 14px;
  display: grid; place-items: center;
  flex-shrink: 0;
  border: 1px solid;
}
.hero-icon svg {
  width: 1.5rem; height: 1.5rem;
  stroke: currentColor; opacity: 0.85;
}

.hero-info { flex: 1; min-width: 0; }

.hero-title-row {
  display: flex; align-items: center; gap: 0.6rem;
  flex-wrap: wrap; margin-bottom: 0.6rem;
}

.hero-name {
  margin: 0;
  font-size: 1.2rem; font-weight: 700;
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  letter-spacing: -0.02em; color: var(--text);
  word-break: break-all;
}

.type-pill {
  display: inline-flex; align-items: center;
  padding: 0.2rem 0.6rem;
  border-radius: 8px; font-size: 0.72rem; font-weight: 700;
  border: 1px solid; letter-spacing: 0.05em;
}

.conn-chip {
  display: inline-flex; align-items: center; gap: 0.3rem;
  font-size: 0.76rem; color: var(--muted);
  font-variant-numeric: tabular-nums;
}
.conn-chip svg { width: 0.78rem; height: 0.78rem; fill: none; stroke: currentColor; stroke-width: 1.6; stroke-linecap: round; }

.hero-addrs {
  display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;
}

.addr-item {
  display: inline-flex; align-items: center; gap: 0.35rem;
}

.addr-label {
  font-size: 0.68rem; font-weight: 600; text-transform: uppercase;
  letter-spacing: 0.06em; color: var(--muted);
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  padding: 0.08rem 0.4rem; border-radius: 4px;
}

.mono {
  font-family: 'IBM Plex Mono', ui-monospace, monospace;
  font-size: 0.78rem; color: var(--text-secondary);
}

.addr-sep { color: var(--muted); font-weight: 600; }

/* mid row */
.mid-row {
  display: grid;
  grid-template-columns: 1fr 1.4fr;
  gap: 1rem;
  align-items: start;
}

/* fields grid */
.fields-grid { display: flex; flex-direction: column; gap: 0; }

.field-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
  padding: 0.52rem 0.5rem;
  border-bottom: 1px solid var(--line);
  transition: background 0.14s;
}
.field-row:hover { background: color-mix(in srgb, var(--muted) 5%, transparent); }
.field-row:last-child { border-bottom: none; }

.field-label {
  font-size: 0.78rem; color: var(--muted); font-weight: 500; white-space: nowrap;
}

.field-val {
  font-size: 0.8rem; font-weight: 600; color: var(--text);
  text-align: right; font-variant-numeric: tabular-nums;
  max-width: 60%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.field-val.mono { font-family: 'IBM Plex Mono', ui-monospace, monospace; font-size: 0.76rem; }

/* range seg */
.range-seg {
  display: inline-flex; padding: 2px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid var(--line);
}
.seg-btn {
  border: 0; background: transparent;
  color: var(--muted); font: inherit; font-size: 0.75rem; font-weight: 600;
  padding: 0.24rem 0.62rem; border-radius: 999px; cursor: pointer;
  transition: color 0.15s, background 0.15s, box-shadow 0.15s;
}
.seg-btn.active {
  color: var(--text); background: var(--panel);
  box-shadow: 0 1px 3px rgba(0,0,0,0.3);
}
.seg-btn:hover:not(.active) { color: var(--text); }

@media (max-width: 860px) {
  .detail-hero { flex-wrap: wrap; }
  .mid-row { grid-template-columns: 1fr; }
}
</style>
