// This std-only module is compiled directly into two small binaries with rustc;
// each binary intentionally uses a different subset of the shared helpers.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
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
    pub threads: String,
    pub ai_agents: String,
    pub ai_agents_breakdown: BTreeMap<&'static str, u64>,
    pub processkit_mode: String,
    pub processkit_mcp: String,
    pub processkit_display: String,
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
        let process_count = count_proc_entries(Path::new(PROC_ROOT)) as u64;
        let thread_count =
            read_u64(Path::new(CGROUP_ROOT).join("pids.current")).unwrap_or(process_count);
        let degraded = thread_count >= degraded_pid_threshold();
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
        let processkit_display =
            format_processkit_display(&proc_details.processkit_mode, proc_details.processkit_mcp);

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
            processes: process_count.to_string(),
            threads: thread_count.to_string(),
            ai_agents: proc_details.ai_agents.to_string(),
            ai_agents_breakdown: proc_details.ai_agents_breakdown,
            processkit_mode: proc_details.processkit_mode,
            processkit_mcp: proc_details.processkit_mcp.to_string(),
            processkit_display,
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
            "AIBOX HOST {} | MEM {}/{} OOM {}/{} HI {} MAX {} | CPU {} LOAD {} NET {} | DISK {}/{} | LOG {}/{}/{} | PROC {}/{} AI {} MCP {} MIG {} DEG {} UP {} | GIT {}:{}",
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
            self.threads,
            self.ai_agents,
            self.processkit_display,
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
            "{{\"timestamp_unix\":{},\"degraded\":{},\"host\":{},\"memory_current\":{},\"memory_max\":{},\"oom_events\":{},\"oom_kill\":{},\"memory_high\":{},\"memory_max_events\":{},\"cpu_throttling\":{},\"load_average\":{},\"net\":{},\"processes\":{},\"threads\":{},\"ai_agents\":{},\"ai_agents_breakdown\":{},\"processkit_mode\":{},\"processkit_mcp\":{},\"processkit_display\":{},\"disk_used\":{},\"disk_total\":{},\"log_info\":{},\"log_warn\":{},\"log_error\":{},\"container_uptime\":{},\"git_branch\":{},\"git_state\":{},\"migrations\":{},\"plain\":{}}}",
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
            json_string(&self.threads),
            json_string(&self.ai_agents),
            json_breakdown_map(&self.ai_agents_breakdown),
            json_string(&self.processkit_mode),
            json_string(&self.processkit_mcp),
            json_string(&self.processkit_display),
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
            "\u{1b}[7m AIBOX \u{1b}[27m \u{1b}[2m HOST \u{1b}[22m\u{1b}[1m{}\u{1b}[22m  \u{1b}[2mMEM \u{1b}[22m\u{1b}[1m{}\u{1b}[22m/{} \u{1b}[2mOOM \u{1b}[22m\u{1b}[1m{}/{}\u{1b}[22m \u{1b}[2mHI \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mMAX \u{1b}[22m\u{1b}[1m{}\u{1b}[22m  \u{1b}[2mCPU \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mLOAD \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mNET \u{1b}[22m\u{1b}[1m{}\u{1b}[22m  \u{1b}[2mDISK \u{1b}[22m\u{1b}[1m{}/{}\u{1b}[22m  \u{1b}[2mLOG \u{1b}[22m\u{1b}[1m{}/{}/{}\u{1b}[22m  \u{1b}[2mPROC \u{1b}[22m\u{1b}[1m{}/{}\u{1b}[22m \u{1b}[2mAI \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mMCP \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mMIG \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mDEG \u{1b}[22m\u{1b}[1m{}\u{1b}[22m \u{1b}[2mUP \u{1b}[22m\u{1b}[1m{}\u{1b}[22m",
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
            self.threads,
            self.ai_agents,
            self.processkit_display,
            self.migrations,
            if self.degraded { "yes" } else { "no" },
            self.container_uptime,
        )
    }
}

/// All known AI agent family names in alphabetical order.
const AI_FAMILIES: &[&str] = &[
    "aider", "claude", "codex", "copilot", "gemini", "hermes", "opencode",
];

fn empty_breakdown() -> BTreeMap<&'static str, u64> {
    AI_FAMILIES.iter().map(|&f| (f, 0u64)).collect()
}

#[derive(Clone, Debug)]
struct ProcessDetails {
    ai_agents: u64,
    ai_agents_breakdown: BTreeMap<&'static str, u64>,
    processkit_mode: String,
    processkit_mcp: u64,
}

