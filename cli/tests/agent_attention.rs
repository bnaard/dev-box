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
    let output = run(
        dir,
        &[
            "init",
            "attention-test",
            "--base",
            "debian",
            "--harness",
            "codex",
        ],
    );
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
        "format = \"{state_symbol}{repository}{agent_suffix}\"",
        "max-length",
        "repository-style = \"basename\"",
        "agent-style = \"basename\"",
        "[customization.tmux.title.states]",
        "question =",
    ] {
        assert!(
            config.contains(expected),
            "generated aibox.toml is missing title contract entry {expected:?}:\n{config}"
        );
    }

    let codex_config =
        fs::read_to_string(dir.path().join(".codex/config.toml")).expect("read Codex config");
    assert!(
        !codex_config.contains("notify"),
        "project-local Codex config must not contain the unsupported notify key:\n{codex_config}"
    );
    let codex_hooks =
        fs::read_to_string(dir.path().join(".codex/hooks.json")).expect("read Codex hooks");
    assert!(
        codex_hooks.contains(r#""Stop""#)
            && codex_hooks.contains("aibox-agent-signal done --harness codex"),
        "Codex completion must be registered through the project Stop hook:\n{codex_hooks}"
    );
    let notify = dir.path().join(".aibox-home/.local/bin/aibox-codex-notify");
    assert!(notify.is_file(), "Codex notify adapter must be generated");
    assert_ne!(
        fs::metadata(notify).unwrap().permissions().mode() & 0o111,
        0,
        "Codex notify adapter must be executable"
    );
}

