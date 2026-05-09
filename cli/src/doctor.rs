use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::{
    AiHarness, AiboxConfig, CONTAINER_WORKSPACE_DIR, McpGatewayMode, PROCESSKIT_VERSION_UNSET,
};
use crate::output;
use crate::processkit_vocab::{AGENTS_FILENAME, PROVENANCE_FILENAME};
use crate::runtime::{ContainerState, Runtime};

/// Embedded schema document for v1.0.0.
const SCHEMA_V1_0_0: &str = include_str!("../../schemas/v1.0.0/context-schema.md");
const CURRENT_CONTEXT_SCHEMA_VERSION: &str = "1.0.0";

/// Diagnostic counters.
#[derive(Debug)]
struct DiagResult {
    warnings: u32,
    errors: u32,
}

impl DiagResult {
    fn new() -> Self {
        Self {
            warnings: 0,
            errors: 0,
        }
    }
}

struct ProcesskitTemplateFiles {
    current: BTreeSet<String>,
    known: BTreeSet<String>,
}

/// Return the list of project-side files `aibox doctor` checks for.
///
/// Since v0.16.0 the bulk of context content (BACKLOG, DECISIONS, skills,
/// AGENTS.md, …) is owned by processkit and may or may not be present
/// depending on whether the user has run `aibox init` against a real
/// processkit version. Doctor only checks the slice that aibox itself
/// creates: the version marker, the gitignore, and the canonical
/// agent entrypoint installed by processkit.
fn expected_files(_packages: &[String]) -> Vec<&'static str> {
    vec![AGENTS_FILENAME, "aibox.lock", ".gitignore"]
}

/// Look up the embedded schema for a given version string.
fn schema_for_version(version: &str) -> Option<&'static str> {
    match version {
        "1.0.0" => Some(SCHEMA_V1_0_0),
        _ => None,
    }
}

/// Run full diagnostics.
pub fn cmd_doctor(config_path: &Option<String>) -> Result<()> {
    let mut diag = DiagResult::new();

    output::info("Running diagnostics...");

    // 1. Load and validate config
    let config = match AiboxConfig::from_cli_option(config_path) {
        Ok(c) => {
            output::ok(&format!(
                "Config: valid (v{}, {}, {:?})",
                c.aibox.version, c.aibox.base, c.context.packages
            ));
            Some(c)
        }
        Err(e) => {
            output::error(&format!("Config: {}", e));
            diag.errors += 1;
            output::info("Checking aibox.toml schema...");
            check_aibox_toml_schema(config_path, &mut diag);
            None
        }
    };

    // 2. Check container runtime (informational — not required for init/generate/doctor)
    match Runtime::detect() {
        Ok(rt) => output::ok(&format!("Container runtime: {} detected", rt.runtime_bin)),
        Err(_) => {
            output::warn(
                "No container runtime found (podman or docker needed for build/start/stop/attach)",
            );
            diag.warnings += 1;
        }
    }

    // If we couldn't load config, we can't do the remaining checks
    let config = match config {
        Some(c) => c,
        None => {
            print_summary(&diag);
            return Ok(());
        }
    };

    output::info("Checking aibox.toml schema...");
    check_aibox_toml_schema(config_path, &mut diag);

    // BR-TEST-GAPS H2 / LINT-POWERLINE-ALIAS: warn when the legacy
    // "powerline" mode alias appears in the raw TOML.
    check_legacy_powerline_mode_alias(config_path, &mut diag);

    // BR-LEGACY-MUX-EXCISE (DEC-20260508_1515-SilentAsh) hard-cut the legacy
    // multiplexer status alias in v0.25.6 — no doctor check is needed:
    // the schema validator above rejects unknown sections.

    // 3. Check .aibox-home/ directory (or legacy .root/)
    let root = config.host_root_dir();
    let root_label = root.display().to_string();
    if root.exists() {
        output::ok(&format!("{} directory exists", root_label));
        // Check expected subdirectories
        check_root_subdirs(&root, &root_label, &mut diag);

        // Check mount source paths match config (AI providers, audio)
        check_mount_sources(&root, &root_label, &config, &mut diag);

        // Check standard runtime theme/config files against the current
        // generated baseline. These files are user-editable, so drift is a
        // warning, but it is important signal after release upgrades.
        check_runtime_theme_template_drift(&config, &mut diag);

        // Suggest migration from .root/ to .aibox-home/
        if root_label == ".root" && !std::path::Path::new(".aibox-home").exists() {
            output::warn(
                ".root/ is the legacy name — consider renaming to .aibox-home/ \
                 (mv .root .aibox-home)",
            );
            diag.warnings += 1;
        }
    } else {
        output::warn(&format!(
            "{} directory not found -- run 'aibox init' or 'aibox up' to create it",
            root_label
        ));
        diag.warnings += 1;
    }

    // 4. Check .devcontainer/ files
    check_devcontainer_files(&mut diag);

    // 4b. lnav availability — surfaces the `Prefix L` log popup
    // (BR-LOG-PANEL, v0.25.6). Warn (not error) if missing because the
    // tmux binding falls back to `less`.
    output::info("Checking log viewer (lnav)...");
    check_lnav_installed(&mut diag);

    // 4c. BR-DOCTOR-GAPS: tmux.conf v0.25.3 corruption signature.
    output::info("Checking tmux.conf drift signature...");
    check_tmux_conf_drift_signature(&config, &mut diag);

    // 4d. BR-DOCTOR-GAPS: PowerKit plugin tree presence (extended mode only).
    output::info("Checking PowerKit plugin tree...");
    check_powerkit_plugin_tree(&config, &mut diag);

    // 4d-2. LINT-POWERKIT-STATUS-PLUGINS (BACK-20260508_1603-QuietCedar): warn if
    // any required plugin script referenced by the generated tmux.conf is
    // absent on disk.  The aibox-metrics path-a split registers six individual
    // segments (aibox_log … aibox_mig); if their .sh files are missing the
    // segments render blank.
    output::info("Checking PowerKit status plugin scripts...");
    check_powerkit_status_plugins(&config, &mut diag);

    // 4e. BR-LEGACY-MUX-EXCISE: legacy multiplexer artifact scan. Variant 1
    // hard-purge per DEC-20260508_1515-SilentAsh — `aibox apply` deletes
    // these unconditionally; survival is a doctor-level error.
    output::info("Checking for legacy multiplexer artifacts...");
    check_legacy_multiplexer_artifacts(&config, &mut diag);

    // 5. Check context structure
    output::info(&format!(
        "Checking context structure ({:?})...",
        config.context.packages
    ));
    check_context_structure(&config.context.packages, &mut diag);

    // 6. Check .gitignore
    output::info("Checking .gitignore...");
    let gitignore_warnings = crate::context::check_gitignore_entries();
    if gitignore_warnings.is_empty() {
        output::ok(".gitignore has all required entries");
    } else {
        for warning in &gitignore_warnings {
            output::warn(warning);
            diag.warnings += 1;
        }
    }

    // 6b. [skills].include / [skills].exclude validation (DEC-035)
    output::info("Validating [skills] overrides...");
    if let Ok(cwd) = std::env::current_dir() {
        match crate::content_init::validate_skill_overrides(&cwd, &config) {
            Ok(unknown) if unknown.is_empty() => {
                output::ok("[skills] overrides reference known skills");
            }
            Ok(unknown) => {
                for u in &unknown {
                    output::warn(u);
                    diag.warnings += 1;
                }
            }
            Err(e) => {
                output::warn(&format!("[skills] override validation failed: {}", e));
                diag.warnings += 1;
            }
        }
    }

    // 6c. Check live processkit-managed skills against the current template.
    let template_files = if let Ok(cwd) = std::env::current_dir() {
        output::info("Checking processkit template membership...");
        check_processkit_template_membership(&cwd, &config, &mut diag)
    } else {
        None
    };

    // 6d. Check command file registrations (BACK-20260423_2050-EagerStone)
    output::info("Checking command file registrations...");
    check_command_registrations(&config, template_files.as_ref(), &mut diag);

    // 6e. Check processkit MCP gateway selection.
    output::info("Checking processkit MCP gateway...");
    check_processkit_mcp_gateway(&config, &mut diag);
    check_claude_code_runtime_drift(&config, &mut diag);

    // 6f. Codex prompt-path drift check (BACK-20260426_1627-StrongHawk).
    // Loud failure if `pk-*` managed files reappear in the legacy
    // `~/.codex/prompts/` path that aibox v0.21.1 mistakenly used —
    // catches a regression in the codex profile of harness_commands.
    check_codex_prompt_path_drift(&config, &mut diag);

    // 6g. Codex sandbox prerequisites and compose posture.
    check_codex_sandbox_environment(&config, &mut diag);

    // 6h. Draft LivelyMoss addon metadata checks. Warning-only until
    // processkit publishes the canonical addon-spec schema.
    output::info("Checking addon profile metadata...");
    check_addon_profile_metadata(&config, &mut diag);

    // 6h. Draft LivelyMoss provider-backend checks. Warning-only until
    // processkit publishes the canonical provider-backend schema.
    output::info("Checking provider backend metadata...");
    check_provider_backend_metadata(&config, &mut diag);

    // 6i. Draft LivelyMoss image provenance policy checks. Warning-only until
    // processkit publishes the canonical image-provenance-policy schema.
    output::info("Checking image provenance policy...");
    check_image_provenance_policy(&config, &mut diag);

    // 7. Security audit tools
    crate::audit::doctor_check_audit_tools();

    // 8. Schema version check
    output::info("Schema version check");
    check_schema_version(&config, &mut diag)?;

    // 9. Container image version check (only if a runtime is available)
    if let Ok(runtime) = Runtime::detect() {
        check_container_image_version(&runtime, &config, &mut diag);
    }

    // 10. CLI version file check
    check_cli_version_file(&mut diag);

    // 11. Runtime resource pressure check (best-effort Linux procfs/cgroupfs)
    output::info("Checking runtime resource pressure...");
    check_runtime_resource_pressure(&config, &mut diag);

    // 12. BR-DOCTOR-GAPS: runtime startup hygiene (stale managed socket,
    // yazi terminal-response cache pollution). Warn-level.
    output::info("Checking runtime startup hygiene...");
    check_runtime_startup_hygiene(&config, &mut diag);

    print_summary(&diag);
    Ok(())
}

