//! Release-gated disposable Kubernetes validation.
//!
//! This remains ignored in a plain `cargo test --features e2e` invocation
//! because it creates a real cluster. `maintain.sh test-e2e` opts in and makes
//! it part of release validation. The companion runs systemd as PID 1 and the
//! cluster creation command in an explicitly delegated user scope.

use super::runner::E2eRunner;

#[test]
#[ignore = "release gate: run through maintain.sh test-e2e"]
fn kubernetes_kind_prerequisite_and_deterministic_cleanup() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();
    let runtime = runner.runtime_bin();
    assert_eq!(
        runtime, "podman",
        "Kubernetes release gate is intentionally pinned to the Podman companion provider"
    );

    let preflight = runner.exec(
        "set -eu; \
         kind version; \
         kubectl version --client; \
         test \"$(ps -p 1 -o comm= | tr -d ' ')\" = systemd; \
         test \"$(stat -fc %T /sys/fs/cgroup)\" = cgroup2fs; \
         grep -qw pids /sys/fs/cgroup/cgroup.controllers; \
         grep -qw pids /sys/fs/cgroup/cgroup.subtree_control; \
         test \"$(podman info --format '{{.Host.CgroupManager}}')\" = systemd; \
         systemd-run --user --scope -p Delegate=yes --quiet true; \
         test -d /lib/modules",
    );
    assert!(
        preflight.status.success(),
        "kind requires systemd PID 1, a delegated SSH user scope, cgroup v2 controllers, and /lib/modules. Rebuild the companion from the current Dockerfile.e2e and docker-compose.override.yml:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&preflight.stdout),
        String::from_utf8_lossy(&preflight.stderr)
    );

    // A unique name keeps concurrent release jobs isolated.  Trap cleanup is
    // registered before cluster creation so a failed first/changed/drift run
    // cannot leak node containers or kubeconfig state.
    let run = runner.exec(
        "set -eu; \
         export KIND_EXPERIMENTAL_PROVIDER=podman; \
         name=aibox-m7c-${RANDOM}; \
         trap 'kind delete cluster --name \"$name\" >/dev/null 2>&1 || true' EXIT; \
         systemd-run --user --scope -p Delegate=yes --quiet \
           env KIND_EXPERIMENTAL_PROVIDER=podman \
           kind create cluster --name \"$name\" --wait 90s; \
         kubectl --context \"kind-$name\" get nodes; \
         kind delete cluster --name \"$name\"; \
         trap - EXIT",
    );
    assert!(
        run.status.success(),
        "kind disposable-cluster smoke failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}
