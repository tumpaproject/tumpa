<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import CardConnectMobile from '@/views-mobile/CardConnectMobile.vue'
import { useAppStore } from '@/stores/appStore'
import {
  keyForKeyPassphrase,
  saveSecret,
  readSecretAsString,
  isMissingSecretError,
  isCancelledError,
} from '@/utils/keyring'
import { setCardTransport } from '@/utils/cardTransport'

const router = useRouter()
const route = useRoute()
const appStore = useAppStore()

const fingerprint = ref('')
const password = ref('')
const loading = ref(true)
const uploading = ref(false)
const uploaded = ref(false)
const errorMessage = ref('')

// Availability flags
const primaryCanSign = ref(false)
const signingSubkeyAvailable = ref(false)
const encryptionAvailable = ref(false)
const authenticationAvailable = ref(false)

// User selections
const signingSlot = ref('')         // 'primary' | 'subkey'
const uploadEncryption = ref(false)
const uploadAuthentication = ref(false)
// Transport the user wants to use for this upload. Default 'nfc' —
// works on every device with NFC hardware; USB needs a CCID reader
// plugged in before upload.
const transport = ref('nfc')

// Keyring — only exposed on mobile. Checkbox persists the passphrase
// to the platform keyring (iOS Keychain / Android EncryptedSharedPrefs)
// after a successful upload, gated by biometrics on read-back.
const rememberPassphrase = ref(false)
const biometricBusy = ref(false)

// Overlay phase. `waiting` while we've armed reader mode but no tap
// has happened yet; `connected` once the plugin fires its
// `card-connected` event (meaning the SELECT succeeded and APDUs are
// now flowing).
const overlayPhase = ref('waiting')
let unlistenCardConnected = null

onMounted(async () => {
  // Subscribe to `plugin:tumpa-card:card-connected` so the overlay can
  // swap from "Hold your card" to "Card found — uploading…" once the
  // plugin has SELECTed the OpenPGP applet.
  try {
    unlistenCardConnected = await listen('plugin:tumpa-card:card-connected', () => {
      overlayPhase.value = 'connected'
    })
  } catch (e) {
    // Desktop build or plugin not loaded — the overlay will stay in
    // `waiting` which is still descriptive enough.
    console.debug('card-connected listener not registered:', e)
  }

  fingerprint.value = route.query.fingerprint || route.params.fingerprint || ''
  if (!fingerprint.value) {
    router.back()
    return
  }
  try {
    const avail = await invoke('get_available_subkeys', { fingerprint: fingerprint.value })
    primaryCanSign.value = avail.primary_can_sign
    signingSubkeyAvailable.value = avail.signing_subkey
    encryptionAvailable.value = avail.encryption
    authenticationAvailable.value = avail.authentication

    uploadEncryption.value = avail.encryption
    uploadAuthentication.value = avail.authentication
    signingSlot.value = avail.signing_subkey
      ? 'subkey'
      : (avail.primary_can_sign ? 'primary' : '')
  } catch (e) {
    errorMessage.value = String(e)
  }
  loading.value = false
})

onUnmounted(() => {
  if (typeof unlistenCardConnected === 'function') unlistenCardConnected()
})

const canUpload = computed(() => {
  return !!password.value && (
    uploadEncryption.value ||
    uploadAuthentication.value ||
    signingSlot.value !== ''
  )
})