fn check_aibox_toml_schema(config_path: &Option<String>, diag: &mut DiagResult) {
    let path = config_path
        .as_deref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new("aibox.toml"));
    match std::fs::read_to_string(path) {
        Ok(body) => match AiboxConfig::schema_mismatches(&body) {
            Ok(mismatches) if mismatches.is_empty() => {
                output::ok("aibox.toml schema: no unknown keys");
            }
            Ok(mismatches) => {
                for mismatch in mismatches {
                    output::error(&format!(
                        "aibox.toml schema mismatch: {mismatch}. \
                         Ask the project agent to update aibox.toml for this aibox release."
                    ));
                    diag.errors += 1;
                }
            }
            Err(e) => {
                output::error(&format!("aibox.toml schema check failed: {}", e));
                diag.errors += 1;
            }
        },
        Err(e) => {
            output::warn(&format!(
                "aibox.toml schema check skipped; could not read {}: {}",
                path.display(),
                e
            ));
            diag.warnings += 1;
        }
    }
}

/// BR-TEST-GAPS H2 / LINT-POWERLINE-ALIAS: emit a warning when the raw
/// aibox.toml text uses the deprecated `mode = "powerline"` alias.
///
/// Since `powerline` is a `#[serde(alias = ...)]` for `Extended`, it parses
/// without error, but the canonical name is `extended`.  We surface it as a
/// doctor warning (not an error) so the project agent can update the file.
fn check_legacy_powerline_mode_alias(config_path: &Option<String>, diag: &mut DiagResult) {
    let path = config_path
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("aibox.toml"));
    let Ok(body) = std::fs::read_to_string(path) else {
        return; // file not readable — other checks will surface the problem
    };
    let uses_alias = body.lines().any(|line| {
        let t = line.trim_start();
        t.starts_with("mode") && t.contains("\"powerline\"")
    });
    if uses_alias {
        output::warn(
            "[LINT-POWERLINE-ALIAS] customization.tmux.status.mode = \"powerline\" is a \
             deprecated alias for \"extended\". Update aibox.toml to use mode = \"extended\" \
             to suppress this warning.",
        );
        diag.warnings += 1;
    } else {
        output::ok("tmux status mode: no deprecated alias");
    }
}

fn check_runtime_theme_template_drift(config: &AiboxConfig, diag: &mut DiagResult) {
    output::info("Checking runtime theme templates...");
    let root = config.host_root_dir();
    let mut drift = 0u32;

    for (rel_path, expected) in runtime_theme_reference_files(config) {
        let path = root.join(&rel_path);
        let rel = rel_path.to_string_lossy().replace('\\', "/");
        match std::fs::read_to_string(&path) {
            Ok(actual) if actual == expected => {}
            Ok(_) => {
                output::warn(&format!(
                    "Runtime theme/template drift: {} differs from the aibox {} reference. \
                     Ask the project agent to review local edits, then run `aibox apply` or \
                     the appropriate migration.",
                    rel,
                    env!("CARGO_PKG_VERSION")
                ));
                drift += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                output::warn(&format!(
                    "Runtime theme/template drift: {} is missing from {}. \
                     Run `aibox apply` to regenerate standard runtime theme files.",
                    rel,
                    root.display()
                ));
                drift += 1;
            }
            Err(e) => {
                output::warn(&format!(
                    "Runtime theme/template drift check skipped for {}: {}",
                    rel, e
                ));
                drift += 1;
            }
        }
    }

    if !lazygit_effective_enabled(config) {
        let stale_lazygit = root.join(".config").join("lazygit").join("config.yml");
        if stale_lazygit.exists() {
            output::warn(
                "Runtime theme/template drift: .config/lazygit/config.yml exists but \
                 git-ui.lazygit is disabled. Run `aibox apply` to remove stale lazygit \
                 runtime config.",
            );
            drift += 1;
        }
    }

    if drift == 0 {
        output::ok("Runtime theme files match the current aibox reference");
    } else {
        diag.warnings += drift;
    }
}

fn runtime_theme_reference_files(config: &AiboxConfig) -> Vec<(std::path::PathBuf, String)> {
    crate::seed::managed_runtime_files(config)
        .into_iter()
        .filter(|(path, _)| is_runtime_theme_reference_file(path))
        .collect()
}

fn is_runtime_theme_reference_file(path: &Path) -> bool {
    let rel = path.to_string_lossy().replace('\\', "/");
    rel == ".vim/vimrc"
        || rel == ".config/tmux/tmux.conf"
        || rel.starts_with(".config/tmux/layouts/")
        || rel == ".config/yazi/theme.toml"
        || rel == ".config/starship.toml"
        || rel == ".config/lazygit/config.yml"
        || rel == ".local/bin/aibox-status-toggle"
}

fn lazygit_effective_enabled(config: &AiboxConfig) -> bool {
    let Some(addon_section) = config.addons.get_addon("git-ui") else {
        return false;
    };

    if let Some(entry) = addon_section.tools.get("lazygit") {
        return entry.enabled.unwrap_or(true);
    }

    crate::addon_loader::get_addon("git-ui")
        .and_then(|addon| {
            addon
                .tools
                .iter()
                .find(|tool| tool.name == "lazygit")
                .map(|tool| tool.default_enabled)
        })
        .unwrap_or(true)
}

fn check_runtime_resource_pressure(config: &AiboxConfig, diag: &mut DiagResult) {
    if !is_running_inside_aibox_container() {
        output::ok(
            "Runtime resource pressure: skipped outside the aibox container \
             (run doctor inside the workspace container for cgroup/procfs counters)",
        );
        return;
    }

    let diagnostics = crate::runtime_resources::read_runtime_resource_diagnostics();
    let thresholds = &config.container.resource_thresholds;

    if let Some(memory_current) = diagnostics.memory_current_bytes {
        if let Some(limit_mib) = thresholds.memory_mib_warn {
            let limit_bytes = limit_mib.saturating_mul(1024 * 1024);
            if memory_current > limit_bytes {
                output::warn(&format!(
                    "Runtime memory usage is {} above configured warning threshold {}",
                    crate::runtime_resources::format_bytes(memory_current),
                    crate::runtime_resources::format_bytes(limit_bytes)
                ));
                diag.warnings += 1;
            } else {
                output::ok(&format!(
                    "Runtime memory usage: {}",
                    crate::runtime_resources::format_bytes(memory_current)
                ));
            }
        } else {
            output::ok(&format!(
                "Runtime memory usage: {}",
                crate::runtime_resources::format_bytes(memory_current)
            ));
        }
    } else {
        output::warn("Runtime memory usage unavailable (missing cgroup memory.current)");
        diag.warnings += 1;
    }

    if let Some(max) = diagnostics.memory_max {
        output::ok(&format!(
            "Runtime memory limit: {}",
            crate::runtime_resources::format_memory_max(max)
        ));
    }

    if let Some(oom_kill_count) = diagnostics.oom_kill_count {
        if thresholds
            .oom_kill_warn
            .is_some_and(|threshold| oom_kill_count > threshold)
        {
            output::warn(&format!(
                "Runtime cgroup reports {} OOM kill(s)",
                oom_kill_count
            ));
            diag.warnings += 1;
        } else {
            output::ok(&format!("Runtime OOM kills: {}", oom_kill_count));
        }
    } else {
        output::warn("Runtime OOM counter unavailable (missing cgroup memory.events)");
        diag.warnings += 1;
    }

    if let Some(threshold) = thresholds.process_count_warn
        && threshold > 0
        && diagnostics.total_process_count > threshold
    {
        output::warn(&format!(
            "Runtime process count is {} above warning threshold {}",
            diagnostics.total_process_count, threshold
        ));
        diag.warnings += 1;
    } else {
        output::ok(&format!(
            "Runtime process count: {}",
            diagnostics.total_process_count
        ));
    }

    if let Some(threshold) = thresholds.processkit_mcp_python_warn
        && threshold > 0
        && diagnostics.processkit_mcp_python_process_count > threshold
    {
        output::warn(&format!(
            "processkit MCP Python process count is {} above warning threshold {}",
            diagnostics.processkit_mcp_python_process_count, threshold
        ));
        diag.warnings += 1;
    } else {
        output::ok(&format!(
            "processkit MCP Python processes: {}",
            diagnostics.processkit_mcp_python_process_count
        ));
    }
}

/// Check that hot skills still belong to the current processkit template.
///
/// This catches renamed or removed upstream skills that survived in the live
/// `context/skills/` tree after a migration. Those stale directories used to
/// be treated as canonical by the command registration checker.
fn check_processkit_template_membership(
    project_root: &Path,
    config: &AiboxConfig,
    diag: &mut DiagResult,
) -> Option<ProcesskitTemplateFiles> {
    let template_files = match processkit_template_files(project_root, config) {
        Ok(Some(files)) => files,
        Ok(None) => {
            output::ok("No current processkit template provenance found");
            return None;
        }
        Err(e) => {
            output::warn(&format!(
                "Could not read current processkit template provenance: {e}"
            ));
            diag.warnings += 1;
            return None;
        }
    };

    let stale_skills =
        stale_processkit_skill_dirs(project_root, &template_files.current, &template_files.known);
    if stale_skills.is_empty() {
        output::ok("Installed processkit skills match the current template");
    } else {
        for skill_dir in &stale_skills {
            output::warn(&format!(
                "stale processkit skill: {} is absent from the current processkit template; remove the skill directory and rerun `aibox apply`",
                skill_dir.join("SKILL.md").display()
            ));
            diag.warnings += 1;
        }
    }

    Some(template_files)
}