impl ProcessDetails {
    fn minimal() -> Self {
        Self {
            ai_agents: 0,
            ai_agents_breakdown: empty_breakdown(),
            processkit_mode: "unknown".to_string(),
            processkit_mcp: 0,
        }
    }

    fn degraded() -> Self {
        Self {
            ai_agents: 0,
            ai_agents_breakdown: empty_breakdown(),
            processkit_mode: "degraded".to_string(),
            processkit_mcp: 0,
        }
    }
}

fn format_processkit_display(mode: &str, process_count: u64) -> String {
    match mode {
        "daemon" | "gateway" => format!("dmn/1/{process_count}"),
        "stdio" => format!("stdio/1/{process_count}"),
        "separate" | "granular" => format!("sep/1/{process_count}"),
        "none" => "none".to_string(),
        "unknown" => "unkwn".to_string(),
        "degraded" => "degraded".to_string(),
        _ => "unkwn".to_string(),
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
        ai_agents_breakdown: empty_breakdown(),
        processkit_mode: "none".to_string(),
        processkit_mcp: 0,
    };
    let entries = match fs::read_dir(proc_root) {
        Ok(entries) => entries,
        Err(_) => return details,
    };
    let mut ai_seen_families = BTreeSet::new();

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
        if let Some(family) = ai_agent_family(&cmdline) {
            ai_seen_families.insert(family);
            *details.ai_agents_breakdown.entry(family).or_insert(0) += 1;
        }
        if cmdline.contains("processkit-gateway") && cmdline.contains("server.py") {
            let detected_mode = if cmdline.contains("stdio-proxy")
                || cmdline.contains(" streamable-http")
                || contains_command_token(&cmdline, "serve")
            {
                "daemon"
            } else {
                "stdio"
            };
            if processkit_mode_rank(detected_mode) > processkit_mode_rank(&details.processkit_mode)
            {
                details.processkit_mode = detected_mode.to_string();
            }
            details.processkit_mcp += 1;
        } else if cmdline.contains("processkit") && cmdline.contains("mcp") {
            if processkit_mode_rank("separate") > processkit_mode_rank(&details.processkit_mode) {
                details.processkit_mode = "separate".to_string();
            }
            details.processkit_mcp += 1;
        }
    }

    details.ai_agents = ai_seen_families.len() as u64;
    details
}

fn processkit_mode_rank(mode: &str) -> u8 {
    match mode {
        "daemon" | "gateway" => 3,
        "stdio" => 2,
        "separate" | "granular" => 1,
        _ => 0,
    }
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

fn ai_agent_family(cmdline: &str) -> Option<&'static str> {
    [
        "codex", "claude", "gemini", "aider", "copilot", "opencode", "hermes",
    ]
    .iter()
    .find(|family| contains_command_token(cmdline, family))
    .copied()
}

