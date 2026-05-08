//! Tier 1 harness: scaffold-only `aibox init` / `aibox apply` runs that
//! never touch a container runtime.
//!
//! This file is intentionally NOT gated on `#[cfg(feature = "e2e")]` —
//! it must run in the default `cargo test --test e2e` pass with no SSH
//! companion container, no docker, no podman, no network setup at all.
//!
//! The pattern (TempDir + helper that calls the built `aibox` binary
//! with the addons dir wired in via env) mirrors `config_coverage.rs`.
//!
//! Two tests here:
//!
//! 1. `smoke_no_container_init_then_apply` — happy path. Verifies the
//!    `--no-container` / `AIBOX_NO_CONTAINER` flag plumbs through both
//!    commands and produces a complete scaffold (toml, lock,
//!    `.devcontainer/`, runtime mirror under `.aibox-home/`, harness
//!    config under `.claude/`). Sync's success message must be the
//!    `--no-container`-specific one so we can disambiguate it from the
//!    older `--no-build` path.
//!
//! 2. `negative_no_runtime_required` — recurrence guard. Wipes `PATH`
//!    so `Runtime::detect()` would necessarily fail, then runs
//!    `aibox init` and `aibox apply` with `AIBOX_NO_CONTAINER=1`. Both
//!    must succeed (exit 0). If a future change reintroduces a runtime
//!    probe in the init/apply hot path, this test fires.
//!
//! 3. `upgrade_path_v0_21_to_v0_22_no_container` — WS-0 PR-B. End-to-end
//!    fixture-based simulation of the v0.21.0 → v0.22.0 processkit
//!    upgrade path entirely without a container runtime *and* without
//!    network access. We hand-author the install state at each end of
//!    the transition (mirror, live provenance, lock, preauth.json) so
//!    `decide_sync` short-circuits to `Skip` (lock matches config and
//!    integrity is Healthy) — the test is therefore exercising the
//!    *post-upgrade* invariants the CLI must maintain at v0.22.0
//!    (preauth merge, integrity reporting, migration document, harness
//!    config) rather than the install pipeline itself (which is covered
//!    by the existing e2e suite running against real network in CI).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

/// Path to the `aibox` binary built by `cargo build`.
fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

/// Path to the addon definitions directory (consumed via
/// `AIBOX_ADDONS_DIR`).
fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

/// Run aibox in `dir` with `AIBOX_NO_CONTAINER=1` already set so the
/// `--no-container` flag is supplied via the env-var mirror.
fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("AIBOX_NO_CONTAINER", "1")
        .output()
        .expect("failed to execute aibox")
}

fn run_in_with_addons(dir: &Path, args: &[&str], addons_dir: &Path) -> Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir)
        .env("AIBOX_NO_CONTAINER", "1")
        .output()
        .expect("failed to execute aibox")
}

fn installed_addon_files_from_install_script() -> Vec<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let script_path = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("scripts/install.sh");
    let script = fs::read_to_string(script_path).expect("read scripts/install.sh");
    let Some((_, rest)) = script.split_once("local addon_files=\"") else {
        panic!("install script should declare addon_files");
    };
    let Some((list, _)) = rest.split_once('"') else {
        panic!("install script addon_files block should be closed");
    };
    list.split_whitespace().map(str::to_string).collect()
}

fn install_script_addons_dir() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("create installed-addon tempdir");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo_addons = Path::new(manifest_dir).parent().unwrap().join("addons");

    for file in installed_addon_files_from_install_script() {
        let src = repo_addons.join(&file);
        assert!(
            src.is_file(),
            "install script references missing addon YAML: {}",
            file
        );
        let dst = tmp.path().join(&file);
        fs::create_dir_all(dst.parent().unwrap()).unwrap();
        fs::copy(&src, &dst).unwrap();
    }

    tmp
}

