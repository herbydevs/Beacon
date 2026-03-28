use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::path::{PathBuf};
use std::env;

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

    // Define the path logically immediately
    let compose_path = env::current_dir()?.join("docker-compose.yml");

    println!("========================================");
    println!("      BEACON HOST ORCHESTRATOR          ");
    println!("========================================");

    // 1. PRIVILEGE CHECK
    if !is_admin() {
        show_permission_error();
        pause_and_exit();
    }

    // 2. DOCKER CHECK & INSTALL
    if !is_docker_installed() {
        println!("⚠️ Docker not found on this system.");
        print!("Would you like to automatically install Docker Desktop? (y/n): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            install_docker().await?;
            println!("✅ Headless installation complete or initiated.");
            println!("Please restart the app (and possibly your system) once Docker is running.");
            pause_and_exit();
        } else {
            println!("❌ Docker is required. Exiting.");
            pause_and_exit();
        }
    }

    // 3. THE "TRUST" MECHANISM: Create if missing
    ensure_compose_exists(&compose_path)?;

    // 4. DNS SETUP
    println!("🌐 Mapping subdomains in /etc/hosts...");
    update_hosts(&aliases, true)?;

    // 5. RUN STACK
    println!("🐳 Launching Beacon Stack...");
    Command::new("docker")
        .args(&["compose", "-f", compose_path.to_str().unwrap(), "up", "-d"])
        .status()?;

    println!("\n🚀 BEACON LIVE | http://beacon.local");
    println!("--------------------------------------------------");
    println!("COMMANDS: [start] [stop] [status] [restart] [exit]");
    println!("--------------------------------------------------");

    // Setup Graceful Shutdown (Ctrl+C)
    let aliases_cleanup = aliases.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let compose_path_cleanup = compose_path.clone();
    ctrlc::set_handler(move || {
        println!("\n🛑 Emergency shutdown initiated...");
        let refs: Vec<&str> = aliases_cleanup.iter().map(|s| s.as_str()).collect();
        full_cleanup(&refs, &compose_path_cleanup);
        std::process::exit(0);
    })?;

    // 6. HUB LOOP
    loop {
        print!("beacon > ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let cmd = input.trim().to_lowercase();

        match cmd.as_str() {
            "start" => {
                println!("Starting Beacon containers...");
                ensure_compose_exists(&compose_path)?;
                Command::new("docker")
                    .args(&["compose", "-f", compose_path.to_str().unwrap(), "start"])
                    .status()?;
            }
            "stop" => {
                println!("Stopping Beacon containers...");
                ensure_compose_exists(&compose_path)?;
                Command::new("docker")
                    .args(&["compose", "-f", compose_path.to_str().unwrap(), "stop"])
                    .status()?;
            }
            "exit" | "quit" => {
                println!("Stopping services and cleaning system...");
                full_cleanup(&aliases, &compose_path);
                println!("✅ Cleanup complete. Goodbye!");
                break;
            }
            "status" => {
                ensure_compose_exists(&compose_path)?;
                let _ = Command::new("docker")
                    .args(&["compose", "-f", compose_path.to_str().unwrap(), "ps"])
                    .status();
            }
            "restart" => {
                println!("Restarting containers...");
                ensure_compose_exists(&compose_path)?;
                let _ = Command::new("docker")
                    .args(&["compose", "-f", compose_path.to_str().unwrap(), "restart"])
                    .status();
            }
            _ => {
                if !cmd.is_empty() {
                    println!("Unknown command: '{}'. Type 'exit' to close.", cmd);
                }
            }
        }
    }

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
        println!("🐳 Installing Docker CLI via Winget...");
        // Use winget to install Docker Desktop but we will skip launching the GUI
        let status = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",     
                "winget install -e --id Docker.DockerDesktop --accept-package-agreements --accept-source-agreements --quiet"
            ])
            .status()?;

        if !status.success() {
            return Err("Winget failed. Ensure Winget is installed and you are connected to the internet.".into());
        }
    }
    #[cfg(target_os = "macos")] {
        println!("🍎 Installing Docker CLI and Colima (Headless Engine)...");
        env::set_var("HOMEBREW_NO_INTERACTIVE", "1");

        // Install docker (CLI) and colima (The actual background engine)
        Command::new("brew").args(&["install", "docker", "colima"]).status()?;

        println!("🚀 Starting Colima engine...");
        Command::new("colima").args(&["start", "--cpu", "2", "--memory", "4"]).status()?;
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