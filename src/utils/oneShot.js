// Helpers for toggling One Shot mode. Thin wrappers around the Rust
// commands + confirmation + store update; shared by the desktop
// sidebar and the mobile overflow menu so the two UIs can't drift.
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'

const ENTER_MESSAGE =
  'Switch to One Shot mode?\n\n' +
  'Your saved keys on disk are kept, but they will be hidden from this ' +
  'session. Any keys you import or generate from now on live only in ' +
  'memory and will be erased when you close Tumpa.'

const EXIT_MESSAGE =
  'Exit One Shot mode?\n\n' +
  'Tumpa will close immediately. Any keys that are only in memory ' +
  'will be erased. Open Tumpa again to return to your saved keys.'

export async function enterOneShot() {
  if (!window.confirm(ENTER_MESSAGE)) return false
  await invoke('enter_one_shot')
  const store = useAppStore()
  store.setMode('one-shot')
  await store.refreshKeys()
  return true
}

export async function exitOneShot() {
  if (!window.confirm(EXIT_MESSAGE)) return false
  // Flip store.mode BEFORE invoking the Rust command. The Rust side
  // calls `app.exit(0)`, which fires the window's close-requested
  // hook on its way out — that hook checks `store.mode === 'one-shot'`
  // to decide whether to show the "you have N keys in memory" prompt.
  // Flipping here ensures the user isn't asked to confirm a second
  // time for the shutdown they just triggered.
  const store = useAppStore()
  store.setMode('persistent')
  try {
    await invoke('exit_one_shot')
  } catch (_) {
    // swallow — the process teardown races the invoke reply
  }
  return true
}

/// Bootstrap — called once on app startup from main.js. Fetches the
/// current mode from the Rust side so a cold launch correctly renders
/// either the banner or the normal UI. Swallows errors so a missing
/// command (older binary) doesn't block startup.
export async function bootstrapMode() {
  const store = useAppStore()
  try {
    const mode = await invoke('get_app_mode')
    store.setMode(mode)
  } catch (e) {
    console.debug('get_app_mode unavailable, staying on persistent:', e)
  }
}