fn processkit_template_files(
    project_root: &Path,
    config: &AiboxConfig,
) -> Result<Option<ProcesskitTemplateFiles>> {
    let Some(version) = current_processkit_version(project_root, config) else {
        return Ok(None);
    };
    let provenance_path = crate::content_init::templates_dir_for_version(project_root, &version)
        .join(PROVENANCE_FILENAME);
    if !provenance_path.is_file() {
        return Ok(None);
    }
    let current = processkit_template_files_from_provenance(&provenance_path)?;
    let mut known = known_processkit_template_files(project_root)?;
    known.extend(current.iter().cloned());
    Ok(Some(ProcesskitTemplateFiles { current, known }))
}

fn current_processkit_version(project_root: &Path, config: &AiboxConfig) -> Option<String> {
    if let Ok(Some(lock)) = crate::lock::read_lock(project_root)
        && let Some(processkit) = lock.processkit
        && !processkit.version.is_empty()
        && processkit.version != PROCESSKIT_VERSION_UNSET
    {
        return Some(processkit.version);
    }

    if config.processkit.version == PROCESSKIT_VERSION_UNSET {
        None
    } else {
        Some(config.processkit.version.clone())
    }
}

fn processkit_template_files_from_provenance(path: &Path) -> Result<BTreeSet<String>> {
    let body = std::fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&body)?;
    let files = value
        .get("files")
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default();
    Ok(files)
}

fn known_processkit_template_files(project_root: &Path) -> Result<BTreeSet<String>> {
    let templates_root = project_root.join(crate::processkit_vocab::TEMPLATES_PROCESSKIT_DIR);
    let mut known = BTreeSet::new();
    if !templates_root.is_dir() {
        return Ok(known);
    }

    for entry in std::fs::read_dir(templates_root)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        let provenance_path = entry.path().join(PROVENANCE_FILENAME);
        if provenance_path.is_file() {
            known.extend(processkit_template_files_from_provenance(&provenance_path)?);
        }
    }

    Ok(known)
}

fn stale_processkit_skill_dirs(
    project_root: &Path,
    current_template_files: &BTreeSet<String>,
    known_template_files: &BTreeSet<String>,
) -> Vec<PathBuf> {
    let skills_dir = project_root.join("context/skills");
    let mut stale = Vec::new();
    let Ok(categories) = std::fs::read_dir(&skills_dir) else {
        return stale;
    };

    for category in categories.flatten() {
        if !category.path().is_dir() {
            continue;
        }
        let Ok(skills) = std::fs::read_dir(category.path()) else {
            continue;
        };
        for skill in skills.flatten() {
            let skill_path = skill.path();
            if skill_path.is_dir()
                && skill_path.join("SKILL.md").is_file()
                && processkit_skill_is_known_stale(
                    project_root,
                    &skill_path,
                    current_template_files,
                    known_template_files,
                )
            {
                stale.push(skill_path);
            }
        }
    }
    stale.sort();
    stale
}

fn processkit_skill_is_known_stale(
    project_root: &Path,
    skill_path: &Path,
    current_template_files: &BTreeSet<String>,
    known_template_files: &BTreeSet<String>,
) -> bool {
    let Some(rel) = skill_md_relpath(project_root, skill_path) else {
        return false;
    };
    !current_template_files.contains(&rel)
        && (known_template_files.contains(&rel)
            || skill_has_processkit_metadata(project_root, &rel))
}

fn skill_md_relpath(project_root: &Path, skill_path: &Path) -> Option<String> {
    let skill_md = if skill_path.is_absolute() {
        skill_path.join("SKILL.md")
    } else {
        project_root.join(skill_path).join("SKILL.md")
    };
    skill_md
        .strip_prefix(project_root)
        .ok()
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
}

fn skill_has_processkit_metadata(project_root: &Path, rel_skill_md: &str) -> bool {
    crate::processkit_vocab::parse_skill_frontmatter(&project_root.join(rel_skill_md))
        .ok()
        .and_then(|frontmatter| frontmatter.metadata)
        .and_then(|metadata| metadata.processkit)
        .is_some()
}

