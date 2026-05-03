use std::process::Command;

use serde_json::Value;

/// Get the path to the built binary.
fn aibox_bin() -> String {
    // Use the debug binary built by cargo test
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

/// Get the path to the addon YAML definitions in the repo.
fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

/// Run the aibox binary with the given args and return the output.
fn run(args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox binary")
}

/// Run the aibox binary in a specific directory.
fn run_in_dir(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox binary")
}

fn parse_json(output: &std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "command failed\nstatus: {}\nstderr:\n{}\nstdout:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout should be JSON: {}\nstdout:\n{}\nstderr:\n{}",
            err,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn write_projection_fixture(dir: &std::path::Path) {
    std::fs::write(
        dir.join("aibox.toml"),
        r#"[aibox]
version = "0.22.0"
base = "debian"
profile = "headless-runner"

[container]
name = "projection-test"
user = "agent"
keepalive = true

[context]
packages = ["software", "managed"]

[ai]
harnesses = ["cursor", "codex"]
model_providers = ["openai", "anthropic"]

[addons.rust.tools]
rustc = { version = "1.94" }
clippy = {}

[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }

[[mcp.servers]]
name = "team-tool"
command = "uv"
args = ["run", "server.py"]

[mcp.servers.env]
TEAM_TOKEN = "secret-token"
"#,
    )
    .unwrap();
}

#[test]
fn help_exits_zero() {
    let output = run(&["--help"]);
    assert!(output.status.success(), "aibox --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aibox") || stdout.contains("development container"),
        "help output should mention aibox"
    );
}

#[test]
fn init_help_exits_zero() {
    let output = run(&["init", "--help"]);
    assert!(output.status.success(), "aibox init --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[NAME]") || stdout.contains("name"),
        "init help should mention positional name"
    );
    assert!(
        stdout.contains("--profile"),
        "init help should mention --profile"
    );
    assert!(
        stdout.contains("headless-runner"),
        "init help should mention headless-runner"
    );
}

#[test]
fn apply_help_mentions_no_cache_and_rebuild_alias() {
    let output = run(&["apply", "--help"]);
    assert!(output.status.success(), "aibox apply --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--no-cache"),
        "apply help should expose --no-cache"
    );
    assert!(
        stdout.contains("--rebuild"),
        "apply help should keep --rebuild as an alias"
    );
}

#[test]
fn apply_no_cache_parses() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("aibox.toml"),
        r#"[aibox]
version = "0.22.0"

[container]
name = "parse-test"

[processkit]
version = "unset"
"#,
    )
    .unwrap();

    let output = run_in_dir(dir.path(), &["apply", "--no-cache", "--no-container"]);
    assert!(
        output.status.success(),
        "aibox apply --no-cache should parse and run in no-container mode: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn reset_context_dry_run_parses() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("aibox.toml"),
        r#"[aibox]
version = "0.22.0"

[container]
name = "reset-context-test"

[processkit]
version = "unset"
"#,
    )
    .unwrap();

    let output = run_in_dir(
        dir.path(),
        &[
            "reset",
            "context",
            "--from-processkit",
            "v0.25.0",
            "--dry-run",
        ],
    );
    assert!(
        output.status.success(),
        "aibox reset context --dry-run should parse and produce a plan: stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Context reset plan"));
}

#[test]
fn apply_without_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["apply"]);
    assert!(
        !output.status.success(),
        "aibox apply without aibox.toml should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("aibox.toml") || stderr.contains("No aibox.toml"),
        "error should mention missing config file"
    );
}

#[test]
fn status_without_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["get", "runtime"]);
    assert!(
        !output.status.success(),
        "aibox get runtime without aibox.toml should fail"
    );
}

#[test]
fn init_creates_expected_files() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "test-project",
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset", // avoid network fetch in tests
        ],
    );
    assert!(
        output.status.success(),
        "init should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        dir.path().join("aibox.toml").exists(),
        "aibox.toml should be created"
    );
    assert!(
        dir.path().join(".devcontainer/Dockerfile").exists(),
        "Dockerfile should be created"
    );
    assert!(
        dir.path().join(".devcontainer/docker-compose.yml").exists(),
        "docker-compose.yml should be created"
    );
    assert!(
        dir.path().join(".devcontainer/devcontainer.json").exists(),
        "devcontainer.json should be created"
    );
    // AGENTS.md is owned by processkit since v0.16.0 and lands only
    // when [processkit].version is pinned. The default `aibox init`
    // writes "unset", so AGENTS.md is intentionally absent here.
    assert!(
        !dir.path().join("AGENTS.md").exists(),
        "AGENTS.md should NOT be created when processkit version is unset"
    );
    assert!(
        dir.path().join("CLAUDE.md").exists(),
        "CLAUDE.md (thin pointer) should be created"
    );
    let claude_body = std::fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap();
    assert!(
        claude_body.contains("AGENTS.md") && claude_body.contains("Pointer file"),
        "thin-pointer CLAUDE.md should reference AGENTS.md"
    );
    assert!(
        !dir.path().join(".aibox-version").exists(),
        ".aibox-version must NOT be created (absorbed into aibox.lock since v0.17.0)"
    );
}

