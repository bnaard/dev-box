//! Companion-independent lifecycle contracts.
//!
//! These tests intentionally run in the default E2E suite. Their observable
//! behavior is limited to CLI results and files inside a unique temporary
//! workspace, so an SSH host or container runtime would add privilege without
//! adding product evidence.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn aibox_bin() -> String {
    format!("{}/target/debug/aibox", env!("CARGO_MANIFEST_DIR"))
}

fn addons_dir() -> String {
    format!("{}/../addons", env!("CARGO_MANIFEST_DIR"))
}

fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("AIBOX_NO_CONTAINER", "1")
        .env_remove("DOCKER_HOST")
        .env_remove("CONTAINER_HOST")
        .env_remove("E2E_HOST")
        .output()
        .expect("failed to execute aibox")
}

fn init(dir: &Path, name: &str, context: &str) -> Output {
    run_in(
        dir,
        &[
            "init",
            name,
            "--base",
            "debian",
            "--context",
            context,
            "--processkit-version",
            "unset",
            "--no-container",
        ],
    )
}

fn assert_success(label: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{label} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn lifecycle_init_apply() {
    let tmp = tempfile::tempdir().expect("create lifecycle workspace");
    let dir = tmp.path();

    assert_success("init", &init(dir, "lifecycle-init-apply", "managed"));
    for path in [
        "aibox.toml",
        ".devcontainer/Dockerfile",
        ".devcontainer/docker-compose.yml",
        "CLAUDE.md",
    ] {
        assert!(dir.join(path).exists(), "{path} should exist after init");
    }

    assert_success("apply", &run_in(dir, &["apply", "--no-container"]));
}

#[test]
fn claudemd_preserved_on_sync() {
    let tmp = tempfile::tempdir().expect("create CLAUDE.md workspace");
    let dir = tmp.path();
    assert_success("init", &init(dir, "claudemd-preserve", "managed"));

    let custom = "# My Custom CLAUDE.md\n\nUser-specific content here.\n";
    fs::write(dir.join("CLAUDE.md"), custom).expect("write custom CLAUDE.md");
    assert_success("apply", &run_in(dir, &["apply", "--no-container"]));

    let content = fs::read_to_string(dir.join("CLAUDE.md")).expect("read CLAUDE.md");
    assert!(content.contains("User-specific content"));
}

#[test]
fn generated_files_overwritten_on_sync() {
    let tmp = tempfile::tempdir().expect("create regeneration workspace");
    let dir = tmp.path();
    assert_success("init", &init(dir, "gen-overwrite", "managed"));

    fs::write(
        dir.join(".devcontainer/Dockerfile"),
        "# tampered\nFROM scratch\n",
    )
    .expect("tamper with generated Dockerfile");
    assert_success("apply", &run_in(dir, &["apply", "--no-container"]));

    let dockerfile =
        fs::read_to_string(dir.join(".devcontainer/Dockerfile")).expect("read Dockerfile");
    assert!(!dockerfile.contains("# tampered"));
    assert!(dockerfile.contains("FROM"));
}

#[test]
fn runtime_without_container_shows_missing() {
    let tmp = tempfile::tempdir().expect("create no-runtime workspace");
    let dir = tmp.path();
    assert_success("init", &init(dir, "runtime-missing", "managed"));

    let output = run_in(dir, &["get", "runtime"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!output.status.success(), "missing runtime must fail closed");
    assert!(
        combined.contains("Neither podman nor docker found"),
        "get runtime should report the precise missing-runtime prerequisite: {combined}"
    );
}

#[test]
fn init_with_managed_preset_creates_context_files() {
    let tmp = tempfile::tempdir().expect("create managed workspace");
    let dir = tmp.path();
    assert_success("init", &init(dir, "init-managed-preset", "managed"));

    // With processkit explicitly unset, aibox owns the slim project shell:
    // provider pointer, config, and empty context root. AGENTS.md is supplied
    // by processkit and is therefore not an aibox-init contract here.
    for path in ["CLAUDE.md", "aibox.toml", "context"] {
        assert!(dir.join(path).exists(), "{path} should exist");
    }
    assert!(!dir.join("AGENTS.md").exists());
    let config = fs::read_to_string(dir.join("aibox.toml")).expect("read aibox.toml");
    assert!(config.contains("[skills]"));
}

#[test]
fn init_with_software_preset_creates_code_files() {
    let tmp = tempfile::tempdir().expect("create software workspace");
    let dir = tmp.path();
    assert_success("init", &init(dir, "init-software-preset", "software"));

    assert!(dir.join("context").is_dir());
    assert!(dir.join("CLAUDE.md").is_file());
    assert!(!dir.join("AGENTS.md").exists());
}
