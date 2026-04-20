import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import { useAppStore } from './stores/appStore'
import { isMobilePlatform } from './utils/platform'
import './styles/main.css'

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
app.use(router)

// Prime the mobile flag before mounting so the layout picks the right
// component on first render. The router also reads isMobilePlatform()
// directly to decide which views to mount.
const store = useAppStore(pinia)
store.setMobile(isMobilePlatform())

app.mount('#app')
