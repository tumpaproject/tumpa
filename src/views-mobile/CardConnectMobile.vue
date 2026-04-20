<script setup>
// Overlay shown while a mobile card operation is in flight.
//
// On iOS, Core NFC renders its own system prompt; our overlay sits
// behind it and really only takes over once Core NFC dismisses. On
// Android, we draw the entire prompt ourselves because the platform
// doesn't provide one.
//
// The `phase` prop drives the copy:
// - `waiting`   — no tag detected yet; prompt the user to tap.
// - `connected` — SELECT succeeded; APDUs are flowing. Tell the user
//                 to keep the card pressed.
//
// The parent view (UploadToCardMobile, etc.) owns the phase — it
// toggles to `connected` when it receives the
// `plugin:tumpa-card:card-connected` event from the plugin.

const props = defineProps({
  phase: {
    type: String,
    default: 'waiting',
    validator: (v) => v === 'waiting' || v === 'connected',
  },
  // Verb describing what's happening (for the `connected` phase
  // subtitle). Upload flows pass 'uploading', expiry flows 'updating',
  // PIN changes 'changing PIN', etc.
  action: { type: String, default: 'working' },
  error: { type: String, default: '' },
})
defineEmits(['cancel'])
</script>

<template>
  <div class="overlay" role="dialog" aria-modal="true" aria-labelledby="cc-title">
    <div class="card">
      <div class="pulse" aria-hidden="true">
        <div class="pulse-dot" :class="{ active: phase === 'connected' }"></div>
        <div v-if="phase === 'waiting'" class="pulse-ring"></div>
      </div>
      <h2 id="cc-title" class="title">
        <template v-if="phase === 'waiting'">Hold your security key to the phone</template>
        <template v-else>Card found — {{ action }}…</template>
      </h2>
      <p class="subtitle">
        <template v-if="phase === 'waiting'">
          Hold the back of the phone against the card, or plug it in via USB-C.
        </template>
        <template v-else>
          Keep the card pressed against the phone. Do not remove until this
          screen disappears.
        </template>
      </p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <!-- Always rendered. Cancel is the only exit when the native
           session is waiting and the user wants to back out (no tap
           coming, wrong card, plugged in the wrong reader, …). -->
      <button class="cancel" @click="$emit('cancel')">Cancel</button>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.55);
  z-index: 100;
  padding: 24px;
  padding-bottom: calc(24px + env(safe-area-inset-bottom, 0px));
}

.card {
  background: var(--color-bg);
  border-radius: 16px;
  padding: 28px 24px;
  width: 100%;
  max-width: 360px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.25);
}

.pulse {
  position: relative;
  width: 80px;
  height: 80px;
  margin: 6px 0 10px;
}

.pulse-dot {
  position: absolute;
  inset: 25%;
  border-radius: 50%;
  background: var(--color-sidebar);
  transition: background 0.25s ease;
}

.pulse-dot.active {
  background: var(--color-green);
  animation: breathe 1.4s ease-in-out infinite;
}

.pulse-ring {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  border: 3px solid var(--color-sidebar);
  opacity: 0.6;
  animation: pulse-ring 1.6s ease-out infinite;
}

@keyframes pulse-ring {
  0% { transform: scale(0.6); opacity: 0.8; }
  100% { transform: scale(1.15); opacity: 0; }
}

@keyframes breathe {
  0%, 100% { transform: scale(1); }
  50%      { transform: scale(1.12); }
}

.title {
  font-size: 18px;
  font-weight: 700;
  margin: 0;
}

.subtitle {
  font-size: 14px;
  color: var(--color-text-muted);
  margin: 0;
  line-height: 1.4;
}

.error {
  font-size: 13px;
  color: var(--color-red-text);
  background: var(--color-expired-bg);
  border: 1px solid var(--color-expired-border);
  border-radius: 8px;
  padding: 8px 10px;
  margin: 4px 0 0;
  word-break: break-word;
}

.cancel {
  min-height: 44px;
  min-width: 140px;
  padding: 0 16px;
  border-radius: 10px;
  border: 1px solid var(--color-border-input);
  background: #fff;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-family);
  margin-top: 6px;
}

.cancel:active { background: var(--color-bg-light); }
</style>
