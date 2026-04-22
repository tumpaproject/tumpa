<script setup>
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import bigKeySvg from '@/assets/icons/big_key.svg'
import plusSvg from '@/assets/icons/plus.svg'
import importSvg from '@/assets/icons/import.svg'

const router = useRouter()
const store = useAppStore()

// No onMounted refresh here — SidebarLayout.onMounted owns the initial
// keystore load + redirect. Rendering the empty state unconditionally
// means the content area is never blank while the refresh is in
// flight; if the store turns out to have keys the layout will redirect
// us to /keys before WebKit paints a second frame, so in practice the
// user doesn't see the empty state flash on a cold start with keys.

async function importKey() {
  const path = await open({
    title: 'Import Key',
    multiple: false,
  })
  if (!path) return

  try {
    // Symmetric with export: read via plugin-fs (handles Android SAF
    // `content://` URIs the dialog plugin returns) and pass bytes to
    // Rust, instead of a path that `std::fs::read` can't resolve on
    // Android.
    const data = await readFile(path)
    await invoke('import_public_key', { data: Array.from(data) })
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
