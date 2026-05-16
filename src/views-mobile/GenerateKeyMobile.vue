<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'
import PasswordInputMobile from '@/components/PasswordInputMobile.vue'

const router = useRouter()
const store = useAppStore()

const name = ref('')
const emails = ref('')
const passphrase = ref('')
const expiryDate = ref('')
const keyAlgo = ref('curve25519')
const showAdvanced = ref(false)
const encryption = ref(true)
const signing = ref(false)
const authentication = ref(true)
const submitting = ref(false)

function submit() {
  if (!name.value.trim()) { alert('Please enter your name.'); return }
  if (!emails.value.trim()) { alert('Please enter at least one email address.'); return }
  if (!passphrase.value) { alert('Please set a passphrase.'); return }

  const emailList = emails.value
    .split('\n')
    .map(e => e.trim())
    .filter(e => e.length > 0)

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
  submitting.value = true
  router.push({ name: 'generating' })
}
</script>

<template>
  <form class="form" @submit.prevent="submit">
    <h2 class="heading">Generate new key</h2>

    <label class="label" for="gm-name">Your Name</label>
    <input id="gm-name" v-model="name" type="text" autocomplete="name" autocorrect="off" />

    <label class="label" for="gm-emails">Email addresses</label>
    <textarea
      id="gm-emails"
      v-model="emails"
      rows="3"
      placeholder="One email per line"
      autocomplete="email"
      autocorrect="off"
      autocapitalize="none"
      spellcheck="false"
    ></textarea>

    <label class="label" for="gm-pass">Key Passphrase</label>
    <PasswordInputMobile id="gm-pass" v-model="passphrase" autocomplete="new-password" />
    <small class="hint">Recommended: 10+ chars in length</small>

    <label class="label" for="gm-algo">Key algorithm</label>
    <select id="gm-algo" v-model="keyAlgo">
      <option value="curve25519">Curve25519 (Legacy)</option>
      <option value="cv25519modern">Curve25519 (Modern, Nitrokey 3)</option>
      <option value="rsa4096">RSA 4096</option>
    </select>

    <label class="label" for="gm-exp">Expiration date (optional)</label>
    <input id="gm-exp" v-model="expiryDate" type="date" :min="new Date().toISOString().split('T')[0]" />

    <button
      type="button"
      class="advanced-toggle"
      :aria-expanded="showAdvanced"
      @click="showAdvanced = !showAdvanced"
    >
      Advanced {{ showAdvanced ? '\u2303' : '\u2304' }}
    </button>

    <fieldset v-if="showAdvanced" class="checkbox-group">
      <legend class="label">Key capabilities</legend>
      <label class="check"><input type="checkbox" v-model="encryption" /> Encryption subkey</label>
      <label class="check"><input type="checkbox" v-model="signing" /> Signing subkey</label>
      <label class="check"><input type="checkbox" v-model="authentication" /> Authentication subkey</label>
    </fieldset>

    <button type="submit" class="primary" :disabled="submitting">
      {{ submitting ? 'Please wait…' : 'Confirm' }}
    </button>
  </form>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px;
}

.heading {
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 8px;
}

.label {
  font-size: 13px;
  font-weight: 600;
  margin-top: 10px;
  color: var(--color-text-dark);
}

input, textarea, select {
  padding: 10px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 10px;
  font-size: 16px; /* prevents iOS zoom-on-focus */
  font-family: var(--font-family);
  background: #fff;
  box-sizing: border-box;
  width: 100%;
}

input, select { min-height: 44px; }

/* Scoped `background: #fff` above resets the global chevron; restore it
   here so the dropdown keeps a custom arrow instead of native chrome. */
select {
  padding-right: 36px;
  background-color: #fff;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'%3E%3Cpath fill='none' stroke='%236B7280' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round' d='M1 1.5 6 6.5 11 1.5'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
}

textarea {
  resize: vertical;
  min-height: 96px;
  line-height: 1.4;
}

input:focus, textarea:focus, select:focus {
  outline: 2px solid var(--color-sidebar-focus);
  outline-offset: 1px;
}

.hint {
  color: var(--color-text-muted);
  font-size: 12px;
}

.advanced-toggle {
  align-self: flex-start;
  margin-top: 14px;
  padding: 8px 14px;
  min-height: 40px;
  background: var(--color-border);
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  font-family: var(--font-family);
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 12px 14px;
  margin-top: 8px;
}

.checkbox-group legend {
  padding: 0 6px;
}

.check {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  cursor: pointer;
  min-height: 32px;
}

.check input[type="checkbox"] {
  width: 20px;
  height: 20px;
  min-height: 20px;
  margin: 0;
  flex-shrink: 0;
}

.primary {
  margin-top: 20px;
  min-height: 50px;
  border-radius: 12px;
  border: none;
  background: var(--color-green);
  color: #0a2e1c;
  font-size: 16px;
  font-weight: 700;
  cursor: pointer;
  font-family: var(--font-family);
}

.primary:disabled { opacity: 0.6; cursor: wait; }
</style>
