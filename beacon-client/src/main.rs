use std::fs::{File};
use std::io::{Write, BufRead, BufReader, self};
use std::process::{Command, Stdio};
use std::path::{PathBuf};
use std::env;
use std::net::{UdpSocket, SocketAddr};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("   BEACON HUB: P2P HOLE-PUNCH CLIENT    ");
    println!("   (Windows & macOS Supported)          ");
    println!("========================================");

    // 1. PRIVILEGE CHECK
    if !check_permissions() {
        show_permission_error();
        pause_and_exit();
    }

    // 2. STUN IDENTITY (Find Client's Public Identity)
    // We bind to 25565 locally to ensure the NAT mapping matches the game port
    let socket = match UdpSocket::bind("0.0.0.0:25565") {
        Ok(s) => s,
        Err(_) => {
            println!("❌ Error: Port 25565 is busy. Close Minecraft or existing proxies.");
            pause_and_exit();
            return Ok(());
        }
    };
    socket.set_nonblocking(true)?;

    println!("📡 Querying STUN server for your public identity...");
    let stun_addr = "74.125.200.127:19302"; // stun.l.google.com
    let _ = socket.send_to(&[0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], stun_addr);
    
    sleep(Duration::from_secs(1)).await;
    
    println!("\n----------------------------------------");
    println!("📢 GIVE THESE DETAILS TO THE SERVER OWNER:");
    println!("Your Public IP: (Check 'what is my ip' on Google)");
    println!("Your Punch Port: 25565");
    println!("----------------------------------------\n");

    // 3. INPUT SERVER DETAILS
    print!("🌐 Enter Server's Public IP: ");
    io::stdout().flush()?;
    let mut server_ip = String::new();
    io::stdin().read_line(&mut server_ip)?;
    let server_ip = server_ip.trim().to_string();

    print!("🔢 Enter Server's Punch Port: ");
    io::stdout().flush()?;
    let mut server_port = String::new();
    io::stdin().read_line(&mut server_port)?;
    let server_port = server_port.trim().to_string();

    if server_ip.is_empty() || server_port.is_empty() {
        println!("❌ Error: Server details cannot be empty.");
        pause_and_exit();
    }

    let server_addr: SocketAddr = format!("{}:{}", server_ip, server_port).parse()?;

    // 4. THE PUNCH (Keep-Alive)
    println!("🥊 Punching hole to server...");
    for _ in 0..10 { // Increased iterations for more reliable NAT traversal
        let _ = socket.send_to(b"BEACON_PUNCH", server_addr);
        sleep(Duration::from_millis(200)).await;
    }

    // 5. INSTALLATION & DAEMON CHECKS
    if !is_docker_installed() {
        println!("⚠️ Docker components not found.");
        print!("Would you like to attempt an automated installation? (y/n): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            install_docker_distro().await?;
            println!("\n✅ Installation logic finished.");
            println!("⚠️ CRITICAL: You MUST restart your computer now for changes to take effect.");
            pause_and_exit();
        } else {
            println!("❌ Docker is required. Exiting.");
            pause_and_exit();
        }
    }

    if !is_docker_daemon_running() {
        println!("🐋 Waking up Docker Engine...");
        start_docker()?;
        let mut attempts = 0;
        while !is_docker_daemon_running() && attempts < 15 {
            print!("."); io::stdout().flush()?;
            sleep(Duration::from_secs(5)).await;
            attempts += 1;
        }
        println!();
        if !is_docker_daemon_running() {
            println!("❌ Docker Engine failed to start. Please open it manually.");
            pause_and_exit();
        }
    }

    // 6. NGINX & HOSTS SETUP
    let local_alias = "beacon.local";
    let container_name = "beacon-proxy-helper";
    let conf_path = prepare_nginx_config(&server_ip, &server_port)?;
    update_hosts(local_alias, true)?;

    // 7. START PROXY
    println!("\n🔄 Starting Nginx Proxy (TCP/UDP Bridge)...");
    let _ = Command::new("docker").args(&["rm", "-f", container_name]).output();
    let status = Command::new("docker")
        .args(&[
            "run", "-d",
            "--name", container_name,
            "-p", "25565:25565", 
            "-p", "25565:25565/udp",
            "-v", &format!("{}:/etc/nginx/nginx.conf:ro", conf_path.to_str().expect("Path error")),
            "nginx:alpine"
        ])
        .status()?;

    if !status.success() {
        println!("❌ Failed to start container. Is Port 25565 already in use?");
        cleanup(container_name, local_alias);
        pause_and_exit();
    }

    println!("\n🚀 HUB ACTIVE");
    println!("Minecraft Address: {}", local_alias);
    println!("Tunneling to: {}:{}", server_ip, server_port);
    println!("----------------------------------------");
    println!("Press Ctrl+C to stop the proxy and exit.");

    // 8. HANG & CLEANUP
    ctrlc::set_handler(move || {
        println!("\n🛑 Stopping services...");
        cleanup("beacon-proxy-helper", "beacon.local");
        println!("✅ Cleanup complete. System restored.");
        std::process::exit(0);
    })?;

    loop { sleep(Duration::from_secs(3600)).await; }
}