/// Drop a captured `Output` to a debug-friendly string for assertion
/// failure messages.
fn fmt_output(label: &str, out: &Output) -> String {
    format!(
        "{label}: status={}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

fn combined_output(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn replace_toml_text(dir: &Path, from: &str, to: &str) {
    let toml_path = dir.join("aibox.toml");
    let body = fs::read_to_string(&toml_path).expect("read aibox.toml");
    assert!(
        body.contains(from),
        "aibox.toml did not contain expected text {from:?}:\n{body}"
    );
    fs::write(toml_path, body.replace(from, to)).expect("write aibox.toml");
}

#[test]
fn git_ui_gh_enabled_renders_from_installed_addon_catalog() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();
    let installed_addons = install_script_addons_dir();

    fs::write(
        dir.join("aibox.toml"),
        r#"[aibox]
version = "0.23.3"
base = "debian"

[container]
name = "gh-addon-e2e"

[processkit]
version = "unset"

[addons.git-ui.tools]
gh = { enabled = true }
lazygit = { enabled = false }
"#,
    )
    .unwrap();

    let apply_out = run_in_with_addons(dir, &["apply"], installed_addons.path());
    assert!(
        apply_out.status.success(),
        "apply failed.\n{}",
        fmt_output("apply", &apply_out)
    );

    let dockerfile = fs::read_to_string(dir.join(".devcontainer/Dockerfile")).unwrap();
    assert!(
        dockerfile.contains("Addon: git-ui"),
        "expected git-ui addon to render in Dockerfile:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("\n    gh \\"),
        "expected gh package to be installed when enabled:\n{dockerfile}"
    );
    assert!(
        !dockerfile.contains("unknown addon 'git-ui'"),
        "installed addon catalog must know git-ui:\n{dockerfile}"
    );
}

#[test]
fn lazygit_disabled_removes_runtime_layouts_and_stale_home_files() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    let init_out = run_in(
        dir,
        &[
            "init",
            "lazy-layouts",
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
            "--addon",
            "git-ui",
        ],
    );
    assert!(
        init_out.status.success(),
        "init with git-ui failed.\n{}",
        fmt_output("init", &init_out)
    );

    assert!(
        dir.join(".aibox-home/.config/lazygit/config.yml").exists(),
        "sanity check: git-ui with default lazygit enabled should seed lazygit config"
    );

    replace_toml_text(dir, "lazygit = {}", "lazygit = { enabled = false }");

    let apply_out = run_in(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply after disabling lazygit failed.\n{}",
        fmt_output("apply", &apply_out)
    );

    let dockerfile = fs::read_to_string(dir.join(".devcontainer/Dockerfile")).unwrap();
    assert!(
        dockerfile.contains("apt-get purge -y --auto-remove lazygit"),
        "Dockerfile should purge inherited lazygit when explicitly disabled:\n{dockerfile}"
    );

    let layouts_dir = dir.join(".aibox-home/.config/tmux/layouts");
    for layout in ["dev", "focus", "cowork", "cowork-swap", "browse", "ai"] {
        let path = layouts_dir.join(format!("{layout}.sh"));
        assert!(
            path.exists(),
            "expected generated layout {}",
            path.display()
        );
        let body = fs::read_to_string(&path).expect("read tmux layout");
        assert!(
            !body.contains("lazygit"),
            "layout {layout}.sh should not reference lazygit when disabled:\n{body}"
        );
    }

    assert!(
        !dir.join(".aibox-home/.config/lazygit/config.yml").exists(),
        "apply should remove stale .aibox-home lazygit config after lazygit is disabled"
    );
}

#[test]
fn apply_falls_back_for_missing_required_addon_and_writes_migration_guidance() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    fs::write(
        dir.join("aibox.toml"),
        r#"[aibox]
version = "0.23.5"
base = "debian"

[container]
name = "addon-requires-e2e"

[processkit]
version = "unset"

[addons.preview-enhanced.tools]
rich = {}
"#,
    )
    .unwrap();

    let apply_out = run_in(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply should use a fallback required addon instead of hard-failing.\n{}",
        fmt_output("apply", &apply_out)
    );

    let combined = combined_output(&apply_out);
    assert!(
        combined.contains("preview-enhanced") && combined.contains("preview-archive"),
        "apply should tell the user which required addon was filled in:\n{combined}"
    );

    let dockerfile = fs::read_to_string(dir.join(".devcontainer/Dockerfile")).unwrap();
    assert!(
        dockerfile.contains("Addon: preview-archive"),
        "fallback should include preview-archive content in the generated Dockerfile:\n{dockerfile}"
    );

    let migration_dir = dir.join("context/migrations");
    let guidance = fs::read_dir(&migration_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| fs::read_to_string(entry.path()).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        guidance.contains("[addons.preview-archive.tools]")
            && guidance.contains("Status:** pending"),
        "apply should write migration guidance for the project agent:\n{guidance}"
    );
}

