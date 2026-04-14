<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import tickSvg from '@/assets/icons/tick_mark.svg'

const router = useRouter()
const store = useAppStore()
const adminPin = ref('')
const newPin = ref('')

onMounted(() => {
  store.setActiveSection('card', 'change-user-pin')
})

async function save() {
  if (!adminPin.value || !newPin.value) {
    alert('Please fill in all fields.')
    return
  }
  try {
    await invoke('change_user_pin', { adminPin: adminPin.value, newPin: newPin.value })
    await store.fetchCardDetails()
    router.push('/card')
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div class="form-view">
    <div class="form-content">
      <h2>Change User Pin</h2>
      <label class="field-label" for="cup-admin">Current Admin Pin</label>
      <PasswordInput id="cup-admin" v-model="adminPin" />
      <label class="field-label" for="cup-new">New User Pin</label>
      <PasswordInput id="cup-new" v-model="newPin" />
    </div>
    <div class="form-footer">
      <div></div>
      <TButton variant="green" :icon="tickSvg" @click="save">Save</TButton>
    </div>
  </div>
</template>

<style scoped>
.form-view { display: flex; flex-direction: column; height: 100%; }
.form-content { flex: 1; padding: 24px 32px; display: flex; flex-direction: column; gap: 8px; max-width: 700px; }
h2 { font-size: 24px; font-weight: 700; margin-bottom: 8px; }
.field-label { font-size: 14px; font-weight: 500; margin-top: 8px; }
.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
