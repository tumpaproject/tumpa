<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import PasswordInput from '@/components/PasswordInput.vue'

const router = useRouter()
const store = useAppStore()

const slots = ref([])
const loading = ref(true)
const error = ref('')
const adminPin = ref('')
const changingSlot = ref(null)
const selectedMode = ref('')
const changeError = ref('')

const touchModes = ['Off', 'On', 'Fixed', 'Cached', 'CachedFixed']

onMounted(async () => {
  store.setActiveSection('card', 'touch-mode')
  await loadTouchModes()
})

async function loadTouchModes() {
  loading.value = true
  error.value = ''
  try {
    slots.value = await invoke('get_card_touch_modes')
  } catch (e) {
    error.value = String(e)
  }
  loading.value = false
}

function startChange(slot) {
  changingSlot.value = slot.slot
  selectedMode.value = slot.mode
  adminPin.value = ''
  changeError.value = ''
}

function cancelChange() {
  changingSlot.value = null
  changeError.value = ''
}

async function applyChange() {
  if (!adminPin.value) {
    changeError.value = 'Please enter the admin PIN.'
    return
  }

  if (selectedMode.value === 'Fixed' || selectedMode.value === 'CachedFixed') {
    if (!confirm(`WARNING: Setting ${selectedMode.value} is PERMANENT and cannot be changed, even with a factory reset. Are you sure?`)) {
      return
    }
  }

  changeError.value = ''
  try {
    await invoke('set_card_touch_mode', {
      slot: changingSlot.value,
      mode: selectedMode.value,
      adminPin: adminPin.value,
    })
    changingSlot.value = null
    adminPin.value = ''
    await loadTouchModes()
  } catch (e) {
    changeError.value = String(e)
  }
}
</script>

<template>
  <div class="touch-view">
    <h1>Touch Mode Settings</h1>
    <p class="subtitle">Configure whether physical touch is required for each key slot on the smartcard.</p>

    <div v-if="loading" class="loading">Loading touch mode settings...</div>
    <div v-else-if="error" class="error-msg" role="alert">{{ error }}</div>

    <div v-else class="slots">
      <div v-for="slot in slots" :key="slot.slot" class="slot-card">
        <div class="slot-header">
          <span class="slot-name">{{ slot.slot }}</span>
          <span class="slot-mode" :class="slot.mode.toLowerCase()">{{ slot.mode }}</span>
          <TButton
            v-if="slot.supported && !slot.is_fixed"
            variant="white"
            thin
            @click="startChange(slot)"
          >Change</TButton>
          <span v-if="slot.is_fixed" class="fixed-warning">Permanent</span>
          <span v-if="!slot.supported" class="not-supported">Not supported</span>
        </div>

        <div v-if="changingSlot === slot.slot" class="slot-edit">
          <div class="edit-row">
            <label class="edit-label" for="touch-mode-select">New mode:</label>
            <select id="touch-mode-select" v-model="selectedMode" class="mode-select">
              <option v-for="m in touchModes" :key="m" :value="m">{{ m }}</option>
            </select>
          </div>
          <div v-if="selectedMode === 'Fixed' || selectedMode === 'CachedFixed'" class="fixed-alert" role="alert">
            This setting is PERMANENT and cannot be changed even with a factory reset!
          </div>
          <div class="edit-row">
            <label class="edit-label" for="touch-admin-pin">Admin PIN:</label>
            <PasswordInput id="touch-admin-pin" v-model="adminPin" placeholder="Admin PIN" />
          </div>
          <div v-if="changeError" class="change-error" role="alert">{{ changeError }}</div>
          <div class="edit-actions">
            <TButton variant="green" thin @click="applyChange">Apply</TButton>
            <TButton variant="default" thin @click="cancelChange">Cancel</TButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.touch-view {
  padding: 24px 32px;
}

h1 {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 8px;
}

.subtitle {
  font-size: 14px;
  color: var(--color-text-muted);
  margin-bottom: 24px;
}

.loading, .error-msg {
  color: var(--color-text-muted);
  font-size: 14px;
}

.error-msg { color: var(--color-red); }

.slots {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 600px;
}

.slot-card {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.slot-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  background: var(--color-bg-light);
}

.slot-name {
  font-size: 14px;
  font-weight: 600;
  min-width: 120px;
}

.slot-mode {
  font-size: 13px;
  font-weight: 500;
  padding: 2px 10px;
  border-radius: 3px;
}

.slot-mode.off { background: var(--color-border); color: var(--color-text-muted); }
.slot-mode.on { background: #D1FAE5; color: #065F46; }
.slot-mode.fixed { background: #FEE2E2; color: #991B1B; }
.slot-mode.cached { background: #FEF9C3; color: #A16207; }
.slot-mode.cachedfixed { background: #FEE2E2; color: #991B1B; }

.fixed-warning {
  font-size: 12px;
  color: var(--color-red);
  font-weight: 500;
  font-style: italic;
  margin-left: auto;
}

.not-supported {
  font-size: 12px;
  color: var(--color-text-muted);
  font-style: italic;
  margin-left: auto;
}

.slot-edit {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  border-top: 1px solid var(--color-border);
}

.edit-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.edit-label {
  font-size: 13px;
  font-weight: 500;
  min-width: 80px;
}

.mode-select {
  padding: 6px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 6px;
  font-size: 13px;
  font-family: var(--font-family);
  background: white;
}

.edit-row .password-input {
  max-width: 200px;
}

.fixed-alert {
  background: #FEF2F2;
  border: 1px solid #FCA5A5;
  border-radius: 4px;
  padding: 8px 12px;
  color: var(--color-red);
  font-size: 13px;
  font-weight: 500;
}

.change-error {
  color: var(--color-red);
  font-size: 13px;
}

.edit-actions {
  display: flex;
  gap: 8px;
}
</style>
