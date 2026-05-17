//! Table-driven tests for comprehensive aibox.toml settings coverage.
//!
//! Each test case:
//! 1. Runs `aibox init` with a base config
//! 2. Patches `aibox.toml` with specific settings
//! 3. Runs `aibox apply`
//! 4. Asserts the generated files contain/don't contain expected strings
//!
//! These are Tier 1 tests — no running container needed, fast.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Get the path to the aibox binary.
fn aibox_bin() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/target/debug/aibox", manifest_dir)
}

/// Get the path to addon definitions.
fn addons_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    format!("{}/../addons", manifest_dir)
}

/// Run aibox in a directory.
fn run_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(aibox_bin())
        .args(args)
        .current_dir(dir)
        .env("AIBOX_ADDONS_DIR", addons_dir())
        .output()
        .expect("failed to execute aibox")
}

/// Initialize a project in a temp dir with default settings.
fn init_project(dir: &std::path::Path, name: &str) {
    let output = run_in(
        dir,
        &["init", name, "--base", "debian", "--context", "managed"],
    );
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Patch the aibox.toml in a directory by appending TOML content.
/// For replacing sections, read the file, do string replacement, write back.
fn patch_toml(dir: &std::path::Path, patch: &str) {
    let toml_path = dir.join("aibox.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read aibox.toml");
    fs::write(&toml_path, format!("{}\n{}", content, patch)).expect("failed to write aibox.toml");
}

/// Replace a section in aibox.toml.
fn replace_toml_section(dir: &std::path::Path, section: &str, replacement: &str) {
    let toml_path = dir.join("aibox.toml");
    let content = fs::read_to_string(&toml_path).expect("failed to read aibox.toml");

    // Find the section header at the start of a line (not in comments).
    // We search for "\n[section]" to avoid matching comment lines like "# [section]".
    let section_header = format!("[{}]", section);
    let needle = format!("\n{}", section_header);
    if let Some(needle_pos) = content.find(&needle) {
        // start = position of the "[" in the section header (after the \n)
        let start = needle_pos + 1;
        // Find next top-level single-bracket section. Array-of-table entries
        // like `[[ai.harnesses]]` belong to the current section.
        let mut end = content.len();
        let mut cursor = start + section_header.len();
        while let Some(relative) = content[cursor..].find("\n[") {
            let candidate = cursor + relative + 1;
            if !content[candidate..].starts_with("[[") {
                end = candidate;
                break;
            }
            cursor = candidate + 2;
        }
        let new_content = format!(
            "{}{}\n{}{}",
            &content[..start],
            &section_header,
            replacement,
            &content[end..]
        );
        fs::write(&toml_path, new_content).expect("failed to write aibox.toml");
    } else {
        // Section doesn't exist, append it
        patch_toml(dir, &format!("[{}]\n{}", section, replacement));
    }
}

/// Sync (regenerate) the project files.
fn sync_project(dir: &std::path::Path) {
    let output = run_in(dir, &["apply"]);
    assert!(
        output.status.success(),
        "apply failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn processkit_version_from_toml(dir: &Path) -> String {
    let body = fs::read_to_string(dir.join("aibox.toml")).expect("failed to read aibox.toml");
    let mut in_processkit = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('[') {
            in_processkit = trimmed.starts_with("[processkit]");
        }
        if in_processkit && trimmed.starts_with("version") {
            return trimmed
                .split_once('=')
                .map(|(_, value)| value.trim().trim_matches('"').to_string())
                .expect("processkit version line should contain =");
        }
    }
    panic!("aibox.toml should contain [processkit].version");
}

fn seed_processkit_install_fixture(dir: &Path) {
    let version = processkit_version_from_toml(dir);
    let source = "https://github.com/projectious-work/processkit.git";
    let cli_version = env!("CARGO_PKG_VERSION");
    fs::write(
        dir.join("aibox.lock"),
        format!(
            "[aibox]\n\
             cli_version = \"{cli_version}\"\n\
             synced_at = \"2026-05-07T00:00:00Z\"\n\
             \n\
             [processkit]\n\
             source = \"{source}\"\n\
             version = \"{version}\"\n\
             src_path = \"src\"\n\
             installed_at = \"2026-05-07T00:00:00Z\"\n\
             processkit_install_hash = \"stale-fixture-hash\"\n"
        ),
    )
    .expect("failed to write processkit fixture lock");

    let mirror = dir.join("context/templates/processkit").join(&version);
    fs::create_dir_all(&mirror).expect("failed to create processkit fixture mirror");
    fs::write(
        mirror.join("PROVENANCE.toml"),
        format!(
            "[source]\n\
             project = \"processkit\"\n\
             upstream = \"{source}\"\n\
             generated_at = \"2026-05-07T00:00:00Z\"\n\
             generated_for_tag = \"{version}\"\n"
        ),
    )
    .expect("failed to write processkit fixture provenance");

    let context = dir.join("context");
    fs::create_dir_all(&context).expect("failed to create processkit fixture context");
    fs::write(
        context.join(".processkit-provenance.toml"),
        format!(
            "schema_version = 1\n\
             \n\
             [install]\n\
             processkit_version = \"{version}\"\n\
             processkit_source = \"{source}\"\n\
             installed_at = \"2026-05-07T00:00:00Z\"\n\
             cli_version = \"{cli_version}\"\n\
             \n\
             [manifest]\n\
             skill_count = 0\n\
             schema_count = 0\n\
             process_count = 0\n\
             state_machine_count = 0\n"
        ),
    )
    .expect("failed to write processkit fixture live provenance");

    let skill = context.join("skills/processkit/_fixture-marker");
    fs::create_dir_all(&skill).expect("failed to create processkit fixture skill");
    fs::write(
        skill.join("SKILL.md"),
        format!("# fixture-marker\n\nSynthetic processkit skill marker for {version}.\n"),
    )
    .expect("failed to write processkit fixture skill");
}

/// Read a generated file relative to the project directory.
fn read_generated(dir: &std::path::Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).unwrap_or_else(|e| panic!("failed to read {}: {}", path, e))
}

