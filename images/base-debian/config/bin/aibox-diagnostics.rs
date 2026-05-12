mod aibox_status_core;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use aibox_status_core::{write_snapshot, ProcScanMode, Snapshot};

fn main() {
    let config = Config::from_args(std::env::args().skip(1).collect());
    let mut events =
        RuntimeEventLogger::new(config.event_log_path.clone(), config.event_interval_secs);

    loop {
        let snapshot = Snapshot::collect(ProcScanMode::Detailed);
        if let Err(error) = write_snapshot(&config.state_dir, config.ring_size, &snapshot) {
            eprintln!(
                "aibox-diagnostics: failed to write {}: {error}",
                config.state_dir.display()
            );
            std::process::exit(1);
        }
        if let Err(error) = events.observe(&snapshot) {
            eprintln!(
                "aibox-diagnostics: failed to write {}: {error}",
                config.event_log_path.display()
            );
        }

        if !config.watch {
            let _ = events.flush();
            break;
        }
        thread::sleep(Duration::from_secs(config.interval_secs.max(1)));
    }
}

#[derive(Debug)]
struct Config {
    state_dir: PathBuf,
    event_log_path: PathBuf,
    ring_size: usize,
    interval_secs: u64,
    event_interval_secs: u64,
    watch: bool,
}

impl Config {
    fn from_args(args: Vec<String>) -> Self {
        let mut config = Self {
            state_dir: std::env::var("AIBOX_DIAGNOSTICS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/workspace/.aibox/diagnostics")),
            event_log_path: std::env::var("AIBOX_RUNTIME_EVENTS_LOG")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/workspace/.aibox/runtime-events.log")),
            ring_size: std::env::var("AIBOX_DIAGNOSTICS_RING_SIZE")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(12),
            interval_secs: std::env::var("AIBOX_DIAGNOSTICS_INTERVAL")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(5),
            event_interval_secs: std::env::var("AIBOX_RUNTIME_EVENT_INTERVAL")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(60),
            watch: false,
        };

        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--watch" => config.watch = true,
                "--once" => config.watch = false,
                "--state-dir" | "--output" => {
                    if let Some(value) = iter.next() {
                        config.state_dir = PathBuf::from(value);
                    } else {
                        exit_usage("--state-dir/--output requires a path");
                    }
                }
                "--event-log" => {
                    if let Some(value) = iter.next() {
                        config.event_log_path = PathBuf::from(value);
                    } else {
                        exit_usage("--event-log requires a path");
                    }
                }
                "--ring-size" => {
                    if let Some(value) = iter.next().and_then(|value| value.parse::<usize>().ok()) {
                        config.ring_size = value.max(1);
                    } else {
                        exit_usage("--ring-size requires a positive integer");
                    }
                }
                "--interval" => {
                    if let Some(value) = iter.next().and_then(|value| value.parse::<u64>().ok()) {
                        config.interval_secs = value.max(1);
                    } else {
                        exit_usage("--interval requires a positive integer");
                    }
                }
                "--event-interval" => {
                    if let Some(value) = iter.next().and_then(|value| value.parse::<u64>().ok()) {
                        config.event_interval_secs = value.max(1);
                    } else {
                        exit_usage("--event-interval requires a positive integer");
                    }
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => exit_usage(&format!("unknown argument: {other}")),
            }
        }

        config
    }
}

fn exit_usage(message: &str) -> ! {
    eprintln!("aibox-diagnostics: {message}");
    print_help();
    std::process::exit(2);
}

fn print_help() {
    eprintln!(
        "Usage: aibox-diagnostics [--once|--watch] [--state-dir PATH|--output PATH] [--event-log PATH] [--ring-size N] [--interval SECONDS] [--event-interval SECONDS]\n\nWrites latest.json plus a bounded snapshot-NN.json ring from direct runtime diagnostics, and a low-volume runtime-events.log stream."
    );
}

