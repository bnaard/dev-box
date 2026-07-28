//! Release-gated live Kubernetes lifecycle evidence.
//!
//! It is ignored by ordinary Tier 2 runs because it creates a real kind
//! cluster. The release gate invokes it through maintain.sh test-e2e.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::runner::E2eRunner;

fn candidate_commit() -> String {
    if let Ok(commit) = std::env::var("RELEASE_CANDIDATE_SHA")
        && !commit.trim().is_empty()
    {
        return commit;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli has a repository parent")
        .to_path_buf();
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()
        .expect("resolve test candidate commit");
    assert!(output.status.success(), "git rev-parse HEAD must succeed");
    String::from_utf8(output.stdout)
        .expect("commit is UTF-8")
        .trim()
        .to_string()
}

#[test]
#[ignore = "release gate: run through maintain.sh test-e2e"]
fn kubernetes_kind_lifecycle_produces_release_candidate_evidence() {
    let runner = E2eRunner::new();
    runner.ensure_deployed();
    assert_eq!(
        runner.runtime_bin(),
        "podman",
        "Kubernetes release gate is intentionally pinned to the Podman companion provider"
    );

    let preflight = runner.exec(
        "set -eu; kind version; kubectl version --client; \
         test \"$(ps -p 1 -o comm= | tr -d ' ')\" = systemd; \
         test \"$(stat -fc %T /sys/fs/cgroup)\" = cgroup2fs; \
         grep -qw pids /sys/fs/cgroup/cgroup.controllers; \
         grep -qw pids /sys/fs/cgroup/cgroup.subtree_control; \
         test \"$(podman info --format '{{.Host.CgroupManager}}')\" = systemd; \
         systemd-run --user --scope -p Delegate=yes --quiet true; test -d /lib/modules",
    );
    assert!(
        preflight.status.success(),
        "kind prerequisite failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&preflight.stdout),
        String::from_utf8_lossy(&preflight.stderr)
    );

    let commit = candidate_commit();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli has a repository parent")
        .join("scripts/test-kubernetes-kind.sh");
    runner.copy_file_to(
        script.to_str().expect("script path is UTF-8"),
        "/tmp/test-kubernetes-kind.sh",
    );
    let run = runner.exec(&format!(
        "chmod +x /tmp/test-kubernetes-kind.sh && \
         AIBOX_M7C_COMMIT={commit} AIBOX_BIN=/usr/local/bin/aibox \
         AIBOX_ADDONS_DIR=/opt/aibox/addons /tmp/test-kubernetes-kind.sh"
    ));
    assert!(
        run.status.success(),
        "kind disposable-cluster lifecycle failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let evidence: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("live M7c suite must return JSON evidence");
    assert_eq!(evidence["status"], "passed");
    assert_eq!(evidence["candidateCommit"], commit);
    assert!(
        evidence["binarySha256"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:") && digest.len() == 71)
    );
    assert_eq!(
        evidence["scenarios"].as_array().map(Vec::len),
        Some(8),
        "the live producer must report every executed scenario"
    );
    assert!(
        evidence["cluster"]
            .as_str()
            .is_some_and(|name| name.starts_with("aibox-m7c-"))
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("cli has a repository parent")
        .join(".aibox/release-evidence/m7c-live.json");
    fs::create_dir_all(path.parent().expect("attestation parent"))
        .expect("create local M7c evidence directory");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&evidence).expect("serialize evidence"),
    )
    .expect("retain local M7c attestation");
}
