🗼 Beacon

A high-performance, container-native Minecraft orchestrator.

Beacon is a lightweight management suite designed to simplify the deployment and scaling of Minecraft servers. By leveraging a Rust (Axum) control plane and a Vue 3 dashboard, Beacon provides a "blazingly fast" interface to spawn, monitor, and back up containerized game instances with near-zero host overhead.

✨ Features

⚡ One-Click Deployment: Spin up Vanilla, Paper, or Forge servers in seconds.

📈 Real-time Monitoring: Live CPU/RAM stats and console streaming via WebSockets.

📦 Container-First: Every server runs in an isolated Docker environment for maximum security.

💾 Automated Backups: Integrated snapshot system to keep your worlds safe.

🛠️ Developer-Friendly API: A fully documented REST API for custom integrations.

🏗️ Architecture

Beacon uses a reverse-proxy model to manage multiple local domains and provide a seamless SSO experience via Keycloak.

                          [ User Browser ]
                                 |
                    (app/api/sso.beacon.local)
                                 |
                      [ Nginx Reverse Proxy ]
                                 |
     ----------------------------+----------------------------
     |                           |                           |
[ Vue Dashboard ]       [ Rust Control Plane ]       [ Keycloak SSO ]
(app.beacon.local)      (api.beacon.local)           (sso.beacon.local)
                                 |                           |
                         [ Docker Engine ]           [ Postgres Database ]
                                 |
                -----------------------------------
                |                                 |
       [ MC Server: Survival ]           [ MC Server: Creative ]


🌐 Local DNS & Networking

To access the dashboard and services via custom local domains, you must map them in your system's hosts file.

1. Configure Hosts File

Add the following lines to your operating system's hosts file:

127.0.0.1 app.beacon.local
127.0.0.1 api.beacon.local
127.0.0.1 sso.beacon.local


File Locations:

Windows: C:\Windows\System32\drivers\etc\hosts (Run Notepad as Administrator)

Linux/macOS: /etc/hosts (Use sudo nano /etc/hosts)

2. Reverse Proxy (Nginx)

The stack includes an Nginx container that listens on port 80 and routes traffic internally:

app.beacon.local -> frontend:3000

api.beacon.local -> backend:8000

sso.beacon.local -> keycloak:8080

🚀 Quick Start & Deployment

1. Prerequisites

Ensure you have the following installed on your host machine:

Docker & Docker Compose

Rust (Latest Stable) & Node.js (v20+) — Only required for local development

2. Initial Setup

Clone the repository and prepare the environment:

git clone [https://github.com/herbydevs/beacon.git](https://github.com/herbydevs/beacon.git)
cd beacon
cp .env.example .env


3. Deploy with Docker Compose

The easiest way to run Beacon is via the provided compose file. This starts the entire stack including Nginx and Keycloak.

docker-compose up -d --build


Dashboard: http://app.beacon.local

API Docs: http://api.beacon.local/docs

SSO Admin: http://sso.beacon.local

🛠️ Development & Contributing

Project Structure

/beacon
├── backend/              # Rust (Axum, Bollard, SQLx)
├── frontend/             # Vue 3 (Vite, Tailwind CSS, Pinia)
├── nginx/                # Nginx configuration (default.conf)
├── keycloak/             # Keycloak realm exports & themes
├── docker-compose.yml    # Main deployment manifest
└── .env                  # Global configuration


Backend Setup (Rust)

Ensure Postgres is running.

cd backend

cargo watch -x run

Frontend Setup (Vue 3)

cd frontend

npm install

npm run dev

Contribution Workflow

Fork the Project.

Create your Feature Branch (git checkout -b feature/AmazingFeature).

Commit your Changes (git commit -m 'Add some AmazingFeature').

Push to the Branch (git push origin feature/AmazingFeature).

Open a Pull Request.

📄 License

Distributed under the MIT License. See LICENSE for more information.

🛡️ Security Note

Beacon requires access to the Docker Socket (/var/run/docker.sock). In production environments, it is highly recommended to use a Docker Socket Proxy to limit the API calls Beacon can make to only necessary container management functions.