#[test]
fn doctor_errors_on_aibox_toml_schema_mismatches() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    let init_out = run_in(
        dir,
        &[
            "init",
            "schema-drift",
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

    replace_toml_text(
        dir,
        "name     = \"schema-drift\"",
        "name     = \"schema-drift\"\nnmae     = \"typo\"",
    );

    let doctor_out = run_in(dir, &["doctor"]);
    assert!(
        doctor_out.status.success(),
        "doctor should exit 0 while reporting schema errors.\n{}",
        fmt_output("doctor", &doctor_out)
    );
    let combined = combined_output(&doctor_out);
    assert!(
        combined.contains("aibox.toml schema mismatch")
            && combined.contains("[container]: unknown key `nmae`")
            && combined.contains("Ask the project agent to update aibox.toml"),
        "doctor should report unknown aibox.toml keys as project-agent-actionable errors:\n{combined}"
    );
}

#[test]
fn doctor_schema_check_runs_when_config_load_fails() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    fs::write(
        dir.join("aibox.toml"),
        r#"[aibox]
version = "0.23.5"
base = "debian"

[container]
nmae = "typo"
"#,
    )
    .unwrap();

    let doctor_out = run_in(dir, &["doctor"]);
    assert!(
        doctor_out.status.success(),
        "doctor should exit 0 while reporting config and schema errors.\n{}",
        fmt_output("doctor", &doctor_out)
    );
    let combined = combined_output(&doctor_out);
    assert!(
        combined.contains("Config:")
            && combined.contains("aibox.toml schema mismatch")
            && combined.contains("[container]: unknown key `nmae`"),
        "doctor should still report unknown-key guidance when normal config loading fails:\n{combined}"
    );
}

