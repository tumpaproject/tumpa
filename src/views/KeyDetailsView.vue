<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'
import cardPurpleSvg from '@/assets/icons/card_purple.svg'
import exportSvg from '@/assets/icons/export_purple.svg'
import deleteSvg from '@/assets/icons/delete_purple.svg'
import keyIconSvg from '@/assets/icons/keyIcon.svg'
import downIconSvg from '@/assets/icons/down_icon.svg'

const props = defineProps({ fingerprint: String })
const router = useRouter()
const store = useAppStore()

const keyData = ref(null)
const primaryExpanded = ref(true)
const expandedSubkeys = ref({})
const newExpiryDate = ref('')
const expiryPassword = ref('')
const showExpiryEdit = ref(false)

onMounted(async () => {
  await loadKey()
})

async function loadKey() {
  try {
    keyData.value = await invoke('get_key_details', { fingerprint: props.fingerprint })
  } catch (e) {
    alert(String(e))
    router.back()
  }
}

function toggleSubkey(fp) {
  expandedSubkeys.value[fp] = !expandedSubkeys.value[fp]
}

async function uploadToCard() {
  const connected = await store.checkCard()
  if (!connected) {
    alert('No smartcard connected.')
    return
  }
  router.push({ name: 'upload-to-card', query: { fingerprint: props.fingerprint } })
}

async function exportKey() {
  const path = await save({
    title: 'Export Public Key',
    defaultPath: `${props.fingerprint}.pub`,
  })
  if (!path) return
  try {
    await invoke('export_public_key', { fingerprint: props.fingerprint, filePath: path })
  } catch (e) {
    alert(String(e))
  }
}

async function deleteKey() {
  if (!confirm('Are you sure you want to delete the selected key?')) return
  try {
    await invoke('delete_key', { fingerprint: props.fingerprint })
    await store.refreshKeys()
    router.replace('/keys')
  } catch (e) {
    alert(String(e))
  }
}

async function updateExpiry() {
  if (!newExpiryDate.value || !expiryPassword.value) {
    alert('Please enter both date and password.')
    return
  }
  try {
    keyData.value = await invoke('update_key_expiry', {
      fingerprint: props.fingerprint,
      newDate: newExpiryDate.value,
      password: expiryPassword.value,
    })
    showExpiryEdit.value = false
    expiryPassword.value = ''
    await store.refreshKeys()
  } catch (e) {
    alert(String(e))
  }
}

const primaryKey = () => {
  if (!keyData.value) return null
  return keyData.value.subkeys.find(s => s.key_type === 'certification') || null
}

const nonPrimarySubkeys = () => {
  if (!keyData.value) return []
  return keyData.value.subkeys.filter(s => s.key_type !== 'certification')
}
</script>