fn contains_command_token(cmdline: &str, needle: &str) -> bool {
    cmdline
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'))
        .any(|token| token == needle)
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

/// Default fallback freshness window for status-bar log counts. Current
/// runtimes prefer `.aibox/runtime-session.json` so the counter means
/// "logs emitted by this running container". The window is only used for
/// older images or damaged runtime-session metadata.
const DEFAULT_LOG_WINDOW_SECS: u64 = 24 * 60 * 60;

fn read_log_counts() -> (u64, u64, u64) {
    let workspace = Path::new(WORKSPACE);
    let runtime_session_id =
        read_runtime_session_id(&workspace.join(".aibox/runtime-session.json"));
    let fallback_window = std::env::var("AIBOX_LOG_WINDOW_HOURS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(|hours| std::time::Duration::from_secs(hours * 3600))
        .unwrap_or_else(|| std::time::Duration::from_secs(DEFAULT_LOG_WINDOW_SECS));
    let fallback_cutoff_unix = unix_now().saturating_sub(
        read_container_uptime_seconds(Path::new(PROC_ROOT)).unwrap_or(fallback_window.as_secs()),
    ) as i64;

    let cli_counts = read_log_counts_at(
        workspace.join(".aibox/aibox.log"),
        runtime_session_id.as_deref(),
        fallback_cutoff_unix,
    );
    let runtime_counts = read_log_counts_at(
        workspace.join(".aibox/runtime-events.log"),
        runtime_session_id.as_deref(),
        fallback_cutoff_unix,
    );
    (
        cli_counts.0 + runtime_counts.0,
        cli_counts.1 + runtime_counts.1,
        cli_counts.2 + runtime_counts.2,
    )
}

fn read_log_counts_at(
    path: PathBuf,
    runtime_session_id: Option<&str>,
    fallback_cutoff_unix: i64,
) -> (u64, u64, u64) {
    let mut file = match File::open(&path) {
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
    // After a non-zero seek we may land mid-record; drop the partial first line.
    let mut iter = text.lines();
    if size > LIMIT {
        let _ = iter.next();
    }

    let (mut info, mut warn, mut error) = (0u64, 0u64, 0u64);
    for line in iter {
        if let Some(current_session) = runtime_session_id {
            if extract_json_string_field(line, "runtime_session_id") != Some(current_session) {
                continue;
            }
        } else {
            // Legacy fallback: skip lines we can't timestamp (defensive)
            // and lines older than this container's start/window cutoff.
            let t = extract_json_string_field(line, "ts")
                .and_then(parse_rfc3339_unix)
                .or_else(|| extract_json_number_field(line, "timestamp_unix").and_then(parse_i64));
            let Some(t) = t else { continue };
            if t < fallback_cutoff_unix {
                continue;
            }
        }

        if !classify_log_line(line, &mut info, &mut warn, &mut error) {
            continue;
        }
    }
    (info, warn, error)
}

fn classify_log_line(line: &str, info: &mut u64, warn: &mut u64, error: &mut u64) -> bool {
    if line.contains("\"level\":\"ERROR\"") || line.contains("\"level\":\"error\"") {
        *error += 1;
    } else if line.contains("\"level\":\"WARN\"") || line.contains("\"level\":\"warn\"") {
        *warn += 1;
    } else if line.contains("\"level\":\"INFO\"") || line.contains("\"level\":\"info\"") {
        *info += 1;
    } else if line.contains("\"exit_code\":0") {
        *info += 1;
    } else if line.contains("\"exit_code\":") {
        *error += 1;
    } else {
        return false;
    }
    true
}

fn read_runtime_session_id(path: &Path) -> Option<String> {
    let text = read_trimmed(path.to_path_buf())?;
    extract_json_string_field(&text, "runtime_session_id")
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

/// Extract the value of a JSON string field from one NDJSON line. Std-only
/// (no serde). Returns `None` for absent fields, fields without quotes,
/// or fields whose value contains an embedded quote (we don't unescape).
/// Sufficient for `ts` since aibox emits `chrono::Utc::now().to_rfc3339()`
/// values with no embedded quotes.
fn extract_json_string_field<'a>(line: &'a str, field: &str) -> Option<&'a str> {
    let needle = format!("\"{}\":\"", field);
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn extract_json_number_field<'a>(line: &'a str, field: &str) -> Option<&'a str> {
    let needle = format!("\"{}\":", field);
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let len = rest
        .bytes()
        .take_while(|byte| byte.is_ascii_digit() || *byte == b'-')
        .count();
    if len == 0 {
        return None;
    }
    Some(&rest[..len])
}

fn parse_i64(value: &str) -> Option<i64> {
    value.parse::<i64>().ok()
}

/// Parse RFC3339 to unix seconds (std-only — this binary is built with
/// plain rustc and has no chrono dependency). Accepts:
///   `YYYY-MM-DDTHH:MM:SS[.fraction]( Z | +HH:MM | -HH:MM )`
/// Returns `None` on any parse failure. Uses Howard Hinnant's
/// civil-from-fields algorithm for the date portion (proleptic
/// Gregorian, valid for years > 0).
fn parse_rfc3339_unix(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let parse_n = |start: usize, len: usize| -> Option<i64> {
        let slice = std::str::from_utf8(&bytes[start..start + len]).ok()?;
        slice.parse::<i64>().ok()
    };
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || (bytes[10] != b'T' && bytes[10] != b' ')
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let year = parse_n(0, 4)?;
    let month = parse_n(5, 2)?;
    let day = parse_n(8, 2)?;
    let hour = parse_n(11, 2)?;
    let minute = parse_n(14, 2)?;
    let second = parse_n(17, 2)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let mut idx = 19;
    if idx < bytes.len() && bytes[idx] == b'.' {
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_digit() {
            idx += 1;
        }
    }
    let offset_secs: i64 = if idx >= bytes.len() {
        0
    } else if bytes[idx] == b'Z' || bytes[idx] == b'z' {
        0
    } else if bytes[idx] == b'+' || bytes[idx] == b'-' {
        let sign: i64 = if bytes[idx] == b'-' { -1 } else { 1 };
        idx += 1;
        if idx + 5 > bytes.len() || bytes[idx + 2] != b':' {
            return None;
        }
        let oh = parse_n(idx, 2)?;
        let om = parse_n(idx + 3, 2)?;
        sign * (oh * 3600 + om * 60)
    } else {
        return None;
    };

    // Howard Hinnant's `days_from_civil` (yyyy-mm-dd → days since epoch).
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y / 400 } else { (y - 399) / 400 };
    let yoe = y - era * 400; // [0, 399]
    let m = month;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468; // days since 1970-01-01

    let local_unix = days * 86400 + hour * 3600 + minute * 60 + second;
    Some(local_unix - offset_secs)
}

