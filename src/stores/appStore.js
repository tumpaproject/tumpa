import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

let cardPollInterval = null

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
      try {
        this.keys = await invoke('list_keys')
      } catch (e) {
        this.errorMessage = String(e)
      }
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
