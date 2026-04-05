<script setup>
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import revokeSvg from '@/assets/icons/revoke.svg'
import deleteSvg from '@/assets/icons/delete_purple.svg'

const props = defineProps({
  fingerprint: String,
  uidIndex: String,
})
const router = useRouter()
const store = useAppStore()

const keyData = ref(null)
const password = ref('')

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

async function revokeUid() {
  const pw = prompt('Enter key password to revoke this User ID:')
  if (!pw) return
  try {
    await invoke('revoke_user_id', {
      fingerprint: props.fingerprint,
      uid: uidString.value,
      password: pw,
    })
    await store.refreshKeys()
    router.back()
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div class="details-view" v-if="uid">
    <div class="toolbar">
      <TButton variant="red-alt" :icon="revokeSvg" thin @click="revokeUid">Revoke User ID</TButton>
    </div>

    <div class="details-content">
      <h2>User details</h2>

      <div class="info-table">
        <div class="info-row">
          <span class="info-label">Status</span>
          <span class="info-value" :class="{ revoked: uid.revoked }">{{ uid.revoked ? 'Revoked' : 'Valid' }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Name</span>
          <span class="info-value">{{ uid.name }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Email</span>
          <span class="info-value">{{ uid.email }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Created</span>
          <span class="info-value">{{ keyData.creation_time }}</span>
        </div>
        <div class="info-row">
          <span class="info-label">Key ID</span>
          <span class="info-value">{{ keyData.key_id }}</span>
        </div>
      </div>
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

.info-table { display: flex; flex-direction: column; gap: 16px; max-width: 500px; }

.info-row { display: flex; gap: 16px; }
.info-label { min-width: 100px; text-align: right; color: var(--color-text-muted); font-size: 14px; }
.info-value { font-size: 14px; font-weight: 500; }
.info-value.revoked { color: var(--color-red); }

.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