#[derive(Debug, Default)]
struct RuntimeEventLogger {
    path: PathBuf,
    session_id: Option<String>,
    runtime_started_at: Option<String>,
    aggregate_secs: u64,
    bucket_start: Option<u64>,
    samples: u64,
    memory_sum: u64,
    memory_max: u64,
    process_sum: u64,
    process_max: u64,
    thread_sum: u64,
    thread_max: u64,
    load_sum: f64,
    load_max: f64,
    last_degraded: Option<bool>,
    last_oom_kill: Option<u64>,
    last_oom_events: Option<u64>,
    last_memory_high: Option<u64>,
    last_memory_max_events: Option<u64>,
    last_processkit_display: Option<String>,
    emitted_start: bool,
}

impl RuntimeEventLogger {
    fn new(path: PathBuf, aggregate_secs: u64) -> Self {
        let (session_id, runtime_started_at) = read_runtime_session(Path::new("/workspace"));
        Self {
            path,
            session_id,
            runtime_started_at,
            aggregate_secs: aggregate_secs.max(1),
            ..Self::default()
        }
    }

    fn observe(&mut self, snapshot: &Snapshot) -> io::Result<()> {
        if !self.emitted_start {
            self.emit(
                snapshot,
                "INFO",
                "runtime.diagnostics.started",
                "diagnostics sidecar started",
            )?;
            self.emitted_start = true;
        }

        self.emit_state_changes(snapshot)?;
        self.add_sample(snapshot);

        let bucket_start = self.bucket_start.unwrap_or(snapshot.timestamp_unix);
        if snapshot.timestamp_unix.saturating_sub(bucket_start) >= self.aggregate_secs {
            self.flush()?;
        }
        Ok(())
    }