// ─── Container Section Tests ─────────────────────────────────────────────────

#[test]
fn container_name_in_compose() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "my-project");
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains("container_name: my-project"),
        "compose should contain container_name: my-project"
    );
}

#[test]
fn container_hostname_in_compose() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "my-project");
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    // Default hostname is "aibox" unless overridden
    assert!(
        compose.contains("hostname:"),
        "compose should contain hostname"
    );
}

#[test]
fn tmux_runtime_config_and_cache_are_mounted() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "tmux-runtime");
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".config:/home/aibox/.config:rw")
            && compose.contains(".vim:/home/aibox/.vim:rw")
            && compose.contains(".tmux:/home/aibox/.tmux:rw")
            && compose.contains(".cache:/home/aibox/.cache:rw")
            && compose.contains(".local:/home/aibox/.local:rw")
            && !compose.contains(".config/tmux:/home/aibox/.config/tmux")
            && !compose.contains(".vim/vimrc:/home/aibox/.vim/vimrc")
            && !compose.contains(".vim/undo:/home/aibox/.vim/undo")
            && !compose.contains(".cargo/registry:/home/aibox/.cargo/registry")
            && !compose.contains(".cargo/git:/home/aibox/.cargo/git"),
        "compose must mount broad writable runtime-home config/cache/local/vim/tmux parents:\n{compose}"
    );
}

#[test]
fn rust_addon_cargo_cache_mounts_do_not_shadow_toolchain_shims() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "cargo-cache");
    patch_toml(
        dir.path(),
        r#"
[addons.rust.tools]
rustc = {}
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".cargo/registry:/home/aibox/.cargo/registry:rw")
            && compose.contains(".cargo/git:/home/aibox/.cargo/git:rw")
            && !compose.contains(".cargo:/home/aibox/.cargo:rw"),
        "compose must mount Cargo caches without shadowing image-provided cargo/rustc shims:\n{compose}"
    );
}