async function upload() {
  if (!password.value) {
    errorMessage.value = 'Please enter the key password.'
    return
  }

  let whichSubkeys = 0
  if (uploadEncryption.value) whichSubkeys += 1
  if (signingSlot.value === 'primary') whichSubkeys += 2
  if (uploadAuthentication.value) whichSubkeys += 4
  if (signingSlot.value === 'subkey') whichSubkeys += 8

  if (whichSubkeys === 0) {
    errorMessage.value = 'Please select at least one key to upload.'
    return
  }

  // No `window.confirm()` here — on Android that would render a
  // native dialog that defocuses the Tauri activity; any NFC tap
  // during the defocus window gets dispatched to the system NDEF
  // handler (e.g. Chrome) instead of our reader-mode callback. The
  // destructive-action warning is static text above the Upload
  // button — visible from the moment the form loads — so the user
  // has already consented by clicking Upload.
  errorMessage.value = ''
  overlayPhase.value = 'waiting'
  uploading.value = true

  try {
    await setCardTransport(transport.value)
    await invoke('upload_key_to_card', {
      fingerprint: fingerprint.value,
      password: password.value,
      whichSubkeys,
    })
    // Upload succeeded → the passphrase was correct, so it's safe to
    // persist if the user opted in. We save BEFORE clearing the
    // reactive ref so the bytes are still in memory, then overwrite.
    if (rememberPassphrase.value && appStore.isMobile) {
      try {
        await saveSecret(keyForKeyPassphrase(fingerprint.value), password.value)
      } catch (e) {
        // Save failure shouldn't block the "upload succeeded" flow,
        // just surface a debug-level message.
        console.warn('save passphrase to keyring failed:', e)
      }
    }
    password.value = ''
    uploaded.value = true
  } catch (e) {
    errorMessage.value = String(e)
  } finally {
    uploading.value = false
  }
}

function done() {
  router.replace('/card')
}

async function useSavedPassphrase() {
  if (biometricBusy.value) return
  biometricBusy.value = true
  errorMessage.value = ''
  try {
    const cached = await readSecretAsString(
      keyForKeyPassphrase(fingerprint.value),
      'Unlock key passphrase',
    )
    password.value = cached
  } catch (e) {
    if (isMissingSecretError(e)) {
      errorMessage.value = 'No saved passphrase for this key yet.'
    } else if (isCancelledError(e)) {
      // User dismissed the biometric sheet — stay quiet.
    } else {
      errorMessage.value = String(e)
    }
  } finally {
    biometricBusy.value = false
  }
}

function cancelUpload() {
  // The NFC session is driven by the Rust backend; we can't actually
  // interrupt it mid-APDU, but hiding the overlay lets the user back
  // out of the app / put the card away. The invoke will eventually
  // reject and clear `uploading` via the finally block.
  uploading.value = false
}
</script>

<template>
  <div class="upload-view">
    <div v-if="loading" class="muted">Loading key details…</div>
    <div v-else-if="uploaded" class="success-view">
      <div class="success-icon" aria-hidden="true">&#x2713;</div>
      <h2 class="success-title">Upload complete</h2>
      <p class="success-sub">
        Your key is on the card. Open the SmartCards tab to review
        which keys are in each slot.
      </p>
      <button class="primary" @click="done">Done</button>
    </div>
    <template v-else>
      <p class="fp">{{ fingerprint }}</p>

      <div class="pass-header">
        <label class="label" for="up-pass">Key password</label>
        <button
          v-if="appStore.isMobile"
          type="button"
          class="link"
          :disabled="biometricBusy"
          @click="useSavedPassphrase"
        >
          Use saved
        </button>
      </div>
      <input
        id="up-pass"
        v-model="password"
        type="password"
        autocomplete="current-password"
      />
      <label v-if="appStore.isMobile" class="opt remember">
        <input type="checkbox" v-model="rememberPassphrase" />
        Remember passphrase (Face ID / Touch ID)
      </label>

      <fieldset class="group">
        <legend class="label">Card transport</legend>
        <label class="opt">
          <input type="radio" name="transport" value="nfc" v-model="transport" />
          NFC (tap the card)
        </label>
        <label class="opt">
          <input type="radio" name="transport" value="usb" v-model="transport" />
          USB-C (plug the reader in)
        </label>
      </fieldset>

      <fieldset class="group">
        <legend class="label">Signing slot</legend>
        <label :class="{ disabled: !primaryCanSign }" class="opt">
          <input
            type="radio"
            name="signing-slot"
            value="primary"
            v-model="signingSlot"
            :disabled="!primaryCanSign"
          />
          Primary key
        </label>
        <label :class="{ disabled: !signingSubkeyAvailable }" class="opt">
          <input
            type="radio"
            name="signing-slot"
            value="subkey"
            v-model="signingSlot"
            :disabled="!signingSubkeyAvailable"
          />
          Signing subkey
        </label>
      </fieldset>

      <fieldset class="group">
        <legend class="label">Other subkeys</legend>
        <label :class="{ disabled: !encryptionAvailable }" class="opt">
          <input
            type="checkbox"
            v-model="uploadEncryption"
            :disabled="!encryptionAvailable"
          />
          Encryption subkey
        </label>
        <label :class="{ disabled: !authenticationAvailable }" class="opt">
          <input
            type="checkbox"
            v-model="uploadAuthentication"
            :disabled="!authenticationAvailable"
          />
          Authentication subkey
        </label>
      </fieldset>

      <p class="warning" role="alert">
        ⚠ Uploading resets the card. Any keys already on the card will
        be erased.
      </p>

      <p v-if="errorMessage && !uploading" class="error" role="alert">{{ errorMessage }}</p>

      <button class="primary" :disabled="!canUpload" @click="upload">
        Upload
      </button>
    </template>

    <CardConnectMobile
      v-if="uploading"
      :phase="overlayPhase"
      action="uploading"
      :error="errorMessage"
      @cancel="cancelUpload"
    />
  </div>
