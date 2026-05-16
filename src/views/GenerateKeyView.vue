<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import PasswordInput from '@/components/PasswordInput.vue'
import DatePicker from '@/components/DatePicker.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'

const router = useRouter()
const store = useAppStore()

const name = ref('')
const emails = ref('')
const passphrase = ref('')
const expiryDate = ref('')
const showAdvanced = ref(false)
const encryption = ref(true)
const signing = ref(false)
const authentication = ref(true)
const keyAlgo = ref('curve25519')

function goBack() {
  router.back()
}

function submit() {
  if (!name.value.trim()) {
    alert('Please enter your name.')
    return
  }
  if (!emails.value.trim()) {
    alert('Please enter at least one email address.')
    return
  }
  if (!passphrase.value) {
    alert('Please enter a passphrase.')
    return
  }

  const emailList = emails.value
    .split('\n')
    .map(e => e.trim())
    .filter(e => e.length > 0)

  // Store params in Pinia (not URL) to avoid secrets in browser history
  store.setPendingKeyGen({
    name: name.value.trim(),
    emails: emailList,
    password: passphrase.value,
    expiryDate: expiryDate.value || null,
    encryption: encryption.value,
    signing: signing.value,
    authentication: authentication.value,
    cipher: keyAlgo.value,
  })

  router.push({ name: 'generating' })
}
</script>

<template>
  <div class="generate-view">
    <div class="form-content">
      <h1>Generate new key</h1>

      <label class="field-label" for="gen-name">Your Name:</label>
      <input id="gen-name" type="text" v-model="name" />

      <label class="field-label" for="gen-emails">Email addresses:</label>
      <textarea id="gen-emails" v-model="emails" placeholder="One email per line" rows="3"></textarea>

      <label class="field-label" for="gen-passphrase">Key Passphrase:</label>
      <PasswordInput id="gen-passphrase" v-model="passphrase" aria-describedby="gen-passphrase-hint" />
      <span id="gen-passphrase-hint" class="field-hint">Recommended: 10+ chars in length</span>

      <label class="field-label" for="gen-expiry">Expiration date:</label>
      <DatePicker id="gen-expiry" v-model="expiryDate" :min-date="new Date().toISOString().split('T')[0]" />

      <button class="advanced-toggle" :aria-expanded="showAdvanced" @click="showAdvanced = !showAdvanced">
        Advanced {{ showAdvanced ? '\u2303' : '\u2304' }}
      </button>

      <template v-if="showAdvanced">
        <fieldset class="checkbox-group">
          <legend class="field-label">Key capabilities</legend>
          <label><input type="checkbox" v-model="encryption" /> Encryption subkey</label>
          <label><input type="checkbox" v-model="signing" /> Signing subkey</label>
          <label><input type="checkbox" v-model="authentication" /> Authentication subkey</label>
        </fieldset>

        <label class="field-label" for="gen-algo">Key algorithm:</label>
        <select id="gen-algo" v-model="keyAlgo">
          <option value="curve25519">Curve25519 (Legacy)</option>
          <option value="cv25519modern">Curve25519 (Modern, Nitrokey 3)</option>
          <option value="rsa4096">RSA4096</option>
        </select>
      </template>
    </div>

    <div class="form-footer">
      <TButton variant="default" :icon="backIconSvg" @click="goBack">Back</TButton>
      <TButton variant="green" :icon="tickSvg" @click="submit">Confirm</TButton>
    </div>
  </div>
</template>

<style scoped>
.generate-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.form-content {
  flex: 1;
  padding: 24px 32px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 700px;
}

h1 {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 8px;
}

.field-label {
  font-size: 14px;
  font-weight: 500;
  margin-top: 8px;
}

.field-hint {
  font-size: 12px;
  color: var(--color-text-muted);
  margin-top: -4px;
}

.advanced-toggle {
  align-self: flex-start;
  margin-top: 8px;
  padding: 6px 12px;
  background: var(--color-border);
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  font-family: var(--font-family);
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.checkbox-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  cursor: pointer;
}

.form-footer {
  display: flex;
  justify-content: space-between;
  padding: 16px 32px;
  border-top: 1px solid var(--color-border);
}
</style>
