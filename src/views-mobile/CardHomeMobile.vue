<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '@/stores/appStore'
import CardConnectMobile from '@/views-mobile/CardConnectMobile.vue'

const router = useRouter()
const store = useAppStore()

const reading = ref(false)
const overlayPhase = ref('waiting')
const errorMessage = ref('')

let unlistenCardConnected = null

onMounted(async () => {
  try {
    unlistenCardConnected = await listen('plugin:tumpa-card:card-connected', () => {
      overlayPhase.value = 'connected'
    })
  } catch (e) {
    // Plugin not loaded (desktop / web); overlay stays in waiting phase.
  }
})

onUnmounted(() => {
  if (typeof unlistenCardConnected === 'function') unlistenCardConnected()
})

async function readCard() {
  errorMessage.value = ''
  overlayPhase.value = 'waiting'
  reading.value = true
  try {
    await store.fetchCardDetails()
    if (store.errorMessage) {
      errorMessage.value = store.errorMessage
      store.clearError()
    }
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    reading.value = false
  }
}

function cancel() {
  reading.value = false
}
</script>

<template>
  <div class="card-home">
    <div class="toolbar">
      <button class="primary" :disabled="reading" @click="readCard">
        {{ store.cardDetails ? 'Read again' : 'Read card' }}
      </button>
    </div>

    <p v-if="!store.cardDetails && !reading" class="hint">
      Tap <strong>Read card</strong>, then hold your OpenPGP smartcard
      against the top of the phone.
    </p>

    <p v-if="errorMessage && !reading" class="error" role="alert">
      {{ errorMessage }}
    </p>

    <dl v-if="store.cardDetails" class="info">
      <div class="row">
        <dt>Serial</dt>
        <dd class="mono">{{ store.cardDetails.serial_number }}</dd>
      </div>
      <div class="row">
        <dt>Cardholder</dt>
        <dd>{{ store.cardDetails.cardholder_name || '[not set]' }}</dd>
      </div>
      <div class="row">
        <dt>Public URL</dt>
        <dd class="break">{{ store.cardDetails.public_key_url || '[not set]' }}</dd>
      </div>
      <div class="row">
        <dt>Manufacturer</dt>
        <dd>{{ store.cardDetails.manufacturer_name || store.cardDetails.manufacturer || '[unknown]' }}</dd>
      </div>
      <div class="row">
        <dt>User PIN retries</dt>
        <dd>{{ store.cardDetails.pin_retry_counter }}</dd>
      </div>
      <div class="row">
        <dt>Reset PIN retries</dt>
        <dd>{{ store.cardDetails.reset_code_retry_counter }}</dd>
      </div>
      <div class="row">
        <dt>Admin PIN retries</dt>
        <dd>{{ store.cardDetails.admin_pin_retry_counter }}</dd>
      </div>
      <div v-if="store.cardDetails.signature_fingerprint" class="row">
        <dt>Signing</dt>
        <dd class="mono break">{{ store.cardDetails.signature_fingerprint.toUpperCase() }}</dd>
      </div>
      <div v-if="store.cardDetails.encryption_fingerprint" class="row">
        <dt>Encryption</dt>
        <dd class="mono break">{{ store.cardDetails.encryption_fingerprint.toUpperCase() }}</dd>
      </div>
      <div v-if="store.cardDetails.authentication_fingerprint" class="row">
        <dt>Authentication</dt>
        <dd class="mono break">{{ store.cardDetails.authentication_fingerprint.toUpperCase() }}</dd>
      </div>
    </dl>

    <div v-if="store.cardDetails" class="actions">
      <button class="action" @click="router.push('/card/edit-name')">
        Edit cardholder name
      </button>
      <button class="action" @click="router.push('/card/edit-url')">
        Edit public URL
      </button>
      <button class="action" @click="router.push('/card/change-user-pin')">
        Change user PIN
      </button>
      <button class="action" @click="router.push('/card/change-admin-pin')">
        Change admin PIN
      </button>
    </div>

    <CardConnectMobile
      v-if="reading"
      :phase="overlayPhase"
      action="reading"
      :error="errorMessage"
      @cancel="cancel"
    />
  </div>
</template>

<style scoped>
.card-home {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.toolbar {
  display: flex;
  gap: 10px;
}

.primary {
  flex: 1;
  min-height: 48px;
  font-size: 16px;
  font-weight: 700;
  border-radius: 12px;
  border: none;
  background: var(--color-green);
  color: #0a2e1c;
  cursor: pointer;
  font-family: var(--font-family);
}

.primary:disabled { opacity: 0.55; cursor: wait; }

.hint {
  font-size: 14px;
  color: var(--color-text-muted);
  line-height: 1.4;
  margin: 0;
}

.error {
  font-size: 13px;
  color: var(--color-red-text);
  background: var(--color-expired-bg);
  border: 1px solid var(--color-expired-border);
  border-radius: 8px;
  padding: 8px 10px;
  margin: 0;
}

.info {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 4px 0 0;
  padding: 12px 14px;
  background: var(--color-bg-light);
  border-radius: 10px;
}

.row {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

dt {
  font-size: 12px;
  color: var(--color-text-muted);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

dd {
  margin: 0;
  font-size: 15px;
}

.mono {
  font-family: ui-monospace, Menlo, monospace;
  font-size: 12px;
}

.break { overflow-wrap: anywhere; }

.actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.action {
  min-height: 48px;
  border-radius: 10px;
  border: 1px solid var(--color-border-input);
  background: #fff;
  color: var(--color-text);
  font-size: 15px;
  font-weight: 600;
  text-align: left;
  padding: 0 14px;
  cursor: pointer;
  font-family: var(--font-family);
}

.action:active { background: var(--color-bg-light); }
</style>