</template>

<style scoped>
.upload-view {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.muted { color: var(--color-text-muted); padding: 24px; text-align: center; }

.success-view {
  padding: 32px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  text-align: center;
}

.success-icon {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: var(--color-green);
  color: #0a2e1c;
  font-size: 40px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
}

.success-title {
  font-size: 20px;
  font-weight: 700;
  margin: 0;
}

.success-sub {
  font-size: 14px;
  color: var(--color-text-muted);
  margin: 0;
  line-height: 1.45;
  max-width: 320px;
}

.success-view .primary {
  margin-top: 8px;
  width: 100%;
  max-width: 320px;
}

.fp {
  font-family: ui-monospace, Menlo, monospace;
  font-size: 11px;
  color: var(--color-text-muted);
  overflow-wrap: anywhere;
  padding: 10px 12px;
  background: var(--color-bg-light);
  border-radius: 8px;
  margin: 0;
}

.label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-dark);
  margin-top: 10px;
}

.pass-header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 12px;
  margin-top: 10px;
}

.pass-header .label { margin: 0; }

.link {
  background: transparent;
  border: none;
  color: var(--color-sidebar);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  padding: 4px 2px;
  font-family: var(--font-family);
}

.link:disabled { opacity: 0.5; cursor: wait; }

.remember {
  margin-top: 6px;
  font-size: 14px;
}

input[type="password"] {
  min-height: 44px;
  padding: 10px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 10px;
  font-size: 16px;
  font-family: var(--font-family);
  background: #fff;
  box-sizing: border-box;
  width: 100%;
}

input[type="password"]:focus {
  outline: 2px solid var(--color-sidebar-focus);
  outline-offset: 1px;
}

.group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 12px 14px;
  margin: 0;
}

.group legend { padding: 0 6px; }

.opt {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  min-height: 32px;
  cursor: pointer;
}

.opt input[type="radio"],
.opt input[type="checkbox"] {
  width: 20px;
  height: 20px;
  margin: 0;
  flex-shrink: 0;
}

.opt.disabled { opacity: 0.45; cursor: not-allowed; }

.error {
  color: var(--color-red-text);
  font-size: 13px;
  background: var(--color-expired-bg);
  border: 1px solid var(--color-expired-border);
  border-radius: 8px;
  padding: 8px 10px;
  margin: 4px 0 0;
}

.warning {
  color: #8a6400;
  font-size: 13px;
  background: #fff8dc;
  border: 1px solid #f2d57a;
  border-radius: 8px;
  padding: 8px 10px;
  margin: 8px 0 0;
  line-height: 1.4;
}

.primary {
  margin-top: 14px;
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

.primary:disabled { opacity: 0.55; cursor: not-allowed; }
</style>
