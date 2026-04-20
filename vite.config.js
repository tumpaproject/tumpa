import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  // Tauri sets TAURI_DEV_HOST when running on mobile so the on-device
  // webview can reach the host's Vite dev server.
  clearScreen: false,
  server: {
    host: host || false,
    port: 5173,
    strictPort: true,
    hmr: host
      ? { protocol: 'ws', host, port: 5174 }
      : undefined,
  },
  test: {
    environment: 'happy-dom',
    globals: true,
  },
})
