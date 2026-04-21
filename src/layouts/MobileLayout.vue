<script setup>
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'
import OneShotBanner from '@/components/OneShotBanner.vue'
import { enterOneShot, exitOneShot } from '@/utils/oneShot'

const route = useRoute()
const router = useRouter()
const store = useAppStore()

const title = computed(() => route.meta?.title || 'Tumpa')
const canGoBack = computed(() => route.name !== 'start' && route.name !== 'key-list' && route.name !== 'card-home')

// Tabs hide on the welcome screen (where the user has no keys yet and
// the layout is centered); otherwise they're always visible so the
// user can flip between Keys and SmartCards from any screen.
const showTabs = computed(() => route.name !== 'start')
const activeTab = computed(() => route.path.startsWith('/card') ? 'card' : 'keys')

// Overflow menu state. Simple dropdown anchored to the kebab button;
// open via tap, close via outside click. Mobile doesn't have room for
// a third tab so the One Shot toggle lives here.
const menuOpen = ref(false)

function toggleMenu() { menuOpen.value = !menuOpen.value }
function closeMenu() { menuOpen.value = false }

async function onOneShotToggle() {
  closeMenu()
  try {
    if (store.mode === 'one-shot') {
      await exitOneShot()
    } else {
      await enterOneShot()
    }
    router.replace(store.hasKeys ? '/keys' : '/')
  } catch (e) {
    alert(`Could not change mode: ${e}`)
  }
}

function back() {
  router.back()
}

function onDocClick(evt) {
  if (!menuOpen.value) return
  if (!evt.target.closest('.kebab-menu, .kebab-btn')) closeMenu()
}
onMounted(() => document.addEventListener('click', onDocClick))
onUnmounted(() => document.removeEventListener('click', onDocClick))
</script>

<template>
  <div class="mobile-layout">
    <OneShotBanner />
    <header class="app-bar">
      <button
        v-if="canGoBack"
        class="back-btn"
        aria-label="Back"
        @click="back"
      >
        &#x2190;
      </button>
      <h1 class="title">{{ title }}</h1>
      <div class="kebab-wrap">
        <button
          class="kebab-btn"
          aria-label="More"
          :aria-expanded="menuOpen"
          @click.stop="toggleMenu"
        >
          &#x22EE;
        </button>
        <div v-if="menuOpen" class="kebab-menu" role="menu">
          <button class="menu-item" role="menuitem" @click="onOneShotToggle">
            {{ store.mode === 'one-shot' ? 'Exit One Shot' : 'One Shot mode' }}
          </button>
        </div>
      </div>
    </header>
    <main class="content">
      <slot />
    </main>
    <nav v-if="showTabs" class="tab-bar" role="tablist">
      <button
        class="tab"
        :class="{ active: activeTab === 'keys' }"
        role="tab"
        :aria-selected="activeTab === 'keys'"
        @click="router.push('/keys')"
      >
        <span class="tab-icon" aria-hidden="true">&#x1F511;</span>
        <span class="tab-label">Keys</span>
      </button>
      <button
        class="tab"
        :class="{ active: activeTab === 'card' }"
        role="tab"
        :aria-selected="activeTab === 'card'"
        @click="router.push('/card')"
      >
        <span class="tab-icon" aria-hidden="true">&#x1F4B3;</span>
        <span class="tab-label">SmartCards</span>
      </button>
    </nav>
  </div>
</template>

<style scoped>
.mobile-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--color-bg);
}

.app-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  /* Extra top padding accounts for iOS/Android status bar when app is
     rendered fullscreen via the Tauri mobile webview. */
  padding-top: calc(12px + env(safe-area-inset-top, 0px));
  background: var(--color-sidebar);
  color: #fff;
  min-height: 56px;
  box-sizing: content-box;
}

.back-btn {
  background: transparent;
  border: none;
  color: #fff;
  font-size: 22px;
  line-height: 1;
  padding: 6px 10px;
  cursor: pointer;
  border-radius: 8px;
  min-width: 44px;
  min-height: 44px;
}

.back-btn:active {
  background: rgba(255, 255, 255, 0.12);
}

.title {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  flex: 1;
}

.kebab-wrap {
  position: relative;
}

.kebab-btn {
  background: transparent;
  border: none;
  color: #fff;
  font-size: 22px;
  line-height: 1;
  padding: 6px 10px;
  cursor: pointer;
  border-radius: 8px;
  min-width: 44px;
  min-height: 44px;
}

.kebab-btn:active {
  background: rgba(255, 255, 255, 0.12);
}

.kebab-menu {
  position: absolute;
  top: calc(100% + 4px);
  right: 4px;
  min-width: 160px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
  padding: 4px 0;
  z-index: 10;
}

.menu-item {
  width: 100%;
  display: block;
  padding: 12px 14px;
  background: transparent;
  border: none;
  text-align: left;
  font-size: 14px;
  font-family: var(--font-family);
  color: var(--color-text);
  cursor: pointer;
}

.menu-item:active { background: var(--color-bg-light); }

.content {
  flex: 1;
  overflow-y: auto;
}

.tab-bar {
  display: flex;
  border-top: 1px solid var(--color-border);
  background: var(--color-bg);
  padding-bottom: env(safe-area-inset-bottom, 0px);
}

.tab {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  min-height: 56px;
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  font-family: var(--font-family);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}

.tab:active { background: var(--color-bg-light); }

.tab.active {
  color: var(--color-sidebar);
}

.tab-icon {
  font-size: 20px;
  line-height: 1;
}

.tab-label {
  font-size: 12px;
}
</style>
