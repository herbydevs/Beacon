import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAuthStore = defineStore('auth', () => {
    // State
    const token = ref(localStorage.getItem('beacon_token') || null)
    const user = ref(JSON.parse(localStorage.getItem('beacon_user')) || null)
    const isLoading = ref(false)

    // Getters
    const isAuthenticated = computed(() => !!token.value)
    const userRole = computed(() => user.value?.role || 'Guest')

    // Actions
    async function login(credentials) {
        isLoading.value = true
        try {
            // Mocking the response from your Rust (Axum) / Keycloak backend
            const mockResponse = {
                token: 'bk_dev_session_' + Math.random().toString(36).substr(2),
                userData: {
                    id: 'u1',
                    username: 'herbydevs',
                    role: 'System Architect',
                    region: 'SVG-North'
                }
            }

            token.value = mockResponse.token
            user.value = mockResponse.userData

            localStorage.setItem('beacon_token', token.value)
            localStorage.setItem('beacon_user', JSON.stringify(user.value))

            return true
        } catch (error) {
            console.error('Auth Error:', error)
            throw error
        } finally {
            isLoading.value = false
        }
    }

    function logout() {
        token.value = null
        user.value = null
        localStorage.removeItem('beacon_token')
        localStorage.removeItem('beacon_user')
        // Reset UserDataStore here if needed
    }

    return { token, user, isAuthenticated, userRole, isLoading, login, logout }
})