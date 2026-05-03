use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::{AiHarness, AiboxConfig, McpGatewayMode};
use crate::output;
use crate::processkit_vocab::AGENTS_FILENAME;
use crate::runtime::{ContainerState, Runtime};

/// Embedded schema document for v1.0.0.
const SCHEMA_V1_0_0: &str = include_str!("../../schemas/v1.0.0/context-schema.md");

/// Diagnostic counters.
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

    // 3. Check .aibox-home/ directory (or legacy .root/)
    let root = config.host_root_dir();
    let root_label = root.display().to_string();
    if root.exists() {
        output::ok(&format!("{} directory exists", root_label));
        // Check expected subdirectories
        check_root_subdirs(&root, &root_label, &mut diag);

        // Check mount source paths match config (AI providers, audio)
        check_mount_sources(&root, &root_label, &config, &mut diag);

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

    // 6c. Check command file registrations (BACK-20260423_2050-EagerStone)
    output::info("Checking command file registrations...");
    check_command_registrations(&config, &mut diag);

    // 6d. Check processkit MCP gateway selection.
    output::info("Checking processkit MCP gateway...");
    check_processkit_mcp_gateway(&config, &mut diag);

    // 6e. Codex prompt-path drift check (BACK-20260426_1627-StrongHawk).
    // Loud failure if `pk-*` managed files reappear in the legacy
    // `~/.codex/prompts/` path that aibox v0.21.1 mistakenly used —
    // catches a regression in the codex profile of harness_commands.
    check_codex_prompt_path_drift(&config, &mut diag);

    // 6f. Codex sandbox prerequisites and compose posture.
    check_codex_sandbox_environment(&config, &mut diag);

    // 6g. Draft LivelyMoss addon metadata checks. Warning-only until
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

    print_summary(&diag);
    Ok(())
}