fn check_command_registrations(
    config: &AiboxConfig,
    template_files: Option<&ProcesskitTemplateFiles>,
    diag: &mut DiagResult,
) {
    let skills_dir = std::path::Path::new("context/skills");
    if !skills_dir.is_dir() {
        output::ok("No context/skills/ found (expected in new projects)");
        return;
    }
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Gather every command basename from the live skills tree.
    let mut source_commands: Vec<(std::path::PathBuf, String)> = Vec::new();
    if let Ok(categories) = std::fs::read_dir(skills_dir) {
        for category in categories.flatten() {
            if !category.path().is_dir() {
                continue;
            }
            if let Ok(skills) = std::fs::read_dir(category.path()) {
                for skill in skills.flatten() {
                    let skill_path = skill.path();
                    if !skill_path.is_dir() {
                        continue;
                    }
                    if let Some(template_files) = template_files
                        && processkit_skill_is_known_stale(
                            &project_root,
                            &skill_path,
                            &template_files.current,
                            &template_files.known,
                        )
                    {
                        continue;
                    }
                    let commands_src = skill_path.join("commands");
                    if !commands_src.is_dir() {
                        continue;
                    }
                    if let Ok(cmds) = std::fs::read_dir(&commands_src) {
                        for cmd in cmds.flatten() {
                            let cmd_path = cmd.path();
                            if let Some(filename) = cmd_path
                                .file_name()
                                .and_then(|f| f.to_str())
                                .filter(|s| s.ends_with(".md"))
                            {
                                source_commands.push((commands_src.clone(), filename.to_string()));
                            }
                        }
                    }
                }
            }
        }
    }

    // Per-harness target dirs. `path_template` is a `{stem}` substitution
    // pattern relative to the project root. Mirrors the profiles in
    // `harness_commands::profile_for`. Keep this list in sync when
    // adding new scaffoldable harnesses.
    use crate::config::AiHarness;
    // (label, target_dir_for_message, path_template_with_{stem}, enabled)
    let mut targets: Vec<(&'static str, &'static str, &'static str, bool)> = Vec::new();
    targets.push((
        "claude",
        ".claude/skills",
        ".claude/skills/{stem}/SKILL.md",
        config.ai.harnesses.contains(&AiHarness::Claude),
    ));
    targets.push((
        "codex",
        ".agents/skills",
        ".agents/skills/{stem}/SKILL.md",
        config.ai.harnesses.contains(&AiHarness::Codex),
    ));
    targets.push((
        "cursor",
        ".cursor/commands",
        ".cursor/commands/{stem}.md",
        config.ai.harnesses.contains(&AiHarness::Cursor),
    ));
    targets.push((
        "gemini",
        ".gemini/commands",
        ".gemini/commands/{stem}.toml",
        config.ai.harnesses.contains(&AiHarness::Gemini),
    ));
    targets.push((
        "opencode",
        ".opencode/commands",
        ".opencode/commands/{stem}.md",
        config.ai.harnesses.contains(&AiHarness::OpenCode),
    ));

    for (harness, target_dir, path_template, enabled) in &targets {
        if !*enabled {
            continue;
        }
        let mut missing_count = 0;
        for (commands_src, filename) in &source_commands {
            let stem = match filename.strip_suffix(".md") {
                Some(s) => s,
                None => continue,
            };
            let deployed = std::path::PathBuf::from(path_template.replace("{stem}", stem));
            if !deployed.exists() {
                output::warn(&format!(
                    "{harness}: command file missing: {}/{} exists but {} is not registered",
                    commands_src.display(),
                    filename,
                    deployed.display()
                ));
                diag.warnings += 1;
                missing_count += 1;
            }
        }
        if missing_count == 0 {
            output::ok(&format!(
                "[{harness}] all installed skill commands are registered in {target_dir}/"
            ));
        } else {
            output::warn(&format!(
                "[{harness}] {missing_count} command file(s) missing — run 'aibox apply' to register them"
            ));
        }
    }
}

fn check_addon_profile_metadata(config: &AiboxConfig, diag: &mut DiagResult) {
    let all_addons = crate::addon_loader::all_addons();
    let selected_addons: Vec<String> = config.addons.addons.keys().cloned().collect();
    let mut warnings = crate::addon_loader::addon_metadata_warnings(all_addons);
    warnings.extend(crate::addon_loader::addon_profile_compatibility_warnings(
        all_addons,
        &selected_addons,
        config.aibox.profile.as_str(),
    ));
    if warnings.is_empty() {
        output::ok("Addon profile metadata is complete");
    } else {
        for warning in warnings {
            output::warn(&warning);
            diag.warnings += 1;
        }
    }
}

fn check_processkit_mcp_gateway(config: &AiboxConfig, diag: &mut DiagResult) {
    let gateway = &config.mcp.gateway;
    let gateway_script = Path::new("context/skills/processkit/processkit-gateway/mcp/server.py");
    let gateway_available = gateway_script.is_file();
    let aggregate_script =
        Path::new("context/skills/processkit/aggregate-mcp/mcp/mcp-config.aggregate.json");
    let aggregate_available = aggregate_script.is_file();

    match gateway.mode {
        McpGatewayMode::Granular => {
            output::ok("processkit MCP gateway disabled; granular MCP servers selected");
            return;
        }
        McpGatewayMode::Aggregate if !aggregate_available => {
            output::warn(
                "processkit aggregate MCP mode requested, but \
                 context/skills/processkit/aggregate-mcp/mcp/mcp-config.aggregate.json is missing; \
                 run `aibox apply` after upgrading processkit or enabling the aggregate-mcp skill",
            );
            diag.warnings += 1;
            return;
        }
        McpGatewayMode::Aggregate => {
            output::ok(
                "processkit aggregate MCP mode: single aggregate server replaces per-skill servers",
            );
        }
        McpGatewayMode::Auto if !gateway_available && !aggregate_available => {
            output::ok("processkit MCP gateway not installed; auto mode will use granular servers");
            return;
        }
        McpGatewayMode::Auto if !gateway_available && aggregate_available => {
            output::ok(
                "processkit gateway not installed; auto mode will use aggregate server \
                 (aggregate-mcp skill is available)",
            );
        }
        McpGatewayMode::Stdio | McpGatewayMode::DaemonProxy if !gateway_available => {
            output::warn(
                "processkit MCP gateway mode requested, but \
                 context/skills/processkit/processkit-gateway/mcp/server.py is missing; \
                 run `aibox apply` after upgrading processkit",
            );
            diag.warnings += 1;
            return;
        }
        _ => {}
    }

    if gateway_available {
        output::ok("processkit MCP gateway is installed");
        check_processkit_semantic_capability(diag);
    }

    if matches!(gateway.mode, McpGatewayMode::DaemonProxy) {
        let devcontainer = Path::new(".devcontainer/devcontainer.json");
        match std::fs::read_to_string(devcontainer) {
            Ok(body) if body.contains("processkit-gateway/mcp/server.py") => {
                output::ok("processkit gateway daemon startup is present in devcontainer.json");
            }
            Ok(_) => {
                output::warn(
                    "[mcp.gateway] selects the processkit gateway daemon, but devcontainer.json \
                     does not start processkit-gateway; run `aibox apply`",
                );
                diag.warnings += 1;
            }
            Err(_) => {
                output::warn(
                    "[mcp.gateway] selects the processkit gateway daemon, but .devcontainer/devcontainer.json \
                     is missing; run `aibox apply`",
                );
                diag.warnings += 1;
            }
        }
    } else if matches!(gateway.mode, McpGatewayMode::Auto) {
        if gateway_available {
            output::ok(
                "processkit gateway auto mode uses stdio-proxy-owned daemon startup \
                 (processkit v0.25.4+)",
            );
        }
    }

    if config.ai.harnesses.contains(&AiHarness::Codex) {
        match std::fs::read_to_string(".codex/config.toml") {
            Ok(body)
                if !body.contains("[mcp_servers.processkit-gateway]")
                    && !body.contains("[mcp_servers.processkit-aggregate-mcp]") =>
            {
                output::warn(
                    "Codex is enabled but .codex/config.toml does not register \
                     processkit-gateway or processkit-aggregate-mcp; run `aibox apply`",
                );
                diag.warnings += 1;
            }
            Ok(body) if codex_config_has_non_container_processkit_script_path(&body) => {
                output::warn(&format!(
                    "Codex MCP config points at a host-side processkit script path; \
                     run `aibox apply` with aibox 0.23.7+ so Codex uses \
                     {}/context/skills/... inside the container",
                    CONTAINER_WORKSPACE_DIR
                ));
                diag.warnings += 1;
            }
            Ok(body) if body.contains("[mcp_servers.processkit-aggregate-mcp]") => {
                output::ok(
                    "Codex MCP config uses processkit-aggregate-mcp (single-process, \
                     reduced startup latency)",
                );
                check_codex_hidden_apps_mcp_state(diag);
            }
            Ok(_) => {
                output::ok("Codex MCP config points at processkit-gateway");
                check_codex_hidden_apps_mcp_state(diag);
            }
            Err(_) => {
                output::warn(
                    "Codex is enabled but .codex/config.toml is missing; run `aibox apply`",
                );
                diag.warnings += 1;
            }
        }
    }
}

fn check_codex_hidden_apps_mcp_state(diag: &mut DiagResult) {
    let codex_home = codex_home_dir();
    let apps_cache = codex_home.join("cache").join("codex_apps_tools");
    if !dir_has_entries(&apps_cache) {
        return;
    }

    output::warn(
        "Codex hidden app-tool cache detected at cache/codex_apps_tools. \
         Some Codex versions eagerly start a hidden `codex_apps` MCP server for subagents; \
         if subagents hang on MCP startup, avoid delegation or clear that Codex app cache \
         until the upstream Codex behavior is fixed.",
    );
    diag.warnings += 1;
}

fn check_claude_code_runtime_drift(config: &AiboxConfig, diag: &mut DiagResult) {
    if !config.ai.harnesses.contains(&AiHarness::Claude) {
        return;
    }

    output::info("Checking Claude Code runtime drift...");

    let dot_mcp = Path::new(".mcp.json");
    if !dot_mcp.is_file() {
        output::warn("claude: .mcp.json is missing; run `aibox apply` to register MCP servers");
        diag.warnings += 1;
    } else if claude_dot_mcp_uses_only_processkit_gateway(dot_mcp) {
        check_claude_settings_for_stale_processkit_servers(
            Path::new(".claude/settings.json"),
            "claude settings",
            diag,
        );
        check_claude_settings_for_stale_processkit_servers(
            Path::new(".claude/settings.local.json"),
            "claude local settings",
            diag,
        );
    }

    let root = config.host_root_dir();
    let home_claude = root.join(".local").join("bin").join("claude");
    if home_claude.symlink_metadata().is_err() {
        output::warn(&format!(
            "claude: {}/.local/bin/claude is missing; run `aibox apply` to seed the stable /usr/local/bin/claude shim",
            root.display()
        ));
        diag.warnings += 1;
    } else if stale_claude_home_installer_symlink(&home_claude) {
        output::warn(&format!(
            "claude: {}/.local/bin/claude points at a stale native home install; run `aibox apply` to replace it with the stable /usr/local/bin/claude shim",
            root.display()
        ));
        diag.warnings += 1;
    } else {
        output::ok("claude: home-bin shim is present");
    }

    let claude_state = root.join(".claude.json");
    if let Ok(body) = std::fs::read_to_string(&claude_state)
        && body.contains("\"installMethod\"")
        && body.contains("\"native\"")
        && home_claude.symlink_metadata().is_err()
    {
        output::warn(&format!(
            "claude: {} still records native install metadata but the mounted home-bin shim is missing; run `aibox apply`",
            claude_state.display()
        ));
        diag.warnings += 1;
    }
}

fn claude_dot_mcp_uses_only_processkit_gateway(path: &Path) -> bool {
    let Ok(body) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) else {
        return false;
    };
    let Some(servers) = parsed.get("mcpServers").and_then(|value| value.as_object()) else {
        return false;
    };
    servers.contains_key("processkit-gateway")
        && !servers
            .keys()
            .any(|name| name.starts_with("processkit-") && name != "processkit-gateway")
}

fn check_claude_settings_for_stale_processkit_servers(
    path: &Path,
    label: &str,
    diag: &mut DiagResult,
) {
    let Ok(body) = std::fs::read_to_string(path) else {
        return;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) else {
        output::warn(&format!(
            "claude: could not parse {}; run `aibox apply` after repairing the JSON",
            path.display()
        ));
        diag.warnings += 1;
        return;
    };

    let stale_servers: Vec<String> = json_string_array(&parsed, &["enabledMcpjsonServers"])
        .into_iter()
        .filter(|entry| entry.starts_with("processkit-") && entry != "processkit-gateway")
        .collect();
    let stale_permissions: Vec<String> = json_string_array(&parsed, &["permissions", "allow"])
        .into_iter()
        .filter(|entry| {
            entry.starts_with("mcp__processkit-") && entry != "mcp__processkit-gateway__*"
        })
        .collect();

    if stale_servers.is_empty() && stale_permissions.is_empty() {
        output::ok(&format!(
            "claude: {label} match processkit-gateway MCP topology"
        ));
        return;
    }

    let stale_count = stale_servers.len() + stale_permissions.len();
    output::warn(&format!(
        "claude: {} contains {stale_count} stale granular processkit MCP entr{} while .mcp.json is collapsed to processkit-gateway; run `aibox apply` to reconcile Claude MCP auth state",
        path.display(),
        if stale_count == 1 { "y" } else { "ies" }
    ));
    diag.warnings += 1;
}

fn json_string_array(value: &serde_json::Value, path: &[&str]) -> Vec<String> {
    let mut current = value;
    for key in path {
        let Some(next) = current.get(*key) else {
            return Vec::new();
        };
        current = next;
    }
    current
        .as_array()
        .map(|array| {
            array
                .iter()
                .filter_map(|value| value.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn stale_claude_home_installer_symlink(path: &Path) -> bool {
    let Ok(target) = std::fs::read_link(path) else {
        return false;
    };
    target
        .to_string_lossy()
        .contains(".local/share/claude/versions/")
}

fn codex_home_dir() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".codex")))
        .unwrap_or_else(|| PathBuf::from("/home/aibox/.codex"))
}

fn dir_has_entries(path: &Path) -> bool {
    std::fs::read_dir(path)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

fn codex_config_has_non_container_processkit_script_path(body: &str) -> bool {
    let Ok(parsed) = body.parse::<toml::Value>() else {
        return body.lines().any(|line| {
            line.contains("/context/skills/processkit/") && !line.contains(CONTAINER_WORKSPACE_DIR)
        });
    };
    let Some(servers) = parsed.get("mcp_servers").and_then(toml::Value::as_table) else {
        return false;
    };
    servers.values().any(|server| {
        server
            .get("args")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .any(is_non_container_processkit_script_path)
    })
}

fn is_non_container_processkit_script_path(arg: &str) -> bool {
    arg.ends_with(".py")
        && arg.contains("/context/skills/processkit/")
        && Path::new(arg).is_absolute()
        && !arg.starts_with(&format!("{}/", CONTAINER_WORKSPACE_DIR))
}

fn check_processkit_semantic_capability(diag: &mut DiagResult) {
    let scripts = [
        Path::new("context/skills/processkit/index-management/mcp/server.py"),
        Path::new("context/skills/processkit/processkit-gateway/mcp/server.py"),
    ];
    let declares_sqlite_vec = scripts.iter().any(|path| {
        std::fs::read_to_string(path)
            .map(|body| body.contains("sqlite-vec"))
            .unwrap_or(false)
    });
    if !declares_sqlite_vec {
        return;
    }

    let uv = find_command_on_path(&["uv"]);
    let Some(uv) = uv else {
        output::warn(
            "processkit semantic search: installed MCP scripts declare sqlite-vec, but `uv` is not available to resolve PEP 723 dependencies",
        );
        diag.warnings += 1;
        return;
    };

    let status = Command::new(uv)
        .env("UV_CACHE_DIR", "/tmp/aibox/uv-cache")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "run",
            "--offline",
            "--no-project",
            "--with",
            "sqlite-vec>=0.1.0",
            "python",
            "-c",
            "import sqlite3, sqlite_vec; db=sqlite3.connect(':memory:'); db.enable_load_extension(True); sqlite_vec.load(db); print('ok')",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            output::ok("processkit semantic search: sqlite-vec is available to uv");
        }
        Ok(_) => {
            output::warn(
                "processkit semantic search degraded: sqlite-vec is declared but not available in the current uv cache; MCP servers will fall back to FTS until uv can install sqlite-vec",
            );
            diag.warnings += 1;
        }
        Err(err) => {
            output::warn(&format!(
                "processkit semantic search: sqlite-vec probe could not run: {}",
                err
            ));
            diag.warnings += 1;
        }
    }
}

