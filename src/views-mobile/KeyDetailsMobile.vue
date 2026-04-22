<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { save, confirm as confirmDialog } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { useAppStore } from '@/stores/appStore'

const props = defineProps({ fingerprint: { type: String, required: true } })

const router = useRouter()
const store = useAppStore()
const key = ref(null)
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    key.value = await invoke('get_key_details', { fingerprint: props.fingerprint })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

async function exportKey() {
  const path = await save({
    title: 'Export Public Key',
    defaultPath: `${props.fingerprint}.pub`,
  })
  if (!path) return
  try {
    // On Android `save()` returns a SAF `content://` URI, which the
    // Rust `export_public_key` command can't handle (std::fs::write
    // treats it as a literal path and fails silently — a 0-byte
    // placeholder gets left behind). Fetch the armored text from Rust
    // and write it through the fs plugin, which understands content
    // URIs on Android and behaves like a normal filesystem write on
    // desktop.
    const armored = await invoke('get_public_armored', { fingerprint: props.fingerprint })
    await writeTextFile(path, armored)
  } catch (e) {
    alert(String(e))
  }
}

async function deleteKey() {
  // `window.confirm` is a no-op in Tauri's Android WebView (JS dialogs
  // are suppressed), so the delete would fire through unguarded.
  // Use plugin-dialog's native confirm, which works on every platform.
  const ok = await confirmDialog('Delete this key? This cannot be undone.', {
    title: 'Delete key',
    kind: 'warning',
    okLabel: 'Delete',
    cancelLabel: 'Cancel',
  })
  if (!ok) return
  try {
    await invoke('delete_key', { fingerprint: props.fingerprint })
    await store.refreshKeys()
    router.replace(store.hasKeys ? '/keys' : '/')
  } catch (e) {
    alert(String(e))
  }
}

function displayUid(uid) {
  return uid.name
    ? `${uid.name}${uid.email ? ' <' + uid.email + '>' : ''}`
    : uid.email
}
</script>

<template>
  <div class="details">
    <p v-if="loading" class="muted">Loading…</p>
    <p v-else-if="error" class="error">{{ error }}</p>
    <template v-else-if="key">
      <section class="block">
        <h2>Identity</h2>
        <ul class="uid-list">
          <li
            v-for="(uid, idx) in key.user_ids"
            :key="idx"
            :class="{ revoked: uid.revoked }"
          >
            {{ displayUid(uid) }}
            <span v-if="uid.revoked" class="pill">revoked</span>
          </li>
        </ul>
      </section>

      <section class="block">
        <h2>Primary key</h2>
        <div class="kv"><span>Fingerprint</span><code>{{ key.fingerprint }}</code></div>
        <div class="kv"><span>Created</span><span>{{ key.creation_time }}</span></div>
        <div class="kv"><span>Expires</span><span>{{ key.expiration_time || 'Never' }}</span></div>
        <div class="kv"><span>Type</span><span>{{ key.key_type || '—' }}</span></div>
        <div class="kv"><span>Kind</span><span>{{ key.is_secret ? 'Private' : 'Public' }}</span></div>
      </section>

      <section v-if="key.subkeys && key.subkeys.length" class="block">
        <h2>Subkeys</h2>
        <ul class="sub-list">
          <li v-for="sk in key.subkeys" :key="sk.fingerprint" :class="{ revoked: sk.is_revoked }">
            <div class="sub-title">{{ sk.key_type }}</div>
            <div class="sub-fp">{{ sk.fingerprint }}</div>
            <div class="sub-meta">
              Created {{ sk.creation_time }} · Expires {{ sk.expiration_time }}
              <span v-if="sk.is_revoked" class="pill">revoked</span>
            </div>
          </li>
        </ul>
      </section>

      <div class="actions">
        <button
          v-if="key.is_secret"
          class="primary"
          @click="router.push({ name: 'upload-to-card', query: { fingerprint } })"
        >
          Upload to card
        </button>
        <button class="secondary" @click="exportKey">Export public key</button>
        <button class="danger" @click="deleteKey">Delete key</button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.details { padding: 16px; display: flex; flex-direction: column; gap: 16px; }

.block { background: var(--color-bg); border: 1px solid var(--color-border); border-radius: 12px; padding: 16px; }
.block h2 { font-size: 13px; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; color: var(--color-text-muted); margin: 0 0 10px; }

.kv { display: flex; justify-content: space-between; gap: 12px; padding: 6px 0; font-size: 14px; border-bottom: 1px dashed var(--color-border); }
.kv:last-child { border-bottom: none; }
.kv span:first-child { color: var(--color-text-muted); }
.kv code { font-family: ui-monospace, Menlo, monospace; font-size: 12px; text-align: right; overflow-wrap: anywhere; }

.uid-list, .sub-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 8px; }
.uid-list li, .sub-list li { font-size: 14px; }
.sub-title { font-weight: 600; font-size: 13px; text-transform: capitalize; }
.sub-fp { font-family: ui-monospace, Menlo, monospace; font-size: 11px; color: var(--color-text-muted); overflow-wrap: anywhere; }
.sub-meta { font-size: 12px; color: var(--color-text-muted); margin-top: 2px; }
.revoked { opacity: 0.6; }

.pill { display: inline-block; background: var(--color-expired-bg); color: var(--color-expired-text); border: 1px solid var(--color-expired-border); font-size: 10px; font-weight: 700; padding: 2px 6px; border-radius: 6px; margin-left: 6px; }

.actions { display: flex; flex-direction: column; gap: 10px; }
.actions button { min-height: 48px; font-size: 15px; font-weight: 600; border-radius: 10px; cursor: pointer; font-family: var(--font-family); border: 1px solid transparent; }
.primary { background: var(--color-green); color: #0a2e1c; }
.secondary { background: #fff; color: var(--color-text); border-color: var(--color-border-input); }
.danger { background: var(--color-red); color: #fff; }

.muted { color: var(--color-text-muted); padding: 24px; text-align: center; }
.error { color: var(--color-red-text); padding: 16px; }
</style>
