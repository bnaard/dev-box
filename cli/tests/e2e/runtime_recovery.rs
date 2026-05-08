//! M2 — Corrupted tmux.conf is recovered by `aibox apply` (BR-TEST-GAPS).
//!
//! Fixture: place a file with the known v0.25.3 corruption signature
//! (`set -g status off` + `set -g status-right " off_RIGHT "`) into
//! `.aibox-home/.config/tmux/tmux.conf`.
//!
//! Assert: after `aibox apply`, the file is rewritten to current generated
//! content and DOES contain the `tmux-powerkit.tmux` if-shell guard.
//!
//! This exercises the cross-version managed-runtime auto-overwrite recognizer
//! that landed in commit e0ee7bc.
//!
//! Tier 1 test — no container needed.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

fn run_no_container(dir: &Path, args: &[&str]) -> Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("AIBOX_NO_CONTAINER", "1")
        .output()
        .expect("failed to execute aibox")
}

fn fmt_output(label: &str, out: &Output) -> String {
    format!(
        "{label}: status={}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

/// The v0.25.3 corruption signature exactly as recognised by
/// `cli::runtime_sync::live_is_corrupted_v0_25_3_tmux_conf`.
const CORRUPTED_V0_25_3_TMUX_CONF: &str = r#"# corrupted tmux config — v0.25.3 signature
set -g status off
set -g status-right " off_RIGHT "
"#;

#[test]
fn corrupted_v0_25_3_tmux_conf_is_overwritten_by_apply() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    // 1. Init a minimal project.
    let init_out = run_no_container(
        dir,
        &[
            "init",
            "m2-recovery",
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    // 2. Overwrite the generated tmux.conf with the corrupted v0.25.3 signature.
    let tmux_conf_path = dir.join(".aibox-home/.config/tmux/tmux.conf");
    assert!(
        tmux_conf_path.exists(),
        "init should have created .aibox-home/.config/tmux/tmux.conf"
    );
    fs::write(&tmux_conf_path, CORRUPTED_V0_25_3_TMUX_CONF).expect("write corrupted tmux.conf");

    // Verify the signature is recognised by the library function.
    {
        let body = fs::read_to_string(&tmux_conf_path).expect("read tmux.conf");
        assert!(
            body.contains("set -g status off") && body.contains("off_RIGHT"),
            "test setup: corrupted signature should be present before apply"
        );
    }

    // 3. Run apply — should detect the corruption signature and overwrite.
    let apply_out = run_no_container(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply should succeed when recovering corrupted tmux.conf.\n{}",
        fmt_output("apply", &apply_out)
    );

    // 4. The tmux.conf must now contain current generated content, specifically
    //    the `tmux-powerkit.tmux` if-shell guard that the corruption removed.
    let recovered = fs::read_to_string(&tmux_conf_path).expect("read recovered tmux.conf");
    assert!(
        !recovered.contains("off_RIGHT"),
        "apply must overwrite the corrupted v0.25.3 tmux.conf: off_RIGHT corruption signature still present"
    );
    assert!(
        recovered.contains("tmux-powerkit.tmux"),
        "recovered tmux.conf must contain the tmux-powerkit.tmux if-shell guard:\n{recovered}"
    );

    // 5. Doctor should also report clean after the recovery.
    let doctor_out = run_no_container(dir, &["doctor"]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&doctor_out.stdout),
        String::from_utf8_lossy(&doctor_out.stderr)
    );
    assert!(
        !combined.contains("Corrupted v0.25.3 tmux.conf"),
        "doctor should be clean after apply recovered the corrupted tmux.conf:\n{combined}"
    );
}

#[test]
fn doctor_detects_corrupted_v0_25_3_tmux_conf_before_apply() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    let init_out = run_no_container(
        dir,
        &[
            "init",
            "m2-doctor-corrupt",
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    // Plant the corruption signature.
    let tmux_conf_path = dir.join(".aibox-home/.config/tmux/tmux.conf");
    fs::write(&tmux_conf_path, CORRUPTED_V0_25_3_TMUX_CONF).expect("write corrupted tmux.conf");

    let doctor_out = run_no_container(dir, &["doctor"]);
    // Doctor always exits 0.
    assert!(
        doctor_out.status.success(),
        "doctor should exit 0 even when corruption is detected.\n{}",
        fmt_output("doctor", &doctor_out)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&doctor_out.stdout),
        String::from_utf8_lossy(&doctor_out.stderr)
    );
    assert!(
        combined.contains("Corrupted v0.25.3 tmux.conf"),
        "doctor should report the v0.25.3 corruption signature before apply:\n{combined}"
    );
    assert!(
        combined.contains("error(s)") && !combined.contains("0 error(s)"),
        "doctor summary should reflect at least one error for corrupted tmux.conf:\n{combined}"
    );
}
