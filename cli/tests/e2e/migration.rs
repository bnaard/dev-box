//! Version migration E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn sync_updates_version_file() {
    let runner = E2eRunner::new();
    let test = "migration-version";
    runner.cleanup(test);

    // Init project
    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    // Tamper with .aibox-version to simulate older version
    runner.write_file(test, ".aibox-version", "0.1.0");

    // TODO(hard-break): confirm whether `reset project`/`apply` still owns
    // `.aibox-version`; newer tests assert that this file is absent.
    runner.aibox(test, &["apply"]);

    let version = runner.read_file(test, ".aibox-version");
    assert!(
        !version.trim().is_empty(),
        ".aibox-version should not be empty after apply"
    );
    assert!(
        version.trim() != "0.1.0",
        ".aibox-version should be updated from 0.1.0 to current version"
    );

    runner.cleanup(test);
}