#[test]
fn init_generates_tmux_customization_surface() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "tmux-config");
    let toml = fs::read_to_string(dir.path().join("aibox.toml")).unwrap();
    assert!(
        toml.contains("[customization.tmux]")
            && toml.contains("[customization.tmux.status]")
            && toml.contains("# Layout sketches, one screen each:")
            && toml.contains("# +---- ai ----+  +--- dev ----+  +-- focus --+  +-- cowork -+")
            && !toml.contains("[customization.zellij_status]"),
        "generated aibox.toml should expose tmux customization and omit Zellij status:\n{toml}"
    );
    assert!(
        toml.contains("legacy: powerline -> extended")
            && toml.contains("[customization.tmux.status.layout]")
            && toml.contains("# Allowed line1-left entries:")
            && toml.contains("# - cloudstatus: networked public provider status checks; opt-in, not enabled by default")
            && toml.contains("[customization.tmux.status.labels]")
            && toml.contains("# Visible headers/icons for status segments.")
            && toml.contains(r#"aibox-log = "󱖫""#)
            && toml.contains("netspeed-download")
            && toml.contains("[customization.tmux.status.separators]")
            && toml.contains("style = \"rounded\"")
            && toml.contains("edge-style = \"rounded\"")
            && toml.contains("elements-spacing = \"both\"")
            && toml.contains("[customization.tmux.status.refresh]")
            && toml.contains("interval-seconds = 15")
            && toml.contains("aibox-metrics-cache-ttl-seconds = 30")
            && toml.contains("[customization.tmux.status.model-providers]")
            && toml.contains("#   Symbols: ✓ ok, 󰀦 degraded, 󰚌 outage, ? unknown.")
            && toml.contains(r#"provider = "openai""#)
            && toml.contains(r#"line1-left = ["session", "windows"]"#)
            && toml
                .contains(r#"line2-left = ["forge", "kubernetes", "terraform", "cloud"]"#),
        "generated aibox.toml should explain tmux status mode aliases and row layout lists:\n{toml}"
    );
}

// ─── AI Section Tests ────────────────────────────────────────────────────────

#[test]
fn ai_claude_provider_volume_mount() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ai-claude");
    // Default is claude, so compose should already have .claude mount
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".claude"),
        "compose should mount .claude for claude provider"
    );
}

#[test]
fn ai_aider_provider_volume_mount() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ai-aider");
    replace_toml_section(
        dir.path(),
        "ai",
        r#"
harnesses = ["aider"]
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".aider"),
        "compose should mount .aider for aider provider"
    );
}

#[test]
fn ai_multiple_providers_volume_mounts() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "ai-multi");
    replace_toml_section(
        dir.path(),
        "ai",
        r#"
harnesses = ["claude", "aider", "gemini"]
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(compose.contains(".claude"), "compose should mount .claude");
    assert!(compose.contains(".aider"), "compose should mount .aider");
    assert!(compose.contains(".gemini"), "compose should mount .gemini");
}

// ─── Audio Section Tests ─────────────────────────────────────────────────────

#[test]
fn audio_enabled_adds_mounts() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "audio-on");
    replace_toml_section(
        dir.path(),
        "audio",
        r#"
enabled = true
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        compose.contains(".asoundrc"),
        "compose should mount .asoundrc when audio enabled"
    );
}

#[test]
fn audio_disabled_no_mounts() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "audio-off");
    replace_toml_section(
        dir.path(),
        "audio",
        r#"
enabled = false
"#,
    );
    sync_project(dir.path());
    let compose = read_generated(dir.path(), ".devcontainer/docker-compose.yml");
    assert!(
        !compose.contains(".asoundrc"),
        "compose should not mount .asoundrc when audio disabled"
    );
}

