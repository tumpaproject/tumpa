import { defineStore } from 'pinia'
import { markRaw } from 'vue'
import { invoke } from '@tauri-apps/api/core'

let cardPollInterval = null
// Shared across all callers so overlapping refreshes (e.g. SidebarLayout
// + a route view mounting together) collapse into a single IPC.
let refreshInFlight = null

export const useAppStore = defineStore('app', {
  state: () => ({
    keys: [],
    // Set by `refreshKeys` once the store has been populated from the
    // Rust side at least once. Route views check this before firing
    // their own onMounted refresh so the StartView → KeyListView
    // redirect chain doesn't cause two identical IPCs on cold start.
    keysLoaded: false,
    currentFingerprint: '',
    cardConnected: false,
    cardDetails: null,
    pendingKeyGen: null,
    errorMessage: '',
    activeSection: 'keys',
    activeSubItem: '',
    // Runtime platform flag. Populated once at app startup by main.js.
    // Mobile builds skip card routes and use a simplified UI.
    isMobile: false,
    // 'persistent' (disk-backed, the default) or 'one-shot'
    // (in-memory SQLite; see src-tauri/src/commands/mode.rs). Flipped
    // by `enter_one_shot` / `exit_one_shot` Tauri commands; the banner
    // and conditional Generate-key UI read this.
    mode: 'persistent',
  }),

  getters: {
    hasKeys: (state) => state.keys.length > 0,
  },

  actions: {
    async refreshKeys() {
      // Collapse overlapping calls into a single IPC. Without this,
      // two concurrent onMounted hooks on startup (e.g. layout shell +
      // route view) would each pay the full list_keys cost.
      //
      // Uses `list_keys_summary` (schema v4+) so the list view never
      // triggers a per-key rpgp parse. Per-key details are fetched on
      // drill-in via `get_key_details` as before.
      if (refreshInFlight) {
        if (import.meta.env.DEV) {
          console.log('[tumpa/perf] refreshKeys: collapsed into in-flight call')
        }
        return refreshInFlight
      }
      const t0 = import.meta.env.DEV ? performance.now() : 0
      refreshInFlight = (async () => {
        try {
          // markRaw each row so Pinia / Vue doesn't wrap every field
          // in a reactive proxy. The key rows are read-only snapshots
          // returned by Rust; nothing in the UI mutates their fields
          // in place. Profiling showed the deep-reactive wrap adding
          // ~120 ms per refresh on a 137-key store.
          const raw = await invoke('list_keys_summary')
          this.keys = Array.isArray(raw) ? raw.map(markRaw) : raw
          this.keysLoaded = true
          if (import.meta.env.DEV) {
            console.log(
              `[tumpa/perf] refreshKeys: invoke+assign took ${(performance.now() - t0).toFixed(1)}ms, n=${this.keys.length}`
            )
          }
        } catch (e) {
          this.errorMessage = String(e)
        } finally {
          refreshInFlight = null
        }
      })()
      return refreshInFlight
    },

    async checkCard() {
      this.cardConnected = await invoke('is_card_connected')
      return this.cardConnected
    },

    async fetchCardDetails() {
      try {
        this.cardDetails = await invoke('get_card_details')
      } catch (e) {
        this.errorMessage = String(e)
      }
    },

    clearCardDetails() {
      this.cardDetails = null
    },

    startCardPolling() {
      if (cardPollInterval) return
      cardPollInterval = setInterval(async () => {
        try {
          this.cardConnected = await invoke('is_card_connected')
        } catch {
          this.cardConnected = false
        }
      }, 2000)
    },

    stopCardPolling() {
      if (cardPollInterval) {
        clearInterval(cardPollInterval)
        cardPollInterval = null
      }
    },

    setCurrentFingerprint(fp) {
      this.currentFingerprint = fp
    },

    setActiveSection(section, subItem = '') {
      this.activeSection = section
      this.activeSubItem = subItem
    },

    setError(msg) {
      this.errorMessage = msg
    },

    setPendingKeyGen(params) {
      this.pendingKeyGen = params
    },

    clearPendingKeyGen() {
      this.pendingKeyGen = null
    },

    clearError() {
      this.errorMessage = ''
    },

    setMobile(value) {
      this.isMobile = !!value
    },

    setMode(mode) {
      this.mode = mode === 'one-shot' ? 'one-shot' : 'persistent'
    },
  },
})
