<script setup>
import {onMounted, ref} from 'vue'

const servers = ref([
  { id: 1, name: 'Survival Hub', address: 'hub.beacon.local', status: 'online', cpu: 42, ram: 2.4, version: '1.21.1', difficulty: 'Normal', type: 'Spigot', players: [{ name: 'herbydevs', health: 20, xp: 85, level: 42 }] },
  { id: 2, name: 'Creative Test', address: 'dev.beacon.local', status: 'offline', cpu: 0, ram: 0, version: '1.20.1', difficulty: 'Peaceful', type: 'Vanilla', players: [] },
])

const platforms = [
  { id: 'vanilla', name: 'Vanilla', icon: 'https://minecraft.wiki/images/thumb/Crafting_Table_JE4_BE3.png/120px-Crafting_Table_JE4_BE3.png?5767f' },
  { id: 'paper', name: 'Paper', icon: 'https://assets.papermc.io/brand/papermc_logo.512.png' },
  { id: 'forge', name: 'Forge', icon: 'https://avatars.githubusercontent.com/u/1390178?s=48&v=4' },
  { id: 'fabric', name: 'Fabric', icon: 'https://fabricmc.net/assets/logo.png' }
]

const isCreating = ref(false)
const currentView = ref('grid')
const selectedServer = ref(null)
const selectedPlayer = ref(null)

// const servers = ref([])
const isLoading = ref(true)
const error = ref(null)


const fetchServers = async () => {
  try {
    isLoading.value = true
    const response = await fetch('http://api.beacon.local/api/v1/servers/get')
    if (!response.ok) throw Error('Cluster unreachable')
    servers.value = response.json()

  }catch (error) {
    console.log(error)

  } finally {
    isLoading.value = false
  }
}

// onMounted(() => {
//   fetchServers()
// })


const newServer = ref({
  name: '',
  motd: 'A Project Beacon Server',
  version: '1.21.1',
  type: 'Vanilla',
  difficulty: 'Normal'
})



const deployServer = () => {
  if (!newServer.value.name) return
  servers.value.push({
    id: Date.now(),
    ...newServer.value,
    address: `${newServer.value.name.toLowerCase().replace(/\s+/g, '-')}.beacon.local`,
    status: 'offline',
    cpu: 0,
    ram: 0,
    players: []
  })
  isCreating.value = false
  newServer.value = { name: '', motd: 'A Project Beacon Server', version: '1.21.1', type: 'Vanilla', difficulty: 'Normal' }
}

const toggleStatus = (server) => {
  server.status = server.status === 'online' ? 'offline' : 'online'
  server.cpu = server.status === 'online' ? Math.floor(Math.random() * 40) + 10 : 0
}

const openServerStats = (server) => {
  selectedServer.value = server
  currentView.value = 'stats'
}

const closeStats = () => {
  currentView.value = 'grid'
  selectedServer.value = null
}
</script>

<template>
  <div class="dashboard-container">

    <div v-if="currentView === 'grid'" class="view-layer">
      <header class="dashboard-header">
        <div class="title-section">
          <h1>Cloud Instances</h1>
          <p>Project Beacon / <span>SVG-North Cluster</span></p>
        </div>
        <button class="primary-btn sparkle-hover" @click="isCreating = true">
          <span class="plus">✦</span> New Instance
        </button>
      </header>

      <div v-if="isLoading" class="status-msg">Synchronizing Cluster Data...</div>
      <div v-else-if="error" class="status-msg error">{{ error }}</div>

      <div v-else class="server-grid">
        <div v-for="server in servers" :key="server.id" class="card server-card" :class="server.status" @click="openServerStats(server)">

        </div>

        <div class="card create-card" @click="isCreating = true">
          <div class="plus-icon">✦</div>
          <div class="create-text">
            <h3>Deploy Instance</h3>
            <p>Initialize a new node</p>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.dashboard-container { padding: 40px; max-width: 1400px; margin: 0 auto; color: #1d1d1f; }
.dashboard-header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 40px; }
h1 { font-size: 2.5rem; font-weight: 900; letter-spacing: -1.5px; margin: 0; }
.title-section p { color: #86868b; margin-top: 4px; }
.title-section span { color: #0071e3; font-weight: 700; }

.server-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(360px, 1fr)); gap: 24px; }

.card {
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 30px;
  padding: 30px;
  transition: all 0.4s cubic-bezier(0.165, 0.84, 0.44, 1);
  cursor: pointer;
}
.card:hover { transform: translateY(-8px); box-shadow: 0 30px 60px rgba(0,0,0,0.06); }

