//! Reset and backup E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use super::runner::E2eRunner;

#[test]
fn reset_project_creates_backup() {
    let runner = E2eRunner::new();
    let test = "reset-backup";
    runner.cleanup(test);

    // Init project
    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );
    assert!(runner.file_exists(test, "aibox.toml"));

    // Reset project (with backup, auto-confirm)
    let output = runner.aibox(test, &["reset", "project", "--yes"]);
    assert!(
        output.status.success(),
        "reset project failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // aibox.toml should be deleted after reset
    assert!(
        !runner.file_exists(test, "aibox.toml"),
        "aibox.toml should be deleted after reset"
    );

    // Backup directory should exist under the managed .aibox state dir.
    assert!(
        runner.dir_exists(test, ".aibox/backup"),
        ".aibox/backup should exist after reset"
    );
    assert!(
        !runner.dir_exists(test, ".aibox-backup"),
        "legacy .aibox-backup should not be created after reset"
    );

    runner.cleanup(test);
}

#[test]
fn reset_project_no_backup_deletes_all() {
    let runner = E2eRunner::new();
    let test = "reset-no-backup";
    runner.cleanup(test);

    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    let output = runner.aibox(test, &["reset", "project", "--no-backup", "--yes"]);
    assert!(
        output.status.success(),
        "reset project --no-backup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        !runner.file_exists(test, "aibox.toml"),
        "aibox.toml should be gone"
    );
    assert!(
        !runner.dir_exists(test, ".devcontainer"),
        ".devcontainer should be gone"
    );
    assert!(
        !runner.dir_exists(test, ".aibox/backup"),
        ".aibox/backup should not exist with --no-backup"
    );
    assert!(
        !runner.dir_exists(test, ".aibox-backup"),
        "legacy .aibox-backup should not exist with --no-backup"
    );

    runner.cleanup(test);
}