#[test]
fn init_existing_config_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    // First init
    run_in_dir(
        dir.path(),
        &["init", "test", "--base", "debian", "--context", "managed"],
    );
    // Second init should fail
    let output = run_in_dir(
        dir.path(),
        &["init", "test", "--base", "debian", "--context", "managed"],
    );
    assert!(
        !output.status.success(),
        "init with existing aibox.toml should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists"),
        "error should mention config already exists"
    );
}

#[test]
fn apply_after_init_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    // Init first
    let init_output = run_in_dir(
        dir.path(),
        &[
            "init",
            "gen-test",
            "--base",
            "debian",
            "--context",
            "managed",
        ],
    );
    assert!(init_output.status.success(), "init should succeed");

    // Apply should work
    let apply_output = run_in_dir(dir.path(), &["apply"]);
    assert!(
        apply_output.status.success(),
        "apply after init should succeed: {}",
        String::from_utf8_lossy(&apply_output.stderr)
    );
}

#[test]
fn init_invalid_base_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "test",
            "--base",
            "invalid-base",
            "--context",
            "managed",
        ],
    );
    assert!(
        !output.status.success(),
        "init with invalid base should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("Invalid") || stderr.contains("error"),
        "error should mention invalid base: {}",
        stderr
    );
}

#[test]
fn init_profile_headless_runner_is_written_to_config() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "runner",
            "--base",
            "debian",
            "--profile",
            "headless-runner",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        output.status.success(),
        "init should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let content = std::fs::read_to_string(dir.path().join("aibox.toml")).unwrap();
    assert!(
        content.contains("profile = \"headless-runner\""),
        "generated aibox.toml should preserve the requested profile:\n{content}"
    );
}

#[test]
fn init_invalid_process_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(
        dir.path(),
        &[
            "init",
            "test",
            "--base",
            "debian",
            "--context",
            "invalid-process!",
        ],
    );
    assert!(
        !output.status.success(),
        "init with invalid process should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("Invalid") || stderr.contains("error"),
        "error should mention invalid process: {}",
        stderr
    );
}

