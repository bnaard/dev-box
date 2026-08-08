//! Container lifecycle E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").
//! Tests lifecycle behavior that inherently requires a real container runtime.

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial(companion_runtime)]
#[ntest::timeout(240_000)]
fn lifecycle_apply_starts_generated_container() {
    let runner = E2eRunner::new();
    let test = "lifecycle-container-up";
    let runtime = runner.runtime_bin();
    runner.exec(&format!("{runtime} rm -f {test} >/dev/null 2>&1 || true"));
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
    let published_version = runner
        .latest_published_image_version(test)
        .unwrap_or_else(|| {
            panic!(
                "required lifecycle gate has no usable published Debian image manifest; \
             mandatory runtime evidence must not be recorded as a passing skip"
            )
        });
    let version = runner.exec(&format!(
        "cd {workspace} && sed -i 's/^release_version = .*/release_version = \"{published_version}\"/' aibox.toml"
    ));
    assert!(
        version.status.success(),
        "failed to switch lifecycle container test to published image {published_version}:\nstdout:\n{}\nstderr:\n{}",
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

    let compose_file = format!("{workspace}/.devcontainer/docker-compose.yml");
    let up = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f {compose_file} up -d {test}"
    ));
    assert!(
        up.status.success(),
        "compose up failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );

    let probe_command = if published_version == env!("CARGO_PKG_VERSION") {
        "bash -lc 'test -r /etc/aibox-version && tmux -V && yazi --version && aibox-status --plugin-json >/tmp/aibox-status.json && jq -e .plain /tmp/aibox-status.json >/dev/null'"
    } else {
        "bash -lc 'test -r /etc/aibox-version && yazi --version >/dev/null'"
    };
    let probe = runner.container_exec(test, probe_command);
    assert!(
        probe.status.success(),
        "container probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );

    let down = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f {compose_file} down -v"
    ));
    assert!(
        down.status.success(),
        "compose down failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&down.stdout),
        String::from_utf8_lossy(&down.stderr)
    );

    runner.cleanup(test);
}

// ─── M1 companion: `--forget-tmux-state` clean attach ────────────────────────
//
// Gated `#[ignore]` because it requires a running container.
// Run on demand:
//   cargo test m1_forget_tmux_state_no_connect_error -- --ignored
//
// Lifts the forget-tmux-state assertions from
// `scripts/release-runtime-smoke.sh:303-340` into Rust e2e so they run
// every PR when the companion suite is available.
#[test]
#[serial(companion_runtime)]
#[ignore]
#[ntest::timeout(300_000)]
fn m1_forget_tmux_state_no_connect_error() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();

    let test = "m1-forget-tmux";
    let runtime = runner.runtime_bin();
    runner.exec(&format!("{runtime} rm -f {test} >/dev/null 2>&1 || true"));
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

    // Pin to a published image so we don't need a local image build.
    let workspace = format!("/workspaces/{test}");
    let published_version = runner
        .latest_published_image_version(test)
        .unwrap_or_else(|| {
            panic!(
                "required tmux-state gate has no usable published Debian image manifest; \
             mandatory runtime evidence must not be recorded as a passing skip"
            )
        });
    let pin = runner.exec(&format!(
        "cd {workspace} && sed -i 's/^release_version = .*/release_version = \"{published_version}\"/' aibox.toml"
    ));
    assert!(pin.status.success(), "failed to pin published version");

    let apply = runner.aibox(test, &["apply"]);
    assert!(
        apply.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&apply.stderr)
    );

    let compose_file = format!("{workspace}/.devcontainer/docker-compose.yml");
    let up = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f {compose_file} up -d {test}"
    ));
    assert!(
        up.status.success(),
        "compose up failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );

    // Wait for the container to be responsive.
    let _ = runner.exec(&format!(
        "for i in $(seq 1 30); do {runtime} exec {test} bash -c 'test -r /etc/aibox-version' >/dev/null 2>&1 && break; sleep 2; done"
    ));

    // The key assertion: `aibox up --forget-tmux-state` must not print
    // "error connecting to /tmp/tmux-1000/default" to stderr.
    let forget_out = runner.container_exec(
        test,
        "bash -lc 'aibox up --forget-tmux-state 2>&1 | head -50 || true'",
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&forget_out.stdout),
        String::from_utf8_lossy(&forget_out.stderr)
    );
    assert!(
        !combined.contains("error connecting to /tmp/tmux-1000/default"),
        "aibox up --forget-tmux-state must not produce stale-socket error:\n{combined}"
    );

    let down = runner.exec(&format!(
        "cd {workspace} && {runtime} compose -f {compose_file} down -v"
    ));
    assert!(
        down.status.success(),
        "compose down failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&down.stdout),
        String::from_utf8_lossy(&down.stderr)
    );

    runner.cleanup(test);
}
