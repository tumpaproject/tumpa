<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useAppStore } from '@/stores/appStore'

const router = useRouter()
const store = useAppStore()
const searchQuery = ref('')

onMounted(async () => {
  // See src/views/KeyListView.vue — skip the refresh when StartMobile
  // has already primed the store for this session.
  if (!store.keysLoaded) {
    await store.refreshKeys()
  }
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

function keyMatchesSearch(key, needle) {
  if (key.fingerprint && key.fingerprint.toLowerCase().includes(needle)) return true
  if (Array.isArray(key.user_ids)) {
    for (const uid of key.user_ids) {
      if (uid?.name && uid.name.toLowerCase().includes(needle)) return true
      if (uid?.email && uid.email.toLowerCase().includes(needle)) return true
    }
  }
  return false
}

const filteredKeys = computed(() => {
  const needle = searchQuery.value.trim().toLowerCase()
  if (!needle) return store.keys
  return store.keys.filter(k => keyMatchesSearch(k, needle))
})

function clearSearch() {
  searchQuery.value = ''
}
</script>

<template>
  <div class="list-view">
    <div class="search-wrap">
      <div class="search-box" :class="{ 'has-value': searchQuery }">
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="Search by name, UID or fingerprint"
          aria-label="Search keys by name, UID or fingerprint"
        />
        <button
          v-if="searchQuery"
          type="button"
          class="search-clear"
          aria-label="Clear search"
          @click="clearSearch"
        >
          <span aria-hidden="true">&times;</span>
        </button>
      </div>
    </div>

    <!-- Persistent mode: Import only. One Shot mode: Generate is
         re-enabled, since the whole point is "create a key in RAM,
         push it to a card, export pub, discard". -->
    <div class="toolbar">
      <button
        v-if="store.mode === 'one-shot'"
        class="primary"
        @click="router.push('/keys/generate')"
      >
        + Generate
      </button>
      <button class="secondary" @click="importKey">Import key</button>
    </div>

    <RecycleScroller
      v-if="filteredKeys.length > 0"
      :items="filteredKeys"
      :item-size="60"
      key-field="fingerprint"
      class="keys-scroller"
      v-slot="{ item }"
    >
      <div
        class="row"
        @click="router.push(`/keys/${item.fingerprint}`)"
      >
        <div class="row-main">
          <div class="row-title">{{ primaryUid(item) }}</div>
          <div class="row-fp">{{ item.fingerprint }}</div>
        </div>
        <div class="row-tags">
          <span v-if="item.is_secret" class="tag tag-private">PRIVATE</span>
          <span v-else class="tag tag-public">PUBLIC</span>
        </div>
      </div>
    </RecycleScroller>
    <p v-else-if="searchQuery" class="empty">
      No keys match "{{ searchQuery }}".
    </p>
  </div>
</template>

<style scoped>
.list-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/* RecycleScroller needs a concrete height + its own scroll region. */
.keys-scroller {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.search-wrap {
  padding: 12px 16px;
  background: var(--color-bg);
  border-bottom: 1px solid var(--color-border);
}

.search-box {
  position: relative;
}

.search-input {
  width: 100%;
  padding: 10px 40px 10px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 10px;
  font-size: 15px;
  font-family: var(--font-family);
  background: var(--color-bg-light);
  outline: none;
  transition: border-color 0.15s;
  min-height: 44px;
}

.search-input:focus {
  border-color: var(--color-sidebar);
  box-shadow: 0 0 0 2px var(--color-sidebar-focus);
  background: var(--color-bg);
}

.search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: none;
  background: var(--color-border);
  color: var(--color-text-muted);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  line-height: 1;
  padding: 0;
}

.search-clear:active,
.search-clear:focus-visible {
  background: var(--color-sidebar);
  color: white;
}

.empty {
  padding: 24px 16px;
  color: var(--color-text-muted);
  font-size: 14px;
  text-align: center;
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
