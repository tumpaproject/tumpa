<script setup>
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useAppStore } from '@/stores/appStore'

const router = useRouter()
const store = useAppStore()

onMounted(async () => {
  await store.refreshKeys()
  if (!store.hasKeys) {
    router.replace('/')
  }
})

async function importKey() {
  const path = await open({ title: 'Import Key', multiple: false })
  if (!path) return
  try {
    await invoke('import_public_key', { filePath: path })
    await store.refreshKeys()
  } catch (e) {
    alert(String(e))
  }
}

function formatUid(uid) {
  return uid.name ? `${uid.name}${uid.email ? ' <' + uid.email + '>' : ''}` : uid.email
}

function primaryUid(key) {
  const uid = key.user_ids?.find(u => !u.revoked) ?? key.user_ids?.[0]
  return uid ? formatUid(uid) : '(no user id)'
}
</script>

<template>
  <div class="list-view">
    <!-- Generate is intentionally desktop-only. Key generation on a
         phone is slow (no PCSC / hardware entropy helpers) and offers
         a weaker UX than just importing an existing key; mobile users
         should create their key on desktop and sync. -->
    <div class="toolbar">
      <button class="secondary" @click="importKey">Import key</button>
    </div>

    <ul class="keys">
      <li
        v-for="key in store.keys"
        :key="key.fingerprint"
        class="row"
        @click="router.push(`/keys/${key.fingerprint}`)"
      >
        <div class="row-main">
          <div class="row-title">{{ primaryUid(key) }}</div>
          <div class="row-fp">{{ key.fingerprint }}</div>
        </div>
        <div class="row-tags">
          <span v-if="key.is_secret" class="tag tag-private">PRIVATE</span>
          <span v-else class="tag tag-public">PUBLIC</span>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.list-view {
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

.toolbar {
  display: flex;
  gap: 10px;
  padding: 12px 16px;
  background: var(--color-bg-light);
  border-bottom: 1px solid var(--color-border);
}

.toolbar button {
  flex: 1;
  min-height: 44px;
  font-size: 15px;
  font-weight: 600;
  border-radius: 10px;
  border: 1px solid transparent;
  font-family: var(--font-family);
  cursor: pointer;
}

.primary { background: var(--color-green); color: #0a2e1c; }
.secondary { background: #fff; color: var(--color-text); border-color: var(--color-border-input); }

.keys {
  list-style: none;
  padding: 0;
  margin: 0;
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  min-height: 60px;
}

.row:active { background: var(--color-bg-light); }

.row-main {
  flex: 1;
  min-width: 0;
}

.row-title {
  font-size: 15px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.row-fp {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: ui-monospace, Menlo, monospace;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

.row-tags {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.tag {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.03em;
  padding: 3px 6px;
  border-radius: 4px;
}

.tag-private { background: #E0F2FE; color: #075985; }
.tag-public { background: #F3F4F6; color: #374151; }
</style>
