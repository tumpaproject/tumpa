<script setup>
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import TButton from '@/components/TButton.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'

const router = useRouter()
const route = useRoute()

const fingerprint = ref('')
const password = ref('')
const encryption = ref(false)
const signing = ref(false)
const authentication = ref(false)
const loading = ref(true)

onMounted(async () => {
  fingerprint.value = route.query.fingerprint || ''
  if (!fingerprint.value) {
    alert('No key selected.')
    router.back()
    return
  }
  try {
    const avail = await invoke('get_available_subkeys', { fingerprint: fingerprint.value })
    encryption.value = avail.encryption
    signing.value = avail.signing
    authentication.value = avail.authentication
  } catch (e) {
    alert(String(e))
  }
  loading.value = false
})

async function upload() {
  if (!password.value) {
    alert('Please enter the key password.')
    return
  }

  let whichSubkeys = 0
  if (encryption.value) whichSubkeys += 1
  if (signing.value) whichSubkeys += 2
  if (authentication.value) whichSubkeys += 4

  if (whichSubkeys === 0) {
    alert('Please select at least one subkey type.')
    return
  }

  if (!confirm('This will reset the card and upload the selected subkeys. Any existing keys on the card will be deleted. Continue?')) {
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

      <label class="field-label">Subkeys to upload:</label>
      <div class="checkbox-group">
        <label><input type="checkbox" v-model="encryption" /> Encryption subkey</label>
        <label><input type="checkbox" v-model="signing" /> Signing subkey</label>
        <label><input type="checkbox" v-model="authentication" /> Authentication subkey</label>
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
.field-label { font-size: 14px; font-weight: 500; margin-top: 8px; }
.checkbox-group { display: flex; flex-direction: column; gap: 8px; }
.checkbox-group label { display: flex; align-items: center; gap: 8px; font-size: 14px; cursor: pointer; }
.form-footer { display: flex; justify-content: space-between; padding: 16px 32px; border-top: 1px solid var(--color-border); }
</style>
