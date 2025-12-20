import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'home',
    component: () => import('../views/HomeView.vue'),
    meta: { title: 'Whis' },
  },
  {
    path: '/presets',
    name: 'presets',
    component: () => import('../views/PresetsView.vue'),
    meta: { title: 'Presets' },
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('../views/SettingsView.vue'),
    meta: { title: 'Settings' },
  },
]

const router = createRouter({
  // Use hash mode for Tauri (no server-side routing)
  history: createWebHashHistory(),
  routes,
})

// Update document title on navigation
router.afterEach((to) => {
  const title = to.meta.title as string | undefined
  document.title = title ? `${title} - Whis` : 'Whis'
})

export default router