// ─── MCP Permissions Section Tests ──────────────────────────────────────────

#[test]
fn mcp_permissions_in_aibox_toml() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "mcp-perms");
    let toml = read_generated(dir.path(), "aibox.toml");
    assert!(
        toml.contains("[ai.mcp.permissions]"),
        "aibox.toml should contain [ai.mcp.permissions] block"
    );
}

// ─── Addon Section Tests ─────────────────────────────────────────────────────

#[test]
fn addon_python_in_dockerfile() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "addon-py");
    patch_toml(
        dir.path(),
        r#"
[addons.python.tools]
python = { version = "3.13" }
uv = { version = "0.7" }
"#,
    );
    sync_project(dir.path());
    let dockerfile = read_generated(dir.path(), ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("python") || dockerfile.contains("Python"),
        "Dockerfile should contain python addon commands"
    );
}

#[test]
fn addon_rust_in_dockerfile() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "addon-rs");
    patch_toml(
        dir.path(),
        r#"
[addons.rust.tools]
rustc = {}
"#,
    );
    sync_project(dir.path());
    let dockerfile = read_generated(dir.path(), ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("rustup") || dockerfile.contains("rust"),
        "Dockerfile should contain rust addon commands"
    );
}

#[test]
fn addon_multiple_in_dockerfile() {
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "addon-multi");
    patch_toml(
        dir.path(),
        r#"
[addons.python.tools]
python = { version = "3.13" }

[addons.node.tools]
node = { version = "22" }
"#,
    );
    sync_project(dir.path());
    let dockerfile = read_generated(dir.path(), ".devcontainer/Dockerfile");
    assert!(
        dockerfile.contains("python") || dockerfile.contains("Python"),
        "Dockerfile should contain python addon"
    );
    assert!(
        dockerfile.contains("node") || dockerfile.contains("Node"),
        "Dockerfile should contain node addon"
    );
}

// ─── processkit package selection tests ──────────────────────────────────────
//
// Since v0.16.0 aibox no longer scaffolds context-doc files (BACKLOG.md,
// DECISIONS.md, PRD.md, PROJECTS.md, …). Those are created lazily by the
// single-file processkit skills (backlog-context, decisions-adr, …) when
// an agent first uses them, OR by the entity-sharded processkit skills
// (workitem-management, decision-record, …) which write per-entity YAML
// files under context/workitems/ etc.
//
// What aibox owns at init time is the slim project skeleton: context/
// directory, aibox.lock, .gitignore, CLAUDE.md thin pointer, and
// (when [processkit] version != "unset") the processkit content
// installed by content_init. The tests below verify that contract for
// each of the five processkit package presets.

fn assert_post_init_skeleton(dir: &std::path::Path) {
    assert!(dir.join("context").exists(), "context/ should exist");
    assert!(
        dir.join("CLAUDE.md").exists(),
        "CLAUDE.md thin pointer should exist"
    );
    assert!(
        !dir.join(".aibox-version").exists(),
        ".aibox-version must NOT be created (absorbed into aibox.lock)"
    );
    assert!(dir.join(".gitignore").exists(), ".gitignore should exist");
    let claude = fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
    assert!(
        claude.contains("Pointer file") && claude.contains("AGENTS.md"),
        "CLAUDE.md should be a thin pointer to AGENTS.md, got: {claude}"
    );
}

#[test]
fn process_minimal_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "proc-min",
            "--base",
            "debian",
            "--context",
            "minimal",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
}

#[test]
fn process_managed_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "proc-mgd",
            "--base",
            "debian",
            "--context",
            "managed",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
    // Context-doc files are NOT created by aibox post-v0.16.0 — they
    // are owned by processkit single-file skills.
    assert!(
        !dir.path().join("context/BACKLOG.md").exists(),
        "aibox v0.16.0 must not scaffold context/BACKLOG.md (owned by processkit backlog-context skill)"
    );
}

