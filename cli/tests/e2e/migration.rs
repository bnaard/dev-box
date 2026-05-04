//! Version migration E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn apply_absorbs_legacy_version_file_into_lock() {
    let runner = E2eRunner::new();
    let test = "migration-version";
    runner.cleanup(test);

    // Init project
    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    // Legacy projects used .aibox-version. Current projects absorb CLI
    // sync state into aibox.lock and remove the standalone file on apply.
    runner.write_file(test, ".aibox-version", "0.1.0");

    let output = runner.aibox(test, &["apply"]);
    assert!(
        output.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        !runner.file_exists(test, ".aibox-version"),
        ".aibox-version should be removed after apply"
    );

    let lock = runner.read_file(test, "aibox.lock");
    assert!(
        lock.contains("[aibox]"),
        "aibox.lock should contain an [aibox] section after apply:\n{lock}"
    );
    assert!(
        lock.contains("cli_version ="),
        "aibox.lock should record the synced CLI version after apply:\n{lock}"
    );
    assert!(
        !lock.contains("0.1.0"),
        "legacy .aibox-version content should not survive in aibox.lock:\n{lock}"
    );

    runner.cleanup(test);
}
