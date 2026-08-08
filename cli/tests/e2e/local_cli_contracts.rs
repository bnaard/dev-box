//! CLI, migration, reset, and addon contracts that require no remote host.

use super::local_runner::LocalProject;

#[test]
fn doctor_reports_missing_files() {
    let project = LocalProject::empty();
    let output = project.run(&["doctor"]);
    assert!(output.status.success(), "doctor is a diagnostic command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("aibox.toml") || stderr.contains("Config"));
}

#[test]
fn doctor_after_init_reports_healthy_checks() {
    let project = LocalProject::initialized("doctor-healthy", "managed", &[]);
    let output = project.run(&["doctor"]);
    project.assert_success("doctor", &output);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("ok")
            || combined.contains("OK")
            || combined.contains('✓')
            || combined.contains("pass")
    );
}

#[test]
fn apply_absorbs_legacy_version_file_into_lock() {
    let project = LocalProject::initialized("migration-version", "managed", &[]);
    project.write(".aibox-version", "0.1.0");
    let output = project.run(&["apply", "--no-container"]);
    project.assert_success("apply", &output);

    assert!(!project.exists(".aibox-version"));
    let lock = project.read("aibox.lock");
    assert!(lock.contains("[aibox]"));
    assert!(lock.contains("cli_version ="));
    assert!(!lock.contains("0.1.0"));
}

#[test]
fn reset_project_creates_backup() {
    let project = LocalProject::initialized("reset-backup", "managed", &[]);
    let output = project.run(&["reset", "project", "--yes"]);
    project.assert_success("reset project", &output);

    assert!(!project.exists("aibox.toml"));
    assert!(project.is_dir(".aibox/backup"));
    assert!(!project.is_dir(".aibox-backup"));
}

#[test]
fn reset_project_without_backup_deletes_generated_state() {
    let project = LocalProject::initialized("reset-no-backup", "managed", &[]);
    let output = project.run(&["reset", "project", "--no-backup", "--yes"]);
    project.assert_success("reset project --no-backup", &output);

    assert!(!project.exists("aibox.toml"));
    assert!(!project.is_dir(".devcontainer"));
    assert!(!project.is_dir(".aibox/backup"));
    assert!(!project.is_dir(".aibox-backup"));
}

#[test]
fn set_addon_modifies_toml() {
    let project = LocalProject::initialized("addon-add", "managed", &[]);
    let output = project.run(&["set", "addon", "python"]);
    project.assert_success("set addon python", &output);
    assert!(project.read("aibox.toml").contains("[addons.python"));
}

#[test]
fn delete_addon_cleans_toml() {
    let project = LocalProject::initialized("addon-remove", "managed", &["python"]);
    assert!(project.read("aibox.toml").contains("[addons.python"));
    let output = project.run(&["delete", "addon", "python"]);
    project.assert_success("delete addon python", &output);
    assert!(!project.read("aibox.toml").contains("[addons.python"));
}

#[test]
fn addon_apply_renders_tools_without_building_image() {
    let project = LocalProject::initialized("addon-rebuild", "managed", &["python"]);
    let output = project.run(&["apply", "--no-container"]);
    project.assert_success("apply", &output);
    let dockerfile = project.read(".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("python") || dockerfile.contains("Python") || dockerfile.contains("uv")
    );
}

#[test]
fn get_addon_shows_available_catalog_entries() {
    let project = LocalProject::initialized("addon-list", "managed", &[]);
    let output = project.run(&["get", "addon"]);
    project.assert_success("get addon", &output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("python") || stdout.contains("Python"));
}
