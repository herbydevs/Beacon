import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

export default defineConfig(({ command }) => {
  return {
    plugins: [
      vue(),
      // Only enable devtools in development mode
      command === 'serve' ? vueDevTools() : [],
    ],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url))
      },
    },
    // This entire block now only applies during 'npm run dev'
    server: command === 'serve' ? {
      host: true,
      port: 5173,
      allowedHosts: [
        'beacon.local',
        'api.beacon.local',
        'localhost',
        "beacon_frontend_dev"
      ],
      hmr: {
        clientPort: 80,
        host: 'app.beacon.local'
      }
    } : {}
  }
})