import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import { useAppStore } from './stores/appStore'
import { isMobilePlatform } from './utils/platform'
import { bootstrapMode } from './utils/oneShot'
import './styles/main.css'
// vue-virtual-scroller keeps only the on-screen key rows in the DOM.
// Without it, WebKitGTK paints ~700 ms for a 137-key list on cold
// start; with it, the DOM stays small regardless of store size.
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'
import VueVirtualScroller from 'vue-virtual-scroller'

// Perf-trace hooks — only active in Vite dev builds so production
// bundles don't ship the console noise. Vite statically removes the
// dead branch at build time when import.meta.env.DEV is false.
const __perfBootStart = import.meta.env.DEV ? performance.now() : 0
if (import.meta.env.DEV) {
  console.log('[tumpa/perf] main.js module eval')
}

const app = createApp(App)
const pinia = createPinia()
app.use(pinia)
app.use(router)
app.use(VueVirtualScroller)

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
if (import.meta.env.DEV) {
  console.log(
    `[tumpa/perf] app.mount done at ${(performance.now() - __perfBootStart).toFixed(1)}ms`
  )
  requestAnimationFrame(() => {
    console.log(
      `[tumpa/perf] first rAF after mount at ${(performance.now() - __perfBootStart).toFixed(1)}ms`
    )
  })
}

// Hold the boot splash for a minimum visible duration, then fade it
// out. Without a floor, release builds mount Vue fast enough that
// WebKit never commits a frame with the splash painted — the user
// sees the default (white/black) window bg instead of the branded
// screen. 400 ms + 220 ms fade matches typical desktop-app feel.
{
  const splash = document.getElementById('tumpa-boot')
  if (splash) {
    const MIN_SPLASH_MS = 400
    const FADE_MS = 220
    setTimeout(() => {
      splash.classList.add('fade-out')
      setTimeout(() => splash.remove(), FADE_MS + 50)
    }, MIN_SPLASH_MS)
  }
}
