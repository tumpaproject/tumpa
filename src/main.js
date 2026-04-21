import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import { useAppStore } from './stores/appStore'
import { isMobilePlatform } from './utils/platform'
import { bootstrapMode } from './utils/oneShot'
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

// Pull the current app mode from the Rust side so One Shot survives a
// webview reload (and is always accurate on cold start).
bootstrapMode()

// Desktop-only: intercept the native window close so we can warn
// about losing in-memory keys. Mobile lifecycle doesn't expose an
// equivalent hook — the banner carries that weight on phones.
if (!store.isMobile) {
  import('@tauri-apps/api/window')
    .then(({ getCurrentWindow }) => {
      const win = getCurrentWindow()
      win.onCloseRequested(async (evt) => {
        if (store.mode !== 'one-shot') return
        const count = store.keys.length
        const noun = count === 1 ? 'key' : 'keys'
        const ok = window.confirm(
          `You have ${count} ${noun} in One Shot memory.\n\n` +
          'Closing Tumpa will erase them with no way to recover. ' +
          'Close anyway?'
        )
        if (!ok) evt.preventDefault()
      })
    })
    .catch((e) => console.debug('window close hook unavailable:', e))
}

app.mount('#app')
