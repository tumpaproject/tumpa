<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import TButton from '@/components/TButton.vue'
import DatePicker from '@/components/DatePicker.vue'
import backIconSvg from '@/assets/icons/backIcon.svg'
import tickSvg from '@/assets/icons/tick_mark.svg'

const router = useRouter()

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

  router.push({
    name: 'generating',
    query: {
      name: name.value.trim(),
      emails: JSON.stringify(emailList),
      password: passphrase.value,
      expiryDate: expiryDate.value || '',
      encryption: encryption.value,
      signing: signing.value,
      authentication: authentication.value,
      cipher: keyAlgo.value,
    },
  })
}
</script>

<template>
  <div class="generate-view">
    <div class="form-content">
      <h2>Generate new key</h2>

      <label class="field-label">Your Name:</label>
      <input type="text" v-model="name" />

      <label class="field-label">Email addresses:</label>
      <textarea v-model="emails" placeholder="One email per line" rows="3"></textarea>

      <label class="field-label">Key Passphrase:</label>
      <input type="password" v-model="passphrase" />
      <span class="field-hint">Recommended: 10+ chars in length</span>

      <label class="field-label">Expiration date:</label>
      <DatePicker v-model="expiryDate" :min-date="new Date().toISOString().split('T')[0]" />

      <button class="advanced-toggle" @click="showAdvanced = !showAdvanced">
        Advanced {{ showAdvanced ? '\u2303' : '\u2304' }}
      </button>

      <template v-if="showAdvanced">
        <label class="field-label">Key type:</label>
        <div class="checkbox-group">
          <label><input type="checkbox" v-model="encryption" /> Encryption subkey</label>
          <label><input type="checkbox" v-model="signing" /> Signing subkey</label>
          <label><input type="checkbox" v-model="authentication" /> Authentication subkey</label>
        </div>

        <label class="field-label">Key type:</label>
        <select v-model="keyAlgo">
          <option value="curve25519">Curve25519</option>
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

h2 {
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
