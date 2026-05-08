// This std-only module is compiled directly into two small binaries with rustc;
// each binary intentionally uses a different subset of the shared helpers.
#![allow(dead_code)]

use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs::File, process::Command};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";
const PROC_ROOT: &str = "/proc";
const WORKSPACE: &str = "/workspace";
const DEGRADED_PID_THRESHOLD: u64 = 800;
const PROC_SCAN_LIMIT: usize = 2048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcScanMode {
    Minimal,
    Detailed,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub timestamp_unix: u64,
    pub degraded: bool,
    pub host: String,
    pub memory_current: String,
    pub memory_max: String,
    pub oom_kill: String,
    pub oom_events: String,
    pub memory_high: String,
    pub memory_max_events: String,
    pub cpu_throttling: String,
    pub load_average: String,
    pub net: String,
    pub processes: String,
    pub ai_agents: String,
    pub processkit_mode: String,
    pub processkit_mcp: String,
    pub disk_used: String,
    pub disk_total: String,
    pub log_info: String,
    pub log_warn: String,
    pub log_error: String,
    pub container_uptime: String,
    pub git_branch: String,
    pub git_state: String,
    pub migrations: String,
}

impl Snapshot {
    pub fn collect(mode: ProcScanMode) -> Self {
        let pid_count = read_u64(Path::new(CGROUP_ROOT).join("pids.current"))
            .unwrap_or_else(|| count_proc_entries(Path::new(PROC_ROOT)) as u64);
        let degraded = pid_count >= degraded_pid_threshold();
        let detailed = mode == ProcScanMode::Detailed && !degraded;
        let (disk_used, disk_total) = read_workspace_disk();
        let (log_info, log_warn, log_error) = read_log_counts();
        let proc_details = if detailed {
            read_process_details(Path::new(PROC_ROOT))
        } else if degraded {
            ProcessDetails::degraded()
        } else {
            ProcessDetails::minimal()
        };

        Self {
            timestamp_unix: unix_now(),
            degraded,
            host: read_hostname(),
            memory_current: format_bytes(
                read_u64(Path::new(CGROUP_ROOT).join("memory.current")).unwrap_or(0),
            ),
            memory_max: read_memory_max(Path::new(CGROUP_ROOT).join("memory.max")),
            oom_kill: read_cgroup_event("oom_kill"),
            oom_events: read_cgroup_event("oom"),
            memory_high: read_cgroup_event("high"),
            memory_max_events: read_cgroup_event("max"),
            cpu_throttling: read_cpu_throttling(Path::new(CGROUP_ROOT).join("cpu.stat")),
            load_average: read_load_average(Path::new(PROC_ROOT).join("loadavg")),
            net: "n/a".to_string(),
            processes: pid_count.to_string(),
            ai_agents: proc_details.ai_agents.to_string(),
            processkit_mode: proc_details.processkit_mode,
            processkit_mcp: proc_details.processkit_mcp.to_string(),
            disk_used,
            disk_total,
            log_info: log_info.to_string(),
            log_warn: log_warn.to_string(),
            log_error: log_error.to_string(),
            container_uptime: read_container_uptime(Path::new(PROC_ROOT)),
            git_branch: read_git_branch(Path::new(WORKSPACE)),
            git_state: read_git_state(Path::new(WORKSPACE)),
            migrations: count_open_migrations(Path::new(WORKSPACE)).to_string(),
        }
    }

    pub fn plain(&self) -> String {
        format!(
            "AIBOX HOST {} | MEM {}/{} OOM {}/{} HI {} MAX {} | CPU {} LOAD {} NET {} | DISK {}/{} | LOG {}/{}/{} | PROC {} AI {} MCP {}/{} MIG {} DEG {} UP {} | GIT {}:{}",
            self.host,
            self.memory_current,
            self.memory_max,
            self.oom_events,
            self.oom_kill,
            self.memory_high,
            self.memory_max_events,
            self.cpu_throttling,
            self.load_average,
            self.net,
            self.disk_used,
            self.disk_total,
            self.log_info,
            self.log_warn,
            self.log_error,
            self.processes,
            self.ai_agents,
            self.processkit_mode,
            self.processkit_mcp,
            self.migrations,
            if self.degraded { "yes" } else { "no" },
            self.container_uptime,
            self.git_branch,
            self.git_state,
        )
    }

