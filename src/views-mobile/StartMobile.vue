<script setup>
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'

const router = useRouter()
const store = useAppStore()
// Show the empty-state UI only after we've confirmed there are no keys.
// On cold launch the store is empty, so without this flag we'd flash
// "No keys yet" even for users who have existing keys.
const ready = ref(false)

onMounted(async () => {
  await store.refreshKeys()
  if (store.hasKeys) {
    router.replace('/keys')
    return
  }
  ready.value = true
})

async function importKey() {
  const path = await open({ title: 'Import Key', multiple: false })
  if (!path) return
  try {
    await invoke('import_public_key', { filePath: path })
    await store.refreshKeys()
    router.push('/keys')
  } catch (e) {
    alert(String(e))
  }
}
</script>

<template>
  <div v-if="ready" class="start">
    <h1>No keys yet</h1>
    <p class="subtitle">Generate a new OpenPGP key or import an existing one.</p>
    <div class="actions">
      <button class="primary" @click="router.push('/keys/generate')">Generate new key</button>
      <button class="secondary" @click="importKey">Import key</button>
    </div>
  </div>
</template>

<style scoped>
.start {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  padding: 32px 24px;
  text-align: center;
}

h1 {
  font-size: 22px;
  font-weight: 700;
  margin: 0;
}

.subtitle {
  font-size: 15px;
  color: var(--color-text-muted);
  margin: 0;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  max-width: 320px;
  margin-top: 12px;
}

button {
  min-height: 48px;
  font-size: 16px;
  font-weight: 600;
  border-radius: 10px;
  border: 1px solid transparent;
  cursor: pointer;
  font-family: var(--font-family);
}

.primary {
  background: var(--color-green);
  color: #0a2e1c;
}

.primary:active { background: var(--color-green-hover); }

.secondary {
  background: #fff;
  color: var(--color-text);
  border-color: var(--color-border-input);
}

.secondary:active { background: var(--color-bg-light); }
</style>
