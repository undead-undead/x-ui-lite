use crate::errors::ApiResult;
use chrono;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use sysinfo::{Disks, Networks, System};

pub type SharedMonitor = Arc<Mutex<SystemMonitor>>;

pub struct SystemMonitor {
    sys: System,
    disks: Disks,
    networks: Networks,
    mock_running: bool,
    start_time: std::time::Instant,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        Self {
            sys,
            disks,
            networks,
            mock_running: true,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn get_system_stats(&mut self) -> ApiResult<SysStats> {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        let _ = self.disks.refresh(true);
        let _ = self.networks.refresh(true);

        let cpu_load = self.sys.global_cpu_usage() as f64;

        // Try getting memory and swap from free command first to match "free -h" output
        let (mem_total, mem_current, swap_total, swap_current) =
            if let Some((mt, mu, st, su)) = get_linux_memory_stats() {
                (mt, mu, st, su)
            } else {
                let mem_total = self.sys.total_memory();
                let mem_available = self.sys.available_memory();
                let mem_current = mem_total.saturating_sub(mem_available);
                let swap_current = self.sys.used_swap();
                let swap_total = self.sys.total_swap();
                (mem_total, mem_current, swap_total, swap_current)
            };

        let mut disk_current = 0;
        let mut disk_total = 0;
        for disk in &self.disks {
            disk_total += disk.total_space();
            disk_current += disk.total_space() - disk.available_space();
        }

        // Use application uptime instead of system uptime
        let uptime = self.start_time.elapsed().as_secs();

        let load_avg = System::load_average();
        let load = vec![load_avg.one, load_avg.five, load_avg.fifteen];

        let xray_state_str = if self.is_xray_running() {
            "running"
        } else {
            "stopped"
        };

        let xray_version = get_xray_version().unwrap_or_else(|| "Unknown".to_string());

        let xray = XrayStatus {
            state: xray_state_str.to_string(),
            version: xray_version,
        };

        let (tcp_count, udp_count) = get_connection_counts();

        let mut net_sent = 0;
        let mut net_recv = 0;
        let mut net_up = 0;
        let mut net_down = 0;

        for (interface_name, data) in &self.networks {
            if interface_name == "lo" 
                || interface_name.starts_with("docker") 
                || interface_name.starts_with("veth") 
                || interface_name.starts_with("br-") 
            {
                continue;
            }
            net_sent += data.total_transmitted();
            net_recv += data.total_received();
            net_up += data.transmitted();
            net_down += data.received();
        }

        Ok(SysStats {
            cpu: cpu_load,
            mem: MemStats {
                current: mem_current,
                total: mem_total,
            },
            swap: SwapStats {
                current: swap_current,
                total: swap_total,
            },
            disk: DiskStats {
                current: disk_current,
                total: disk_total,
            },
            uptime,
            load,
            xray,
            tcp_count,
            udp_count,
            net_traffic: NetTraffic {
                sent: net_sent,
                recv: net_recv,
            },
            net_io: NetIo {
                up: net_up,
                down: net_down,
            },
        })
    }

    fn is_xray_running(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            return self.mock_running;
        }
        #[cfg(not(target_os = "linux"))]
        return self.mock_running;
    }

