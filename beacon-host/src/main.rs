use std::fs::{self, File};
use std::io::{self, Write, BufRead, BufReader};
use std::process::{Command, Stdio, Child};
use std::path::{PathBuf, Path};
use std::env;
use std::net::UdpSocket;
use tokio::time::{sleep, Duration};
use std::collections::HashMap;
use log::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let aliases = vec![
        "beacon.local",
        "app.beacon.local",
        "api.beacon.local",
        "sso.beacon.local"
    ];

    // --- SMART PATH RESOLUTION ---
    let exe_path = std::env::current_exe()?;
    let mut root_dir = exe_path.parent()
        .ok_or("Could not find parent directory")?
        .to_path_buf();

    if root_dir.ends_with("backend") {
        root_dir.pop();
    }

    let compose_path = root_dir.join("docker-compose.yml");
    let env_path = root_dir.join(".env");


    println!("========================================");
    println!("      BEACON HUB: P2P ORCHESTRATOR      ");
    println!("========================================");
    println!("🚀 Executing from: {:?}", exe_path);
    println!("📂 Project Root: {:?}", root_dir);

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



    // 2. LOAD ENVIRONMENT
    // We store these in a HashMap and pass them directly to the process environment
    let env_vars = if env_path.exists() {
        println!("📝 Loading configuration from: {:?}", env_path);
        load_env_map(&env_path)
    } else {
        println!("⚠️  No .env found at {:?}, using defaults.", env_path);
        HashMap::new()
    };

    let vars = env_vars;
    print!("{:?}", &vars);

    // 3. NAT HOLE PUNCHING
    println!("📡 Initializing P2P Connectivity...");
    let _socket = match UdpSocket::bind("0.0.0.0:25565") {
        Ok(s) => s,
        Err(_) => UdpSocket::bind("0.0.0.0:0")?
    };

    // 4. CLOUDFLARE TUNNEL
    println!("☁️  Spinning up Cloudflare Quick Tunnel...");
    let mut tunnel_process = start_cloudflare_tunnel(25565)?;
    sleep(Duration::from_secs(3)).await;

    // 5. ORCHESTRATION SETUP
    ensure_compose_exists(&compose_path)?;
    println!("🌐 Mapping subdomains (Password may be required for hosts file)...");
    update_hosts(&aliases, true)?;

    // 6. LAUNCH DOCKER STACK
    println!("🐳 Launching Beacon Stack...");
    let mut docker_cmd = Command::new("docker");

    #[cfg(windows)]
    {
        // Force Docker to use the Windows Named Pipe instead of a Unix Socket
        docker_cmd.env("DOCKER_HOST", "npipe:////./pipe/docker_engine");
    }
    
    // Ensure the process starts in the project root
    docker_cmd.current_dir(&root_dir);

    // Narrowed down arguments
    docker_cmd.args(&["compose", "up", "-d"]);

    // Inject the HashMap directly into the process environment
    docker_cmd.envs(&vars)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    // Pass the compose file explicitly
    // docker_cmd.args(&["compose", "docker-compose.yml", "up", "-d", "--remove-orphans"]);

    // FEATURE: Instead of --env-file, we pass the stored env_vars directly to the child process
    docker_cmd.envs(&vars)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    println!("\n🚀 BEACON LIVE | http://beacon.local");
    println!("--------------------------------------------------");
    println!("COMMANDS: [start] [stop] [connect <url>] [create] [exit]");
    println!("--------------------------------------------------");

    // 7. HUB LOOP & CLEANUP
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
    let mut lines = stdin.lock().lines();

    while let Some(Ok(line)) = lines.next() {
        let input = line.trim().to_lowercase();
        let parts: Vec<&str> = input.split_whitespace().collect();

        match parts.get(0) {
            Some(&"exit") => break,
            Some(&"stop") => {
                let _ = Command::new("docker")
                    .current_dir(&root_dir)
                    .args(&["compose", "stop"])
                    .status();
            },
            Some(&"start") => {
                let _ = Command::new("docker")
                    .current_dir(&root_dir)
                    .args(&["compose", "start"])
                    .status();
            },
            Some(&"create") => {
                println!("🐳 Re-initializing full Beacon stack...");
                #[cfg(windows)]
                {
                    // Force Docker to use the Windows Named Pipe instead of a Unix Socket
                    docker_cmd.env("DOCKER_HOST", "npipe:////./pipe/docker_engine");
                }
                let mut cmd = Command::new("docker");
                cmd.current_dir(&root_dir);
                cmd.args(&["compose", "up", "-d"]);

                // Pass stored vars here too
                match cmd.envs(&vars).status() {
                    Ok(_) => println!("✅ Beacon stack is updated/running!"),
                    Err(e) => println!("❌ Failed to launch: {}", e),
                }
            },
            Some(&"connect") => {
                if let Some(url) = parts.get(1) {
                    println!("🔗 Joining tunnel: {}", url);
                    connect_to_tunnel(url)?;
                }
            },
            _ => println!("Unknown command."),
        }
        print!("> ");
        let _ = io::stdout().flush();
    }

    Ok(())
}

// --- HELPERS ---

fn load_env_map(path: &PathBuf) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            let line = line.trim();
            // Ignore comments and empty lines
            if line.is_empty() || line.starts_with('#') { continue; }
            if let Some((key, value)) = line.split_once('=') {
                map.insert(key.trim().to_string(), value.trim().to_string());
            }
        }
    }
    map
}

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
        .stderr(Stdio::inherit())
        .spawn()
}

fn connect_to_tunnel(hostname: &str) -> io::Result<()> {
    Command::new("cloudflared")
        .args(&["access", "tcp", "--hostname", hostname, "--url", "localhost:25565"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    Ok(())
}

fn ensure_compose_exists(path: &PathBuf) -> io::Result<()> {
    if !path.exists() {
        println!("docker compose does not exist");
    }
    Ok(())
}

fn full_cleanup(aliases: &[&str], compose_path: &PathBuf) {
    let _ = update_hosts(aliases, false);
    let _ = Command::new("docker")
        .args(&["compose", "-f", compose_path.to_str().unwrap(), "down"])
        .stdout(Stdio::inherit())
        .status();
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
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status()?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Failed to write to /etc/hosts via sudo"));
        }
        let _ = Command::new("sudo").args(&["killall", "-HUP", "mDNSResponder"]).status();
    }
    #[cfg(windows)] { fs::write(path, new_content)?; }
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