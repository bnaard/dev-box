//! Addon management E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
#[serial]
fn set_addon_modifies_toml() {
    let runner = E2eRunner::new();
    let test = "addon-add";
    runner.cleanup(test);

    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    // Add python addon
    let output = runner.aibox(test, &["set", "addon", "python"]);
    assert!(
        output.status.success(),
        "set addon python failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that aibox.toml now contains the python addon
    let toml = runner.read_file(test, "aibox.toml");
    assert!(
        toml.contains("[addons.python"),
        "aibox.toml should contain [addons.python] after set addon"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn delete_addon_cleans_toml() {
    let runner = E2eRunner::new();
    let test = "addon-remove";
    runner.cleanup(test);

    // Init with python addon
    runner.aibox(
        test,
        &[
            "init",
            test,
            "--base",
            "debian",
            "--context",
            "managed",
            "--addon",
            "python",
        ],
    );

    // Verify it's there
    let toml = runner.read_file(test, "aibox.toml");
    assert!(
        toml.contains("[addons.python"),
        "python addon should be in toml after init"
    );

    // Remove it
    let output = runner.aibox(test, &["delete", "addon", "python"]);
    assert!(
        output.status.success(),
        "delete addon python failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify it's gone
    let toml = runner.read_file(test, "aibox.toml");
    assert!(
        !toml.contains("[addons.python"),
        "aibox.toml should not contain [addons.python] after delete addon"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn addon_rebuild_includes_tools_in_dockerfile() {
    let runner = E2eRunner::new();
    let test = "addon-rebuild";
    runner.cleanup(test);

    // Init with python addon
    runner.aibox(
        test,
        &[
            "init",
            test,
            "--base",
            "debian",
            "--context",
            "managed",
            "--addon",
            "python",
        ],
    );

    // Apply to regenerate
    runner.aibox(test, &["apply"]);

    // Check Dockerfile contains python-related content
    let dockerfile = runner.read_file(test, ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("python") || dockerfile.contains("Python") || dockerfile.contains("uv"),
        "Dockerfile should contain python addon build stages"
    );

    runner.cleanup(test);
}

#[test]
#[serial]
fn get_addon_shows_available() {
    let runner = E2eRunner::new();
    let test = "addon-list";
    runner.cleanup(test);

    runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );

    let output = runner.aibox(test, &["get", "addon"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("python") || stdout.contains("Python"),
        "get addon should show python as available"
    );

    runner.cleanup(test);
}
