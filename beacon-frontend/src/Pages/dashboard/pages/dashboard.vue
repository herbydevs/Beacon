<template>
  <div class="dashboard">
    <div class="top-row">
      <h1>Cloud Console</h1>
      <button class="pulse-btn">PROVISION NEW NODE</button>
    </div>

    <div class="grid">
      <div class="server-card" v-for="i in 3" :key="i">
        <div class="card-glow"></div>
        <div class="card-content">
          <div class="status-badge">
            <span class="pulse-dot"></span> Online
          </div>
          <h3>survival-prod-{{i}}</h3>

          <div class="usage-bars">
            <div class="bar-group">
              <label>CPU Usage</label>
              <div class="track"><div class="fill blue" :style="{width: '65%'}"></div></div>
            </div>
          </div>

          <div class="footer-actions">
            <button class="glass-btn">Console</button>
            <button class="glass-btn">Restart</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard { padding: 40px; }

h1 { font-size: 2rem; font-weight: 800; color: #1d1d1f; margin-bottom: 30px; }

.pulse-btn {
  background: #0071e3;
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 14px;
  font-weight: 700;
  cursor: pointer;
  box-shadow: 0 0 0 0 rgba(0, 113, 227, 0.4);
  animation: pulse-shadow 2s infinite;
}

@keyframes pulse-shadow {
  0% { box-shadow: 0 0 0 0 rgba(0, 113, 227, 0.7); }
  70% { box-shadow: 0 0 0 15px rgba(0, 113, 227, 0); }
  100% { box-shadow: 0 0 0 0 rgba(0, 113, 227, 0); }
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 25px;
}

.server-card {
  position: relative;
  background: rgba(255, 255, 255, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.8);
  border-radius: 20px;
  padding: 24px;
  overflow: hidden;
  transition: all 0.4s ease;
}

.server-card:hover {
  transform: translateY(-10px) scale(1.02);
  background: #fff;
  box-shadow: 0 20px 40px rgba(0,0,0,0.05);
}

.pulse-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  background: #34c759;
  border-radius: 50%;
  margin-right: 8px;
  animation: blink 1.5s infinite;
}

@keyframes blink {
  50% { opacity: 0.3; transform: scale(0.8); }
}

.track {
  height: 6px;
  background: #f2f2f7;
  border-radius: 10px;
  margin-top: 8px;
  overflow: hidden;
}

.fill.blue {
  background: linear-gradient(90deg, #0071e3, #52e5ff);
  height: 100%;
  border-radius: 10px;
}

.glass-btn {
  background: rgba(0, 113, 227, 0.05);
  border: 1px solid rgba(0, 113, 227, 0.1);
  color: #0071e3;
  padding: 8px 16px;
  border-radius: 10px;
  font-weight: 600;
  cursor: pointer;
  margin-top: 20px;
  margin-right: 10px;
}
</style>