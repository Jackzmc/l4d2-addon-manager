import { createMemoryHistory, createRouter, createWebHashHistory } from 'vue-router'

import Setup from '../pages/Setup.vue'
import MainView from '../components/MainView.vue'
import Loading from '../pages/Loading.vue'
import LogsPage from '@/pages/Logs.vue'

const routes = [
  { path: '/', component: Loading },
  { path: '/setup', component: Setup, name: "setup" },
  { 
    path: '/app', 
    component: MainView,
    children: [
      { name: "addons-manual", path: 'addons/manual', component: () => import('@/pages/ManagedAddons.vue') },
      { name: "addons-workshop", path: 'addons/workshop',  component: () => import('@/pages/WorkshopAddons.vue')  },
      { name: "settings", path: 'settings',  component: () => import('@/pages/Settings.vue')  },
      { name: "export", path: 'export',  component: () => import('@/pages/Export.vue')  },
      { name: "logs", path: 'logs',  component: LogsPage  },
    ]
  }
]

export const router = createRouter({
  // history: createWebHashHistory(), // TODO: in future can use this to return to same page - but we need to retrigger init() somehow
  history: createMemoryHistory(),
  linkActiveClass: "is-active",
  routes,
})