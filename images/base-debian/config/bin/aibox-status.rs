mod aibox_status_core;

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use aibox_status_core::{ProcScanMode, Snapshot};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("--plugin-json") => {
            println!("{}", Snapshot::collect(ProcScanMode::Detailed).json());
        }
        Some("--once") => {
            println!("{}", Snapshot::collect(ProcScanMode::Minimal).plain());
        }
        Some("--watch") => watch(),
        Some("--help") | Some("-h") => print_help(),
        Some(other) => {
            eprintln!("aibox-status: unknown argument: {other}");
            std::process::exit(2);
        }
        None => {
            println!("{}", Snapshot::collect(ProcScanMode::Detailed).plain());
        }
    }
}

fn watch() {
    let interval = std::env::var("AIBOX_STATUS_INTERVAL")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5);
    let mut previous_width = 0usize;

    loop {
        let snapshot = Snapshot::collect(ProcScanMode::Detailed);
        let plain = snapshot.plain();
        let line = snapshot.styled();
        print!("\r{line}");
        if previous_width > plain.len() {
            print!("{}", " ".repeat(previous_width - plain.len()));
        }
        let _ = io::stdout().flush();
        previous_width = plain.len();
        thread::sleep(Duration::from_secs(interval.max(1)));
    }
}

fn print_help() {
    println!(
        "Usage: aibox-status [--plugin-json|--once|--watch]\n\nPrint compact runtime status from bounded direct reads in the current container."
    );
}
