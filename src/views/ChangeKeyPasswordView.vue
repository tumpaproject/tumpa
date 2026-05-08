<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TButton from '@/components/TButton.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'

const props = defineProps({ fingerprint: String })
const router = useRouter()

const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')

async function save() {
  if (!oldPassword.value) {
    alert('Please enter the current password.')
    return
  }
  if (!newPassword.value) {
    alert('Please enter a new password.')
    return
  }
  if (newPassword.value !== confirmPassword.value) {
    alert('New passwords do not match.')
    return
  }
  try {
    await invoke('change_key_password', {
      fingerprint: props.fingerprint,
      oldPassword: oldPassword.value,
      newPassword: newPassword.value,
    })
    alert('Password changed successfully.')
    router.back()
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div class="form-view">
    <div class="form-content">
      <h1>Change Key Password</h1>

      <p class="fp-display">{{ fingerprint }}</p>

      <label class="field-label" for="ckp-old">Current Password:</label>
      <PasswordInput id="ckp-old" v-model="oldPassword" />

      <label class="field-label" for="ckp-new">New Password:</label>
      <PasswordInput id="ckp-new" v-model="newPassword" aria-describedby="ckp-new-hint" />
      <span id="ckp-new-hint" class="field-hint">Recommended: 10+ chars in length</span>

      <label class="field-label" for="ckp-confirm">Confirm New Password:</label>
      <PasswordInput id="ckp-confirm" v-model="confirmPassword" />
    </div>

    <div class="form-footer">
      <TButton variant="default" :icon="backIconSvg" @click="router.back()">Back</TButton>
      <TButton variant="green" :icon="tickSvg" @click="save">Save</TButton>
    </div>
  </div>
</template>

<style scoped>
.form-view { display: flex; flex-direction: column; height: 100%; }
.form-content { flex: 1; padding: 24px 32px; display: flex; flex-direction: column; gap: 8px; max-width: 700px; }
h1 { font-size: 24px; font-weight: 700; margin-bottom: 8px; }
.fp-display { font-size: 13px; font-weight: 600; color: var(--color-text-muted); font-family: monospace; margin-bottom: 8px; }
.field-label { font-size: 14px; font-weight: 500; margin-top: 8px; }
.field-hint { font-size: 12px; color: var(--color-text-muted); margin-top: -4px; }
.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
