<script setup>
import { computed } from 'vue'
import SidebarLayout from '@/components/SidebarLayout.vue'
import MobileLayout from '@/layouts/MobileLayout.vue'
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()
const Layout = computed(() => (store.isMobile ? MobileLayout : SidebarLayout))
</script>

<template>
  <component :is="Layout">
    <router-view />
  </component>
</template>
