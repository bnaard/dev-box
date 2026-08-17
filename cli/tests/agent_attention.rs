//! Black-box coverage for the configurable tmux agent-attention title contract.
//!
//! These tests intentionally inspect only generated project artifacts. They do
//! not start a tmux server, so they are safe to run in parallel and do not
//! depend on the caller's `TMUX` or `AIBOX_TMUX_SOCKET` environment.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

fn aibox_bin() -> String {
    format!("{}/target/debug/aibox", env!("CARGO_MANIFEST_DIR"))
}

fn addons_dir() -> String {
    format!("{}/../addons", env!("CARGO_MANIFEST_DIR"))
}

fn run(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env_remove("TMUX")
        .env_remove("AIBOX_TMUX_SOCKET")
        .output()
        .expect("failed to execute aibox")
}

fn init(dir: &std::path::Path) {
    let output = run(dir, &["init", "attention-test", "--base", "debian"]);
    assert!(
        output.status.success(),
        "aibox init failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generated_catalog_documents_title_configuration() {
    let dir = tempfile::tempdir().expect("create temporary project");
    init(dir.path());

    let config = fs::read_to_string(dir.path().join("aibox.toml")).expect("read aibox.toml");
    for expected in [
        "[customization.tmux.title]",
        "format =",
        "max-length",
        "[customization.tmux.title.states]",
        "question =",
    ] {
        assert!(
            config.contains(expected),
            "generated aibox.toml is missing title contract entry {expected:?}:\n{config}"
        );
    }
}

#[test]
fn custom_title_format_reaches_generated_tmux_config() {
    let dir = tempfile::tempdir().expect("create temporary project");
    init(dir.path());

    let config_path = dir.path().join("aibox.toml");
    let mut config = fs::read_to_string(&config_path).expect("read aibox.toml");
    let custom_format = "format = \"{state_symbol}{session}::{window} [{directory}]\"";
    if let Some(title_start) = config.find("[customization.tmux.title]") {
        let format_start = config[title_start..]
            .find("format =")
            .map(|offset| title_start + offset);
        if let Some(format_start) = format_start {
            let format_end = config[format_start..]
                .find('\n')
                .map(|offset| format_start + offset)
                .unwrap_or(config.len());
            config.replace_range(format_start..format_end, custom_format);
        }
    } else {
        config.push_str(&format!(
            "\n\n[customization.tmux.title]\nenabled = true\n{custom_format}\nmax-length = 72\n"
        ));
    }
    fs::write(&config_path, config).expect("write custom title config");

    let output = run(dir.path(), &["apply"]);
    assert!(
        output.status.success(),
        "aibox apply failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let tmux = fs::read_to_string(dir.path().join(".aibox-home/.config/tmux/tmux.conf"))
        .expect("read generated tmux.conf");
    assert!(
        tmux.contains("set -g set-titles on"),
        "title ownership missing:\n{tmux}"
    );
    assert!(
        tmux.contains("#S::#W [#{b:pane_current_path}]"),
        "custom title placeholders were not rendered into tmux expressions:\n{tmux}"
    );
    assert!(
        tmux.contains("#{=60:"),
        "custom title max-length was not rendered:\n{tmux}"
    );
}

#[test]
fn signal_helper_aggregates_panes_and_expires_done() {
    if Command::new("tmux").arg("-V").output().is_err() {
        eprintln!("tmux is not installed; skipping runtime attention test");
        return;
    }

    let dir = tempfile::tempdir().expect("create isolated tmux directory");
    let socket = dir.path().join("attention.sock");
    let helper = dir.path().join("aibox-agent-signal");
    fs::copy(
        format!(
            "{}/src/templates/aibox-agent-signal.sh",
            env!("CARGO_MANIFEST_DIR")
        ),
        &helper,
    )
    .expect("copy attention helper");
    let mut permissions = fs::metadata(&helper)
        .expect("helper metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&helper, permissions).expect("make helper executable");

    let tmux = |args: &[&str]| {
        Command::new("tmux")
            .arg("-S")
            .arg(&socket)
            .args(args)
            .env_remove("TMUX")
            .env_remove("TMUX_PANE")
            .output()
            .expect("execute isolated tmux")
    };
    let success = |args: &[&str]| {
        let output = tmux(args);
        assert!(
            output.status.success(),
            "tmux {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    let value = |args: &[&str]| {
        String::from_utf8(success(args).stdout)
            .expect("tmux output is UTF-8")
            .trim_end()
            .to_string()
    };

    success(&[
        "-f",
        "/dev/null",
        "new-session",
        "-d",
        "-s",
        "attention",
        "-n",
        "agents",
        "sleep 30",
    ]);
    let pane_one = value(&[
        "display-message",
        "-p",
        "-t",
        "attention:agents.1",
        "#{pane_id}",
    ]);
    success(&["split-window", "-d", "-t", "attention:agents", "sleep 30"]);
    let pane_two = value(&["list-panes", "-t", "attention:agents", "-F", "#{pane_id}"])
        .lines()
        .last()
        .expect("second pane")
        .to_string();
    for (key, setting) in [
        ("@aibox_title_message_max_length", "32"),
        ("@aibox_done_ttl_seconds", "1"),
        ("@aibox_notifications_enabled", "0"),
        ("@aibox_notifications_protocol", "bell"),
        ("@aibox_title_state_working", "working"),
        ("@aibox_title_state_question", "question"),
        ("@aibox_title_state_done", "done"),
        ("@aibox_title_state_error", "error"),
        ("@aibox_title_state_idle", ""),
    ] {
        success(&["set-option", "-g", key, setting]);
    }

    let signal = |pane: &str, args: &str| {
        let command = format!("TMUX_PANE='{}' '{}' {}", pane, helper.display(), args);
        success(&["run-shell", "-t", pane, &command]);
    };
    let window_value =
        |option: &str| value(&["show-window-option", "-v", "-t", "attention:agents", option]);

    signal(&pane_one, "working --harness codex --message build");
    signal(&pane_two, "question --harness claude --message 'Frage ä?'");
    assert_eq!(window_value("@aibox_attention_state"), "question");
    assert_eq!(window_value("@aibox_attention_harness"), "claude");
    assert_eq!(window_value("@aibox_attention_message"), "Frage ä?");

    signal(&pane_two, "clear");
    assert_eq!(window_value("@aibox_attention_state"), "working");
    signal(&pane_one, "done --harness codex");
    assert_eq!(window_value("@aibox_attention_state"), "done");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(window_value("@aibox_attention_state"), "idle");

    success(&["kill-server"]);
}
