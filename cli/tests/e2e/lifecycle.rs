//! Container lifecycle E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").
//! Tests the full init → apply lifecycle.

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn companion_is_reachable() {
    let runner = E2eRunner::new();
    runner.assert_reachable();
}

#[test]
#[serial]
fn lifecycle_init_apply() {
    let runner = E2eRunner::new();
    let test = "lifecycle-init-apply";
    runner.cleanup(test);

    // Init
    let output = runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify files created
    assert!(
        runner.file_exists(test, "aibox.toml"),
        "aibox.toml should exist"
    );
    assert!(
        runner.file_exists(test, ".devcontainer/Dockerfile"),
        "Dockerfile should exist"
    );
    assert!(
        runner.file_exists(test, ".devcontainer/docker-compose.yml"),
        "docker-compose.yml should exist"
    );
    assert!(
        runner.file_exists(test, "CLAUDE.md"),
        "CLAUDE.md should exist"
    );

    // Apply and verify generated project surfaces.
    let output = runner.aibox(test, &["apply", "--no-container"]);
    assert!(
        output.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    runner.cleanup(test);
}

#[test]
#[serial]
#[ntest::timeout(240_000)]
fn lifecycle_apply_starts_generated_container() {
    let runner = E2eRunner::new();
    let test = "lifecycle-container-up";
    runner.cleanup(test);

    let init = runner.aibox(
        test,
        &[
            "init",
            test,
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );
    let workspace = format!("/workspaces/{test}");
    let version = runner.exec(&format!(
        "cd {workspace} && sed -i 's/^release_version = .*/release_version = \"latest\"/' aibox.toml"
    ));
    assert!(
        version.status.success(),
        "failed to switch lifecycle container test to published latest image:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&version.stdout),
        String::from_utf8_lossy(&version.stderr)
    );

    let apply = runner.aibox(test, &["apply"]);
    assert!(
        apply.status.success(),
        "apply failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );

    let runtime = runner.runtime_bin();
    let up = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f .devcontainer/docker-compose.yml up -d {test}"
    ));
    assert!(
        up.status.success(),
        "compose up failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );

    let probe = runner.container_exec(
        test,
        "bash -lc 'test -r /etc/aibox-version && zellij --version && yazi --version && aibox-status --plugin-json >/tmp/aibox-status.json && jq -e .plain /tmp/aibox-status.json >/dev/null'",
    );
    assert!(
        probe.status.success(),
        "container probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );

    let down = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f .devcontainer/docker-compose.yml down -v"
    ));
    assert!(
        down.status.success(),
        "compose down failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&down.stdout),
        String::from_utf8_lossy(&down.stderr)
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn claudemd_preserved_on_sync() {
    let runner = E2eRunner::new();
    let test = "claudemd-preserve";
    runner.cleanup(test);

    // Init
    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    // Modify CLAUDE.md with user content
    runner.write_file(
        test,
        "CLAUDE.md",
        "# My Custom CLAUDE.md\n\nUser-specific content here.\n",
    );

    // Apply should not overwrite CLAUDE.md
    runner.aibox(test, &["apply"]);

    let content = runner.read_file(test, "CLAUDE.md");
    assert!(
        content.contains("User-specific content"),
        "CLAUDE.md user content should be preserved after apply"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn generated_files_overwritten_on_sync() {
    let runner = E2eRunner::new();
    let test = "gen-overwrite";
    runner.cleanup(test);

    // Init
    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    // Tamper with generated Dockerfile
    runner.write_file(
        test,
        ".devcontainer/Dockerfile",
        "# tampered\nFROM scratch\n",
    );

    // Apply should regenerate it
    runner.aibox(test, &["apply"]);

    let dockerfile = runner.read_file(test, ".devcontainer/Dockerfile");
    assert!(
        !dockerfile.contains("# tampered"),
        "Dockerfile should be regenerated, not contain tampered content"
    );
    assert!(
        dockerfile.contains("ghcr.io") || dockerfile.contains("FROM"),
        "Dockerfile should contain valid generated content"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn runtime_without_container_shows_missing() {
    let runner = E2eRunner::new();
    let test = "runtime-missing";
    runner.cleanup(test);

    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    let output = runner.aibox(test, &["get", "runtime"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("missing")
            || combined.contains("Missing")
            || combined.contains("not found"),
        "get runtime should report missing when no container exists: {}",
        combined
    );

    runner.cleanup(test);
}
/// Verify that `aibox init --context managed` creates the current slim skeleton.
///
/// processkit owns concrete workitem/decision/standup entities now; aibox init
/// only creates the project shell, context directory, and provider pointers.
#[test]
#[serial]
fn init_with_managed_preset_creates_context_files() {
    let runner = E2eRunner::new();
    let test = "init-managed-preset";
    runner.cleanup(test);

    let output = runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );
    assert!(
        output.status.success(),
        "init --context managed failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        runner.file_exists(test, "CLAUDE.md"),
        "CLAUDE.md should exist"
    );
    assert!(
        runner.file_exists(test, "aibox.toml"),
        "aibox.toml should exist"
    );

    assert!(
        runner.dir_exists(test, "context"),
        "context/ should exist for managed preset"
    );
    assert!(
        runner.file_exists(test, "AGENTS.md"),
        "AGENTS.md should exist for managed preset"
    );

    // aibox.toml should record the preset name
    let toml = runner.read_file(test, "aibox.toml");
    assert!(
        toml.contains("[skills]"),
        "aibox.toml should contain the skills selector, got:\n{}",
        toml
    );

    runner.cleanup(test);
}

/// Verify that `aibox init --context software` still creates the slim project shell.
#[test]
#[serial]
fn init_with_software_preset_creates_code_files() {
    let runner = E2eRunner::new();
    let test = "init-software-preset";
    runner.cleanup(test);

    let output = runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "software"],
    );
    assert!(
        output.status.success(),
        "init --context software failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        runner.dir_exists(test, "context"),
        "context/ should exist for software preset"
    );
    assert!(
        runner.file_exists(test, "AGENTS.md"),
        "AGENTS.md should exist for software preset"
    );

    runner.cleanup(test);
}