fn check_runtime_resource_pressure(config: &AiboxConfig, diag: &mut DiagResult) {
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

/// Check that installed skills have their command files registered in
/// each enabled harness's command directory.
///
/// For every `context/skills/*/commands/*.md` (source), validates that for
/// every enabled scaffolded harness the corresponding deployed file exists
/// under that harness's commands dir (Claude is always-on; others gated on
/// `[ai].harnesses`). Helps detect incomplete skill distributions and stale
/// scaffolds that were dropped before `aibox apply` was rerun.
fn check_command_registrations(config: &AiboxConfig, diag: &mut DiagResult) {
    let skills_dir = std::path::Path::new("context/skills");
    if !skills_dir.is_dir() {
        output::ok("No context/skills/ found (expected in new projects)");
        return;
    }

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
        ".claude/commands",
        ".claude/commands/{stem}.md",
        true, // always-on
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

    match gateway.mode {
        McpGatewayMode::Granular => {
            output::ok("processkit MCP gateway disabled; granular MCP servers selected");
            return;
        }
        McpGatewayMode::Auto if !gateway_available => {
            output::ok("processkit MCP gateway not installed; auto mode will use granular servers");
            return;
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

    output::ok("processkit MCP gateway is installed");

    if gateway.mode == McpGatewayMode::DaemonProxy {
        let devcontainer = Path::new(".devcontainer/devcontainer.json");
        match std::fs::read_to_string(devcontainer) {
            Ok(body) if body.contains("processkit-gateway/mcp/server.py") => {
                output::ok("processkit gateway daemon startup is present in devcontainer.json");
            }
            Ok(_) => {
                output::warn(
                    "[mcp.gateway].mode = \"daemon-proxy\" but devcontainer.json does not \
                     start processkit-gateway; run `aibox apply`",
                );
                diag.warnings += 1;
            }
            Err(_) => {
                output::warn(
                    "[mcp.gateway].mode = \"daemon-proxy\" but .devcontainer/devcontainer.json \
                     is missing; run `aibox apply`",
                );
                diag.warnings += 1;
            }
        }
    }

    if config.ai.harnesses.contains(&AiHarness::Codex) {
        match std::fs::read_to_string(".codex/config.toml") {
            Ok(body) if body.contains("[mcp_servers.processkit-gateway]") => {
                output::ok("Codex MCP config points at processkit-gateway");
            }
            Ok(_) => {
                output::warn(
                    "Codex is enabled but .codex/config.toml does not register \
                     processkit-gateway; run `aibox apply`",
                );
                diag.warnings += 1;
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
            Ok(true) => output::ok("codex: bubblewrap namespace smoke probe succeeded"),
            Ok(false) => {
                output::warn(
                    "codex: bubblewrap namespace smoke probe failed. Check host/runtime unprivileged user namespace and seccomp settings.",
                );
                diag.warnings += 1;
            }
            Err(err) => {
                output::warn(&format!(
                    "codex: bubblewrap namespace smoke probe could not run: {}",
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

fn run_bwrap_smoke_probe(binary: &str) -> Result<bool> {
    let status = Command::new(binary)
        .args([
            "--ro-bind",
            "/",
            "/",
            "--dev",
            "/dev",
            "--proc",
            "/proc",
            "/bin/true",
        ])
        .status()?;
    Ok(status.success())
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
    let expected_dirs = [".ssh", ".vim", ".config/zellij", ".config/git"];
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

    // Check for extra files in context/ that aren't expected (warning only)
    if Path::new("context").exists() {
        check_extra_files("context", &expected, diag);
    }
}

/// Walk the context/ directory and report files not in the expected list.
fn check_extra_files(dir: &str, expected: &[&str], diag: &mut DiagResult) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.to_string_lossy().to_string();

        if path.is_dir() {
            check_extra_files(&rel, expected, diag);
            continue;
        }

        // Normalize path separators and check against expected list
        let normalized = rel.replace('\\', "/");
        if !expected.iter().any(|e| normalized == *e) {
            // Don't warn about OWNER.md if it's a symlink (it's always expected via the list)
            output::warn(&format!(
                "Extra file: {} (not in {} schema)",
                normalized, "context"
            ));
            diag.warnings += 1;
        }
    }
}

/// Check schema version and generate migration artifacts if needed.
fn check_schema_version(config: &AiboxConfig, diag: &mut DiagResult) -> Result<()> {
    let target_version = &config.context.schema_version;

    let lock = match crate::lock::read_lock(Path::new("."))? {
        Some(l) => l,
        None => {
            output::warn("aibox.lock not found -- run 'aibox init' to create it");
            diag.warnings += 1;
            return Ok(());
        }
    };

    let current_version = &lock.aibox.cli_version;

    if current_version == target_version {
        output::ok(&format!(
            "Current: {}, Target: {} (up to date)",
            current_version, target_version
        ));
    } else {
        output::warn(&format!(
            "Current: {}, Target: {} (migration needed)",
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
            if container_ver == config.aibox.version {
                output::ok(&format!(
                    "Container image version: {} (matches config)",
                    container_ver
                ));
            } else {
                output::warn(&format!(
                    "Container image version mismatch: container={} config={} — \
                     run `aibox apply` to rebuild",
                    container_ver, config.aibox.version
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

/// Warn if `aibox.lock [aibox].cli_version` doesn't match the current CLI version.
///
/// `cli_version` is written at init/sync time. A mismatch means generated files
/// may be stale for this CLI version.
fn check_cli_version_file(diag: &mut DiagResult) {
    let lock = match crate::lock::read_lock(Path::new(".")) {
        Ok(Some(l)) => l,
        _ => return, // No lock or read error — already reported by check_schema_version.
    };

    let file_version = &lock.aibox.cli_version;
    if file_version.is_empty() {
        return; // Unknown version — skip.
    }

    let cli_version = env!("CARGO_PKG_VERSION");
    if file_version != cli_version {
        output::warn(&format!(
            "CLI version mismatch: aibox.lock cli_version={} current={} — \
             run `aibox apply` to update generated files",
            file_version, cli_version
        ));
        diag.warnings += 1;
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
}
