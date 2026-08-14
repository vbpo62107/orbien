import { createRouter, createWebHashHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import Monitor from '@/views/Monitor.vue'
import Proxies from '@/views/Proxies.vue'
import ProxyDetail from '@/views/ProxyDetail.vue'
import Clients from '@/views/Clients.vue'
import ClientDetail from '@/views/ClientDetail.vue'
import Login from '@/views/Login.vue'

export const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/login', name: 'login', component: Login, meta: { public: true } },
    { path: '/', name: 'monitor', component: Monitor },
    { path: '/proxies', name: 'proxies', component: Proxies },
    { path: '/proxies/:name', name: 'proxy-detail', component: ProxyDetail },
    { path: '/clients', name: 'clients', component: Clients },
    { path: '/clients/:runId', name: 'client-detail', component: ClientDetail },
    { path: '/overview', redirect: '/' },
  ],
})

/**
 * Navigation guard.
 *
 * - Public routes (meta.public) always pass through.
 * - If auth store already says authenticated, allow through immediately.
 * - Otherwise call auth.fetchStatus() which hits /api/v1/auth/status
 *   (same endpoint used by the rest of the app) instead of a raw fetch
 *   against /api/v1/system/info, so the logic stays consistent.
 */
router.beforeEach(async (to) => {
  if (to.meta.public) return true
  const auth = useAuthStore()
  if (auth.authenticated) return true
  try {
    const ok = await auth.fetchStatus()
    if (ok) return true
    return { name: 'login' }
  } catch {
    return { name: 'login' }
  }
})