fn check_provider_backend_metadata(config: &AiboxConfig, diag: &mut DiagResult) {
    let warnings = crate::provider_backend::provider_backend_warnings(
        config,
        crate::addon_loader::all_addons(),
    );
    if warnings.is_empty() {
        output::ok("Provider backend metadata is compatible");
    } else {
        for warning in warnings {
            output::warn(&warning);
            diag.warnings += 1;
        }
    }
}

fn check_image_provenance_policy(config: &AiboxConfig, diag: &mut DiagResult) {
    let project_root = std::path::Path::new(".");
    let warnings = crate::image_provenance::image_provenance_warnings(config, project_root);
    if warnings.is_empty() {
        output::ok("Image provenance policy is compatible");
    } else {
        for warning in warnings {
            output::warn(&warning);
            diag.warnings += 1;
        }
    }
}

/// Detect drift on the Codex slash-command path. Codex CLI 0.125.0 surfaces
/// custom workflows as Skills under `<workspace>/.agents/skills/<name>/SKILL.md`
/// — NOT from `~/.codex/prompts/` (the legacy aibox v0.21.1 location). If
/// any managed `pk-*.md` file reappears in the legacy path, treat that as
/// a regression error: aibox is again writing to the wrong place. Also
/// errors if Codex is enabled but no skills landed under `.agents/skills/`.
///
/// See DEC-20260426_1636-MightySky and BACK-20260426_1627-StrongHawk.
fn check_codex_prompt_path_drift(config: &AiboxConfig, diag: &mut DiagResult) {
    let codex_enabled = config.ai.harnesses.contains(&AiHarness::Codex);

    let legacy_dir = std::path::Path::new(".aibox-home/.codex/prompts");
    if let Ok(entries) = std::fs::read_dir(legacy_dir) {
        let stale: Vec<String> = entries
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| n.starts_with("pk-") && n.ends_with(".md"))
            .collect();
        if !stale.is_empty() {
            output::error(&format!(
                "codex: stale managed prompt(s) in legacy path .aibox-home/.codex/prompts/: \
                 {}. Codex 0.125.0 ignores this directory; commands must be Codex Skills \
                 under .agents/skills/<name>/SKILL.md (DEC-20260426_1636-MightySky). \
                 Run 'aibox apply' to migrate.",
                stale.join(", ")
            ));
            diag.errors += 1;
        }
    }

    if codex_enabled {
        let skills_dir = std::path::Path::new(".agents/skills");
        let has_pk_skill = std::fs::read_dir(skills_dir)
            .map(|it| {
                it.flatten().any(|e| {
                    e.file_name()
                        .to_str()
                        .map(|n| n.starts_with("pk-"))
                        .unwrap_or(false)
                        && e.path().join("SKILL.md").is_file()
                })
            })
            .unwrap_or(false);
        if !has_pk_skill {
            output::warn(
                "codex: no pk-* Codex Skills found under .agents/skills/ — \
                 run 'aibox apply' to scaffold them (Codex 0.125.0 surfaces \
                 these as $skill-name mentions and via /skills)",
            );
            diag.warnings += 1;
        } else {
            output::ok("codex: pk-* Codex Skills present under .agents/skills/");
        }
    }
}

fn check_codex_sandbox_environment(config: &AiboxConfig, diag: &mut DiagResult) {
    if !config.ai.harnesses.contains(&AiHarness::Codex) {
        return;
    }

    output::info("Checking Codex sandbox environment...");

    if !is_running_inside_aibox_container() {
        output::ok(
            "codex: sandbox probe skipped outside the aibox container \
             (bubblewrap is validated inside the generated workspace runtime)",
        );
        return;
    }

    let bwrap = find_command_on_path(&["bwrap", "bubblewrap"]);
    let Some(bwrap) = bwrap else {
        output::warn(
            "codex: bubblewrap/bwrap not found on PATH. Codex sandboxed shell commands may fail; run `aibox apply` and rebuild the container.",
        );
        diag.warnings += 1;
        return;
    };

    output::ok(&format!(
        "codex: bubblewrap helper available as `{}`",
        bwrap
    ));

    if pid1_is_sleep_infinity() {
        output::warn(
            "codex: current container PID 1 is `sleep infinity`; skipping active bubblewrap namespace probe. Run `aibox apply` and recreate the container so Compose init can reap sandbox helpers, then rerun doctor.",
        );
        diag.warnings += 1;
    } else {
        match run_bwrap_smoke_probe(&bwrap) {
            Ok(true) => output::ok("codex: bubblewrap user-namespace smoke probe succeeded"),
            Ok(false) => {
                output::warn(
                    "codex: bubblewrap user-namespace smoke probe failed. Ordinary sandboxed file reads may require escalation until the container runtime allows unprivileged user namespaces and seccomp=unconfined is active.",
                );
                diag.warnings += 1;
            }
            Err(err) => {
                output::warn(&format!(
                    "codex: bubblewrap user-namespace smoke probe could not run: {}",
                    err
                ));
                diag.warnings += 1;
            }
        }
    }

    for path in [
        crate::config::COMPOSE_FILE,
        ".devcontainer/docker-compose.override.yml",
    ] {
        match read_codex_compose_posture(Path::new(path), &config.container.name) {
            Ok(Some(posture)) => {
                if path == crate::config::COMPOSE_FILE && !posture.init_true {
                    output::warn(
                        "codex: generated compose service is missing init=true; run `aibox apply` and recreate the container so sandbox helper zombies are reaped",
                    );
                    diag.warnings += 1;
                }
                for warning in posture.broad_grant_warnings {
                    output::warn(&warning);
                    diag.warnings += 1;
                }
                if posture.seccomp_unconfined {
                    output::ok(&format!(
                        "codex: {} uses seccomp=unconfined as a project-local sandbox fallback",
                        path
                    ));
                    // S5 — BR-SEC-HARDEN: warn when seccomp=unconfined is active
                    // without the explicit consent flag.
                    if !config.security.acknowledge_seccomp_unconfined {
                        output::warn(
                            "codex: seccomp=unconfined is in use but \
                             `[security].acknowledge_seccomp_unconfined` is not set to `true` \
                             in aibox.toml. Add `[security]\\nacknowledge_seccomp_unconfined = true` \
                             to suppress this warning and confirm intentional opt-in.",
                        );
                        diag.warnings += 1;
                    }
                }
            }
            Ok(None) => {}
            Err(err) => {
                output::warn(&format!(
                    "codex: could not inspect compose sandbox posture in {}: {}",
                    path, err
                ));
                diag.warnings += 1;
            }
        }
    }
}

fn find_command_on_path(candidates: &[&str]) -> Option<String> {
    candidates
        .iter()
        .find(|candidate| {
            Command::new("sh")
                .args(["-c", &format!("command -v {} >/dev/null 2>&1", candidate)])
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        })
        .map(|candidate| (*candidate).to_string())
}

/// BR-LOG-PANEL (v0.25.6): warn if `lnav` is not on PATH. The `Prefix L`
/// tmux binding falls back to `less` if lnav is missing, so this is a
/// warning rather than an error.
fn check_lnav_installed(diag: &mut DiagResult) {
    if find_command_on_path(&["lnav"]).is_some() {
        output::ok("lnav: installed (Prefix L opens the structured log popup)");
    } else {
        output::warn(
            "lnav not found on PATH — Prefix L log popup will fall back to less. \
             Rebuild the container image to install it (added to base-debian in v0.25.6).",
        );
        diag.warnings += 1;
    }
}

// ---------------------------------------------------------------------------
// BR-DOCTOR-GAPS (v0.25.6, DEC-20260508_1515-SilentAsh)
// Six coverage gaps: drift signature, legacy aliases, PowerKit plugin
// tree, lockfile-vs-CLI skew (replaces the old check), startup hygiene,
// and legacy multiplexer artifacts.
// ---------------------------------------------------------------------------

/// Loud (error-level) detection of the v0.25.3 substitution-order
/// corruption in the live `tmux.conf`. Reuses the runtime_sync
/// recognizer added by BR-CLEANUP-ARCH item 2 so the heuristic stays in
/// one place.
fn check_tmux_conf_drift_signature(config: &AiboxConfig, diag: &mut DiagResult) {
    let path = config.host_root_dir().join(".config/tmux/tmux.conf");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    if crate::runtime_sync::live_is_corrupted_v0_25_3_tmux_conf(&content) {
        output::error(&format!(
            "Corrupted v0.25.3 tmux.conf detected at {} (status off + off_RIGHT signature). \
             Run `aibox apply` from a v0.25.6+ host CLI to overwrite with current generated content.",
            path.display(),
        ));
        diag.errors += 1;
    } else {
        output::ok("tmux.conf: no v0.25.3 corruption signature");
    }
}

