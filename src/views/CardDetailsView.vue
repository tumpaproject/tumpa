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
    <div class="card-info" v-if="store.cardDetails">
      <div class="info-row">
        <span class="info-label">Serial Number:</span>
        <span class="info-value">{{ store.cardDetails.serial_number }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Name of cardholder:</span>
        <span class="info-value">{{ store.cardDetails.cardholder_name || '[not set]' }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Public URL:</span>
        <span class="info-value">{{ store.cardDetails.public_key_url || '[not set]' }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Manufacturer:</span>
        <span class="info-value">{{ store.cardDetails.manufacturer || '[unknown]' }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">User PIN retries:</span>
        <span class="info-value">{{ store.cardDetails.pin_retry_counter }}</span>
      </div>
      <div class="info-row">
        <span class="info-label">Admin PIN retries:</span>
        <span class="info-value">{{ store.cardDetails.admin_pin_retry_counter }}</span>
      </div>
    </div>
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

.loading {
  color: var(--color-text-muted);
}
</style>
