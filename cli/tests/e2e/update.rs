//! Update command E2E tests.
//!
//! Runs in an isolated local project. These tests require public GHCR network
//! access, but no container runtime, clean host, or elevated authority.

use super::local_runner::LocalProject;

/// Verify that `aibox self update --check` successfully fetches version info from GHCR.
///
/// The GHCR packages are public, so anonymous token exchange should succeed and
/// the CLI should find published tags matching the `base-debian-v*` pattern.
/// This test catches tag-prefix mismatches between the CLI and the registry.
#[test]
fn update_check_fetches_from_registry() {
    let project = LocalProject::initialized("update-registry-fetch", "managed", &[]);

    // Run self update --check — should fetch real version info from GHCR.
    let output = project.run(&["self", "update", "--check"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        output.status.success(),
        "aibox self update --check should exit 0.\nOutput:\n{}",
        combined
    );

    // Verify the registry fetch succeeded — output should show the image is
    // up to date or that a new version is available, NOT a "Could not" warning.
    assert!(
        !combined.contains("Could not fetch latest image version"),
        "expected successful registry fetch, but got a warning.\nOutput:\n{}",
        combined
    );
    assert!(
        !combined.contains("No published tags found"),
        "tag prefix mismatch: no tags matched the expected pattern.\nOutput:\n{}",
        combined
    );

    // Should report image status (either up-to-date or upgrade available)
    assert!(
        combined.contains("is up to date") || combined.contains("New image version available"),
        "expected image version status in output, got:\n{}",
        combined
    );
}

/// Verify `aibox self update --dry-run` fetches the latest version from GHCR without
/// applying changes.
///
/// This exercises the full `do_upgrade` code path including the tag-prefix
/// matching, but stops before writing to aibox.toml thanks to `--dry-run`.
#[test]
fn update_dry_run_fetches_from_registry() {
    let project = LocalProject::initialized("update-dry-run", "managed", &[]);

    let output = project.run(&["self", "update", "--dry-run"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        output.status.success(),
        "aibox self update --dry-run should exit 0.\nOutput:\n{}",
        combined
    );

    let has_incomplete_manifest_fallback = combined
        .contains("No usable published tags found for flavor")
        && combined.contains("incomplete manifests");
    assert!(
        !combined.contains("Could not fetch latest image version")
            || has_incomplete_manifest_fallback,
        "expected successful registry fetch or known pre-publish incomplete-manifest fallback.\nOutput:\n{}",
        combined
    );

    // Should show current version and either "already at the latest" or "[dry-run]"
    assert!(
        combined.contains("Current image version:"),
        "expected 'Current image version:' in output, got:\n{}",
        combined
    );
    assert!(
        combined.contains("is already at the latest")
            || combined.contains("[dry-run]")
            || has_incomplete_manifest_fallback,
        "expected dry-run, up-to-date, or known incomplete-manifest fallback output, got:\n{}",
        combined
    );
}
