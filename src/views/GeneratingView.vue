<script setup>
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import WaitSpinner from '@/components/WaitSpinner.vue'

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
  <WaitSpinner message="Creating new OpenPGP key, please wait!" />
</template>

<style scoped>
</style>
