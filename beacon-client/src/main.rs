use std::fs::{OpenOptions, File};
use std::io::{Write, Read, BufRead, BufReader, self};
use std::process::{Command, Stdio};
use std::path::{PathBuf};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target_host_ip = "12.34.56.78";
    let target_port = "8080";
    let local_alias = "beacon.local";
    let container_name = "beacon-proxy-helper";

    println!("========================================");
    println!("   BEACON HUB: GAME SERVER HELPER       ");
    println!("========================================");

    // 1. PRIVILEGE CHECK
    if !check_permissions() {
        show_permission_error();
        pause_and_exit();
    }

    // 2. DOCKER CLI INSTALLATION LOGIC
    if !is_docker_installed() {
        println!("⚠️ Docker CLI not found.");
        print!("Would you like to attempt an automated installation? (y/n): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            install_docker_distro().await?;
        } else {
            println!("❌ Docker is required. Exiting.");
            pause_and_exit();
        }
    }

    // 3. ENVIRONMENT PREP
    let conf_path = prepare_nginx_config(target_host_ip, target_port, local_alias)?;
    update_hosts(local_alias, true)?;

    // 4. START PROXY
    println!("🔄 Starting Nginx Proxy...");
    let _ = Command::new("docker").args(&["rm", "-f", container_name]).output();
    let status = Command::new("docker")
        .args(&[
            "run", "-d",
            "--name", container_name,
            "-p", "80:80",
            "-v", &format!("{}:/etc/nginx/nginx.conf:ro", conf_path.to_str().unwrap()),
            "nginx:alpine"
        ])
        .status()?;

    if !status.success() {
        println!("❌ Failed to start Docker. Is Port 80 busy?");
        cleanup(container_name, local_alias);
        pause_and_exit();
    }

    println!("\n🚀 HUB ACTIVE");
    println!("URL: http://{}", local_alias);
    println!("Target: {}:{}", target_host_ip, target_port);
    println!("----------------------------------------");
    println!("The hub is now hanging. Press Ctrl+C to stop the proxy and exit.");

    // 5. HANG & CLEANUP
    ctrlc::set_handler(move || {
        println!("\n🛑 Stopping services...");
        cleanup("beacon-proxy-helper", "beacon.local");
        println!("✅ Cleanup complete. System restored.");
        std::process::exit(0);
    })?;

    // Infinite hang
    loop { tokio::time::sleep(std::time::Duration::from_secs(3600)).await; }
}

// --- INSTALLATION LOGIC ---

async fn install_docker_distro() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "macos")] {
        println!("🍎 Detecting Mac... Attempting install via Homebrew.");
        let brew_check = Command::new("brew").arg("--version").status();
        if brew_check.is_ok() && brew_check.unwrap().success() {
            Command::new("brew").args(&["install", "--cask", "docker"]).status()?;
            println!("✅ Docker Desktop triggered. Please complete the UI instructions.");
        } else {
            println!("❌ Homebrew not found. Please install Docker manually: https://www.docker.com/products/docker-desktop");
        }
    }

    #[cfg(target_os = "windows")] {
        println!("🪟 Detecting Windows... Downloading Docker Desktop Installer.");
        let url = "https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe";
        let mut response = reqwest::get(url).await?;
        let mut dest = File::create("docker_installer.exe")?;
        io::copy(&mut response.text().await?.as_bytes(), &mut dest)?; // Simplified for brevity

        println!("🚀 Running Installer... Follow the prompts.");
        Command::new("docker_installer.exe").status()?;
    }
    Ok(())
}

// --- UTILS & CLEANUP ---

fn cleanup(container: &str, alias: &str) {
    let _ = Command::new("docker").args(&["rm", "-f", container]).output();
    let _ = update_hosts(alias, false);
    let _ = std::fs::remove_file("nginx_gen.conf");
}

fn is_docker_installed() -> bool {
    Command::new("docker").arg("--version").stdout(Stdio::null()).status().map_or(false, |s| s.success())
}

fn check_permissions() -> bool {
    #[cfg(windows)] {
        Command::new("net").arg("session").stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success())
    }
    #[cfg(unix)] {
        unsafe { libc::getuid() == 0 }
    }
}

fn show_permission_error() {
    #[cfg(windows)] { println!("❌ Error: Run as Administrator."); }
    #[cfg(unix)] { println!("❌ Error: Run with sudo."); }
}

fn update_hosts(alias: &str, add: bool) -> std::io::Result<()> {
    let path = if cfg!(windows) { r"C:\Windows\System32\drivers\etc\hosts" } else { "/etc/hosts" };
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

fn prepare_nginx_config(ip: &str, port: &str, alias: &str) -> std::io::Result<PathBuf> {
    let content = format!("events {{}} http {{ server {{ listen 80; server_name {alias}; location / {{ proxy_pass http://{ip}:{port}; }} }} }}");
    let path = env::current_dir()?.join("nginx_gen.conf");
    File::create(&path)?.write_all(content.as_bytes())?;
    Ok(path)
}

fn pause_and_exit() {
    println!("Press Enter to exit...");
    let mut s = String::new();
    let _ = io::stdin().read_line(&mut s);
    std::process::exit(1);
}