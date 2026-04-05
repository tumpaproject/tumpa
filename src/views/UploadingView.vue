<script setup>
import { onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

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
  <div class="uploading-view">
    <div class="spinner"></div>
    <p>Uploading key to card...</p>
    <p class="hint">Do not remove the card during this operation.</p>
  </div>
</template>

<style scoped>
.uploading-view {
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

p { font-size: 14px; color: var(--color-text-muted); }
.hint { font-size: 12px; }
</style>