fn read_container_uptime(proc_root: &Path) -> String {
    read_container_uptime_seconds(proc_root)
        .map(format_duration)
        .unwrap_or_else(|| "n/a".to_string())
}

fn read_container_uptime_seconds(proc_root: &Path) -> Option<u64> {
    let uptime = match read_trimmed(proc_root.join("uptime"))
        .and_then(|text| text.split_whitespace().next()?.parse::<f64>().ok())
    {
        Some(uptime) => uptime,
        None => return None,
    };
    let stat = match read_trimmed(proc_root.join("1/stat")) {
        Some(stat) => stat,
        None => return None,
    };
    let start_ticks = stat
        .split_whitespace()
        .nth(21)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0);
    let elapsed = (uptime - (start_ticks / 100.0)).max(0.0) as u64;
    Some(elapsed)
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

/// Serialise a `BTreeMap<&str, u64>` as a JSON object.
/// Keys are already sorted (BTreeMap) so output is deterministic.
fn json_breakdown_map(map: &BTreeMap<&'static str, u64>) -> String {
    let mut output = String::from('{');
    let mut first = true;
    for (key, count) in map {
        if !first {
            output.push(',');
        }
        first = false;
        output.push('"');
        output.push_str(key);
        output.push_str("\":");
        output.push_str(&count.to_string());
    }
    output.push('}');
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
            threads: "123".to_string(),
            ai_agents: "3".to_string(),
            ai_agents_breakdown: super::empty_breakdown(),
            processkit_mode: "daemon".to_string(),
            processkit_mcp: "12".to_string(),
            processkit_display: "dmn/1/12".to_string(),
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
        assert!(plain.contains("PROC 99/123"));
        assert!(plain.contains("AI 3"));
        assert!(plain.contains("MCP dmn/1/12"));
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
        assert!(json.contains("\"processes\":\"99\""));
        assert!(json.contains("\"threads\":\"123\""));
        assert!(json.contains("\"processkit_display\":\"dmn/1/12\""));
    }

    #[test]
    fn format_processkit_display_uses_compact_topology_states() {
        assert_eq!(super::format_processkit_display("daemon", 5), "dmn/1/5");
        assert_eq!(super::format_processkit_display("stdio", 5), "stdio/1/5");
        assert_eq!(super::format_processkit_display("separate", 5), "sep/1/5");
        assert_eq!(super::format_processkit_display("none", 0), "none");
        assert_eq!(super::format_processkit_display("unknown", 0), "unkwn");
        assert_eq!(super::format_processkit_display("degraded", 0), "degraded");
    }

    #[test]
    fn read_process_details_counts_distinct_ai_agent_families() {
        let dir = std::env::temp_dir().join(format!(
            "aibox-proc-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for (pid, cmdline) in [
            (
                "1",
                "bash -lc if command -v codex >/dev/null; then codex; fi",
            ),
            ("2", "node /usr/bin/codex"),
            ("3", "/opt/codex/codex"),
            (
                "4",
                "bash -lc if command -v claude >/dev/null; then claude; fi",
            ),
            ("5", "claude"),
            ("6", "uv run processkit-gateway/mcp/server.py stdio-proxy"),
        ] {
            let pid_dir = dir.join(pid);
            std::fs::create_dir_all(&pid_dir).unwrap();
            std::fs::write(pid_dir.join("cmdline"), cmdline.replace(' ', "\0")).unwrap();
        }

        let details = super::read_process_details(&dir);
        assert_eq!(
            details.ai_agents, 2,
            "codex wrappers/binary should count as one family, claude as another"
        );
        assert_eq!(details.processkit_mode, "daemon");
        assert_eq!(details.processkit_mcp, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ai_agents_breakdown_counts_per_process_not_per_family() {
        let dir = std::env::temp_dir().join(format!(
            "aibox-breakdown-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        // Two claude processes, one codex process
        for (pid, cmdline) in [
            ("1", "claude"),
            ("2", "claude --dangerously-skip-permissions"),
            ("3", "node /usr/bin/codex"),
        ] {
            let pid_dir = dir.join(pid);
            std::fs::create_dir_all(&pid_dir).unwrap();
            std::fs::write(pid_dir.join("cmdline"), cmdline.replace(' ', "\0")).unwrap();
        }

        let details = super::read_process_details(&dir);
        // Two distinct families (claude, codex)
        assert_eq!(details.ai_agents, 2, "should count 2 distinct families");
        assert_eq!(details.ai_agents_breakdown["claude"], 2, "two claude processes");
        assert_eq!(details.ai_agents_breakdown["codex"], 1, "one codex process");
        // All other families must be zero
        for family in super::AI_FAMILIES {
            if *family != "claude" && *family != "codex" {
                assert_eq!(
                    details.ai_agents_breakdown[family], 0,
                    "{family} should be zero"
                );
            }
        }
        // JSON snapshot contains the breakdown
        let json = super::json_breakdown_map(&details.ai_agents_breakdown);
        assert!(json.contains("\"claude\":2"), "json must show claude:2");
        assert!(json.contains("\"codex\":1"), "json must show codex:1");
        assert!(json.contains("\"aider\":0"), "json must include zero-count families");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ai_agents_breakdown_all_zeros_when_no_ai_processes() {
        let dir = std::env::temp_dir().join(format!(
            "aibox-breakdown-empty-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        // Only a non-AI process
        let pid_dir = dir.join("1");
        std::fs::create_dir_all(&pid_dir).unwrap();
        std::fs::write(pid_dir.join("cmdline"), "bash\0-c\0echo hello").unwrap();

        let details = super::read_process_details(&dir);
        assert_eq!(details.ai_agents, 0, "no AI families");
        for family in super::AI_FAMILIES {
            assert_eq!(
                details.ai_agents_breakdown[family], 0,
                "{family} should be zero"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_process_details_keeps_daemon_mode_when_stdio_process_is_seen_later() {
        let dir = std::env::temp_dir().join(format!(
            "aibox-proc-mode-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for (pid, cmdline) in [
            ("1", "uv run processkit-gateway/mcp/server.py serve"),
            ("2", "uv run processkit-gateway/mcp/server.py"),
            ("3", "uv run context/skills/processkit/aggregate-mcp/mcp/server.py"),
        ] {
            let pid_dir = dir.join(pid);
            std::fs::create_dir_all(&pid_dir).unwrap();
            std::fs::write(pid_dir.join("cmdline"), cmdline.replace(' ', "\0")).unwrap();
        }

        let details = super::read_process_details(&dir);
        assert_eq!(details.processkit_mode, "daemon");
        assert_eq!(details.processkit_mcp, 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- BR-LOG-PANEL: runtime-session scoped counter freshness ------------

    #[test]
    fn parse_rfc3339_basic_utc_offset() {
        // 2026-05-08T12:00:00+00:00 = ?
        // Compute expected via UNIX_EPOCH + days
        let t = super::parse_rfc3339_unix("2026-05-08T12:00:00+00:00").expect("parse");
        // Sanity: must be > epoch and a 10-digit unix second
        assert!(t > 1_700_000_000);
        // Round-trip basic check
        let secs_per_day: i64 = 86_400;
        assert!((t / secs_per_day) > 20_000);
    }

    #[test]
    fn parse_rfc3339_with_microseconds_and_zulu() {
        let t1 = super::parse_rfc3339_unix("2026-05-08T13:38:00.874664+00:00").unwrap();
        let t2 = super::parse_rfc3339_unix("2026-05-08T13:38:00Z").unwrap();
        assert_eq!(t1, t2, "fractional must not shift the second");
    }

    #[test]
    fn parse_rfc3339_negative_offset_normalizes() {
        let z = super::parse_rfc3339_unix("2026-05-08T12:00:00Z").unwrap();
        let cest = super::parse_rfc3339_unix("2026-05-08T14:00:00+02:00").unwrap();
        assert_eq!(z, cest, "+02:00 wall clock 14:00 == 12:00 UTC");
    }

    #[test]
    fn parse_rfc3339_rejects_garbage() {
        assert!(super::parse_rfc3339_unix("not a date").is_none());
        assert!(super::parse_rfc3339_unix("2026").is_none());
        assert!(super::parse_rfc3339_unix("2026-05-08").is_none()); // too short
    }

    #[test]
    fn extract_json_string_field_works() {
        let line = r#"{"ts":"2026-05-08T12:00:00+00:00","cmd":"sync","exit_code":0}"#;
        assert_eq!(
            super::extract_json_string_field(line, "ts"),
            Some("2026-05-08T12:00:00+00:00")
        );
        assert_eq!(super::extract_json_string_field(line, "cmd"), Some("sync"));
        assert_eq!(super::extract_json_string_field(line, "absent"), None);
    }

    #[test]
    fn read_log_counts_filters_by_runtime_session_id() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!(
            "aibox-log-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aibox.log");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"{{"ts":"2026-05-08T12:00:00+00:00","cmd":"sync","version":"0.25.5","exit_code":1,"duration_ms":10,"msg":"old fail","runtime_session_id":"old-session"}}"#
        )
        .unwrap();
        writeln!(
            f,
            r#"{{"ts":"2026-05-08T12:01:00+00:00","cmd":"sync","version":"0.25.5","exit_code":0,"duration_ms":10,"msg":"ok","runtime_session_id":"current-session"}}"#
        )
        .unwrap();
        writeln!(
            f,
            r#"{{"ts":"2026-05-08T12:02:00+00:00","level":"WARN","msg":"warn","runtime_session_id":"current-session"}}"#
        )
        .unwrap();
        drop(f);

        let (info, warn, error) =
            super::read_log_counts_at(path.clone(), Some("current-session"), 0);
        assert_eq!(
            (info, warn, error),
            (1, 1, 0),
            "only matching runtime_session_id entries should be counted"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_log_counts_falls_back_to_start_cutoff_for_legacy_entries() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!(
            "aibox-log-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("aibox.log");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"{{"ts":"2026-05-08T11:59:00+00:00","cmd":"sync","version":"0.25.5","exit_code":1,"duration_ms":10,"msg":"before start"}}"#
        )
        .unwrap();
        writeln!(
            f,
            r#"{{"ts":"2026-05-08T12:01:00+00:00","cmd":"sync","version":"0.25.5","exit_code":0,"duration_ms":10,"msg":"after start"}}"#
        )
        .unwrap();
        drop(f);

        let cutoff = super::parse_rfc3339_unix("2026-05-08T12:00:00+00:00").unwrap();
        let (info, warn, error) = super::read_log_counts_at(path.clone(), None, cutoff);
        assert_eq!(
            (info, warn, error),
            (1, 0, 0),
            "legacy fallback should count only entries after container start"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_log_counts_accepts_runtime_event_unix_timestamps() {
        use std::io::Write;
        let dir = std::env::temp_dir().join(format!(
            "aibox-runtime-log-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("runtime-events.log");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"{{"timestamp_unix":100,"source":"runtime","level":"INFO","event":"runtime.sample","runtime_session_id":"current-session"}}"#
        )
        .unwrap();
        writeln!(
            f,
            r#"{{"timestamp_unix":101,"source":"runtime","level":"WARN","event":"runtime.mcp.changed","runtime_session_id":"current-session"}}"#
        )
        .unwrap();
        drop(f);

        let (info, warn, error) =
            super::read_log_counts_at(path.clone(), Some("current-session"), 0);
        assert_eq!((info, warn, error), (1, 1, 0));

        let (info, warn, error) = super::read_log_counts_at(path.clone(), None, 101);
        assert_eq!((info, warn, error), (0, 1, 0));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_runtime_session_id_reads_metadata_file() {
        let dir = std::env::temp_dir().join(format!(
            "aibox-session-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("runtime-session.json");
        std::fs::write(
            &path,
            r#"{"runtime_session_id":"session-123","container_started_at":"2026-05-08T12:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(
            super::read_runtime_session_id(&path),
            Some("session-123".to_string())
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_log_counts_returns_zero_for_missing_file() {
        let bogus = std::env::temp_dir().join("definitely-not-here-aibox.log");
        let _ = std::fs::remove_file(&bogus);
        let (info, warn, error) = super::read_log_counts_at(bogus, Some("session"), 0);
        assert_eq!((info, warn, error), (0, 0, 0));
    }
}
