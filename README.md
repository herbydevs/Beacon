# 🗼 Beacon
### A high-performance, container-native Minecraft orchestrator.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/backend-Rust-orange.svg)](https://www.rust-lang.org/)
[![Vue 3](https://img.shields.io/badge/frontend-Vue%203-42b883.svg)](https://vuejs.org/)
[![Docker](https://img.shields.io/badge/platform-Docker-blue.svg)](https://www.docker.com/)

**Beacon** is a lightweight management suite designed to simplify the deployment and scaling of Minecraft servers. Built with a **Rust (Axum)** control plane and a **Vue 3** dashboard, Beacon provides a "blazingly fast" interface to spawn, monitor, and back up containerized game instances with near-zero host overhead.

---

## ✨ Features

* ⚡ **One-Click Deployment:** Spin up Vanilla, Paper, or Forge servers in seconds.
* 📈 **Real-time Monitoring:** Live CPU/RAM stats and console streaming via WebSockets.
* 📦 **Container-First:** Every server runs in an isolated Docker environment for maximum security.
* 💾 **Automated Backups:** Integrated snapshot system to keep your worlds safe.
* 🛠️ **Developer-Friendly API:** A fully documented REST API for custom integrations.
* 🔐 **Enterprise SSO:** Identity management powered by Keycloak.

---

## 🏗️ Architecture

Beacon uses a reverse-proxy model to manage internal services and provide a seamless Single Sign-On (SSO) experience.

    [ User Browser ]
           |
    [ Nginx Proxy ] 
           |
    ---------------------------------------------------
    |                 |                               |
    [ Vue Dashboard ] [ Rust Control Plane ] [ Keycloak SSO ]
    (app.beacon.local) (api.beacon.local)    (sso.beacon.local)
                      |                               |
                [ Docker Engine ]             [ Postgres DB ]
                      |
                -----------------------
                |                     |
          [ MC: Survival ]      [ MC: Creative ]

---

## 🚀 Quick Start (End-Users)

### 1. Host Configuration
Add the following to your system hosts file to enable local domain routing:
* Linux/macOS: /etc/hosts
* Windows: C:\Windows\System32\drivers\etc\hosts

    127.0.0.1 app.beacon.local api.beacon.local sso.beacon.local

### 2. Configure Environment
Download the release, enter the directory, and initialize your configuration:

    cp .env.example .env

Edit .env and set secure values for POSTGRES_PASSWORD, KEYCLOAK_ADMIN_PASSWORD, and BEACON_SECRET_KEY.

### 3. Deployment
Run the stack using Docker Compose:

    docker-compose up -d

* Dashboard: http://app.beacon.local
* API Docs: http://api.beacon.local/docs
* SSO Admin: http://sso.beacon.local

---

## 🛠️ Development & Contributing

### Project Structure
| Directory | Description |
| :--- | :--- |
| /backend | Rust Control Plane (Axum, Bollard, SQLx) |
| /frontend | Vue 3 Dashboard (Vite, Tailwind, Pinia) |
| /nginx | Reverse proxy configuration |
| /keycloak | Realm exports & custom themes |

### Local Setup
1. Backend: Ensure Postgres is running, then: cd backend && cargo watch -x run
2. Frontend: cd frontend && npm install && npm run dev

### Contribution Workflow
1. Fork the Project.
2. Create your Feature Branch (git checkout -b feature/AmazingFeature).
3. Commit your Changes (git commit -m 'Add some AmazingFeature').
4. Push to the Branch (git push origin feature/AmazingFeature).
5. Open a Pull Request.

---

## 🛡️ Security Note

IMPORTANT: Beacon requires access to the Docker Socket (/var/run/docker.sock). In production environments, it is highly recommended to use a Docker Socket Proxy to limit the API calls Beacon can make to only necessary container management functions.

---

## 📄 License
Distributed under the MIT License. See LICENSE for more information.
