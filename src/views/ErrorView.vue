<script setup>
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/appStore'
import TButton from '@/components/TButton.vue'

const router = useRouter()
const store = useAppStore()

function goBack() {
  store.clearError()
  router.push('/')
}
</script>

<template>
  <div class="error-view">
    <h1>Error</h1>
    <p role="alert">{{ store.errorMessage || 'An unknown error occurred.' }}</p>
    <TButton variant="default" @click="goBack">Go Back</TButton>
  </div>
</template>

<style scoped>
.error-view {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
}

h1 {
  font-size: 20px;
  font-weight: 700;
  color: var(--color-red);
}

p {
  font-size: 14px;
  color: var(--color-text-muted);
  max-width: 400px;
  text-align: center;
}
</style>
