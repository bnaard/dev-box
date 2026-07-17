//! Application smoke tests — verify container runtime access on the companion.
//!
//! Requires the e2e-runner companion container (feature = "e2e").
//! These tests validate that the remote host can run container operations using
//! whichever runtime is available there (`docker` preferred, `podman` fallback).
//!
//! NOTE: Keep these checks minimal. The lifecycle suite owns the generated
//! aibox container build/start probe; this module only proves the companion
//! runtime can execute a small public image.

use serial_test::serial;

use super::runner::E2eRunner;

/// Verify that a responsive container runtime is available on the companion.
#[test]
#[serial(companion_runtime)]
fn runtime_available_on_companion() {
    let runner = E2eRunner::new();
    let runtime = runner.runtime_bin();
    let output = runner.exec(&format!("{} --version", runtime));
    assert!(
        output.status.success(),
        "{} should be available on e2e-runner: {}",
        runtime,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
    assert!(
        stdout.contains(&runtime),
        "{} --version should output version info",
        runtime
    );
}

/// Verify that the companion runtime can pull and run a minimal image.
#[test]
#[serial(companion_runtime)]
#[ntest::timeout(120_000)]
fn runtime_can_pull_and_run_container() {
    let runner = E2eRunner::new();
    let runtime = runner.runtime_bin();
    let output = runner.exec(&format!(
        "{} run --rm --network none docker.io/library/alpine:latest echo hello-e2e",
        runtime
    ));
    assert!(
        output.status.success(),
        "{} run should succeed: {}",
        runtime,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("hello-e2e"),
        "container should output hello-e2e"
    );
}
