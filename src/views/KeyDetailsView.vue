<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import DatePicker from '@/components/DatePicker.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'
import cardPurpleSvg from '@/assets/icons/card_purple.svg'
import exportSvg from '@/assets/icons/export_purple.svg'
import deleteSvg from '@/assets/icons/delete_purple.svg'
import revokeSvg from '@/assets/icons/revoke.svg'
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
const selectedSubkeys = ref({})
const showSubkeyExpiryEdit = ref(false)
const subkeyExpiryDate = ref('')
const subkeyExpiryPassword = ref('')
const showRevokeKey = ref(false)
const revokeKeyPassword = ref('')
const showAdvanced = ref(false)
const keyserverUploading = ref(false)
const keyserverResult = ref(null)
const keyserverError = ref('')

const anySubkeySelected = computed(() => {
  return Object.values(selectedSubkeys.value).some(v => v)
})

const selectedSubkeyFingerprints = computed(() => {
  return Object.entries(selectedSubkeys.value)
    .filter(([_, v]) => v)
    .map(([fp]) => fp)
})

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

async function updateSubkeysExpiry() {
  if (!subkeyExpiryDate.value || !subkeyExpiryPassword.value) {
    alert('Please enter both date and password.')
    return
  }
  try {
    keyData.value = await invoke('update_selected_subkeys_expiry', {
      fingerprint: props.fingerprint,
      subkeyFingerprints: selectedSubkeyFingerprints.value,
      newDate: subkeyExpiryDate.value,
      password: subkeyExpiryPassword.value,
    })
    showSubkeyExpiryEdit.value = false
    subkeyExpiryPassword.value = ''
    subkeyExpiryDate.value = ''
    selectedSubkeys.value = {}
    await store.refreshKeys()
  } catch (e) {
    alert(String(e))
  }
}

async function revokeKey() {
  if (!revokeKeyPassword.value) {
    alert('Please enter the key password.')
    return
  }
  try {
    keyData.value = await invoke('revoke_key_cmd', {
      fingerprint: props.fingerprint,
      password: revokeKeyPassword.value,
    })
    showRevokeKey.value = false
    revokeKeyPassword.value = ''
    await store.refreshKeys()
  } catch (e) {
    alert(String(e))
  }
}

async function syncToKeyserver() {
  keyserverUploading.value = true
  keyserverError.value = ''
  keyserverResult.value = null
  try {
    keyserverResult.value = await invoke('upload_to_keyserver', {
      fingerprint: props.fingerprint,
    })
  } catch (e) {
    keyserverError.value = String(e)
  }
  keyserverUploading.value = false
}