    fn add_sample(&mut self, snapshot: &Snapshot) {
        if self.bucket_start.is_none() {
            self.bucket_start = Some(snapshot.timestamp_unix);
        }
        self.samples += 1;
        let memory = parse_bytes(&snapshot.memory_current).unwrap_or(0);
        self.memory_sum += memory;
        self.memory_max = self.memory_max.max(memory);
        let processes = snapshot.processes.parse::<u64>().unwrap_or(0);
        self.process_sum += processes;
        self.process_max = self.process_max.max(processes);
        let threads = snapshot.threads.parse::<u64>().unwrap_or(0);
        self.thread_sum += threads;
        self.thread_max = self.thread_max.max(threads);
        let load = snapshot.load_average.parse::<f64>().unwrap_or(0.0);
        self.load_sum += load;
        if load > self.load_max {
            self.load_max = load;
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.samples == 0 {
            return Ok(());
        }
        let samples = self.samples;
        let message = format!(
            "sample {}s samples={} mem_avg={} mem_max={} proc_avg={} proc_max={} threads_avg={} threads_max={} load_avg={:.2} load_max={:.2}",
            self.aggregate_secs,
            samples,
            format_bytes(self.memory_sum / samples),
            format_bytes(self.memory_max),
            self.process_sum / samples,
            self.process_max,
            self.thread_sum / samples,
            self.thread_max,
            self.load_sum / samples as f64,
            self.load_max,
        );
        let timestamp = self.bucket_start.unwrap_or_else(|| current_unix());
        append_event(
            &self.path,
            timestamp,
            "INFO",
            "runtime.sample",
            &message,
            self.session_id.as_deref(),
            self.runtime_started_at.as_deref(),
        )?;
        self.bucket_start = None;
        self.samples = 0;
        self.memory_sum = 0;
        self.memory_max = 0;
        self.process_sum = 0;
        self.process_max = 0;
        self.thread_sum = 0;
        self.thread_max = 0;
        self.load_sum = 0.0;
        self.load_max = 0.0;
        Ok(())
    }

    fn emit_state_changes(&mut self, snapshot: &Snapshot) -> io::Result<()> {
        if let Some(previous) = self.last_degraded {
            if previous != snapshot.degraded {
                let level = if snapshot.degraded { "WARN" } else { "INFO" };
                self.emit(
                    snapshot,
                    level,
                    "runtime.degraded.changed",
                    &format!("degraded {} -> {}", previous, snapshot.degraded),
                )?;
            }
        }
        self.last_degraded = Some(snapshot.degraded);

        self.emit_counter_increase(snapshot, "oom_kill", &snapshot.oom_kill)?;
        self.emit_counter_increase(snapshot, "oom", &snapshot.oom_events)?;
        self.emit_counter_increase(snapshot, "memory_high", &snapshot.memory_high)?;
        self.emit_counter_increase(snapshot, "memory_max", &snapshot.memory_max_events)?;

        if let Some(previous) = &self.last_processkit_display {
            if previous != &snapshot.processkit_display {
                let level = if snapshot.processkit_display == "none"
                    || snapshot.processkit_display == "degraded"
                {
                    "WARN"
                } else {
                    "INFO"
                };
                self.emit(
                    snapshot,
                    level,
                    "runtime.mcp.changed",
                    &format!("mcp {} -> {}", previous, snapshot.processkit_display),
                )?;
            }
        }
        self.last_processkit_display = Some(snapshot.processkit_display.clone());
        Ok(())
    }

    fn emit_counter_increase(
        &mut self,
        snapshot: &Snapshot,
        name: &str,
        value: &str,
    ) -> io::Result<()> {
        let current = value.parse::<u64>().unwrap_or(0);
        let mut previous_value = None;
        let slot = match name {
            "oom_kill" => &mut self.last_oom_kill,
            "oom" => &mut self.last_oom_events,
            "memory_high" => &mut self.last_memory_high,
            "memory_max" => &mut self.last_memory_max_events,
            _ => return Ok(()),
        };
        if let Some(previous) = *slot {
            if current > previous {
                previous_value = Some(previous);
            }
        }
        *slot = Some(current);
        if let Some(previous) = previous_value {
            self.emit(
                snapshot,
                "ERROR",
                &format!("runtime.{name}.increased"),
                &format!("{name} {} -> {}", previous, current),
            )?;
        }
        Ok(())
    }

    fn emit(&self, snapshot: &Snapshot, level: &str, event: &str, message: &str) -> io::Result<()> {
        append_event(
            &self.path,
            snapshot.timestamp_unix,
            level,
            event,
            message,
            self.session_id.as_deref(),
            self.runtime_started_at.as_deref(),
        )
    }
}

fn append_event(
    path: &Path,
    timestamp_unix: u64,
    level: &str,
    event: &str,
    message: &str,
    runtime_session_id: Option<&str>,
    runtime_started_at: Option<&str>,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    rotate_if_needed(path, 1_048_576)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(
        file,
        "{{\"timestamp_unix\":{},\"source\":\"runtime\",\"level\":{},\"event\":{},\"msg\":{},\"runtime_session_id\":{},\"runtime_started_at\":{}}}",
        timestamp_unix,
        json_string(level),
        json_string(event),
        json_string(message),
        json_optional(runtime_session_id),
        json_optional(runtime_started_at),
    )
}

fn rotate_if_needed(path: &Path, max_bytes: u64) -> io::Result<()> {
    if path.exists() && fs::metadata(path)?.len() > max_bytes {
        fs::rename(path, path.with_extension("log.1"))?;
    }
    Ok(())
}

fn read_runtime_session(project_root: &Path) -> (Option<String>, Option<String>) {
    let path = project_root.join(".aibox/runtime-session.json");
    let Ok(text) = fs::read_to_string(path) else {
        return (None, None);
    };
    (
        extract_json_string(&text, "runtime_session_id"),
        extract_json_string(&text, "container_started_at"),
    )
}

fn extract_json_string(text: &str, field: &str) -> Option<String> {
    let needle = format!("\"{}\":\"", field);
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_optional(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".to_string())
}

fn current_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn parse_bytes(value: &str) -> Option<u64> {
    let mut parts = value.split_whitespace();
    let amount = parts.next()?.parse::<f64>().ok()?;
    let unit = parts.next().unwrap_or("B");
    let multiplier = match unit {
        "B" => 1.0,
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        "TiB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((amount * multiplier) as u64)
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.1} MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.1} KiB", b / KIB)
    } else {
        format!("{bytes} B")
    }
}
