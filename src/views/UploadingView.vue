<script setup>
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import WaitSpinner from '@/components/WaitSpinner.vue'

const router = useRouter()
const route = useRoute()

onMounted(async () => {
  try {
    const { fingerprint, password, whichSubkeys } = route.query
    await invoke('upload_key_to_card', {
      fingerprint,
      password,
      whichSubkeys: parseInt(whichSubkeys),
    })
    alert('Key uploaded to card successfully.')
    router.replace('/keys')
  } catch (e) {
    alert(String(e))
    router.back()
  }
})
</script>

<template>
  <WaitSpinner
    message="Uploading to smartcard, please wait!"
    hint="Do not remove the card during this operation."
  />
</template>

<style scoped>
</style>
