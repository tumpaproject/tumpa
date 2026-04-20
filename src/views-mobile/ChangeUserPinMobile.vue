<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'
import CardConnectMobile from '@/views-mobile/CardConnectMobile.vue'
import { useCardOp } from '@/utils/useCardOp'

const router = useRouter()
const store = useAppStore()

const adminPin = ref('')
const newPin = ref('')

const { busy, phase, error, run, cancel } = useCardOp()

async function save() {
  if (!adminPin.value || !newPin.value) {
    error.value = 'Please fill in all fields.'
    return
  }
  try {
    await run('change_user_pin', {
      adminPin: adminPin.value,
      newPin: newPin.value,
    })
    await store.fetchCardDetails()
    router.replace('/card')
  } catch {
    // surfaced inline
  }
}
</script>

<template>
  <div class="form">
    <label class="label" for="cup-admin">Current admin PIN</label>
    <input
      id="cup-admin"
      v-model="adminPin"
      type="password"
      autocomplete="current-password"
      inputmode="numeric"
    />

    <label class="label" for="cup-new">New user PIN</label>
    <input
      id="cup-new"
      v-model="newPin"
      type="password"
      autocomplete="new-password"
      inputmode="numeric"
    />

    <p v-if="error && !busy" class="error" role="alert">{{ error }}</p>

    <button class="primary" :disabled="busy" @click="save">Save</button>

    <CardConnectMobile
      v-if="busy"
      :phase="phase"
      action="changing PIN"
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
</style>
