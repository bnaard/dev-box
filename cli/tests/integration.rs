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

fn write_orchestration_compile_fixture(dir: &std::path::Path) {
    std::fs::write(
        dir.join("aibox.toml"),
        r#"[container]
name = "compile-test"

[orchestration]
enabled = true

[orchestration.image]
reference = "ghcr.io/acme/workspace"
digest = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
platform = "linux-amd64"

[orchestration.fleet]
name = "workspace"
services = [{ name = "workspace", ports = [{ container_port = 8080 }] }]

[orchestration.target]
backend = "compose"
reference = "docker-context:default"
scope = "workspace"

[orchestration.deployment]
name = "workspace-dev"
owner_id = "team-a"
"#,
    )
    .unwrap();
}

#[test]
fn config_compile_json_is_deterministic_and_read_only() {
    let dir = tempfile::tempdir().unwrap();
    write_orchestration_compile_fixture(dir.path());
    let before = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();

    let first = Command::new(aibox_bin())
        .args(["config", "compile", "--output", "json"])
        .current_dir(dir.path())
        .env("AIBOX_ADDONS_DIR", dir.path().join("missing-addons"))
        .output()
        .expect("run config compile without an addon catalog");
    let second = run_in_dir(dir.path(), &["config", "compile", "--output", "json"]);
    let first_json = parse_json(&first);
    let second_json = parse_json(&second);

    assert_eq!(first_json, second_json);
    assert!(
        first_json["desiredSpecDigest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert_eq!(first_json["actions"].as_array().unwrap().len(), 1);
    assert_eq!(first_json["actions"][0]["type"], "deploy-fleet");

    let after = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(
        before, after,
        "config compile must not create project files"
    );
}

#[test]
fn config_compile_human_reports_digest_and_disabled_build() {
    let dir = tempfile::tempdir().unwrap();
    write_orchestration_compile_fixture(dir.path());

    let output = run_in_dir(dir.path(), &["config", "compile"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("desired spec digest: sha256:"));
    assert!(stdout.contains("image build: disabled"));
    assert!(stdout.contains("target: compose:docker-context:default"));
}

#[test]
fn deploy_plan_renders_compose_artifacts_without_writing_project_files() {
    let dir = tempfile::tempdir().unwrap();
    write_orchestration_compile_fixture(dir.path());
    let before = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();

    let output = run_in_dir(dir.path(), &["deploy", "plan", "--output", "json"]);
    let json = parse_json(&output);
    assert_eq!(json["backend"], "compose");
    assert!(
        json["deploymentId"]
            .as_str()
            .unwrap()
            .starts_with("workspace-")
    );
    assert!(
        json["composeYaml"]
            .as_str()
            .unwrap()
            .contains("aibox.projectious.work/deployment-id")
    );
    assert!(
        json["devcontainerJson"]
            .as_str()
            .unwrap()
            .contains("dockerComposeFile")
    );

    let after = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(before, after, "deploy plan must not write project files");
}

#[test]
fn config_compile_rejects_disabled_orchestration() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("aibox.toml"),
        "[container]\nname = \"legacy\"\n",
    )
    .unwrap();

    let output = run_in_dir(dir.path(), &["config", "compile", "--output", "json"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("orchestration is not enabled"));
}

fn installed_addon_files_from_install_script() -> Vec<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("scripts/install.sh");
    let script = std::fs::read_to_string(script_path).expect("read scripts/install.sh");
    let Some((_, rest)) = script.split_once("local addon_files=\"") else {
        panic!("install script should declare addon_files");
    };
    let Some((list, _)) = rest.split_once('"') else {
        panic!("install script addon_files block should be closed");
    };
    list.split_whitespace().map(str::to_string).collect()
}

fn install_script_addons_dir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create installed-addon tempdir");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_addons = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("addons");

    for file in installed_addon_files_from_install_script() {
        let src = repo_addons.join(&file);
        assert!(
            src.is_file(),
            "install script references missing addon YAML: {}",
            file
        );
        let dst = tmp.path().join(&file);
        std::fs::create_dir_all(dst.parent().unwrap()).unwrap();
        std::fs::copy(&src, &dst).unwrap();
    }

    tmp
}

