use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::cli::OutputFormat;

const CGROUP_ROOT: &str = "/sys/fs/cgroup";
const PROC_ROOT: &str = "/proc";

/// Runtime memory/process pressure snapshot collected from Linux procfs/cgroupfs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimeResourceDiagnostics {
    /// Current cgroup memory usage in bytes, when `memory.current` is readable.
    pub memory_current_bytes: Option<u64>,
    /// Configured cgroup memory limit, when `memory.max` is readable.
    pub memory_max: Option<MemoryMax>,
    /// Cumulative cgroup OOM kill count, when `memory.events` is readable.
    pub oom_kill_count: Option<u64>,
    /// Count of numeric entries under `/proc`.
    pub total_process_count: usize,
    /// Count of Python processes whose cmdline indicates a processkit MCP server.
    pub processkit_mcp_python_process_count: usize,
}

/// Cgroup memory limit as reported by `memory.max`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryMax {
    Bytes(u64),
    Unlimited,
}

pub fn cmd_runtime_resources(format: OutputFormat) -> Result<()> {
    let diagnostics = read_runtime_resource_diagnostics();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&diagnostics)?);
        }
        OutputFormat::Yaml => {
            print!("{}", serde_yaml::to_string(&diagnostics)?);
        }
        OutputFormat::Table => {
            println!("Runtime resources");
            println!(
                "  Memory current:       {}",
                diagnostics
                    .memory_current_bytes
                    .map(format_bytes)
                    .unwrap_or_else(|| "unavailable".to_string())
            );
            println!(
                "  Memory max:           {}",
                diagnostics
                    .memory_max
                    .map(format_memory_max)
                    .unwrap_or_else(|| "unavailable".to_string())
            );
            println!(
                "  OOM kills:            {}",
                diagnostics
                    .oom_kill_count
                    .map(|count| count.to_string())
                    .unwrap_or_else(|| "unavailable".to_string())
            );
            println!(
                "  Processes:            {}",
                diagnostics.total_process_count
            );
            println!(
                "  processkit MCP Python: {}",
                diagnostics.processkit_mcp_python_process_count
            );
        }
    }

    Ok(())
}

/// Read best-effort runtime resource diagnostics from the current Linux runtime.
///
/// This function intentionally does not call external tools such as `ps` or
/// `free`; it reads cgroupfs and procfs directly and treats missing files as
/// unavailable data.
pub fn read_runtime_resource_diagnostics() -> RuntimeResourceDiagnostics {
    read_runtime_resource_diagnostics_from_paths(Path::new(CGROUP_ROOT), Path::new(PROC_ROOT))
}

fn read_runtime_resource_diagnostics_from_paths(
    cgroup_root: &Path,
    proc_root: &Path,
) -> RuntimeResourceDiagnostics {
    let process_counts = read_process_counts(proc_root);

    RuntimeResourceDiagnostics {
        memory_current_bytes: read_u64_file(&cgroup_root.join("memory.current")),
        memory_max: read_memory_max(&cgroup_root.join("memory.max")),
        oom_kill_count: read_oom_kill_count(&cgroup_root.join("memory.events")),
        total_process_count: process_counts.total,
        processkit_mcp_python_process_count: process_counts.processkit_mcp_python,
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ProcessCounts {
    total: usize,
    processkit_mcp_python: usize,
}

fn read_u64_file(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub fn format_memory_max(value: MemoryMax) -> String {
    match value {
        MemoryMax::Bytes(bytes) => format_bytes(bytes),
        MemoryMax::Unlimited => "unlimited".to_string(),
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn read_memory_max(path: &Path) -> Option<MemoryMax> {
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim();

    if value == "max" {
        Some(MemoryMax::Unlimited)
    } else {
        value.parse().ok().map(MemoryMax::Bytes)
    }
}

fn read_oom_kill_count(path: &Path) -> Option<u64> {
    let events = fs::read_to_string(path).ok()?;
    Some(parse_oom_kill_count(&events))
}

fn parse_oom_kill_count(events: &str) -> u64 {
    events
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let key = parts.next()?;
            let value = parts.next()?;
            (key == "oom_kill").then(|| value.parse::<u64>().ok())?
        })
        .next()
        .unwrap_or(0)
}

fn read_process_counts(proc_root: &Path) -> ProcessCounts {
    let Ok(entries) = fs::read_dir(proc_root) else {
        return ProcessCounts::default();
    };

    let mut counts = ProcessCounts::default();

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };

        if !file_name.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }

        counts.total += 1;

        let cmdline_path = entry.path().join("cmdline");
        if let Some(cmdline) = read_cmdline(&cmdline_path)
            && is_processkit_mcp_python_cmdline(&cmdline)
        {
            counts.processkit_mcp_python += 1;
        }
    }

    counts
}

fn read_cmdline(path: &Path) -> Option<Vec<String>> {
    let bytes = fs::read(path).ok()?;

    let args = bytes
        .split(|byte| *byte == b'\0')
        .filter(|arg| !arg.is_empty())
        .map(|arg| String::from_utf8_lossy(arg).to_string())
        .collect();

    Some(args)
}

