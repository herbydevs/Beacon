# 🚀 Beacon Hub
P2P Orchestrator and Control Plane

## 🛠 Features
* **P2P Connectivity:** NAT hole-punching for seamless node connection.
* **Docker Orchestration:** Automated management of the Beacon stack.
* **Cloudflare Integration:** Secure tunneling out of the box.

## 📁 Project Structure
* `/backend`: Rust-based orchestrator core.
* `/frontend`: Vue/Vite dashboard (contains `index.html` and assets).
* `/database`: Initialization scripts and SQL schemas.

## 🚀 Getting Started
### Prerequisites
* Docker Desktop (ensure the daemon is running)
* Rust (latest stable)
* Administrative/Sudo privileges (for `hosts` file mapping)

### Installation
1. Clone the repository.
2. Configure your `.env` file in the root directory.
3. Run the setup script to initialize the database:
   
   # On macOS/Linux:
   chmod +x ./setup_db.sh
   ./setup_db.sh
   
   # On Windows (CMD/PowerShell as Admin):
   setup_db.bat

## 🎮 Usage
Once the stack is live, the orchestrator provides a CLI for managing your nodes and tunnels.

- start          : Resumes the Docker stack if it was previously stopped.
- stop           : Safely halts the running containers.
- create         : Re-initializes the full stack (useful after config changes).
- connect <url>  : Joins a remote P2P tunnel using the provided hostname.
- exit           : Shuts down the orchestrator, stops tunnels, and cleans up.

---

## 📡 Networking & Subdomains
Beacon automatically maps the following subdomains to your local loopback address. 

- beacon.local      -> Main Dashboard
- api.beacon.local  -> Backend API
- sso.beacon.local  -> Authentication Service

---

## 🔧 Troubleshooting
- Docker Connection: On Windows, ensure you "Run as Administrator" so the orchestrator can access the Docker Named Pipe (npipe://).
- Architecture Errors: If using Apple Silicon (M1/M2/M3), ensure your docker-compose.yml specifies "platform: linux/amd64" for x86_64 binaries.
- GLIBC Version: If you see "GLIBC_2.39 not found", ensure your Docker image is set to ubuntu:24.04 or higher.

---

## 🛡 Security & Environment
Configuration is handled via a .env file. Ensure these match your orchestrator settings:

POSTGRES_USER=user
POSTGRES_PASSWORD=password
POSTGRES_DB=beacon

---

## 🤝 Contributing
1. Fork the Project.
2. Create your Feature Branch (git checkout -b feature/AmazingFeature).
3. Commit your Changes (git commit -m 'Add some AmazingFeature').
4. Push to the Branch (git push origin feature/AmazingFeature).
5. Open a Pull Request.

---

## ⚖️ License
Distributed under the GPL-3.0 License. See LICENSE for more information.
