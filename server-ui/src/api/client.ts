import type {
    Page,
    SystemInfo,
    ClientInfo,
    ProxyInfo,
    ProxyTrafficResp,
    ApiResponse,
} from '@/types/api'
import { ApiError } from './errors'

async function api<T>(path: string, init?: RequestInit): Promise<T> {
    const res = await fetch(path, { credentials: 'include', ...init })
    if (res.status === 401) throw new ApiError('unauthorized')
    if (!res.ok) throw new ApiError('http', { status: res.status, statusText: res.statusText })
    const body = (await res.json()) as ApiResponse<T>
    if (body.code !== 200) throw new ApiError('api', { msg: body.msg })
    return body.data
}

// ── auth ──────────────────────────────────────────────────────────────────────

export interface AuthStatus {
    webauthn: boolean
    password: boolean
}

/**
 * GET /api/v1/auth/status — always public, no credentials needed.
 * Returns which login methods the server has configured.
 * Silently returns defaults (password only) on any error so the UI never
 * breaks even if the endpoint is momentarily unreachable.
 */
export async function fetchAuthStatus(): Promise<AuthStatus> {
    try {
        return await api<AuthStatus>('/api/v1/auth/status')
    } catch {
        return { webauthn: false, password: true }
    }
}

// ── system ────────────────────────────────────────────────────────────────────

export function fetchSystemInfo() {
    return api<SystemInfo>('/api/v1/system/info')
}

export function fetchClients(page = 1, pageSize = 200) {
    return api<Page<ClientInfo>>(`/api/v1/clients?page=${page}&pageSize=${pageSize}`)
}

export function fetchClient(runId: string) {
    return api<ClientInfo>(`/api/v1/clients/${encodeURIComponent(runId)}`)
}

export function kickClient(runId: string) {
    return api<unknown>(`/api/v1/clients/${encodeURIComponent(runId)}/kick`, { method: 'POST' })
}

export type ProxyListParams = {
    page?: number
    pageSize?: number
    clientId?: string
    q?: string
}

export function fetchProxies(pageOrParams: number | ProxyListParams = 1, pageSize = 200) {
    const params: ProxyListParams =
        typeof pageOrParams === 'number' ? { page: pageOrParams, pageSize } : pageOrParams
    const qs = new URLSearchParams()
    qs.set('page', String(params.page ?? 1))
    qs.set('pageSize', String(params.pageSize ?? 200))
    if (params.clientId) qs.set('clientId', params.clientId)
    if (params.q) qs.set('q', params.q)
    return api<Page<ProxyInfo>>(`/api/v1/proxies?${qs.toString()}`)
}

export type TrafficRange = '7d' | '24h'

function trafficQuery(range: TrafficRange = '7d') {
    return range === '24h' ? '?range=24h' : '?range=7d'
}

export function fetchProxyTraffic(name: string, range: TrafficRange = '7d') {
    return api<ProxyTrafficResp>(
        `/api/v1/proxies/${encodeURIComponent(name)}/traffic${trafficQuery(range)}`,
    )
}

export function fetchSystemTraffic(range: TrafficRange = '7d') {
    return api<ProxyTrafficResp>(`/api/v1/system/traffic${trafficQuery(range)}`)
}