/// When extended/PowerKit status mode is selected, ensure the PowerKit
/// plugin tree exists either in the host's managed plugin dir or baked
/// into the image. Missing plugin tree means the status row silently
/// fails to render — exactly the symptom DEC-1515 was reacting to.
fn check_powerkit_plugin_tree(config: &AiboxConfig, diag: &mut DiagResult) {
    use crate::config::TmuxStatusMode;
    if !matches!(
        config.customization.tmux.status.mode,
        TmuxStatusMode::Extended
    ) {
        return;
    }
    let host = config
        .host_root_dir()
        .join(".tmux/plugins/tmux-powerkit/tmux-powerkit.tmux");
    let in_image =
        Path::new("/usr/local/share/aibox/tmux/plugins/tmux-powerkit/tmux-powerkit.tmux");
    if host.exists() || in_image.exists() {
        output::ok("PowerKit plugin tree present");
    } else {
        output::error(&format!(
            "PowerKit plugin tree missing: neither {} nor {} exist. \
             Rebuild the container image and run `aibox apply`.",
            host.display(),
            in_image.display(),
        ));
        diag.errors += 1;
    }
}

/// LINT-POWERKIT-STATUS-PLUGINS (BACK-20260508_1603-QuietCedar, v0.25.6)
///
/// Warn when any plugin script listed in the generated tmux.conf is absent
/// from the PowerKit plugin tree.  Extended mode only.
///
/// The aibox-metrics block uses path-a split: each metric (`log`, `oom`,
/// `proc`, `ai`, `mcp`, `mig`) is registered as its own plugin
/// (`aibox_log` … `aibox_mig`) so it renders with chevron/color-rotation
/// segment styling.  If the individual `.sh` files are not installed the
/// segments render blank or cause PowerKit bootstrap errors.
///
/// Required plugin scripts (relative to `tmux-powerkit/src/plugins/`):
///   hostname.sh, external_ip.sh, ssh.sh, uptime.sh, weather.sh, datetime.sh,
///   git.sh, github.sh, kubernetes.sh, terraform.sh, cloud.sh, cloudstatus.sh,
///   cpu.sh, loadavg.sh, memory.sh, swap.sh, disk.sh, gpu.sh, netspeed.sh,
///   ping.sh, aibox_log.sh, aibox_oom.sh, aibox_proc.sh, aibox_ai.sh,
///   aibox_mcp.sh, aibox_mig.sh
fn check_powerkit_status_plugins(config: &AiboxConfig, diag: &mut DiagResult) {
    use crate::config::TmuxStatusMode;
    if !matches!(
        config.customization.tmux.status.mode,
        TmuxStatusMode::Extended
    ) {
        return;
    }

    // Resolve the PowerKit plugin scripts directory: prefer the host-root
    // installation, fall back to the in-image path.
    let host_plugins_dir = config
        .host_root_dir()
        .join(".tmux/plugins/tmux-powerkit/src/plugins");
    let image_plugins_dir =
        std::path::PathBuf::from("/usr/local/share/aibox/tmux/plugins/tmux-powerkit/src/plugins");
    let plugins_dir = if host_plugins_dir.exists() {
        host_plugins_dir
    } else {
        image_plugins_dir
    };

    if !plugins_dir.exists() {
        // PowerKit tree itself is missing — check_powerkit_plugin_tree() already
        // reports this; do not double-count.
        return;
    }

    // All plugin script names required by the fixed slot order.
    // Slot order is fixed per DEC-20260508_2115-SilentFern.
    let required: &[&str] = &[
        // Line 1 right
        "hostname",
        "external_ip",
        "ssh",
        "uptime",
        "weather",
        "datetime",
        // Line 2 left
        "git",
        "github",
        "kubernetes",
        "terraform",
        "cloud",
        "cloudstatus",
        // Line 2 right — system
        "cpu",
        "loadavg",
        "memory",
        "swap",
        "disk",
        "gpu",
        "netspeed",
        "ping",
        // Line 2 right — aibox-metrics block (path-a split, per-metric segments)
        "aibox_log",
        "aibox_oom",
        "aibox_proc",
        "aibox_ai",
        "aibox_mcp",
        "aibox_mig",
    ];

    let mut missing = Vec::new();
    for name in required {
        let script = plugins_dir.join(format!("{name}.sh"));
        if !script.exists() {
            missing.push(*name);
        }
    }

    if missing.is_empty() {
        output::ok("PowerKit status plugin scripts: all required scripts present");
    } else {
        for name in &missing {
            output::warn(&format!(
                "[LINT-POWERKIT-STATUS-PLUGINS] Required PowerKit plugin script \
                 missing: {name}.sh (expected under {}). \
                 Rebuild the container image to install it; the segment will \
                 render blank until then.",
                plugins_dir.display()
            ));
            diag.warnings += 1;
        }
    }
}

/// Scan for legacy multiplexer artifacts under the host root. Variant 1
/// hard-purge per DEC-20260508_1515-SilentAsh — owner approved; presence
/// is an error. `aibox apply` runs the legacy-multiplexer cleanup
/// unconditionally, so survival here means either the host CLI is
/// pre-v0.25.6 or the user re-introduced the files manually.
fn check_legacy_multiplexer_artifacts(config: &AiboxConfig, diag: &mut DiagResult) {
    let root = config.host_root_dir();
    let found = crate::seed::surviving_legacy_multiplexer_paths(&root);
    if found.is_empty() {
        output::ok("No legacy multiplexer artifacts under host root");
        return;
    }
    let list = found
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    output::error(&format!(
        "Legacy multiplexer artifacts present: {list}. Run `aibox apply` from \
         a v0.25.6+ host CLI; if the warning persists, remove the listed \
         paths manually."
    ));
    diag.errors += 1;
}

/// Lightweight startup-hygiene check: a non-socket file at the managed
/// tmux socket path blocks `aibox up`; yazi terminal-response cache
/// pollution is a known yazi-bug source. Both are warning-level — they
/// don't break doctor as a whole but tell the user where to look.
fn check_runtime_startup_hygiene(config: &AiboxConfig, diag: &mut DiagResult) {
    let root = config.host_root_dir();
    let sock = root.join(".tmux/aibox.sock");
    if let Ok(meta) = std::fs::symlink_metadata(&sock) {
        use std::os::unix::fs::FileTypeExt;
        if !meta.file_type().is_socket() {
            output::warn(&format!(
                "Stale tmux runtime socket at {} (not a UNIX socket). \
                 Remove the file or run `aibox down && aibox up`.",
                sock.display(),
            ));
            diag.warnings += 1;
        }
    }
    let yazi_cache = root.join(".cache/yazi");
    if yazi_cache.exists()
        && let Ok(entries) = std::fs::read_dir(&yazi_cache)
        && entries.filter_map(Result::ok).any(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".terminal-response.log")
        })
    {
        output::warn(&format!(
            "Yazi terminal-response cache pollution under {}. \
             Remove the directory and run `aibox apply`.",
            yazi_cache.display(),
        ));
        diag.warnings += 1;
    }
}

fn run_bwrap_smoke_probe(binary: &str) -> Result<bool> {
    let status = Command::new(binary)
        .args(bwrap_smoke_probe_args())
        .status()?;
    Ok(status.success())
}

fn bwrap_smoke_probe_args() -> [&'static str; 13] {
    [
        "--unshare-user",
        "--uid",
        "0",
        "--gid",
        "0",
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "/bin/true",
    ]
}

fn pid1_is_sleep_infinity() -> bool {
    let comm = std::fs::read_to_string("/proc/1/comm")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if comm != "sleep" {
        return false;
    }

    std::fs::read("/proc/1/cmdline")
        .map(|bytes| {
            let cmdline = bytes
                .split(|byte| *byte == b'\0')
                .filter(|part| !part.is_empty())
                .map(|part| String::from_utf8_lossy(part).to_string())
                .collect::<Vec<_>>();
            cmdline.len() == 2 && cmdline[0] == "sleep" && cmdline[1] == "infinity"
        })
        .unwrap_or(false)
}

#[derive(Debug, Default, PartialEq, Eq)]
struct CodexComposePosture {
    broad_grant_warnings: Vec<String>,
    init_true: bool,
    seccomp_unconfined: bool,
}

fn read_codex_compose_posture(
    path: &Path,
    service_name: &str,
) -> Result<Option<CodexComposePosture>> {
    if !path.exists() {
        return Ok(None);
    }
    let body = std::fs::read_to_string(path)?;
    let parsed: serde_yaml::Value = serde_yaml::from_str(&body)?;
    Ok(Some(codex_compose_posture(
        &parsed,
        service_name,
        &path.display().to_string(),
    )))
}

fn codex_compose_posture(
    compose: &serde_yaml::Value,
    service_name: &str,
    source_label: &str,
) -> CodexComposePosture {
    let mut posture = CodexComposePosture::default();
    let Some(services) = compose.get("services").and_then(|value| value.as_mapping()) else {
        return posture;
    };

    let mut candidates = std::collections::BTreeSet::new();
    candidates.insert(service_name);
    candidates.insert("aibox");

    for candidate in candidates {
        let Some(service) = services.get(serde_yaml::Value::String(candidate.to_string())) else {
            continue;
        };

        if service
            .get("init")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            posture.init_true = true;
        }

        if service
            .get("privileged")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            posture.broad_grant_warnings.push(format!(
                "codex: {} service `{}` sets privileged=true; generated aibox containers should not require privileged mode for bubblewrap",
                source_label, candidate
            ));
        }

        if yaml_sequence_contains_ci(service.get("cap_add"), "SYS_ADMIN") {
            posture.broad_grant_warnings.push(format!(
                "codex: {} service `{}` adds SYS_ADMIN; generated aibox containers should not require SYS_ADMIN for Codex bubblewrap",
                source_label, candidate
            ));
        }

        if yaml_sequence_contains_ci(service.get("security_opt"), "seccomp=unconfined") {
            posture.seccomp_unconfined = true;
        }
    }

    posture
}