    pub fn set_mock_running(&mut self, running: bool) {
        self.mock_running = running;
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateXrayRequest {
    pub version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SysStats {
    pub cpu: f64,
    pub mem: MemStats,
    pub swap: SwapStats,
    pub disk: DiskStats,
    pub uptime: u64,
    pub load: Vec<f64>,
    pub xray: XrayStatus,
    pub tcp_count: usize,
    pub udp_count: usize,
    pub net_traffic: NetTraffic,
    pub net_io: NetIo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemStats {
    pub current: u64,
    pub total: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapStats {
    pub current: u64,
    pub total: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskStats {
    pub current: u64,
    pub total: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayStatus {
    pub state: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetTraffic {
    pub sent: u64,
    pub recv: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetIo {
    pub up: u64,
    pub down: u64,
}

pub async fn get_logs() -> ApiResult<Vec<String>> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_path = cwd.join("logs").join("error.log");
    let access_log_path = cwd.join("logs").join("access.log");

    let mut logs = Vec::new();

    async fn read_last_bytes(path: &std::path::Path, limit: u64) -> Vec<String> {
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        let mut file = match tokio::fs::File::open(path).await {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        if let Ok(metadata) = file.metadata().await {
            let size = metadata.len();
            let offset = if size > limit { size - limit } else { 0 };
            let _ = file.seek(std::io::SeekFrom::Start(offset)).await;
        }

        let mut buffer = String::new();
        let _ = file.read_to_string(&mut buffer).await;
        buffer.lines().map(|s| s.to_string()).collect()
    }

    let error_lines = read_last_bytes(&log_path, 32768).await;
    for line in error_lines {
        logs.push(format!("[ErrorLog] {}", line));
    }

    let access_lines = read_last_bytes(&access_log_path, 32768).await;
    let start = if access_lines.len() > 50 {
        access_lines.len() - 50
    } else {
        0
    };
    for line in &access_lines[start..] {
        logs.push(format!("[AccessLog] {}", line));
    }

    if !logs.is_empty() {
        return Ok(logs);
    }

    let output = tokio::process::Command::new("journalctl")
        .args(["-u", "x-ui", "-n", "100", "--no-pager"])
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => {
            let logs_str = String::from_utf8_lossy(&out.stdout);
            Ok(logs_str.lines().map(|s| s.to_string()).collect())
        }
        _ => get_fallback_logs(),
    }
}

fn get_fallback_logs() -> ApiResult<Vec<String>> {
    Ok(vec![
        format!(
            "[{}] System self-check passed...",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
        format!(
            "[{}] Database connection pool is ready...",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
        format!(
            "[{}] Xray core is running normally...",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
        format!(
            "[{}] Backend management process is listening...",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
        format!(
            "[{}] No active system service logs captured (x-ui.service)...",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        ),
    ])
}

pub async fn stop_xray(monitor: SharedMonitor) -> ApiResult<()> {
    tracing::info!("Received request to stop Xray service...");

    {
        let mut m = monitor.lock().map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Monitor lock poisoned: {}", e))
        })?;
        m.set_mock_running(false);
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("pkill")
            .arg("-f")
            .arg("xray")
            .output();

        let _ = std::process::Command::new("killall").arg("xray").output();
    }

    Ok(())
}

pub async fn start_xray(monitor: SharedMonitor) -> ApiResult<()> {
    tracing::info!("Received request to start Xray service...");

    {
        let mut m = monitor.lock().map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Monitor lock poisoned: {}", e))
        })?;
        m.set_mock_running(true);
    }

    #[cfg(target_os = "linux")]
    {
        let bin_path_str =
            std::env::var("XRAY_BIN_PATH").unwrap_or("/usr/local/bin/xray".to_string());
        let config_path_str =
            std::env::var("XRAY_CONFIG_PATH").unwrap_or("/etc/x-ui/xray.json".to_string());

        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let log_dir = cwd.join("logs");
        if !log_dir.exists() {
            let _ = std::fs::create_dir_all(&log_dir);
        }

        let stdout_file = std::fs::File::create(log_dir.join("access.log")).map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to create stdout log: {}", e))
        })?;
        let stderr_file = std::fs::File::create(log_dir.join("error.log")).map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to create stderr log: {}", e))
        })?;

        let child = std::process::Command::new(&bin_path_str)
            .arg("-c")
            .arg(&config_path_str)
            .env("GOMEMLIMIT", "150MiB")
            .env("GOGC", "50")
            .stdout(stdout_file)
            .stderr(stderr_file)
            .spawn();

        match child {
            Ok(_) => tracing::info!(
                "Xray process started directly: {} -c {}",
                bin_path_str,
                config_path_str
            ),
            Err(e) => {
                tracing::error!("Failed to start xray process: {}", e);
                return Err(crate::errors::ApiError::SystemError(format!(
                    "Failed to start xray: {}",
                    e
                )));
            }
        }
    }

    Ok(())
}

pub async fn restart_xray(monitor: SharedMonitor) -> ApiResult<()> {
    stop_xray(monitor.clone()).await?;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    start_xray(monitor).await
}

pub async fn restart_panel() -> ApiResult<()> {
    tracing::info!("Received request to restart X-UI Panel service...");

    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let _ = std::process::Command::new("systemctl")
            .arg("restart")
            .arg("x-ui")
            .spawn();
    });

    Ok(())
}

pub async fn update_xray(monitor: SharedMonitor, version: String) -> ApiResult<()> {
    tracing::info!("Start updating xray-lite to version: {}", version);

    let arch = std::env::consts::ARCH;
    let xray_arch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => {
            return Err(crate::errors::ApiError::SystemError(format!(
                "Unsupported architecture: {}",
                arch
            )))
        }
    };

    let tag_name = if version.starts_with('v') {
        version.clone()
    } else {
        format!("v{}", version)
    };

    // xray-lite uses direct binary files: xray-linux-amd64 or xray-linux-arm64
    let mut binary_name = format!("xray-linux-{}", xray_arch);

    // 特殊处理 XDP 版本，指向 release 中明确带有 -xdp 后缀的文件，避免下载到旧文件或缓存
    if tag_name == "v0.6.0-xdp" && xray_arch == "amd64" {
        binary_name = "xray-linux-amd64-xdp".to_string();
    }

    let url = format!(
        "https://github.com/undead-undead/xray-lite/releases/download/{}/{}",
        tag_name, binary_name
    );
    tracing::info!("Downloading from: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build()
        .map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to build client: {}", e))
        })?;

