import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

let cardPollInterval = null
// Shared across all callers so overlapping refreshes (e.g. SidebarLayout
// + a route view mounting together) collapse into a single IPC.
let refreshInFlight = null

export const useAppStore = defineStore('app', {
  state: () => ({
    keys: [],
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
      if (refreshInFlight) return refreshInFlight
      refreshInFlight = (async () => {
        try {
          this.keys = await invoke('list_keys_summary')
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