fn yaml_sequence_contains_ci(value: Option<&serde_yaml::Value>, needle: &str) -> bool {
    value
        .and_then(|value| value.as_sequence())
        .map(|items| {
            items.iter().any(|item| {
                item.as_str()
                    .map(|item| item.eq_ignore_ascii_case(needle))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Check that mount source directories exist for configured features.
fn check_mount_sources(root: &Path, root_label: &str, config: &AiboxConfig, diag: &mut DiagResult) {
    // AI providers — check the .aibox-home/<provider>/ persistence dir
    // for the in-container CLI tools that have one. Cursor is the only
    // provider with no container CLI binary (host-side IDE extension only).
    for provider in &config.ai.harnesses {
        let Some(dir_name) = provider.config_dir() else {
            continue;
        };
        let path = root.join(dir_name);
        if path.exists() {
            output::ok(&format!(
                "{}/{} exists ({})",
                root_label, dir_name, provider
            ));
        } else {
            output::warn(&format!(
                "{}/{} missing — run 'aibox apply' to create it",
                root_label, dir_name
            ));
            diag.warnings += 1;
        }
    }

    // Audio
    if config.audio.enabled {
        let asoundrc = root.join(".asoundrc");
        if asoundrc.exists() {
            output::ok(&format!("{}/.asoundrc exists", root_label));
        } else {
            output::warn(&format!(
                "{}/.asoundrc missing — run 'aibox apply' to create it",
                root_label
            ));
            diag.warnings += 1;
        }
    }
}

/// Check home directory subdirectories.
fn check_root_subdirs(root: &Path, root_label: &str, diag: &mut DiagResult) {
    let expected_dirs = [
        ".ssh",
        ".vim",
        ".config/tmux",
        ".tmux/plugins",
        ".config/git",
    ];
    for dir in &expected_dirs {
        let path = root.join(dir);
        if path.exists() {
            output::ok(&format!("{}/{} exists", root_label, dir));
        } else {
            output::warn(&format!("{}/{} missing", root_label, dir));
            diag.warnings += 1;
        }
    }
}

/// Check .devcontainer/ files.
fn check_devcontainer_files(diag: &mut DiagResult) {
    let files = [
        crate::config::DOCKERFILE,
        crate::config::COMPOSE_FILE,
        crate::config::DEVCONTAINER_JSON,
    ];

    let mut all_present = true;
    for f in &files {
        if !Path::new(f).exists() {
            output::warn(&format!("{} missing -- run 'aibox apply'", f));
            diag.warnings += 1;
            all_present = false;
        }
    }

    if all_present {
        output::ok(".devcontainer/ files present");
    }
}

/// Check context structure against the process packages.
fn check_context_structure(packages: &[String], diag: &mut DiagResult) {
    let expected = expected_files(packages);

    for file in &expected {
        let path = Path::new(file);
        // For OWNER.md, also accept symlinks
        if path.exists() || path.symlink_metadata().is_ok() {
            output::ok(&format!("{} exists", file));
        } else {
            output::warn(&format!("{} missing", file));
            diag.warnings += 1;
        }
    }

    // Since v0.16.0 processkit owns almost all live content under context/.
    // aibox doctor only validates the aibox-owned perimeter; processkit health
    // is checked by pk-doctor/index-management so normal entity files do not
    // flood this report as "extra".
    if Path::new("context").exists() {
        output::ok("context/ exists (processkit content validated by pk-doctor)");
    } else {
        output::warn("context/ missing -- run 'aibox apply' to install processkit content");
        diag.warnings += 1;
    }
}

/// Check schema version and generate migration artifacts if needed.
fn check_schema_version(config: &AiboxConfig, diag: &mut DiagResult) -> Result<()> {
    let target_version = &config.context.schema_version;
    let current_version = CURRENT_CONTEXT_SCHEMA_VERSION;

    if current_version == target_version {
        output::ok(&format!(
            "Context schema: current {}, target {} (up to date)",
            current_version, target_version
        ));
    } else {
        output::warn(&format!(
            "Context schema: current {}, target {} (migration needed)",
            current_version, target_version
        ));
        diag.warnings += 1;
        generate_migration_artifacts(current_version, target_version, config)?;
    }

    Ok(())
}

/// Generate migration artifacts when schema versions differ.
fn generate_migration_artifacts(
    current_version: &str,
    target_version: &str,
    config: &AiboxConfig,
) -> Result<()> {
    let migration_dir = Path::new(".aibox/migration");
    std::fs::create_dir_all(migration_dir)?;

    // Write schema-current.md
    let current_schema = schema_for_version(current_version)
        .unwrap_or("# Unknown Schema Version\n\nNo embedded schema found for this version.\n");
    std::fs::write(migration_dir.join("schema-current.md"), current_schema)?;
    output::ok("Generated .aibox/migration/schema-current.md");

    // Write schema-target.md
    let target_schema = schema_for_version(target_version)
        .unwrap_or("# Unknown Schema Version\n\nNo embedded schema found for this version.\n");
    std::fs::write(migration_dir.join("schema-target.md"), target_schema)?;
    output::ok("Generated .aibox/migration/schema-target.md");

    // Write diff.md
    let diff_content = format!(
        "# Schema Diff: {} -> {}\n\n\
         ## Summary\n\n\
         Migration from schema version {} to {}.\n\n\
         ## Structural Differences\n\n\
         {}\n",
        current_version,
        target_version,
        current_version,
        target_version,
        if current_schema == target_schema {
            "No structural differences detected between these schema versions.".to_string()
        } else {
            "Schema content differs. Review schema-current.md and schema-target.md for details."
                .to_string()
        }
    );
    std::fs::write(migration_dir.join("diff.md"), diff_content)?;
    output::ok("Generated .aibox/migration/diff.md");

    // Write migration-prompt.md
    let prompt_content = format!(
        "# Migration Prompt\n\n\
         You are migrating the project context structure for **{}**.\n\n\
         ## Current State\n\n\
         - Schema version: {}\n\
         - Process packages: {:?}\n\
         - Container name: {}\n\n\
         ## Target State\n\n\
         - Schema version: {}\n\n\
         ## Instructions\n\n\
         1. Read `schema-current.md` to understand the current structure\n\
         2. Read `schema-target.md` to understand the target structure\n\
         3. Read `diff.md` for a summary of differences\n\
         4. Examine the project's `context/` directory\n\
         5. Generate a migration plan that:\n\
            - Adds any missing files or sections\n\
            - Never removes or overwrites existing user content\n\
            - Preserves all existing formatting and IDs\n\
            - Marks each change as \"required\" or \"recommended\"\n\n\
         ## Files to Reference\n\n\
         - `.aibox/migration/schema-current.md`\n\
         - `.aibox/migration/schema-target.md`\n\
         - `.aibox/migration/diff.md`\n\
         - `context/` directory (current project files)\n\
         - `CLAUDE.md` (project root)\n",
        config.container.name,
        current_version,
        config.context.packages,
        config.container.name,
        target_version,
    );
    std::fs::write(migration_dir.join("migration-prompt.md"), prompt_content)?;
    output::ok("Generated .aibox/migration/migration-prompt.md");

    output::info(&format!(
        "Migration artifacts written to {}",
        migration_dir.display()
    ));

    Ok(())
}

/// Check that the running container's image version matches the config version.
///
/// Reads the `aibox.version` Docker label set at build time. Skips silently
/// if the container is missing or has no label (pre-BACK-060 image).
fn check_container_image_version(runtime: &Runtime, config: &AiboxConfig, diag: &mut DiagResult) {
    let name = &config.container.name;
    let state = match runtime.container_status(name) {
        Ok(s) => s,
        Err(_) => return,
    };
    if state == ContainerState::Missing {
        return;
    }

    match runtime.get_container_image_version(name) {
        Ok(Some(container_ver)) => {
            let expected_version = expected_container_image_version(config);
            if expected_version.as_deref() == Some(container_ver.as_str()) {
                output::ok(&format!(
                    "Container image version: {} (matches resolved config)",
                    container_ver
                ));
            } else {
                output::warn(&format!(
                    "Container image version mismatch: container={} config={} — \
                     run `aibox apply` to rebuild",
                    container_ver,
                    expected_version.as_deref().unwrap_or(&config.aibox.version)
                ));
                diag.warnings += 1;
            }
        }
        Ok(None) => {
            // Pre-BACK-060 image: no label — informational only
            output::ok("Container image version: no label (pre-v0.13 image, rebuild recommended)");
        }
        Err(_) => {} // inspect failed — skip silently
    }
}

fn expected_container_image_version(config: &AiboxConfig) -> Option<String> {
    if config.aibox.version != "latest" {
        return Some(config.aibox.version.clone());
    }

    crate::lock::read_lock(Path::new("."))
        .ok()
        .flatten()
        .map(|lock| lock.aibox.cli_version)
        .filter(|version| !version.is_empty())
}

fn is_running_inside_aibox_container() -> bool {
    Path::new("/etc/aibox-version").is_file()
}

/// Warn if `aibox.lock [aibox].cli_version` is out of step with the
/// running CLI in a way the user should act on. v0.25.6 BR-DOCTOR-GAPS:
/// upgraded from string-equality to semver-aware comparison so a
/// patch-only delta produces an informational note (apply on next
/// change) while major/minor skew warns.
fn check_cli_version_file(diag: &mut DiagResult) {
    let lock = match crate::lock::read_lock(Path::new(".")) {
        Ok(Some(l)) => l,
        _ => return, // No lock or read error — already reported by check_schema_version.
    };

    let file_version = &lock.aibox.cli_version;
    if file_version.is_empty() {
        return;
    }

    let cli_version = env!("CARGO_PKG_VERSION");
    if file_version == cli_version {
        return;
    }

    let lock_v = semver::Version::parse(file_version).ok();
    let cli_v = semver::Version::parse(cli_version).ok();
    let major_minor_skew = matches!(
        (&lock_v, &cli_v),
        (Some(a), Some(b)) if a.major != b.major || a.minor != b.minor
    );

    if major_minor_skew {
        output::warn(&format!(
            "Host CLI is out of step with aibox.lock (lock={}, cli={}). \
             Run `aibox apply` to regenerate runtime files at the current version — \
             this is the most common cause of 'why am I still seeing the old bug'.",
            file_version, cli_version
        ));
        diag.warnings += 1;
    } else {
        output::ok(&format!(
            "CLI version skew within patch range (lock={}, cli={}); apply on next change.",
            file_version, cli_version
        ));
    }
}

/// Print final summary.
fn print_summary(diag: &DiagResult) {
    output::info(&format!(
        "Diagnostics complete: {} warning(s), {} error(s)",
        diag.warnings, diag.errors
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn codex_compose_posture_warns_on_privileged_and_sys_admin() {
        let compose: serde_yaml::Value = serde_yaml::from_str(
            r#"
services:
  demo:
    privileged: true
    cap_add:
      - SYS_ADMIN
"#,
        )
        .unwrap();

        let posture = codex_compose_posture(&compose, "demo", "compose.yml");

        assert_eq!(posture.broad_grant_warnings.len(), 2);
        assert!(
            posture
                .broad_grant_warnings
                .iter()
                .any(|warning| warning.contains("privileged=true"))
        );
        assert!(
            posture
                .broad_grant_warnings
                .iter()
                .any(|warning| warning.contains("SYS_ADMIN"))
        );
    }

    #[test]
    fn codex_compose_posture_accepts_seccomp_fallback_without_broad_grant_warning() {
        let compose: serde_yaml::Value = serde_yaml::from_str(
            r#"
services:
  aibox:
    init: true
    security_opt:
      - seccomp=unconfined
"#,
        )
        .unwrap();

        let posture = codex_compose_posture(&compose, "aibox", "compose.override.yml");

        assert!(posture.seccomp_unconfined);
        assert!(posture.init_true);
        assert!(posture.broad_grant_warnings.is_empty());
    }

    #[test]
    fn codex_compose_posture_detects_missing_init_reaper() {
        let compose: serde_yaml::Value = serde_yaml::from_str(
            r#"
services:
  aibox:
    command: sleep infinity
"#,
        )
        .unwrap();

        let posture = codex_compose_posture(&compose, "aibox", "compose.yml");

        assert!(!posture.init_true);
    }

    // S5 — BR-SEC-HARDEN: seccomp=unconfined without consent gate
    #[test]
    fn seccomp_unconfined_flagged_when_consent_missing() {
        // posture detects seccomp=unconfined; caller is responsible for
        // checking security.acknowledge_seccomp_unconfined before suppressing.
        let compose: serde_yaml::Value = serde_yaml::from_str(
            r#"
services:
  aibox:
    init: true
    security_opt:
      - seccomp=unconfined
"#,
        )
        .unwrap();

        let posture = codex_compose_posture(&compose, "aibox", "compose.yml");

        // The posture must detect seccomp_unconfined so the caller can apply
        // the consent gate (security.acknowledge_seccomp_unconfined).
        assert!(
            posture.seccomp_unconfined,
            "posture must detect seccomp=unconfined"
        );
        // seccomp=unconfined alone is NOT a broad-grant warning — it's the
        // narrow approved fallback; the consent gate is a separate layer.
        assert!(posture.broad_grant_warnings.is_empty());
    }

    #[test]
    fn seccomp_unconfined_not_flagged_without_security_opt() {
        let compose: serde_yaml::Value = serde_yaml::from_str(
            r#"
services:
  aibox:
    init: true
"#,
        )
        .unwrap();

        let posture = codex_compose_posture(&compose, "aibox", "compose.yml");

        assert!(!posture.seccomp_unconfined);
    }

    #[test]
    fn bwrap_smoke_probe_exercises_user_namespace_creation() {
        let args = bwrap_smoke_probe_args();
        assert!(args.contains(&"--unshare-user"));
        assert!(args.contains(&"--uid"));
        assert!(args.contains(&"--gid"));
        assert!(args.contains(&"/bin/true"));
    }

    #[test]
    fn codex_config_detects_host_absolute_processkit_script_paths() {
        let body = r#"
[mcp_servers.processkit-gateway]
command = "uv"
args = ["run", "/Users/example/project/context/skills/processkit/processkit-gateway/mcp/server.py", "stdio-proxy"]
"#;

        assert!(codex_config_has_non_container_processkit_script_path(body));
    }

    #[test]
    fn codex_config_accepts_container_workspace_processkit_script_paths() {
        let body = r#"
[mcp_servers.processkit-gateway]
command = "uv"
args = ["run", "/workspace/context/skills/processkit/processkit-gateway/mcp/server.py", "stdio-proxy"]
"#;

        assert!(!codex_config_has_non_container_processkit_script_path(body));
    }

    #[test]
    fn stale_processkit_skill_dirs_reports_hot_skills_missing_from_template() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(
            temp.path()
                .join("context/skills/processkit/status-briefing"),
        )
        .unwrap();
        fs::create_dir_all(
            temp.path()
                .join("context/skills/processkit/morning-briefing"),
        )
        .unwrap();
        fs::create_dir_all(temp.path().join("context/skills/product/retrospective")).unwrap();
        fs::write(
            temp.path()
                .join("context/skills/processkit/status-briefing/SKILL.md"),
            "# status\n",
        )
        .unwrap();
        fs::write(
            temp.path()
                .join("context/skills/processkit/morning-briefing/SKILL.md"),
            "# morning\n",
        )
        .unwrap();
        fs::write(
            temp.path()
                .join("context/skills/product/retrospective/SKILL.md"),
            "# retrospective\n",
        )
        .unwrap();

        let mut current_template_files = BTreeSet::new();
        current_template_files
            .insert("context/skills/processkit/status-briefing/SKILL.md".to_string());
        current_template_files
            .insert("context/skills/product/sprint-retrospective/SKILL.md".to_string());
        let mut known_template_files = current_template_files.clone();
        known_template_files
            .insert("context/skills/processkit/morning-briefing/SKILL.md".to_string());
        known_template_files.insert("context/skills/product/retrospective/SKILL.md".to_string());

        let stale = stale_processkit_skill_dirs(
            temp.path(),
            &current_template_files,
            &known_template_files,
        );
        let stale_rel: Vec<String> = stale
            .iter()
            .map(|path| {
                path.strip_prefix(temp.path())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();

        assert_eq!(
            stale_rel,
            vec![
                "context/skills/processkit/morning-briefing",
                "context/skills/product/retrospective"
            ]
        );
    }

    // BR-LEGACY-MUX-EXCISE (DEC-20260508_1515-SilentAsh): doctor must
    // raise an error when legacy multiplexer artifacts survive on disk
    // after `aibox apply`.
    fn legacy_mux_test_config(host_root: &std::path::Path) -> AiboxConfig {
        let cfg = AiboxConfig::from_str(
            r#"[aibox]
version = "0.25.6"

[container]
name = "doctor-legacy-mux"
"#,
        )
        .expect("test config parses");
        // Override host_root_dir() via the public override field used
        // by other tests. Use AIBOX_HOST_ROOT to keep this independent
        // of struct internals.
        unsafe {
            std::env::set_var("AIBOX_HOST_ROOT", host_root);
        }
        // Avoid leaking the env var: read once, then unset; cfg captures
        // the path on the next call to host_root_dir() in the tested fn.
        let _ = cfg.host_root_dir();
        cfg
    }

    #[test]
    #[serial_test::serial]
    fn doctor_errors_on_surviving_legacy_multiplexer_artifact() {
        let tmp = tempfile::tempdir().unwrap();
        let host = tmp.path().join("home");
        // Use the shared cleanup-target list as the source of truth so
        // doctor.rs stays free of literal legacy-path strings.
        let rel = crate::seed::LEGACY_MUX_RELPATHS[0];
        fs::create_dir_all(host.join(rel)).unwrap();
        fs::write(host.join(rel).join("config.kdl"), "// stale\n").unwrap();

        let cfg = legacy_mux_test_config(&host);
        let mut diag = DiagResult::new();
        check_legacy_multiplexer_artifacts(&cfg, &mut diag);

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
        assert!(
            diag.errors >= 1,
            "doctor must error when legacy multiplexer artifacts survive: {diag:?}"
        );
    }

    #[test]
    #[serial_test::serial]
    fn doctor_clean_when_legacy_multiplexer_purged() {
        let tmp = tempfile::tempdir().unwrap();
        let host = tmp.path().join("home");
        fs::create_dir_all(host.join(".config/tmux")).unwrap();

        let cfg = legacy_mux_test_config(&host);
        let mut diag = DiagResult::new();
        check_legacy_multiplexer_artifacts(&cfg, &mut diag);

        unsafe {
            std::env::remove_var("AIBOX_HOST_ROOT");
        }
        assert_eq!(
            diag.errors, 0,
            "clean host root should not raise multiplexer doctor errors: {diag:?}"
        );
    }

    #[test]
    #[serial_test::serial]
    fn context_structure_accepts_processkit_owned_context_files() {
        let temp = tempfile::tempdir().unwrap();
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();

        fs::create_dir_all("context/workitems").unwrap();
        fs::write("AGENTS.md", "# agents\n").unwrap();
        fs::write("aibox.lock", "").unwrap();
        fs::write(".gitignore", "").unwrap();
        fs::write("context/workitems/BACK-example.md", "# work\n").unwrap();

        let mut diag = DiagResult::new();
        check_context_structure(&["product".to_string()], &mut diag);

        std::env::set_current_dir(original).unwrap();
        assert_eq!(diag.warnings, 0);
    }
}
