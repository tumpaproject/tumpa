<script setup>
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'

const router = useRouter()
const route = useRoute()
const store = useAppStore()

onMounted(async () => {
  try {
    const { name, emails, password, expiryDate, encryption, signing, authentication, cipher } = route.query

    await invoke('generate_key', {
      name,
      emails: JSON.parse(emails),
      password,
      expiryDate: expiryDate || null,
      encryption: encryption === 'true',
      signing: signing === 'true',
      authentication: authentication === 'true',
      cipherSuite: cipher || 'curve25519',
    })

    await store.refreshKeys()
    router.replace('/keys')
  } catch (e) {
    store.setError(String(e))
    router.replace('/error')
  }
})
</script>

<template>
  <div class="generating-view">
    <div class="spinner"></div>
    <p>Please wait for the operation to finish...</p>
  </div>
</template>

<style scoped>
.generating-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 24px;
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid var(--color-border);
  border-top-color: var(--color-sidebar);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

p {
  font-size: 14px;
  color: var(--color-text-muted);
}
</style>
