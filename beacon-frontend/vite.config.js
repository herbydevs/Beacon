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
    host: true, // Listen on all network interfaces
    port: 5173,
    allowedHosts: [
      'beacon.local',
      'api.beacon.local',
      'localhost',
        "beacon_frontend_dev"
    ],
    hmr: {
      clientPort: 80, // Force HMR to communicate through Nginx
      host: 'app.beacon.local'
    }
  }
})