fn is_processkit_mcp_python_cmdline(args: &[String]) -> bool {
    let has_python = args
        .iter()
        .flat_map(|arg| arg.split_whitespace())
        .any(|arg| {
            let name = Path::new(arg)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(arg);

            name.to_ascii_lowercase().starts_with("python")
        });

    if !has_python {
        return false;
    }

    let joined = args.join(" ").to_ascii_lowercase();
    joined.contains("processkit") && joined.contains("mcp")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn missing_runtime_files_return_unavailable_values_and_zero_counts() {
        let temp = TestDir::new("missing-runtime-files");
        let missing_cgroup = temp.path().join("missing-cgroup");
        let missing_proc = temp.path().join("missing-proc");

        let diagnostics =
            read_runtime_resource_diagnostics_from_paths(&missing_cgroup, &missing_proc);

        assert_eq!(
            diagnostics,
            RuntimeResourceDiagnostics {
                memory_current_bytes: None,
                memory_max: None,
                oom_kill_count: None,
                total_process_count: 0,
                processkit_mcp_python_process_count: 0,
            }
        );
    }

    #[test]
    fn reads_cgroup_memory_values_and_oom_kill_count() {
        let temp = TestDir::new("cgroup-values");
        let cgroup = temp.path().join("cgroup");
        let proc_root = temp.path().join("proc");
        fs::create_dir_all(&cgroup).unwrap();
        fs::create_dir_all(&proc_root).unwrap();

        fs::write(cgroup.join("memory.current"), "12345\n").unwrap();
        fs::write(cgroup.join("memory.max"), "67890\n").unwrap();
        fs::write(
            cgroup.join("memory.events"),
            "low 0\nhigh 0\nmax 4\noom 2\noom_kill 3\n",
        )
        .unwrap();

        let diagnostics = read_runtime_resource_diagnostics_from_paths(&cgroup, &proc_root);

        assert_eq!(diagnostics.memory_current_bytes, Some(12345));
        assert_eq!(diagnostics.memory_max, Some(MemoryMax::Bytes(67890)));
        assert_eq!(diagnostics.oom_kill_count, Some(3));
    }

    #[test]
    fn reports_unlimited_memory_max() {
        let temp = TestDir::new("unlimited-memory-max");
        let cgroup = temp.path().join("cgroup");
        let proc_root = temp.path().join("proc");
        fs::create_dir_all(&cgroup).unwrap();
        fs::create_dir_all(&proc_root).unwrap();

        fs::write(cgroup.join("memory.max"), "max\n").unwrap();

        let diagnostics = read_runtime_resource_diagnostics_from_paths(&cgroup, &proc_root);

        assert_eq!(diagnostics.memory_max, Some(MemoryMax::Unlimited));
    }

    #[test]
    fn counts_total_processes_and_processkit_mcp_python_processes() {
        let temp = TestDir::new("process-counts");
        let cgroup = temp.path().join("cgroup");
        let proc_root = temp.path().join("proc");
        fs::create_dir_all(&cgroup).unwrap();
        fs::create_dir_all(&proc_root).unwrap();

        write_cmdline(
            &proc_root.join("1"),
            &[b"/usr/bin/bash".as_slice(), b"-l".as_slice()],
        )
        .unwrap();
        write_cmdline(
            &proc_root.join("2"),
            &[
                b"/usr/bin/python3".as_slice(),
                b"context/skills/processkit/index-management/mcp/server.py".as_slice(),
            ],
        )
        .unwrap();
        write_cmdline(
            &proc_root.join("3"),
            &[b"python".as_slice(), b"unrelated.py".as_slice()],
        )
        .unwrap();
        fs::create_dir_all(proc_root.join("4")).unwrap();
        write_cmdline(
            &proc_root.join("not-a-pid"),
            &[
                b"python".as_slice(),
                b"context/skills/processkit/id-management/mcp/server.py".as_slice(),
            ],
        )
        .unwrap();

        let diagnostics = read_runtime_resource_diagnostics_from_paths(&cgroup, &proc_root);

        assert_eq!(diagnostics.total_process_count, 4);
        assert_eq!(diagnostics.processkit_mcp_python_process_count, 1);
    }

    #[test]
    fn processkit_mcp_count_requires_python() {
        let temp = TestDir::new("python-required");
        let proc_root = temp.path().join("proc");
        fs::create_dir_all(&proc_root).unwrap();

        write_cmdline(
            &proc_root.join("1"),
            &[
                b"uv".as_slice(),
                b"run".as_slice(),
                b"context/skills/processkit/index-management/mcp/server.py".as_slice(),
            ],
        )
        .unwrap();

        let counts = read_process_counts(&proc_root);

        assert_eq!(counts.total, 1);
        assert_eq!(counts.processkit_mcp_python, 0);
    }

    #[test]
    fn processkit_mcp_match_accepts_whitespace_separated_cmdline_like_data() {
        let args =
            vec!["python context/skills/processkit/index-management/mcp/server.py".to_string()];

        assert!(is_processkit_mcp_python_cmdline(&args));
    }

    fn write_cmdline(dir: &Path, args: &[&[u8]]) -> io::Result<()> {
        fs::create_dir_all(dir)?;

        let mut bytes = Vec::new();
        for arg in args {
            bytes.extend_from_slice(arg);
            bytes.push(0);
        }

        fs::write(dir.join("cmdline"), bytes)
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "aibox-runtime-resources-{label}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