/* Card Header - Version & Status Fixes */
.card-head { display: flex; justify-content: space-between; align-items: center; }
.status-pill {
  display: flex; align-items: center; gap: 8px; font-size: 0.75rem; font-weight: 800;
  text-transform: uppercase; background: #f5f5f7; padding: 6px 14px; border-radius: 20px;
}
.online .status-pill { color: #166534; background: #dcfce7; }
.offline .status-pill { color: #1d1d1f; background: #f5f5f7; }

.dot { width: 8px; height: 8px; border-radius: 50%; background: #86868b; transition: 0.3s; }
.online .dot { background: #22c55e; box-shadow: 0 0 10px #22c55e; }

.badge-group { display: flex; gap: 8px; }
.version-badge, .type-badge {
  font-size: 0.65rem; font-weight: 800; color: #86868b; background: #f5f5f7;
  padding: 4px 10px; border-radius: 8px; white-space: nowrap;
}

.server-info h3 { font-size: 1.4rem; font-weight: 800; margin: 20px 0 5px; }
.server-info code { color: #0071e3; font-weight: 600; font-size: 0.85rem; }

.metrics { margin: 25px 0; }
.metric-labels { display: flex; justify-content: space-between; font-size: 0.7rem; font-weight: 700; color: #86868b; margin-bottom: 8px; }

.progress-bg { height: 6px; background: #f5f5f7; border-radius: 10px; overflow: hidden; }
.progress-fill { height: 100%; background: #0071e3; transition: width 0.8s ease; }
.progress-fill.xp { background: #32d74b; box-shadow: 0 0 10px rgba(50, 215, 75, 0.3); }

.card-actions { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-top: 25px; }
.btn-action { border: none; padding: 12px; border-radius: 14px; font-weight: 800; cursor: pointer; transition: 0.2s; }
.btn-action.offline { background: #0071e3; color: white; }
.btn-action.online { background: #ff3b30; color: white; }
.btn-secondary { background: #f5f5f7; border: none; font-weight: 700; border-radius: 14px; cursor: pointer; }

/* Stats View & Modals */
.stats-layout { display: grid; grid-template-columns: 2fr 1fr; gap: 24px; }
.terminal-box { background: #1d1d1f; border-radius: 18px; padding: 20px; height: 300px; overflow-y: auto; color: #32d74b; font-family: 'JetBrains Mono', monospace; font-size: 0.85rem; }
.player-list { display: flex; flex-direction: column; gap: 10px; }
.player-row { display: flex; align-items: center; gap: 12px; padding: 12px; background: #f5f5f7; border-radius: 16px; cursor: pointer; }
.player-row:hover { background: #fff; transform: translateX(5px); box-shadow: 0 5px 15px rgba(0,0,0,0.05); }

.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.1); backdrop-filter: blur(20px); display: flex; align-items: center; justify-content: center; z-index: 1000; }
.modal-glass { background: #fff; padding: 40px; border-radius: 40px; box-shadow: 0 40px 100px rgba(0,0,0,0.1); }
.section-label { display: block; font-size: 0.7rem; font-weight: 900; color: #86868b; text-transform: uppercase; margin-bottom: 12px; }
.primary-btn { background: #0071e3; color: white; border: none; padding: 14px 28px; border-radius: 16px; font-weight: 800; cursor: pointer; }

/* Form Elements */
.input-stack { background: #f5f5f7; border-radius: 24px; padding: 8px; margin: 25px 0; }
.fancy-input { width: 100%; border: none; background: transparent; padding: 12px 16px; outline: none; }
.platform-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; margin-bottom: 25px; }
.platform-option { background: #f5f5f7; padding: 15px 10px; border-radius: 18px; text-align: center; cursor: pointer; border: 2px solid transparent; }
.platform-option.active { border-color: #0071e3; background: #fff; }
.platform-img { width: 30px; height: 30px; object-fit: contain; margin-bottom: 8px; filter: grayscale(1); opacity: 0.5; }
.active .platform-img { filter: grayscale(0); opacity: 1; }

.pop-enter-active { transition: all 0.4s cubic-bezier(0.165, 0.84, 0.44, 1); }
.pop-enter-from { opacity: 0; transform: scale(0.9) translateY(20px); }

.status-msg {
  padding: 40px;
  text-align: center;
  background: rgba(255, 255, 255, 0.5);
  border-radius: 20px;
  font-weight: 700;
  color: #86868b;
}
.status-msg.error {
  color: #ff3b30;
  border: 1px solid rgba(255, 59, 48, 0.2);
}
</style>