    pub fn json(&self) -> String {
        let plain = self.plain();
        format!(
            "{{\"timestamp_unix\":{},\"degraded\":{},\"host\":{},\"memory_current\":{},\"memory_max\":{},\"oom_events\":{},\"oom_kill\":{},\"memory_high\":{},\"memory_max_events\":{},\"cpu_throttling\":{},\"load_average\":{},\"net\":{},\"processes\":{},\"ai_agents\":{},\"processkit_mode\":{},\"processkit_mcp\":{},\"disk_used\":{},\"disk_total\":{},\"log_info\":{},\"log_warn\":{},\"log_error\":{},\"container_uptime\":{},\"git_branch\":{},\"git_state\":{},\"migrations\":{},\"plain\":{}}}",
            self.timestamp_unix,
            self.degraded,
            json_string(&self.host),
            json_string(&self.memory_current),
            json_string(&self.memory_max),
            json_string(&self.oom_events),
            json_string(&self.oom_kill),
            json_string(&self.memory_high),
            json_string(&self.memory_max_events),
            json_string(&self.cpu_throttling),
            json_string(&self.load_average),
            json_string(&self.net),
            json_string(&self.processes),
            json_string(&self.ai_agents),
            json_string(&self.processkit_mode),
            json_string(&self.processkit_mcp),
            json_string(&self.disk_used),
            json_string(&self.disk_total),
            json_string(&self.log_info),
            json_string(&self.log_warn),
            json_string(&self.log_error),
            json_string(&self.container_uptime),
            json_string(&self.git_branch),
            json_string(&self.git_state),
            json_string(&self.migrations),
            json_string(&plain),
        )
    }

    pub fn styled(&self) -> String {
        if std::env::var_os("NO_COLOR").is_some()
            || std::env::var("AIBOX_STATUS_STYLE").as_deref() == Ok("plain")
        {
            return self.plain();
        }

        format!(
            "\u{1b}[7m AIBOX \u{1b}[27m \u{1b}[2m HOST \u{1b}[22m\u{1b}[1m{}\u{1b}[22m  \u{1b}[2mMEM \u{1b}[22m\u{1b}[1m{}\u{1b}[22m/{} \u{1b}[2mOOM \u{1b}[22m\u{1b}[1m{}/{}\u{1b}[22m \u{1b}[2mHI \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mMAX \u{1b}[22m\u{1b}[1m{}\u{1b}[22m  \u{1b}[2mCPU \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mLOAD \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mNET \u{1b}[22m\u{1b}[1m{}\u{1b}[22m  \u{1b}[2mDISK \u{1b}[22m\u{1b}[1m{}/{}\u{1b}[22m  \u{1b}[2mLOG \u{1b}[22m\u{1b}[1m{}/{}/{}\u{1b}[22m  \u{1b}[2mPROC \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mAI \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mMCP \u{1b}[22m\u{1b}[1m{} {}\u{1b}[22m \u{1b}[2mMIG \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mDEG \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mUP \u{1b}[22m\u{1b}[1m{}\u{1b}[22m",
            self.host,
            self.memory_current,
            self.memory_max,
            self.oom_events,
            self.oom_kill,
            self.memory_high,
            self.memory_max_events,
            self.cpu_throttling,
            self.load_average,
            self.net,
            self.disk_used,
            self.disk_total,
            self.log_info,
            self.log_warn,
            self.log_error,
            self.processes,
            self.ai_agents,
            self.processkit_mode,
            self.processkit_mcp,
            self.migrations,
            if self.degraded { "yes" } else { "no" },
            self.container_uptime,
        )
    }
}

