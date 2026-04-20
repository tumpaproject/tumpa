import { createRouter, createWebHistory } from 'vue-router'
import { isMobilePlatform } from '@/utils/platform'

import StartView from '@/views/StartView.vue'
import KeyListView from '@/views/KeyListView.vue'
import GenerateKeyView from '@/views/GenerateKeyView.vue'
import GeneratingView from '@/views/GeneratingView.vue'
import CardDetailsView from '@/views/CardDetailsView.vue'
import EditNameView from '@/views/EditNameView.vue'
import EditUrlView from '@/views/EditUrlView.vue'
import ChangeUserPinView from '@/views/ChangeUserPinView.vue'
import ChangeAdminPinView from '@/views/ChangeAdminPinView.vue'
import KeyDetailsView from '@/views/KeyDetailsView.vue'
import AddUserIdView from '@/views/AddUserIdView.vue'
import UserDetailsView from '@/views/UserDetailsView.vue'
import ChangeKeyPasswordView from '@/views/ChangeKeyPasswordView.vue'
import UploadToCardView from '@/views/UploadToCardView.vue'
import UploadingView from '@/views/UploadingView.vue'
import TouchModeView from '@/views/TouchModeView.vue'
import ErrorView from '@/views/ErrorView.vue'

import StartMobile from '@/views-mobile/StartMobile.vue'
import KeyListMobile from '@/views-mobile/KeyListMobile.vue'
import GenerateKeyMobile from '@/views-mobile/GenerateKeyMobile.vue'
import KeyDetailsMobile from '@/views-mobile/KeyDetailsMobile.vue'

const desktopRoutes = [
  { path: '/', name: 'start', component: StartView, meta: { title: 'Welcome' } },
  { path: '/keys', name: 'key-list', component: KeyListView, meta: { title: 'Keys' } },
  { path: '/keys/generate', name: 'generate-key', component: GenerateKeyView, meta: { title: 'Generate Key' } },
  { path: '/keys/generating', name: 'generating', component: GeneratingView, meta: { title: 'Generating Key' } },
  { path: '/keys/:fingerprint', name: 'key-details', component: KeyDetailsView, props: true, meta: { title: 'Key Details' } },
  { path: '/keys/:fingerprint/add-uid', name: 'add-uid', component: AddUserIdView, props: true, meta: { title: 'Add User ID' } },
  { path: '/keys/:fingerprint/uid/:uidIndex', name: 'user-details', component: UserDetailsView, props: true, meta: { title: 'User Details' } },
  { path: '/keys/:fingerprint/change-password', name: 'change-key-password', component: ChangeKeyPasswordView, props: true, meta: { title: 'Change Key Password' } },
  { path: '/card/upload', name: 'upload-to-card', component: UploadToCardView, meta: { title: 'Upload to Card' } },
  { path: '/card/uploading', name: 'uploading', component: UploadingView, meta: { title: 'Uploading to Card' } },
  { path: '/card', name: 'card-details', component: CardDetailsView, meta: { title: 'Smart Card Details' } },
  { path: '/card/edit-name', name: 'edit-name', component: EditNameView, meta: { title: 'Edit Card Name' } },
  { path: '/card/edit-url', name: 'edit-url', component: EditUrlView, meta: { title: 'Edit Public URL' } },
  { path: '/card/change-user-pin', name: 'change-user-pin', component: ChangeUserPinView, meta: { title: 'Change User PIN' } },
  { path: '/card/change-admin-pin', name: 'change-admin-pin', component: ChangeAdminPinView, meta: { title: 'Change Admin PIN' } },
  { path: '/card/touch-mode', name: 'touch-mode', component: TouchModeView, meta: { title: 'Touch Mode' } },
  { path: '/error', name: 'error', component: ErrorView, meta: { title: 'Error' } },
]

// Mobile surface is intentionally small: list, details, generate.
// Card + UID management + keyserver flows stay desktop-only for Phase 1.
const mobileRoutes = [
  { path: '/', name: 'start', component: StartMobile, meta: { title: 'Tumpa' } },
  { path: '/keys', name: 'key-list', component: KeyListMobile, meta: { title: 'Keys' } },
  { path: '/keys/generate', name: 'generate-key', component: GenerateKeyMobile, meta: { title: 'Generate key' } },
  { path: '/keys/generating', name: 'generating', component: GeneratingView, meta: { title: 'Generating…' } },
  { path: '/keys/:fingerprint', name: 'key-details', component: KeyDetailsMobile, props: true, meta: { title: 'Key' } },
  { path: '/error', name: 'error', component: ErrorView, meta: { title: 'Error' } },
  // Anything else on mobile falls back to the start screen.
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

const router = createRouter({
  history: createWebHistory(),
  routes: isMobilePlatform() ? mobileRoutes : desktopRoutes,
})

router.afterEach((to) => {
  const title = to.meta?.title
  document.title = title ? `${title} - Tumpa` : 'Tumpa'
})

export default router
