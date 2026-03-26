import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUserDataStore = defineStore('userData', () => {
    // State
    const servers = ref([
        { id: 1, name: 'Survival Hub', address: 'hub.beacon.local', status: 'online', cpu: 42, ram: 2.4, version: '1.21.1', difficulty: 'Normal', type: 'Paper' },
        { id: 2, name: 'Creative Test', address: 'dev.beacon.local', status: 'offline', cpu: 0, ram: 0, version: '1.20.1', difficulty: 'Peaceful', type: 'Vanilla' },
    ])

    const clusterStats = ref({
        totalNodes: 2,
        activeRam: 2.4,
        region: 'Saint Vincent'
    })

    // Actions
    function createServer(config) {
        const newServer = {
            id: Date.now(),
            ...config,
            address: `${config.name.toLowerCase().replace(/\s+/g, '-')}.beacon.local`,
            status: 'offline',
            cpu: 0,
            ram: 0
        }
        servers.value.push(newServer)
        // Update local stats
        clusterStats.value.totalNodes = servers.value.length
    }

    function toggleServer(id) {
        const index = servers.value.findIndex(s => s.id === id)
        if (index !== -1) {
            const s = servers.value[index]
            s.status = s.status === 'online' ? 'offline' : 'online'
            s.cpu = s.status === 'online' ? Math.floor(Math.random() * 50) + 10 : 0
        }
    }

    function deleteServer(id) {
        servers.value = servers.value.filter(s => s.id !== id)
    }

    return { servers, clusterStats, createServer, toggleServer, deleteServer }
})