#[derive(Clone, Debug)]
struct ProcessDetails {
    ai_agents: u64,
    processkit_mode: String,
    processkit_mcp: u64,
}

impl ProcessDetails {
    fn minimal() -> Self {
        Self {
            ai_agents: 0,
            processkit_mode: "unknown".to_string(),
            processkit_mcp: 0,
        }
    }

    fn degraded() -> Self {
        Self {
            ai_agents: 0,
            processkit_mode: "degraded".to_string(),
            processkit_mcp: 0,
        }
    }
}

pub fn write_snapshot(dir: &Path, ring_size: usize, snapshot: &Snapshot) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let payload = format!("{}\n", snapshot.json());
    write_atomic(&dir.join("latest.json"), &payload)?;

    let ring_size = ring_size.max(1);
    let slot = (snapshot.timestamp_unix as usize) % ring_size;
    write_atomic(&dir.join(format!("snapshot-{slot:02}.json")), &payload)?;
    write_atomic(&dir.join("ring-size"), &format!("{ring_size}\n"))?;
    Ok(())
}

fn write_atomic(path: &Path, payload: &str) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, payload)?;
    fs::rename(tmp, path)
}

fn read_process_details(proc_root: &Path) -> ProcessDetails {
    let mut details = ProcessDetails {
        ai_agents: 0,
        processkit_mode: "none".to_string(),
        processkit_mcp: 0,
    };
    let entries = match fs::read_dir(proc_root) {
        Ok(entries) => entries,
        Err(_) => return details,
    };

    for entry in entries.flatten().take(PROC_SCAN_LIMIT) {
        let name = entry.file_name();
        if !name
            .to_string_lossy()
            .bytes()
            .all(|byte| byte.is_ascii_digit())
        {
            continue;
        }
        let cmdline = match read_cmdline_limited(&entry.path().join("cmdline")) {
            Some(cmdline) => cmdline.to_ascii_lowercase(),
            None => continue,
        };
        if contains_ai_agent(&cmdline) {
            details.ai_agents += 1;
        }
        if cmdline.contains("processkit-gateway") && cmdline.contains("server.py") {
            details.processkit_mode = "gateway".to_string();
            details.processkit_mcp += 1;
        } else if cmdline.contains("processkit") && cmdline.contains("mcp") {
            if details.processkit_mode != "gateway" {
                details.processkit_mode = "granular".to_string();
            }
            details.processkit_mcp += 1;
        }
    }

    details
}

fn read_cmdline_limited(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let text = bytes
        .into_iter()
        .take(4096)
        .map(|byte| if byte == 0 { b' ' } else { byte })
        .collect::<Vec<_>>();
    String::from_utf8(text).ok()
}

fn contains_ai_agent(cmdline: &str) -> bool {
    [
        "/codex ",
        " codex ",
        "claude ",
        "gemini ",
        "aider ",
        "copilot ",
        "opencode ",
        "hermes ",
    ]
    .iter()
    .any(|needle| cmdline.contains(needle))
}

fn read_memory_max(path: PathBuf) -> String {
    match read_trimmed(path) {
        Some(value) if value == "max" => "unlimited".to_string(),
        Some(value) => value
            .parse::<u64>()
            .map(format_bytes)
            .unwrap_or_else(|_| "unavailable".to_string()),
        None => "unavailable".to_string(),
    }
}

fn read_cgroup_event(name: &str) -> String {
    let events = match read_trimmed(Path::new(CGROUP_ROOT).join("memory.events")) {
        Some(events) => events,
        None => return "0".to_string(),
    };
    for line in events.lines() {
        let mut parts = line.split_whitespace();
        if parts.next() == Some(name) {
            return parts.next().unwrap_or("0").to_string();
        }
    }
    "0".to_string()
}