#[test]
fn release_scripts_publish_checksum_sidecars() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = std::path::Path::new(manifest_dir).parent().unwrap();
    let maintain =
        std::fs::read_to_string(repo_root.join("scripts/maintain.sh")).expect("read maintain.sh");
    let build_macos = std::fs::read_to_string(repo_root.join("scripts/build-macos.sh"))
        .expect("read build-macos.sh");
    let install =
        std::fs::read_to_string(repo_root.join("scripts/install.sh")).expect("read install.sh");

    assert!(
        maintain.contains(
            r#"sha256_file "${DIST_DIR}/${binary_name}.tar.gz" > "${DIST_DIR}/${binary_name}.tar.gz.sha256""#,
        )
            && maintain.contains(r#"built_archives+=("${archive}" "${checksum}")"#)
            && maintain.contains(r#""${DIST_DIR}"/aibox-v${version}-*-apple-darwin.tar.gz.sha256"#),
        "maintain.sh must generate and upload sha256 sidecars for Linux and macOS release assets"
    );
    assert!(
        maintain.contains("release_validate_license_guardrails")
            && maintain.contains(r#"-C "${PROJECT_ROOT}" LICENSE"#)
            && maintain.contains(r#""${PROJECT_ROOT}/LICENSE""#)
            && maintain.contains("--clobber"),
        "maintain.sh must enforce README license notice, include LICENSE in Linux tarballs, and upload LICENSE to GitHub releases"
    );
    assert!(
        build_macos.contains(r#"shasum -a 256 "${DIST_DIR}/${local_name}.tar.gz""#)
            && build_macos.contains(r#"${DIST_DIR}/${local_name}.tar.gz.sha256"#)
            && build_macos.contains(r#"-C "${PROJECT_ROOT}" LICENSE"#),
        "build-macos.sh must generate sha256 sidecars on macOS and include LICENSE in macOS tarballs"
    );
    assert!(
        install.contains("sha256_digest()")
            && install.contains("command -v sha256sum")
            && install.contains("command -v shasum")
            && install
                .contains(r#"computed_digest="$(sha256_digest "${tmpdir}/${tarball_name}")""#),
        "install.sh must verify checksums using sha256sum or macOS shasum"
    );
    assert!(
        maintain.contains("image_source_sha()")
            && maintain.contains("image_foundation_tag()")
            && maintain.contains("image_runtime_tag()")
            && maintain.contains("image_runtime_latest_tag()")
            && maintain.contains("read:packages")
            && maintain.contains("delete:packages")
            && maintain.contains("--repair-mixed")
            && maintain.contains("--prefer-index=true")
            && maintain.contains("source-tag-detached")
            && maintain.contains("buildx imagetools inspect")
            && maintain.contains("without rebuilding layers")
            && maintain.contains("require_docker_buildx_for_images")
            && maintain.contains("Docker Buildx is required"),
        "maintain.sh must support label-based image retagging and require Docker Buildx for BuildKit-only image builds"
    );
}

#[test]
fn image_fallback_tmux_config_does_not_bind_global_ctrl_j() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_root = std::path::Path::new(manifest_dir).parent().unwrap();
    let tmux = std::fs::read_to_string(repo_root.join("images/base-debian/config/tmux/tmux.conf"))
        .expect("read image fallback tmux.conf");

    assert!(
        tmux.contains(r#"set -g @vim_navigator_mapping_down "C-Down""#)
            && tmux.contains("unbind-key -q -n C-j")
            && tmux.contains("unbind-key -q -T copy-mode-vi C-j")
            && tmux.contains("bind-key -n C-Down")
            && tmux.contains("bind-key -T copy-mode-vi C-Down")
            && !tmux.contains("bind-key -n C-j")
            && !tmux.contains("bind-key -T copy-mode-vi C-j"),
        "image fallback tmux.conf must not bind global C-j because pasted newlines arrive as LF/C-j:\n{tmux}"
    );
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
fn install_script_lists_every_repo_addon_yaml() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_addons = std::path::Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("addons");
    let mut repo_files = Vec::new();
    for category in ["ai", "docs", "languages", "tools"] {
        for entry in std::fs::read_dir(repo_addons.join(category)).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                repo_files.push(format!(
                    "{}/{}",
                    category,
                    path.file_name().unwrap().to_string_lossy()
                ));
            }
        }
    }
    repo_files.sort();

    let mut script_files = installed_addon_files_from_install_script();
    script_files.sort();

    assert_eq!(
        script_files, repo_files,
        "install.sh must publish the same addon catalog that the repo tests use"
    );
}

#[test]
fn apply_with_installed_catalog_installs_gh_from_git_ui() {
    let dir = tempfile::tempdir().unwrap();
    let installed_addons = install_script_addons_dir();
    std::fs::write(
        dir.path().join("aibox.toml"),
        r#"[aibox]
version = "0.23.3"
base = "debian"

[container]
name = "gh-addon-test"

[processkit]
version = "unset"

[addons.git-ui.tools]
gh = { enabled = true }
lazygit = { enabled = false }
"#,
    )
    .unwrap();

    let output = Command::new(aibox_bin())
        .args(["apply", "--no-container"])
        .current_dir(dir.path())
        .env("AIBOX_ADDONS_DIR", installed_addons.path())
        .output()
        .expect("failed to execute aibox apply");
    assert!(
        output.status.success(),
        "aibox apply should succeed with install-script addon catalog\nstatus: {}\nstderr:\n{}\nstdout:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );

    let dockerfile = std::fs::read_to_string(dir.path().join(".devcontainer/Dockerfile")).unwrap();
    assert!(
        dockerfile.contains("Addon: git-ui"),
        "Dockerfile should render the git-ui addon, not skip it:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains(" gh") || dockerfile.contains("\n      gh"),
        "Dockerfile should install gh when [addons.git-ui.tools].gh is enabled:\n{dockerfile}"
    );
    assert!(
        !dockerfile.contains("unknown addon 'git-ui'"),
        "git-ui must be known in installed-catalog simulation:\n{dockerfile}"
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
    assert!(
        stdout.contains("--standardize-config"),
        "apply help should expose the opt-in canonical config rewrite"
    );
}

#[test]
fn emergency_help_exposes_harness_entrypoint() {
    let output = run(&["emergency", "--help"]);
    assert!(
        output.status.success(),
        "aibox emergency --help should exit 0"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("<HARNESS>"),
        "emergency help should expose the harness positional:\n{stdout}"
    );
    assert!(
        stdout.contains("without tmux"),
        "emergency help should describe the non-tmux recovery path:\n{stdout}"
    );
}

#[test]
fn emergency_rejects_unknown_harness_before_runtime_work() {
    let output = run(&["emergency", "not-a-harness"]);
    assert!(
        !output.status.success(),
        "aibox emergency should reject unknown harness values"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid value") || stderr.contains("not-a-harness"),
        "error should mention the invalid harness:\n{stderr}"
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
        content.contains("profile      = \"headless-runner\""),
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
