<template>
  <aside class="sidebar">
    <div class="brand">BEACON</div>

    <nav class="nav">
      <router-link
          v-for="item in menu"
          :key="item.name"
          :to="item.path"
          class="nav-link"
      >
        <span class="dot"></span>
        {{ item.name }}
      </router-link>
    </nav>
  </aside>
</template>

<script setup>
// You need this script block to define the data for the v-for loop
const menu = [
  { name: 'Dashboard', path: '/dashboard' },
  { name: 'Instances', path: '/servers' },
  { name: 'Networking', path: '/network' },
  { name: 'Vault', path: '/settings' }
]
</script>

<style scoped>
/* Styles remain the same as your previous working version */
.sidebar { width: 220px; padding: 40px 20px; z-index: 2; }

.brand {
  font-weight: 900;
  font-size: 1.2rem;
  background: linear-gradient(90deg, #0071e3, #52e5ff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin-bottom: 50px;
  padding-left: 15px;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 15px;
  color: #6e6e73;
  text-decoration: none;
  font-weight: 600;
  border-radius: 12px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.nav-link:hover { color: #1d1d1f; transform: translateX(5px); }

.router-link-active {
  background: #fff;
  color: #0071e3 !important;
  box-shadow: 0 4px 15px rgba(0, 113, 227, 0.1);
}

.router-link-active .dot {
  width: 6px;
  height: 6px;
  background: #0071e3;
  border-radius: 50%;
  box-shadow: 0 0 8px #0071e3;
}
</style>