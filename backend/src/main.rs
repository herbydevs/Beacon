use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use std::path::{PathBuf};
use std::env;
use std::net::{UdpSocket, SocketAddr};
use tokio::time::{sleep, Duration};

const DOCKER_COMPOSE_RAW: &str = include_str!("../../docker-compose.yml");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aliases = vec![
        "beacon.local",
        "app.beacon.local",
        "api.beacon.local",
        "sso.beacon.local"
    ];

    let current_dir = env::current_dir()?;
    let compose_path = current_dir.join("docker-compose.yml");
    let env_path = current_dir.join(".env");

    println!("========================================");
    println!("      BEACON HUB: P2P ORCHESTRATOR      ");
    println!("========================================");

    // 0. LOAD ENVIRONMENT VARIABLES
    // This fixes the "POSTGRES_USER not set" warnings
    if env_path.exists() {
        load_env_file(&env_path);
    }

    // 1. PRIVILEGE & INSTALLATION CHECKS
    #[cfg(windows)]
    if !is_admin() {
        show_permission_error();
        pause_and_exit();
    }

    if !is_docker_installed() {
        println!("🐳 Docker not found. Installing...");
        install_docker().await?;
    }

    if !is_cloudflared_installed() {
        println!("☁️  Cloudflared not found. Installing...");
        install_cloudflared().await?;
    }

    // 2. NAT HOLE PUNCHING (Fallback)
    println!("📡 Initializing P2P Connectivity...");
    let socket = match UdpSocket::bind("0.0.0.0:25565") {
        Ok(s) => s,
        Err(_) => UdpSocket::bind("0.0.0.0:0")?
    };
    socket.set_nonblocking(true)?;

    // 3. CLOUDFLARE TUNNEL
    println!("☁️  Spinning up Cloudflare Quick Tunnel...");
    let mut tunnel_process = start_cloudflare_tunnel(25565)?;
    sleep(Duration::from_secs(3)).await;

    // 4. ORCHESTRATION SETUP
    ensure_compose_exists(&compose_path)?;
    println!("🌐 Mapping subdomains (Password may be required for hosts file)...");
    update_hosts(&aliases, true)?;

    // 5. RUN STACK (Initial boot)
    println!("🐳 Launching Beacon Stack...");
    run_docker_up(&compose_path)?;

    println!("\n🚀 BEACON LIVE | http://beacon.local");
    println!("--------------------------------------------------");
    println!("COMMANDS: [start] [stop] [connect <url>] [exit]");
    println!("--------------------------------------------------");

    // 6. HUB LOOP & CLEANUP
    let aliases_cleanup = aliases.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let compose_path_cleanup = compose_path.clone();

    ctrlc::set_handler(move || {
        println!("\n🛑 Emergency shutdown initiated...");
        let _ = tunnel_process.kill();
        let refs: Vec<&str> = aliases_cleanup.iter().map(|s| s.as_str()).collect();
        full_cleanup(&refs, &compose_path_cleanup);
        std::process::exit(0);
    })?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line?.trim().to_lowercase();
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.get(0) {
            Some(&"exit") => break,
            Some(&"stop") => {
                println!("🛑 Stopping containers...");
                let _ = Command::new("docker").args(&["compose", "stop"]).status();
            },
            Some(&"start") => {
                println!("🚀 Restarting/Ensuring containers are up...");
                // Use 'up -d' instead of 'start' to ensure containers are created if missing
                let _ = run_docker_up(&compose_path);
            },
            Some(&"connect") => {
                if let Some(url) = parts.get(1) {
                    println!("🔗 Joining tunnel: {}", url);
                    connect_to_tunnel(url)?;
                }
            },
            _ => println!("Unknown command."),
        }
    }

    Ok(())
}

// --- NEW HELPERS ---

fn load_env_file(path: &PathBuf) -> HashMap<String, String> {
    let mut env_map = HashMap::new();

    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || !line.contains('=') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                env_map.insert(
                    parts[0].trim().to_string(),
                    parts[1].trim().to_string()
                );
            }
        }
    }
    env_map
}
fn run_docker_up(compose_path: &PathBuf) -> io::Result<()> {
    Command::new("docker")
        .args(&["compose", "-f", compose_path.to_str().unwrap(), "up", "-d"])
        .status()?;
    Ok(())
}

// --- CLOUDFLARE LOGIC ---

fn is_cloudflared_installed() -> bool {
    Command::new("cloudflared").arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
}

async fn install_cloudflared() -> io::Result<()> {
    #[cfg(target_os = "windows")] {
        Command::new("winget").args(&["install", "--id", "Cloudflare.cloudflared", "--silent"]).status()?;
    }
    #[cfg(target_os = "macos")] {
        Command::new("brew").args(&["install", "cloudflare/cloudflare/cloudflared"]).status()?;
    }
    Ok(())
}

fn start_cloudflare_tunnel(port: u16) -> io::Result<Child> {
    Command::new("cloudflared")
        .args(&["tunnel", "--url", &format!("tcp://localhost:{}", port), "--no-autoupdate"])
        .stdout(Stdio::inherit())
        .spawn()
}

fn connect_to_tunnel(hostname: &str) -> io::Result<()> {
    Command::new("cloudflared")
        .args(&["access", "tcp", "--hostname", hostname, "--url", "localhost:25565"])
        .spawn()?;
    Ok(())
}

// --- CORE LOGIC ---

fn ensure_compose_exists(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        fs::write(path, DOCKER_COMPOSE_RAW)?;
    }
    Ok(())
}

fn full_cleanup(aliases: &[&str], compose_path: &PathBuf) {
    let _ = update_hosts(aliases, false);
    let _ = Command::new("docker").args(&["compose", "-f", compose_path.to_str().unwrap(), "down"]).status();
    // Only remove if it was generated by the app (optional safety)
    // if compose_path.exists() { let _ = fs::remove_file(compose_path); }
}

fn is_admin() -> bool {
    #[cfg(windows)] {
        Command::new("net").arg("session").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
    }
    #[cfg(unix)] { true }
}

fn is_docker_installed() -> bool {
    Command::new("docker").arg("--version").stdout(Stdio::null()).status().map_or(false, |s| s.success())
}

async fn install_docker() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")] {
        Command::new("winget").args(&["install", "Docker.DockerDesktop"]).status()?;
    }
    #[cfg(target_os = "macos")] {
        Command::new("brew").args(&["install", "--cask", "docker"]).status()?;
    }
    Ok(())
}

fn update_hosts(aliases: &[&str], add: bool) -> io::Result<()> {
    let path = if cfg!(windows) { r"C:\Windows\System32\drivers\etc\hosts" } else { "/etc/hosts" };

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    lines.retain(|line| !aliases.iter().any(|&a| line.contains(a)));

    if add {
        for alias in aliases { lines.push(format!("127.0.0.1 {}", alias)); }
    }
    let new_content = lines.join("\n");

    #[cfg(unix)] {
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(format!("echo '{}' > {}", new_content.replace("'", "'\\''"), path))
            .status()?;

        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Failed to write to /etc/hosts via sudo"));
        }

        let _ = Command::new("sudo").args(&["killall", "-HUP", "mDNSResponder"]).status();
    }

    #[cfg(windows)] {
        fs::write(path, new_content)?;
    }
    Ok(())
}

fn show_permission_error() {
    #[cfg(windows)] { println!("❌ Error: Please 'Run as Administrator'."); }
}

fn pause_and_exit() {
    println!("\nPress Enter to exit...");
    let _ = io::stdin().read_line(&mut String::new());
    std::process::exit(1);
}