#[test]
fn init_with_all_base_images() {
    // Currently only "debian" is supported; add more entries when new bases land
    let bases = ["debian"];
    for base in &bases {
        let dir = tempfile::tempdir().unwrap();
        let output = run_in_dir(
            dir.path(),
            &["init", "test", "--base", base, "--context", "managed"],
        );
        assert!(
            output.status.success(),
            "init with base '{}' should succeed: {}",
            base,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn init_with_all_process_packages() {
    for pkg in &["minimal", "managed", "software", "research", "product"] {
        let dir = tempfile::tempdir().unwrap();
        let output = run_in_dir(
            dir.path(),
            &["init", "test", "--base", "debian", "--context", pkg],
        );
        assert!(
            output.status.success(),
            "init with process '{}' should succeed: {}",
            pkg,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn init_generated_toml_is_parseable() {
    let dir = tempfile::tempdir().unwrap();
    run_in_dir(
        dir.path(),
        &[
            "init",
            "parse-test",
            "--base",
            "debian",
            "--context",
            "managed",
        ],
    );
    let content = std::fs::read_to_string(dir.path().join("aibox.toml")).unwrap();
    // Should be valid TOML
    let value: toml::Value =
        toml::from_str(&content).expect("generated aibox.toml should be valid TOML");
    assert_eq!(value["aibox"]["profile"].as_str(), Some("human-dev"));
}

#[test]
fn completions_bash_exits_zero() {
    let output = run(&["self", "completion", "bash"]);
    assert!(
        output.status.success(),
        "aibox self completion bash should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aibox"),
        "bash completion should contain aibox"
    );
}

#[test]
fn completions_zsh_exits_zero() {
    let output = run(&["self", "completion", "zsh"]);
    assert!(
        output.status.success(),
        "aibox self completion zsh should exit 0"
    );
}

#[test]
fn completions_invalid_shell_exits_nonzero() {
    let output = run(&["self", "completion", "tcsh"]);
    assert!(
        !output.status.success(),
        "aibox self completion tcsh should fail"
    );
}

#[test]
fn doctor_without_config_reports_errors() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in_dir(dir.path(), &["doctor"]);
    // Doctor exits 0 even when reporting errors (it's a diagnostic tool)
    assert!(output.status.success(), "doctor should always exit 0");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("aibox.toml") || stderr.contains("Config"),
        "doctor should report missing config"
    );
}

#[test]
fn describe_addon_catalog_json_contract() {
    let output = run(&["describe", "addon-catalog", "-o", "json"]);
    let json = parse_json(&output);

    assert_eq!(json["schema_version"], "aibox.addon-catalog.v0");
    let addons = json["addons"]
        .as_array()
        .expect("addons should be an array");
    assert!(!addons.is_empty(), "addon catalog should not be empty");

    let python = addons
        .iter()
        .find(|addon| addon["name"] == "python")
        .expect("catalog should include python addon");
    assert_eq!(python["profile_intent"], "runtime");
    assert_eq!(python["usage_class"], "automated");
    assert!(
        python["profiles"]
            .as_array()
            .expect("profiles should be an array")
            .iter()
            .any(|profile| profile == "headless-runner")
    );
    assert!(
        python["exported_surfaces"]
            .as_array()
            .expect("exported_surfaces should be an array")
            .iter()
            .any(|surface| surface == "language-runtime")
    );
    assert!(
        python["tools"]
            .as_array()
            .expect("tools should be an array")
            .iter()
            .any(|tool| tool["name"] == "python")
    );
}

#[test]
fn describe_workspace_manifest_json_contract() {
    let dir = tempfile::tempdir().unwrap();
    write_projection_fixture(dir.path());

    let output = run_in_dir(
        dir.path(),
        &["describe", "workspace-manifest", "-o", "json"],
    );
    let json = parse_json(&output);

    assert_eq!(json["schema_version"], "aibox.workspace-manifest.v0");
    assert_eq!(json["project"]["name"], "projection-test");
    assert_eq!(json["project"]["profile"], "headless-runner");
    assert_eq!(
        json["context"]["packages"],
        serde_json::json!(["managed", "software"])
    );
    assert_eq!(
        json["ai"]["harnesses"],
        serde_json::json!(["codex", "cursor"])
    );
    assert_eq!(
        json["addons"]
            .as_array()
            .expect("addons should be an array")
            .iter()
            .map(|addon| addon["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["ai-codex", "python", "rust"]
    );

    let server = &json["mcp"]["extra_servers"][0];
    assert_eq!(server["name"], "team-tool");
    assert_eq!(server["env_keys"], serde_json::json!(["TEAM_TOKEN"]));
    assert!(
        !serde_json::to_string(&json)
            .unwrap()
            .contains("secret-token"),
        "workspace manifest must not expose MCP env values"
    );
}

#[test]
fn describe_provider_backends_json_contract() {
    let dir = tempfile::tempdir().unwrap();
    write_projection_fixture(dir.path());

    let output = run_in_dir(dir.path(), &["describe", "provider-backends", "-o", "json"]);
    let json = parse_json(&output);

    assert_eq!(json["schema_version"], "aibox.provider-backends.v0-preview");
    assert_eq!(
        json["selected_backends"],
        serde_json::json!(["codex", "cursor"])
    );

    let backends = json["backends"]
        .as_array()
        .expect("backends should be an array");
    let codex = backends
        .iter()
        .find(|backend| backend["name"] == "codex")
        .expect("codex backend should be present");
    assert_eq!(codex["selected"], true);
    assert_eq!(codex["mcp_config_target"], ".codex/config.toml");
    assert_eq!(codex["permission_target"], ".codex/config.toml");

    let cursor = backends
        .iter()
        .find(|backend| backend["name"] == "cursor")
        .expect("cursor backend should be present");
    assert_eq!(cursor["selected"], true);
    assert_eq!(cursor["container_cli"], false);
    assert_eq!(cursor["addon_name"], Value::Null);
    assert_eq!(cursor["mcp_config_target"], ".cursor/mcp.json");
}

#[test]
fn describe_image_provenance_policy_json_contract() {
    let dir = tempfile::tempdir().unwrap();
    write_projection_fixture(dir.path());

    let output = run_in_dir(
        dir.path(),
        &["describe", "image-provenance-policy", "-o", "json"],
    );
    let json = parse_json(&output);

    assert_eq!(
        json["schema_version"],
        "aibox.image-provenance-policy.v0-preview"
    );
    assert_eq!(json["image"]["registry"], "ghcr.io/projectious-work/aibox");
    assert_eq!(json["image"]["flavor"], "base-debian");
    assert_eq!(json["image"]["tag"], "base-debian-v0.22.0");
    assert_eq!(json["image"]["tag_template"], "base-debian-v{version}");
    assert_eq!(
        json["generated_files"]["dockerfile"],
        ".devcontainer/Dockerfile"
    );
    assert_eq!(
        json["generated_files"]["compose_file"],
        ".devcontainer/docker-compose.yml"
    );
    assert_eq!(json["runtime_markers"]["docker_label"], "aibox.version");
    assert_eq!(json["runtime_markers"]["profile_label"], "aibox.profile");
    assert_eq!(
        json["runtime_markers"]["version_file"],
        "/etc/aibox-version"
    );
    assert_eq!(
        json["selected_addons"],
        serde_json::json!(["ai-codex", "python", "rust"])
    );
    assert_eq!(
        json["release_phase"]["host_command_template"],
        "./scripts/maintain.sh release-host {version}"
    );
}
