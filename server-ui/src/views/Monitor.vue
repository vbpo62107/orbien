<script setup lang="ts">
import {computed, ref} from 'vue'
import ConfigValue from '@/components/ConfigValue.vue'
import DescList, {type DescItem} from '@/components/DescList.vue'
import DonutChart, {type ChartSlice} from '@/components/DonutChart.vue'
import SectionCard from '@/components/SectionCard.vue'
import StatCard from '@/components/StatCard.vue'
import TrafficChart from '@/components/TrafficChart.vue'
import TrafficSummary from '@/components/TrafficSummary.vue'
import type {TrafficRange} from '@/api/client'
import {useDashboardStore} from '@/stores/dashboard'
import {useLocale} from '@/composables/useLocale'
import {isUnsetPort, isUnsetText} from '@/utils/format'

const store = useDashboardStore()
const {t} = useLocale()
const trafficRange = ref<TrafficRange>('24h')
const chartVariant = ref<'bar' | 'line'>('line')

const PROXY_COLORS: Record<string, string> = {
  http: '#3b82f6',
  https: '#93c5fd',
  tcp: '#cbd5e1',
  udp: '#2dd4bf',
  socks5: '#f97316',
  file: '#fb7185',
  stcp: '#a78bfa',
  xtcp: '#f472b6',
}

const FALLBACK_COLORS = ['#60a5fa', '#34d399', '#fbbf24', '#f87171', '#818cf8', '#94a3b8']

const cfg = computed(() => store.info?.config)
const status = computed(() => store.info?.status)

const trafficIn = computed(() => status.value?.totalTrafficIn ?? 0)
const trafficOut = computed(() => status.value?.totalTrafficOut ?? 0)

const onlineClients = computed(() => status.value?.clientCounts ?? 0)
const totalClients = computed(() =>
    Math.max(status.value?.totalClientCounts ?? 0, onlineClients.value),
)

const proxyTotal = computed(() => {
  const m = status.value?.proxyTypeCount || {}
  return Object.values(m).reduce((a, b) => a + b, 0)
})

const chartSlices = computed<ChartSlice[]>(() => {
  const m = status.value?.proxyTypeCount || {}
  const entries = Object.entries(m).sort(([a], [b]) => a.localeCompare(b))
  return entries.map(([key, value], i) => ({
    key,
    label: key,
    value,
    color: PROXY_COLORS[key.toLowerCase()] ?? FALLBACK_COLORS[i % FALLBACK_COLORS.length]!,
  }))
})

function formatHeartbeat(secs: number | undefined | null): string {
  if (secs == null) return '—'
  if (secs < 0) return t('common.disabled')
  return `${secs}s`
}

type ConfigValueType = 'text' | 'port' | 'bool' | 'raw'

interface ConfigField {
  key: string
  label: string
  type: ConfigValueType
  value: string | number | boolean | null
}

const configFields = computed<ConfigField[]>(() => {
  const c = cfg.value
  if (!c) return []
  const fields: ConfigField[] = [
    {key: 'listen', label: t('monitor.bindAddr'), type: 'raw', value: `${c.bindAddr || '—'}:${c.bindPort ?? '—'}`},
  ]
  if (!isUnsetPort(c.kcpBindPort)) fields.push({key: 'kcp', label: t('monitor.kcpBindPort'), type: 'port', value: c.kcpBindPort})
  if (!isUnsetPort(c.quicBindPort)) fields.push({key: 'quic', label: t('monitor.quicBindPort'), type: 'port', value: c.quicBindPort})
  if (!isUnsetPort(c.vhostHTTPPort)) fields.push({key: 'http', label: t('monitor.vhostHTTPPort'), type: 'port', value: c.vhostHTTPPort})
  if (!isUnsetPort(c.vhostHTTPSPort)) fields.push({key: 'https', label: t('monitor.vhostHTTPSPort'), type: 'port', value: c.vhostHTTPSPort})
  if (!isUnsetText(c.subDomainHost ?? '')) fields.push({key: 'subdomain', label: t('monitor.subDomainHost'), type: 'text', value: c.subDomainHost})
  fields.push(
    {key: 'mux', label: t('monitor.tcpMux'), type: 'bool', value: c.tcpMux},
    {key: 'tls', label: t('monitor.tlsForce'), type: 'bool', value: c.tlsForce},
    {key: 'pool', label: t('monitor.metricMaxPool'), type: 'raw', value: c.maxPoolCount ?? 0},
    {key: 'heartbeat', label: t('monitor.metricHeartbeat'), type: 'raw', value: formatHeartbeat(c.heartbeatTimeout)},
  )
  return fields
})

const descItems = computed<DescItem[]>(() =>
    configFields.value.map(({key, label}) => ({key, label})),
)
</script>

