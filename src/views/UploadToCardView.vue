<script setup>
import { ref, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TButton from '@/components/TButton.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import WaitSpinner from '@/components/WaitSpinner.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'

const router = useRouter()
const route = useRoute()

const fingerprint = ref('')
const password = ref('')
const loading = ref(true)
const uploading = ref(false)
const errorMessage = ref('')

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
    router.back()
    return
  }
  // Restore error from a failed upload attempt
  if (route.query.error) {
    errorMessage.value = route.query.error
    password.value = route.query.savedPassword || ''
  }
  try {
    const avail = await invoke('get_available_subkeys', { fingerprint: fingerprint.value })
    primaryCanSign.value = avail.primary_can_sign
    signingSubkeyAvailable.value = avail.signing_subkey
    encryptionAvailable.value = avail.encryption
    authenticationAvailable.value = avail.authentication

    uploadEncryption.value = avail.encryption
    uploadAuthentication.value = avail.authentication

    if (avail.signing_subkey) {
      uploadSigningSubkey.value = true
      uploadPrimary.value = false
    } else if (avail.primary_can_sign) {
      uploadPrimary.value = true
      uploadSigningSubkey.value = false
    }
  } catch (e) {
    errorMessage.value = String(e)
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

// Clear error when password changes
watch(password, () => {
  errorMessage.value = ''
})

async function upload() {
  if (!password.value) {
    errorMessage.value = 'Please enter the key password.'
    return
  }

  let whichSubkeys = 0
  if (uploadEncryption.value) whichSubkeys += 1
  if (uploadPrimary.value) whichSubkeys += 2
  if (uploadAuthentication.value) whichSubkeys += 4
  if (uploadSigningSubkey.value) whichSubkeys += 8

  if (whichSubkeys === 0) {
    errorMessage.value = 'Please select at least one key to upload.'
    return
  }

  if (!confirm('This will reset the card and upload the selected keys. Any existing keys on the card will be deleted. Continue?')) {
    return
  }

  errorMessage.value = ''
  uploading.value = true

  try {
    await invoke('upload_key_to_card', {
      fingerprint: fingerprint.value,
      password: password.value,
      whichSubkeys,
    })
    router.replace('/keys')
  } catch (e) {
    uploading.value = false
    errorMessage.value = String(e)
  }
}
</script>

<template>
  <WaitSpinner
    v-if="uploading"
    message="Uploading to smartcard, please wait!"
    hint="Do not remove the card during this operation."
  />
  <div v-else class="form-view">
    <div class="form-content" v-if="!loading">
      <h2>Upload key to card</h2>

      <p class="fp-display">{{ fingerprint }}</p>

      <div class="password-row">
        <label class="field-label">Key Password:</label>
        <span v-if="errorMessage" class="error-text">{{ errorMessage }}</span>
      </div>
      <PasswordInput v-model="password" />

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
.password-row { display: flex; align-items: baseline; gap: 12px; margin-top: 12px; }
.password-row .field-label { margin-top: 0; }
.error-text { color: var(--color-red); font-size: 13px; font-weight: 500; }
.checkbox-group { display: flex; flex-direction: column; gap: 8px; }
.checkbox-group label { display: flex; align-items: center; gap: 8px; font-size: 14px; cursor: pointer; }
.checkbox-group label.disabled { opacity: 0.4; cursor: not-allowed; }
.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
