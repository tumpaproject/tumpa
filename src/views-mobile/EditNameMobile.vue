<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'
import CardConnectMobile from '@/views-mobile/CardConnectMobile.vue'
import PasswordInputMobile from '@/components/PasswordInputMobile.vue'
import { useCardOp } from '@/utils/useCardOp'
import { setCardTransport } from '@/utils/cardTransport'
import { isIosPlatform } from '@/utils/platform'

const isIos = isIosPlatform()

const router = useRouter()
const store = useAppStore()

const name = ref(store.cardDetails?.cardholder_name || '')
const adminPin = ref('')
const transport = ref('nfc')

const { busy, phase, error, run, cancel } = useCardOp()

async function save() {
  if (!name.value.trim() || !adminPin.value) {
    error.value = 'Please fill in all fields.'
    return
  }
  try {
    await setCardTransport(transport.value)
    await run('update_card_name', {
      name: name.value.trim(),
      adminPin: adminPin.value,
    })
    await store.fetchCardDetails()
    router.replace('/card')
  } catch {
    // error already surfaced by useCardOp
  }
}
</script>

<template>
  <div class="form">
    <label class="label" for="card-name">Cardholder name</label>
    <input id="card-name" v-model="name" type="text" autocomplete="off" />

    <label class="label" for="card-name-pin">Current admin PIN</label>
    <PasswordInputMobile
      id="card-name-pin"
      v-model="adminPin"
      autocomplete="current-password"
      inputmode="numeric"
    />

    <fieldset v-if="!isIos" class="group">
      <legend class="label">Card transport</legend>
      <label class="opt">
        <input type="radio" name="transport" value="nfc" v-model="transport" />
        NFC
      </label>
      <label class="opt">
        <input type="radio" name="transport" value="usb" v-model="transport" />
        USB-C
      </label>
    </fieldset>

    <p v-if="error && !busy" class="error" role="alert">{{ error }}</p>

    <button class="primary" :disabled="busy" @click="save">Save</button>

    <CardConnectMobile
      v-if="busy"
      :phase="phase"
      action="updating"
      :transport="transport"
      :error="error"
      @cancel="cancel"
    />
  </div>
</template>

<style scoped>
.form {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.label {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-dark);
  margin-top: 6px;
}

input[type="text"],
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

input:focus {
  outline: 2px solid var(--color-sidebar-focus);
  outline-offset: 1px;
}

.error {
  color: var(--color-red-text);
  font-size: 13px;
  background: var(--color-expired-bg);
  border: 1px solid var(--color-expired-border);
  border-radius: 8px;
  padding: 8px 10px;
  margin: 4px 0 0;
}

.primary {
  margin-top: 10px;
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

.primary:disabled { opacity: 0.55; cursor: wait; }

.group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  border: 1px solid var(--color-border);
  border-radius: 10px;
  padding: 10px 12px;
  margin: 6px 0 0;
}
.group legend { padding: 0 4px; }
.opt {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  min-height: 32px;
  cursor: pointer;
}
.opt input[type="radio"] {
  width: 20px;
  height: 20px;
  margin: 0;
  flex-shrink: 0;
}
</style>
