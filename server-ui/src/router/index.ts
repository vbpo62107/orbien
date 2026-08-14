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

// Navigation guard — redirect to /login on 401 or unauthenticated
router.beforeEach(async (to) => {
  if (to.meta.public) return true
  const auth = useAuthStore()
  // If already marked authenticated, allow through
  if (auth.authenticated) return true
  // Probe the server to check if session cookie is valid
  try {
    const res = await fetch('/api/v1/system/info', { credentials: 'include' })
    if (res.status === 401) return { name: 'login' }
    auth.setAuthenticated(true)
    return true
  } catch {
    return { name: 'login' }
  }
})
