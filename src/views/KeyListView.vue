<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'
import KeyItem from '@/components/KeyItem.vue'
import plusSvg from '@/assets/icons/plus.svg'
import importSvg from '@/assets/icons/import.svg'

const router = useRouter()
const store = useAppStore()
const keyFilter = ref('all')
const searchQuery = ref('')

onMounted(async () => {
  // Skip the refresh if the store was already primed for this session
  // (e.g. StartView just redirected us here after its own refresh).
  // Every mutating command — import / generate / delete / add-uid /
  // revoke / expiry update — already calls store.refreshKeys on its
  // own, so KeyListView only needs to load on a truly cold mount.
  if (!store.keysLoaded) {
    await store.refreshKeys()
  }
  if (!store.hasKeys) {
    router.replace('/')
  }
})

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
  let keys = store.keys
  if (keyFilter.value === 'private') keys = keys.filter(k => k.is_secret)
  else if (keyFilter.value === 'public') keys = keys.filter(k => !k.is_secret)

  const needle = searchQuery.value.trim().toLowerCase()
  if (!needle) return keys
  return keys.filter(k => keyMatchesSearch(k, needle))
})

function clearSearch() {
  searchQuery.value = ''
}

const filterLabel = computed(() => {
  if (keyFilter.value === 'private') return 'Private keys'
  if (keyFilter.value === 'public') return 'Public keys'
  return 'All keys'
})

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

async function uploadToCard(fingerprint) {
  const connected = await store.checkCard()
  if (!connected) {
    alert('No smartcard connected.')
    return
  }
  router.push({ name: 'upload-to-card', query: { fingerprint } })
}
</script>

<template>
  <div class="key-list-view">
    <div class="toolbar">
      <TButton variant="green" :icon="plusSvg" @click="router.push('/keys/generate')">
        Generate New Key
      </TButton>
      <TButton variant="white" :icon="importSvg" @click="importKey">
        Import Key
      </TButton>
      <div class="toolbar-spacer"></div>
      <select v-model="keyFilter" class="key-filter" aria-label="Filter keys by type">
        <option value="all">All keys</option>
        <option value="private">Private keys</option>
        <option value="public">Public keys</option>
      </select>
    </div>

    <div class="key-list-content">
      <div class="list-header">
        <h1>{{ filterLabel }}</h1>
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
      <DynamicScroller
        v-if="filteredKeys.length > 0"
        :items="filteredKeys"
        :min-item-size="170"
        key-field="fingerprint"
        class="key-list-scroller"
      >
        <template #default="{ item, index, active }">
          <DynamicScrollerItem
            :item="item"
            :active="active"
            :data-index="index"
            :size-dependencies="[item.user_ids, item.card_idents]"
          >
            <div class="key-list-row">
              <KeyItem
                :key-data="item"
                @details="router.push(`/keys/${item.fingerprint}`)"
                @upload="uploadToCard(item.fingerprint)"
                @export="exportKey(item.fingerprint)"
                @delete="deleteKey(item.fingerprint)"
              />
            </div>
          </DynamicScrollerItem>
        </template>
      </DynamicScroller>
      <p v-else class="empty-filter">
        <template v-if="searchQuery">No keys match "{{ searchQuery }}".</template>
        <template v-else>No {{ keyFilter }} keys found.</template>
      </p>
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
  align-items: center;
}

.toolbar-spacer {
  flex: 1;
}

.key-filter {
  padding: 6px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 6px;
  font-size: 13px;
  font-family: var(--font-family);
  background: white;
  cursor: pointer;
  width: auto;
  max-width: 150px;
}

.key-list-content {
  padding: 24px;
  flex: 1;
  /* DynamicScroller owns its own scrolling now; removing the outer
     overflow-y keeps only one scroll axis on the page. */
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.key-list-scroller {
  flex: 1;
  min-height: 0;
}

/* DynamicScrollerItem wraps each row; add the 12px gap the old
   .key-list flex column used to supply. */
.key-list-row {
  padding-bottom: 12px;
}

.list-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

h1 {
  font-size: 24px;
  font-weight: 700;
}

.search-box {
  position: relative;
  flex: 1;
  max-width: 360px;
}

.search-input {
  width: 100%;
  padding: 8px 36px 8px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 6px;
  font-size: 13px;
  font-family: var(--font-family);
  background: white;
  outline: none;
  transition: border-color 0.15s;
}

.search-input:focus {
  border-color: var(--color-sidebar);
  box-shadow: 0 0 0 2px var(--color-sidebar-focus);
}

.search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  border-radius: 50%;
  border: none;
  background: var(--color-border);
  color: var(--color-text-muted);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  line-height: 1;
  padding: 0;
}

.search-clear:hover,
.search-clear:focus-visible {
  background: var(--color-sidebar);
  color: white;
}

.key-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.empty-filter {
  color: var(--color-text-muted);
  font-size: 14px;
  text-align: center;
  padding: 40px;
}
</style>