fn read_cpu_throttling(path: PathBuf) -> String {
    let stat = match read_trimmed(path) {
        Some(stat) => stat,
        None => return "n/a".to_string(),
    };
    let mut throttled = 0;
    let mut throttled_usec = 0;
    for line in stat.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("nr_throttled") => throttled = parts.next().and_then(parse_u64).unwrap_or(0),
            Some("throttled_usec") => {
                throttled_usec = parts.next().and_then(parse_u64).unwrap_or(0)
            }
            _ => {}
        }
    }
    format!("{}/{}s", throttled, throttled_usec / 1_000_000)
}

fn read_load_average(path: PathBuf) -> String {
    read_trimmed(path)
        .and_then(|load| load.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "n/a".to_string())
}

fn read_hostname() -> String {
    read_trimmed(PathBuf::from("/etc/hostname")).unwrap_or_else(|| "n/a".to_string())
}

fn read_workspace_disk() -> (String, String) {
    let output = match Command::new("df").args(["-B1", WORKSPACE]).output() {
        Ok(output) if output.status.success() => output,
        _ => return ("n/a".to_string(), "n/a".to_string()),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let _ = lines.next();
    let Some(line) = lines.next() else {
        return ("n/a".to_string(), "n/a".to_string());
    };
    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 3 {
        return ("n/a".to_string(), "n/a".to_string());
    }
    let total = parts
        .get(1)
        .and_then(|value| value.parse::<u64>().ok())
        .map(format_bytes)
        .unwrap_or_else(|| "n/a".to_string());
    let used = parts
        .get(2)
        .and_then(|value| value.parse::<u64>().ok())
        .map(format_bytes)
        .unwrap_or_else(|| "n/a".to_string());
    (used, total)
}

fn read_log_counts() -> (u64, u64, u64) {
    let path = Path::new(WORKSPACE).join(".aibox/aibox.log");
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return (0, 0, 0),
    };
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    const LIMIT: u64 = 256 * 1024;
    if size > LIMIT {
        let _ = file.seek(SeekFrom::Start(size - LIMIT));
    }
    let mut text = String::new();
    if file.read_to_string(&mut text).is_err() {
        return (0, 0, 0);
    }
    let mut info = 0;
    let mut warn = 0;
    let mut error = 0;
    for line in text.lines() {
        if line.contains("\"level\":\"INFO\"") || line.contains("\"level\":\"info\"") {
            info += 1;
        } else if line.contains("\"level\":\"WARN\"") || line.contains("\"level\":\"warn\"") {
            warn += 1;
        } else if line.contains("\"level\":\"ERROR\"") || line.contains("\"level\":\"error\"") {
            error += 1;
        } else if line.contains("\"exit_code\":0") {
            info += 1;
        } else if line.contains("\"exit_code\":") {
            error += 1;
        }
    }
    (info, warn, error)
}

fn read_container_uptime(proc_root: &Path) -> String {
    let uptime = match read_trimmed(proc_root.join("uptime"))
        .and_then(|text| text.split_whitespace().next()?.parse::<f64>().ok())
    {
        Some(uptime) => uptime,
        None => return "n/a".to_string(),
    };
    let stat = match read_trimmed(proc_root.join("1/stat")) {
        Some(stat) => stat,
        None => return "n/a".to_string(),
    };
    let start_ticks = stat
        .split_whitespace()
        .nth(21)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    let elapsed = (uptime - (start_ticks / 100.0)).max(0.0) as u64;
    format_duration(elapsed)
}

fn read_git_branch(workspace: &Path) -> String {
    let head = match read_trimmed(workspace.join(".git/HEAD")) {
        Some(head) => head,
        None => return "n/a".to_string(),
    };
    head.strip_prefix("ref: refs/heads/")
        .unwrap_or("detached")
        .to_string()
}

