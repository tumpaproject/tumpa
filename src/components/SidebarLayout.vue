<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'
import logoSvg from '@/assets/icons/logo.svg'
import keyIconSvg from '@/assets/icons/key_icon.svg'
import usbkeySvg from '@/assets/icons/usbkey.svg'
import cardStatusSvg from '@/assets/icons/card_status.svg'

const router = useRouter()
const store = useAppStore()
const smartCardOpen = ref(true)

onMounted(async () => {
  await store.refreshKeys()
  if (store.hasKeys) {
    router.replace('/keys')
  }
  await store.checkCard()
  store.startCardPolling()
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
</script>

<template>
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
      </nav>

      <div class="sidebar-status" v-if="store.cardConnected">
        <img :src="cardStatusSvg" alt="" class="status-icon" />
        <span>Card detected</span>
      </div>
    </aside>

    <main id="main-content" class="content">
      <h1 class="visually-hidden">Tumpa - OpenPGP Key Manager</h1>
      <slot />
    </main>
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  width: 100vw;
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
