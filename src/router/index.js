import { createRouter, createWebHistory } from 'vue-router'

import StartView from '@/views/StartView.vue'
import KeyListView from '@/views/KeyListView.vue'
import GenerateKeyView from '@/views/GenerateKeyView.vue'
import GeneratingView from '@/views/GeneratingView.vue'
import CardDetailsView from '@/views/CardDetailsView.vue'
import EditNameView from '@/views/EditNameView.vue'
import EditUrlView from '@/views/EditUrlView.vue'
import ChangeUserPinView from '@/views/ChangeUserPinView.vue'
import ChangeAdminPinView from '@/views/ChangeAdminPinView.vue'
import ErrorView from '@/views/ErrorView.vue'

const routes = [
  { path: '/', name: 'start', component: StartView },
  { path: '/keys', name: 'key-list', component: KeyListView },
  { path: '/keys/generate', name: 'generate-key', component: GenerateKeyView },
  { path: '/keys/generating', name: 'generating', component: GeneratingView },
  { path: '/card', name: 'card-details', component: CardDetailsView },
  { path: '/card/edit-name', name: 'edit-name', component: EditNameView },
  { path: '/card/edit-url', name: 'edit-url', component: EditUrlView },
  { path: '/card/change-user-pin', name: 'change-user-pin', component: ChangeUserPinView },
  { path: '/card/change-admin-pin', name: 'change-admin-pin', component: ChangeAdminPinView },
  { path: '/error', name: 'error', component: ErrorView },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
