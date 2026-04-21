<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'

// Defer to requestIdleCallback when available (real browsers), else
// fall back to a short-delay setTimeout. queueMicrotask isn't enough —
// we want the first paint to commit before we fire card IPC.
const defer = typeof requestIdleCallback === 'function'
  ? (fn) => requestIdleCallback(fn, { timeout: 400 })
  : (fn) => setTimeout(fn, 50)
import logoSvg from '@/assets/icons/logo.svg'
import keyIconSvg from '@/assets/icons/key_icon.svg'
import usbkeySvg from '@/assets/icons/usbkey.svg'
import cardStatusSvg from '@/assets/icons/card_status.svg'
import OneShotBanner from '@/components/OneShotBanner.vue'
import { enterOneShot, exitOneShot } from '@/utils/oneShot'

const router = useRouter()
const store = useAppStore()
const smartCardOpen = ref(true)

// Key loading + initial redirect are owned by the route views
// (StartView / KeyListView) so the sidebar shell doesn't block the
// first paint on a Rust IPC. Card detection is deferred past first
// paint so the window renders before we touch PCSC.
onMounted(() => {
  defer(async () => {
    await store.checkCard()
    store.startCardPolling()
  })
})

onUnmounted(() => {
  store.stopCardPolling()
})

function goToKeys() {
  store.setActiveSection('keys')
  if (store.hasKeys) {
    router.push('/keys')
  } else {
    router.push('/')
  }
}

async function goToCard(subItem = '') {
  const connected = await store.checkCard()
  if (!connected) {
    alert('Can not access any Yubikey!')
    return
  }
  store.setActiveSection('card', subItem || 'card-details')
  if (!subItem || subItem === 'card-details') {
    await store.fetchCardDetails()
    router.push('/card')
  } else {
    router.push(`/card/${subItem}`)
  }
}

async function onOneShotToggle() {
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
</script>

<template>
  <div class="shell">
    <OneShotBanner />
    <div class="layout">
      <a href="#main-content" class="skip-link">Skip to content</a>
      <aside class="sidebar" aria-label="Sidebar">
      <div class="sidebar-logo">
        <img :src="logoSvg" alt="Tumpa" />
      </div>

      <nav class="sidebar-nav" aria-label="Main navigation">
        <button
          class="nav-item nav-item--icon"
          :class="{ active: store.activeSection === 'keys' }"
          @click="goToKeys"
        >
          <img :src="keyIconSvg" alt="" class="nav-icon" />
          <span>Key Management</span>
        </button>

        <div class="nav-spacer"></div>

        <div class="nav-item-group">
          <button
            class="nav-item nav-item--icon nav-item--grow"
            :class="{ active: store.activeSection === 'card' && !store.activeSubItem }"
            @click="goToCard()"
          >
            <img :src="usbkeySvg" alt="" class="nav-icon" />
            <span>Smart Card</span>
          </button>
          <button
            type="button"
            class="nav-chevron"
            :class="{ open: smartCardOpen }"
            :aria-expanded="smartCardOpen"
            aria-label="Toggle Smart Card submenu"
            @click="smartCardOpen = !smartCardOpen"
          >&#9662;</button>
        </div>

        <template v-if="smartCardOpen">
          <button
            class="nav-item nav-item--sub"
            :class="{ active: store.activeSubItem === 'card-details' }"
            @click="goToCard('card-details')"
          >Card details</button>
          <button
            class="nav-item nav-item--sub"
            :class="{ active: store.activeSubItem === 'edit-name' }"
            @click="goToCard('edit-name')"
          >Edit name</button>
          <button
            class="nav-item nav-item--sub"
            :class="{ active: store.activeSubItem === 'edit-url' }"
            @click="goToCard('edit-url')"
          >Edit Public URL</button>
          <button
            class="nav-item nav-item--sub"
            :class="{ active: store.activeSubItem === 'change-user-pin' }"
            @click="goToCard('change-user-pin')"
          >Change User Pin</button>
          <button
            class="nav-item nav-item--sub"
            :class="{ active: store.activeSubItem === 'change-admin-pin' }"
            @click="goToCard('change-admin-pin')"
          >Change Admin Pin</button>
          <button
            class="nav-item nav-item--sub"
            :class="{ active: store.activeSubItem === 'touch-mode' }"
            @click="goToCard('touch-mode')"
          >Touch Mode</button>
        </template>

        <div class="nav-spacer"></div>

        <button
          type="button"
          class="nav-item nav-item--icon one-shot-toggle"
          :class="{ 'one-shot-active': store.mode === 'one-shot' }"
          @click="onOneShotToggle"
        >
          <span class="nav-icon" aria-hidden="true">&#9888;</span>
          <span>{{ store.mode === 'one-shot' ? 'Exit One Shot' : 'One Shot mode' }}</span>
        </button>
      </nav>

      <div class="sidebar-status" v-if="store.cardConnected">
        <img :src="cardStatusSvg" alt="" class="status-icon" />
        <span>Card detected</span>
      </div>
    </aside>

    <main id="main-content" class="content">
      <slot />
    </main>
    </div>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
}

.layout {
  display: flex;
  flex: 1;
  min-height: 0;
}

.one-shot-toggle {
  color: #d24545;
  margin-top: auto;
}

.one-shot-toggle.one-shot-active {
  color: #ff6b6b;
  font-weight: 700;
}

.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  background: var(--color-sidebar);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

.sidebar-logo {
  padding: 14px;
}

.sidebar-logo img {
  height: 40px;
}

.sidebar-nav {
  padding: 0 14px 0 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.nav-spacer {
  height: 10px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  height: 40px;
  padding: 0 8px;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  font-family: var(--font-family);
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.nav-item.active {
  background: var(--color-sidebar-active);
}

.nav-item--sub {
  padding-left: 40px;
  color: var(--color-sidebar-submenu-text);
}

.nav-item-group {
  display: flex;
  align-items: center;
  gap: 0;
}

.nav-item--grow {
  flex: 1;
}

.nav-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.nav-chevron {
  margin-left: 0;
  font-size: 12px;
  transition: transform 0.2s;
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  padding: 4px;
  line-height: 1;
  min-width: 24px;
  min-height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-chevron.open {
  transform: rotate(0deg);
}

.sidebar-status {
  padding: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--color-sidebar-submenu-text);
  font-size: 12px;
}

.status-icon {
  width: 20px;
  height: 20px;
}

.content {
  flex: 1;
  background: var(--color-bg);
  overflow-y: auto;
}
</style>
