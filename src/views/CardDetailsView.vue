<script setup>
import { onMounted } from 'vue'
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()

onMounted(async () => {
  store.setActiveSection('card', 'card-details')
  await store.fetchCardDetails()
})
</script>

<template>
  <div class="card-view">
    <h2>Smart Card details</h2>
    <dl class="card-info" v-if="store.cardDetails">
      <div class="info-row">
        <dt class="info-label">Serial Number</dt>
        <dd class="info-value">{{ store.cardDetails.serial_number }}</dd>
      </div>
      <div class="info-row">
        <dt class="info-label">Name of cardholder</dt>
        <dd class="info-value">{{ store.cardDetails.cardholder_name || '[not set]' }}</dd>
      </div>
      <div class="info-row">
        <dt class="info-label">Public URL</dt>
        <dd class="info-value">{{ store.cardDetails.public_key_url || '[not set]' }}</dd>
      </div>
      <div class="info-row">
        <dt class="info-label">Manufacturer</dt>
        <dd class="info-value">{{ store.cardDetails.manufacturer_name || store.cardDetails.manufacturer || '[unknown]' }}</dd>
      </div>
      <div class="info-row">
        <dt class="info-label">User PIN retries left</dt>
        <dd class="info-value">{{ store.cardDetails.pin_retry_counter }}</dd>
      </div>
      <div class="info-row">
        <dt class="info-label">Reset PIN retries left</dt>
        <dd class="info-value">{{ store.cardDetails.reset_code_retry_counter }}</dd>
      </div>
      <div class="info-row">
        <dt class="info-label">Admin PIN retries left</dt>
        <dd class="info-value">{{ store.cardDetails.admin_pin_retry_counter }}</dd>
      </div>
      <div class="info-row" v-if="store.cardDetails.signature_fingerprint">
        <dt class="info-label">Signing key</dt>
        <dd class="info-value fingerprint">{{ store.cardDetails.signature_fingerprint.toUpperCase() }}</dd>
      </div>
      <div class="info-row" v-if="store.cardDetails.encryption_fingerprint">
        <dt class="info-label">Encryption key</dt>
        <dd class="info-value fingerprint">{{ store.cardDetails.encryption_fingerprint.toUpperCase() }}</dd>
      </div>
      <div class="info-row" v-if="store.cardDetails.authentication_fingerprint">
        <dt class="info-label">Authentication key</dt>
        <dd class="info-value fingerprint">{{ store.cardDetails.authentication_fingerprint.toUpperCase() }}</dd>
      </div>
    </dl>
    <p v-else class="loading">Loading card details...</p>
  </div>
</template>

<style scoped>
.card-view {
  padding: 24px 32px;
}

h2 {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 24px;
}

.card-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 600px;
}

.info-row {
  display: flex;
  gap: 16px;
}

.info-label {
  min-width: 180px;
  text-align: right;
  color: var(--color-text-muted);
  font-size: 14px;
}

.info-value {
  font-size: 14px;
  font-weight: 500;
}

.fingerprint {
  font-family: monospace;
  font-size: 12px;
  word-break: break-all;
}

.loading {
  color: var(--color-text-muted);
}
</style>