<template>
  <div class="details-view" v-if="keyData">
    <div class="toolbar">
      <TButton variant="green" :icon="cardPurpleSvg" thin @click="uploadToCard">Send Key to Card</TButton>
      <TButton variant="white" :icon="exportSvg" thin @click="exportKey">Export Public Key</TButton>
      <TButton variant="white" :icon="deleteSvg" thin @click="deleteKey">Remove</TButton>
    </div>

    <div class="details-content">
      <!-- User IDs -->
      <div class="section">
        <div class="section-header">
          <h3>User ID</h3>
          <TButton variant="white" thin @click="router.push(`/keys/${fingerprint}/add-uid`)">Add new user</TButton>
        </div>
        <table class="uid-table">
          <thead>
            <tr><th>Name</th><th>Email</th><th></th></tr>
          </thead>
          <tbody>
            <tr
              v-for="(uid, i) in keyData.user_ids"
              :key="i"
              class="uid-row"
              @click="router.push(`/keys/${fingerprint}/uid/${i}`)"
            >
              <td>{{ uid.name }}</td>
              <td>{{ uid.email }}</td>
              <td class="uid-arrow">&rsaquo;</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Primary Key -->
      <div class="section">
        <h3>Primary key</h3>
        <div class="accordion" :class="{ expanded: primaryExpanded }">
          <button class="accordion-header" @click="primaryExpanded = !primaryExpanded">
            <img :src="keyIconSvg" alt="" class="acc-icon" />
            <span class="acc-fp">{{ keyData.fingerprint }}</span>
            <img :src="downIconSvg" alt="" class="acc-chevron" :class="{ rotated: primaryExpanded }" />
          </button>
          <div class="accordion-body" v-if="primaryExpanded">
            <div class="detail-row">
              <span class="detail-label">Status:</span>
              <span class="detail-value">{{ keyData.expiration_time === 'Never' || new Date(keyData.expiration_time) > new Date() ? 'Valid' : 'Expired' }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Created on:</span>
              <span class="detail-value">{{ keyData.creation_time }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Expiration date</span>
              <div class="expiry-widget" v-if="!showExpiryEdit">
                <span class="expiry-value">{{ keyData.expiration_time }}</span>
                <button class="expiry-change-btn" @click="showExpiryEdit = true">Change</button>
              </div>
              <div class="expiry-edit" v-else>
                <input type="date" v-model="newExpiryDate" @change="(e) => e.target.blur()" />
                <input type="password" v-model="expiryPassword" placeholder="Key password" />
                <TButton variant="green" thin @click="updateExpiry">Save</TButton>
                <TButton variant="default" thin @click="showExpiryEdit = false">Cancel</TButton>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Subkeys -->
      <div class="section">
        <h3>Subkeys</h3>
        <div
          v-for="sk in nonPrimarySubkeys()"
          :key="sk.fingerprint"
          class="accordion"
        >
          <button class="accordion-header" @click="toggleSubkey(sk.fingerprint)">
            <img :src="keyIconSvg" alt="" class="acc-icon" />
            <span class="acc-fp">{{ sk.fingerprint }}</span>
            <img :src="downIconSvg" alt="" class="acc-chevron" :class="{ rotated: expandedSubkeys[sk.fingerprint] }" />
          </button>
          <div class="accordion-body" v-if="expandedSubkeys[sk.fingerprint]">
            <div class="detail-row">
              <span class="detail-label">Type:</span>
              <span class="detail-value">{{ sk.key_type }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Created on:</span>
              <span class="detail-value">{{ sk.creation_time }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Expires on:</span>
              <span class="detail-value">{{ sk.expiration_time }}</span>
            </div>
            <div class="detail-row" v-if="sk.is_revoked">
              <span class="detail-label">Status:</span>
              <span class="detail-value" style="color: var(--color-red)">Revoked</span>
            </div>
          </div>
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

.details-content { flex: 1; padding: 24px; overflow-y: auto; }

.section { margin-bottom: 24px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
h3 { font-size: 18px; font-weight: 700; margin-bottom: 12px; }
.section-header h3 { margin-bottom: 0; }

.uid-table { width: 100%; border-collapse: collapse; }
.uid-table th { text-align: left; padding: 8px 12px; font-size: 13px; font-weight: 500; color: var(--color-text-muted); background: var(--color-bg-light); border-bottom: 1px solid var(--color-border); }
.uid-table td { padding: 10px 12px; font-size: 14px; border-bottom: 1px solid var(--color-border); }
.uid-row { cursor: pointer; }
.uid-row:hover { background: var(--color-bg-light); }
.uid-arrow { text-align: right; font-size: 20px; color: var(--color-text-muted); }

.accordion { border: 1px solid var(--color-border); border-radius: 6px; margin-bottom: 8px; overflow: hidden; }
.accordion-header { display: flex; align-items: center; gap: 12px; width: 100%; padding: 12px; background: var(--color-bg-light); border: none; cursor: pointer; font-family: var(--font-family); font-size: 14px; text-align: left; }
.acc-icon { width: 18px; height: 18px; }
.acc-fp { flex: 1; font-weight: 600; }
.acc-chevron { width: 16px; height: 16px; transition: transform 0.2s; }
.acc-chevron.rotated { transform: rotate(180deg); }
.accordion-body { padding: 12px 16px; display: flex; flex-direction: column; gap: 10px; }

.detail-row { display: flex; align-items: center; gap: 16px; }
.detail-label { min-width: 120px; font-size: 13px; color: var(--color-text-muted); }
.detail-value { font-size: 14px; font-weight: 500; }

.expiry-widget { display: flex; align-items: center; gap: 0; border: 1px solid var(--color-border); border-radius: 4px; overflow: hidden; }
.expiry-value { padding: 6px 12px; font-size: 14px; border-right: 1px solid var(--color-border); }
.expiry-change-btn { padding: 6px 12px; background: none; border: none; font-size: 14px; font-weight: 500; cursor: pointer; font-family: var(--font-family); }
.expiry-change-btn:hover { background: var(--color-bg-light); }

.expiry-edit { display: flex; align-items: center; gap: 8px; flex: 1; }
.expiry-edit input { width: auto; max-width: 180px; }

.form-footer { display: flex; justify-content: space-between; padding: 16px 24px; border-top: 1px solid var(--color-border); }
</style>
