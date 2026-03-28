use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use std::path::{PathBuf};
use std::env;
use std::net::{UdpSocket, SocketAddr};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("   BEACON HUB: CLOUD-POWERED CLIENT    ");
    println!("   (Windows & macOS Supported)          ");
    println!("========================================");

    // 1. PRIVILEGE & INSTALLATION CHECKS
    #[cfg(windows)]
    if !is_admin() {
        show_permission_error();
        pause_and_exit();
    }

    if !is_cloudflared_installed() {
        println!("☁️  Cloudflared not found. Installing...");
        install_cloudflared().await?;
    }

    // 2. INPUT TUNNEL DETAILS
    println!("\n🔗 CONNECT TO SERVER");
    print!("🌐 Enter Server's Tunnel URL (e.g., words-word-word.trycloudflare.com): ");
    io::stdout().flush()?;
    let mut tunnel_url = String::new();
    io::stdin().read_line(&mut tunnel_url)?;
    let tunnel_url = tunnel_url.trim().to_string();

    if tunnel_url.is_empty() {
        println!("❌ Error: Tunnel URL cannot be empty.");
        pause_and_exit();
    }

    // 3. HOSTS MAPPING (May ask for password on Mac)
    let local_alias = "beacon.local";
    println!("🌐 Mapping '{}' to localhost (Password may be required)...", local_alias);
    update_hosts(local_alias, true)?;

    // 4. START CLOUDFLARE ACCESS BRIDGE
    println!("🥊 Establishing secure bridge to {}...", tunnel_url);
    let mut bridge_process = connect_to_tunnel(&tunnel_url)?;

    // Give the bridge a second to initialize
    sleep(Duration::from_secs(2)).await;

    println!("\n🚀 CLIENT BRIDGE ACTIVE");
    println!("----------------------------------------");
    println!("Minecraft Address: {}", local_alias);
    println!("Target Tunnel:     {}", tunnel_url);
    println!("Local Port:        25565");
    println!("----------------------------------------");
    println!("Press Ctrl+C to disconnect and exit.");

    // 5. HANG & CLEANUP
    ctrlc::set_handler(move || {
        println!("\n🛑 Disconnecting...");
        let _ = bridge_process.kill();
        let _ = update_hosts("beacon.local", false);
        println!("✅ System restored.");
        std::process::exit(0);
    })?;

    loop { sleep(Duration::from_secs(3600)).await; }
}

// --- CLOUDFLARE LOGIC ---

fn is_cloudflared_installed() -> bool {
    Command::new("cloudflared")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_or(false, |s| s.success())
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

fn connect_to_tunnel(hostname: &str) -> io::Result<Child> {
    // Maps the remote tunnel back to the client's local port 25565
    Command::new("cloudflared")
        .args(&[
            "access", "tcp",
            "--hostname", hostname,
            "--url", "localhost:25565"
        ])
        .spawn()
}

// --- SYSTEM UTILITIES ---

fn update_hosts(alias: &str, add: bool) -> io::Result<()> {
    let path = if cfg!(target_os = "windows") {
        r"C:\Windows\System32\drivers\etc\hosts"
    } else {
        "/etc/hosts"
    };

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    lines.retain(|line| !line.contains(alias));

    if add {
        lines.push(format!("127.0.0.1 {}", alias));
    }
    let new_content = lines.join("\n");

    #[cfg(unix)] {
        // Use sh -c to allow the sudo context to handle the redirection/write properly
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(format!("echo '{}' > {}", new_content.replace("'", "'\\''"), path))
            .status()?;

        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Failed to write to /etc/hosts via sudo"));
        }

        // Flush the DNS cache so macOS picks up the change immediately
        let _ = Command::new("sudo").args(&["killall", "-HUP", "mDNSResponder"]).status();
    }

    #[cfg(windows)] {
        fs::write(path, new_content)?;
    }

    Ok(())
}

fn is_admin() -> bool {
    #[cfg(target_os = "windows")] {
        Command::new("net").arg("session").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
    }
    #[cfg(target_os = "macos")] {
        true // Handled via targeted sudo in update_hosts
    }
}

fn show_permission_error() {
    #[cfg(target_os = "windows")]
    println!("❌ Error: Please run this terminal as Administrator.");
}

fn pause_and_exit() {
    println!("\nPress Enter to exit...");
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    std::process::exit(1);
}