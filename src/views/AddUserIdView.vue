<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'

const props = defineProps({ fingerprint: String })
const router = useRouter()
const store = useAppStore()

const name = ref('')
const email = ref('')
const password = ref('')

async function create() {
  if (!name.value.trim() || !email.value.trim()) {
    alert('Please enter both name and email.')
    return
  }
  if (!password.value) {
    alert('Please enter the key password.')
    return
  }
  try {
    await invoke('add_user_id', {
      fingerprint: props.fingerprint,
      name: name.value.trim(),
      email: email.value.trim(),
      password: password.value,
    })
    await store.refreshKeys()
    router.back()
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div class="form-view">
    <div class="form-content">
      <h2>Add new user ID</h2>

      <label class="field-label">Name:</label>
      <input type="text" v-model="name" />
      <span class="field-hint">Full name of the key owner</span>

      <label class="field-label">Email:</label>
      <input type="text" v-model="email" />

      <label class="field-label">Key Password:</label>
      <input type="password" v-model="password" />
    </div>

    <div class="form-footer">
      <TButton variant="default" :icon="backIconSvg" @click="router.back()">Back</TButton>
      <TButton variant="green" :icon="tickSvg" @click="create">Create</TButton>
    </div>
  </div>
</template>

<style scoped>
.form-view { display: flex; flex-direction: column; height: 100%; }
.form-content { flex: 1; padding: 24px 32px; display: flex; flex-direction: column; gap: 8px; max-width: 700px; }
h2 { font-size: 24px; font-weight: 700; margin-bottom: 8px; }
.field-label { font-size: 14px; font-weight: 500; margin-top: 8px; }
.field-hint { font-size: 12px; color: var(--color-text-muted); margin-top: -4px; }
.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
