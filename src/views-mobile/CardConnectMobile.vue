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
  // Transport the parent chose. Drives the waiting-phase copy so the
  // user sees "Hold your card" on NFC or "Plug in your reader" on
  // USB-C — without this, USB flows confused users into thinking the
  // app was stuck trying NFC.
  transport: {
    type: String,
    default: 'nfc',
    validator: (v) => v === 'nfc' || v === 'usb',
  },
  error: { type: String, default: '' },
})
defineEmits(['cancel'])
</script>

<template>
  <div class="overlay" role="dialog" aria-modal="true" aria-labelledby="cc-title">
    <div class="card" :class="{ 'phase-connected': phase === 'connected' }">
      <!-- Two-phase indicator. In 'waiting' the outer ring pulses to
           prompt a tap; in 'connected' a check mark and a rotating
           progress arc make it obvious the flow has moved on and the
           native side is driving APDUs. -->
      <div class="pulse" aria-hidden="true">
        <div class="pulse-dot" :class="{ active: phase === 'connected' }">
          <span v-if="phase === 'connected'" class="check">&#x2713;</span>
        </div>
        <div v-if="phase === 'waiting'" class="pulse-ring"></div>
        <div v-else class="spinner-ring"></div>
      </div>
      <h2 id="cc-title" class="title">
        <template v-if="phase === 'waiting' && transport === 'usb'">
          Plug your smartcard reader in
        </template>
        <template v-else-if="phase === 'waiting'">
          Hold your security key to the phone
        </template>
        <template v-else>
          Card connected — {{ action }}<span class="ellipsis" aria-hidden="true"></span>
        </template>
      </h2>
      <p class="subtitle">
        <template v-if="phase === 'waiting' && transport === 'usb'">
          Connect your YubiKey or other CCID reader to the phone's USB-C
          port and accept the permission prompt.
        </template>
        <template v-else-if="phase === 'waiting'">
          Hold the back of the phone against the card.
        </template>
        <template v-else>
          Keep the card in place. This can take 10–30 seconds for a full
          upload — do not remove until this screen disappears.
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
  transition: transform 0.3s ease;
}

/* A small scale-up the moment we flip from "waiting" to "connected",
   so the transition is unmistakable even on a quick USB session where
   the card is found a split-second after Upload is tapped. */
.card.phase-connected {
  animation: found-pop 0.45s ease-out;
}

@keyframes found-pop {
  0%   { transform: scale(0.96); }
  55%  { transform: scale(1.04); }
  100% { transform: scale(1); }
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
  display: flex;
  align-items: center;
  justify-content: center;
  color: #0a2e1c;
  font-size: 22px;
  font-weight: 700;
  line-height: 1;
}

.pulse-dot.active {
  background: var(--color-green);
  animation: breathe 1.4s ease-in-out infinite;
}

.check {
  /* Check mark only exists during connected phase. Pops in briefly
     then sits there while the spinner rotates around it. */
  animation: check-in 0.4s ease-out;
}

.pulse-ring {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  border: 3px solid var(--color-sidebar);
  opacity: 0.6;
  animation: pulse-ring 1.6s ease-out infinite;
}

/* Rotating arc around the check mark — indicates the card is live
   and APDUs are in flight. */
.spinner-ring {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  border: 3px solid rgba(0, 0, 0, 0.08);
  border-top-color: var(--color-green);
  animation: spin 1s linear infinite;
}

@keyframes pulse-ring {
  0% { transform: scale(0.6); opacity: 0.8; }
  100% { transform: scale(1.15); opacity: 0; }
}

@keyframes breathe {
  0%, 100% { transform: scale(1); }
  50%      { transform: scale(1.12); }
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes check-in {
  0%   { transform: scale(0); opacity: 0; }
  60%  { transform: scale(1.2); opacity: 1; }
  100% { transform: scale(1); opacity: 1; }
}

/* Animated ellipsis after "uploading" so the text feels alive even
   if a single APDU happens to take several seconds (RSA key import
   on YubiKey can pause for ~5s). */
.ellipsis::after {
  content: '';
  display: inline-block;
  width: 1.2em;
  text-align: left;
  animation: ellipsis 1.2s steps(4, end) infinite;
}

@keyframes ellipsis {
  0%   { content: ''; }
  25%  { content: '.'; }
  50%  { content: '..'; }
  75%  { content: '...'; }
  100% { content: ''; }
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
