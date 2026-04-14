<script setup>
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import bigKeySvg from '@/assets/icons/big_key.svg'
import plusSvg from '@/assets/icons/plus.svg'
import importSvg from '@/assets/icons/import.svg'

const router = useRouter()
const store = useAppStore()

async function importKey() {
  const path = await open({
    title: 'Import Key',
    multiple: false,
  })
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
  <div class="start-view">
    <img :src="bigKeySvg" alt="" class="big-key" />
    <h1>No keys added yet</h1>
    <p>You can import an existing key or generate a new one</p>
    <div class="start-actions">
      <TButton variant="green" :icon="plusSvg" @click="router.push('/keys/generate')">
        Generate New Key
      </TButton>
      <TButton variant="transparent" :icon="importSvg" @click="importKey">
        Import Key
      </TButton>
    </div>
  </div>
</template>

<style scoped>
.start-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
}

.big-key {
  width: 80px;
  height: 80px;
  margin-bottom: 8px;
  opacity: 0.5;
}

h1 {
  font-size: 20px;
  font-weight: 700;
}

p {
  color: var(--color-text-muted);
  font-size: 14px;
}

.start-actions {
  display: flex;
  gap: 16px;
  margin-top: 8px;
  align-items: center;
}
</style>
