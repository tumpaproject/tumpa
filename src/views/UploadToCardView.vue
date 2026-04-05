<script setup>
import { ref, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TButton from '@/components/TButton.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'

const router = useRouter()
const route = useRoute()

const fingerprint = ref('')
const password = ref('')
const loading = ref(true)

// Availability from backend
const primaryCanSign = ref(false)
const signingSubkeyAvailable = ref(false)
const encryptionAvailable = ref(false)
const authenticationAvailable = ref(false)

// User selections
const uploadPrimary = ref(false)
const uploadSigningSubkey = ref(false)
const uploadEncryption = ref(false)
const uploadAuthentication = ref(false)

onMounted(async () => {
  fingerprint.value = route.query.fingerprint || ''
  if (!fingerprint.value) {
    alert('No key selected.')
    router.back()
    return
  }
  try {
    const avail = await invoke('get_available_subkeys', { fingerprint: fingerprint.value })
    primaryCanSign.value = avail.primary_can_sign
    signingSubkeyAvailable.value = avail.signing_subkey
    encryptionAvailable.value = avail.encryption
    authenticationAvailable.value = avail.authentication

    // Default selections
    uploadEncryption.value = avail.encryption
    uploadAuthentication.value = avail.authentication

    // If signing subkey exists, preselect it and deselect primary
    if (avail.signing_subkey) {
      uploadSigningSubkey.value = true
      uploadPrimary.value = false
    } else if (avail.primary_can_sign) {
      uploadPrimary.value = true
      uploadSigningSubkey.value = false
    }
  } catch (e) {
    alert(String(e))
  }
  loading.value = false
})

// Primary and signing subkey are mutually exclusive for the signing slot
watch(uploadPrimary, (val) => {
  if (val) uploadSigningSubkey.value = false
})
watch(uploadSigningSubkey, (val) => {
  if (val) uploadPrimary.value = false
})

async function upload() {
  if (!password.value) {
    alert('Please enter the key password.')
    return
  }

  let whichSubkeys = 0
  if (uploadEncryption.value) whichSubkeys += 1
  if (uploadPrimary.value) whichSubkeys += 2
  if (uploadAuthentication.value) whichSubkeys += 4
  if (uploadSigningSubkey.value) whichSubkeys += 8

  if (whichSubkeys === 0) {
    alert('Please select at least one key to upload.')
    return
  }

  if (!confirm('This will reset the card and upload the selected keys. Any existing keys on the card will be deleted. Continue?')) {
    return
  }

  router.push({
    name: 'uploading',
    query: {
      fingerprint: fingerprint.value,
      password: password.value,
      whichSubkeys: String(whichSubkeys),
    },
  })
}
</script>

<template>
  <div class="form-view">
    <div class="form-content" v-if="!loading">
      <h2>Upload key to card</h2>

      <p class="fp-display">{{ fingerprint }}</p>

      <label class="field-label">Key Password:</label>
      <input type="password" v-model="password" />

      <label class="field-label">Signing slot:</label>
      <div class="checkbox-group">
        <label :class="{ disabled: !primaryCanSign }">
          <input
            type="checkbox"
            v-model="uploadPrimary"
            :disabled="!primaryCanSign"
          />
          Primary key (signing)
        </label>
        <label :class="{ disabled: !signingSubkeyAvailable }">
          <input
            type="checkbox"
            v-model="uploadSigningSubkey"
            :disabled="!signingSubkeyAvailable"
          />
          Signing subkey
        </label>
      </div>

      <label class="field-label">Other subkeys:</label>
      <div class="checkbox-group">
        <label :class="{ disabled: !encryptionAvailable }">
          <input
            type="checkbox"
            v-model="uploadEncryption"
            :disabled="!encryptionAvailable"
          />
          Encryption subkey
        </label>
        <label :class="{ disabled: !authenticationAvailable }">
          <input
            type="checkbox"
            v-model="uploadAuthentication"
            :disabled="!authenticationAvailable"
          />
          Authentication subkey
        </label>
      </div>
    </div>

    <div class="form-footer">
      <TButton variant="default" :icon="backIconSvg" @click="router.back()">Back</TButton>
      <TButton variant="green" :icon="tickSvg" @click="upload" :disabled="loading">Upload</TButton>
    </div>
  </div>
</template>

<style scoped>
.form-view { display: flex; flex-direction: column; height: 100%; }
.form-content { flex: 1; padding: 24px 32px; display: flex; flex-direction: column; gap: 8px; max-width: 700px; }
h2 { font-size: 24px; font-weight: 700; margin-bottom: 8px; }
.fp-display { font-size: 13px; font-weight: 600; color: var(--color-text-muted); font-family: monospace; margin-bottom: 8px; }
.field-label { font-size: 14px; font-weight: 500; margin-top: 12px; }
.checkbox-group { display: flex; flex-direction: column; gap: 8px; }
.checkbox-group label { display: flex; align-items: center; gap: 8px; font-size: 14px; cursor: pointer; }
.checkbox-group label.disabled { opacity: 0.4; cursor: not-allowed; }
.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
