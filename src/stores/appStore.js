import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useAppStore = defineStore('app', {
  state: () => ({
    keys: [],
    currentFingerprint: '',
    cardConnected: false,
    cardDetails: null,
    errorMessage: '',
    activeSection: 'keys',
    activeSubItem: '',
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

    clearError() {
      this.errorMessage = ''
    },
  },
})
