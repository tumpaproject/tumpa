<script setup>
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import KeyItem from '@/components/KeyItem.vue'
import plusSvg from '@/assets/icons/plus.svg'
import importSvg from '@/assets/icons/import.svg'

const router = useRouter()
const store = useAppStore()

onMounted(async () => {
  await store.refreshKeys()
  if (!store.hasKeys) {
    router.replace('/')
  }
})

async function importKey() {
  const path = await open({
    title: 'Import Secret Key',
    multiple: false,
  })
  if (!path) return

  try {
    await invoke('import_key', { filePath: path })
    await store.refreshKeys()
  } catch (e) {
    alert(String(e))
  }
}

async function exportKey(fingerprint) {
  const path = await save({
    title: 'Export Public Key',
    defaultPath: `${fingerprint}.pub`,
  })
  if (!path) return

  try {
    await invoke('export_public_key', { fingerprint, filePath: path })
  } catch (e) {
    alert(String(e))
  }
}

async function deleteKey(fingerprint) {
  if (!confirm('Are you sure you want to delete the selected key?')) return

  try {
    await invoke('delete_key', { fingerprint })
    await store.refreshKeys()
    if (!store.hasKeys) {
      router.replace('/')
    }
  } catch (e) {
    alert(String(e))
  }
}

function uploadToCard(fingerprint) {
  store.setCurrentFingerprint(fingerprint)
  // TODO: navigate to upload view in Phase 2
  alert('Upload to card - coming in Phase 2')
}
</script>

<template>
  <div class="key-list-view">
    <div class="toolbar">
      <TButton variant="green" :icon="plusSvg" @click="router.push('/keys/generate')">
        Generate New Key
      </TButton>
      <TButton variant="white" :icon="importSvg" @click="importKey">
        Import Secret Key
      </TButton>
    </div>

    <div class="key-list-content">
      <h2>All keys</h2>
      <div class="key-list">
        <KeyItem
          v-for="key in store.keys"
          :key="key.fingerprint"
          :key-data="key"
          @upload="uploadToCard(key.fingerprint)"
          @export="exportKey(key.fingerprint)"
          @delete="deleteKey(key.fingerprint)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.key-list-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.toolbar {
  display: flex;
  gap: 12px;
  padding: 12px 24px;
  background: var(--color-bg-light);
  border-bottom: 1px solid var(--color-border);
}

.key-list-content {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

h2 {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 16px;
}

.key-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>