// --- CONFIG GENERATOR ---

fn prepare_nginx_config(ip: &str, port: &str) -> std::io::Result<PathBuf> {
    let content = format!(
        "events {{ worker_connections 1024; }} \
         stream {{ \
            server {{ listen 25565; proxy_pass {ip}:{port}; }} \
            server {{ listen 25565 udp; proxy_pass {ip}:{port}; }} \
         }}"
    );
    let path = env::current_dir()?.join("nginx_gen.conf");
    File::create(&path)?.write_all(content.as_bytes())?;
    Ok(path)
}

// --- SHARED UTILITIES ---

fn update_hosts(alias: &str, add: bool) -> std::io::Result<()> {
    let path = if cfg!(target_os = "windows") { 
        r"C:\Windows\System32\drivers\etc\hosts" 
    } else { 
        "/etc/hosts" 
    };

    let mut lines = Vec::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let l = line?;
            if !l.contains(alias) { lines.push(l); }
        }
    }
    if add { lines.push(format!("127.0.0.1 {}", alias)); }
    let mut file = File::create(path)?;
    for line in lines { writeln!(file, "{}", line)?; }
    Ok(())
}

async fn install_docker_distro() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")] {
        let _ = Command::new("wsl").args(&["--install", "--no-distribution"]).status();
        let _ = Command::new("wsl").args(&["--update"]).status();
        Command::new("winget").args(&["install", "Docker.DockerDesktop", "--accept-source-agreements"]).status()?;
    }
    #[cfg(target_os = "macos")] {
        println!("🚀 Attempting Homebrew installation...");
        Command::new("brew").args(&["install", "--cask", "docker"]).status()?;
    }
    Ok(())
}

fn is_docker_installed() -> bool {
    Command::new("docker").arg("--version").stdout(Stdio::null()).status().map_or(false, |s| s.success())
}

fn is_docker_daemon_running() -> bool {
    Command::new("docker").arg("info").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
}

fn start_docker() -> std::io::Result<()> {
    #[cfg(target_os = "windows")] {
        Command::new("cmd").args(&["/C", "start", "docker-desktop://dashboard"]).spawn()?;
    }
    #[cfg(target_os = "macos")] {
        Command::new("open").args(&["-a", "Docker"]).spawn()?;
    }
    Ok(())
}

fn check_permissions() -> bool { 
    #[cfg(target_os = "windows")] {
        Command::new("net").arg("session").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
    }
    #[cfg(target_os = "macos")] {
        unsafe { libc::getuid() == 0 }
    }
}

fn cleanup(container: &str, alias: &str) {
    let _ = Command::new("docker").args(&["rm", "-f", container]).output();
    let _ = update_hosts(alias, false);
    let _ = std::fs::remove_file("nginx_gen.conf");
}

fn pause_and_exit() {
    println!("\nPress Enter to exit...");
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    std::process::exit(1);
}

fn show_permission_error() {
    #[cfg(target_os = "windows")]
    println!("❌ Error: Please run this terminal as Administrator to modify the hosts file.");
    #[cfg(target_os = "macos")]
    println!("❌ Error: Please run this script with sudo (e.g., sudo ./beacon_hub).");
}