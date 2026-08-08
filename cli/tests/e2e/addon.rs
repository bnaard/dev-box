//! Addon management E2E tests.
//!
//! Requires the e2e-runner companion container (feature = "e2e").

use serial_test::serial;

use super::runner::E2eRunner;

#[test]
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

    // Apply to regenerate. This test only inspects generated Dockerfile
    // content, so skip image build/runtime work.
    runner.aibox(test, &["apply", "--no-container"]);

    // Check Dockerfile contains python-related content
    let dockerfile = runner.read_file(test, ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("python") || dockerfile.contains("Python") || dockerfile.contains("uv"),
        "Dockerfile should contain python addon build stages"
    );

    runner.cleanup(test);
}

fn download_based_addons_build_with_published_defaults(test: &str, addons: &[&str]) {
    let runner = E2eRunner::new();
    runner.cleanup(test);

    let mut args = vec!["init", test, "--base", "debian", "--context", "managed"];
    for addon in addons {
        args.extend(["--addon", addon]);
    }

    let started = std::time::Instant::now();
    eprintln!("[e2e-stage] {test}: init started");
    let init = runner.aibox(test, &args);
    eprintln!(
        "[e2e-stage] {test}: init completed in {}s",
        started.elapsed().as_secs()
    );
    assert!(
        init.status.success(),
        "initializing the download-based add-on smoke project failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );

    let apply_started = std::time::Instant::now();
    eprintln!("[e2e-stage] {test}: apply/build started");
    let apply = runner.aibox(test, &["apply"]);
    eprintln!(
        "[e2e-stage] {test}: apply/build completed in {}s",
        apply_started.elapsed().as_secs()
    );
    if !apply.stdout.is_empty() {
        eprintln!(
            "[e2e-stage] {test}: apply stdout:\n{}",
            String::from_utf8_lossy(&apply.stdout)
        );
    }
    if !apply.stderr.is_empty() {
        eprintln!(
            "[e2e-stage] {test}: apply stderr:\n{}",
            String::from_utf8_lossy(&apply.stderr)
        );
    }
    assert!(
        apply.status.success(),
        "one or more published add-on defaults cannot build:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );

    runner.cleanup(test);
}

#[test]
#[ignore = "resource-heavy addon image build; run through scripts/run-e2e-shards.sh addon or all"]
#[ntest::timeout(3_600_000)]
fn download_based_addons_build_with_published_defaults_languages() {
    download_based_addons_build_with_published_defaults(
        "addon-download-languages",
        &[
            "docs-hugo",
            "docs-mdbook",
            "go",
            "go-quality",
            "go-release",
            "node",
            "rust",
            "typst",
        ],
    );
}

#[test]
#[ignore = "resource-heavy addon image build; run through scripts/run-e2e-shards.sh addon or all"]
#[ntest::timeout(3_600_000)]
fn download_based_addons_build_with_published_defaults_platforms() {
    download_based_addons_build_with_published_defaults(
        "addon-download-platforms",
        &[
            "cloud-aws",
            "cloud-gcp",
            "cloudflare",
            "infrastructure",
            "kubernetes",
        ],
    );
}

#[test]
#[ignore = "resource-heavy addon image build; run through scripts/run-e2e-shards.sh addon or all"]
#[ntest::timeout(3_600_000)]
fn download_based_addons_build_with_published_defaults_tools() {
    download_based_addons_build_with_published_defaults(
        "addon-download-tools",
        &[
            "ai-claude",
            "ai-opencode",
            "preview-archive",
            "supply-chain",
        ],
    );
}

#[test]
#[serial(companion_runtime)]
#[ignore = "resource-heavy nested Podman image build; run explicitly on the E2E companion"]
#[ntest::timeout(900_000)]
fn infrastructure_podman_runs_rootless_inside_generated_container() {
    let runner = E2eRunner::new();
    let test = "infrastructure-podman";
    runner.cleanup(test);

    let init = runner.aibox(
        test,
        &["init", test, "--base", "debian", "--context", "managed"],
    );
    assert!(
        init.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let mut config = runner.read_file(test, "aibox.toml");
    config.push_str(
        "\n[addons.infrastructure.tools]\n\
         opentofu = { enabled = false }\n\
         ansible = { enabled = false }\n\
         packer = { enabled = false }\n\
         podman = {}\n",
    );
    runner.write_file(test, "aibox.toml", &config);

    let apply = runner.aibox(test, &["apply"]);
    assert!(
        apply.status.success(),
        "Podman-enabled project failed to build:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&apply.stdout),
        String::from_utf8_lossy(&apply.stderr)
    );

    let runtime = runner.runtime_bin();
    let up = runner.exec(&format!(
        "{runtime} compose -f /workspaces/{test}/.devcontainer/docker-compose.yml up -d {test}"
    ));
    assert!(
        up.status.success(),
        "Podman-enabled project failed to start:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&up.stdout),
        String::from_utf8_lossy(&up.stderr)
    );

    let probe = runner.container_exec(
        test,
        "podman --version && podman-compose --version && podman info --format '{{.Host.Security.Rootless}}'",
    );
    assert!(
        probe.status.success() && String::from_utf8_lossy(&probe.stdout).contains("true"),
        "rootless Podman probe failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&probe.stdout),
        String::from_utf8_lossy(&probe.stderr)
    );

    runner.cleanup(test);
}

#[test]
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
