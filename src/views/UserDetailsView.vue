<script setup>
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import revokeSvg from '@/assets/icons/revoke.svg'

const props = defineProps({
  fingerprint: String,
  uidIndex: String,
})
const router = useRouter()
const store = useAppStore()

const keyData = ref(null)
const showRevokeForm = ref(false)
const revokePassword = ref('')

const uid = computed(() => {
  if (!keyData.value) return null
  const idx = parseInt(props.uidIndex)
  return keyData.value.user_ids[idx] || null
})

const uidString = computed(() => {
  if (!uid.value) return ''
  return uid.value.email
    ? `${uid.value.name} <${uid.value.email}>`
    : uid.value.name
})

onMounted(async () => {
  try {
    keyData.value = await invoke('get_key_details', { fingerprint: props.fingerprint })
  } catch (e) {
    alert(String(e))
    router.back()
  }
})

function startRevoke() {
  showRevokeForm.value = true
  revokePassword.value = ''
}

async function confirmRevoke() {
  if (!revokePassword.value) {
    alert('Please enter the key password.')
    return
  }
  try {
    keyData.value = await invoke('revoke_user_id', {
      fingerprint: props.fingerprint,
      uid: uidString.value,
      password: revokePassword.value,
    })
    showRevokeForm.value = false
    revokePassword.value = ''
    await store.refreshKeys()
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div class="details-view" v-if="uid">
    <div class="toolbar">
      <TButton variant="red-alt" :icon="revokeSvg" thin @click="startRevoke" :disabled="uid.revoked || keyData.is_revoked">Revoke User ID</TButton>
    </div>

    <div class="details-content">
      <h2>User details</h2>

      <div v-if="showRevokeForm && !uid.revoked" class="revoke-form">
        <label class="field-label" for="revoke-uid-password">Enter key password to revoke this User ID:</label>
        <div class="revoke-row">
          <PasswordInput id="revoke-uid-password" v-model="revokePassword" placeholder="Key password" />
          <TButton variant="red" thin @click="confirmRevoke">Revoke</TButton>
          <TButton variant="default" thin @click="showRevokeForm = false">Cancel</TButton>
        </div>
      </div>

      <dl class="info-table">
        <div class="info-row">
          <dt class="info-label">Status</dt>
          <dd class="info-value" :class="{ revoked: uid.revoked }">{{ uid.revoked ? 'Revoked' : 'Valid' }}</dd>
        </div>
        <div class="info-row" v-if="uid.revoked && uid.revocation_time">
          <dt class="info-label">Revoked on</dt>
          <dd class="info-value revoked">{{ uid.revocation_time }}</dd>
        </div>
        <div class="info-row">
          <dt class="info-label">Name</dt>
          <dd class="info-value">{{ uid.name }}</dd>
        </div>
        <div class="info-row">
          <dt class="info-label">Email</dt>
          <dd class="info-value">{{ uid.email }}</dd>
        </div>
        <div class="info-row">
          <dt class="info-label">Created</dt>
          <dd class="info-value">{{ keyData.creation_time }}</dd>
        </div>
        <div class="info-row">
          <dt class="info-label">Key ID</dt>
          <dd class="info-value">{{ keyData.key_id }}</dd>
        </div>
      </dl>
    </div>

    <div class="form-footer">
      <TButton variant="default" :icon="backIconSvg" @click="router.back()">Back</TButton>
      <div></div>
    </div>
  </div>
</template>

<style scoped>
.details-view { display: flex; flex-direction: column; height: 100%; }

.toolbar {
  display: flex;
  gap: 12px;
  padding: 12px 24px;
  background: var(--color-bg-light);
  border-bottom: 1px solid var(--color-border);
}

.details-content { flex: 1; padding: 24px 32px; overflow-y: auto; }

h2 { font-size: 24px; font-weight: 700; margin-bottom: 24px; }

.revoke-form { margin-bottom: 24px; display: flex; flex-direction: column; gap: 8px; }
.revoke-form .field-label { font-size: 13px; color: var(--color-text-muted); }
.revoke-row { display: flex; align-items: center; gap: 8px; }
.revoke-row .password-input { max-width: 300px; }

.info-table { display: flex; flex-direction: column; gap: 16px; max-width: 500px; }

.info-row { display: flex; gap: 16px; }
.info-label { min-width: 100px; text-align: right; color: var(--color-text-muted); font-size: 14px; }
.info-value { font-size: 14px; font-weight: 500; }
.info-value.revoked { color: var(--color-red); }

.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
