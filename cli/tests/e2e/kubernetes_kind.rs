//! Release-gated disposable Kubernetes validation.
//!
//! This deliberately remains ignored in ordinary Tier-2 runs: kind requires
//! the nested Podman host to delegate the cgroup `pids` controller.  The
//! runner image has pinned kubectl/kind, while CI/release infrastructure must
//! opt in with `cargo test --features e2e kubernetes_kind -- --ignored`.

use super::runner::E2eRunner;

#[test]
#[ignore = "release gate: requires nested Podman with delegated cgroup pids controller"]
fn kubernetes_kind_prerequisite_and_deterministic_cleanup() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();
    let runtime = runner.runtime_bin();
    assert_eq!(
        runtime, "podman",
        "Kubernetes release gate is intentionally pinned to the Podman companion provider"
    );

    let preflight = runner.exec("kind version && kubectl version --client && grep -qw pids /sys/fs/cgroup/cgroup.controllers");
    assert!(
        preflight.status.success(),
        "kind requires nested Podman cgroup delegation including the pids controller. The current companion cannot create a kind node until that host prerequisite is enabled:\n{}",
        String::from_utf8_lossy(&preflight.stderr)
    );

    // A unique name keeps concurrent release jobs isolated.  Trap cleanup is
    // registered before cluster creation so a failed first/changed/drift run
    // cannot leak node containers or kubeconfig state.
    let run = runner.exec("set -eu; name=aibox-m7c-${RANDOM}; trap 'kind delete cluster --name \"$name\" >/dev/null 2>&1 || true' EXIT; kind create cluster --name \"$name\" --wait 90s; kubectl --context \"kind-$name\" get nodes; kind delete cluster --name \"$name\"; trap - EXIT");
    assert!(
        run.status.success(),
        "kind disposable-cluster smoke failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}
