mod aibox_status_core;

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use aibox_status_core::{ProcScanMode, Snapshot};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.first().map(String::as_str) {
        Some("--plugin-json") => {
            println!(
                "{}",
                latest_json().unwrap_or_else(|| Snapshot::collect(ProcScanMode::Minimal).json())
            );
        }
        Some("--watch") => watch(),
        Some("--help") | Some("-h") => print_help(),
        Some(other) => {
            eprintln!("aibox-status: unknown argument: {other}");
            std::process::exit(2);
        }
        None => {
            println!(
                "{}",
                latest_plain().unwrap_or_else(|| Snapshot::collect(ProcScanMode::Minimal).plain())
            );
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
        let fallback;
        let (plain, line) = if let Some(plain) = latest_plain() {
            (plain.clone(), style_plain_snapshot(&plain))
        } else {
            fallback = Snapshot::collect(ProcScanMode::Minimal);
            (fallback.plain(), fallback.styled())
        };
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
        "Usage: aibox-status [--plugin-json|--watch]\n\nPrint compact runtime status from the diagnostics sidecar snapshot, falling back to bounded direct reads when unavailable."
    );
}

fn latest_json() -> Option<String> {
    let path = latest_path();
    let body = fs::read_to_string(path).ok()?;
    let trimmed = body.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn latest_plain() -> Option<String> {
    let json = latest_json()?;
    extract_json_string_field(&json, "plain")
}

fn latest_path() -> PathBuf {
    std::env::var("AIBOX_DIAGNOSTICS_LATEST")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/workspace/.aibox/diagnostics/latest.json"))
}

fn extract_json_string_field(json: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\":");
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start();
    let mut chars = rest.chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            match ch {
                '"' => value.push('"'),
                '\\' => value.push('\\'),
                'n' => value.push('\n'),
                'r' => value.push('\r'),
                't' => value.push('\t'),
                other => value.push(other),
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return Some(value);
        } else {
            value.push(ch);
        }
    }
    None
}

fn style_plain_snapshot(plain: &str) -> String {
    if std::env::var_os("NO_COLOR").is_some()
        || std::env::var("AIBOX_STATUS_STYLE").as_deref() == Ok("plain")
    {
        plain.to_string()
    } else {
        format!("\u{1b}[7m AIBOX \u{1b}[27m {plain}")
    }
}
