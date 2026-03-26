<script setup>
import Sidebar from "@/UI/sidebar.vue";
</script>
<template>
  <div class="app-shell">
    <div class="aura aura-1"></div>
    <div class="aura aura-2"></div>
    <div class="aura aura-3"></div>

    <Sidebar />
    <main class="viewport">
      <div class="glass-canvas">
        <router-view />
      </div>
    </main>
  </div>
</template>

<style scoped>
.app-shell {
  position: relative;
  display: flex;
  height: 100vh;
  background: #f8fafc; /* Base light gray */
  overflow: hidden;
}

/* The Animated Blobs */
.aura {
  position: absolute;
  width: 800px;
  height: 800px;
  border-radius: 50%;
  filter: blur(120px);
  z-index: 0;
  opacity: 0.5;
  mix-blend-mode: multiply;
  animation: moveAura 25s infinite alternate ease-in-out;
}

.aura-1 {
  background: #dbeafe; /* Soft Blue */
  top: -200px;
  right: -100px;
}

.aura-2 {
  background: #f0f9ff; /* Sky Blue */
  bottom: -300px;
  left: -100px;
  animation-duration: 30s;
  animation-delay: -5s;
}

.aura-3 {
  background: #e0f2fe; /* Light Cyan */
  top: 20%;
  left: 30%;
  width: 600px;
  height: 600px;
  animation-duration: 20s;
  animation-delay: -10s;
}

@keyframes moveAura {
  0% { transform: translate(0, 0) scale(1); }
  33% { transform: translate(100px, 50px) scale(1.1); }
  66% { transform: translate(-50px, 100px) scale(0.9); }
  100% { transform: translate(20px, -20px) scale(1); }
}

.viewport {
  flex: 1;
  padding: 24px;
  z-index: 1; /* Sits above the blobs */
}

.glass-canvas {
  height: 100%;
  background: rgba(255, 255, 255, 0.4); /* Semi-transparent */
  backdrop-filter: blur(40px) saturate(180%); /* This creates the "alive" look */
  border-radius: 28px;
  border: 1px solid rgba(255, 255, 255, 0.7);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.03);
}
</style>