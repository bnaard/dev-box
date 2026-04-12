//! Application smoke tests — verify container runtime access on the companion.
//!
//! Requires the e2e-runner companion container (feature = "e2e").
//! These tests validate that the remote host can run container operations using
//! whichever runtime is available there (`docker` preferred, `podman` fallback).
//!
//! NOTE: These tests are slow (image pull/build takes time). They are the most
//! comprehensive validation that the full pipeline works end-to-end.

use serial_test::serial;

use super::runner::E2eRunner;

/// Verify that a responsive container runtime is available on the companion.
#[test]
#[serial]
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

/// Verify that the companion runtime can pull a minimal image.
#[test]
#[serial]
#[ntest::timeout(120_000)]
fn runtime_can_pull_image() {
    let runner = E2eRunner::new();
    let runtime = runner.runtime_bin();
    let output = runner.exec(&format!(
        "{} pull --quiet docker.io/library/alpine:latest",
        runtime
    ));
    assert!(
        output.status.success(),
        "{} pull should succeed: {}",
        runtime,
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Verify that the companion runtime can run a container.
#[test]
#[serial]
#[ntest::timeout(120_000)]
fn runtime_can_run_container() {
    let runner = E2eRunner::new();
    let runtime = runner.runtime_bin();
    let output = runner.exec(&format!(
        "{} run --rm docker.io/library/alpine:latest echo hello-e2e",
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