<template>
  <div class="monitor">
    <!-- ── KPI row ── -->
    <section class="kpi-grid" aria-label="Overview metrics">
      <StatCard :label="t('overview.totalClients')" icon="users" tone="blue">
        {{ totalClients }}
      </StatCard>
      <StatCard :label="t('overview.onlineClients')" icon="user" tone="green">
        {{ onlineClients }}
      </StatCard>
      <StatCard :label="t('overview.proxies')" icon="proxies" tone="violet">
        {{ proxyTotal }}
      </StatCard>
      <StatCard :label="t('overview.connections')" icon="link" tone="orange">
        {{ status?.curConns ?? 0 }}
      </StatCard>
    </section>

    <!-- ── Traffic + Proxy distribution ── -->
    <div class="middle-row">
      <SectionCard class="panel traffic-panel" :title="t('traffic.network')">
        <template #extra>
          <span class="badge">{{ t('traffic.today') }}</span>
        </template>
        <TrafficSummary :traffic-in="trafficIn" :traffic-out="trafficOut"/>
      </SectionCard>

      <SectionCard class="panel donut-panel" :title="t('monitor.proxyDist')">
        <DonutChart :slices="chartSlices"/>
      </SectionCard>
    </div>

    <!-- ── Traffic history ── -->
    <SectionCard class="history-panel" :title="t('traffic.historyAll')">
      <template #extra>
        <div class="chart-toolbar">
          <div class="seg" role="group" :aria-label="t('traffic.chartType')">
            <button type="button" class="seg-btn" :class="{active: chartVariant==='line'}" @click="chartVariant='line'">{{ t('traffic.chartLine') }}</button>
            <button type="button" class="seg-btn" :class="{active: chartVariant==='bar'}" @click="chartVariant='bar'">{{ t('traffic.chartBar') }}</button>
          </div>
          <div class="seg" role="group" :aria-label="t('traffic.range')">
            <button type="button" class="seg-btn" :class="{active: trafficRange==='24h'}" @click="trafficRange='24h'">{{ t('traffic.range24h') }}</button>
            <button type="button" class="seg-btn" :class="{active: trafficRange==='7d'}" @click="trafficRange='7d'">{{ t('traffic.range7d') }}</button>
          </div>
        </div>
      </template>
      <TrafficChart :variant="chartVariant" :range="trafficRange" :refresh-ms="5000"/>
    </SectionCard>

    <!-- ── Server config ── -->
    <SectionCard class="config-panel" :title="t('monitor.serverConfig')">
      <div v-if="configFields.length" class="config-grid">
        <div v-for="field in configFields" :key="field.key" class="config-row">
          <span class="config-label">{{ field.label }}</span>
          <span class="config-val"><ConfigValue :type="field.type" :value="field.value"/></span>
        </div>
      </div>
      <p v-else class="empty-hint">{{ t('overview.emptyConfig') }}</p>
    </SectionCard>
  </div>
</template>

<style scoped>
.monitor {
  display: flex;
  flex-direction: column;
  gap: 1.1rem;
  animation: page-in 0.35s ease both;
}

@keyframes page-in {
  from { opacity: 0; transform: translateY(6px); }
  to   { opacity: 1; transform: translateY(0); }
}

/* KPI grid */
.kpi-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1rem;
  margin: 0;
}

/* Middle row */
.middle-row {
  display: grid;
  grid-template-columns: minmax(280px, 1.1fr) minmax(240px, 0.9fr);
  gap: 1rem;
  align-items: stretch;
}

.panel { height: 100%; }

/* Badge */
.badge {
  display: inline-flex;
  align-items: center;
  padding: 0.16rem 0.52rem;
  border-radius: 999px;
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--accent-text);
  background: var(--accent-soft);
  border: 1px solid color-mix(in srgb, var(--accent) 28%, transparent);
  letter-spacing: 0.01em;
}

/* Chart toolbar */
.chart-toolbar {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  justify-content: flex-end;
}

.seg {
  display: inline-flex;
  padding: 2px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--muted) 10%, transparent);
  border: 1px solid var(--line);
}

.seg-btn {
  border: 0;
  background: transparent;
  color: var(--muted);
  font: inherit;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.26rem 0.65rem;
  border-radius: 999px;
  cursor: pointer;
  transition: color 0.15s, background 0.15s, box-shadow 0.15s;
}

.seg-btn.active {
  color: var(--text);
  background: var(--panel);
  box-shadow: 0 1px 3px rgba(0,0,0,0.3);
}

.seg-btn:hover:not(.active) { color: var(--text); }

/* Config grid — 2-column key/value pairs */
.config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
}

.config-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
  padding: 0.58rem 0.65rem;
  border-bottom: 1px solid var(--line);
  border-radius: 0;
  transition: background 0.15s;
}

.config-row:hover { background: color-mix(in srgb, var(--muted) 5%, transparent); }
.config-row:last-child, .config-row:nth-last-child(2):nth-child(odd) { border-bottom: none; }

.config-row:nth-child(odd) { border-right: 1px solid var(--line); }

.config-label {
  font-size: 0.8rem;
  color: var(--muted);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.config-val {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--text);
  font-variant-numeric: tabular-nums;
  text-align: right;
}

.empty-hint {
  margin: 0;
  color: var(--muted);
  font-size: 0.88rem;
  text-align: center;
  padding: 1.5rem 0;
}

/* Responsive */
@media (max-width: 1200px) {
  .kpi-grid { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 1000px) {
  .middle-row { grid-template-columns: 1fr; }
}
@media (max-width: 640px) {
  .kpi-grid { grid-template-columns: 1fr 1fr; }
  .config-grid { grid-template-columns: 1fr; }
  .config-row:nth-child(odd) { border-right: none; }
}
@media (max-width: 400px) {
  .kpi-grid { grid-template-columns: 1fr; }
}
</style>