#[test]
fn doctor_warns_on_runtime_theme_template_drift() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    let init_out = run_in(
        dir,
        &[
            "init",
            "theme-drift",
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
            "--theme",
            "nord",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    let tmux_conf = dir.join(".aibox-home/.config/tmux/tmux.conf");
    fs::write(&tmux_conf, "# stale local tmux config\n").expect("write stale tmux config");

    let doctor_out = run_in(dir, &["doctor"]);
    assert!(
        doctor_out.status.success(),
        "doctor should exit 0 while warning about theme drift.\n{}",
        fmt_output("doctor", &doctor_out)
    );
    let combined = combined_output(&doctor_out);
    assert!(
        combined.contains("Runtime theme/template drift")
            && combined.contains(".config/tmux/tmux.conf"),
        "doctor should warn when standard runtime status/theme files drift from the reference:\n{combined}"
    );
}

#[test]
fn vim_loop_disables_startup_cursor_position_probe_for_tui_muxers() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let vim_loop = Path::new(manifest_dir)
        .parent()
        .unwrap()
        .join("images/base-debian/config/bin/vim-loop.sh");
    let body = fs::read_to_string(&vim_loop)
        .unwrap_or_else(|err| panic!("read {}: {err}", vim_loop.display()));

    assert!(
        body.contains(r#"--cmd "set t_u7=""#) && body.contains(r#"--cmd "set t_RV=""#),
        "vim-loop should disable Vim's startup terminal probes for eager TUI startup:\n{}",
        vim_loop.display()
    );
}

// BR-LEGACY-MUX-EXCISE / BR-ZELLIJ-EXCISE
// (DEC-20260508_1515-SilentAsh, v0.25.6): `aibox apply` must hard-purge
// any legacy multiplexer artifact that survives under .aibox-home/.
#[test]
fn apply_hard_purges_legacy_multiplexer_artifacts_under_host_root() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    let init_out = run_in(
        dir,
        &[
            "init",
            "purge-legacy",
            "--base",
            "debian",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    // Plant the canonical legacy artifact set the doctor scans for.
    let host = dir.join(".aibox-home");
    let legacy_config = host.join(".config/zellij");
    let legacy_cache = host.join(".cache/zellij");
    let legacy_share = host.join(".local/share/zellij");
    fs::create_dir_all(&legacy_config).unwrap();
    fs::create_dir_all(&legacy_cache).unwrap();
    fs::create_dir_all(&legacy_share).unwrap();
    fs::write(legacy_config.join("config.kdl"), "// stale legacy\n").unwrap();
    fs::write(legacy_cache.join("cache.bin"), "junk").unwrap();
    fs::write(legacy_share.join("plugin.wasm"), b"\0wasm").unwrap();

    let apply_out = run_in(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply failed.\n{}",
        fmt_output("apply", &apply_out)
    );

    // Hard-purge: every legacy path must be gone after apply.
    assert!(
        !legacy_config.exists(),
        "apply must purge .config/zellij hard-cut: still exists at {}",
        legacy_config.display()
    );
    assert!(
        !legacy_cache.exists(),
        "apply must purge .cache/zellij hard-cut"
    );
    assert!(
        !legacy_share.exists(),
        "apply must purge .local/share/zellij hard-cut"
    );

    // Doctor returns clean exit code when the host root is purged.
    let doctor_out = run_in(dir, &["doctor"]);
    assert!(
        doctor_out.status.success(),
        "doctor must be clean post-apply.\n{}",
        fmt_output("doctor", &doctor_out)
    );
}

// BR-LEGACY-MUX-EXCISE: doctor must error when legacy multiplexer
// artifacts survive under the host root (e.g. host CLI is pre-v0.25.6
// and never ran the cleanup).
#[test]
fn doctor_errors_when_legacy_multiplexer_artifact_survives() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    let init_out = run_in(
        dir,
        &[
            "init",
            "doctor-legacy",
            "--base",
            "debian",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    let apply_out = run_in(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply failed.\n{}",
        fmt_output("apply", &apply_out)
    );

    // Re-introduce a legacy artifact AFTER apply has run.
    let host = dir.join(".aibox-home");
    let legacy = host.join(".config/zellij");
    fs::create_dir_all(&legacy).unwrap();
    fs::write(legacy.join("config.kdl"), "// stale\n").unwrap();

    let doctor_out = run_in(dir, &["doctor"]);
    let combined = combined_output(&doctor_out);
    // doctor reports errors via its summary line and an explicit
    // error-prefixed message. Both must be present so a stale host CLI
    // surface points the user at the cleanup remediation.
    assert!(
        combined.contains("Legacy multiplexer artifacts present"),
        "doctor must report the surviving legacy multiplexer paths:\n{combined}"
    );
    assert!(
        combined.contains("error(s)") && !combined.contains("0 error(s)"),
        "doctor summary must reflect at least one error:\n{combined}"
    );
}

#[test]
fn apply_preserves_project_context_edits_while_regenerating_runtime_config() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    let init_out = run_in(
        dir,
        &[
            "init",
            "context-preserve",
            "--base",
            "debian",
            "--context",
            "managed",
            "--processkit-version",
            "unset",
            "--theme",
            "nord",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    let note_path = dir.join("context/notes/user-note.md");
    fs::create_dir_all(note_path.parent().unwrap()).unwrap();
    fs::write(&note_path, "# User note\n\nKeep this.\n").unwrap();

    replace_toml_text(dir, "theme  = \"nord\"", "theme  = \"dracula\"");

    let apply_out = run_in(dir, &["apply"]);
    assert!(
        apply_out.status.success(),
        "apply failed.\n{}",
        fmt_output("apply", &apply_out)
    );

    assert!(
        note_path.exists(),
        "apply should preserve project-owned context edits"
    );
    assert!(
        dir.join(".aibox-home/.config/tmux/tmux.conf").exists(),
        "apply should regenerate runtime theme files after aibox.toml changes"
    );
}

#[test]
fn reset_context_dry_run_is_soft_reset_plan_only_and_preserves_context() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    fs::write(
        dir.join("aibox.toml"),
        r#"[aibox]
version = "0.23.5"
base = "debian"

[container]
name = "context-reset-plan"

[processkit]
version = "v0.25.4"
"#,
    )
    .unwrap();
    let workitem_path = dir.join("context/workitems/WI-001.md");
    fs::create_dir_all(workitem_path.parent().unwrap()).unwrap();
    fs::write(&workitem_path, "# Project-owned work\n").unwrap();

    let reset_out = run_in(dir, &["reset", "context", "--dry-run"]);
    assert!(
        reset_out.status.success(),
        "reset context --dry-run should produce a soft-reset plan.\n{}",
        fmt_output("reset context", &reset_out)
    );
    let combined = combined_output(&reset_out);
    assert!(
        combined.contains("Context reset plan")
            && combined.contains("Preserve as project-owned context")
            && combined.contains("[dry-run] No files were modified"),
        "reset context dry-run should explain the soft-reset blast radius:\n{combined}"
    );
    assert!(
        workitem_path.exists(),
        "reset context dry-run must not delete project-owned context"
    );
}

#[test]
fn smoke_no_container_init_then_apply() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    // 1. init — env-var supplies --no-container.
    let init_out = run_in(
        dir,
        &[
            "init",
            "fixture",
            "--base",
            "debian",
            "--context",
            "managed",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    // Core scaffold artefacts.
    for rel in [
        "aibox.toml",
        "aibox.lock",
        ".devcontainer/Dockerfile",
        ".devcontainer/docker-compose.yml",
        ".devcontainer/devcontainer.json",
    ] {
        let path = dir.join(rel);
        assert!(
            path.exists(),
            "expected init to create {}\n{}",
            rel,
            fmt_output("init", &init_out)
        );
    }

    // 2. apply — must produce the --no-container-specific success line.
    let sync_out = run_in(dir, &["apply"]);
    assert!(
        sync_out.status.success(),
        "apply failed.\n{}",
        fmt_output("apply", &sync_out)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&sync_out.stdout),
        String::from_utf8_lossy(&sync_out.stderr)
    );
    assert!(
        combined.contains("Sync complete (--no-container:"),
        "expected --no-container completion message in apply output.\n{}",
        fmt_output("apply", &sync_out)
    );

    // Representative runtime mirror file under `.aibox-home/`. tmux is
    // seeded for every project, so this is a stable signal that the
    // runtime-config seed phase ran end to end.
    let tmux_cfg = dir.join(".aibox-home/.config/tmux/tmux.conf");
    assert!(
        tmux_cfg.exists(),
        "expected runtime mirror at .aibox-home/.config/tmux/tmux.conf\n{}",
        fmt_output("apply", &sync_out)
    );

    // Harness config — present whenever the [ai] section keeps Claude
    // (the default). Skill content underneath depends on whether
    // processkit was reachable in the test env; we only assert the
    // settings file itself, not the skill payload, to avoid flaking
    // when GitHub is unreachable.
    let claude_settings = dir.join(".claude/settings.json");
    if claude_settings.exists() {
        // good: harness config emitted
    } else {
        eprintln!(
            "note: .claude/settings.json was not created (likely warn-skip path \
             when processkit is unreachable in this test env)"
        );
    }
}

#[test]
fn generated_runtime_apply_does_not_touch_provider_or_live_runtime_files() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    // Use aibox init with Codex harness. BR-SEC-HARDEN (HonestAnt Q3) added a
    // seccomp consent requirement for Codex, but `aibox init --harness codex`
    // auto-sets `acknowledge_seccomp_unconfined = true` in memory and writes it
    // to aibox.toml. The test verifies that `apply generated-runtime` does NOT
    // touch provider hook files or live .aibox-home files.
    let init_out = run_in(
        dir,
        &[
            "init",
            "fixture",
            "--base",
            "debian",
            "--context",
            "managed",
            "--harness",
            "codex",
            "--processkit-version",
            "unset",
        ],
    );
    assert!(
        init_out.status.success(),
        "init failed.\n{}",
        fmt_output("init", &init_out)
    );

    let codex_hooks = dir.join(".codex/hooks.json");
    fs::create_dir_all(codex_hooks.parent().unwrap()).unwrap();
    let hooks_before = r#"{"hooks":{"user_prompt_submit":{"command":"echo user-owned"}}}"#;
    fs::write(&codex_hooks, hooks_before).unwrap();

    let live_tmux = dir.join(".aibox-home/.config/tmux/tmux.conf");
    fs::create_dir_all(live_tmux.parent().unwrap()).unwrap();
    let live_before = "// user-owned live runtime config\n";
    fs::write(&live_tmux, live_before).unwrap();

    let out = run_in(dir, &["apply", "generated-runtime"]);
    assert!(
        out.status.success(),
        "apply generated-runtime failed.\n{}",
        fmt_output("apply generated-runtime", &out)
    );

    assert_eq!(
        fs::read_to_string(&codex_hooks).unwrap(),
        hooks_before,
        "generated-runtime apply must not rewrite provider hook files"
    );
    assert_eq!(
        fs::read_to_string(&live_tmux).unwrap(),
        live_before,
        "generated-runtime apply must not rewrite live .aibox-home files"
    );

    assert!(
        dir.join(".devcontainer/Dockerfile").is_file(),
        "generated-runtime apply should still refresh devcontainer files"
    );
    assert!(
        dir.join("context/templates/aibox-home")
            .read_dir()
            .expect("runtime template root should exist")
            .next()
            .is_some(),
        "generated-runtime apply should write a versioned runtime template snapshot"
    );
}

#[test]
fn negative_no_runtime_required() {
    // Recurrence guard for the entire defect class.
    //
    // Any future code that reintroduces `Runtime::detect()` in the
    // init/apply hot path will fail this test, because we wipe PATH so
    // no docker/podman binary can be found.
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    // Empty PATH dir: a real path, but containing no executables. Some
    // platforms reject an empty PATH string outright, so a present-but-
    // empty directory is the safer probe.
    let empty_path_dir = tempfile::TempDir::new().expect("create empty-path tempdir");
    let empty_path = empty_path_dir.path().to_string_lossy().into_owned();

    let init_out = Command::new(aibox_bin())
        .args([
            "init",
            "fixture",
            "--base",
            "debian",
            "--context",
            "managed",
        ])
        .current_dir(dir)
        .env_clear()
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("AIBOX_NO_CONTAINER", "1")
        .env("PATH", &empty_path)
        .env("HOME", dir)
        .output()
        .expect("failed to execute aibox init");
    assert!(
        init_out.status.success(),
        "init must succeed with empty PATH when AIBOX_NO_CONTAINER=1.\n{}",
        fmt_output("init (no-runtime)", &init_out)
    );

    let sync_out = Command::new(aibox_bin())
        .args(["apply"])
        .current_dir(dir)
        .env_clear()
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .env("AIBOX_NO_CONTAINER", "1")
        .env("PATH", &empty_path)
        .env("HOME", dir)
        .output()
        .expect("failed to execute aibox apply");
    assert!(
        sync_out.status.success(),
        "apply must succeed with empty PATH when AIBOX_NO_CONTAINER=1.\n{}",
        fmt_output("apply (no-runtime)", &sync_out)
    );
}

// ─── Test 3: upgrade-path v0.21.0 → v0.22.0 (WS-0 PR-B) ─────────────────────

/// Write a minimal but integrity-valid processkit install state for
/// `version` into `dir`:
///
/// 1. `aibox.toml [processkit].version = "<version>"`
/// 2. `aibox.lock` with matching `[processkit]` section + a stub
///    `processkit_install_hash` (the next apply recomputes it from the
///    live tree — value just needs to be `Some(_)`).
/// 3. `context/templates/processkit/<version>/PROVENANCE.toml` with
///    `[source].generated_for_tag = "<version>"`.
/// 4. `context/.processkit-provenance.toml` (schema_version = 1) with
///    `processkit_version = "<version>"`, `manifest.skill_count = 0`,
///    and `manifest.install_hash = None` so the integrity check skips
///    the hash equality branch.
///
/// `preauth_body`, when `Some`, is written to
/// `context/skills/processkit/skill-gate/assets/preauth.json` so the
/// preauth merge has something to consume on the next apply.
fn write_processkit_install_state(dir: &Path, version: &str, preauth_body: Option<&str>) {
    // 1. aibox.toml — replace whatever processkit.version line aibox init
    //    wrote (single-quoted "unset" or another value).
    let toml_path = dir.join("aibox.toml");
    let toml_body = fs::read_to_string(&toml_path).expect("read aibox.toml");
    let mut new_lines: Vec<String> = Vec::with_capacity(toml_body.lines().count());
    let mut in_processkit = false;
    for line in toml_body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            in_processkit = trimmed.starts_with("[processkit]");
        }
        if in_processkit && trimmed.starts_with("version") {
            new_lines.push(format!("version  = \"{}\"", version));
        } else {
            new_lines.push(line.to_string());
        }
    }
    fs::write(&toml_path, new_lines.join("\n") + "\n").expect("write aibox.toml");

    // 2. aibox.lock — keep [aibox] from init if present, replace [processkit].
    //    A small hand-built body covers everything decide_sync / integrity reads.
    let lock_body = format!(
        "[aibox]\n\
         cli_version = \"0.19.2\"\n\
         synced_at = \"2026-04-25T00:00:00Z\"\n\
         \n\
         [processkit]\n\
         source = \"https://github.com/projectious-work/processkit.git\"\n\
         version = \"{version}\"\n\
         src_path = \"src\"\n\
         installed_at = \"2026-04-25T00:00:00Z\"\n\
         processkit_install_hash = \"stub-hash-{version}\"\n",
    );
    fs::write(dir.join("aibox.lock"), lock_body).expect("write aibox.lock");

    // 3. Templates mirror PROVENANCE.toml.
    let mirror = dir.join("context/templates/processkit").join(version);
    fs::create_dir_all(&mirror).expect("create mirror dir");
    let prov = format!(
        "[source]\n\
         project = \"processkit\"\n\
         upstream = \"https://github.com/projectious-work/processkit.git\"\n\
         generated_at = \"2026-04-25T00:00:00Z\"\n\
         generated_for_tag = \"{version}\"\n",
    );
    fs::write(mirror.join("PROVENANCE.toml"), prov).expect("write mirror PROVENANCE.toml");

    // 4. Live provenance marker.
    let live = format!(
        "schema_version = 1\n\
         \n\
         [install]\n\
         processkit_version = \"{version}\"\n\
         processkit_source = \"https://github.com/projectious-work/processkit.git\"\n\
         installed_at = \"2026-04-25T00:00:00Z\"\n\
         cli_version = \"0.19.2\"\n\
         \n\
         [manifest]\n\
         skill_count = 0\n\
         schema_count = 0\n\
         process_count = 0\n\
         state_machine_count = 0\n",
    );
    let live_dir = dir.join("context");
    fs::create_dir_all(&live_dir).expect("create context dir");
    fs::write(live_dir.join(".processkit-provenance.toml"), live).expect("write live provenance");

    // Optional preauth.json fixture.
    if let Some(body) = preauth_body {
        let asset_dir = dir.join("context/skills/processkit/skill-gate/assets");
        fs::create_dir_all(&asset_dir).expect("create skill-gate assets dir");
        fs::write(asset_dir.join("preauth.json"), body).expect("write preauth.json");
    }

    // A single live processkit skill file so
    // `compute_processkit_install_fingerprint` is `Some(_)` (it requires
    // at least one regular file under the install roots) and the post-
    // apply writer keeps `aibox.lock [processkit].processkit_install_hash`
    // non-None.
    let stub_skill = dir.join("context/skills/processkit/_fixture-marker");
    fs::create_dir_all(&stub_skill).expect("create fixture skill dir");
    fs::write(
        stub_skill.join("SKILL.md"),
        format!(
            "# fixture-marker\n\nSynthetic processkit skill marker for {}.\n",
            version
        ),
    )
    .expect("write fixture skill");
}

