use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::path::{PathBuf};
use std::env;
use std::net::{UdpSocket, SocketAddr};
use tokio::time::{sleep, Duration};

// Baked into the binary at compile-time.
// Adjusted path to look two levels up from src/ to find the root docker-compose.yml
const DOCKER_COMPOSE_RAW: &str = include_str!("../../docker-compose.yml");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aliases = vec![
        "beacon.local",
        "app.beacon.local",
        "api.beacon.local",
        "sso.beacon.local"
    ];

    let compose_path = env::current_dir()?.join("docker-compose.yml");

    println!("========================================");
    println!("      BEACON HUB: P2P ORCHESTRATOR      ");
    println!("========================================");

    // 1. PRIVILEGE & DOCKER CHECKS (Existing)
    if !is_admin() {
        show_permission_error();
        pause_and_exit();
    }

    if !is_docker_installed() {
        // ... (Keep your existing install_docker() logic here)
    }

    // 2. NAT HOLE PUNCHING SEQUENCE (New Feature)
    println!("📡 Initializing P2P Connectivity...");
    
    // Bind to the port your Docker stack expects (usually 25565 or 80/443)
    let socket = match UdpSocket::bind("0.0.0.0:25565") {
        Ok(s) => s,
        Err(_) => {
            println!("⚠️ Warning: Local port 25565 is busy. P2P Punching might be limited.");
            UdpSocket::bind("0.0.0.0:0")?
        }
    };
    socket.set_nonblocking(true)?;

    // Identify Public IP/Port via STUN
    let stun_addr = "74.125.200.127:19302"; // stun.l.google.com
    let _ = socket.send_to(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], stun_addr);
    sleep(Duration::from_secs(1)).await;

    println!("\n----------------------------------------");
    println!("📢 SHARE YOUR PUBLIC IP WITH THE PEER");
    println!("Your Port: 25565 (UDP)");
    println!("----------------------------------------\n");

    print!("🌐 Enter Peer's Public IP: ");
    io::stdout().flush()?;
    let mut peer_ip = String::new();
    io::stdin().read_line(&mut peer_ip)?;
    let peer_ip = peer_ip.trim();

    if !peer_ip.is_empty() {
        let peer_addr: SocketAddr = format!("{}:25565", peer_ip).parse()?;
        println!("🥊 Punching hole to {}...", peer_addr);
        for _ in 0..10 {
            let _ = socket.send_to(b"BEACON_PUNCH", peer_addr);
            sleep(Duration::from_millis(200)).await;
        }
    }

    // 3. ORCHESTRATION SETUP (Existing)
    ensure_compose_exists(&compose_path)?;
    println!("🌐 Mapping subdomains in hosts file...");
    update_hosts(&aliases, true)?;

    // 4. RUN STACK (Existing)
    println!("🐳 Launching Beacon Stack...");
    Command::new("docker")
        .args(&["compose", "-f", compose_path.to_str().unwrap(), "up", "-d"])
        .status()?;

    println!("\n🚀 BEACON LIVE | http://beacon.local");
    println!("--------------------------------------------------");
    println!("COMMANDS: [start] [stop] [status] [restart] [exit]");
    println!("--------------------------------------------------");

    // 5. HUB LOOP & CLEANUP (Existing)
    let aliases_cleanup = aliases.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let compose_path_cleanup = compose_path.clone();
    
    ctrlc::set_handler(move || {
        println!("\n🛑 Emergency shutdown initiated...");
        let refs: Vec<&str> = aliases_cleanup.iter().map(|s| s.as_str()).collect();
        full_cleanup(&refs, &compose_path_cleanup);
        std::process::exit(0);
    })?;

    // ... (Keep your existing loop match logic here)
    
    Ok(())
}

// --- CORE LOGIC FUNCTIONS ---

fn ensure_compose_exists(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        println!("🛠️  Syncing orchestration files...");
        fs::write(path, DOCKER_COMPOSE_RAW)?;
    }
    Ok(())
}

fn full_cleanup(aliases: &[&str], compose_path: &PathBuf) {
    let _ = update_hosts(aliases, false);
    let _ = Command::new("docker")
        .args(&["compose", "-f", compose_path.to_str().unwrap(), "down"])
        .status();
    if compose_path.exists() {
        let _ = fs::remove_file(compose_path);
    }
}

// --- SYSTEM UTILITIES ---

fn is_admin() -> bool {
    #[cfg(windows)] {
        Command::new("net").arg("session").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
    }
    #[cfg(unix)] {
        unsafe { libc::getuid() == 0 }
    }
}

fn is_docker_installed() -> bool {
    Command::new("docker").arg("--version").stdout(Stdio::null()).status().map_or(false, |s| s.success())
}

#[allow(dead_code)]
fn is_daemon_running() -> bool {
    Command::new("docker").arg("info").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
}

async fn install_docker() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")] {
        let _ = Command::new("wsl").args(&["--install", "--no-distribution"]).status();
        let _ = Command::new("winget").args(&["install", "Docker.DockerDesktop"]).status();
    }
    #[cfg(target_os = "macos")] {
        println!("🚀 Using Homebrew to install Docker...");
        let _ = Command::new("brew").args(&["install", "--cask", "docker"]).status();
    }
    Ok(())
}


fn update_hosts(aliases: &[&str], add: bool) -> io::Result<()> {
    let path = if cfg!(windows) { r"C:\Windows\System32\drivers\etc\hosts" } else { "/etc/hosts" };
    let file = File::open(path)?;
    let mut lines: Vec<String> = BufReader::new(file).lines().map(|l| l.unwrap()).collect();

    // Clean old entries
    lines.retain(|line| !aliases.iter().any(|&a| line.contains(a)));

    if add {
        for alias in aliases {
            lines.push(format!("127.0.0.1 {}", alias));
        }
    }

    let mut file = File::create(path)?;
    for line in lines { writeln!(file, "{}", line)?; }
    Ok(())
}

fn show_permission_error() {
    #[cfg(windows)] { println!("❌ Error: This app must be 'Run as Administrator'."); }
    #[cfg(unix)] { println!("❌ Error: This app must be run with 'sudo'."); }
}

fn pause_and_exit() {
    println!("\nPress Enter to exit...");
    let _ = io::stdin().read_line(&mut String::new());
    std::process::exit(1);
}