<script setup>
// A persistent banner shown at the top of every screen while the app
// is running in One Shot mode. Its only purpose is to keep the user
// aware that nothing they do in this session will be saved to disk —
// if this is hidden or missed, the mode stops being self-evident.
import { useAppStore } from '@/stores/appStore'
import { exitOneShot } from '@/utils/oneShot'

const store = useAppStore()

async function onExit() {
  try {
    await exitOneShot()
  } catch (e) {
    alert(`Could not exit One Shot: ${e}`)
  }
}
</script>

<template>
  <div v-if="store.mode === 'one-shot'" class="one-shot-banner" role="status">
    <span class="dot" aria-hidden="true"></span>
    <span class="text">
      <strong>One Shot mode</strong> — keys live only in memory and
      will disappear when you close Tumpa.
    </span>
    <button class="exit" @click="onExit">Exit One Shot</button>
  </div>
</template>

<style scoped>
.one-shot-banner {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: #7a1f1f;
  color: #fff;
  font-size: 13px;
  line-height: 1.3;
  padding-top: calc(8px + env(safe-area-inset-top, 0px));
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #ff5a5a;
  box-shadow: 0 0 0 0 rgba(255, 90, 90, 0.7);
  animation: pulse 1.6s ease-out infinite;
  flex-shrink: 0;
}

@keyframes pulse {
  0%   { box-shadow: 0 0 0 0   rgba(255, 90, 90, 0.7); }
  70%  { box-shadow: 0 0 0 10px rgba(255, 90, 90, 0); }
  100% { box-shadow: 0 0 0 0   rgba(255, 90, 90, 0); }
}

.text { flex: 1; }
.text strong { font-weight: 700; }

.exit {
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.6);
  color: #fff;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-family);
  flex-shrink: 0;
}

.exit:active { background: rgba(255, 255, 255, 0.15); }
</style>
