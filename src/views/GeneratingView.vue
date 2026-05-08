<script setup>
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import WaitSpinner from '@/components/WaitSpinner.vue'

const router = useRouter()
const store = useAppStore()

onMounted(async () => {
  const params = store.pendingKeyGen
  if (!params) {
    router.replace('/')
    return
  }

  try {
    await invoke('generate_key', {
      name: params.name,
      emails: params.emails,
      password: params.password,
      expiryDate: params.expiryDate || null,
      encryption: params.encryption,
      signing: params.signing,
      authentication: params.authentication,
      cipherSuite: params.cipher || 'curve25519',
    })

    await store.refreshKeys()
    router.replace('/keys')
  } catch (e) {
    store.setError(String(e))
    router.replace('/error')
  } finally {
    // Clear sensitive params from store immediately
    store.clearPendingKeyGen()
  }
})
</script>

<template>
  <WaitSpinner message="Creating new OpenPGP key, please wait!" />
</template>

<style scoped>
</style>
