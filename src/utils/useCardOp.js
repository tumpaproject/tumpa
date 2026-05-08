// Helper for mobile card-write flows. Wraps a Tauri invoke() in the
// NFC overlay state machine: arms a `plugin:tumpa-card:card-connected`
// listener so the overlay switches from "waiting" to "connected" when
// the plugin reports SELECT success, toggles a `busy` ref around the
// call, and exposes `error` for inline error display.
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export function useCardOp() {
  const busy = ref(false)
  const phase = ref('waiting')
  const error = ref('')

  let unlistenCardConnected = null

  onMounted(async () => {
    try {
      unlistenCardConnected = await listen('plugin:tumpa-card:card-connected', () => {
        phase.value = 'connected'
      })
    } catch {
      // Plugin unavailable (desktop/web). Overlay stays on 'waiting',
      // which is still accurate for the cold path.
    }
  })

  onUnmounted(() => {
    if (typeof unlistenCardConnected === 'function') unlistenCardConnected()
  })

  async function run(cmd, args) {
    error.value = ''
    phase.value = 'waiting'
    busy.value = true
    try {
      return await invoke(cmd, args)
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      busy.value = false
    }
  }

  function cancel() {
    busy.value = false
  }

  return { busy, phase, error, run, cancel }
}
