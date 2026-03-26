import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vueDevTools(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
  server: {
    // This allows the specific hostname causing the error
    allowedHosts: ['beacon_frontend_dev','beacon.local'],

    // Optional: If you are running inside Docker,
    // you likely also need this to expose it to your network
    host: true,
    port: 5173,

    // Strict Port ensures it doesn't jump to 5174 if 5173 is busy
    strictPort: true,
  }
})