async function requestVerification(email) {
  try {
    await invoke('request_keyserver_verification', {
      token: keyserverResult.value.token,
      email,
    })
    // Update the status in the result
    const entry = keyserverResult.value.email_status.find(e => e.email === email)
    if (entry) entry.status = 'pending'
  } catch (e) {
    keyserverError.value = String(e)
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
      <TButton variant="green" :icon="cardPurpleSvg" thin @click="uploadToCard" :disabled="keyData.is_revoked">Send Key to Card</TButton>
      <TButton variant="white" :icon="exportSvg" thin @click="exportKey">Export Public Key</TButton>
      <TButton variant="red-alt" :icon="revokeSvg" thin @click="showRevokeKey = true" :disabled="keyData.is_revoked">Revoke Key</TButton>
      <TButton variant="white" :icon="deleteSvg" thin @click="deleteKey">Remove</TButton>
    </div>

    <div class="details-content">
      <!-- Revoke Key -->
      <div v-if="showRevokeKey && !keyData.is_revoked" class="revoke-key-form">
        <p class="revoke-warning">This will permanently revoke this key. This action cannot be undone.</p>
        <label class="field-label">Enter key password to confirm revocation:</label>
        <div class="revoke-key-row">
          <PasswordInput v-model="revokeKeyPassword" placeholder="Key password" />
          <TButton variant="red" thin @click="revokeKey">Revoke Key</TButton>
          <TButton variant="default" thin @click="showRevokeKey = false">Cancel</TButton>
        </div>
      </div>

      <div v-if="keyData.is_revoked" class="revoked-banner">
        <span>This key has been revoked{{ keyData.revocation_time ? ` on ${keyData.revocation_time}` : '' }}.</span>
      </div>

      <!-- Keyserver sync result -->
      <div v-if="keyserverUploading" class="keyserver-status">
        Uploading to keys.openpgp.org...
      </div>
      <div v-if="keyserverError" class="keyserver-status keyserver-error">
        {{ keyserverError }}
      </div>
      <div v-if="keyserverResult" class="keyserver-status keyserver-success">
        <p>Key uploaded to keys.openpgp.org</p>
        <div v-for="es in keyserverResult.email_status" :key="es.email" class="email-verify-row">
          <span class="email-addr">{{ es.email }}</span>
          <span class="email-status" :class="es.status">{{ es.status }}</span>
          <TButton
            v-if="es.status === 'unpublished'"
            variant="green"
            thin
            @click="requestVerification(es.email)"
          >Verify</TButton>
          <span v-if="es.status === 'pending'" class="email-hint">Check your inbox</span>
        </div>
      </div>

      <!-- User IDs -->
      <div class="section">
        <div class="section-header">
          <h3>User ID</h3>
          <TButton variant="white" thin @click="router.push(`/keys/${fingerprint}/add-uid`)" :disabled="keyData.is_revoked">Add new user</TButton>
        </div>
        <table class="uid-table">
          <thead>
            <tr><th>Name</th><th>Email</th><th>Status</th><th></th></tr>
          </thead>
          <tbody>
            <tr
              v-for="(uid, i) in keyData.user_ids"
              :key="i"
              class="uid-row"
              :class="{ 'uid-revoked': uid.revoked }"
              @click="router.push(`/keys/${fingerprint}/uid/${i}`)"
            >
              <td>{{ uid.name }}</td>
              <td>{{ uid.email }}</td>
              <td><span v-if="uid.revoked" class="revoked-badge">Revoked</span><span v-else class="valid-badge">Valid</span></td>
              <td class="uid-arrow">&rsaquo;</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Primary Key -->
      <div class="section">
        <h3>Primary key</h3>
        <div class="accordion" :class="{ expanded: primaryExpanded }">
          <div class="accordion-header">
            <button class="accordion-header-btn" @click="primaryExpanded = !primaryExpanded">
              <img :src="keyIconSvg" alt="" class="acc-icon" />
              <span class="acc-fp">{{ keyData.fingerprint }}</span>
              <img :src="downIconSvg" alt="" class="acc-chevron" :class="{ rotated: primaryExpanded }" />
            </button>
          </div>
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
                <button class="expiry-change-btn" @click="showExpiryEdit = true" :disabled="keyData.is_revoked">Change</button>
              </div>
              <div class="expiry-edit" v-else>
                <DatePicker v-model="newExpiryDate" :min-date="new Date().toISOString().split('T')[0]" />
                <PasswordInput v-model="expiryPassword" placeholder="Key password" />
                <TButton variant="green" thin @click="updateExpiry">Save</TButton>
                <TButton variant="default" thin @click="showExpiryEdit = false">Cancel</TButton>
              </div>
            </div>

            <button class="advanced-toggle" @click="showAdvanced = !showAdvanced">
              Advanced {{ showAdvanced ? '\u2303' : '\u2304' }}
            </button>

            <div v-if="showAdvanced" class="advanced-section">
              <TButton variant="white" thin @click="syncToKeyserver" :disabled="keyserverUploading">Sync to Keyserver</TButton>
              <TButton variant="white" thin @click="router.push(`/keys/${fingerprint}/change-password`)" :disabled="keyData.is_revoked">Change Password</TButton>
            </div>
          </div>
        </div>
      </div>

      <!-- Subkeys -->
      <div class="section">
        <div class="section-header">
          <h3>Subkeys</h3>
          <TButton
            variant="white"
            thin
            :disabled="!anySubkeySelected || keyData.is_revoked"
            @click="showSubkeyExpiryEdit = true"
          >Change expiry</TButton>
        </div>

        <div v-if="showSubkeyExpiryEdit && anySubkeySelected" class="subkey-expiry-edit">
          <label class="field-label">New expiry date for selected subkeys:</label>
          <div class="subkey-expiry-row">
            <DatePicker v-model="subkeyExpiryDate" :min-date="new Date().toISOString().split('T')[0]" />
            <PasswordInput v-model="subkeyExpiryPassword" placeholder="Key password" />
            <TButton variant="green" thin @click="updateSubkeysExpiry">Update</TButton>
            <TButton variant="default" thin @click="showSubkeyExpiryEdit = false">Cancel</TButton>
          </div>
        </div>

        <div
          v-for="sk in nonPrimarySubkeys()"
          :key="sk.fingerprint"
          class="accordion"
        >
          <div class="accordion-header">
            <input
              type="checkbox"
              :checked="selectedSubkeys[sk.fingerprint]"
              @change="selectedSubkeys[sk.fingerprint] = $event.target.checked"
              @click.stop
              class="subkey-checkbox"
            />
            <button class="accordion-header-btn" @click="toggleSubkey(sk.fingerprint)">
              <img :src="keyIconSvg" alt="" class="acc-icon" />
              <span class="acc-fp">{{ sk.fingerprint }}</span>
              <img :src="downIconSvg" alt="" class="acc-chevron" :class="{ rotated: expandedSubkeys[sk.fingerprint] }" />
            </button>
          </div>
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

.revoke-key-form { margin-bottom: 20px; padding: 16px; border: 1px solid var(--color-expired-border); border-radius: 6px; background: var(--color-expired-bg); display: flex; flex-direction: column; gap: 8px; }
.revoke-warning { font-size: 14px; font-weight: 500; color: var(--color-red); }
.revoke-key-form .field-label { font-size: 13px; color: var(--color-text-muted); }
.revoke-key-row { display: flex; align-items: center; gap: 8px; }
.revoke-key-row .password-input { max-width: 300px; }

.keyserver-status { margin-bottom: 16px; padding: 12px 16px; border-radius: 6px; font-size: 14px; }
.keyserver-error { background: var(--color-expired-bg); border: 1px solid var(--color-expired-border); color: var(--color-red); }
.keyserver-success { background: #f0fdf4; border: 1px solid #86efac; }
.keyserver-success p { font-weight: 500; margin-bottom: 8px; }
.email-verify-row { display: flex; align-items: center; gap: 12px; padding: 4px 0; }
.email-addr { font-size: 13px; min-width: 200px; }
.email-status { font-size: 12px; font-weight: 500; padding: 2px 8px; border-radius: 3px; }
.email-status.published { background: #dcfce7; color: #16a34a; }
.email-status.unpublished { background: var(--color-border); color: var(--color-text-muted); }
.email-status.pending { background: #fef9c3; color: #a16207; }
.email-status.revoked { background: var(--color-expired-bg); color: var(--color-red); }
.email-hint { font-size: 12px; color: var(--color-text-muted); font-style: italic; }

.revoked-banner { margin-bottom: 20px; padding: 12px 16px; border: 1px solid var(--color-expired-border); border-radius: 6px; background: var(--color-expired-bg); color: var(--color-red); font-size: 14px; font-weight: 500; }

.section { margin-bottom: 24px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
h3 { font-size: 18px; font-weight: 700; margin-bottom: 12px; }
.section-header h3 { margin-bottom: 0; }

.uid-table { width: 100%; border-collapse: collapse; }
.uid-table th { text-align: left; padding: 8px 12px; font-size: 13px; font-weight: 500; color: var(--color-text-muted); background: var(--color-bg-light); border-bottom: 1px solid var(--color-border); }
.uid-table td { padding: 10px 12px; font-size: 14px; border-bottom: 1px solid var(--color-border); }
.uid-row { cursor: pointer; }
.uid-row:hover { background: var(--color-bg-light); }
.uid-revoked { opacity: 0.6; }
.uid-arrow { text-align: right; font-size: 20px; color: var(--color-text-muted); }
.revoked-badge { color: var(--color-red); font-size: 12px; font-weight: 500; }
.valid-badge { color: #16a34a; font-size: 12px; font-weight: 500; }

.subkey-expiry-edit { margin-bottom: 16px; display: flex; flex-direction: column; gap: 8px; }
.subkey-expiry-edit .field-label { font-size: 13px; color: var(--color-text-muted); }
.subkey-expiry-row { display: flex; align-items: center; gap: 8px; }
.subkey-expiry-row .date-picker { width: 160px; flex-shrink: 0; }
.subkey-expiry-row .password-input { width: auto; min-width: 220px; flex: 1; }

.accordion { border: 1px solid var(--color-border); border-radius: 6px; margin-bottom: 8px; overflow: hidden; }
.accordion-header { display: flex; align-items: center; gap: 8px; background: var(--color-bg-light); padding: 0 12px; }
.subkey-checkbox { width: 16px; height: 16px; cursor: pointer; flex-shrink: 0; }
.accordion-header-btn { display: flex; align-items: center; gap: 12px; flex: 1; padding: 12px 0; background: none; border: none; cursor: pointer; font-family: var(--font-family); font-size: 14px; text-align: left; }
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

.advanced-toggle { align-self: flex-start; margin-top: 8px; padding: 6px 12px; background: var(--color-border); border: none; border-radius: 4px; font-size: 14px; font-weight: 500; cursor: pointer; font-family: var(--font-family); }
.advanced-toggle:hover { background: #DADDE2; }
.advanced-section { display: flex; gap: 12px; margin-top: 12px; }

.form-footer { display: flex; justify-content: space-between; padding: 16px 24px; border-top: 1px solid var(--color-border); }
</style>
