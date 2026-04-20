<script setup>
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'

const route = useRoute()
const router = useRouter()

const title = computed(() => route.meta?.title || 'Tumpa')
const canGoBack = computed(() => route.name !== 'start' && route.name !== 'key-list')

function back() {
  router.back()
}
</script>

<template>
  <div class="mobile-layout">
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
    </header>
    <main class="content">
      <slot />
    </main>
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
}

.content {
  flex: 1;
  overflow-y: auto;
  padding-bottom: env(safe-area-inset-bottom, 0px);
}
</style>