fn read_git_state(workspace: &Path) -> String {
    if workspace.join(".git").exists() {
        "unknown".to_string()
    } else {
        "n/a".to_string()
    }
}

fn count_open_migrations(workspace: &Path) -> usize {
    [
        workspace.join("context/migrations/pending"),
        workspace.join("context/migrations/in-progress"),
    ]
    .iter()
    .map(|dir| count_markdown_files(dir))
    .sum()
}

fn count_markdown_files(dir: &Path) -> usize {
    fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
                .count()
        })
        .unwrap_or(0)
}

fn count_proc_entries(proc_root: &Path) -> usize {
    fs::read_dir(proc_root)
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .bytes()
                        .all(|byte| byte.is_ascii_digit())
                })
                .count()
        })
        .unwrap_or(0)
}

fn read_u64(path: PathBuf) -> Option<u64> {
    read_trimmed(path).and_then(|value| value.parse::<u64>().ok())
}

fn read_trimmed(path: PathBuf) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
}

fn parse_u64(value: &str) -> Option<u64> {
    value.parse::<u64>().ok()
}

fn degraded_pid_threshold() -> u64 {
    std::env::var("AIBOX_DIAGNOSTICS_DEGRADED_PIDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEGRADED_PID_THRESHOLD)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{days}d{hours}h")
    } else if hours > 0 {
        format!("{hours}h{minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => output.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => output.push(ch),
        }
    }
    output.push('"');
    output
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::Snapshot;

    fn sample_snapshot() -> Snapshot {
        Snapshot {
            timestamp_unix: 1,
            degraded: false,
            host: "host-a".to_string(),
            memory_current: "1.0 GiB".to_string(),
            memory_max: "2.0 GiB".to_string(),
            oom_kill: "0".to_string(),
            oom_events: "0".to_string(),
            memory_high: "3".to_string(),
            memory_max_events: "4".to_string(),
            cpu_throttling: "2/1s".to_string(),
            load_average: "0.22".to_string(),
            net: "n/a".to_string(),
            processes: "99".to_string(),
            ai_agents: "3".to_string(),
            processkit_mode: "gateway".to_string(),
            processkit_mcp: "12".to_string(),
            disk_used: "10.0 GiB".to_string(),
            disk_total: "100.0 GiB".to_string(),
            log_info: "9".to_string(),
            log_warn: "1".to_string(),
            log_error: "2".to_string(),
            container_uptime: "12m".to_string(),
            git_branch: "main".to_string(),
            git_state: "unknown".to_string(),
            migrations: "0".to_string(),
        }
    }

    #[test]
    fn plain_contains_operational_labels() {
        let plain = sample_snapshot().plain();
        assert!(plain.contains("AIBOX HOST"));
        assert!(plain.contains("CPU"));
        assert!(plain.contains("LOAD"));
        assert!(plain.contains("NET"));
        assert!(plain.contains("MEM"));
        assert!(plain.contains("DISK 10.0 GiB/100.0 GiB"));
        assert!(plain.contains("LOG 9/1/2"));
        assert!(plain.contains("OOM 0/0"));
        assert!(plain.contains("PROC 99"));
        assert!(plain.contains("AI 3"));
        assert!(plain.contains("MCP gateway/12"));
        assert!(plain.contains("MIG 0"));
    }

    #[test]
    fn json_contains_new_snapshot_fields() {
        let json = sample_snapshot().json();
        assert!(json.contains("\"host\":\"host-a\""));
        assert!(json.contains("\"disk_used\":\"10.0 GiB\""));
        assert!(json.contains("\"disk_total\":\"100.0 GiB\""));
        assert!(json.contains("\"log_info\":\"9\""));
        assert!(json.contains("\"log_warn\":\"1\""));
        assert!(json.contains("\"log_error\":\"2\""));
        assert!(json.contains("\"net\":\"n/a\""));
    }
}