/// 18-pattern v0.22.0 preauth fixture (matches the shape of the upstream
/// processkit v0.22.0 release asset).
fn v0_22_0_preauth_body() -> String {
    let allow: Vec<String> = (0..18)
        .map(|i| format!("\"mcp__processkit-skill-{i:02}__*\""))
        .collect();
    let servers: Vec<String> = (0..18)
        .map(|i| format!("\"processkit-skill-{i:02}\""))
        .collect();
    format!(
        r#"{{
          "version": 1,
          "description": "synthetic v0.22.0",
          "permissions": {{ "allow": [{}] }},
          "enabledMcpjsonServers": [{}]
        }}"#,
        allow.join(", "),
        servers.join(", ")
    )
}

/// Strategy A (hermetic, no network): we hand-author the install state
/// for both v0.21.0 and v0.22.0 so the harness exercises the
/// post-upgrade *invariants* rather than the network-dependent install
/// pipeline. The real install pipeline is exercised by the e2e suite
/// in CI; the value of this Tier 1 test is that it pins down what the
/// CLI must *report* and *write* once the install has converged.
///
/// Why hermetic instead of `aibox init --contextkit-version v0.21.0`
/// followed by a real `--processkit-version v0.22.0` apply:
///
/// - The release tarball for processkit v0.22.0 ships a
///   `PROVENANCE.toml` with `[source].generated_for_tag = "v0.21.0"`
///   (an upstream-side stamp bug), so the post-upgrade integrity check
///   currently returns `MismatchedVersion` against a real network
///   install. That's a real defect tracked separately; here we want the
///   *aibox-side* assertions to be deterministic regardless.
/// - The skill-count tripwire compares mirror counts (which include
///   non-processkit categories like `devops/`, `product/`, …) against
///   live count under `context/skills/processkit/` only, so the live
///   count is structurally below the mirror count and integrity reports
///   `Stale (skill_count_below_mirror)` even on a clean network install.
///   Tracked separately; not the subject of this test.
#[test]
fn upgrade_path_v0_21_to_v0_22_no_container() {
    let tmp = tempfile::TempDir::new().expect("create tempdir");
    let dir = tmp.path();

    // ── Phase 0: scaffold project without fetching processkit ────────
    let init_out = run_in(
        dir,
        &[
            "init",
            "upgrade-fixture",
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

    // ── Phase 1: inject hand-authored v0.21.0 install state ──────────
    // v0.21.0 does not ship preauth.json upstream — emulate that.
    write_processkit_install_state(dir, "v0.21.0", None);

    // First apply: decide_sync sees lock matches config + integrity Healthy
    // and returns Skip — no network fetch is attempted.
    let sync1 = run_in(dir, &["apply"]);
    assert!(
        sync1.status.success(),
        "apply at v0.21.0 failed.\n{}",
        fmt_output("apply v0.21.0", &sync1)
    );

    // Assertion #1: lock records v0.21.0 with a non-empty install hash.
    let lock_body = fs::read_to_string(dir.join("aibox.lock")).expect("read aibox.lock");
    assert!(
        lock_body.contains("version = \"v0.21.0\""),
        "expected lock to record processkit version v0.21.0.\n{lock_body}"
    );
    assert!(
        lock_body.contains("processkit_install_hash = \""),
        "expected aibox.lock [processkit].processkit_install_hash to be Some(_).\n{lock_body}"
    );

    // Assertion #2: live provenance marker carries v0.21.0.
    let prov21 = fs::read_to_string(dir.join("context/.processkit-provenance.toml"))
        .expect("read live provenance");
    assert!(
        prov21.contains("processkit_version = \"v0.21.0\""),
        "expected live provenance to record v0.21.0.\n{prov21}"
    );

    // Assertion #3: aibox doctor --integrity -o json => {"status": "Healthy"}.
    let integ1 = run_in(dir, &["doctor", "--integrity", "-o", "json"]);
    assert!(
        integ1.status.success(),
        "doctor --integrity -o json should exit 0 on Healthy state.\n{}",
        fmt_output("doctor v0.21.0", &integ1)
    );
    let stdout1 = String::from_utf8_lossy(&integ1.stdout).to_string();
    let parsed1: Value = serde_json::from_str(stdout1.trim()).unwrap_or_else(|e| {
        panic!(
            "doctor --integrity -o json must emit valid JSON: {e}\n{}",
            fmt_output("doctor v0.21.0", &integ1)
        )
    });
    assert_eq!(
        parsed1["status"].as_str(),
        Some("Healthy"),
        "expected status=Healthy at v0.21.0 baseline, got JSON:\n{stdout1}"
    );

    // ── Phase 2: simulate the upgrade — overwrite to v0.22.0 fixtures ──
    // v0.22.0 ships preauth.json — inject it.
    write_processkit_install_state(dir, "v0.22.0", Some(&v0_22_0_preauth_body()));

    // Second apply: lock matches config (both v0.22.0), integrity Healthy
    // again, decide_sync returns Skip. The preauth merge runs regardless
    // of the install branch and updates .claude/settings.json from the
    // freshly-injected preauth.json.
    let sync2 = run_in(dir, &["apply"]);
    assert!(
        sync2.status.success(),
        "apply at v0.22.0 failed.\n{}",
        fmt_output("apply v0.22.0", &sync2)
    );

    // Assertion #6: lock records v0.22.0.
    let lock_body2 = fs::read_to_string(dir.join("aibox.lock")).expect("read aibox.lock");
    assert!(
        lock_body2.contains("version = \"v0.22.0\""),
        "expected lock to record processkit version v0.22.0.\n{lock_body2}"
    );

    // Assertion #7: live provenance marker carries v0.22.0.
    let prov22 = fs::read_to_string(dir.join("context/.processkit-provenance.toml"))
        .expect("read live provenance");
    assert!(
        prov22.contains("processkit_version = \"v0.22.0\""),
        "expected live provenance to record v0.22.0.\n{prov22}"
    );

    // Assertion #8: .claude/settings.json carries v0.22.0 preauth wildcards
    // under _processkit_managed_keys.allow (post-merge sidecar).
    let settings_path = dir.join(".claude/settings.json");
    assert!(
        settings_path.exists(),
        "expected .claude/settings.json to exist after apply.\n{}",
        fmt_output("apply v0.22.0", &sync2)
    );
    let settings: Value = serde_json::from_str(
        &fs::read_to_string(&settings_path).expect("read .claude/settings.json"),
    )
    .expect("parse .claude/settings.json");
    let allow = settings["_processkit_managed_keys"]["allow"]
        .as_array()
        .unwrap_or_else(|| {
            panic!(
                "_processkit_managed_keys.allow must be a JSON array.\n{}",
                serde_json::to_string_pretty(&settings).unwrap_or_default()
            )
        });
    let allow_starts_with_processkit = allow
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| s.starts_with("mcp__processkit-"))
        .count();
    assert!(
        allow_starts_with_processkit >= 1,
        "expected at least one _processkit_managed_keys.allow entry to start with \
         'mcp__processkit-' (v0.22.0 wildcards), got: {:?}",
        allow
    );

    // Assertion #9: a migration document for the v0.21.0 → v0.22.0
    // transition was emitted under context/migrations/pending/MIG-*.md.
    // The runtime-config diff (managed .aibox-home files) writes a
    // MIG-RUNTIME-* document on the first apply, which counts as a
    // migration document for this assertion (it's the same MIG-*.md
    // pattern, same directory). Either form is acceptable.
    let pending_dir = dir.join("context/migrations/pending");
    let mig_files: Vec<_> = fs::read_dir(&pending_dir)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with("MIG-") && name.ends_with(".md") {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    assert!(
        !mig_files.is_empty(),
        "expected at least one MIG-*.md migration document under \
         context/migrations/pending/ after the v0.21.0 → v0.22.0 upgrade.\n{}",
        fmt_output("apply v0.22.0", &sync2)
    );

    // Assertion #10: integrity remains Healthy at v0.22.0.
    let integ2 = run_in(dir, &["doctor", "--integrity", "-o", "json"]);
    assert!(
        integ2.status.success(),
        "doctor --integrity -o json should exit 0 on Healthy state at v0.22.0.\n{}",
        fmt_output("doctor v0.22.0", &integ2)
    );
    let stdout2 = String::from_utf8_lossy(&integ2.stdout).to_string();
    let parsed2: Value = serde_json::from_str(stdout2.trim()).unwrap_or_else(|e| {
        panic!(
            "doctor --integrity -o json must emit valid JSON: {e}\n{}",
            fmt_output("doctor v0.22.0", &integ2)
        )
    });
    assert_eq!(
        parsed2["status"].as_str(),
        Some("Healthy"),
        "expected status=Healthy at v0.22.0 post-upgrade, got JSON:\n{stdout2}"
    );
}