#[test]
fn signal_helper_refreshes_terminal_titles_after_state_changes() {
    let helper = fs::read_to_string(format!(
        "{}/src/templates/aibox-agent-signal.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("read attention helper template");

    assert!(
        helper.contains("tmux list-clients -F '#{client_name}'")
            && helper.contains("tmux refresh-client -t \"$client_name\""),
        "attention changes must explicitly refresh every attached client so set-titles-string is repainted"
    );
    assert!(
        !helper.contains("tmux refresh-client -S 2>/dev/null || true"),
        "a status-only refresh can leave the host terminal title stale"
    );
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
    assert!(
        tmux.contains("set -g @aibox_title_repository_style \"basename\""),
        "repository title style was not rendered:\n{tmux}"
    );
    assert!(
        tmux.contains("set -g @aibox_title_agent_style \"basename\""),
        "agent title style was not rendered:\n{tmux}"
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
    let codex_notify = dir.path().join("aibox-codex-notify");
    let repository = dir.path().join("checkout");
    let codex_home = dir.path().join("codex-home");
    fs::create_dir_all(&repository).expect("create repository");
    fs::create_dir_all(&codex_home).expect("create Codex home");
    let database = codex_home.join("state_5.sqlite");
    let output = Command::new("python3")
        .arg("-c")
        .arg("import sqlite3,sys; db=sqlite3.connect(sys.argv[1]); db.execute('create table threads (id text primary key, cwd text, model text, reasoning_effort text, archived integer, updated_at integer, updated_at_ms integer)'); db.execute('insert into threads values (?, ?, ?, ?, ?, ?, ?)', ('thread-test', sys.argv[2], 'gpt-5.6-sol', 'low', 0, 1, 1000)); db.commit()")
        .arg(&database)
        .arg(&repository)
        .output()
        .expect("create Codex state fixture");
    assert!(
        output.status.success(),
        "failed to create Codex state fixture"
    );
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .args(args)
            .current_dir(&repository)
            .output()
            .expect("execute git");
        assert!(
            output.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init"]);
    git(&[
        "remote",
        "add",
        "origin",
        "https://github.com/projectious-work/aibox.git",
    ]);
    fs::copy(
        format!(
            "{}/src/templates/aibox-agent-signal.sh",
            env!("CARGO_MANIFEST_DIR")
        ),
        &helper,
    )
    .expect("copy attention helper");
    fs::copy(
        format!(
            "{}/src/templates/aibox-codex-notify.sh",
            env!("CARGO_MANIFEST_DIR")
        ),
        &codex_notify,
    )
    .expect("copy Codex notify adapter");
    let mut permissions = fs::metadata(&helper)
        .expect("helper metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&helper, permissions).expect("make helper executable");
    let mut permissions = fs::metadata(&codex_notify)
        .expect("Codex notify metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&codex_notify, permissions).expect("make Codex notify executable");

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
        "-c",
        repository.to_str().expect("repository path is UTF-8"),
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
        ("@aibox_title_repository_style", "basename"),
        ("@aibox_title_agent_style", "full"),
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
        let command = format!(
            "TMUX_PANE='{}' CODEX_HOME='{}' CODEX_THREAD_ID='' CODEX_SESSION_ID='' '{}' {}",
            pane,
            codex_home.display(),
            helper.display(),
            args
        );
        success(&["run-shell", "-t", pane, &command]);
    };
    let signal_hook = |pane: &str, args: &str, payload: &str| {
        let command = format!(
            "printf '%s' '{}' | TMUX_PANE='{}' '{}' {} --hook-input",
            payload,
            pane,
            helper.display(),
            args
        );
        success(&["run-shell", "-t", pane, &command]);
    };
    let window_value =
        |option: &str| value(&["show-window-option", "-v", "-t", "attention:agents", option]);

    signal(&pane_one, "working --harness codex --message build");
    assert_eq!(window_value("@aibox_attention_repository"), "aibox");
    assert_eq!(window_value("@aibox_attention_agent"), "gpt-5.6-sol low");
    assert_eq!(
        value(&[
            "show-option",
            "-pqv",
            "-t",
            &pane_one,
            "@aibox_attention_agent"
        ]),
        "gpt-5.6-sol low"
    );
    signal_hook(
        &pane_two,
        "question --harness claude --message 'Frage ä?'",
        r#"{"model":"claude-opus-4-6","effort":{"level":"high"}}"#,
    );
    assert_eq!(window_value("@aibox_attention_state"), "question");
    assert_eq!(window_value("@aibox_attention_harness"), "claude");
    assert_eq!(
        window_value("@aibox_attention_agent"),
        "claude-opus-4-6 high"
    );
    assert_eq!(window_value("@aibox_attention_message"), "Frage ä?");

    for (harness, payload, expected) in [
        (
            "gemini",
            r#"{"llm_request":{"model":"gemini-2.5-pro"}}"#,
            "gemini-2.5-pro",
        ),
        (
            "copilot",
            r#"{"message":{"modelId":"claude-sonnet-4.6"}}"#,
            "claude-sonnet-4.6",
        ),
        (
            "cursor",
            r#"{"model":"composer-2","reasoningEffort":"high"}"#,
            "composer-2 high",
        ),
    ] {
        signal_hook(&pane_two, &format!("working --harness {harness}"), payload);
        assert_eq!(
            value(&[
                "show-option",
                "-pqv",
                "-t",
                &pane_two,
                "@aibox_attention_agent",
            ]),
            expected,
            "unexpected hook-derived identity for {harness}"
        );
    }
    signal(
        &pane_two,
        "working --harness opencode --agent openai/gpt-5.2 --effort high",
    );
    assert_eq!(
        value(&[
            "show-option",
            "-pqv",
            "-t",
            &pane_two,
            "@aibox_attention_agent",
        ]),
        "openai/gpt-5.2 high"
    );
    signal_hook(
        &pane_two,
        "question --harness claude --message 'Frage ä?'",
        r#"{"model":"claude-opus-4-6","effort":{"level":"high"}}"#,
    );

    signal_hook(&pane_two, "clear --harness claude", "{}");
    assert_eq!(
        value(&[
            "show-option",
            "-pqv",
            "-t",
            &pane_two,
            "@aibox_attention_agent"
        ]),
        "claude-opus-4-6 high"
    );
    let claude_transcript = dir.path().join("claude-transcript.jsonl");
    fs::write(
        &claude_transcript,
        r#"{"type":"assistant","effort":"medium","message":{"model":"claude-sonnet-4-6"}}"#,
    )
    .expect("write Claude transcript fixture");
    let transcript_payload = format!(r#"{{"transcript_path":"{}"}}"#, claude_transcript.display());
    signal_hook(&pane_two, "question --harness claude", &transcript_payload);
    assert_eq!(
        window_value("@aibox_attention_agent"),
        "claude-sonnet-4-6 medium"
    );
    signal_hook(&pane_two, "clear --harness claude", "{}");
    assert_eq!(window_value("@aibox_attention_state"), "working");

    let codex_transcript = dir.path().join("codex-transcript.jsonl");
    fs::write(
        &codex_transcript,
        r#"{"type":"response_item","payload":{"type":"custom_tool_call","name":"request_user_input"}}"#,
    )
    .expect("write Codex transcript fixture");
    let codex_question_payload =
        format!(r#"{{"transcript_path":"{}"}}"#, codex_transcript.display());
    signal_hook(
        &pane_one,
        "question --harness codex",
        &codex_question_payload,
    );
    assert_eq!(window_value("@aibox_attention_state"), "question");
    use std::io::Write as _;
    writeln!(
        fs::OpenOptions::new()
            .append(true)
            .open(&codex_transcript)
            .expect("open Codex transcript fixture"),
        r#"{{"type":"response_item","payload":{{"type":"custom_tool_call_output","output":"accepted"}}}}"#,
    )
    .expect("append Codex inline response");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(window_value("@aibox_attention_state"), "working");

    let notify_payload = r#"{"type":"agent-turn-complete","last-assistant-message":"Finished."}"#;
    let notify_command = format!(
        "PATH='{}':\"$PATH\" TMUX_PANE='{}' '{}' '{}'",
        dir.path().display(),
        pane_one,
        codex_notify.display(),
        notify_payload
    );
    success(&["run-shell", "-t", &pane_one, &notify_command]);
    assert_eq!(window_value("@aibox_attention_state"), "done");
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert_eq!(window_value("@aibox_attention_state"), "idle");
    assert_eq!(window_value("@aibox_attention_repository"), "aibox");
    assert_eq!(window_value("@aibox_attention_harness"), "codex");
    assert_eq!(window_value("@aibox_attention_agent"), "gpt-5.6-sol low");

    success(&["set-option", "-g", "@aibox_title_agent_style", "basename"]);
    signal(&pane_one, "idle --harness codex");
    assert_eq!(window_value("@aibox_attention_agent"), "gpt-5.6-sol");
    success(&["set-option", "-g", "@aibox_title_agent_style", "full"]);

    success(&["set-option", "-g", "@aibox_title_repository_style", "full"]);
    for (remote, expected) in [
        (
            "https://github.com/projectious-work/aibox.git",
            "projectious-work/aibox",
        ),
        (
            "git@gitlab.com:projectious-work/platform/aibox.git",
            "projectious-work/platform/aibox",
        ),
        (
            "ssh://git@gitea.example.net/projectious-work/aibox.git",
            "projectious-work/aibox",
        ),
        (
            "https://forgejo.example.net/projectious-work/aibox",
            "projectious-work/aibox",
        ),
    ] {
        git(&["remote", "set-url", "origin", remote]);
        signal(&pane_one, "idle --harness codex");
        assert_eq!(
            window_value("@aibox_attention_repository"),
            expected,
            "unexpected full repository name for {remote}"
        );
    }

    signal(&pane_one, "working --harness codex --agent Rowan");
    assert_eq!(window_value("@aibox_attention_agent"), "Rowan");

    success(&["kill-server"]);
}
