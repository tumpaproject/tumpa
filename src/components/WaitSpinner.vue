<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

import s05 from '@/assets/stickers/snowden_05.webp'
import s06 from '@/assets/stickers/snowden_06.webp'
import s19 from '@/assets/stickers/snowden_19.webp'
import s24 from '@/assets/stickers/snowden_24.webp'
import tor32 from '@/assets/stickers/tor_32.webp'

defineProps({
  message: { type: String, default: 'Please wait for the operation to finish...' },
  hint: { type: String, default: '' },
})

const stickers = [s05, s06, s19, s24, tor32]
const currentSticker = ref(stickers[Math.floor(Math.random() * stickers.length)])
let intervalId = null

onMounted(() => {
  intervalId = setInterval(() => {
    currentSticker.value = stickers[Math.floor(Math.random() * stickers.length)]
  }, 500)
})

onUnmounted(() => {
  if (intervalId) clearInterval(intervalId)
})
</script>

<template>
  <div class="wait-spinner">
    <img :src="currentSticker" alt="" class="sticker" />
    <p class="message">{{ message }}</p>
    <p v-if="hint" class="hint">{{ hint }}</p>
  </div>
</template>

<style scoped>
.wait-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 24px;
}

.sticker {
  width: 128px;
  height: 128px;
  object-fit: contain;
}

.message {
  font-size: 14px;
  color: var(--color-text-muted);
}

.hint {
  font-size: 12px;
  color: var(--color-text-muted);
}
</style>
