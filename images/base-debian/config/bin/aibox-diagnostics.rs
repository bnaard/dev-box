mod aibox_status_core;

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use aibox_status_core::{write_snapshot, ProcScanMode, Snapshot};

fn main() {
    let config = Config::from_args(std::env::args().skip(1).collect());

    loop {
        let snapshot = Snapshot::collect(ProcScanMode::Detailed);
        if let Err(error) = write_snapshot(&config.state_dir, config.ring_size, &snapshot) {
            eprintln!(
                "aibox-diagnostics: failed to write {}: {error}",
                config.state_dir.display()
            );
            std::process::exit(1);
        }

        if !config.watch {
            break;
        }
        thread::sleep(Duration::from_secs(config.interval_secs.max(1)));
    }
}

#[derive(Debug)]
struct Config {
    state_dir: PathBuf,
    ring_size: usize,
    interval_secs: u64,
    watch: bool,
}

impl Config {
    fn from_args(args: Vec<String>) -> Self {
        let mut config = Self {
            state_dir: std::env::var("AIBOX_DIAGNOSTICS_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("/workspace/.aibox/diagnostics")),
            ring_size: std::env::var("AIBOX_DIAGNOSTICS_RING_SIZE")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(12),
            interval_secs: std::env::var("AIBOX_DIAGNOSTICS_INTERVAL")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(5),
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
        "Usage: aibox-diagnostics [--once|--watch] [--state-dir PATH|--output PATH] [--ring-size N] [--interval SECONDS]\n\nWrites latest.json plus a bounded snapshot-NN.json ring from direct runtime diagnostics."
    );
}