#[test]
fn process_product_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "proc-prod",
            "--base",
            "debian",
            "--context",
            "product",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
    assert!(
        !dir.path().join("context/PRD.md").exists(),
        "aibox v0.16.0 must not scaffold context/PRD.md"
    );
}

#[test]
fn process_research_creates_skeleton() {
    let dir = tempfile::tempdir().unwrap();
    let output = run_in(
        dir.path(),
        &[
            "init",
            "proc-res",
            "--base",
            "debian",
            "--context",
            "research",
        ],
    );
    assert!(output.status.success());
    assert_post_init_skeleton(dir.path());
}

#[test]
fn sync_updates_processkit_install_hash_in_lock() {
    // After an apply that installs processkit content, aibox.lock should
    // contain a non-empty processkit_install_hash under [processkit]
    // (WS-7: renamed from mcp_config_hash and broadened to cover the
    // full processkit-shipped install payload).
    let dir = tempfile::tempdir().unwrap();
    init_project(dir.path(), "mcp-test");
    seed_processkit_install_fixture(dir.path());
    sync_project(dir.path());

    // Read the lock file and verify processkit_install_hash is set
    let lock_path = dir.path().join("aibox.lock");
    let lock_content = fs::read_to_string(&lock_path).expect("failed to read aibox.lock");
    assert!(
        lock_content.contains("processkit_install_hash"),
        "aibox.lock should contain processkit_install_hash field after apply"
    );
    // The hash should be a non-empty hex string
    assert!(
        lock_content.contains("processkit_install_hash = \""),
        "processkit_install_hash should have a value"
    );
}

// ─── H2 — Legacy powerline deprecation warning ───────────────────────────────
//
// When [customization.tmux.status] mode = "powerline" is in aibox.toml:
//   • `aibox apply` must exit 0 AND emit a LINT-POWERLINE-ALIAS deprecation
//     warning to stderr.
//   • `aibox doctor` must also surface a warning row containing the lint code.

/// Minimal aibox.toml with the legacy powerline mode alias.
fn powerline_alias_toml(name: &str) -> String {
    format!(
        r#"[aibox]
version = "0.25.5"
base = "debian"

[container]
name = "{name}"

[processkit]
version = "unset"

[customization.tmux.status]
mode = "powerline"
"#
    )
}

#[test]
fn legacy_powerline_mode_apply_warns_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let name = "h2-powerline-apply";
    fs::write(dir.path().join("aibox.toml"), powerline_alias_toml(name)).expect("write aibox.toml");

    let output = run_in(dir.path(), &["apply"]);
    assert!(
        output.status.success(),
        "apply with powerline alias should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("LINT-POWERLINE-ALIAS"),
        "apply should emit LINT-POWERLINE-ALIAS deprecation warning:\n{combined}"
    );
    assert!(
        combined.contains("powerline") && combined.contains("extended"),
        "apply deprecation message should explain the alias mapping:\n{combined}"
    );
}

#[test]
fn legacy_powerline_mode_doctor_warns_with_lint_code() {
    let dir = tempfile::tempdir().unwrap();
    let name = "h2-powerline-doctor";
    fs::write(dir.path().join("aibox.toml"), powerline_alias_toml(name)).expect("write aibox.toml");

    let output = run_in(dir.path(), &["doctor"]);
    assert!(
        output.status.success(),
        "doctor with powerline alias should exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("LINT-POWERLINE-ALIAS"),
        "doctor should report LINT-POWERLINE-ALIAS warning:\n{combined}"
    );
    // Doctor should also mention "warning(s)" in its summary.
    assert!(
        combined.contains("warning(s)") && !combined.contains("0 warning(s)"),
        "doctor summary should reflect at least one warning for the powerline alias:\n{combined}"
    );
}