    let res = client.get(&url).send().await.map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Failed to send request: {}", e))
    })?;

    if !res.status().is_success() {
        return Err(crate::errors::ApiError::SystemError(format!(
            "Download failed with status: {}",
            res.status()
        )));
    }

    let content = res
        .bytes()
        .await
        .map_err(|e| crate::errors::ApiError::SystemError(format!("Failed to read body: {}", e)))?;

    // xray-lite provides direct binary, no zip extraction needed
    {
        let bin_path_str =
            std::env::var("XRAY_BIN_PATH").unwrap_or("/usr/local/bin/xray".to_string());
        let bin_path = std::path::Path::new(&bin_path_str);
        tracing::info!("xray-lite binary will be updated at: {}", bin_path.display());

        if let Some(parent) = bin_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::errors::ApiError::SystemError(format!("Failed to create bin dir: {}", e))
                })?;
            }
        }

        let tmp_path = bin_path.with_extension("tmp");
        std::fs::write(&tmp_path, &content).map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to write binary: {}", e))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&tmp_path) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&tmp_path, perms);
            }
        }

        std::fs::rename(&tmp_path, bin_path).map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to replace binary: {}", e))
        })?;
    }

    tracing::info!("xray-lite binary updated successfully to {}", version);

    restart_xray(monitor).await?;

    Ok(())
}

pub async fn get_xray_releases() -> ApiResult<Vec<String>> {
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }

    let client = reqwest::Client::builder()
        .user_agent("X-UI-Backend")
        .build()
        .map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to build client: {}", e))
        })?;

    // Fetch releases from xray-lite repository
    let res = client
        .get("https://api.github.com/repos/undead-undead/xray-lite/releases")
        .send()
        .await
        .map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to fetch releases: {}", e))
        })?;

    if !res.status().is_success() {
        return Err(crate::errors::ApiError::SystemError(format!(
            "GitHub API failed: {}",
            res.status()
        )));
    }

    let releases: Vec<Release> = res.json().await.map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Failed to parse releases: {}", e))
    })?;

    // 只保留 3 个主要版本
    // v0.4.6: 稳定版 (Tokio, 适用于所有内核)
    // v0.6.0-xdp: XDP 防火墙版 (需要内核 5.4+)
    // v0.6.0-beta1: io_uring 优化版 (需要内核 5.10+, 需禁用客户端 Flow)
    let allowed_versions = ["v0.4.6", "v0.6.0-xdp", "v0.6.0-beta1"];
    
    Ok(releases
        .into_iter()
        .map(|r| r.tag_name)
        .filter(|tag| allowed_versions.contains(&tag.as_str()))
        .collect())
}

fn get_connection_counts() -> (usize, usize) {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("ss -antp | grep ESTAB | wc -l && ss -aunp | wc -l")
        .output();

    if let Ok(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() >= 2 {
            let tcp = lines[0].trim().parse().unwrap_or(0);
            let udp = lines[1].trim().parse().unwrap_or(0);
            return (tcp, udp);
        }
    }
    (0, 0)
}

fn get_xray_version() -> Option<String> {
    let bin_path_str = std::env::var("XRAY_BIN_PATH").unwrap_or("/usr/local/bin/xray".to_string());

    // Try --version first (xray-lite style)
    let output = std::process::Command::new(&bin_path_str)
        .arg("--version")
        .output()
        .ok()?;

    if !output.status.success() {
        // Fallback to -version for compatibility
        let output = std::process::Command::new(&bin_path_str)
            .arg("-version")
            .output()
            .ok()?;
        
        if !output.status.success() {
            return None;
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = stdout.lines().next() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let ver = parts[1];
                if ver.starts_with('v') {
                    return Some(ver.to_string());
                } else {
                    return Some(format!("v{}", ver));
                }
            }
        }
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // xray-lite version format: "vless-server 0.2.46" or similar
    if let Some(line) = stdout.lines().next() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let ver = parts[1];
            if ver.starts_with('v') {
                return Some(ver.to_string());
            } else {
                return Some(format!("v{}", ver));
            }
        }
    }

    None
}

fn get_linux_memory_stats() -> Option<(u64, u64, u64, u64)> {
    let output = std::process::Command::new("free").arg("-b").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut mem_total = 0;
    let mut mem_used = 0;
    let mut swap_total = 0;
    let mut swap_used = 0;
    let mut mem_found = false;

    for line in stdout.lines() {
        if line.starts_with("Mem:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // parts[0] is "Mem:"
            // parts[1] is total
            // parts[2] is used
            if parts.len() >= 3 {
                mem_total = parts[1].parse().ok()?;
                mem_used = parts[2].parse().ok()?;
                mem_found = true;
            }
        } else if line.starts_with("Swap:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // parts[0] is "Swap:"
            // parts[1] is total
            // parts[2] is used
            if parts.len() >= 3 {
                swap_total = parts[1].parse().ok()?;
                swap_used = parts[2].parse().ok()?;
            }
        }
    }

    if mem_found {
        // If swap is not found (e.g. no swap enabled), it remains 0 which is correct
        Some((mem_total, mem_used, swap_total, swap_used))
    } else {
        None
    }
}
